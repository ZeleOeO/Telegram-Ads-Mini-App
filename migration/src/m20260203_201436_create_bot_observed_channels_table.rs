use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BotObservedChannels::Table)
                    .if_not_exists()
                    .col(pk_auto(BotObservedChannels::Id))
                    .col(big_integer(BotObservedChannels::TelegramChatId).unique_key())
                    .col(string_null(BotObservedChannels::Title))
                    .col(string_null(BotObservedChannels::Username))
                    .col(timestamp(BotObservedChannels::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BotObservedChannels::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BotObservedChannels {
    Table,
    Id,
    TelegramChatId,
    Title,
    Username,
    CreatedAt,
}
