use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .add_column(string(Deals::DealType).default("channel_listing"))
                    .add_column(decimal_len_null(Deals::PriceTon, 18, 9))
                    .add_column(string(Deals::PaymentStatus).default("pending"))
                    .add_column(string(Deals::CreativeStatus).default("draft"))
                    .add_column(timestamp_null(Deals::CreativeSubmittedAt))
                    .add_column(timestamp_null(Deals::CreativeApprovedAt))
                    .add_column(timestamp_null(Deals::ScheduledPostTime))
                    .add_column(timestamp_null(Deals::ActualPostTime))
                    .add_column(timestamp_null(Deals::PostVerifiedAt))
                    .add_column(timestamp_null(Deals::FundsReleasedAt))
                    .add_column(timestamp_null(Deals::TimeoutAt))
                    .add_column(timestamp_null(Deals::CancelledAt))
                    .add_column(integer_null(Deals::AdFormatId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DealCreatives::Table)
                    .if_not_exists()
                    .col(pk_auto(DealCreatives::Id))
                    .col(integer(DealCreatives::DealId))
                    .col(integer(DealCreatives::Version).default(1))
                    .col(text(DealCreatives::Content))
                    .col(json_null(DealCreatives::MediaUrls))
                    .col(integer(DealCreatives::SubmittedBy))
                    .col(text_null(DealCreatives::Feedback))
                    .col(string(DealCreatives::Status).default("submitted"))
                    .col(timestamp(DealCreatives::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(DealCreatives::Table, DealCreatives::DealId)
                            .to(Deals::Table, Deals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DealCreatives::Table, DealCreatives::SubmittedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DealNegotiations::Table)
                    .if_not_exists()
                    .col(pk_auto(DealNegotiations::Id))
                    .col(integer(DealNegotiations::DealId))
                    .col(integer(DealNegotiations::FromUserId))
                    .col(string(DealNegotiations::MessageType))
                    .col(text_null(DealNegotiations::MessageText))
                    .col(decimal_len_null(DealNegotiations::OfferedPriceTon, 18, 9))
                    .col(timestamp(DealNegotiations::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(DealNegotiations::Table, DealNegotiations::DealId)
                            .to(Deals::Table, Deals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(DealNegotiations::Table, DealNegotiations::FromUserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(DealCreatives::Table)
                    .name("idx-deal-creatives-deal-id")
                    .col(DealCreatives::DealId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(DealNegotiations::Table)
                    .name("idx-deal-negotiations-deal-id")
                    .col(DealNegotiations::DealId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Deals::Table)
                    .name("idx-deals-state")
                    .col(Deals::State)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DealNegotiations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(DealCreatives::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deals::Table)
                    .drop_column(Deals::DealType)
                    .drop_column(Deals::PriceTon)
                    .drop_column(Deals::PaymentStatus)
                    .drop_column(Deals::CreativeStatus)
                    .drop_column(Deals::CreativeSubmittedAt)
                    .drop_column(Deals::CreativeApprovedAt)
                    .drop_column(Deals::ScheduledPostTime)
                    .drop_column(Deals::ActualPostTime)
                    .drop_column(Deals::PostVerifiedAt)
                    .drop_column(Deals::FundsReleasedAt)
                    .drop_column(Deals::TimeoutAt)
                    .drop_column(Deals::CancelledAt)
                    .drop_column(Deals::AdFormatId)
                    .to_owned(),
            )
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
enum Deals {
    Table,
    Id,
    State,
    DealType,
    PriceTon,
    PaymentStatus,
    CreativeStatus,
    CreativeSubmittedAt,
    CreativeApprovedAt,
    ScheduledPostTime,
    ActualPostTime,
    PostVerifiedAt,
    FundsReleasedAt,
    TimeoutAt,
    CancelledAt,
    AdFormatId,
}

#[derive(DeriveIden)]
enum DealCreatives {
    Table,
    Id,
    DealId,
    Version,
    Content,
    MediaUrls,
    SubmittedBy,
    Feedback,
    Status,
    CreatedAt,
}

#[derive(DeriveIden)]
enum DealNegotiations {
    Table,
    Id,
    DealId,
    FromUserId,
    MessageType,
    MessageText,
    OfferedPriceTon,
    CreatedAt,
}
