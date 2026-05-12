//! Playlist and media playback functionality
//!
//! This module provides access to Baidu NetDisk's playlist and media streaming features:
//!
//! # Features
//!
//! - **Playlist management**: List playlists and their contents
//! - **Media streaming**: Get media playback information and m3u8 streams
//! - **Quality selection**: Video/Audio quality enums with VIP level support
//! - **Transcoding check**: Verify if media is fully transcoded
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::{BaiduNetDiskClient, playlist::VideoQuality};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client and load token
//! let client = BaiduNetDiskClient::builder().build()?;
//! let token = client.load_token_from_env()?;
//!
//! // List all playlists
//! let playlists = client.playlist().get_playlist_list(&token).await?;
//!
//! // Get media m3u8 with highest quality for VIP 2
//! let m3u8 = client.playlist()
//!     .get_video_m3u8_highest(&token, "/video.mp4", 2)
//!     .await?;
//! # Ok(())
//! # }
//! ```

use log::{debug, error, info};
use serde::Deserialize;

use crate::auth::AccessToken;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// Video quality levels for m3u8 streaming
///
/// # Examples
///
/// ```
/// use baidu_netdisk_sdk::playlist::VideoQuality;
///
/// let quality = VideoQuality::Quality1080P;
/// assert_eq!(quality.to_media_type(), "M3U8_AUTO_1080");
///
/// // Get highest quality for VIP level
/// let quality = VideoQuality::highest_for_vip_level(2);
/// assert_eq!(quality, VideoQuality::Quality1080P);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoQuality {
    /// 480P quality - available to all users
    Quality480P,
    /// 720P quality - highest for regular users
    Quality720P,
    /// 1080P quality - usually requires super VIP
    Quality1080P,
}

impl VideoQuality {
    /// Get the corresponding media_type string for API
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::playlist::VideoQuality;
    ///
    /// assert_eq!(VideoQuality::Quality480P.to_media_type(), "M3U8_AUTO_480");
    /// assert_eq!(VideoQuality::Quality720P.to_media_type(), "M3U8_AUTO_720");
    /// assert_eq!(VideoQuality::Quality1080P.to_media_type(), "M3U8_AUTO_1080");
    /// ```
    pub fn to_media_type(self) -> &'static str {
        match self {
            VideoQuality::Quality480P => "M3U8_AUTO_480",
            VideoQuality::Quality720P => "M3U8_AUTO_720",
            VideoQuality::Quality1080P => "M3U8_AUTO_1080",
        }
    }

    /// Get the highest quality available for a given VIP level
    ///
    /// - **VIP 0-1**: Max 480P
    /// - **VIP 2+**: Max 1080P (includes 720P)
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::playlist::VideoQuality;
    ///
    /// assert_eq!(VideoQuality::highest_for_vip_level(0), VideoQuality::Quality480P);
    /// assert_eq!(VideoQuality::highest_for_vip_level(2), VideoQuality::Quality1080P);
    /// ```
    pub fn highest_for_vip_level(vip_level: u32) -> Self {
        match vip_level {
            0 | 1 => VideoQuality::Quality480P,
            _ => VideoQuality::Quality1080P,
        }
    }

    /// Get all qualities available for a given VIP level (from lowest to highest)
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::playlist::VideoQuality;
    ///
    /// let qualities = VideoQuality::available_for_vip_level(2);
    /// assert_eq!(qualities.len(), 3);
    /// assert_eq!(qualities[0], VideoQuality::Quality480P);
    /// assert_eq!(qualities[2], VideoQuality::Quality1080P);
    /// ```
    pub fn available_for_vip_level(vip_level: u32) -> Vec<Self> {
        match vip_level {
            0 | 1 => vec![VideoQuality::Quality480P],
            _ => vec![
                VideoQuality::Quality480P,
                VideoQuality::Quality720P,
                VideoQuality::Quality1080P,
            ],
        }
    }
}

/// Audio quality levels for m3u8 streaming
///
/// # Examples
///
/// ```
/// use baidu_netdisk_sdk::playlist::AudioQuality;
///
/// let quality = AudioQuality::Quality128K;
/// assert_eq!(quality.to_media_type(), "M3U8_MP3_128");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioQuality {
    /// 128kbps MP3
    Quality128K,
}

impl AudioQuality {
    /// Get the corresponding media_type string for API
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::playlist::AudioQuality;
    ///
    /// assert_eq!(AudioQuality::Quality128K.to_media_type(), "M3U8_MP3_128");
    /// ```
    pub fn to_media_type(self) -> &'static str {
        match self {
            AudioQuality::Quality128K => "M3U8_MP3_128",
        }
    }
}

/// Playlist client for interacting with playlist-related APIs
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
/// // Access playlist functionality
/// let playlists = client.playlist().get_playlist_list(&token).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PlaylistClient {
    http_client: HttpClient,
    app_id: Option<String>,
}

impl PlaylistClient {
    /// Create a new PlaylistClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::playlist()` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::HttpClient;
    /// use baidu_netdisk_sdk::playlist::PlaylistClient;
    ///
    /// let http_client = HttpClient::new();
    /// let playlist_client = PlaylistClient::new(http_client);
    /// ```
    pub fn new(http_client: HttpClient) -> Self {
        PlaylistClient {
            http_client,
            app_id: None,
        }
    }

    /// Create a new PlaylistClient instance with app_id
    ///
    /// Some API endpoints may require an app_id for authentication.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::HttpClient;
    /// use baidu_netdisk_sdk::playlist::PlaylistClient;
    ///
    /// let http_client = HttpClient::new();
    /// let playlist_client = PlaylistClient::new_with_app_id(http_client, "123456".to_string());
    /// ```
    pub fn new_with_app_id(http_client: HttpClient, app_id: String) -> Self {
        PlaylistClient {
            http_client,
            app_id: Some(app_id),
        }
    }

    /// Set app_id for API authentication
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::HttpClient;
    /// use baidu_netdisk_sdk::playlist::PlaylistClient;
    ///
    /// let http_client = HttpClient::new();
    /// let mut playlist_client = PlaylistClient::new(http_client);
    /// playlist_client.set_app_id("123456".to_string());
    /// ```
    pub fn set_app_id(&mut self, app_id: String) {
        self.app_id = Some(app_id);
    }

    /// Get a list of playlists with default options
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
    /// let playlists = client.playlist().get_playlist_list(&token).await?;
    /// println!("Found {} playlists", playlists.list.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_playlist_list(
        &self,
        access_token: &AccessToken,
    ) -> NetDiskResult<PlaylistList> {
        self.get_playlist_list_with_options(access_token, PlaylistListOptions::default())
            .await
    }

    /// Get a list of playlists with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `get_playlist_list()` instead.
    pub async fn get_playlist_list_with_options(
        &self,
        access_token: &AccessToken,
        options: PlaylistListOptions,
    ) -> NetDiskResult<PlaylistList> {
        let mut params = Vec::new();

        params.push(("method", "list".to_string()));
        params.push(("access_token", access_token.access_token.clone()));

        if let Some(p) = options.page {
            params.push(("page", p.to_string()));
        }
        if let Some(s) = options.psize {
            params.push(("psize", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        debug!("Getting playlist list with options: {:?}", options);

        let response: PlaylistListResponse = self
            .http_client
            .get("/rest/2.0/xpan/broadcast/list", Some(&params_ref))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        info!(
            "Playlist list retrieved successfully, count: {}",
            list.len()
        );

        Ok(PlaylistList {
            has_more: response.has_more,
            list,
        })
    }

    /// Get playlist file download list with default options
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
    /// // First get playlists to find mb_id
    /// let playlists = client.playlist().get_playlist_list(&token).await?;
    /// if let Some(playlist) = playlists.list.first() {
    ///     // Then get files in playlist
    ///     let files = client.playlist()
    ///         .get_playlist_file_list(&token, playlist.mb_id)
    ///         .await?;
    ///     println!("Found {} files", files.list.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_playlist_file_list(
        &self,
        access_token: &AccessToken,
        mb_id: u64,
    ) -> NetDiskResult<PlaylistFileList> {
        self.get_playlist_file_list_with_options(
            access_token,
            mb_id,
            PlaylistFileListOptions::default(),
        )
        .await
    }

    /// Get playlist file download list with custom options
    ///
    /// This is a lower-level method for advanced use cases.
    /// Most users should use `get_playlist_file_list()` instead.
    pub async fn get_playlist_file_list_with_options(
        &self,
        access_token: &AccessToken,
        mb_id: u64,
        options: PlaylistFileListOptions,
    ) -> NetDiskResult<PlaylistFileList> {
        let mut params = Vec::new();

        params.push(("access_token", access_token.access_token.clone()));
        params.push(("mb_id", mb_id.to_string()));

        if let Some(s) = options.showmeta {
            params.push(("showmeta", s.to_string()));
        }
        if let Some(p) = options.page {
            params.push(("page", p.to_string()));
        }
        if let Some(s) = options.psize {
            params.push(("psize", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        debug!(
            "Getting playlist file list for mb_id: {} with options: {:?}",
            mb_id, options
        );

        let response: PlaylistFileListResponse = self
            .http_client
            .post("/rest/2.0/xpan/broadcast/filelist", Some(&params_ref))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        info!(
            "Playlist file list retrieved successfully, count: {}",
            list.len()
        );

        Ok(PlaylistFileList {
            has_more: response.has_more,
            list,
        })
    }

    /// Get media playback information (audio or video)
    ///
    /// Takes either fs_id or path (one of them is required)
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
    /// // Get by path
    /// let info = client.playlist()
    ///     .get_media_play_info(&token, None, Some("/video.mp4"), "M3U8_AUTO_1080")
    ///     .await?;
    ///
    /// // Or get by fs_id
    /// let info = client.playlist()
    ///     .get_media_play_info(&token, Some(123456), None, "M3U8_AUTO_1080")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_media_play_info(
        &self,
        access_token: &AccessToken,
        fsid: Option<u64>,
        path: Option<&str>,
        media_type: &str,
    ) -> NetDiskResult<MediaPlayInfo> {
        let mut params = Vec::new();

        params.push(("method", "streaming".to_string()));
        params.push(("access_token", access_token.access_token.clone()));
        params.push(("type", media_type.to_string()));

        if let Some(f) = fsid {
            params.push(("fid", f.to_string()));
        }
        if let Some(p) = path {
            params.push(("path", p.to_string()));
        }

        if let Some(app_id) = &self.app_id {
            params.push(("app_id", app_id.clone()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let headers = [
            (
                "User-Agent",
                "xpanvideo;netdisk;iPhone13;ios-iphone;15.1;ts",
            ),
            ("Host", "pan.baidu.com"),
            ("Accept", "*/*"),
            ("Accept-Language", "zh-CN,zh;q=0.9"),
        ];

        debug!("Getting media play info with params: {:?}", params_ref);

        let response: MediaPlayInfoResponse = self
            .http_client
            .get_with_headers("/rest/2.0/xpan/file", Some(&params_ref), Some(&headers))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        let list = response.list.unwrap_or_default();
        info!("Media play info retrieved successfully");

        Ok(MediaPlayInfo {
            list,
            request_id: response.request_id,
        })
    }

    /// Fetch m3u8 playlist content with special headers
    ///
    /// This is for checking if the media is fully transcoded
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    ///
    /// // First get play info to get m3u8_url
    /// // Then fetch the content
    /// // let content = client.playlist().fetch_m3u8(m3u8_url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_m3u8(&self, m3u8_url: &str) -> NetDiskResult<String> {
        debug!("Fetching m3u8 content from: {}", m3u8_url);

        let client = reqwest::Client::new();
        let response = client
            .get(m3u8_url)
            .header(
                "User-Agent",
                "xpanvideo;netdisk;iPhone13;ios-iphone;15.1;ts",
            )
            .header("Host", "pan.baidu.com")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NetDiskError::Unknown {
                message: format!("Failed to fetch m3u8: {}", response.status()),
            });
        }

        let content = response.text().await?;
        debug!(
            "Successfully fetched m3u8 content, length: {}",
            content.len()
        );

        Ok(content)
    }

    /// Check if media is fully transcoded by checking if m3u8 contains #EXT-X-ENDLIST
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    ///
    /// // Check if transcoding is complete
    /// // let is_complete = client.playlist().is_media_fully_transcoded(m3u8_url).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_media_fully_transcoded(&self, m3u8_url: &str) -> NetDiskResult<bool> {
        let content = self.fetch_m3u8(m3u8_url).await?;
        Ok(content.contains("#EXT-X-ENDLIST"))
    }

    /// Get raw m3u8 content for media file by path
    ///
    /// This returns the raw m3u8 playlist content. It does NOT poll for completion.
    /// Use `is_media_fully_transcoded` or implement your own polling if needed.
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
    /// let m3u8 = client.playlist()
    ///     .get_media_m3u8_content(&token, "/video.mp4", "M3U8_AUTO_1080")
    ///     .await?;
    ///
    /// // Check if fully transcoded
    /// let is_complete = m3u8.contains("#EXT-X-ENDLIST");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_media_m3u8_content(
        &self,
        access_token: &AccessToken,
        path: &str,
        media_type: &str,
    ) -> NetDiskResult<String> {
        let mut params = vec![
            ("method", "streaming".to_string()),
            ("access_token", access_token.access_token.clone()),
            ("path", path.to_string()),
            ("type", media_type.to_string()),
        ];

        if let Some(app_id) = &self.app_id {
            params.push(("app_id", app_id.clone()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        debug!(
            "Getting raw m3u8 content for path: {}, type: {}",
            path, media_type
        );

        let mut url = reqwest::Url::parse("https://pan.baidu.com/rest/2.0/xpan/file")?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in &params_ref {
                pairs.append_pair(key, value);
            }
        }

        debug!("Built URL for m3u8: {}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(url.clone())
            .header(
                "User-Agent",
                "xpanvideo;netdisk;iPhone13;ios-iphone;15.1;ts",
            )
            .header("Host", "pan.baidu.com")
            .header("Accept", "*/*")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to get m3u8 content: {} - {}", status, body);
            return Err(NetDiskError::http_error(status.as_u16(), url.as_ref()));
        }

        let content = response.text().await?;
        debug!(
            "Successfully fetched m3u8 content, length: {}",
            content.len()
        );

        Ok(content)
    }

    // Convenience methods using quality enums

    /// Get video m3u8 content with specified quality
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    /// use baidu_netdisk_sdk::playlist::VideoQuality;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let m3u8 = client.playlist()
    ///     .get_video_m3u8(&token, "/video.mp4", VideoQuality::Quality1080P)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_video_m3u8(
        &self,
        access_token: &AccessToken,
        path: &str,
        quality: VideoQuality,
    ) -> NetDiskResult<String> {
        self.get_media_m3u8_content(access_token, path, quality.to_media_type())
            .await
    }

    /// Get video m3u8 content with highest available quality for given VIP level
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
    /// // Get highest quality for VIP 2 (1080P)
    /// let m3u8 = client.playlist()
    ///     .get_video_m3u8_highest(&token, "/video.mp4", 2)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_video_m3u8_highest(
        &self,
        access_token: &AccessToken,
        path: &str,
        vip_level: u32,
    ) -> NetDiskResult<String> {
        let quality = VideoQuality::highest_for_vip_level(vip_level);
        self.get_video_m3u8(access_token, path, quality).await
    }

    /// Get audio m3u8 content with specified quality
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    /// use baidu_netdisk_sdk::playlist::AudioQuality;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    ///
    /// let m3u8 = client.playlist()
    ///     .get_audio_m3u8(&token, "/audio.mp3", AudioQuality::Quality128K)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_audio_m3u8(
        &self,
        access_token: &AccessToken,
        path: &str,
        quality: AudioQuality,
    ) -> NetDiskResult<String> {
        self.get_media_m3u8_content(access_token, path, quality.to_media_type())
            .await
    }

    /// Get audio m3u8 content with default quality (128K)
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
    /// let m3u8 = client.playlist()
    ///     .get_audio_m3u8_default(&token, "/audio.mp3")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_audio_m3u8_default(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<String> {
        self.get_audio_m3u8(access_token, path, AudioQuality::Quality128K)
            .await
    }
}

/// Options for get_playlist_list
#[derive(Debug, Clone, Default)]
pub struct PlaylistListOptions {
    /// Current page number (default 1)
    pub page: Option<i32>,
    /// Number of items per page (default 20)
    pub psize: Option<i32>,
}

impl PlaylistListOptions {
    /// Create a new PlaylistListOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page number
    pub fn page(mut self, page: i32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set page size
    pub fn psize(mut self, psize: i32) -> Self {
        self.psize = Some(psize);
        self
    }
}

/// Options for get_playlist_file_list
#[derive(Debug, Clone, Default)]
pub struct PlaylistFileListOptions {
    /// Show file details (1 or 0)
    pub showmeta: Option<i32>,
    /// Current page number (default 1)
    pub page: Option<i32>,
    /// Number of items per page (default 20)
    pub psize: Option<i32>,
}

impl PlaylistFileListOptions {
    /// Create a new PlaylistFileListOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set showmeta flag (1 to show details, 0 otherwise)
    pub fn showmeta(mut self, showmeta: i32) -> Self {
        self.showmeta = Some(showmeta);
        self
    }

    /// Set page number
    pub fn page(mut self, page: i32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set page size
    pub fn psize(mut self, psize: i32) -> Self {
        self.psize = Some(psize);
        self
    }
}

/// Playlist information
#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistInfo {
    /// Playlist name
    pub name: String,
    /// Playlist ID
    pub mb_id: u64,
    /// File count in playlist
    pub file_count: u32,
    /// Creation time (timestamp)
    pub ctime: u32,
    /// Modification time (timestamp)
    pub mtime: u32,
    /// Broadcast type (0: audio)
    pub btype: u32,
    /// Broadcast sub-type (0: normal, 1: music, 2: course)
    pub bstype: u32,
}

/// List of playlists
#[derive(Debug, Clone)]
pub struct PlaylistList {
    pub has_more: u32,
    pub list: Vec<PlaylistInfo>,
}

/// Playlist file information
#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistFileInfo {
    /// File server ID
    pub fs_id: String,
    /// File path
    pub path: String,
    /// Broadcast file modification time
    pub broadcast_file_mtime: u64,
    /// Server creation time (optional)
    #[serde(default)]
    pub server_ctime: Option<String>,
    /// Server modification time (optional)
    #[serde(default)]
    pub server_mtime: Option<String>,
    /// Local creation time (optional)
    #[serde(default)]
    pub local_ctime: Option<String>,
    /// Local modification time (optional)
    #[serde(default)]
    pub local_mtime: Option<String>,
    /// Is directory (optional)
    #[serde(default)]
    pub isdir: Option<String>,
    /// File size (optional)
    #[serde(default)]
    pub size: Option<String>,
    /// File category (optional)
    #[serde(default)]
    pub category: Option<String>,
    /// Server MD5 hash (optional)
    #[serde(default)]
    pub md5: Option<String>,
    /// Privacy level (optional)
    #[serde(default)]
    pub privacy: Option<String>,
    /// File name (optional)
    #[serde(default)]
    pub server_filename: Option<String>,
}

/// List of playlist files
#[derive(Debug, Clone)]
pub struct PlaylistFileList {
    pub has_more: u32,
    pub list: Vec<PlaylistFileInfo>,
}

/// Media file entry for playback
#[derive(Debug, Clone, Deserialize)]
pub struct MediaFileEntry {
    /// File ID
    pub fs_id: u64,
    /// File name
    pub server_filename: String,
    /// File path
    pub path: String,
    /// File size
    pub size: u64,
    /// Category
    pub category: i32,
    /// Media info
    pub media_info: Option<MediaInfo>,
}

/// Media information
#[derive(Debug, Clone, Deserialize)]
pub struct MediaInfo {
    /// Video or audio streams
    pub streams: Option<Vec<MediaStream>>,
    /// Duration in seconds
    pub duration: Option<f64>,
    /// Bitrate
    pub bitrate: Option<i32>,
}

/// Media stream information
#[derive(Debug, Clone, Deserialize)]
pub struct MediaStream {
    /// Stream type
    pub stream_type: Option<String>,
    /// Video width
    pub width: Option<i32>,
    /// Video height
    pub height: Option<i32>,
    /// Codec
    pub codec: Option<String>,
    /// Playback URL
    pub url: Option<String>,
    /// Video or audio file
    pub file: Option<MediaFile>,
}

/// Media file for playback
#[derive(Debug, Clone, Deserialize)]
pub struct MediaFile {
    /// File size
    pub size: u64,
    /// MD5
    pub md5: String,
    /// Server filename
    pub server_filename: String,
    /// Path
    pub path: String,
    /// File extension
    pub file_ext: String,
    /// Video info
    pub video: Option<VideoInfo>,
    /// Audio info
    pub audio: Option<AudioInfo>,
}

/// Video information
#[derive(Debug, Clone, Deserialize)]
pub struct VideoInfo {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f64>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
}

/// Audio information
#[derive(Debug, Clone, Deserialize)]
pub struct AudioInfo {
    pub duration: Option<f64>,
    pub bitrate: Option<i32>,
    pub codec: Option<String>,
    pub sample_rate: Option<i32>,
}

/// Media playback information
#[derive(Debug, Clone)]
pub struct MediaPlayInfo {
    pub list: Vec<MediaFileEntry>,
    pub request_id: u64,
}

#[derive(Debug, Deserialize)]
struct PlaylistListResponse {
    has_more: u32,
    #[serde(default)]
    list: Option<Vec<PlaylistInfo>>,
    errno: i32,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlaylistFileListResponse {
    has_more: u32,
    #[serde(default)]
    list: Option<Vec<PlaylistFileInfo>>,
    errno: i32,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MediaPlayInfoResponse {
    #[serde(default)]
    list: Option<Vec<MediaFileEntry>>,
    request_id: u64,
    errno: i32,
    errmsg: Option<String>,
}
