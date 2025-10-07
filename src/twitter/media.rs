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
}

#[derive(Debug, Deserialize)]
pub struct UploadMediaError {
    pub message: String,
}

pub async fn upload(client: reqwest::Client, path: PathBuf) -> Result<String, UploadMediaError> {
    let upload_url = "https://api.x.com/2/media/upload";
    println!("> Uploading image.");

    let cfg = Config::load();
    let token = Token::from_parts(
        cfg.consumer_key,
        cfg.consumer_secret,
        cfg.access_token,
        cfg.access_secret,
    );
    let auth_header = oauth::post(upload_url, &(), &token, HMAC_SHA1);
    let file_kind = infer::get_from_path(&path);

    let media_type = match file_kind {
        Ok(kind) => kind.unwrap().mime_type(),
        Err(_) => {
            return Err(UploadMediaError {
                message: "Could not get the file type.".to_string(),
            });
        }
    };

    let form = multipart::Form::new()
        .text("media_category", "tweet_image")
        .text("media_type", media_type)
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
    let status = response.status();

    let response_text = response.text().await.map_err(|err| UploadMediaError {
        message: err.to_string(),
    })?;

    if status.is_success() {
        let media_upload_res: MediaUploadResponse =
            serde_json::from_str(&response_text).map_err(|err| UploadMediaError {
                message: err.to_string(),
            })?;

        println!("> Image uploaded to Twitter.");

        Ok(media_upload_res.data.id)
    } else {
        Err(UploadMediaError {
            message: "Please provive a valid image file. Videos are not supported".to_string(),
        })
    }
}
