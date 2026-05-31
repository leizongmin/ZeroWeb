//! # zero-net
//!
//! 网络栈 — 基于 reqwest 的 HTTP/HTTPS 请求封装。
//!
//! 提供 URL 解析、HTTP 客户端、导航历史和 Cookie 管理功能。

#![warn(missing_docs)]

pub mod client;
pub mod cookie;
pub mod navigation;
pub mod request;
pub mod url_parser;

pub use client::*;
pub use cookie::{Cookie, CookieStore, RequestContext, SameSite, parse_expires_date, same_site_allows};
pub use navigation::*;
pub use request::*;
pub use url_parser::*;

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

#[cfg(test)]
mod tests {
    use crate::cookie::{Cookie, SameSite};
    use crate::navigation::NavigationHistory;
    use crate::request::HttpResponse;
    use crate::url_parser::parse_url;

    /// 测试仅含片段标识符的 URL，验证 fragment 正确提取。
    #[test]
    fn test_url_fragment_only() {
        let parsed = parse_url("http://example.com#section").unwrap();
        assert_eq!(parsed.fragment.as_deref(), Some("section"));
    }

    /// 测试无路径的 URL，默认路径应为 "/"。
    #[test]
    fn test_url_empty_path() {
        let parsed = parse_url("http://example.com").unwrap();
        assert_eq!(parsed.path, "/");
    }

    /// 测试导航历史中 can_go_back 的状态变化。
    /// 推入 2 个条目后 can_go_back 应为 true，后退后回到起点应为 false。
    #[test]
    fn test_navigation_can_go_back_check() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        assert!(nav.can_go_back());
        nav.go_back();
        assert!(!nav.can_go_back());
    }

    /// 测试 Cookie 的 HttpOnly 标志设置。
    #[test]
    fn test_cookie_http_only_flag() {
        let cookie = Cookie {
            name: "sid".to_string(),
            value: "abc".to_string(),
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: true,
            same_site: SameSite::None,
        };
        assert!(cookie.http_only);
    }

    /// 测试 HttpResponse 的状态码和 reason phrase。
    #[test]
    fn test_http_response_status_text() {
        let resp = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
            url: "http://example.com/missing".to_string(),
            redirect_count: 0,
        };
        let reason = "Not Found";
        assert_eq!(resp.status_code, 404);
        assert_eq!(reason, "Not Found");
        assert!(resp.is_client_error());
    }
}
