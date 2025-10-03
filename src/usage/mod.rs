use crate::config::Config;
use reqwest::StatusCode;
use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize)]
struct OkResponse {
    data: TweetUsage,
}

#[derive(Deserialize)]
struct TweetUsage {
    project_usage: String,
    project_cap: String,
}

impl Display for OkResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project usage: {}/{}",
            self.data.project_usage, self.data.project_cap
        )
    }
}

pub async fn show() {
    let cfg = Config::load();
    let token = cfg.bearer_token;
    let auth_header = format!("Bearer {}", token);
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.x.com/2/usage/tweets")
        .query(&[("days", 1)])
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .send()
        .await
        .unwrap();
    let status = response.status();

    let response_text = response.text().await.unwrap();
    if status.is_success() {
        let usage: OkResponse = serde_json::from_str(&response_text).unwrap();

        println!("{}", usage);
    } else if status == StatusCode::from_u16(429).unwrap() {
        eprintln!("You have reached a rate limit. Try again later.")
    } else {
        eprintln!("{}", response_text)
    }
}
