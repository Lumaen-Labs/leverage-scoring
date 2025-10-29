use crate::AppState;
use crate::handlers::binance;
use crate::handlers::proofs;
use axum::{Router, routing::{get, post}};
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/webhook", post(binance::fetch_data))
        .route("/api/proofs/submit", post(proofs::submit_proof))
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}