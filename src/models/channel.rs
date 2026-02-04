use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AddChannelPayload {
    pub username: String,
}

#[derive(Deserialize)]
pub struct UpdateChannelRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub reach: Option<i64>,
    pub language: Option<String>,
    pub premium_percentage: Option<f32>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct AddAdFormatRequest {
    pub format_name: String,
    pub format_description: Option<String>,
    pub price_ton: f64,
}

#[derive(Serialize)]
pub struct AdFormatResponse {
    pub id: i32,
    pub format_name: String,
    pub format_description: Option<String>,
    pub price_ton: String,
}

#[derive(Serialize)]
pub struct ChannelResponse {
    pub id: i32,
    pub owner_id: i32,
    pub telegram_channel_id: i64,
    pub title: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub subscribers: Option<i64>,
    pub reach: Option<i64>,
    pub language: Option<String>,
    pub premium_percentage: Option<f32>,
    pub status: Option<String>,
    pub ad_formats: Vec<AdFormatResponse>,
}

#[derive(Deserialize)]
pub struct ChannelFilterParams {
    pub min_subscribers: Option<i64>,
    pub max_subscribers: Option<i64>,
    pub language: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_reach: Option<i64>,
    pub format_name: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct AddPrManagerRequest {
    pub username_or_id: String,
}
