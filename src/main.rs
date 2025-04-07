use tokio::fs;
use tokio::signal;
use tokio::sync::mpsc;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicI64, Ordering}};
use std::time::Instant;
use regex::Regex;
use anyhow::Error;
use std::pin::Pin;
use std::future::Future;
use std::os::unix::fs::{PermissionsExt, FileTypeExt};

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <directory> <filename> [--regex]", args[0]);
        return;
    }

    let directory = args[1].clone();
    let filename = args[2].clone();
    let use_regex = args.len() > 3 && args[3] == "--regex";
    let regex = if use_regex {
        Some(Arc::new(Regex::new(&filename).expect("Invalid regex pattern")))
    } else {
        None
    };

    let (tx, mut rx) = mpsc::channel(1000);
    let (err_tx, mut err_rx) = mpsc::channel(1000);

    let dir_path = PathBuf::from(directory);

    let active = Arc::new(AtomicI64::new(0));
    let deferred = Arc::new(AtomicI64::new(0));

    let tx_clone = tx.clone();
    let err_tx_clone = err_tx.clone();
    let active_clone = active.clone();
    let deferred_clone = deferred.clone();
    active_clone.fetch_add(1, Ordering::SeqCst);
    tokio::spawn(async move {
        search_files(tx_clone, err_tx_clone, regex, filename, dir_path, active_clone, deferred_clone).await;
    });

    let ctrl_c = signal::ctrl_c();

    tokio::spawn(async move {
        let mut i = 1;
        while let Some(f) = rx.recv().await {
            println!("\x1b[32m#{}: {}\x1b[0m", i, f.display());
            i += 1;
        }
    });

    tokio::spawn(async move {
        while let Some(e) = err_rx.recv().await {
            eprintln!("\x1b[31merror: {}\x1b[0m", e);
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let a = active.load(Ordering::SeqCst);
            let d = deferred.load(Ordering::SeqCst);
            println!("active: {}, deferred: {}, diff: {}", a, d, a - d);
        }
    });

    ctrl_c.await.unwrap();
    println!("Elapsed: {:?}", start.elapsed());
}

fn search_files(
    tx: mpsc::Sender<PathBuf>,
    err_tx: mpsc::Sender<Error>,
    regex: Option<Arc<Regex>>,
    file_name: String,
    dir_path: PathBuf,
    active: Arc<AtomicI64>,
    deferred: Arc<AtomicI64>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        let mut entries = match fs::read_dir(&dir_path).await {
            Ok(entries) => entries,
            Err(e) => {
                let _ = err_tx.send(Error::new(e)).await;
                return;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let file_type = match entry.file_type().await {
                Ok(file_type) => file_type,
                Err(e) => {
                    let _ = err_tx.send(Error::new(e)).await;
                    continue;
                }
            };

            if is_symlink_or_irregular(&file_type).await {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(metadata) => metadata,
                Err(e) => {
                    let _ = err_tx.send(Error::new(e)).await;
                    continue;
                }
            };

            if metadata.len() == 0 || !is_readable(&path).await {
                continue;
            }

            if file_type.is_file() {
                let file_name_match = match &regex {
                    Some(regex) => regex.is_match(&path.file_name().unwrap().to_string_lossy()),
                    None => path.file_name().unwrap().to_string_lossy() == file_name,
                };

                if file_name_match {
                    let _ = tx.send(path.clone()).await;
                }
            } else if file_type.is_dir() {
                let tx_clone = tx.clone();
                let err_tx_clone = err_tx.clone();
                let regex_clone = regex.clone();
                let file_name_clone = file_name.clone();
                let active_clone = active.clone();
                let deferred_clone = deferred.clone();
                active_clone.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(search_files(tx_clone, err_tx_clone, regex_clone, file_name_clone, path, active_clone, deferred_clone));
            }
        }

        deferred.fetch_add(1, Ordering::SeqCst);
    })
}

async fn is_symlink_or_irregular(file_type: &std::fs::FileType) -> bool {
    file_type.is_symlink() || file_type.is_block_device() || file_type.is_char_device() || file_type.is_fifo() || file_type.is_socket()
}

async fn is_readable(path: &Path) -> bool {
    match fs::metadata(path).await {
        Ok(metadata) => metadata.permissions().mode() & 0o444 != 0,
        Err(_) => false,
    }
}
