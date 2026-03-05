use std::fmt::Display;

use serde::Deserialize;

use crate::{twitter::Response, utils::oauth_get_header};

#[derive(Debug, Deserialize)]
pub struct CurrentUserData {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserResponse {
    pub data: CurrentUserData,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserError {
    pub message: String,
}

impl Display for CurrentUserResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User Id: {}\nName: {}\nUsername: @{}",
            self.data.id, self.data.name, self.data.username
        )
    }
}

pub fn me() -> Result<Response<CurrentUserResponse>, CurrentUserError> {
    let url = "https://api.x.com/2/users/me";
    let auth_header = oauth_get_header(url, &());

    let response = curl_rest::Client::default()
        .get()
        .header(curl_rest::Header::Authorization(auth_header.into()))
        .send(url)
        .map_err(|err| CurrentUserError {
            message: err.to_string(),
        })?;

    if (200..300).contains(&response.status.as_u16()) {
        let user_data: CurrentUserResponse =
            serde_json::from_slice(&response.body).map_err(|err| CurrentUserError {
                message: err.to_string(),
            })?;
        Ok(Response {
            status: response.status.as_u16(),
            content: user_data,
        })
    } else {
        Err(CurrentUserError {
            message: String::from_utf8_lossy(&response.body).to_string(),
        })
    }
}
