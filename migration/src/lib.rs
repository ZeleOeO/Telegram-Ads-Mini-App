pub use sea_orm_migration::prelude::*;

mod m20260202_130751_initial_setup;
mod m20260202_132826_trigger_automatic_update;
mod m20260203_174902_add_media_to_campaigns;
mod m20260203_201436_create_bot_observed_channels_table;
mod m20260203_210710_add_user_details;
mod m20260203_213822_fix_missing_updated_at;
mod m20260203_220707_add_channel_category;
mod m20260203_add_channel_enhancements;
mod m20260203_add_campaigns;
mod m20260203_enhance_deals;
mod m20260203_add_escrow;
mod m20260205_add_deal_refactor_columns;
mod m20260205_add_owner_and_fk_constraints;
mod m20260208_analytics_update;
mod m20260213_add_campaign_enhancements;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260202_130751_initial_setup::Migration),
            Box::new(m20260202_132826_trigger_automatic_update::Migration),
            Box::new(m20260203_add_campaigns::Migration),
            Box::new(m20260203_174902_add_media_to_campaigns::Migration),
            Box::new(m20260203_201436_create_bot_observed_channels_table::Migration),
            Box::new(m20260203_210710_add_user_details::Migration),
            Box::new(m20260203_213822_fix_missing_updated_at::Migration),
            Box::new(m20260203_220707_add_channel_category::Migration),
            Box::new(m20260203_add_channel_enhancements::Migration),
            Box::new(m20260203_enhance_deals::Migration),
            Box::new(m20260203_add_escrow::Migration),
            Box::new(m20260205_add_deal_refactor_columns::Migration),
            Box::new(m20260205_add_owner_and_fk_constraints::Migration),
            Box::new(m20260208_analytics_update::Migration),
            Box::new(m20260213_add_campaign_enhancements::Migration),
        ]
    }
}
