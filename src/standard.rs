use std::path::Path;

use rayon::slice::ParallelSliceMut;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt};

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

pub async fn sort(mut content: Vec<String>, output_path: &Path) {
    log::info!("Sorting...");
    content.par_sort_unstable();

    log::info!("Writing Output...");
    tokio::fs::write(&output_path, content.join("\n").as_bytes())
        .await
        .unwrap();

    log::info!("Finished!");
}
