use baidu_netdisk_sdk::upload::SimpleUploadOptions;
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
        println!(
            "Usage: {} <local_file> <remote_path> [chunk_size] [concurrency]",
            args[0]
        );
        println!("Example: {} test.mp4 /upload/video.mp4 8388608 20", args[0]);
        println!("  - chunk_size: bytes per chunk (default: 4194304 = 4MB)");
        println!("  - concurrency: parallel uploads (default: 10)");
        return Ok(());
    }

    let local_file = &args[1];
    let remote_path = &args[2];

    let chunk_size: usize = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4 * 1024 * 1024);
    let concurrency: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(10);

    let options = SimpleUploadOptions::default()
        .chunk_size(chunk_size)
        .max_concurrency(concurrency);

    println!("=== Baidu NetDisk File Upload (Custom Options) ===");
    println!("Local file: {}", local_file);
    println!("Remote path: {}", remote_path);
    println!(
        "Chunk size: {} bytes ({:.2} MB)",
        chunk_size,
        chunk_size as f64 / 1024.0 / 1024.0
    );
    println!("Concurrency: {}", concurrency);
    println!();

    let start_time = std::time::Instant::now();

    let response = client
        .upload()
        .upload_file_with_options(local_file, remote_path, options)
        .await?;

    println!("File uploaded successfully!");
    println!("  FS ID: {}", response.fs_id);
    println!("  Path: {}", response.path);
    println!("  Size: {} bytes", response.size);
    println!("  Category: {}", response.category);
    println!("  MD5: {}", response.md5.unwrap_or_default());
    println!("  Upload time: {:?}", start_time.elapsed());

    Ok(())
}
