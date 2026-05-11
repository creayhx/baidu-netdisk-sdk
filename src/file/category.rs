use std::future::Future;

/// File category module
///
/// Provides file category search functionality
///
/// # Category values:
/// - 1: Video
/// - 2: Music
/// - 3: Image
/// - 4: Document
/// - 5: Application
/// - 6: Other
/// - 7: Torrent
use log::debug;
use serde::Deserialize;

use super::query::FileInfo;
use super::FileClient;
use crate::auth::AccessToken;
use crate::errors::{NetDiskError, NetDiskResult};

/// Extension trait for file category operations
pub(crate) trait FileCategoryExt {
    /// Get the count of files in a specific category (simple version)
    ///
    /// # Arguments
    /// * `access_token` - Access token
    /// * `category` - File category (1-7)
    ///
    /// # Returns
    /// * The number of files in the specified category
    fn get_category_file_count(
        &self,
        access_token: &AccessToken,
        category: u32,
    ) -> impl Future<Output = NetDiskResult<u64>> + Send;

    /// Get the count of files in a specific category (full parameters)
    ///
    /// According to Baidu NetDisk Open Platform documentation:
    /// <https://pan.baidu.com/union/doc/dksg0sanx>
    ///
    /// # Arguments
    /// * `access_token` - Access token
    /// * `category` - File category (1-7)
    /// * `options` - Count options
    ///
    /// # Returns
    /// * The number of files in the specified category
    fn get_category_file_count_with_options(
        &self,
        access_token: &AccessToken,
        category: u32,
        options: CategoryCountOptions,
    ) -> impl Future<Output = NetDiskResult<u64>> + Send;

    /// Search files in a specific category (simple version)
    ///
    /// # Arguments
    /// * `access_token` - Access token
    /// * `category` - File category (1-7)
    /// * `start` - Start index for pagination
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// * A tuple containing the list of files and the total count
    fn search_category_files(
        &self,
        access_token: &AccessToken,
        category: u32,
        start: i32,
        limit: i32,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, u64)>> + Send;

    /// Search files in a specific category (full parameters)
    ///
    /// According to Baidu NetDisk Open Platform documentation:
    /// <https://pan.baidu.com/union/doc/Sksg0sb40>
    ///
    /// # Arguments
    /// * `access_token` - Access token
    /// * `category` - File category (1-7), multiple categories can be separated by commas
    /// * `options` - Search options
    ///
    /// # Returns
    /// * A tuple containing the list of files and the total count
    fn search_category_files_with_options(
        &self,
        access_token: &AccessToken,
        category: &str,
        options: CategorySearchOptions,
    ) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, u64)>> + Send;

    fn list_documents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_documents_with_options(
        &self,
        access_token: &AccessToken,
        options: DocumentListOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_images(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_images_with_options(
        &self,
        access_token: &AccessToken,
        options: ImageListOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_videos(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_videos_with_options(
        &self,
        access_token: &AccessToken,
        options: VideoListOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_torrents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;

    fn list_torrents_with_options(
        &self,
        access_token: &AccessToken,
        options: BtListOptions,
    ) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
}

impl FileCategoryExt for FileClient {
    async fn get_category_file_count(
        &self,
        access_token: &AccessToken,
        category: u32,
    ) -> NetDiskResult<u64> {
        self.get_category_file_count_with_options(
            access_token,
            category,
            CategoryCountOptions::default(),
        )
        .await
    }

    async fn get_category_file_count_with_options(
        &self,
        access_token: &AccessToken,
        category: u32,
        options: CategoryCountOptions,
    ) -> NetDiskResult<u64> {
        let cat_str = category.to_string();
        let recursion_str = options.recursion.to_string();
        let params = [
            ("access_token", access_token.access_token.as_str()),
            ("category", cat_str.as_str()),
            ("parent_path", options.parent_path.as_str()),
            ("recursion", recursion_str.as_str()),
        ];

        debug!(
            "Getting category file count: category={}, options={:?}",
            category, options
        );

        let response: CategoryInfoResponse = self
            .http_client()
            .get("/api/categoryinfo", Some(&params))
            .await?;

        debug!("CategoryInfoResponse: {:?}", response);

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        Ok(response.total.unwrap_or(0))
    }

    async fn search_category_files(
        &self,
        access_token: &AccessToken,
        category: u32,
        start: i32,
        limit: i32,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let options = CategorySearchOptions::new().start(start).limit(limit);
        self.search_category_files_with_options(access_token, &category.to_string(), options)
            .await
    }

    async fn search_category_files_with_options(
        &self,
        access_token: &AccessToken,
        category: &str,
        options: CategorySearchOptions,
    ) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let params = [
            ("method", "categorylist"),
            ("access_token", &access_token.access_token),
            ("category", category),
            ("show_dir", &options.show_dir.to_string()),
            ("parent_path", &options.parent_path),
            ("recursion", &options.recursion.to_string()),
            ("ext", &options.ext),
            ("start", &options.start.to_string()),
            ("limit", &options.limit.to_string()),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("device_id", &options.device_id),
        ];

        debug!(
            "Searching category files: category={}, options={:?}",
            category, options
        );

        let response: CategorySearchResponse = self
            .http_client()
            .get("/rest/2.0/xpan/multimedia", Some(&params))
            .await?;

        debug!("CategorySearchResponse: {:?}", response);

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        let total = list.len() as u64;

        let file_info_list = list
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
            .collect();

        Ok((file_info_list, total))
    }

    async fn list_documents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.list_documents_with_options(
            access_token,
            DocumentListOptions::new(parent_path).page(page).num(num),
        )
        .await
    }

    async fn list_documents_with_options(
        &self,
        access_token: &AccessToken,
        options: DocumentListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let params = [
            ("method", "doclist"),
            ("access_token", &access_token.access_token),
            ("parent_path", &options.parent_path),
            ("page", &options.page.to_string()),
            ("num", &options.num.to_string()),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("recursion", &options.recursion.to_string()),
            ("web", &options.web.to_string()),
        ];

        debug!("Listing documents with options: {:?}", options);

        let response: FixedCategoryListResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.info.unwrap_or_default();

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

        Ok(file_info_list)
    }

    async fn list_images(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.list_images_with_options(
            access_token,
            ImageListOptions::new(parent_path).page(page).num(num),
        )
        .await
    }

    async fn list_images_with_options(
        &self,
        access_token: &AccessToken,
        options: ImageListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let params = [
            ("method", "imagelist"),
            ("access_token", &access_token.access_token),
            ("parent_path", &options.parent_path),
            ("page", &options.page.to_string()),
            ("num", &options.num.to_string()),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("recursion", &options.recursion.to_string()),
            ("web", &options.web.to_string()),
        ];

        debug!("Listing images with options: {:?}", options);

        let response: FixedCategoryListResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.info.unwrap_or_default();

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

        Ok(file_info_list)
    }

    async fn list_videos(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.list_videos_with_options(
            access_token,
            VideoListOptions::new(parent_path).page(page).num(num),
        )
        .await
    }

    async fn list_videos_with_options(
        &self,
        access_token: &AccessToken,
        options: VideoListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let params = [
            ("method", "videolist"),
            ("access_token", &access_token.access_token),
            ("parent_path", &options.parent_path),
            ("page", &options.page.to_string()),
            ("num", &options.num.to_string()),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("recursion", &options.recursion.to_string()),
            ("web", &options.web.to_string()),
        ];

        debug!("Listing videos with options: {:?}", options);

        let response: FixedCategoryListResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.info.unwrap_or_default();

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

        Ok(file_info_list)
    }

    async fn list_torrents(
        &self,
        access_token: &AccessToken,
        parent_path: &str,
        page: i32,
        num: i32,
    ) -> NetDiskResult<Vec<FileInfo>> {
        self.list_torrents_with_options(
            access_token,
            BtListOptions::new(parent_path).page(page).num(num),
        )
        .await
    }

    async fn list_torrents_with_options(
        &self,
        access_token: &AccessToken,
        options: BtListOptions,
    ) -> NetDiskResult<Vec<FileInfo>> {
        let params = [
            ("method", "btlist"),
            ("access_token", &access_token.access_token),
            ("parent_path", &options.parent_path),
            ("page", &options.page.to_string()),
            ("num", &options.num.to_string()),
            ("order", &options.order),
            ("desc", &options.desc.to_string()),
            ("recursion", &options.recursion.to_string()),
        ];

        debug!("Listing torrents with options: {:?}", options);

        let response: FixedCategoryListResponse = self
            .http_client()
            .get("/rest/2.0/xpan/file", Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.info.unwrap_or_default();

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

        Ok(file_info_list)
    }
}

/// Options for category file count
#[derive(Debug, Clone, Default)]
pub struct CategoryCountOptions {
    pub parent_path: String,
    pub recursion: i32,
}

impl CategoryCountOptions {
    pub fn new() -> Self {
        Self {
            parent_path: "/".to_string(),
            recursion: 1,
        }
    }

    /// Set parent directory path
    pub fn parent_path(mut self, parent_path: &str) -> Self {
        self.parent_path = parent_path.to_string();
        self
    }

    /// Set recursion mode (0: no recursion, 1: recursion)
    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }
}

/// Options for category file search
#[derive(Debug, Clone, Default)]
pub struct CategorySearchOptions {
    pub show_dir: i32,
    pub parent_path: String,
    pub recursion: i32,
    pub ext: String,
    pub start: i32,
    pub limit: i32,
    pub order: String,
    pub desc: i32,
    pub device_id: String,
}

impl CategorySearchOptions {
    pub fn new() -> Self {
        Self {
            show_dir: 0,
            parent_path: "/".to_string(),
            recursion: 1,
            ext: String::new(),
            start: 0,
            limit: 1000,
            order: "time".to_string(),
            desc: 1,
            device_id: String::new(),
        }
    }

    /// Set whether to show directories (0: no, 1: yes)
    pub fn show_dir(mut self, show_dir: i32) -> Self {
        self.show_dir = show_dir;
        self
    }

    /// Set parent directory path
    pub fn parent_path(mut self, parent_path: &str) -> Self {
        self.parent_path = parent_path.to_string();
        self
    }

    /// Set recursion mode (0: no recursion, 1: recursion)
    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }

    /// Set file extensions (e.g., "mp3,txt")
    pub fn ext(mut self, ext: &str) -> Self {
        self.ext = ext.to_string();
        self
    }

    /// Set start index for pagination
    pub fn start(mut self, start: i32) -> Self {
        self.start = start;
        self
    }

    /// Set limit for pagination (max 1000)
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = limit.min(1000);
        self
    }

    /// Set order field ("time", "name", "size")
    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    /// Set sort direction (0: ascending, 1: descending)
    pub fn desc(mut self, desc: i32) -> Self {
        self.desc = desc;
        self
    }

    /// Set device ID
    pub fn device_id(mut self, device_id: &str) -> Self {
        self.device_id = device_id.to_string();
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentListOptions {
    pub parent_path: String,
    pub page: i32,
    pub num: i32,
    pub order: String,
    pub desc: i32,
    pub recursion: i32,
    pub web: i32,
}

impl DocumentListOptions {
    pub fn new(parent_path: &str) -> Self {
        Self {
            parent_path: parent_path.to_string(),
            page: 1,
            num: 100,
            order: "time".to_string(),
            desc: 1,
            recursion: 0,
            web: 0,
        }
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn num(mut self, num: i32) -> Self {
        self.num = num.min(1000);
        self
    }

    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    pub fn desc(mut self, desc: i32) -> Self {
        self.desc = desc;
        self
    }

    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }

    pub fn web(mut self, web: i32) -> Self {
        self.web = web;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImageListOptions {
    pub parent_path: String,
    pub page: i32,
    pub num: i32,
    pub order: String,
    pub desc: i32,
    pub recursion: i32,
    pub web: i32,
}

impl ImageListOptions {
    pub fn new(parent_path: &str) -> Self {
        Self {
            parent_path: parent_path.to_string(),
            page: 1,
            num: 100,
            order: "time".to_string(),
            desc: 1,
            recursion: 0,
            web: 0,
        }
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn num(mut self, num: i32) -> Self {
        self.num = num.min(1000);
        self
    }

    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    pub fn desc(mut self, desc: i32) -> Self {
        self.desc = desc;
        self
    }

    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }

    pub fn web(mut self, web: i32) -> Self {
        self.web = web;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct VideoListOptions {
    pub parent_path: String,
    pub page: i32,
    pub num: i32,
    pub order: String,
    pub desc: i32,
    pub recursion: i32,
    pub web: i32,
}

impl VideoListOptions {
    pub fn new(parent_path: &str) -> Self {
        Self {
            parent_path: parent_path.to_string(),
            page: 1,
            num: 100,
            order: "time".to_string(),
            desc: 1,
            recursion: 0,
            web: 0,
        }
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn num(mut self, num: i32) -> Self {
        self.num = num.min(1000);
        self
    }

    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    pub fn desc(mut self, desc: i32) -> Self {
        self.desc = desc;
        self
    }

    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }

    pub fn web(mut self, web: i32) -> Self {
        self.web = web;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct BtListOptions {
    pub parent_path: String,
    pub page: i32,
    pub num: i32,
    pub order: String,
    pub desc: i32,
    pub recursion: i32,
}

impl BtListOptions {
    pub fn new(parent_path: &str) -> Self {
        Self {
            parent_path: parent_path.to_string(),
            page: 1,
            num: 100,
            order: "time".to_string(),
            desc: 1,
            recursion: 0,
        }
    }

    pub fn page(mut self, page: i32) -> Self {
        self.page = page;
        self
    }

    pub fn num(mut self, num: i32) -> Self {
        self.num = num.min(1000);
        self
    }

    pub fn order(mut self, order: &str) -> Self {
        self.order = order.to_string();
        self
    }

    pub fn desc(mut self, desc: i32) -> Self {
        self.desc = desc;
        self
    }

    pub fn recursion(mut self, recursion: i32) -> Self {
        self.recursion = recursion;
        self
    }
}

/// File category enum for type-safe category selection
#[derive(Debug, Clone, Copy)]
pub enum Category {
    Video = 1,
    Music = 2,
    Image = 3,
    Document = 4,
    Application = 5,
    Other = 6,
    Torrent = 7,
}

impl Category {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CategorySearchResponse {
    errno: i32,
    errmsg: Option<String>,
    list: Option<Vec<CategorySearchItem>>,
    cursor: Option<u64>,
    has_more: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CategoryInfoResponse {
    errno: i32,
    errmsg: Option<String>,
    total: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CategorySearchItem {
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
struct FixedCategoryListResponse {
    errno: i32,
    errmsg: Option<String>,
    guid_info: Option<String>,
    info: Option<Vec<FixedCategoryListItem>>,
}

#[derive(Debug, Deserialize)]
struct FixedCategoryListItem {
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
}
