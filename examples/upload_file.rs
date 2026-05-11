use baidu_netdisk_sdk::{BaiduNetDiskClient, CreateFileOptions, PrecreateOptions};
use log::info;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB
const MAX_CONCURRENCY: usize = 10; // Maximum concurrent chunk uploads

fn calculate_md5(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}

fn get_file_chunks<P: AsRef<Path>>(
    path: P,
) -> Result<(u64, Vec<(u32, Vec<u8>, String)>), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let total_size = metadata.len() as u64;

    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();
    let mut partseq = 0u32;

    loop {
        let mut buffer = Vec::with_capacity(CHUNK_SIZE);
        let bytes_read = reader
            .by_ref()
            .take(CHUNK_SIZE as u64)
            .read_to_end(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let chunk_md5 = calculate_md5(&buffer);
        chunks.push((partseq, buffer, chunk_md5));
        partseq += 1;
    }

    Ok((total_size, chunks))
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

    println!("=== Baidu NetDisk File Upload Test (Parallel) ===");
    println!("Local file: {}", local_file);
    println!("Remote path: {}", remote_path);
    println!("Max concurrency: {}", MAX_CONCURRENCY);
    println!();

    // Step 1: Read file and prepare chunks
    println!("Step 1: Reading file and calculating chunks...");
    let start_time = std::time::Instant::now();
    let (file_size, chunks) = get_file_chunks(local_file)?;
    let block_list: Vec<String> = chunks.iter().map(|(_, _, md5)| md5.clone()).collect();

    println!("File size: {} bytes", file_size);
    println!("Chunk count: {}", chunks.len());
    println!(
        "Block list (first 5): {:?}",
        &block_list[..5.min(block_list.len())]
    );
    if block_list.len() > 5 {
        println!("  ...");
    }
    println!("Read time: {:?}", start_time.elapsed());
    println!();

    // Step 2: Precreate
    println!("Step 2: Precreate upload...");
    let precreate_start = std::time::Instant::now();
    let precreate_options =
        PrecreateOptions::new(remote_path, file_size, block_list.clone()).rtype(1);

    let precreate_response = client.upload().precreate(&token, precreate_options).await?;
    println!("Precreate success!");
    println!("  Upload ID: {}", precreate_response.uploadid);
    println!("  Blocks to upload: {:?}", precreate_response.block_list);
    println!("  Precreate time: {:?}", precreate_start.elapsed());
    println!();

    // Step 3: Upload chunks in parallel
    println!(
        "Step 3: Uploading chunks in parallel (max_concurrency: {})...",
        MAX_CONCURRENCY
    );
    let upload_start = std::time::Instant::now();

    let chunks_to_upload: Vec<(u32, Vec<u8>)> = chunks
        .into_iter()
        .filter(|(i, _, _)| precreate_response.block_list.contains(i))
        .map(|(i, data, md5)| {
            println!("  Will upload chunk {} (MD5: {})", i, md5);
            (i, data)
        })
        .collect();

    println!(
        "  Starting parallel upload of {} chunks...",
        chunks_to_upload.len()
    );

    let chunk_results = client
        .upload()
        .upload_chunks_parallel(
            &token,
            remote_path,
            &precreate_response.uploadid,
            chunks_to_upload,
            MAX_CONCURRENCY,
        )
        .await?;

    println!("  All chunks uploaded!");
    println!("  Upload time: {:?}", upload_start.elapsed());
    println!();

    // Sort by partseq to maintain order for block_list
    let mut sorted_results = chunk_results;
    sorted_results.sort_by_key(|(i, _)| *i);
    let new_block_list: Vec<String> = sorted_results.into_iter().map(|(_, md5)| md5).collect();

    // Step 4: Create file
    println!("Step 4: Creating file...");
    let create_start = std::time::Instant::now();
    let create_options = CreateFileOptions::new(
        remote_path,
        file_size,
        new_block_list,
        &precreate_response.uploadid,
    )
    .rtype(1);

    let create_response = client.upload().create_file(&token, create_options).await?;
    println!("File created successfully!");
    println!("  FS ID: {}", create_response.fs_id);
    println!("  Server filename: {:?}", create_response.server_filename);
    println!("  Path: {}", create_response.path);
    println!("  Size: {} bytes", create_response.size);
    println!("  Category: {}", create_response.category);
    println!("  MD5: {}", create_response.md5.unwrap_or_default());
    println!("  Create time: {:?}", create_start.elapsed());
    println!();

    println!("=== Upload Completed ===");
    println!("Total time: {:?}", start_time.elapsed());

    Ok(())
}
