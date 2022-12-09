use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long)]
    configuration_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    println!("Hello, world!");
    println!("{:?}", cli);
}
