//! # zero-net
//!
//! 网络栈 — 基于 reqwest 的 HTTP/HTTPS 请求封装。
//!
//! 提供 URL 解析、HTTP 客户端、导航历史和 Cookie 管理功能。

#![warn(missing_docs)]

pub mod url_parser;
pub mod request;
pub mod client;
pub mod navigation;
pub mod cookie;

pub use url_parser::*;
pub use request::*;
pub use client::*;
pub use navigation::*;
pub use cookie::*;

use thiserror::Error;

/// 网络错误类型。
#[derive(Error, Debug)]
pub enum NetError {
    /// URL 解析错误。
    #[error("URL parse error: {0}")]
    UrlParse(String),
    /// HTTP 错误。
    #[error("HTTP error: {0}")]
    Http(String),
    /// 网络连接错误。
    #[error("Network error: {0}")]
    Network(String),
    /// 请求超时。
    #[error("Timeout")]
    Timeout,
    /// 重定向次数超出限制。
    #[error("Redirect limit exceeded")]
    TooManyRedirects,
    /// 无效的 Cookie。
    #[error("Invalid cookie: {0}")]
    InvalidCookie(String),
}

impl From<url::ParseError> for NetError {
    fn from(e: url::ParseError) -> Self {
        NetError::UrlParse(e.to_string())
    }
}
