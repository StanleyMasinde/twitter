use crate::{
    twitter::{
        Response, USER_FIELDS, UserData,
        params::{
            Pagination, apply_query_params, oauth_param_list, paginated_oauth_entries,
            print_next_page_hint, user_field_entries,
        },
    },
    utils::{get_current_user_id, oauth_get_header, oauth_post_header},
};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct FollowingMeta {
    #[allow(dead_code)]
    pub result_count: u32,
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
    pagination: Pagination,
}

#[derive(Debug)]
pub struct Followers {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowData {
    pub following: bool,
    pub pending_follow: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowResponse {
    pub data: CreateFollowData,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateFollow {
    user_id: String,
    target_user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFollowData {
    pub following: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFollowResponse {
    pub data: DeleteFollowData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFollowError {
    pub message: String,
}

#[derive(Debug)]
pub struct DeleteFollow {
    source_user_id: String,
    target_user_id: String,
}

#[derive(Serialize)]
struct CreateFollowBody<'a> {
    target_user_id: &'a str,
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
            pagination: Pagination::new(),
        }
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
        format!("https://api.x.com/2/users/{}/following", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<FollowingResponse>, FollowingError> {
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
            pagination: Pagination::new(),
        }
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
        format!("https://api.x.com/2/users/{}/followers", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<FollowersResponse>, FollowersError> {
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

impl CreateFollow {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, CreateFollowError> {
        let user_id = get_current_user_id().map_err(|message| CreateFollowError { message })?;
        Ok(Self {
            user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/following", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateFollowResponse>, CreateFollowError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateFollowBody {
            target_user_id: self.target_user_id.as_str(),
        })
        .map_err(|err| CreateFollowError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateFollowError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let follow_data: CreateFollowResponse = serde_json::from_slice(&response.body)
                .map_err(|err| CreateFollowError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: follow_data,
            })
        } else {
            Err(CreateFollowError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteFollow {
    pub fn for_current_user(target_user_id: impl Into<String>) -> Result<Self, DeleteFollowError> {
        let source_user_id =
            get_current_user_id().map_err(|message| DeleteFollowError { message })?;
        Ok(Self {
            source_user_id,
            target_user_id: target_user_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/following/{}",
            self.source_user_id, self.target_user_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteFollowResponse>, DeleteFollowError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteFollowError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let follow_data: DeleteFollowResponse = serde_json::from_slice(&response.body)
                .map_err(|err| DeleteFollowError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: follow_data,
            })
        } else {
            Err(DeleteFollowError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

pub fn print_following(response: &FollowingResponse) {
    if response.data.is_empty() {
        println!("No following users found.");
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

pub fn print_followers(response: &FollowersResponse) {
    if response.data.is_empty() {
        println!("No followers found.");
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

    #[test]
    fn test_create_follow_url_uses_current_user_id() {
        let endpoint = CreateFollow {
            user_id: "123".to_string(),
            target_user_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/following");
    }

    #[test]
    fn test_delete_follow_url_uses_current_user_and_target_id() {
        let endpoint = DeleteFollow {
            source_user_id: "123".to_string(),
            target_user_id: "456".to_string(),
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/following/456"
        );
    }
}
