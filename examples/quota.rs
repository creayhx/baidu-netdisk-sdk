use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== Baidu NetDisk Quota Test ===\n");

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    println!("1. Testing get_quota...");
    let quota = client.quota().get_quota().await?;

    println!("\n=== Basic Quota Information ===");
    println!(
        "Total:    {}",
        baidu_netdisk_sdk::CapacityInfo::format_bytes(quota.total)
    );
    println!(
        "Used:     {}",
        baidu_netdisk_sdk::CapacityInfo::format_bytes(quota.used)
    );
    println!(
        "Free:     {}",
        baidu_netdisk_sdk::CapacityInfo::format_bytes(quota.free)
    );

    println!("\n2. Testing get_quota_with_expire...");
    let capacity = client.quota().get_quota_with_expire().await?;
    println!("\n=== Detailed Capacity Information ===");
    println!("Total:        {}", capacity.format_total());
    println!(
        "Used:         {} ({:.2}%)",
        capacity.format_used(),
        capacity.usage_percentage()
    );
    println!("Free:         {}", capacity.format_free());
    println!(
        "Expired:      {}",
        if capacity.expire { "Yes" } else { "No" }
    );

    println!("\n3. Testing get_capacity (check_free only)...");
    let capacity_free = client.quota().get_capacity(true, false).await?;

    println!("\n=== Capacity with Free Check ===");
    println!("Total:        {}", capacity_free.format_total());
    println!("Used:         {}", capacity_free.format_used());
    println!("Free:         {}", capacity_free.format_free());

    println!("\n4. Testing get_capacity (check_expire only)...");
    let capacity_expire = client.quota().get_capacity(false, true).await?;

    println!("\n=== Capacity with Expire Check ===");
    println!("Total:        {}", capacity_expire.format_total());
    println!("Used:         {}", capacity_expire.format_used());
    println!(
        "Expired:      {}",
        if capacity_expire.expire { "Yes" } else { "No" }
    );

    println!("\n=== All quota tests completed successfully ===");

    Ok(())
}
