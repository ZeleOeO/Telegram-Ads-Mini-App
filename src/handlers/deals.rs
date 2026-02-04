use crate::{
    AppState, auth,
    entity::{channels, deal_creatives, deal_negotiations, deals, users},
    models::{
        deal::*,
        errors::{ApiError, ApiResult},
    },
    services,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use teloxide::types::ChatId;

pub async fn create_deal(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Json(payload): Json<CreateDealRequest>,
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

    let price_ton = payload
        .proposed_price_ton
        .map(|p| rust_decimal::Decimal::try_from(p).ok())
        .flatten();

    let channel = channels::Entity::find_by_id(payload.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    // Prevent owner from starting a deal with their own channel
    if channel.owner_id == db_user.id {
        return Err(ApiError::Forbidden(
            "You cannot start a deal with your own channel".to_string(),
        ));
    }

    let deal = services::deal_workflow::create_deal_from_listing(
        &state.db,
        db_user.id,
        payload.channel_id,
        payload.ad_format_id,
        price_ton,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(json!(deal))).into_response())
}

pub async fn get_my_deals(
    State(state): State<AppState>,
    user: auth::TelegramUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
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

    let my_deals = deals::Entity::find()
        .find_also_related(channels::Entity)
        .filter(
            sea_orm::Condition::any()
                .add(deals::Column::AdvertiserId.eq(db_user.id))
                .add(channels::Column::OwnerId.eq(db_user.id)),
        )
        .all(&state.db)
        .await?;

    let response: Vec<serde_json::Value> = my_deals
        .into_iter()
        .map(|(deal, channel)| {
            let mut val = json!(deal);
            if let Some(c) = channel {
                val["channel_title"] = json!(c.title);
                val["channel_username"] = json!(c.username);
                val["channel_owner_id"] = json!(c.owner_id);
            }
            val
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_deal(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<serde_json::Value>> {
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

    let deal_with_channel = deals::Entity::find_by_id(deal_id)
        .find_also_related(channels::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let (deal, channel) = deal_with_channel;
    let channel = channel.ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized to view this deal".to_string(),
        ));
    }

    let mut response = json!(deal);
    response["channel_title"] = json!(channel.title);
    response["channel_username"] = json!(channel.username);
    response["channel_owner_id"] = json!(channel.owner_id);

    Ok(Json(response))
}

pub async fn send_negotiation(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<NegotiationMessage>,
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized for this deal".to_string(),
        ));
    }

    let offered_price = payload
        .offered_price_ton
        .map(|p| rust_decimal::Decimal::try_from(p).ok())
        .flatten();

    let new_negotiation = deal_negotiations::ActiveModel {
        deal_id: Set(deal_id),
        from_user_id: Set(db_user.id),
        message_type: Set(payload.message_type),
        message_text: Set(payload.message_text),
        offered_price_ton: Set(offered_price),
        ..Default::default()
    };

    let saved_negotiation = new_negotiation.insert(&state.db).await?;

    Ok((StatusCode::CREATED, Json(json!(saved_negotiation))).into_response())
}

pub async fn accept_deal(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized to accept this deal".to_string(),
        ));
    }

    let deal = services::deal_workflow::transition_state(
        &state.db,
        deal_id,
        services::deal_workflow::DealState::AwaitingPayment,
    )
    .await?;

    Ok(Json(json!(deal)).into_response())
}

pub async fn submit_creative(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<SubmitCreativeRequest>,
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let existing_creatives = deal_creatives::Entity::find()
        .filter(deal_creatives::Column::DealId.eq(deal_id))
        .all(&state.db)
        .await?;

    let version = existing_creatives.len() as i32 + 1;

    let media_urls_json = payload
        .media_urls
        .map(|urls| serde_json::to_value(urls).ok())
        .flatten();

    let new_creative = deal_creatives::ActiveModel {
        deal_id: Set(deal_id),
        version: Set(version),
        content: Set(payload.content.clone()),
        media_urls: Set(media_urls_json),
        submitted_by: Set(db_user.id),
        status: Set("submitted".to_string()),
        ..Default::default()
    };

    let saved_creative = new_creative.insert(&state.db).await?;

    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.creative_status = Set("submitted".to_string());
    active_deal.creative_submitted_at = Set(Some(chrono::Utc::now().naive_utc()));
    active_deal.post_content = Set(Some(payload.content));
    active_deal.state = Set("creative_submitted".to_string());
    active_deal.update(&state.db).await?;

    Ok((StatusCode::CREATED, Json(json!(saved_creative))).into_response())
}

pub async fn review_creative(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<ApproveCreativeRequest>,
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized to review creative".to_string(),
        ));
    }

    let deal = services::deal_workflow::handle_creative_approval(
        &state.db,
        deal_id,
        payload.approved,
        payload.feedback,
    )
    .await?;

    Ok(Json(json!(deal)).into_response())
}

pub async fn get_negotiations(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized to view negotiations".to_string(),
        ));
    }

    let negotiations = deal_negotiations::Entity::find()
        .filter(deal_negotiations::Column::DealId.eq(deal_id))
        .all(&state.db)
        .await?;

    let response: Vec<serde_json::Value> = negotiations.iter().map(|n| json!(n)).collect();

    Ok(Json(response))
}

pub async fn trigger_auto_post(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
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

    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;

    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if deal.advertiser_id != db_user.id && channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Not authorized to trigger post".to_string(),
        ));
    }

    let content = deal
        .post_content
        .ok_or_else(|| ApiError::BadRequest("No content to post".to_string()))?;

    let channel_id = ChatId(channel.telegram_channel_id);

    let post_link = services::auto_post::execute_auto_post(
        &state.db, &state.bot, deal_id, channel_id, &content,
    )
    .await?;

    Ok(Json(json!({"post_link": post_link})).into_response())
}
