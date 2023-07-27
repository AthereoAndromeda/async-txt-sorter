use clap::Parser;
use large_txt_file_sorter::read_file;
use simple_logger::SimpleLogger;
use std::path::Path;

/// Sorts massive files alphabetically
#[derive(Parser, Debug)]
struct Args {
    /// Path to file
    path: String,

    /// Output path. Defaults to res.txt
    #[arg(short, long)]
    output: Option<String>,
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
    let mut content = read_file(input_path).await.unwrap();

    log::info!("Sorting...");
    content.sort_unstable();

    log::info!("Writing Output...");
    tokio::fs::write(output_path, content.join("\n").as_bytes())
        .await
        .unwrap();

    log::info!("Finished!");
}

// #[cfg(test)]
// mod test {
//     use large_txt_file_sorter::{sort_files, write_tmp_files};
//     use std::io::{BufReader, BufWriter};

//     const EXPECTED_ANSWER: &[u8; 190] = include_bytes!("../test/correct.txt");
//     const TEST_FILE: &[u8; 189] = include_bytes!("../test/text.txt");

//     #[test]
//     fn integration_test() {
//         let tmp_dir = tempfile::tempdir().unwrap();
//         let tmp_path = tmp_dir.path();

//         let mut reader = BufReader::new(&TEST_FILE[..]);
//         let files = write_tmp_files(&mut reader, tmp_path);

//         let mut writer = BufWriter::new(Vec::with_capacity(190));
//         sort_files(files, &mut writer);

//         let res = writer.into_inner().unwrap();
//         assert_eq!(&res[..], EXPECTED_ANSWER);
//     }
// }
