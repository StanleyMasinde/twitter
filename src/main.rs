pub mod cli;
pub mod config;
pub mod twitter;
pub mod usage;
pub mod utils;

#[tokio::main]
async fn main() {
    cli::run().await;
}
