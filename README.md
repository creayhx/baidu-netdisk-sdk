# Baidu NetDisk Rust SDK

[English](README.md) | [中文](README_zh.md)

A Rust SDK for Baidu NetDisk Open Platform API, providing file management, upload/download and other functionalities.

## Features

- **API Coverage**: File management, upload/download, media processing
- **High Performance**: Supports parallel upload, streaming download, and multi-threaded download
- **Elegant Error Handling**: Layered error types with Chinese error descriptions
- **Thread Safety**: Uses `RwLock` for concurrent safety
- **Flexible Configuration**: Builder pattern for easy client configuration
- **Async First**: Built on `tokio` async runtime
- **Convenient Shortcut APIs**: Client automatically encapsulates submodule methods and manages tokens internally

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
baidu-netdisk-sdk = "0.1.3"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

> **Note**: The `BaiduNetDiskClient` encapsulates all submodule methods and automatically manages tokens internally. Most operations can be called directly on the client without needing to access submodules or pass tokens explicitly.

### 1. Create Client

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")
    .app_secret("your_app_secret")
    .build()?;
```

### 2. Authorization

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

// Get device code for authorization
let device_code = client.authorize().get_device_code().await?;
println!("Please visit: {}", device_code.verification_url);
println!("Enter user code: {}", device_code.user_code);

// Poll for access token
let token = loop {
    match client.authorize().request_access_token(&device_code).await? {
        Some(t) => break t,
        None => tokio::time::sleep(std::time::Duration::from_secs(5)).await,
    }
};
```

### 3. File Operations

```rust
// List directory - using shortcut API (no explicit token required)
let files = client.list_directory("/").await?;

// Search files - using shortcut API
let (results, has_more) = client.search_files("documents", "/").await?;

// Upload file - using shortcut API
client.upload_file("local/path.txt", "/remote/path.txt").await?;

// Download file - using shortcut API
client.download_single("/remote/file.txt", "./local/file.txt").await?;
```

### 4. TokenScopedClient for Multi-User Scenarios

For multi-user scenarios where you need to use multiple tokens concurrently, you can create a `TokenScopedClient`:

```rust
// Create a scoped client bound to a specific token
let scoped_client = client.with_token(token);

// Use the scoped client - token is automatically used
let files = scoped_client.list_directory("/").await?;
let quota = scoped_client.get_quota().await?;

// Each scoped client has its own isolated token context
// Multiple scoped clients can be used concurrently without conflicts
```

**Key Benefits of TokenScopedClient:**
- **Thread-safe**: Each scoped client has its own token context
- **Isolation**: Changes to one scoped client don't affect others
- **Convenience**: No need to pass token explicitly for each call
- **Cacheable**: You can cache scoped clients per user for better performance

## Configuration

### Environment Variables

#### Silently Read During Builder Initialization

These variables are automatically read when `ClientBuilder::default()` is called (i.e., when `BaiduNetDiskClient::builder()` is invoked). Values from environment variables are used as defaults but can be overridden by explicit builder calls.

| Variable | Description |
|----------|-------------|
| `BD_NETDISK_APP_ID` | Application ID (optional) |
| `BD_NETDISK_APP_KEY` | Application Key |
| `BD_NETDISK_SECRET_KEY` | Application Secret |
| `BD_NETDISK_APP_NAME` | Application Name (optional, for identifying multiple apps) |

**Note**: If you explicitly set these via the builder (e.g., `.app_key("...")`), they will override any environment variable values.

#### Explicitly Loaded (Manual Reading)

These variables are NOT automatically read during builder initialization. You must call `load_token_from_env()` explicitly to load them.

| Variable | Required | Description |
|----------|--------------------------------------|-------------|
| `BD_NETDISK_ACCESS_TOKEN` | Yes | Access Token |
| `BD_NETDISK_REFRESH_TOKEN` | Yes | Refresh Token |
| `BD_NETDISK_EXPIRES_IN` | Yes | Token expiration in seconds |
| `BD_NETDISK_SCOPE` | No | Permission scope (default: "basic netdisk") |
| `BD_NETDISK_SESSION_KEY` | No | Session key |
| `BD_NETDISK_SESSION_SECRET` | No | Session secret |
| `BD_NETDISK_ACQUIRED_AT` | No | Token acquisition timestamp (for testing) |

### Avoiding Duplicate Configuration

If you want to avoid environment variable interference, simply set all required values explicitly via the builder:

```rust
let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")           // Explicitly set, overrides BD_NETDISK_APP_KEY
    .app_secret("your_app_secret")     // Explicitly set, overrides BD_NETDISK_SECRET_KEY
    .build()?;

// Tokens must still be loaded explicitly or set manually
client.load_token_from_env()?;  // Loads token variables
// OR
client.set_access_token(manual_token)?;  // Set manually
```

### Client Builder Options

```rust
let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")           // Application Key
    .app_secret("your_app_secret")       // Application Secret
    .app_name("My App")                  // Application Name (optional)
    .timeout(Duration::from_secs(30))    // Request timeout
    .auto_refresh(true)                  // Auto refresh token
    .refresh_ahead_seconds(86400)        // Refresh 24h before expiration
    .max_retries(3)                      // Max retry attempts
    .build()?;
```

## API Modules

### File Management (`client.file()`)

Core file operations:
- `list_directory()` - List directory files
- `list_all()` - List all files recursively
- `get_file_info()` - Get file information by path
- `get_file_meta()` - Get file metadata (**with download link dlink**) by fs_id
- `search_files()` - Search files by keyword
- `semantic_search()` - Semantic search
- `create_folder()` - Create directory
- `rename()` - Rename file/folder
- `move_file()` - Move file/folder
- `copy_file()` - Copy file/folder
- `delete()` - Delete file/folder

### Download (`client.download()`)

Download methods:
- `get_dlink_from_path()` - Get download link by file path
- `get_dlink_from_fsid()` - Get download link by file fs_id
- `auto_download()` - Auto-select best method based on file size
- `auto_download_by_fsid()` - Auto-select by fs_id
- `download_single()` - Single-threaded download (by path)
- `download_single_by_fsid()` - Single-threaded download (by fs_id)
- `download_single_with_meta()` - Single-threaded download (with FileMeta)
- `download_parallel()` - Multi-thread parallel download (by path)
- `download_parallel_by_fsid()` - Multi-thread parallel download (by fs_id)
- `download_parallel_multi_threaded()` - Multi-thread parallel download (with FileMeta)
- `download_streaming()` - Async concurrency download (by path)
- `download_streaming_by_fsid()` - Async concurrency download (by fs_id)
- `download_streaming_with_meta()` - Streaming download (with FileMeta)

### Upload (`client.upload()`)

The SDK provides multiple upload methods to handle different scenarios:

#### Upload Methods Comparison

| Method | Data Source | Memory Usage | Streaming | Best For |
|--------|-------------|--------------|-----------|----------|
| [`upload_file()`](#1-upload-file-from-path) | File path | ~80MB | ✅ | Most common scenarios |
| [`upload_reader()`](#2-upload-from-reader) | Reader + size | ~80MB | ✅ | Custom readers, wrapped streams |
| [`upload_bytes()`](#3-upload-bytes-from-memory) | `&[u8]` slice | Full data | ❌ | Small data in memory |

#### Key Features

- **Resumable Upload**: Automatically detects partially uploaded chunks and skips them
- **Parallel Upload**: Uploads multiple chunks concurrently (default: 10 parallel)
- **Memory Optimized**: For large files, memory is bounded by batch size (~80MB default)
- **Automatic Chunking**: Files are automatically split into 4MB chunks

#### 1. Upload File from Path

The simplest way to upload a file:

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let client = BaiduNetDiskClient::builder().build()?;
client.load_token_from_env()?;

// Simple upload - using shortcut API (no explicit token required)
let response = client.upload_file("local_file.txt", "/remote/file.txt").await?;

println!("Uploaded: {} ({} bytes)", response.path, response.size);
```

With custom options:

```rust
use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::SimpleUploadOptions};

let options = SimpleUploadOptions::default()
    .chunk_size(8 * 1024 * 1024)  // 8MB chunks
    .max_concurrency(20);         // 20 parallel uploads

let response = client.upload_file_with_options("video.mp4", "/remote/video.mp4", options).await?;
```

#### 2. Upload from Reader

For streaming upload with custom readers (requires `Read + Seek`):

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;
use std::io::BufReader;

let file = std::fs::File::open("local_file.txt")?;
let metadata = file.metadata()?;
let file_size = metadata.len();

let mut reader = BufReader::new(file);

let response = client.upload()
    .upload_reader(&token, &mut reader, file_size, "/remote/file.txt")
    .await?;
```

#### 3. Upload Bytes from Memory

For data already in memory:

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let data = b"Hello, World!";
// Using shortcut API (no explicit token required)
let response = client.upload_bytes(data, "/remote/hello.txt").await?;

println!("Uploaded: {} bytes", response.size);
```

#### How Resumable Upload Works

1. **First pass**: Read file → calculate MD5 for each chunk
2. **Precreate**: Call API → get uploadid and list of existing chunks
3. **Second pass**: Read file again → only upload missing chunks
4. **Create**: Merge chunks into final file

This means if an upload is interrupted, restarting will only upload the missing chunks.

### Authorization (`client.authorize()`)

- `get_device_code()` - Get authorization device code
- `request_access_token()` - Poll for access token

### User & Quota

- `client.user().info()` - Get user info
- `client.quota().info()` - Get storage quota

### Playlist (`client.playlist()`)

Playlist and media functionality:

**Playlist Operations:**
- `get_playlist_list()` - List playlists
- `get_playlist_file_list()` - List files in playlist

**Media Playback:**
- `get_media_play_info()` - Get media playback info (supports path or fs_id)
- `get_media_m3u8_content()` - Get raw m3u8 content by path

**Convenience Methods (Quality Enums):**
- `get_video_m3u8()` - Get video m3u8 with VideoQuality
- `get_video_m3u8_highest()` - Get video m3u8 with highest quality for VIP level
- `get_audio_m3u8()` - Get audio m3u8 with AudioQuality
- `get_audio_m3u8_default()` - Get audio m3u8 with default quality (128K)

**Transcoding Status Check:**
- `fetch_m3u8()` - Fetch m3u8 content from URL
- `is_media_fully_transcoded()` - Check if media is fully transcoded (#EXT-X-ENDLIST)

**Quality Enums:**
- `VideoQuality` - Video quality levels (480P, 720P, 1080P)
- `AudioQuality` - Audio quality levels (MP3 128K)
- Quality methods: `to_media_type()`, `highest_for_vip_level()`, `available_for_vip_level()`
- Automatic quality selection based on VIP level

## Error Handling

```rust
use baidu_netdisk_sdk::{NetDiskError, NetDiskResult};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(e) => {
        eprintln!("Error: {}", e);
        if e.is_auth_error() {
            // Handle auth error - re-authenticate
        } else if e.is_not_found_error() {
            // Handle not found
        }
    }
}
```

## Token Management

```rust
// Set token manually
let token = AccessToken::new(
    "access_token_string".to_string(),
    "refresh_token_string".to_string(),
    2592000,  // expires_in
    "basic netdisk".to_string(),
);
client.set_access_token(token)?;

// Load token from environment
let token = client.load_token_from_env()?;

// Validate token status
match client.validate_token() {
    Ok(TokenStatus::Valid) => println!("Token is valid"),
    Ok(TokenStatus::ExpiringSoon) => println!("Token expiring soon"),
    Ok(TokenStatus::Expired) => println!("Token expired"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

Run examples:

```bash
# Authorization flow
cargo run --example auth_flow

# File operations
cargo run --example file

# Search
cargo run --example search

# Upload
cargo run --example upload_file
cargo run --example upload_bytes
cargo run --example upload_reader
cargo run --example upload_file_options

# Download
cargo run --example download

# Download comparison
cargo run --example download_compare

# Token test
cargo run --example token_test

# User info
cargo run --example user_info

# Quota info
cargo run --example quota

# Playlist
cargo run --example playlist
```

## Download Strategy Guide

This SDK provides multiple download strategies for different scenarios:

### Concurrency vs Parallelism

**Concurrency** (`download_streaming`):
- Uses async tasks on a single thread (or thread pool)
- Efficient for many small files or when network is the bottleneck
- Lower memory overhead
- Ideal for: Downloading multiple small files, limited memory environments

**Parallelism** (`download_parallel`):
- Uses true multi-threading with dedicated OS threads
- Higher throughput for large files (maximizes network bandwidth)
- Higher memory usage (each thread has its own stack)
- Ideal for: Large files (>100MB), maximum speed requirements

### When to Use Which

| Scenario | Recommendation |
|----------|----------------|
| Small files (<10MB) | `auto_download()` or concurrent streaming |
| Medium files (10-100MB) | `auto_download()` will choose best |
| Large files (>100MB) | Parallel multi-threaded |
| Multiple files | Concurrent streaming |
| Memory constrained | Single-thread or concurrent streaming |
| Maximum speed | Parallel multi-threaded |

### Quick Reference

```rust
// Auto-select based on file size (recommended for most cases)
// - < 10MB: single-threaded
// - > 10MB: futures concurrent streaming (good performance regardless of CPU cores)
// Note: For maximum speed, use `download_parallel` manually
// Using shortcut API (no explicit token required)
client.auto_download("/remote/file.zip", "./local/file.zip").await?;

// For maximum speed with large files (6+ cores recommended)
client.download_parallel("/remote/large.iso", "./local/large.iso", Some(8)).await?;

// For many small files or limited cores (<= 4)
client.download_streaming("/remote/small.txt", "./local/small.txt", 4).await?;
```

### Not Sure Which to Use? Run the Comparison Test!

If you're unsure about the best download method for your hardware, run the comparison test:

```bash
cargo run --example download_compare
```

The test will ask you to enter your CPU core count (e.g., 4, 8, 12) and then:
1. Download using **Streaming (Futures/Concurrency)**
2. Download using **Parallel (Multi-thread)**
3. Show a side-by-side speed comparison

Use the results to decide which method works best for your specific hardware!

**Key Findings from Tests:**
- 4 cores: Futures (Streaming) is often 1.5-2x faster than Parallel
- 6-8 cores: Both methods perform similarly
- 8+ cores: Parallel pulls ahead slightly due to better multi-core utilization

## Performance Tips

1. **Large File Upload**: Use `upload_file()` which automatically chunks and parallelizes
2. **Large File Download**: Use `download_parallel_multi_threaded()` for maximum speed
3. **Token Refresh**: Set `refresh_ahead_seconds` to a value that matches your usage pattern

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
