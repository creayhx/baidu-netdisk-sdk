use baidu_netdisk_sdk::{BaiduNetDiskClient, PrecreateOptions};
use log::info;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn calculate_md5<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(format!("{:x}", md5::compute(&buffer)))
}

fn get_file_md5blocks<P: AsRef<Path>>(
    path: P,
    chunk_size: usize,
) -> Result<(u64, Vec<String>), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let total_size = metadata.len() as u64;

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    let mut block_list = Vec::new();

    loop {
        buffer.clear();
        let bytes_read = reader
            .by_ref()
            .take(chunk_size as u64)
            .read_to_end(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        block_list.push(format!("{:x}", md5::compute(&buffer)));
    }

    Ok((total_size, block_list))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <local_file> <remote_path>", args[0]);
        println!("Example: {} test.txt /apps/test/test.txt", args[0]);
        return Ok(());
    }

    let local_file = &args[1];
    let remote_path = &args[2];

    println!("=== Baidu NetDisk Preupload Test ===");
    println!("Local file: {}", local_file);
    println!("Remote path: {}", remote_path);
    println!();

    let (file_size, block_list) = get_file_md5blocks(local_file, 4 * 1024 * 1024)?;
    let file_md5 = calculate_md5(local_file)?;

    println!("File size: {} bytes", file_size);
    println!("File MD5: {}", file_md5);
    println!("Block count: {}", block_list.len());
    println!("Block list: {:?}", block_list);
    println!();

    let options = PrecreateOptions::new(remote_path, file_size, block_list)
        .content_md5(&file_md5)
        .rtype(1);

    println!("Sending precreate request...");
    match client.upload().precreate(&token, options).await {
        Ok(response) => {
            println!("Precreate success!");
            println!("  Upload ID: {}", response.uploadid);
            println!("  Path: {:?}", response.path);
            println!("  Return type: {}", response.return_type);
            println!("  Block list to upload: {:?}", response.block_list);
        }
        Err(e) => {
            println!("Precreate failed: {}", e);
        }
    }

    Ok(())
}
