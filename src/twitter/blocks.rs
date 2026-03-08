use serde::Deserialize;

use crate::{
    twitter::Response,
    utils::{get_current_user_id, oauth_post_header},
};

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

#[derive(Debug)]
pub struct DeleteBlock {
    source_user_id: String,
    target_user_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_block_url_uses_current_user_and_target_id() {
        let endpoint = DeleteBlock {
            source_user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/blocking/99");
    }
}
