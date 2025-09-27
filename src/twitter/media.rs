use std::path::PathBuf;

use oauth::{HMAC_SHA1, Token};
use reqwest::multipart;
use serde::Deserialize;

use crate::config::Config;

#[derive(Debug, Deserialize)]
struct MediaUploadResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    id: String,
    media_key: String,
    size: u64,
    expires_after_secs: u64,
    image: Image,
}

#[derive(Debug, Deserialize)]
struct Image {
    image_type: String,
    w: u32,
    h: u32,
}

#[derive(Debug, Deserialize)]
pub struct UploadMediaError {
    pub message: String,
}

pub async fn upload(client: reqwest::Client, path: PathBuf) -> Result<String, UploadMediaError> {
    let upload_url = "https://api.x.com/2/media/upload";

    let cfg = Config::load();
    let token = Token::from_parts(
        cfg.consumer_key,
        cfg.consumer_secret,
        cfg.access_token,
        cfg.access_secret,
    );
    let auth_header = oauth::post(upload_url, &(), &token, HMAC_SHA1);

    let form = multipart::Form::new()
        .text("media_category", "tweet_image")
        .text("media_type", "image/png")
        .file("media", path)
        .await
        .map_err(|err| UploadMediaError {
            message: err.to_string(),
        })?;

    let response = client
        .post(upload_url)
        .header(reqwest::header::AUTHORIZATION, &auth_header)
        .multipart(form)
        .send()
        .await
        .map_err(|err| UploadMediaError {
            message: err.to_string(),
        })?;

    let response_text = response.text().await.map_err(|err| UploadMediaError {
        message: err.to_string(),
    })?;

    let media_upload_res: MediaUploadResponse =
        serde_json::from_str(&response_text).map_err(|err| UploadMediaError {
            message: err.to_string(),
        })?;

    println!("{:?}", response_text);

    Ok(media_upload_res.data.id)
}
