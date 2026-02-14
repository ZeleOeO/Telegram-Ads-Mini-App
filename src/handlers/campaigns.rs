use crate::{
    AppState, auth,
    entity::{campaign_applications, campaigns, channels, users},
    models::{
        campaign::*,
        errors::{ApiError, ApiResult},
    },
    services::deal_workflow,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};
use serde_json::json;
use teloxide::prelude::Requester;
use tracing::info;
pub async fn create_campaign(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Json(payload): Json<CreateCampaignRequest>,
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
    let target_languages_str = payload.target_languages.map(|langs| langs.join(","));
    let media_urls_str = payload.media_urls.map(|urls| urls.join(","));
    
    let categories_str = if let Some(cats) = payload.categories {
        if cats.len() > 3 {
             return Err(ApiError::BadRequest("Maximum 3 categories allowed".to_string()));
        }
        Some(serde_json::to_string(&cats).map_err(|e| ApiError::Internal(format!("Serialization error: {}", e)))?)
    } else {
        None
    };

    let new_campaign = campaigns::ActiveModel {
        advertiser_id: Set(db_user.id),
        title: Set(payload.title),
        brief: Set(payload.brief),
        budget_ton: Set(rust_decimal::Decimal::try_from(payload.budget_ton).unwrap()),
        target_subscribers_min: Set(payload.target_subscribers_min),
        target_subscribers_max: Set(payload.target_subscribers_max),
        target_languages: Set(target_languages_str),
        media_urls: Set(media_urls_str),
        categories: Set(categories_str),
        status: Set("active".to_string()),
        ..Default::default()
    };
    let saved_campaign = new_campaign.insert(&state.db).await?;
    info!("User {} created campaign {}", user.id, saved_campaign.id);
    Ok((StatusCode::CREATED, Json(json!(saved_campaign))).into_response())
}
pub async fn edit_campaign(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(campaign_id): Path<i32>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> ApiResult<Response> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    if campaign.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden("You don't own this campaign".to_string()));
    }
    let mut active: campaigns::ActiveModel = campaign.into();
    if let Some(title) = payload.title {
        active.title = Set(title);
    }
    if let Some(brief) = payload.brief {
        active.brief = Set(brief);
    }
    if let Some(bt) = payload.budget_ton {
        active.budget_ton = Set(rust_decimal::Decimal::try_from(bt).unwrap());
    }
    if let Some(min) = payload.target_subscribers_min {
        active.target_subscribers_min = Set(Some(min));
    }
    if let Some(max) = payload.target_subscribers_max {
        active.target_subscribers_max = Set(Some(max));
    }
    if let Some(langs) = payload.target_languages {
        active.target_languages = Set(Some(langs.join(",")));
    }
    if let Some(urls) = payload.media_urls {
        active.media_urls = Set(Some(urls.join(",")));
    }
    if let Some(cats) = payload.categories {
        if cats.len() > 3 {
             return Err(ApiError::BadRequest("Maximum 3 categories allowed".to_string()));
        }
        let cats_str = serde_json::to_string(&cats).map_err(|e| ApiError::Internal(format!("Serialization error: {}", e)))?;
        active.categories = Set(Some(cats_str));
    }
    let updated = active.update(&state.db).await?;
    Ok(Json(json!(updated)).into_response())
}
pub async fn delete_campaign(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(campaign_id): Path<i32>,
) -> ApiResult<Response> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    if campaign.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden("You don't own this campaign".to_string()));
    }
    campaign.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
pub async fn list_campaigns(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let active_campaigns = campaigns::Entity::find()
        .filter(campaigns::Column::Status.eq("active"))
        .all(&state.db)
        .await?;
    let response: Vec<serde_json::Value> = active_campaigns.iter().map(|c| {
        let mut val = json!(c);
        if let Some(cats_str) = &c.categories {
            if let Ok(cats) = serde_json::from_str::<Vec<String>>(cats_str) {
                val["categories"] = json!(cats);
            }
        }
        val
    }).collect();
    Ok(Json(response))
}
pub async fn get_campaign(
    State(state): State<AppState>,
    Path(campaign_id): Path<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    let mut val = json!(campaign);
    if let Some(cats_str) = &campaign.categories {
        if let Ok(cats) = serde_json::from_str::<Vec<String>>(cats_str) {
            val["categories"] = json!(cats);
        }
    }
    Ok(Json(val))
}
pub async fn get_my_campaigns(
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
    let my_campaigns = campaigns::Entity::find()
        .filter(campaigns::Column::AdvertiserId.eq(db_user.id))
        .all(&state.db)
        .await?;
    let response: Vec<serde_json::Value> = my_campaigns.iter().map(|c| {
        let mut val = json!(c);
        if let Some(cats_str) = &c.categories {
            if let Ok(cats) = serde_json::from_str::<Vec<String>>(cats_str) {
                val["categories"] = json!(cats);
            }
        }
        val
    }).collect();
    Ok(Json(response))
}
pub async fn apply_to_campaign(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path((campaign_id, channel_id)): Path<(i32, i32)>,
    Json(payload): Json<ApplyToCampaignRequest>,
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
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    if campaign.status.as_str() != "active" {
        return Err(ApiError::BadRequest("Campaign is not active".to_string()));
    }
    if campaign.advertiser_id == db_user.id {
        return Err(ApiError::Forbidden(
            "You cannot apply to your own campaign".to_string(),
        ));
    }
    let channel = channels::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if channel.owner_id != db_user.id {
        return Err(ApiError::Forbidden(
            "You don't own this channel".to_string(),
        ));
    }
    let existing_application = campaign_applications::Entity::find()
        .filter(campaign_applications::Column::CampaignId.eq(campaign_id))
        .filter(campaign_applications::Column::ChannelId.eq(channel_id))
        .one(&state.db)
        .await?;
    if existing_application.is_some() {
        return Err(ApiError::Conflict(
            "Already applied to this campaign".to_string(),
        ));
    }
    let new_application = campaign_applications::ActiveModel {
        campaign_id: Set(campaign_id),
        channel_id: Set(channel_id),
        proposed_price_ton: Set(
            rust_decimal::Decimal::try_from(payload.proposed_price_ton).unwrap()
        ),
        message: Set(payload.message),
        status: Set("pending".to_string()),
        ..Default::default()
    };
    let saved_application = new_application.insert(&state.db).await?;
    let deal = deal_workflow::create_deal_from_campaign(
        &state.db,
        campaign.id,
        saved_application.id,
    ).await?;
    
    let mut active_app: campaign_applications::ActiveModel = saved_application.clone().into();
    active_app.deal_id = Set(Some(deal.id));
    let updated_application = active_app.update(&state.db).await?;

    info!("Channel {} applied to campaign {}, deal {} created", channel_id, campaign_id, deal.id);
    Ok((StatusCode::CREATED, Json(json!({
        "application": updated_application,
        "deal": deal
    }))).into_response())
}
pub async fn get_campaign_applications(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(campaign_id): Path<i32>,
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
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    if campaign.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "You don't own this campaign".to_string(),
        ));
    }
    let applications = campaign_applications::Entity::find()
        .find_also_related(channels::Entity)
        .filter(campaign_applications::Column::CampaignId.eq(campaign_id))
        .all(&state.db)
        .await?;
    let response: Vec<serde_json::Value> = applications
        .into_iter()
        .map(|(app, channel)| {
            let mut val = json!(app);
            val["price_ton"] = val["proposed_price_ton"].clone();
            if let Some(c) = channel {
                val["channel_title"] = json!(c.title);
                val["channel_username"] = json!(c.username);
                val["channel_subscribers"] = json!(c.subscribers);
            }
            val
        })
        .collect();
    Ok(Json(response))
}
pub async fn update_application_status(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(application_id): Path<i32>,
    Json(payload): Json<serde_json::Value>,
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
    let application = campaign_applications::Entity::find_by_id(application_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Application not found".to_string()))?;
    let campaign = campaigns::Entity::find_by_id(application.campaign_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    if campaign.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "You don't own this campaign".to_string(),
        ));
    }
    let new_status = payload["status"]
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("Missing status field".to_string()))?;
    let mut active_application: campaign_applications::ActiveModel = application.clone().into();
    active_application.status = Set(new_status.to_string());
    let updated_application = active_application.update(&state.db).await?;
    let channel = channels::Entity::find_by_id(application.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    let owner = users::Entity::find_by_id(channel.owner_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Owner not found".to_string()))?;
    if new_status == "accepted" {
        info!(
            "Application {} accepted (deal was already created on apply)",
            application.id
        );
        let msg = format!(
            "Your application for campaign '{}' was ACCEPTED! Open the app to continue the deal.",
            campaign.title
        );
        let _ = state
            .bot
            .send_message(teloxide::types::ChatId(owner.telegram_id), msg)
            .await;
    } else if new_status == "rejected" {
        let msg = format!(
            "Your application for campaign '{}' was rejected by the advertiser.",
            campaign.title
        );
        let _ = state
            .bot
            .send_message(teloxide::types::ChatId(owner.telegram_id), msg)
            .await;
    }
    Ok(Json(json!(updated_application)).into_response())
}
