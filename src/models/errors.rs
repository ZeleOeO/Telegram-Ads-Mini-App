use axum::{Json, http::StatusCode, response::IntoResponse};
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;
pub type ApiResult<T> = Result<T, ApiError>;
pub type BotResult<T> = Result<T, BotError>;
#[derive(Debug)]
pub enum ApiError {
    DbErr(DbErr),
    TelegramErr(teloxide::RequestError),
    NotFound(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}
#[derive(Debug, Error)]
pub enum BotError {
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("Telegram request error: {0}")]
    Telegram(#[from] teloxide::RequestError),
}
impl From<DbErr> for ApiError {
    fn from(err: DbErr) -> Self {
        ApiError::DbErr(err)
    }
}
impl From<teloxide::RequestError> for ApiError {
    fn from(err: teloxide::RequestError) -> Self {
        ApiError::TelegramErr(err)
    }
}
impl From<ton::errors::TonError> for ApiError {
    fn from(err: ton::errors::TonError) -> Self {
        ApiError::Internal(format!("TON Error: {}", err))
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::DbErr(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal database error occurred.".to_string(),
                )
            }
            ApiError::TelegramErr(err) => {
                tracing::error!("Telegram API error: {:?}", err);
                (
                    StatusCode::BAD_GATEWAY,
                    format!("Telegram error: {}", err),
                )
            }
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                )
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
