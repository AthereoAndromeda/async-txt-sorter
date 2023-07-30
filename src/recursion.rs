use tokio::fs::OpenOptions;

use crate::args::Args;
use std::{
    io,
    path::Path,
    sync::{atomic::AtomicU64, Arc},
};

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

    if tokio::fs::read_dir(base_path).await.is_err() {
        log::info!("Creating `res` dir");
        tokio::fs::create_dir(base_path).await?;
    }

    let mut handles = Vec::new();

    let file_count = files.len();
    let files_finished = Arc::new(AtomicU64::new(0));

    for f in files {
        // Clone to avoid async issues
        let base_path = base_path.clone();
        let args = args.clone();
        let files_finished = Arc::clone(&files_finished);

        let path = base_path.join(f.file_name());

        let h = tokio::spawn(async move {
            let file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(&path)
                .await
                .unwrap();

            super::run(&args, file, Some(&path)).await;

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
        h.await.unwrap();
    }

    Ok(())
}
