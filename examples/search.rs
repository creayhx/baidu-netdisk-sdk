use baidu_netdisk_sdk::{BaiduNetDiskClient, SearchOptions, SemanticSearchOptions};
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

fn print_file_list(files: &[baidu_netdisk_sdk::FileInfo], title: &str) {
    println!("{}", title);
    if files.is_empty() {
        println!("  (No files found)");
        return;
    }
    for file in files.iter().take(10) {
        let size_str = file
            .size
            .map(|s| format_size(s))
            .unwrap_or_else(|| "N/A".to_string());
        let file_type = if file.isdir == Some(1) {
            "[DIR]"
        } else {
            "[FILE]"
        };
        println!("  {} {} ({})", file_type, file.name, size_str);
    }
    if files.len() > 10 {
        println!("  ... and {} more files", files.len() - 10);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    println!("=== Baidu NetDisk Search API Test ===");
    println!();

    // Part 1: Keyword Search (simple)
    println!("--- Part 1: Keyword Search (simple) ---");
    let (files, has_more) = client
        .file()
        .search_files(&token, "test", "/")
        .await
        .map_err(|e| format!("Search failed: {}", e))?;
    print_file_list(&files, &format!("Results (has_more: {}):", has_more));
    wait_for_rate_limit().await;

    // Part 2: Keyword Search with category filter
    println!("\n--- Part 2: Keyword Search with Category Filter ---");
    println!("Searching for '*' in category 4 (Documents):");
    let options = SearchOptions::new("/").category(4);

    let (files, has_more) = client
        .file()
        .search_files_with_options(&token, "*", options)
        .await
        .map_err(|e| format!("Search failed: {}", e))?;
    print_file_list(&files, &format!("Results (has_more: {}):", has_more));
    wait_for_rate_limit().await;

    // Part 3: Keyword Search with recursion
    println!("\n--- Part 3: Keyword Search with Recursion ---");
    println!("Recursive search for '*':");
    let options = SearchOptions::new("/").recursion(true);
    let (files, has_more) = client
        .file()
        .search_files_with_options(&token, "*", options)
        .await
        .map_err(|e| format!("Search failed: {}", e))?;
    print_file_list(&files, &format!("Results (has_more: {}):", has_more));
    wait_for_rate_limit().await;

    // Part 4: Semantic Search (simple)
    println!("\n--- Part 4: Semantic Search (simple query) ---");
    println!("Query: \"find text files\"");
    let files = client
        .file()
        .semantic_search(&token, "find text files")
        .await
        .map_err(|e| format!("Semantic search failed: {}", e))?;
    print_file_list(&files, "Results:");

    // Part 5: Semantic Search with options
    println!("\n--- Part 5: Semantic Search with Options ---");
    println!("Query: \"search for images\" (semantic mode)");
    let options = SemanticSearchOptions::new().search_type(1);
    let files = client
        .file()
        .semantic_search_with_options(&token, "search for images", options)
        .await
        .map_err(|e| format!("Semantic search failed: {}", e))?;
    print_file_list(&files, "Results:");

    // Part 6: Semantic Search with auto mode
    println!("\n--- Part 6: Semantic Search (auto mode) ---");
    println!("Query: \"video files\" (auto: uses keyword for short queries)");
    let options = SemanticSearchOptions::new().search_type(2);
    let files = client
        .file()
        .semantic_search_with_options(&token, "video files", options)
        .await
        .map_err(|e| format!("Semantic search failed: {}", e))?;
    print_file_list(&files, "Results:");

    println!("\n=== Test Completed ===");

    Ok(())
}
