use std::fmt::Display;

use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub trait HttpClient {
    fn new() -> Self;
    fn get(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<String, reqwest::Error>> + Send;
    fn post(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>> + Send;
    fn json<T: Serialize + ?Sized>(&mut self, json: &T) -> &Self;
}

pub struct Response {
    pub status: u16,
    pub content: TweetCreateResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetCreateResponse {
    pub data: TweetData,
}

impl Display for TweetCreateResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tweet Id: {}\nTweet body: {}",
            self.data.id, self.data.text
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetData {
    pub text: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    bearer_token: String,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
}

impl ApiClient {
    pub fn with_bearer(&mut self, token: String) -> &Self {
        self.bearer_token = token;

        self
    }
}

impl HttpClient for ApiClient {
    fn new() -> Self {
        let client = Client::new();
        let bearer_token = String::new();
        Self {
            client,
            bearer_token,
            body: None,
            content_type: None,
        }
    }

    async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        let res = self.client.get(url).send().await?.text().await?;

        Ok(res)
    }

    async fn post(&self, url: &str) -> Result<Response, reqwest::Error> {
        info!("foo: {}", self.bearer_token);
        let res = self
            .client
            .post(url)
            .header(reqwest::header::AUTHORIZATION, &self.bearer_token)
            .json(&self.body)
            .send()
            .await?;

        let status = res.status();

        let content = res.json().await?;

        let response = Response {
            status: status.into(),
            content,
        };
        Ok(response)
    }

    fn json<T: Serialize + ?Sized>(&mut self, json: &T) -> &Self {
        match serde_json::to_vec(json) {
            Ok(data) => {
                self.body = Some(data);
                self.content_type = Some("application/json".into());
            }
            Err(err) => {
                // up to you: either panic, return Result<Self>, or store the error
                panic!("Failed to serialize body: {err}");
            }
        }

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_api_client() {
        struct MockClient {}

        impl HttpClient for MockClient {
            fn new() -> Self {
                Self {}
            }

            async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
                Ok(format!("GET {url}").to_string())
            }

            async fn post(&self, _url: &str) -> Result<Response, reqwest::Error> {
                let data = TweetData {
                    text: "fooo".to_string(),
                    edit_history_tweet_ids: vec![],
                    id: 0.to_string(),
                };
                let res = TweetCreateResponse { data };

                let response = Response {
                    content: res,
                    status: 200,
                };

                Ok(response)
            }

            fn json<T: Serialize + ?Sized>(&mut self, _json: &T) -> &Self {
                self
            }
        }

        let http_client = MockClient::new();

        let result = http_client.get("https://stanleymasinde.com").await.unwrap();
        assert_eq!(result, "GET https://stanleymasinde.com".to_string())
    }
}
