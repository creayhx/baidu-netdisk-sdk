use baidu_netdisk_sdk::{
    BaiduNetDiskClient, BtListOptions, DocumentListOptions, ImageListOptions, VideoListOptions,
};
use log::info;
use tokio::time::{sleep, Duration};

async fn wait_for_rate_limit() {
    sleep(Duration::from_millis(500)).await;
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    let test_dir = "/apps/product";
    let num_per_page = 5;

    println!("=== Baidu NetDisk Fixed Category List Test ===");
    println!("Directory: {}", test_dir);
    println!("Items per page: {}\n", num_per_page);

    println!("--- Part 1: Document List (doclist) ---");
    match client
        .file()
        .list_documents(&token, test_dir, 1, num_per_page)
        .await
    {
        Ok(files) => {
            println!("Found {} documents", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- Part 2: Image List (imagelist) ---");
    match client
        .file()
        .list_images(&token, test_dir, 1, num_per_page)
        .await
    {
        Ok(files) => {
            println!("Found {} images", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- Part 3: Video List (videolist) ---");
    match client
        .file()
        .list_videos(&token, test_dir, 1, num_per_page)
        .await
    {
        Ok(files) => {
            println!("Found {} videos", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- Part 4: BT List (btlist) ---");
    match client
        .file()
        .list_torrents(&token, test_dir, 1, num_per_page)
        .await
    {
        Ok(files) => {
            println!("Found {} torrent files", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n=== Test with Options (recursion=1) ===");
    println!("Directory: {}\n", test_dir);

    println!("--- Document List with recursion ---");
    let options = DocumentListOptions::new(test_dir)
        .recursion(1)
        .num(num_per_page);
    match client
        .file()
        .list_documents_with_options(&token, options)
        .await
    {
        Ok(files) => {
            println!("Found {} documents", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- Image List with recursion ---");
    let options = ImageListOptions::new(test_dir)
        .recursion(1)
        .num(num_per_page);
    match client
        .file()
        .list_images_with_options(&token, options)
        .await
    {
        Ok(files) => {
            println!("Found {} images", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- Video List with recursion ---");
    let options = VideoListOptions::new(test_dir)
        .recursion(1)
        .num(num_per_page);
    match client
        .file()
        .list_videos_with_options(&token, options)
        .await
    {
        Ok(files) => {
            println!("Found {} videos", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    wait_for_rate_limit().await;

    println!("\n--- BT List with recursion ---");
    let options = BtListOptions::new(test_dir).recursion(1).num(num_per_page);
    match client
        .file()
        .list_torrents_with_options(&token, options)
        .await
    {
        Ok(files) => {
            println!("Found {} torrent files", files.len());
            for file in &files {
                let size_str = file
                    .size
                    .map(|s| format_size(s))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  - {} ({})", file.name, size_str);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\n=== Test Completed ===");

    Ok(())
}
