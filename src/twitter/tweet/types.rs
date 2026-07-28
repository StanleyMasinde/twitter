use std::{
    fmt::{Display, Error},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::twitter::{Response, TweetCreateResponse};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTweetError {
    pub detail: String,
    pub status: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

impl Display for CreateTweetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An Error occurred.\nTitle: {}\nDetail: {}",
            self.title, self.detail
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteTweetResponse {
    pub data: DeleteTweetData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTweetData {
    pub deleted: bool,
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
    fn create(&mut self) -> Result<Response<TweetCreateResponse>, CreateTweetError>;
}

#[derive(Default)]
pub struct Tweet<'t> {
    pub(crate) previous_tweet: Option<String>,
    pub(crate) separator: &'t str,
    pub(crate) payload: TweetBody,
    pub(crate) tweet_parts: Vec<String>,
}

#[derive(Debug)]
pub struct DeleteTweet {
    pub tweet_id: String,
}

impl<'t> FromStr for Tweet<'t> {
    type Err = CreateTweetError;

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
