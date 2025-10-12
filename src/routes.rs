use axum::{Router, routing::{get, post}};
use crate::handlers::binance;

pub fn binance_routes() -> Router {
    Router::new()
        .route("/binance", post(binance::get_balance))
}
