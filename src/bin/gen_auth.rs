use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use urlencoding::encode;
fn main() {
    dotenvy::dotenv().ok();
    let bot_token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");
    let user_id = 1424079034;
    let username = "test_user";
    let now = chrono::Utc::now().timestamp().to_string();
    let mut data = BTreeMap::new();
    data.insert("auth_date", now);
    data.insert("query_id", "test_query_id".to_string());
    data.insert("user", format!(r#"{{"id":{},"first_name":"Test","last_name":"User","username":"{}","language_code":"en","is_premium":true}}"#, user_id, username));
    let data_check_string = data
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData").unwrap();
    mac.update(bot_token.as_bytes());
    let secret_key = mac.finalize().into_bytes();
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).unwrap();
    mac.update(data_check_string.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    let mut final_parts = data
        .iter()
        .map(|(k, v)| format!("{}={}", k, encode(v)))
        .collect::<Vec<_>>();
    final_parts.push(format!("hash={}", signature));
    let init_data = final_parts.join("&");
    println!("\n--- Generated initData for Testing ---");
    println!("\n{}", init_data);
}
