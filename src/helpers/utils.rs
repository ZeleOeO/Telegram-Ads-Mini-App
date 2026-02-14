use crate::{
    auth,
    entity::{users, channels, deals},
    models::errors::{ApiError, ApiResult},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use teloxide::{prelude::*, types::{ChatId, UserId, ChatMemberKind}};
use tracing;

pub async fn verify_channel_admin(
    bot: &Bot,
    channel_telegram_id: i64,
    user_telegram_id: i64,
) -> ApiResult<()> {
    let chat_id = ChatId(channel_telegram_id);
    let user_id = UserId(user_telegram_id as u64);
    
    let member = bot.get_chat_member(chat_id, user_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch chat member: {:?}", e);
            ApiError::Internal("Failed to verify admin status with Telegram".to_string())
        })?;

    match member.kind {
        ChatMemberKind::Administrator(_) | ChatMemberKind::Owner(_) => Ok(()),
        _ => Err(ApiError::Forbidden("You are no longer an admin of this channel.".to_string())),
    }
}
pub async fn sync_user_or_create(
    db: &sea_orm::DatabaseConnection,
    user: &auth::TelegramUser,
) -> ApiResult<users::Model> {
    match users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(db)
        .await?
    {
        Some(u) => Ok(u),
        None => {
            let new_user = users::ActiveModel {
                telegram_id: Set(user.id),
                ..Default::default()
            };
            Ok(new_user.insert(db).await?)
        }
    }
}
pub async fn get_deal_and_channel(
    db: &sea_orm::DatabaseConnection,
    deal_id: i32,
) -> ApiResult<(deals::Model, channels::Model)> {
    let deal = deals::Entity::find_by_id(deal_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    Ok((deal, channel))
}
