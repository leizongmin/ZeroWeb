//! HTTP 响应缓存。
//!
//! 内存热缓存 + 可选磁盘持久层（对齐浏览器 memory cache / disk cache）。

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::cache_policy::{parse_cache_control, storable_ttl};
use crate::disk_cache::DiskHttpCache;
use crate::request::HttpResponse;

/// 缓存条目（内存层）。
#[derive(Debug, Clone)]
struct CacheEntry {
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    status_code: u16,
    url: String,
    stored_at: Instant,
    ttl_secs: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
    #[allow(dead_code)]
    is_shared: bool,
}

/// HTTP 响应缓存（内存 + 可选磁盘）。
///
/// - 内存层：LRU，默认 1000 条 / 50MB
/// - 磁盘层：[`DiskHttpCache`]，跨会话保留；通过 [`Self::open_persistent`] 启用
#[derive(Debug)]
pub struct HttpCache {
    entries: HashMap<String, CacheEntry>,
    lru_order: Vec<String>,
    max_entries: usize,
    max_bytes: usize,
    disk: Option<DiskHttpCache>,
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
            disk: None,
        }
    }

    /// 打开带磁盘持久层的缓存（浏览器默认路径）。
    pub fn open_persistent() -> Self {
        let mut cache = Self::new();
        match DiskHttpCache::open_default() {
            Ok(disk) => {
                tracing::info!(dir = %crate::disk_cache::default_cache_dir().display(), "HTTP disk cache opened");
                cache.disk = Some(disk);
            }
            Err(e) => tracing::warn!("HTTP disk cache unavailable: {e}"),
        }
        cache
    }

    /// 尝试从缓存获取响应。
    ///
    /// 返回 None 表示缓存未命中或缓存已过期。
    /// 返回 Some 表示缓存命中，返回缓存的响应。
    pub fn get(&mut self, url: &str) -> Option<CachedResponse> {
        if let Some(hit) = self.get_memory(url) {
            return Some(hit);
        }
        let disk = self.disk.as_mut()?;
        let hit = disk.get(url)?;
        tracing::info!(url, "HTTP disk cache hit");
        let cached = CachedResponse {
            body: hit.body.clone(),
            headers: hit.headers.clone(),
            status_code: hit.status_code,
            url: hit.url.clone(),
            etag: hit.etag.clone(),
            last_modified: hit.last_modified.clone(),
        };
        self.insert_memory_from_hit(url, &cached, hit);
        Some(cached)
    }

    fn get_memory(&mut self, url: &str) -> Option<CachedResponse> {
        let entry = self.entries.get(url)?;

        if let Some(ttl) = entry.ttl_secs {
            if entry.stored_at.elapsed() > Duration::from_secs(ttl) {
                self.remove(url);
                return None;
            }
        } else {
            return None;
        }

        let result = CachedResponse {
            body: entry.body.clone(),
            headers: entry.headers.clone(),
            status_code: entry.status_code,
            url: entry.url.clone(),
            etag: entry.etag.clone(),
            last_modified: entry.last_modified.clone(),
        };

        self.promote(url);
        Some(result)
    }

    fn insert_memory_from_hit(&mut self, url: &str, cached: &CachedResponse, disk: crate::disk_cache::DiskCacheHit) {
        let cc = parse_cache_control(&HttpResponse {
            status_code: cached.status_code,
            headers: cached.headers.clone(),
            body: vec![],
            url: cached.url.clone(),
            redirect_count: 0,
        });
        if disk.fresh_for_secs == 0 || self.max_entries == 0 {
            return;
        }
        self.evict_if_needed(cached.body.len());
        if self.entries.contains_key(url) {
            self.remove(url);
        }
        self.entries.insert(
            url.to_string(),
            CacheEntry {
                body: cached.body.clone(),
                headers: cached.headers.clone(),
                status_code: cached.status_code,
                url: cached.url.clone(),
                stored_at: Instant::now(),
                ttl_secs: Some(disk.fresh_for_secs),
                etag: cached.etag.clone(),
                last_modified: cached.last_modified.clone(),
                is_shared: cc.public,
            },
        );
        self.lru_order.push(url.to_string());
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
        let Some(ttl_secs) = storable_ttl(response) else {
            return false;
        };
        let cc = parse_cache_control(response);
        let etag = response.header("etag").map(|s| s.to_string());
        let last_modified = response.header("last-modified").map(|s| s.to_string());

        if self.max_entries > 0 {
            self.evict_if_needed(response.body.len());
            if self.entries.contains_key(url) {
                self.remove(url);
            }
            self.entries.insert(
                url.to_string(),
                CacheEntry {
                    body: response.body.clone(),
                    headers: response.headers.clone(),
                    status_code: response.status_code,
                    url: response.url.clone(),
                    stored_at: Instant::now(),
                    ttl_secs: Some(ttl_secs),
                    etag: etag.clone(),
                    last_modified: last_modified.clone(),
                    is_shared: cc.public,
                },
            );
            self.lru_order.push(url.to_string());
        }

        let mut stored = self.entries.contains_key(url);
        if let Some(disk) = self.disk.as_mut() {
            stored |= disk.put(url, response);
        }
        stored
    }

    /// 移除缓存条目。
    pub fn remove(&mut self, url: &str) -> bool {
        let mem = if self.entries.remove(url).is_some() {
            self.lru_order.retain(|u| u != url);
            true
        } else {
            false
        };
        let disk = self.disk.as_mut().is_some_and(|d| d.remove(url));
        mem || disk
    }

    /// 清空内存与磁盘缓存。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        if let Some(disk) = self.disk.as_mut() {
            let _ = disk.clear();
        }
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
            return headers;
        }
        if let Some(disk) = &self.disk {
            return disk.conditional_headers(url);
        }
        headers
    }

    /// 磁盘缓存占用（无磁盘层时为 0）。
    pub fn disk_bytes(&self) -> u64 {
        self.disk.as_ref().map(DiskHttpCache::total_bytes).unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpResponse;

    fn make_response(status: u16, body: &[u8], headers: Vec<(&str, &str)>) -> HttpResponse {
        HttpResponse {
            status_code: status,
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
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
        let resp = make_response(200, b"hello", vec![("cache-control", "no-cache")]);
        assert!(!cache.put("https://example.com/test", &resp));
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
        let ts = crate::cookie::parse_expires_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert!(ts > 0, "应返回有效的 Unix 时间戳");
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

    #[test]
    fn test_cache_no_cache_control_no_ttl() {
        let mut cache = HttpCache::new();
        // 无 Cache-Control 也无 Expires，不应缓存
        let resp = make_response(200, b"hello", vec![]);
        assert!(!cache.put("https://example.com/test", &resp));
    }

    #[test]
    fn test_cache_s_maxage() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "s-maxage=3600")]);
        assert!(cache.put("https://example.com/test", &resp));
        let cached = cache.get("https://example.com/test").unwrap();
        assert_eq!(cached.body, b"hello");
    }

    #[test]
    fn test_cache_must_revalidate_still_caches() {
        // must-revalidate 允许缓存但在过期后必须重新验证
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60, must-revalidate")]);
        assert!(cache.put("https://example.com/test", &resp));
        assert!(cache.get("https://example.com/test").is_some());
    }

    #[test]
    fn test_cache_private_still_caches() {
        // 私有缓存在浏览器端是允许的
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"private data", vec![("cache-control", "private, max-age=60")]);
        assert!(cache.put("https://example.com/test", &resp));
    }

    #[test]
    fn test_cache_different_urls_independent() {
        let mut cache = HttpCache::new();
        let resp_a = make_response(200, b"page a", vec![("cache-control", "max-age=60")]);
        let resp_b = make_response(200, b"page b", vec![("cache-control", "max-age=60")]);
        cache.put("https://a.com/page", &resp_a);
        cache.put("https://b.com/page", &resp_b);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get("https://a.com/page").unwrap().body, b"page a");
        assert_eq!(cache.get("https://b.com/page").unwrap().body, b"page b");
    }

    #[test]
    fn test_cache_remove_nonexistent() {
        let mut cache = HttpCache::new();
        assert!(!cache.remove("https://example.com/notexist"));
    }

    #[test]
    fn test_cache_default() {
        let cache = HttpCache::default();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_zero_max_entries() {
        let mut cache = HttpCache::with_config(0, 1024);
        let resp = make_response(200, b"hello", vec![("cache-control", "max-age=60")]);
        // max_entries=0 时不应缓存任何条目
        assert!(!cache.put("https://example.com/test", &resp));
    }

    #[test]
    fn test_cache_multiple_etag_last_modified() {
        let mut cache = HttpCache::new();
        let resp = make_response(
            200,
            b"hello",
            vec![
                ("cache-control", "max-age=60"),
                ("etag", "\"v2\""),
                ("last-modified", "Thu, 01 Jan 2026 00:00:00 GMT"),
            ],
        );
        cache.put("https://example.com/test", &resp);
        let headers = cache.conditional_headers("https://example.com/test");
        assert_eq!(headers.len(), 2);
        let etag_header = headers.iter().find(|(k, _)| k == "If-None-Match").unwrap();
        assert_eq!(etag_header.1, "\"v2\"");
    }

    #[test]
    fn test_conditional_headers_no_entry() {
        let cache = HttpCache::new();
        let headers = cache.conditional_headers("https://example.com/notexist");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_cacheable_status_codes() {
        // 200, 203, 204, 206, 300, 301, 302, 304, 307, 308, 404, 405, 410, 414, 501
        let cacheable = [200, 203, 204, 300, 301, 302, 304, 307, 308, 404, 410];
        let mut cache = HttpCache::new();
        for status in cacheable {
            let resp = make_response(status, b"body", vec![("cache-control", "max-age=60")]);
            assert!(
                cache.put(&format!("https://example.com/{status}"), &resp),
                "status {status} should be cacheable"
            );
        }
        assert_eq!(cache.len(), cacheable.len());
    }

    #[test]
    fn test_non_cacheable_status_codes() {
        let non_cacheable = [201, 205, 400, 403, 500, 502, 503];
        let mut cache = HttpCache::new();
        for status in non_cacheable {
            let resp = make_response(status, b"body", vec![("cache-control", "max-age=60")]);
            assert!(
                !cache.put(&format!("https://example.com/{status}"), &resp),
                "status {status} should not be cacheable"
            );
        }
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_parse_cache_control_case_insensitive() {
        let mut cache = HttpCache::new();
        let resp = make_response(200, b"hello", vec![("Cache-Control", "Max-Age=60")]);
        assert!(cache.put("https://example.com/test", &resp));
    }

    #[test]
    fn test_tiered_disk_promotes_to_memory() {
        use crate::disk_cache::DiskHttpCache;
        use std::fs;

        let dir = std::env::temp_dir().join(format!("zero-tiered-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut cache = HttpCache::with_config(100, 50 * 1024 * 1024);
        cache.disk = DiskHttpCache::open(&dir).ok();

        let url = "https://example.com/tier.js";
        let resp = make_response(200, b"tiered", vec![("cache-control", "max-age=600")]);
        assert!(cache.put(url, &resp));
        cache.entries.clear();
        cache.lru_order.clear();

        let hit = cache.get(url).expect("disk hit promotes");
        assert_eq!(hit.body, b"tiered");
        assert!(cache.entries.contains_key(url), "应回填内存层");
        let _ = fs::remove_dir_all(dir);
    }
}
