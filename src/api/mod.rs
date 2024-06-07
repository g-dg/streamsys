pub mod server_info;

use std::sync::Arc;

use axum::Router;

use crate::{app::AppServices, auth, state, users};

pub fn route() -> Router<Arc<AppServices>> {
    Router::new()
        .nest("/server-info", server_info::route())
        .nest("/auth", auth::api::route())
        .nest("/users", users::api::route())
        .nest("/state", state::api::route())
}
