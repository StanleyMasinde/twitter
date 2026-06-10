use serde::{Deserialize, Serialize};

use crate::{
    twitter::{
        Response, USER_FIELDS,
        params::{
            Pagination, apply_query_params, oauth_param_list, paginated_oauth_entries,
            print_next_page_hint, user_field_entries,
        },
    },
    utils::{get_current_user_id, oauth_get_header, oauth_post_header},
};

#[derive(Debug, Deserialize)]
pub struct CreateMuteData {
    pub muting: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateMuteResponse {
    pub data: CreateMuteData,
}

#[derive(Debug, Deserialize)]
pub struct CreateMuteError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMuteData {
    pub muting: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMuteResponse {
    pub data: DeleteMuteData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMuteError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct MutedUsersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MutedUser {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct MutedUsersResponse {
    #[serde(default)]
    pub data: Vec<MutedUser>,
    #[allow(dead_code)]
    pub meta: Option<MutedUsersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct MutedUsersError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateMute {
    user_id: String,
    target_user_id: String,
}

#[derive(Debug)]
pub struct DeleteMute {
    source_user_id: String,
    target_user_id: String,
}

#[derive(Debug)]
pub struct MutedUsers {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Serialize)]
struct CreateMuteBody<'a> {
    target_user_id: &'a str,
}

impl MutedUsers {
    pub fn current_user() -> Result<Self, MutedUsersError> {
        let user_id = get_current_user_id().map_err(|message| MutedUsersError { message })?;
        Ok(Self {
            user_id,
            max_results: 10,
            pagination: Pagination::new(),
        })
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.pagination_token(token);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/muting", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<MutedUsersResponse>, MutedUsersError> {
        let url = self.url();
        let oauth_entries =
            paginated_oauth_entries(self.max_results, &user_field_entries(), &self.pagination);
        let auth_header = oauth_get_header(url.as_str(), &oauth_param_list(oauth_entries));

        let max_results_query = self.max_results.to_string();
        let pagination_entries = self.pagination.oauth_entries();
        let mut request = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("user.fields", USER_FIELDS);
        request = apply_query_params(request, &pagination_entries);

        let response = request
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| MutedUsersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: MutedUsersResponse =
                serde_json::from_slice(&response.body).map_err(|err| MutedUsersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(MutedUsersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateMute {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, CreateMuteError> {
        let user_id = get_current_user_id().map_err(|message| CreateMuteError { message })?;
        Ok(Self {
            user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/muting", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateMuteResponse>, CreateMuteError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateMuteBody {
            target_user_id: self.target_user_id.as_str(),
        })
        .map_err(|err| CreateMuteError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateMuteError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateMuteResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateMuteError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateMuteError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteMute {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, DeleteMuteError> {
        let source_user_id =
            get_current_user_id().map_err(|message| DeleteMuteError { message })?;
        Ok(Self {
            source_user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/muting/{}",
            self.source_user_id, self.target_user_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteMuteResponse>, DeleteMuteError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteMuteError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteMuteResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteMuteError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(DeleteMuteError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

pub fn print_muted_users(response: &MutedUsersResponse) {
    if response.data.is_empty() {
        println!("No muted users found.");
        return;
    }

    for (index, user) in response.data.iter().enumerate() {
        if index > 0 {
            println!();
            println!();
        }

        println!(
            "User Id: {}\nName: {}\nUsername: @{}",
            user.id, user.name, user.username
        );
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
}

impl std::fmt::Display for MutedUsersResponse {
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
    fn test_muted_users_url_uses_current_user_id() {
        let endpoint = MutedUsers {
            user_id: "42".to_string(),
            max_results: 10,
            pagination: Pagination::new(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/muting");
    }

    #[test]
    fn test_create_mute_url_uses_current_user_id() {
        let endpoint = CreateMute {
            user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/muting");
    }

    #[test]
    fn test_delete_mute_url_uses_current_user_and_target_id() {
        let endpoint = DeleteMute {
            source_user_id: "42".to_string(),
            target_user_id: "99".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/42/muting/99");
    }
}
