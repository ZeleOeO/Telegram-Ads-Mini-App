use serde::Serialize;
#[derive(Serialize)]
pub struct InitiatePaymentResponse {
    pub escrow_address: String,
    pub amount_ton: String,
    pub deal_id: i32,
}
#[derive(Serialize)]
pub struct PaymentStatusResponse {
    pub deal_id: i32,
    pub payment_status: String,
    pub escrow_balance: String,
}
#[derive(Serialize)]
pub struct TransactionResponse {
    pub id: i32,
    pub deal_id: i32,
    pub transaction_hash: Option<String>,
    pub transaction_type: String,
    pub from_address: String,
    pub to_address: String,
    pub amount_ton: String,
    pub status: String,
    pub created_at: String,
    pub confirmed_at: Option<String>,
}
