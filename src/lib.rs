pub mod slow;
pub mod standard;

use slow::NamedReader;
use std::path::Path;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

#[derive(Debug)]
pub enum ReadResult {
    StandardReadResult(Vec<String>),
    SlowReadResult(Vec<NamedReader<File>>),
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
            let content = slow::read(reader, tmpdir_path).await?;

            Ok(ReadResult::SlowReadResult(content))
        }
        MemoryMode::Standard => {
            log::info!("Memory Mode: Standard");
            let reader = BufReader::new(file);
            let content = standard::read(reader).await?;

            Ok(ReadResult::StandardReadResult(content))
        }
    }
}
