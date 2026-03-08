use std::fmt::Display;

use serde::Deserialize;

use crate::{twitter::Response, utils::oauth_get_header};

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserResponse {
    pub data: UserData,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserError {
    pub message: String,
}

#[derive(Debug)]
pub struct UserLookup {
    user_id: String,
}

#[derive(Debug)]
pub struct UsersLookup {
    user_ids: Vec<String>,
}

#[derive(Debug)]
pub struct UserLookupByUsername {
    username: String,
}

#[derive(Debug)]
pub struct UsersLookupByUsernames {
    usernames: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserLookupResponse {
    pub data: UserData,
}

#[derive(Debug, Deserialize)]
pub struct UserLookupError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UsersLookupResponse {
    #[serde(default)]
    pub data: Vec<UserData>,
}

#[derive(Debug, Deserialize)]
pub struct UsersLookupError {
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

impl Display for UserLookupResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User Id: {}\nName: {}\nUsername: @{}",
            self.data.id, self.data.name, self.data.username
        )
    }
}

impl Display for UsersLookupResponse {
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

impl UserLookup {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<UserLookupResponse>, UserLookupError> {
        let url = self.url();
        let auth_header = oauth_get_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .get()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| UserLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: UserLookupResponse =
                serde_json::from_slice(&response.body).map_err(|err| UserLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(UserLookupError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl UsersLookup {
    pub fn new(user_ids: Vec<String>) -> Self {
        Self { user_ids }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/users"
    }

    pub fn fetch(&self) -> Result<Response<UsersLookupResponse>, UsersLookupError> {
        let url = self.url();
        let ids = self.user_ids.join(",");
        let auth_params = oauth::ParameterList::new([("ids", &ids as &dyn Display)]);
        let auth_header = oauth_get_header(url, &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("ids", ids.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url)
            .map_err(|err| UsersLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: UsersLookupResponse =
                serde_json::from_slice(&response.body).map_err(|err| UsersLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(UsersLookupError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl UserLookupByUsername {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/by/username/{}", self.username)
    }

    pub fn fetch(&self) -> Result<Response<UserLookupResponse>, UserLookupError> {
        let url = self.url();
        let auth_header = oauth_get_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .get()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| UserLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: UserLookupResponse =
                serde_json::from_slice(&response.body).map_err(|err| UserLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(UserLookupError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl UsersLookupByUsernames {
    pub fn new(usernames: Vec<String>) -> Self {
        Self { usernames }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/users/by"
    }

    pub fn fetch(&self) -> Result<Response<UsersLookupResponse>, UsersLookupError> {
        let url = self.url();
        let usernames = self.usernames.join(",");
        let auth_params = oauth::ParameterList::new([("usernames", &usernames as &dyn Display)]);
        let auth_header = oauth_get_header(url, &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("usernames", usernames.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url)
            .map_err(|err| UsersLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let user_data: UsersLookupResponse =
                serde_json::from_slice(&response.body).map_err(|err| UsersLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: user_data,
            })
        } else {
            Err(UsersLookupError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_lookup_url_uses_user_id() {
        let endpoint = UserLookup::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123");
    }

    #[test]
    fn test_users_lookup_url_uses_collection_endpoint() {
        let endpoint = UsersLookup::new(vec!["123".to_string(), "456".to_string()]);

        assert_eq!(endpoint.url(), "https://api.x.com/2/users");
    }

    #[test]
    fn test_users_lookup_display_renders_multiple_users() {
        let response = UsersLookupResponse {
            data: vec![
                UserData {
                    id: "123".to_string(),
                    name: "Jane Doe".to_string(),
                    username: "janedoe".to_string(),
                },
                UserData {
                    id: "456".to_string(),
                    name: "John Doe".to_string(),
                    username: "johndoe".to_string(),
                },
            ],
        };

        assert_eq!(
            response.to_string(),
            "User Id: 123\nName: Jane Doe\nUsername: @janedoe\n\nUser Id: 456\nName: John Doe\nUsername: @johndoe"
        );
    }

    #[test]
    fn test_user_lookup_by_username_url_uses_username() {
        let endpoint = UserLookupByUsername::new("janedoe");

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/by/username/janedoe"
        );
    }

    #[test]
    fn test_users_lookup_by_usernames_url_uses_collection_endpoint() {
        let endpoint =
            UsersLookupByUsernames::new(vec!["janedoe".to_string(), "johndoe".to_string()]);

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/by");
    }
}
