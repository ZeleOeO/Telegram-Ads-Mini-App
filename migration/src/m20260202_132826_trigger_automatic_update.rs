use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"
            CREATE OR REPLACE FUNCTION sea_orm_set_updated_at() RETURNS trigger AS $$
            BEGIN
                IF (
                    NEW IS DISTINCT FROM OLD AND
                    NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
                ) THEN
                    NEW.updated_at := current_timestamp;
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
        "#).await?;

        db.execute_unprepared(r#"
            CREATE OR REPLACE FUNCTION manage_updated_at(_tbl regclass) RETURNS VOID AS $$
            BEGIN
                EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                                FOR EACH ROW EXECUTE PROCEDURE sea_orm_set_updated_at()', _tbl);
            END;
            $$ LANGUAGE plpgsql;
        "#).await?;

        db.execute_unprepared("SELECT manage_updated_at('deals');").await?;
        db.execute_unprepared("SELECT manage_updated_at('channels');").await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Cleanup: Drop triggers first, then functions
        db.execute_unprepared("DROP TRIGGER IF EXISTS set_updated_at ON deals;").await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS set_updated_at ON channels;").await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS manage_updated_at(regclass);").await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS sea_orm_set_updated_at();").await?;

        Ok(())
    }
}