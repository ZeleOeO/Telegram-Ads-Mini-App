use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Campaigns::Table)
                    .add_column(text_null(Campaigns::MediaUrls))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Campaigns::Table)
                    .drop_column(Campaigns::MediaUrls)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Campaigns {
    Table,
    MediaUrls,
}
