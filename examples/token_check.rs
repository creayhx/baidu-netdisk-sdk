use baidu_netdisk_sdk::auth::AccessToken;
use baidu_netdisk_sdk::BaiduNetDiskClient;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Token Validation and Refresh Test ===");
    println!();

    // Create client
    let client = BaiduNetDiskClient::builder().build()?;

    println!("Client created successfully");
    println!();

    // Test 1: Load token from environment
    println!("Test 1: Loading token from environment...");
    client.load_token_from_env()?;

    let token = client.get_valid_token().await?;
    println!("Token loaded successfully: {}", token.access_token);
    println!();

    // Test 2: Check token validity
    println!("Test 2: Checking token validity...");
    println!("Token acquired at: {}", token.acquired_at);
    println!("Token expires in: {} seconds", token.expires_in);
    println!("Token remaining seconds: {}", token.remaining_seconds());
    println!("Token is expired: {}", token.is_expired());

    if !token.is_expired() {
        println!("✓ Token is valid");
    } else {
        println!("✗ Token is expired");
    }
    println!();

    // Test 3: Test needs_refresh logic
    println!("Test 3: Testing needs_refresh logic...");
    let needs_refresh = client.token_provider().needs_refresh()?;
    println!("Token needs refresh: {}", needs_refresh);

    if !needs_refresh {
        println!("✓ Token doesn't need refresh");
    } else {
        println!("✗ Token needs refresh");
    }
    println!();

    // Test 4: Try to get valid token
    println!("Test 4: Getting valid token...");
    match client.get_valid_token().await {
        Ok(valid_token) => {
            println!("✓ Got valid token: {}", valid_token.access_token);
            println!(
                "Token expires in: {} seconds",
                valid_token.remaining_seconds()
            );
        }
        Err(e) => {
            println!("✗ Failed to get valid token: {}", e);
            println!("Note: This is expected if refresh token is invalid/mock");
        }
    }
    println!();

    // Test 5: Test with an expired token
    println!("Test 5: Testing with expired token...");
    let expired_token = AccessToken {
        access_token: "expired_token".to_string(),
        expires_in: 1, // 1 second
        refresh_token: "mock_refresh_token".to_string(),
        scope: "basic netdisk".to_string(),
        session_key: "".to_string(),
        session_secret: "".to_string(),
        acquired_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - 10, // 10 seconds ago
    };

    client.set_access_token(expired_token.clone())?;
    println!(
        "Expired token remaining seconds: {}",
        expired_token.remaining_seconds()
    );
    println!("Expired token is_expired: {}", expired_token.is_expired());

    if expired_token.is_expired() {
        println!("✓ Expired token correctly identified");
    } else {
        println!("✗ Failed to identify expired token");
    }
    println!();

    println!("=== Test completed ===");
    Ok(())
}
