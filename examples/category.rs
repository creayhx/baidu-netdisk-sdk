use baidu_netdisk_sdk::{BaiduNetDiskClient, Category, CategorySearchOptions};
use log::info;
use tokio::time::{sleep, Duration};

async fn wait_for_rate_limit() {
    sleep(Duration::from_millis(500)).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    let test_dir = "/apps/product";

    println!("=== Baidu NetDisk Category Test ===\n");

    println!("Testing directory: {}\n", test_dir);

    println!("=== Part 1: Global Category Counts (All files) ===");
    println!("----------------------------------------");
    test_global_counts(&client).await?;

    println!("\n\n=== Part 2: Category Counts in {} ===", test_dir);
    println!("----------------------------------------");
    test_directory_counts(&client, &test_dir).await?;

    println!("\n\n=== Part 3: List Files in Each Category ===");
    println!("----------------------------------------");
    test_list_category_files(&client, &test_dir).await?;

    println!("\n\n=== Category Test Completed ===");

    Ok(())
}

async fn test_global_counts(client: &BaiduNetDiskClient) -> Result<(), Box<dyn std::error::Error>> {
    let categories = [
        (Category::Video, "Video"),
        (Category::Music, "Music"),
        (Category::Image, "Image"),
        (Category::Document, "Document"),
        (Category::Application, "Application"),
        (Category::Other, "Other"),
        (Category::Torrent, "Torrent"),
    ];

    for (category, name) in categories {
        let options = CategorySearchOptions::new()
            .parent_path("/")
            .recursion(1)
            .limit(1);

        match client
            .file()
            .search_category_files_with_options(&category.as_u32().to_string(), options)
            .await
        {
            Ok((_, total)) => {
                println!("  {}: {} files", name, total);
            }
            Err(e) => {
                println!("  {}: Error - {}", name, e);
            }
        }

        wait_for_rate_limit().await;
    }

    Ok(())
}

async fn test_directory_counts(
    client: &BaiduNetDiskClient,
    dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let categories = [
        (Category::Video, "Video"),
        (Category::Music, "Music"),
        (Category::Image, "Image"),
        (Category::Document, "Document"),
        (Category::Application, "Application"),
        (Category::Other, "Other"),
        (Category::Torrent, "Torrent"),
    ];

    for (category, name) in categories {
        let options = CategorySearchOptions::new()
            .parent_path(dir)
            .recursion(1)
            .limit(1);

        match client
            .file()
            .search_category_files_with_options(&category.as_u32().to_string(), options)
            .await
        {
            Ok((_, total)) => {
                println!("  {}: {} files", name, total);
            }
            Err(e) => {
                println!("  {}: Error - {}", name, e);
            }
        }

        wait_for_rate_limit().await;
    }

    Ok(())
}

async fn test_list_category_files(
    client: &BaiduNetDiskClient,
    dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let categories = [
        (1, "Video"),
        (2, "Music"),
        (3, "Image"),
        (4, "Document"),
        (5, "Application"),
        (6, "Other"),
        (7, "Torrent"),
    ];

    for (category, name) in categories {
        println!("\n  [{}] Files:", name);

        let options = CategorySearchOptions::new()
            .parent_path(dir)
            .recursion(1)
            .limit(10);

        match client
            .file()
            .search_category_files_with_options(&category.to_string(), options)
            .await
        {
            Ok((files, total)) => {
                if files.is_empty() {
                    println!("    (no files)");
                } else {
                    for file in files.iter().take(5) {
                        let size_str = file
                            .size
                            .map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0)))
                            .unwrap_or_else(|| "N/A".to_string());
                        println!("    - {} ({})", file.name, size_str);
                    }
                    if files.len() > 5 && total > 5 {
                        println!("    ... and {} more files", total - 5);
                    }
                    println!("    Total in this category: {}", total);
                }
            }
            Err(e) => {
                println!("    Error - {}", e);
            }
        }

        wait_for_rate_limit().await;
    }

    Ok(())
}
