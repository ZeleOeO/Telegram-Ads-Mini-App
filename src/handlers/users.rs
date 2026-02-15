use crate::{
    auth,
    entity::users,
    models::errors::ApiResult,
    AppState,
};
use axum::{
    extract::{State},
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateWalletRequest {
    pub address: String,
}

pub async fn update_wallet_address(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Json(payload): Json<UpdateWalletRequest>,
) -> ApiResult<Response> {
    let db_user = match users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
    {
        Some(u) => u,
        None => {
            let new_user = users::ActiveModel {
                telegram_id: Set(user.id),
                ..Default::default()
            };
            new_user.insert(&state.db).await?
        }
    };

    let mut active_user: users::ActiveModel = db_user.into();
    active_user.ton_wallet_address = Set(Some(payload.address));
    active_user.updated_at = Set(chrono::Utc::now().naive_utc());
    active_user.update(&state.db).await?;

    Ok(Json(serde_json::json!({"status": "ok"})).into_response())
}
