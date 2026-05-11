//! File upload functionality for Baidu NetDisk
//!
//! This module provides multi-step file upload capability:
//! 1. Precreate - Initiate the upload and check for existing chunks
//! 2. Upload Chunks - Upload individual file chunks
//! 3. Create File - Merge chunks into a final file on the server
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::PrecreateOptions};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder().build()?;
//! let token = client.load_token_from_env()?;
//!
//! // Assume we have a file and its block md5 list
//! let block_list = vec!["md5_of_block1".to_string()];
//! let options = PrecreateOptions::new("/test_file.txt", 1024, block_list);
//!
//! // Step 1: Precreate
//! let precreate_resp = client.upload().precreate(&token, options).await?;
//!
//! // If needed, upload missing chunks
//! // Step 2: Upload chunks
//! // Step 3: Create final file
//! # Ok(())
//! # }
//! ```
use crate::auth::AccessToken;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;
use futures::stream::{self, StreamExt};
use log::debug;
use serde::Deserialize;

/// Upload client for Baidu NetDisk
#[derive(Debug, Clone)]
pub struct UploadClient {
    http_client: HttpClient,
}

impl UploadClient {
    /// Create a new UploadClient instance
    ///
    /// Usually you don't need to call this directly - use BaiduNetDiskClient::upload() instead.
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Precreate an upload session
    ///
    /// Initiates an upload session and checks which chunks (if any) already exist on the server.
    /// This is the first step of the multi-step upload process.
    pub async fn precreate(
        &self,
        access_token: &AccessToken,
        options: PrecreateOptions,
    ) -> NetDiskResult<PrecreateResponse> {
        let block_list_json =
            serde_json::to_string(&options.block_list).map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to serialize block_list: {}", e),
            })?;

        let params = vec![
            ("method", "precreate"),
            ("access_token", access_token.access_token.as_str()),
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

impl UploadClient {
    /// Upload a single chunk
    pub async fn upload_chunk(
        &self,
        access_token: &AccessToken,
        options: UploadChunkOptions,
    ) -> NetDiskResult<UploadChunkResponse> {
        let url = format!(
            "https://c3.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq={}",
            urlencoding::encode(&access_token.access_token),
            urlencoding::encode(&options.path),
            urlencoding::encode(&options.uploadid),
            options.partseq
        );

        debug!(
            "Upload chunk: path={}, uploadid={}, partseq={}, data_size={}",
            options.path,
            options.uploadid,
            options.partseq,
            options.data.len()
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
    pub async fn upload_chunks_parallel(
        &self,
        access_token: &AccessToken,
        remote_path: &str,
        uploadid: &str,
        chunks: Vec<(u32, Vec<u8>)>,
        max_concurrency: usize,
    ) -> NetDiskResult<Vec<(u32, String)>> {
        debug!(
            "Uploading {} chunks in parallel (max_concurrency: {})",
            chunks.len(),
            max_concurrency
        );

        let access_token_str = access_token.access_token.clone();
        let remote_path_str = remote_path.to_string();
        let uploadid_str = uploadid.to_string();
        let http_client = self.http_client.clone();

        let mut stream = stream::iter(chunks)
            .map(|(partseq, data)| {
                let path = remote_path_str.clone();
                let uid = access_token_str.clone();
                let upid = uploadid_str.clone();
                let client = http_client.clone();

                async move {
                    let url = format!(
                        "https://c3.pcs.baidu.com/rest/2.0/pcs/superfile2?method=upload&access_token={}&type=tmpfile&path={}&uploadid={}&partseq={}",
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
        access_token: &AccessToken,
        options: CreateFileOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let block_list_json =
            serde_json::to_string(&options.block_list).map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to serialize block_list: {}", e),
            })?;

        let params = vec![
            ("method", "create"),
            ("access_token", access_token.access_token.as_str()),
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
