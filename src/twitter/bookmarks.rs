use std::fmt::Display;
use std::io;

use oauth2::AuthUrl;
use oauth2::AuthorizationCode;
use oauth2::ClientId;
use oauth2::ClientSecret;
use oauth2::CsrfToken;
use oauth2::CurlHttpClient;
use oauth2::PkceCodeChallenge;
use oauth2::RedirectUrl;
use oauth2::Scope;
use oauth2::TokenUrl;
use oauth2::basic::BasicClient;
use oauth2::url::Url;
use serde::Deserialize;
use serde::Serialize;

use oauth2::TokenResponse;

use crate::utils::load_config;
use crate::{
    twitter::{AUTHOR_EXPANSION, Includes, Response, TWEET_FIELDS, TweetData, USER_FIELDS},
    utils::{get_current_user_id, oauth_get_header, oauth_post_header},
};

#[derive(Debug, Deserialize)]
pub struct BookmarksMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarksResponse {
    #[serde(default)]
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    #[allow(dead_code)]
    pub meta: Option<BookmarksMeta>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarksError {
    pub message: String,
}

#[derive(Debug)]
pub struct Bookmarks {
    user_id: String,
    max_results: u8,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkData {
    pub bookmarked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkResponse {
    pub data: CreateBookmarkData,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateBookmark {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkData {
    pub bookmarked: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkResponse {
    pub data: DeleteBookmarkData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkError {
    pub message: String,
}

#[derive(Debug)]
pub struct DeleteBookmark {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFolder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tweet_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersResponse {
    #[serde(default)]
    pub data: Vec<BookmarkFolder>,
    #[allow(dead_code)]
    pub meta: Option<BookmarkFoldersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersError {
    pub message: String,
}

#[derive(Debug)]
pub struct BookmarkFolders {
    user_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct BookmarkFolderTweets {
    user_id: String,
    folder_id: String,
    max_results: u8,
}

#[derive(Serialize)]
struct CreateBookmarkBody<'a> {
    tweet_id: &'a str,
}

impl Bookmarks {
    pub fn current_user() -> Result<Self, BookmarksError> {
        let user_id = get_current_user_id().map_err(|message| BookmarksError { message })?;
        Ok(Self {
            user_id,
            max_results: 10,
        })
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/bookmarks", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<BookmarksResponse>, BookmarksError> {
        let url = self.url();
        let max_results = self.max_results;
        let max_results_query = max_results.to_string();
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let auth_params = oauth::ParameterList::new([
            ("max_results", &max_results as &dyn Display),
            ("tweet.fields", &tweet_fields as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
            ("expansions", &expansions as &dyn Display),
        ]);

        let mut cfg = load_config();
        let current_account = cfg.current_account();
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

        // if returned_state != *csrf_token.secret() {
        // return Err("state mismatch");
        // }

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

        let access_token = token.access_token().secret();
        println!("{access_token}");

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .send(url.as_str())
            .map_err(|err| BookmarksError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmarks_data: BookmarksResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarksError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmarks_data,
            })
        } else {
            Err(BookmarksError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateBookmark {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, CreateBookmarkError> {
        let user_id = get_current_user_id().map_err(|message| CreateBookmarkError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/bookmarks", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateBookmarkResponse>, CreateBookmarkError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateBookmarkBody {
            tweet_id: self.tweet_id.as_str(),
        })
        .map_err(|err| CreateBookmarkError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateBookmarkError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmark_data: CreateBookmarkResponse = serde_json::from_slice(&response.body)
                .map_err(|err| CreateBookmarkError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmark_data,
            })
        } else {
            Err(CreateBookmarkError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteBookmark {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, DeleteBookmarkError> {
        let user_id = get_current_user_id().map_err(|message| DeleteBookmarkError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/bookmarks/{}",
            self.user_id, self.tweet_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteBookmarkResponse>, DeleteBookmarkError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteBookmarkError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmark_data: DeleteBookmarkResponse = serde_json::from_slice(&response.body)
                .map_err(|err| DeleteBookmarkError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmark_data,
            })
        } else {
            Err(DeleteBookmarkError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl Display for BookmarkFoldersResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, folder) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(f, "Folder Id: {}\nName: {}", folder.id, folder.name)?;
            if let Some(tweet_count) = folder.tweet_count {
                write!(f, "\nTweet count: {}", tweet_count)?;
            }
        }

        Ok(())
    }
}

impl BookmarkFolders {
    pub fn current_user() -> Result<Self, BookmarkFoldersError> {
        let user_id = get_current_user_id().map_err(|message| BookmarkFoldersError { message })?;
        Ok(Self {
            user_id,
            max_results: 10,
        })
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/bookmarks/folders",
            self.user_id
        )
    }

    pub fn fetch(&self) -> Result<Response<BookmarkFoldersResponse>, BookmarkFoldersError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let auth_params =
            oauth::ParameterList::new([("max_results", &max_results as &dyn Display)]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| BookmarkFoldersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let folder_data: BookmarkFoldersResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarkFoldersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: folder_data,
            })
        } else {
            Err(BookmarkFoldersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl BookmarkFolderTweets {
    pub fn current_user(folder_id: impl Into<String>) -> Result<Self, BookmarksError> {
        let user_id = get_current_user_id().map_err(|message| BookmarksError { message })?;
        Ok(Self {
            user_id,
            folder_id: folder_id.into(),
            max_results: 10,
        })
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/bookmarks/folders/{}",
            self.user_id, self.folder_id
        )
    }

    pub fn fetch(&self) -> Result<Response<BookmarksResponse>, BookmarksError> {
        let url = self.url();
        let max_results = self.max_results;
        let max_results_query = max_results.to_string();
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let auth_params = oauth::ParameterList::new([
            ("max_results", &max_results as &dyn Display),
            ("tweet.fields", &tweet_fields as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
            ("expansions", &expansions as &dyn Display),
        ]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| BookmarksError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmarks_data: BookmarksResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarksError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmarks_data,
            })
        } else {
            Err(BookmarksError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmarks_url_uses_current_user_id() {
        let endpoint = Bookmarks {
            user_id: "123".to_string(),
            max_results: 10,
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/bookmarks");
    }

    #[test]
    fn test_create_bookmark_url_uses_current_user_id() {
        let endpoint = CreateBookmark {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/bookmarks");
    }

    #[test]
    fn test_delete_bookmark_url_uses_current_user_and_tweet_id() {
        let endpoint = DeleteBookmark {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/456"
        );
    }

    #[test]
    fn test_bookmark_folders_url_uses_current_user_id() {
        let endpoint = BookmarkFolders {
            user_id: "123".to_string(),
            max_results: 10,
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/folders"
        );
    }

    #[test]
    fn test_bookmark_folder_tweets_url_uses_current_user_and_folder_id() {
        let endpoint = BookmarkFolderTweets {
            user_id: "123".to_string(),
            folder_id: "456".to_string(),
            max_results: 10,
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/folders/456"
        );
    }
}
