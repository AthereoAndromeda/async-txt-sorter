use rayon::slice::ParallelSliceMut;
use std::{
    collections::HashMap,
    io::SeekFrom,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{
        self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader,
        BufWriter,
    },
};

use crate::SortError;

/// `tokio::io::BufReader` but with a path attached
#[derive(Debug)]
pub struct NamedReader<T> {
    path: PathBuf,
    reader: BufReader<T>,
}

/// Reads file but much more slowly, but with lower memory consumption since
/// the file contents are stored in tmpfiles within storage.
pub async fn read<R>(reader: R, tmpdir_path: &Path) -> io::Result<Vec<NamedReader<File>>>
where
    R: AsyncBufRead + Unpin,
{
    log::info!("Performing slow read...");
    let mut lines = reader.lines();

    // Avoid duplicating tmpfiles. At top scope to keep the files persisting.
    let mut map = HashMap::new();

    log::debug!("Iterating Lines");
    while let Some(line) = lines.next_line().await? {
        let first_char = line.chars().next();

        let Some(first_char) = first_char else { continue };

        // Fallback for invalid file names
        let first_char = if first_char.is_alphanumeric() {
            first_char
        } else {
            '-'
        };

        let path = tmpdir_path.join(&format!("{first_char}.txt"));

        if !map.contains_key(&path) {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
                .await
                .unwrap();

            let writer = BufWriter::new(file);
            map.insert(path.clone(), writer);
        }

        let writer = map.get_mut(&path).unwrap();
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    let mut files = Vec::new();

    // Flush values
    for (path, mut writer) in map {
        writer.flush().await?;
        let mut f = writer.into_inner();

        // Reset Cursor
        f.seek(SeekFrom::Start(0)).await?;

        files.push(NamedReader {
            path,
            reader: BufReader::new(f),
        });
    }

    Ok(files)
}

pub async fn sort(
    mut read_result: Vec<NamedReader<File>>,
    output_path: &Path,
) -> Result<(), SortError> {
    log::info!("Sorting and Writing...");
    let mut output_writer = BufWriter::new(File::create(output_path).await?);

    // sort alphabetically
    read_result.par_sort_by(|a, b| a.path.cmp(&b.path));

    for named_reader in read_result {
        let mut buf = String::new();

        let mut reader = named_reader.reader;
        reader.read_to_string(&mut buf).await?;

        let mut s_vec = buf.split('\n').collect::<Vec<_>>();
        s_vec.par_sort_unstable();

        let res = s_vec.join("\n");

        output_writer.write_all(res.as_bytes()).await?;
    }

    log::info!("Finished!");
    output_writer.flush().await?;

    Ok(())
}
