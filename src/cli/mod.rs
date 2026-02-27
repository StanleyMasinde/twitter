pub mod config;
pub mod update;

use std::{
    env::temp_dir,
    fs,
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand, ValueEnum};
use tabled::builder::Builder;

use crate::{schedule, utils::send_due_tweets};
use crate::{
    twitter::{
        self,
        tweet::{self, Media, TweetBody, TwitterApi},
    },
    usage,
    utils::{self, gracefully_exit},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new tweet.
    Tweet {
        /// The body of the tweet
        #[arg(long, short, name = "body")]
        body: Option<String>,

        /// An image to attach to the tweet
        #[arg(long, short, name = "image")]
        image: Option<PathBuf>,

        /// Launch the editor
        #[arg(long, short, name = "editor")]
        editor: bool,
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

    /// API usage
    Usage {},

    /// Self update
    Update {},

    /// Schedule Tweets
    Schedule {
        #[command(subcommand)]
        command: ScheduleEnum,
    },
}

#[derive(Debug, Subcommand)]
enum ScheduleEnum {
    /// Add a new tweet to the Schedule
    New {
        /// The body of the new tweet
        #[arg(long)]
        body: String,

        /// The time to send the tweet
        #[arg(long, visible_alias = "at")]
        on: String,
    },

    /// List all the scheduled tweets
    List(ListArgs),
    /// Clear all the scheduled tweets
    Clear {},

    /// Run all ready-to-send scheduled tweets
    Run {},
}

#[derive(Debug, clap::Args)]
struct ListArgs {
    #[arg(long, value_enum, default_value_t = ListFilter::All)]
    filter: ListFilter,
}

#[derive(Debug, Clone, ValueEnum)]
enum ListFilter {
    All,
    Failed,
    Sent,
}

pub async fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Tweet {
            body,
            image,
            editor,
        } => {
            let client = reqwest::Client::new();
            let mut media_id: Option<String> = None;

            if let Some(image_path) = image {
                let upload_result = twitter::media::upload(client.clone(), image_path).await;
                media_id = match upload_result {
                    Ok(media) => Some(media),
                    Err(err) => gracefully_exit(&err.message),
                };
            }

            let tweet_body: Option<String> = match body {
                Some(tweet) => Some(tweet),
                None => {
                    if !io::stdin().is_terminal() {
                        let mut buf = String::new();
                        let read_stdin_string = io::stdin().read_to_string(&mut buf);

                        if read_stdin_string.is_ok() {
                            Some(buf.trim().to_string())
                        } else {
                            gracefully_exit(
                                "Failed to read stdin as text.\nMake sure you are piping UTF-8 text.",
                            )
                        }
                    } else if editor {
                        let temp_file = temp_dir().join("tweet.txt");
                        let status = utils::open_editor(&temp_file);

                        if status.success() {
                            match fs::read_to_string(&temp_file) {
                                Ok(tweet) => {
                                    let _ = fs::remove_file(temp_file);
                                    Some(tweet)
                                }
                                Err(_) => {
                                    gracefully_exit("Failed to read the tweet from the editor.")
                                }
                            }
                        } else {
                            gracefully_exit("Failed to open the default editor");
                        }
                    } else {
                        None
                    }
                }
            };

            let mut payload = TweetBody {
                text: tweet_body,
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
                    println!("{}", ok.content)
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
        Commands::Usage {} => usage::show().await,
        Commands::Update {} => update::run().await,
        Commands::Schedule { command } => match command {
            ScheduleEnum::New { body, on } => {
                let schedule = schedule::Schedule::new(&body, &on);
                if schedule.save() {
                    println!("Tweet scheduled for {on}.");
                } else {
                    eprintln!("Could not schedule tweet.");
                }
            }
            ScheduleEnum::Clear {} => {
                let schedule = schedule::Schedule::default();
                let cleared = schedule.clear();
                let suffix = if cleared == 1 { "" } else { "s" };
                println!("Cleared {cleared} scheduled tweet{suffix}.");
            }
            ScheduleEnum::Run {} => {
                send_due_tweets().await;
            }
            ScheduleEnum::List(list_args) => {
                let schedule = schedule::Schedule::default();
                let mut table_builder = Builder::new();
                let tweets = match list_args.filter {
                    ListFilter::All => schedule.all(),
                    ListFilter::Failed => schedule.failed(),
                    ListFilter::Sent => schedule.sent(),
                };
                if tweets.is_empty() {
                    println!("No scheduled tweets were found.");
                    return;
                }
                table_builder.push_record(["Id", "Body", "Send time"]);
                for row in tweets {
                    table_builder.push_record([row.id.to_string(), row.body, row.scheduled_for]);
                }

                let table = table_builder.build();
                println!("{table}");
            }
        },
    }
}
