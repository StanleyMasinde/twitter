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

    /// Bookmarks
    Bookmarks {
        #[command(subcommand)]
        command: BookmarksEnum,
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

    /// Mutes
    Mutes {
        #[command(subcommand)]
        command: MutesEnum,
    },

    /// Blocks
    Blocks {
        #[command(subcommand)]
        command: BlocksEnum,
    },
    /// Timeline
    Timeline {
        #[command(subcommand)]
        command: TimelineEnum,
    },

    /// Mentions
    Mentions {},

    /// Users
    Users {
        #[command(subcommand)]
        command: UsersEnum,
    },

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
enum BookmarksEnum {
    /// Fetch bookmarks for the current authenticated user
    List {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Bookmark a tweet for the current authenticated user
    Create {
        /// The tweet id to bookmark
        #[arg(long)]
        tweet_id: String,
    },

    /// Remove a bookmark for the current authenticated user
    Delete {
        /// The tweet id to remove from bookmarks
        #[arg(long)]
        tweet_id: String,
    },

    /// List bookmark folders for the current authenticated user
    Folders {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },
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

    /// Fetch the lists owned by the current authenticated user
    Owned {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Update a list
    Update {
        /// The list id
        #[arg(long)]
        list_id: String,

        /// The new list name
        #[arg(long)]
        name: Option<String>,

        /// The new list description
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

    /// Fetch the tweets in a list
    Tweets {
        /// The list id
        #[arg(long)]
        list_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Add a user to a list
    AddMember {
        /// The list id
        #[arg(long)]
        list_id: String,

        /// The user id to add
        #[arg(long)]
        user_id: String,
    },

    /// Remove the current authenticated user from a list
    RemoveMember {
        /// The list id
        #[arg(long)]
        list_id: String,

        /// The user id to remove. Defaults to the current authenticated user.
        #[arg(long)]
        user_id: Option<String>,
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
    /// Show DM events for a conversation
    ConversationEvents {
        /// The conversation id
        #[arg(long)]
        conversation_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Show DM events for the current authenticated user
    Events {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Show DM events with a participant
    With {
        /// The participant id
        #[arg(long)]
        participant_id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Create a DM conversation and send the initial message
    Create {
        /// Comma-separated participant ids
        #[arg(long, value_delimiter = ',')]
        participant_ids: Vec<String>,

        /// The initial message text
        #[arg(long)]
        text: String,
    },

    /// Send a message by participant id
    SendWith {
        /// The participant id
        #[arg(long)]
        participant_id: String,

        /// The message text
        #[arg(long)]
        text: String,
    },

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
enum MutesEnum {
    /// Mute a user for the current authenticated user
    Create {
        /// The target user id to mute
        #[arg(long)]
        target_user_id: String,
    },

    /// Show the users muted by the current authenticated user
    List {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Unmute a user for the current authenticated user
    Delete {
        /// The target user id to unmute
        #[arg(long)]
        target_user_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum BlocksEnum {
    /// Block a user for the current authenticated user
    Create {
        /// The target user id to block
        #[arg(long)]
        target_user_id: String,
    },

    /// Show the users blocked by the current authenticated user
    List {
        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Unblock a user for the current authenticated user
    Delete {
        /// The target user id to unblock
        #[arg(long)]
        target_user_id: String,
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

    /// Fetch multiple tweets by ids
    ByIds {
        /// Comma-separated tweet ids
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
    },

    /// Delete a tweet by id
    Delete {
        /// The id of the tweet to delete
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

    /// Get recent tweet counts for a search query
    CountRecent {
        /// Search query
        #[arg(long)]
        query: String,
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

    /// Get all-time tweet counts for a search query
    CountAll {
        /// Search query
        #[arg(long)]
        query: String,
    },
}

#[derive(Debug, Subcommand)]
enum UsersEnum {
    /// Fetch a user by id
    ById {
        /// The id of the user to fetch
        #[arg(long)]
        id: String,
    },

    /// Fetch multiple users by ids
    ByIds {
        /// Comma-separated user ids
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
    },

    /// Fetch a user by username
    ByUsername {
        /// The username to fetch
        #[arg(long)]
        username: String,
    },

    /// Fetch multiple users by usernames
    ByUsernames {
        /// Comma-separated usernames
        #[arg(long, value_delimiter = ',')]
        usernames: Vec<String>,
    },

    /// Fetch the accounts a user follows
    Following {
        /// The user id to fetch
        #[arg(long)]
        id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Fetch a user's followers
    Followers {
        /// The user id to fetch
        #[arg(long)]
        id: String,

        /// Number of results to fetch
        #[arg(long, default_value_t = 10)]
        max_results: u8,
    },

    /// Follow a user for the current authenticated user
    Follow {
        /// The target user id to follow
        #[arg(long)]
        target_user_id: String,
    },

    /// Unfollow a user for the current authenticated user
    Unfollow {
        /// The target user id to unfollow
        #[arg(long)]
        target_user_id: String,
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
            TweetsEnum::ByIds { ids } => {
                let tweets = twitter::tweets::TweetsLookup::new(ids).fetch();
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
            TweetsEnum::Delete { id } => {
                let delete = twitter::tweet::DeleteTweet::new(id);

                match delete.send() {
                    Ok(ok) => {
                        if ok.content.data.deleted {
                            println!("Deleted tweet.");
                        } else {
                            eprintln!("Tweet was not deleted.");
                        }
                    }
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
            TweetsEnum::CountRecent { query } => {
                let counts = twitter::tweets::RecentTweetCounts::new(query).fetch();
                match counts {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No tweet counts found.");
                            return;
                        }

                        println!("{}", ok.content);
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
            TweetsEnum::CountAll { query } => {
                let counts = twitter::tweets::AllTweetCounts::new(query).fetch();
                match counts {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No tweet counts found.");
                            return;
                        }

                        println!("{}", ok.content);
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
        Commands::Bookmarks { command } => match command {
            BookmarksEnum::List { max_results } => {
                let bookmarks = twitter::bookmarks::Bookmarks::current_user()
                    .map(|bookmarks| bookmarks.max_results(max_results));

                match bookmarks {
                    Ok(bookmarks) => match bookmarks.fetch() {
                        Ok(ok) => {
                            let tweets = ok.content.data;
                            let includes = ok.content.includes;
                            if tweets.is_empty() {
                                println!("No bookmarks found.");
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
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            BookmarksEnum::Create { tweet_id } => {
                let bookmark = twitter::bookmarks::CreateBookmark::for_current_user(tweet_id);

                match bookmark {
                    Ok(bookmark) => match bookmark.send() {
                        Ok(ok) => {
                            if ok.content.data.bookmarked {
                                println!("Bookmarked tweet.");
                            } else {
                                eprintln!("Tweet was not bookmarked.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            BookmarksEnum::Delete { tweet_id } => {
                let bookmark = twitter::bookmarks::DeleteBookmark::for_current_user(tweet_id);

                match bookmark {
                    Ok(bookmark) => match bookmark.send() {
                        Ok(ok) => {
                            if ok.content.data.bookmarked {
                                eprintln!("Tweet is still bookmarked.");
                            } else {
                                println!("Removed bookmark.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            BookmarksEnum::Folders { max_results } => {
                let folders = twitter::bookmarks::BookmarkFolders::current_user()
                    .map(|folders| folders.max_results(max_results));

                match folders {
                    Ok(folders) => match folders.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No bookmark folders found.");
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
            ListsEnum::Owned { max_results } => {
                let lists = twitter::lists::OwnedLists::current_user()
                    .map(|lists| lists.max_results(max_results));

                match lists {
                    Ok(lists) => match lists.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No owned lists found.");
                                return;
                            }

                            println!("{}", ok.content);
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::Update {
                list_id,
                name,
                description,
                private,
            } => {
                let update = twitter::lists::UpdateList::new(list_id)
                    .name(name)
                    .description(description)
                    .private(private);

                if !update.has_changes() {
                    eprintln!("Provide at least one field to update.");
                    return;
                }

                match update.send() {
                    Ok(ok) => {
                        if ok.content.data.updated {
                            println!("Updated list.");
                        } else {
                            eprintln!("List was not updated.");
                        }
                    }
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
            ListsEnum::Tweets {
                list_id,
                max_results,
            } => {
                let tweets = twitter::lists::ListTweets::new(list_id).max_results(max_results);

                match tweets.fetch() {
                    Ok(ok) => {
                        let tweets = ok.content.data;
                        let includes = ok.content.includes;
                        if tweets.is_empty() {
                            println!("No list tweets found.");
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
            ListsEnum::AddMember { list_id, user_id } => {
                let add = twitter::lists::CreateListMember::new(list_id, user_id);

                match add.send() {
                    Ok(ok) => {
                        if ok.content.data.is_member {
                            println!("Added user to the list.");
                        } else {
                            eprintln!("User was not added to the list.");
                        }
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            ListsEnum::RemoveMember { list_id, user_id } => {
                let remove = match user_id {
                    Some(user_id) => Ok(twitter::lists::DeleteListMember::new(list_id, user_id)),
                    None => twitter::lists::DeleteListMember::for_current_user(list_id),
                };

                match remove {
                    Ok(remove) => match remove.send() {
                        Ok(ok) => {
                            if ok.content.data.is_member {
                                eprintln!("User is still a member of the list.");
                            } else {
                                println!("Removed user from the list.");
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
            DmsEnum::ConversationEvents {
                conversation_id,
                max_results,
            } => {
                let events = twitter::dms::ConversationDmEvents::new(conversation_id)
                    .max_results(max_results);

                match events.fetch() {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No conversation DM events found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            DmsEnum::Events { max_results } => {
                let events = twitter::dms::UserDmEvents::current_user()
                    .map(|events| events.max_results(max_results));

                match events {
                    Ok(events) => match events.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No DM events found.");
                                return;
                            }

                            println!("{}", ok.content);
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            DmsEnum::With {
                participant_id,
                max_results,
            } => {
                let events =
                    twitter::dms::ParticipantDmEvents::new(participant_id).max_results(max_results);

                match events.fetch() {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No participant DM events found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            DmsEnum::Create {
                participant_ids,
                text,
            } => {
                let conversation = twitter::dms::CreateConversation::new(participant_ids, text);

                match conversation.send() {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            DmsEnum::SendWith {
                participant_id,
                text,
            } => {
                let message = twitter::dms::SendWithParticipantMessage::new(participant_id, text);

                match message.send() {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
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
        Commands::Mutes { command } => match command {
            MutesEnum::Create { target_user_id } => {
                let create = twitter::mutes::CreateMute::for_current_user(target_user_id);

                match create {
                    Ok(create) => match create.send() {
                        Ok(ok) => {
                            if ok.content.data.muting {
                                println!("Muted user.");
                            } else {
                                eprintln!("User was not muted.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            MutesEnum::List { max_results } => {
                let users = twitter::mutes::MutedUsers::current_user()
                    .map(|users| users.max_results(max_results));

                match users {
                    Ok(users) => match users.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No muted users found.");
                                return;
                            }

                            println!("{}", ok.content);
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            MutesEnum::Delete { target_user_id } => {
                let delete = twitter::mutes::DeleteMute::for_current_user(target_user_id);

                match delete {
                    Ok(delete) => match delete.send() {
                        Ok(ok) => {
                            if ok.content.data.muting {
                                eprintln!("Current user is still muting that user.");
                            } else {
                                println!("Unmuted user.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Blocks { command } => match command {
            BlocksEnum::Create { target_user_id } => {
                let create = twitter::blocks::CreateBlock::for_current_user(target_user_id);

                match create {
                    Ok(create) => match create.send() {
                        Ok(ok) => {
                            if ok.content.data.blocking {
                                println!("Blocked user.");
                            } else {
                                eprintln!("User was not blocked.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            BlocksEnum::List { max_results } => {
                let users = twitter::blocks::BlockedUsers::current_user()
                    .map(|users| users.max_results(max_results));

                match users {
                    Ok(users) => match users.fetch() {
                        Ok(ok) => {
                            if ok.content.data.is_empty() {
                                println!("No blocked users found.");
                                return;
                            }

                            println!("{}", ok.content);
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            BlocksEnum::Delete { target_user_id } => {
                let delete = twitter::blocks::DeleteBlock::for_current_user(target_user_id);

                match delete {
                    Ok(delete) => match delete.send() {
                        Ok(ok) => {
                            if ok.content.data.blocking {
                                eprintln!("Current user is still blocking that user.");
                            } else {
                                println!("Unblocked user.");
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
        Commands::Users { command } => match command {
            UsersEnum::ById { id } => {
                let user = twitter::user::UserLookup::new(id).fetch();
                match user {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::ByIds { ids } => {
                let users = twitter::user::UsersLookup::new(ids).fetch();
                match users {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No users found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::ByUsername { username } => {
                let user = twitter::user::UserLookupByUsername::new(username).fetch();
                match user {
                    Ok(ok) => println!("{}", ok.content),
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::ByUsernames { usernames } => {
                let users = twitter::user::UsersLookupByUsernames::new(usernames).fetch();
                match users {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No users found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::Following { id, max_results } => {
                let users = twitter::follows::Following::new(id)
                    .max_results(max_results)
                    .fetch();
                match users {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No following users found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::Followers { id, max_results } => {
                let users = twitter::follows::Followers::new(id)
                    .max_results(max_results)
                    .fetch();
                match users {
                    Ok(ok) => {
                        if ok.content.data.is_empty() {
                            println!("No followers found.");
                            return;
                        }

                        println!("{}", ok.content);
                    }
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::Follow { target_user_id } => {
                let follow = twitter::follows::CreateFollow::for_current_user(target_user_id);

                match follow {
                    Ok(follow) => match follow.send() {
                        Ok(ok) => {
                            if ok.content.data.following || ok.content.data.pending_follow {
                                println!("Follow request sent.");
                            } else {
                                eprintln!("User was not followed.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
            UsersEnum::Unfollow { target_user_id } => {
                let unfollow = twitter::follows::DeleteFollow::for_current_user(target_user_id);

                match unfollow {
                    Ok(unfollow) => match unfollow.send() {
                        Ok(ok) => {
                            if ok.content.data.following {
                                eprintln!("User is still followed.");
                            } else {
                                println!("Unfollowed user.");
                            }
                        }
                        Err(err) => eprintln!("{}", err.message),
                    },
                    Err(err) => eprintln!("{}", err.message),
                }
            }
        },
        Commands::Me {} => {
            let me_res = twitter::user::me();
            match me_res {
                Ok(ok) => println!("{}", ok.content),
                Err(err) => eprintln!("{}", err.message),
            }
        }
    }
}
