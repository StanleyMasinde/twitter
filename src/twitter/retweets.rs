use serde::{Deserialize, Serialize};

use crate::{
    twitter::Response,
    utils::{get_current_user_id, oauth_post_header},
};

#[derive(Debug, Deserialize)]
pub struct CreateRetweetData {
    pub retweeted: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateRetweetResponse {
    pub data: CreateRetweetData,
}

#[derive(Debug, Deserialize)]
pub struct CreateRetweetError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRetweetData {
    pub retweeted: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRetweetResponse {
    pub data: DeleteRetweetData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRetweetError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateRetweet {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug)]
pub struct DeleteRetweet {
    user_id: String,
    tweet_id: String,
}

#[derive(Serialize)]
struct CreateRetweetBody<'a> {
    tweet_id: &'a str,
}

impl CreateRetweet {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, CreateRetweetError> {
        let user_id = get_current_user_id().map_err(|message| CreateRetweetError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/retweets", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateRetweetResponse>, CreateRetweetError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateRetweetBody {
            tweet_id: self.tweet_id.as_str(),
        })
        .map_err(|err| CreateRetweetError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateRetweetError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateRetweetResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateRetweetError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateRetweetError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteRetweet {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, DeleteRetweetError> {
        let user_id = get_current_user_id().map_err(|message| DeleteRetweetError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/retweets/{}",
            self.user_id, self.tweet_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteRetweetResponse>, DeleteRetweetError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteRetweetError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteRetweetResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteRetweetError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(DeleteRetweetError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_retweet_url_uses_current_user_id() {
        let endpoint = CreateRetweet {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/retweets");
    }

    #[test]
    fn test_delete_retweet_url_uses_current_user_and_tweet_id() {
        let endpoint = DeleteRetweet {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/retweets/456");
    }
}
