use clap::{Parser, Subcommand};

use crate::{
    api::client::{ApiClient, HttpClient},
    server::{self, routes::api::CreateTweet},
    twitter::tweet,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run in server mode
    Serve {
        /// Specify the server port
        #[arg(long)]
        port: Option<u16>,
    },

    /// Create a new tweet.
    Tweet {
        /// The body of the tweet
        #[arg(long, short, name = "body")]
        body: String,
    },
}

pub async fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Serve { port } => server::run(port).await,
        Commands::Tweet { body } => {
            let client = ApiClient::new();
            let payload = CreateTweet { text: body };
            let api_res = tweet::create(client, payload).await;

            match api_res {
                Ok(ok) => {
                    println!("{}", ok.content)
                }
                Err(err) => println!("Error:{}", err),
            }
        }
    }
}
