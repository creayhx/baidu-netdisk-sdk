/// Download module test example
///
/// This example demonstrates the download functionality including:
/// - Getting file info to obtain fs_id
/// - Single-threaded download with timing
/// - Producer-consumer parallel download (queue-based)
/// - Auto download mode (automatic selection based on file size)
/// - Download speed comparison
use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;
use std::io::{self, BufRead};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Create client
    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    // Load token from environment
    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    println!("=== Baidu NetDisk Download Test ===");
    println!("Enter the remote file path to download (e.g., /upload/test.txt):");

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut input = String::new();
    reader.read_line(&mut input)?;
    let remote_path = input.trim();

    if remote_path.is_empty() {
        eprintln!("Error: File path cannot be empty");
        return Ok(());
    }

    // Step 1: Get file info (to obtain fs_id)
    println!("\n--- Step 1: Get file info (to obtain fs_id) ---");
    println!("Press Enter to continue...");
    reader.read_line(&mut String::new())?;

    let file_info = client.file().get_file_info(&token, remote_path).await?;
    let file_size = file_info.size.unwrap_or(0);
    let fs_id = file_info.fs_id.ok_or_else(|| "File has no fs_id")?;
    let file_meta = client.file().get_file_meta(&token, fs_id).await?;

    println!("File Info:");
    println!("  Name: {}", file_info.name);
    println!("  Path: {}", file_info.path);
    println!(
        "  Size: {} bytes ({:.2} MB)",
        file_size,
        file_size as f64 / (1024.0 * 1024.0)
    );
    println!("  FS ID: {:?}", file_info.fs_id);

    // Step 2: Single-threaded download (skip for large files > 50MB)
    println!("\n--- Step 2: Single-threaded download ---");
    if file_size > 50 * 1024 * 1024 {
        println!(
            "File size {} bytes exceeds 50MB, skipping single-threaded download...",
            file_size
        );
    } else {
        println!("Press Enter to continue...");
        reader.read_line(&mut String::new())?;

        let single_save_path = format!("./download_single_{}", file_info.name);
        let start_time = Instant::now();

        match client
            .download()
            .download_single(&token, remote_path, &single_save_path)
            .await
        {
            Ok(_) => {
                let duration = start_time.elapsed();
                print_download_stats("Single-threaded", file_size, duration);
                println!("Single-threaded download successful: {}", single_save_path);
            }
            Err(e) => eprintln!("Single-threaded download failed: {}", e),
        }
    }

    // Step 3: Producer-consumer parallel download (queue-based)
    println!("\n--- Step 3: Parallel download (Producer-Consumer queue-based) ---");
    println!("Press Enter to continue...");
    reader.read_line(&mut String::new())?;

    let parallel_save_path = format!("./download_parallel_{}", file_info.name);
    let start_time = Instant::now();

    match client
        .download()
        .download_parallel_multi_threaded(&token, &file_meta, &parallel_save_path, Some(12))
        .await
    {
        Ok(_) => {
            let duration = start_time.elapsed();
            print_download_stats("Parallel (Producer-Consumer)", file_size, duration);
            println!("Parallel download successful: {}", parallel_save_path);
        }
        Err(e) => eprintln!("Parallel download failed: {}", e),
    }

    // Step 4: Auto download (recommended)
    println!("\n--- Step 4: Auto download (recommended) ---");
    println!("Press Enter to continue...");
    reader.read_line(&mut String::new())?;

    let auto_save_path = format!("./download_auto_{}", file_info.name);
    let start_time = Instant::now();

    match client
        .download()
        .auto_download(&token, remote_path, &auto_save_path)
        .await
    {
        Ok(_) => {
            let duration = start_time.elapsed();
            print_download_stats("Auto download", file_size, duration);
            println!("Auto download successful: {}", auto_save_path);
        }
        Err(e) => eprintln!("Auto download failed: {}", e),
    }

    println!("\n=== Download test completed ===");

    Ok(())
}

/// Print download statistics
fn print_download_stats(method: &str, file_size: u64, duration: Duration) {
    let seconds = duration.as_secs_f64();
    let mb_per_sec = if seconds > 0.0 {
        (file_size as f64 / (1024.0 * 1024.0)) / seconds
    } else {
        0.0
    };

    println!("{} Download Stats:", method);
    println!("  Duration: {:.2} seconds", seconds);
    println!("  Speed: {:.2} MB/s", mb_per_sec);
}
