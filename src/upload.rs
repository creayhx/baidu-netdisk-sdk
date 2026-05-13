//! File upload functionality for Baidu NetDisk
//!
//! This module provides comprehensive file upload capabilities with support for:
//! - Simple file upload via path
//! - Reader-based upload for streaming data
//! - Byte array upload for in-memory data
//! - Resumable upload (automatic detection of partially uploaded chunks)
//! - Parallel chunk upload for better performance
//!
//! # Architecture
//!
//! The upload process consists of 3 steps:
//! 1. **Precreate** - Initiate upload session, get uploadid, and check existing chunks
//! 2. **Upload Chunks** - Upload missing chunks in parallel
//! 3. **Create File** - Merge chunks into final file on server
//!
//! # Upload Methods Comparison
//!
//! | Method | Data Source | Memory Usage | Streaming | Use Case |
//! |--------|-------------|--------------|-----------|----------|
//! | [`UploadClient::upload_file`] | File path | ~80MB | ✅ | Most common, upload from disk |
//! | [`UploadClient::upload_reader`] | Reader + size | ~80MB | ✅ | Custom readers, wrapped streams |
//! | [`UploadClient::upload_bytes`] | `&[u8]` slice | Full data | ❌ | Small data already in memory |
//!
//! # Memory Optimization
//!
//! For large files, memory usage is bounded by:
//! - Batch size: `max_concurrency * 2` chunks (default: 20 * 4MB = 80MB)
//! - Only missing chunks are copied to memory
//! - Existing chunks are skipped automatically (resumable upload)
//!
//! # Example
//!
//! ```no_run
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder().build()?;
//! client.load_token_from_env()?;
//!
//! // Simple file upload
//! let response = client.upload()
//!     .upload_file("test.txt", "/remote/test.txt")
//!     .await?;
//!
//! println!("Uploaded: {} ({} bytes)", response.path, response.size);
//! # Ok(())
//! # }
//! ```
//!
//! # Low-level API
//!
//! For advanced use cases, you can use the step-by-step API:
//! - [`UploadClient::precreate`] - Start upload session
//! - [`UploadClient::upload_chunks_parallel`] - Upload specific chunks
//! - [`UploadClient::create_file`] - Complete the upload
use crate::client::TokenGetter;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;
use futures::stream::{self, StreamExt};
use log::debug;
use serde::Deserialize;
use std::sync::Arc;

/// Upload client for Baidu NetDisk
#[derive(Debug, Clone)]
pub struct UploadClient {
    http_client: HttpClient,
    token_getter: Arc<dyn TokenGetter>,
}

impl UploadClient {
    /// Create a new UploadClient instance
    ///
    /// Usually you don't need to call this directly - use BaiduNetDiskClient::upload() instead.
    pub fn new(http_client: HttpClient, token_getter: Arc<dyn TokenGetter>) -> Self {
        Self {
            http_client,
            token_getter,
        }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Precreate an upload session
    ///
    /// Initiates an upload session and checks which chunks (if any) already exist on the server.
    /// This is the first step of the multi-step upload process.
    pub async fn precreate(&self, options: PrecreateOptions) -> NetDiskResult<PrecreateResponse> {
        let token = self.token_getter.get_token().await?;
        let block_list_json =
            serde_json::to_string(&options.block_list).map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to serialize block_list: {}", e),
            })?;

        let params = vec![
            ("method", "precreate"),
            ("access_token", token.access_token.as_str()),
        ];

        let size_str = options.size.to_string();
        let isdir_str = options.isdir.to_string();
        let rtype_str = options.rtype.to_string();

        let form_data = vec![
            ("path", options.path.as_str()),
            ("size", size_str.as_str()),
            ("isdir", isdir_str.as_str()),
            ("block_list", block_list_json.as_str()),
            ("autoinit", "1"),
            ("rtype", rtype_str.as_str()),
        ];

        debug!(
            "Precreate upload: path={}, size={}, isdir={}, block_list={:?}",
            options.path, options.size, options.isdir, options.block_list
        );

        let response: PrecreateResponse = self
            .http_client
            .post_form("/rest/2.0/xpan/file", Some(&form_data), Some(&params))
            .await?;

        if response.errno != 0 {
            let error_msg = get_error_message(response.errno);
            return Err(NetDiskError::api_error(response.errno, &error_msg));
        }

        debug!(
            "Precreate success: uploadid={}, block_list={:?}",
            response.uploadid, response.block_list
        );

        Ok(response)
    }
}

/// Options for precreate upload
#[derive(Debug, Clone, Default)]
pub struct PrecreateOptions {
    /// Remote file path on Baidu NetDisk
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Is directory (0 for file, 1 for directory)
    pub isdir: i32,
    /// List of block MD5s (each block is typically 4MB)
    pub block_list: Vec<String>,
    /// Conflict resolution type (1=overwrite, 2=rename, 3=new copy)
    pub rtype: i32,
    /// Optional uploadid for resuming interrupted uploads
    pub uploadid: Option<String>,
    /// Optional content MD5 for entire file
    pub content_md5: Option<String>,
    /// Optional slice MD5
    pub slice_md5: Option<String>,
    /// Optional local creation time (timestamp)
    pub local_ctime: Option<u64>,
    /// Optional local modification time (timestamp)
    pub local_mtime: Option<u64>,
}

impl PrecreateOptions {
    /// Create new PrecreateOptions with basic required fields
    pub fn new(path: &str, size: u64, block_list: Vec<String>) -> Self {
        Self {
            path: path.to_string(),
            size,
            isdir: 0,
            block_list,
            rtype: 1,
            uploadid: None,
            content_md5: None,
            slice_md5: None,
            local_ctime: None,
            local_mtime: None,
        }
    }

    /// Set whether this is a directory
    pub fn isdir(mut self, isdir: bool) -> Self {
        self.isdir = if isdir { 1 } else { 0 };
        self
    }

    /// Set conflict resolution type (1=overwrite, 2=rename, 3=new copy)
    pub fn rtype(mut self, rtype: i32) -> Self {
        self.rtype = rtype;
        self
    }

    /// Set uploadid for resuming an interrupted upload
    pub fn uploadid(mut self, uploadid: &str) -> Self {
        self.uploadid = Some(uploadid.to_string());
        self
    }

    /// Set content MD5 for the entire file
    pub fn content_md5(mut self, md5: &str) -> Self {
        self.content_md5 = Some(md5.to_string());
        self
    }

    /// Set slice MD5
    pub fn slice_md5(mut self, md5: &str) -> Self {
        self.slice_md5 = Some(md5.to_string());
        self
    }

    /// Set local creation time (timestamp)
    pub fn local_ctime(mut self, ctime: u64) -> Self {
        self.local_ctime = Some(ctime);
        self
    }

    /// Set local modification time (timestamp)
    pub fn local_mtime(mut self, mtime: u64) -> Self {
        self.local_mtime = Some(mtime);
        self
    }
}

/// Precreate response
#[derive(Debug, Deserialize)]
pub struct PrecreateResponse {
    /// Error code (0 indicates success)
    pub errno: i32,
    /// File path
    #[serde(default)]
    pub path: Option<String>,
    /// Upload session ID (use this for subsequent steps)
    pub uploadid: String,
    /// Return type
    #[serde(rename = "return_type")]
    pub return_type: i32,
    /// List of missing block indices that need to be uploaded
    #[serde(rename = "block_list")]
    pub block_list: Vec<u32>,
}

/// Server info for locate upload response
#[derive(Debug, Deserialize)]
pub struct LocateUploadServer {
    /// Server URL
    pub server: String,
}

/// Locate upload response
#[derive(Debug, Deserialize)]
pub struct LocateUploadResponse {
    /// Backup servers
    #[serde(default)]
    pub bak_server: Vec<String>,
    /// Backup servers list
    #[serde(default)]
    pub bak_servers: Vec<LocateUploadServer>,
    /// Client IP address
    pub client_ip: String,
    /// Error code (0 indicates success)
    pub error_code: i32,
    /// Error message
    pub error_msg: String,
    /// Expiration time in seconds
    pub expire: i32,
    /// Host name
    pub host: String,
    /// New number
    #[serde(default)]
    pub newno: String,
    /// QUIC servers
    #[serde(default)]
    pub quic_server: Vec<String>,
    /// QUIC servers list
    #[serde(default)]
    pub quic_servers: Vec<LocateUploadServer>,
    /// Request ID
    pub request_id: u64,
    /// Servers list
    #[serde(default)]
    pub server: Vec<String>,
    /// Server timestamp
    pub server_time: u64,
    /// Servers list (detailed)
    pub servers: Vec<LocateUploadServer>,
    /// SL value
    pub sl: i32,
}

impl LocateUploadResponse {
    /// Get all HTTPS servers from the response
    ///
    /// Returns a vector of HTTPS server URLs that can be used for uploading chunks.
    pub fn get_https_servers(&self) -> Vec<String> {
        self.servers
            .iter()
            .filter(|s| s.server.starts_with("https://"))
            .map(|s| s.server.clone())
            .collect()
    }

    /// Get the first HTTPS server from the available servers
    ///
    /// According to Baidu NetDisk API documentation, the servers are sorted by proximity and speed.
    /// The first server is recommended as the default choice for optimal upload performance.
    ///
    /// Returns `None` if no HTTPS servers are available.
    pub fn get_first_https_server(&self) -> Option<String> {
        let https_servers = self.get_https_servers();
        if https_servers.is_empty() {
            None
        } else {
            Some(https_servers[0].clone())
        }
    }
}

impl UploadClient {
    /// Locate upload domain
    ///
    /// Gets the upload domain before uploading chunks.
    /// This is required before uploading file data.
    ///
    /// According to Baidu NetDisk API documentation, the servers are sorted by proximity and speed.
    /// The first server is recommended as the default choice for optimal upload performance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let response = client.upload()
    ///     .locate_upload("/apps/appName/filename.jpg", "P1-MTAuMjI4LjQzLjMxOjE1OTU4NTg==")
    ///     .await?;
    ///
    /// // Get the first HTTPS server (recommended by Baidu for optimal performance)
    /// if let Some(server) = response.get_first_https_server() {
    ///     println!("Upload server: {}", server);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn locate_upload(
        &self,
        path: &str,
        uploadid: &str,
    ) -> NetDiskResult<LocateUploadResponse> {
        let token = self.token_getter.get_token().await?;

        let url = format!(
            "https://d.pcs.baidu.com/rest/2.0/pcs/file?method=locateupload&appid=250528&access_token={}&path={}&uploadid={}&upload_version=2.0",
            urlencoding::encode(&token.access_token),
            urlencoding::encode(path),
            urlencoding::encode(uploadid)
        );

        debug!("Locate upload: path={}, uploadid={}", path, uploadid);

        let response: LocateUploadResponse = self.http_client.get(&url, None).await?;

        if response.error_code != 0 {
            return Err(NetDiskError::api_error(
                response.error_code,
                &response.error_msg,
            ));
        }

        debug!(
            "Locate upload success: host={}, servers={}",
            response.host,
            response.servers.len()
        );

        Ok(response)
    }

    /// Upload a single chunk
    ///
    /// If `server_url` is not provided, the default server `https://c3.pcs.baidu.com` will be used.
    pub async fn upload_chunk(
        &self,
        options: UploadChunkOptions,
        server_url: Option<&str>,
    ) -> NetDiskResult<UploadChunkResponse> {
        let token = self.token_getter.get_token().await?;

        let server = server_url.unwrap_or("https://c3.pcs.baidu.com");
        let url = format!(
            "{}/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq={}",
            server,
            urlencoding::encode(&token.access_token),
            urlencoding::encode(&options.path),
            urlencoding::encode(&options.uploadid),
            options.partseq
        );

        debug!(
            "Upload chunk: path={}, uploadid={}, partseq={}, data_size={}, server={}",
            options.path,
            options.uploadid,
            options.partseq,
            options.data.len(),
            server
        );

        let response: UploadChunkResponse = self
            .http_client
            .post_multipart(
                &url,
                "file".to_string(),
                "chunk.dat".to_string(),
                options.data,
            )
            .await?;

        Ok(response)
    }

    /// Upload multiple chunks in parallel
    ///
    /// If `server_url` is not provided, the default server `https://c3.pcs.baidu.com` will be used.
    pub async fn upload_chunks_parallel(
        &self,
        remote_path: &str,
        uploadid: &str,
        chunks: Vec<(u32, Vec<u8>)>,
        max_concurrency: usize,
        server_url: Option<&str>,
    ) -> NetDiskResult<Vec<(u32, String)>> {
        let token = self.token_getter.get_token().await?;
        let server = server_url.unwrap_or("https://c3.pcs.baidu.com").to_string();

        debug!(
            "Uploading {} chunks in parallel (max_concurrency: {}, server: {})",
            chunks.len(),
            max_concurrency,
            server
        );

        let access_token_str = token.access_token;
        let remote_path_str = remote_path.to_string();
        let uploadid_str = uploadid.to_string();
        let http_client = self.http_client.clone();

        let mut stream = stream::iter(chunks)
            .map(move |(partseq, data)| {
                let path = remote_path_str.clone();
                let uid = access_token_str.clone();
                let upid = uploadid_str.clone();
                let client = http_client.clone();
                let server_clone = server.clone();

                async move {
                    let url = format!(
                        "{}/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq={}",
                        server_clone,
                        urlencoding::encode(&uid),
                        urlencoding::encode(&path),
                        urlencoding::encode(&upid),
                        partseq
                    );

                    debug!("Uploading chunk {} ({} bytes)", partseq, data.len());

                    let response: UploadChunkResponse = client
                        .post_multipart(&url, "file".to_string(), "chunk.dat".to_string(), data)
                        .await?;

                    Ok((partseq, response.md5))
                }
            })
            .buffer_unordered(max_concurrency);

        let mut chunk_md5s = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok((partseq, md5)) => {
                    chunk_md5s.push((partseq, md5));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!("All chunks uploaded successfully");
        Ok(chunk_md5s)
    }

    /// Create the final file on the server
    ///
    /// This is the final step of the upload process, which merges all uploaded chunks into a single file.
    pub async fn create_file(
        &self,
        options: CreateFileOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let token = self.token_getter.get_token().await?;
        let block_list_json =
            serde_json::to_string(&options.block_list).map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to serialize block_list: {}", e),
            })?;

        let params = vec![
            ("method", "create"),
            ("access_token", token.access_token.as_str()),
        ];

        let size_str = options.size.to_string();
        let isdir_str = options.isdir.to_string();
        let rtype_str = options.rtype.to_string();

        let mut form_data = vec![
            ("path", options.path.as_str()),
            ("size", size_str.as_str()),
            ("isdir", isdir_str.as_str()),
            ("block_list", block_list_json.as_str()),
            ("uploadid", options.uploadid.as_str()),
            ("rtype", rtype_str.as_str()),
        ];

        let ctime_str = options.local_ctime.map(|t| t.to_string());
        let mtime_str = options.local_mtime.map(|t| t.to_string());

        if let Some(ref ctime) = ctime_str {
            form_data.push(("local_ctime", ctime.as_str()));
        }
        if let Some(ref mtime) = mtime_str {
            form_data.push(("local_mtime", mtime.as_str()));
        }

        debug!(
            "Create file: path={}, size={}, isdir={}, uploadid={}",
            options.path, options.size, options.isdir, options.uploadid
        );

        let response: CreateFileResponse = self
            .http_client
            .post_form("/rest/2.0/xpan/file", Some(&form_data), Some(&params))
            .await?;

        if response.errno != 0 {
            let error_msg = get_create_error_message(response.errno);
            return Err(NetDiskError::api_error(response.errno, &error_msg));
        }

        Ok(response)
    }
}

/// Options for uploading a single chunk
#[derive(Debug, Clone)]
pub struct UploadChunkOptions {
    /// Remote file path on Baidu NetDisk
    pub path: String,
    /// Upload session ID from precreate
    pub uploadid: String,
    /// Chunk sequence number (starting from 0)
    pub partseq: u32,
    /// Chunk data bytes
    pub data: Vec<u8>,
}

impl UploadChunkOptions {
    /// Create new UploadChunkOptions
    pub fn new(path: &str, uploadid: &str, partseq: u32, data: Vec<u8>) -> Self {
        Self {
            path: path.to_string(),
            uploadid: uploadid.to_string(),
            partseq,
            data,
        }
    }
}

/// Chunk upload response
#[derive(Debug, Deserialize)]
pub struct UploadChunkResponse {
    /// MD5 hash of the uploaded chunk
    pub md5: String,
}

/// Options for creating final file on server
#[derive(Debug, Clone, Default)]
pub struct CreateFileOptions {
    /// Remote file path on Baidu NetDisk
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Is directory (0 for file, 1 for directory)
    pub isdir: i32,
    /// List of block MD5s
    pub block_list: Vec<String>,
    /// Upload session ID from precreate
    pub uploadid: String,
    /// Conflict resolution type (1=overwrite, 2=rename, 3=new copy)
    pub rtype: i32,
    /// Optional local creation time (timestamp)
    pub local_ctime: Option<u64>,
    /// Optional local modification time (timestamp)
    pub local_mtime: Option<u64>,
}

impl CreateFileOptions {
    /// Create new CreateFileOptions with basic required fields
    pub fn new(path: &str, size: u64, block_list: Vec<String>, uploadid: &str) -> Self {
        Self {
            path: path.to_string(),
            size,
            isdir: 0,
            block_list,
            uploadid: uploadid.to_string(),
            rtype: 1,
            local_ctime: None,
            local_mtime: None,
        }
    }

    /// Set whether this is a directory
    pub fn isdir(mut self, isdir: bool) -> Self {
        self.isdir = if isdir { 1 } else { 0 };
        self
    }

    /// Set conflict resolution type (1=overwrite, 2=rename, 3=new copy)
    pub fn rtype(mut self, rtype: i32) -> Self {
        self.rtype = rtype;
        self
    }

    /// Set local creation time (timestamp)
    pub fn local_ctime(mut self, ctime: u64) -> Self {
        self.local_ctime = Some(ctime);
        self
    }

    /// Set local modification time (timestamp)
    pub fn local_mtime(mut self, mtime: u64) -> Self {
        self.local_mtime = Some(mtime);
        self
    }
}

/// Create file response
#[derive(Debug, Deserialize)]
pub struct CreateFileResponse {
    /// Error code (0 indicates success)
    pub errno: i32,
    /// File server ID
    #[serde(rename = "fs_id")]
    pub fs_id: u64,
    /// File MD5
    pub md5: Option<String>,
    /// Server filename
    #[serde(rename = "server_filename")]
    #[serde(default)]
    pub server_filename: Option<String>,
    /// File category
    pub category: i32,
    /// File path
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Creation time (timestamp)
    pub ctime: u64,
    /// Modification time (timestamp)
    pub mtime: u64,
    /// Is directory (0 for file, 1 for directory)
    pub isdir: i32,
    /// File name
    #[serde(default)]
    pub name: Option<String>,
    /// From type
    #[serde(rename = "from_type")]
    #[serde(default)]
    pub from_type: Option<i32>,
}

fn get_create_error_message(errno: i32) -> String {
    match errno {
        -7 => "File or directory name error or access denied".to_string(),
        -8 => "File or directory already exists".to_string(),
        -10 => "Cloud storage capacity full".to_string(),
        10 => "Failed to create file".to_string(),
        31190 => "File not found".to_string(),
        31355 => "Invalid parameter".to_string(),
        31365 => "Total file size limit exceeded".to_string(),
        _ => format!("Unknown error: {}", errno),
    }
}

fn get_error_message(errno: i32) -> String {
    match errno {
        -7 => "File or directory name error or access denied".to_string(),
        -10 => "Insufficient capacity".to_string(),
        _ => format!("Unknown error: {}", errno),
    }
}

const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024;
const DEFAULT_MAX_CONCURRENCY: usize = 10;

#[derive(Debug, Clone)]
/// Options for simple upload methods
///
/// Use the builder pattern to customize upload behavior:
///
/// # Example
///
/// ```
/// use baidu_netdisk_sdk::upload::SimpleUploadOptions;
///
/// let options = SimpleUploadOptions::default()
///     .chunk_size(8 * 1024 * 1024)  // 8MB chunks
///     .max_concurrency(20)         // 20 parallel uploads
///     .r#type(1);                  // file type
/// ```
///
/// # Default Values
///
/// - `chunk_size`: 4MB (4194304 bytes)
/// - `max_concurrency`: 10
/// - `r#type`: 1
pub struct SimpleUploadOptions {
    /// Size of each chunk in bytes (default: 4MB)
    pub chunk_size: usize,
    /// Maximum number of parallel chunk uploads (default: 10)
    pub max_concurrency: usize,
    /// File type hint (default: 1)
    pub r#type: i32,
}

impl Default for SimpleUploadOptions {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            r#type: 1,
        }
    }
}

impl SimpleUploadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn max_concurrency(mut self, concurrency: usize) -> Self {
        self.max_concurrency = concurrency;
        self
    }

    pub fn r#type(mut self, r#type: i32) -> Self {
        self.r#type = r#type;
        self
    }
}

impl UploadClient {
    /// Upload a file from local path (simple API)
    ///
    /// This is the simplest way to upload a file. It handles everything automatically:
    /// - Opens and reads the file
    /// - Calculates MD5 for each chunk
    /// - Uploads missing chunks in parallel
    /// - Creates the final file on the server
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let response = client.upload()
    ///     .upload_file("test.txt", "/remote/test.txt")
    ///     .await?;
    ///
    /// println!("Uploaded: {} ({} bytes)", response.path, response.size);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`UploadClient::upload_file_with_options`] for custom chunk size and concurrency
    /// - [`UploadClient::upload_reader`] for streaming upload with custom readers
    /// - [`UploadClient::upload_bytes`] for uploading data already in memory
    pub async fn upload_file<P: AsRef<std::path::Path>>(
        &self,
        local_path: P,
        remote_path: &str,
    ) -> NetDiskResult<CreateFileResponse> {
        self.upload_file_with_options(local_path, remote_path, SimpleUploadOptions::default())
            .await
    }

    /// Upload a file from local path with custom options
    ///
    /// Use this method to customize upload behavior:
    /// - `chunk_size`: Size of each chunk (default: 4MB)
    /// - `max_concurrency`: Maximum parallel uploads (default: 10)
    /// - `r#type`: File type hint (default: 1)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::SimpleUploadOptions};
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let options = SimpleUploadOptions::default()
    ///     .chunk_size(8 * 1024 * 1024)  // 8MB chunks
    ///     .max_concurrency(20);         // 20 parallel uploads
    ///
    /// let response = client.upload()
    ///     .upload_file_with_options("large_video.mp4", "/remote/video.mp4", options)
    ///     .await?;
    ///
    /// println!("Uploaded: {}", response.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file_with_options<P: AsRef<std::path::Path>>(
        &self,
        local_path: P,
        remote_path: &str,
        options: SimpleUploadOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let file = std::fs::File::open(&local_path).map_err(|e| NetDiskError::Unknown {
            message: format!(
                "Failed to open file {}: {}",
                local_path.as_ref().display(),
                e
            ),
        })?;

        let metadata = file.metadata().map_err(|e| NetDiskError::Unknown {
            message: format!(
                "Failed to get file metadata {}: {}",
                local_path.as_ref().display(),
                e
            ),
        })?;

        let file_size = metadata.len();
        debug!("File opened successfully: {} bytes", file_size);

        let mut reader = std::io::BufReader::new(file);
        self.upload_reader_with_options(&mut reader, file_size, remote_path, options)
            .await
    }

    /// Upload from a Reader with seek support (streaming API)
    ///
    /// This is a lower-level API that accepts any `Read + Seek` reader.
    /// Useful for:
    /// - Custom file wrapping (e.g., encrypted files)
    /// - Upload from memory-mapped files
    /// - Testing with custom readers
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    /// use std::io::BufReader;
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let file = std::fs::File::open("test.txt")?;
    /// let metadata = file.metadata()?;
    /// let file_size = metadata.len();
    ///
    /// let reader = BufReader::new(file);
    /// let mut reader = reader;  // mutable for rewind
    ///
    /// let response = client.upload()
    ///     .upload_reader(&mut reader, file_size, "/remote/test.txt")
    ///     .await?;
    ///
    /// println!("Uploaded: {}", response.path);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Memory Usage
    ///
    /// Memory is bounded by batch size (`max_concurrency * 2 * chunk_size`),
    /// approximately 80MB by default, regardless of file size.
    pub async fn upload_reader<R: std::io::Read + std::io::Seek>(
        &self,
        reader: &mut R,
        file_size: u64,
        remote_path: &str,
    ) -> NetDiskResult<CreateFileResponse> {
        self.upload_reader_with_options(
            reader,
            file_size,
            remote_path,
            SimpleUploadOptions::default(),
        )
        .await
    }

    /// Upload from a Reader with custom options
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::SimpleUploadOptions};
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let options = SimpleUploadOptions::default()
    ///     .chunk_size(8 * 1024 * 1024)
    ///     .max_concurrency(20);
    ///
    /// let file = std::fs::File::open("test.txt")?;
    /// let metadata = file.metadata()?;
    /// let file_size = metadata.len();
    ///
    /// let mut reader = std::io::BufReader::new(file);
    /// let response = client.upload()
    ///     .upload_reader_with_options(&mut reader, file_size, "/remote/test.txt", options)
    ///     .await?;
    ///
    /// println!("Uploaded: {}", response.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_reader_with_options<R: std::io::Read + std::io::Seek>(
        &self,
        reader: &mut R,
        file_size: u64,
        remote_path: &str,
        options: SimpleUploadOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let chunk_size = options.chunk_size;
        let max_concurrency = options.max_concurrency;
        let r#type = options.r#type;

        debug!(
            "upload_reader: file_size={} bytes, chunk_size={}",
            file_size, chunk_size
        );

        let mut block_list: Vec<String> = Vec::new();
        let mut read_chunks = 0usize;

        loop {
            let mut buffer = vec![0u8; chunk_size];
            let bytes_read = match reader.read(&mut buffer) {
                Ok(n) => n,
                Err(e) => {
                    debug!("First pass read error: {}", e);
                    break;
                }
            };

            if bytes_read == 0 {
                break;
            }

            buffer.truncate(bytes_read);
            let chunk_md5 = format!("{:x}", md5::compute(&buffer));
            block_list.push(chunk_md5);
            read_chunks += 1;
        }

        debug!("First pass: read {} chunks", read_chunks);

        reader.rewind().map_err(|e| NetDiskError::Unknown {
            message: format!("Failed to rewind reader: {}", e),
        })?;

        let precreate_options =
            PrecreateOptions::new(remote_path, file_size, block_list.clone()).rtype(r#type);

        let precreate_response = self.precreate(precreate_options).await?;

        let missing_blocks: Vec<u32> = precreate_response.block_list;
        debug!(
            "Server returned {} blocks need upload",
            missing_blocks.len()
        );

        if !missing_blocks.is_empty() {
            // Get upload server domain dynamically
            let locate_response = self
                .locate_upload(remote_path, &precreate_response.uploadid)
                .await?;
            let upload_server = locate_response.get_first_https_server();
            debug!("Located upload server: {:?}", upload_server);

            let missing_blocks_set: std::collections::HashSet<u32> =
                missing_blocks.into_iter().collect();

            let batch_size = max_concurrency * 2;
            let mut pending_chunks: Vec<(u32, Vec<u8>)> = Vec::with_capacity(batch_size);
            let mut all_chunk_md5s: Vec<(u32, String)> = Vec::new();
            let mut partseq = 0u32;

            loop {
                let mut buffer = vec![0u8; chunk_size];
                let bytes_read = match reader.read(&mut buffer) {
                    Ok(n) => n,
                    Err(e) => {
                        debug!("Second pass read error: {}", e);
                        break;
                    }
                };

                if bytes_read == 0 {
                    break;
                }

                buffer.truncate(bytes_read);
                let chunk_md5 = format!("{:x}", md5::compute(&buffer));

                if missing_blocks_set.contains(&partseq) {
                    pending_chunks.push((partseq, buffer));
                } else {
                    all_chunk_md5s.push((partseq, chunk_md5.clone()));
                }

                if pending_chunks.len() >= batch_size
                    || (partseq + 1 == read_chunks as u32 && !pending_chunks.is_empty())
                {
                    let batch_results = self
                        .upload_chunks_parallel(
                            remote_path,
                            &precreate_response.uploadid,
                            std::mem::take(&mut pending_chunks),
                            max_concurrency,
                            upload_server.as_deref(),
                        )
                        .await?;

                    for (seq, md5) in batch_results {
                        all_chunk_md5s.push((seq, md5));
                    }
                }

                partseq += 1;
            }

            all_chunk_md5s.sort_by_key(|(i, _)| *i);
            let new_block_list: Vec<String> =
                all_chunk_md5s.into_iter().map(|(_, md5)| md5).collect();

            let create_options = CreateFileOptions::new(
                remote_path,
                file_size,
                new_block_list,
                &precreate_response.uploadid,
            )
            .rtype(r#type);

            self.create_file(create_options).await
        } else {
            let create_options = CreateFileOptions::new(
                remote_path,
                file_size,
                block_list,
                &precreate_response.uploadid,
            )
            .rtype(r#type);

            self.create_file(create_options).await
        }
    }

    /// Upload bytes from memory (simple API)
    ///
    /// Use this method when you already have the data in memory.
    /// For large data, consider using [`UploadClient::upload_file`] or [`UploadClient::upload_reader`] instead
    /// to avoid loading the entire data into memory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let data = b"Hello, World!";
    /// let response = client.upload()
    ///     .upload_bytes(data, "/remote/hello.txt")
    ///     .await?;
    ///
    /// println!("Uploaded: {}", response.path);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Memory Note
    ///
    /// The entire `data` slice will be held in memory during upload.
    /// For large files, use [`UploadClient::upload_file`] which streams from disk.
    pub async fn upload_bytes(
        &self,
        data: &[u8],
        remote_path: &str,
    ) -> NetDiskResult<CreateFileResponse> {
        self.upload_bytes_with_options(data, remote_path, SimpleUploadOptions::default())
            .await
    }

    /// Upload bytes from memory with custom options
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::SimpleUploadOptions};
    ///
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let options = SimpleUploadOptions::default()
    ///     .chunk_size(8 * 1024 * 1024)
    ///     .max_concurrency(20);
    ///
    /// let data = b"Hello, World!";
    /// let response = client.upload()
    ///     .upload_bytes_with_options(data, "/remote/hello.txt", options)
    ///     .await?;
    ///
    /// println!("Uploaded: {}", response.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_bytes_with_options(
        &self,
        data: &[u8],
        remote_path: &str,
        options: SimpleUploadOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let file_size = data.len() as u64;
        let chunk_size = options.chunk_size;
        let max_concurrency = options.max_concurrency;
        let r#type = options.r#type;

        let block_list: Vec<String> = data
            .chunks(chunk_size)
            .map(|chunk| format!("{:x}", md5::compute(chunk)))
            .collect();

        let precreate_options =
            PrecreateOptions::new(remote_path, file_size, block_list.clone()).rtype(r#type);

        let precreate_response = self.precreate(precreate_options).await?;

        let missing_blocks_set: std::collections::HashSet<u32> =
            precreate_response.block_list.into_iter().collect();

        let chunks_to_upload: Vec<(u32, Vec<u8>)> = data
            .chunks(chunk_size)
            .enumerate()
            .filter(|(i, _)| missing_blocks_set.contains(&(*i as u32)))
            .map(|(i, chunk)| (i as u32, chunk.to_vec()))
            .collect();

        if !chunks_to_upload.is_empty() {
            // Get upload server domain dynamically
            let locate_response = self
                .locate_upload(remote_path, &precreate_response.uploadid)
                .await?;
            let upload_server = locate_response.get_first_https_server();
            debug!("Located upload server: {:?}", upload_server);

            let chunk_results = self
                .upload_chunks_parallel(
                    remote_path,
                    &precreate_response.uploadid,
                    chunks_to_upload,
                    max_concurrency,
                    upload_server.as_deref(),
                )
                .await?;

            let mut sorted_results = chunk_results;
            sorted_results.sort_by_key(|(i, _)| *i);
            let new_block_list: Vec<String> =
                sorted_results.into_iter().map(|(_, md5)| md5).collect();

            let create_options = CreateFileOptions::new(
                remote_path,
                file_size,
                new_block_list,
                &precreate_response.uploadid,
            )
            .rtype(r#type);

            self.create_file(create_options).await
        } else {
            let create_options = CreateFileOptions::new(
                remote_path,
                file_size,
                block_list,
                &precreate_response.uploadid,
            )
            .rtype(r#type);

            self.create_file(create_options).await
        }
    }
}
