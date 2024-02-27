use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use clap::Parser;
use tokio::join;

const ROOT_DIR: &str = "/";

#[derive(Parser, Debug)]
struct Config {
    /// Directory in which the search will be performed
    #[arg(short, long)]
    dir: String,

    /// Target file name for search
    #[arg(short, long)]
    file: String,
}

#[tokio::main]
async fn main() {
    let cfg = &Arc::new(match parse_args() {
        Ok(cfg) => cfg,
        Err(err) => panic!("{}", err)
    });

    let file = cfg.file.clone();
    let dir  = cfg.dir.clone();

    let (file_tx, file_rx) = std::sync::mpsc::channel::<String>();
    let (err_tx, err_rx) = std::sync::mpsc::channel::<String>();

    let files_handle  = tokio::spawn(print_files(file_rx));
    let errors_handle = tokio::spawn(print_errors(err_rx));
    let find_handle   = tokio::spawn(find(file_tx, err_tx, file, dir));

    join!(find_handle, files_handle, errors_handle);
}

async fn print_files(files_rx: Receiver<String>) {
    let mut i = 1;
    for f in files_rx.iter() {
        println!("#{}: {}", i, f);
        i += 1;
    }
}

async fn print_errors(errors_rx: Receiver<String>) {
    for e in errors_rx.iter() {
        println!("error: {}", e);
    }
}

fn parse_args() -> Result<Config, String> {
    let mut cfg = Config::parse();

    if cfg.dir.is_empty() {
        cfg.dir = ROOT_DIR.to_string()
    }

    if cfg.file.is_empty() {
        return Err("file cannot be empty or omitted".to_string());
    }

    return Ok(cfg);
}

async fn find(file_tx: Sender<String>, err_tx: Sender<String>, file: String, dir: String)  {
    let dir_entries = match std::fs::read_dir(dir.as_str()) {
        Ok(dir_entries) => dir_entries,
        Err(err) => {
            err_tx.send(err.to_string());
            return;
        }
    };

    dir_entries.for_each(|entry| {
        let dir_entry = match entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                err_tx.send(err.to_string());
                return;
            }
        };

        if dir_entry.file_name().to_str().unwrap() == &*file {
            file_tx.send(format!(
                "{}/{}", dir, dir_entry.file_name().to_os_string().to_str().unwrap()
            ));
        }

        if dir_entry.path().is_dir() {
            let mut into_dir: String;
            if dir_entry.path().to_str().unwrap_or("/") == "/" {
                into_dir = "/".to_string();
            } else {
                into_dir = format!(
                    "{}/{}", dir, dir_entry.path().to_str().unwrap().to_string()
                )
            }

            let file_tx_cloned = file_tx.clone();
            let err_tx_cloned = err_tx.clone();
            let file_cloned = file.clone();
            let dir_cloned = dir.clone();

            tokio::spawn(find(file_tx_cloned, err_tx_cloned, file_cloned, dir_cloned));
        }
    });
}