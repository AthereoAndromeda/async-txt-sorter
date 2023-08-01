use rayon::slice::ParallelSliceMut;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use crate::SortError;

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

pub async fn sort(
    mut content: Vec<String>,
    mut writer: impl AsyncWrite + Unpin,
) -> Result<(), SortError> {
    log::info!("Sorting...");
    content.par_sort_unstable();

    let output_content = content.join("\n");
    let output_content = output_content.as_bytes();

    writer.write_all(output_content).await?;
    writer.flush().await?;

    log::info!("Finished!");
    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{read, sort};

    #[tokio::test]
    async fn read_test() {
        let expected = vec!["b".to_string(), "a".to_string(), "c".to_string()];
        let reader = Cursor::new("b\na\nc");

        let a = read(reader).await.unwrap();

        assert_eq!(a, expected);
    }

    #[tokio::test]
    async fn sort_test() {
        let input = vec!["b".to_string(), "a".to_string(), "c".to_string()];

        let mut writer = Cursor::new(Vec::with_capacity(std::mem::size_of::<String>() * 3 * 2));
        sort(input, &mut writer).await.unwrap();

        let inner = writer.into_inner();

        let expected = "a\nb\nc".to_string();
        let processed = String::from_utf8_lossy(&inner).to_string();
        assert_eq!(processed, expected);
    }
}
