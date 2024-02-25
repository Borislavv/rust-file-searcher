use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use clap::Parser;

const ROOT_DIR: &str = "/";

#[derive(Parser, Debug)]
struct Config {
    /// Directory in which the search will be performed
    #[arg(short, long)]
    dir: String,

    /// Target file name for search
    #[arg(short, long)]
    file: String,

    /// Acceptable found files extensions
    #[arg(short, long, value_delimiter = ',')]
    extensions: Vec<String>
}

#[tokio::main]
async fn main() {
    let cfg = &Arc::new(match parse_args() {
        Ok(cfg) => cfg,
        Err(err) => panic!("{}", err)
    });

    let (sender, mut receiver) = std::sync::mpsc::channel::<&str>();

    let mut find_handle = tokio::spawn(find(sender, cfg).await?);

    let mut i = 1;
    for path in receiver.iter() {
        println!("#{}: {}\n", i, path);
        i += 1;
    }

    find_handle.await?;

    println!("{:?}", cfg);
}

fn parse_args() -> Result<Config, &'static str> {
    let mut cfg = Config::parse();

    if cfg.dir.is_empty() {
        cfg.dir = ROOT_DIR.to_string()
    }

    if cfg.file.is_empty() {
        return Err("file cannot be empty or omitted");
    }

    return Ok(cfg);
}

async fn find(ch: Sender<&str>, cfg: &Arc<Config>)  {
    for entry in std::fs::read_dir(cfg.dir.clone()).unwrap() {
        let dir = entry.unwrap().;
    }
}