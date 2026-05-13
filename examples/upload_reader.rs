use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;
use std::io::BufReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    let args: Vec<String> = std::env::args().collect();

    let local_file = if args.len() >= 2 {
        &args[1]
    } else {
        println!("Usage: {} <local_file> [remote_path]", args[0]);
        println!("Example: {} test.txt /upload/test.txt", args[0]);
        return Ok(());
    };

    let remote_path = if args.len() >= 3 {
        &args[2]
    } else {
        "/upload/test_reader.txt"
    };

    println!("=== Baidu NetDisk Reader Upload ===");
    println!("Local file: {}", local_file);
    println!("Remote path: {}", remote_path);
    println!();

    let file = std::fs::File::open(local_file)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    println!("File size: {} bytes", file_size);

    let mut reader = BufReader::new(file);

    let start_time = std::time::Instant::now();

    let response = client
        .upload()
        .upload_reader(&mut reader, file_size, remote_path)
        .await?;

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
