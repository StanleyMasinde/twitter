use serde::Deserialize;

use crate::{
    twitter::Response,
    utils::{get_current_user_id, oauth_post_header},
};

#[derive(Debug, Deserialize)]
pub struct DeleteListMemberData {
    pub is_member: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListMemberResponse {
    pub data: DeleteListMemberData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListMemberError {
    pub message: String,
}

#[derive(Debug)]
pub struct DeleteListMember {
    list_id: String,
    user_id: String,
}

impl DeleteListMember {
    pub fn for_current_user(list_id: impl Into<String>) -> Result<Self, DeleteListMemberError> {
        let user_id = get_current_user_id().map_err(|message| DeleteListMemberError { message })?;
        Ok(Self {
            list_id: list_id.into(),
            user_id,
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/lists/{}/members/{}",
            self.list_id, self.user_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteListMemberResponse>, DeleteListMemberError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteListMemberError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteListMemberResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteListMemberError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(DeleteListMemberError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_list_member_url_uses_list_and_user_ids() {
        let endpoint = DeleteListMember {
            list_id: "123".to_string(),
            user_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123/members/456");
    }
}
