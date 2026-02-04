use sea_orm::DatabaseConnection;
use teloxide::prelude::*;
use tracing::info;

use crate::models::errors::ApiResult;

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub subscribers: i64,
    pub reach: Option<i64>,
    pub language: Option<String>,
    pub premium_percentage: Option<f32>,
}

// Fetch verified channel statistics from Telegram
//  Telegram Bot API has limited analytics. Full analytics we'll use an API or Channel Stats Bot
pub async fn fetch_channel_stats(bot: &Bot, channel_id: ChatId) -> ApiResult<ChannelStats> {
    info!("Fetching stats for channel {}", channel_id);

    let count_result = bot.get_chat_member_count(channel_id).await;
    let subscribers = match count_result {
        Ok(count) => count as i64,
        Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(ref s)))
            if s.contains("member list is inaccessible") =>
        {
            0
        }
        Err(e) => return Err(e.into()),
    };

    Ok(ChannelStats {
        subscribers: subscribers as i64,
        reach: None,
        language: None,
        premium_percentage: None,
    })
}

// Update channel stats in database
pub async fn update_channel_stats(
    db: &DatabaseConnection,
    bot: &Bot,
    channel_id: i32,
    telegram_channel_id: i64,
) -> ApiResult<()> {
    use crate::entity::channels;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let chat_id = ChatId(telegram_channel_id);
    let stats = fetch_channel_stats(bot, chat_id).await?;

    let channel = channels::Entity::find_by_id(channel_id)
        .one(db)
        .await?
        .ok_or_else(|| {
            crate::models::errors::ApiError::NotFound("Channel not found".to_string())
        })?;

    let mut active_channel: channels::ActiveModel = channel.into();
    active_channel.subscribers = Set(Some(stats.subscribers));
    active_channel.last_stats_update = Set(Some(chrono::Utc::now().naive_utc()));

    active_channel.update(db).await?;

    info!(
        "Updated stats for channel {}: {} subscribers",
        channel_id, stats.subscribers
    );

    Ok(())
}
