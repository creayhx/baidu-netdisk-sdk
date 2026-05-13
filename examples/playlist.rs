use baidu_netdisk_sdk::playlist::VideoQuality;
use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    println!("=== Baidu NetDisk Playlist Test ===\n");

    // Create client - will automatically load from .env file
    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    // Load token from environment variables
    client.load_token_from_env()?;
    info!("Token loaded successfully");

    // === Part 1: Get Playlist List ===
    println!("=== Part 1: Get Playlist List ===");
    println!("This will fetch all playlists from your Baidu NetDisk");
    wait_for_enter();

    let playlists = match client.playlist().get_playlist_list().await {
        Ok(pl) => {
            println!("✓ Successfully retrieved playlist list!");
            println!(
                "  Has more: {}",
                if pl.has_more == 1 { "Yes" } else { "No" }
            );
            println!("  Found {} playlist(s):", pl.list.len());
            for (idx, playlist) in pl.list.iter().enumerate() {
                println!("    {}. {}", idx + 1, playlist.name);
                println!("       mb_id: {}", playlist.mb_id);
                println!("       file_count: {}", playlist.file_count);
                println!("       btype: {}", playlist.btype);
                println!("       bstype: {}", playlist.bstype);
                println!("       ctime: {}", playlist.ctime);
                println!("       mtime: {}", playlist.mtime);
            }
            Some(pl)
        }
        Err(e) => {
            println!("! Failed to get playlist list: {}", e);
            None
        }
    };

    // === Part 2: Get Playlist File List ===
    println!("\n=== Part 2: Get Playlist File List ===");
    println!("mb_id is the unique identifier for a playlist (numeric).");
    if playlists.is_some() && !playlists.as_ref().unwrap().list.is_empty() {
        println!("Available playlists, please select one's mb_id:");
        for (idx, playlist) in playlists.as_ref().unwrap().list.iter().enumerate() {
            println!(
                "  {}. {} - mb_id: {}",
                idx + 1,
                playlist.name,
                playlist.mb_id
            );
        }
    }
    println!("Please enter mb_id, or press Enter to skip:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input_trimmed = input.trim();

    let playlist_mb_id = if input_trimmed.is_empty() && playlists.is_some() {
        // If user just pressed enter and we have playlists, use the first one
        println!("  No input provided, using the first playlist...");
        playlists
            .as_ref()
            .and_then(|pl| pl.list.first())
            .map(|p| p.mb_id)
    } else {
        // Try to parse as number
        match input_trimmed.parse::<u64>() {
            Ok(mb_id) => Some(mb_id),
            Err(_) => {
                println!("  Invalid input, please enter a valid mb_id");
                None
            }
        }
    };

    let playlist_files = if let Some(mb_id) = playlist_mb_id {
        println!("\nGetting files from playlist with mb_id: {}", mb_id);
        wait_for_enter();

        match client.playlist().get_playlist_file_list(mb_id).await {
            Ok(pl_files) => {
                println!("✓ Successfully retrieved playlist file list!");
                println!(
                    "  Has more: {}",
                    if pl_files.has_more == 1 { "Yes" } else { "No" }
                );
                println!("  Found {} file(s) in playlist:", pl_files.list.len());
                for (idx, file) in pl_files.list.iter().enumerate() {
                    println!(
                        "    {}. {}",
                        idx + 1,
                        file.server_filename.as_deref().unwrap_or("(no name)")
                    );
                    println!("       fs_id: {}", file.fs_id);
                    println!("       path: {}", file.path);
                    if let Some(size) = &file.size {
                        println!("       size: {}", size);
                    }
                    if let Some(category) = &file.category {
                        println!("       category: {}", category);
                    }
                    if let Some(isdir) = &file.isdir {
                        println!("       isdir: {}", isdir);
                    }
                }
                Some(pl_files)
            }
            Err(e) => {
                println!("! Failed to get playlist file list: {}", e);
                None
            }
        }
    } else {
        println!("  Skipping playlist file list as no valid mb_id was provided");
        None
    };

    // === Part 3-4: Get Media and Check M3U8 Generation ===
    println!("\n=== Part 3-4: Get Media and Check M3U8 Generation ===");
    println!("Enter the path of the media file to get playback info:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let media_path = input.trim();

    println!("\nEnter media type (e.g., M3U8_MP3_128, M3U8_AUTO_720, M3U8_AUTO_1080) or press Enter for default:");
    let mut media_type_input = String::new();
    std::io::stdin().read_line(&mut media_type_input)?;
    let media_type = if media_type_input.trim().is_empty() {
        "M3U8_AUTO_1080".to_string()
    } else {
        media_type_input.trim().to_string()
    };

    let (_selected_fs_id, selected_path) = if media_path.is_empty() && playlist_files.is_some() {
        // If user just pressed enter and we have files, use the first one
        println!("  No input provided, using the first file in playlist...");
        let file = playlist_files.as_ref().and_then(|pf| pf.list.first());
        (
            file.and_then(|f| f.fs_id.parse::<u64>().ok()),
            file.map(|f| f.path.as_str()),
        )
    } else {
        (None, Some(media_path))
    };

    let target_path = selected_path.unwrap_or_default().to_string();
    if !target_path.is_empty() {
        println!("\nStarting m3u8 check process with type: {}", media_type);
        println!("Path: {}", target_path);
        println!("Press Enter to check, any other key + Enter to skip");

        let mut attempts = 0;
        let max_attempts = 100;
        let mut last_content_len = 0;

        loop {
            // Wait for user to press enter
            let mut user_input = String::new();
            std::io::stdin().read_line(&mut user_input)?;
            if !user_input.trim().is_empty() && user_input.trim() != "y" && user_input.trim() != "Y"
            {
                println!("  Skipping further checks");
                break;
            }

            attempts += 1;
            println!("\n=== Check attempt {}/{} ===", attempts, max_attempts);

            // Try to get m3u8 content directly
            match client
                .playlist()
                .get_media_m3u8_content(&target_path, &media_type)
                .await
            {
                Ok(content) => {
                    let len = content.len();
                    let has_end_list = content.contains("#EXT-X-ENDLIST");

                    println!("✓ Got m3u8 content!");
                    println!("  Length: {} bytes", len);
                    println!(
                        "  Has #EXT-X-ENDLIST: {}",
                        if has_end_list { "Yes" } else { "No" }
                    );

                    if len > last_content_len {
                        println!("  Content is growing (was {} bytes)", last_content_len);
                        last_content_len = len;
                    } else if len > 0 && len == last_content_len {
                        println!("  Content size unchanged");
                    }

                    // Show preview
                    let preview_lines = content.lines().take(20);
                    println!("\n  Preview:");
                    for line in preview_lines {
                        println!("    {}", line);
                    }
                    if content.lines().count() > 20 {
                        println!("    ... ({} more lines)", content.lines().count() - 20);
                    }

                    if has_end_list {
                        println!("\n✅ Media is fully transcoded! Done.");
                        break;
                    } else {
                        println!("\n⏳ Media still transcoding... Press Enter to check again");
                    }
                }
                Err(e) => {
                    println!("❌ Error getting m3u8: {}", e);
                    println!("  Stopping check process");
                    break;
                }
            }

            if attempts >= max_attempts {
                println!("\n⚠️ Max attempts reached. Giving up.");
                break;
            }
        }
    } else {
        println!("  Skipping as no valid path was provided");
    }

    // === Part 5: Using Quality Enums ===
    println!("\n=== Part 5: Using Quality Enums ===");
    println!("Test the new VideoQuality and AudioQuality enums? (Y/n):");
    let mut enum_test_input = String::new();
    std::io::stdin().read_line(&mut enum_test_input)?;

    if enum_test_input.trim().to_lowercase() != "n" {
        println!("\nAvailable video qualities for different VIP levels:");
        println!("  VIP 0-1: {:?}", VideoQuality::available_for_vip_level(0));
        println!("  VIP 2+: {:?}", VideoQuality::available_for_vip_level(2));

        println!("\nHighest quality for each level:");
        println!(
            "  VIP 0: {:?} ({})",
            VideoQuality::highest_for_vip_level(0),
            VideoQuality::highest_for_vip_level(0).to_media_type()
        );
        println!(
            "  VIP 2: {:?} ({})",
            VideoQuality::highest_for_vip_level(2),
            VideoQuality::highest_for_vip_level(2).to_media_type()
        );

        // Ask user to test enum methods
        println!("\nTest enum-based m3u8 methods? (Y/n):");
        let mut enum_m3u8_input = String::new();
        std::io::stdin().read_line(&mut enum_m3u8_input)?;

        if enum_m3u8_input.trim().to_lowercase() != "n" {
            println!("\nEnter media file path:");
            let mut enum_path_input = String::new();
            std::io::stdin().read_line(&mut enum_path_input)?;
            let enum_path = enum_path_input.trim();

            if !enum_path.is_empty() {
                println!("\nSelect video quality (1=480P, 2=720P, 3=1080P, or press Enter for highest VIP 2):");
                let mut quality_input = String::new();
                std::io::stdin().read_line(&mut quality_input)?;

                let quality = match quality_input.trim() {
                    "1" => VideoQuality::Quality480P,
                    "2" => VideoQuality::Quality720P,
                    "3" => VideoQuality::Quality1080P,
                    _ => VideoQuality::highest_for_vip_level(2), // Default to highest for VIP 2
                };

                println!(
                    "\nUsing quality: {:?} ({})",
                    quality,
                    quality.to_media_type()
                );
                println!("Getting m3u8...");
                wait_for_enter();

                match client.playlist().get_video_m3u8(enum_path, quality).await {
                    Ok(content) => {
                        println!("✓ Successfully got m3u8 content!");
                        println!("  Length: {} bytes", content.len());
                        println!(
                            "  Has #EXT-X-ENDLIST: {}",
                            content.contains("#EXT-X-ENDLIST")
                        );
                    }
                    Err(e) => {
                        println!("! Failed to get m3u8: {}", e);
                    }
                }
            }
        }
    }

    println!("\n=== All playlist tests completed! ===");

    Ok(())
}

/// Wait for user to press Enter before continuing
fn wait_for_enter() {
    println!("Press Enter to continue...");
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let _ = lines.next();
}
