//! File query module
//!
//! Provides file and folder query functionality (list, get info, get metadata)
use std::future::Future;
use log::{debug, info};
use serde::Deserialize;

use super::FileClient;
use crate::auth::AccessToken;
use crate::errors::{NetDiskError, NetDiskResult};

/// Extension trait for file query operations
pub(crate) trait FileQueryExt {
    /// List directory contents with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let files = client.file().list_directory(&token, "/").await?;
    /// println!("Found {} items", files.len());
    /// # Ok(())
    /// # }
    /// ```
    fn list_directory(
        &self,
        access_token: &AccessToken,
        dir: &str,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    /// List directory contents with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `list_directory()` instead.
    fn list_directory_with_options(
        &self,
        access_token: &AccessToken,
        dir: &str,
        options: ListOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    /// List all files recursively with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let (files, has_more) = client.file().list_all(&token, "/").await?;
    /// println!("Found {} items", files.len());
    /// # Ok(())
    /// # }
    /// ```
    fn list_all(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;

    /// List all files recursively with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `list_all()` instead.
    fn list_all_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: ListAllOptions,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;

    /// Get file or folder information by path
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let file_info = client.file().get_file_info(&token, "/myfile.txt").await?;
    /// println!("File size: {:?} bytes", file_info.size);
    /// # Ok(())
    /// # }
    /// ```
    fn get_file_info(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> impl Future<Output = NetDiskResult<FileInfo>> + Send;

    /// Get file metadata (including download link) by fs_id
    ///
    /// This method is specifically used for getting download links.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let fs_id = 123456;
    /// let meta = client.file().get_file_meta(&token, fs_id).await?;
    /// if let Some(dlink) = meta.dlink {
    ///     println!("Download link: {}", dlink);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn get_file_meta(
        &self,
        access_token: &AccessToken,
        fs_id: u64,
    ) -> impl Future<Output = NetDiskResult<FileMeta>> + Send;

    /// Search files by keyword
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let (files, has_more) = client.file().search_files(&token, "document", "/").await?;
    /// println!("Found {} items", files.len());
    /// # Ok(())
    /// # }
    /// ```
    fn search_files(
        &self,
        access_token: &AccessToken,
        key: &str,
        dir: &str,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;

    /// Search files by keyword with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `search_files()` instead.
    fn search_files_with_options(
        &self,
        access_token: &AccessToken,
        key: &str,
        options: SearchOptions,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;

    /// Semantic search for files (AI-powered search)
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let files = client.file().semantic_search(&token, "photos from 2024").await?;
    /// println!("Found {} items", files.len());
    /// # Ok(())
    /// # }
    /// ```
    fn semantic_search(
        &self,
        access_token: &AccessToken,
        query: &str,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    /// Semantic search with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `semantic_search()` instead.
    fn semantic_search_with_options(
        &self,
        access_token: &AccessToken,
        query: &str,
        options: SemanticSearchOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
}

impl FileQueryExt for FileClient {
    async fn list_directory(
        &self,
        access_token: &AccessToken,
        dir: &str,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.list_directory_with_options(access_token, dir, ListOptions::default())
            .await
    }

    async fn list_directory_with_options(
        &self,
        access_token: &AccessToken,
        dir: &str,
        options: ListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let params = [
            ("method", "list"),
            ("dir", dir),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("start", &options.start.to_string()),
            ("limit", &options.limit.to_string()),
            ("web", &options.web.to_string()),
            ("folder", &options.folder.to_string()),
            ("showempty", &options.showempty.to_string()),
            ("access_token", &access_token.access_token),
        ];

        debug!("Listing directory: {} with options: {:?}", dir, options);

        let response: ListResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        info!("Listed {} items in directory: {}", list.len(), dir);

        Ok(list
            .into_iter()
            .map(|item| FileInfo {
                fs_id: item.fs_id,
                path: item.path,
                size: item.size,
                ctime: item.ctime,
                mtime: item.mtime,
                isdir: item.isdir,
                name: item.server_filename,
                md5: item.md5,
                category: item.category,
                oper_id: item.oper_id,
                owner_id: item.owner_id,
                owner_type: item.owner_type,
                server_atime: item.server_atime,
                server_ctime: item.server_ctime,
                server_mtime: item.server_mtime,
            })
            .collect())
    }

    async fn list_all(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        self.list_all_with_options(access_token, path, ListAllOptions::new())
            .await
    }

    async fn list_all_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: ListAllOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let recursion_str = options.recursion.to_string();
        let desc_str = options.desc.to_string();
        let start_str = options.start.to_string();
        let limit_str = options.limit.to_string();
        let web_str = options.web.to_string();
        let ctime_str = options.ctime.map(|c| c.to_string()).unwrap_or_default();
        let mtime_str = options.mtime.map(|m| m.to_string()).unwrap_or_default();

        let mut params: Vec<(&str, &str)> = vec![
            ("method", "listall"),
            ("access_token", &access_token.access_token),
            ("path", path),
            ("recursion", &recursion_str),
            ("order", &options.order),
            ("desc", &desc_str),
            ("start", &start_str),
            ("limit", &limit_str),
            ("web", &web_str),
        ];

        if !ctime_str.is_empty() {
            params.push(("ctime", &ctime_str));
        }

        if !mtime_str.is_empty() {
            params.push(("mtime", &mtime_str));
        }

        if !options.device_id.is_empty() {
            params.push(("device_id", &options.device_id));
        }

        debug!(
            "Listing all files recursively: {} with options: {:?}",
            path, options
        );

        let response: ListAllResponse = self
            .http_client()
            .get("/rest/2.0/xpan/multimedia", Some(&params))
            .await?;

        debug!("ListAllResponse: {:?}", response);

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        let has_more = response.has_more.unwrap_or(0) == 1;

        let file_info_list = list
            .into_iter()
            .map(|item| FileInfo {
                fs_id: item.fs_id,
                path: item.path,
                size: item.size,
                ctime: item.local_ctime,
                mtime: item.local_mtime,
                isdir: item.isdir,
                name: item.server_filename,
                md5: item.md5,
                category: item.category,
                oper_id: None,
                owner_id: None,
                owner_type: None,
                server_atime: None,
                server_ctime: item.server_ctime,
                server_mtime: item.server_mtime,
            })
            .collect();

        Ok((file_info_list, has_more))
    }

    async fn get_file_info(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<FileInfo> {
        debug!("Getting file info for: {}", path);

        // Normalize path
        let normalized_path = if path.is_empty() { "/" } else { path };

        // Get parent directory
        let parent_path = if normalized_path == "/" {
            "/".to_string()
        } else {
            let parent_parts: Vec<&str> = normalized_path.split('/').collect();
            if parent_parts.len() <= 2 {
                "/".to_string()
            } else {
                let parent = parent_parts[..parent_parts.len() - 1].join("/");
                if parent.is_empty() {
                    "/".to_string()
                } else {
                    parent
                }
            }
        };

        let folder_name = normalized_path
            .rsplit('/')
            .next()
            .unwrap_or(normalized_path);

        // Use list_directory_with_options to get the parent directory listing
        let files = self
            .list_directory_with_options(access_token, &parent_path, ListOptions::default())
            .await?;

        for item in files {
            if item.path == normalized_path || item.name == folder_name {
                return Ok(item);
            }
        }

        Err(NetDiskError::api_error(-6, "File or folder not found"))
    }

    /// Get file metadata for download
    ///
    /// This method is specifically used for getting download links.
    /// According to Baidu NetDisk Open Platform documentation:
    /// <https://pan.baidu.com/union/doc/Fksg0sbcm>
    ///
    /// Note: The API endpoint is /rest/2.0/xpan/multimedia (not /rest/2.0/xpan/file)
    async fn get_file_meta(
        &self,
        access_token: &AccessToken,
        fs_id: u64,
    ) -> NetDiskResult<FileMeta> {
        let fsids = serde_json::to_string(&[fs_id]).map_err(|e| NetDiskError::Unknown {
            message: format!("Failed to serialize fsids: {}", e),
        })?;

        let params = [
            ("method", "filemetas"),
            ("access_token", &access_token.access_token),
            ("fsids", &fsids),
            ("dlink", "1"),
        ];

        debug!("Getting file metadata for download, fs_id: {}", fs_id);

        let response: FileMetaResponse = self
            .http_client()
            .get("/rest/2.0/xpan/multimedia", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = match response.list {
            Some(list) if !list.is_empty() => list,
            _ => {
                return Err(NetDiskError::api_error(
                    -1,
                    "File not found or no download link available",
                ))
            }
        };

        let file_info = &list[0];

        Ok(FileMeta {
            fs_id: Some(file_info.fs_id),
            path: file_info.path.clone(),
            size: file_info.size,
            name: file_info.server_filename.clone(),
            dlink: file_info.dlink.clone(),
        })
    }

    async fn search_files(
        &self,
        access_token: &AccessToken,
        key: &str,
        dir: &str,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        self.search_files_with_options(access_token, key, SearchOptions::new(dir))
            .await
    }

    async fn search_files_with_options(
        &self,
        access_token: &AccessToken,
        key: &str,
        options: SearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let category_str = options.category.map(|c| c.to_string());
        let mut params = vec![
            ("method", "search"),
            ("access_token", &access_token.access_token),
            ("key", key),
            ("dir", &options.dir),
            ("num", "500"),
        ];

        if let Some(ref category) = category_str {
            params.push(("category", category));
        }
        if options.recursion {
            params.push(("recursion", "1"));
        }
        if options.web {
            params.push(("web", "1"));
        }
        if !options.device_id.is_empty() {
            params.push(("device_id", &options.device_id));
        }

        debug!("Searching files with key: {}, options: {:?}", key, options);

        let response: SearchResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        let has_more = response.has_more == 1;

        let file_info_list = list
            .into_iter()
            .map(|item| FileInfo {
                fs_id: item.fs_id,
                path: item.path,
                size: item.size,
                ctime: item.local_ctime,
                mtime: item.local_mtime,
                isdir: item.isdir,
                name: item.server_filename,
                md5: item.md5,
                category: item.category,
                oper_id: None,
                owner_id: None,
                owner_type: None,
                server_atime: None,
                server_ctime: item.server_ctime,
                server_mtime: item.server_mtime,
            })
            .collect();

        Ok((file_info_list, has_more))
    }

    async fn semantic_search(
        &self,
        access_token: &AccessToken,
        query: &str,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.semantic_search_with_options(access_token, query, SemanticSearchOptions::default())
            .await
    }

    async fn semantic_search_with_options(
        &self,
        access_token: &AccessToken,
        query: &str,
        options: SemanticSearchOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let num_str = options.num.to_string();
        let stream_str = options.stream.to_string();
        let search_type_str = options.search_type.to_string();

        let url = format!("https://pan.baidu.com/xpan/unisearch?access_token={}&scene=mcpserver&query={}&num={}&stream={}&search_type={}",
            urlencoding::encode(&access_token.access_token),
            urlencoding::encode(query),
            num_str,
            stream_str,
            search_type_str
        );

        debug!(
            "Semantic search with query: {}, options: {:?}",
            query, options
        );

        let response: SemanticSearchResponse = self
            .http_client()
            .post_json(&url, &serde_json::json!({}))
            .await?;

        if response.error_no != 0 {
            let errmsg = response.error_msg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.error_no, errmsg));
        }

        let mut file_info_list = Vec::new();

        for data_item in response.data.unwrap_or_default() {
            for item in data_item.list.unwrap_or_default() {
                file_info_list.push(FileInfo {
                    fs_id: Some(item.fsid),
                    path: item.path,
                    size: item.size,
                    ctime: item.local_createtime,
                    mtime: item.local_mtime,
                    isdir: item.isdir,
                    name: item.filename,
                    md5: item.md5,
                    category: Some(item.category as u32),
                    oper_id: None,
                    owner_id: None,
                    owner_type: None,
                    server_atime: None,
                    server_ctime: item.server_ctime,
                    server_mtime: item.server_mtime,
                });
            }
        }

        Ok(file_info_list)
    }
}

/// Options for listing directory contents
#[derive(Debug)]
pub struct ListOptions {
    /// Sort order field (default: "name")
    pub order: String,
    /// Sort order descending (default: true)
    pub desc: i32,
    /// Start offset (default: 0)
    pub start: i32,
    /// Limit per page (default: 100)
    pub limit: i32,
    /// Web mode flag (default: true)
    pub web: i32,
    /// Show only folders (default: false)
    pub folder: i32,
    /// Show empty folders (default: false)
    pub showempty: i32,
}

impl Default for ListOptions {
    fn default() -> Self {
        ListOptions {
            order: "name".to_string(),
            desc: 1,
            start: 0,
            limit: 100,
            web: 1,
            folder: 0,
            showempty: 0,
        }
    }
}

impl ListOptions {
    /// Create new ListOptions with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set sort order field
    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    /// Set sort order (descending if true)
    pub fn desc(mut self, desc: bool) -> Self {
        self.desc = if desc { 1 } else { 0 };
        self
    }

    /// Set start offset
    pub fn start(mut self, start: i32) -> Self {
        self.start = start;
        self
    }

    /// Set limit per page
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = limit;
        self
    }

    /// Set web mode flag
    pub fn web(mut self, web: bool) -> Self {
        self.web = if web { 1 } else { 0 };
        self
    }

    /// Set show only folders flag
    pub fn folder(mut self, folder: bool) -> Self {
        self.folder = if folder { 1 } else { 0 };
        self
    }

    /// Set show empty folders flag
    pub fn showempty(mut self, showempty: bool) -> Self {
        self.showempty = if showempty { 1 } else { 0 };
        self
    }
}

/// Options for listing all files recursively
#[derive(Debug, Clone)]
pub struct ListAllOptions {
    /// Recursive listing (default: false)
    pub recursion: i32,
    /// Sort order field (default: "name")
    pub order: String,
    /// Sort order descending (default: false)
    pub desc: i32,
    /// Start offset (default: 0)
    pub start: i32,
    /// Limit per page (default: 1000)
    pub limit: i32,
    /// Filter by creation time
    pub ctime: Option<u64>,
    /// Filter by modification time
    pub mtime: Option<u64>,
    /// Web mode flag (default: false)
    pub web: i32,
    /// Device ID
    pub device_id: String,
}

impl Default for ListAllOptions {
    fn default() -> Self {
        ListAllOptions {
            recursion: 0,
            order: "name".to_string(),
            desc: 0,
            start: 0,
            limit: 1000,
            ctime: None,
            mtime: None,
            web: 0,
            device_id: String::new(),
        }
    }
}

impl ListAllOptions {
    /// Create new ListAllOptions with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set recursive mode
    pub fn recursion(mut self, recursion: bool) -> Self {
        self.recursion = if recursion { 1 } else { 0 };
        self
    }

    /// Set sort order field
    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    /// Set sort order (descending if true)
    pub fn desc(mut self, desc: bool) -> Self {
        self.desc = if desc { 1 } else { 0 };
        self
    }

    /// Set start offset
    pub fn start(mut self, start: i32) -> Self {
        self.start = start;
        self
    }

    /// Set limit per page (max 1000)
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = limit.min(1000);
        self
    }

    /// Set creation time filter
    pub fn ctime(mut self, ctime: u64) -> Self {
        self.ctime = Some(ctime);
        self
    }

    /// Set modification time filter
    pub fn mtime(mut self, mtime: u64) -> Self {
        self.mtime = Some(mtime);
        self
    }

    /// Set web mode flag
    pub fn web(mut self, web: bool) -> Self {
        self.web = if web { 1 } else { 0 };
        self
    }

    /// Set device ID
    pub fn device_id(mut self, device_id: &str) -> Self {
        self.device_id = device_id.to_string();
        self
    }
}

/// File or folder information
#[derive(Debug, Deserialize, Clone)]
pub struct FileInfo {
    /// File server ID
    pub fs_id: Option<u64>,
    /// File full path
    pub path: String,
    /// File size in bytes
    pub size: Option<u64>,
    /// Creation time (timestamp)
    pub ctime: Option<u64>,
    /// Modification time (timestamp)
    pub mtime: Option<u64>,
    /// Is directory (1=directory, 0=file)
    pub isdir: Option<i32>,
    /// File name
    pub name: String,
    /// MD5 hash
    pub md5: Option<String>,
    /// File category
    pub category: Option<u32>,
    /// Operator ID
    pub oper_id: Option<u64>,
    /// Owner ID
    pub owner_id: Option<u64>,
    /// Owner type
    pub owner_type: Option<u32>,
    /// Server access time
    pub server_atime: Option<u64>,
    /// Server creation time
    pub server_ctime: Option<u64>,
    /// Server modification time
    pub server_mtime: Option<u64>,
}

/// File metadata containing download link
#[derive(Debug, Deserialize, Clone)]
pub struct FileMeta {
    /// File server ID
    pub fs_id: Option<u64>,
    /// File path
    pub path: String,
    /// File size in bytes
    pub size: Option<u64>,
    /// File name
    pub name: String,
    /// Download link (dlink)
    pub dlink: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListResponse {
    errno: i32,
    errmsg: Option<String>,
    list: Option<Vec<ListItem>>,
}

#[derive(Debug, Deserialize)]
struct ListItem {
    fs_id: Option<u64>,
    path: String,
    size: Option<u64>,
    ctime: Option<u64>,
    mtime: Option<u64>,
    isdir: Option<i32>,
    server_filename: String,
    md5: Option<String>,
    category: Option<u32>,
    oper_id: Option<u64>,
    owner_id: Option<u64>,
    owner_type: Option<u32>,
    server_atime: Option<u64>,
    server_ctime: Option<u64>,
    server_mtime: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListAllResponse {
    errno: i32,
    errmsg: Option<String>,
    cursor: Option<u64>,
    has_more: Option<i32>,
    list: Option<Vec<ListAllItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListAllItem {
    category: Option<u32>,
    #[serde(rename = "fs_id")]
    fs_id: Option<u64>,
    isdir: Option<i32>,
    local_ctime: Option<u64>,
    local_mtime: Option<u64>,
    md5: Option<String>,
    path: String,
    server_filename: String,
    server_ctime: Option<u64>,
    server_mtime: Option<u64>,
    size: Option<u64>,
    thumbs: Option<Thumbs>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Thumbs {
    url1: Option<String>,
    url2: Option<String>,
    url3: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FileMetaResponse {
    errno: i32,
    errmsg: Option<String>,
    list: Option<Vec<FileMetaItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FileMetaItem {
    category: Option<u32>,
    dlink: Option<String>,
    #[serde(rename = "filename")]
    server_filename: String,
    fs_id: u64,
    isdir: Option<u32>,
    local_ctime: Option<u64>,
    local_mtime: Option<u64>,
    md5: Option<String>,
    oper_id: Option<u64>,
    path: String,
    server_ctime: Option<u64>,
    server_mtime: Option<u64>,
    size: Option<u64>,
}

/// Options for keyword search
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Directory to search in
    pub dir: String,
    /// File category filter
    pub category: Option<u32>,
    /// Recursive search flag
    pub recursion: bool,
    /// Web mode flag
    pub web: bool,
    /// Device ID
    pub device_id: String,
}

impl SearchOptions {
    /// Create new SearchOptions with directory
    pub fn new(dir: &str) -> Self {
        Self {
            dir: dir.to_string(),
            category: None,
            recursion: false,
            web: false,
            device_id: "".to_string(),
        }
    }

    /// Set file category filter
    pub fn category(mut self, category: u32) -> Self {
        self.category = Some(category);
        self
    }

    /// Set recursive search flag
    pub fn recursion(mut self, recursion: bool) -> Self {
        self.recursion = recursion;
        self
    }

    /// Set web mode flag
    pub fn web(mut self, web: bool) -> Self {
        self.web = web;
        self
    }

    /// Set device ID
    pub fn device_id(mut self, device_id: &str) -> Self {
        self.device_id = device_id.to_string();
        self
    }
}

/// Options for semantic search
#[derive(Debug, Clone, Default)]
pub struct SemanticSearchOptions {
    /// Directories to search in
    pub dirs: Vec<String>,
    /// File categories to search
    pub categories: Vec<u32>,
    /// Max number of results to return (default: 50)
    pub num: i32,
    /// Stream flag (default: 1)
    pub stream: i32,
    /// Search type (default: 0)
    pub search_type: i32,
    /// Sources to search from
    pub sources: Vec<i32>,
}

impl SemanticSearchOptions {
    /// Create new SemanticSearchOptions with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set directories to search in
    pub fn dirs(mut self, dirs: Vec<&str>) -> Self {
        self.dirs = dirs.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set file categories to search
    pub fn categories(mut self, categories: Vec<u32>) -> Self {
        self.categories = categories;
        self
    }

    /// Set max number of results
    pub fn num(mut self, num: i32) -> Self {
        self.num = num;
        self
    }

    /// Set stream flag
    pub fn stream(mut self, stream: i32) -> Self {
        self.stream = stream;
        self
    }

    /// Set search type
    pub fn search_type(mut self, search_type: i32) -> Self {
        self.search_type = search_type;
        self
    }

    /// Set sources to search from
    pub fn sources(mut self, sources: Vec<i32>) -> Self {
        self.sources = sources;
        self
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    errno: i32,
    errmsg: Option<String>,
    list: Option<Vec<SearchItem>>,
    has_more: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchItem {
    category: Option<u32>,
    #[serde(rename = "fs_id")]
    fs_id: Option<u64>,
    isdir: Option<i32>,
    local_ctime: Option<u64>,
    local_mtime: Option<u64>,
    server_ctime: Option<u64>,
    server_mtime: Option<u64>,
    md5: Option<String>,
    path: String,
    server_filename: String,
    size: Option<u64>,
    thumbs: Option<Thumbs>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SemanticSearchResponse {
    #[serde(rename = "error_no")]
    error_no: i32,
    #[serde(rename = "error_msg")]
    error_msg: Option<String>,
    data: Option<Vec<SemanticSearchDataItem>>,
    #[serde(rename = "is_end")]
    is_end: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SemanticSearchDataItem {
    list: Option<Vec<SemanticSearchItem>>,
    source: i32,
}

#[derive(Debug, Deserialize)]
struct SemanticSearchItem {
    category: i32,
    filename: String,
    fsid: u64,
    isdir: Option<i32>,
    #[serde(rename = "local_createtime")]
    local_createtime: Option<u64>,
    #[serde(rename = "local_mtime")]
    local_mtime: Option<u64>,
    md5: Option<String>,
    path: String,
    #[serde(rename = "server_ctime")]
    server_ctime: Option<u64>,
    #[serde(rename = "server_mtime")]
    server_mtime: Option<u64>,
    size: Option<u64>,
}
