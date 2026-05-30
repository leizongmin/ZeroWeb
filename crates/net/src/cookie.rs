//! Cookie 管理模块。
//!
//! 提供 HTTP Cookie 解析、存储和匹配功能。

use crate::NetError;
use crate::url_parser::ParsedUrl;

/// SameSite 属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// 无限制。
    None,
    /// Lax 模式。
    Lax,
    /// Strict 模式。
    Strict,
}

/// HTTP Cookie。
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie 名称。
    pub name: String,
    /// Cookie 值。
    pub value: String,
    /// 域名。
    pub domain: Option<String>,
    /// 路径。
    pub path: Option<String>,
    /// 过期时间。
    pub expires: Option<String>,
    /// 是否仅 HTTPS。
    pub secure: bool,
    /// 是否仅 HTTP（禁止 JS 访问）。
    pub http_only: bool,
    /// SameSite 策略。
    pub same_site: SameSite,
}

/// Cookie 存储。
pub struct CookieStore {
    cookies: Vec<Cookie>,
}

impl CookieStore {
    /// 创建空的 Cookie 存储。
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    /// 解析 Set-Cookie header 值。
    ///
    /// 格式：`name=value; Path=/; Secure; HttpOnly; SameSite=Lax`
    pub fn parse_set_cookie(header_value: &str) -> Result<Cookie, NetError> {
        let parts: Vec<&str> = header_value.split(';').collect();
        if parts.is_empty() {
            return Err(NetError::InvalidCookie("empty cookie".to_string()));
        }

        // 解析 name=value
        let first = parts[0].trim();
        let eq_pos = first
            .find('=')
            .ok_or_else(|| NetError::InvalidCookie("no '=' in cookie".to_string()))?;
        let name = first[..eq_pos].trim().to_string();
        let value = first[eq_pos + 1..].trim().to_string();

        if name.is_empty() {
            return Err(NetError::InvalidCookie("empty cookie name".to_string()));
        }

        let mut cookie = Cookie {
            name,
            value,
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: SameSite::None,
        };

        // 解析属性
        for part in parts.iter().skip(1) {
            let part = part.trim();
            if part.eq_ignore_ascii_case("secure") {
                cookie.secure = true;
            } else if part.eq_ignore_ascii_case("httponly") {
                cookie.http_only = true;
            } else if let Some(val) = part.strip_prefix("Path=") {
                cookie.path = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("path=") {
                cookie.path = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("Domain=") {
                cookie.domain = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("domain=") {
                cookie.domain = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("Expires=") {
                cookie.expires = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("expires=") {
                cookie.expires = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("Max-Age=") {
                cookie.expires = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("max-age=") {
                cookie.expires = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("SameSite=") {
                cookie.same_site = match val.trim() {
                    "Strict" => SameSite::Strict,
                    "Lax" => SameSite::Lax,
                    _ => SameSite::None,
                };
            } else if let Some(val) = part.strip_prefix("samesite=") {
                cookie.same_site = match val.trim() {
                    "strict" => SameSite::Strict,
                    "lax" => SameSite::Lax,
                    _ => SameSite::None,
                };
            }
        }

        Ok(cookie)
    }

    /// 添加 cookie。
    pub fn add(&mut self, cookie: Cookie) {
        // 如果同名同 domain 同 path，替换旧值
        self.cookies.retain(|c| {
            !(c.name == cookie.name && c.domain == cookie.domain && c.path == cookie.path)
        });
        self.cookies.push(cookie);
    }

    /// 获取匹配 URL 的所有 cookies。
    pub fn get_for_url(&self, url: &ParsedUrl) -> Vec<&Cookie> {
        self.cookies
            .iter()
            .filter(|c| cookie_matches_url(c, url))
            .collect()
    }

    /// 生成 Cookie header 值。
    pub fn cookie_header(&self, url: &ParsedUrl) -> String {
        self.get_for_url(url)
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// 清除所有 cookies。
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Cookie 总数。
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Cookie 存储是否为空。
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查 cookie 是否匹配给定 URL。
fn cookie_matches_url(cookie: &Cookie, url: &ParsedUrl) -> bool {
    // Secure cookie 只能用于 HTTPS
    if cookie.secure && url.scheme != "https" {
        return false;
    }

    // 域名匹配
    if let Some(ref domain) = cookie.domain {
        let host = url.host.as_deref().unwrap_or("");
        if !domain_matches(domain, host) {
            return false;
        }
    }

    // 路径匹配
    if let Some(ref cookie_path) = cookie.path
        && !url.path.starts_with(cookie_path)
    {
        return false;
    }

    true
}

/// 检查域名是否匹配（支持子域名匹配）。
fn domain_matches(cookie_domain: &str, host: &str) -> bool {
    if cookie_domain.is_empty() {
        return true;
    }

    let cookie_domain = cookie_domain.trim_start_matches('.');
    let host = host.trim_start_matches('.');

    // 精确匹配
    if host.eq_ignore_ascii_case(cookie_domain) {
        return true;
    }

    // 子域名匹配：host 是 cookie_domain 的子域名
    if host
        .to_ascii_lowercase()
        .ends_with(&format!(".{}", cookie_domain.to_ascii_lowercase()))
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url_parser::parse_url;

    #[test]
    fn test_parse_simple_cookie() {
        let cookie = CookieStore::parse_set_cookie("session=abc123").unwrap();
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
    }

    #[test]
    fn test_parse_cookie_with_attributes() {
        let cookie = CookieStore::parse_set_cookie("id=42; Path=/app; Domain=example.com").unwrap();
        assert_eq!(cookie.name, "id");
        assert_eq!(cookie.value, "42");
        assert_eq!(cookie.path.as_deref(), Some("/app"));
        assert_eq!(cookie.domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_parse_cookie_secure() {
        let cookie = CookieStore::parse_set_cookie("token=xyz; Secure").unwrap();
        assert!(cookie.secure);
    }

    #[test]
    fn test_parse_cookie_httponly() {
        let cookie = CookieStore::parse_set_cookie("jsession=abc; HttpOnly").unwrap();
        assert!(cookie.http_only);
    }

    #[test]
    fn test_parse_cookie_samesite() {
        let cookie = CookieStore::parse_set_cookie("test=1; SameSite=Strict").unwrap();
        assert_eq!(cookie.same_site, SameSite::Strict);

        let cookie2 = CookieStore::parse_set_cookie("test=1; SameSite=Lax").unwrap();
        assert_eq!(cookie2.same_site, SameSite::Lax);

        let cookie3 = CookieStore::parse_set_cookie("test=1; SameSite=None").unwrap();
        assert_eq!(cookie3.same_site, SameSite::None);
    }

    #[test]
    fn test_cookie_store_add_get() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("b=2; Domain=other.com").unwrap());
        assert_eq!(store.len(), 2);

        let url = parse_url("http://example.com/page").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "a");
    }

    #[test]
    fn test_cookie_store_for_url() {
        let mut store = CookieStore::new();
        store
            .add(CookieStore::parse_set_cookie("sess=abc; Domain=example.com; Path=/app").unwrap());

        let matching = parse_url("http://example.com/app/page").unwrap();
        let not_matching = parse_url("http://example.com/other").unwrap();
        let secure_only = parse_url("https://example.com/app").unwrap();

        // Path /app should match /app/page
        assert_eq!(store.get_for_url(&matching).len(), 1);
        // Path /app should NOT match /other
        assert!(store.get_for_url(&not_matching).is_empty());
        // Non-secure cookie should match both http and https
        assert_eq!(store.get_for_url(&secure_only).len(), 1);
    }

    #[test]
    fn test_cookie_header() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("b=2; Domain=example.com").unwrap());

        let url = parse_url("http://example.com/").unwrap();
        let header = store.cookie_header(&url);
        assert!(header.contains("a=1"));
        assert!(header.contains("b=2"));
    }

    #[test]
    fn test_parse_cookie_empty_string() {
        assert!(CookieStore::parse_set_cookie("").is_err());
    }

    #[test]
    fn test_parse_cookie_no_equals() {
        assert!(CookieStore::parse_set_cookie("justacookie").is_err());
    }

    #[test]
    fn test_parse_cookie_empty_name() {
        assert!(CookieStore::parse_set_cookie("=value").is_err());
    }

    #[test]
    fn test_parse_cookie_value_with_equals() {
        let cookie = CookieStore::parse_set_cookie("a=b=c").unwrap();
        assert_eq!(cookie.value, "b=c");
    }

    #[test]
    fn test_parse_cookie_max_age() {
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=3600").unwrap();
        assert_eq!(cookie.expires.as_deref(), Some("3600"));
    }

    #[test]
    fn test_parse_cookie_expires() {
        let cookie =
            CookieStore::parse_set_cookie("a=1; Expires=Wed, 09 Jun 2021 10:18:14 GMT").unwrap();
        assert!(cookie.expires.is_some());
    }

    #[test]
    fn test_cookie_store_replace_same_name() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=x.com").unwrap());
        store.add(CookieStore::parse_set_cookie("a=2; Domain=x.com").unwrap());
        assert_eq!(store.len(), 1);
        let url = parse_url("http://x.com/").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies[0].value, "2");
    }

    #[test]
    fn test_cookie_store_clear() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1").unwrap());
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_cookie_store_default() {
        let store = CookieStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn test_cookie_secure_over_http_blocked() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("secret=abc; Secure; Domain=example.com").unwrap());
        let http_url = parse_url("http://example.com/").unwrap();
        assert!(store.get_for_url(&http_url).is_empty());
    }

    #[test]
    fn test_cookie_header_no_match() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=x.com").unwrap());
        let other_url = parse_url("http://other.com/").unwrap();
        let header = store.cookie_header(&other_url);
        assert!(header.is_empty());
    }

    #[test]
    fn test_cookie_domain_subdomain_match() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=.example.com").unwrap());
        let sub_url = parse_url("http://sub.example.com/").unwrap();
        let cookies = store.get_for_url(&sub_url);
        assert_eq!(cookies.len(), 1);
    }
}
