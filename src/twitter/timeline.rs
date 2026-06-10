use crate::{
    twitter::{
        EXPANSIONS, Includes, Response, TWEET_FIELDS, TweetData, USER_FIELDS,
        params::{
            Pagination, apply_query_params, collect_oauth_entries, max_results_entry,
            oauth_param_list, print_next_page_hint, tweet_field_entries,
        },
    },
    utils::oauth_get_header,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TimelineMeta {
    pub next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    pub data: Vec<TweetData>,
    #[serde(default)]
    pub includes: Option<Includes>,
    pub meta: Option<TimelineMeta>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineError {
    pub message: String,
}

#[derive(Debug)]
pub struct Timeline {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

impl Timeline {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            max_results: 10,
            pagination: Pagination::new(),
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(5, 100);
        self
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.pagination_token(token);
        self
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/users/{}/timelines/reverse_chronological",
            self.user_id
        )
    }

    pub fn fetch(&self) -> Result<Response<TimelineResponse>, TimelineError> {
        let url = self.url();
        let oauth_entries = collect_oauth_entries(
            vec![max_results_entry(self.max_results)],
            &tweet_field_entries(),
        );
        let oauth_entries = collect_oauth_entries(oauth_entries, &self.pagination.oauth_entries());
        let auth_header = oauth_get_header(url.as_str(), &oauth_param_list(oauth_entries.clone()));

        let max_results_query = self.max_results.to_string();
        let pagination_entries = self.pagination.oauth_entries();
        let mut request = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str())
            .query_param_kv("tweet.fields", TWEET_FIELDS)
            .query_param_kv("user.fields", USER_FIELDS)
            .query_param_kv("expansions", EXPANSIONS);
        request = apply_query_params(request, &pagination_entries);

        let response = request
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| TimelineError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let timeline_data: TimelineResponse =
                serde_json::from_slice(&response.body).map_err(|err| TimelineError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: timeline_data,
            })
        } else {
            let err_data = String::from_utf8_lossy(&response.body).to_string();
            Err(TimelineError { message: err_data })
        }
    }
}

pub fn print_timeline(response: &TimelineResponse) {
    if response.data.is_empty() {
        println!("No tweets found in timeline.");
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
