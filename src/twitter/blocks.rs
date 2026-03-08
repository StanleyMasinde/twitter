use serde::{Deserialize, Serialize};

use crate::{
    twitter::Response,
    utils::{bearer_auth_header, get_current_user_id, oauth_post_header},
};

#[derive(Debug, Deserialize)]
pub struct CreateBlockData {
    pub blocking: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateBlockResponse {
    pub data: CreateBlockData,
}

#[derive(Debug, Deserialize)]
pub struct CreateBlockError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBlockData {
    pub blocking: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBlockResponse {
    pub data: DeleteBlockData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBlockError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockedUsersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlockedUser {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockedUsersResponse {
    #[serde(default)]
    pub data: Vec<BlockedUser>,
    #[allow(dead_code)]
    pub meta: Option<BlockedUsersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct BlockedUsersError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateBlock {
    user_id: String,
    target_user_id: String,
}

#[derive(Debug)]
pub struct DeleteBlock {
    source_user_id: String,
    target_user_id: String,
}

#[derive(Debug)]
pub struct BlockedUsers {
    user_id: String,
    max_results: u8,
}

#[derive(Serialize)]
struct CreateBlockBody<'a> {
    target_user_id: &'a str,
}

impl BlockedUsers {
    pub fn current_user() -> Result<Self, BlockedUsersError> {
        let user_id = get_current_user_id().map_err(|message| BlockedUsersError { message })?;
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
        format!("https://api.x.com/2/users/{}/blocking", self.user_id)
    }

    fn authorization_header(&self) -> String {
        bearer_auth_header()
    }

    pub fn fetch(&self) -> Result<Response<BlockedUsersResponse>, BlockedUsersError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let user_fields = "name,username";
        let authorization = self.authorization_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("user.fields", user_fields)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| BlockedUsersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: BlockedUsersResponse =
                serde_json::from_slice(&response.body).map_err(|err| BlockedUsersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(BlockedUsersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateBlock {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, CreateBlockError> {
        let user_id = get_current_user_id().map_err(|message| CreateBlockError { message })?;
        Ok(Self {
            user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/blocking", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateBlockResponse>, CreateBlockError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateBlockBody {
            target_user_id: self.target_user_id.as_str(),
        })
        .map_err(|err| CreateBlockError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateBlockError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateBlockResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateBlockError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateBlockError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteBlock {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, DeleteBlockError> {
        let source_user_id =
            get_current_user_id().map_err(|message| DeleteBlockError { message })?;
        Ok(Self {
            source_user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/blocking/{}",
            self.source_user_id, self.target_user_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteBlockResponse>, DeleteBlockError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteBlockError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteBlockResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteBlockError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(DeleteBlockError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl std::fmt::Display for BlockedUsersResponse {
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
    fn test_blocked_users_url_uses_current_user_id() {
        let endpoint = BlockedUsers {
            user_id: "42".to_string(),
            max_results: 10,
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/blocking");
    }

    #[test]
    fn test_blocked_users_fetch_uses_bearer_auth_header() {
        let endpoint = BlockedUsers {
            user_id: "42".to_string(),
            max_results: 10,
        };

        assert!(endpoint.authorization_header().starts_with("Bearer "));
    }

    #[test]
    fn test_create_block_url_uses_current_user_id() {
        let endpoint = CreateBlock {
            user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/blocking");
    }

    #[test]
    fn test_delete_block_url_uses_current_user_and_target_id() {
        let endpoint = DeleteBlock {
            source_user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/blocking/99");
    }
}
