use std::fmt::Display;

use crate::{
    twitter::{Response, TweetData},
    utils::oauth_get_header,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MentionsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MentionsResponse {
    pub data: Vec<TweetData>,
    #[allow(dead_code)]
    pub meta: Option<MentionsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct MentionsError {
    pub message: String,
}

#[derive(Debug)]
pub struct Mentions {
    user_id: String,
    max_results: u8,
}

impl Mentions {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(5, 100);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/mentions", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<MentionsResponse>, MentionsError> {
        let url = self.url();
        let max_results = self.max_results;
        let auth_params =
            oauth::ParameterList::new([("max_results", &max_results as &dyn Display)]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);
        let max_results_query = max_results.to_string();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| MentionsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let mentions_data: MentionsResponse =
                serde_json::from_slice(&response.body).map_err(|err| MentionsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: mentions_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(MentionsError { message: err_data })
        }
    }
}
