use crate::entity::bot_observed_channels;
use crate::models::errors::BotResult;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use teloxide::types::{ChatMemberKind, ChatMemberUpdated};
use tracing::info;

pub async fn handle_my_chat_member(
    db: sea_orm::DatabaseConnection,
    update: ChatMemberUpdated,
) -> BotResult<()> {
    let chat_id = update.chat.id.0;
    let title = update.chat.title().map(|s: &str| s.to_string());
    let username = update.chat.username().map(|s: &str| s.to_string());

    match update.new_chat_member.kind {
        ChatMemberKind::Administrator(_) => {
            info!("Bot added as admin to channel: {} ({})", title.as_deref().unwrap_or("unknown"), chat_id);
            
            let existing = bot_observed_channels::Entity::find()
                .filter(bot_observed_channels::Column::TelegramChatId.eq(chat_id))
                .one(&db)
                .await?;

            if existing.is_none() {
                let new_obs = bot_observed_channels::ActiveModel {
                    telegram_chat_id: Set(chat_id),
                    title: Set(title),
                    username: Set(username),
                    ..Default::default()
                };
                new_obs.insert(&db).await?;
            } else {
                let mut active: bot_observed_channels::ActiveModel = existing.unwrap().into();
                active.title = Set(title);
                active.username = Set(username);
                active.update(&db).await?;
            }
        }
        ChatMemberKind::Left => {
            info!("Bot removed from channel: {}", chat_id);
            bot_observed_channels::Entity::delete_many()
                .filter(bot_observed_channels::Column::TelegramChatId.eq(chat_id))
                .exec(&db)
                .await?;
        }
        _ => {}
    }

    Ok(())
}
