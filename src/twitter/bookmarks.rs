use serde::Deserialize;
use serde::Serialize;

use crate::auth::oauth2::TokenManager;
use crate::{
    twitter::{
        Includes, Response, TweetData,
        params::{
            Pagination, apply_query_params, collect_oauth_entries, max_results_entry,
            print_next_page_hint, tweet_field_entries,
        },
    },
    utils::get_current_user_id,
};

#[derive(Debug, Deserialize)]
pub struct BookmarksMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarksResponse {
    #[serde(default)]
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    #[allow(dead_code)]
    pub meta: Option<BookmarksMeta>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarksError {
    pub message: String,
}

#[derive(Debug)]
pub struct Bookmarks {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkData {
    pub bookmarked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkResponse {
    pub data: CreateBookmarkData,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkError {
    pub message: String,
}

#[derive(Debug)]
pub struct CreateBookmark {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkData {
    pub bookmarked: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkResponse {
    pub data: DeleteBookmarkData,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkError {
    pub message: String,
}

#[derive(Debug)]
pub struct DeleteBookmark {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFolder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tweet_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersResponse {
    #[serde(default)]
    pub data: Vec<BookmarkFolder>,
    #[allow(dead_code)]
    pub meta: Option<BookmarkFoldersMeta>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkFoldersError {
    pub message: String,
}

#[derive(Debug)]
pub struct BookmarkFolders {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug)]
pub struct BookmarkFolderTweets {
    user_id: String,
    folder_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Serialize)]
struct CreateBookmarkBody<'a> {
    tweet_id: &'a str,
}

impl Bookmarks {
    pub fn current_user() -> Result<Self, BookmarksError> {
        let user_id = get_current_user_id().map_err(|message| BookmarksError { message })?;
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
        format!("https://api.x.com/2/users/{}/bookmarks", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<BookmarksResponse>, BookmarksError> {
        let url = self.url();
        let query_entries = collect_oauth_entries(
            vec![max_results_entry(self.max_results)],
            &tweet_field_entries(),
        );
        let query_entries = collect_oauth_entries(query_entries, &self.pagination.oauth_entries());

        let token_manager = TokenManager::new();
        let access_token = token_manager.get_token();

        let mut request = curl_rest::Client::default().get();
        request = apply_query_params(request, &query_entries);

        let response = request
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .send(url.as_str())
            .map_err(|err| BookmarksError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmarks_data: BookmarksResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarksError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmarks_data,
            })
        } else {
            Err(BookmarksError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateBookmark {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, CreateBookmarkError> {
        let user_id = get_current_user_id().map_err(|message| CreateBookmarkError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/bookmarks", self.user_id)
    }

    pub fn send(&self) -> Result<Response<CreateBookmarkResponse>, CreateBookmarkError> {
        let url = self.url();
        let body = serde_json::to_string(&CreateBookmarkBody {
            tweet_id: self.tweet_id.as_str(),
        })
        .map_err(|err| CreateBookmarkError {
            message: err.to_string(),
        })?;

        let token_manager = TokenManager::new();
        let access_token = token_manager.get_token();

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| CreateBookmarkError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmark_data: CreateBookmarkResponse = serde_json::from_slice(&response.body)
                .map_err(|err| CreateBookmarkError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmark_data,
            })
        } else {
            Err(CreateBookmarkError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteBookmark {
    pub fn for_current_user(tweet_id: impl Into<String>) -> Result<Self, DeleteBookmarkError> {
        let user_id = get_current_user_id().map_err(|message| DeleteBookmarkError { message })?;
        Ok(Self {
            user_id,
            tweet_id: tweet_id.into(),
        })
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/bookmarks/{}",
            self.user_id, self.tweet_id
        )
    }

    pub fn send(&self) -> Result<Response<DeleteBookmarkResponse>, DeleteBookmarkError> {
        let url = self.url();
        let token_manager = TokenManager::new();
        let access_token = token_manager.get_token();

        let response = curl_rest::Client::default()
            .delete()
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .send(url.as_str())
            .map_err(|err| DeleteBookmarkError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmark_data: DeleteBookmarkResponse = serde_json::from_slice(&response.body)
                .map_err(|err| DeleteBookmarkError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmark_data,
            })
        } else {
            Err(DeleteBookmarkError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl BookmarkFolders {
    pub fn current_user() -> Result<Self, BookmarkFoldersError> {
        let user_id = get_current_user_id().map_err(|message| BookmarkFoldersError { message })?;
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
        format!(
            "https://api.x.com/2/users/{}/bookmarks/folders",
            self.user_id
        )
    }

    pub fn fetch(&self) -> Result<Response<BookmarkFoldersResponse>, BookmarkFoldersError> {
        let url = self.url();
        let query_entries = collect_oauth_entries(
            vec![max_results_entry(self.max_results)],
            &self.pagination.oauth_entries(),
        );

        let token_manager = TokenManager::new();
        let access_token = token_manager.get_token();

        let mut request = curl_rest::Client::default().get();
        request = apply_query_params(request, &query_entries);

        let response = request
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .send(url.as_str())
            .map_err(|err| BookmarkFoldersError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let folder_data: BookmarkFoldersResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarkFoldersError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: folder_data,
            })
        } else {
            Err(BookmarkFoldersError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl BookmarkFolderTweets {
    pub fn current_user(folder_id: impl Into<String>) -> Result<Self, BookmarksError> {
        let user_id = get_current_user_id().map_err(|message| BookmarksError { message })?;
        Ok(Self {
            user_id,
            folder_id: folder_id.into(),
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
        format!(
            "https://api.x.com/2/users/{}/bookmarks/folders/{}",
            self.user_id, self.folder_id
        )
    }

    pub fn fetch(&self) -> Result<Response<BookmarksResponse>, BookmarksError> {
        let url = self.url();
        let query_entries = collect_oauth_entries(
            vec![max_results_entry(self.max_results)],
            &tweet_field_entries(),
        );
        let query_entries = collect_oauth_entries(query_entries, &self.pagination.oauth_entries());

        let token_manager = TokenManager::new();
        let access_token = token_manager.get_token();

        let mut request = curl_rest::Client::default().get();
        request = apply_query_params(request, &query_entries);

        let response = request
            .header(curl_rest::Header::Authorization(
                format!("Bearer {}", access_token).into(),
            ))
            .send(url.as_str())
            .map_err(|err| BookmarksError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let bookmarks_data: BookmarksResponse = serde_json::from_slice(&response.body)
                .map_err(|err| BookmarksError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: bookmarks_data,
            })
        } else {
            Err(BookmarksError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

pub fn print_bookmarks(response: &BookmarksResponse) {
    if response.data.is_empty() {
        println!("No bookmarks found.");
        return;
    }

    for tweet in &response.data {
        println!(
            "{}\n",
            crate::twitter::TweetCreateResponse {
                data: tweet.clone(),
                includes: response.includes.clone(),
            }
        );
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
}

pub fn print_bookmark_folders(response: &BookmarkFoldersResponse) {
    if response.data.is_empty() {
        println!("No bookmark folders found.");
        return;
    }

    for (index, folder) in response.data.iter().enumerate() {
        if index > 0 {
            println!();
            println!();
        }

        print!("Folder Id: {}\nName: {}", folder.id, folder.name);
        if let Some(tweet_count) = folder.tweet_count {
            print!("\nTweet count: {}", tweet_count);
        }
        println!();
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
}

pub fn print_folder_tweets(response: &BookmarksResponse) {
    print_bookmarks(response);
}

impl std::fmt::Display for BookmarkFoldersResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, folder) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(f, "Folder Id: {}\nName: {}", folder.id, folder.name)?;
            if let Some(tweet_count) = folder.tweet_count {
                write!(f, "\nTweet count: {}", tweet_count)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmarks_url_uses_current_user_id() {
        let endpoint = Bookmarks {
            user_id: "123".to_string(),
            max_results: 10,
            pagination: Pagination::new(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/bookmarks");
    }

    #[test]
    fn test_create_bookmark_url_uses_current_user_id() {
        let endpoint = CreateBookmark {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/bookmarks");
    }

    #[test]
    fn test_delete_bookmark_url_uses_current_user_and_tweet_id() {
        let endpoint = DeleteBookmark {
            user_id: "123".to_string(),
            tweet_id: "456".to_string(),
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/456"
        );
    }

    #[test]
    fn test_bookmark_folders_url_uses_current_user_id() {
        let endpoint = BookmarkFolders {
            user_id: "123".to_string(),
            max_results: 10,
            pagination: Pagination::new(),
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/folders"
        );
    }

    #[test]
    fn test_bookmark_folder_tweets_url_uses_current_user_and_folder_id() {
        let endpoint = BookmarkFolderTweets {
            user_id: "123".to_string(),
            folder_id: "456".to_string(),
            max_results: 10,
            pagination: Pagination::new(),
        };

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/users/123/bookmarks/folders/456"
        );
    }
}
