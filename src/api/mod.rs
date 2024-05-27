pub mod server_info;

use std::sync::Arc;

use axum::Router;

use crate::{app::AppState, auth, display_state, users};

pub fn route() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/server-info", server_info::route())
        .nest("/auth", auth::api::route())
        .nest("/users", users::api::route())
        .nest("/display-state", display_state::api::route())
}
