use clap::Parser;
use large_txt_file_sorter::read_and_get_lines;
use rayon::prelude::*;
use simple_logger::SimpleLogger;
use std::path::Path;
use tokio::{fs::File, io::BufReader};

/// Sorts massive files alphabetically
#[derive(Parser, Debug)]
struct Args {
    /// Path to file
    path: String,

    /// Output path. Defaults to res.txt
    #[arg(short, long)]
    output: Option<String>,

    /// Determines on which character to split the file to. Defaults to newline
    #[arg(short, long, default_value_t = String::from("\n"))]
    delimiter: String,

    /// Determines how the output should be joined together. Defaults to newline
    #[arg(short, long, default_value_t = String::from("\n"))]
    output_delimiter: String,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();
    let args = Args::parse();

    let output_path = match args.output {
        Some(s) => Path::new(&s).to_owned(),
        None => std::env::current_dir().unwrap().join("res.txt"),
    };

    let input_path = Path::new(&args.path);
    let file = File::open(input_path).await.unwrap();
    let reader = BufReader::new(file);

    let mut content = read_and_get_lines(reader).await.unwrap();

    log::info!("Sorting...");
    content.par_sort_unstable();

    log::info!("Writing Output...");
    tokio::fs::write(output_path, content.join("\n").as_bytes())
        .await
        .unwrap();

    log::info!("Finished!");
}

#[cfg(test)]
mod test {
    use rayon::slice::ParallelSliceMut;
    use tokio::io::BufReader;

    use large_txt_file_sorter::read_and_get_lines;

    const EXPECTED_ANSWER: &[u8; 189] = include_bytes!("../test/correct.txt");
    const TEST_FILE: &[u8; 189] = include_bytes!("../test/text.txt");

    #[tokio::test]
    async fn integration_test() {
        let reader = BufReader::new(&TEST_FILE[..]);
        let mut files = read_and_get_lines(reader).await.unwrap();

        files.par_sort_unstable();

        let res = files.join("\n");
        assert_eq!(&res.as_bytes(), EXPECTED_ANSWER);
    }
}
