//! User information module
//!
//! Provides user-related functionality for Baidu NetDisk
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder().build()?;
//! let token = client.load_token_from_env()?;
//!
//! // Get basic user info
//! let user_info = client.user().get_user_info(&token, None).await?;
//! println!("User name: {}", user_info.baidu_name);
//!
//! // Get real identity with v2 version
//! let real_user_info = client.user().get_user_info(&token, Some("v2")).await?;
//! # Ok(())
//! # }
//! ```
use log::{debug, info};
use serde::Deserialize;

use crate::auth::{AccessToken, UserInfo};
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// User client for interacting with user-related APIs
#[derive(Debug, Clone)]
pub struct UserClient {
    http_client: HttpClient,
}

impl UserClient {
    /// Create a new UserClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::user()` instead
    pub fn new(http_client: HttpClient) -> Self {
        UserClient { http_client }
    }

    /// Get user information
    ///
    /// # Arguments
    ///
    /// * `access_token` - Access token for authentication
    /// * `vip_version` - Optional vip_version parameter (set to "v2" to get real user identity)
    ///
    /// # Returns
    ///
    /// Returns UserInfo containing user details
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let token = client.load_token_from_env()?;
    /// let user_info = client.user().get_user_info(&token, None).await?;
    /// println!("VIP type: {}", user_info.vip_type);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_user_info(
        &self,
        access_token: &AccessToken,
        vip_version: Option<&str>,
    ) -> NetDiskResult<UserInfo> {
        let mut params = vec![
            ("method", "uinfo"),
            ("access_token", &access_token.access_token),
        ];

        if let Some(v) = vip_version {
            params.push(("vip_version", v));
        }

        debug!("Getting user info with params: {:?}", params);

        let response: UserInfoResponse = self
            .http_client
            .get("/rest/2.0/xpan/nas", Some(&params))
            .await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, &response.errmsg));
        }

        info!("User info retrieved successfully: {}", response.baidu_name);

        Ok(UserInfo {
            baidu_name: response.baidu_name,
            netdisk_name: response.netdisk_name,
            avatar_url: response.avatar_url,
            vip_type: response.vip_type,
            uk: response.uk,
        })
    }
}

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    baidu_name: String,
    netdisk_name: String,
    avatar_url: String,
    vip_type: i32,
    uk: u64,
    errno: i32,
    errmsg: String,
}
