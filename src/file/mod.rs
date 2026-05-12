//! File management and query functionality
//!
//! Provides file and folder management functionality, organized into three sub-modules:
//! - management: File/folder management operations (create, rename, delete, move, copy)
//! - query: File/folder query operations (list, get info, get metadata)
//! - category: File category search operations
//!
//! All operations are accessible through a unified FileClient interface.
//!
//! # Quick Start
//!
//! ```rust
//! # use tokio::main;
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder().build()?;
//! let token = client.load_token_from_env()?;
//!
//! // List directory
//! let files = client.file().list_directory(&token, "/").await?;
//!
//! // Create folder
//! let folder = client.file().create_folder(&token, "/my_new_folder").await?;
//!
//! // Get file metadata for download
//! let file_info = client.file().get_file_info(&token, "/myfile.txt").await?;
//! if let Some(fs_id) = file_info.fs_id {
//!     let meta = client.file().get_file_meta(&token, fs_id).await?;
//! }
//! # Ok(())
//! # }
//! ```
pub mod category;
pub mod management;
pub mod query;

pub use category::{
    BtListOptions, Category, CategoryCountOptions, CategorySearchOptions, DocumentListOptions,
    ImageListOptions, VideoListOptions,
};
pub use management::{FolderCreateOptions, FolderInfo};
pub use query::{
    FileInfo, FileMeta, ListAllOptions, ListOptions, SearchOptions, SemanticSearchOptions,
};

use crate::auth::AccessToken;
use crate::errors::NetDiskResult;
use crate::http::HttpClient;
use category::FileCategoryExt;
use management::FileManagementExt;
use query::FileQueryExt;

/// Unified File Client
///
/// Provides access to all file operations through a single interface.
/// Operations are implemented via extension traits in sub-modules.
///
/// # Examples
///
/// ```rust
/// use baidu_netdisk_sdk::BaiduNetDiskClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = BaiduNetDiskClient::builder().build()?;
/// let token = client.load_token_from_env()?;
///
/// // List files in directory
/// let files = client.file().list_directory(&token, "/").await?;
///
/// // Create a new folder
/// let folder = client.file().create_folder(&token, "/test").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileClient {
    http_client: HttpClient,
}

impl FileClient {
    /// Create a new FileClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::file()` instead.
    pub fn new(http_client: HttpClient) -> Self {
        FileClient { http_client }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    // ===== Query Operations =====

    /// List directory contents with default options
    pub async fn list_directory(
        &self,
        access_token: &AccessToken,
        dir: &str,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::list_directory(self, access_token, dir).await
    }

    /// List directory contents with custom options
    pub async fn list_directory_with_options(
        &self,
        access_token: &AccessToken,
        dir: &str,
        options: ListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::list_directory_with_options(self, access_token, dir, options).await
    }

    /// List all files recursively with default options
    pub async fn list_all(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::list_all(self, access_token, path).await
    }

    /// List all files recursively with custom options
    pub async fn list_all_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: ListAllOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::list_all_with_options(self, access_token, path, options).await
    }

    /// Get file or folder information by path
    pub async fn get_file_info(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<FileInfo> {
        FileQueryExt::get_file_info(self, access_token, path).await
    }

    /// Get file metadata (including download link) by fs_id
    pub async fn get_file_meta(
        &self,
        access_token: &AccessToken,
        fs_id: u64,
    ) -> NetDiskResult<FileMeta> {
        FileQueryExt::get_file_meta(self, access_token, fs_id).await
    }

    /// Search files by keyword
    pub async fn search_files(
        &self,
        access_token: &AccessToken,
        key: &str,
        dir: &str,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::search_files(self, access_token, key, dir).await
    }

    /// Search files by keyword with custom options
    pub async fn search_files_with_options(
        &self,
        access_token: &AccessToken,
        key: &str,
        options: SearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::search_files_with_options(self, access_token, key, options).await
    }

    /// Semantic search for files (AI-powered search)
    pub async fn semantic_search(
        &self,
        access_token: &AccessToken,
        query: &str,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::semantic_search(self, access_token, query).await
    }

    /// Semantic search with custom options
    pub async fn semantic_search_with_options(
        &self,
        access_token: &AccessToken,
        query: &str,
        options: SemanticSearchOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::semantic_search_with_options(self, access_token, query, options).await
    }

    // ===== Management Operations =====

    /// Create a folder
    pub async fn create_folder(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<FolderInfo> {
        FileManagementExt::create_folder(self, access_token, path).await
    }

    /// Create a folder with custom options
    pub async fn create_folder_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: FolderCreateOptions,
    ) -> NetDiskResult<FolderInfo> {
        FileManagementExt::create_folder_with_options(self, access_token, path, options).await
    }

    /// Rename a file or folder
    pub async fn rename(
        &self,
        access_token: &AccessToken,
        path: &str,
        new_name: &str,
    ) -> NetDiskResult<()> {
        FileManagementExt::rename(self, access_token, path, new_name).await
    }

    /// Delete a file or folder
    pub async fn delete(&self, access_token: &AccessToken, path: &str) -> NetDiskResult<()> {
        FileManagementExt::delete(self, access_token, path).await
    }

    /// Move a file or folder
    pub async fn move_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> NetDiskResult<()> {
        FileManagementExt::move_file(self, access_token, path, dest).await
    }

    /// Copy a file or folder
    pub async fn copy_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> NetDiskResult<()> {
        FileManagementExt::copy_file(self, access_token, path, dest).await
    }

    // ===== Category Operations =====

    /// Get the count of files in a specific category (simple version)
    pub async fn get_category_file_count(
        &self,
        access_token: &AccessToken,
        category: u32,
    ) -> NetDiskResult<u64> {
        FileCategoryExt::get_category_file_count(self, access_token, category).await
    }

    /// Get the count of files in a specific category (full parameters)
    pub async fn get_category_file_count_with_options(
        &self,
        access_token: &AccessToken,
        category: u32,
        options: CategoryCountOptions,
    ) -> NetDiskResult<u64> {
        FileCategoryExt::get_category_file_count_with_options(self, access_token, category, options)
            .await
    }

    /// Search files in a specific category (simple version)
    pub async fn search_category_files(
        &self,
        access_token: &AccessToken,
        category: u32,
        start: i32,
        limit: i32,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        FileCategoryExt::search_category_files(self, access_token, category, start, limit).await
    }

    /// Search files in a specific category (full parameters)
    pub async fn search_category_files_with_options(
        &self,
        access_token: &AccessToken,
        category: &str,
        options: CategorySearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        FileCategoryExt::search_category_files_with_options(self, access_token, category, options)
            .await
    }

    /// List documents
    pub async fn list_documents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_documents(self, access_token, parent_path, page, num).await
    }

    /// List documents with custom options
    pub async fn list_documents_with_options(
        &self,
        access_token: &AccessToken,
        options: DocumentListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_documents_with_options(self, access_token, options).await
    }

    /// List images
    pub async fn list_images(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_images(self, access_token, parent_path, page, num).await
    }

    /// List images with custom options
    pub async fn list_images_with_options(
        &self,
        access_token: &AccessToken,
        options: ImageListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_images_with_options(self, access_token, options).await
    }

    /// List videos
    pub async fn list_videos(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_videos(self, access_token, parent_path, page, num).await
    }

    /// List videos with custom options
    pub async fn list_videos_with_options(
        &self,
        access_token: &AccessToken,
        options: VideoListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_videos_with_options(self, access_token, options).await
    }

    /// List torrents
    pub async fn list_torrents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_torrents(self, access_token, parent_path, page, num).await
    }

    /// List torrents with custom options
    pub async fn list_torrents_with_options(
        &self,
        access_token: &AccessToken,
        options: BtListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileCategoryExt::list_torrents_with_options(self, access_token, options).await
    }
}
