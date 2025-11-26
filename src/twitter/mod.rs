use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod media;
pub mod tweet;

pub struct Response<T> {
    pub _status: u16,
    pub content: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetCreateResponse {
    pub data: TweetData,
}

impl Display for TweetCreateResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tweet Id: {}\nTweet body: {}",
            self.data.id, self.data.text
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetData {
    pub text: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub id: String,
}
