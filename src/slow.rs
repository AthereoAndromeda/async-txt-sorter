use rayon::slice::ParallelSliceMut;
use std::{collections::HashMap, io::SeekFrom, path::Path};
use tokio::{
    fs::{File, OpenOptions},
    io::{
        self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader,
        BufWriter,
    },
};

/// Reads file but much more slowly, but with lower memory consumption since
/// the file contents are stored in tmpfiles within storage.
pub async fn read<R: AsyncBufRead + Unpin>(
    reader: R,
    tmpdir_path: &Path,
) -> io::Result<Vec<BufReader<File>>> {
    log::info!("Performing slow read...");

    let mut lines = reader.lines();

    // Avoid duplicating tmpfiles. At top scope to keep the files persisting.
    let mut map = HashMap::new();

    log::debug!("Iterating Lines");
    while let Some(line) = lines.next_line().await? {
        let first_char = line.chars().next();

        let first_char = match first_char {
            Some(c) => c,
            None => continue,
        };

        // Fallback for invalid file names
        let first_char = if first_char.is_alphanumeric() {
            first_char
        } else {
            '-'
        };

        let path = tmpdir_path.join(&format!("{}.txt", first_char));

        if !map.contains_key(&first_char) {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)
                .await
                .unwrap();

            let writer = BufWriter::new(file);
            map.insert(first_char.clone(), writer);
        }

        let writer = map.get_mut(&first_char).unwrap();
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    let mut files = Vec::new();

    // Flush values
    for (_, mut writer) in map {
        writer.flush().await?;
        let mut f = writer.into_inner();

        // Reset Cursor
        f.seek(SeekFrom::Start(0)).await?;
        files.push(BufReader::new(f));
    }

    Ok(files)
}

pub async fn sort(read_result: Vec<BufReader<File>>, output_path: &Path) {
    log::info!("Sorting and Writing...");
    let mut output_writer = BufWriter::new(File::create(output_path).await.unwrap());

    for mut reader in read_result {
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await.unwrap();

        let mut s_vec = buf.split("\n").collect::<Vec<_>>();
        s_vec.par_sort_unstable();

        let res = s_vec.join("\n");

        output_writer.write_all(res.as_bytes()).await.unwrap()
    }

    log::info!("Finished!");
    output_writer.flush().await.unwrap();
}
