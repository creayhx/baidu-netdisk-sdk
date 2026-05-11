//! Authentication and token management module
//!
//! Provides device code authorization, access token management, and automatic refresh functionality
//!
//! # Features
//!
//! - **Device Code OAuth Flow**: Get device code and poll for access token
//! - **Token Management**: Store, validate, and refresh access tokens
//! - **Automatic Refresh**: Auto-refresh tokens before expiration
//! - **Thread Safe**: Safe concurrent access with RwLock
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//! use baidu_netdisk_sdk::auth::{Authorization, TokenProvider, TokenProviderConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder()
//!     .app_key("your_app_key")
//!     .app_secret("your_app_secret")
//!     .build()?;
//!
//! // Device code authorization
//! let auth = client.authorization();
//! let device_code = auth.get_device_code().await?;
//! println!("Please visit: {} and enter code: {}", 
//!     device_code.verification_url, 
//!     device_code.user_code
//! );
//!
//! // Poll for token
//! let token = loop {
//!     if let Some(token) = auth.request_access_token(&device_code).await? {
//!         break token;
//!     }
//!     tokio::time::sleep(tokio::time::Duration::from_secs(device_code.interval as u64)).await;
//! };
//!
//! // Use token provider for auto-refresh
//! let provider = client.token_provider(TokenProviderConfig::default());
//! provider.set_access_token(token)?;
//!
//! // Get valid token (auto-refreshes if needed)
//! let valid_token = provider.get_valid_token().await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod authorization;
pub mod token_provider;

pub use self::authorization::Authorization;
pub use self::token_provider::{TokenProvider, TokenProviderConfig};

/// Device code response from Baidu OAuth API
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceCodeResponse {
    /// Device code for polling
    pub device_code: String,
    /// User code for user input
    pub user_code: String,
    /// Verification URL for user to visit
    pub verification_url: String,
    /// QR code URL for scanning
    pub qrcode_url: String,
    /// Polling interval in seconds
    pub interval: u32,
    /// Expiration time in seconds
    pub expires_in: u32,
}

/// Device code information with expiration timestamp
#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub qrcode_url: String,
    pub interval: u32,
    pub expires_at: u64,
}

impl From<DeviceCodeResponse> for DeviceCode {
    fn from(response: DeviceCodeResponse) -> Self {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + response.expires_in as u64;

        DeviceCode {
            device_code: response.device_code,
            user_code: response.user_code,
            verification_url: response.verification_url,
            qrcode_url: response.qrcode_url,
            interval: response.interval,
            expires_at,
        }
    }
}

/// Access token response from Baidu OAuth API
///
/// Fields match exactly what Baidu API returns:
/// {"expires_in":2592000,"refresh_token":"...","access_token":"...","session_secret":"","session_key":"","scope":"basic netdisk"}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessTokenResponse {
    /// Access token for API requests
    pub access_token: String,
    /// Expiration time in seconds (typically 2592000 seconds = 30 days)
    pub expires_in: u64,
    /// Refresh token for obtaining new access token
    pub refresh_token: String,
    /// Scope of permissions (e.g., "basic netdisk")
    pub scope: String,
    /// Session key for signed API requests (may be empty string)
    pub session_key: String,
    /// Session secret for signed API requests (may be empty string)
    pub session_secret: String,
}

/// Access token information with acquisition timestamp
#[derive(Debug, Clone)]
pub struct AccessToken {
    /// Access token for API requests
    pub access_token: String,
    /// Expiration time in seconds
    pub expires_in: u64,
    /// Refresh token for obtaining new access token
    pub refresh_token: String,
    /// Scope of permissions
    pub scope: String,
    /// Session key for signed API requests
    pub session_key: String,
    /// Session secret for signed API requests
    pub session_secret: String,
    /// Acquisition timestamp in seconds
    pub acquired_at: u64,
}

impl From<AccessTokenResponse> for AccessToken {
    fn from(response: AccessTokenResponse) -> Self {
        let acquired_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        AccessToken {
            access_token: response.access_token,
            expires_in: response.expires_in,
            refresh_token: response.refresh_token,
            scope: response.scope,
            session_key: response.session_key,
            session_secret: response.session_secret,
            acquired_at,
        }
    }
}

impl AccessToken {
    /// Create a new AccessToken with the current timestamp
    ///
    /// # Arguments
    /// * `access_token` - The access token string
    /// * `refresh_token` - The refresh token string
    /// * `expires_in` - Expiration time in seconds
    /// * `scope` - Scope of permissions
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: u64,
        scope: String,
    ) -> Self {
        let acquired_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        AccessToken {
            access_token,
            refresh_token,
            expires_in,
            scope,
            session_key: String::new(),
            session_secret: String::new(),
            acquired_at,
        }
    }

    /// Create an AccessToken with all fields specified
    ///
    /// Use this when you need to set session_key, session_secret, or a specific acquired_at
    /// (e.g., for testing expired tokens)
    pub fn with_all(
        access_token: String,
        refresh_token: String,
        expires_in: u64,
        scope: String,
        session_key: String,
        session_secret: String,
        acquired_at: u64,
    ) -> Self {
        AccessToken {
            access_token,
            refresh_token,
            expires_in,
            scope,
            session_key,
            session_secret,
            acquired_at,
        }
    }

    /// Check if the token is expired
    /// Returns true if token has expired or will expire within 60 seconds
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.acquired_at + self.expires_in - 60
    }

    /// Get remaining valid seconds of the token
    pub fn remaining_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires_at = self.acquired_at + self.expires_in;
        expires_at.saturating_sub(now)
    }

    /// Validate the token and return its status
    pub fn validate(&self) -> TokenStatus {
        let remaining = self.remaining_seconds();
        if remaining == 0 {
            TokenStatus::Expired
        } else if remaining <= 300 {
            TokenStatus::ExpiringSoon
        } else {
            TokenStatus::Valid
        }
    }

    /// Check if token is valid (not expired and not expiring soon)
    pub fn is_valid(&self) -> bool {
        matches!(self.validate(), TokenStatus::Valid)
    }

    /// Get expiration timestamp in seconds since epoch
    pub fn expires_at(&self) -> u64 {
        self.acquired_at + self.expires_in
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token(acquired_at: u64, expires_in: u64) -> AccessToken {
        AccessToken {
            access_token: "test_token".to_string(),
            refresh_token: "test_refresh".to_string(),
            expires_in,
            scope: "basic netdisk".to_string(),
            session_key: String::new(),
            session_secret: String::new(),
            acquired_at,
        }
    }

    #[test]
    fn test_token_new() {
        let token = AccessToken::new(
            "access".to_string(),
            "refresh".to_string(),
            3600,
            "basic netdisk".to_string(),
        );

        assert_eq!(token.access_token, "access");
        assert_eq!(token.refresh_token, "refresh");
        assert_eq!(token.expires_in, 3600);
        assert!(token.session_key.is_empty());
    }

    #[test]
    fn test_token_with_all() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let token = AccessToken::with_all(
            "access".to_string(),
            "refresh".to_string(),
            7200,
            "netdisk".to_string(),
            "session_key".to_string(),
            "session_secret".to_string(),
            now,
        );

        assert_eq!(token.scope, "netdisk");
        assert_eq!(token.session_key, "session_key");
        assert_eq!(token.session_secret, "session_secret");
        assert_eq!(token.acquired_at, now);
    }

    #[test]
    fn test_token_is_expired() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let valid_token = create_test_token(now, 3600);
        assert!(!valid_token.is_expired());

        let expired_token = create_test_token(now - 7200, 3600);
        assert!(expired_token.is_expired());
    }

    #[test]
    fn test_token_validate() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let valid_token = create_test_token(now, 3600);
        assert_eq!(valid_token.validate(), TokenStatus::Valid);

        let expiring_soon_token = create_test_token(now, 200);
        assert_eq!(expiring_soon_token.validate(), TokenStatus::ExpiringSoon);

        let expired_token = create_test_token(now - 4000, 3600);
        assert_eq!(expired_token.validate(), TokenStatus::Expired);
    }

    #[test]
    fn test_token_is_valid() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let valid_token = create_test_token(now, 3600);
        assert!(valid_token.is_valid());

        let expired_token = create_test_token(now - 4000, 3600);
        assert!(!expired_token.is_valid());
    }

    #[test]
    fn test_token_remaining_seconds() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let token = create_test_token(now, 1000);
        let remaining = token.remaining_seconds();
        assert!(remaining >= 990 && remaining <= 1000);

        let expired_token = create_test_token(now - 100, 50);
        assert_eq!(expired_token.remaining_seconds(), 0);
    }

    #[test]
    fn test_token_expires_at() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let token = create_test_token(now, 500);
        assert_eq!(token.expires_at(), now + 500);
    }
}

/// Token validation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStatus {
    /// Token is valid and has plenty of time left
    Valid,
    /// Token will expire soon (within 5 minutes)
    ExpiringSoon,
    /// Token has already expired
    Expired,
}

/// API error response structure
///
/// Baidu NetDisk API may return error codes in different formats:
/// - `errno` and `errmsg` (common format)
/// - `error_code` and `error_msg` (alternative format)
///
/// All fields are optional to handle various response formats.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ApiErrorResponse {
    /// Error code (format 1)
    pub errno: Option<i32>,
    /// Error message (format 1)
    pub errmsg: Option<String>,
    /// Alternative error code field (format 2)
    #[serde(rename = "error_code")]
    pub error_code: Option<i32>,
    /// Alternative error message field (format 2)
    #[serde(rename = "error_msg")]
    pub error_msg: Option<String>,
}

impl ApiErrorResponse {
    /// Get error code, prioritizing error_code over errno
    pub fn get_errno(&self) -> i32 {
        self.error_code.or(self.errno).unwrap_or(-1)
    }

    /// Get error message, prioritizing error_msg over errmsg
    pub fn get_errmsg(&self) -> &str {
        if let Some(msg) = &self.error_msg {
            msg
        } else if let Some(msg) = &self.errmsg {
            msg
        } else {
            "Unknown error"
        }
    }

    /// Check if this response contains any error information
    pub fn has_error(&self) -> bool {
        self.errno.is_some()
            || self.error_code.is_some()
            || self.errmsg.is_some()
            || self.error_msg.is_some()
    }
}

/// Authentication error response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthErrorResponse {
    /// Error type
    pub error: String,
    /// Error description
    pub error_description: String,
}

/// User information response from Baidu NetDisk API
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    /// Baidu account name
    pub baidu_name: String,
    /// NetDisk account name
    pub netdisk_name: String,
    /// Avatar URL
    pub avatar_url: String,
    /// VIP type: 0=normal, 1=VIP, 2=Super VIP
    pub vip_type: i32,
    /// User ID
    pub uk: u64,
}

/// Quota information response from Baidu NetDisk API
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuotaInfo {
    /// Total storage capacity in bytes
    pub total: u64,
    /// Used storage capacity in bytes
    pub used: u64,
    /// Free storage capacity in bytes
    pub free: u64,
}
