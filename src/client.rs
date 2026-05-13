use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
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

/// Token 获取器 trait - 抽象 token 获取逻辑
#[async_trait]
pub trait TokenGetter: Debug + Send + Sync + 'static {
    async fn get_token(&self) -> NetDiskResult<AccessToken>;
}

/// 动态 token 获取器 - 使用 TokenProvider 动态获取和刷新 token
#[derive(Debug)]
pub struct DynamicTokenGetter {
    token_provider: Arc<TokenProvider>,
}

impl DynamicTokenGetter {
    pub fn new(token_provider: Arc<TokenProvider>) -> Self {
        Self { token_provider }
    }
}

#[async_trait]
impl TokenGetter for DynamicTokenGetter {
    async fn get_token(&self) -> NetDiskResult<AccessToken> {
        self.token_provider.get_valid_token().await
    }
}

/// 静态 token 获取器 - 返回固定的 token
#[derive(Debug)]
pub struct StaticTokenGetter {
    token: Arc<AccessToken>,
}

impl StaticTokenGetter {
    pub fn new(token: AccessToken) -> Self {
        Self {
            token: Arc::new(token),
        }
    }
}

#[async_trait]
impl TokenGetter for StaticTokenGetter {
    async fn get_token(&self) -> NetDiskResult<AccessToken> {
        Ok((*self.token).clone())
    }
}

#[allow(dead_code)]
pub(crate) trait ClientAccessor: Send + Sync {
    fn get_token(&self) -> impl Future<Output = NetDiskResult<AccessToken>> + Send + '_;
    fn user_client(&self) -> &UserClient;
    fn quota_client(&self) -> &QuotaClient;
    fn file_client(&self) -> &FileClient;
    fn download_client(&self) -> &DownloadClient;
    fn upload_client(&self) -> &UploadClient;
    fn playlist_client(&self) -> &PlaylistClient;
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

    pub fn load_token_from_env(&self) -> NetDiskResult<()> {
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

        Ok(())
    }

    pub fn validate_token(&self) -> NetDiskResult<TokenStatus> {
        self.token_provider.validate_token()
    }

    pub fn with_token(&self, token: AccessToken) -> TokenScopedClient {
        let token_getter: Arc<dyn TokenGetter> = Arc::new(StaticTokenGetter::new(token.clone()));

        let user_client = Arc::new(UserClient::new(
            self.user_client.http_client().clone(),
            token_getter.clone(),
        ));
        let quota_client = Arc::new(QuotaClient::new(
            self.quota_client.http_client().clone(),
            token_getter.clone(),
        ));
        let file_client = Arc::new(FileClient::new(
            self.file_client.http_client().clone(),
            token_getter.clone(),
        ));
        let download_client = Arc::new(DownloadClient::new(
            file_client.clone(),
            token_getter.clone(),
        ));
        let upload_client = Arc::new(UploadClient::new(
            self.upload_client.http_client().clone(),
            token_getter.clone(),
        ));
        let playlist_client = Arc::new(PlaylistClient::new(
            self.playlist_client.http_client().clone(),
            token_getter.clone(),
        ));

        TokenScopedClient::new(
            Arc::new(token),
            user_client,
            quota_client,
            file_client,
            download_client,
            upload_client,
            playlist_client,
        )
    }
}

impl ClientAccessor for BaiduNetDiskClient {
    async fn get_token(&self) -> NetDiskResult<AccessToken> {
        self.get_valid_token().await
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
    async fn get_token(&self) -> NetDiskResult<AccessToken> {
        Ok((*self.token).clone())
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

        let token_provider_ref = Arc::new(token_provider.clone());
        let token_getter: Arc<dyn TokenGetter> =
            Arc::new(DynamicTokenGetter::new(token_provider_ref));

        let user_client = Arc::new(UserClient::new(http_client.clone(), token_getter.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone(), token_getter.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone(), token_getter.clone()));
        let download_client = Arc::new(DownloadClient::new(
            file_client.clone(),
            token_getter.clone(),
        ));
        let upload_client = Arc::new(UploadClient::new(http_client.clone(), token_getter.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(
            http_client.clone(),
            token_getter.clone(),
        ));

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
        assert_eq!(
            client.config().http_config.connect_timeout,
            Duration::from_secs(10)
        );
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
        let token_getter: Arc<dyn TokenGetter> = Arc::new(StaticTokenGetter::new(token.clone()));
        let user_client = Arc::new(UserClient::new(http_client.clone(), token_getter.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone(), token_getter.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone(), token_getter.clone()));
        let download_client = Arc::new(DownloadClient::new(
            file_client.clone(),
            token_getter.clone(),
        ));
        let upload_client = Arc::new(UploadClient::new(http_client.clone(), token_getter.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(
            http_client.clone(),
            token_getter.clone(),
        ));

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
        let token_getter: Arc<dyn TokenGetter> = Arc::new(StaticTokenGetter::new(token.clone()));
        let user_client = Arc::new(UserClient::new(http_client.clone(), token_getter.clone()));
        let quota_client = Arc::new(QuotaClient::new(http_client.clone(), token_getter.clone()));
        let file_client = Arc::new(FileClient::new(http_client.clone(), token_getter.clone()));
        let download_client = Arc::new(DownloadClient::new(
            file_client.clone(),
            token_getter.clone(),
        ));
        let upload_client = Arc::new(UploadClient::new(http_client.clone(), token_getter.clone()));
        let playlist_client = Arc::new(PlaylistClient::new(
            http_client.clone(),
            token_getter.clone(),
        ));

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
        assert_ne!(
            scoped_client1.token().access_token,
            scoped_client2.token().access_token
        );
    }
}
