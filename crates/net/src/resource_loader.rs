//! 缓存感知的页面资源加载入口。
//!
//! 该模块把私有 HTTP 缓存、在途请求合并和并发调度放在同一边界。P0 仅处理
//! 无 body 的 GET；其他方法必须经 [`HttpClient`] write-through，并在成功后使相关缓存失效。

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::{
    CacheLookup, FetchJobResult, FetchPriority, HttpCache, HttpClient, HttpMethod, HttpRequest,
    PerOriginFetchScheduler, shared_http_cache,
};

/// 请求对缓存的要求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// 遵循响应新鲜度，正常复用缓存。
    Default,
    /// 不使用未验证的缓存响应。
    NoCache,
    /// 不读写此请求对应的缓存条目。
    NoStore,
    /// 只允许命中缓存；未命中返回 504 语义错误。
    OnlyIfCached,
}

/// 可聚合的资源加载计数；不包含 URL、请求头或响应体等敏感内容。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLoadStats {
    /// 新鲜缓存命中次数。
    pub fresh_hits: u64,
    /// 发起条件再验证次数。
    pub revalidations: u64,
    /// 进入网络调度的次数。
    pub network_requests: u64,
    /// `only-if-cached` 未命中次数。
    pub only_if_cached_misses: u64,
    /// 已完成网络事务的响应体字节数。
    pub network_response_bytes: u64,
    /// 已完成网络事务的总等待毫秒（包含调度等待）。
    pub network_elapsed_ms: u64,
}

/// 可缓存资源请求。
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    /// URL（fragment 不参与请求身份）。
    pub url: String,
    /// 原始请求头。
    pub headers: Vec<(String, String)>,
    /// 浏览器上下文/顶级站点分区；不同分区绝不共享缓存或在途网络事务。
    pub partition: String,
    /// 本地调度优先级。
    pub priority: FetchPriority,
    /// 缓存模式。
    pub cache_mode: CacheMode,
}

impl ResourceRequest {
    /// 构造默认分区的 GET 请求。
    pub fn get(url: impl Into<String>, priority: FetchPriority) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            partition: "default".to_string(),
            priority,
            cache_mode: CacheMode::Default,
        }
    }

    /// 设置请求头。
    pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }

    /// 设置缓存分区。
    pub fn with_partition(mut self, partition: impl Into<String>) -> Self {
        self.partition = partition.into();
        self
    }

    /// 设置缓存模式。
    pub fn with_cache_mode(mut self, cache_mode: CacheMode) -> Self {
        self.cache_mode = cache_mode;
        self
    }

    fn identity_key(&self) -> String {
        // https://www.rfc-editor.org/rfc/rfc9111#section-4.1
        // 在未知 Vary 前保守地纳入全部请求头；这可能少合并，但不会将不同变体错误合并。
        let mut headers: Vec<_> = self
            .headers
            .iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value.clone()))
            .collect();
        headers.sort_unstable();
        let normalized_url = crate::cache_key::strip_url_fragment(&self.url);
        format!(
            "GET\0partition={}\0cache={:?}\0url={}\0headers={headers:?}",
            self.partition, self.cache_mode, normalized_url
        )
    }
}

/// 统一资源加载器。
pub struct ResourceLoader {
    scheduler: Arc<Mutex<PerOriginFetchScheduler>>,
    /// 默认分区的缓存。普通 profile 可在此使用持久化缓存；私密 profile 传入内存缓存。
    default_cache: Arc<Mutex<HttpCache>>,
    /// 非默认顶级站点分区的独立内存缓存。
    ///
    /// P0 保守地不把这些分区写入共享磁盘缓存：磁盘索引尚未携带 partition key 时，
    /// 宁可降低跨会话命中率，也不能让不同顶级站点复用同一条目。
    partition_caches: Mutex<HashMap<String, Arc<Mutex<HttpCache>>>>,
    partition: String,
    stats: Arc<Mutex<ResourceLoadStats>>,
}

impl ResourceLoader {
    /// 使用共享普通浏览上下文缓存创建加载器。
    pub fn shared() -> Arc<Self> {
        static LOADER: OnceLock<Arc<ResourceLoader>> = OnceLock::new();
        LOADER
            .get_or_init(|| Arc::new(Self::new(shared_http_cache(), "default")))
            .clone()
    }

    /// 用指定缓存和分区创建加载器（无痕 profile 使用独立内存缓存）。
    pub fn new(cache: Arc<Mutex<HttpCache>>, partition: impl Into<String>) -> Self {
        Self {
            scheduler: PerOriginFetchScheduler::new_shared(),
            default_cache: cache,
            partition_caches: Mutex::new(HashMap::new()),
            partition: partition.into(),
            stats: Arc::new(Mutex::new(ResourceLoadStats::default())),
        }
    }

    /// 受理 GET 资源请求并立即返回结果接收器。
    pub fn submit(&self, mut request: ResourceRequest) -> Receiver<FetchJobResult> {
        if request.partition == "default" {
            request.partition = self.partition.clone();
        }
        let request_mode = request_cache_mode(&request.headers).unwrap_or(request.cache_mode);
        request.cache_mode = request_mode;
        let cache = self.cache_for_partition(&request.partition);
        if request.cache_mode == CacheMode::NoStore {
            return self.submit_network(cache, request, Vec::new(), false);
        }

        let lookup = cache
            .lock()
            .expect("HTTP cache lock")
            .lookup(&request.url, &request.headers);
        match lookup {
            CacheLookup::Hit(cached) if request.cache_mode != CacheMode::NoCache => {
                self.stats.lock().expect("resource loader stats lock").fresh_hits += 1;
                immediate(Ok(cached.into_response()))
            }
            CacheLookup::Hit(_) | CacheLookup::Revalidate { .. } if request.cache_mode == CacheMode::OnlyIfCached => {
                self.stats
                    .lock()
                    .expect("resource loader stats lock")
                    .only_if_cached_misses += 1;
                // only-if-cached 可使用 fresh 条目；stale 条目不能启动验证网络请求。
                immediate(Err("only-if-cached cache miss (504)".to_string()))
            }
            CacheLookup::Hit(_) => {
                self.stats.lock().expect("resource loader stats lock").revalidations += 1;
                let conditional = cache
                    .lock()
                    .expect("HTTP cache lock")
                    .conditional_headers_with_request(&request.url, &request.headers);
                self.submit_network(cache, request, conditional, true)
            }
            CacheLookup::Revalidate {
                conditional_headers, ..
            } => {
                self.stats.lock().expect("resource loader stats lock").revalidations += 1;
                self.submit_network(cache, request, conditional_headers, true)
            }
            CacheLookup::Miss if request.cache_mode == CacheMode::OnlyIfCached => {
                self.stats
                    .lock()
                    .expect("resource loader stats lock")
                    .only_if_cached_misses += 1;
                immediate(Err("only-if-cached cache miss (504)".to_string()))
            }
            CacheLookup::Miss => self.submit_network(cache, request, Vec::new(), true),
        }
    }

    /// 返回当前加载器的匿名聚合计数。
    pub fn stats(&self) -> ResourceLoadStats {
        *self.stats.lock().expect("resource loader stats lock")
    }

    /// 受理任意 HTTP 请求。
    ///
    /// 无 body 的 GET 使用缓存和共享调度；其他方法 write-through，成功后使目标 URI
    /// 的全部缓存变体失效。
    pub fn submit_http(&self, request: HttpRequest, priority: FetchPriority) -> Receiver<FetchJobResult> {
        self.submit_http_in_partition(request, priority, self.partition.clone())
    }

    /// 受理 HTTP 请求并显式指定顶级站点缓存分区。
    pub fn submit_http_in_partition(
        &self,
        request: HttpRequest,
        priority: FetchPriority,
        partition: impl Into<String>,
    ) -> Receiver<FetchJobResult> {
        let partition = partition.into();
        if request.method == HttpMethod::Get && request.body.is_none() {
            return self.submit(
                ResourceRequest::get(request.url, priority)
                    .with_headers(request.headers)
                    .with_partition(partition),
            );
        }
        let cache = self.cache_for_partition(&partition);
        let url = request.url.clone();
        let (tx, rx) = mpsc::channel();
        crate::client::async_runtime().spawn(async move {
            let result = HttpClient::send_async_with_timeout(30, request)
                .await
                .map_err(|error| error.to_string());
            if matches!(result, Ok(ref response) if response.is_success()) {
                cache.lock().expect("HTTP cache lock").invalidate(&url);
            }
            let _ = tx.send(result);
        });
        rx
    }

    /// 成功的 unsafe 请求必须使目标 URI 的缓存变为不可复用。
    pub fn invalidate_after_unsafe(&self, url: &str) {
        self.cache_for_partition(&self.partition)
            .lock()
            .expect("HTTP cache lock")
            .invalidate(url);
    }

    fn submit_network(
        &self,
        cache: Arc<Mutex<HttpCache>>,
        request: ResourceRequest,
        conditional_headers: Vec<(String, String)>,
        may_store: bool,
    ) -> Receiver<FetchJobResult> {
        self.stats.lock().expect("resource loader stats lock").network_requests += 1;
        let mut headers = request.headers.clone();
        for (name, value) in conditional_headers {
            if !headers.iter().any(|(existing, _)| existing.eq_ignore_ascii_case(&name)) {
                headers.push((name, value));
            }
        }
        let key = request.identity_key();
        let rx = PerOriginFetchScheduler::submit_shared_with_key_and_headers(
            &self.scheduler,
            key,
            request.url.clone(),
            request.priority,
            headers,
        );
        let url = request.url;
        let request_headers = request.headers;
        let started = Instant::now();
        let stats = Arc::clone(&self.stats);
        bridge(rx, move |result| {
            let mut stats = stats.lock().expect("resource loader stats lock");
            stats.network_elapsed_ms += started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            if let Ok(response) = &result {
                stats.network_response_bytes += response.body.len() as u64;
            }
            drop(stats);
            match result {
                Ok(response) if response.status_code == 304 => cache
                    .lock()
                    .expect("HTTP cache lock")
                    .not_modified(&url, &request_headers, &response)
                    .map(|cached| cached.into_response())
                    .ok_or_else(|| "304 without cached entry".to_string()),
                Ok(response) => {
                    if may_store && response.is_success() {
                        let _ =
                            cache
                                .lock()
                                .expect("HTTP cache lock")
                                .put_with_headers(&url, &request_headers, &response);
                    }
                    Ok(response)
                }
                Err(error) => Err(error),
            }
        })
    }

    fn cache_for_partition(&self, partition: &str) -> Arc<Mutex<HttpCache>> {
        if partition == self.partition {
            return Arc::clone(&self.default_cache);
        }
        let mut caches = self.partition_caches.lock().expect("HTTP cache partition lock");
        caches
            .entry(partition.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(HttpCache::new())))
            .clone()
    }
}

fn immediate(result: FetchJobResult) -> Receiver<FetchJobResult> {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(result);
    rx
}

fn bridge<F>(rx: Receiver<FetchJobResult>, map: F) -> Receiver<FetchJobResult>
where
    F: FnOnce(FetchJobResult) -> FetchJobResult + Send + 'static,
{
    let (tx, out) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(result) = rx.recv() {
            let _ = tx.send(map(result));
        }
    });
    out
}

fn request_cache_mode(headers: &[(String, String)]) -> Option<CacheMode> {
    let value = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("cache-control"))?
        .1
        .as_str();
    if value
        .split(',')
        .any(|directive| directive.trim().eq_ignore_ascii_case("only-if-cached"))
    {
        Some(CacheMode::OnlyIfCached)
    } else if value
        .split(',')
        .any(|directive| directive.trim().eq_ignore_ascii_case("no-store"))
    {
        Some(CacheMode::NoStore)
    } else if value
        .split(',')
        .any(|directive| directive.trim().eq_ignore_ascii_case("no-cache"))
    {
        Some(CacheMode::NoCache)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_response(url: &str) -> crate::HttpResponse {
        crate::HttpResponse {
            status_code: 200,
            headers: vec![("Cache-Control".into(), "max-age=60".into())],
            body: b"cached".to_vec(),
            url: url.into(),
            redirect_count: 0,
        }
    }

    #[test]
    fn request_identity_is_order_independent_but_partitioned() {
        let a = ResourceRequest::get("https://example.com/a#fragment", FetchPriority::MEDIUM).with_headers(vec![
            ("Accept-Language".into(), "en".into()),
            ("Cookie".into(), "a=1".into()),
        ]);
        let b = ResourceRequest::get("https://example.com/a", FetchPriority::MEDIUM).with_headers(vec![
            ("Cookie".into(), "a=1".into()),
            ("accept-language".into(), "en".into()),
        ]);
        assert_eq!(a.identity_key(), b.identity_key());
        assert_ne!(a.identity_key(), b.with_partition("other-site").identity_key());
        assert_ne!(
            a.identity_key(),
            ResourceRequest::get("https://example.com/a", FetchPriority::MEDIUM)
                .with_headers(vec![("Cookie".into(), "a=2".into())])
                .identity_key()
        );
        assert_ne!(
            a.identity_key(),
            a.clone().with_cache_mode(CacheMode::NoCache).identity_key(),
            "a forced revalidation must not join a normal cache transaction"
        );
    }

    #[test]
    fn request_cache_control_is_parsed() {
        assert_eq!(
            request_cache_mode(&[("Cache-Control".into(), "no-cache".into())]),
            Some(CacheMode::NoCache)
        );
        assert_eq!(
            request_cache_mode(&[("Cache-Control".into(), "only-if-cached".into())]),
            Some(CacheMode::OnlyIfCached)
        );
        assert_eq!(
            request_cache_mode(&[("cache-control".into(), "max-age=0, NO-CACHE".into())]),
            Some(CacheMode::NoCache)
        );
        assert_eq!(
            request_cache_mode(&[("Cache-Control".into(), "no-store, no-cache".into())]),
            Some(CacheMode::NoStore)
        );
        assert_eq!(request_cache_mode(&[]), None);
    }

    #[test]
    fn partition_cache_instances_are_stable_and_isolated() {
        let default_cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&default_cache), "site-a");

        assert!(Arc::ptr_eq(&loader.cache_for_partition("site-a"), &default_cache));
        let first_site_b = loader.cache_for_partition("site-b");
        let second_site_b = loader.cache_for_partition("site-b");
        assert!(Arc::ptr_eq(&first_site_b, &second_site_b));
        assert!(!Arc::ptr_eq(&first_site_b, &default_cache));
    }

    #[test]
    fn fresh_default_partition_entry_is_not_visible_to_other_partition() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "site-a");
        let url = "https://cdn.example/asset.css";
        assert!(cache.lock().unwrap().put(url, &fresh_response(url)));

        let cached = loader
            .submit(ResourceRequest::get(url, FetchPriority::CRITICAL))
            .recv()
            .expect("fresh cache response")
            .expect("successful fresh cache response");
        assert_eq!(cached.body, b"cached");
        assert_eq!(loader.stats().fresh_hits, 1);
        assert_eq!(loader.stats().network_requests, 0);

        let isolated = loader
            .submit(
                ResourceRequest::get(url, FetchPriority::CRITICAL)
                    .with_partition("site-b")
                    .with_cache_mode(CacheMode::OnlyIfCached),
            )
            .recv()
            .expect("only-if-cached result");
        assert!(isolated.is_err(), "another partition must not see site-a's entry");
        assert_eq!(loader.stats().only_if_cached_misses, 1);
    }
}
