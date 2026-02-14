use crate::{
    AppState, auth,
    entity::{campaigns, channels, deal_creatives, deal_negotiations, deals, users},
    models::{
        deal::*,
        errors::{ApiError, ApiResult},
    },
    services,
    helpers::notifications,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, QueryOrder};
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
        .order_by_desc(deals::Column::CreatedAt)
        .all(&state.db)
        .await?;
    let mut response: Vec<serde_json::Value> = Vec::new();
    for (deal, channel) in my_deals {
        let mut val = json!(deal);
        if let Ok(Some(advertiser)) = users::Entity::find_by_id(deal.advertiser_id)
            .one(&state.db)
            .await
        {
            val["advertiser_username"] = json!(advertiser.username);
            val["advertiser_telegram_id"] = json!(advertiser.telegram_id);
        }
        if let Ok(Some(owner_user)) = users::Entity::find_by_id(deal.owner_id)
            .one(&state.db)
            .await
        {
            val["owner_telegram_id"] = json!(owner_user.telegram_id);
        }
        if let Ok(Some(applicant_user)) = users::Entity::find_by_id(deal.applicant_id)
            .one(&state.db)
            .await
        {
            val["applicant_telegram_id"] = json!(applicant_user.telegram_id);
        }
        if let Some(c) = channel.clone() {
            val["channel_title"] = json!(c.title);
            val["channel_username"] = json!(c.username);
            val["channel_owner_id"] = json!(c.owner_id);
            if let Ok(Some(channel_owner)) = users::Entity::find_by_id(c.owner_id)
                .one(&state.db)
                .await
            {
                val["channel_owner_username"] = json!(channel_owner.username);
                val["channel_owner_telegram_id"] = json!(channel_owner.telegram_id);
            }
        }
        if let Some(campaign_id) = deal.campaign_id {
            if let Ok(Some(campaign)) = campaigns::Entity::find_by_id(campaign_id).one(&state.db).await {
                val["campaign_title"] = json!(campaign.title);
            }
        }
        response.push(val);
    }
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
    if let Ok(Some(advertiser)) = users::Entity::find_by_id(deal.advertiser_id)
        .one(&state.db)
        .await
    {
        response["advertiser_username"] = json!(advertiser.username);
    }
    if let Ok(Some(owner)) = users::Entity::find_by_id(channel.owner_id)
        .one(&state.db)
        .await
    {
        response["channel_owner_username"] = json!(owner.username);
    }
    if let Some(campaign_id) = deal.campaign_id {
        if let Ok(Some(campaign)) = campaigns::Entity::find_by_id(campaign_id).one(&state.db).await {
            response["campaign_title"] = json!(campaign.title);
        }
    }
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
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.applicant_id == db_user.id {
        return Err(ApiError::Forbidden(
            "Only the Owner (recipient) can accept deals. You are the applicant.".to_string(),
        ));
    }
    
    
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
        
    if channel.owner_id == db_user.id {
        crate::helpers::utils::verify_channel_admin(
            &state.bot, 
            channel.telegram_channel_id, 
            db_user.telegram_id
        ).await?;
    }
    if deal.state != "pending" {
        return Err(ApiError::BadRequest(
            format!("Cannot accept deal from state: {}", deal.state),
        ));
    }
    let updated_deal = services::deal_workflow::transition_state(
        &state.db,
        deal_id,
        services::deal_workflow::DealState::Accepted,
    )
    .await?;
    let escrow_wallet = services::escrow_ton::generate_escrow_wallet(&state.db, deal_id).await?;
    let mut active_deal: deals::ActiveModel = updated_deal.into();
    active_deal.escrow_address = Set(Some(escrow_wallet.address.clone()));
    active_deal.state = Set("awaiting_payment".to_string());
    let final_deal = active_deal.update(&state.db).await?;
        
    let deal_name = format!("Deal for {}", channel.title.as_deref().unwrap_or("Untitled Channel"));

    if let Some(advertiser) = users::Entity::find_by_id(deal.advertiser_id).one(&state.db).await? {
        let _ = notifications::notify_deal_status_change(
            &state.bot, 
            advertiser.telegram_id, 
            &deal_name, 
            "accepted", 
            None
        ).await;
    }
    Ok(Json(json!(final_deal)).into_response())
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
pub async fn mark_paid(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the advertiser can mark payment as sent".to_string(),
        ));
    }
    if deal.state != "awaiting_payment" {
        return Err(ApiError::BadRequest(
            format!("Cannot mark as paid from state: {}", deal.state),
        ));
    }

    use crate::entity::escrow_wallets;
    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;
        
    let expected_amount = deal.price_ton
        .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
        
    let verified = services::escrow_ton::verify_payment(&state.db, &wallet.address, expected_amount).await?;
    
    if !verified {
        return Err(ApiError::BadRequest(
            "Payment not detected on chain. It may take a minute to process, please try again shortly.".to_string(),
        ));
    }

    let updated_deal = services::deal_workflow::transition_state(
        &state.db,
        deal_id,
        services::deal_workflow::DealState::Drafting,
    )
    .await?;
    let mut active_deal: deals::ActiveModel = updated_deal.into();
    active_deal.payment_status = Set("confirmed".to_string());
    let final_deal = active_deal.update(&state.db).await?;
    
    let channel = channels::Entity::find_by_id(final_deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    
    if let Some(owner) = users::Entity::find_by_id(channel.owner_id).one(&state.db).await? {
        let deal_name = format!("Deal for {}", channel.title.as_deref().unwrap_or("Untitled Channel"));
        let _ = notifications::notify_deal_status_change(
            &state.bot, 
            owner.telegram_id, 
            &deal_name, 
            "payment_confirmed", 
            None
        ).await;
    }

    Ok(Json(json!(final_deal)).into_response())
}
pub async fn confirm_payment(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the channel owner can confirm payment".to_string(),
        ));
    }
    crate::helpers::utils::verify_channel_admin(
        &state.bot,
        channel.telegram_channel_id,
        db_user.telegram_id
    ).await?;
    if deal.state != "awaiting_payment" {
        return Err(ApiError::BadRequest(
            format!("Cannot confirm payment from state: {}", deal.state),
        ));
    }
    
    
    use crate::entity::escrow_wallets;
    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;
        
    let expected_amount = deal.price_ton
        .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
        
    let verified = services::escrow_ton::verify_payment(&state.db, &wallet.address, expected_amount).await?;
    if !verified {
        return Err(ApiError::BadRequest(
            "Payment not found on chain. Please ensure funds are sent to the escrow wallet.".to_string(),
        ));
    }

    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.payment_status = Set("confirmed".to_string());
    active_deal.state = Set("drafting".to_string());
    active_deal.updated_at = Set(chrono::Utc::now().naive_utc());
    let final_deal = active_deal.update(&state.db).await?;
    Ok(Json(json!(final_deal)).into_response())
}
pub async fn mark_posted(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the channel owner can mark as posted".to_string(),
        ));
    }
    crate::helpers::utils::verify_channel_admin(
        &state.bot,
        channel.telegram_channel_id,
        db_user.telegram_id
    ).await?;
    if deal.state != "scheduled" {
        return Err(ApiError::BadRequest(
            format!("Cannot mark as posted from state: {}", deal.state),
        ));
    }
    let updated_deal = services::deal_workflow::transition_state(
        &state.db,
        deal_id,
        services::deal_workflow::DealState::Published,
    )
    .await?;
    Ok(Json(json!(updated_deal)).into_response())
}
pub async fn verify_post(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the advertiser can verify the post".to_string(),
        ));
    }
    if deal.state != "published" {
        return Err(ApiError::BadRequest(
            format!("Cannot verify from state: {}", deal.state),
        ));
    }
    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.payment_status = Set("released".to_string());
    active_deal.state = Set("completed".to_string());
    active_deal.funds_released_at = Set(Some(chrono::Utc::now().naive_utc()));
    let final_deal = active_deal.update(&state.db).await?;
    Ok(Json(json!(final_deal)).into_response())
}
pub async fn reject_deal(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<RejectDealRequest>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.applicant_id == db_user.id {
        return Err(ApiError::Forbidden(
            "Only the Owner (recipient) can reject deals. You are the applicant.".to_string(),
        ));
    }
    
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;

    if channel.owner_id == db_user.id {
        crate::helpers::utils::verify_channel_admin(
            &state.bot, 
            channel.telegram_channel_id, 
            db_user.telegram_id
        ).await?;
    }
    if deal.state != "pending" {
        return Err(ApiError::BadRequest(
            format!("Cannot reject deal from state: {}", deal.state),
        ));
    }
    if payload.reason.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Rejection reason is required".to_string(),
        ));
    }
    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.state = Set("rejected".to_string());
    active_deal.rejection_reason = Set(Some(payload.reason.clone()));
    active_deal.updated_at = Set(chrono::Utc::now().naive_utc());
    let final_deal = active_deal.update(&state.db).await?;
        
    let deal_name = format!("Deal for {}", channel.title.as_deref().unwrap_or("Untitled Channel"));

    if let Some(applicant) = users::Entity::find_by_id(final_deal.applicant_id).one(&state.db).await? {
        let _ = notifications::notify_deal_status_change(
            &state.bot, 
            applicant.telegram_id, 
            &deal_name, 
            "rejected", 
            Some(&payload.reason)
        ).await;
    }
    Ok(Json(json!(final_deal)).into_response())
}
pub async fn submit_draft(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<SubmitDraftRequest>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the Channel Owner can submit drafts".to_string(),
        ));
    }
    crate::helpers::utils::verify_channel_admin(
        &state.bot,
        channel.telegram_channel_id,
        db_user.telegram_id
    ).await?;
    if deal.state != "drafting" && deal.state != "reviewing" {
        return Err(ApiError::BadRequest(
            format!("Cannot submit draft from state: {}. Payment must be confirmed first.", deal.state),
        ));
    }
    let scheduled_time = chrono::NaiveDateTime::parse_from_str(&payload.scheduled_post_time, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&payload.scheduled_post_time, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&payload.scheduled_post_time, "%Y-%m-%dT%H:%M"))
        .map_err(|_| ApiError::BadRequest("Invalid scheduled_post_time format. Use ISO 8601.".to_string()))?;
    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.post_content = Set(Some(payload.content));
    active_deal.scheduled_post_time = Set(Some(scheduled_time));
    active_deal.state = Set("reviewing".to_string());
    active_deal.creative_status = Set("submitted".to_string());
    active_deal.creative_submitted_at = Set(Some(chrono::Utc::now().naive_utc()));
    active_deal.edit_request_reason = Set(None); 
    active_deal.updated_at = Set(chrono::Utc::now().naive_utc());
    let final_deal = active_deal.update(&state.db).await?;
    let deal_name = format!("Deal for {}", channel.title.as_deref().unwrap_or("Untitled Channel"));
    if let Some(advertiser) = users::Entity::find_by_id(final_deal.advertiser_id).one(&state.db).await? {
        let _ = notifications::notify_deal_status_change(
            &state.bot, 
            advertiser.telegram_id, 
            &deal_name, 
            "draft_submitted", 
            None
        ).await;
    }
    Ok(Json(json!(final_deal)).into_response())
}
pub async fn review_draft(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
    Json(payload): Json<ReviewDraftRequest>,
) -> ApiResult<Response> {
    let db_user = services::user_sync::sync_user(&state.db, &user).await?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only the Advertiser can review drafts".to_string(),
        ));
    }
    if deal.state != "reviewing" {
        return Err(ApiError::BadRequest(
            format!("Cannot review draft from state: {}", deal.state),
        ));
    }
    let mut active_deal: deals::ActiveModel = deal.into();
    if payload.approved {
        active_deal.state = Set("scheduled".to_string());
        active_deal.creative_status = Set("approved".to_string());
        active_deal.creative_approved_at = Set(Some(chrono::Utc::now().naive_utc()));
    } else {
        let edit_reason = payload.edit_reason.clone().ok_or_else(|| {
            ApiError::BadRequest("Edit reason is required when not approving".to_string())
        })?;
        if edit_reason.trim().is_empty() {
            return Err(ApiError::BadRequest("Edit reason cannot be empty".to_string()));
        }
        active_deal.state = Set("drafting".to_string());
        active_deal.creative_status = Set("edit_requested".to_string());
        active_deal.edit_request_reason = Set(Some(edit_reason));
    }
    active_deal.updated_at = Set(chrono::Utc::now().naive_utc());
    let final_deal = active_deal.update(&state.db).await?;
    let channel = channels::Entity::find_by_id(final_deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if let Some(owner) = users::Entity::find_by_id(channel.owner_id).one(&state.db).await? {
        let status_type = if payload.approved { "review_approved" } else { "review_rejected" };
        let reason = payload.edit_reason.as_deref();
        let deal_name = format!("Deal for {}", channel.title.as_deref().unwrap_or("Untitled Channel"));
        
        let _ = notifications::notify_deal_status_change(
            &state.bot, 
            owner.telegram_id, 
            &deal_name, 
            status_type, 
            reason
        ).await;
    }
    Ok(Json(json!(final_deal)).into_response())
}
