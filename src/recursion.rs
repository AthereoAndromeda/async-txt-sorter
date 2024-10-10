use thiserror::Error;
use tokio::{fs::OpenOptions, io};

use crate::cli::Args;
use std::{
    path::Path,
    sync::{atomic::AtomicU64, Arc},
};

#[derive(Debug, Error)]
/// Possible Errors when recursing through a directory
pub enum RecurseError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("File Name is not UTF-8 Valid")]
    NonUtf8FileName,

    #[error("Tokio Task Error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

pub async fn recurse(input_path: &Path, args: &Args) -> Result<(), RecurseError> {
    log::info!("Entering Recursive Mode...");

    // Read input dir
    let input_path = input_path.canonicalize().unwrap();
    let mut dir = tokio::fs::read_dir(&input_path).await?;

    let mut files = Vec::new();

    while let Some(file) = dir.next_entry().await? {
        log::info!(
            "File detected: {}",
            file.file_name()
                .to_str()
                .ok_or(RecurseError::NonUtf8FileName)?
        );

        // Only push files
        // TODO: Add actual recursion
        if file.file_type().await?.is_file() {
            files.push(file);
        }
    }

    // Check if output dir exists, create if not
    let output_path = match &args.output {
        Some(p) => Path::new(p).to_owned(),
        None => input_path.join("..").join("res"),
    };

    let output_path = &output_path;

    if tokio::fs::read_dir(output_path).await.is_err() {
        log::info!("Creating {} dir", output_path.display());
        tokio::fs::create_dir(output_path).await?;
    }

    let mut handles = Vec::new();

    let file_count = files.len();
    let files_finished = Arc::new(AtomicU64::new(0));

    for f in files {
        // Clone to avoid async issues
        let base_path = output_path.clone();
        let args = args.clone();
        let files_finished = Arc::clone(&files_finished);

        let input_files_path = input_path.join(f.file_name());
        let output_files_path = base_path.join(f.file_name());
        println!("{}", input_files_path.display());

        let h = tokio::spawn(async move {
            let file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(&input_files_path)
                .await
                .unwrap();

            super::run(&args, file, Some(&output_files_path))
                .await
                .unwrap();

            // Keep count of files
            let ff = files_finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            log::info!(
                "Files Sorted: {}/{}",
                ff + 1, /* add 1 since ff is the old value */
                file_count
            );
        });

        handles.push(h);
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}
