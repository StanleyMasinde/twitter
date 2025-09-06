pub mod config;

use std::{
    io::{self, IsTerminal, Read},
    process,
};

use clap::{Command, Parser, Subcommand};

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
                        io::stdin()
                            .read_to_string(&mut buf)
                            .expect("Failed to read tweet!");

                        buf.trim().to_string()
                    } else {
                        println!("Could not find tweet body.");
                        process::exit(1)
                    }
                }
            };

            let payload = CreateTweet { text: tweet_body };
            let api_res = tweet::create(client, payload).await;

            match api_res {
                Ok(ok) => {
                    println!("{}", ok.content)
                }
                Err(err) => println!("Error:{}", err),
            }
        }
        Commands::Config { edit, show, init } => {
            if edit {
                config::edit();
            } else if show {
                config::show();
            } else if init {
                config::init();
            } else {
                Args::parse_from(["", "config", "--help"]);
            }
        }
    }
}
