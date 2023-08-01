use rayon::slice::ParallelSliceMut;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

/// Read file async and return lines. Stores entire file content in-memory.
/// Use with care with large files
pub async fn read<R: AsyncBufRead + Unpin>(reader: R) -> io::Result<Vec<String>> {
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

pub async fn sort(mut content: Vec<String>, mut writer: impl AsyncWrite + Unpin) {
    log::info!("Sorting...");
    content.par_sort_unstable();

    let output_content = content.join("\n");
    let output_content = output_content.as_bytes();

    writer.write_all(output_content).await.unwrap();
    writer.flush().await.unwrap();

    log::info!("Finished!");
}
