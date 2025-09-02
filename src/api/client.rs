use log::info;
use reqwest::Client;

pub trait HttpClient {
    fn new() -> Self;
    fn get(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<String, reqwest::Error>> + Send;
    fn post(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>> + Send;
}

pub struct Response {
    pub status: u16,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    bearer_token: String,
}

impl ApiClient {
    pub fn with_bearer(&mut self, token: &str) -> &Self {
        self.bearer_token = token.to_string();

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
        }
    }

    async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        let res = self.client.get(url).send().await?.text().await?;

        Ok(res)
    }

    async fn post(&self, url: &str, body: serde_json::Value) -> Result<Response, reqwest::Error> {
        info!("foo: {}", self.bearer_token);
        let res = self
            .client
            .post(url)
            .header(reqwest::header::AUTHORIZATION, &self.bearer_token)
            .json(&body)
            .send()
            .await?;

        let response = Response {
            status: res.status().into(),
            content: res.text().await?,
        };
        Ok(response)
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

            async fn post(
                &self,
                url: &str,
                _body: serde_json::Value,
            ) -> Result<Response, reqwest::Error> {
                let res = format!("POST {url}").to_string();

                let response = Response {
                    content: res,
                    status: 200,
                };

                Ok(response)
            }
        }

        let http_client = MockClient::new();

        let result = http_client.get("https://stanleymasinde.com").await.unwrap();
        assert_eq!(result, "GET https://stanleymasinde.com".to_string())
    }
}
