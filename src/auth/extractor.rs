use crate::{app::AppState, users::db::DbUser};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use super::{db::UserPermission, service::AuthError};

/// Auth token extractor
pub struct AuthToken {
    /// API token
    token: Option<String>,
}

impl AuthToken {
    /// Gets the API token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Helper method to authorize and get the user for the extracted token.
    /// Returns None if authorization failed.
    pub fn authorize(
        &self,
        state: &AppState,
        allowed_permissions: i64,
    ) -> Result<Option<DbUser>, AuthError> {
        if let Some(token) = &self.token {
            if let Some(user) = state.auth_service.authorize(token, allowed_permissions) {
                Ok(Some(user))
            } else {
                Err(AuthError::Generic)
            }
        } else if allowed_permissions == UserPermission::ANY {
            Ok(None)
        } else {
            Err(AuthError::NoToken)
        }
    }

    /// Helper method to invalidate the extracted API token
    pub fn logout(&self, state: &AppState) {
        if let Some(token) = &self.token {
            state.auth_service.logout(token)
        }
    }

    pub fn failure_response() -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("WWW-Authenticate", "Bearer".parse().unwrap());
        (StatusCode::UNAUTHORIZED, headers).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, HeaderMap);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_result = parts.extract::<TypedHeader<Authorization<Bearer>>>().await;

        if let Ok(TypedHeader(Authorization(bearer))) = auth_result {
            Ok(Self {
                token: Some(String::from(bearer.token())),
            })
        } else {
            Ok(Self { token: None })
        }
    }
}
