pub mod cli;
pub mod config;
pub mod server;
pub(crate) mod twitter;
pub mod utils;

#[tokio::main]
async fn main() {
    cli::run().await;
}
