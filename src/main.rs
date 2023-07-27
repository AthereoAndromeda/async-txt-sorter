mod args;

use args::Args;
use async_txt_sorter::{read_start, slow, standard, MemoryMode, ReadResult};
use clap::Parser;
use simple_logger::SimpleLogger;
use std::path::Path;
use tokio::{fs::File, io};

fn get_memory_mode(args: &Args, file_size: u64) -> MemoryMode {
    const THRESHOLD: u64 = 1000 * 1000 * 500; // 500MB

    // Disallow simulataneously using disable_lmm and lmm flags
    if args.low_memory_mode && args.disable_low_memory_mode {
        log::error!("You cannot have both --low-memory-mode and --disable-low-memory-mode flag active at the same time!");
        panic!("Incompatible flags active together.");
    }

    let mut is_low_memory_mode_enabled = args.low_memory_mode && !args.disable_low_memory_mode;

    // Enable for files 500MB+
    if file_size > THRESHOLD && !args.disable_low_memory_mode {
        is_low_memory_mode_enabled = true;
    }

    if is_low_memory_mode_enabled {
        MemoryMode::Low
    } else {
        MemoryMode::Standard
    }
}

pub async fn recurse(input_path: &Path) -> io::Result<()> {
    log::info!("Entering Recursive Mode...");
    let mut dir = tokio::fs::read_dir(input_path).await?;

    while let Some(file) = dir.next_entry().await? {
        log::info!("{}", file.file_name().to_str().unwrap());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();
    let args = Args::parse();

    let output_path = match &args.output {
        Some(s) => Path::new(s).to_owned(),
        None => std::env::current_dir().unwrap().join("res.txt"),
    };

    let input_path = Path::new(&args.path);
    let file = File::open(input_path).await.unwrap();
    let file_metadata = file.metadata().await.unwrap();
    let file_size = file_metadata.len();

    // Do recursive sorting
    if file_metadata.is_dir() {
        recurse(input_path).await.unwrap();
        return;
    }

    let memory_mode = get_memory_mode(&args, file_size);

    // Persist tmpdir at top scope
    let tmpdir = tempfile::tempdir().unwrap();
    let tmpdir_path = tmpdir.path();

    match read_start(memory_mode, file, tmpdir_path).await.unwrap() {
        ReadResult::SlowReadResult(r) => slow::sort(r, &output_path).await,
        ReadResult::StandardReadResult(r) => standard::sort(r, &output_path).await,
    };
}

#[cfg(test)]
mod test {
    use async_txt_sorter::standard;
    use rayon::slice::ParallelSliceMut;
    use tokio::io::BufReader;

    const EXPECTED_ANSWER: &[u8; 189] = include_bytes!("../test/correct.txt");
    const TEST_FILE: &[u8; 189] = include_bytes!("../test/text.txt");

    #[tokio::test]
    async fn integration_test() {
        let reader = BufReader::new(&TEST_FILE[..]);
        let mut files = standard::read(reader).await.unwrap();

        files.par_sort_unstable();

        let res = files.join("\n");
        assert_eq!(&res.as_bytes(), EXPECTED_ANSWER);
    }
}
