use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

use clap::Parser;
use large_txt_file_sorter::{sort_files, write_tmp_files};

/// Sorts massive files alphabetically
#[derive(Parser, Debug)]
struct Args {
    /// Path to file
    path: String,

    /// Output path. Defaults to res.txt
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let output_path = match args.output {
        Some(s) => Path::new(&s).to_owned(),
        None => std::env::current_dir().unwrap().join("res.txt"),
    };

    // Open and persist tmpdir at top scope
    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_path = tmp_dir.path();

    let file = File::open(&args.path).unwrap();
    let mut reader = BufReader::new(file);

    println!("Reading file...");
    let files = write_tmp_files(&mut reader, tmp_path);

    let output_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
        .unwrap();

    let mut output_writer = BufWriter::new(output_file);

    println!("Sorting Files...");
    sort_files(files, &mut output_writer);

    println!("Finished!");
}

#[cfg(test)]
mod test {
    use large_txt_file_sorter::{sort_files, write_tmp_files};
    use std::io::{BufReader, BufWriter};

    const EXPECTED_ANSWER: &[u8; 190] = include_bytes!("../test/correct.txt");
    const TEST_FILE: &[u8; 189] = include_bytes!("../test/text.txt");

    #[test]
    fn integration_test() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let mut reader = BufReader::new(&TEST_FILE[..]);
        let files = write_tmp_files(&mut reader, tmp_path);

        let mut writer = BufWriter::new(Vec::with_capacity(190));
        sort_files(files, &mut writer);

        let res = writer.into_inner().unwrap();
        assert_eq!(&res[..], EXPECTED_ANSWER);
    }
}
