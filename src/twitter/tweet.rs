use crate::twitter::{Response, TweetCreateResponse, TweetData};
use crate::utils::load_config;
use oauth::{HMAC_SHA1, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTweetErr {
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

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub in_reply_to_tweet_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Media {
    pub media_ids: Vec<String>,
}

pub trait TwitterApi {
    fn create(
        &mut self,
    ) -> impl Future<Output = Result<Response<TweetCreateResponse>, CreateTweetErr>>;
}

#[derive(Default)]
pub struct Tweet<'t> {
    client: reqwest::Client,
    previous_tweet: Option<String>,
    separator: &'t str,
    payload: TweetBody,
    tweet_parts: Vec<String>,
}

impl<'t> Tweet<'t> {
    pub fn new(client: reqwest::Client, payload: TweetBody) -> Self {
        Self {
            client,
            previous_tweet: None,
            separator: "---",
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
        let mut cfg = load_config();
        let current_account = cfg.current_account();
        let token = Token::from_parts(
            current_account.consumer_key.as_str(),
            current_account.consumer_secret.as_str(),
            current_account.access_token.as_str(),
            current_account.access_secret.as_str(),
        );
        let url = "https://api.twitter.com/2/tweets";
        let auth_header = oauth::post(url, &(), &token, HMAC_SHA1);
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

impl<'t> TwitterApi for Tweet<'t> {
    async fn create(&mut self) -> Result<Response<TweetCreateResponse>, CreateTweetErr> {
        let text = self.payload.text.clone().unwrap_or_default();
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
                // Only attach media to the first tweet
                if index > 0 {
                    self.payload.media = None
                }
                println!("> Sending tweet {}/{}", index + 1, num_of_tweets);
                let res = self.send(Some(index)).await?;
                let tweet_id = &res.data.id;
                self.previous_tweet = Some(tweet_id.to_string());
                response.content = res;
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

        assert!(!tweet_2.is_thread(&tweet));
    }
}
