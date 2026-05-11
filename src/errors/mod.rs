/// Error handling module for Baidu NetDisk SDK
///
/// This module provides error types and utilities for handling API errors.
pub mod errno;

use thiserror::Error;

/// SDK error types
#[derive(Error, Debug)]
pub enum NetDiskError {
    /// HTTP request error
    #[error("HTTP request error: {status_code} - {url}")]
    HttpError {
        status_code: u16,
        url: String,
        source: Option<reqwest::Error>,
    },

    /// API business error
    #[error("API error: {errno} - {errmsg}")]
    ApiError { errno: i32, errmsg: String },

    /// Authentication error
    #[error("Authentication error: {description}")]
    AuthError { description: String },

    /// File already exists
    #[error("File already exists: {path}")]
    FileExists { path: String },

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Directory not found
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// URL encoding error
    #[error("URL encoding error: {0}")]
    UrlEncodeError(String),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Request timeout
    #[error("Request timed out")]
    Timeout,

    /// Request cancelled
    #[error("Request cancelled")]
    Cancelled,

    /// Invalid parameter
    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Synchronization primitive error
    #[error("Synchronization primitive error: {0}")]
    SyncError(String),

    /// Invalid header value
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),

    /// Reqwest HTTP client error
    #[error("HTTP client error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// MPSC send error
    #[error("MPSC send error: {0}")]
    MpscSendError(String),

    /// Join error (task join error)
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    /// Unknown error
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl NetDiskError {
    /// Create HTTP error
    pub fn http_error(status_code: u16, url: &str) -> Self {
        NetDiskError::HttpError {
            status_code,
            url: url.to_string(),
            source: None,
        }
    }

    /// Create HTTP error with source
    pub fn http_error_with_source(status_code: u16, url: &str, source: reqwest::Error) -> Self {
        NetDiskError::HttpError {
            status_code,
            url: url.to_string(),
            source: Some(source),
        }
    }

    /// Create API error with standard error description
    pub fn api_error(errno: i32, errmsg: &str) -> Self {
        let description = errno::get_error_description(errno).unwrap_or(errmsg);

        NetDiskError::ApiError {
            errno,
            errmsg: description.to_string(),
        }
    }

    /// Create API error with raw error message
    pub fn api_error_raw(errno: i32, errmsg: &str) -> Self {
        NetDiskError::ApiError {
            errno,
            errmsg: errmsg.to_string(),
        }
    }

    /// Create authentication error
    pub fn auth_error(description: &str) -> Self {
        NetDiskError::AuthError {
            description: description.to_string(),
        }
    }

    /// Create invalid parameter error
    pub fn invalid_parameter(message: &str) -> Self {
        NetDiskError::InvalidParameter {
            message: message.to_string(),
        }
    }

    /// Create file exists error
    pub fn file_exists(path: &str) -> Self {
        NetDiskError::FileExists {
            path: path.to_string(),
        }
    }

    /// Create file not found error
    pub fn file_not_found(path: &str) -> Self {
        NetDiskError::FileNotFound {
            path: path.to_string(),
        }
    }

    /// Create directory not found error
    pub fn directory_not_found(path: &str) -> Self {
        NetDiskError::DirectoryNotFound {
            path: path.to_string(),
        }
    }

    /// Check if error is authentication related
    pub fn is_auth_error(&self) -> bool {
        match self {
            NetDiskError::ApiError { errno, .. } => errno::is_auth_error(*errno),
            NetDiskError::TokenExpired => true,
            NetDiskError::AuthError { .. } => true,
            _ => false,
        }
    }

    /// Check if error indicates not found
    pub fn is_not_found_error(&self) -> bool {
        match self {
            NetDiskError::ApiError { errno, .. } => errno::is_not_found_error(*errno),
            NetDiskError::FileNotFound { .. } => true,
            NetDiskError::DirectoryNotFound { .. } => true,
            _ => false,
        }
    }

    /// Check if error is permission related
    pub fn is_permission_error(&self) -> bool {
        match self {
            NetDiskError::ApiError { errno, .. } => errno::is_permission_error(*errno),
            _ => false,
        }
    }

    /// Check if error is quota related
    pub fn is_quota_error(&self) -> bool {
        match self {
            NetDiskError::ApiError { errno, .. } => errno::is_quota_error(*errno),
            _ => false,
        }
    }

    /// Check if error indicates token expired
    pub fn is_token_expired(&self) -> bool {
        match self {
            NetDiskError::ApiError { errno, .. } => errno::is_token_expired(*errno),
            NetDiskError::TokenExpired => true,
            _ => false,
        }
    }
}

/// SDK result type
pub type NetDiskResult<T> = Result<T, NetDiskError>;
