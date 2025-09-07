pub mod config;

use std::{
    io::{self, IsTerminal, Read},
    process,
};

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
        body: Option<String>,
    },

    /// Manage config
    Config {
        /// init the config file
        #[arg(long)]
        init: bool,

        /// Open the config in an editor
        #[arg(long, short)]
        edit: bool,

        /// Show the config file.
        #[arg(long)]
        show: bool,
    },
}

pub async fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Serve { port } => server::run(port).await,
        Commands::Tweet { body } => {
            let client = ApiClient::new();

            let tweet_body = match body {
                Some(tweet) => tweet,
                None => {
                    if !io::stdin().is_terminal() {
                        let mut buf = String::new();
                        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                            eprintln!("Failed to read tweet from stdin: {e}");
                            process::exit(1);
                        });

                        buf.trim().to_string()
                    } else {
                        eprintln!("Error: No tweet body provided. Use --body flag or pipe input.");
                        process::exit(1)
                    }
                }
            };

            let payload = CreateTweet { text: tweet_body };
            let api_res = tweet::create(client, payload).await;

            match api_res {
                Ok(ok) => {
                    println!("Tweet posted successfully!");
                    println!("Response: {}", ok.content)
                }
                Err(err) => {
                    eprintln!("Error posting tweet: {err}");
                    process::exit(1);
                }
            }
        }
        Commands::Config { edit, show, init } => {
            if edit {
                if let Err(e) = config::edit() {
                    eprintln!("Error editing config: {e}");
                    process::exit(1);
                }
            } else if show {
                if let Err(e) = config::show() {
                    eprintln!("Error showing config: {e}");
                    process::exit(1);
                }
            } else if init {
                if let Err(e) = config::init() {
                    eprintln!("Error initializing config: {e}");
                    process::exit(1);
                }
            } else {
                Args::parse_from(["", "config", "--help"]);
            }
        }
    }
}
