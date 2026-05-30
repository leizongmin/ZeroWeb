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
                let is_default = (self.scheme == "http" && port == 80)
                    || (self.scheme == "https" && port == 443);
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
            let is_default = (self.scheme == "http" && port == 80)
                || (self.scheme == "https" && port == 443);
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
}
