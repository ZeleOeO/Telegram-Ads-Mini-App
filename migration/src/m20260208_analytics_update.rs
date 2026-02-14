use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channels::Table)
                    .add_column(ColumnDef::new(Channels::EnabledNotifications).float().null())
                    .add_column(ColumnDef::new(Channels::SharesPerPost).float().null())
                    .add_column(ColumnDef::new(Channels::ReactionsPerPost).float().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channels::Table)
                    .drop_column(Channels::EnabledNotifications)
                    .drop_column(Channels::SharesPerPost)
                    .drop_column(Channels::ReactionsPerPost)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    EnabledNotifications,
    SharesPerPost,
    ReactionsPerPost,
}
