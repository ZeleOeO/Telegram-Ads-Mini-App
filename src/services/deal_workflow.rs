use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::info;
use crate::entity::{channel_ad_formats, channels, deals};
use crate::models::errors::{ApiError, ApiResult};
#[derive(Debug, Clone, PartialEq)]
pub enum DealState {
    Pending,        
    Rejected,       
    Accepted,       
    Drafting,       
    Reviewing,      
    AwaitingPayment, 
    Scheduled,      
    Published,      
    Cancelled,
    Refunded,
    Completed,
}
impl DealState {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "rejected" => Self::Rejected,
            "accepted" => Self::Accepted,
            "drafting" => Self::Drafting,
            "reviewing" => Self::Reviewing,
            "awaiting_payment" => Self::AwaitingPayment,
            "scheduled" => Self::Scheduled,
            "published" => Self::Published,
            "cancelled" => Self::Cancelled,
            "refunded" => Self::Refunded,
            "completed" => Self::Completed,
            "negotiating" => Self::Pending,
            "draft" => Self::Pending,
            "payment_received" => Self::Drafting,
            "creative_submitted" => Self::Reviewing,
            "creative_approved" => Self::AwaitingPayment,
            "posted" => Self::Published,
            "verified" => Self::Published,
            "released" => Self::Published,
            _ => Self::Pending,
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Rejected => "rejected",
            Self::Accepted => "accepted",
            Self::Drafting => "drafting",
            Self::Reviewing => "reviewing",
            Self::AwaitingPayment => "awaiting_payment",
            Self::Scheduled => "scheduled",
            Self::Published => "published",
            Self::Cancelled => "cancelled",
            Self::Refunded => "refunded",
            Self::Completed => "completed",
        }
    }
}
pub async fn create_deal_from_listing(
    db: &DatabaseConnection,
    advertiser_id: i32,
    channel_id: i32,
    ad_format_id: Option<i32>,
    proposed_price: Option<rust_decimal::Decimal>,
) -> ApiResult<deals::Model> {
    let channel = channels::Entity::find_by_id(channel_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    if channel.status.as_str() != "active" {
        return Err(ApiError::BadRequest("Channel is not active".to_string()));
    }
    let price_ton = match proposed_price {
        Some(price) => Some(price),
        None => {
            if let Some(format_id) = ad_format_id {
                channel_ad_formats::Entity::find_by_id(format_id)
                    .one(db)
                    .await?
                    .map(|fmt| fmt.price_ton)
            } else {
                None
            }
        }
    };
    let timeout_at = Utc::now().naive_utc() + Duration::days(7);
    let new_deal = deals::ActiveModel {
        advertiser_id: Set(advertiser_id),
        channel_id: Set(channel_id),
        owner_id: Set(channel.owner_id),        
        applicant_id: Set(advertiser_id),       
        is_campaign_application: Set(false),    
        deal_type: Set("channel_listing".to_string()),
        price_ton: Set(price_ton),
        ad_format_id: Set(ad_format_id),
        state: Set("pending".to_string()),
        payment_status: Set("pending".to_string()),
        creative_status: Set("draft".to_string()),
        timeout_at: Set(Some(timeout_at)),
        ..Default::default()
    };
    let deal = new_deal.insert(db).await?;
    info!("Created deal from listing: {} (owner: {}, applicant: {})", deal.id, channel.owner_id, advertiser_id);
    Ok(deal)
}
pub async fn create_deal_from_campaign(
    db: &DatabaseConnection,
    campaign_id: i32,
    application_id: i32,
) -> ApiResult<deals::Model> {
    use crate::entity::{campaign_applications, campaigns};
    let campaign = campaigns::Entity::find_by_id(campaign_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Campaign not found".to_string()))?;
    let application = campaign_applications::Entity::find_by_id(application_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Application not found".to_string()))?;
    let channel = channels::Entity::find_by_id(application.channel_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
    let timeout_at = Utc::now().naive_utc() + Duration::days(7);
    let new_deal = deals::ActiveModel {
        advertiser_id: Set(campaign.advertiser_id),
        channel_id: Set(application.channel_id),
        owner_id: Set(campaign.advertiser_id),     
        applicant_id: Set(channel.owner_id),       
        is_campaign_application: Set(true),        
        deal_type: Set("campaign_request".to_string()),
        price_ton: Set(Some(application.proposed_price_ton)),
        state: Set("pending".to_string()),
        payment_status: Set("pending".to_string()),
        creative_status: Set("draft".to_string()),
        timeout_at: Set(Some(timeout_at)),
        campaign_id: Set(Some(campaign.id)),
        ..Default::default()
    };
    let deal = new_deal.insert(db).await?;
    info!("Created deal from campaign application: {} (owner: {}, applicant: {})", deal.id, campaign.advertiser_id, channel.owner_id);
    Ok(deal)
}
pub async fn transition_state(
    db: &DatabaseConnection,
    deal_id: i32,
    new_state: DealState,
) -> ApiResult<deals::Model> {
    let deal = deals::Entity::find_by_id(deal_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let current_state = DealState::from_str(&deal.state);
    let mut active_deal: deals::ActiveModel = deal.clone().into();
    active_deal.state = Set(new_state.as_str().to_string());
    active_deal.updated_at = Set(Utc::now().naive_utc());
    let updated_deal = active_deal.update(db).await?;
    info!(
        "Deal {} transitioned from {:?} to {:?}",
        deal_id, current_state, new_state
    );
    Ok(updated_deal)
}
pub async fn check_timeout(db: &DatabaseConnection, deal_id: i32) -> ApiResult<bool> {
    let deal = deals::Entity::find_by_id(deal_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    if let Some(timeout_at) = deal.timeout_at {
        let now = Utc::now().naive_utc();
        if now > timeout_at {
            info!("Deal {} has timed out", deal_id);
            return Ok(true);
        }
    }
    Ok(false)
}
pub async fn handle_creative_approval(
    db: &DatabaseConnection,
    deal_id: i32,
    approved: bool,
    feedback: Option<String>,
) -> ApiResult<deals::Model> {
    use crate::entity::deal_creatives;
    use sea_orm::QueryOrder;
    let deal = deals::Entity::find_by_id(deal_id)
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Deal not found".to_string()))?;
    let latest_creative = deal_creatives::Entity::find()
        .filter(deal_creatives::Column::DealId.eq(deal_id))
        .order_by_desc(deal_creatives::Column::Version)
        .one(db)
        .await?;
    let mut active_deal: deals::ActiveModel = deal.into();
    if approved {
        active_deal.creative_status = Set("approved".to_string());
        active_deal.creative_approved_at = Set(Some(Utc::now().naive_utc()));
        active_deal.state = Set("creative_approved".to_string());
        if let Some(creative) = latest_creative {
            let mut creative_am: deal_creatives::ActiveModel = creative.into();
            creative_am.status = Set("approved".to_string());
            creative_am.update(db).await?;
        }
    } else {
        active_deal.creative_status = Set("revision_requested".to_string());
        active_deal.state = Set("revision_requested".to_string());
        if let Some(creative) = latest_creative {
            let mut creative_am: deal_creatives::ActiveModel = creative.into();
            creative_am.status = Set("rejected".to_string());
            creative_am.feedback = Set(feedback);
            creative_am.update(db).await?;
        }
    }
    active_deal.updated_at = Set(Utc::now().naive_utc());
    let updated_deal = active_deal.update(db).await?;
    info!(
        "Creative for deal {} {}",
        deal_id,
        if approved {
            "approved"
        } else {
            "revision requested"
        }
    );
    Ok(updated_deal)
}
pub async fn auto_cancel_stale_deals(db: &DatabaseConnection) -> ApiResult<usize> {
    let now = Utc::now().naive_utc();
    let stale_deals = deals::Entity::find()
        .filter(deals::Column::TimeoutAt.lte(now))
        .filter(deals::Column::State.is_in(["draft", "negotiating", "creative_submitted"]))
        .all(db)
        .await?;
    let mut cancelled_count = 0;
    for deal in stale_deals {
        let mut active_deal: deals::ActiveModel = deal.clone().into();
        active_deal.state = Set("cancelled".to_string());
        active_deal.cancelled_at = Set(Some(now));
        active_deal.update(db).await?;
        info!("Auto-cancelled stale deal {}", deal.id);
        cancelled_count += 1;
    }
    Ok(cancelled_count)
}
