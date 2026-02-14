use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add owner_id column (who created the listing/campaign that the deal is for)
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(ColumnDef::new(Deals::OwnerId).integer().not_null().default(0))
                    .to_owned(),
            )
            .await?;

        // Add is_campaign_application flag to determine role logic
        // true = channel owner applied to advertiser's campaign (owner = advertiser)
        // false = advertiser applied to channel owner's listing (owner = channel owner)
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(
                        ColumnDef::new(Deals::IsCampaignApplication)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Backfill owner_id from existing data before adding FK constraints
        // For channel_listing: owner = channel owner
        // For campaign_request: owner = campaign advertiser
        let db = manager.get_connection();
        
        // Update channel_listing deals: owner_id = channel.owner_id
        db.execute_unprepared(
            r#"
            UPDATE deals
            SET owner_id = channels.owner_id
            FROM channels
            WHERE deals.channel_id = channels.id
            AND deals.deal_type = 'channel_listing'
            "#
        ).await?;

        // Update campaign_request deals: owner_id = advertiser_id (campaign owner)
        db.execute_unprepared(
            r#"
            UPDATE deals
            SET owner_id = advertiser_id,
                is_campaign_application = true
            WHERE deal_type = 'campaign_request'
            "#
        ).await?;

        // Add foreign key constraint for applicant_id -> users.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_deals_applicant_id")
                    .from(Deals::Table, Deals::ApplicantId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for owner_id -> users.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_deals_owner_id")
                    .from(Deals::Table, Deals::OwnerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for ad_format_id -> channel_ad_formats.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_deals_ad_format_id")
                    .from(Deals::Table, Deals::AdFormatId)
                    .to(ChannelAdFormats::Table, ChannelAdFormats::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create indexes for faster lookups
        manager
            .create_index(
                Index::create()
                    .table(Deals::Table)
                    .name("idx_deals_owner_id")
                    .col(Deals::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Deals::Table)
                    .name("idx_deals_applicant_id")
                    .col(Deals::ApplicantId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx_deals_applicant_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_deals_owner_id").to_owned())
            .await?;

        // Drop foreign keys
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Deals::Table)
                    .name("fk_deals_ad_format_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Deals::Table)
                    .name("fk_deals_owner_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Deals::Table)
                    .name("fk_deals_applicant_id")
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::IsCampaignApplication)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::OwnerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Deals {
    Table,
    OwnerId,
    ApplicantId,
    IsCampaignApplication,
    AdFormatId,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum ChannelAdFormats {
    Table,
    Id,
}
