pub mod auth;
pub mod server_info;
pub mod users;

use std::sync::Arc;

use axum::Router;

use crate::app::AppState;

pub fn route() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/server-info", server_info::route())
        .nest("/auth", auth::route())
        .nest("/users", users::route())
}
