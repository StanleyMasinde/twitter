use std::fmt::Error;
use std::str::FromStr;

use crate::twitter::{Response, TweetCreateResponse, TweetData};
use crate::utils::oauth_post_header;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTweetErr {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTweetData {
    pub deleted: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTweetResponse {
    pub data: DeleteTweetData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTweetErr {
    pub message: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct TweetBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<Reply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Media>,
}

impl FromStr for TweetBody {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            text: Some(s.to_owned()),
            reply: None,
            media: None,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub in_reply_to_tweet_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Media {
    pub media_ids: Vec<String>,
}

pub trait TwitterApi {
    fn create(&mut self) -> Result<Response<TweetCreateResponse>, CreateTweetErr>;
}

#[derive(Default)]
pub struct Tweet<'t> {
    previous_tweet: Option<String>,
    separator: &'t str,
    payload: TweetBody,
    tweet_parts: Vec<String>,
}

#[derive(Debug)]
pub struct DeleteTweet {
    tweet_id: String,
}

impl<'t> FromStr for Tweet<'t> {
    type Err = CreateTweetErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            previous_tweet: None,
            separator: "---",
            payload: TweetBody {
                text: Some(s.to_string()),
                reply: None,
                media: None,
            },
            tweet_parts: vec![],
        })
    }
}

impl<'t> Tweet<'t> {
    pub fn new(payload: TweetBody) -> Self {
        Self {
            previous_tweet: None,
            separator: "---",
            payload,
            tweet_parts: vec![],
        }
    }

    fn is_thread(&self, tweet: &str) -> bool {
        tweet.lines().any(|line| line.trim() == self.separator)
    }

    pub fn split_tweet(&self, tweet: &str, separator: &str) -> Vec<String> {
        tweet
            .lines()
            .collect::<Vec<&str>>()
            .split(|line| line.trim() == separator)
            .map(|chuck| chuck.join("\n").trim().to_string())
            .collect()
    }

    fn send(&mut self, index: Option<usize>) -> Result<TweetCreateResponse, CreateTweetErr> {
        let url = "https://api.twitter.com/2/tweets";
        let auth_header = oauth_post_header(url, &());
        let media = self.payload.media.clone();
        let mut reply = None;
        if self.previous_tweet.is_some() {
            reply = Some(Reply {
                in_reply_to_tweet_id: self.previous_tweet.clone().unwrap(),
            });
        }

        let mut tweet_text: String = self.payload.text.clone().unwrap_or_default();

        if let Some(current_index) = index {
            tweet_text = self
                .tweet_parts
                .get(current_index)
                .map_or("", |v| v)
                .to_string();
        }

        let new_tweet = TweetBody {
            text: Some(tweet_text),
            reply,
            media,
        };

        let body = serde_json::to_string(&new_tweet).map_err(|e| CreateTweetErr {
            message: e.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url)
            .map_err(|e| CreateTweetErr {
                message: e.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let res_data: TweetCreateResponse =
                serde_json::from_slice(&response.body).map_err(|_| CreateTweetErr {
                    message: "Invalid response body.".into(),
                })?;
            Ok(res_data)
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(CreateTweetErr { message: err_data })
        }
    }
}

impl DeleteTweet {
    pub fn new(tweet_id: impl Into<String>) -> Self {
        Self {
            tweet_id: tweet_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/tweets/{}", self.tweet_id)
    }

    pub fn send(&self) -> Result<Response<DeleteTweetResponse>, DeleteTweetErr> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|e| DeleteTweetErr {
                message: e.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let res_data: DeleteTweetResponse =
                serde_json::from_slice(&response.body).map_err(|e| DeleteTweetErr {
                    message: e.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: res_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(DeleteTweetErr { message: err_data })
        }
    }
}

impl<'t> TwitterApi for Tweet<'t> {
    fn create(&mut self) -> Result<Response<TweetCreateResponse>, CreateTweetErr> {
        let text = self.payload.text.clone().unwrap_or_default();
        let tweet_data = TweetData {
            text: "".to_string(),
            edit_history_tweet_ids: vec![],
            id: 0.to_string(),
            author_id: None,
            created_at: None,
        };
        let content = TweetCreateResponse {
            data: tweet_data,
            includes: None,
        };
        let mut response = Response {
            status: 200,
            content,
        };

        if self.is_thread(&text) {
            let parts = self.split_tweet(&text, self.separator);
            self.tweet_parts = parts.clone();
            let num_of_tweets = parts.len();
            for index in 0..num_of_tweets {
                // Only attach media to the first tweet
                if index > 0 {
                    self.payload.media = None
                }
                println!("> Sending tweet {}/{}", index + 1, num_of_tweets);
                let res = self.send(Some(index))?;
                let tweet_id = &res.data.id;
                self.previous_tweet = Some(tweet_id.to_string());
                response.content = res;
            }
        } else {
            let res = self.send(None)?;
            response.content = res;
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_tweet() {
        let thread = r#"This is an awesome thread.
        We be talking about things.
        ---
        This should be the second part.
        "#
        .to_string();

        let tweet = Tweet::default();

        let tweets = tweet.split_tweet(&thread, "---");

        assert_eq!(tweets.len(), 2);
    }

    #[test]
    fn test_is_thread() {
        let thread = r#"This is an awesome thread.
        We be talking about things.
        ---
        This should be the second part.
        "#
        .to_string();

        let tweet_2 = Tweet::default();

        assert!(tweet_2.is_thread(&thread));

        let tweet = "This is a normal tweet.".to_string();

        assert!(!tweet_2.is_thread(&tweet));
    }

    #[test]
    fn test_delete_tweet_url_uses_tweet_id() {
        let endpoint = DeleteTweet::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/tweets/123");
    }
}
