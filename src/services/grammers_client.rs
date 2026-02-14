use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use grammers_client::{Client, Config, InitParams};
use grammers_session::{PackedType, Session};
use grammers_tl_types as tl;
use tl::enums::{StatsAbsValueAndPrev, StatsPercentValue, InputChannel};
use tl::types::{InputChannel as InputChannelType};
use tl::functions::stats::GetBroadcastStats;
use tl::enums::stats::BroadcastStats;
use crate::models::errors::{ApiError, ApiResult};

const SESSION_FILE: &str = "telegram_session.session";

#[derive(Debug, Clone, Default)]
pub struct ChannelAnalytics {
    pub subscribers: i64,
    pub reach: Option<i64>,
    pub views_per_post: Option<f64>,
    pub shares_per_post: Option<f64>,
    pub reactions_per_post: Option<f64>,
    pub enabled_notifications_percent: Option<f64>,
    pub languages: Option<String>,
    pub premium_percentage: Option<f32>,
}

fn extract_abs_value(v: &StatsAbsValueAndPrev) -> f64 {
    match v {
        StatsAbsValueAndPrev::Prev(inner) => inner.current,
    }
}

fn extract_percent_value(v: &StatsPercentValue) -> f64 {
    match v {
        StatsPercentValue::Value(inner) => inner.part * 100.0,
    }
}

pub struct GrammersClient {
    client: Arc<RwLock<Option<Client>>>,
    api_id: i32,
    api_hash: String,
}

impl GrammersClient {
    pub fn new(api_id: i32, api_hash: String) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            api_id,
            api_hash,
        }
    }
    
    pub async fn connect(&self) -> ApiResult<()> {
        let session = Session::load_file_or_create(SESSION_FILE)
            .map_err(|e| ApiError::Internal(format!("Session error: {}", e)))?;
        let client = Client::connect(Config {
            session,
            api_id: self.api_id,
            api_hash: self.api_hash.clone(),
            params: InitParams::default(),
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Connection error: {}", e)))?;
        let is_authorized = client.is_authorized().await
            .map_err(|e| ApiError::Internal(format!("Auth check error: {}", e)))?;
        if !is_authorized {
            return Err(ApiError::Forbidden(
                "Telegram session not authorized. Run login flow first.".to_string()
            ));
        }
        let mut guard = self.client.write().await;
        *guard = Some(client);
        info!("Grammers client connected and authorized");
        Ok(())
    }

    pub async fn get_broadcast_stats(&self, channel_username: &str) -> ApiResult<ChannelAnalytics> {
        let guard = self.client.read().await;
        let client = guard.as_ref().ok_or_else(|| {
            ApiError::Internal("Grammers client not connected".to_string())
        })?;
        let chat = client
            .resolve_username(channel_username)
            .await
            .map_err(|e| ApiError::Internal(format!("Resolve channel error: {}", e)))?
            .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
        let packed = chat.pack();
        let input_channel = match packed.ty {
            PackedType::Broadcast | PackedType::Megagroup => {
                InputChannel::Channel(InputChannelType {
                    channel_id: packed.id,
                    access_hash: packed.access_hash.unwrap_or(0),
                })
            }
            _ => return Err(ApiError::BadRequest("Not a channel or supergroup".to_string())),
        };
        let request = GetBroadcastStats {
            dark: false,
            channel: input_channel,
        };
        let stats: BroadcastStats = client
            .invoke(&request)
            .await
            .map_err(|e| {
                warn!("Failed to get broadcast stats: {}", e);
                ApiError::Internal(format!("Stats error: {}", e))
            })?;
        let analytics = match stats {
            BroadcastStats::Stats(s) => {
                let followers = extract_abs_value(&s.followers);
                let views = extract_abs_value(&s.views_per_post);
                let shares = extract_abs_value(&s.shares_per_post);
                let reactions = extract_abs_value(&s.reactions_per_post);
                let enabled_pct = extract_percent_value(&s.enabled_notifications);
                ChannelAnalytics {
                    subscribers: followers as i64,
                    reach: Some((views * 0.8) as i64),
                    views_per_post: Some(views),
                    shares_per_post: Some(shares),
                    reactions_per_post: Some(reactions),
                    enabled_notifications_percent: Some(enabled_pct),
                    languages: None,
                    premium_percentage: None,
                }
            }
        };
        Ok(analytics)
    }

    pub async fn verify_message_exists(&self, channel_username: &str, message_id: i32) -> ApiResult<bool> {
        let guard = self.client.read().await;
        let client = guard.as_ref().ok_or_else(|| {
            ApiError::Internal("Grammers client not connected".to_string())
        })?;
        let chat = client
            .resolve_username(channel_username)
            .await
            .map_err(|e| ApiError::Internal(format!("Resolve channel error: {}", e)))?
            .ok_or_else(|| ApiError::NotFound("Channel not found".to_string()))?;
        let messages = client.get_messages_by_id(chat, &[message_id]).await
            .map_err(|e| ApiError::Internal(format!("Fetch message error: {}", e)))?;
        if let Some(msg) = messages.first() {
            return Ok(msg.is_some());
        }
        Ok(false)
    }

    pub async fn disconnect(&self) {
        let mut guard = self.client.write().await;
        if let Some(client) = guard.take() {
            if let Err(e) = client.session().save_to_file(SESSION_FILE) {
                error!("Failed to save session: {}", e);
            }
        }
    }
}

impl Default for GrammersClient {
    fn default() -> Self {
        let api_id = std::env::var("TELEGRAM_API_ID")
            .expect("TELEGRAM_API_ID env var required")
            .parse()
            .expect("TELEGRAM_API_ID must be an integer");
        let api_hash = std::env::var("TELEGRAM_API_HASH")
            .expect("TELEGRAM_API_HASH env var required");
        Self::new(api_id, api_hash)
    }
}
