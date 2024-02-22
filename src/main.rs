use clap::Parser;

#[derive(Parser, Debug)]
struct Config {
    #[arg(short, long)]
    file: String,
}

fn main() {
    let cfg = Config::parse();

    println!("{:?}", cfg);
}
