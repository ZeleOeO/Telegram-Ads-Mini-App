use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCampaignRequest {
    pub title: String,
    pub brief: String,
    pub budget_ton: f64,
    pub target_subscribers_min: Option<i64>,
    pub target_subscribers_max: Option<i64>,
    pub target_languages: Option<Vec<String>>,
    pub media_urls: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct CampaignResponse {
    pub id: i32,
    pub advertiser_id: i32,
    pub title: String,
    pub brief: String,
    pub budget_ton: String,
    pub target_subscribers_min: Option<i64>,
    pub target_subscribers_max: Option<i64>,
    pub target_languages: Option<String>,
    pub status: String,
    pub media_urls: Option<Vec<String>>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ApplyToCampaignRequest {
    pub proposed_price_ton: f64,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct CampaignApplicationResponse {
    pub id: i32,
    pub campaign_id: i32,
    pub channel_id: i32,
    pub proposed_price_ton: String,
    pub message: Option<String>,
    pub status: String,
    pub created_at: String,
}
