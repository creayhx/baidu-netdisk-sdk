use baidu_netdisk_sdk::BaiduNetDiskClient;
use log::info;
use std::path::Path;

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client = BaiduNetDiskClient::builder().build()?;
    info!("Client created successfully");

    let token = client.load_token_from_env()?;
    info!("Token loaded successfully");

    println!("=== Baidu NetDisk Download Comparison Test ===");
    println!();
    println!(
        "Enter the number of threads/concurrency to use (recommended: match your CPU core count)"
    );
    println!("Example: 4 for 4 cores, 8 for 8 cores, etc.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let thread_num: usize = input.trim().parse().unwrap_or(4);
    let concurrency = thread_num * 3;
    println!();
    println!("Thread count: {}", thread_num);
    println!("Concurrency: {}", concurrency);
    println!();

    println!("=== Method 1: Streaming (Futures/Concurrency) ===");
    println!("Enter the remote file path to download:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let remote_path_streaming = input.trim();

    if remote_path_streaming.is_empty() {
        eprintln!("Error: File path cannot be empty");
        return Ok(());
    }

    println!("\n--- Step 1: Get file info for Streaming ---");
    let file_info_streaming = client
        .file()
        .get_file_info(&token, remote_path_streaming)
        .await?;
    let file_size_streaming = file_info_streaming.size.unwrap_or(0);
    let fs_id_streaming = file_info_streaming.fs_id.ok_or("File has no fs_id")?;
    let file_meta_streaming = client.file().get_file_meta(&token, fs_id_streaming).await?;

    println!("File: {}", file_info_streaming.name);
    println!(
        "Size: {} bytes ({:.2} MB)",
        file_size_streaming,
        file_size_streaming as f64 / (1024.0 * 1024.0)
    );
    let total_chunks_streaming = (file_size_streaming + CHUNK_SIZE - 1) / CHUNK_SIZE;
    println!(
        "Chunks: {} ({} bytes each)",
        total_chunks_streaming, CHUNK_SIZE
    );
    println!();

    println!("Press Enter to start Streaming download...");
    std::io::stdin().read_line(&mut String::new())?;

    let streaming_save_path = format!("./download_streaming_{}", file_info_streaming.name);
    let start_streaming = std::time::Instant::now();

    client
        .download()
        .download_streaming_with_meta(
            &token,
            &file_meta_streaming,
            Path::new(&streaming_save_path),
            concurrency,
        )
        .await?;

    let duration_streaming = start_streaming.elapsed();
    let streaming_mb = file_size_streaming as f64 / (1024.0 * 1024.0);
    let streaming_sec = duration_streaming.as_secs_f64();
    let streaming_speed = streaming_mb / streaming_sec;

    println!();
    println!("=== Method 2: Parallel (Multi-thread) ===");
    println!("Enter the remote file path to download:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let remote_path_parallel = input.trim();

    if remote_path_parallel.is_empty() {
        eprintln!("Error: File path cannot be empty");
        return Ok(());
    }

    println!("\n--- Step 1: Get file info for Parallel ---");
    let file_info_parallel = client
        .file()
        .get_file_info(&token, remote_path_parallel)
        .await?;
    let file_size_parallel = file_info_parallel.size.unwrap_or(0);
    let fs_id_parallel = file_info_parallel.fs_id.ok_or("File has no fs_id")?;
    let file_meta_parallel = client.file().get_file_meta(&token, fs_id_parallel).await?;

    println!("File: {}", file_info_parallel.name);
    println!(
        "Size: {} bytes ({:.2} MB)",
        file_size_parallel,
        file_size_parallel as f64 / (1024.0 * 1024.0)
    );
    let total_chunks_parallel = (file_size_parallel + CHUNK_SIZE - 1) / CHUNK_SIZE;
    println!(
        "Chunks: {} ({} bytes each)",
        total_chunks_parallel, CHUNK_SIZE
    );
    println!();

    println!("Press Enter to start Parallel download...");
    std::io::stdin().read_line(&mut String::new())?;

    let parallel_save_path = format!("./download_parallel_{}", file_info_parallel.name);
    let start_parallel = std::time::Instant::now();

    client
        .download()
        .download_parallel_multi_threaded(
            &token,
            &file_meta_parallel,
            Path::new(&parallel_save_path),
            Some(thread_num),
        )
        .await?;

    let duration_parallel = start_parallel.elapsed();
    let parallel_mb = file_size_parallel as f64 / (1024.0 * 1024.0);
    let parallel_sec = duration_parallel.as_secs_f64();
    let parallel_speed = parallel_mb / parallel_sec;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                        DOWNLOAD COMPARISON RESULTS                         ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║  Threads: {}  │  Concurrency (Streaming): {}  │                  ║",
        thread_num, concurrency
    );
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  Method                      │ Time       │ Speed       │ File           ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║  Streaming (Futures)         │ {:>8.2}s  │ {:>8.2} MB/s │ {:.15} ║",
        streaming_sec, streaming_speed, file_info_streaming.name
    );
    println!(
        "║  Parallel (Multi-thread)     │ {:>8.2}s  │ {:>8.2} MB/s │ {:.15} ║",
        parallel_sec, parallel_speed, file_info_parallel.name
    );
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    let speedup = if streaming_sec < parallel_sec {
        parallel_sec / streaming_sec
    } else {
        streaming_sec / parallel_sec
    };
    let faster = if streaming_sec < parallel_sec {
        "Streaming"
    } else {
        "Parallel"
    };
    println!(
        "║  Faster: {} ({:.2}x)                                              ║",
        faster, speedup
    );
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  Note: Parallel mode works best with 6+ cores. On 4 cores, Streaming may  ║");
    println!("║        be 2x faster. Test with your actual core count for best results!   ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");

    println!();
    println!("Output files:");
    println!("  Parallel:   {}", parallel_save_path);
    println!("  Streaming:  {}", streaming_save_path);

    Ok(())
}
