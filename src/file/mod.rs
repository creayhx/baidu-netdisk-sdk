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
//! client.load_token_from_env()?;
//!
//! // List directory
//! let files = client.file().list_directory("/").await?;
//!
//! // Create folder
//! let folder = client.file().create_folder("/my_new_folder").await?;
//!
//! // Get file metadata for download
//! let file_info = client.file().get_file_info("/myfile.txt").await?;
//! if let Some(fs_id) = file_info.fs_id {
//!     let meta = client.file().get_file_meta(fs_id).await?;
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

use std::sync::Arc;

use crate::client::TokenGetter;
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
/// client.load_token_from_env()?;
///
/// // List files in directory
/// let files = client.file().list_directory("/").await?;
///
/// // Create a new folder
/// let folder = client.file().create_folder("/test").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileClient {
    http_client: HttpClient,
    token_getter: Arc<dyn TokenGetter>,
}

impl FileClient {
    /// Create a new FileClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::file()` instead.
    pub fn new(http_client: HttpClient, token_getter: Arc<dyn TokenGetter>) -> Self {
        FileClient {
            http_client,
            token_getter,
        }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    // ===== Query Operations =====

    /// List directory contents with default options
    pub async fn list_directory(&self, dir: &str) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::list_directory(self, dir).await
    }

    /// List directory contents with custom options
    pub async fn list_directory_with_options(
        &self,
        dir: &str,
        options: ListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::list_directory_with_options(self, dir, options).await
    }

    /// List all files recursively with default options
    pub async fn list_all(&self, path: &str) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::list_all(self, path).await
    }

    /// List all files recursively with custom options
    pub async fn list_all_with_options(
        &self,
        path: &str,
        options: ListAllOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::list_all_with_options(self, path, options).await
    }

    /// Get file or folder information by path
    pub async fn get_file_info(&self, path: &str) -> NetDiskResult<FileInfo> {
        FileQueryExt::get_file_info(self, path).await
    }

    /// Get file metadata (including download link) by fs_id
    pub async fn get_file_meta(&self, fs_id: u64) -> NetDiskResult<FileMeta> {
        FileQueryExt::get_file_meta(self, fs_id).await
    }

    /// Search files by keyword
    pub async fn search_files(&self, key: &str, dir: &str) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::search_files(self, key, dir).await
    }

    /// Search files by keyword with custom options
    pub async fn search_files_with_options(
        &self,
        key: &str,
        options: SearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        FileQueryExt::search_files_with_options(self, key, options).await
    }

    /// Semantic search for files (AI-powered search)
    pub async fn semantic_search(&self, query: &str) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::semantic_search(self, query).await
    }

    /// Semantic search with custom options
    pub async fn semantic_search_with_options(
        &self,
        query: &str,
        options: SemanticSearchOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        FileQueryExt::semantic_search_with_options(self, query, options).await
    }

    // ===== Management Operations =====

    /// Create a folder
    pub async fn create_folder(&self, path: &str) -> NetDiskResult<FolderInfo> {
        FileManagementExt::create_folder(self, path).await
    }

    /// Create a folder with custom options
    pub async fn create_folder_with_options(
        &self,
        path: &str,
        options: FolderCreateOptions,
    ) -> NetDiskResult<FolderInfo> {
        FileManagementExt::create_folder_with_options(self, path, options).await
    }

    /// Rename a file or folder
    pub async fn rename(&self, path: &str, new_name: &str) -> NetDiskResult<()> {
        FileManagementExt::rename(self, path, new_name).await
    }

    /// Delete a file or folder
    pub async fn delete(&self, path: &str) -> NetDiskResult<()> {
        FileManagementExt::delete(self, path).await
    }

    /// Move a file or folder
    pub async fn move_file(&self, path: &str, dest: &str) -> NetDiskResult<()> {
        FileManagementExt::move_file(self, path, dest).await
    }

    /// Copy a file or folder
    pub async fn copy_file(&self, path: &str, dest: &str) -> NetDiskResult<()> {
        FileManagementExt::copy_file(self, path, dest).await
    }

    // ===== Category Operations =====

    /// Get the count of files in a specific category (simple version)
    pub async fn get_category_file_count(&self, category: u32) -> NetDiskResult<u64> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::get_category_file_count(self, &token, category).await
    }

    /// Get the count of files in a specific category (full parameters)
    pub async fn get_category_file_count_with_options(
        &self,
        category: u32,
        options: CategoryCountOptions,
    ) -> NetDiskResult<u64> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::get_category_file_count_with_options(self, &token, category, options).await
    }

    /// Search files in a specific category (simple version)
    pub async fn search_category_files(
        &self,
        category: u32,
        start: i32,
        limit: i32,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::search_category_files(self, &token, category, start, limit).await
    }

    /// Search files in a specific category (full parameters)
    pub async fn search_category_files_with_options(
        &self,
        category: &str,
        options: CategorySearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::search_category_files_with_options(self, &token, category, options).await
    }

    /// List documents
    pub async fn list_documents(
        &self,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_documents(self, &token, parent_path, page, num).await
    }

    /// List documents with custom options
    pub async fn list_documents_with_options(
        &self,
        options: DocumentListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_documents_with_options(self, &token, options).await
    }

    /// List images
    pub async fn list_images(
        &self,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_images(self, &token, parent_path, page, num).await
    }

    /// List images with custom options
    pub async fn list_images_with_options(
        &self,
        options: ImageListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_images_with_options(self, &token, options).await
    }

    /// List videos
    pub async fn list_videos(
        &self,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_videos(self, &token, parent_path, page, num).await
    }

    /// List videos with custom options
    pub async fn list_videos_with_options(
        &self,
        options: VideoListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_videos_with_options(self, &token, options).await
    }

    /// List torrents
    pub async fn list_torrents(
        &self,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_torrents(self, &token, parent_path, page, num).await
    }

    /// List torrents with custom options
    pub async fn list_torrents_with_options(
        &self,
        options: BtListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.token_getter.get_token().await?;
        FileCategoryExt::list_torrents_with_options(self, &token, options).await
    }
}
