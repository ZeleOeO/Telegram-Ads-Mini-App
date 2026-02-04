use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Campaigns::Table)
                    .if_not_exists()
                    .col(pk_auto(Campaigns::Id))
                    .col(integer(Campaigns::AdvertiserId))
                    .col(string(Campaigns::Title))
                    .col(text(Campaigns::Brief))
                    .col(decimal_len(Campaigns::BudgetTon, 18, 9))
                    .col(big_integer_null(Campaigns::TargetSubscribersMin))
                    .col(big_integer_null(Campaigns::TargetSubscribersMax))
                    .col(text_null(Campaigns::TargetLanguages))
                    .col(string(Campaigns::Status).default("draft"))
                    .col(timestamp(Campaigns::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Campaigns::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Campaigns::Table, Campaigns::AdvertiserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CampaignApplications::Table)
                    .if_not_exists()
                    .col(pk_auto(CampaignApplications::Id))
                    .col(integer(CampaignApplications::CampaignId))
                    .col(integer(CampaignApplications::ChannelId))
                    .col(decimal_len(CampaignApplications::ProposedPriceTon, 18, 9))
                    .col(text_null(CampaignApplications::Message))
                    .col(string(CampaignApplications::Status).default("pending"))
                    .col(
                        timestamp(CampaignApplications::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                CampaignApplications::Table,
                                CampaignApplications::CampaignId,
                            )
                            .to(Campaigns::Table, Campaigns::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CampaignApplications::Table, CampaignApplications::ChannelId)
                            .to(Channels::Table, Channels::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Campaigns::Table)
                    .name("idx-campaigns-advertiser-id")
                    .col(Campaigns::AdvertiserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Campaigns::Table)
                    .name("idx-campaigns-status")
                    .col(Campaigns::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(CampaignApplications::Table)
                    .name("idx-campaign-applications-campaign-id")
                    .col(CampaignApplications::CampaignId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(CampaignApplications::Table)
                    .name("idx-campaign-applications-channel-id")
                    .col(CampaignApplications::ChannelId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CampaignApplications::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Campaigns::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Campaigns {
    Table,
    Id,
    AdvertiserId,
    Title,
    Brief,
    BudgetTon,
    TargetSubscribersMin,
    TargetSubscribersMax,
    TargetLanguages,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CampaignApplications {
    Table,
    Id,
    CampaignId,
    ChannelId,
    ProposedPriceTon,
    Message,
    Status,
    CreatedAt,
}
