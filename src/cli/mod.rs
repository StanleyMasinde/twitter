pub mod config;

use std::{
    env::temp_dir,
    fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
    process,
};

use clap::{Parser, Subcommand};

use crate::{
    server::{self},
    twitter::{
        self,
        tweet::{self, Media, TweetBody, TwitterApi},
    },
    utils,
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

        /// An image to attach to the tweet
        #[arg(long, short, name = "image")]
        image: Option<PathBuf>,
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

        /// Validate config
        #[arg(long)]
        validate: bool,
    },
}

pub async fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Serve { port } => server::run(port).await,
        Commands::Tweet { body, image } => {
            let client = reqwest::Client::new();
            let mut media_id: Option<String> = None;

            if let Some(image_path) = image {
                let upload_result = twitter::media::upload(client.clone(), image_path).await;
                media_id = match upload_result {
                    Ok(media) => Some(media),
                    Err(err) => {
                        eprintln!("{}", err.message);
                        process::exit(1);
                    }
                };
            }

            let tweet_body = match body {
                Some(tweet) => tweet,
                None => {
                    if !io::stdin().is_terminal() {
                        let mut buf = String::new();
                        let read_stdin_string = io::stdin().read_to_string(&mut buf);

                        if read_stdin_string.is_ok() {
                            buf.trim().to_string()
                        } else {
                            eprintln!(
                                "Failed to read stdin as text.\nMake sure you are piping UTF-8 text."
                            );
                            process::exit(1)
                        }
                    } else {
                        let temp_file = temp_dir().join("tweet.txt");
                        let status = utils::open_editor(&temp_file);

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

            let mut payload = TweetBody {
                text: Some(tweet_body),
                reply: None,
                media: None,
            };

            if let Some(media) = media_id {
                let media_body = Media {
                    media_ids: [media].to_vec(),
                };
                payload.media = Some(media_body);
            }
            let mut tweet = tweet::Tweet::new(client, payload);
            let api_res = tweet.create().await;

            match api_res {
                Ok(ok) => {
                    println!("{:?}", ok.content)
                }
                Err(err) => println!("{}", err.message),
            }
        }
        Commands::Config {
            edit,
            show,
            init,
            validate,
        } => {
            if edit {
                config::edit();
            } else if show {
                config::show();
            } else if init {
                config::init();
            } else if validate {
                config::validate();
            } else {
                Args::parse_from(["", "config", "--help"]);
            }
        }
    }
}
