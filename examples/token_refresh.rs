use baidu_netdisk_sdk::auth::AccessToken;
use baidu_netdisk_sdk::BaiduNetDiskClient;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Token Refresh Test ===");
    println!();

    // Create client with auto-refresh enabled
    let client = BaiduNetDiskClient::builder()
        .auto_refresh(true)
        .refresh_ahead_seconds(60)
        .build()?;

    println!("Client created successfully");
    println!();

    // Step 1: Load token from environment
    println!("Step 1: Loading token from environment...");
    client.load_token_from_env()?;
    println!("Token loaded successfully");

    let original_token = client.get_valid_token().await?;
    println!("Original Token: {:?}", original_token);
    println!();

    // Step 2: Test manual refresh
    println!("Step 2: Testing manual token refresh...");
    match client.token_provider().refresh_token().await {
        Ok(new_token) => {
            println!("✓ Token refreshed successfully!");
            println!("  New Access Token: {}", new_token.access_token);
            println!("  New Refresh Token: {}", new_token.refresh_token);
            println!("  Expires in: {} seconds", new_token.expires_in);
            println!();

            // Verify the tokens are different
            if new_token.access_token != original_token.access_token {
                println!("✓ Access token changed after refresh");
            } else {
                println!("✗ Access token remains the same");
            }

            // Verify refresh token may have changed
            if new_token.refresh_token != original_token.refresh_token {
                println!("✓ Refresh token changed after refresh");
            } else {
                println!("Note: Refresh token remains the same (may be expected)");
            }
        }
        Err(e) => {
            println!("✗ Failed to refresh token: {}", e);
            println!("Note: This may be due to invalid credentials or network issues");
        }
    }

    println!();

    // Step 3: Test auto-refresh with expired token simulation
    println!("Step 3: Testing auto-refresh behavior...");

    // Create an expired token to test auto-refresh
    let expired_token = AccessToken {
        access_token: "expired_access_token".to_string(),
        expires_in: 60,                                      // 60 seconds
        refresh_token: original_token.refresh_token.clone(), // Use real refresh token
        scope: "basic netdisk".to_string(),
        session_key: "".to_string(),
        session_secret: "".to_string(),
        acquired_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - 3600, // 1 hour ago (expired)
    };

    client.set_access_token(expired_token)?;

    // Now try to get valid token - should trigger auto-refresh
    match client.get_valid_token().await {
        Ok(refreshed_token) => {
            println!("✓ Auto-refresh triggered successfully!");
            println!("  New Access Token: {}", refreshed_token.access_token);
            println!(
                "  Expires in: {} seconds",
                refreshed_token.remaining_seconds()
            );
        }
        Err(e) => {
            println!("✗ Auto-refresh failed: {}", e);
            println!("Note: This may be due to invalid credentials");
        }
    }

    println!();
    println!("=== Refresh test completed ===");
    Ok(())
}
