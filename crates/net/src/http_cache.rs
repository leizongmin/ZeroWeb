//! HTTP 响应缓存。
//!
//! 内存热缓存 + 可选磁盘持久层（对齐浏览器 memory cache / disk cache）。

use std::collections::HashMap;
use std::time::Instant;

use crate::cache_key::{cache_lookup_key, cache_store_key, strip_url_fragment};
use crate::cache_policy::{CacheStoreMode, parse_cache_control, storable_mode};
use crate::disk_cache::DiskHttpCache;
use crate::private_mode::private_browsing_enabled;
use crate::request::HttpResponse;

/// 缓存查找结果。
#[derive(Debug, Clone)]
pub enum CacheLookup {
    /// 新鲜命中，可直接使用。
    Hit(CachedResponse),
    /// 过期但可再验证（已附带条件请求头）。
    Revalidate {
        /// 缓存中的旧响应（含 body）。
        cached: CachedResponse,
        /// 应附加到条件 GET 的请求头。
        conditional_headers: Vec<(String, String)>,
    },
    /// 未命中。
    Miss,
}

/// 缓存条目（内存层）。
#[derive(Debug, Clone)]
struct CacheEntry {
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    status_code: u16,
    url: String,
    resource_base: String,
    stored_at: Instant,
    ttl_secs: Option<u64>,
    /// RFC 9111 §4.2.3——响应接收时的「初始年龄」（秒），freshness 检查 `resident_time + initial_age <= ttl`。
    /// 新鲜 put 时从响应的 Age/Date 头算出；磁盘→内存提升时为 0（年龄已并入 `ttl_secs`=剩余新鲜期）。
    initial_age_secs: u64,
    revalidate_only: bool,
    vary: Option<String>,
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
    resource_index: HashMap<String, Vec<String>>,
    disk_index_loaded: bool,
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
            resource_index: HashMap::new(),
            disk_index_loaded: false,
            max_entries,
            max_bytes,
            disk: None,
        }
    }

    /// 打开带磁盘持久层的缓存（浏览器默认路径；隐私模式仅内存）。
    pub fn open_persistent() -> Self {
        let mut cache = Self::new();
        if private_browsing_enabled() {
            tracing::info!("private browsing: HTTP disk cache disabled");
            return cache;
        }
        match DiskHttpCache::open_default() {
            Ok(disk) => {
                tracing::info!(dir = %crate::disk_cache::default_cache_dir().display(), "HTTP disk cache opened");
                cache.disk = Some(disk);
            }
            Err(e) => tracing::warn!("HTTP disk cache unavailable: {e}"),
        }
        cache
    }

    /// 查找缓存（新鲜命中 / 需再验证 / 未命中）。
    pub fn lookup(&mut self, url: &str, request_headers: &[(String, String)]) -> CacheLookup {
        self.ensure_disk_index();
        let base = strip_url_fragment(url);
        if let Some(keys) = self.resource_index.get(&base).cloned() {
            for key in keys {
                let vary = self.vary_for_key(&key);
                let candidate = cache_lookup_key(&base, request_headers, vary.as_deref());
                if candidate == key {
                    let lookup = self.lookup_key(&key);
                    if !matches!(lookup, CacheLookup::Miss) {
                        return lookup;
                    }
                }
            }
        }
        self.lookup_key(&base)
    }

    fn lookup_key(&mut self, key: &str) -> CacheLookup {
        if let Some(lookup) = self.lookup_memory(key) {
            return lookup;
        }
        let Some(disk) = self.disk.as_mut() else {
            return CacheLookup::Miss;
        };
        let Some(hit) = disk.read(key) else {
            return CacheLookup::Miss;
        };
        let cached = cached_from_disk_hit(&hit);
        if hit.revalidate_only {
            if is_revalidatable(&hit.etag, &hit.last_modified) {
                tracing::info!(url = %key, "HTTP disk cache revalidate-only");
                return CacheLookup::Revalidate {
                    conditional_headers: conditional_from_validators(&hit.etag, &hit.last_modified),
                    cached,
                };
            }
            let _ = disk.remove(key);
            return CacheLookup::Miss;
        }
        if hit.fresh_for_secs > 0 {
            tracing::info!(url = %key, "HTTP disk cache hit");
            self.insert_memory_from_hit(key, &cached, hit);
            return CacheLookup::Hit(cached);
        }
        if is_revalidatable(&hit.etag, &hit.last_modified) {
            tracing::info!(url = %key, "HTTP disk cache stale, revalidate");
            return CacheLookup::Revalidate {
                conditional_headers: conditional_from_validators(&hit.etag, &hit.last_modified),
                cached,
            };
        }
        let _ = disk.remove(key);
        CacheLookup::Miss
    }

    /// 304 Not Modified — 刷新新鲜期并返回缓存 body。
    pub fn not_modified(
        &mut self,
        url: &str,
        request_headers: &[(String, String)],
        response: &HttpResponse,
    ) -> Option<CachedResponse> {
        let key = self.resolve_lookup_key(url, request_headers)?;
        if let Some(entry) = self.entries.get(&key) {
            let mut cached = CachedResponse {
                body: entry.body.clone(),
                headers: entry.headers.clone(),
                status_code: entry.status_code,
                url: entry.url.clone(),
                etag: entry.etag.clone(),
                last_modified: entry.last_modified.clone(),
            };
            if let Some(mode) = storable_mode(response)
                && let Some(e) = self.entries.get_mut(&key)
            {
                e.stored_at = Instant::now();
                match mode {
                    CacheStoreMode::Fresh(ttl) => {
                        e.ttl_secs = Some(ttl);
                        e.revalidate_only = false;
                    }
                    CacheStoreMode::RevalidateOnly => {
                        e.ttl_secs = Some(0);
                        e.revalidate_only = true;
                    }
                }
                // R3233：304 可携新 Age/Date（CDN 重报）→ 重算 initial_age（与 R3231/R3232 头并入一致）。
                e.initial_age_secs = crate::cache_policy::compute_initial_age(response);
                if let Some(etag) = response.header("etag") {
                    e.etag = Some(etag.to_string());
                    cached.etag = e.etag.clone();
                }
                if let Some(lm) = response.header("last-modified") {
                    e.last_modified = Some(lm.to_string());
                    cached.last_modified = e.last_modified.clone();
                }
                // R3231：RFC 9111 §4.3.4——304 的元数据字段须并入存储 + 返回的 headers（同名替换，
                // 缺则追加）。旧实现仅更 etag/last_modified 便捷字段，headers Vec 保留旧 Cache-Control/
                // Expires/Date/Vary——返回给调用方（JS response.headers）+ 内存持久化的头为旧值。
                for field in [
                    "cache-control",
                    "content-location",
                    "date",
                    "expires",
                    "vary",
                    "etag",
                    "last-modified",
                ] {
                    if let Some(val) = response.header(field) {
                        merge_header(&mut e.headers, field, val);
                        merge_header(&mut cached.headers, field, val);
                    }
                }
            }
            if let Some(disk) = self.disk.as_mut() {
                let _ = disk.refresh_not_modified(&key, response);
            }
            tracing::info!(url = %key, "HTTP cache 304 revalidated");
            return Some(cached);
        }
        if let Some(disk) = self.disk.as_mut()
            && let Some(hit) = disk.read(&key)
            && disk.refresh_not_modified(&key, response)
        {
            let cached = cached_from_disk_hit(&hit);
            self.insert_memory_from_hit(&key, &cached, hit);
            tracing::info!(url = %key, "HTTP disk cache 304 revalidated");
            return Some(cached);
        }
        None
    }

    /// 尝试从缓存获取新鲜响应（兼容旧 API）。
    pub fn get(&mut self, url: &str) -> Option<CachedResponse> {
        match self.lookup(url, &[]) {
            CacheLookup::Hit(r) => Some(r),
            _ => None,
        }
    }
    fn lookup_memory(&mut self, key: &str) -> Option<CacheLookup> {
        let entry = self.entries.get(key)?;
        let cached = CachedResponse {
            body: entry.body.clone(),
            headers: entry.headers.clone(),
            status_code: entry.status_code,
            url: entry.url.clone(),
            etag: entry.etag.clone(),
            last_modified: entry.last_modified.clone(),
        };
        if entry.revalidate_only {
            if is_revalidatable(&entry.etag, &entry.last_modified) {
                return Some(CacheLookup::Revalidate {
                    conditional_headers: conditional_from_validators(&entry.etag, &entry.last_modified),
                    cached,
                });
            }
            self.remove(key);
            return None;
        }
        let fresh = entry.ttl_secs.is_some_and(|ttl| {
            // R3233：RFC 9111 §4.2.4——`current_age = initial_age + resident_time`，新鲜 ⇔ `current_age < lifetime`。
            // R3371：用 u64 saturating_add 计算 current_age（秒），避免 `Duration + Duration` 在
            // `initial_age_secs == u64::MAX`（来自恶意/畸形 `Age:` 头）时溢出 panic。
            current_age_secs(entry.stored_at.elapsed(), entry.initial_age_secs) <= ttl
        });
        if fresh {
            self.promote(key);
            return Some(CacheLookup::Hit(cached));
        }
        if is_revalidatable(&entry.etag, &entry.last_modified) {
            return Some(CacheLookup::Revalidate {
                conditional_headers: conditional_from_validators(&entry.etag, &entry.last_modified),
                cached,
            });
        }
        self.remove(key);
        None
    }

    fn insert_memory_from_hit(&mut self, key: &str, cached: &CachedResponse, disk: crate::disk_cache::DiskCacheHit) {
        if (disk.fresh_for_secs == 0 && !disk.revalidate_only) || self.max_entries == 0 {
            return;
        }
        let cc = parse_cache_control(&HttpResponse {
            status_code: cached.status_code,
            headers: cached.headers.clone(),
            body: vec![],
            url: cached.url.clone(),
            redirect_count: 0,
        });
        self.evict_if_needed(cached.body.len());
        if self.entries.contains_key(key) {
            self.remove(key);
        }
        let resource_base = strip_url_fragment(&cached.url);
        self.entries.insert(
            key.to_string(),
            CacheEntry {
                body: cached.body.clone(),
                headers: cached.headers.clone(),
                status_code: cached.status_code,
                url: cached.url.clone(),
                resource_base: resource_base.clone(),
                stored_at: Instant::now(),
                ttl_secs: if disk.revalidate_only {
                    Some(0)
                } else {
                    Some(disk.fresh_for_secs)
                },
                // R3233：磁盘→内存提升时年龄已并入 ttl_secs（= fresh_for_secs 剩余新鲜期），故 initial_age=0。
                initial_age_secs: 0,
                revalidate_only: disk.revalidate_only,
                vary: disk.vary.clone(),
                etag: cached.etag.clone(),
                last_modified: cached.last_modified.clone(),
                is_shared: cc.public,
            },
        );
        self.lru_order.push(key.to_string());
        self.register_resource_key(&resource_base, key);
    }

    /// 检查是否有有效的缓存条目（不提升 LRU 顺序）。
    pub fn contains(&self, url: &str) -> bool {
        if let Some(entry) = self.entries.get(url)
            && let Some(ttl) = entry.ttl_secs
        {
            // R3233：与 lookup_memory 同——freshness 计入 initial_age（Age/Date 头）。
            // R3371：同 lookup 新鲜度检查——用 u64 saturating_add 避免 `Duration + Duration` 溢出 panic。
            return current_age_secs(entry.stored_at.elapsed(), entry.initial_age_secs) <= ttl;
        }
        false
    }

    /// 存储响应到缓存（自动计算 cache key）。
    pub fn put(&mut self, url: &str, response: &HttpResponse) -> bool {
        self.put_with_headers(url, &[], response)
    }

    /// 存储响应到缓存（含请求头以构造 Vary cache key）。
    pub fn put_with_headers(
        &mut self,
        url: &str,
        request_headers: &[(String, String)],
        response: &HttpResponse,
    ) -> bool {
        let key = cache_store_key(url, request_headers, response);
        self.put_key(&key, response)
    }

    fn put_key(&mut self, key: &str, response: &HttpResponse) -> bool {
        let mode = match storable_mode(response) {
            Some(m) => m,
            None => return false,
        };
        let (ttl_secs, revalidate_only) = match mode {
            CacheStoreMode::Fresh(ttl) => (Some(ttl), false),
            CacheStoreMode::RevalidateOnly => (Some(0), true),
        };
        let cc = parse_cache_control(response);
        let etag = response.header("etag").map(|s| s.to_string());
        let last_modified = response.header("last-modified").map(|s| s.to_string());
        let resource_url = response.url.clone();
        let resource_base = strip_url_fragment(&resource_url);
        let vary = response.header("vary").map(|s| s.to_string());

        if self.max_entries > 0 {
            self.evict_if_needed(response.body.len());
            if self.entries.contains_key(key) {
                self.remove(key);
            }
            self.entries.insert(
                key.to_string(),
                CacheEntry {
                    body: response.body.clone(),
                    headers: response.headers.clone(),
                    status_code: response.status_code,
                    url: resource_url.clone(),
                    resource_base: resource_base.clone(),
                    stored_at: Instant::now(),
                    ttl_secs,
                    // R3233：新鲜 put 从响应的 Age/Date 头算 initial_age（§4.2.3）。
                    initial_age_secs: crate::cache_policy::compute_initial_age(response),
                    revalidate_only,
                    vary: vary.clone(),
                    etag: etag.clone(),
                    last_modified: last_modified.clone(),
                    is_shared: cc.public,
                },
            );
            self.lru_order.push(key.to_string());
            self.register_resource_key(&resource_base, key);
        }

        let mut stored = self.entries.contains_key(key);
        if let Some(disk) = self.disk.as_mut() {
            stored |= disk.put(key, response);
        }
        stored
    }

    /// 移除缓存条目。
    pub fn remove(&mut self, url: &str) -> bool {
        let resource_base = self.entries.get(url).map(|e| e.resource_base.clone());
        if let Some(base) = resource_base {
            self.unregister_resource_key(&base, url);
        }
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
        self.resource_index.clear();
        self.disk_index_loaded = false;
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
        self.conditional_headers_with_request(url, &[])
    }

    /// 为条件请求生成请求头（含 Vary 维度）。
    pub fn conditional_headers_with_request(
        &self,
        url: &str,
        request_headers: &[(String, String)],
    ) -> Vec<(String, String)> {
        let Some(key) = self.resolve_lookup_key_readonly(url, request_headers) else {
            return Vec::new();
        };
        let mut headers = Vec::new();
        if let Some(entry) = self.entries.get(&key) {
            if let Some(ref etag) = entry.etag {
                headers.push(("If-None-Match".to_string(), etag.clone()));
            }
            if let Some(ref lm) = entry.last_modified {
                headers.push(("If-Modified-Since".to_string(), lm.clone()));
            }
            return headers;
        }
        if let Some(disk) = &self.disk {
            return disk.conditional_headers(&key);
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
                self.remove(&oldest_url);
            } else {
                break;
            }
        }
    }

    fn ensure_disk_index(&mut self) {
        if self.disk_index_loaded {
            return;
        }
        if let Some(disk) = self.disk.as_ref() {
            for entry in disk.list_index_entries() {
                if let Some(resource_url) = entry.resource_url {
                    let base = strip_url_fragment(&resource_url);
                    self.register_resource_key(&base, &entry.key);
                }
            }
        }
        self.disk_index_loaded = true;
    }

    fn resolve_lookup_key(&self, url: &str, request_headers: &[(String, String)]) -> Option<String> {
        self.resolve_lookup_key_readonly(url, request_headers)
    }

    fn resolve_lookup_key_readonly(&self, url: &str, request_headers: &[(String, String)]) -> Option<String> {
        let base = strip_url_fragment(url);
        if let Some(keys) = self.resource_index.get(&base) {
            for key in keys {
                let vary = self.vary_for_key(key);
                let candidate = cache_lookup_key(&base, request_headers, vary.as_deref());
                if &candidate == key {
                    return Some(key.clone());
                }
            }
        }
        Some(base)
    }

    fn vary_for_key(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.entries.get(key) {
            return entry.vary.clone();
        }
        self.disk.as_ref()?.entry_vary(key)
    }

    fn register_resource_key(&mut self, resource_base: &str, key: &str) {
        let keys = self.resource_index.entry(resource_base.to_string()).or_default();
        if !keys.iter().any(|k| k == key) {
            keys.push(key.to_string());
        }
    }

    fn unregister_resource_key(&mut self, resource_base: &str, key: &str) {
        if let Some(keys) = self.resource_index.get_mut(resource_base) {
            keys.retain(|k| k != key);
            if keys.is_empty() {
                self.resource_index.remove(resource_base);
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

fn cached_from_disk_hit(hit: &crate::disk_cache::DiskCacheHit) -> CachedResponse {
    CachedResponse {
        body: hit.body.clone(),
        headers: hit.headers.clone(),
        status_code: hit.status_code,
        url: hit.resource_url.clone().unwrap_or_else(|| hit.url.clone()),
        etag: hit.etag.clone(),
        last_modified: hit.last_modified.clone(),
    }
}

fn is_revalidatable(etag: &Option<String>, last_modified: &Option<String>) -> bool {
    etag.is_some() || last_modified.is_some()
}

/// RFC 9111 §4.2.4——`current_age = initial_age + resident_time`（秒）。
///
/// R3371：用 `u64::saturating_add` 而非 `Duration + Duration`。`initial_age_secs` 来自响应的
/// `Age` 头（信任边界输入，远端可控），可被恶意/畸形服务器设为 `u64::MAX`；`Duration` 的 `+`
/// 在溢出时 **panic**（core/src/time.rs:1257），而 `saturating_add` 溢出到 `u64::MAX`——
/// 后者天然表示「已远超任何 TTL → 不新鲜」，符合语义且不 panic。
fn current_age_secs(resident: std::time::Duration, initial_age_secs: u64) -> u64 {
    initial_age_secs.saturating_add(resident.as_secs())
}

/// 用 304 响应的元数据字段更新 header 列表（RFC 9111 §4.3.4——同名替换，缺则追加；name 大小写不敏感）。
fn merge_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    for (n, v) in headers.iter_mut() {
        if n.eq_ignore_ascii_case(name) {
            *v = value.to_string();
            return;
        }
    }
    headers.push((name.to_string(), value.to_string()));
}

fn conditional_from_validators(etag: &Option<String>, last_modified: &Option<String>) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    if let Some(e) = etag {
        headers.push(("If-None-Match".to_string(), e.clone()));
    }
    if let Some(lm) = last_modified {
        headers.push(("If-Modified-Since".to_string(), lm.clone()));
    }
    headers
}

/// 全局共享 HTTP 缓存（性能门禁优化 S6，2026-08-08）。
///
/// webview（主文档缓存）、fetch_proxy（renderer IPC 路径）、net_pool（进程内
/// async 路径）共用同一份缓存——此前三处各自独立/缺失，同一 URL 在不同路径
/// 反复走网络。持久化对齐原 webview/fetch_proxy 的 `open_persistent` 行为。
pub fn shared_http_cache() -> std::sync::Arc<std::sync::Mutex<HttpCache>> {
    static CACHE: std::sync::OnceLock<std::sync::Arc<std::sync::Mutex<HttpCache>>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(HttpCache::open_persistent())))
        .clone()
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
        let resp = make_response(200, b"hello", vec![("cache-control", "no-cache"), ("etag", "\"abc\"")]);
        assert!(cache.put("https://example.com/test", &resp));
        match cache.lookup("https://example.com/test", &[]) {
            CacheLookup::Revalidate { .. } => {}
            other => panic!("expected revalidate-only lookup, got {other:?}"),
        }
        assert!(cache.get("https://example.com/test").is_none());
    }

    #[test]
    fn test_cache_vary_multi_field() {
        let mut cache = HttpCache::new();
        let req = vec![
            ("Accept-Encoding".into(), "gzip".into()),
            ("Accept-Language".into(), "en".into()),
        ];
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=60".into()),
                ("Vary".into(), "Accept-Encoding, Accept-Language".into()),
            ],
            body: b"vary-body".to_vec(),
            url: "https://example.com/vary".into(),
            redirect_count: 0,
        };
        assert!(cache.put_with_headers("https://example.com/vary", &req, &resp));
        let hit = cache.lookup("https://example.com/vary", &req);
        assert!(matches!(hit, CacheLookup::Hit(_)));
        let wrong_lang = vec![
            ("Accept-Encoding".into(), "gzip".into()),
            ("Accept-Language".into(), "fr".into()),
        ];
        assert!(matches!(
            cache.lookup("https://example.com/vary", &wrong_lang),
            CacheLookup::Miss
        ));
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

    /// R3231：304 Not Modified 须并入 304 的元数据 header（RFC 9111 §4.3.4）——
    /// 旧 not_modified 仅更 etag/last_modified 便捷字段，headers Vec 保留旧 Cache-Control/etag，
    /// 致返回调用方（JS response.headers）+ 内存持久化的头为旧值。
    #[test]
    fn test_cache_304_merges_metadata_headers_r3231() {
        let mut cache = HttpCache::new();
        // 存 200：Cache-Control: max-age=60 + ETag "v1" + 旧 Expires。
        let resp = make_response(
            200,
            b"hello",
            vec![
                ("cache-control", "max-age=60"),
                ("etag", "\"v1\""),
                ("expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
            ],
        );
        cache.put("https://example.com/test", &resp);

        // 304 携新 Cache-Control: max-age=300 + ETag "v2"。
        let not_mod = make_response(304, b"", vec![("cache-control", "max-age=300"), ("etag", "\"v2\"")]);
        let cached = cache
            .not_modified("https://example.com/test", &[], &not_mod)
            .expect("304 须刷新缓存条目");

        let header_val = |name: &str| {
            cached
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_str())
        };
        // R3231：返回的 headers 须反映 304 的元数据（旧 max-age=60 / "v1" 被替换）。
        assert_eq!(
            header_val("cache-control"),
            Some("max-age=300"),
            "304 的 Cache-Control 须并入 headers"
        );
        assert_eq!(header_val("etag"), Some("\"v2\""), "304 的 ETag 须并入 headers");
        assert_eq!(cached.etag, Some("\"v2\"".to_string()), "etag 便捷字段亦更新");
        // body + status 仍为缓存的 200（304 仅 revalidate，不替换 body）。
        assert_eq!(cached.body, b"hello");
        assert_eq!(cached.status_code, 200);
    }

    /// R3233：RFC 9111 §4.2.3——响应的 `Age` 头（CDN/共享缓存上报）须计入新鲜度：
    /// `current_age = initial_age + resident_time`，`fresh ⇔ current_age < lifetime`。
    /// 旧实现忽略 Age，把 CDN 已存活 N 秒的响应当全新鲜 → 可能服务过期内容。
    #[test]
    fn test_cache_age_header_reduces_freshness_r3233() {
        let mut cache = HttpCache::new();
        // Age(150) > max-age(100) → 接收时 current_age=150 已过期 → 须 Revalidate（有 ETag）。
        let aged = make_response(
            200,
            b"cdn",
            vec![("cache-control", "max-age=100"), ("age", "150"), ("etag", "\"v1\"")],
        );
        assert!(cache.put("https://example.com/aged", &aged));
        match cache.lookup("https://example.com/aged", &[]) {
            CacheLookup::Revalidate { .. } => {}
            other => panic!("Age>max-age 须判过期→Revalidate，got {other:?}"),
        }
        // 对照：无 Age，max-age=100 → 立即查为 Hit（resident_time≈0 < 100）。
        let fresh = make_response(
            200,
            b"fresh",
            vec![("cache-control", "max-age=100"), ("etag", "\"v1\"")],
        );
        assert!(cache.put("https://example.com/fresh", &fresh));
        match cache.lookup("https://example.com/fresh", &[]) {
            CacheLookup::Hit(_) => {}
            other => panic!("无 Age 的 max-age=100 须 Hit，got {other:?}"),
        }
    }

    /// R3233：RFC 9111 §4.2.3 apparent_age——`Date` 头在远过去表明响应早已在源站生成（过期）。
    #[test]
    fn test_cache_date_apparent_age_r3233() {
        let mut cache = HttpCache::new();
        let stale = make_response(
            200,
            b"old",
            vec![
                ("cache-control", "max-age=100"),
                ("date", "Wed, 21 Oct 2015 07:28:00 GMT"),
                ("etag", "\"v1\""),
            ],
        );
        assert!(cache.put("https://example.com/date-stale", &stale));
        match cache.lookup("https://example.com/date-stale", &[]) {
            CacheLookup::Revalidate { .. } => {}
            other => panic!("远过去 Date 的 apparent_age 须判过期→Revalidate，got {other:?}"),
        }
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

    // ── R3371：Age 头巨值致 current_age Duration 加法溢出 panic ──

    #[test]
    // R3371：响应带 `Age: <u64::MAX>`（恶意/畸形，远端可控信任边界）+ 可缓存（max-age=100）
    // → put 时 `initial_age_secs = u64::MAX` → 后续 lookup 新鲜度检查
    // `stored_at.elapsed() + Duration::from_secs(u64::MAX)` 旧实现 **溢出 panic**
    // （core/src/time.rs:1257）。修复后用 `u64::saturating_add` → current_age=u64::MAX，
    // 大于 ttl=100 → 判为不新鲜（stale），不 panic。
    // 端到端复现：修复前此测 panic（overflow when adding durations）；修复后 lookup 不 panic。
    fn huge_age_header_does_not_panic_on_lookup_r3371() {
        let mut cache = HttpCache::with_config(10, 1024 * 1024);
        let url = "https://example.com/aged.js";
        let resp = make_response(
            200,
            b"x",
            vec![("cache-control", "max-age=100"), ("age", &u64::MAX.to_string())],
        );
        assert!(cache.put(url, &resp), "应作为 Fresh 存入");

        // 关键：lookup 不得 panic。修复前在此 overflow-panic（overflow when adding durations）。
        let lookup = cache.lookup(url, &[]);
        // current_age = u64::MAX + resident > ttl=100 → 不新鲜。无 validator → Miss（条目被移除）。
        assert!(
            !matches!(lookup, crate::CacheLookup::Hit(_)),
            "Age=u64::MAX 的响应不应判为新鲜 Hit"
        );
    }

    #[test]
    /// R3371：`current_age_secs` 在 `initial_age_secs == u64::MAX` 时 saturating 不 panic。
    fn current_age_secs_saturates_on_huge_initial_age_r3371() {
        use std::time::Duration;
        // 任意 resident + u64::MAX → u64::MAX（不 panic，不回绕）
        assert_eq!(current_age_secs(Duration::from_secs(5), u64::MAX), u64::MAX);
        assert_eq!(current_age_secs(Duration::from_secs(0), u64::MAX), u64::MAX);
        // 正常值正确相加
        assert_eq!(current_age_secs(Duration::from_secs(30), 100), 130);
        // resident 的亚秒部分被 as_secs 截断
        assert_eq!(current_age_secs(Duration::from_millis(999), 100), 100);
    }
}
