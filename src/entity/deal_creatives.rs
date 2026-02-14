use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "deal_creatives")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub deal_id: i32,
    pub version: i32,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub media_urls: Option<Json>,
    pub submitted_by: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub feedback: Option<String>,
    pub status: String,
    pub created_at: DateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::deals::Entity",
        from = "Column::DealId",
        to = "super::deals::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Deals,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SubmittedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}
impl Related<super::deals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deals.def()
    }
}
impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
