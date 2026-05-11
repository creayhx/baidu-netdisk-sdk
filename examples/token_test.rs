//! Token Refresh and Validation Test
//!
//! This example demonstrates:
//! 1. Creating AccessToken using builder methods
//! 2. Loading token from environment variables
//! 3. Validating token status
//! 4. Testing auto-refresh mechanism with expired tokens

use baidu_netdisk_sdk::{AccessToken, BaiduNetDiskClient, TokenStatus};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Baidu NetDisk Token Test ===\n");

    let client = BaiduNetDiskClient::builder()
        .auto_refresh(true)
        .refresh_ahead_seconds(300)
        .build()?;

    println!("--- Part 1: Create AccessToken using builder ---");
    let token = AccessToken::new(
        "your_access_token".to_string(),
        "your_refresh_token".to_string(),
        2592000,
        "basic netdisk".to_string(),
    );
    println!("✓ Created AccessToken");
    println!("  Valid for: {} seconds", token.remaining_seconds());

    println!("\n--- Part 2: Load Token from Environment ---");
    match client.load_token_from_env() {
        Ok(token) => {
            println!("✓ Token loaded from environment");
            println!("  Scope: {}", token.scope);
            println!("  Valid for: {} seconds", token.remaining_seconds());
        }
        Err(e) => {
            println!("✗ Failed to load token: {}", e);
            println!("\nSet these environment variables:");
            println!("  BD_NETDISK_ACCESS_TOKEN");
            println!("  BD_NETDISK_REFRESH_TOKEN");
            println!("  BD_NETDISK_EXPIRES_IN");
        }
    }

    println!("\n--- Part 3: Validate Token Status ---");
    match client.validate_token() {
        Ok(status) => match status {
            TokenStatus::Valid => println!("✓ Token is valid"),
            TokenStatus::ExpiringSoon => println!("⚠ Token is expiring soon (< 5 min)"),
            TokenStatus::Expired => println!("✗ Token is expired"),
        },
        Err(e) => println!("✗ No token set: {}", e),
    }

    println!("\n--- Part 4: Test Expired Token Auto-Refresh ---");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let expired_token = AccessToken::with_all(
        "test_access_token".to_string(),
        "test_refresh_token".to_string(),
        2592000,
        "basic netdisk".to_string(),
        String::new(),
        String::new(),
        now - 2592000 - 3600,
    );

    println!("Created expired token:");
    println!("  Acquired: {} seconds ago", 2592000 + 3600);
    match expired_token.validate() {
        TokenStatus::Expired => println!("  Status: Expired ✓"),
        _ => println!("  Status: Unexpected"),
    }

    client.set_access_token(expired_token)?;

    println!("\nAttempting to get valid token (will try auto-refresh)...");
    match client.get_valid_token().await {
        Ok(new_token) => {
            println!("✓ Auto-refresh succeeded!");
            println!(
                "  New token valid for: {} seconds",
                new_token.remaining_seconds()
            );
        }
        Err(e) => {
            println!("✗ Auto-refresh failed: {}", e);
            println!("  → Need to re-authenticate via device code flow");
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
