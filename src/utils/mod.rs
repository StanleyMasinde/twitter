use std::{
    env::{self, var},
    fs,
    path::PathBuf,
    process::{self, Command, ExitStatus},
    str::FromStr,
};

use dirs::home_dir;
use oauth::{HMAC_SHA1, Request, Token};
use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::{
    config::{Account, Config},
    schedule::Schedule,
    twitter::tweet::{Tweet, TwitterApi},
};

const CACHE_DIR: &str = "twitter-cli";
const DB_FILENAME: &str = "db.sqlite3";
const USER_CACHE_TABLE: &str = "account_user_cache";

#[derive(Deserialize)]
struct UsersMeResponse {
    data: UsersMeData,
}

#[derive(Deserialize)]
struct UsersMeData {
    id: String,
}

pub fn load_config() -> Config {
    let config_dir = home_dir()
        .expect("Home dir not found!")
        .join(".config/twitter_cli/config.toml");

    let binary_name = var("CARGO_BIN_NAME").unwrap_or("twitter".to_string());

    let data = match fs::read_to_string(config_dir) {
        Ok(data) => data,
        Err(_) => {
            let message =
                format!("Failed to read the config file.\nPlease run {binary_name} config --init");
            gracefully_exit(&message)
        }
    };
    match Config::from_str(&data) {
        Ok(cfg) => cfg,
        Err(_) => {
            gracefully_exit("Failed to load config. Try and run {binary_name} config --init");
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory missing!")
        .join(".config/twitter_cli")
}

pub fn get_config_file() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory missing!")
        .join(".config/twitter_cli/config.toml")
}

pub fn get_current_user_id() -> Result<String, String> {
    let mut cfg = load_config();
    let account_index = cfg.current_account;
    let account = cfg.current_account();
    let connection = open_cache_connection()?;

    if let Some(cached_user_id) = get_cached_user_id(&connection, account_index)? {
        return Ok(cached_user_id);
    }

    let user_id = fetch_user_id(account)?;
    save_cached_user_id(&connection, account_index, &user_id)?;
    Ok(user_id)
}

pub fn oauth_get_header<R>(url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let mut cfg = load_config();
    let account = cfg.current_account();
    oauth_get_header_for_account(account, url, request)
}

pub fn oauth_post_header<R>(url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let mut cfg = load_config();
    let account = cfg.current_account();
    oauth_post_header_for_account(account, url, request)
}

pub fn oauth_put_header<R>(url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let mut cfg = load_config();
    let account = cfg.current_account();
    oauth_put_header_for_account(account, url, request)
}

pub fn bearer_auth_header() -> String {
    let mut cfg = load_config();
    let account = cfg.current_account();
    format!("Bearer {}", account.bearer_token)
}

pub fn open_editor(file: &PathBuf) -> ExitStatus {
    #[cfg(unix)]
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    #[cfg(windows)]
    let editor = "notepad".to_string();

    Command::new(editor)
        .arg(file)
        .status()
        .expect("Failed to open the editor.")
}

pub fn check_permissions(path: &PathBuf, is_dir: bool) {
    if let Ok(metadata) = fs::metadata(path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = metadata.permissions().mode() & 0o777;
            let expected = if is_dir { 0o700 } else { 0o600 };

            if mode != expected {
                println!(
                    "⚠️  Permissions for {:?} are {:o}, expected {:o}",
                    path, mode, expected
                );
                println!("Run chmod {:o} {:?}", expected, path)
            }
        }

        #[cfg(windows)]
        {
            println!("> Windows does not support POSIX permissions. Skipping check.");
        }
    }
}

pub(crate) fn gracefully_exit(message: &str) -> ! {
    println!("{message}");
    process::exit(1)
}

pub(crate) fn send_due_tweets() {
    let schedule = Schedule::default();
    let due_tweets = schedule.due();
    if due_tweets.is_empty() {
        println!("No pending scheduled tweets to run.");
        return;
    }

    let mut sent_count = 0;
    let mut failed_count = 0;
    for (index, due_tweet) in due_tweets.iter().enumerate() {
        println!("> Sending tweet {}/{}", index + 1, due_tweets.len());
        let mut tweet = match Tweet::from_str(&due_tweet.body) {
            Ok(tweet) => tweet,
            Err(err) => {
                eprintln!(
                    "Failed to build tweet payload for schedule id {}: {}",
                    due_tweet.id, err.message
                );
                schedule.mark_failed(due_tweet.id, &err.message);
                failed_count += 1;
                continue;
            }
        };
        let api_res = tweet.create();
        match api_res {
            Ok(res) => {
                println!("{}", res.content);
                schedule.mark_sent(due_tweet.id);
                sent_count += 1;
            }
            Err(err) => {
                eprintln!("{}", err.message);
                schedule.mark_failed(due_tweet.id, &err.message);
                failed_count += 1;
            }
        }
    }

    println!(
        "Finished sending scheduled tweets. Sent: {}, Failed: {}.",
        sent_count, failed_count
    );
}

fn fetch_user_id(account: &Account) -> Result<String, String> {
    let url = "https://api.x.com/2/users/me";
    let auth_header = oauth_get_header_for_account(account, url, &());
    let response = curl_rest::Client::default()
        .get()
        .header(curl_rest::Header::Authorization(auth_header.into()))
        .send(url)
        .map_err(|err| err.to_string())?;

    if (200..300).contains(&response.status.as_u16()) {
        let user_res: UsersMeResponse = serde_json::from_slice(&response.body)
            .map_err(|err| format!("Failed to parse users/me response: {err}"))?;
        Ok(user_res.data.id)
    } else {
        Err(String::from_utf8_lossy(&response.body).to_string())
    }
}

fn oauth_get_header_for_account<R>(account: &Account, url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let token = Token::from_parts(
        account.consumer_key.as_str(),
        account.consumer_secret.as_str(),
        account.access_token.as_str(),
        account.access_secret.as_str(),
    );
    oauth::get(url, request, &token, HMAC_SHA1)
}

fn oauth_post_header_for_account<R>(account: &Account, url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let token = Token::from_parts(
        account.consumer_key.as_str(),
        account.consumer_secret.as_str(),
        account.access_token.as_str(),
        account.access_secret.as_str(),
    );
    oauth::post(url, request, &token, HMAC_SHA1)
}

fn oauth_put_header_for_account<R>(account: &Account, url: &str, request: &R) -> String
where
    R: Request + ?Sized,
{
    let token = Token::from_parts(
        account.consumer_key.as_str(),
        account.consumer_secret.as_str(),
        account.access_token.as_str(),
        account.access_secret.as_str(),
    );
    oauth::put(url, request, &token, HMAC_SHA1)
}

fn open_cache_connection() -> Result<Connection, String> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| "Failed to locate a data directory for cache.".to_string())?;
    let cli_data_dir = data_dir.join(CACHE_DIR);
    fs::create_dir_all(&cli_data_dir).map_err(|err| {
        format!(
            "Failed to create cache data directory '{}': {err}",
            cli_data_dir.display()
        )
    })?;

    let path = cli_data_dir.join(DB_FILENAME);
    let connection = Connection::open(path).map_err(|err| err.to_string())?;
    let query = format!(
        "
        CREATE TABLE IF NOT EXISTS {USER_CACHE_TABLE} (
            account_index INTEGER PRIMARY KEY,
            user_id TEXT NOT NULL,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "
    );
    connection
        .execute(&query, [])
        .map_err(|err| err.to_string())?;

    Ok(connection)
}

fn get_cached_user_id(
    connection: &Connection,
    account_index: usize,
) -> Result<Option<String>, String> {
    let query = format!("SELECT user_id FROM {USER_CACHE_TABLE} WHERE account_index = ?1");
    let mut stmt = connection.prepare(&query).map_err(|err| err.to_string())?;
    let result: Result<String, _> = stmt.query_row([account_index as i64], |row| row.get(0));
    match result {
        Ok(user_id) => Ok(Some(user_id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

fn save_cached_user_id(
    connection: &Connection,
    account_index: usize,
    user_id: &str,
) -> Result<(), String> {
    let query = format!(
        "
        INSERT INTO {USER_CACHE_TABLE} (account_index, user_id, updated_at)
        VALUES (?1, ?2, CURRENT_TIMESTAMP)
        ON CONFLICT(account_index)
        DO UPDATE SET
            user_id = excluded.user_id,
            updated_at = CURRENT_TIMESTAMP;
        "
    );
    connection
        .execute(&query, params![account_index as i64, user_id])
        .map(|_| ())
        .map_err(|err| err.to_string())
}
