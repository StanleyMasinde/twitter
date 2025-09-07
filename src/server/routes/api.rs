use axum::{
    Router,
    extract::{self, State},
    response::IntoResponse,
    routing::{get, post},
};
use log::warn;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    api::client::{ApiClient, HttpClient},
    twitter::tweet::create,
};

#[derive(Debug, Clone)]
struct AppState {
    api_client: ApiClient,
}

impl std::ops::Deref for AppState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

#[derive(Deserialize)]
pub struct CreateTweet {
    pub text: String,
}

/// Handle API Routes
pub fn api_routes() -> Router {
    let api_client = ApiClient::new();
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

async fn create_tweet(
    State(state): State<AppState>,
    extract::Json(payload): extract::Json<CreateTweet>,
) -> impl IntoResponse {
    let resp = create(state.api_client, payload).await;

    match resp {
        Ok(res) => (
            StatusCode::from_u16(res.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            res.content,
        ),
        Err(err) => {
            warn!("{err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {err}"),
            )
        }
    }
}
