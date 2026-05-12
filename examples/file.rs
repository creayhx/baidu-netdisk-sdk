use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    println!("=== Baidu NetDisk File Management Test ===\n");

    // Create client - will automatically load from .env file
    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    // Load token from environment variables
    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    // === Test 0: List root directory to verify permission ===
    println!("=== Test 0: List Root Directory ===");
    println!("Listing contents of root directory: /");
    wait_for_enter();

    match client.file().list_directory(&token, "/").await {
        Ok(files) => {
            println!("✓ Successfully listed root directory!");
            println!("  Found {} items:", files.len());
            for file in files {
                let type_str = if file.isdir == Some(1) { "DIR" } else { "FILE" };
                let size_str = if let Some(s) = file.size {
                    format!("{} bytes", s)
                } else {
                    "N/A".to_string()
                };
                println!("    [{}] {} ({})", type_str, file.name, size_str);
            }
        }
        Err(e) => {
            println!("! Failed to list root directory: {}", e);
            println!("  Please check your token and permissions.");
            return Ok(());
        }
    }

    // Test folder and file paths
    let test_folder = "/upload";
    let test_file = "/upload/subscribe.json";
    let test_folder_new = "/upload_new";
    let test_file_new = "/upload_new/subscribe.json";
    let test_file_renamed = "/upload_new/subscribe_renamed.json";

    // === Test 1: Check if /upload folder exists ===
    println!("\n=== Test 1: Check /upload folder ===");
    println!("Checking folder: {}", test_folder);
    wait_for_enter();

    match client.file().get_file_info(&token, test_folder).await {
        Ok(info) => {
            println!("✓ Folder /upload exists!");
            println!("  fs_id: {:?}", info.fs_id);
            println!("  path: {}", info.path);
            println!("  isdir: {:?}", info.isdir);
            println!("  ctime: {:?}", info.ctime);
            println!("  mtime: {:?}", info.mtime);
        }
        Err(e) => {
            println!(
                "! Folder /upload does not exist or cannot be accessed: {}",
                e
            );
            println!("  Please create the folder manually or check your permissions.");
            return Ok(());
        }
    }

    // === Test 2: List upload directory first ===
    println!("\n=== Test 2: List /upload Directory ===");
    println!("Listing contents of: {}", test_folder);
    wait_for_enter();

    let upload_files = match client.file().list_directory(&token, test_folder).await {
        Ok(files) => {
            println!("✓ Directory listed successfully!");
            println!("  Found {} items:", files.len());
            for file in &files {
                let type_str = if file.isdir == Some(1) { "DIR" } else { "FILE" };
                let size_str = if let Some(s) = file.size {
                    format!("{} bytes", s)
                } else {
                    "N/A".to_string()
                };
                println!("    [{}] {} ({})", type_str, file.name, size_str);
            }
            Some(files)
        }
        Err(e) => {
            println!("! Failed to list directory: {}", e);
            None
        }
    };

    // === Test 3: Check if subscribe.json exists ===
    println!("\n=== Test 3: Check subscribe.json file ===");
    println!("Checking file: {}", test_file);
    wait_for_enter();

    match client.file().get_file_info(&token, test_file).await {
        Ok(info) => {
            println!("✓ File {} exists!", test_file);
            println!("  fs_id: {:?}", info.fs_id);
            println!("  path: {}", info.path);
            println!("  name: {}", info.name);
            println!("  size: {:?} bytes", info.size);
            println!("  isdir: {:?}", info.isdir);
            println!("  md5: {:?}", info.md5);
            println!("  ctime: {:?}", info.ctime);
            println!("  mtime: {:?}", info.mtime);
        }
        Err(e) => {
            println!(
                "! File {} does not exist or cannot be accessed: {}",
                test_file, e
            );
            // If directory content is listed, show to user
            if let Some(files) = upload_files {
                println!("  Available files in /upload:");
                for file in files {
                    let type_str = if file.isdir == Some(1) { "DIR" } else { "FILE" };
                    println!("    [{}] {}", type_str, file.name);
                }
            }
            return Ok(());
        }
    }

    // === Test 4: Create new folder ===
    println!("\n=== Test 4: Create Folder ===");
    println!("Creating folder: {}", test_folder_new);
    wait_for_enter();

    match client.file().create_folder(&token, test_folder_new).await {
        Ok(folder) => {
            println!("✓ Folder created successfully!");
            println!("  fs_id: {:?}", folder.fs_id);
            println!("  path: {}", folder.path);
            println!("  ctime: {:?}", folder.ctime);
        }
        Err(e) => {
            println!("! Failed to create folder: {}", e);
            return Ok(());
        }
    }

    // === Test 5: Copy file ===
    println!("\n=== Test 5: Copy File ===");
    println!("Copying file: {} -> {}", test_file, test_folder_new);
    wait_for_enter();

    match client
        .file()
        .copy_file(&token, test_file, test_folder_new)
        .await
    {
        Ok(_) => println!("✓ File copied successfully!"),
        Err(e) => {
            println!("! File copy failed: {}", e);
            return Ok(());
        }
    }

    // === Test 6: Verify copied file ===
    println!("\n=== Test 6: Verify Copied File ===");
    println!("Listing directory: {}", test_folder_new);
    wait_for_enter();

    let copied_files = match client.file().list_directory(&token, test_folder_new).await {
        Ok(files) => {
            println!("✓ Directory listed!");
            println!("  Found {} items:", files.len());
            for file in &files {
                let type_str = if file.isdir == Some(1) { "DIR" } else { "FILE" };
                let size_str = if let Some(s) = file.size {
                    format!("{} bytes", s)
                } else {
                    "N/A".to_string()
                };
                println!("    [{}] {} ({})", type_str, file.name, size_str);
            }
            Some(files)
        }
        Err(e) => {
            println!("! Failed to list directory: {}", e);
            None
        }
    };

    println!("\nChecking copied file: {}", test_file_new);
    wait_for_enter();

    match client.file().get_file_info(&token, test_file_new).await {
        Ok(info) => {
            println!("✓ Copied file verified!");
            println!("  name: {}", info.name);
            println!("  size: {:?} bytes", info.size);
        }
        Err(e) => {
            println!("! Failed to verify copied file: {}", e);
            // If we listed the directory, show available files
            if let Some(files) = copied_files {
                println!("  Available files:");
                for file in files {
                    println!("    {}", file.name);
                }
            }
            return Ok(());
        }
    }

    // === Test 7: Rename file ===
    println!("\n=== Test 7: Rename File ===");
    println!("Renaming file: {} -> {}", test_file_new, test_file_renamed);
    wait_for_enter();

    match client
        .file()
        .rename(&token, test_file_new, "subscribe_renamed.json")
        .await
    {
        Ok(_) => println!("✓ File renamed successfully!"),
        Err(e) => {
            println!("! File rename failed: {}", e);
            return Ok(());
        }
    }

    // === Test 8: Move file ===
    println!("\n=== Test 8: Move File ===");
    println!("Moving file: {} -> {}", test_file_renamed, test_folder);
    wait_for_enter();

    match client
        .file()
        .move_file(&token, test_file_renamed, test_folder)
        .await
    {
        Ok(_) => println!("✓ File moved successfully!"),
        Err(e) => {
            println!("! File move failed: {}", e);
            return Ok(());
        }
    }

    // === Test 9: Verify moved file ===
    println!("\n=== Test 9: Verify Moved File ===");
    let moved_file_path = "/upload/subscribe_renamed.json";
    println!("Checking moved file: {}", moved_file_path);
    wait_for_enter();

    match client.file().get_file_info(&token, moved_file_path).await {
        Ok(info) => {
            println!("✓ Moved file verified!");
            println!("  name: {}", info.name);
            println!("  path: {}", info.path);
        }
        Err(e) => {
            println!("! Failed to verify moved file: {}", e);
            return Ok(());
        }
    }

    // === Test 10: Delete renamed file ===
    println!("\n=== Test 10: Delete File ===");
    println!("Deleting file: {}", moved_file_path);
    wait_for_enter();

    match client.file().delete(&token, moved_file_path).await {
        Ok(_) => println!("✓ File deleted successfully!"),
        Err(e) => {
            println!("! File deletion failed: {}", e);
            return Ok(());
        }
    }

    // === Test 11: Cleanup - Delete test folder ===
    println!("\n=== Test 11: Cleanup ===");
    println!("Deleting empty folder: {}", test_folder_new);
    wait_for_enter();

    match client.file().delete(&token, test_folder_new).await {
        Ok(_) => println!("✓ Test folder deleted successfully!"),
        Err(e) => {
            println!("! Failed to delete test folder: {}", e);
            return Ok(());
        }
    }

    // === Final verification ===
    println!("\n=== Final Verification ===");
    println!("Listing final contents of: {}", test_folder);
    wait_for_enter();

    match client.file().list_directory(&token, test_folder).await {
        Ok(files) => {
            println!("✓ Final directory listing:");
            println!("  Found {} items:", files.len());
            for file in files {
                let type_str = if file.isdir == Some(1) { "DIR" } else { "FILE" };
                println!("    [{}] {}", type_str, file.name);
            }
        }
        Err(e) => {
            println!("! Failed to list directory: {}", e);
            return Ok(());
        }
    }

    println!("\n=== All file management tests completed successfully! ===");

    Ok(())
}

/// Wait for user to press Enter before continuing
fn wait_for_enter() {
    println!("Press Enter to continue...");
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let _ = lines.next();
}
