use axum::{Json, http::StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

/// Represents one item in the "results" array from your webhook payload.
#[derive(Debug, Deserialize)]
struct WebhookShare {
    payload: Value,     // actual Spotify (or arbitrary) data
    signature: Option<String>,
    message: Option<String>,
    hash: Option<String>,
}

/// Represents the entire webhook request body.
#[derive(Debug, Deserialize)]
pub struct WebhookRequest {
    results: Option<Vec<WebhookShare>>,
}

pub async fn get_balance(
    Json(payload): Json<WebhookRequest>,
) -> (StatusCode, Json<Value>) {
    // Validate results field
    let Some(results) = payload.results else {
        info!("Missing or empty response results. Unable to proceed.");
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "message": "Missing results field in response."
            })),
        );
    };

    if results.is_empty() {
        info!("Empty results received. Nothing to process.");
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "message": "Empty results field in response."
            })),
        );
    }

    info!("{} share(s) received...", results.len());

    for share in results {
        let data = share.payload;
        info!("Spotify data: {:?}", data);

        // Here’s where your logic goes
        handle_spotify_data(data);
    }

    (
        StatusCode::OK,
        Json(json!({ "success": true })),
    )
}

/// Example data handler (stub).
fn handle_spotify_data(payload: Value) {
    // Replace this with real logic — parsing, writing to Solana, whatever your backend needs.
    info!("Processing payload: {}", payload);
}
