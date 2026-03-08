use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub(crate) mod likes;
pub(crate) mod lists;
pub mod media;
pub(crate) mod mentions;
pub(crate) mod retweets;
pub(crate) mod timeline;
pub mod tweet;
pub(crate) mod tweets;
pub mod user;

const TWEET_FIELDS: &str = "author_id,created_at";
const USER_FIELDS: &str = "name,username";
const AUTHOR_EXPANSION: &str = "author_id";

pub struct Response<T> {
    #[allow(dead_code)]
    pub status: u16,
    pub content: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweetCreateResponse {
    pub data: TweetData,
    #[serde(default)]
    pub includes: Option<Includes>,
}

impl Display for TweetCreateResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tweet Id: {}", self.data.id)?;

        if let Some(author) = self.author() {
            write!(f, "\nUser: {} (@{})", author.name, author.username)?;
        } else if let Some(author_id) = &self.data.author_id {
            write!(f, "\nAuthor Id: {}", author_id)?;
        }

        if let Some(created_at) = &self.data.created_at {
            write!(f, "\nCreated at: {}", created_at)?;
        }

        write!(f, "\nTweet body: {}", self.data.text)
    }
}

impl TweetCreateResponse {
    fn author(&self) -> Option<&UserData> {
        let author_id = self.data.author_id.as_deref()?;
        let users = self.includes.as_ref()?.users.as_ref()?;
        users.iter().find(|user| user.id == author_id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweetData {
    pub text: String,
    #[serde(default)]
    pub edit_history_tweet_ids: Vec<String>,
    pub id: String,
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Includes {
    #[serde(default)]
    pub users: Option<Vec<UserData>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tweet_display_with_author_details() {
        let response = TweetCreateResponse {
            data: TweetData {
                text: "Hello, world".to_string(),
                edit_history_tweet_ids: vec!["1".to_string()],
                id: "1".to_string(),
                author_id: Some("42".to_string()),
                created_at: Some("2026-03-06T10:00:00.000Z".to_string()),
            },
            includes: Some(Includes {
                users: Some(vec![UserData {
                    id: "42".to_string(),
                    name: "Jane Doe".to_string(),
                    username: "janedoe".to_string(),
                }]),
            }),
        };

        assert_eq!(
            response.to_string(),
            "Tweet Id: 1\nUser: Jane Doe (@janedoe)\nCreated at: 2026-03-06T10:00:00.000Z\nTweet body: Hello, world"
        );
    }

    #[test]
    fn test_tweet_display_with_author_id_fallback() {
        let response = TweetCreateResponse {
            data: TweetData {
                text: "Hello, world".to_string(),
                edit_history_tweet_ids: vec!["1".to_string()],
                id: "1".to_string(),
                author_id: Some("42".to_string()),
                created_at: None,
            },
            includes: None,
        };

        assert_eq!(
            response.to_string(),
            "Tweet Id: 1\nAuthor Id: 42\nTweet body: Hello, world"
        );
    }
}
