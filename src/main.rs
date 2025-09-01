pub mod api;
pub mod cli;
pub mod config;
pub mod server;
pub(crate) mod twitter;

#[tokio::main]
async fn main() {
    cli::run().await;
}
