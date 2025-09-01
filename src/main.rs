use clap::Parser;

use crate::cli::Args;

pub mod api;
pub mod cli;
pub mod config;
pub mod server;
pub(crate) mod twitter;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.serve {
        let port = args.port;
        server::run(port).await
    }
}
