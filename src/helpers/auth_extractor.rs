use crate::{database::users::DbUser, AppState};
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

/// Auth token extractor
pub struct AuthToken {
    /// API token
    pub token: String,
}

impl AuthToken {
    /// Gets the API token
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    /// Helper method to authorize and get the user for the extracted token.
    /// Returns None if authorization failed.
    pub fn authorize(&self, state: &AppState, allowed_permissions: i64) -> Option<DbUser> {
        state
            .auth_service
            .authorize(&self.token, allowed_permissions)
    }

    /// Helper method to invalidate the extracted API token
    pub fn logout(&self, state: &AppState) {
        state.auth_service.logout(&self.token)
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
                token: String::from(bearer.token()),
            })
        } else {
            let mut headers = HeaderMap::new();
            headers.insert("WWW-Authenticate", "Bearer".parse().unwrap());
            Err((StatusCode::UNAUTHORIZED, headers))
        }
    }
}
