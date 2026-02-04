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
                    .add_column(timestamp(Channels::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(timestamp(Users::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared("SELECT manage_updated_at('users');")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS set_updated_at ON users;")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Channels::Table)
                    .drop_column(Channels::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UpdatedAt,
}
