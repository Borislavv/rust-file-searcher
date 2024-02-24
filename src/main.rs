use std::error::Error;
use std::string::ToString;
use clap::Parser;

const ROOT_DIR: String = "/".to_string();

#[derive(Parser, Debug)]
struct Args {
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

fn main() {
    let args = parse_args();

    println!("{:?}", args);
}

fn parse_args() -> Result<Args, dyn Error> {
    let mut args = Args::parse();

    if args.dir.is_empty() {
        args.dir = ROOT_DIR
    }

    return Ok(args);
}
