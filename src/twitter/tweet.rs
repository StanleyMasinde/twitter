use std::fmt::format;

use crate::{
    api::client::{Response, TweetCreateResponse, TweetData},
    config::Config,
};
use oauth::{HMAC_SHA1, Token};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTweetErr {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct TweetBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<Reply>,
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub in_reply_to_tweet_id: String,
}

pub trait TwitterApi {
    async fn create(&mut self) -> Result<Response, CreateTweetErr>;
}

pub struct Tweet {
    client: reqwest::Client,
    previous_tweet: Option<String>,
    separator: String,
    payload: TweetBody,
    tweet_parts: Vec<String>,
}

impl Default for Tweet {
    fn default() -> Self {
        let payload = TweetBody {
            text: Some(String::new()),
            reply: None,
        };

        Self {
            client: Default::default(),
            previous_tweet: None,
            separator: Default::default(),
            payload,
            tweet_parts: Default::default(),
        }
    }
}

impl Tweet {
    pub fn new(client: reqwest::Client, payload: TweetBody) -> Self {
        Self {
            client,
            previous_tweet: None,
            separator: "---".to_string(),
            payload,
            tweet_parts: vec![],
        }
    }

    fn is_thread(&self, tweet: &str) -> bool {
        tweet.lines().any(|line| line.trim() == self.separator)
    }

    fn split_tweet(&self, tweet: &str, separator: &str) -> Vec<String> {
        tweet
            .lines()
            .collect::<Vec<&str>>()
            .split(|line| line.trim() == separator)
            .map(|chuck| chuck.join("\n").trim().to_string())
            .collect()
    }

    async fn send(&mut self, index: Option<usize>) -> Result<TweetCreateResponse, CreateTweetErr> {
        let cfg = Config::load();
        let token = Token::from_parts(
            cfg.consumer_key,
            cfg.consumer_secret,
            cfg.access_token,
            cfg.access_secret,
        );
        let url = "https://api.twitter.com/2/tweets";
        let auth_header = oauth::post(url, &(), &token, HMAC_SHA1);
        let mut reply = None;
        if self.previous_tweet.is_some() {
            reply = Some(Reply {
                in_reply_to_tweet_id: self.previous_tweet.clone().unwrap(),
            });
        }

        let mut tweet_text: String = self.payload.text.clone().unwrap();

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
        };

        let response = self
            .client
            .post(url)
            .header(reqwest::header::AUTHORIZATION, &auth_header)
            .json(&new_tweet)
            .send()
            .await
            .map_err(|e| CreateTweetErr {
                message: e.to_string(),
            })?;
        let status = response.status();

        if status.is_success() {
            let bytes = response.bytes().await.map_err(|err| CreateTweetErr {
                message: err.to_string(),
            })?;
            let res_data: TweetCreateResponse =
                serde_json::from_slice(&bytes).map_err(|_| CreateTweetErr {
                    message: "Invalid response body.".into(),
                })?;
            Ok(res_data)
        } else {
            let err_data = response.text().await.map_err(|e| CreateTweetErr {
                message: format!("{:?}", e),
            })?;
            Err(CreateTweetErr { message: err_data })
        }
    }
}

impl TwitterApi for Tweet {
    async fn create(&mut self) -> Result<Response, CreateTweetErr> {
        let text = self.payload.text.clone().unwrap();
        let tweet_data = TweetData {
            text: "".to_string(),
            edit_history_tweet_ids: vec![],
            id: 0.to_string(),
        };
        let content = TweetCreateResponse { data: tweet_data };
        let mut response = Response {
            status: 200,
            content,
        };

        if self.is_thread(&text) {
            let parts = self.split_tweet(&text, &self.separator);
            self.tweet_parts = parts.clone();
            let num_of_tweets = parts.len();
            for index in 0..num_of_tweets {
                println!("Sending tweet {}/{}", index + 1, num_of_tweets);
                let res = self.send(Some(index)).await?;
                let tweet_id = &res.data.id;
                self.previous_tweet = Some(tweet_id.to_string());
                response.content = res
            }
        } else {
            let res = self.send(None).await?;
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

        assert!(!tweet_2.is_thread(&tweet))
    }
}
