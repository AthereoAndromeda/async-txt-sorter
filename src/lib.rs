use std::{collections::HashMap, io::SeekFrom, path::Path};

use rayon::slice::ParallelSliceMut;
use tokio::{
    fs::{File, OpenOptions},
    io::{
        self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader,
        BufWriter,
    },
};

#[derive(Debug)]
pub enum ReadResult {
    StandardReadResult(Vec<String>),
    SlowReadResult(Vec<BufReader<File>>),
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryMode {
    Standard,
    Low,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingStrategy {
    Alphabetical,
}

pub async fn read_start(
    mode: MemoryMode,
    file: File,
    tmpdir_path: &Path,
) -> io::Result<ReadResult> {
    match mode {
        MemoryMode::Low => {
            log::info!("Memory Mode: Low");

            let reader = BufReader::new(file);
            let content = read_slow(reader, tmpdir_path).await?;

            Ok(ReadResult::SlowReadResult(content))
        }
        MemoryMode::Standard => {
            log::info!("Memory Mode: Standard");
            let reader = BufReader::new(file);
            let content = read_standard(reader).await?;

            Ok(ReadResult::StandardReadResult(content))
        }
    }
}

/// Read file async and return lines. Stores entire file content in-memory.
/// Use with care with large files
pub async fn read_standard<R: AsyncBufRead + Unpin>(reader: R) -> io::Result<Vec<String>> {
    log::info!("Reading from reader...");
    let mut lines = reader.lines();
    let mut content = Vec::new();

    #[cfg(debug_assertions)]
    let mut line_count = 0;

    log::debug!("Iterating Lines");
    while let Some(line) = lines.next_line().await? {
        content.push(line);

        #[cfg(debug_assertions)]
        {
            line_count += 1;
        }
    }
    log::debug!("Finished iterating lines");

    #[cfg(debug_assertions)]
    log::debug!("Lines Counted: {}", line_count);

    Ok(content)
}

/// Reads file but much more slowly, but with lower memory consumption since
/// the file contents are stored in tmpfiles within storage.
pub async fn read_slow<R: AsyncBufRead + Unpin>(
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

pub async fn slow_sort(read_result: Vec<BufReader<File>>, output_path: &Path) {
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

pub async fn standard_sort(mut content: Vec<String>, output_path: &Path) {
    log::info!("Sorting...");
    content.par_sort_unstable();

    log::info!("Writing Output...");
    tokio::fs::write(&output_path, content.join("\n").as_bytes())
        .await
        .unwrap();

    log::info!("Finished!");
}
