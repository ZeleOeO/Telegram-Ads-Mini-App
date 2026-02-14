use sea_orm::{DatabaseConnection, ActiveModelTrait, EntityTrait, Set};
use teloxide::prelude::*;
use tracing::{info, warn};
use crate::models::errors::ApiResult;
use crate::services::grammers_client::GrammersClient;
#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub subscribers: i64,
    pub reach: Option<i64>,
    pub language: Option<String>,
    pub premium_percentage: Option<f32>,
    pub enabled_notifications: Option<f32>,
    pub shares_per_post: Option<f32>,
    pub reactions_per_post: Option<f32>,
}
pub async fn fetch_channel_stats(
    bot: &Bot, 
    grammers: &GrammersClient, 
    channel_id: ChatId, 
    username: Option<&str>
) -> ApiResult<ChannelStats> {
    info!("Fetching stats for channel {}", channel_id);
    if let Some(uname) = username {
        match grammers.get_broadcast_stats(uname).await {
            Ok(analytics) => {
                info!("Fetched full stats via MTProto for {}", uname);
                return Ok(ChannelStats {
                    subscribers: analytics.subscribers,
                    reach: analytics.reach,
                    language: analytics.languages,
                    premium_percentage: analytics.premium_percentage,
                    enabled_notifications: analytics.enabled_notifications_percent.map(|p| p as f32),
                    shares_per_post: analytics.shares_per_post.map(|s| s as f32),
                    reactions_per_post: analytics.reactions_per_post.map(|r| r as f32),
                });
            }
            Err(e) => {
                warn!("MTProto stats fetch failed for {} (falling back to Bot API): {:?}", uname, e);
            }
        }
    }
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
        enabled_notifications: None,
        shares_per_post: None,
        reactions_per_post: None,
    })
}
pub async fn update_channel_stats(
    db: &DatabaseConnection,
    bot: &Bot,
    grammers: &GrammersClient,
    channel_id: i32,
    telegram_channel_id: i64,
    username: Option<String>,
) -> ApiResult<()> {
    use crate::entity::channels;
    let chat_id = ChatId(telegram_channel_id);
    let stats = fetch_channel_stats(bot, grammers, chat_id, username.as_deref()).await?;
    let channel = channels::Entity::find_by_id(channel_id)
        .one(db)
        .await?
        .ok_or_else(|| {
            crate::models::errors::ApiError::NotFound("Channel not found".to_string())
        })?;
    let mut active_channel: channels::ActiveModel = channel.into();
    active_channel.subscribers = Set(Some(stats.subscribers));
    if let Some(reach) = stats.reach {
        active_channel.reach = Set(Some(reach));
    }
    if let Some(lang) = stats.language {
        active_channel.language = Set(Some(lang));
    }
    if let Some(premium) = stats.premium_percentage {
        active_channel.premium_percentage = Set(Some(premium));
    }
    if let Some(enabled) = stats.enabled_notifications {
        active_channel.enabled_notifications = Set(Some(enabled));
    }
    if let Some(shares) = stats.shares_per_post {
        active_channel.shares_per_post = Set(Some(shares));
    }
    if let Some(reactions) = stats.reactions_per_post {
        active_channel.reactions_per_post = Set(Some(reactions));
    }
    active_channel.last_stats_update = Set(Some(chrono::Utc::now().naive_utc()));
    active_channel.update(db).await?;
    info!(
        "Updated stats for channel {}: {} subscribers",
        channel_id, stats.subscribers
    );
    Ok(())
}
