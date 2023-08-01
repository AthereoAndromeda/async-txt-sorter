use rayon::slice::ParallelSliceMut;
use std::{
    collections::HashMap,
    io::SeekFrom,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{
        self, AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite,
        AsyncWriteExt, BufReader, BufWriter,
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

pub async fn sort<T>(
    mut read_result: Vec<NamedReader<T>>,
    mut writer: impl AsyncWrite + Unpin,
) -> Result<(), SortError>
where
    T: Send + AsyncRead + Unpin,
{
    log::info!("Sorting and Writing...");

    // sort alphabetically
    read_result.par_sort_by(|a, b| a.path.cmp(&b.path));

    for named_reader in read_result {
        let mut buf = String::new();

        let mut reader = named_reader.reader;
        reader.read_to_string(&mut buf).await?;

        let mut s_vec = buf.split('\n').collect::<Vec<_>>();
        s_vec.par_sort_unstable();

        let res = s_vec.join("\n");

        writer.write_all(res.as_bytes()).await?;
    }

    log::info!("Finished!");
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::{read, sort, NamedReader};
    use tokio::io::{BufReader, BufWriter};

    // #[tokio::test]
    // async fn read_test() {
    //     let s = "a\naa\nb\nc".to_string();
    //     let reader = BufReader::new(s.as_bytes());

    //     let tmpdir = tempfile::tempdir().unwrap();
    //     let tmpdir_path = tmpdir.path();
    //     let g = read(reader, tmpdir_path).await.unwrap();

    //     let expected = vec![
    //         NamedReader {
    //             path
    //         }
    //     ]
    // }

    // #[tokio::test]
    // async fn sort_test() {
    //     let a = vec![
    //         NamedReader {
    //             path: Path::new("a.txt").to_path_buf(),
    //             reader: BufReader::new("a\naa".as_bytes()),
    //         },
    //         NamedReader {
    //             path: Path::new("b.txt").to_path_buf(),
    //             reader: BufReader::new("b".as_bytes()),
    //         },
    //     ];

    //     let mut writer = BufWriter::new(Vec::new());
    //     sort(a, &mut writer).await.unwrap();
    //     let inner = writer.into_inner();

    //     let processed = String::from_utf8_lossy(&inner).to_string();

    //     let expected = "a\naa\nb";
    //     assert_eq!(expected.as_bytes(), processed.as_bytes());
    // }
}
