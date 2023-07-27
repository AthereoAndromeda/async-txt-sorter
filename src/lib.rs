use std::path::Path;
use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, BufReader},
};

/// Read file async and return lines
pub async fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    log::info!("Reading File...");
    let file = File::open(path).await?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut words = Vec::new();

    #[cfg(debug_assertions)]
    let mut line_count = 0;

    log::debug!("Iterating Lines");
    while let Some(line) = lines.next_line().await? {
        words.push(line);

        #[cfg(debug_assertions)]
        {
            line_count += 1;
        }
    }
    log::debug!("Finished iterating lines");

    #[cfg(debug_assertions)]
    log::debug!("Lines Counted: {}", line_count);

    Ok(words)
}
