//! Device code authorization module
//!
//! Implements OAuth 2.0 device code flow for Baidu NetDisk authorization
use log::{debug, error, info, warn};

use super::{AccessToken, AccessTokenResponse, DeviceCode, DeviceCodeResponse, TokenStatus};
use crate::errors::{NetDiskError, NetDiskResult};
use crate::http::HttpClient;

/// Device code authorization client
///
/// Provides functionality for OAuth 2.0 device code flow
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
/// let auth = client.authorize();
///
/// // Get device code
/// let device_code = auth.get_device_code().await?;
/// println!("Visit: {}", device_code.verification_url);
/// println!("Enter code: {}", device_code.user_code);
///
/// // Poll for token
/// loop {
///     if let Some(token) = auth.request_access_token(&device_code).await? {
///         println!("Authorization successful!");
///         break;
///     }
///     tokio::time::sleep(tokio::time::Duration::from_secs(device_code.interval as u64)).await;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Authorization {
    http_client: HttpClient,
    app_key: String,
    app_secret: String,
    scope: String,
}

impl Authorization {
    /// Create a new Authorization instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::authorization()` instead
    pub fn new(http_client: HttpClient, app_key: &str, app_secret: &str, scope: &str) -> Self {
        Authorization {
            http_client,
            app_key: app_key.to_string(),
            app_secret: app_secret.to_string(),
            scope: scope.to_string(),
        }
    }

    /// Get device code for OAuth device authorization flow
    ///
    /// Returns DeviceCode containing verification URL, user code, and polling interval
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let auth = client.authorize();
    /// let device_code = auth.get_device_code().await?;
    /// println!("Please visit: {}", device_code.verification_url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_device_code(&self) -> NetDiskResult<DeviceCode> {
        let params = [
            ("response_type", "device_code"),
            ("client_id", &self.app_key),
            ("scope", &self.scope),
        ];

        let url = format!(
            "https://openapi.baidu.com/oauth/2.0/device/code?{}",
            serde_urlencoded::to_string(params).map_err(|e| {
                NetDiskError::Unknown {
                    message: format!("Failed to encode params: {}", e),
                }
            })?
        );

        debug!("Requesting device code...");

        let response: DeviceCodeResponse = self.http_client.get(&url, None).await?;

        info!(
            "Device code obtained: user_code={}, verification_url={}",
            response.user_code, response.verification_url
        );

        Ok(response.into())
    }

    /// Single attempt to get access token using device code
    ///
    /// Note: This method makes a single request without polling.
    /// The caller should implement polling logic based on the result.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(AccessToken))` - Token acquired successfully
    /// - `Ok(None)` - User has not authorized yet, continue polling
    /// - `Err` - An error occurred (e.g., device code expired)
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// let auth = client.authorize();
    /// let device_code = auth.get_device_code().await?;
    ///
    /// // Poll for token
    /// let token = loop {
    ///     if let Some(token) = auth.request_access_token(&device_code).await? {
    ///         break token;
    ///     }
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(device_code.interval as u64)).await;
    /// };
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request_access_token(
        &self,
        device_code: &DeviceCode,
    ) -> NetDiskResult<Option<AccessToken>> {
        let params = [
            ("grant_type", "device_token"),
            ("code", &device_code.device_code),
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

        debug!("Polling for device code authorization...");

        match self
            .http_client
            .get::<AccessTokenResponse>(&url, None)
            .await
        {
            Ok(response) => {
                let token: AccessToken = response.into();
                let status = token.validate();
                match status {
                    TokenStatus::Valid => {
                        info!(
                            "Access token obtained successfully, valid for {} seconds",
                            token.remaining_seconds()
                        );
                    }
                    TokenStatus::ExpiringSoon => {
                        warn!(
                            "Access token obtained but will expire soon ({} seconds)",
                            token.remaining_seconds()
                        );
                    }
                    TokenStatus::Expired => {
                        error!("Access token obtained but is already expired!");
                    }
                }
                Ok(Some(token))
            }
            Err(NetDiskError::AuthError { description }) => {
                if description.contains("authorization_pending")
                    || description.contains("slow_down")
                {
                    debug!("Authorization pending or slow_down: {}", description);
                    Ok(None)
                } else if description.contains("invalid_grant") {
                    error!("Invalid grant or device code expired: {}", description);
                    Err(NetDiskError::auth_error(&description))
                } else {
                    warn!("Other auth error: {}", description);
                    Ok(None)
                }
            }
            Err(e) => {
                error!("Token request failed: {}", e);
                Ok(None)
            }
        }
    }
}
