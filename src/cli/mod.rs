use clap::Parser;

use crate::{
    api::client::{ApiClient, HttpClient},
    server::{self, routes::api::CreateTweet},
    twitter::tweet,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Run is server mode.
    #[arg[short, long]]
    pub serve: bool,

    /// Server port
    #[arg[long]]
    pub port: Option<String>,

    /// Create a new tweet
    #[arg[short, long]]
    pub tweet: Option<String>,
}

pub async fn run() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        Args::parse_from(["", "--help"]);
    }
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
            Err(err) => println!("Error:{}", err),
        }
    }
}
