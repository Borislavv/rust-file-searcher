use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    dir: String,

    #[arg(short, long)]
    file: String,

    #[arg(short, long, value_delimiter = ',')]
    extensions: Vec<String>
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);
}
