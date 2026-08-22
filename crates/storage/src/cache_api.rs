//! Cache API 实现 — 用于缓存 Request/Response 对。

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::StorageError;

pub(crate) mod persistence;

/// 缓存请求的简化表示。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheRequest {
    /// 请求 URL。
    pub url: String,
    /// 请求方法（GET、POST 等）。
    pub method: String,
    /// 请求头快照（按创建时顺序保存）。
    pub headers: Vec<(String, String)>,
}

impl CacheRequest {
    /// 创建新的 GET 请求。
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
        }
    }

    /// 创建指定方法的请求。
    pub fn with_method(url: &str, method: &str) -> Self {
        Self {
            url: url.to_string(),
            method: method.to_string(),
            headers: Vec::new(),
        }
    }

    /// 创建指定方法和请求头的请求。
    pub fn with_method_and_headers(url: &str, method: &str, headers: Vec<(String, String)>) -> Self {
        Self {
            url: url.to_string(),
            method: method.to_string(),
            headers,
        }
    }
}

/// Cache API 查询选项。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheQueryOptions {
    /// 忽略 URL 查询参数。
    pub ignore_search: bool,
    /// 忽略请求方法。
    pub ignore_method: bool,
    /// 忽略 Vary 头。
    pub ignore_vary: bool,
}

/// 缓存响应的简化表示。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheResponse {
    /// Response URL associated with the stored response.
    pub url: String,
    /// 响应状态码。
    pub status: u16,
    /// 响应状态文本。
    pub status_text: String,
    /// Fetch response type (`default`/`basic`/`cors`/`opaque`/`opaqueredirect`/`error`).
    pub response_type: String,
    /// 响应头。
    pub headers: HashMap<String, String>,
    /// 响应体。
    pub body: Vec<u8>,
}

impl CacheResponse {
    /// 创建新的响应。
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            url: String::new(),
            status,
            status_text: String::new(),
            response_type: "default".to_string(),
            headers: HashMap::new(),
            body,
        }
    }

    /// 创建 200 OK 响应。
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            url: String::new(),
            status: 200,
            status_text: "OK".to_string(),
            response_type: "default".to_string(),
            headers: HashMap::new(),
            body,
        }
    }

    /// 设置响应类型。
    pub fn with_response_type(mut self, response_type: &str) -> Self {
        self.response_type = response_type.to_string();
        self
    }

    /// 设置响应 URL。
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    /// 添加响应头。
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
}

/// Serializable snapshot of one [`CacheStorage`] instance.
///
/// This is an interchange format for storage owners that need to persist
/// CacheStorage data while keeping [`CacheStorage`] internals private.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheStorageSnapshot {
    /// Named caches in creation order.
    pub caches: Vec<CacheSnapshot>,
}

/// Serializable snapshot of one named [`Cache`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheSnapshot {
    /// Cache name.
    pub name: String,
    /// Cached request/response entries in insertion order.
    pub entries: Vec<CacheEntrySnapshot>,
}

/// Serializable snapshot of one Cache entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheEntrySnapshot {
    /// Stored request snapshot.
    pub request: CacheRequest,
    /// Stored response snapshot.
    pub response: CacheResponse,
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
#[derive(Debug, Clone)]
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

    pub(crate) fn from_entries(name: &str, entries: Vec<(CacheRequest, CacheResponse)>) -> Result<Self, StorageError> {
        let mut cache = Self::new(name);
        for (request, response) in entries {
            cache.put(request, response)?;
        }
        Ok(cache)
    }

    /// 获取缓存名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 查找匹配请求的第一个缓存响应。
    ///
    /// 匹配规则：URL 完全相等，方法完全匹配。
    pub fn match_request(&self, request: &CacheRequest) -> Option<&CacheResponse> {
        self.match_request_with_options(request, CacheQueryOptions::default())
    }

    /// 查找匹配请求的第一个缓存响应，并应用查询选项。
    pub fn match_request_with_options(
        &self,
        request: &CacheRequest,
        options: CacheQueryOptions,
    ) -> Option<&CacheResponse> {
        self.entries
            .iter()
            .find(|entry| cache_entry_matches(entry, request, options))
            .map(|entry| &entry.response)
    }

    /// 查找匹配请求的所有缓存响应。
    pub fn match_all(&self, request: &CacheRequest) -> Vec<&CacheResponse> {
        self.match_all_with_options(request, CacheQueryOptions::default())
    }

    /// 查找匹配请求的所有缓存响应，并应用查询选项。
    pub fn match_all_with_options(&self, request: &CacheRequest, options: CacheQueryOptions) -> Vec<&CacheResponse> {
        self.entries
            .iter()
            .filter(|entry| cache_entry_matches(entry, request, options))
            .map(|entry| &entry.response)
            .collect()
    }

    /// 缓存一对 Request/Response（如已存在则覆盖）。
    pub fn put(&mut self, request: CacheRequest, response: CacheResponse) -> Result<(), StorageError> {
        validate_cache_put(&request, &response)?;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| cache_entry_matches(entry, &request, CacheQueryOptions::default()))
        {
            entry.request = request;
            entry.response = response;
        } else {
            self.entries.push(CacheEntry { request, response });
        }
        Ok(())
    }

    /// 删除匹配请求的缓存条目，返回是否删除成功。
    pub fn delete(&mut self, request: &CacheRequest) -> bool {
        self.delete_with_options(request, CacheQueryOptions::default())
    }

    /// 删除匹配请求的缓存条目，并应用查询选项。
    pub fn delete_with_options(&mut self, request: &CacheRequest, options: CacheQueryOptions) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|entry| !cache_entry_matches(entry, request, options));
        self.entries.len() < before
    }

    /// 获取缓存中所有请求的 URL 列表。
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.request.url.as_str()).collect()
    }

    /// 获取缓存中所有请求列表。
    pub fn request_keys(&self) -> Vec<&CacheRequest> {
        self.entries.iter().map(|e| &e.request).collect()
    }

    /// 获取匹配请求的缓存请求列表，并应用查询选项。
    pub fn request_keys_with_options(&self, request: &CacheRequest, options: CacheQueryOptions) -> Vec<&CacheRequest> {
        self.entries
            .iter()
            .filter(|entry| cache_entry_matches(entry, request, options))
            .map(|entry| &entry.request)
            .collect()
    }

    /// 获取缓存条目数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over request/response entries in insertion order.
    pub(crate) fn entries(&self) -> impl Iterator<Item = (&CacheRequest, &CacheResponse)> {
        self.entries.iter().map(|entry| (&entry.request, &entry.response))
    }
}

/// CacheStorage（等价于 Web API 的 CacheStorage）。
#[derive(Debug, Clone)]
pub struct CacheStorage {
    /// 按名称组织的缓存实例。
    caches: IndexMap<String, Cache>,
}

impl CacheStorage {
    /// 创建新的 CacheStorage。
    pub fn new() -> Self {
        Self {
            caches: IndexMap::new(),
        }
    }

    pub(crate) fn from_caches(caches: Vec<(String, Cache)>) -> Result<Self, StorageError> {
        let mut cache_storage = Self::new();
        for (name, cache) in caches {
            if cache.name() != name {
                return Err(StorageError::Serialization(
                    "CacheStorage persistence cache name mismatch".to_string(),
                ));
            }
            if cache_storage.caches.insert(name, cache).is_some() {
                return Err(StorageError::Serialization(
                    "duplicate cache in CacheStorage persistence data".to_string(),
                ));
            }
        }
        Ok(cache_storage)
    }

    /// 查找匹配请求的响应（在所有缓存中搜索第一个匹配）。
    pub fn match_request(&self, request: &CacheRequest) -> Option<&CacheResponse> {
        self.match_request_with_options(request, CacheQueryOptions::default())
    }

    /// 查找匹配请求的响应（在所有缓存中搜索第一个匹配），并应用查询选项。
    pub fn match_request_with_options(
        &self,
        request: &CacheRequest,
        options: CacheQueryOptions,
    ) -> Option<&CacheResponse> {
        for cache in self.caches.values() {
            if let Some(response) = cache.match_request_with_options(request, options) {
                return Some(response);
            }
        }
        None
    }

    /// 打开指定名称的缓存（如不存在则创建）。
    pub fn open(&mut self, name: &str) -> &mut Cache {
        self.caches.entry(name.to_string()).or_insert_with(|| Cache::new(name))
    }

    /// 获取已存在的指定名称缓存。
    pub fn get(&self, name: &str) -> Option<&Cache> {
        self.caches.get(name)
    }

    /// 获取已存在的指定名称缓存的可变引用。
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Cache> {
        self.caches.get_mut(name)
    }

    /// 是否包含指定名称的缓存。
    pub fn has(&self, name: &str) -> bool {
        self.caches.contains_key(name)
    }

    /// 删除指定名称的缓存，返回是否成功。
    pub fn delete(&mut self, name: &str) -> bool {
        self.caches.shift_remove(name).is_some()
    }

    /// 获取所有缓存名称。
    pub fn keys(&self) -> Vec<&str> {
        // https://w3c.github.io/ServiceWorker/#cache-storage-keys
        self.caches.keys().map(|s| s.as_str()).collect()
    }

    /// CacheStorage 是否没有任何命名 cache。
    pub fn is_empty(&self) -> bool {
        self.caches.is_empty()
    }

    /// Iterate over named caches in creation order.
    pub(crate) fn iter_caches(&self) -> impl Iterator<Item = (&str, &Cache)> {
        self.caches.iter().map(|(name, cache)| (name.as_str(), cache))
    }

    /// Create a serializable snapshot in cache creation and entry insertion order.
    pub fn snapshot(&self) -> CacheStorageSnapshot {
        CacheStorageSnapshot {
            caches: self
                .iter_caches()
                .map(|(name, cache)| CacheSnapshot {
                    name: name.to_string(),
                    entries: cache
                        .entries()
                        .map(|(request, response)| CacheEntrySnapshot {
                            request: request.clone(),
                            response: response.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    /// Restore CacheStorage from a snapshot, validating entries through [`Cache::put`].
    pub fn from_snapshot(snapshot: CacheStorageSnapshot) -> Result<Self, StorageError> {
        let caches = snapshot
            .caches
            .into_iter()
            .map(|cache| {
                let entries = cache
                    .entries
                    .into_iter()
                    .map(|entry| (entry.request, entry.response))
                    .collect();
                Cache::from_entries(&cache.name, entries).map(|cache_data| (cache.name, cache_data))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_caches(caches)
    }
}

fn cache_requests_match(cached: &CacheRequest, query: &CacheRequest, options: CacheQueryOptions) -> bool {
    (options.ignore_method || cached.method == query.method)
        && cache_urls_match(&cached.url, &query.url, options.ignore_search)
}

fn cache_entry_matches(entry: &CacheEntry, query: &CacheRequest, options: CacheQueryOptions) -> bool {
    cache_requests_match(&entry.request, query, options)
        && cache_vary_matches(&entry.response, &entry.request, query, options)
}

fn validate_cache_put(request: &CacheRequest, response: &CacheResponse) -> Result<(), StorageError> {
    // https://w3c.github.io/ServiceWorker/#cache-put
    if !request.method.eq_ignore_ascii_case("GET") {
        return Err(StorageError::Type("Cache.put request method must be GET".to_string()));
    }
    let url = url::Url::parse(&request.url)
        .map_err(|_| StorageError::Type("Cache.put request URL must be an HTTP(S) URL".to_string()))?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(StorageError::Type(
            "Cache.put request URL must be an HTTP(S) URL".to_string(),
        ));
    }
    if response.response_type.eq_ignore_ascii_case("opaque") {
        return Ok(());
    }
    if response.status == 206 {
        return Err(StorageError::Type(
            "Cache.put cannot store a 206 Partial Content response".to_string(),
        ));
    }
    if response_vary_has_star(response) {
        return Err(StorageError::Type(
            "Cache.put cannot store a response with Vary: *".to_string(),
        ));
    }
    Ok(())
}

fn response_vary_has_star(response: &CacheResponse) -> bool {
    header_value(&response.headers, "vary").is_some_and(|vary| {
        vary.split(',')
            .map(str::trim)
            .any(|field| field.eq_ignore_ascii_case("*"))
    })
}

fn cache_vary_matches(
    response: &CacheResponse,
    cached_request: &CacheRequest,
    query: &CacheRequest,
    options: CacheQueryOptions,
) -> bool {
    if options.ignore_vary {
        return true;
    }
    if response.response_type.eq_ignore_ascii_case("opaque") {
        return true;
    }
    let Some(vary) = header_value(&response.headers, "vary") else {
        return true;
    };
    // https://w3c.github.io/ServiceWorker/#query-cache
    let mut has_field = false;
    for field in vary.split(',').map(str::trim).filter(|field| !field.is_empty()) {
        if field == "*" {
            return false;
        }
        has_field = true;
        if request_header_value(&cached_request.headers, field) != request_header_value(&query.headers, field) {
            return false;
        }
    }
    has_field || vary.trim().is_empty()
}

fn header_value<'a>(headers: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn request_header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    let mut values = headers
        .iter()
        .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str());
    let first = values.next()?;
    let mut combined = first.to_string();
    for value in values {
        combined.push_str(", ");
        combined.push_str(value);
    }
    Some(combined)
}

fn cache_urls_match(cached: &str, query: &str, ignore_search: bool) -> bool {
    match (url::Url::parse(cached), url::Url::parse(query)) {
        (Ok(mut cached_url), Ok(mut query_url)) => {
            cached_url.set_fragment(None);
            query_url.set_fragment(None);
            if ignore_search {
                cached_url.set_query(None);
                query_url.set_query(None);
            }
            cached_url == query_url
        }
        _ if ignore_search => strip_url_query_and_fragment(cached) == strip_url_query_and_fragment(query),
        _ => strip_url_fragment(cached) == strip_url_fragment(query),
    }
}

fn strip_url_query_and_fragment(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

fn strip_url_fragment(url: &str) -> &str {
    url.split('#').next().unwrap_or(url)
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
    fn test_cache_put_rejects_uncacheable_requests_and_responses() {
        let mut cache = Cache::new("v1");
        let cases = [
            (
                CacheRequest::with_method("https://example.com/page", "POST"),
                CacheResponse::ok(Vec::new()),
                "method must be GET",
            ),
            (
                CacheRequest::new("ftp://example.com/page"),
                CacheResponse::ok(Vec::new()),
                "HTTP(S) URL",
            ),
            (
                CacheRequest::new("https://example.com/partial"),
                CacheResponse::new(206, b"partial".to_vec()),
                "206 Partial Content",
            ),
            (
                CacheRequest::new("https://example.com/vary-star"),
                CacheResponse::ok(Vec::new()).with_header("Vary", "Accept-Encoding, *"),
                "Vary: *",
            ),
        ];

        for (request, response, expected) in cases {
            let error = cache.put(request, response).unwrap_err();
            assert!(matches!(error, StorageError::Type(message) if message.contains(expected)));
        }
        assert!(cache.is_empty());

        cache
            .put(
                CacheRequest::new("https://example.com/error-response"),
                CacheResponse::new(0, Vec::new()),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::new("https://example.com/server-error"),
                CacheResponse::new(500, b"server error".to_vec()),
            )
            .unwrap();
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_put_preserves_error_response_type() {
        let mut cache = Cache::new("v1");
        let request = CacheRequest::new("https://example.com/error");
        let response = CacheResponse::new(0, Vec::new()).with_response_type("error");

        cache.put(request.clone(), response).unwrap();
        let matched = cache.match_request(&request).unwrap();
        assert_eq!(matched.status, 0);
        assert_eq!(matched.response_type, "error");
    }

    #[test]
    fn test_cache_match_not_found() {
        let cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/missing");
        assert!(cache.match_request(&req).is_none());
    }

    #[test]
    fn test_cache_match_all_and_request_keys_preserve_method() {
        let mut cache = Cache::new("v1");
        let get_req = CacheRequest::new("https://example.com/api");
        let post_req = CacheRequest::with_method("https://example.com/api", "POST");
        cache.put(get_req.clone(), CacheResponse::ok(b"get".to_vec())).unwrap();

        let matched = cache.match_all(&post_req);
        assert!(matched.is_empty());
        let ignored_method = cache.match_all_with_options(
            &post_req,
            CacheQueryOptions {
                ignore_method: true,
                ..CacheQueryOptions::default()
            },
        );
        assert_eq!(ignored_method.len(), 1);
        assert_eq!(ignored_method[0].body, b"get".to_vec());

        let keys = cache.request_keys();
        assert_eq!(keys.len(), 1);
        assert!(keys.iter().any(|key| key.url == get_req.url && key.method == "GET"));
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
    fn test_cache_query_options_ignore_search_and_method() {
        let mut cache = Cache::new("v1");
        let cached = CacheRequest::new("https://example.com/api?version=1#old");
        cache.put(cached, CacheResponse::ok(b"cached".to_vec())).unwrap();

        let query = CacheRequest::with_method("https://example.com/api?version=2#new", "HEAD");
        assert!(cache.match_request(&query).is_none());
        assert!(
            cache
                .match_request_with_options(
                    &query,
                    CacheQueryOptions {
                        ignore_search: true,
                        ignore_method: true,
                        ignore_vary: false,
                    },
                )
                .is_some()
        );
    }

    #[test]
    fn test_cache_urls_match_relative_fallback_ignores_fragment() {
        assert!(cache_urls_match("/api?version=1#old", "/api?version=1#new", false));
        assert!(cache_urls_match("/api?version=1#old", "/api?version=2#new", true));
    }

    #[test]
    fn test_cache_query_options_filter_keys_and_delete() {
        let mut cache = Cache::new("v1");
        cache
            .put(
                CacheRequest::new("https://example.com/api?one"),
                CacheResponse::ok(b"one".to_vec()),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::new("https://example.com/api?two"),
                CacheResponse::ok(b"two".to_vec()),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::new("https://example.com/other?one"),
                CacheResponse::ok(b"other".to_vec()),
            )
            .unwrap();

        let query = CacheRequest::with_method("https://example.com/api?three", "HEAD");
        let keys = cache.request_keys_with_options(
            &query,
            CacheQueryOptions {
                ignore_search: true,
                ignore_method: true,
                ignore_vary: false,
            },
        );
        assert_eq!(keys.len(), 2);

        assert!(cache.delete_with_options(
            &query,
            CacheQueryOptions {
                ignore_search: true,
                ignore_method: true,
                ignore_vary: false,
            },
        ));
        assert_eq!(cache.len(), 1);
        assert!(
            cache
                .match_request(&CacheRequest::new("https://example.com/other?one"))
                .is_some()
        );
    }

    #[test]
    fn test_cache_vary_matches_request_header_snapshot() {
        let mut cache = Cache::new("v1");
        let stored = CacheRequest::with_method_and_headers(
            "https://example.com/c",
            "GET",
            vec![("Cookies".into(), "is-for-cookie".into())],
        );
        cache
            .put(
                stored.clone(),
                CacheResponse::ok(Vec::new()).with_header("Vary", "Cookies"),
            )
            .unwrap();

        assert!(cache.match_request(&stored).is_some());
        assert!(
            cache
                .match_request(&CacheRequest::new("https://example.com/c"))
                .is_none()
        );
        assert!(
            cache
                .match_request_with_options(
                    &CacheRequest::new("https://example.com/c"),
                    CacheQueryOptions {
                        ignore_vary: true,
                        ..CacheQueryOptions::default()
                    },
                )
                .is_some()
        );
    }

    #[test]
    fn test_cache_vary_ignored_for_opaque_response() {
        let mut cache = Cache::new("v1");
        let stored =
            CacheRequest::with_method_and_headers("https://example.com/c", "GET", vec![("foo".into(), "bar".into())]);
        cache
            .put(
                stored.clone(),
                CacheResponse::ok(Vec::new())
                    .with_response_type("opaque")
                    .with_header("Vary", "foo"),
            )
            .unwrap();

        let query = CacheRequest::with_method_and_headers(
            "https://example.com/c",
            "GET",
            vec![("foo".into(), "CHANGED".into())],
        );
        assert!(
            cache.match_request(&query).is_some(),
            "Cache.match ignores Vary headers for opaque responses"
        );
    }

    #[test]
    fn test_cache_put_accepts_opaque_internal_uncacheable_metadata() {
        let mut cache = Cache::new("v1");
        let request = CacheRequest::new("https://example.com/opaque");
        cache
            .put(
                request.clone(),
                CacheResponse::new(206, b"hidden".to_vec())
                    .with_response_type("opaque")
                    .with_header("Vary", "*"),
            )
            .unwrap();

        let matched = cache.match_request(&request).unwrap();
        assert_eq!(matched.response_type, "opaque");
        assert_eq!(matched.status, 206);
        assert_eq!(matched.body, b"hidden".to_vec());
    }

    #[test]
    fn test_cache_put_keeps_distinct_vary_variants() {
        let mut cache = Cache::new("v1");
        let url = "https://example.com/c";
        cache
            .put(
                CacheRequest::with_method_and_headers(url, "GET", vec![("Cookies".into(), "is-for-cookie".into())]),
                CacheResponse::ok(b"cookie".to_vec()).with_header("Vary", "Cookies"),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::with_method_and_headers(
                    url,
                    "GET",
                    vec![("Cookies".into(), "is-good-enough-for-me".into())],
                ),
                CacheResponse::ok(b"good".to_vec()).with_header("Vary", "Cookies"),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::new(url),
                CacheResponse::ok(b"absent".to_vec()).with_header("Vary", "Cookies"),
            )
            .unwrap();

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.keys(), vec![url, url, url]);

        let matched = cache.match_request(&CacheRequest::with_method_and_headers(
            url,
            "GET",
            vec![("cookies".into(), "is-good-enough-for-me".into())],
        ));
        assert_eq!(matched.unwrap().body, b"good".to_vec());
    }

    #[test]
    fn test_cache_delete_respects_ignore_vary() {
        let mut cache = Cache::new("v1");
        let stored = CacheRequest::with_method_and_headers(
            "https://example.com/c",
            "GET",
            vec![("Cookies".into(), "is-for-cookie".into())],
        );
        cache
            .put(stored, CacheResponse::ok(Vec::new()).with_header("Vary", "Cookies"))
            .unwrap();

        let query = CacheRequest::new("https://example.com/c");
        assert!(!cache.delete(&query));
        assert_eq!(cache.len(), 1);
        assert!(cache.delete_with_options(
            &query,
            CacheQueryOptions {
                ignore_vary: true,
                ..CacheQueryOptions::default()
            },
        ));
        assert!(cache.is_empty());
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
        cache
            .put(CacheRequest::new("https://a.com"), CacheResponse::ok(vec![]))
            .unwrap();
        cache
            .put(CacheRequest::new("https://b.com"), CacheResponse::ok(vec![]))
            .unwrap();

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
        cs.open("v1")
            .put(CacheRequest::new("https://a.com"), CacheResponse::ok(b"x".to_vec()))
            .unwrap();
        cs.open("v1")
            .put(CacheRequest::new("https://b.com"), CacheResponse::ok(b"y".to_vec()))
            .unwrap();

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
        assert_eq!(keys, vec!["v1", "v2"]);
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
        // First-created cache that matches wins.
        assert_eq!(matched.body, b"first".to_vec());
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
        cache
            .put(CacheRequest::new("https://a.com/1"), CacheResponse::ok(b"one".to_vec()))
            .unwrap();
        cache
            .put(CacheRequest::new("https://a.com/2"), CacheResponse::ok(b"two".to_vec()))
            .unwrap();
        cache
            .put(
                CacheRequest::new("https://a.com/3"),
                CacheResponse::ok(b"three".to_vec()),
            )
            .unwrap();
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
        cache
            .put(CacheRequest::new("https://a.com"), CacheResponse::ok(vec![]))
            .unwrap();
        cache
            .put(CacheRequest::new("https://b.com"), CacheResponse::ok(vec![]))
            .unwrap();
        cache
            .put(CacheRequest::new("https://c.com"), CacheResponse::ok(vec![]))
            .unwrap();
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
        cache
            .put(get_req.clone(), CacheResponse::ok(b"get_resp".to_vec()))
            .unwrap();
        assert!(matches!(
            cache.put(post_req.clone(), CacheResponse::ok(b"post_resp".to_vec())),
            Err(StorageError::Type(_))
        ));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.match_request(&get_req).unwrap().body, b"get_resp".to_vec());
        assert!(cache.match_request(&post_req).is_none());
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

    // ── Cache API 集成测试 ──

    /// 测试 Cache API 基本操作：put 存入请求/响应对，match_request 取回，delete 删除。
    #[test]
    fn test_cache_api_put_get_delete() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("assets");
        let req = CacheRequest::new("https://example.com/style.css");
        let resp = CacheResponse::ok(b"body { color: red; }".to_vec());

        // put
        cache.put(req.clone(), resp).unwrap();

        // get（通过 CacheStorage 全局匹配）
        let matched = cs.match_request(&req).unwrap();
        assert_eq!(matched.status, 200);
        assert_eq!(matched.body, b"body { color: red; }".to_vec());

        // delete
        assert!(cs.open("assets").delete(&req));
        assert!(cs.match_request(&req).is_none());
        // 重复删除返回 false
        assert!(!cs.open("assets").delete(&req));
    }

    /// 测试 Cache API 覆盖行为：同一 URL 存入两次，第二次响应覆盖第一次。
    #[test]
    fn test_cache_api_overwrite() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/app.js");

        // 第一次写入
        cs.open("v1")
            .put(req.clone(), CacheResponse::ok(b"console.log('v1');".to_vec()))
            .unwrap();

        // 第二次写入同一 URL
        cs.open("v1")
            .put(req.clone(), CacheResponse::new(200, b"console.log('v2');".to_vec()))
            .unwrap();

        // 应返回第二次的响应
        let matched = cs.match_request(&req).unwrap();
        assert_eq!(matched.body, b"console.log('v2');".to_vec());

        // 缓存中应只有一条记录
        assert_eq!(cs.open("v1").len(), 1);
    }

    /// 测试 CacheStorage::delete — 打开缓存，添加多条目，删除整个缓存后条目不可达
    #[test]
    fn test_cache_storage_delete_entries() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("temp-cache");
        cache
            .put(
                CacheRequest::new("https://example.com/a"),
                CacheResponse::ok(b"a".to_vec()),
            )
            .unwrap();
        cache
            .put(
                CacheRequest::new("https://example.com/b"),
                CacheResponse::ok(b"b".to_vec()),
            )
            .unwrap();

        assert!(cs.has("temp-cache"));
        assert!(cs.delete("temp-cache"));
        assert!(!cs.has("temp-cache"));
        // 删除后全局匹配也不应找到
        let req = CacheRequest::new("https://example.com/a");
        assert!(cs.match_request(&req).is_none());
        // 重复删除返回 false
        assert!(!cs.delete("temp-cache"));
    }

    /// 测试 CacheStorage::has — 多个缓存时 has 对已存在和不存在名称的判断
    #[test]
    fn test_cache_storage_has_multiple() {
        let mut cs = CacheStorage::new();
        assert!(!cs.has("v1"), "未创建的缓存应返回 false");
        assert!(!cs.has("v2"), "未创建的缓存应返回 false");

        cs.open("v1");
        assert!(cs.has("v1"), "已创建的缓存应返回 true");
        assert!(!cs.has("v2"), "未创建的缓存仍应返回 false");
    }

    /// 测试 CacheStorage::keys — 打开多个缓存后 keys() 返回全部名称，删除后更新
    #[test]
    fn test_cache_storage_keys_after_delete() {
        let mut cs = CacheStorage::new();
        cs.open("cache-alpha");
        cs.open("cache-beta");
        cs.open("cache-gamma");

        assert_eq!(cs.keys(), vec!["cache-alpha", "cache-beta", "cache-gamma"]);

        // 删除一个后 keys 更新
        cs.delete("cache-beta");
        assert_eq!(cs.keys(), vec!["cache-alpha", "cache-gamma"]);
    }

    /// 测试 CacheStorage::keys 保留创建顺序，同名 open 不重排，删除后重建追加到末尾。
    #[test]
    fn test_cache_storage_keys_preserve_creation_order() {
        let mut cs = CacheStorage::new();
        cs.open("");
        cs.open("example");
        cs.open("Another cache name");
        cs.open("A");
        cs.open("a");
        cs.open("ex ample");

        assert_eq!(
            cs.keys(),
            vec!["", "example", "Another cache name", "A", "a", "ex ample"]
        );

        cs.open("example");
        assert_eq!(
            cs.keys(),
            vec!["", "example", "Another cache name", "A", "a", "ex ample"],
            "opening an existing cache must not move it"
        );

        assert!(cs.delete("example"));
        cs.open("example");
        assert_eq!(
            cs.keys(),
            vec!["", "Another cache name", "A", "a", "ex ample", "example"],
            "recreating a deleted cache appends it to the end"
        );
    }

    /// 测试 Cache::match_request 方法不匹配 — 缓存 GET 请求后用 POST 匹配应返回 None
    #[test]
    fn test_cache_match_request_method_mismatch() {
        let mut cache = Cache::new("v1");
        let get_req = CacheRequest::new("https://example.com/api");
        let resp = CacheResponse::ok(b"get-response".to_vec());
        cache.put(get_req.clone(), resp).unwrap();

        // GET 请求应能匹配
        assert!(cache.match_request(&get_req).is_some());

        // POST 请求不应匹配
        let post_req = CacheRequest::with_method("https://example.com/api", "POST");
        assert!(cache.match_request(&post_req).is_none(), "方法不匹配时不应返回缓存");
    }

    /// 测试 put 同一请求两次，第二次应覆盖第一次的响应。
    #[test]
    fn test_cache_put_overwrites() {
        let mut cache = Cache::new("v1");
        let req = CacheRequest::new("https://example.com/page");

        // 第一次 put
        cache
            .put(req.clone(), CacheResponse::ok(b"response-v1".to_vec()))
            .unwrap();
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.match_request(&req).unwrap().body, b"response-v1".to_vec());

        // 第二次 put 同一请求 → 覆盖
        cache
            .put(req.clone(), CacheResponse::new(200, b"response-v2".to_vec()))
            .unwrap();
        assert_eq!(cache.len(), 1, "覆盖后条目数应仍为 1");

        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.body, b"response-v2".to_vec(), "应返回第二次 put 的响应");
        assert_eq!(matched.status, 200);
    }

    /// 测试 Cache API keys()：存入多个条目，验证 keys() 返回所有 URL。
    #[test]
    fn test_cache_api_keys() {
        let mut cs = CacheStorage::new();
        let cache = cs.open("resources");

        let urls = [
            "https://example.com/index.html",
            "https://example.com/style.css",
            "https://example.com/app.js",
        ];
        for url in &urls {
            cache.put(CacheRequest::new(url), CacheResponse::ok(vec![])).unwrap();
        }

        let keys = cache.keys();
        assert_eq!(keys.len(), 3);
        for url in &urls {
            assert!(keys.contains(url), "keys() 应包含 {}", url);
        }
    }

    /// 测试 Cache::delete 只删除匹配方法和 URL 的条目，同 URL 不同方法不受影响。
    #[test]
    fn test_cache_delete_preserves_different_method() {
        let mut cache = Cache::new("v1");
        let url = "https://example.com/api";
        let get_req = CacheRequest::new(url);
        let post_req = CacheRequest::with_method(url, "POST");
        let put_req = CacheRequest::with_method(url, "PUT");

        cache.put(get_req.clone(), CacheResponse::ok(b"get".to_vec())).unwrap();
        assert!(matches!(
            cache.put(post_req.clone(), CacheResponse::ok(b"post".to_vec())),
            Err(StorageError::Type(_))
        ));
        assert!(matches!(
            cache.put(put_req.clone(), CacheResponse::ok(b"put".to_vec())),
            Err(StorageError::Type(_))
        ));
        assert_eq!(cache.len(), 1);

        // 非 GET 查询不匹配，除非显式忽略方法。
        assert!(!cache.delete(&post_req));
        assert_eq!(cache.len(), 1);

        assert_eq!(cache.match_request(&get_req).unwrap().body, b"get".to_vec());
        assert!(cache.match_request(&post_req).is_none());

        assert!(cache.delete_with_options(
            &put_req,
            CacheQueryOptions {
                ignore_method: true,
                ..CacheQueryOptions::default()
            },
        ));
        assert!(cache.is_empty());
    }

    // ── Cache API 边界条件测试 ──

    /// 测试 CacheStorage::match_request 在第一个匹配的缓存中返回响应。
    ///
    /// 即使后续缓存有更新的响应，match_request 也只返回第一个匹配。
    #[test]
    fn test_cache_storage_match_first_cache_wins_deterministic() {
        let mut cs = CacheStorage::new();
        let req = CacheRequest::new("https://example.com/data");

        // 先向 v1 写入，再向 v2 写入
        cs.open("v1")
            .put(req.clone(), CacheResponse::ok(b"from-v1".to_vec()))
            .unwrap();
        cs.open("v2")
            .put(req.clone(), CacheResponse::ok(b"from-v2".to_vec()))
            .unwrap();

        // 全局匹配应返回其中一个（HashMap 迭代顺序不确定但应一致）
        let matched = cs.match_request(&req).unwrap();
        let body = matched.body.clone();
        assert!(
            body == b"from-v1".to_vec() || body == b"from-v2".to_vec(),
            "应从某个缓存中匹配到响应"
        );
        assert_eq!(matched.status, 200);
    }

    /// 测试 Cache::put 使用各种 HTTP 状态码的响应。
    #[test]
    fn test_cache_put_various_response_types() {
        let mut cache = Cache::new("multi-status");

        // 200 OK
        let ok_resp = CacheResponse::ok(b"success".to_vec());
        cache.put(CacheRequest::new("https://example.com/ok"), ok_resp).unwrap();

        // 204 No Content
        let no_content = CacheResponse::new(204, vec![]);
        cache
            .put(CacheRequest::new("https://example.com/no-content"), no_content)
            .unwrap();

        // 301 Moved Permanently
        let moved = CacheResponse::new(301, vec![]).with_header("Location", "https://example.com/new");
        cache.put(CacheRequest::new("https://example.com/old"), moved).unwrap();

        // 500 Internal Server Error
        let error_resp = CacheResponse::new(500, b"internal error".to_vec()).with_header("Content-Type", "text/plain");
        cache
            .put(CacheRequest::new("https://example.com/error"), error_resp)
            .unwrap();

        assert_eq!(cache.len(), 4);

        // 验证每个响应的状态码和体
        let ok_matched = cache
            .match_request(&CacheRequest::new("https://example.com/ok"))
            .unwrap();
        assert_eq!(ok_matched.status, 200);
        assert_eq!(ok_matched.body, b"success".to_vec());

        let nc_matched = cache
            .match_request(&CacheRequest::new("https://example.com/no-content"))
            .unwrap();
        assert_eq!(nc_matched.status, 204);
        assert!(nc_matched.body.is_empty());

        let moved_matched = cache
            .match_request(&CacheRequest::new("https://example.com/old"))
            .unwrap();
        assert_eq!(moved_matched.status, 301);
        assert_eq!(
            moved_matched.headers.get("Location"),
            Some(&"https://example.com/new".to_string())
        );

        let err_matched = cache
            .match_request(&CacheRequest::new("https://example.com/error"))
            .unwrap();
        assert_eq!(err_matched.status, 500);
        assert_eq!(err_matched.body, b"internal error".to_vec());
    }

    /// 测试 Cache::delete 后 keys() 正确更新，不影响其他条目。
    #[test]
    fn test_cache_delete_complex_keys_update() {
        let mut cache = Cache::new("cdn");
        let urls: Vec<&str> = (1..=5)
            .map(|i| {
                let urls = [
                    "https://cdn.example.com/app.js",
                    "https://cdn.example.com/style.css",
                    "https://cdn.example.com/logo.png",
                    "https://cdn.example.com/data.json",
                    "https://cdn.example.com/font.woff",
                ];
                urls[i - 1]
            })
            .collect();

        for url in &urls {
            cache
                .put(CacheRequest::new(url), CacheResponse::ok(b"content".to_vec()))
                .unwrap();
        }
        assert_eq!(cache.len(), 5);

        // 删除中间两个
        assert!(cache.delete(&CacheRequest::new("https://cdn.example.com/style.css")));
        assert!(cache.delete(&CacheRequest::new("https://cdn.example.com/data.json")));

        // keys() 应只有 3 个
        let keys = cache.keys();
        assert_eq!(keys.len(), 3);
        assert!(!keys.contains(&"https://cdn.example.com/style.css"));
        assert!(!keys.contains(&"https://cdn.example.com/data.json"));
        assert!(keys.contains(&"https://cdn.example.com/app.js"));
        assert!(keys.contains(&"https://cdn.example.com/logo.png"));
        assert!(keys.contains(&"https://cdn.example.com/font.woff"));
    }

    /// 测试 CacheStorage::keys 在空存储和删除全部缓存后返回空列表。
    #[test]
    fn test_cache_storage_keys_empty_and_after_full_delete() {
        let mut cs = CacheStorage::new();
        // 空存储
        assert!(cs.keys().is_empty());

        // 创建并填充
        cs.open("a")
            .put(CacheRequest::new("https://a.com"), CacheResponse::ok(b"a".to_vec()))
            .unwrap();
        cs.open("b")
            .put(CacheRequest::new("https://b.com"), CacheResponse::ok(b"b".to_vec()))
            .unwrap();
        assert_eq!(cs.keys().len(), 2);

        // 删除全部
        cs.delete("a");
        cs.delete("b");
        assert!(cs.keys().is_empty());
    }

    /// 测试 Cache put 后立即 delete 再 put 同一请求，验证最终状态正确。
    #[test]
    fn test_cache_put_delete_reput() {
        let mut cache = Cache::new("api");

        let req = CacheRequest::new("https://example.com/resource");

        // 第一次 put
        cache.put(req.clone(), CacheResponse::ok(b"v1".to_vec())).unwrap();
        assert_eq!(cache.match_request(&req).unwrap().body, b"v1".to_vec());

        // 删除
        assert!(cache.delete(&req));
        assert!(cache.match_request(&req).is_none());
        assert!(cache.is_empty());

        // 重新 put
        cache.put(req.clone(), CacheResponse::ok(b"v2".to_vec())).unwrap();
        assert_eq!(cache.match_request(&req).unwrap().body, b"v2".to_vec());
        assert_eq!(cache.len(), 1);
    }

    /// 测试 Cache 响应头覆盖：put 新响应时头信息完全替换，不追加。
    #[test]
    fn test_cache_response_headers_overwrite_not_merge() {
        let mut cache = Cache::new("api");
        let req = CacheRequest::new("https://example.com/api");

        // 第一次 put：3 个头
        let resp1 = CacheResponse::ok(b"{}".to_vec())
            .with_header("Content-Type", "application/json")
            .with_header("X-Version", "1")
            .with_header("X-Custom", "old");
        cache.put(req.clone(), resp1).unwrap();
        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.headers.len(), 3);

        // 第二次 put：1 个头（Content-Type 不同）
        let resp2 = CacheResponse::ok(b"{}".to_vec()).with_header("Content-Type", "text/plain");
        cache.put(req.clone(), resp2).unwrap();
        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.headers.len(), 1, "覆盖后应只有新响应的头");
        assert_eq!(matched.headers.get("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(matched.headers.get("X-Version"), None, "旧头应被移除");
        assert_eq!(matched.headers.get("X-Custom"), None, "旧头应被移除");
    }

    /// 测试 CacheStorage 对同名缓存反复 open/delete 周期，验证无内存残留。
    #[test]
    fn test_cache_storage_open_delete_cycle() {
        let mut cs = CacheStorage::new();

        for cycle in 0..3 {
            let name = format!("cache-{cycle}");
            cs.open(&name)
                .put(
                    CacheRequest::new("https://example.com/page"),
                    CacheResponse::ok(format!("cycle-{cycle}").into_bytes()),
                )
                .unwrap();

            assert!(cs.has(&name));
            let matched = cs.match_request(&CacheRequest::new("https://example.com/page"));
            assert!(matched.is_some());

            cs.delete(&name);
            assert!(!cs.has(&name));
        }

        // 最终无任何缓存
        assert!(cs.keys().is_empty());
    }

    /// 测试 Cache::put 空响应体的缓存。
    #[test]
    fn test_cache_put_empty_body() {
        let mut cache = Cache::new("empty");
        let req = CacheRequest::new("https://example.com/no-body");
        cache.put(req.clone(), CacheResponse::new(204, vec![])).unwrap();

        let matched = cache.match_request(&req).unwrap();
        assert_eq!(matched.status, 204);
        assert!(matched.body.is_empty());
        assert_eq!(cache.len(), 1);
    }

    /// 测试 CacheStorage::open 返回的 &mut Cache 可以链式调用 put。
    #[test]
    fn test_cache_storage_open_chain_put() {
        let mut cs = CacheStorage::new();
        cs.open("chain")
            .put(CacheRequest::new("https://a.com/1"), CacheResponse::ok(b"1".to_vec()))
            .unwrap();
        cs.open("chain")
            .put(CacheRequest::new("https://a.com/2"), CacheResponse::ok(b"2".to_vec()))
            .unwrap();
        cs.open("chain")
            .put(CacheRequest::new("https://a.com/3"), CacheResponse::ok(b"3".to_vec()))
            .unwrap();

        let cache = cs.open("chain");
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.keys().len(), 3);
    }
}
