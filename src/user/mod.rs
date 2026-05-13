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
//! client.load_token_from_env()?;
//!
//! // Get basic user info
//! let user_info = client.user().get_user_info(None).await?;
//! println!("User name: {}", user_info.baidu_name);
//!
//! // Get real identity with v2 version
//! let real_user_info = client.user().get_user_info(Some("v2")).await?;
//! # Ok(())
//! # }
//! ```
use log::{debug, info};
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::UserInfo;
use crate::client::TokenGetter;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// User client for interacting with user-related APIs
#[derive(Debug, Clone)]
pub struct UserClient {
    http_client: HttpClient,
    token_getter: Arc<dyn TokenGetter>,
}

impl UserClient {
    /// Create a new UserClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::user()` instead
    pub fn new(http_client: HttpClient, token_getter: Arc<dyn TokenGetter>) -> Self {
        UserClient {
            http_client,
            token_getter,
        }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Get user information
    ///
    /// # Arguments
    ///
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
    /// client.load_token_from_env()?;
    /// let user_info = client.user().get_user_info(None).await?;
    /// println!("VIP type: {}", user_info.vip_type);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_user_info(&self, vip_version: Option<&str>) -> NetDiskResult<UserInfo> {
        let token = self.token_getter.get_token().await?;

        let mut params = vec![("method", "uinfo"), ("access_token", &token.access_token)];

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
