use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

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
}
