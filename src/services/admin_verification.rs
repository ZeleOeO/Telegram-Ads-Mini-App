use teloxide::prelude::*;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::{info, warn};

use crate::entity::{channel_admins, channels};
use crate::models::errors::ApiResult;

pub async fn verify_bot_admin_status(bot: &Bot, channel_id: ChatId) -> ApiResult<bool> {
    let me = bot.get_me().await?;
    let admins_result = bot.get_chat_administrators(channel_id).await;

    match admins_result {
        Ok(admins) => {
            let bot_admin = admins.iter().find(|admin| admin.user.id == me.id);
            let can_post = bot_admin.map_or(false, |admin| admin.can_post_messages());
            Ok(can_post)
        }
        Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(ref s)))
            if s.contains("member list is inaccessible") =>
        {
            // This means the bot is not added by the user as an admin
            Ok(false)
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn verify_user_admin_status(
    bot: &Bot,
    channel_id: ChatId,
    user_id: UserId,
) -> ApiResult<(bool, bool)> {
    let admins = bot.get_chat_administrators(channel_id).await?;

    let user_admin = admins.iter().find(|admin| admin.user.id == user_id);

    if let Some(admin) = user_admin {
        let can_post = admin.can_post_messages();
        Ok((true, can_post))
    } else {
        Ok((false, false))
    }
}

pub async fn re_verify_channel_admins(db: &DatabaseConnection, bot: &Bot) -> ApiResult<()> {
    use sea_orm::ActiveModelTrait;

    info!("Starting channel admin re-verification");

    let active_channels = channels::Entity::find()
        .filter(channels::Column::Status.eq("active"))
        .all(db)
        .await?;

    for channel in active_channels {
        let channel_id = ChatId(channel.telegram_channel_id);

        match verify_bot_admin_status(bot, channel_id).await {
            Ok(can_post) => {
                if !can_post {
                    warn!("Bot lost admin permissions on channel {}", channel.id);
                    let mut active_channel: channels::ActiveModel = channel.clone().into();
                    active_channel.status = sea_orm::Set("inactive".to_string());
                    active_channel.update(db).await?;
                    continue;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to verify bot admin for channel {}: {:?}",
                    channel.id, e
                );
                continue;
            }
        }

        let admins = channel_admins::Entity::find()
            .filter(channel_admins::Column::ChannelId.eq(channel.id))
            .all(db)
            .await?;

        for admin in admins {
            let user = crate::entity::users::Entity::find_by_id(admin.user_id)
                .one(db)
                .await?;

            if let Some(user) = user {
                let user_id = UserId(user.telegram_id as u64);

                match verify_user_admin_status(bot, channel_id, user_id).await {
                    Ok((is_admin, can_post)) => {
                        if !is_admin || !can_post {
                            warn!(
                                "User {} lost admin permissions on channel {}",
                                admin.user_id, channel.id
                            );
                            let mut active_admin: channel_admins::ActiveModel = admin.into();
                            active_admin.can_post_messages = sea_orm::Set(can_post && is_admin);
                            active_admin.update(db).await?;
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to verify admin {} for channel {}: {:?}",
                            admin.user_id, channel.id, e
                        );
                    }
                }
            }
        }
    }

    info!("Completed channel admin re-verification");
    Ok(())
}
