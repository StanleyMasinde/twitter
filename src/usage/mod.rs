use crate::utils::load_config;
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
            "Daily project usage: {}/{}",
            self.data.project_usage, self.data.project_cap
        )
    }
}

pub fn show() {
    let mut cfg = load_config();
    let account = cfg.current_account();
    let token = account.bearer_token.as_str();
    let auth_header = format!("Bearer {}", token);

    let response = match curl_rest::Client::default()
        .get()
        .query_param_kv("days", "1")
        .header(curl_rest::Header::Authorization(auth_header.into()))
        .send("https://api.x.com/2/usage/tweets")
    {
        Ok(response) => response,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let response_text = String::from_utf8_lossy(&response.body).to_string();
    if (200..300).contains(&response.status.as_u16()) {
        let usage: OkResponse = serde_json::from_str(&response_text).unwrap();

        println!("{}", usage);
    } else if response.status.as_u16() == 429 {
        eprintln!("You have reached a rate limit. Try again later.")
    } else {
        eprintln!("{}", response_text)
    }
}
