//! Quota and storage capacity module
//!
//! Provides disk quota and storage capacity information for Baidu NetDisk
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
//! // Get basic quota info
//! let quota = client.quota().get_quota().await?;
//! println!("Total storage: {} GB", quota.total / 1024 / 1024 / 1024);
//!
//! // Get detailed capacity with expiration check
//! let capacity = client.quota().get_capacity(true, true).await?;
//! println!("Usage percentage: {:.1}%", capacity.usage_percentage());
//! println!("Used: {}", capacity.format_used());
//! # Ok(())
//! # }
//! ```
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::QuotaInfo;
use crate::client::TokenGetter;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// Quota client for interacting with quota-related APIs
#[derive(Debug, Clone)]
pub struct QuotaClient {
    http_client: HttpClient,
    token_getter: Arc<dyn TokenGetter>,
}

impl QuotaClient {
    /// Create a new QuotaClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::quota()` instead
    pub fn new(http_client: HttpClient, token_getter: Arc<dyn TokenGetter>) -> Self {
        QuotaClient {
            http_client,
            token_getter,
        }
    }

    /// Get a reference to the internal HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Get user's netdisk quota information
    ///
    /// # Returns
    ///
    /// Returns QuotaInfo containing total, used, and free space
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    /// let quota = client.quota().get_quota().await?;
    /// println!("Free space: {} bytes", quota.free);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_quota(&self) -> NetDiskResult<QuotaInfo> {
        let token = self.token_getter.get_token().await?;

        let params = [("access_token", token.access_token.as_str())];

        debug!("Getting quota info with params: {:?}", params);

        let response: QuotaResponse = self.http_client.get("/api/quota", Some(&params)).await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, &response.errmsg));
        }

        info!("Quota info retrieved successfully");

        Ok(QuotaInfo {
            total: response.total,
            used: response.used,
            free: response.total - response.used,
        })
    }

    /// Get detailed capacity information
    ///
    /// # Arguments
    ///
    /// * `check_free` - Whether to check free space
    /// * `check_expire` - Whether to check expiration status
    ///
    /// # Returns
    ///
    /// Returns CapacityInfo containing detailed capacity information
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    /// let capacity = client.quota().get_capacity(true, true).await?;
    /// println!("Expired: {}", capacity.expire);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_capacity(
        &self,
        check_free: bool,
        check_expire: bool,
    ) -> NetDiskResult<CapacityInfo> {
        let token = self.token_getter.get_token().await?;

        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("access_token", token.access_token.as_str()));

        if check_free {
            params.push(("checkfree", "1"));
        }
        if check_expire {
            params.push(("checkexpire", "1"));
        }

        debug!("Getting capacity info with params: {:?}", params);

        let response: CapacityResponse = self.http_client.get("/api/quota", Some(&params)).await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, &response.errmsg));
        }

        info!("Capacity info retrieved successfully");

        Ok(CapacityInfo {
            total: response.total,
            used: response.used,
            free: response.free,
            expire: response.expire,
        })
    }

    /// Get quota info with expiration check
    ///
    /// A convenience method that checks both free space and expiration
    ///
    /// # Returns
    ///
    /// Returns CapacityInfo containing detailed capacity information
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    /// let capacity = client.quota().get_quota_with_expire().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_quota_with_expire(&self) -> NetDiskResult<CapacityInfo> {
        self.get_capacity(true, true).await
    }
}

#[derive(Debug, Deserialize)]
struct QuotaResponse {
    errno: i32,
    errmsg: String,
    total: u64,
    used: u64,
}

#[derive(Debug, Deserialize)]
struct CapacityResponse {
    errno: i32,
    errmsg: String,
    total: u64,
    used: u64,
    free: u64,
    expire: bool,
}

/// Detailed capacity information from Baidu NetDisk API
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CapacityInfo {
    /// Total storage capacity in bytes
    pub total: u64,
    /// Used storage capacity in bytes
    pub used: u64,
    /// Free storage capacity in bytes
    pub free: u64,
    /// Whether the storage is expired
    pub expire: bool,
}

impl CapacityInfo {
    /// Calculate usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64 / self.total as f64) * 100.0
        }
    }

    /// Format bytes to human readable string
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        match bytes {
            b if b < KB => format!("{} B", b),
            b if b < MB => format!("{:.2} KB", b as f64 / KB as f64),
            b if b < GB => format!("{:.2} MB", b as f64 / MB as f64),
            b if b < TB => format!("{:.2} GB", b as f64 / GB as f64),
            b => format!("{:.2} TB", b as f64 / TB as f64),
        }
    }

    /// Format total capacity to human readable string
    pub fn format_total(&self) -> String {
        Self::format_bytes(self.total)
    }

    /// Format used capacity to human readable string
    pub fn format_used(&self) -> String {
        Self::format_bytes(self.used)
    }

    /// Format free capacity to human readable string
    pub fn format_free(&self) -> String {
        Self::format_bytes(self.free)
    }
}
