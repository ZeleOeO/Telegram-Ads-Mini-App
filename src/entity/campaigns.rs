use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "campaigns")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub advertiser_id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub brief: String,
    #[sea_orm(column_type = "Decimal(Some((18, 9)))")]
    pub budget_ton: Decimal,
    pub target_subscribers_min: Option<i64>,
    pub target_subscribers_max: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub target_languages: Option<String>,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(column_type = "Text", nullable)]
    pub media_urls: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub categories: Option<String>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::campaign_applications::Entity")]
    CampaignApplications,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AdvertiserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}
impl Related<super::campaign_applications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CampaignApplications.def()
    }
}
impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
