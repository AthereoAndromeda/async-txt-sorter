use tokio::io::{self, AsyncBufRead, AsyncBufReadExt};

/// Read file async and return lines. Stores entire file content in-memory.
/// Use with care with large files
pub async fn read_and_get_lines<R: AsyncBufRead + Unpin>(reader: R) -> io::Result<Vec<String>> {
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
pub async fn read_slow<R: AsyncBufRead + Unpin>(reader: R) {
    log::info!("Performing slow read...");
}
