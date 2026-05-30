//! Cache API 实现 — 用于缓存 Request/Response 对。

use std::collections::HashMap;

use crate::StorageError;

/// 缓存请求的简化表示。
#[derive(Debug, Clone, PartialEq)]
pub struct CacheRequest {
    /// 请求 URL。
    pub url: String,
    /// 请求方法（GET、POST 等）。
    pub method: String,
}

impl CacheRequest {
    /// 创建新的 GET 请求。
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: "GET".to_string(),
        }
    }

    /// 创建指定方法的请求。
    pub fn with_method(url: &str, method: &str) -> Self {
        Self {
            url: url.to_string(),
            method: method.to_string(),
        }
    }
}

/// 缓存响应的简化表示。
#[derive(Debug, Clone, PartialEq)]
pub struct CacheResponse {
    /// 响应状态码。
    pub status: u16,
    /// 响应状态文本。
    pub status_text: String,
    /// 响应头。
    pub headers: HashMap<String, String>,
    /// 响应体。
    pub body: Vec<u8>,
}

impl CacheResponse {
    /// 创建新的响应。
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            status_text: String::new(),
            headers: HashMap::new(),
            body,
        }
    }

    /// 创建 200 OK 响应。
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body,
        }
    }

    /// 添加响应头。
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
}

/// 缓存条目。
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 对应的请求。
    request: CacheRequest,
    /// 对应的响应。
    response: CacheResponse,
}

/// 单个 Cache（等价于 Web API 的 Cache）。
pub struct Cache {
    /// 缓存名称。
    name: String,
    /// 缓存条目列表。
    entries: Vec<CacheEntry>,
}

impl Cache {
    /// 创建新的 Cache 实例。
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// 获取缓存名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 查找匹配请求的第一个缓存响应。
    ///
    /// 匹配规则：URL 完全相等，方法默认只匹配 GET（除非 vary_method 为 true）。
    pub fn match_request(&self, request: &CacheRequest) -> Option<&CacheResponse> {
        self.entries.iter().find_map(|entry| {
            if entry.request.url != request.url {
                return None;
            }
            if entry.request.method != request.method {
                return None;
            }
            Some(&entry.response)
        })
    }

    /// 缓存一对 Request/Response（如已存在则覆盖）。
    pub fn put(
        &mut self,
        request: CacheRequest,
        response: CacheResponse,
    ) -> Result<(), StorageError> {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.request.url == request.url && e.request.method == request.method)
        {
            entry.response = response;
        } else {
            self.entries.push(CacheEntry { request, response });
        }
        Ok(())
    }

    /// 删除匹配请求的缓存条目，返回是否删除成功。
    pub fn delete(&mut self, request: &CacheRequest) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| {
            !(e.request.url == request.url && e.request.method == request.method)
        });
        self.entries.len() < before
    }

    /// 获取缓存中所有请求的 URL 列表。
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.request.url.as_str()).collect()
    }

    /// 获取缓存条目数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// CacheStorage（等价于 Web API 的 CacheStorage）。
pub struct CacheStorage {
    /// 按名称组织的缓存实例。
    caches: HashMap<String, Cache>,
}

impl CacheStorage {
    /// 创建新的 CacheStorage。
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }

    /// 查找匹配请求的响应（在所有缓存中搜索第一个匹配）。
    pub fn match_request(&self, request: &CacheRequest) -> Option<&CacheResponse> {
        for cache in self.caches.values() {
            if let Some(response) = cache.match_request(request) {
                return Some(response);
            }
        }
        None
    }

    /// 打开指定名称的缓存（如不存在则创建）。
    pub fn open(&mut self, name: &str) -> &mut Cache {
        self.caches
            .entry(name.to_string())
            .or_insert_with(|| Cache::new(name))
    }

    /// 是否包含指定名称的缓存。
    pub fn has(&self, name: &str) -> bool {
        self.caches.contains_key(name)
    }

    /// 删除指定名称的缓存，返回是否成功。
    pub fn delete(&mut self, name: &str) -> bool {
        self.caches.remove(name).is_some()
    }

    /// 获取所有缓存名称。
    pub fn keys(&self) -> Vec<&str> {
        self.caches.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for CacheStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_request_new() {
        let req = CacheRequest::new("https://example.com/page");
        assert_eq!(req.url, "https://example.com/page");
        assert_eq!(req.method, "GET");
    }

    #[test]
    fn test_cache_request_with_method() {
        let req = CacheRequest::with_method("https://example.com/api", "POST");
        assert_eq!(req.method, "POST");
    }

    #[test]
    fn test_cache_response_ok() {
        let resp = CacheResponse::ok(b"hello".to_vec());
        assert_eq!(resp.status, 200);
        assert_eq!(resp.status_text, "OK");
        assert_eq!(resp.body, b"hello".to_vec());
    }

    #[test]
    fn test_cache_response_with_header() {
        let resp = CacheResponse::ok(b"data".to_vec()).with_header("Content-Type", "text/html");
        assert_eq!(resp.headers.get("Content-Type"), Some(&"text/html".to_string()));
    }

    #[test]
    fn test_cache_put_and_match() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/page");
        let resp = CacheResponse::ok(b"body".to_vec());

        cache.put(req.clone(), resp).unwrap();
        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.body, b"body".to_vec());
    }

    #[test]
    fn test_cache_match_not_found() {
        let cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/missing");
        assert!(cache.match_request(&req).is_none());
    }

    #[test]
    fn test_cache_match_different_method() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/api");
        cache.put(req.clone(), CacheResponse::ok(b"get".to_vec())).unwrap();

        let post_req = CacheRequest::with_method("https://example.com/api", "POST");
        assert!(cache.match_request(&post_req).is_none());
    }

    #[test]
    fn test_cache_put_overwrite() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/page");
        cache.put(req.clone(), CacheResponse::ok(b"v1".to_vec())).unwrap();
        cache.put(req.clone(), CacheResponse::ok(b"v2".to_vec())).unwrap();

        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.body, b"v2".to_vec());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_delete() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/page");
        cache.put(req.clone(), CacheResponse::ok(b"body".to_vec())).unwrap();

        assert!(cache.delete(&req));
        assert!(cache.is_empty());
        assert!(!cache.delete(&req)); // already deleted
    }

    #[test]
    fn test_cache_keys() {
        let mut cache = Cache::new("v1");
        cache.put(CacheRequest::new("https://a.com"), CacheResponse::ok(vec![])).unwrap();
        cache.put(CacheRequest::new("https://b.com"), CacheResponse::ok(vec![])).unwrap();

        let keys = cache.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"https://a.com"));
        assert!(keys.contains(&"https://b.com"));
    }

    #[test]
    fn test_cache_len_and_empty() {
        let cache = Cache::new("v1");
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    // ── CacheStorage 测试 ──

    #[test]
    fn test_cache_storage_new() {
        let cs = CacheStorage::new();
        assert!(cs.keys().is_empty());
    }

    #[test]
    fn test_cache_storage_open() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("v1");
        assert_eq!(cache.name(), "v1");
    }

    #[test]
    fn test_cache_storage_open_existing() {
        let mut cs = CacheStorage::new();
        cs.open("v1").put(CacheRequest::new("https://a.com"), CacheResponse::ok(b"x".to_vec())).unwrap();
        cs.open("v1").put(CacheRequest::new("https://b.com"), CacheResponse::ok(b"y".to_vec())).unwrap();

        let cache = cs.open("v1");
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_storage_has() {
        let mut cs = CacheStorage::new();
        assert!(!cs.has("v1"));
        cs.open("v1");
        assert!(cs.has("v1"));
    }

    #[test]
    fn test_cache_storage_delete() {
        let mut cs = CacheStorage::new();
        cs.open("v1");
        assert!(cs.delete("v1"));
        assert!(!cs.has("v1"));
        assert!(!cs.delete("v1"));
    }

    #[test]
    fn test_cache_storage_keys() {
        let mut cs = CacheStorage::new();
        cs.open("v1");
        cs.open("v2");
        let keys = cs.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"v1"));
        assert!(keys.contains(&"v2"));
    }

    #[test]
    fn test_cache_storage_match() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/page");
        cs.open("v1")
            .put(req.clone(), CacheResponse::ok(b"cached".to_vec()))
            .unwrap();

        let matched = cs.match_request(&req).unwrap();
        assert_eq!(matched.body, b"cached".to_vec());
    }

    #[test]
    fn test_cache_storage_match_not_found() {
        let cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/missing");
        assert!(cs.match_request(&req).is_none());
    }

    #[test]
    fn test_cache_storage_match_first_wins() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/page");
        cs.open("v1")
            .put(req.clone(), CacheResponse::ok(b"first".to_vec()))
            .unwrap();
        cs.open("v2")
            .put(req.clone(), CacheResponse::ok(b"second".to_vec()))
            .unwrap();

        let matched = cs.match_request(&req).unwrap();
        // First cache that matches wins (HashMap iteration order)
        assert!(matched.body == b"first".to_vec() || matched.body == b"second".to_vec());
    }

    // ── 新增测试 ──

    #[test]
    fn test_cache_response_new_custom_status() {
        let resp = CacheResponse::new(404, b"not found".to_vec());
        assert_eq!(resp.status, 404);
        assert_eq!(resp.body, b"not found".to_vec());
        assert!(resp.status_text.is_empty());
    }

    #[test]
    fn test_cache_put_multiple_urls() {
        let mut cache = Cache::new("v1");
        cache.put(CacheRequest::new("https://a.com/1"), CacheResponse::ok(b"one".to_vec())).unwrap();
        cache.put(CacheRequest::new("https://a.com/2"), CacheResponse::ok(b"two".to_vec())).unwrap();
        cache.put(CacheRequest::new("https://a.com/3"), CacheResponse::ok(b"three".to_vec())).unwrap();
        assert_eq!(cache.len(), 3);

        let req1 = CacheRequest::new("https://a.com/1");
        assert_eq!(cache.match_request(&req1).unwrap().body, b"one".to_vec());
        let req3 = CacheRequest::new("https://a.com/3");
        assert_eq!(cache.match_request(&req3).unwrap().body, b"three".to_vec());
    }

    #[test]
    fn test_cache_delete_nonexistent() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/missing");
        assert!(!cache.delete(&req));
    }

    #[test]
    fn test_cache_keys_after_delete() {
        let mut cache = Cache::new("v1");
        cache.put(CacheRequest::new("https://a.com"), CacheResponse::ok(vec![])).unwrap();
        cache.put(CacheRequest::new("https://b.com"), CacheResponse::ok(vec![])).unwrap();
        cache.put(CacheRequest::new("https://c.com"), CacheResponse::ok(vec![])).unwrap();
        assert_eq!(cache.keys().len(), 3);
        cache.delete(&CacheRequest::new("https://b.com"));
        let keys = cache.keys();
        assert_eq!(keys.len(), 2);
        assert!(!keys.contains(&"https://b.com"));
    }

    #[test]
    fn test_cache_different_methods_same_url() {
        let mut cache = Cache::new("v1");
        let get_req = CacheRequest::new("https://example.com/api");
        let post_req = CacheRequest::with_method("https://example.com/api", "POST");
        cache.put(get_req.clone(), CacheResponse::ok(b"get_resp".to_vec())).unwrap();
        cache.put(post_req.clone(), CacheResponse::ok(b"post_resp".to_vec())).unwrap();
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.match_request(&get_req).unwrap().body, b"get_resp".to_vec());
        assert_eq!(cache.match_request(&post_req).unwrap().body, b"post_resp".to_vec());
    }

    #[test]
    fn test_cache_storage_multiple_caches_isolation() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/data");
        cs.open("cache-a")
            .put(req.clone(), CacheResponse::ok(b"from-a".to_vec()))
            .unwrap();
        cs.open("cache-b")
            .put(req.clone(), CacheResponse::ok(b"from-b".to_vec()))
            .unwrap();

        // Both caches have the URL
        let resp = cs.open("cache-a").match_request(&req).unwrap();
        assert_eq!(resp.body, b"from-a".to_vec());
        let resp = cs.open("cache-b").match_request(&req).unwrap();
        assert_eq!(resp.body, b"from-b".to_vec());

        // Deleting one cache doesn't affect the other
        cs.delete("cache-a");
        assert!(!cs.has("cache-a"));
        assert!(cs.has("cache-b"));
        assert!(cs.match_request(&req).is_some());
    }

    #[test]
    fn test_cache_with_response_headers() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/page");
        let resp = CacheResponse::ok(b"html".to_vec())
            .with_header("Content-Type", "text/html")
            .with_header("Cache-Control", "max-age=3600");
        cache.put(req.clone(), resp).unwrap();
        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.headers.get("Content-Type"), Some(&"text/html".to_string()));
        assert_eq!(matched.headers.get("Cache-Control"), Some(&"max-age=3600".to_string()));
    }

    #[test]
    fn test_cache_storage_default() {
        let cs = CacheStorage::default();
        assert!(cs.keys().is_empty());
    }

    #[test]
    fn test_cache_name() {
        let cache = Cache::new("my-cache");
        assert_eq!(cache.name(), "my-cache");
    }
}
