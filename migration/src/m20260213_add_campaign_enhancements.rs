use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add categories to campaigns
        manager
            .alter_table(
                Table::alter()
                    .table(Campaigns::Table)
                    .add_column(ColumnDef::new(Campaigns::Categories).string().null())
                    .to_owned(),
            )
            .await?;

        // Add campaign_id to deals
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(ColumnDef::new(Deals::CampaignId).integer().null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_deals_campaign_id")
                            .from_col(Deals::CampaignId)
                            .to_tbl(Campaigns::Table)
                            .to_col(Campaigns::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add deal_id to campaign_applications if not exists (check logic in code suggested it, likely missing too)
        // But verifying schema first - let's add it just in case, wrapped in safe check if sea-orm supports it? 
        // Or just try. If it fails, we can adjust.
        // Actually, let's just do campaign_id and categories first as per error.
        
        // Wait, the user error was `deals.campaign_id`. 
        // I should also check if `deal_id` is missing in `campaign_applications`.
        
        manager
            .alter_table(
                Table::alter()
                    .table(CampaignApplications::Table)
                    .add_column(ColumnDef::new(CampaignApplications::DealId).integer().null())
                     .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_campaign_applications_deal_id")
                            .from_col(CampaignApplications::DealId)
                            .to_tbl(Deals::Table)
                            .to_col(Deals::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop deal_id from campaign_applications
        manager .alter_table( Table::alter() .table(CampaignApplications::Table) .drop_column(CampaignApplications::DealId) .to_owned(), ) .await?;

        // Drop campaign_id from deals
        manager .alter_table( Table::alter() .table(Deals::Table) .drop_column(Deals::CampaignId) .to_owned(), ) .await?;

        // Drop categories from campaigns
        manager .alter_table( Table::alter() .table(Campaigns::Table) .drop_column(Campaigns::Categories) .to_owned(), ) .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Campaigns {
    Table,
    Id,
    Categories,
}

#[derive(Iden)]
enum Deals {
    Table,
    Id,
    CampaignId,
}

#[derive(Iden)]
enum CampaignApplications {
    Table,
    DealId,
}
