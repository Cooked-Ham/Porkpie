use clap::Parser;
use porkpie_cli::{run, Cli};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
