//! HTTP client implementation
//!
//! Provides robust HTTP request handling with retry, timeout, and error handling
use std::time::Duration;

use log::{debug, error, warn};
use reqwest::{Client, ClientBuilder, Response, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::auth::{ApiErrorResponse, AuthErrorResponse};
use crate::errors::{NetDiskError, NetDiskResult};

/// HTTP client configuration
///
/// Customizes HTTP client behavior
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Request timeout duration
    pub timeout: Duration,
    /// Connection timeout duration
    pub connect_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// User-Agent string
    pub user_agent: String,
    /// Whether to follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects
    pub max_redirects: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        HttpClientConfig {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_delay_ms: 1000,
            user_agent: "pan.baidu.com".to_string(),
            follow_redirects: true,
            max_redirects: 10,
        }
    }
}

/// HTTP client for making API requests
///
/// Provides GET, POST, POST form, POST JSON, and multipart file upload methods
/// with automatic retry and error handling
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: Client,
    config: HttpClientConfig,
    base_url: Url,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::{HttpClient, HttpClientConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = HttpClientConfig::default();
    /// let client = HttpClient::new(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: HttpClientConfig) -> NetDiskResult<Self> {
        let redirect_policy = if config.follow_redirects {
            reqwest::redirect::Policy::limited(config.max_redirects)
        } else {
            reqwest::redirect::Policy::none()
        };

        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .redirect(redirect_policy)
            .user_agent("pan.baidu.com")
            .build()
            .map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to build HTTP client: {}", e),
            })?;

        let base_url = Url::parse("https://pan.baidu.com").map_err(|e| NetDiskError::Unknown {
            message: format!("Failed to parse base URL: {}", e),
        })?;

        Ok(HttpClient {
            inner: client,
            config,
            base_url,
        })
    }

    /// Create a default HTTP client
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::HttpClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::try_default()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_default() -> NetDiskResult<Self> {
        Self::new(HttpClientConfig::default())
    }

    /// Send GET request and parse JSON response
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::http::HttpClient;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Response {
    ///     // fields
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::try_default()?;
    /// let response: Response = client.get("https://example.com/api", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&[(&str, &str)]>,
    ) -> NetDiskResult<T> {
        self.get_with_headers(url, params, None).await
    }

    /// Send GET request with custom headers and parse JSON response
    pub async fn get_with_headers<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&[(&str, &str)]>,
        headers: Option<&[(&str, &str)]>,
    ) -> NetDiskResult<T> {
        let url = if url.starts_with("http") {
            Url::parse(url)?
        } else {
            self.build_url(url, params.unwrap_or(&[]))?
        };

        debug!("HTTP GET: {}", url);
        if let Some(p) = params {
            if !p.is_empty() {
                debug!("  Query params: {:?}", p);
            }
        }
        if let Some(h) = headers {
            if !h.is_empty() {
                debug!("  Headers: {:?}", h);
            }
        }

        self.execute_request_with_retry(|| async {
            let mut request = self.inner.get(url.clone());
            if let Some(h) = headers {
                for (key, value) in h.iter() {
                    request = request.header(*key, *value);
                }
            }
            request.send().await
        })
        .await
    }

    /// Send POST form request and parse JSON response
    pub async fn post_form<T: DeserializeOwned>(
        &self,
        url: &str,
        form: Option<&[(&str, &str)]>,
        params: Option<&[(&str, &str)]>,
    ) -> NetDiskResult<T> {
        let url = if url.starts_with("http") {
            Url::parse(url)?
        } else {
            self.build_url(url, params.unwrap_or(&[]))?
        };

        let form = form.unwrap_or(&[]);
        debug!("HTTP POST Form: {}", url);
        if !form.is_empty() {
            debug!("  Form data: {:?}", form);
        }

        self.execute_request_with_retry(|| async {
            self.inner.post(url.clone()).form(form).send().await
        })
        .await
    }

    /// Send POST request with query parameters and parse JSON response
    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Option<&[(&str, &str)]>,
    ) -> NetDiskResult<T> {
        let url = if url.starts_with("http") {
            Url::parse(url)?
        } else {
            self.build_url(url, params.unwrap_or(&[]))?
        };

        debug!("HTTP POST: {}", url);
        if let Some(p) = params {
            if !p.is_empty() {
                debug!("  Query params: {:?}", p);
            }
        }

        self.execute_request_with_retry(|| async { self.inner.post(url.clone()).send().await })
            .await
    }

    /// Send POST JSON request and parse JSON response
    pub async fn post_json<T: DeserializeOwned, U: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &U,
    ) -> NetDiskResult<T> {
        let url = if url.starts_with("http") {
            Url::parse(url)?
        } else {
            self.build_url(url, &[])?
        };

        let json_body =
            serde_json::to_string(body).unwrap_or_else(|_| "serialization failed".to_string());
        debug!("HTTP POST JSON: {}", url);
        debug!("  Body: {}", json_body);

        self.execute_request_with_retry(|| async {
            self.inner.post(url.clone()).json(body).send().await
        })
        .await
    }

    /// Send multipart file POST request and parse JSON response
    pub async fn post_multipart<T: DeserializeOwned>(
        &self,
        url: &str,
        field_name: String,
        file_name: String,
        data: Vec<u8>,
    ) -> NetDiskResult<T> {
        let url = Url::parse(url)?;
        debug!("HTTP POST Multipart: {}", url);
        debug!(
            "  Field: {}, File: {}, Size: {} bytes",
            field_name,
            file_name,
            data.len()
        );

        self.execute_request_with_retry(|| async {
            let form = reqwest::multipart::Form::new().part(
                field_name.clone(),
                reqwest::multipart::Part::bytes(data.clone()).file_name(file_name.clone()),
            );
            self.inner.post(url.clone()).multipart(form).send().await
        })
        .await
    }

    /// Build URL with path and query parameters
    fn build_url(&self, path: &str, params: &[(&str, &str)]) -> NetDiskResult<Url> {
        let mut url = self.base_url.join(path)?;

        if !params.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(key, value);
            }
        }

        debug!("Built URL: {}", url);
        Ok(url)
    }

    /// Execute request with retry logic and parse JSON response
    async fn execute_request_with_retry<T: DeserializeOwned, F, Fut>(
        &self,
        make_request: F,
    ) -> NetDiskResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
    {
        let mut attempts = 0;

        loop {
            attempts += 1;

            match make_request().await {
                Ok(response) => {
                    if response.status().is_server_error() && attempts < self.config.max_retries {
                        warn!(
                            "Server error ({}), attempt {}/{}, retrying...",
                            response.status(),
                            attempts,
                            self.config.max_retries
                        );
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                        continue;
                    }
                    return self.parse_response(response).await;
                }
                Err(e) => {
                    if attempts < self.config.max_retries && self.should_retry(&e) {
                        warn!(
                            "Request failed, attempt {}/{}, retrying...: {}",
                            attempts, self.config.max_retries, e
                        );
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                        continue;
                    }
                    return Err(NetDiskError::http_error_with_source(0, "unknown", e));
                }
            }
        }
    }

    /// Determine if the request should be retried
    fn should_retry(&self, err: &reqwest::Error) -> bool {
        err.is_timeout() || err.is_connect() || err.is_body()
    }

    /// Parse HTTP response
    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> NetDiskResult<T> {
        let status = response.status();
        let url = response.url().to_string();

        if status.is_success() {
            let body = response.text().await.map_err(|e| NetDiskError::Unknown {
                message: format!("Failed to read response body: {}", e),
            })?;

            debug!("Response body (status {}): {}", status, body);

            match serde_json::from_str(&body) {
                Ok(data) => Ok(data),
                Err(e) => {
                    error!("Failed to parse JSON response: {}", e);
                    error!("Response body that failed to parse: {}", body);
                    Err(NetDiskError::Unknown {
                        message: format!("Failed to parse JSON: {}", e),
                    })
                }
            }
        } else {
            let body = match response.text().await {
                Ok(b) => b,
                Err(_) => String::from("Unknown error"),
            };

            error!("API request failed: {} - {}", status, body);

            if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&body) {
                Err(NetDiskError::api_error(
                    api_error.get_errno(),
                    api_error.get_errmsg(),
                ))
            } else if let Ok(auth_error) = serde_json::from_str::<AuthErrorResponse>(&body) {
                Err(NetDiskError::auth_error(&auth_error.error_description))
            } else {
                Err(NetDiskError::http_error(status.as_u16(), &url))
            }
        }
    }
}
