use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add applicant_id column (who clicked Apply)
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(ColumnDef::new(Deals::ApplicantId).integer().not_null().default(0))
                    .to_owned(),
            )
            .await?;

        // Add rejection_reason column
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(ColumnDef::new(Deals::RejectionReason).text().null())
                    .to_owned(),
            )
            .await?;

        // Add edit_request_reason column
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(ColumnDef::new(Deals::EditRequestReason).text().null())
                    .to_owned(),
            )
            .await?;

        // Backfill applicant_id from existing data
        // For channel_listing: applicant = advertiser
        // For campaign_request: applicant = channel owner
        let db = manager.get_connection();
        
        // Update channel_listing deals: applicant_id = advertiser_id
        db.execute_unprepared(
            r#"
            UPDATE deals
            SET applicant_id = advertiser_id
            WHERE deal_type = 'channel_listing'
            "#
        ).await?;

        // Update campaign_request deals: applicant_id = channel.owner_id
        db.execute_unprepared(
            r#"
            UPDATE deals
            SET applicant_id = channels.owner_id
            FROM channels
            WHERE deals.channel_id = channels.id
            AND deals.deal_type = 'campaign_request'
            "#
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::ApplicantId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::RejectionReason)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::EditRequestReason)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Deals {
    Table,
    ApplicantId,
    RejectionReason,
    EditRequestReason,
}
