use crate::{
    api::client::{HttpClient, Response},
    config::Config,
    error::Result,
};
use oauth::{HMAC_SHA1, Token};
use serde_json::json;

use crate::{api::client::ApiClient, server::routes::api::CreateTweet};

pub async fn create(client: ApiClient, payload: CreateTweet) -> Result<Response> {
    let cfg = Config::load()?;

    let token = Token::from_parts(
        cfg.consumer_key,
        cfg.consumer_secret,
        cfg.access_token,
        cfg.access_secret,
    );

    let url = "https://api.twitter.com/2/tweets";
    let auth_header = oauth::post(url, &(), &token, HMAC_SHA1);

    client
        .with_bearer(&auth_header)
        .post(url, json!({ "text": payload.text }))
        .await
        .map_err(crate::error::TwitterError::ReqwestError)
}
