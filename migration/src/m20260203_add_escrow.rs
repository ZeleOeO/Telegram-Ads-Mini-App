use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_null(Users::TonWalletAddress))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EscrowWallets::Table)
                    .if_not_exists()
                    .col(pk_auto(EscrowWallets::Id))
                    .col(integer_uniq(EscrowWallets::DealId))
                    .col(string_uniq(EscrowWallets::Address))
                    .col(text(EscrowWallets::PrivateKeyEncrypted))
                    .col(decimal_len(EscrowWallets::BalanceTon, 18, 9).default(0.0))
                    .col(timestamp(EscrowWallets::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(EscrowWallets::Table, EscrowWallets::DealId)
                            .to(Deals::Table, Deals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(pk_auto(Transactions::Id))
                    .col(integer(Transactions::DealId))
                    .col(string_null(Transactions::TransactionHash))
                    .col(string(Transactions::TransactionType))
                    .col(string(Transactions::FromAddress))
                    .col(string(Transactions::ToAddress))
                    .col(decimal_len(Transactions::AmountTon, 18, 9))
                    .col(string(Transactions::Status).default("pending"))
                    .col(timestamp(Transactions::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Transactions::ConfirmedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Transactions::Table, Transactions::DealId)
                            .to(Deals::Table, Deals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(EscrowWallets::Table)
                    .name("idx-escrow-wallets-deal-id")
                    .col(EscrowWallets::DealId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transactions::Table)
                    .name("idx-transactions-deal-id")
                    .col(Transactions::DealId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transactions::Table)
                    .name("idx-transactions-hash")
                    .col(Transactions::TransactionHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transactions::Table)
                    .name("idx-transactions-status")
                    .col(Transactions::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EscrowWallets::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::TonWalletAddress)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    TonWalletAddress,
}

#[derive(DeriveIden)]
enum Deals {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum EscrowWallets {
    Table,
    Id,
    DealId,
    Address,
    PrivateKeyEncrypted,
    BalanceTon,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    Id,
    DealId,
    TransactionHash,
    TransactionType,
    FromAddress,
    ToAddress,
    AmountTon,
    Status,
    CreatedAt,
    ConfirmedAt,
}
