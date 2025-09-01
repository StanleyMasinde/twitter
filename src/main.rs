use clap::Parser;
use log::{error, info};

use crate::{
    api::client::{ApiClient, HttpClient},
    cli::Args,
    server::routes::api::CreateTweet,
    twitter::tweet,
};

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

    if let Some(tweet_text) = args.tweet {
        let client = ApiClient::new();
        let payload = CreateTweet { text: tweet_text };
        let api_res = tweet::create(client, payload).await;

        match api_res {
            Ok(ok) => {
                println!("{}", ok.content)
            }
            Err(err) => error!("Error:{}", err),
        }
    }
}
