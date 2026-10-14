//! Browser 侧 fetch 代理 — 优先级、HTTP 缓存、安全策略、导航 cancel。

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

use zero_browser_shell::TabId;
use zero_net::{
    CacheLookup, FetchPriority, HttpCache, HttpClient, HttpMethod, HttpRequest, HttpResponse, ResourceLoader,
    ResourceRequest, is_file_url, read_file_url,
};
use zero_protocol::message::FetchParams;
use zero_script_sandbox::ServiceWorkerFetchRequest;
use zero_security::{ResourceCheckResult, SecurityContext};

/// 已完成、待回传 renderer 的 fetch 结果。
pub struct CompletedFetch {
    /// 目标 Tab。
    pub tab_id: TabId,
    /// IPC request_id（与 renderer 侧 inflight 表匹配）。
    pub request_id: u64,
    /// HTTP 状态码（失败时为 0）。
    pub status: u16,
    /// 响应头；流式块以内部 `X-Zero-Stream-Chunk` 标记区分于最终响应。
    pub headers: Vec<(String, String)>,
    /// 响应体。
    pub body: Vec<u8>,
}

struct PendingFetch {
    tab_id: TabId,
    request_id: u64,
    url: String,
    resource_type: &'static str,
    rx: Receiver<Result<HttpResponse, String>>,
}

struct ImageStreamCache {
    loader: Arc<ResourceLoader>,
    request: ResourceRequest,
}

const MAX_IPC_STREAM_CHUNK_BYTES: usize = 64 * 1024;
// Bound the extra body copy retained for cache insertion while images stream to the renderer.
const MAX_STREAM_IMAGE_CACHE_BODY_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) enum ServiceWorkerScriptRequestMode {
    SameOrigin,
    Cors,
    NoCors,
}

impl ServiceWorkerScriptRequestMode {
    fn header_value(self) -> &'static str {
        match self {
            Self::SameOrigin => "same-origin",
            Self::Cors => "cors",
            Self::NoCors => "no-cors",
        }
    }
}

/// 多进程 browser 进程的 fetch 调度状态。
pub struct TabFetchProxy {
    /// 普通 profile 与 WebView 共用的资源加载器。
    normal_loader: Arc<ResourceLoader>,
    /// 无痕 profile 的独立内存缓存/调度上下文。
    private_loader: Arc<ResourceLoader>,
    private_tabs: HashSet<TabId>,
    pending: Vec<PendingFetch>,
    stream_tx: Sender<CompletedFetch>,
    stream_rx: Receiver<CompletedFetch>,
    stream_pending: HashSet<(TabId, u64)>,
    stream_limiter: Arc<Semaphore>,
    tab_epochs: HashMap<TabId, u64>,
    security: HashMap<TabId, SecurityContext>,
}

impl TabFetchProxy {
    /// 创建 fetch 代理。
    pub fn new() -> Self {
        let (stream_tx, stream_rx) = channel();
        Self {
            normal_loader: ResourceLoader::shared(),
            private_loader: Arc::new(ResourceLoader::new(Arc::new(Mutex::new(HttpCache::new())), "private")),
            private_tabs: HashSet::new(),
            pending: Vec::new(),
            stream_tx,
            stream_rx,
            stream_pending: HashSet::new(),
            stream_limiter: Arc::new(Semaphore::new(zero_net::max_connections_total())),
            tab_epochs: HashMap::new(),
            security: HashMap::new(),
        }
    }

    /// 标记 Tab 为无痕（仅内存缓存，不写磁盘）。
    pub fn set_tab_private(&mut self, tab_id: TabId, private: bool) {
        if private {
            self.private_tabs.insert(tab_id);
        } else {
            self.private_tabs.remove(&tab_id);
        }
    }

    /// Tab 关闭时清理状态。
    pub fn remove_tab(&mut self, tab_id: TabId) {
        self.private_tabs.remove(&tab_id);
        self.tab_epochs.remove(&tab_id);
        self.security.remove(&tab_id);
        self.stream_pending.retain(|(pending_tab, _)| *pending_tab != tab_id);
    }

    fn loader_for(&self, tab_id: TabId) -> Arc<ResourceLoader> {
        if self.private_tabs.contains(&tab_id) {
            Arc::clone(&self.private_loader)
        } else {
            Arc::clone(&self.normal_loader)
        }
    }

    /// 新 Tab 或首次 fetch 前初始化安全上下文。
    pub fn ensure_tab(&mut self, tab_id: TabId) {
        self.tab_epochs.entry(tab_id).or_insert(0);
        self.security.entry(tab_id).or_default();
    }

    /// 导航开始：更新页面源并 cancel 该 Tab 旧 fetch。
    pub fn on_navigate(&mut self, tab_id: TabId, page_url: &str) {
        self.ensure_tab(tab_id);
        self.cancel_tab(tab_id);
        if let Some(ctx) = self.security.get_mut(&tab_id) {
            ctx.set_page_origin(page_url);
        }
    }

    /// 强制刷新前清除指定 URL 的缓存条目（绕过 HTTP 缓存）。
    /// 仅清除该 tab 对应缓存（普通/无痕）中的主资源条目。
    pub fn invalidate_url(&self, tab_id: TabId, url: &str) {
        self.loader_for(tab_id).invalidate_after_unsafe(url);
    }

    /// 取消 Tab 当前 pending fetch。
    pub fn cancel_tab(&mut self, tab_id: TabId) {
        let epoch = self.tab_epochs.entry(tab_id).or_insert(0);
        *epoch += 1;
        let before = self.pending.len();
        self.pending.retain(|p| p.tab_id != tab_id);
        self.stream_pending.retain(|(pending_tab, _)| *pending_tab != tab_id);
        let dropped = before.saturating_sub(self.pending.len());
        if dropped > 0 {
            tracing::info!("fetch cancel tab {} dropped {dropped} pending IPC fetches", tab_id.0);
        }
    }

    /// 受理 renderer 的 `FetchRequest`。
    pub fn enqueue(&mut self, tab_id: TabId, params: &FetchParams) {
        self.ensure_tab(tab_id);
        let mut url = params.url.clone();
        let request_headers = params.headers.clone();
        let (priority, resource_type) = FetchPriority::from_fetch_headers(&request_headers, &url);
        let stream_image = request_headers
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-stream-image") && value == "1");

        if let Some(ctx) = self.security.get_mut(&tab_id) {
            match ctx.check_resource_url(&url, resource_type) {
                ResourceCheckResult::Allow => {}
                ResourceCheckResult::Upgraded(https_url) => {
                    tracing::info!("fetch upgraded tab {}: {url} -> {https_url}", tab_id.0);
                    url = https_url;
                }
                ResourceCheckResult::Blocked(reason) => {
                    tracing::warn!(
                        "fetch blocked tab {} req_id={} {url}: {reason}",
                        tab_id.0,
                        params.request_id
                    );
                    self.pending.push(PendingFetch {
                        tab_id,
                        request_id: params.request_id,
                        url,
                        resource_type,
                        rx: immediate_err(format!("资源被安全策略阻止: {reason}")),
                    });
                    return;
                }
            }
        }

        if is_file_url(&url) {
            let (tx, rx) = channel();
            let _ = tx.send(read_file_url(&url).map_err(|error| error.to_string()));
            self.pending.push(PendingFetch {
                tab_id,
                request_id: params.request_id,
                url,
                resource_type,
                rx,
            });
            return;
        }

        if params.method.eq_ignore_ascii_case("DNS-PREFETCH") {
            self.pending.push(PendingFetch {
                tab_id,
                request_id: params.request_id,
                url: url.clone(),
                resource_type,
                rx: dns_prefetch(url),
            });
            return;
        }

        // 性能门禁优化 S6（2026-08-08）：失败 URL 负缓存——renderer 每次 publish 会
        // 重请求「未缓存/解码失败」的图片（paint_export fetch_image_payloads_with_cache），
        // 冷却期内直接拒绝，不再每 publish 重试网络。
        if zero_net::shared_negative_cache()
            .lock()
            .unwrap()
            .is_recently_failed(&url)
        {
            tracing::debug!(
                "fetch negative-cache reject tab {} req_id={} {url}",
                tab_id.0,
                params.request_id
            );
            let (tx, rx) = channel();
            let _ = tx.send(Err("negative cache (recent failure)".to_string()));
            self.pending.push(PendingFetch {
                tab_id,
                request_id: params.request_id,
                url,
                resource_type,
                rx,
            });
            return;
        }

        if stream_image && resource_type == "image" && params.method.eq_ignore_ascii_case("GET") {
            let partition = self
                .security
                .get(&tab_id)
                .and_then(|context| context.page_origin())
                .map(|origin| format!("{}://{}:{}", origin.scheme, origin.host, origin.port))
                .unwrap_or_else(|| "default".to_string());
            let mut cache_request = ResourceRequest::get(url.clone(), priority)
                .with_headers(request_headers.clone())
                .with_partition(partition)
                .with_destination(resource_type);
            if let Some(navigation_id) = self.tab_epochs.get(&tab_id).copied() {
                cache_request = cache_request.with_navigation_id(navigation_id);
            }
            let loader = self.loader_for(tab_id);
            match loader.lookup_cache(&cache_request) {
                CacheLookup::Hit(cached) => {
                    self.pending.push(PendingFetch {
                        tab_id,
                        request_id: params.request_id,
                        url,
                        resource_type,
                        rx: immediate_response(cached.into_response()),
                    });
                    return;
                }
                CacheLookup::Revalidate { .. } => {
                    let rx = loader.submit(cache_request);
                    self.pending.push(PendingFetch {
                        tab_id,
                        request_id: params.request_id,
                        url,
                        resource_type,
                        rx,
                    });
                    return;
                }
                CacheLookup::Miss if request_requires_only_if_cached(&request_headers) => {
                    let rx = loader.submit(cache_request);
                    self.pending.push(PendingFetch {
                        tab_id,
                        request_id: params.request_id,
                        url,
                        resource_type,
                        rx,
                    });
                    return;
                }
                CacheLookup::Miss => {
                    self.start_image_stream(
                        tab_id,
                        params.request_id,
                        url,
                        request_headers,
                        priority,
                        ImageStreamCache {
                            loader,
                            request: cache_request,
                        },
                    );
                }
            }
            return;
        }

        tracing::info!(
            "fetch IPC enqueue tab {} req_id={} priority={priority:?} type={resource_type} {url}",
            tab_id.0,
            params.request_id
        );

        let method = match params.method.to_ascii_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => {
                self.pending.push(PendingFetch {
                    tab_id,
                    request_id: params.request_id,
                    url,
                    resource_type,
                    rx: immediate_err(format!("unsupported HTTP method: {}", params.method)),
                });
                return;
            }
        };
        let partition = self
            .security
            .get(&tab_id)
            .and_then(|context| context.page_origin())
            .map(|origin| format!("{}://{}:{}", origin.scheme, origin.host, origin.port))
            .unwrap_or_else(|| "default".to_string());
        let navigation_id = self.tab_epochs.get(&tab_id).copied();
        let rx = self.loader_for(tab_id).submit_http_with_context_in_partition(
            HttpRequest {
                method,
                url: url.clone(),
                headers: request_headers,
                body: params.body.clone(),
            },
            priority,
            partition,
            navigation_id,
            resource_type,
        );
        self.pending.push(PendingFetch {
            tab_id,
            request_id: params.request_id,
            url,
            resource_type,
            rx,
        });
    }

    /// Submit a Service Worker main script through the browser-owned loader.
    ///
    /// The caller owns response correlation, so this returns a dedicated
    /// receiver instead of entering the renderer FetchRequest queue.
    pub fn fetch_service_worker_script(
        &mut self,
        tab_id: TabId,
        script_url: &str,
        bypass_cache: bool,
        request_mode: ServiceWorkerScriptRequestMode,
        is_main_script: bool,
    ) -> Receiver<Result<HttpResponse, String>> {
        self.ensure_tab(tab_id);
        let url = match self.security.get_mut(&tab_id) {
            Some(context) => match context.check_resource_url(script_url, "script") {
                ResourceCheckResult::Allow => script_url.to_string(),
                ResourceCheckResult::Upgraded(url) => url,
                ResourceCheckResult::Blocked(reason) => {
                    return immediate_err(format!("resource blocked by security policy: {reason}"));
                }
            },
            None => script_url.to_string(),
        };
        let partition = self
            .security
            .get(&tab_id)
            .and_then(|context| context.page_origin())
            .map(|origin| format!("{}://{}:{}", origin.scheme, origin.host, origin.port))
            .unwrap_or_else(|| "default".to_string());
        let mut headers = vec![("Sec-Fetch-Mode".into(), request_mode.header_value().into())];
        if is_main_script {
            headers.push(("Service-Worker".into(), "script".into()));
        }
        if bypass_cache {
            headers.push(("Cache-Control".into(), "no-cache".into()));
        }
        self.loader_for(tab_id).submit_http_with_context_in_partition(
            HttpRequest {
                method: HttpMethod::Get,
                url,
                headers,
                body: None,
            },
            FetchPriority::HIGH,
            partition,
            self.tab_epochs.get(&tab_id).copied(),
            "serviceworker",
        )
    }

    /// Submit an ordinary worker-global `fetch()` through the browser-owned loader.
    ///
    /// This is intentionally separate from Service Worker script fetches: ordinary fetches are
    /// request/response traffic and must not inherit script MIME or redirect checks.
    pub fn fetch_service_worker_request(
        &mut self,
        tab_id: TabId,
        request: &ServiceWorkerFetchRequest,
    ) -> Receiver<Result<HttpResponse, String>> {
        self.ensure_tab(tab_id);
        let mut url = request.url.clone();
        if let Some(context) = self.security.get_mut(&tab_id) {
            match context.check_resource_url(&url, "connect") {
                ResourceCheckResult::Allow => {}
                ResourceCheckResult::Upgraded(upgraded) => url = upgraded,
                ResourceCheckResult::Blocked(reason) => {
                    return immediate_err(format!("resource blocked by security policy: {reason}"));
                }
            }
        }
        if is_file_url(&url) {
            let (tx, rx) = channel();
            let _ = tx.send(read_file_url(&url).map_err(|error| error.to_string()));
            return rx;
        }
        let method = match request.method.to_ascii_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => return immediate_err(format!("unsupported HTTP method: {}", request.method)),
        };
        let priority = FetchPriority::from_fetch_headers(&request.headers, &url).0;
        let partition = self
            .security
            .get(&tab_id)
            .and_then(|context| context.page_origin())
            .map(|origin| format!("{}://{}:{}", origin.scheme, origin.host, origin.port))
            .unwrap_or_else(|| "default".to_string());
        self.loader_for(tab_id).submit_http_with_context_in_partition(
            HttpRequest {
                method,
                url,
                headers: request.headers.clone(),
                body: request.body.as_ref().map(|body| body.as_bytes().to_vec()),
            },
            priority,
            partition,
            self.tab_epochs.get(&tab_id).copied(),
            "connect",
        )
    }

    /// 非阻塞轮询 pending fetch。
    pub fn drain(&mut self) -> Vec<CompletedFetch> {
        let mut still_pending = Vec::new();
        let mut completed = Vec::new();
        let drained = std::mem::take(&mut self.pending);
        for pending in drained {
            match pending.rx.try_recv() {
                Ok(Ok(mut resp)) => {
                    tracing::info!(
                        "fetch IPC done tab {} req_id={} {} status={} bytes={}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url,
                        resp.status_code,
                        resp.body.len()
                    );
                    // 信任边界：响应头来自外部服务器，内部命名空间头（x-zero-*）先剥离
                    // 再推入浏览器实例，保证下游按「唯一浏览器推入实例」做路由决策。
                    strip_spoofed_internal_headers(&mut resp.headers);
                    resp.headers.push(("X-Zero-Final-URL".to_string(), resp.url.clone()));
                    resp.headers
                        .push(("X-Zero-Resource-Type".to_string(), pending.resource_type.to_string()));
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: resp.status_code,
                        headers: resp.headers,
                        body: resp.body,
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!(
                        "fetch IPC failed tab {} req_id={} {}: {e}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url
                    );
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 0,
                        headers: Vec::new(),
                        body: format!("网络请求失败: {e}").into_bytes(),
                    });
                }
                Err(TryRecvError::Empty) => still_pending.push(pending),
                Err(TryRecvError::Disconnected) => {
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 0,
                        headers: Vec::new(),
                        body: "网络请求失败: fetch worker exited".as_bytes().to_vec(),
                    });
                }
            }
        }
        self.pending = still_pending;
        while let Ok(mut item) = self.stream_rx.try_recv() {
            let key = (item.tab_id, item.request_id);
            if self.stream_pending.contains(&key) {
                // 流式完成块（含最终头）同样先剥离可伪造头；分块头由本地构造、仅含
                // X-Zero-Stream-Chunk，剥离对其无影响（该路径不推入内部资源类型头）。
                strip_spoofed_internal_headers(&mut item.headers);
                if !is_stream_chunk(&item.headers) {
                    self.stream_pending.remove(&key);
                }
                completed.push(item);
            }
        }
        completed
    }

    /// 仍在等待 HTTP 结果的 in-flight 数量。
    pub fn pending_count(&self) -> usize {
        self.pending.len() + self.stream_pending.len()
    }

    fn start_image_stream(
        &mut self,
        tab_id: TabId,
        request_id: u64,
        url: String,
        request_headers: Vec<(String, String)>,
        priority: FetchPriority,
        cache: ImageStreamCache,
    ) {
        let max_pending = zero_net::max_connections_total().saturating_mul(16).clamp(64, 1024);
        if self.stream_pending.len() >= max_pending {
            self.pending.push(PendingFetch {
                tab_id,
                request_id,
                url,
                resource_type: "image",
                rx: immediate_err("image stream queue is full".to_string()),
            });
            return;
        }
        self.stream_pending.insert((tab_id, request_id));
        let tx = self.stream_tx.clone();
        let limiter = Arc::clone(&self.stream_limiter);
        zero_net::client::spawn_network_task(async move {
            let Ok(_permit) = limiter.acquire_owned().await else {
                let _ = tx.send(CompletedFetch {
                    tab_id,
                    request_id,
                    status: 0,
                    headers: Vec::new(),
                    body: b"network request failed: image stream scheduler closed".to_vec(),
                });
                return;
            };
            let mut headers: Vec<_> = request_headers
                .into_iter()
                .filter(|(name, _)| !name.to_ascii_lowercase().starts_with("x-zero-"))
                .collect();
            if zero_net::connect::http2_enabled()
                && !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("priority"))
            {
                headers.push(("Priority".into(), priority.rfc9218_header_value().into()));
            }
            let chunk_tx = tx.clone();
            let mut cache_body = Vec::new();
            let mut cache_body_within_limit = true;
            let result = HttpClient::send_async_stream_with_timeout(
                30,
                HttpRequest {
                    method: HttpMethod::Get,
                    url: url.clone(),
                    headers,
                    body: None,
                },
                |chunk| {
                    if cache_body_within_limit {
                        if cache_body.len().saturating_add(chunk.len()) <= MAX_STREAM_IMAGE_CACHE_BODY_BYTES {
                            cache_body.extend_from_slice(chunk);
                        } else {
                            cache_body.clear();
                            cache_body_within_limit = false;
                        }
                    }
                    for part in chunk.chunks(MAX_IPC_STREAM_CHUNK_BYTES) {
                        let _ = chunk_tx.send(CompletedFetch {
                            tab_id,
                            request_id,
                            status: 200,
                            headers: vec![("X-Zero-Stream-Chunk".into(), "1".into())],
                            body: part.to_vec(),
                        });
                    }
                },
            )
            .await;
            let completed = match result {
                Ok(head) => {
                    if cache_body_within_limit && (200..300).contains(&head.status_code) {
                        let response = HttpResponse {
                            status_code: head.status_code,
                            headers: head.headers.clone(),
                            body: std::mem::take(&mut cache_body),
                            url: head.url.clone(),
                            redirect_count: head.redirect_count,
                        };
                        let _ = cache.loader.store_streamed_response(&cache.request, &response);
                    }
                    CompletedFetch {
                        tab_id,
                        request_id,
                        status: head.status_code,
                        headers: head.headers,
                        body: Vec::new(),
                    }
                }
                Err(error) => CompletedFetch {
                    tab_id,
                    request_id,
                    status: 0,
                    headers: Vec::new(),
                    body: format!("网络请求失败: {error}").into_bytes(),
                },
            };
            let _ = tx.send(completed);
        });
    }
}

fn request_requires_only_if_cached(headers: &[(String, String)]) -> bool {
    headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("cache-control"))
        .flat_map(|(_, value)| value.split(','))
        .any(|directive| directive.trim().eq_ignore_ascii_case("only-if-cached"))
}

fn is_stream_chunk(headers: &[(String, String)]) -> bool {
    headers
        .iter()
        .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-stream-chunk") && value == "1")
}

/// 剥离外部服务器可伪造的内部命名空间响应头（信任边界：响应头来自外部服务器）。
/// `X-Zero-Resource-Type` / `X-Zero-Final-URL` 只承载浏览器自身写入的路由元数据，
/// 下游 `any()` / `find()` 判定必须只见到浏览器推入的实例；ServiceWorker 响应侧
/// 有同样的剥离再重推处理（service_worker_owner 完成页 fetch 时）。
fn strip_spoofed_internal_headers(headers: &mut Vec<(String, String)>) {
    headers.retain(|(name, _)| {
        !name.eq_ignore_ascii_case("x-zero-resource-type") && !name.eq_ignore_ascii_case("x-zero-final-url")
    });
}

impl Default for TabFetchProxy {
    fn default() -> Self {
        Self::new()
    }
}

fn immediate_response(response: HttpResponse) -> Receiver<Result<HttpResponse, String>> {
    let (tx, rx) = channel();
    let _ = tx.send(Ok(response));
    rx
}

fn immediate_err(msg: String) -> Receiver<Result<HttpResponse, String>> {
    let (tx, rx) = channel();
    let _ = tx.send(Err(msg));
    rx
}

/// 在 browser 进程预解析 DNS；成功以空 204 响应回收 renderer 的内部请求槽。
fn dns_prefetch(origin: String) -> Receiver<Result<HttpResponse, String>> {
    let (tx, rx) = channel();
    zero_net::client::spawn_network_task(async move {
        let response = HttpClient::dns_prefetch_origin_async(&origin)
            .await
            .map_err(|error| error.to_string())
            .map(|()| HttpResponse {
                status_code: 204,
                headers: Vec::new(),
                body: Vec::new(),
                url: origin,
                redirect_count: 0,
            });
        let _ = tx.send(response);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;
    use zero_protocol::message::FetchParams;

    fn params(request_id: u64, url: &str) -> FetchParams {
        FetchParams {
            request_id,
            url: url.to_string(),
            method: "GET".into(),
            headers: Vec::new(),
            body: None,
        }
    }

    #[test]
    fn enqueue_returns_immediately_with_pending_entry() {
        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(1), &params(10, "http://127.0.0.1:1/res"));
        assert_eq!(proxy.pending_count(), 1);
        assert!(proxy.drain().is_empty());
    }

    #[test]
    fn cancel_tab_drops_pending_without_complete() {
        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(1), &params(1, "http://127.0.0.1:1/slow"));
        proxy.cancel_tab(TabId(1));
        assert_eq!(proxy.pending_count(), 0);
        assert!(proxy.drain().is_empty());
    }

    #[test]
    fn drain_delivers_completed_fetch_with_matching_request_id() {
        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(2), &params(42, "http://127.0.0.1:1/asset.css"));
        let mut done = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while done.is_empty() && std::time::Instant::now() < deadline {
            done.extend(proxy.drain());
            if done.is_empty() {
                std::thread::sleep(Duration::from_millis(30));
            }
        }
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].request_id, 42);
        assert_eq!(done[0].tab_id, TabId(2));
    }

    #[test]
    fn multiple_concurrent_fetches_keep_distinct_request_ids() {
        let mut proxy = TabFetchProxy::new();
        for (id, path) in [(1_u64, "a.css"), (2, "b.css"), (3, "c.png")] {
            proxy.enqueue(TabId(1), &params(id, &format!("http://127.0.0.1:1/{path}")));
        }
        assert_eq!(proxy.pending_count(), 3);
        let mut seen = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while seen.len() < 3 && std::time::Instant::now() < deadline {
            for item in proxy.drain() {
                seen.push(item.request_id);
            }
            if seen.len() < 3 {
                std::thread::sleep(Duration::from_millis(30));
            }
        }
        seen.sort_unstable();
        assert_eq!(seen, vec![1, 2, 3]);
    }

    #[test]
    fn file_url_fetch_reads_local_file_without_http_scheduler() {
        let file = std::env::temp_dir().join("zero_browser_fetch_proxy_file_url_test.html");
        std::fs::write(&file, b"<html><body>local file</body></html>").unwrap();
        let url = url::Url::from_file_path(&file).unwrap().to_string();

        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(1), &params(7, &url));
        let completed = proxy.drain();

        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].request_id, 7);
        assert_eq!(completed[0].status, 200);
        assert_eq!(completed[0].body, b"<html><body>local file</body></html>");
        assert!(
            completed[0]
                .headers
                .iter()
                .any(|(name, value)| name == "X-Zero-Final-URL" && value == &url)
        );
        assert!(
            completed[0]
                .headers
                .iter()
                .any(|(name, value)| name == "X-Zero-Resource-Type" && value == "document")
        );

        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn dns_prefetch_completes_without_an_http_connection() {
        let mut request = params(8, "http://localhost:9");
        request.method = "DNS-PREFETCH".into();
        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(1), &request);

        let mut done = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while done.is_empty() && std::time::Instant::now() < deadline {
            done.extend(proxy.drain());
            if done.is_empty() {
                std::thread::sleep(Duration::from_millis(30));
            }
        }
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].request_id, 8);
        assert_eq!(done[0].status, 204);
        assert!(done[0].body.is_empty());
    }

    #[test]
    fn service_worker_script_uses_browser_owned_loader() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/sw.js", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let size = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..size]);
            assert!(request.starts_with("GET /sw.js "));
            assert!(request.to_ascii_lowercase().contains("\r\nservice-worker: script\r\n"));
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("\r\nsec-fetch-mode: same-origin\r\n")
            );
            assert!(request.to_ascii_lowercase().contains("\r\ncache-control: no-cache\r\n"));
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: 7\r\n\r\nvoid 0;",
                )
                .unwrap();
        });

        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(4), &url);
        let receiver =
            proxy.fetch_service_worker_script(TabId(4), &url, true, ServiceWorkerScriptRequestMode::SameOrigin, true);
        let response = receiver.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        server.join().unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.url, url);
        assert_eq!(response.body, b"void 0;");
    }

    #[test]
    fn service_worker_script_cache_mode_bypasses_or_reuses_fresh_entry() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/sw-cache.js", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for (index, body) in ["version-1", "version-2"].into_iter().enumerate() {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 1024];
                let size = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..size]).to_ascii_lowercase();
                assert!(request.starts_with("get /sw-cache.js "));
                assert!(request.contains("\r\nservice-worker: script\r\n"));
                assert!(request.contains("\r\nsec-fetch-mode: same-origin\r\n"));
                assert!(request.contains("\r\ncache-control: no-cache\r\n"));
                if index == 1 {
                    assert!(request.contains("\r\nif-none-match: etag\r\n"));
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nCache-Control: max-age=3600\r\nETag: etag\r\n\
                     Content-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });

        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(14), &url);
        let first = proxy
            .fetch_service_worker_script(TabId(14), &url, true, ServiceWorkerScriptRequestMode::SameOrigin, true)
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap();
        let cached = proxy
            .fetch_service_worker_script(TabId(14), &url, false, ServiceWorkerScriptRequestMode::SameOrigin, true)
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap();
        let refreshed = proxy
            .fetch_service_worker_script(TabId(14), &url, true, ServiceWorkerScriptRequestMode::SameOrigin, true)
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap();
        server.join().unwrap();

        assert_eq!(first.body, b"version-1");
        assert_eq!(cached.body, b"version-1");
        assert_eq!(refreshed.body, b"version-2");
    }

    #[test]
    fn image_stream_waits_for_global_connection_permit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/queued.png", listener.local_addr().unwrap());
        let limiter = Arc::new(Semaphore::new(1));
        let permit = Arc::clone(&limiter).try_acquire_owned().unwrap();
        let mut proxy = TabFetchProxy::new();
        proxy.stream_limiter = limiter;
        let mut request = params(16, &url);
        request.headers = vec![
            ("X-Zero-Resource-Type".into(), "image".into()),
            ("X-Zero-Stream-Image".into(), "1".into()),
        ];

        proxy.enqueue(TabId(3), &request);
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "stream must not open a socket before acquiring a permit"
        );

        drop(permit);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(std::time::Instant::now() < deadline, "stream did not acquire permit");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        let mut incoming = [0_u8; 1024];
        let _ = stream.read(&mut incoming).unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\nx")
            .unwrap();

        let mut completed = Vec::new();
        while std::time::Instant::now() < deadline {
            completed.extend(proxy.drain());
            if completed.iter().any(|item| !is_stream_chunk(&item.headers)) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(completed.iter().any(|item| item.status == 200));
    }

    #[test]
    fn image_stream_queue_rejects_requests_above_bound() {
        let mut proxy = TabFetchProxy::new();
        let max_pending = zero_net::max_connections_total().saturating_mul(16).clamp(64, 1024);
        for request_id in 0..max_pending as u64 {
            proxy.stream_pending.insert((TabId(1), request_id));
        }

        let loader = proxy.loader_for(TabId(1));
        let cache_request = ResourceRequest::get("http://127.0.0.1/overflow.png", FetchPriority::MEDIUM);
        proxy.start_image_stream(
            TabId(1),
            max_pending as u64,
            "http://127.0.0.1/overflow.png".to_string(),
            Vec::new(),
            FetchPriority::MEDIUM,
            ImageStreamCache {
                loader,
                request: cache_request,
            },
        );

        let completed = proxy.drain();
        assert_eq!(proxy.stream_pending.len(), max_pending);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].status, 0);
        assert!(String::from_utf8_lossy(&completed[0].body).contains("queue is full"));
    }

    #[test]
    fn image_stream_forwards_chunks_then_a_final_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/image.png", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nfirst")
                .unwrap();
            stream.write_all(b"second").unwrap();
        });

        let mut request = params(17, &url);
        request.headers = vec![
            ("X-Zero-Resource-Type".into(), "image".into()),
            ("X-Zero-Stream-Image".into(), "1".into()),
        ];
        let mut proxy = TabFetchProxy::new();
        proxy.enqueue(TabId(3), &request);
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut completed = Vec::new();
        while std::time::Instant::now() < deadline {
            completed.extend(proxy.drain());
            if completed.iter().any(|item| !is_stream_chunk(&item.headers)) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        server.join().unwrap();

        assert!(completed.iter().any(|item| is_stream_chunk(&item.headers)));
        let body: Vec<u8> = completed
            .iter()
            .filter(|item| is_stream_chunk(&item.headers))
            .flat_map(|item| item.body.iter().copied())
            .collect();
        assert_eq!(body, b"firstsecond");
        assert!(
            completed
                .iter()
                .any(|item| !is_stream_chunk(&item.headers) && item.status == 200)
        );
    }

    #[test]
    fn repeated_streamed_image_reuses_fresh_http_cache_entry() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/cached.png", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_millis(750);
            let mut request_count = 0;
            while std::time::Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        request_count += 1;
                        let mut request = [0_u8; 1024];
                        let _ = stream.read(&mut request).unwrap();
                        stream
                            .write_all(
                                b"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nCache-Control: max-age=3600\r\nETag: \"image-v1\"\r\nConnection: close\r\nContent-Length: 7\r\n\r\nimgdata",
                            )
                            .unwrap();
                        if request_count > 1 {
                            break;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept failed: {error}"),
                }
            }
            request_count
        });

        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(23), &url);
        for request_id in [101, 102] {
            let mut request = params(request_id, &url);
            request.headers = vec![
                ("X-Zero-Resource-Type".into(), "image".into()),
                ("X-Zero-Stream-Image".into(), "1".into()),
            ];
            proxy.enqueue(TabId(23), &request);

            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut completed = Vec::new();
            while std::time::Instant::now() < deadline {
                completed.extend(proxy.drain());
                if completed.iter().any(|item| !is_stream_chunk(&item.headers)) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let final_response = completed
                .iter()
                .find(|item| !is_stream_chunk(&item.headers))
                .expect("image response completes");
            assert_eq!(final_response.request_id, request_id);
            assert_eq!(final_response.status, 200);
            let body: Vec<u8> = completed
                .iter()
                .filter(|item| is_stream_chunk(&item.headers))
                .flat_map(|item| item.body.iter().copied())
                .chain(final_response.body.iter().copied())
                .collect();
            assert_eq!(body, b"imgdata");
        }

        assert_eq!(
            server.join().unwrap(),
            1,
            "fresh image should be reused without a second request"
        );
    }

    #[test]
    fn expired_streamed_image_revalidates_and_uses_cached_body_on_304() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/revalidated.png", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for index in 0..2 {
                let deadline = std::time::Instant::now() + Duration::from_secs(3);
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(accepted) => break accepted,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(std::time::Instant::now() < deadline, "request {index} was not received");
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("accept failed: {error}"),
                    }
                };
                let mut request = [0_u8; 2048];
                let size = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..size]).to_ascii_lowercase();
                if index == 0 {
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nCache-Control: no-cache\r\nETag: \"image-v1\"\r\nConnection: close\r\nContent-Length: 7\r\n\r\nimgdata",
                        )
                        .unwrap();
                } else {
                    assert!(
                        request.contains("\r\nif-none-match: \"image-v1\"\r\n"),
                        "revalidation request was missing the validator: {request}"
                    );
                    stream
                        .write_all(
                            b"HTTP/1.1 304 Not Modified\r\nCache-Control: max-age=3600\r\nETag: \"image-v1\"\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
                        )
                        .unwrap();
                }
            }
        });

        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(24), &url);
        let mut final_responses = Vec::new();
        for request_id in [201, 202] {
            let mut request = params(request_id, &url);
            request.headers = vec![
                ("X-Zero-Resource-Type".into(), "image".into()),
                ("X-Zero-Stream-Image".into(), "1".into()),
            ];
            proxy.enqueue(TabId(24), &request);

            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut completed = Vec::new();
            while std::time::Instant::now() < deadline {
                completed.extend(proxy.drain());
                if completed.iter().any(|item| !is_stream_chunk(&item.headers)) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            final_responses.push(
                completed
                    .into_iter()
                    .find(|item| !is_stream_chunk(&item.headers))
                    .expect("image response completes"),
            );
        }
        server.join().unwrap();

        assert_eq!(final_responses[0].status, 200);
        assert_eq!(final_responses[1].request_id, 202);
        assert_eq!(final_responses[1].status, 200);
        assert_eq!(final_responses[1].body, b"imgdata");
        assert!(
            final_responses[1]
                .headers
                .iter()
                .any(|(name, value)| { name.eq_ignore_ascii_case("cache-control") && value == "max-age=3600" })
        );
    }

    #[test]
    fn no_store_streamed_image_is_requested_again() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/no-store.png", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for index in 0..2 {
                let deadline = std::time::Instant::now() + Duration::from_secs(3);
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(accepted) => break accepted,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(std::time::Instant::now() < deadline, "request {index} was not received");
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("accept failed: {error}"),
                    }
                };
                let mut request = [0_u8; 1024];
                let _ = stream.read(&mut request).unwrap();
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nCache-Control: max-age=3600\r\nConnection: close\r\nContent-Length: 7\r\n\r\nimgdata",
                    )
                    .unwrap();
            }
        });

        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(25), &url);
        for request_id in [301, 302] {
            let mut request = params(request_id, &url);
            request.headers = vec![
                ("X-Zero-Resource-Type".into(), "image".into()),
                ("X-Zero-Stream-Image".into(), "1".into()),
                ("Cache-Control".into(), "no-store".into()),
            ];
            proxy.enqueue(TabId(25), &request);

            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut completed = Vec::new();
            while std::time::Instant::now() < deadline {
                completed.extend(proxy.drain());
                if completed.iter().any(|item| !is_stream_chunk(&item.headers)) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(completed.iter().any(|item| {
                !is_stream_chunk(&item.headers) && item.request_id == request_id && item.status == 200
            }));
        }
        server.join().unwrap();
    }

    #[test]
    fn only_if_cached_image_miss_does_not_start_network_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/offline.png", listener.local_addr().unwrap());
        let mut request = params(401, &url);
        request.headers = vec![
            ("X-Zero-Resource-Type".into(), "image".into()),
            ("X-Zero-Stream-Image".into(), "1".into()),
            ("Cache-Control".into(), "only-if-cached".into()),
        ];
        let mut proxy = TabFetchProxy::new();
        proxy.on_navigate(TabId(26), &url);
        proxy.enqueue(TabId(26), &request);

        let completed = proxy.drain();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].status, 0);
        assert!(String::from_utf8_lossy(&completed[0].body).contains("504"));
        assert!(matches!(
            listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
    }
}
