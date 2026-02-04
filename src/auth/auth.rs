use axum::{
    Json, async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use init_data_rs::{self, InitData};
use std::env;

#[derive(Debug)]
pub struct AuthError(String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": self.0
        });
        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

pub struct TelegramUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub auth_date: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for TelegramUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AuthError("Missing Authorization header".to_string()))?
            .to_str()
            .map_err(|_| AuthError("Invalid Authorization header".to_string()))?;

        let init_data = if let Some(stripped) = auth_header.strip_prefix("tma ") {
            stripped
        } else {
            auth_header
        };

        let bot_token = env::var("TELOXIDE_TOKEN")
            .map_err(|_| AuthError("Server configuration error".to_string()))?;

        validate_init_data(init_data, &bot_token)
    }
}

fn validate_init_data(init_data: &str, bot_token: &str) -> Result<TelegramUser, AuthError> {
    let init_data: InitData = init_data_rs::validate(init_data, bot_token, None)
        .map_err(|e| AuthError(format!("Init Data Error: {}", e)))?;

    let user = init_data
        .user
        .ok_or(AuthError("Missing user data".to_string()))?;

    Ok(TelegramUser {
        id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        username: user.username,
        auth_date: init_data.auth_date as i64,
    })
}
