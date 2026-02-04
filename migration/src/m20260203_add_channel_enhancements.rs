use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channels::Table)
                    .add_column(string_null(Channels::Title))
                    .add_column(string_null(Channels::Username))
                    .add_column(text_null(Channels::Description))
                    .add_column(string(Channels::Status).default("active"))
                    .add_column(timestamp_null(Channels::LastStatsUpdate))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChannelAdFormats::Table)
                    .if_not_exists()
                    .col(pk_auto(ChannelAdFormats::Id))
                    .col(integer(ChannelAdFormats::ChannelId))
                    .col(string(ChannelAdFormats::FormatName))
                    .col(text_null(ChannelAdFormats::FormatDescription))
                    .col(decimal_len(ChannelAdFormats::PriceTon, 18, 9).default(0.0))
                    .col(timestamp(ChannelAdFormats::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(ChannelAdFormats::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelAdFormats::Table, ChannelAdFormats::ChannelId)
                            .to(Channels::Table, Channels::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ChannelAdFormats::Table)
                    .name("idx-channel-ad-formats-channel-id")
                    .col(ChannelAdFormats::ChannelId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelAdFormats::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Channels::Table)
                    .drop_column(Channels::Title)
                    .drop_column(Channels::Username)
                    .drop_column(Channels::Description)
                    .drop_column(Channels::Status)
                    .drop_column(Channels::LastStatsUpdate)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    Id,
    Title,
    Username,
    Description,
    Status,
    LastStatsUpdate,
}

#[derive(DeriveIden)]
enum ChannelAdFormats {
    Table,
    Id,
    ChannelId,
    FormatName,
    FormatDescription,
    PriceTon,
    CreatedAt,
    UpdatedAt,
}
