use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    app::AppState,
    auth::{db::UserPermission, extractor::AuthToken},
};

use super::service::User;

pub fn route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:user_id", get(get_user))
        .route("/:user_id", put(update_user))
        .route("/:user_id", delete(delete_user))
        .route("/:user_id/sessions", delete(invalidate_sessions))
        .route("/:user_id/password", put(change_password))
}

pub async fn list_users(State(state): State<Arc<AppState>>, token: AuthToken) -> impl IntoResponse {
    let Ok(Some(_current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    let users = state.users_service.list();

    Json(users).into_response()
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    token: AuthToken,
) -> impl IntoResponse {
    let Ok(Some(_current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    let result = state.users_service.get(user_id);

    match result {
        Some(user) => Json(user).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    token: AuthToken,
    Json(request): Json<User>,
) -> impl IntoResponse {
    let Ok(Some(current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    let result = state.users_service.create(&request);

    state.audit_service.log_data(
        Some(current_user.id),
        "user_create",
        json!({
            "user_id": result.as_ref().ok(),
            "username": request.username,
            "enabled": request.enabled,
            "permissions": request.permissions,
            "success": result.is_ok()
        }),
    );

    match result {
        Ok(id) => Json(id).into_response(),
        Err(err) => err.to_status_code().into_response(),
    }
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    token: AuthToken,
    Json(request): Json<User>,
) -> impl IntoResponse {
    let Ok(Some(current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    let mut user = request.clone();
    user.id = Some(user_id);

    let result = state.users_service.update(&user);

    state.audit_service.log_data(
        Some(current_user.id),
        "user_update",
        json!({
            "user_id": user_id,
            "username": user.username,
            "enabled": user.enabled,
            "permissions": user.permissions,
            "success": result.is_ok()
        }),
    );

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.to_status_code().into_response(),
    }
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    token: AuthToken,
) -> impl IntoResponse {
    let Ok(Some(current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    let result = state.users_service.delete(user_id);

    state.audit_service.log_data(
        Some(current_user.id),
        "user_delete",
        json!({
            "user_id": user_id,
            "success": result.is_ok()
        }),
    );

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.to_status_code().into_response(),
    }
}

pub async fn invalidate_sessions(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    token: AuthToken,
) -> impl IntoResponse {
    let Ok(Some(current_user)) = token.authorize(&state, UserPermission::USER_ADMIN) else {
        return AuthToken::failure_response();
    };

    state.auth_service.invalidate_sessions(user_id, None);

    state.audit_service.log_data(
        Some(current_user.id),
        "user_sessions_invalidate",
        json!({
            "user_id": user_id,
        }),
    );

    StatusCode::NO_CONTENT.into_response()
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    token: AuthToken,
    Json(request): Json<String>,
) -> impl IntoResponse {
    let Ok(Some(current_user)) = token.authorize(&state, UserPermission::ANY) else {
        return AuthToken::failure_response();
    };

    // only allow user to change their own password or admin users to change any password
    if (user_id == current_user.id && current_user.permissions & UserPermission::MODIFY_SELF == 0)
        || current_user.permissions & UserPermission::USER_ADMIN == 0
    {
        return StatusCode::FORBIDDEN.into_response();
    }

    let result = state
        .users_service
        .change_password(user_id, &request, token.token());

    state.audit_service.log_data(
        Some(current_user.id),
        "user_password_change",
        json!({
            "user_id": user_id,
            "success": result.is_ok()
        }),
    );

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.to_status_code().into_response(),
    }
}
