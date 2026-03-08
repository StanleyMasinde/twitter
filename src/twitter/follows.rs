use crate::{
    twitter::{Response, UserData},
    utils::oauth_get_header,
};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct FollowingMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FollowingResponse {
    #[serde(default)]
    pub data: Vec<UserData>,
    #[allow(dead_code)]
    pub meta: Option<FollowingMeta>,
}

#[derive(Debug, Deserialize)]
pub struct FollowingError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct FollowersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FollowersResponse {
    #[serde(default)]
    pub data: Vec<UserData>,
    #[allow(dead_code)]
    pub meta: Option<FollowersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct FollowersError {
    pub message: String,
}

#[derive(Debug)]
pub struct Following {
    user_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct Followers {
    user_id: String,
    max_results: u8,
}

impl Display for FollowingResponse {
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

impl Following {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/following", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<FollowingResponse>, FollowingError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let user_fields = "name,username".to_string();
        let auth_params = oauth::ParameterList::new([
            ("max_results", &max_results as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
        ]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| FollowingError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: FollowingResponse =
                serde_json::from_slice(&response.body).map_err(|err| FollowingError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(FollowingError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl Display for FollowersResponse {
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

impl Followers {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/followers", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<FollowersResponse>, FollowersError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let user_fields = "name,username".to_string();
        let auth_params = oauth::ParameterList::new([
            ("max_results", &max_results as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
        ]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| FollowersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: FollowersResponse =
                serde_json::from_slice(&response.body).map_err(|err| FollowersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(FollowersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_following_url_uses_user_id() {
        let endpoint = Following::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/following");
    }

    #[test]
    fn test_followers_url_uses_user_id() {
        let endpoint = Followers::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/followers");
    }
}
