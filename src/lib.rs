mod args;
pub use args::Args;
pub mod recursion;
pub mod slow;
pub mod standard;
pub mod utils;

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

pub async fn run(args: &Args, file: File, output_path: Option<&Path>) {
    // Persist tmpdir at top scope
    let tmpdir = tempfile::tempdir().unwrap();
    let tmpdir_path = tmpdir.path();

    let file_size = file.metadata().await.unwrap().len();
    let memory_mode = utils::get_memory_mode(args, file_size);

    let output_path = match output_path {
        Some(s) => Path::new(s).to_owned(),
        None => std::env::current_dir().unwrap().join("res.txt"),
    };

    match read_start(memory_mode, file, tmpdir_path).await.unwrap() {
        ReadResult::SlowReadResult(r) => slow::sort(r, &output_path).await,
        ReadResult::StandardReadResult(r) => standard::sort(r, &output_path).await,
    };
}
