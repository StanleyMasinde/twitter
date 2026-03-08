use crate::{
    twitter::{
        AUTHOR_EXPANSION, Includes, Response, TWEET_FIELDS, TweetCreateResponse, TweetData,
        USER_FIELDS,
    },
    utils::{bearer_auth_header, oauth_get_header},
};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct TweetLookupError {
    pub message: String,
}

#[derive(Debug)]
pub struct TweetLookup {
    tweet_id: String,
}

#[derive(Debug)]
pub struct TweetsLookup {
    tweet_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecentTweetsMeta {
    #[allow(dead_code)]
    pub newest_id: Option<String>,
    #[allow(dead_code)]
    pub oldest_id: Option<String>,
    #[allow(dead_code)]
    pub result_count: u32,
    #[allow(dead_code)]
    pub next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecentTweetsResponse {
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    #[allow(dead_code)]
    pub meta: Option<RecentTweetsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct RecentTweetsError {
    pub message: String,
}

#[derive(Debug)]
pub struct RecentTweets {
    query: String,
    max_results: u8,
}

#[derive(Debug, Deserialize)]
pub struct TweetCount {
    pub start: String,
    pub end: String,
    pub tweet_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct TweetCountsMeta {
    #[allow(dead_code)]
    pub total_tweet_count: Option<u64>,
    #[allow(dead_code)]
    pub next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TweetCountsResponse {
    pub data: Vec<TweetCount>,
    #[allow(dead_code)]
    pub meta: Option<TweetCountsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct TweetCountsError {
    pub message: String,
}

#[derive(Debug)]
pub struct RecentTweetCounts {
    query: String,
}

#[derive(Debug)]
pub struct AllTweets {
    query: String,
    max_results: u16,
}

#[derive(Debug)]
pub struct UserTweets {
    user_id: String,
    max_results: u8,
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

impl TweetsLookup {
    pub fn new(tweet_ids: Vec<String>) -> Self {
        Self { tweet_ids }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets"
    }

    pub fn fetch(&self) -> Result<Response<RecentTweetsResponse>, RecentTweetsError> {
        let url = self.url();
        let ids = self.tweet_ids.join(",");
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let auth_params = oauth::ParameterList::new([
            ("ids", &ids as &dyn Display),
            ("tweet.fields", &tweet_fields as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
            ("expansions", &expansions as &dyn Display),
        ]);
        let auth_header = oauth_get_header(url, &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("ids", ids.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url)
            .map_err(|err| RecentTweetsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: RecentTweetsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| RecentTweetsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(RecentTweetsError { message: err_data })
        }
    }
}

impl RecentTweets {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(10, 100);
        self
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/search/recent"
    }

    pub fn fetch(&self) -> Result<Response<RecentTweetsResponse>, RecentTweetsError> {
        let url = self.url();
        let query = self.query.as_str();
        let max_results = self.max_results;
        let max_results_query = max_results.to_string();
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("query", query)
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url)
            .map_err(|err| RecentTweetsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: RecentTweetsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| RecentTweetsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(RecentTweetsError { message: err_data })
        }
    }
}

impl RecentTweetCounts {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/counts/recent"
    }

    pub fn fetch(&self) -> Result<Response<TweetCountsResponse>, TweetCountsError> {
        let url = self.url();
        let query = self.query.as_str();
        let auth_params = oauth::ParameterList::new([("query", &query as &dyn Display)]);
        let auth_header = oauth_get_header(url, &auth_params);

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("query", query)
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url)
            .map_err(|err| TweetCountsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: TweetCountsResponse =
                serde_json::from_slice(&response.body).map_err(|err| TweetCountsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(TweetCountsError { message: err_data })
        }
    }
}

impl AllTweets {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            max_results: 10,
        }
    }

    pub fn max_results(mut self, max_results: u16) -> Self {
        self.max_results = max_results.clamp(10, 500);
        self
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/search/all"
    }

    pub fn fetch(&self) -> Result<Response<RecentTweetsResponse>, RecentTweetsError> {
        let url = self.url();
        let query = self.query.as_str();
        let max_results_query = self.max_results.to_string();
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("query", query)
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url)
            .map_err(|err| RecentTweetsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: RecentTweetsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| RecentTweetsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(RecentTweetsError { message: err_data })
        }
    }
}

impl UserTweets {
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
        format!("https://api.x.com/2/users/{}/tweets", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<RecentTweetsResponse>, RecentTweetsError> {
        let url = self.url();
        let max_results = self.max_results;
        let tweet_fields = TWEET_FIELDS.to_string();
        let user_fields = USER_FIELDS.to_string();
        let expansions = AUTHOR_EXPANSION.to_string();
        let auth_params = oauth::ParameterList::new([
            ("max_results", &max_results as &dyn Display),
            ("tweet.fields", &tweet_fields as &dyn Display),
            ("user.fields", &user_fields as &dyn Display),
            ("expansions", &expansions as &dyn Display),
        ]);
        let auth_header = oauth_get_header(url.as_str(), &auth_params);
        let max_results_query = max_results.to_string();

        let response = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", tweet_fields.as_str())
            .query_param_kv("user.fields", user_fields.as_str())
            .query_param_kv("expansions", expansions.as_str())
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| RecentTweetsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let tweets_data: RecentTweetsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| RecentTweetsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: tweets_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(RecentTweetsError { message: err_data })
        }
    }
}

impl std::fmt::Display for TweetCountsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, count) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(
                f,
                "Start: {}\nEnd: {}\nTweet count: {}",
                count.start, count.end, count.tweet_count
            )?;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tweets_lookup_url_uses_collection_endpoint() {
        let endpoint = TweetsLookup::new(vec!["1".to_string(), "2".to_string()]);

        assert_eq!(endpoint.url(), "https://api.x.com/2/tweets");
    }

    #[test]
    fn test_recent_tweet_counts_url_uses_recent_counts_endpoint() {
        let endpoint = RecentTweetCounts::new("rustlang");

        assert_eq!(endpoint.url(), "https://api.x.com/2/tweets/counts/recent");
    }
}
