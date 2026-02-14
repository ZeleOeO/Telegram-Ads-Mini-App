pub mod bot;
pub mod campaigns;
pub mod channels;
pub mod deals;
pub mod payments;
pub use teloxide::types::Update;
use crate::models::errors::ApiResult;
use crate::{AppState, auth};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;
pub async fn health_check(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    state
        .db
        .ping()
        .await
        .map_err(crate::models::errors::ApiError::DbErr)?;
    Ok("OK")
}
pub async fn me_handler(
    State(state): State<AppState>,
    user: auth::TelegramUser,
) -> ApiResult<impl IntoResponse> {
    use crate::entity::users;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let existing_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?;
    if let Some(db_user) = existing_user {
        let mut active_user: users::ActiveModel = db_user.into();
        active_user.username = Set(user.username.clone());
        active_user.first_name = Set(Some(user.first_name.clone()));
        active_user.last_name = Set(user.last_name.clone());
        active_user.update(&state.db).await?;
    } else {
        let new_user = users::ActiveModel {
            telegram_id: Set(user.id),
            username: Set(user.username.clone()),
            first_name: Set(Some(user.first_name.clone())),
            last_name: Set(user.last_name.clone()),
            ..Default::default()
        };
        new_user.insert(&state.db).await?;
    }
    Ok(Json(json!({
        "status": "authenticated",
        "user": {
            "id": user.id,
            "first_name": user.first_name,
            "username": user.username,
            "auth_date": user.auth_date,
        }
    })))
}
