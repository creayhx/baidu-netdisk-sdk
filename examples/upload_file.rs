use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <local_file> <remote_path>", args[0]);
        println!("Example: {} test.txt /apps/test/test.txt", args[0]);
        return Ok(());
    }

    let local_file = &args[1];
    let remote_path = &args[2];

    println!("=== Baidu NetDisk File Upload (Simple) ===");
    println!("Local file: {}", local_file);
    println!("Remote path: {}", remote_path);
    println!();

    let start_time = std::time::Instant::now();

    let response = client.upload().upload_file(local_file, remote_path).await?;

    println!("File uploaded successfully!");
    println!("  FS ID: {}", response.fs_id);
    println!("  Server filename: {:?}", response.server_filename);
    println!("  Path: {}", response.path);
    println!("  Size: {} bytes", response.size);
    println!("  Category: {}", response.category);
    println!("  MD5: {}", response.md5.unwrap_or_default());
    println!("  Upload time: {:?}", start_time.elapsed());

    Ok(())
}
