use crate::{
    twitter::{AUTHOR_EXPANSION, Response, TWEET_FIELDS, TweetCreateResponse, USER_FIELDS},
    utils::oauth_get_header,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TweetLookupError {
    pub message: String,
}

#[derive(Debug)]
pub struct TweetLookup {
    tweet_id: String,
}

impl TweetLookup {
    pub fn new(tweet_id: impl Into<String>) -> Self {
        Self {
            tweet_id: tweet_id.into(),
        }
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/tweets/{}", self.tweet_id)
    }

    pub fn fetch(&self) -> Result<Response<TweetCreateResponse>, TweetLookupError> {
        let url = self.url();
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let auth_params = oauth::ParameterList::new([
            ("tweet.fields", &tweet_fields as &dyn std::fmt::Display),
            ("user.fields", &user_fields as &dyn std::fmt::Display),
            ("expansions", &expansions as &dyn std::fmt::Display),
        ]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| TweetLookupError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweet_data: TweetCreateResponse =
                serde_json::from_slice(&response.body).map_err(|err| TweetLookupError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweet_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(TweetLookupError { message: err_data })
        }
    }
}
