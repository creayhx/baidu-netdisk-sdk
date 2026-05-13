use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use dotenv::dotenv;
use log::{debug, error, info, warn};

use crate::auth::{AccessToken, QuotaInfo, TokenStatus, UserInfo};
use crate::auth::{Authorization, TokenProvider, TokenProviderConfig};
use crate::download::DownloadClient;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::file::{
    BtListOptions, CategoryCountOptions, CategorySearchOptions, DocumentListOptions,
    FileClient, FileInfo, FileMeta, FolderCreateOptions, FolderInfo, ImageListOptions,
    ListAllOptions, ListOptions, SearchOptions, SemanticSearchOptions, VideoListOptions,
};
use crate::http::{client::HttpClientConfig, HttpClient};
use crate::playlist::{AudioQuality, MediaPlayInfo, PlaylistClient, PlaylistFileList, PlaylistList, VideoQuality};
use crate::quota::{CapacityInfo, QuotaClient};
use crate::upload::{CreateFileResponse, SimpleUploadOptions, UploadClient};
use crate::user::UserClient;

pub(crate) trait ClientAccessor {
    fn get_token(&self) -> impl Future<Output = NetDiskResult<AccessToken>> + Send + '_;
    fn user_client(&self) -> &UserClient;
    fn quota_client(&self) -> &QuotaClient;
    fn file_client(&self) -> &FileClient;
    fn download_client(&self) -> &DownloadClient;
    fn upload_client(&self) -> &UploadClient;
    fn playlist_client(&self) -> &PlaylistClient;
}

pub trait NetDiskApi {
    fn get_user_info(&self, vip_version: Option<&str>) -> impl Future<Output = NetDiskResult<UserInfo>> + Send;
    fn get_quota(&self) -> impl Future<Output = NetDiskResult<QuotaInfo>> + Send;
    fn get_capacity(&self, check_free: bool, check_expire: bool) -> impl Future<Output = NetDiskResult<CapacityInfo>> + Send;
    fn get_quota_with_expire(&self) -> impl Future<Output = NetDiskResult<CapacityInfo>> + Send;
    fn list_directory(&self, dir: &str) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_directory_with_options(&self, dir: &str, options: ListOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_all(&self, path: &str) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;
    fn list_all_with_options(&self, path: &str, options: ListAllOptions) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;
    fn get_file_info(&self, path: &str) -> impl Future<Output = NetDiskResult<FileInfo>> + Send;
    fn get_file_meta(&self, fs_id: u64) -> impl Future<Output = NetDiskResult<FileMeta>> + Send;
    fn search_files(&self, key: &str, dir: &str) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;
    fn search_files_with_options(&self, key: &str, options: SearchOptions) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, bool)>> + Send;
    fn semantic_search(&self, query: &str) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn semantic_search_with_options(&self, query: &str, options: SemanticSearchOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn get_category_file_count(&self, category: u32) -> impl Future<Output = NetDiskResult<u64>> + Send;
    fn get_category_file_count_with_options(&self, category: u32, options: CategoryCountOptions) -> impl Future<Output = NetDiskResult<u64>> + Send;
    fn search_category_files(&self, category: u32, start: i32, limit: i32) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, u64)>> + Send;
    fn search_category_files_with_options(&self, category: &str, options: CategorySearchOptions) -> impl Future<Output = NetDiskResult<(Vec<FileInfo>, u64)>> + Send;
    fn list_documents(&self, parent_path: &str, page: i32, num: i32) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_documents_with_options(&self, options: DocumentListOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_images(&self, parent_path: &str, page: i32, num: i32) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_images_with_options(&self, options: ImageListOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_videos(&self, parent_path: &str, page: i32, num: i32) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_videos_with_options(&self, options: VideoListOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_torrents(&self, parent_path: &str, page: i32, num: i32) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn list_torrents_with_options(&self, options: BtListOptions) -> impl Future<Output = NetDiskResult<Vec<FileInfo>>> + Send;
    fn create_folder(&self, path: &str) -> impl Future<Output = NetDiskResult<FolderInfo>> + Send;
    fn create_folder_with_options(&self, path: &str, options: FolderCreateOptions) -> impl Future<Output = NetDiskResult<FolderInfo>> + Send;
    fn delete_file(&self, path: &str) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn rename_file(&self, path: &str, new_name: &str) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn move_file(&self, path: &str, dest: &str) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn copy_file(&self, path: &str, dest: &str) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn upload_file<P: AsRef<std::path::Path> + Send>(&self, local_path: P, remote_path: &str) -> impl Future<Output = NetDiskResult<CreateFileResponse>> + Send;
    fn upload_file_with_options<P: AsRef<std::path::Path> + Send>(
        &self,
        local_path: P,
        remote_path: &str,
        options: SimpleUploadOptions,
    ) -> impl Future<Output = NetDiskResult<CreateFileResponse>> + Send;
    fn upload_bytes(&self, data: &[u8], remote_path: &str) -> impl Future<Output = NetDiskResult<CreateFileResponse>> + Send;
    fn upload_bytes_with_options(&self, data: &[u8], remote_path: &str, options: SimpleUploadOptions) -> impl Future<Output = NetDiskResult<CreateFileResponse>> + Send;
    fn auto_download(&self, path: &str, save_path: impl AsRef<Path> + Send) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn auto_download_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_single(&self, path: &str, save_path: impl AsRef<Path> + Send) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_single_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_parallel(&self, path: &str, save_path: impl AsRef<Path> + Send, thread_num: Option<usize>) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_parallel_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send, thread_num: Option<usize>) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_streaming(&self, path: &str, save_path: impl AsRef<Path> + Send, max_concurrency: usize) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn download_streaming_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send, max_concurrency: usize) -> impl Future<Output = NetDiskResult<()>> + Send;
    fn get_dlink_from_path(&self, path: &str) -> impl Future<Output = NetDiskResult<FileMeta>> + Send;
    fn get_dlink_from_fsid(&self, fs_id: u64) -> impl Future<Output = NetDiskResult<FileMeta>> + Send;
    fn get_playlist_list(&self) -> impl Future<Output = NetDiskResult<PlaylistList>> + Send;
    fn get_playlist_list_with_options(&self, options: crate::playlist::PlaylistListOptions) -> impl Future<Output = NetDiskResult<PlaylistList>> + Send;
    fn get_playlist_file_list(&self, mb_id: u64) -> impl Future<Output = NetDiskResult<PlaylistFileList>> + Send;
    fn get_playlist_file_list_with_options(&self, mb_id: u64, options: crate::playlist::PlaylistFileListOptions) -> impl Future<Output = NetDiskResult<PlaylistFileList>> + Send;
    fn get_media_play_info(&self, fsid: Option<u64>, path: Option<&str>, media_type: &str) -> impl Future<Output = NetDiskResult<MediaPlayInfo>> + Send;
    fn get_media_m3u8_content(&self, path: &str, media_type: &str) -> impl Future<Output = NetDiskResult<String>> + Send;
    fn get_video_m3u8(&self, path: &str, quality: VideoQuality) -> impl Future<Output = NetDiskResult<String>> + Send;
    fn get_video_m3u8_highest(&self, path: &str, vip_level: u32) -> impl Future<Output = NetDiskResult<String>> + Send;
    fn get_audio_m3u8(&self, path: &str, quality: AudioQuality) -> impl Future<Output = NetDiskResult<String>> + Send;
    fn get_audio_m3u8_default(&self, path: &str) -> impl Future<Output = NetDiskResult<String>> + Send;
}

impl<T: ClientAccessor + Send + Sync> NetDiskApi for T {
    async fn get_user_info(&self, vip_version: Option<&str>) -> NetDiskResult<UserInfo> {
        let token = self.get_token().await?;
        self.user_client().get_user_info(&token, vip_version).await
    }

    async fn get_quota(&self) -> NetDiskResult<QuotaInfo> {
        let token = self.get_token().await?;
        self.quota_client().get_quota(&token).await
    }

    async fn get_capacity(&self, check_free: bool, check_expire: bool) -> NetDiskResult<CapacityInfo> {
        let token = self.get_token().await?;
        self.quota_client().get_capacity(&token, check_free, check_expire).await
    }

    async fn get_quota_with_expire(&self) -> NetDiskResult<CapacityInfo> {
        let token = self.get_token().await?;
        self.quota_client().get_quota_with_expire(&token).await
    }

    async fn list_directory(&self, dir: &str) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_directory(&token, dir).await
    }

    async fn list_directory_with_options(&self, dir: &str, options: ListOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_directory_with_options(&token, dir, options).await
    }

    async fn list_all(&self, path: &str) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let token = self.get_token().await?;
        self.file_client().list_all(&token, path).await
    }

    async fn list_all_with_options(&self, path: &str, options: ListAllOptions) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let token = self.get_token().await?;
        self.file_client().list_all_with_options(&token, path, options).await
    }

    async fn get_file_info(&self, path: &str) -> NetDiskResult<FileInfo> {
        let token = self.get_token().await?;
        self.file_client().get_file_info(&token, path).await
    }

    async fn get_file_meta(&self, fs_id: u64) -> NetDiskResult<FileMeta> {
        let token = self.get_token().await?;
        self.file_client().get_file_meta(&token, fs_id).await
    }

    async fn search_files(&self, key: &str, dir: &str) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let token = self.get_token().await?;
        self.file_client().search_files(&token, key, dir).await
    }

    async fn search_files_with_options(&self, key: &str, options: SearchOptions) -> NetDiskResult<(Vec<FileInfo>, bool)> {
        let token = self.get_token().await?;
        self.file_client().search_files_with_options(&token, key, options).await
    }

    async fn semantic_search(&self, query: &str) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().semantic_search(&token, query).await
    }

    async fn semantic_search_with_options(&self, query: &str, options: SemanticSearchOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().semantic_search_with_options(&token, query, options).await
    }

    async fn get_category_file_count(&self, category: u32) -> NetDiskResult<u64> {
        let token = self.get_token().await?;
        self.file_client().get_category_file_count(&token, category).await
    }

    async fn get_category_file_count_with_options(&self, category: u32, options: CategoryCountOptions) -> NetDiskResult<u64> {
        let token = self.get_token().await?;
        self.file_client().get_category_file_count_with_options(&token, category, options).await
    }

    async fn search_category_files(&self, category: u32, start: i32, limit: i32) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let token = self.get_token().await?;
        self.file_client().search_category_files(&token, category, start, limit).await
    }

    async fn search_category_files_with_options(&self, category: &str, options: CategorySearchOptions) -> NetDiskResult<(Vec<FileInfo>, u64)> {
        let token = self.get_token().await?;
        self.file_client().search_category_files_with_options(&token, category, options).await
    }

    async fn list_documents(&self, parent_path: &str, page: i32, num: i32) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_documents(&token, parent_path, page, num).await
    }

    async fn list_documents_with_options(&self, options: DocumentListOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_documents_with_options(&token, options).await
    }

    async fn list_images(&self, parent_path: &str, page: i32, num: i32) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_images(&token, parent_path, page, num).await
    }

    async fn list_images_with_options(&self, options: ImageListOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_images_with_options(&token, options).await
    }

    async fn list_videos(&self, parent_path: &str, page: i32, num: i32) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_videos(&token, parent_path, page, num).await
    }

    async fn list_videos_with_options(&self, options: VideoListOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_videos_with_options(&token, options).await
    }

    async fn list_torrents(&self, parent_path: &str, page: i32, num: i32) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_torrents(&token, parent_path, page, num).await
    }

    async fn list_torrents_with_options(&self, options: BtListOptions) -> NetDiskResult<Vec<FileInfo>> {
        let token = self.get_token().await?;
        self.file_client().list_torrents_with_options(&token, options).await
    }

    async fn create_folder(&self, path: &str) -> NetDiskResult<FolderInfo> {
        let token = self.get_token().await?;
        self.file_client().create_folder(&token, path).await
    }

    async fn create_folder_with_options(&self, path: &str, options: FolderCreateOptions) -> NetDiskResult<FolderInfo> {
        let token = self.get_token().await?;
        self.file_client().create_folder_with_options(&token, path, options).await
    }

    async fn delete_file(&self, path: &str) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.file_client().delete(&token, path).await
    }

    async fn rename_file(&self, path: &str, new_name: &str) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.file_client().rename(&token, path, new_name).await
    }

    async fn move_file(&self, path: &str, dest: &str) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.file_client().move_file(&token, path, dest).await
    }

    async fn copy_file(&self, path: &str, dest: &str) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.file_client().copy_file(&token, path, dest).await
    }

    async fn upload_file<P: AsRef<std::path::Path> + Send>(&self, local_path: P, remote_path: &str) -> NetDiskResult<CreateFileResponse> {
        let token = self.get_token().await?;
        self.upload_client().upload_file(&token, local_path, remote_path).await
    }

    async fn upload_file_with_options<P: AsRef<std::path::Path> + Send>(
        &self,
        local_path: P,
        remote_path: &str,
        options: SimpleUploadOptions,
    ) -> NetDiskResult<CreateFileResponse> {
        let token = self.get_token().await?;
        self.upload_client().upload_file_with_options(&token, local_path, remote_path, options).await
    }

    async fn upload_bytes(&self, data: &[u8], remote_path: &str) -> NetDiskResult<CreateFileResponse> {
        let token = self.get_token().await?;
        self.upload_client().upload_bytes(&token, data, remote_path).await
    }

    async fn auto_download(&self, path: &str, save_path: impl AsRef<Path> + Send) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().auto_download(&token, path, save_path).await
    }

    async fn auto_download_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().auto_download_by_fsid(&token, fs_id, save_path).await
    }

    async fn download_single(&self, path: &str, save_path: impl AsRef<Path> + Send) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_single(&token, path, save_path).await
    }

    async fn download_parallel(&self, path: &str, save_path: impl AsRef<Path> + Send, thread_num: Option<usize>) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_parallel(&token, path, save_path, thread_num).await
    }

    async fn download_streaming(&self, path: &str, save_path: impl AsRef<Path> + Send, max_concurrency: usize) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_streaming(&token, path, save_path, max_concurrency).await
    }

    async fn upload_bytes_with_options(&self, data: &[u8], remote_path: &str, options: SimpleUploadOptions) -> NetDiskResult<CreateFileResponse> {
        let token = self.get_token().await?;
        self.upload_client().upload_bytes_with_options(&token, data, remote_path, options).await
    }

    async fn download_single_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_single_by_fsid(&token, fs_id, save_path).await
    }

    async fn download_parallel_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send, thread_num: Option<usize>) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_parallel_by_fsid(&token, fs_id, save_path, thread_num).await
    }

    async fn download_streaming_by_fsid(&self, fs_id: u64, save_path: impl AsRef<Path> + Send, max_concurrency: usize) -> NetDiskResult<()> {
        let token = self.get_token().await?;
        self.download_client().download_streaming_by_fsid(&token, fs_id, save_path, max_concurrency).await
    }

    async fn get_dlink_from_path(&self, path: &str) -> NetDiskResult<FileMeta> {
        let token = self.get_token().await?;
        self.download_client().get_dlink_from_path(&token, path).await
    }

    async fn get_dlink_from_fsid(&self, fs_id: u64) -> NetDiskResult<FileMeta> {
        let token = self.get_token().await?;
        self.download_client().get_dlink_from_fsid(&token, fs_id).await
    }

    async fn get_playlist_list(&self) -> NetDiskResult<PlaylistList> {
        let token = self.get_token().await?;
        self.playlist_client().get_playlist_list(&token).await
    }

    async fn get_playlist_list_with_options(&self, options: crate::playlist::PlaylistListOptions) -> NetDiskResult<PlaylistList> {
        let token = self.get_token().await?;
        self.playlist_client().get_playlist_list_with_options(&token, options).await
    }

    async fn get_playlist_file_list(&self, mb_id: u64) -> NetDiskResult<PlaylistFileList> {
        let token = self.get_token().await?;
        self.playlist_client().get_playlist_file_list(&token, mb_id).await
    }

    async fn get_playlist_file_list_with_options(&self, mb_id: u64, options: crate::playlist::PlaylistFileListOptions) -> NetDiskResult<PlaylistFileList> {
        let token = self.get_token().await?;
        self.playlist_client().get_playlist_file_list_with_options(&token, mb_id, options).await
    }

    async fn get_media_play_info(&self, fsid: Option<u64>, path: Option<&str>, media_type: &str) -> NetDiskResult<MediaPlayInfo> {
        let token = self.get_token().await?;
        self.playlist_client().get_media_play_info(&token, fsid, path, media_type).await
    }

    async fn get_media_m3u8_content(&self, path: &str, media_type: &str) -> NetDiskResult<String> {
        let token = self.get_token().await?;
        self.playlist_client().get_media_m3u8_content(&token, path, media_type).await
    }

    async fn get_video_m3u8(&self, path: &str, quality: VideoQuality) -> NetDiskResult<String> {
        let token = self.get_token().await?;
        self.playlist_client().get_video_m3u8(&token, path, quality).await
    }

    async fn get_video_m3u8_highest(&self, path: &str, vip_level: u32) -> NetDiskResult<String> {
        let token = self.get_token().await?;
        self.playlist_client().get_video_m3u8_highest(&token, path, vip_level).await
    }

    async fn get_audio_m3u8(&self, path: &str, quality: AudioQuality) -> NetDiskResult<String> {
        let token = self.get_token().await?;
        self.playlist_client().get_audio_m3u8(&token, path, quality).await
    }

    async fn get_audio_m3u8_default(&self, path: &str) -> NetDiskResult<String> {
        let token = self.get_token().await?;
        self.playlist_client().get_audio_m3u8_default(&token, path).await
    }
}

#[derive(Debug, Clone)]
pub struct BaiduNetDiskClient {
    token_provider: TokenProvider,
    authorization: Authorization,
    user_client: Arc<UserClient>,
    quota_client: Arc<QuotaClient>,
    file_client: Arc<FileClient>,
    download_client: Arc<DownloadClient>,
    upload_client: Arc<UploadClient>,
    playlist_client: Arc<PlaylistClient>,
    config: ClientConfig,
}

impl BaiduNetDiskClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub fn authorize(&self) -> &Authorization {
        &self.authorization
    }

    pub fn token_provider(&self) -> &TokenProvider {
        &self.token_provider
    }

    pub fn user(&self) -> &UserClient {
        &self.user_client
    }

    pub fn quota(&self) -> &QuotaClient {
        &self.quota_client
    }

    pub fn file(&self) -> &FileClient {
        &self.file_client
    }

    pub fn download(&self) -> &DownloadClient {
        &self.download_client
    }

    pub fn upload(&self) -> &UploadClient {
        &self.upload_client
    }

    pub fn playlist(&self) -> &PlaylistClient {
        &self.playlist_client
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    pub async fn get_valid_token(&self) -> NetDiskResult<AccessToken> {
        self.token_provider.get_valid_token().await
    }

    pub fn set_access_token(&self, token: AccessToken) -> NetDiskResult<()> {
        self.token_provider.set_access_token(token)
    }

    pub fn load_token_from_env(&self) -> NetDiskResult<AccessToken> {
        dotenv().ok();

        let access_token = std::env::var("BD_NETDISK_ACCESS_TOKEN").map_err(|_| {
            NetDiskError::auth_error("BD_NETDISK_ACCESS_TOKEN environment variable not set")
        })?;

        let refresh_token = std::env::var("BD_NETDISK_REFRESH_TOKEN").map_err(|_| {
            NetDiskError::auth_error("BD_NETDISK_REFRESH_TOKEN environment variable not set")
        })?;

        let expires_in: u64 = std::env::var("BD_NETDISK_EXPIRES_IN")
            .map_err(|_| {
                NetDiskError::auth_error("BD_NETDISK_EXPIRES_IN environment variable not set")
            })?
            .parse()
            .map_err(|_| {
                NetDiskError::auth_error("BD_NETDISK_EXPIRES_IN must be a valid number")
            })?;

        let scope =
            std::env::var("BD_NETDISK_SCOPE").unwrap_or_else(|_| "basic netdisk".to_string());
        let session_key = std::env::var("BD_NETDISK_SESSION_KEY").unwrap_or_default();
        let session_secret = std::env::var("BD_NETDISK_SESSION_SECRET").unwrap_or_default();

        let acquired_at = if let Ok(ts_str) = std::env::var("BD_NETDISK_ACQUIRED_AT") {
            ts_str.parse().unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        };

        let token = AccessToken {
            access_token,
            expires_in,
            refresh_token,
            scope,
            session_key,
            session_secret,
            acquired_at,
        };

        let token_status = token.validate();
        match token_status {
            TokenStatus::Valid => {
                self.set_access_token(token.clone())?;
                info!(
                    "Access token loaded from environment variables (valid for {} seconds)",
                    token.remaining_seconds()
                );
            }
            TokenStatus::ExpiringSoon => {
                self.set_access_token(token.clone())?;
                warn!("Access token loaded from environment variables but will expire soon ({} seconds remaining)", token.remaining_seconds());
            }
            TokenStatus::Expired => {
                self.set_access_token(token.clone())?;
                error!("Access token loaded from environment variables but is already expired! Please re-authenticate.");
            }
        }

        debug!(
            "Token details: scope={}, expires_at={}",
            token.scope,
            token.expires_at()
        );

        Ok(token)
    }

    pub fn validate_token(&self) -> NetDiskResult<TokenStatus> {
        self.token_provider.validate_token()
    }

    pub fn with_token(&self, token: AccessToken) -> TokenScopedClient {
        TokenScopedClient::new(
            Arc::new(token),
            Arc::clone(&self.user_client),
            Arc::clone(&self.quota_client),
            Arc::clone(&self.file_client),
            Arc::clone(&self.download_client),
            Arc::clone(&self.upload_client),
            Arc::clone(&self.playlist_client),
        )
    }
}

impl ClientAccessor for BaiduNetDiskClient {
    fn get_token(&self) -> impl Future<Output = NetDiskResult<AccessToken>> + Send + '_ {
        self.get_valid_token()
    }

    fn user_client(&self) -> &UserClient {
        &self.user_client
    }

    fn quota_client(&self) -> &QuotaClient {
        &self.quota_client
    }

    fn file_client(&self) -> &FileClient {
        &self.file_client
    }

    fn download_client(&self) -> &DownloadClient {
        &self.download_client
    }

    fn upload_client(&self) -> &UploadClient {
        &self.upload_client
    }

    fn playlist_client(&self) -> &PlaylistClient {
        &self.playlist_client
    }
}

#[derive(Debug, Clone)]
pub struct TokenScopedClient {
    token: Arc<AccessToken>,
    user_client: Arc<UserClient>,
    quota_client: Arc<QuotaClient>,
    file_client: Arc<FileClient>,
    download_client: Arc<DownloadClient>,
    upload_client: Arc<UploadClient>,
    playlist_client: Arc<PlaylistClient>,
}

impl TokenScopedClient {
    pub fn new(
        token: Arc<AccessToken>,
        user_client: Arc<UserClient>,
        quota_client: Arc<QuotaClient>,
        file_client: Arc<FileClient>,
        download_client: Arc<DownloadClient>,
        upload_client: Arc<UploadClient>,
        playlist_client: Arc<PlaylistClient>,
    ) -> Self {
        TokenScopedClient {
            token,
            user_client,
            quota_client,
            file_client,
            download_client,
            upload_client,
            playlist_client,
        }
    }

    pub fn token(&self) -> &AccessToken {
        &self.token
    }

    pub fn user(&self) -> &UserClient {
        &self.user_client
    }

    pub fn quota(&self) -> &QuotaClient {
        &self.quota_client
    }

    pub fn file(&self) -> &FileClient {
        &self.file_client
    }

    pub fn download(&self) -> &DownloadClient {
        &self.download_client
    }

    pub fn upload(&self) -> &UploadClient {
        &self.upload_client
    }

    pub fn playlist(&self) -> &PlaylistClient {
        &self.playlist_client
    }
}

impl ClientAccessor for TokenScopedClient {
    fn get_token(&self) -> impl Future<Output = NetDiskResult<AccessToken>> + Send + '_ {
        std::future::ready(Ok((*self.token).clone()))
    }

    fn user_client(&self) -> &UserClient {
        &self.user_client
    }

    fn quota_client(&self) -> &QuotaClient {
        &self.quota_client
    }

    fn file_client(&self) -> &FileClient {
        &self.file_client
    }

    fn download_client(&self) -> &DownloadClient {
        &self.download_client
    }

    fn upload_client(&self) -> &UploadClient {
        &self.upload_client
    }

    fn playlist_client(&self) -> &PlaylistClient {
        &self.playlist_client
    }
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub app_id: String,
    pub app_key: String,
    pub app_secret: String,
    pub app_name: String,
    pub scope: String,
    pub http_config: HttpClientConfig,
    pub token_config: TokenProviderConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let _ = dotenv();

        ClientConfig {
            app_id: std::env::var("BD_NETDISK_APP_ID").unwrap_or_default(),
            app_key: std::env::var("BD_NETDISK_APP_KEY").unwrap_or_default(),
            app_secret: std::env::var("BD_NETDISK_SECRET_KEY").unwrap_or_default(),
            app_name: std::env::var("BD_NETDISK_APP_NAME").unwrap_or_default(),
            scope: "basic,netdisk".to_string(),
            http_config: HttpClientConfig::default(),
            token_config: TokenProviderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn app_id(mut self, app_id: &str) -> Self {
        self.config.app_id = app_id.to_string();
        self
    }

    pub fn app_key(mut self, app_key: &str) -> Self {
        self.config.app_key = app_key.to_string();
        self
    }

    pub fn app_secret(mut self, app_secret: &str) -> Self {
        self.config.app_secret = app_secret.to_string();
        self
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.config.app_name = app_name.to_string();
        self
    }

    pub fn scope(mut self, scope: &str) -> Self {
        self.config.scope = scope.to_string();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.http_config.timeout = timeout;
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.http_config.connect_timeout = timeout;
        self
    }

    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.config.http_config.max_retries = max_retries;
        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.config.http_config.user_agent = user_agent.to_string();
        self
    }

    pub fn auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.config.token_config.auto_refresh = auto_refresh;
        self
    }

    pub fn refresh_ahead_seconds(mut self, seconds: u64) -> Self {
        self.config.token_config.refresh_ahead_seconds = seconds;
        self
    }

    pub fn build(self) -> NetDiskResult<BaiduNetDiskClient> {
        if self.config.app_key.is_empty() {
            return Err(NetDiskError::invalid_parameter("app_key is required"));
        }

        if self.config.app_secret.is_empty() {
            return Err(NetDiskError::invalid_parameter("app_secret is required"));
        }

        debug!("Building BaiduNetDiskClient with config: {:?}", self.config);

        let http_client = HttpClient::new(self.config.http_config.clone())?;

        let authorization = Authorization::new(
            http_client.clone(),
            &self.config.app_key,
            &self.config.app_secret,
            &self.config.scope,
        );

        let token_provider = TokenProvider::new(
            http_client.clone(),
            &self.config.app_key,
            &self.config.app_secret,
            self.config.token_config.clone(),
        );

        info!("BaiduNetDiskClient created successfully");

        let user_client = Arc::new(UserClient::new(http_client.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone()));
        let download_client = Arc::new(DownloadClient::new(file_client.clone()));
        let upload_client = Arc::new(UploadClient::new(http_client.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(http_client.clone()));

        Ok(BaiduNetDiskClient {
            token_provider,
            authorization,
            user_client,
            quota_client,
            file_client,
            download_client,
            upload_client,
            playlist_client,
            config: self.config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_client_builder() {
        let client = BaiduNetDiskClient::builder()
            .app_key("test_app_key")
            .app_secret("test_app_secret")
            .timeout(Duration::from_secs(30))
            .max_retries(3)
            .auto_refresh(true)
            .build();

        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_builder_missing_app_key() {
        let client = BaiduNetDiskClient::builder()
            .app_key("")
            .app_secret("test_app_secret")
            .build();

        assert!(client.is_err());
        assert!(matches!(
            client.err(),
            Some(NetDiskError::InvalidParameter { .. })
        ));
    }

    #[tokio::test]
    async fn test_client_builder_missing_app_secret() {
        let client = BaiduNetDiskClient::builder()
            .app_key("test_app_key")
            .app_secret("")
            .build();

        assert!(client.is_err());
        assert!(matches!(
            client.err(),
            Some(NetDiskError::InvalidParameter { .. })
        ));
    }

    #[tokio::test]
    async fn test_client_builder_with_all_options() {
        let client = BaiduNetDiskClient::builder()
            .app_id("test_app_id")
            .app_key("test_app_key")
            .app_secret("test_app_secret")
            .app_name("Test App")
            .scope("basic,netdisk")
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .max_retries(5)
            .user_agent("TestAgent/1.0")
            .auto_refresh(true)
            .refresh_ahead_seconds(86400)
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        
        assert_eq!(client.config().app_id, "test_app_id");
        assert_eq!(client.config().app_key, "test_app_key");
        assert_eq!(client.config().app_secret, "test_app_secret");
        assert_eq!(client.config().app_name, "Test App");
        assert_eq!(client.config().scope, "basic,netdisk");
        assert_eq!(client.config().http_config.timeout, Duration::from_secs(60));
        assert_eq!(client.config().http_config.connect_timeout, Duration::from_secs(10));
        assert_eq!(client.config().http_config.max_retries, 5);
        assert_eq!(client.config().http_config.user_agent, "TestAgent/1.0");
        assert!(client.config().token_config.auto_refresh);
        assert_eq!(client.config().token_config.refresh_ahead_seconds, 86400);
    }

    #[tokio::test]
    async fn test_client_accessors() {
        let client = BaiduNetDiskClient::builder()
            .app_key("test_app_key")
            .app_secret("test_app_secret")
            .build()
            .unwrap();

        let _ = client.authorize();
        let _ = client.token_provider();
        let _ = client.user();
        let _ = client.quota();
        let _ = client.file();
        let _ = client.download();
        let _ = client.upload();
        let _ = client.playlist();
        let _ = client.config();
    }

    #[tokio::test]
    async fn test_token_scoped_client_new() {
        let token = AccessToken {
            access_token: "test_access_token".to_string(),
            expires_in: 3600,
            refresh_token: "test_refresh_token".to_string(),
            scope: "basic netdisk".to_string(),
            session_key: "".to_string(),
            session_secret: "".to_string(),
            acquired_at: 0,
        };

        let http_client = HttpClient::new(HttpClientConfig::default()).unwrap();
        let user_client = Arc::new(UserClient::new(http_client.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone()));
        let download_client = Arc::new(DownloadClient::new(file_client.clone()));
        let upload_client = Arc::new(UploadClient::new(http_client.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(http_client.clone()));

        let scoped_client = TokenScopedClient::new(
            Arc::new(token.clone()),
            user_client.clone(),
            quota_client.clone(),
            file_client.clone(),
            download_client.clone(),
            upload_client.clone(),
            playlist_client.clone(),
        );

        assert_eq!(scoped_client.token().access_token, token.access_token);
        assert_eq!(scoped_client.token().refresh_token, token.refresh_token);
        assert_eq!(scoped_client.token().expires_in, token.expires_in);
    }

    #[tokio::test]
    async fn test_token_scoped_client_from_client() {
        let client = BaiduNetDiskClient::builder()
            .app_key("test_app_key")
            .app_secret("test_app_secret")
            .build()
            .unwrap();

        let token = AccessToken {
            access_token: "test_access_token".to_string(),
            expires_in: 3600,
            refresh_token: "test_refresh_token".to_string(),
            scope: "basic netdisk".to_string(),
            session_key: "".to_string(),
            session_secret: "".to_string(),
            acquired_at: 0,
        };

        let scoped_client = client.with_token(token.clone());

        assert_eq!(scoped_client.token().access_token, token.access_token);
        
        let _ = scoped_client.user();
        let _ = scoped_client.quota();
        let _ = scoped_client.file();
        let _ = scoped_client.download();
        let _ = scoped_client.upload();
        let _ = scoped_client.playlist();
    }

    #[tokio::test]
    async fn test_token_scoped_client_get_token() {
        let token = AccessToken {
            access_token: "test_access_token".to_string(),
            expires_in: 3600,
            refresh_token: "test_refresh_token".to_string(),
            scope: "basic netdisk".to_string(),
            session_key: "".to_string(),
            session_secret: "".to_string(),
            acquired_at: 0,
        };

        let http_client = HttpClient::new(HttpClientConfig::default()).unwrap();
        let user_client = Arc::new(UserClient::new(http_client.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone()));
        let download_client = Arc::new(DownloadClient::new(file_client.clone()));
        let upload_client = Arc::new(UploadClient::new(http_client.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(http_client.clone()));

        let scoped_client = TokenScopedClient::new(
            Arc::new(token.clone()),
            user_client,
            quota_client,
            file_client,
            download_client,
            upload_client,
            playlist_client,
        );

        let retrieved_token = scoped_client.get_token().await.unwrap();
        assert_eq!(retrieved_token.access_token, token.access_token);
        assert_eq!(retrieved_token.refresh_token, token.refresh_token);
    }

    #[tokio::test]
    async fn test_token_scoped_client_independence() {
        let client = BaiduNetDiskClient::builder()
            .app_key("test_app_key")
            .app_secret("test_app_secret")
            .build()
            .unwrap();

        let token1 = AccessToken {
            access_token: "token1".to_string(),
            expires_in: 3600,
            refresh_token: "refresh1".to_string(),
            scope: "basic netdisk".to_string(),
            session_key: "".to_string(),
            session_secret: "".to_string(),
            acquired_at: 0,
        };

        let token2 = AccessToken {
            access_token: "token2".to_string(),
            expires_in: 7200,
            refresh_token: "refresh2".to_string(),
            scope: "basic netdisk".to_string(),
            session_key: "".to_string(),
            session_secret: "".to_string(),
            acquired_at: 0,
        };

        let scoped_client1 = client.with_token(token1.clone());
        let scoped_client2 = client.with_token(token2.clone());

        assert_eq!(scoped_client1.token().access_token, "token1");
        assert_eq!(scoped_client2.token().access_token, "token2");
        assert_ne!(scoped_client1.token().access_token, scoped_client2.token().access_token);
    }

    }