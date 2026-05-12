//! Token provider module
//!
//! Manages access token lifecycle with automatic refresh and thread-safe caching
use std::sync::{Arc, RwLock};

use log::{debug, error, info, warn};

use super::{AccessToken, AccessTokenResponse, TokenStatus};
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// Configuration for TokenProvider
///
/// Customizes token refresh behavior
#[derive(Debug, Clone)]
pub struct TokenProviderConfig {
    /// Whether to automatically refresh tokens
    pub auto_refresh: bool,
    /// Refresh token ahead of expiration by this many seconds
    pub refresh_ahead_seconds: u64,
    /// Maximum number of retry attempts for refresh
    pub max_refresh_retries: usize,
}

impl Default for TokenProviderConfig {
    fn default() -> Self {
        TokenProviderConfig {
            auto_refresh: true,
            refresh_ahead_seconds: 86400,
            max_refresh_retries: 3,
        }
    }
}

/// Token Provider
///
/// Manages access token acquisition, refresh, and caching with thread safety
///
/// # Examples
///
/// ```
/// use baidu_netdisk_sdk::BaiduNetDiskClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = BaiduNetDiskClient::builder()
///     .app_key("your_app_key")
///     .app_secret("your_app_secret")
///     .build()?;
///
/// // Get token provider
/// let provider = client.token_provider();
///
/// // Load or set token
/// // let token = provider.load_from_file("token.json")?;
/// // provider.set_access_token(token)?;
///
/// // Get valid token (auto-refreshes if needed)
/// let valid_token = provider.get_valid_token().await?;
///
/// // Check token status
/// let status = provider.validate_token()?;
/// println!("Token status: {:?}", status);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TokenProvider {
    http_client: HttpClient,
    app_key: String,
    app_secret: String,
    access_token: Arc<RwLock<Option<AccessToken>>>,
    config: TokenProviderConfig,
}

impl TokenProvider {
    /// Create a new TokenProvider instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::token_provider()` instead
    pub fn new(
        http_client: HttpClient,
        app_key: &str,
        app_secret: &str,
        config: TokenProviderConfig,
    ) -> Self {
        TokenProvider {
            http_client,
            app_key: app_key.to_string(),
            app_secret: app_secret.to_string(),
            access_token: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Get the access token string
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let provider = client.token_provider();
    /// // After setting token...
    /// // let token_str = provider.get_access_token()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_access_token(&self) -> NetDiskResult<String> {
        let token = self
            .access_token
            .read()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to read access token: {}", e)))?;
        token
            .as_ref()
            .map(|t| t.access_token.clone())
            .ok_or_else(|| NetDiskError::auth_error("No access token available"))
    }

    /// Get the full AccessToken object (internal use)
    pub fn get_access_token_full(&self) -> NetDiskResult<Option<AccessToken>> {
        let token = self
            .access_token
            .read()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to read access token: {}", e)))?;
        Ok(token.clone())
    }

    /// Validate and get the current token status
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    /// use baidu_netdisk_sdk::auth::TokenStatus;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let provider = client.token_provider();
    /// // After setting token...
    /// // let status = provider.validate_token()?;
    /// // match status {
    /// //     TokenStatus::Valid => println!("Token is valid"),
    /// //     TokenStatus::ExpiringSoon => println!("Token will expire soon"),
    /// //     TokenStatus::Expired => println!("Token is expired"),
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_token(&self) -> NetDiskResult<TokenStatus> {
        let token = self
            .access_token
            .read()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to read access token: {}", e)))?;

        match token.as_ref() {
            Some(t) => Ok(t.validate()),
            None => Ok(TokenStatus::Expired),
        }
    }

    /// Set the access token
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    /// use baidu_netdisk_sdk::auth::AccessToken;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let provider = client.token_provider();
    /// // After getting token from authorization...
    /// // provider.set_access_token(token)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_access_token(&self, token: AccessToken) -> NetDiskResult<()> {
        let status = token.validate();
        let mut current = self
            .access_token
            .write()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to write access token: {}", e)))?;
        *current = Some(token);

        match status {
            TokenStatus::Valid => {
                info!(
                    "Access token updated successfully (valid for {} seconds)",
                    current.as_ref().unwrap().remaining_seconds()
                );
            }
            TokenStatus::ExpiringSoon => {
                warn!(
                    "Access token updated but will expire soon ({} seconds remaining)",
                    current.as_ref().unwrap().remaining_seconds()
                );
            }
            TokenStatus::Expired => {
                error!("Access token updated but is already expired! Please refresh or re-authenticate.");
            }
        }
        Ok(())
    }

    /// Check if the token needs refresh
    pub fn needs_refresh(&self) -> NetDiskResult<bool> {
        let token = self
            .access_token
            .read()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to read access token: {}", e)))?;

        match token.as_ref() {
            Some(t) => Ok(t.remaining_seconds() <= self.config.refresh_ahead_seconds),
            None => Ok(true),
        }
    }

    /// Refresh the access token
    ///
    /// Uses refresh token to obtain a new access token
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder()
    ///     .app_key("your_app_key")
    ///     .app_secret("your_app_secret")
    ///     .build()?;
    /// let provider = client.token_provider();
    /// // After setting token...
    /// // let new_token = provider.refresh_token().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_token(&self) -> NetDiskResult<AccessToken> {
        // First, get refresh_token and release the lock
        let refresh_token = {
            let current_token = self.access_token.read().map_err(|e| {
                NetDiskError::SyncError(format!("Failed to read access token: {}", e))
            })?;
            current_token
                .as_ref()
                .ok_or_else(|| NetDiskError::auth_error("No access token to refresh"))?
                .refresh_token
                .clone()
        };

        info!("Starting token refresh...");

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.app_key),
            ("client_secret", &self.app_secret),
        ];

        let url = format!(
            "https://openapi.baidu.com/oauth/2.0/token?{}",
            serde_urlencoded::to_string(params).map_err(|e| {
                NetDiskError::Unknown {
                    message: format!("Failed to encode params: {}", e),
                }
            })?
        );

        let response: AccessTokenResponse =
            self.http_client.get(&url, None).await.map_err(|e| {
                error!("Token refresh failed: {}", e);
                NetDiskError::auth_error(&format!(
                    "Token refresh failed, please re-authenticate: {}",
                    e
                ))
            })?;

        let token: AccessToken = response.into();
        self.set_access_token(token.clone())?;

        info!(
            "Token refreshed successfully, valid for {} seconds",
            token.remaining_seconds()
        );
        Ok(token)
    }

    /// Try to refresh token safely (returns None on failure)
    pub async fn try_refresh_token(&self) -> NetDiskResult<Option<AccessToken>> {
        if !self.needs_refresh()? {
            debug!("Token doesn't need refresh");
            return Ok(None);
        }

        match self.refresh_token().await {
            Ok(token) => Ok(Some(token)),
            Err(e) => {
                warn!("Failed to refresh token: {}", e);
                Ok(None)
            }
        }
    }

    /// Get a valid token (auto-refresh if enabled)
    ///
    /// Main method to get a valid token. Auto-refreshes if token is expired or expiring soon
    /// and auto-refresh is enabled in config.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let provider = client.token_provider();
    /// // After setting token...
    /// // let token = provider.get_valid_token().await?;
    /// // Use token for API calls...
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_valid_token(&self) -> NetDiskResult<AccessToken> {
        let token_opt = self
            .access_token
            .read()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to read access token: {}", e)))?
            .clone();

        let token =
            token_opt.ok_or_else(|| NetDiskError::auth_error("No access token available"))?;

        let status = token.validate();
        match status {
            TokenStatus::Valid => {
                debug!("Token is valid, returning directly");
                Ok(token)
            }
            TokenStatus::ExpiringSoon => {
                if self.config.auto_refresh {
                    info!("Token is expiring soon, attempting refresh...");
                    match self.refresh_token().await {
                        Ok(new_token) => Ok(new_token),
                        Err(_) => {
                            warn!("Token refresh failed but token is still usable, returning existing token");
                            Ok(token)
                        }
                    }
                } else {
                    warn!("Token is expiring soon, auto-refresh is disabled");
                    Ok(token)
                }
            }
            TokenStatus::Expired => {
                if self.config.auto_refresh {
                    error!("Token is expired, attempting refresh...");
                    self.refresh_token().await
                } else {
                    Err(NetDiskError::auth_error(
                        "Token is expired and auto-refresh is disabled, please re-authenticate",
                    ))
                }
            }
        }
    }

    /// Clear the stored token
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let provider = client.token_provider();
    /// // provider.clear_token()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_token(&self) -> NetDiskResult<()> {
        let mut token = self
            .access_token
            .write()
            .map_err(|e| NetDiskError::SyncError(format!("Failed to write access token: {}", e)))?;
        *token = None;
        info!("Access token cleared");
        Ok(())
    }
}
