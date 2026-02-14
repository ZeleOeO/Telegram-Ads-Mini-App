use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use bip39::Mnemonic;
use ed25519_dalek::SigningKey;
use funty::Fundamental;
use rand::RngCore;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use ton::lite_client::LiteClient;
use ton::net_config::{TON_NET_CONF_MAINNET_PUBLIC, TON_NET_CONF_TESTNET_PUBLIC};
use ton::ton_core::types::TonAddress;
use tracing::info;

use crate::entity::escrow_wallets;
use crate::models::errors::{ApiError, ApiResult};
use crate::services::escrow_mock;

#[derive(Debug, Clone)]
pub struct EscrowWallet {
    pub address: String,
    pub balance: f64,
}

pub async fn generate_escrow_wallet(
    db: &DatabaseConnection,
    deal_id: i32,
) -> ApiResult<EscrowWallet> {
    let _network = std::env::var("TON_NETWORK").unwrap_or_else(|_| "mock".to_string());
    
    let (mnemonic, key) = derive_key_from_mnemonic()?;
    let encrypted_mnemonic = encrypt_key(&mnemonic)?;

    let vk = key.verifying_key();
    let pubkey_bytes = vk.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(pubkey_bytes);
    let hash = hasher.finalize();
    let address = format!("0:{}", hex::encode(hash)); 

    let wallet_model = escrow_wallets::ActiveModel {
        deal_id: Set(deal_id),
        address: Set(address.clone()),
        private_key_encrypted: Set(encrypted_mnemonic),
        balance_ton: Set(rust_decimal::Decimal::new(0, 0)),
        created_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };
    
    if let Ok(Some(existing)) = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(db)
        .await 
    {
        return Ok(EscrowWallet {
            address: existing.address,
            balance: existing.balance_ton.to_string().parse().unwrap_or(0.0),
        });
    }

    wallet_model.insert(db).await
        .map_err(|e| ApiError::Internal(format!("Failed to save wallet: {}", e)))?;

    Ok(EscrowWallet {
        address,
        balance: 0.0,
    })
}

pub async fn verify_payment(
    db: &DatabaseConnection,
    wallet_address: &str,
    expected_amount: f64,
) -> ApiResult<bool> {
    let network = std::env::var("TON_NETWORK").unwrap_or_else(|_| "mock".to_string());
    if network == "mock" {
        return escrow_mock::verify_payment(db, wallet_address, expected_amount).await;
    }
    let client = get_ton_client().await?;
    let address = wallet_address.parse::<TonAddress>()
        .map_err(|e| ApiError::Internal(format!("Invalid address: {}", e)))?;
    let mc_info = client.get_mc_info().await
        .map_err(|e| ApiError::Internal(format!("Failed to get mc info: {}", e)))?;
    
    let account_state = match client.get_account_state(&address, mc_info.last.seqno, None).await {
        Ok(state) => state,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("Can't read BOC from empty slice") || err_str.contains("Fail to read") {
                info!("Account uninitialized or empty for {}, assuming 0 balance", wallet_address);
                return Ok(false);
            }
            return Err(ApiError::Internal(format!("Failed to get account state: {}", e)));
        }
    };

    let balance_nanotons = match account_state {
        ton::block_tlb::MaybeAccount::Account(account) => {
            account.storage.balance.coins.as_u128()
        },
        _ => 0u128, 
    };
    
    let balance_tons = balance_nanotons as f64 / 1_000_000_000.0;
    
    info!(
        "Wallet {} balance: {} TON, expected: {} TON",
        wallet_address, balance_tons, expected_amount
    );
    
    if balance_tons >= expected_amount {
        let wallet = escrow_wallets::Entity::find()
            .filter(escrow_wallets::Column::Address.eq(wallet_address))
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;
            
        let mut active_wallet: escrow_wallets::ActiveModel = wallet.into();
        active_wallet.balance_ton =
            Set(rust_decimal::Decimal::try_from(balance_tons).unwrap_or_default());
        active_wallet.update(db).await?;
        return Ok(true);
    }
    
    Ok(false)
}

pub async fn release_funds(
    db: &DatabaseConnection,
    deal_id: i32,
    to_address: &str,
) -> ApiResult<String> {
    let network = std::env::var("TON_NETWORK").unwrap_or_else(|_| "mock".to_string());
    if network == "mock" {
        return escrow_mock::release_funds(db, deal_id, to_address).await;
    }

    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow wallet not found".to_string()))?;

    let _ = verify_payment(db, &wallet.address, 0.0).await?; 
    
    let wallet = escrow_wallets::Entity::find()
        .filter(escrow_wallets::Column::DealId.eq(deal_id))
        .one(db)
        .await?
        .unwrap();
        
    let current_balance = wallet.balance_ton.to_string().parse::<f64>().unwrap_or(0.0);

    if current_balance < 0.01 { 
        return Err(ApiError::BadRequest("Insufficient funds in escrow wallet".to_string()));
    }

    let _mnemonic_str = decrypt_key(&wallet.private_key_encrypted)?;
    
    info!("SIMULATED Release funds from {} to {} (Amount: {})", wallet.address, to_address, current_balance);
    
    Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
}

pub async fn refund_funds(
    db: &DatabaseConnection,
    deal_id: i32,
    to_address: &str,
) -> ApiResult<String> {
     release_funds(db, deal_id, to_address).await
}

async fn get_ton_client() -> ApiResult<LiteClient> {
    let network = std::env::var("TON_NETWORK").unwrap_or_else(|_| "testnet".to_string());
    let config_json = if network == "mainnet" {
        TON_NET_CONF_MAINNET_PUBLIC
    } else {
        TON_NET_CONF_TESTNET_PUBLIC
    };
    let client = LiteClient::builder()?
        .with_net_config_json(config_json)?
        .build()?;
    Ok(client)
}

fn get_cipher_key() -> ApiResult<[u8; 32]> {
    let secret = std::env::var("ESCROW_SECRET_KEY").unwrap_or_else(|_| "mock_secret_key_needs_replacement".to_string());
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    Ok(hasher.finalize().into())
}

fn encrypt_key(key: &str) -> ApiResult<String> {
    let cipher_key = get_cipher_key()?;
    let cipher = Aes256Gcm::new(&cipher_key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted = cipher.encrypt(nonce, key.as_bytes())
        .map_err(|e| ApiError::Internal(format!("Encryption failed: {}", e)))?;
    
    let mut combined = nonce_bytes.to_vec();
    combined.extend(encrypted);
    Ok(hex::encode(combined))
}

fn decrypt_key(encrypted_hex: &str) -> ApiResult<String> {
    let combined = hex::decode(encrypted_hex)
        .map_err(|e| ApiError::Internal(format!("Hex decode failed: {}", e)))?;
    if combined.len() < 12 {
        return Err(ApiError::Internal("Invalid encrypted data length".to_string()));
    }
    
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let cipher_key = get_cipher_key()?;
    let cipher = Aes256Gcm::new(&cipher_key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| ApiError::Internal(format!("Decryption failed: {}", e)))?;
        
    String::from_utf8(plaintext)
        .map_err(|e| ApiError::Internal(format!("Invalid UTF-8 in decrypted key: {}", e)))
}

fn derive_key_from_mnemonic() -> ApiResult<(String, SigningKey)> {
    let mut entropy = [0u8; 32];
    rand::rng().fill_bytes(&mut entropy);
    
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|e| ApiError::Internal(format!("Mnemonic gen failed: {}", e)))?;
    let phrase = mnemonic.to_string();
    
    let seed = mnemonic.to_seed(""); 
    
    let signing_key = SigningKey::from_bytes(seed[0..32].try_into().unwrap());
    
    Ok((phrase, signing_key))
}
