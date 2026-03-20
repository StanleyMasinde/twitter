use crate::constants::TOKEN_TABLE_NAME;
use jiff::Timestamp;
use oauth2::{RefreshToken, TokenResponse};
use rusqlite::{Connection, params};
use std::{
    io,
    ops::Add,
    process,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, CurlHttpClient,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl, basic::BasicClient, url::Url,
};

use crate::{
    constants::{CACHE_DIR, DB_FILENAME},
    utils::{gracefully_exit, load_config},
};

pub struct TokenManager {
    connection: Connection,
}

struct TokenRecord {
    access_token: String,
    refresh_token: String,
    expires_at: String,
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenManager {
    pub fn new() -> Self {
        let connection = open_connection();
        Self { connection }
    }

    pub fn get_token(&self) -> String {
        let mut cfg = load_config();
        let account_id: u32 = cfg.current_account as u32;
        let current_account = cfg.current_account();
        let exists_query = format!(
            "SELECT * from {} WHERE account_id = ? LIMIT 1",
            TOKEN_TABLE_NAME
        );
        let client = BasicClient::new(ClientId::new(current_account.client_id.clone()))
            .set_client_secret(ClientSecret::new(current_account.client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://x.com/i/oauth2/authorize".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:3000".to_string()).unwrap());

        let token_exists = self
            .connection
            .query_one(&exists_query, params![account_id], |row| {
                Ok(TokenRecord {
                    access_token: row.get(2).unwrap(),
                    refresh_token: row.get(3).unwrap(),
                    expires_at: row.get(5).unwrap(),
                })
            });

        if let Ok(current_token) = token_exists {
            // check if the token has expired
            let expiry_time: Timestamp = current_token.expires_at.parse().unwrap();
            let now = Timestamp::now();
            if now > expiry_time {
                let token = client
                    .exchange_refresh_token(&RefreshToken::new(current_token.refresh_token))
                    .request(&CurlHttpClient)
                    .unwrap();

                let token_string = token.access_token().secret();

                let update_token_query =
                    format!("UPDATE {TOKEN_TABLE_NAME} SET access_token = ? WHERE account_id = ?");
                self.connection
                    .execute(&update_token_query, params![token_string, account_id])
                    .unwrap();

                return token_string.to_string();
            }
            return current_token.access_token;
        }

        let client = BasicClient::new(ClientId::new(current_account.client_id.clone()))
            .set_client_secret(ClientSecret::new(current_account.client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://x.com/i/oauth2/authorize".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:3000".to_string()).unwrap());

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("bookmark.read".to_string()))
            .add_scope(Scope::new("tweet.read".to_string()))
            .add_scope(Scope::new("users.read".to_string()))
            .add_scope(Scope::new("offline.access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("Open this URL in a browser:");
        println!("{auth_url}");
        println!("Paste the full callback URL:");
        let mut callback_url = String::new();
        io::stdin().read_line(&mut callback_url).unwrap();

        let parsed = Url::parse(callback_url.trim()).unwrap();

        let returned_state = parsed
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.into_owned())
            .ok_or("missing state")
            .unwrap();

        if returned_state != *csrf_token.secret() {
            println!("CSRF token mismatch");
            process::exit(1)
        }

        let code = parsed
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.into_owned())
            .ok_or("missing code")
            .unwrap();

        let client = BasicClient::new(ClientId::new(current_account.client_id.clone()))
            .set_client_secret(ClientSecret::new(current_account.client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://x.com/i/oauth2/authorize".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://api.x.com/2/oauth2/token".to_string()).unwrap())
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:3000".to_string()).unwrap());

        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request(&CurlHttpClient)
            .unwrap();

        // Save this
        let insert_query = format!(
            "
            INSERT INTO {TOKEN_TABLE_NAME}
            (account_id, access_token, refresh_token, expires_at)
            VALUES (?, ?, ?, ?)
            "
        );

        let seconds = token.expires_in().map(|t| t.as_secs()).unwrap_or(0);
        let expiry_seconds_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .add(Duration::from_secs(seconds))
            .as_secs() as i64;

        let token_expiry_time = Timestamp::from_second(expiry_seconds_since_epoch)
            .unwrap()
            .to_string();
        let db_res = self.connection.execute(
            &insert_query,
            params![
                account_id,
                token.access_token().secret(),
                token.refresh_token().unwrap().secret(),
                token_expiry_time,
            ],
        );

        if let Err(err) = db_res {
            eprint!("Failed to save the token: {}", err);
            process::exit(1)
        }

        token.access_token().secret().to_owned()
    }
}

fn open_connection() -> Connection {
    let data_dir = match dirs::data_dir() {
        Some(path) => path,
        None => gracefully_exit("Failed to locate a data directory for scheduled tweets."),
    };

    let cli_data_dir = data_dir.join(CACHE_DIR);
    if let Err(err) = std::fs::create_dir_all(&cli_data_dir) {
        gracefully_exit(&format!(
            "Failed to create schedule data directory '{}': {err}",
            cli_data_dir.display()
        ));
    }

    let path = cli_data_dir.join(DB_FILENAME);
    let connection = match Connection::open(path) {
        Ok(connection) => connection,
        Err(err) => gracefully_exit(&format!("Failed to open schedule database: {err}")),
    };

    if let Err(err) = connection.execute(
        &format!(
            "
                CREATE TABLE IF NOT EXISTS {TOKEN_TABLE_NAME} (
                id INTEGER PRIMARY KEY,
                account_id INTEGER UNIQUE,

                access_token TEXT NOT NULL,
                refresh_token TEXT,
                token_type TEXT NOT NULL DEFAULT 'Bearer',

                expires_at DATETIME,

                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            ",
        ),
        [],
    ) {
        gracefully_exit(&format!(
            "Failed to initialize schedule database schema: {err}"
        ));
    }

    connection
}
