use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    let args: Vec<String> = std::env::args().collect();

    let remote_path = if args.len() >= 2 {
        &args[1]
    } else {
        "/upload/hello_bytes.txt"
    };

    println!("=== Baidu NetDisk Bytes Upload (Simple) ===");
    println!("Remote path: {}", remote_path);
    println!();

    let test_data = b"Hello from upload_bytes! This is a simple byte array upload test.";
    println!("Uploading {} bytes of data...", test_data.len());

    let start_time = std::time::Instant::now();

    let response = client
        .upload()
        .upload_bytes(&token, test_data, remote_path)
        .await?;

    println!("Bytes uploaded successfully!");
    println!("  FS ID: {}", response.fs_id);
    println!("  Server filename: {:?}", response.server_filename);
    println!("  Path: {}", response.path);
    println!("  Size: {} bytes", response.size);
    println!("  Category: {}", response.category);
    println!("  MD5: {}", response.md5.unwrap_or_default());
    println!("  Upload time: {:?}", start_time.elapsed());

    Ok(())
}
