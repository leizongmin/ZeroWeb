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

    /// Origin::parse("null") 应返回错误（opaque origin），而非 panic
    #[test]
    fn test_origin_parse_null_opaque() {
        let result = Origin::parse("null");
        // "null" 不是合法 URL，解析应失败
        assert!(result.is_err(), "parse(\"null\") 应返回错误（opaque origin）");
    }

    /// Origin::parse("not-a-url") 应返回错误
    #[test]
    fn test_origin_parse_invalid_url() {
        let result = Origin::parse("not-a-url");
        assert!(result.is_err(), "无效 URL 字符串应解析失败");
        let msg = result.unwrap_err().to_string();
        // 错误信息应包含有意义的内容
        assert!(!msg.is_empty(), "错误信息不应为空");
    }

    /// 同域名不同端口不应视为同源
    #[test]
    fn test_origin_equality_different_ports() {
        let a = Origin::parse("https://example.com:3000").unwrap();
        let b = Origin::parse("https://example.com:4000").unwrap();
        assert_ne!(a, b, "不同端口的 Origin 不应相等");
        assert!(!a.is_same_origin(&b), "不同端口不应为同源");
        assert!(!check_same_origin(&a, &b), "check_same_origin 对不同端口应返回 false");
    }

    /// 测试相同 scheme + host + port 的 Origin 应判定为相等（tuple equality）。
    #[test]
    fn test_origin_tuple_equality() {
        // 不同 URL 路径 → 相同 origin
        let a = Origin::parse("https://example.com/page1").unwrap();
        let b = Origin::parse("https://example.com/page2").unwrap();
        assert_eq!(a, b, "相同 scheme+host+port 的 Origin 应相等");
        assert!(a.is_same_origin(&b));
        assert!(check_same_origin(&a, &b));

        // 相同的完整 URL → 相等
        let c = Origin::parse("https://example.com").unwrap();
        assert_eq!(a, c);

        // 显式指定默认端口 → 仍相等
        let d = Origin::parse("https://example.com:443/page3").unwrap();
        assert_eq!(a, d, "显式默认端口应与隐式默认端口相等");
        assert!(a.is_same_origin(&d));

        // HTTP 默认端口 80
        let e = Origin::parse("http://example.com").unwrap();
        let f = Origin::parse("http://example.com:80").unwrap();
        assert_eq!(e, f, "http 默认端口 80 应相等");

        // 验证三个字段分别不同的比较
        let g = Origin::parse("http://other.com").unwrap();
        assert_ne!(a, g, "不同 host 的 Origin 不应相等");
        assert!(!a.is_same_origin(&g));
    }

    // ── 边界测试（round 23）──

    /// 测试同源策略：http 默认端口 80 与 https 默认端口 443 的规范化。
    ///
    /// 当 URL 不显式指定端口时，http 默认 80，https 默认 443。
    /// 显式指定默认端口与不指定应产生相同的 Origin（同源）。
    #[test]
    fn test_same_origin_default_port_normalization() {
        // http: 默认端口规范化
        let http_implicit = Origin::parse("http://example.com").unwrap();
        let http_explicit = Origin::parse("http://example.com:80").unwrap();
        assert_eq!(http_implicit.port, 80);
        assert_eq!(http_explicit.port, 80);
        assert!(
            http_implicit.is_same_origin(&http_explicit),
            "http 默认端口应规范化为同源"
        );

        // https: 默认端口规范化
        let https_implicit = Origin::parse("https://example.com").unwrap();
        let https_explicit = Origin::parse("https://example.com:443").unwrap();
        assert_eq!(https_implicit.port, 443);
        assert_eq!(https_explicit.port, 443);
        assert!(
            https_implicit.is_same_origin(&https_explicit),
            "https 默认端口应规范化为同源"
        );

        // http:80 与 https:443 不同源
        assert!(
            !http_explicit.is_same_origin(&https_explicit),
            "http:80 与 https:443 不是同源"
        );

        // 显式非默认端口不与默认端口同源
        let http_8080 = Origin::parse("http://example.com:8080").unwrap();
        assert!(
            !http_implicit.is_same_origin(&http_8080),
            "http:80 与 http:8080 不是同源"
        );
    }
}
