mod args;
mod recursion;
mod utils;

use args::Args;
use async_txt_sorter::{read_start, slow, standard, ReadResult};
use clap::Parser;
use recursion::recurse;
use simple_logger::SimpleLogger;
use std::path::Path;
use tokio::fs::File;

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

    let memory_mode = utils::get_memory_mode(&args, file_size);

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
