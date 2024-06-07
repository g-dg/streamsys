use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::app::AppServices;

pub fn route() -> Router<Arc<AppServices>> {
    Router::new()
        .route("/ping", post(ping))
        .route("/version", get(version))
        .route("/license", get(license))
}

pub async fn ping(request: String) -> String {
    request
}
pub async fn license() -> &'static str {
    include_str!("../../LICENSE")
}
pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
