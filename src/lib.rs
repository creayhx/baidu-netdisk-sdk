//! Baidu NetDisk Rust SDK
//!
//! A Rust SDK for Baidu NetDisk API, providing file management,
//! upload/download and other functionalities.
//!
//! ## Features
//!
//! - API coverage: file management, upload/download, media processing, etc.
//! - Elegant error handling: layered error types for easy caller handling
//! - High performance: supports parallel upload and resumable upload
//! - Thread safety: uses RwLock for concurrent safety
//! - Configurable: flexible client configuration via Builder pattern
//!
//! ## Quick Start
//!
//! ```no_run
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = BaiduNetDiskClient::builder()
//!         .app_key("your_app_key")
//!         .app_secret("your_app_secret")
//!         .build()?;
//!     
//!     // Device code authorization
//!     let device_code = client.authorize().get_device_code().await?;
//!     println!("Please visit: {}", device_code.verification_url);
//!     
//!     // Poll for access token
//!     let token = loop {
//!         match client.authorize().request_access_token(&device_code).await? {
//!             Some(t) => break t,
//!             None => tokio::time::sleep(std::time::Duration::from_secs(device_code.interval as u64)).await,
//!         }
//!     };
//!     println!("Token acquired: {}", token.access_token);
//!     
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod download;
pub mod errors;
pub mod file;
pub mod http;
pub mod playlist;
pub mod quota;
pub mod upload;
pub mod user;

pub use crate::auth::{AccessToken, DeviceCode, QuotaInfo, TokenStatus, UserInfo};
pub use crate::client::BaiduNetDiskClient;
pub use crate::download::DownloadClient;
pub use crate::errors::{NetDiskError, NetDiskResult};
pub use crate::file::{
    BtListOptions, Category, CategoryCountOptions, CategorySearchOptions, DocumentListOptions,
    FileClient, FileInfo, FileMeta, FolderCreateOptions, FolderInfo, ImageListOptions,
    ListAllOptions, ListOptions, SearchOptions, SemanticSearchOptions, VideoListOptions,
};
pub use crate::playlist::{
    MediaFile, MediaFileEntry, MediaInfo, MediaPlayInfo, MediaStream, PlaylistClient,
    PlaylistFileInfo, PlaylistFileList, PlaylistInfo, PlaylistList,
};
pub use crate::quota::CapacityInfo;
pub use crate::upload::{
    CreateFileOptions, CreateFileResponse, LocateUploadResponse, LocateUploadServer,
    PrecreateOptions, PrecreateResponse, SimpleUploadOptions, UploadChunkOptions,
    UploadChunkResponse, UploadClient,
};
