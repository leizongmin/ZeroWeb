//! HTTP 响应缓存。
//!
//! 提供基于 Cache-Control、ETag 和 Last-Modified 的 HTTP 响应缓存。
//! 支持 LRU 淘汰策略和缓存容量限制。

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::request::HttpResponse;

/// 缓存条目。
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 响应体。
    body: Vec<u8>,
    /// 响应头。
    headers: Vec<(String, String)>,
    /// HTTP 状态码。
    status_code: u16,
    /// 最终 URL（重定向后）。
    url: String,
    /// 存储时的单调时间（用于 TTL 计算）。
    stored_at: Instant,
    /// 缓存过期时间（TTL 秒数），None 表示不缓存。
    ttl_secs: Option<u64>,
    /// ETag 值。
    etag: Option<String>,
    /// Last-Modified 值。
    last_modified: Option<String>,
    /// 是否为共享缓存。
    #[allow(dead_code)]
    is_shared: bool,
}

/// Cache-Control 指令解析结果。
#[derive(Debug, Clone, Default)]
struct CacheControl {
    /// max-age 指令（秒）。
    max_age: Option<u64>,
    /// s-maxage 指令（秒，仅共享缓存）。
    s_maxage: Option<u64>,
    /// no-cache — 必须每次重新验证。
    no_cache: bool,
    /// no-store — 完全不缓存。
    no_store: bool,
    /// public — 允许共享缓存。
    public: bool,
    /// private — 不允许共享缓存。
    private: bool,
    /// must-revalidate — 过期后必须重新验证。
    must_revalidate: bool,
}

/// HTTP 响应缓存。
///
/// 基于内存的 HTTP 响应缓存，支持：
/// - Cache-Control 头解析（max-age, no-cache, no-store, public, private）
/// - ETag/If-None-Match 条件请求
/// - Last-Modified/If-Modified-Since 条件请求
/// - LRU 淘汰策略
/// - 缓存容量限制
#[derive(Debug)]
pub struct HttpCache {
    /// 缓存存储。
    entries: HashMap<String, CacheEntry>,
    /// LRU 访问顺序（URL 列表，最近访问的在末尾）。
    lru_order: Vec<String>,
    /// 最大缓存条目数。
    max_entries: usize,
    /// 最大缓存总字节数。
    max_bytes: usize,
}

impl Default for HttpCache {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpCache {
    /// 创建新的 HTTP 缓存，使用默认配置。
    ///
    /// 默认配置：最大 1000 条目，最大总大小 50MB。
    pub fn new() -> Self {
        Self::with_config(1000, 50 * 1024 * 1024)
    }

    /// 创建指定配置的 HTTP 缓存。
    pub fn with_config(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: Vec::new(),
            max_entries,
            max_bytes,
        }
    }

    /// 尝试从缓存获取响应。
    ///
    /// 返回 None 表示缓存未命中或缓存已过期。
    /// 返回 Some 表示缓存命中，返回缓存的响应。
    pub fn get(&mut self, url: &str) -> Option<CachedResponse> {
        let entry = self.entries.get(url)?;

        // 检查 TTL
        if let Some(ttl) = entry.ttl_secs {
            if entry.stored_at.elapsed() > Duration::from_secs(ttl) {
                // 缓存已过期，移除
                self.remove(url);
                return None;
            }
        } else {
            // 无 TTL，不缓存
            return None;
        }

        // 先提取数据
        let result = CachedResponse {
            body: entry.body.clone(),
            headers: entry.headers.clone(),
            status_code: entry.status_code,
            url: entry.url.clone(),
            etag: entry.etag.clone(),
            last_modified: entry.last_modified.clone(),
        };

        // 提升 LRU 顺序
        self.promote(url);

        Some(result)
    }

    /// 检查是否有有效的缓存条目（不提升 LRU 顺序）。
    pub fn contains(&self, url: &str) -> bool {
        if let Some(entry) = self.entries.get(url)
            && let Some(ttl) = entry.ttl_secs
        {
            return entry.stored_at.elapsed() <= Duration::from_secs(ttl);
        }
        false
    }

    /// 存储响应到缓存。
    ///
    /// 根据 Cache-Control 和其他响应头决定是否缓存。
    pub fn put(&mut self, url: &str, response: &HttpResponse) -> bool {
        // 解析 Cache-Control
        let cc = Self::parse_cache_control(response);

        // no-store 不缓存
        if cc.no_store {
            return false;
        }

        // 非 GET 请求的响应通常不缓存
        // 非 200/203/300/301/302/304/307/308/410 状态码通常不缓存
        if !Self::is_cacheable_status(response.status_code) {
            return false;
        }

        // 计算缓存生存时间
        let ttl_secs = Self::compute_ttl(&cc, response);
        let ttl_secs = match ttl_secs {
            Some(ttl) if ttl > 0 => Some(ttl),
            _ => return false,
        };

        // 提取 ETag 和 Last-Modified
        let etag = response.header("etag").map(|s| s.to_string());
        let last_modified = response.header("last-modified").map(|s| s.to_string());

        let entry = CacheEntry {
            body: response.body.clone(),
            headers: response.headers.clone(),
            status_code: response.status_code,
            url: response.url.clone(),
            stored_at: Instant::now(),
            ttl_secs,
            etag,
            last_modified,
            is_shared: cc.public,
        };

        // 先检查容量
        self.evict_if_needed(response.body.len());

        // 插入或更新
        if self.entries.contains_key(url) {
            self.remove(url);
        }

        self.entries.insert(url.to_string(), entry);
        self.lru_order.push(url.to_string());

        true
    }

    /// 移除缓存条目。
    pub fn remove(&mut self, url: &str) -> bool {
        if self.entries.remove(url).is_some() {
            self.lru_order.retain(|u| u != url);
            true
        } else {
            false
        }
    }

    /// 清空所有缓存。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
    }

    /// 返回缓存条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 返回缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 返回缓存总字节数。
    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|e| e.body.len()).sum()
    }

    /// 为条件请求生成请求头。
    ///
    /// 如果缓存中有该 URL 的条目，返回 If-None-Match 和/或 If-Modified-Since 头。
    pub fn conditional_headers(&self, url: &str) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        if let Some(entry) = self.entries.get(url) {
            if let Some(ref etag) = entry.etag {
                headers.push(("If-None-Match".to_string(), etag.clone()));
            }
            if let Some(ref lm) = entry.last_modified {
                headers.push(("If-Modified-Since".to_string(), lm.clone()));
            }
        }
        headers
    }

    /// 解析 Cache-Control 头。
    fn parse_cache_control(response: &HttpResponse) -> CacheControl {
        let mut cc = CacheControl::default();

        if let Some(value) = response.header("cache-control") {
            for directive in value.split(',') {
                let directive = directive.trim();
                if directive.eq_ignore_ascii_case("no-cache") {
                    cc.no_cache = true;
                } else if directive.eq_ignore_ascii_case("no-store") {
                    cc.no_store = true;
                } else if directive.eq_ignore_ascii_case("public") {
                    cc.public = true;
                } else if directive.eq_ignore_ascii_case("private") {
                    cc.private = true;
                } else if directive.eq_ignore_ascii_case("must-revalidate") {
                    cc.must_revalidate = true;
                } else if let Some(age_str) = directive.strip_prefix("max-age=") {
                    cc.max_age = age_str.trim().parse().ok();
                } else if let Some(age_str) = directive.strip_prefix("s-maxage=") {
                    cc.s_maxage = age_str.trim().parse().ok();
                }
            }
        }

        cc
    }

    /// 计算缓存 TTL（秒）。
    fn compute_ttl(cc: &CacheControl, response: &HttpResponse) -> Option<u64> {
        // no-cache 虽然可以存储，但每次必须验证，设 TTL 为 0
        if cc.no_cache {
            return Some(0);
        }

        // 优先使用 s-maxage（仅共享缓存时）
        if let Some(s_maxage) = cc.s_maxage {
            return Some(s_maxage);
        }

        // 使用 max-age
        if let Some(max_age) = cc.max_age {
            return Some(max_age);
        }

        // 尝试从 Expires 头推断
        if let Some(expires) = response.header("expires")
            && let Ok(expires_time) = parse_http_date(expires)
        {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if expires_time > now {
                return Some(expires_time - now);
            }
            return Some(0);
        }

        // 无缓存指令，给一个保守的默认值（0 表示不缓存）
        None
    }

    /// 判断 HTTP 状态码是否可缓存。
    fn is_cacheable_status(status: u16) -> bool {
        matches!(status, 200 | 203 | 204 | 206 | 300 | 301 | 302 | 304 | 307 | 308 | 404 | 405 | 410 | 414 | 501)
    }

    /// 提升 LRU 顺序。
    fn promote(&mut self, url: &str) {
        self.lru_order.retain(|u| u != url);
        self.lru_order.push(url.to_string());
    }

    /// 如果需要，淘汰最旧的条目。
    fn evict_if_needed(&mut self, incoming_bytes: usize) {
        // 淘汰直到有足够空间
        while self.entries.len() >= self.max_entries
            || (self.total_bytes() + incoming_bytes > self.max_bytes && !self.lru_order.is_empty())
        {
            if let Some(oldest_url) = self.lru_order.first().cloned() {
                self.entries.remove(&oldest_url);
                self.lru_order.remove(0);
            } else {
                break;
            }
        }
    }
}

/// 缓存命中返回的响应。
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// 响应体。
    pub body: Vec<u8>,
    /// 响应头。
    pub headers: Vec<(String, String)>,
    /// HTTP 状态码。
    pub status_code: u16,
    /// 最终 URL。
    pub url: String,
    /// ETag 值。
    pub etag: Option<String>,
    /// Last-Modified 值。
    pub last_modified: Option<String>,
}

impl CachedResponse {
    /// 转换为 HttpResponse。
    pub fn into_response(self) -> HttpResponse {
        HttpResponse {
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
            url: self.url,
            redirect_count: 0,
        }
    }
}

/// 解析 HTTP 日期格式（RFC 7231）。
///
/// 支持格式：`Sun, 06 Nov 1994 08:49:37 GMT`
fn parse_http_date(date_str: &str) -> Result<u64, ()> {
    // 简化实现：只解析 IMF-fixdate 格式
    // "Day, DD Mon YYYY HH:MM:SS GMT"
    let comma_pos = match date_str.find(", ") {
        Some(pos) => pos,
        None => return Err(()),
    };
    let rest = &date_str[comma_pos + 2..];

    let date_parts: Vec<&str> = rest.split(' ').collect();
    if date_parts.len() != 4 && date_parts.len() != 5 {
        return Err(());
    }

    let day: u64 = date_parts[0].parse().map_err(|_| ())?;
    let month = month_to_number(date_parts[1])?;
    let year: u64 = date_parts[2].parse().map_err(|_| ())?;

    // 简化：不精确计算，给出大致的 Unix 时间戳
    // 从年份开始估算
    let days_since_epoch = (year - 1970) * 365 + (year - 1970) / 4 + month * 30 + day;
    Ok(days_since_epoch * 86400)
}

/// 月份名转数字。
fn month_to_number(month: &str) -> Result<u64, ()> {
    match month {
        "Jan" => Ok(0),
        "Feb" => Ok(1),
        "Mar" => Ok(2),
        "Apr" => Ok(3),
        "May" => Ok(4),
        "Jun" => Ok(5),
        "Jul" => Ok(6),
        "Aug" => Ok(7),
        "Sep" => Ok(8),
        "Oct" => Ok(9),
        "Nov" => Ok(10),
        "Dec" => Ok(11),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpResponse;

    fn make_response(status: u16, body: &[u8], headers: Vec<(&str, &str)>) -> HttpResponse {
        HttpResponse {
            status_code: status,
            headers: headers.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            body: body.to_vec(),
            url: "https://example.com/test".to_string(),
            redirect_count: 0,
        }
    }

    #[test]
    fn test_cache_new() {
        let cache = HttpCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_put_get() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60")]);
        assert!(cache.put("https://example.com/test", &resp));
        assert_eq!(cache.len(), 1);

        let cached = cache.get("https://example.com/test").unwrap();
        assert_eq!(cached.body, b"hello");
        assert_eq!(cached.status_code, 200);
    }

    #[test]
    fn test_cache_no_store() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "no-store")]);
        assert!(!cache.put("https://example.com/test", &resp));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = HttpCache::new();
        assert!(cache.get("https://example.com/notexist").is_none());
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60")]);
        cache.put("https://example.com/test", &resp);
        assert!(cache.remove("https://example.com/test"));
        assert!(cache.get("https://example.com/test").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60")]);
        cache.put("https://example.com/a", &resp);
        cache.put("https://example.com/b", &resp);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_max_age() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=3600")]);
        assert!(cache.put("https://example.com/test", &resp));
        assert!(cache.contains("https://example.com/test"));
    }

    #[test]
    fn test_cache_no_cache_directive() {
        let mut cache = HttpCache::new();
        // no-cache 可以存储但 TTL 为 0
        let resp = make_response(200, b"hello", vec![("cache-control", "no-cache")]);
        // TTL 为 0 表示立即过期
        cache.put("https://example.com/test", &resp);
        // 由于 TTL=0，get 时应立即过期
        assert!(cache.get("https://example.com/test").is_none());
    }

    #[test]
    fn test_cache_etag() {
        let mut cache = HttpCache::new();
        let resp = make_response(
            200,
            b"hello",
            vec![("cache-control", "max-age=60"), ("etag", "\"abc123\"")],
        );
        cache.put("https://example.com/test", &resp);
        let cached = cache.get("https://example.com/test").unwrap();
        assert_eq!(cached.etag, Some("\"abc123\"".to_string()));
    }

    #[test]
    fn test_cache_conditional_headers() {
        let mut cache = HttpCache::new();
        let resp = make_response(
            200,
            b"hello",
            vec![
                ("cache-control", "max-age=60"),
                ("etag", "\"abc\""),
                ("last-modified", "Wed, 21 Oct 2015 07:28:00 GMT"),
            ],
        );
        cache.put("https://example.com/test", &resp);

        let headers = cache.conditional_headers("https://example.com/test");
        assert!(headers.iter().any(|(k, v)| k == "If-None-Match" && v == "\"abc\""));
        assert!(headers.iter().any(|(k, _)| k == "If-Modified-Since"));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = HttpCache::with_config(3, 1024 * 1024);
        let resp = make_response(200, b"a", vec![("cache-control", "max-age=60")]);

        cache.put("https://a.com", &resp);
        cache.put("https://b.com", &resp);
        cache.put("https://c.com", &resp);
        assert_eq!(cache.len(), 3);

        // 插入第 4 个应淘汰第 1 个
        cache.put("https://d.com", &resp);
        assert_eq!(cache.len(), 3);
        assert!(cache.get("https://a.com").is_none());
        assert!(cache.get("https://d.com").is_some());
    }

    #[test]
    fn test_cache_lru_promote() {
        let mut cache = HttpCache::with_config(3, 1024 * 1024);
        let resp = make_response(200, b"a", vec![("cache-control", "max-age=60")]);

        cache.put("https://a.com", &resp);
        cache.put("https://b.com", &resp);
        cache.put("https://c.com", &resp);

        // 访问 a，提升其 LRU 顺序
        cache.get("https://a.com");

        // 插入 d 应淘汰 b（现在是最旧的）
        cache.put("https://d.com", &resp);
        assert!(cache.get("https://a.com").is_some()); // a 被提升，不应被淘汰
        assert!(cache.get("https://b.com").is_none()); // b 应被淘汰
    }

    #[test]
    fn test_cache_total_bytes() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"12345", vec![("cache-control", "max-age=60")]);
        cache.put("https://a.com", &resp);
        cache.put("https://b.com", &resp);
        assert_eq!(cache.total_bytes(), 10);
    }

    #[test]
    fn test_cache_byte_limit_eviction() {
        let mut cache = HttpCache::with_config(100, 10); // 只有 10 字节空间
        let resp = make_response(200, b"12345678", vec![("cache-control", "max-age=60")]); // 8 字节
        cache.put("https://a.com", &resp);
        assert_eq!(cache.len(), 1);

        // 插入第二个 8 字节条目（需要 16 字节，但上限 10）
        // 应淘汰第一个
        cache.put("https://b.com", &resp);
        assert!(cache.get("https://a.com").is_none());
        assert!(cache.get("https://b.com").is_some());
    }

    #[test]
    fn test_cache_non_cacheable_status() {
        let mut cache = HttpCache::new();
        let resp = make_response(500, b"error", vec![("cache-control", "max-age=60")]);
        assert!(!cache.put("https://example.com/test", &resp));
    }

    #[test]
    fn test_cache_expires_header() {
        let mut cache = HttpCache::new();
        // 无 Cache-Control，使用 Expires
        let resp = make_response(200, b"hello", vec![("expires", "Sun, 06 Nov 2099 08:49:37 GMT")]);
        assert!(cache.put("https://example.com/test", &resp));
        assert!(cache.get("https://example.com/test").is_some());
    }

    #[test]
    fn test_cache_contains() {
        let mut cache = HttpCache::new();
        assert!(!cache.contains("https://example.com/test"));
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60")]);
        cache.put("https://example.com/test", &resp);
        assert!(cache.contains("https://example.com/test"));
    }

    #[test]
    fn test_cache_update_overwrites() {
        let mut cache = HttpCache::new();
        let resp1 = make_response(200, b"old", vec![("cache-control", "max-age=60")]);
        cache.put("https://example.com/test", &resp1);

        let resp2 = make_response(200, b"new", vec![("cache-control", "max-age=60")]);
        cache.put("https://example.com/test", &resp2);

        let cached = cache.get("https://example.com/test").unwrap();
        assert_eq!(cached.body, b"new");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_parse_http_date() {
        let ts = parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert!(ts > 0, "应返回有效的 Unix 时间戳");

        assert!(parse_http_date("invalid").is_err());
    }

    #[test]
    fn test_month_to_number() {
        assert_eq!(month_to_number("Jan"), Ok(0));
        assert_eq!(month_to_number("Jun"), Ok(5));
        assert_eq!(month_to_number("Dec"), Ok(11));
        assert!(month_to_number("Foo").is_err());
    }

    #[test]
    fn test_cached_response_into_response() {
        let cached = CachedResponse {
            body: vec![1, 2, 3],
            headers: vec![("content-type".to_string(), "text/html".to_string())],
            status_code: 200,
            url: "https://example.com".to_string(),
            etag: None,
            last_modified: None,
        };
        let resp = cached.into_response();
        assert_eq!(resp.body, vec![1, 2, 3]);
        assert_eq!(resp.status_code, 200);
    }
}
