mod cli;
mod config;
mod twitter;
mod usage;
mod utils;

#[tokio::main]
async fn main() {
    cli::run().await;
}
