use chrono::{NaiveDateTime, Utc};
use sea_orm::DatabaseConnection;
use teloxide::prelude::*;
use teloxide::types::MessageId;
use tracing::{info, warn};
use crate::models::errors::{ApiError, ApiResult};
pub async fn schedule_post(
    deal_id: i32,
    scheduled_time: NaiveDateTime,
) -> ApiResult<NaiveDateTime> {
    info!("Post scheduled for deal {} at {}", deal_id, scheduled_time);
    Ok(scheduled_time)
}
pub async fn publish_post(
    bot: &Bot,
    channel_id: ChatId,
    content: &str,
) -> ApiResult<(MessageId, String)> {
    info!("Publishing post to channel {}", channel_id);
    let message = bot.send_message(channel_id, content).await?;
    let post_link = format!("https://t.me/c/{}/{}", channel_id.0.abs(), message.id.0);
    info!("Post published successfully: {}", post_link);
    Ok((message.id, post_link))
}
use crate::services::grammers_client::GrammersClient;
pub async fn verify_post_exists(
    grammers: &GrammersClient,
    channel_username: &str,
    message_id: i32,
) -> ApiResult<bool> {
    info!(
        "Verifying post exists via Grammers: channel={}, message_id={}",
        channel_username, message_id
    );
    grammers.verify_message_exists(channel_username, message_id).await
}
pub async fn monitor_post_integrity(
    grammers: &GrammersClient,
    channel_username: &str,
    message_id: i32,
    _original_content: &str,
) -> ApiResult<bool> {
    info!(
        "Monitoring post integrity: channel={}, message_id={}",
        channel_username, message_id
    );
    let exists = verify_post_exists(grammers, channel_username, message_id).await?;
    if !exists {
        warn!(
            "Post was deleted: channel={}, message_id={}",
            channel_username, message_id
        );
        return Ok(false);
    }
    Ok(true)
}
pub async fn execute_auto_post(
    db: &DatabaseConnection,
    bot: &Bot,
    deal_id: i32,
    channel_id: ChatId,
    content: &str,
) -> ApiResult<String> {
    use crate::entity::deals;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    let (_message_id, post_link) = publish_post(bot, channel_id, content).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.post_link = Set(Some(post_link.clone()));
    active_deal.actual_post_time = Set(Some(Utc::now().naive_utc()));
    active_deal.state = Set("published".to_string());
    active_deal.update(db).await?;
    info!("Auto-post completed for deal {}: {}", deal_id, post_link);
    Ok(post_link)
}
