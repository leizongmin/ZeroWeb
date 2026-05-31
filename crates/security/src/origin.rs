//! 源（Origin）和同源策略模块。
//!
//! 提供源解析和同源判断功能。

use url::Url;

use crate::SecurityError;

/// 源（Origin）— 协议 + 主机 + 端口。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    /// 协议（scheme），如 http、https。
    pub scheme: String,
    /// 主机名。
    pub host: String,
    /// 端口号。
    pub port: u16,
}

impl Origin {
    /// 从 `Url` 解析 Origin。
    pub fn from_url(url: &Url) -> Result<Self, SecurityError> {
        let host = url
            .host_str()
            .ok_or_else(|| SecurityError::OriginParse("no host in URL".to_string()))?;

        let scheme = url.scheme().to_string();
        let port = url
            .port_or_known_default()
            .ok_or_else(|| SecurityError::OriginParse(format!("unknown scheme: {scheme}")))?;

        Ok(Self {
            scheme,
            host: host.to_string(),
            port,
        })
    }

    /// 从字符串解析 Origin。
    pub fn parse(url_str: &str) -> Result<Self, SecurityError> {
        let url = Url::parse(url_str).map_err(|e| SecurityError::OriginParse(e.to_string()))?;
        Self::from_url(&url)
    }

    /// 是否为相同源。
    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }

    /// 是否为安全上下文（HTTPS）。
    pub fn is_secure(&self) -> bool {
        self.scheme == "https"
    }
}

/// 同源策略检查。
pub fn check_same_origin(origin_a: &Origin, origin_b: &Origin) -> bool {
    origin_a.is_same_origin(origin_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_from_url() {
        let url = Url::parse("https://example.com/path").unwrap();
        let origin = Origin::from_url(&url).unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, 443);
    }

    #[test]
    fn test_origin_same_origin() {
        let a = Origin::parse("https://example.com/page1").unwrap();
        let b = Origin::parse("https://example.com/page2").unwrap();
        assert!(a.is_same_origin(&b));
        assert!(check_same_origin(&a, &b));
    }

    #[test]
    fn test_origin_different_scheme() {
        let a = Origin::parse("https://example.com").unwrap();
        let b = Origin::parse("http://example.com").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_origin_different_host() {
        let a = Origin::parse("https://a.com").unwrap();
        let b = Origin::parse("https://b.com").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_origin_different_port() {
        let a = Origin::parse("http://example.com").unwrap();
        let b = Origin::parse("http://example.com:8080").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_origin_is_secure() {
        let https = Origin::parse("https://example.com").unwrap();
        let http = Origin::parse("http://example.com").unwrap();
        assert!(https.is_secure());
        assert!(!http.is_secure());
    }

    #[test]
    fn test_origin_parse_invalid() {
        assert!(Origin::parse("not-a-url").is_err());
    }

    #[test]
    fn test_origin_parse_custom_port() {
        let origin = Origin::parse("https://example.com:8443").unwrap();
        assert_eq!(origin.port, 8443);
    }

    #[test]
    fn test_origin_parse_default_port_normalization() {
        let explicit = Origin::parse("http://example.com:80").unwrap();
        let implicit = Origin::parse("http://example.com").unwrap();
        assert_eq!(explicit, implicit);
        assert_eq!(implicit.port, 80);
    }

    #[test]
    fn test_origin_parse_empty_string() {
        assert!(Origin::parse("").is_err());
    }

    #[test]
    fn test_origin_from_url_no_host() {
        let url = Url::parse("file:///etc/passwd").unwrap();
        let result = Origin::from_url(&url);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no host"), "message: {msg}");
    }

    #[test]
    fn test_origin_from_url_unknown_scheme() {
        let url = Url::parse("myproto://example.com").unwrap();
        let result = Origin::from_url(&url);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("unknown scheme"), "message: {msg}");
    }

    #[test]
    fn test_origin_parse_data_url() {
        let result = Origin::parse("data:text/plain,hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_same_origin_returns_false() {
        let a = Origin::parse("https://a.com").unwrap();
        let b = Origin::parse("https://b.com").unwrap();
        assert!(!check_same_origin(&a, &b));
    }

    #[test]
    fn test_origin_hash_and_eq() {
        use std::collections::HashSet;
        let a = Origin::parse("https://example.com/page1").unwrap();
        let b = Origin::parse("https://example.com/page2").unwrap();
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
    }

    // ---- 同源检查：显式默认端口与不同方案 ----

    #[test]
    fn test_same_origin_https_explicit_default_port() {
        // https://example.com:443 与 https://example.com 应为同源
        let a = Origin::parse("https://example.com").unwrap();
        let b = Origin::parse("https://example.com:443").unwrap();
        assert!(check_same_origin(&a, &b), "https 默认端口 443 应视为同源");
    }

    #[test]
    fn test_same_origin_http_explicit_default_port() {
        // http://example.com:80 与 http://example.com 应为同源
        let a = Origin::parse("http://example.com").unwrap();
        let b = Origin::parse("http://example.com:80").unwrap();
        assert!(check_same_origin(&a, &b), "http 默认端口 80 应视为同源");
    }

    #[test]
    fn test_not_same_origin_different_scheme_same_port() {
        // http://example.com:443 与 https://example.com:443 不是同源（不同协议）
        let a = Origin::parse("http://example.com:443").unwrap();
        let b = Origin::parse("https://example.com:443").unwrap();
        assert!(!check_same_origin(&a, &b), "不同协议不是同源");
    }

    #[test]
    fn test_not_same_origin_different_port() {
        // http://example.com:80 与 http://example.com:8080 不是同源
        let a = Origin::parse("http://example.com").unwrap();
        let b = Origin::parse("http://example.com:8080").unwrap();
        assert!(!check_same_origin(&a, &b), "不同端口不是同源");
    }
}
