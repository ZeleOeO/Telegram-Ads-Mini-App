use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct CreateDealRequest {
    pub channel_id: i32,
    pub ad_format_id: Option<i32>,
    pub proposed_price_ton: Option<f64>,
}
#[derive(Serialize)]
pub struct DealResponse {
    pub id: i32,
    pub advertiser_id: i32,
    pub channel_id: i32,
    pub deal_type: Option<String>,
    pub price_ton: Option<String>,
    pub state: String,
    pub payment_status: Option<String>,
    pub creative_status: Option<String>,
    pub post_link: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize)]
pub struct NegotiationMessage {
    pub message_type: String, 
    pub message_text: Option<String>,
    pub offered_price_ton: Option<f64>,
}
#[derive(Deserialize)]
pub struct SubmitCreativeRequest {
    pub content: String,
    pub media_urls: Option<Vec<String>>,
}
#[derive(Deserialize)]
pub struct ApproveCreativeRequest {
    pub approved: bool,
    pub feedback: Option<String>,
}
#[derive(Serialize)]
pub struct CreativeResponse {
    pub id: i32,
    pub deal_id: i32,
    pub version: i32,
    pub content: String,
    pub media_urls: Option<Vec<String>>,
    pub status: String,
    pub feedback: Option<String>,
    pub created_at: String,
}
#[derive(Serialize)]
pub struct NegotiationResponse {
    pub id: i32,
    pub deal_id: i32,
    pub from_user_id: i32,
    pub message_type: String,
    pub message_text: Option<String>,
    pub offered_price_ton: Option<String>,
    pub created_at: String,
}
#[derive(Deserialize)]
pub struct RejectDealRequest {
    pub reason: String, 
}
#[derive(Deserialize)]
pub struct SubmitDraftRequest {
    pub content: String,
    pub scheduled_post_time: String, 
}
#[derive(Deserialize)]
pub struct ReviewDraftRequest {
    pub approved: bool,
    pub edit_reason: Option<String>, 
}
