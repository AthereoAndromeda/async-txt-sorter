use async_txt_sorter::{recursion::recurse, Args};
use clap::Parser;
use simple_logger::SimpleLogger;
use std::path::Path;
use tokio::fs::File;

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();
    let args = Args::parse();

    let input_path = Path::new(&args.path);
    let file = File::open(input_path).await.unwrap();
    let file_metadata = file.metadata().await.unwrap();

    // Do recursive sorting
    if file_metadata.is_dir() {
        recurse(input_path, &args).await.unwrap();
        return;
    }

    async_txt_sorter::run(&args, file, None).await;
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
