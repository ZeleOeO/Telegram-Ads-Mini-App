use crate::{
    AppState, auth,
    entity::{channels, deals, escrow_wallets, transactions, users},
    models::{
        errors::{ApiError, ApiResult},
        payment::*,
    },
    services,
};
use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use tracing::info;
pub async fn initiate_payment(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<InitiatePaymentResponse>> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only advertiser can initiate payment".to_string(),
        ));
    }
    let existing_wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(&state.db)
        .await?;
    let wallet = if let Some(existing) = existing_wallet {
        services::escrow_ton::EscrowWallet {
            address: existing.address,
            balance: existing.balance_ton.to_string().parse().unwrap_or(0.0),
        }
    } else {
        services::escrow_ton::generate_escrow_wallet(&state.db, deal_id).await?
    };
    let amount = deal
        .price_ton
        .ok_or_else(|| ApiError::BadRequest("Deal has no price set".to_string()))?;
    Ok(Json(InitiatePaymentResponse {
        escrow_address: wallet.address,
        amount_ton: amount.to_string(),
        deal_id,
    }))
}
pub async fn get_payment_status(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<PaymentStatusResponse>> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
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
            "Not authorized to view payment status".to_string(),
        ));
    }
    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;
    let payment_status = deal.payment_status;
    Ok(Json(PaymentStatusResponse {
        deal_id,
        payment_status,
        escrow_balance: wallet.balance_ton.to_string(),
    }))
}
pub async fn verify_payment(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if deal.advertiser_id != db_user.id {
        return Err(ApiError::Forbidden(
            "Only advertiser can verify payment".to_string(),
        ));
    }
    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;
    let expected_amount = deal
        .price_ton
        .ok_or_else(|| ApiError::BadRequest("Deal has no price set".to_string()))?
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let verified =
        services::escrow_ton::verify_payment(&state.db, &wallet.address, expected_amount).await?;
    if verified {
        let mut active_deal: deals::ActiveModel = deal.into();
        active_deal.payment_status = Set("confirmed".to_string());
        active_deal.state = Set("payment_received".to_string());
        active_deal.update(&state.db).await?;
        info!("Payment verified for deal {}", deal_id);
    }
    Ok(Json(json!({
        "verified": verified,
        "deal_id": deal_id,
    })))
}
pub async fn get_transactions(
    State(state): State<AppState>,
    user: auth::TelegramUser,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let db_user = users::Entity::find()
        .filter(users::Column::TelegramId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
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
            "Not authorized to view transactions".to_string(),
        ));
    }
    let txs = transactions::Entity::find()
        .filter(transactions::Column::DealId.eq(deal_id))
        .all(&state.db)
        .await?;
    let response: Vec<serde_json::Value> = txs.iter().map(|tx| json!(tx)).collect();
    Ok(Json(response))
}
pub async fn release_funds(
    State(state): State<AppState>,
    Path(deal_id): Path<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    let deal = deals::Entity::find_by_id(deal_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let channel = channels::Entity::find_by_id(deal.channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    let channel_owner = users::Entity::find_by_id(channel.owner_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel owner not found".to_string()))?;
    let to_address = channel_owner
        .ton_wallet_address
        .ok_or_else(|| ApiError::BadRequest("Channel owner has no wallet address".to_string()))?;
    let tx_hash = services::escrow_ton::release_funds(&state.db, deal_id, &to_address).await?;
    use sea_orm::{ActiveModelTrait, Set};
    let mut active_deal: deals::ActiveModel = deal.into();
    active_deal.state = Set("released".to_string());
    active_deal.funds_released_at = Set(Some(chrono::Utc::now().naive_utc()));
    active_deal.update(&state.db).await?;
    info!("Funds released for deal {}", deal_id);
    Ok(Json(json!({
        "transaction_hash": tx_hash,
        "deal_id": deal_id,
        "status": "released"
    })))
}
