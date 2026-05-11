use baidu_netdisk_sdk::BaiduNetDiskClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Baidu NetDisk Auth Flow Example ===");
    println!();

    let client = BaiduNetDiskClient::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Client created successfully");
    println!();

    println!("Step 1: Getting device code...");
    let device_code = client.authorize().get_device_code().await?;

    println!("Device Code Info:");
    println!("  User Code: {}", device_code.user_code);
    println!("  Verification URL: {}", device_code.verification_url);
    println!("  QR Code URL: {}", device_code.qrcode_url);
    println!("  Interval: {} seconds", device_code.interval);
    println!(
        "  Expires in: {} seconds",
        device_code.expires_at - chrono::Utc::now().timestamp() as u64
    );
    println!();

    println!("Please visit the verification URL above and enter the user code to authorize.");
    println!("Waiting for authorization...");
    println!();

    let max_attempts = 30;
    let mut attempts = 0;

    let access_token = loop {
        attempts += 1;

        if attempts > max_attempts {
            return Err("Max attempts reached, authorization timeout".into());
        }

        println!("Attempt {}/{}", attempts, max_attempts);

        match client.authorize().request_access_token(&device_code).await {
            Ok(Some(token)) => {
                println!();
                println!("✅ Authorization successful!");
                break token;
            }
            Ok(None) => {
                println!("  Authorization pending, waiting...");
                tokio::time::sleep(Duration::from_secs(device_code.interval as u64)).await;
                continue;
            }
            Err(e) => {
                println!("  Error: {}", e);
                tokio::time::sleep(Duration::from_secs(device_code.interval as u64)).await;
                continue;
            }
        }
    };

    println!();
    println!("=== Access Token Info ===");
    println!("Access Token: {}", access_token.access_token);
    println!("Expires in: {} seconds", access_token.expires_in);
    println!("Refresh Token: {}", access_token.refresh_token);
    println!("Scope: {}", access_token.scope);
    println!("Session Key: {}", access_token.session_key);
    println!("Session Secret: {}", access_token.session_secret);
    println!("Acquired At: {}", access_token.acquired_at);
    println!();

    println!("=== Auth Flow Completed Successfully ===");

    Ok(())
}
