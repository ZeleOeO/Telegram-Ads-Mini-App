use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub telegram_id: i64,
    pub ton_wallet_address: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub updated_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::campaigns::Entity")]
    Campaigns,
    #[sea_orm(has_many = "super::channel_admins::Entity")]
    ChannelAdmins,
    #[sea_orm(has_many = "super::channels::Entity")]
    Channels,
    #[sea_orm(has_many = "super::deal_creatives::Entity")]
    DealCreatives,
    #[sea_orm(has_many = "super::deal_negotiations::Entity")]
    DealNegotiations,
    #[sea_orm(has_many = "super::deals::Entity")]
    Deals,
}
impl Related<super::campaigns::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Campaigns.def()
    }
}
impl Related<super::channel_admins::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChannelAdmins.def()
    }
}
impl Related<super::channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}
impl Related<super::deal_creatives::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DealCreatives.def()
    }
}
impl Related<super::deal_negotiations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DealNegotiations.def()
    }
}
impl Related<super::deals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deals.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
