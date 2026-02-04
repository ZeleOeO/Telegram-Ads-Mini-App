// Mock escrow service
//  replace with  tonlib implementation later
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::errors::{ApiError, ApiResult};

#[derive(Debug, Clone)]
pub struct EscrowWallet {
    pub address: String,
    pub balance: f64,
}

// Generate a mock escrow wallet for a deal
pub async fn generate_escrow_wallet(
    db: &DatabaseConnection,
    deal_id: i32,
) -> ApiResult<EscrowWallet> {
    use crate::entity::escrow_wallets;

    let mock_address = format!("EQ{}", Uuid::new_v4().to_string().replace("-", ""));
    let mock_private_key = Uuid::new_v4().to_string(); // Mock encrypted key

    // Create escrow wallet record
    let new_wallet = escrow_wallets::ActiveModel {
        deal_id: Set(deal_id),
        address: Set(mock_address.clone()),
        private_key_encrypted: Set(mock_private_key),
        balance_ton: Set(rust_decimal::Decimal::from(0)),
        ..Default::default()
    };

    new_wallet.insert(db).await?;

    info!(
        "Generated mock escrow wallet {} for deal {}",
        mock_address, deal_id
    );

    Ok(EscrowWallet {
        address: mock_address,
        balance: 0.0,
    })
}

// Verify payment to escrow wallet
pub async fn verify_payment(
    db: &DatabaseConnection,
    wallet_address: &str,
    expected_amount: f64,
) -> ApiResult<bool> {
    use crate::entity::escrow_wallets;
    use sea_orm::{ColumnTrait, QueryFilter};

    info!("Verifying payment to wallet {}", wallet_address);

    // Query TON  for transactions to this address

    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::Address.eq(wallet_address))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;

    // Update balance for mock, ideally ton should handle this
    let mut active_wallet: escrow_wallets::ActiveModel = wallet.into();
    active_wallet.balance_ton = Set(rust_decimal::Decimal::try_from(expected_amount).unwrap());
    active_wallet.update(db).await?;

    info!(
        "Mock payment verified for wallet {}: {} TON",
        wallet_address, expected_amount
    );
    Ok(true)
}

// Release funds from escrow to channel owner
pub async fn release_funds(
    db: &DatabaseConnection,
    deal_id: i32,
    to_address: &str,
) -> ApiResult<String> {
    use crate::entity::{escrow_wallets, transactions};
    use sea_orm::{ColumnTrait, QueryFilter};

    info!("Releasing funds for deal {} to {}", deal_id, to_address);

    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;

    let amount = wallet.balance_ton;

    let tx_hash = format!("mock_tx_{}", Uuid::new_v4());

    let new_tx = transactions::ActiveModel {
        deal_id: Set(deal_id),
        transaction_hash: Set(Some(tx_hash.clone())),
        transaction_type: Set("release".to_string()),
        from_address: Set(wallet.address.clone()),
        to_address: Set(to_address.to_string()),
        amount_ton: Set(amount),
        status: Set("confirmed".to_string()),
        confirmed_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    new_tx.insert(db).await?;

    let mut active_wallet: escrow_wallets::ActiveModel = wallet.into();
    active_wallet.balance_ton = Set(rust_decimal::Decimal::from(0));
    active_wallet.update(db).await?;

    info!("Mock funds released: {} TON to {}", amount, to_address);
    Ok(tx_hash)
}

// Refund funds from escrow to advertiser
pub async fn refund_funds(
    db: &DatabaseConnection,
    deal_id: i32,
    to_address: &str,
) -> ApiResult<String> {
    use crate::entity::{escrow_wallets, transactions};
    use sea_orm::{ColumnTrait, QueryFilter};

    info!("Refunding funds for deal {} to {}", deal_id, to_address);

    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;

    let amount = wallet.balance_ton;

    let tx_hash = format!("mock_tx_{}", Uuid::new_v4());

    let new_tx = transactions::ActiveModel {
        deal_id: Set(deal_id),
        transaction_hash: Set(Some(tx_hash.clone())),
        transaction_type: Set("refund".to_string()),
        from_address: Set(wallet.address.clone()),
        to_address: Set(to_address.to_string()),
        amount_ton: Set(amount),
        status: Set("confirmed".to_string()),
        confirmed_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    new_tx.insert(db).await?;

    let mut active_wallet: escrow_wallets::ActiveModel = wallet.into();
    active_wallet.balance_ton = Set(rust_decimal::Decimal::from(0));
    active_wallet.update(db).await?;

    info!("Mock funds refunded: {} TON to {}", amount, to_address);
    Ok(tx_hash)
}
