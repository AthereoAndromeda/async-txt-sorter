mod args;
pub use args::Args;
pub mod recursion;
pub mod slow;
pub mod standard;
pub mod utils;

use slow::NamedReader;
use std::path::Path;
use thiserror::Error;
use tokio::{
    fs::File,
    io::{self, BufReader, BufWriter},
};

#[derive(Debug, Error)]
/// Represents possible errors when executing sorts
pub enum SortError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
}

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

#[derive(Debug, thiserror::Error)]
/// Errors when running
pub enum RunError {
    #[error("Tokio IoError: {0}")]
    IoError(#[from] tokio::io::Error),

    #[error("Error during sorting: {0}")]
    SortError(#[from] SortError),
}

pub async fn run(args: &Args, file: File, output_path: Option<&Path>) -> Result<(), RunError> {
    // Persist tmpdir at top scope
    let tmpdir = tempfile::tempdir().unwrap();
    let tmpdir_path = tmpdir.path();

    let file_size = file.metadata().await?.len();
    let memory_mode = utils::get_memory_mode(args, file_size);

    let output_path = match output_path {
        Some(s) => Path::new(s).to_owned(),
        None => std::env::current_dir().unwrap().join("res.txt"),
    };

    match read_start(memory_mode, file, tmpdir_path).await? {
        ReadResult::SlowReadResult(r) => slow::sort(r, &output_path).await?,
        ReadResult::StandardReadResult(r) => {
            let file = File::create(&output_path).await?;
            let mut writer = BufWriter::new(file);

            log::info!("Writing Output...");
            standard::sort(r, &mut writer).await?;
            log::info!("Finished!");
        }
    };

    Ok(())
}
