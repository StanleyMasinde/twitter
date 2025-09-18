use axum::{
    Router,
    extract::{self, State},
    response::IntoResponse,
    routing::{get, post},
};
use axum_macros::debug_handler;
use log::warn;
use reqwest::StatusCode;

use crate::twitter::tweet::{Tweet, TweetBody, TwitterApi};

#[derive(Debug, Clone)]
struct AppState {
    api_client: reqwest::Client,
}

impl std::ops::Deref for AppState {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

/// Handle API Routes
pub fn api_routes() -> Router {
    let api_client = reqwest::Client::new();
    let state = AppState { api_client };
    Router::new()
        .route("/health", get(health_check))
        .route("/tweet", post(create_tweet))
        .with_state(state)
}

/// Health check logic
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Ok".to_string())
}

#[debug_handler]
async fn create_tweet(
    State(state): State<AppState>,
    extract::Json(payload): extract::Json<TweetBody>,
) -> impl IntoResponse {
    let mut tweet = Tweet::new(state.api_client, payload);
    let resp = tweet.create().await;

    match resp {
        Ok(res) => (
            StatusCode::from_u16(res.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            res.content.to_string(),
        ),
        Err(err) => {
            warn!("{:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Somethin broke: {:?}", err),
            )
        }
    }
}
