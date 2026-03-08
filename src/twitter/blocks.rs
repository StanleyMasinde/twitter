use serde::{Deserialize, Serialize};

use crate::{
    twitter::Response,
    utils::{get_current_user_id, oauth_post_header},
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

#[derive(Debug)]
pub struct CreateBlock {
    user_id: String,
    target_user_id: String,
}

#[derive(Serialize)]
struct CreateBlockBody<'a> {
    target_user_id: &'a str,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_block_url_uses_current_user_id() {
        let endpoint = CreateBlock {
            user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/blocking");
    }
}
