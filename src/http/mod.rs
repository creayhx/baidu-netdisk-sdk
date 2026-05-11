//! HTTP client module
//!
//! Encapsulates HTTP requests with retry, timeout, logging, etc.
//!
//! # Features
//!
//! - Automatic retry for network errors and server errors
//! - Configurable timeouts and retry delays
//! - JSON request/response handling
//! - Form and multipart file upload support
//! - Detailed logging for debugging
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::http::{HttpClient, HttpClientConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client with default config
//! let client = HttpClient::try_default()?;
//!
//! // Or with custom config
//! let config = HttpClientConfig {
//!     timeout: std::time::Duration::from_secs(60),
//!     ..Default::default()
//! };
//! let client = HttpClient::new(config)?;
//! # Ok(())
//! # }
//! ```

pub mod client;

pub use self::client::{HttpClient, HttpClientConfig};
