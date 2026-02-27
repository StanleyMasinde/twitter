use std::{
    io::Read,
    path::{Path, PathBuf},
};

use oauth::{HMAC_SHA1, Token};
use serde::Deserialize;

use crate::utils::load_config;

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

pub fn upload(path: PathBuf) -> Result<String, UploadMediaError> {
    let upload_url = "https://api.x.com/2/media/upload";
    println!("> Uploading image to Twitter.");

    let mut cfg = load_config();
    let current_account = cfg.current_account();
    let token = Token::from_parts(
        current_account.consumer_key.as_str(),
        current_account.consumer_secret.as_str(),
        current_account.access_token.as_str(),
        current_account.access_secret.as_str(),
    );
    let auth_header = oauth::post(upload_url, &(), &token, HMAC_SHA1);
    let file_kind = infer::get_from_path(&path);

    let media_type = match file_kind {
        Ok(kind) => kind.unwrap().mime_type().to_string(),
        Err(_) => {
            return Err(UploadMediaError {
                message: "Could not get the file type.".to_string(),
            });
        }
    };

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image.bin")
        .to_string();

    let boundary = format!("----twitter-cli-{}", std::process::id());
    let body = build_multipart_body(&boundary, &file_name, &media_type, &path)?;
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let response = curl_rest::Client::default()
        .post()
        .header(curl_rest::Header::Authorization(auth_header.into()))
        .header(curl_rest::Header::ContentType(content_type.into()))
        .body(curl_rest::Body::Bytes(body.into()))
        .send(upload_url)
        .map_err(|err| UploadMediaError {
            message: err.to_string(),
        })?;

    if (200..300).contains(&response.status.as_u16()) {
        let media_upload_res: MediaUploadResponse = serde_json::from_slice(&response.body)
            .map_err(|err| UploadMediaError {
                message: err.to_string(),
            })?;

        println!("> Image uploaded to Twitter. The image ID will be added to the first tweet.");

        Ok(media_upload_res.data.id)
    } else {
        Err(UploadMediaError {
            message: "Please provive a valid image file. Videos are not supported".to_string(),
        })
    }
}

fn build_multipart_body(
    boundary: &str,
    file_name: &str,
    media_type: &str,
    file_path: &Path,
) -> Result<Vec<u8>, UploadMediaError> {
    let file_size = std::fs::metadata(file_path)
        .map(|meta| meta.len() as usize)
        .unwrap_or(0);
    let mut body = Vec::with_capacity(file_size + 512);

    let write_text_part = |body: &mut Vec<u8>, name: &str, value: &str| {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    };

    write_text_part(&mut body, "media_category", "tweet_image");
    write_text_part(&mut body, "media_type", media_type);

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"media\"; filename=\"{file_name}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {media_type}\r\n\r\n").as_bytes());
    let mut file = std::fs::File::open(file_path).map_err(|err| UploadMediaError {
        message: err.to_string(),
    })?;
    file.read_to_end(&mut body).map_err(|err| UploadMediaError {
        message: err.to_string(),
    })?;
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    Ok(body)
}
