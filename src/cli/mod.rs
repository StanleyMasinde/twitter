pub mod config;

use std::{
    env::{self, temp_dir},
    fs,
    io::{self, IsTerminal, Read},
    process::{self, Command},
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
                        io::stdin()
                            .read_to_string(&mut buf)
                            .expect("Failed to read tweet!");

                        buf.trim().to_string()
                    } else {
                        let editor = env::var("EDITOR")
                            .or_else(|_| env::var("VISUAL"))
                            .unwrap_or_else(|_| "vi".to_string());

                        let file_name = "new_tweet.txt";

                        let temp_file = temp_dir().join(file_name);

                        let status = Command::new(editor)
                            .arg(&temp_file)
                            .status()
                            .expect("Failed to open the editor.");

                        if status.success() {
                            match fs::read_to_string(&temp_file) {
                                Ok(tweet) => {
                                    let _ = fs::remove_file(temp_file);

                                    if tweet.is_empty() {
                                       println!("Could not find the Tweet text. Exiting.");
                                        process::exit(0);
                                    } else {
                                        tweet
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Failed to read the tweet.");
                                    process::exit(1);
                                }
                            }
                        } else {
                            process::exit(1)
                        }
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
