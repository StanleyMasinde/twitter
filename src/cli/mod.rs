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

    /// Fetch tweets
    Tweets {
        #[command(subcommand)]
        command: TweetsEnum,
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

    /// Likes
    Likes {
        #[command(subcommand)]
        command: LikesEnum,
    },

    /// Lists
    Lists {
        #[command(subcommand)]
        command: ListsEnum,
    },

    /// Direct messages
    Dms {
        #[command(subcommand)]
        command: DmsEnum,
    },

    /// Retweets
    Retweets {
        #[command(subcommand)]
        command: RetweetsEnum,
    },

    /// Timeline
    Timeline {
        #[command(subcommand)]
        command: TimelineEnum,
    },

    /// Mentions
    Mentions {},

    /// Show information about the current authenticated user
    Me {},
}

#[derive(Debug, Subcommand)]
enum ScheduleEnum {
    /// Add a new tweet to the Schedule
    New {
        /// The body of the new tweet
        #[arg(long)]
        body: String,

        /// The time to send the tweet
        #[arg(long, visible_aliases = ["at", "in"])]
        on: String,
    },

    /// List all the scheduled tweets
    List(ListArgs),
    /// Clear all the scheduled tweets
    Clear {},

    /// Run all ready-to-send scheduled tweets
    Run {},
}

#[derive(Debug, Subcommand)]
enum LikesEnum {
    /// Show the users who liked a tweet
    By {
        /// The tweet id
        #[arg(long)]
        tweet_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Like a tweet for the current authenticated user
    Create {
        /// The tweet id to like
        #[arg(long)]
        tweet_id: String,
    },

    /// Delete a liked tweet for the current authenticated user
    Delete {
        /// The tweet id to unlike
        #[arg(long)]
        tweet_id: String,
    },

    /// Fetch tweets liked by the current authenticated user
    Tweets {},
}

#[derive(Debug, Subcommand)]
enum ListsEnum {
    /// Fetch a list by id
    ById {
        /// The list id
        #[arg(long)]
        list_id: String,
    },

    /// Create a list
    Create {
        /// The list name
        #[arg(long)]
        name: String,

        /// The list description
        #[arg(long)]
        description: Option<String>,

        /// Whether the list is private
        #[arg(long)]
        private: Option<bool>,
    },

    /// Delete a list
    Delete {
        /// The list id
        #[arg(long)]
        list_id: String,
    },

    /// Fetch the members of a list
    Members {
        /// The list id
        #[arg(long)]
        list_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Remove the current authenticated user from a list
    RemoveMember {
        /// The list id
        #[arg(long)]
        list_id: String,
    },

    /// Fetch the lists the current authenticated user belongs to
    Memberships {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },
}

#[derive(Debug, Subcommand)]
enum DmsEnum {
    /// Send a message to an existing DM conversation
    Send {
        /// The conversation id
        #[arg(long)]
        conversation_id: String,

        /// The message text
        #[arg(long)]
        text: String,
    },
}

#[derive(Debug, Subcommand)]
enum RetweetsEnum {
    /// Show the users who retweeted a tweet
    By {
        /// The tweet id
        #[arg(long)]
        tweet_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Create a retweet for the current authenticated user
    Create {
        /// The tweet id to retweet
        #[arg(long)]
        tweet_id: String,
    },
    /// Delete the current authenticated user's retweet of a tweet
    Delete {
        /// The tweet id to unretweet
        #[arg(long)]
        tweet_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum TimelineEnum {
    /// Fetch the reverse-chronological home timeline
    #[command(visible_alias = "reverse")]
    ReverseChronological {},
}

#[derive(Debug, Subcommand)]
enum TweetsEnum {
    /// Fetch a tweet by id
    ById {
        /// The id of the tweet to fetch
        id: String,
    },

    /// Fetch tweets from a user by id
    User {
        /// The id of the user to fetch tweets for
        #[arg(long)]
        id: String,
    },

    /// Search recent tweets
    Recent {
        /// Search query
        #[arg(long)]
        query: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Search all tweets
    All {
        /// Search query
        #[arg(long)]
        query: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u16,
    },
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

pub fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Tweet {
            body,
            image,
            editor,
        } => {
            let mut media_id: Option<String> = None;

            if let Some(image_path) = image {
                let upload_result = twitter::media::upload(image_path);
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
            let mut tweet = tweet::Tweet::new(payload);
            let api_res = tweet.create();

            match api_res {
                Ok(ok) => {
                    println!("{}", ok.content)
                }
                Err(err) => println!("{}", err.message),
            }
        }
        Commands::Tweets { command } => match command {
            TweetsEnum::ById { id } => {
                let tweet_res = twitter::tweets::TweetLookup::new(id).fetch();
                match tweet_res {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            TweetsEnum::User { id } => {
                let tweets = twitter::tweets::UserTweets::new(id).max_results(10).fetch();
                match tweets {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No tweets found.");
                            return;
                        }

                        for tweet in tweets {
                            println!(
                                "{}\n",
                                twitter::TweetCreateResponse {
                                    data: tweet,
                                    includes: includes.clone(),
                                }
                            );
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            TweetsEnum::Recent { query, max_results } => {
                let tweets = twitter::tweets::RecentTweets::new(query)
                    .max_results(max_results)
                    .fetch();
                match tweets {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No recent tweets found.");
                            return;
                        }

                        for tweet in tweets {
                            println!(
                                "{}\n",
                                twitter::TweetCreateResponse {
                                    data: tweet,
                                    includes: includes.clone(),
                                }
                            );
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            TweetsEnum::All { query, max_results } => {
                let tweets = twitter::tweets::AllTweets::new(query)
                    .max_results(max_results)
                    .fetch();
                match tweets {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No tweets found.");
                            return;
                        }

                        for tweet in tweets {
                            println!(
                                "{}\n",
                                twitter::TweetCreateResponse {
                                    data: tweet,
                                    includes: includes.clone(),
                                }
                            );
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
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
        Commands::Usage {} => usage::show(),
        Commands::Update {} => update::run(),
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
                send_due_tweets();
            }
            ScheduleEnum::List(list_args) => {
                let schedule = schedule::Schedule::default();
                let mut table_builder = Builder::new();
                let filter = list_args.filter.clone();
                let mut tweets = match filter {
                    ListFilter::All => schedule.all(),
                    ListFilter::Failed => schedule.failed(),
                    ListFilter::Sent => schedule.sent(),
                };
                if tweets.is_empty() {
                    println!("No scheduled tweets were found.");
                    return;
                }

                tweets.sort_by(|a, b| a.scheduled_for.cmp(&b.scheduled_for));

                let show_last_error = matches!(filter, ListFilter::All | ListFilter::Failed);
                let show_sent_at = matches!(filter, ListFilter::All | ListFilter::Sent);

                let mut headers = vec![
                    "Id".to_string(),
                    "Status".to_string(),
                    "Body".to_string(),
                    "Send time".to_string(),
                    "Attempts".to_string(),
                ];

                if show_last_error {
                    headers.push("Last error".to_string());
                }

                if show_sent_at {
                    headers.push("Sent at".to_string());
                }

                table_builder.push_record(headers);

                for row in &tweets {
                    let mut record = vec![
                        row.id.to_string(),
                        row.status.to_string(),
                        if row.body.chars().count() > 80 {
                            format!("{}...", row.body.chars().take(77).collect::<String>())
                        } else {
                            row.body.clone()
                        },
                        row.scheduled_for.clone(),
                        row.attempts.to_string(),
                    ];
                    if show_last_error {
                        record.push(row.last_error.clone().unwrap_or_else(|| "-".to_string()));
                    }
                    if show_sent_at {
                        record.push(row.sent_at.clone().unwrap_or_else(|| "-".to_string()));
                    }
                    table_builder.push_record(record);
                }

                let table = table_builder.build();
                println!("{table}");

                if matches!(filter, ListFilter::All) {
                    let pending = tweets
                        .iter()
                        .filter(|row| matches!(row.status, schedule::ScheduleStatus::Pending))
                        .count();
                    let failed = tweets
                        .iter()
                        .filter(|row| matches!(row.status, schedule::ScheduleStatus::Failed))
                        .count();
                    let sent = tweets
                        .iter()
                        .filter(|row| matches!(row.status, schedule::ScheduleStatus::Sent))
                        .count();
                    println!(
                        "Total: {} (Pending: {}, Failed: {}, Sent: {})",
                        tweets.len(),
                        pending,
                        failed,
                        sent
                    );
                } else {
                    println!("Total: {}", tweets.len());
                }
            }
        },
        Commands::Likes { command } => match command {
            LikesEnum::By {
                tweet_id,
                max_results,
            } => {
                let users = twitter::likes::LikingUsers::new(tweet_id).max_results(max_results);

                match users.fetch() {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No liking users found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            LikesEnum::Create { tweet_id } => {
                let like = twitter::likes::CreateLike::for_current_user(tweet_id);

                match like {
                    Ok(like) => match like.send() {
                        Ok(ok) => {
                            if ok.content.data.liked {
                                println!("Liked tweet.");
                            } else {
                                eprintln!("Tweet was not liked.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            LikesEnum::Delete { tweet_id } => {
                let unlike = twitter::likes::DeleteLike::for_current_user(tweet_id);

                match unlike {
                    Ok(unlike) => match unlike.send() {
                        Ok(ok) => {
                            if ok.content.data.liked {
                                eprintln!("Tweet is still liked.");
                            } else {
                                println!("Removed like.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            LikesEnum::Tweets {} => {
                let user_id = match utils::get_current_user_id() {
                    Ok(id) => id,
                    Err(err) => {
                        eprintln!("{err}");
                        return;
                    }
                };

                let likes = twitter::likes::Likes::new(user_id).max_results(10);
                let likes_res = likes.fetch();
                match likes_res {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No liked tweets found.");
                            return;
                        }

                        for tweet in tweets {
                            println!(
                                "{}\n",
                                twitter::TweetCreateResponse {
                                    data: tweet,
                                    includes: includes.clone(),
                                }
                            );
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Lists { command } => match command {
            ListsEnum::ById { list_id } => {
                let list = twitter::lists::ListLookup::new(list_id);

                match list.fetch() {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::Create {
                name,
                description,
                private,
            } => {
                let create = twitter::lists::CreateList::new(name)
                    .description(description)
                    .private(private);

                match create.send() {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::Delete { list_id } => {
                let delete = twitter::lists::DeleteList::new(list_id);

                match delete.send() {
                    Ok(ok) => {
                        if ok.content.data.deleted {
                            println!("Deleted list.");
                        } else {
                            eprintln!("List was not deleted.");
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::Members {
                list_id,
                max_results,
            } => {
                let members = twitter::lists::ListMembers::new(list_id).max_results(max_results);

                match members.fetch() {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No list members found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::RemoveMember { list_id } => {
                let remove = twitter::lists::DeleteListMember::for_current_user(list_id);

                match remove {
                    Ok(remove) => match remove.send() {
                        Ok(ok) => {
                            if ok.content.data.is_member {
                                eprintln!("Current user is still a member of the list.");
                            } else {
                                println!("Removed current user from the list.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::Memberships { max_results } => {
                let lists = twitter::lists::ListMemberships::current_user()
                    .map(|lists| lists.max_results(max_results));

                match lists {
                    Ok(lists) => match lists.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No list memberships found.");
                                return;
                            }

                            println!("{}", ok.content);
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Dms { command } => match command {
            DmsEnum::Send {
                conversation_id,
                text,
            } => {
                let message = twitter::dms::SendConversationMessage::new(conversation_id, text);

                match message.send() {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Retweets { command } => match command {
            RetweetsEnum::By {
                tweet_id,
                max_results,
            } => {
                let users = twitter::retweets::RetweetedBy::new(tweet_id).max_results(max_results);

                match users.fetch() {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No retweeters found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            RetweetsEnum::Create { tweet_id } => {
                let create = twitter::retweets::CreateRetweet::for_current_user(tweet_id);

                match create {
                    Ok(create) => match create.send() {
                        Ok(ok) => {
                            if ok.content.data.retweeted {
                                println!("Retweeted tweet.");
                            } else {
                                eprintln!("Tweet was not retweeted.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            RetweetsEnum::Delete { tweet_id } => {
                let delete = twitter::retweets::DeleteRetweet::for_current_user(tweet_id);

                match delete {
                    Ok(delete) => match delete.send() {
                        Ok(ok) => {
                            if ok.content.data.retweeted {
                                eprintln!("Tweet is still retweeted.");
                            } else {
                                println!("Removed retweet.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Timeline { command } => match command {
            TimelineEnum::ReverseChronological {} => {
                let user_id = match utils::get_current_user_id() {
                    Ok(id) => id,
                    Err(err) => {
                        eprintln!("{err}");
                        return;
                    }
                };

                let timeline = twitter::timeline::Timeline::new(user_id).max_results(10);
                let timeline_res = timeline.fetch();
                match timeline_res {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No tweets found in timeline.");
                            return;
                        }

                        for tweet in tweets {
                            println!(
                                "{}\n",
                                twitter::TweetCreateResponse {
                                    data: tweet,
                                    includes: includes.clone(),
                                }
                            );
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Mentions {} => {
            let user_id = match utils::get_current_user_id() {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("{err}");
                    return;
                }
            };

            let mentions = twitter::mentions::Mentions::new(user_id).max_results(10);
            let mentions_res = mentions.fetch();
            match mentions_res {
                Ok(ok) => {
                    let tweets = ok.content.data;
                    let includes = ok.content.includes;
                    if tweets.is_empty() {
                        println!("No mentions found.");
                        return;
                    }

                    for tweet in tweets {
                        println!(
                            "{}\n",
                            twitter::TweetCreateResponse {
                                data: tweet,
                                includes: includes.clone(),
                            }
                        );
                    }
                }
                Err(err) => eprintln!("{}", err.message),
            }
        }
        Commands::Me {} => {
            let me_res = twitter::user::me();
            match me_res {
                Ok(ok) => println!("{}", ok.content),
                Err(err) => eprintln!("{}", err.message),
            }
        }
    }
}
