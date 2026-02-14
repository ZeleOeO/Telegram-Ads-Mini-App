use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use crate::auth::TelegramUser;
use crate::entity::users;
use crate::models::errors::ApiResult;
pub async fn sync_user(db: &DatabaseConnection, tg_user: &TelegramUser) -> ApiResult<users::Model> {
    let existing = users::Entity::find()
        .filter(users::Column::TelegramId.eq(tg_user.id))
        .one(db)
        .await?;
    match existing {
        Some(user) => {
            let mut active_user: users::ActiveModel = user.clone().into();
            if let Some(ref username) = tg_user.username {
                active_user.username = Set(Some(username.clone()));
            }
            active_user.first_name = Set(Some(tg_user.first_name.clone()));
            if let Some(ref last_name) = tg_user.last_name {
                active_user.last_name = Set(Some(last_name.clone()));
            }
            active_user.updated_at = Set(chrono::Utc::now().naive_utc());
            let updated = active_user.update(db).await?;
            Ok(updated)
        }
        None => {
            let new_user = users::ActiveModel {
                telegram_id: Set(tg_user.id),
                username: Set(tg_user.username.clone()),
                first_name: Set(Some(tg_user.first_name.clone())),
                last_name: Set(tg_user.last_name.clone()),
                ..Default::default()
            };
            let created = new_user.insert(db).await?;
            Ok(created)
        }
    }
}
