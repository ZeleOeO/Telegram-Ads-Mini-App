use crate::AppState;
use crate::entity::{deals, channels};
use crate::services::auto_post;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, ActiveModelTrait};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn};
use chrono::Utc;
use teloxide::types::ChatId;
use tokio_cron_scheduler::{JobScheduler, Job};
use crate::helpers;
pub async fn start_scheduler(state: AppState) {
    info!("Starting background scheduler...");
    let sched = JobScheduler::new().await.unwrap();

    let state_clone = state.clone();
    sched.add(
        Job::new_async("0 * * * * *", move |_uuid, _l| {
            let state = state_clone.clone();
            Box::pin(async move {
                if let Err(e) = check_scheduled_deals(state).await {
                    error!("Scheduled deals check failed: {:?}", e);
                }
            })
        })
        .unwrap(),
    )
    .await
    .unwrap();

    let state_clone_2 = state.clone();
    sched.add(
        Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let state = state_clone_2.clone();
            Box::pin(async move {
                if let Err(e) = check_completed_deals(state).await {
                    error!("Completed deals check failed: {:?}", e);
                }
            })
        })
        .unwrap(),
    )
    .await
    .unwrap();

    let state_clone_3 = state.clone();
    sched.add(
        Job::new_async("0 0 0 * * *", move |_uuid, _l| {
            let state = state_clone_3.clone();
            Box::pin(async move {
                if let Err(e) = check_stalled_deals(state).await {
                    error!("Stalled deals check failed: {:?}", e);
                }
            })
        })
        .unwrap(),
    )
    .await
    .unwrap();

    sched.start().await.unwrap();
}

async fn check_scheduled_deals(state: AppState) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let due_posts = deals::Entity::find()
        .filter(deals::Column::State.eq("scheduled"))
        .filter(deals::Column::ScheduledPostTime.lte(now))
        .all(&state.db)
        .await?;
    for deal in due_posts {
        info!("Processing auto-post for deal #{}", deal.id);
        let channel = channels::Entity::find_by_id(deal.channel_id)
            .one(&state.db)
            .await?;
        if let Some(channel) = channel {
            let chat_id = ChatId(channel.telegram_channel_id);
            let content = deal.post_content.clone().unwrap_or_default();
            if !content.is_empty() {
                match auto_post::execute_auto_post(&state.db, &state.bot, deal.id, chat_id, &content).await {
                    Ok(link) => info!("Auto-posted deal #{} successfully: {}", deal.id, link),
                    Err(e) => error!("Failed to auto-post deal #{}: {:?}", deal.id, e),
                }
            } else {
                error!("Deal #{} scheduled but has no content", deal.id);
            }
        }
    }
    Ok(())
}

async fn check_completed_deals(state: AppState) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let verification_time = now - chrono::Duration::hours(24);
    let due_verifications = deals::Entity::find()
        .filter(deals::Column::State.eq("published"))
        .filter(deals::Column::ActualPostTime.lte(verification_time))
        .all(&state.db)
        .await?;
    for deal in due_verifications {
        info!("Verifying deal #{} (24h passed)", deal.id);
        let channel_res = channels::Entity::find_by_id(deal.channel_id)
            .one(&state.db)
            .await?;
        let mut verified = false;
        if let Some(channel) = channel_res {
            if let Some(username) = &channel.username {
                if let Some(post_link) = &deal.post_link {
                    if let Some(msg_id_str) = post_link.split('/').last() {
                         if let Ok(msg_id) = msg_id_str.parse::<i32>() {
                             match auto_post::verify_post_exists(&state.grammers, username, msg_id).await {
                                 Ok(exists) => {
                                     if exists {
                                         verified = true;
                                     } else {
                                         warn!("Post verification failed for deal #{}: Message not found", deal.id);
                                     }
                                 },
                                 Err(e) => {
                                      error!("Error verifying post for deal #{}: {:?}", deal.id, e);
                                 }
                             }
                         }
                    }
                }
            } else {
                warn!("Cannot verify deal #{}: Channel has no username (private channel?)", deal.id);
            }
        }
        if verified {
            let mut active_deal: deals::ActiveModel = deal.into();
            active_deal.state = Set("completed".to_string());
            active_deal.payment_status = Set("released".to_string());
            active_deal.funds_released_at = Set(Some(Utc::now().naive_utc()));
            active_deal.post_verified_at = Set(Some(Utc::now().naive_utc()));
            active_deal.updated_at = Set(Utc::now().naive_utc());
            match active_deal.update(&state.db).await {
                Ok(updated) => info!("Deal #{} completed and funds released", updated.id),
                Err(e) => error!("Failed to complete deal: {:?}", e),
            }
        } else {
             info!("Deal #{} verification pending/failed. Will retry.", deal.id);
        }
    }
    Ok(())
}

async fn check_stalled_deals(state: AppState) -> anyhow::Result<()> {
    let cutoff = Utc::now().naive_utc() - chrono::Duration::hours(72);
    
    let stalled_deals = deals::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(deals::Column::State.eq("pending"))
                .add(deals::Column::State.eq("negotiating"))
                .add(deals::Column::State.eq("awaiting_payment"))
        )
        .filter(deals::Column::UpdatedAt.lte(cutoff))
        .all(&state.db)
        .await?;

    for deal in stalled_deals {
        let deal_id = deal.id;
        info!("Cancelling stalled deal #{} (inactive > 72h)", deal_id);
        let mut active_deal: deals::ActiveModel = deal.into();
        active_deal.state = Set("cancelled".to_string());
        active_deal.rejection_reason = Set(Some("System: Timeout due to inactivity (>72h)".to_string()));
        active_deal.cancelled_at = Set(Some(Utc::now().naive_utc()));
        active_deal.updated_at = Set(Utc::now().naive_utc());
        
        match active_deal.update(&state.db).await {
            Ok(updated) => info!("Deal #{} auto-cancelled due to timeout", updated.id),
            Err(e) => error!("Failed to auto-cancel deal #{}: {:?}", deal_id, e),
        }
    }


    Ok(())
}
