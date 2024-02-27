use tokio::signal;
use tokio::sync::mpsc;
use walkdir::WalkDir;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <directory> <filename>", args[0]);
        return;
    }

    let directory = args[1].clone();
    let filename = Arc::new(args[2].clone());

    let (tx, mut rx) = mpsc::channel(100);

    let dir_path = PathBuf::from(directory);
    let file_name = filename.clone();

    tokio::spawn(async move {
        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy() == *file_name)
        {
            let path = entry.path().to_path_buf();
            tx.send(path).await.expect("failed to send path");
        }
    });

    let ctrl_c = signal::ctrl_c();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl-C, stopping");
        },
        _ = receive_files(&mut rx) => {},
    }
}

async fn receive_files(rx: &mut mpsc::Receiver<PathBuf>) {
    while let Some(path) = rx.recv().await {
        println!("Found file: {}", path.display());
    }
}