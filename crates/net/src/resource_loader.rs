//! 缓存感知的页面资源加载入口。
//!
//! 该模块把私有 HTTP 缓存、在途请求合并和并发调度放在同一边界。P0 仅处理
//! 无 body 的 GET；其他方法必须经 [`HttpClient`] write-through，并在成功后使相关缓存失效。

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::cache_policy::{bypass_cache_request_injection, has_explicit_conditional_request_headers};
use crate::fetch_scheduler::FetchTelemetry;
use crate::{
    CacheLookup, FetchJobResult, FetchPriority, HttpCache, HttpClient, HttpMethod, HttpRequest, HttpResponse,
    PerOriginFetchScheduler, shared_cookie_store, shared_http_cache, store_set_cookie_headers,
};

const MAX_RESOURCE_LOAD_EVENTS: usize = 1024;

/// 缓存决策结果，不包含 URL 或请求头。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOutcome {
    /// 使用了新鲜缓存响应。
    FreshHit,
    /// 使用条件请求重新验证缓存。
    Revalidated,
    /// 未命中缓存并完成网络请求。
    Network,
    /// `only-if-cached` 无法由缓存满足。
    OnlyIfCachedMiss,
    /// 非安全方法完成后失效缓存。
    UnsafeWrite,
}

/// 单个资源加载的匿名生命周期事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLoadEvent {
    /// 宿主赋予的导航编号；未提供时为 `None`。
    pub navigation_id: Option<u64>,
    /// 资源目的地，如 `image`、`style` 或 `document`。
    pub destination: String,
    /// 不含路径、查询参数的 origin。
    pub origin: String,
    /// 调度器排队到开始网络传输的时间。
    pub queue_wait_ms: u64,
    /// 网络传输及响应体读取时间。
    pub network_ms: u64,
    /// 已读取的响应体字节数。
    pub bytes: u64,
    /// 调度优先级。
    pub priority: FetchPriority,
    /// 已协商协议；当前 transport 未公开该信息时为 `unknown`。
    pub protocol: String,
    /// 缓存决策结果。
    pub cache_outcome: CacheOutcome,
    /// 共享该网络事务的订阅者数。
    pub coalesced_subscriber_count: usize,
}

/// 请求对缓存的要求（对齐 fetch spec `cache` 六模式态；前四态为既有语义，
/// `Reload`/`ForceCache` 为 http-cache goal M1 扩展）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// 遵循响应新鲜度，正常复用缓存。
    Default,
    /// 不使用未验证的缓存响应：有存储条目则条件再验证后使用网络结果。
    ///
    /// 网络可见注入 `Cache-Control: max-age=0`（用户未显式给出 Cache-Control 时；
    /// fetch spec §http-network-or-cache-fetch step 17，WPT no-cache 文件
    /// expected_max_age_headers 断言面）。
    NoCache,
    /// 不读写此请求对应的缓存条目。
    ///
    /// 网络可见注入 `Pragma: no-cache` + `Cache-Control: no-cache`（fetch spec
    /// §http-network-or-cache-fetch step 18，与 reload 同形）。
    NoStore,
    /// 只允许命中缓存；未命中返回 504 语义错误。
    ///
    /// fetch spec §http-cache-fetch：仅 same-origin 模式合法（shim 校验）；
    /// 有存储条目（含 stale）即服务且不验证，零条目 → 504 语义网络错误。
    OnlyIfCached,
    /// 强制走网络（不读缓存）；网络成功后按存储策略覆盖写缓存。
    ///
    /// fetch spec §http-network-or-cache-fetch：reload 请求跳过 HTTP-cache
    /// lookup，响应照常可入缓存（覆盖同键旧条目）；网络可见头注入
    /// `Pragma: no-cache` + `Cache-Control: no-cache`（step 18，用户已给
    /// 对应头则不重复；WPT reload 文件 expected_no_cache_headers 断言面）。
    Reload,
    /// 有存储条目即用（含 stale，不验证）；零条目才走网络并入缓存。
    ///
    /// fetch spec §http-network-or-cache-fetch：force-cache 对 stale 条目
    /// 不发起再验证直接复用；`Vary: *` 条目永不匹配（§4.1）→ 视为未命中。
    ForceCache,
}

impl CacheMode {
    /// fetch 通道 `init.cache` 枚举字符串 → 缓存模式（http-cache goal M2）。
    ///
    /// `None` / 空串 / `default` / 未识别值 → `None`：调用方按 [`CacheMode::Default`]
    /// 处理且**保留**请求头 Cache-Control 启发式推导（引擎内部资源通道既有行为）；
    /// 显式五模式为权威值——fetch spec 模式态仅来自 `init.cache`，头启发式不再覆盖。
    /// 枚举合法性（含 only-if-cached 须配 same-origin 的 TypeError）由 shim 层校验，
    /// 此处不做二次拒绝（未识别值降级 Default——wire 上不可能出现，防御性容错）。
    pub fn from_request_cache(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("") {
            "no-store" => Some(Self::NoStore),
            "reload" => Some(Self::Reload),
            "no-cache" => Some(Self::NoCache),
            "force-cache" => Some(Self::ForceCache),
            "only-if-cached" => Some(Self::OnlyIfCached),
            _ => None,
        }
    }
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
    /// 可选导航编号，仅用于匿名加载事件关联。
    pub navigation_id: Option<u64>,
    /// 资源目的地，仅用于调度与匿名事件；默认 `other`。
    pub destination: String,
    /// 网络超时秒数；不同超时不得合并同一在途事务。
    pub timeout_secs: u64,
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
            navigation_id: None,
            destination: "other".to_string(),
            timeout_secs: 30,
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

    /// 设置宿主导航编号。
    pub fn with_navigation_id(mut self, navigation_id: u64) -> Self {
        self.navigation_id = Some(navigation_id);
        self
    }

    /// 设置资源目的地。
    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = destination.into();
        self
    }

    /// 设置网络超时秒数。
    pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
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
            "GET\0partition={}\0cache={:?}\0timeout={}\0url={}\0headers={headers:?}",
            self.partition, self.cache_mode, self.timeout_secs, normalized_url
        )
    }
}

/// 统一资源加载器。
pub struct ResourceLoader {
    scheduler: Arc<Mutex<PerOriginFetchScheduler>>,
    /// 默认分区的缓存。普通 profile 可在此使用持久化缓存；私密 profile 传入内存缓存。
    default_cache: Arc<Mutex<HttpCache>>,
    persistent_partition_root: Option<std::path::PathBuf>,
    /// 非默认顶级站点分区的独立缓存；普通 profile 按分区持久化，无痕 profile 仅驻留内存。
    partition_caches: Mutex<HashMap<String, Arc<Mutex<HttpCache>>>>,
    partition: String,
    stats: Arc<Mutex<ResourceLoadStats>>,
    events: Arc<Mutex<Vec<ResourceLoadEvent>>>,
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
        let persistent_partition_root = cache.lock().ok().and_then(|cache| cache.persistent_root());
        Self {
            scheduler: PerOriginFetchScheduler::new_shared(),
            default_cache: cache,
            persistent_partition_root,
            partition_caches: Mutex::new(HashMap::new()),
            partition: partition.into(),
            stats: Arc::new(Mutex::new(ResourceLoadStats::default())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 受理 GET 资源请求并立即返回结果接收器。
    pub fn submit(&self, mut request: ResourceRequest) -> Receiver<FetchJobResult> {
        if request.partition == "default" {
            request.partition = self.partition.clone();
        }
        if request
            .url
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return immediate(load_data_url(&request.url));
        }
        // 模式优先级（http-cache goal M2）：显式 cache_mode（fetch 通道 `init.cache`
        // 经 HttpRequest 传播）为权威——fetch spec §4.6 模式态仅来自 init.cache，
        // 请求头 Cache-Control 启发式只服务引擎内部资源通道（cache_mode=Default
        // 时的既有行为，零变化）。
        // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
        let request_mode = if request.cache_mode == CacheMode::Default {
            request_cache_mode(&request.headers).unwrap_or(request.cache_mode)
        } else {
            request.cache_mode
        };
        request.cache_mode = request_mode;
        let cache = self.cache_for_partition(&request.partition);
        if request.cache_mode == CacheMode::NoStore {
            return self.submit_network(cache, request, Vec::new(), false);
        }
        if request.cache_mode == CacheMode::Reload {
            // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            // reload 不读缓存；网络成功后按存储策略覆盖写（may_store=true）。
            return self.submit_network(cache, request, Vec::new(), true);
        }
        // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch（step 16）
        // default 模式带脚本显式条件头（If-Modified-Since 等五头）→ 置为 no-store
        // 旁路：不读缓存、不写缓存，用户条件头原样透传、零验证器合并（调用方自行
        // 管理再验证；Chromium 对齐，WPT request-cache-default-conditional 断言面）。
        // 置为 NoStore 后 submit_network 按 step 18 注入 Pragma/CC no-cache。
        if request.cache_mode == CacheMode::Default && has_explicit_conditional_request_headers(&request.headers) {
            request.cache_mode = CacheMode::NoStore;
            return self.submit_network(cache, request, Vec::new(), false);
        }

        let lookup = cache
            .lock()
            .expect("HTTP cache lock")
            .lookup(&request.url, &request.headers);
        match lookup {
            CacheLookup::Hit(cached) if request.cache_mode != CacheMode::NoCache => {
                self.stats.lock().expect("resource loader stats lock").fresh_hits += 1;
                self.record_event(&request, CacheOutcome::FreshHit, None, 0);
                immediate(Ok(cached.into_response()))
            }
            // https://fetch.spec.whatwg.org/#http-cache-fetch
            // only-if-cached 有存储条目（含 stale）即服务且不验证；零条目 →
            // 504 语义错误，不能启动验证网络请求。`Vary: *` 条目复用禁止
            //（RFC 9111 §4.1）——落条件再验证臂。
            CacheLookup::Revalidate { cached, .. }
                if request.cache_mode == CacheMode::OnlyIfCached && !cached_response_is_vary_star(&cached) =>
            {
                self.stats.lock().expect("resource loader stats lock").fresh_hits += 1;
                self.record_event(&request, CacheOutcome::FreshHit, None, 0);
                immediate(Ok(cached.into_response()))
            }
            // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            // force-cache 对 stale 条目不发起再验证直接复用（`Vary: *` 条目复用
            // 禁止——落条件再验证臂，RFC 9111 §4.1）。
            CacheLookup::Revalidate { cached, .. }
                if request.cache_mode == CacheMode::ForceCache && !cached_response_is_vary_star(&cached) =>
            {
                self.stats.lock().expect("resource loader stats lock").fresh_hits += 1;
                self.record_event(&request, CacheOutcome::FreshHit, None, 0);
                immediate(Ok(cached.into_response()))
            }
            CacheLookup::Hit(_) => {
                let conditional = cache
                    .lock()
                    .expect("HTTP cache lock")
                    .conditional_headers_with_request(&request.url, &request.headers);
                self.submit_network(cache, request, conditional, true)
            }
            CacheLookup::Revalidate {
                conditional_headers, ..
            } => self.submit_network(cache, request, conditional_headers, true),
            CacheLookup::Miss if request.cache_mode == CacheMode::OnlyIfCached => {
                self.stats
                    .lock()
                    .expect("resource loader stats lock")
                    .only_if_cached_misses += 1;
                self.record_event(&request, CacheOutcome::OnlyIfCachedMiss, None, 0);
                immediate(Err("only-if-cached cache miss (504)".to_string()))
            }
            CacheLookup::Miss => self.submit_network(cache, request, Vec::new(), true),
        }
    }

    /// 返回当前加载器的匿名聚合计数。
    pub fn stats(&self) -> ResourceLoadStats {
        *self.stats.lock().expect("resource loader stats lock")
    }

    /// Check an HTTP cache entry before a caller starts a progressive resource transfer.
    pub fn lookup_cache(&self, request: &ResourceRequest) -> CacheLookup {
        let mut request = request.clone();
        if request.partition == "default" {
            request.partition = self.partition.clone();
        }
        // 显式 cache_mode 权威（见 submit 优先级注记——fetch 通道 init.cache）。
        request.cache_mode = if request.cache_mode == CacheMode::Default {
            request_cache_mode(&request.headers).unwrap_or(request.cache_mode)
        } else {
            request.cache_mode
        };
        if matches!(request.cache_mode, CacheMode::NoStore | CacheMode::Reload) {
            // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            // no-store / reload 均不读缓存。
            return CacheLookup::Miss;
        }

        let cache = self.cache_for_partition(&request.partition);
        let lookup = cache
            .lock()
            .expect("HTTP cache lock")
            .lookup(&request.url, &request.headers);
        let lookup = match lookup {
            CacheLookup::Hit(cached) if request.cache_mode == CacheMode::NoCache => {
                let conditional_headers = cache
                    .lock()
                    .expect("HTTP cache lock")
                    .conditional_headers_with_request(&request.url, &request.headers);
                CacheLookup::Revalidate {
                    cached,
                    conditional_headers,
                }
            }
            // https://fetch.spec.whatwg.org/#http-cache-fetch
            // only-if-cached / force-cache 对 stale（待验证）条目直接复用不验证；
            // `Vary: *` 条目复用禁止（RFC 9111 §4.1）——保持 Revalidate 走条件臂。
            CacheLookup::Revalidate { cached, .. }
                if matches!(request.cache_mode, CacheMode::OnlyIfCached | CacheMode::ForceCache)
                    && !cached_response_is_vary_star(&cached) =>
            {
                CacheLookup::Hit(cached)
            }
            lookup => lookup,
        };
        if matches!(lookup, CacheLookup::Hit(_)) {
            self.stats.lock().expect("resource loader stats lock").fresh_hits += 1;
            self.record_event(&request, CacheOutcome::FreshHit, None, 0);
        }
        lookup
    }

    /// Store a fully received response body from a progressive GET transfer.
    pub fn store_streamed_response(&self, request: &ResourceRequest, response: &HttpResponse) -> bool {
        // 显式 cache_mode 权威（见 submit 优先级注记——fetch 通道 init.cache）。
        let mode = if request.cache_mode == CacheMode::Default {
            request_cache_mode(&request.headers).unwrap_or(request.cache_mode)
        } else {
            request.cache_mode
        };
        if matches!(mode, CacheMode::NoStore | CacheMode::OnlyIfCached) || !storeable_response_shape(response) {
            return false;
        }
        let partition = if request.partition == "default" {
            &self.partition
        } else {
            &request.partition
        };
        self.cache_for_partition(partition)
            .lock()
            .expect("HTTP cache lock")
            .put_with_headers(&request.url, &request.headers, response)
    }

    /// 返回最近的匿名加载事件，按完成顺序排列。
    pub fn events(&self) -> Vec<ResourceLoadEvent> {
        self.events.lock().expect("resource loader events lock").clone()
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
        self.submit_http_with_context_in_partition(request, priority, partition, None, "other")
    }

    /// 受理 HTTP 请求并将导航与资源目的地写入匿名加载事件。
    pub fn submit_http_with_context_in_partition(
        &self,
        request: HttpRequest,
        priority: FetchPriority,
        partition: impl Into<String>,
        navigation_id: Option<u64>,
        destination: impl Into<String>,
    ) -> Receiver<FetchJobResult> {
        let partition = partition.into();
        let destination = destination.into();
        if request.method == HttpMethod::Get && request.body.is_none() {
            let mut resource_request = ResourceRequest::get(request.url, priority)
                .with_headers(request.headers)
                .with_partition(partition)
                .with_destination(destination)
                // http-cache goal M2：fetch 通道 `init.cache` 六态经 HttpRequest
                // 传播到缓存决策（显式模式权威优先，见 submit 优先级注记）。
                .with_cache_mode(request.cache_mode);
            if let Some(navigation_id) = navigation_id {
                resource_request = resource_request.with_navigation_id(navigation_id);
            }
            return self.submit(resource_request);
        }
        let cache = self.cache_for_partition(&partition);
        let url = request.url.clone();
        let request_headers = request.headers.clone();
        let event_request = ResourceRequest::get(url.clone(), priority)
            .with_partition(partition)
            .with_destination(destination);
        let event_request = if let Some(navigation_id) = navigation_id {
            event_request.with_navigation_id(navigation_id)
        } else {
            event_request
        };
        let events = Arc::clone(&self.events);
        let (tx, rx) = mpsc::channel();
        crate::client::async_runtime().spawn(async move {
            // FIXME(t8k-delta F1)：此处曾有 HTTP 请求 cookie 注入（merge_cookie_request_header），
            // 因 context-free 超发 SameSite=Lax/Strict（RFC 6265bis §5.4）且注入头不参与
            // identity_key/缓存查找/写入（跨 cookie 态同 partition 条目串味），已在合并前摘除
            // （HTTP 注入面无功能依赖，WS 握手走 ws_handshake_cookie 自行取 jar）。
            // 恢复注入前须：context 计算 + merge 前移到 identity/lookup 之前。
            let result = HttpClient::send_async_with_timeout(30, request)
                .await
                .map_err(|error| error.to_string());
            if let Ok(response) = &result {
                {
                    let jar = shared_cookie_store();
                    let mut store = jar.lock().expect("shared cookie store lock");
                    store_set_cookie_headers(&mut store, &url, &response.headers);
                }
                // RFC 9111 §4.4——non-error（2xx|3xx）响应使目标 URI 缓存失效（含
                // 最终 URL 与同源 Location/Content-Location 目标，见
                // `unsafe_invalidation_targets`）。
                if response.is_success() || response.is_redirect() {
                    let mut cache = cache.lock().expect("HTTP cache lock");
                    for target in unsafe_invalidation_targets(&url, response) {
                        cache.invalidate(&target);
                    }
                    // RFC 9110 §9.3.3 / RFC 9111 §4——POST/PATCH 响应仅在 Content-Location
                    // 与有效请求 URI 一致时可作为该资源的表示落盘，后续 GET 复用
                    // （http-cache goal M3 post-patch 语料：200 + `private, max-age=1000`
                    // + `Content-Location: ""`）。失效先于存储：本响应自身按 §3 可存性
                    // 落盘，不被自己的失效抹除。
                    if response.is_success() && content_location_matches_target(&url, response) {
                        cache.put_with_headers(&url, &request_headers, response);
                    }
                }
            }
            let bytes = result.as_ref().map(|response| response.body.len() as u64).unwrap_or(0);
            record_event_into(&events, &event_request, CacheOutcome::UnsafeWrite, None, bytes);
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

    /// 取消指定 URL 的全部挂起加载（排队回错、在途断连），返回取消的任务数。
    ///
    /// BG-03 M2-S1：文档卸载放弃挂起子资源时调用——不取消则连接滞留至 client 级
    /// 超时，期间持续占用 per-origin 槽位与 FetchBridge launch 名额，队头阻塞同源
    /// 后续请求。同 URL 的缓存合并订阅者一并收到取消错误（合并语义下无法区分发起方）。
    pub fn cancel_inflight_url(&self, url: &str) -> usize {
        self.scheduler.lock().expect("fetch scheduler lock").cancel_by_url(url)
    }

    fn submit_network(
        &self,
        cache: Arc<Mutex<HttpCache>>,
        request: ResourceRequest,
        conditional_headers: Vec<(String, String)>,
        may_store: bool,
    ) -> Receiver<FetchJobResult> {
        let mut headers = request.headers.clone();
        let is_revalidation = !conditional_headers.is_empty();
        for (name, value) in conditional_headers {
            if !headers.iter().any(|(existing, _)| existing.eq_ignore_ascii_case(&name)) {
                headers.push((name, value));
            }
        }
        // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch（step 17/18）
        // 可观察头注入（用户已给对应头则不重复——spec「不含才追加」；WPT
        // request-cache 语料的 expected_no_cache_headers / expected_max_age_headers
        // 即此形态，2026-11-08 本地语料逐文件核实）：
        // - no-cache → `Cache-Control: max-age=0`（step 17；WPT no-cache 文件
        //   expected_max_age_headers [true]）；
        // - no-store / reload（含 default+显式条件头旁路，见 submit）→
        //   `Pragma: no-cache` + `Cache-Control: no-cache`（step 18；WPT reload /
        //   no_store 文件 expected_no_cache_headers [true] 双头在场）。
        // 注入只影响网络请求头，不参与 identity_key（键取 request.headers 原头）。
        if !headers
            .iter()
            .any(|(existing, _)| existing.eq_ignore_ascii_case("cache-control"))
            && request.cache_mode == CacheMode::NoCache
        {
            headers.push(("Cache-Control".into(), "max-age=0".into()));
        }
        if matches!(request.cache_mode, CacheMode::NoStore | CacheMode::Reload) {
            for pair in bypass_cache_request_injection(&headers) {
                headers.push(pair);
            }
        }
        // FIXME(t8k-delta F1)：cookie 注入摘除理由同 submit_http_with_context_in_partition 内注释
        // （SameSite 超发 + 缓存身份不一致；恢复前须 context 计算并前移到 identity_key 之前）。
        let key = request.identity_key();
        let (rx, telemetry_rx, owns_telemetry) = PerOriginFetchScheduler::submit_shared_with_key_headers_and_telemetry(
            &self.scheduler,
            key,
            request.url.clone(),
            request.priority,
            headers,
            request.timeout_secs,
        );
        if owns_telemetry {
            let mut stats = self.stats.lock().expect("resource loader stats lock");
            stats.network_requests += 1;
            if is_revalidation {
                stats.revalidations += 1;
            }
        }
        let event_request = request.clone();
        let cache_outcome = if is_revalidation {
            CacheOutcome::Revalidated
        } else {
            CacheOutcome::Network
        };
        let url = request.url;
        let request_headers = request.headers;
        let started = Instant::now();
        let stats = Arc::clone(&self.stats);
        let events = Arc::clone(&self.events);
        bridge(rx, move |result| {
            let telemetry = owns_telemetry.then(|| telemetry_rx.recv().ok()).flatten();
            let mut stats = stats.lock().expect("resource loader stats lock");
            stats.network_elapsed_ms += started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            if let Ok(response) = &result {
                stats.network_response_bytes += response.body.len() as u64;
            }
            drop(stats);
            let bytes = result.as_ref().map(|response| response.body.len() as u64).unwrap_or(0);
            record_event_into(&events, &event_request, cache_outcome, telemetry.as_ref(), bytes);
            match result {
                Ok(response) if response.status_code == 304 => cache
                    .lock()
                    .expect("HTTP cache lock")
                    .not_modified(&url, &request_headers, &response)
                    .map(|cached| cached.into_response())
                    .ok_or_else(|| "304 without cached entry".to_string()),
                Ok(response) => {
                    {
                        let jar = shared_cookie_store();
                        let mut store = jar.lock().expect("shared cookie store lock");
                        store_set_cookie_headers(&mut store, &url, &response.headers);
                    }
                    if may_store && storeable_response_shape(&response) {
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
            .or_insert_with(|| {
                let cache = self
                    .persistent_partition_root
                    .as_deref()
                    .map(|root| HttpCache::open_persistent_partition(root, partition))
                    .unwrap_or_default();
                Arc::new(Mutex::new(cache))
            })
            .clone()
    }

    fn record_event(
        &self,
        request: &ResourceRequest,
        cache_outcome: CacheOutcome,
        telemetry: Option<&FetchTelemetry>,
        bytes: u64,
    ) {
        record_event_into(&self.events, request, cache_outcome, telemetry, bytes);
    }
}

fn load_data_url(input: &str) -> FetchJobResult {
    let parsed = data_url::DataUrl::process(input).map_err(|error| error.to_string())?;
    let content_type = parsed.mime_type().to_string();
    let (body, _) = parsed.decode_to_vec().map_err(|error| error.to_string())?;
    let mut url = url::Url::parse(input).map_err(|error| error.to_string())?;
    url.set_fragment(None);
    Ok(crate::HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".into(), content_type)],
        body,
        url: url.to_string(),
        redirect_count: 0,
    })
}

/// 计算 unsafe 请求成功后需要失效的同源缓存 URI。
///
/// https://www.rfc-editor.org/rfc/rfc9111#section-4.4
/// pub：headless 代理 fetch 非 GET 直发臂（`apps/browser` session.rs）与 GUI
/// ResourceLoader 共用同一目标集语义（请求 URL + 最终 URL + 同源
/// Location/Content-Location）。
pub fn unsafe_invalidation_targets(request_url: &str, response: &crate::HttpResponse) -> Vec<String> {
    let Ok(request_url) = url::Url::parse(request_url) else {
        return vec![request_url.to_string()];
    };
    let request_origin = request_url.origin();
    let mut targets = vec![request_url.to_string()];
    let response_base = url::Url::parse(&response.url).unwrap_or_else(|_| request_url.clone());
    targets.push(response_base.to_string());
    for (name, value) in &response.headers {
        if (name.eq_ignore_ascii_case("location") || name.eq_ignore_ascii_case("content-location"))
            && let Ok(target) = response_base.join(value)
            && target.origin() == request_origin
        {
            targets.push(target.to_string());
        }
    }
    targets.sort_unstable();
    targets.dedup();
    targets
}

fn record_event_into(
    events: &Arc<Mutex<Vec<ResourceLoadEvent>>>,
    request: &ResourceRequest,
    cache_outcome: CacheOutcome,
    telemetry: Option<&FetchTelemetry>,
    bytes: u64,
) {
    let mut events = events.lock().expect("resource loader events lock");
    if events.len() == MAX_RESOURCE_LOAD_EVENTS {
        events.remove(0);
    }
    events.push(ResourceLoadEvent {
        navigation_id: request.navigation_id,
        destination: request.destination.clone(),
        origin: telemetry
            .map(|event| event.origin.clone())
            .unwrap_or_else(|| crate::resource_policy::origin_from_url(&request.url)),
        queue_wait_ms: telemetry.map(|event| event.queue_wait_ms).unwrap_or(0),
        network_ms: telemetry.map(|event| event.network_ms).unwrap_or(0),
        bytes,
        priority: request.priority,
        protocol: telemetry
            .map(|event| event.protocol.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        cache_outcome,
        coalesced_subscriber_count: telemetry.map(|event| event.coalesced_subscriber_count).unwrap_or(1),
    });
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
    crate::client::spawn_network_bridge(move || {
        if let Ok(result) = rx.recv() {
            let _ = tx.send(map(result));
        }
    });
    out
}

/// GET 通道的响应落盘形态门（§3 可存性本体由 [`crate::cache_policy::storable_mode`]
/// 统一裁决——http-cache goal M3：status 语料 fresh 4xx/5xx/未知码腿依赖非 2xx 落盘）。
/// 3xx 复用需重定向链语义支撑（FIXME(http-cache): 重定向缓存）、206 需 Range 支持
/// （RFC 9111 §3.1——不支持 Range 的缓存 MUST NOT 存部分内容），二者不落盘。
fn storeable_response_shape(response: &HttpResponse) -> bool {
    !(300..=399).contains(&response.status_code) && response.status_code != 206
}

/// POST/PATCH 响应的 `Content-Location` 是否与有效请求 URI 一致（RFC 9110 §9.3.3）；
/// 头缺失或 URL 不可解析按不一致处理。空引用（`""`）join 到响应最终 URL 自身。
fn content_location_matches_target(request_url: &str, response: &HttpResponse) -> bool {
    let Some(content_location) = response.header("content-location") else {
        return false;
    };
    let Ok(request_url) = url::Url::parse(request_url) else {
        return false;
    };
    let base = url::Url::parse(&response.url).unwrap_or_else(|_| request_url.clone());
    base.join(content_location).is_ok_and(|target| target == request_url)
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

/// 缓存条目（回放形态）是否 `Vary: *`——RFC 9111 §4.1 复用禁止。
/// only-if-cached / force-cache 的 stale 免验复用臂不得服务此类条目（条目本体
/// 只能以条件再验证路径消费——storable_mode Vary:* 仅再验证注记）。
fn cached_response_is_vary_star(cached: &crate::CachedResponse) -> bool {
    cached
        .headers
        .iter()
        .any(|(name, value)| name.eq_ignore_ascii_case("vary") && value.split(',').any(|field| field.trim() == "*"))
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
    fn data_url_is_decoded_without_network_or_cache() {
        let loader = ResourceLoader::new(Arc::new(Mutex::new(HttpCache::new())), "data-test");
        let response = loader
            .submit(ResourceRequest::get(
                "data:text/javascript,globalThis.value%20%3D%207%3B#ignored",
                FetchPriority::HIGH,
            ))
            .recv()
            .unwrap()
            .unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_type_mime(), Some("text/javascript"));
        assert_eq!(response.body, b"globalThis.value = 7;");
        assert_eq!(response.url, "data:text/javascript,globalThis.value%20%3D%207%3B");
        assert_eq!(loader.stats(), ResourceLoadStats::default());
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
        assert_ne!(
            a.identity_key(),
            a.with_timeout_secs(5).identity_key(),
            "requests with different network timeouts must not share a transaction"
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

    /// 存一条 `no-cache`+ETag 条目（revalidate_only——lookup 恒给 Revalidate，
    /// 等价 stale 待验证形态，无需时间旅行）。
    fn store_revalidate_only_entry(cache: &Arc<Mutex<HttpCache>>, url: &str, body: &[u8]) {
        assert!(cache.lock().unwrap().put(
            url,
            &crate::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Cache-Control".into(), "no-cache".into()),
                    ("ETag".into(), "\"v1\"".into()),
                ],
                body: body.to_vec(),
                url: url.into(),
                redirect_count: 0,
            }
        ));
    }

    /// http-cache goal M1-a：only-if-cached 对 stale（待验证）条目直接服务、
    /// 零网络触达（fetch spec §http-cache-fetch——WPT only-if-cached stale 臂）。
    #[test]
    fn only_if_cached_serves_stale_entry_without_revalidation() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "oic-test");
        let url = "https://cdn.example/oic.png";
        store_revalidate_only_entry(&cache, url, b"stale-body");

        let response = loader
            .submit(ResourceRequest::get(url, FetchPriority::MEDIUM).with_cache_mode(CacheMode::OnlyIfCached))
            .recv()
            .unwrap()
            .unwrap();
        assert_eq!(response.body, b"stale-body");
        assert_eq!(response.status_code, 200);
        let stats = loader.stats();
        assert_eq!(stats.fresh_hits, 1, "缓存直接服务须计命中");
        assert_eq!(stats.only_if_cached_misses, 0);
        assert_eq!(stats.network_requests, 0, "stale 命中不得发起再验证网络请求");
    }

    /// http-cache goal M1-a：force-cache 对 stale 条目直接服务、零网络触达
    /// （fetch spec §http-network-or-cache-fetch——WPT force-cache stale 臂）。
    #[test]
    fn force_cache_serves_stale_entry_without_revalidation() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "fc-test");
        let url = "https://cdn.example/fc.png";
        store_revalidate_only_entry(&cache, url, b"stale-body");

        let response = loader
            .submit(ResourceRequest::get(url, FetchPriority::MEDIUM).with_cache_mode(CacheMode::ForceCache))
            .recv()
            .unwrap()
            .unwrap();
        assert_eq!(response.body, b"stale-body");
        assert_eq!(loader.stats().network_requests, 0);
    }

    /// http-cache goal M1-a：only-if-cached 零条目 → 504 语义错误（既有语义保留）。
    #[test]
    fn only_if_cached_miss_is_504_error() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "oic-miss-test");
        let result = loader
            .submit(
                ResourceRequest::get("https://cdn.example/absent.png", FetchPriority::MEDIUM)
                    .with_cache_mode(CacheMode::OnlyIfCached),
            )
            .recv()
            .unwrap();
        assert!(result.is_err());
        assert_eq!(loader.stats().only_if_cached_misses, 1);
    }

    /// http-cache goal M1-a：lookup_cache 面——reload 不读缓存；
    /// only-if-cached / force-cache 把待验证条目折叠为直接命中。
    #[test]
    fn lookup_cache_matrix_for_reload_and_serve_stale_modes() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "lookup-matrix-test");
        let url = "https://cdn.example/matrix.png";
        store_revalidate_only_entry(&cache, url, b"stale-body");

        let reload =
            loader.lookup_cache(&ResourceRequest::get(url, FetchPriority::MEDIUM).with_cache_mode(CacheMode::Reload));
        assert!(
            matches!(reload, CacheLookup::Miss),
            "reload 不读缓存（lookup_cache 面同 submit 语义）"
        );

        let oic = loader
            .lookup_cache(&ResourceRequest::get(url, FetchPriority::MEDIUM).with_cache_mode(CacheMode::OnlyIfCached));
        match oic {
            CacheLookup::Hit(cached) => assert_eq!(cached.body, b"stale-body"),
            other => panic!("only-if-cached stale 须折叠为 Hit，得 {other:?}"),
        }

        let fc = loader
            .lookup_cache(&ResourceRequest::get(url, FetchPriority::MEDIUM).with_cache_mode(CacheMode::ForceCache));
        assert!(matches!(fc, CacheLookup::Hit(_)));
    }

    /// http-cache goal M1-a：新两态参与在途事务身份——reload/force-cache
    /// 与 default 同 URL 不共享网络事务（identity_key 含 cache 模式）。
    #[test]
    fn new_cache_modes_join_distinct_transactions() {
        let base = ResourceRequest::get("https://example.com/x", FetchPriority::MEDIUM);
        let reload = base.clone().with_cache_mode(CacheMode::Reload);
        let force = base.clone().with_cache_mode(CacheMode::ForceCache);
        assert_ne!(base.identity_key(), reload.identity_key());
        assert_ne!(base.identity_key(), force.identity_key());
        assert_ne!(reload.identity_key(), force.identity_key());
    }

    #[test]
    fn lookup_cache_no_cache_fresh_entry_requires_revalidation() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "site-a");
        let url = "https://cdn.example/no-cache.png";
        assert!(cache.lock().unwrap().put(
            url,
            &crate::HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Cache-Control".into(), "max-age=3600".into()),
                    ("ETag".into(), "image-v1".into()),
                ],
                body: b"cached image".to_vec(),
                url: url.into(),
                redirect_count: 0,
            }
        ));
        let request = ResourceRequest::get(url, FetchPriority::HIGH)
            .with_headers(vec![("Cache-Control".into(), "no-cache".into())]);

        match loader.lookup_cache(&request) {
            CacheLookup::Revalidate {
                cached,
                conditional_headers,
            } => {
                assert_eq!(cached.body, b"cached image");
                assert!(
                    conditional_headers
                        .iter()
                        .any(|(name, value)| { name.eq_ignore_ascii_case("if-none-match") && value == "image-v1" })
                );
            }
            CacheLookup::Hit(_) => panic!("no-cache must not return a fresh cache hit"),
            CacheLookup::Miss => panic!("the existing cache entry should be revalidated"),
        }
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
    fn persistent_partition_cache_survives_loader_recreation_and_stays_isolated() {
        let root = std::env::temp_dir().join(format!(
            "zero-resource-cache-partitions-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let url = "https://cdn.example/partitioned.css";
        let site_a = "https://site-a.example:443";
        let site_b = "https://site-b.example:443";
        {
            let cache = Arc::new(Mutex::new(HttpCache::open_persistent_at(&root)));
            let loader = ResourceLoader::new(cache, "default");
            assert!(
                loader
                    .cache_for_partition(site_a)
                    .lock()
                    .unwrap()
                    .put(url, &fresh_response(url))
            );
        }

        let reopened = ResourceLoader::new(Arc::new(Mutex::new(HttpCache::open_persistent_at(&root))), "default");
        match reopened.lookup_cache(&ResourceRequest::get(url, FetchPriority::MEDIUM).with_partition(site_a)) {
            CacheLookup::Hit(cached) => assert_eq!(cached.body, b"cached"),
            CacheLookup::Revalidate { .. } => panic!("fresh persistent entry should be a hit"),
            CacheLookup::Miss => panic!("same partition should reopen its disk entry"),
        }
        assert!(matches!(
            reopened.lookup_cache(&ResourceRequest::get(url, FetchPriority::MEDIUM).with_partition(site_b)),
            CacheLookup::Miss
        ));
        let _ = std::fs::remove_dir_all(root);
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

    #[test]
    fn fresh_hit_records_anonymous_navigation_event() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "site-a");
        let url = "https://cdn.example/asset.css?private=value";
        assert!(cache.lock().unwrap().put(url, &fresh_response(url)));

        let result = loader
            .submit(
                ResourceRequest::get(url, FetchPriority::CRITICAL)
                    .with_navigation_id(42)
                    .with_destination("style"),
            )
            .recv()
            .expect("fresh cache response");
        assert!(result.is_ok());

        let events = loader.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].navigation_id, Some(42));
        assert_eq!(events[0].destination, "style");
        assert_eq!(events[0].origin, "https://cdn.example");
        assert_eq!(events[0].cache_outcome, CacheOutcome::FreshHit);
        assert_eq!(events[0].bytes, 0);
        assert_eq!(events[0].coalesced_subscriber_count, 1);
    }

    #[test]
    fn http_submission_context_reaches_fresh_cache_event() {
        let cache = Arc::new(Mutex::new(HttpCache::new()));
        let loader = ResourceLoader::new(Arc::clone(&cache), "site-a");
        let url = "https://cdn.example/app.js";
        assert!(cache.lock().unwrap().put(url, &fresh_response(url)));

        let response = loader
            .submit_http_with_context_in_partition(
                HttpRequest::get(url),
                FetchPriority::HIGH,
                "site-a",
                Some(99),
                "script",
            )
            .recv()
            .expect("cache response");
        assert!(response.is_ok());

        let event = loader.events().pop().expect("event");
        assert_eq!(event.navigation_id, Some(99));
        assert_eq!(event.destination, "script");
    }

    #[test]
    fn network_telemetry_is_recorded_without_request_details() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let request = ResourceRequest::get("https://cdn.example/a.js?secret=value", FetchPriority::HIGH)
            .with_navigation_id(7)
            .with_destination("script");
        let telemetry = FetchTelemetry {
            origin: "https://cdn.example".to_string(),
            queue_wait_ms: 3,
            network_ms: 17,
            protocol: "h2".to_string(),
            coalesced_subscriber_count: 2,
        };

        record_event_into(&events, &request, CacheOutcome::Network, Some(&telemetry), 128);

        let event = events.lock().unwrap().pop().expect("event");
        assert_eq!(event.origin, "https://cdn.example");
        assert_eq!(event.queue_wait_ms, 3);
        assert_eq!(event.network_ms, 17);
        assert_eq!(event.protocol, "h2");
        assert_eq!(event.bytes, 128);
        assert_eq!(event.coalesced_subscriber_count, 2);
        assert_eq!(event.cache_outcome, CacheOutcome::Network);
    }

    #[test]
    fn unsafe_invalidation_includes_only_same_origin_related_uris() {
        let request = "https://example.test/articles/1";
        let response = crate::HttpResponse {
            status_code: 200,
            headers: vec![
                ("Location".into(), "/articles/2".into()),
                (
                    "Content-Location".into(),
                    "https://example.test/articles/current".into(),
                ),
                ("Location".into(), "https://other.test/not-ours".into()),
            ],
            body: Vec::new(),
            url: request.into(),
            redirect_count: 0,
        };

        let targets = unsafe_invalidation_targets(request, &response);
        assert!(targets.contains(&request.to_string()));
        assert!(targets.contains(&"https://example.test/articles/2".to_string()));
        assert!(targets.contains(&"https://example.test/articles/current".to_string()));
        assert!(!targets.iter().any(|target| target.contains("other.test")));
    }

    #[test]
    fn request_cache_enum_maps_explicit_five_modes() {
        assert_eq!(CacheMode::from_request_cache(None), None);
        assert_eq!(CacheMode::from_request_cache(Some("")), None);
        assert_eq!(CacheMode::from_request_cache(Some("default")), None);
        assert_eq!(
            CacheMode::from_request_cache(Some("no-store")),
            Some(CacheMode::NoStore)
        );
        assert_eq!(CacheMode::from_request_cache(Some("reload")), Some(CacheMode::Reload));
        assert_eq!(
            CacheMode::from_request_cache(Some("no-cache")),
            Some(CacheMode::NoCache)
        );
        assert_eq!(
            CacheMode::from_request_cache(Some("force-cache")),
            Some(CacheMode::ForceCache)
        );
        assert_eq!(
            CacheMode::from_request_cache(Some("only-if-cached")),
            Some(CacheMode::OnlyIfCached)
        );
        assert_eq!(CacheMode::from_request_cache(Some("bogus")), None);
    }

    #[test]
    fn explicit_cache_mode_wins_over_request_cache_control_heuristic() {
        // fetch 通道 `init.cache`（显式 cache_mode）不被请求头 CC 启发式覆盖：
        // fresh 条目 + `Cache-Control: no-cache` 请求头——Default 模式走启发式
        // 强制再验证（Hit→Revalidate），显式 force-cache 直接命中（Hit 保持）。
        let loader = ResourceLoader::new(Arc::new(Mutex::new(HttpCache::new())), "cache-precedence");
        let response = crate::HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=3600".into()),
                ("ETag".into(), "\"v1\"".into()),
            ],
            body: b"cached".to_vec(),
            url: "https://cache.test/item".into(),
            redirect_count: 0,
        };
        assert!(
            loader
                .cache_for_partition("cache-precedence")
                .lock()
                .expect("cache lock")
                .put_with_headers("https://cache.test/item", &[], &response)
        );
        let request = ResourceRequest::get("https://cache.test/item", FetchPriority::MEDIUM)
            .with_headers(vec![("Cache-Control".into(), "no-cache".into())]);
        // Default + CC no-cache 头 → 启发式推导 NoCache → 强制条件再验证。
        assert!(matches!(loader.lookup_cache(&request), CacheLookup::Revalidate { .. }));
        // 显式 force-cache 为权威 → fresh 条目直接命中（头启发式不覆盖）。
        assert!(matches!(
            loader.lookup_cache(&request.clone().with_cache_mode(CacheMode::ForceCache)),
            CacheLookup::Hit(_)
        ));
        // 显式 reload 不读缓存（lookup 层恒 Miss——网络臂覆盖写）。
        assert!(matches!(
            loader.lookup_cache(&request.clone().with_cache_mode(CacheMode::Reload)),
            CacheLookup::Miss
        ));
    }

    #[test]
    fn storeable_shape_rejects_redirects_and_partial_content() {
        // 语料 status 腿（fresh 4xx/5xx）依赖非 2xx 落盘；仅 3xx/206 形态排除。
        assert!(storeable_response_shape(&fresh_response("https://a.test/x")));
        assert!(storeable_response_shape(&{
            let mut r = fresh_response("https://a.test/x");
            r.status_code = 404;
            r
        }));
        assert!(!storeable_response_shape(&{
            let mut r = fresh_response("https://a.test/x");
            r.status_code = 302;
            r
        }));
        assert!(!storeable_response_shape(&{
            let mut r = fresh_response("https://a.test/x");
            r.status_code = 206;
            r
        }));
    }

    #[test]
    fn content_location_gate_matches_only_effective_request_uri() {
        // 语料 post-patch 腿：`Content-Location: ""` 空引用 join 到最终 URL 自身。
        let patch = |url: &str, location: Option<&str>| {
            let mut r = fresh_response(url);
            r.status_code = 200;
            r.headers = location
                .map(|value| vec![("Content-Location".into(), value.into())])
                .unwrap_or_default();
            r
        };
        assert!(content_location_matches_target(
            "https://a.test/item",
            &patch("https://a.test/item", Some(""))
        ));
        assert!(content_location_matches_target(
            "https://a.test/item",
            &patch("https://a.test/item", Some("/item"))
        ));
        assert!(content_location_matches_target(
            "https://a.test/item",
            &patch("https://a.test/other", Some("../item"))
        ));
        // 目标不一致 / 头缺失 / 请求 URL 不可解析 → 不落盘。
        assert!(!content_location_matches_target(
            "https://a.test/item",
            &patch("https://a.test/item", Some("/elsewhere"))
        ));
        assert!(!content_location_matches_target(
            "https://a.test/item",
            &patch("https://a.test/item", None)
        ));
        assert!(!content_location_matches_target(
            "not a url",
            &patch("https://a.test/item", Some(""))
        ));
    }
}
