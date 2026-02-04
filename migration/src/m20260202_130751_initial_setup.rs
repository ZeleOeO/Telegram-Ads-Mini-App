use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(big_integer_uniq(Users::TelegramId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Channels::Table)
                    .if_not_exists()
                    .col(pk_auto(Channels::Id))
                    .col(integer(Channels::OwnerId))
                    .col(big_integer_uniq(Channels::TelegramChannelId))
                    .col(big_integer_null(Channels::Subscribers))
                    .col(big_integer_null(Channels::Reach))
                    .col(string_null(Channels::Language))
                    .col(float_null(Channels::PremiumPercentage))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Channels::Table, Channels::OwnerId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChannelAdmins::Table)
                    .if_not_exists()
                    .col(pk_auto(ChannelAdmins::Id))
                    .col(integer(ChannelAdmins::UserId))
                    .col(integer(ChannelAdmins::ChannelId))
                    .col(boolean(ChannelAdmins::CanPostMessages).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelAdmins::Table, ChannelAdmins::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelAdmins::Table, ChannelAdmins::ChannelId)
                            .to(Channels::Table, Channels::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Deals::Table)
                    .if_not_exists()
                    .col(pk_auto(Deals::Id))
                    .col(integer(Deals::AdvertiserId))
                    .col(integer(Deals::ChannelId))
                    .col(integer_null(Deals::PrManagerId))
                    .col(string(Deals::State))
                    .col(string_null(Deals::EscrowAddress))
                    .col(text_null(Deals::PostContent))
                    .col(timestamp_null(Deals::PostTime))
                    .col(string_null(Deals::PostLink))
                    .col(timestamp(Deals::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Deals::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Deals::Table, Deals::AdvertiserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Deals::Table, Deals::ChannelId)
                            .to(Channels::Table, Channels::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .name("idx-telegram-id-user")
                    .col(Users::TelegramId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Deals::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ChannelAdmins::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Channels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    TelegramId,
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    Id,
    OwnerId,
    TelegramChannelId,
    Subscribers,
    Reach,
    Language,
    PremiumPercentage,
}

#[derive(DeriveIden)]
enum ChannelAdmins {
    Table,
    Id,
    UserId,
    ChannelId,
    CanPostMessages,
}

#[derive(DeriveIden)]
enum Deals {
    Table,
    Id,
    AdvertiserId,
    ChannelId,
    PrManagerId,
    State,
    EscrowAddress,
    PostContent,
    PostTime,
    PostLink,
    CreatedAt,
    UpdatedAt,
}

