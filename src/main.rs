mod args;

use args::Args;
use async_txt_sorter::{read_start, slow, standard, MemoryMode, ReadResult};
use clap::Parser;
use simple_logger::SimpleLogger;
use std::{
    path::Path,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{self},
};

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

pub async fn recurse(input_path: &Path, args: &Args) -> io::Result<()> {
    log::info!("Entering Recursive Mode...");

    // Read input dir
    let input_path = input_path.canonicalize().unwrap();
    let mut dir = tokio::fs::read_dir(&input_path).await?;

    let mut files = Vec::new();

    while let Some(file) = dir.next_entry().await? {
        log::info!("{}", file.file_name().to_str().unwrap());

        // Only push files
        // TODO: Add actual recursion
        if file.file_type().await.unwrap().is_file() {
            files.push(file);
        }
    }

    // Check if output dir exists, create if not
    let base_path = &input_path.join("..").join("res");
    match tokio::fs::read_dir(base_path).await {
        Ok(_) => {}

        Err(_) => {
            log::info!("Creating `res` dir");
            tokio::fs::create_dir(base_path).await?;
        }
    };

    let mut handles = Vec::new();

    let file_count = files.len();
    let files_finished = Arc::new(AtomicU64::new(0));

    for f in files {
        // Clone to avoid async issues
        let input_path = input_path.clone();
        let base_path = base_path.clone();

        let path = input_path.join(f.file_name());
        let memory_mode = get_memory_mode(args, f.metadata().await.unwrap().len());

        let files_finished = Arc::clone(&files_finished);

        let h = tokio::spawn(async move {
            let file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(&path)
                .await
                .unwrap();

            let res = read_start(memory_mode, file, &base_path).await.unwrap();
            let output_path = &base_path.join(path.file_name().unwrap());

            match res {
                ReadResult::StandardReadResult(r) => standard::sort(r, output_path).await,
                ReadResult::SlowReadResult(r) => slow::sort(r, output_path).await,
            }

            // Keep count of files
            let ff = files_finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            log::info!("Files Sorted: {}/{}", ff + 1, file_count);
        });

        handles.push(h);
    }

    for h in handles {
        h.await.unwrap();
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
        recurse(input_path, &args).await.unwrap();
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
