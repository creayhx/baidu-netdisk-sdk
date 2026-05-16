use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;

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

    client.load_token_from_env()?;
    info!("Token loaded successfully");

    let test_dir = "/apps/chapters";
    let page_size = 10;

    println!("=== Baidu NetDisk ListAll Test ===");
    println!("Directory: {}", test_dir);
    println!("Page size: {}\n", page_size);

    let mut start = 0;
    let mut page_num = 1;
    let mut total_files = 0;
    let mut has_more = true;

    while has_more {
        // 使用简化的 list_all 方法，默认开启递归
        match client.file().list_all(test_dir, start, page_size).await {
            Ok(result) => {
                has_more = result.has_more;

                println!("--- Page {} ---", page_num);
                println!("Found {} files in this page", result.list.len());

                for file in &result.list {
                    let size_str = file
                        .size
                        .map(|s| format_size(s))
                        .unwrap_or_else(|| "N/A".to_string());
                    let is_dir = file.isdir.map(|i| i == 1).unwrap_or(false);
                    let file_type = if is_dir { "DIR" } else { "FILE" };

                    println!("  [{}] {} ({})", file_type, file.name, size_str);
                }

                total_files += result.list.len();

                if has_more {
                    println!("\nPress Enter to continue to page {}...", page_num + 1);
                    println!("Next cursor: {:?}", result.cursor);
                    // 使用 cursor 作为下一页的 start，用户无需手动计算
                    if let Some(cursor) = result.cursor {
                        start = cursor as i32;
                    }
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                }
                page_num += 1;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    println!("\n=== ListAll Test Completed ===");
    println!("Total files found: {}", total_files);

    Ok(())
}
