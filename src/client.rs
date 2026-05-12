use std::time::Duration;

use dotenv::dotenv;
use log::{debug, error, info, warn};

use crate::auth::{AccessToken, TokenStatus};
use crate::auth::{Authorization, TokenProvider, TokenProviderConfig};
use crate::download::DownloadClient;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::file::FileClient;
use crate::http::{client::HttpClientConfig, HttpClient};
use crate::playlist::PlaylistClient;
use crate::quota::QuotaClient;
use crate::upload::UploadClient;
use crate::user::UserClient;

/// Baidu NetDisk Client
///
/// Entry point for accessing Baidu NetDisk API, providing file management,
/// upload/download, sharing and other functionalities
#[derive(Debug, Clone)]
pub struct BaiduNetDiskClient {
    token_provider: TokenProvider,
    authorization: Authorization,
    user_client: UserClient,
    quota_client: QuotaClient,
    file_client: FileClient,
    download_client: DownloadClient,
    upload_client: UploadClient,
    playlist_client: PlaylistClient,
    config: ClientConfig,
}

impl BaiduNetDiskClient {
    /// Create a client builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Get the authorization module
    pub fn authorize(&self) -> &Authorization {
        &self.authorization
    }

    /// Get the token provider
    pub fn token_provider(&self) -> &TokenProvider {
        &self.token_provider
    }

    /// Get the user client
    pub fn user(&self) -> &UserClient {
        &self.user_client
    }

    /// Get the quota client
    pub fn quota(&self) -> &QuotaClient {
        &self.quota_client
    }

    /// Get the file client
    pub fn file(&self) -> &FileClient {
        &self.file_client
    }

    /// Get the download client
    pub fn download(&self) -> &DownloadClient {
        &self.download_client
    }

    /// Get the upload client
    pub fn upload(&self) -> &UploadClient {
        &self.upload_client
    }

    /// Get the playlist client
    pub fn playlist(&self) -> &PlaylistClient {
        &self.playlist_client
    }

    /// Get current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get a valid access token
    pub async fn get_valid_token(&self) -> NetDiskResult<AccessToken> {
        self.token_provider.get_valid_token().await
    }

    /// Set the access token
    pub fn set_access_token(&self, token: AccessToken) -> NetDiskResult<()> {
        self.token_provider.set_access_token(token)
    }

    /// Load access token from environment variables
    ///
    /// Environment variables (with BD_NETDISK_ prefix):
    /// - BD_NETDISK_ACCESS_TOKEN: The access token string
    /// - BD_NETDISK_REFRESH_TOKEN: The refresh token string
    /// - BD_NETDISK_EXPIRES_IN: Expiration time in seconds
    /// - BD_NETDISK_SCOPE: Scope/permissions
    /// - BD_NETDISK_SESSION_KEY: Session key (optional)
    /// - BD_NETDISK_SESSION_SECRET: Session secret (optional)
    /// - BD_NETDISK_ACQUIRED_AT: Optional acquisition timestamp in seconds (for testing expired tokens)
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

    /// Validate the current token and get its status
    pub fn validate_token(&self) -> NetDiskResult<TokenStatus> {
        self.token_provider.validate_token()
    }
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// App ID
    pub app_id: String,
    /// App Key
    pub app_key: String,
    /// App Secret
    pub app_secret: String,
    /// App Name (for user identification of multiple apps)
    pub app_name: String,
    /// API scope/permissions
    pub scope: String,
    /// HTTP client configuration
    pub http_config: HttpClientConfig,
    /// Token provider configuration
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

/// Client builder for constructing BaiduNetDiskClient
#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Set App ID
    pub fn app_id(mut self, app_id: &str) -> Self {
        self.config.app_id = app_id.to_string();
        self
    }

    /// Set App Key
    pub fn app_key(mut self, app_key: &str) -> Self {
        self.config.app_key = app_key.to_string();
        self
    }

    /// Set App Secret
    pub fn app_secret(mut self, app_secret: &str) -> Self {
        self.config.app_secret = app_secret.to_string();
        self
    }

    /// Set App Name (for user identification of multiple apps)
    pub fn app_name(mut self, app_name: &str) -> Self {
        self.config.app_name = app_name.to_string();
        self
    }

    /// Set API scope
    pub fn scope(mut self, scope: &str) -> Self {
        self.config.scope = scope.to_string();
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.http_config.timeout = timeout;
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.http_config.connect_timeout = timeout;
        self
    }

    /// Set maximum retry attempts
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.config.http_config.max_retries = max_retries;
        self
    }

    /// Set User-Agent
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.config.http_config.user_agent = user_agent.to_string();
        self
    }

    /// Set auto-refresh for tokens
    pub fn auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.config.token_config.auto_refresh = auto_refresh;
        self
    }

    /// Set token refresh ahead time in seconds
    pub fn refresh_ahead_seconds(mut self, seconds: u64) -> Self {
        self.config.token_config.refresh_ahead_seconds = seconds;
        self
    }

    /// Build the client instance
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

        let user_client = UserClient::new(http_client.clone());
        let quota_client = QuotaClient::new(http_client.clone());
        let file_client = FileClient::new(http_client.clone());
        let download_client = DownloadClient::new(file_client.clone());
        let upload_client = UploadClient::new(http_client.clone());
        let playlist_client = PlaylistClient::new(http_client.clone());

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
}
