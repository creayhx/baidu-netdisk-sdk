use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== Baidu NetDisk User Info Test ===\n");

    let client = BaiduNetDiskClient::builder()
        .app_key("your_app_key")
        .app_secret("your_app_secret")
        .build()?;
    info!("Client created successfully");

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    println!("Getting user info...");
    let user_info = client.user().get_user_info(Some("v2")).await?;

    println!("\n=== User Information ===");
    println!("Baidu Name:    {}", user_info.baidu_name);
    println!("NetDisk Name:  {}", user_info.netdisk_name);
    println!("Avatar URL:    {}", user_info.avatar_url);
    println!(
        "VIP Type:      {}",
        match user_info.vip_type {
            0 => "Regular user",
            1 => "VIP member",
            2 => "SVIP super member",
            _ => "Unknown",
        }
    );
    println!("User ID (uk):  {}", user_info.uk);

    Ok(())
}
