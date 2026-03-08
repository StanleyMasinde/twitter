use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    twitter::{AUTHOR_EXPANSION, Includes, Response, TWEET_FIELDS, TweetData, USER_FIELDS},
    utils::{bearer_auth_header, get_current_user_id, oauth_post_header, oauth_put_header},
};

const LIST_FIELDS: &str = "id,name,owner_id,private,description,follower_count,member_count";
const LIST_EXPANSIONS: &str = "owner_id";
const OWNER_USER_FIELDS: &str = "name,username";

#[derive(Clone, Debug, Deserialize)]
pub struct ListUser {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListIncludes {
    #[serde(default)]
    pub users: Option<Vec<ListUser>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub private: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub follower_count: Option<u64>,
    #[serde(default)]
    pub member_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembershipsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembershipsResponse {
    #[serde(default)]
    pub data: Vec<ListData>,
    #[serde(default)]
    pub includes: Option<ListIncludes>,
    #[allow(dead_code)]
    pub meta: Option<ListMembershipsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembershipsError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListLookupResponse {
    pub data: ListData,
    #[serde(default)]
    pub includes: Option<ListIncludes>,
}

#[derive(Debug, Deserialize)]
pub struct ListLookupError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateListResponse {
    pub data: ListData,
}

#[derive(Debug, Deserialize)]
pub struct CreateListError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct OwnedListsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OwnedListsResponse {
    #[serde(default)]
    pub data: Vec<ListData>,
    #[serde(default)]
    pub includes: Option<ListIncludes>,
    #[allow(dead_code)]
    pub meta: Option<OwnedListsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct OwnedListsError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTweetsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTweetsResponse {
    #[serde(default)]
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    #[allow(dead_code)]
    pub meta: Option<ListTweetsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ListTweetsError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMembersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembersResponse {
    #[serde(default)]
    pub data: Vec<ListUser>,
    #[allow(dead_code)]
    pub meta: Option<ListMembersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembersError {
    pub message: String,
}

#[derive(Debug)]
pub struct ListMemberships {
    user_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct OwnedLists {
    user_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct ListLookup {
    list_id: String,
}

#[derive(Debug)]
pub struct CreateList {
    name: String,
    description: Option<String>,
    private: Option<bool>,
}

#[derive(Debug)]
pub struct ListMembers {
    list_id: String,
    max_results: u8,
}

#[derive(Debug)]
pub struct ListTweets {
    list_id: String,
    max_results: u8,
}

#[derive(Debug, Deserialize)]
pub struct CreateListMemberData {
    pub is_member: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateListMemberResponse {
    pub data: CreateListMemberData,
}

#[derive(Debug, Deserialize)]
pub struct CreateListMemberError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateListMember {
    list_id: String,
    user_id: String,
}
#[derive(Debug, Deserialize)]
pub struct DeleteListMemberData {
    pub is_member: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListData {
    pub deleted: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListResponse {
    pub data: DeleteListData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListMemberResponse {
    pub data: DeleteListMemberData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteListMemberError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListData {
    pub updated: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListResponse {
    pub data: UpdateListData,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListError {
    pub message: String,
}

#[derive(Debug)]
pub struct DeleteListMember {
    list_id: String,
    user_id: String,
}

#[derive(Debug)]
pub struct DeleteList {
    list_id: String,
}

#[derive(Serialize)]
struct CreateListBody<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
}

#[derive(Debug)]
pub struct UpdateList {
    list_id: String,
    name: Option<String>,
    description: Option<String>,
    private: Option<bool>,
}

#[derive(Serialize)]
struct UpdateListBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
}

#[derive(Serialize)]
struct CreateListMemberBody<'a> {
    user_id: &'a str,
}
impl ListMemberships {
    pub fn current_user() -> Result<Self, ListMembershipsError> {
        let user_id = get_current_user_id().map_err(|message| ListMembershipsError { message })?;
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
        format!(
            "https://api.x.com/2/users/{}/list_memberships",
            self.user_id
        )
    }

    pub fn fetch(&self) -> Result<Response<ListMembershipsResponse>, ListMembershipsError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("list.fields", LIST_FIELDS)
            .query_param_kv("expansions", LIST_EXPANSIONS)
            .query_param_kv("user.fields", OWNER_USER_FIELDS)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| ListMembershipsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let lists_data: ListMembershipsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| ListMembershipsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: lists_data,
            })
        } else {
            Err(ListMembershipsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl OwnedLists {
    pub fn current_user() -> Result<Self, OwnedListsError> {
        let user_id = get_current_user_id().map_err(|message| OwnedListsError { message })?;
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
        format!("https://api.x.com/2/users/{}/owned_lists", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<OwnedListsResponse>, OwnedListsError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("list.fields", LIST_FIELDS)
            .query_param_kv("expansions", LIST_EXPANSIONS)
            .query_param_kv("user.fields", OWNER_USER_FIELDS)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| OwnedListsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let lists_data: OwnedListsResponse =
                serde_json::from_slice(&response.body).map_err(|err| OwnedListsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: lists_data,
            })
        } else {
            Err(OwnedListsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl ListLookup {
    pub fn new(list_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}", self.list_id)
    }

    pub fn fetch(&self) -> Result<Response<ListLookupResponse>, ListLookupError> {
        let url = self.url();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("list.fields", LIST_FIELDS)
            .query_param_kv("expansions", LIST_EXPANSIONS)
            .query_param_kv("user.fields", OWNER_USER_FIELDS)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| ListLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let list_data: ListLookupResponse =
                serde_json::from_slice(&response.body).map_err(|err| ListLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: list_data,
            })
        } else {
            Err(ListLookupError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateList {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            private: None,
        }
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn private(mut self, private: Option<bool>) -> Self {
        self.private = private;
        self
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/lists"
    }

    pub fn send(&self) -> Result<Response<CreateListResponse>, CreateListError> {
        let url = self.url();
        let auth_header = oauth_post_header(url, &());
        let body = serde_json::to_string(&CreateListBody {
            name: self.name.as_str(),
            description: self.description.as_deref(),
            private: self.private,
        })
        .map_err(|err| CreateListError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url)
            .map_err(|err| CreateListError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateListResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateListError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateListError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateListMember {
    pub fn new(list_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
            user_id: user_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}/members", self.list_id)
    }

    pub fn send(&self) -> Result<Response<CreateListMemberResponse>, CreateListMemberError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&CreateListMemberBody {
            user_id: self.user_id.as_str(),
        })
        .map_err(|err| CreateListMemberError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateListMemberError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateListMemberResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateListMemberError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateListMemberError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl ListMembers {
    pub fn new(list_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}/members", self.list_id)
    }

    pub fn fetch(&self) -> Result<Response<ListMembersResponse>, ListMembersError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("user.fields", OWNER_USER_FIELDS)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| ListMembersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let members_data: ListMembersResponse = serde_json::from_slice(&response.body)
                .map_err(|err| ListMembersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: members_data,
            })
        } else {
            Err(ListMembersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl ListTweets {
    pub fn new(list_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}/tweets", self.list_id)
    }

    pub fn fetch(&self) -> Result<Response<ListTweetsResponse>, ListTweetsError> {
        let url = self.url();
        let max_results = self.max_results.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results.as_str())
            .query_param_kv("tweet.fields", TWEET_FIELDS)
            .query_param_kv("user.fields", USER_FIELDS)
            .query_param_kv("expansions", AUTHOR_EXPANSION)
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url.as_str())
            .map_err(|err| ListTweetsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: ListTweetsResponse =
                serde_json::from_slice(&response.body).map_err(|err| ListTweetsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            Err(ListTweetsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteListMember {
    pub fn new(list_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
            user_id: user_id.into(),
        }
    }

    pub fn for_current_user(list_id: impl Into<String>) -> Result<Self, DeleteListMemberError> {
        let user_id = get_current_user_id().map_err(|message| DeleteListMemberError { message })?;
        Ok(Self::new(list_id, user_id))
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

impl UpdateList {
    pub fn new(list_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
            name: None,
            description: None,
            private: None,
        }
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn private(mut self, private: Option<bool>) -> Self {
        self.private = private;
        self
    }

    pub fn has_changes(&self) -> bool {
        self.name.is_some() || self.description.is_some() || self.private.is_some()
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}", self.list_id)
    }

    pub fn send(&self) -> Result<Response<UpdateListResponse>, UpdateListError> {
        let url = self.url();
        let auth_header = oauth_put_header(url.as_str(), &());
        let body = serde_json::to_string(&UpdateListBody {
            name: self.name.as_deref(),
            description: self.description.as_deref(),
            private: self.private,
        })
        .map_err(|err| UpdateListError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .put()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| UpdateListError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: UpdateListResponse =
                serde_json::from_slice(&response.body).map_err(|err| UpdateListError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(UpdateListError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteList {
    pub fn new(list_id: impl Into<String>) -> Self {
        Self {
            list_id: list_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/lists/{}", self.list_id)
    }

    pub fn send(&self) -> Result<Response<DeleteListResponse>, DeleteListError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| DeleteListError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: DeleteListResponse =
                serde_json::from_slice(&response.body).map_err(|err| DeleteListError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(DeleteListError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl ListMembershipsResponse {
    fn owner_for(&self, owner_id: &str) -> Option<&ListUser> {
        let users = self.includes.as_ref()?.users.as_ref()?;
        users.iter().find(|user| user.id == owner_id)
    }
}

impl Display for ListMembershipsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, list) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(f, "List Id: {}\nName: {}", list.id, list.name)?;

            if let Some(owner_id) = list.owner_id.as_deref() {
                if let Some(owner) = self.owner_for(owner_id) {
                    write!(f, "\nOwner: {} (@{})", owner.name, owner.username)?;
                } else {
                    write!(f, "\nOwner Id: {}", owner_id)?;
                }
            }

            if let Some(private) = list.private {
                write!(f, "\nPrivate: {}", private)?;
            }

            if let Some(member_count) = list.member_count {
                write!(f, "\nMembers: {}", member_count)?;
            }

            if let Some(follower_count) = list.follower_count {
                write!(f, "\nFollowers: {}", follower_count)?;
            }

            if let Some(description) = list.description.as_deref() {
                write!(f, "\nDescription: {}", description)?;
            }
        }

        Ok(())
    }
}

impl Display for CreateListResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "List Id: {}\nName: {}", self.data.id, self.data.name)?;

        if let Some(private) = self.data.private {
            write!(f, "\nPrivate: {}", private)?;
        }

        if let Some(description) = self.data.description.as_deref() {
            write!(f, "\nDescription: {}", description)?;
        }

        Ok(())
    }
}

impl Display for OwnedListsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lists = ListMembershipsResponse {
            data: self.data.clone(),
            includes: self.includes.clone(),
            meta: None,
        };
        write!(f, "{}", lists)
    }
}

impl Display for ListLookupResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response = ListMembershipsResponse {
            data: vec![self.data.clone()],
            includes: self.includes.clone(),
            meta: None,
        };
        write!(f, "{}", response)
    }
}

impl Display for ListMembersResponse {
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
    fn test_delete_list_member_url_uses_list_and_user_ids() {
        let endpoint = DeleteListMember {
            list_id: "123".to_string(),
            user_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123/members/456");
    }

    #[test]
    fn test_create_list_url_is_lists_collection() {
        let endpoint = CreateList::new("cli-builders");

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists");
    }

    #[test]
    fn test_update_list_url_uses_list_id() {
        let endpoint = UpdateList::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123");
    }

    #[test]
    fn test_list_lookup_url_uses_list_id() {
        let endpoint = ListLookup::new("123");

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123");
    }

    #[test]
    fn test_create_list_display() {
        let response = CreateListResponse {
            data: ListData {
                id: "42".to_string(),
                name: "CLI builders".to_string(),
                owner_id: None,
                private: Some(false),
                description: Some("People building CLI tools".to_string()),
                follower_count: None,
                member_count: None,
            },
        };

        assert_eq!(
            response.to_string(),
            "List Id: 42\nName: CLI builders\nPrivate: false\nDescription: People building CLI tools"
        );
    }

    #[test]
    fn test_owned_lists_url_uses_current_user_id() {
        let endpoint = OwnedLists {
            user_id: "123".to_string(),
            max_results: 10,
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/owned_lists");
    }

    #[test]
    fn test_delete_list_url_uses_list_id() {
        let endpoint = DeleteList {
            list_id: "123".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123");
    }

    #[test]
    fn test_update_list_has_changes_only_when_fields_present() {
        let unchanged = UpdateList::new("123");
        let changed = UpdateList::new("123").name(Some("renamed".to_string()));

        assert!(!unchanged.has_changes());
        assert!(changed.has_changes());
    }

    #[test]
    fn test_list_memberships_display_with_owner_details() {
        let response = ListMembershipsResponse {
            data: vec![ListData {
                id: "42".to_string(),
                name: "CLI builders".to_string(),
                owner_id: Some("7".to_string()),
                private: Some(false),
                description: Some("People building CLI tools".to_string()),
                follower_count: Some(10),
                member_count: Some(3),
            }],
            includes: Some(ListIncludes {
                users: Some(vec![ListUser {
                    id: "7".to_string(),
                    name: "Jane Doe".to_string(),
                    username: "janedoe".to_string(),
                }]),
            }),
            meta: None,
        };

        assert_eq!(
            response.to_string(),
            "List Id: 42\nName: CLI builders\nOwner: Jane Doe (@janedoe)\nPrivate: false\nMembers: 3\nFollowers: 10\nDescription: People building CLI tools"
        );
    }

    #[test]
    fn test_list_members_display() {
        let response = ListMembersResponse {
            data: vec![ListUser {
                id: "7".to_string(),
                name: "Jane Doe".to_string(),
                username: "janedoe".to_string(),
            }],
            meta: None,
        };

        assert_eq!(
            response.to_string(),
            "User Id: 7\nName: Jane Doe\nUsername: @janedoe"
        );
    }

    #[test]
    fn test_create_list_member_url_uses_list_id() {
        let endpoint = CreateListMember::new("123", "456");

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123/members");
    }

    #[test]
    fn test_list_tweets_url_uses_list_id() {
        let endpoint = ListTweets {
            list_id: "123".to_string(),
            max_results: 10,
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/lists/123/tweets");
    }
}
