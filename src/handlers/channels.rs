use crate::{
    AppState, auth,
    entity::{channels, users, channel_ad_formats, bot_observed_channels, channel_admins},
    models::{
        errors::{ApiError, ApiResult},
        channel::*,
    },
    services,
};
use axum::{
    Json,
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use teloxide::prelude::*;
use teloxide::types::ChatId;
use tracing::info;

/// Add a new channel (enhanced from original)
pub async fn add_channel_handler(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Json(payload): Json<AddChannelPayload>,
) -> ApiResult<Response> {
    // Check if user exists, otherwise create them
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

    let _me = state.bot.get_me().await?;

    // Sanitize input: handle t.me links, @ prefix, and numeric IDs
    let input = payload.username.trim();
    let target = if input.contains("t.me/") {
        input.split("t.me/").last().unwrap_or(input).to_string()
    } else {
        input.to_string()
    };

    // If it's a numeric ID (starts with - or digit), parse it as i64
    let chat_id_res = if target.starts_with('-') || target.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        target.parse::<i64>().ok().map(ChatId)
    } else {
        None
    };

    let chat = if let Some(id) = chat_id_res {
        state.bot.get_chat(id).await?
    } else {
        // Ensure username has @ for get_chat if it's not an ID
        let username = if target.starts_with('@') { target } else { format!("@{}", target) };
        state.bot.get_chat(username).await?
    };
    
    let channel_id = chat.id;

    // Verify bot is admin with post permissions
    let can_post = services::admin_verification::verify_bot_admin_status(&state.bot, channel_id).await?;

    if !can_post {
        return Err(ApiError::Forbidden(
            "Bot is not an admin or cannot post messages. Please make this bot an admin and grant it access to posting messages"
                .to_string(),
        ));
    }

    let existing_channel = channels::Entity::find()
        .filter(channels::Column::TelegramChannelId.eq(channel_id.0))
        .one(&state.db)
        .await?;

    if existing_channel.is_some() {
        return Err(ApiError::Conflict(
            "This channel is already registered on the bot".to_string(),
        ));
    }

    // Fetch channel stats
    let stats = services::channel_stats::fetch_channel_stats(&state.bot, channel_id).await?;

    let new_channel = channels::ActiveModel {
        owner_id: Set(db_user.id),
        telegram_channel_id: Set(channel_id.0),
        title: Set(chat.title().map(|s| s.to_string())),
        username: Set(chat.username().map(|s| s.to_string())),
        subscribers: Set(Some(stats.subscribers)),
        reach: Set(stats.reach),
        language: Set(stats.language),
        premium_percentage: Set(stats.premium_percentage),
        status: Set("active".to_string()),
        last_stats_update: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    let saved_channel = new_channel.insert(&state.db).await?;

    info!(
        "User {} added new channel {} ({})",
        user.id,
        chat.username().unwrap_or_default(),
        channel_id
    );

    let response_body = json!(saved_channel);
    Ok((StatusCode::CREATED, Json(response_body)).into_response())
}

/// Update channel information
pub async fn update_channel_handler(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(channel_id): Path<i32>,
    Json(payload): Json<UpdateChannelRequest>,
) -> ApiResult<Response> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let channel = channels::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify ownership
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden("You don't own this channel".to_string()));
    }

    let mut active_channel: channels::ActiveModel = channel.into();
    
    if let Some(title) = payload.title {
        active_channel.title = Set(Some(title));
    }
    if let Some(description) = payload.description {
        active_channel.description = Set(Some(description));
    }
    if let Some(language) = payload.language {
        active_channel.language = Set(Some(language));
    }
    if let Some(category) = payload.category {
        active_channel.category = Set(Some(category));
    }
    // Reach and Premium Percentage should be automated/fetched by bot
    /*
    if let Some(reach) = payload.reach {
        active_channel.reach = Set(Some(reach));
    }
    if let Some(premium_percentage) = payload.premium_percentage {
        active_channel.premium_percentage = Set(Some(premium_percentage));
    }
    */

    let updated_channel = active_channel.update(&state.db).await?;

    Ok(Json(json!(updated_channel)).into_response())
}

/// Get user's channels
pub async fn get_my_channels(
    State(state): State<AppState>,
    user: auth::TelegramUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    // Check if user exists, otherwise create them
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

    let my_channels = channels::Entity::find()
        .filter(channels::Column::OwnerId.eq(db_user.id))
        .all(&state.db)
        .await?;

    let response: Vec<serde_json::Value> = my_channels.iter().map(|c| json!(c)).collect();

    Ok(Json(response))
}

pub async fn list_channels(
    State(state): State<AppState>,
    Query(filters): Query<ChannelFilterParams>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    use sea_orm::{QuerySelect, JoinType, RelationTrait};

    let mut query = channels::Entity::find()
        .filter(channels::Column::Status.eq("active"));

    // Join with ad formats if price or format filtering is needed
    if filters.min_price.is_some() || filters.max_price.is_some() || filters.format_name.is_some() {
        query = query.join(JoinType::InnerJoin, channels::Relation::ChannelAdFormats.def());
        
        if let Some(min_p) = filters.min_price {
            let min_dec = rust_decimal::Decimal::from_f64_retain(min_p).unwrap_or_default();
            query = query.filter(channel_ad_formats::Column::PriceTon.gte(min_dec));
        }
        if let Some(max_p) = filters.max_price {
            let max_dec = rust_decimal::Decimal::from_f64_retain(max_p).unwrap_or_default();
            query = query.filter(channel_ad_formats::Column::PriceTon.lte(max_dec));
        }
        if let Some(fmt) = filters.format_name {
            query = query.filter(channel_ad_formats::Column::FormatName.contains(&fmt));
        }
    }

    if let Some(min_subs) = filters.min_subscribers {
        query = query.filter(channels::Column::Subscribers.gte(min_subs));
    }
    if let Some(max_subs) = filters.max_subscribers {
        query = query.filter(channels::Column::Subscribers.lte(max_subs));
    }
    if let Some(lang) = filters.language {
        query = query.filter(channels::Column::Language.eq(lang));
    }
    if let Some(min_reach) = filters.min_reach {
        query = query.filter(channels::Column::Reach.gte(min_reach));
    }
    if let Some(category) = filters.category {
        query = query.filter(channels::Column::Category.eq(category));
    }

    // Group by channel ID to avoid duplicates from joins
    query = query.group_by(channels::Column::Id);

    let all_channels = query.all(&state.db).await?;

    // Enhance response with ad formats
    let mut response = Vec::new();
    for channel in all_channels {
        let ad_formats = channel_ad_formats::Entity::find()
            .filter(channel_ad_formats::Column::ChannelId.eq(channel.id))
            .all(&state.db)
            .await?;
            
        let mut channel_val = json!(channel);
        channel_val.as_object_mut().unwrap().insert("ad_formats".to_string(), json!(ad_formats));
        response.push(channel_val);
    }

    Ok(Json(response))
}

/// Add ad format pricing for a channel
pub async fn add_ad_format(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(channel_id): Path<i32>,
    Json(payload): Json<AddAdFormatRequest>,
) -> ApiResult<Response> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let channel = channels::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify ownership
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden("You don't own this channel".to_string()));
    }

    let new_format = channel_ad_formats::ActiveModel {
        channel_id: Set(channel_id),
        format_name: Set(payload.format_name),
        format_description: Set(payload.format_description),
        price_ton: Set(rust_decimal::Decimal::try_from(payload.price_ton).unwrap()),
        ..Default::default()
    };

    let saved_format = new_format.insert(&state.db).await?;

    Ok((StatusCode::CREATED, Json(json!(saved_format))).into_response())
}

/// Get ad formats for a channel
pub async fn get_channel_ad_formats(
    State(state): State<AppState>,
    Path(channel_id): Path<i32>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let formats = channel_ad_formats::Entity::find()
        .filter(channel_ad_formats::Column::ChannelId.eq(channel_id))
        .all(&state.db)
        .await?;

    let response: Vec<serde_json::Value> = formats.iter().map(|f| json!(f)).collect();

    Ok(Json(response))
}
/// Get channels where bot is already an admin, and user might be too
pub async fn get_bot_admin_suggestions(
    State(state): State<AppState>,
    user: auth::TelegramUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let observed = bot_observed_channels::Entity::find()
        .all(&state.db)
        .await?;

    let mut suggestions = Vec::new();

    for obs in observed {
        // Skip if already in the system
        let exists = channels::Entity::find()
            .filter(channels::Column::TelegramChannelId.eq(obs.telegram_chat_id))
            .one(&state.db)
            .await?;
        
        if exists.is_some() {
            continue;
        }

        // Check if user is also an admin
        let chat_id = ChatId(obs.telegram_chat_id);
        let admins_res = state.bot.get_chat_administrators(chat_id).await;
        
        if let Ok(admins) = admins_res {
            let is_user_admin = admins.iter().any(|a| a.user.id.0 as i64 == user.id);
            if is_user_admin {
                suggestions.push(json!({
                    "telegram_id": obs.telegram_chat_id,
                    "title": obs.title,
                    "username": obs.username,
                }));
            }
        }
    }

    Ok(Json(suggestions))
}

/// Invite a PR manager to the channel
pub async fn add_pr_manager(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(channel_id): Path<i32>,
    Json(payload): Json<AddPrManagerRequest>,
) -> ApiResult<Response> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let channel = channels::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Verify ownership
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden("You don't own this channel".to_string()));
    }

    // Find the target user
    let target_user = if let Ok(tg_id) = payload.username_or_id.parse::<i64>() {
        users::Entity::find()
            .filter(users::Column::TelegramId.eq(tg_id))
            .one(&state.db)
            .await?
    } else {
        let cleaned = payload.username_or_id.trim_start_matches('@');
        users::Entity::find()
            .filter(users::Column::Username.eq(cleaned))
            .one(&state.db)
            .await?
    };

    let target = target_user.ok_or_else(|| ApiError::NotFound("User not found in our system. They must start the bot first.".to_string()))?;

    // Check if already an admin
    let existing: Option<channel_admins::Model> = channel_admins::Entity::find()
        .filter(channel_admins::Column::ChannelId.eq(channel_id))
        .filter(channel_admins::Column::UserId.eq(target.id))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("User is already a manager for this channel".to_string()));
    }

    // Add to channel_admins
    let new_admin = channel_admins::ActiveModel {
        channel_id: Set(channel_id),
        user_id: Set(target.id),
        can_post_messages: Set(false), // PR Managers can view but not post by default
        ..Default::default()
    };
    new_admin.insert(&state.db).await?;

    // Send notification
    let msg = format!(
        "📢 You have been added as a PR Manager for channel: **{}** (@{})\n\nYou can now view and manage ads for this channel in the Ad Mini App.",
        channel.title.as_deref().unwrap_or("Untitled"),
        channel.username.as_deref().unwrap_or("private")
    );
    
    let chat_id = ChatId(target.telegram_id);
    let _ = state.bot.send_message(chat_id, msg).await;

    Ok((StatusCode::OK, Json(json!({"status": "success", "message": "PR Manager added successfully"}))).into_response())
}
