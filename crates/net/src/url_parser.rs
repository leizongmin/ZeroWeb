//! URL 解析与操作模块。
//!
//! 基于 `url` crate 封装，提供 URL 解析和同源判断功能。

use url::Url;

use crate::NetError;

/// URL 解析结果。
#[derive(Debug, Clone)]
pub struct ParsedUrl {
    /// 协议（http, https 等）。
    pub scheme: String,
    /// 主机名。
    pub host: Option<String>,
    /// 端口号。
    pub port: Option<u16>,
    /// 路径部分。
    pub path: String,
    /// 查询字符串。
    pub query: Option<String>,
    /// 片段标识符。
    pub fragment: Option<String>,
    /// 用户名。
    pub username: String,
    /// 密码。
    pub password: Option<String>,
}

/// 解析 URL 字符串。
///
/// # 错误
///
/// 如果 URL 格式无效，返回 `NetError::UrlParse`。
pub fn parse_url(url_str: &str) -> Result<ParsedUrl, NetError> {
    let url = Url::parse(url_str)?;
    Ok(ParsedUrl {
        scheme: url.scheme().to_string(),
        host: url.host_str().map(|h| h.to_string()),
        port: url.port(),
        path: url.path().to_string(),
        query: url.query().map(|q| q.to_string()),
        fragment: url.fragment().map(|f| f.to_string()),
        username: url.username().to_string(),
        password: url.password().map(|p| p.to_string()),
    })
}

impl ParsedUrl {
    /// 获取 origin（scheme + host + port）。
    ///
    /// 如果 port 是默认端口则省略。
    pub fn origin(&self) -> String {
        let host = self.host.as_deref().unwrap_or("");
        match self.port {
            Some(port) => {
                // 如果是默认端口，省略
                let is_default = (self.scheme == "http" && port == 80) || (self.scheme == "https" && port == 443);
                if is_default {
                    format!("{}://{}", self.scheme, host)
                } else {
                    format!("{}://{}:{}", self.scheme, host, port)
                }
            }
            None => format!("{}://{}", self.scheme, host),
        }
    }

    /// 是否为 HTTPS 安全连接。
    pub fn is_secure(&self) -> bool {
        self.scheme == "https"
    }

    /// 是否同源。
    pub fn is_same_origin(&self, other: &ParsedUrl) -> bool {
        self.origin() == other.origin()
    }

    /// 获取完整 URL 字符串（不含 fragment）。
    pub fn to_url_string(&self) -> String {
        let mut result = format!("{}://", self.scheme);
        if !self.username.is_empty() {
            result.push_str(&self.username);
            if let Some(ref pw) = self.password {
                result.push(':');
                result.push_str(pw);
            }
            result.push('@');
        }
        if let Some(ref host) = self.host {
            result.push_str(host);
        }
        if let Some(port) = self.port {
            let is_default = (self.scheme == "http" && port == 80) || (self.scheme == "https" && port == 443);
            if !is_default {
                result.push(':');
                result.push_str(&port.to_string());
            }
        }
        result.push_str(&self.path);
        if let Some(ref query) = self.query {
            result.push('?');
            result.push_str(query);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let parsed = parse_url("http://example.com/path").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));
        assert_eq!(parsed.path, "/path");
        assert!(parsed.port.is_none());
        assert!(parsed.query.is_none());
        assert!(parsed.fragment.is_none());
    }

    #[test]
    fn test_parse_https_url() {
        let parsed = parse_url("https://secure.example.com/").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host.as_deref(), Some("secure.example.com"));
        assert!(parsed.is_secure());
    }

    #[test]
    fn test_parse_url_with_port() {
        let parsed = parse_url("http://example.com:8080/resource").unwrap();
        assert_eq!(parsed.port, Some(8080));
    }

    #[test]
    fn test_parse_url_with_query() {
        let parsed = parse_url("http://example.com/search?q=hello&lang=en").unwrap();
        assert_eq!(parsed.query.as_deref(), Some("q=hello&lang=en"));
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let parsed = parse_url("http://example.com/page#section1").unwrap();
        assert_eq!(parsed.fragment.as_deref(), Some("section1"));
    }

    #[test]
    fn test_parse_url_invalid() {
        assert!(parse_url("not a url at all").is_err());
    }

    #[test]
    fn test_url_origin() {
        let parsed = parse_url("https://example.com:443/path").unwrap();
        assert_eq!(parsed.origin(), "https://example.com");

        let parsed_port = parse_url("https://example.com:8443/path").unwrap();
        assert_eq!(parsed_port.origin(), "https://example.com:8443");
    }

    #[test]
    fn test_url_is_secure() {
        let http = parse_url("http://example.com").unwrap();
        let https = parse_url("https://example.com").unwrap();
        assert!(!http.is_secure());
        assert!(https.is_secure());
    }

    #[test]
    fn test_parse_url_with_credentials() {
        let parsed = parse_url("http://user:pass@example.com/path").unwrap();
        assert_eq!(parsed.username, "user");
        assert_eq!(parsed.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_parse_url_with_default_http_port() {
        // url crate 对默认端口返回 None（即使用户显式写了 :80）
        let parsed = parse_url("http://example.com:80/").unwrap();
        assert!(parsed.port.is_none()); // 80 是 http 默认端口，被 url crate 规范化
        assert_eq!(parsed.origin(), "http://example.com");
    }

    #[test]
    fn test_parse_url_empty_string() {
        assert!(parse_url("").is_err());
    }

    #[test]
    fn test_url_is_same_origin_positive() {
        let a = parse_url("https://example.com/page1").unwrap();
        let b = parse_url("https://example.com/page2?q=1").unwrap();
        assert!(a.is_same_origin(&b));
    }

    #[test]
    fn test_url_is_same_origin_negative() {
        let a = parse_url("https://example.com").unwrap();
        let b = parse_url("https://other.com").unwrap();
        assert!(!a.is_same_origin(&b));
    }

    #[test]
    fn test_url_to_url_string_basic() {
        let parsed = parse_url("http://example.com/path?q=1").unwrap();
        let url_str = parsed.to_url_string();
        assert!(url_str.starts_with("http://example.com"));
        assert!(url_str.contains("/path"));
        assert!(url_str.contains("q=1"));
    }

    #[test]
    fn test_url_to_url_string_with_credentials() {
        let parsed = parse_url("http://user:pass@example.com/path").unwrap();
        let url_str = parsed.to_url_string();
        assert!(url_str.contains("user:pass@"));
    }

    #[test]
    fn test_url_to_url_string_excludes_fragment() {
        let parsed = parse_url("http://example.com/path#section").unwrap();
        let url_str = parsed.to_url_string();
        assert!(!url_str.contains('#'));
    }

    // ── 边界条件补充测试 ──

    /// 测试默认 HTTPS 端口（443）被规范化。
    #[test]
    fn test_parse_url_default_https_port() {
        let parsed = parse_url("https://example.com:443/").unwrap();
        assert!(parsed.port.is_none(), "443 是 https 默认端口，应被规范化为 None");
        assert_eq!(parsed.origin(), "https://example.com");
    }

    /// 测试非标准协议（ftp、data、file）。
    #[test]
    fn test_parse_url_non_standard_schemes() {
        let ftp = parse_url("ftp://files.example.com/pub").unwrap();
        assert_eq!(ftp.scheme, "ftp");

        let data = parse_url("data:text/plain,hello").unwrap();
        assert_eq!(data.scheme, "data");

        let file = parse_url("file:///etc/hosts").unwrap();
        assert_eq!(file.scheme, "file");
    }

    /// 测试同源不同端口不匹配。
    #[test]
    fn test_url_same_origin_different_port() {
        let a = parse_url("http://example.com:8080/page1").unwrap();
        let b = parse_url("http://example.com:9090/page2").unwrap();
        assert!(!a.is_same_origin(&b), "不同端口应不同源");
    }

    /// 测试 to_url_string 最小 URL（无查询、无片段）。
    #[test]
    fn test_url_to_url_string_minimal() {
        let parsed = parse_url("http://example.com/path").unwrap();
        let url_str = parsed.to_url_string();
        assert_eq!(url_str, "http://example.com/path");
    }

    /// 测试只有用户名无密码。
    #[test]
    fn test_parse_url_username_only() {
        let parsed = parse_url("http://user@example.com/path").unwrap();
        assert_eq!(parsed.username, "user");
        assert!(parsed.password.is_none());
    }

    /// 测试查询字符串含特殊字符。
    #[test]
    fn test_parse_url_special_query_chars() {
        let parsed = parse_url("http://example.com/search?q=hello%20world&lang=%E4%B8%AD").unwrap();
        assert!(parsed.query.is_some());
        assert!(parsed.query.as_ref().unwrap().contains("q="));
    }

    /// 测试路径含特殊编码字符。
    #[test]
    fn test_parse_url_encoded_path() {
        let parsed = parse_url("http://example.com/%E8%B7%AF%E5%BE%84/page").unwrap();
        assert!(parsed.path.contains("page"));
    }

    /// 测试 IPv6 地址 URL。
    #[test]
    fn test_parse_url_ipv6() {
        let parsed = parse_url("http://[::1]:8080/path").unwrap();
        assert!(parsed.host.is_some());
        assert_eq!(parsed.port, Some(8080));
    }

    /// 测试根路径 URL。
    #[test]
    fn test_parse_url_root_path() {
        let parsed = parse_url("http://example.com").unwrap();
        assert_eq!(parsed.path, "/");
    }

    /// 测试 to_url_string 不含默认端口。
    #[test]
    fn test_url_to_url_string_no_default_port() {
        let parsed = parse_url("http://example.com/path").unwrap();
        let url_str = parsed.to_url_string();
        assert!(!url_str.contains(":80"), "默认端口不应出现在 URL 字符串中");
    }

    // ── URL edge case tests ──

    /// 测试 URL 同时包含 query 和 fragment。
    #[test]
    fn test_parse_url_query_and_fragment() {
        let parsed = parse_url("http://example.com/page?key=val#section").unwrap();
        assert_eq!(parsed.query.as_deref(), Some("key=val"));
        assert_eq!(parsed.fragment.as_deref(), Some("section"));
    }

    /// 测试 URL 包含非默认端口。
    #[test]
    fn test_parse_url_explicit_port() {
        let parsed = parse_url("http://example.com:3000/api").unwrap();
        assert_eq!(parsed.port, Some(3000));
        assert_eq!(parsed.path, "/api");
    }

    /// 测试 URL 同时包含 userinfo 和端口。
    #[test]
    fn test_parse_url_userinfo_with_port() {
        let parsed = parse_url("https://admin:secret@api.example.com:9090/v2/data").unwrap();
        assert_eq!(parsed.username, "admin");
        assert_eq!(parsed.password.as_deref(), Some("secret"));
        assert_eq!(parsed.host.as_deref(), Some("api.example.com"));
        assert_eq!(parsed.port, Some(9090));
        assert_eq!(parsed.path, "/v2/data");
        assert!(parsed.is_secure());
    }

    /// 测试 percent-encoded 字符在 URL 各部分中正确保留。
    #[test]
    fn test_parse_url_percent_encoded() {
        let parsed = parse_url("http://example.com/%E4%BD%A0%E5%A5%BD?q=%E4%B8%AD%E6%96%87").unwrap();
        assert!(parsed.path.contains("%"));
        assert!(parsed.query.as_ref().unwrap().contains("%"));
    }

    /// 测试 data: URL scheme。
    #[test]
    fn test_parse_url_data_scheme() {
        let parsed = parse_url("data:text/html,<h1>Hello</h1>").unwrap();
        assert_eq!(parsed.scheme, "data");
        assert!(parsed.host.is_none());
    }

    /// 测试 javascript: URL scheme。
    #[test]
    fn test_parse_url_javascript_scheme() {
        let parsed = parse_url("javascript:alert(1)").unwrap();
        assert_eq!(parsed.scheme, "javascript");
    }

    /// 测试 is_secure 对非 https 协议返回 false。
    #[test]
    fn test_url_is_secure_various_schemes() {
        let ftp = parse_url("ftp://files.example.com/").unwrap();
        assert!(!ftp.is_secure());
        let data = parse_url("data:text/plain,hello").unwrap();
        assert!(!data.is_secure());
    }

    /// 测试同源判断：不同 scheme 不匹配。
    #[test]
    fn test_url_same_origin_different_scheme() {
        let http = parse_url("http://example.com/").unwrap();
        let https = parse_url("https://example.com/").unwrap();
        assert!(!http.is_same_origin(&https));
    }

    /// 测试 to_url_string 包含非默认端口。
    #[test]
    fn test_url_to_url_string_with_non_default_port() {
        let parsed = parse_url("http://example.com:8080/api").unwrap();
        let url_str = parsed.to_url_string();
        assert!(url_str.contains(":8080"), "非默认端口应出现在 URL 字符串中");
    }

    /// 测试解析 host 为空的 data URL 不 panic。
    #[test]
    fn test_url_origin_hostless() {
        let parsed = parse_url("data:text/plain,hello").unwrap();
        // host 为空，origin 不应 panic
        let origin = parsed.origin();
        assert!(origin.contains("data://"));
    }

    // ── 高优先级边界条件测试 ──

    /// 测试 URL 同时包含 userinfo、非默认端口、query（含特殊字符）和 fragment。
    /// 验证各部分在组合场景下均正确解析。
    #[test]
    fn test_parse_url_userinfo_port_query_fragment_combined() {
        let parsed =
            parse_url("https://admin:p%40ssw0rd@api.example.com:9090/v2/users?name=foo%20bar&ids=1%262%3D3#results")
                .unwrap();
        assert_eq!(parsed.username, "admin");
        assert_eq!(parsed.password.as_deref(), Some("p%40ssw0rd"));
        assert_eq!(parsed.host.as_deref(), Some("api.example.com"));
        assert_eq!(parsed.port, Some(9090));
        assert_eq!(parsed.path, "/v2/users");
        assert_eq!(parsed.query.as_deref(), Some("name=foo%20bar&ids=1%262%3D3"));
        assert_eq!(parsed.fragment.as_deref(), Some("results"));
        assert!(parsed.is_secure());
    }

    /// 测试 query 中包含多种特殊字符（% encoded、+、&、=、# encoded）。
    #[test]
    fn test_parse_url_query_special_chars() {
        let parsed = parse_url("http://example.com/search?q=%E4%B8%AD%E6%96%87&r=1%2B2%3D3&x=a%26b%23c").unwrap();
        let query = parsed.query.as_deref().unwrap();
        assert!(query.contains("q=%E4%B8%AD%E6%96%87"), "percent-encoded CJK");
        assert!(query.contains("r=1%2B2%3D3"), "percent-encoded + and =");
        assert!(query.contains("x=a%26b%23c"), "percent-encoded & and #");
    }

    /// 测试 to_url_string 在完整组合（userinfo + 非默认端口 + query）下的输出。
    #[test]
    fn test_url_to_url_string_full_roundtrip() {
        let parsed = parse_url("http://user:pass@host.com:8080/api?key=val#anchor").unwrap();
        let url_str = parsed.to_url_string();
        // to_url_string 不含 fragment
        assert!(url_str.contains("user:pass@host.com:8080"));
        assert!(url_str.contains("/api?key=val"));
        assert!(!url_str.contains('#'), "fragment 不应出现在 to_url_string");
    }
}
