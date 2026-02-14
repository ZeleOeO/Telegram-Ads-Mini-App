use teloxide::{prelude::*, types::ChatId};
use crate::models::errors::ApiResult;
pub async fn notify_user(
    bot: &Bot,
    user_telegram_id: i64,
    message: &str
) -> ApiResult<()> {
    match bot.send_message(ChatId(user_telegram_id), message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::warn!("Failed to send notification to {}: {:?}", user_telegram_id, e);
            Ok(())
        }
    }
}
pub async fn notify_deal_status_change(
    bot: &Bot,
    recipient_id: i64,
    deal_name: &str,
    new_status: &str,
    extra_info: Option<&str>
) -> ApiResult<()> {
    let msg = match new_status {
        "accepted" => format!("'{}' has been ACCEPTED! Please proceed to payment.", deal_name),
        "rejected" => format!("'{}' has been REJECTED.\nReason: {}", deal_name, extra_info.unwrap_or("No reason provided")),
        "draft_submitted" => format!("Draft submitted for '{}'. Please review it.", deal_name),
        "review_approved" => format!("Draft APPROVED for '{}'. It will be auto-posted at the scheduled time.", deal_name),
        "review_rejected" => format!("Edit requested for '{}'.\nFeedback: {}", deal_name, extra_info.unwrap_or("No feedback")),
        _ => format!("'{}' status updated to: {}", deal_name, new_status),
    };
    notify_user(bot, recipient_id, &msg).await
}
