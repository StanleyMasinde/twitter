use std::fmt::Display;

use crate::{
    twitter::{AUTHOR_EXPANSION, Includes, Response, TWEET_FIELDS, TweetData, USER_FIELDS},
    utils::{get_current_user_id, oauth_get_header, oauth_post_header},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LikesMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LikesResponse {
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    #[allow(dead_code)]
    pub meta: Option<LikesMeta>,
}

#[derive(Debug, Deserialize)]
pub struct LikesError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLikeData {
    pub liked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateLikeResponse {
    pub data: CreateLikeData,
}

#[derive(Debug, Deserialize)]
pub struct CreateLikeError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteLikeData {
    pub liked: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteLikeResponse {
    pub data: DeleteLikeData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteLikeError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LikingUsersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LikingUser {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LikingUsersResponse {
    #[serde(default)]
    pub data: Vec<LikingUser>,
    #[allow(dead_code)]
    pub meta: Option<LikingUsersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct LikingUsersError {
    pub message: String,
}

#[derive(Debug)]
pub struct Likes {
    user_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct LikingUsers {
    tweet_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct CreateLike {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug)]
pub struct DeleteLike {
    user_id: String,
    tweet_id: String,
}

#[derive(Serialize)]
struct CreateLikeBody<'a> {
    tweet_id: &'a str,
}

impl Likes {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(5, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/liked_tweets", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<LikesResponse>, LikesError> {
        let url = self.url();
        let max_results = self.max_results;
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
        let max_results_query = max_results.to_string();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| LikesError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let likes_data: LikesResponse =
                serde_json::from_slice(&response.body).map_err(|err| LikesError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: likes_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(LikesError { message: err_data })
        }
    }
}

impl LikingUsers {
    pub fn new(tweet_id: impl Into<String>) -> Self {
        Self {
            tweet_id: tweet_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/tweets/{}/liking_users", self.tweet_id)
    }

    pub fn fetch(&self) -> Result<Response<LikingUsersResponse>, LikingUsersError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let auth_header = oauth_get_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("user.fields", "name,username")
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| LikingUsersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: LikingUsersResponse =
                serde_json::from_slice(&response.body).map_err(|err| LikingUsersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(LikingUsersError { message: err_data })
        }
    }
}

impl CreateLike {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, CreateLikeError> {
        let user_id = get_current_user_id().map_err(|message| CreateLikeError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/likes", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateLikeResponse>, CreateLikeError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateLikeBody {
            tweet_id: self.tweet_id.as_str(),
        })
        .map_err(|err| CreateLikeError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateLikeError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateLikeResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateLikeError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(CreateLikeError { message: err_data })
        }
    }
}

impl DeleteLike {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, DeleteLikeError> {
        let user_id = get_current_user_id().map_err(|message| DeleteLikeError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/likes/{}",
            self.user_id, self.tweet_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteLikeResponse>, DeleteLikeError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteLikeError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteLikeResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteLikeError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(DeleteLikeError { message: err_data })
        }
    }
}

impl std::fmt::Display for LikingUsersResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, user) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(
                f,
                "User Id: {}\nName: {}\nUsername: @{}",
                user.id, user.name, user.username
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_like_url_uses_current_user_id() {
        let endpoint = CreateLike {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/likes");
    }

    #[test]
    fn test_delete_like_url_uses_current_user_and_tweet_id() {
        let endpoint = DeleteLike {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/likes/456");
    }

    #[test]
    fn test_liking_users_url_uses_tweet_id() {
        let endpoint = LikingUsers {
            tweet_id: "456".to_string(),
            max_results: 10,
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/tweets/456/liking_users"
        );
    }
}
