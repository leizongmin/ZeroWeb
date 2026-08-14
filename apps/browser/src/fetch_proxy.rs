//! Browser 侧 fetch 代理 — 优先级、HTTP 缓存、安全策略、导航 cancel。

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};

use zero_browser_shell::TabId;
use zero_net::{
    FetchPriority, HttpCache, HttpClient, HttpMethod, HttpRequest, HttpResponse, ResourceLoader, is_file_url,
    read_file_url,
};
use zero_protocol::message::FetchParams;
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
    rx: Receiver<Result<HttpResponse, String>>,
}

const MAX_IPC_STREAM_CHUNK_BYTES: usize = 64 * 1024;

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
                rx,
            });
            return;
        }

        if params.method.eq_ignore_ascii_case("DNS-PREFETCH") {
            self.pending.push(PendingFetch {
                tab_id,
                request_id: params.request_id,
                url: url.clone(),
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
                rx,
            });
            return;
        }

        if stream_image && resource_type == "image" && params.method.eq_ignore_ascii_case("GET") {
            self.start_image_stream(tab_id, params.request_id, url, request_headers, priority);
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
            rx,
        });
    }

    /// 非阻塞轮询 pending fetch。
    pub fn drain(&mut self) -> Vec<CompletedFetch> {
        let mut still_pending = Vec::new();
        let mut completed = Vec::new();
        let drained = std::mem::take(&mut self.pending);
        for pending in drained {
            match pending.rx.try_recv() {
                Ok(Ok(resp)) => {
                    tracing::info!(
                        "fetch IPC done tab {} req_id={} {} status={} bytes={}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url,
                        resp.status_code,
                        resp.body.len()
                    );
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
        while let Ok(item) = self.stream_rx.try_recv() {
            let key = (item.tab_id, item.request_id);
            if self.stream_pending.contains(&key) {
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
    ) {
        self.stream_pending.insert((tab_id, request_id));
        let tx = self.stream_tx.clone();
        let client = HttpClient::new();
        zero_net::client::spawn_network_task(async move {
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
            let result = client
                .send_async_stream(
                    HttpRequest {
                        method: HttpMethod::Get,
                        url: url.clone(),
                        headers,
                        body: None,
                    },
                    move |chunk| {
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
                Ok(head) => CompletedFetch {
                    tab_id,
                    request_id,
                    status: head.status_code,
                    headers: head.headers,
                    body: Vec::new(),
                },
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

fn is_stream_chunk(headers: &[(String, String)]) -> bool {
    headers
        .iter()
        .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-stream-chunk") && value == "1")
}

impl Default for TabFetchProxy {
    fn default() -> Self {
        Self::new()
    }
}

fn immediate_err(msg: String) -> Receiver<Result<HttpResponse, String>> {
    let (tx, rx) = channel();
    let _ = tx.send(Err(msg));
    rx
}

/// 在 browser 进程预解析 DNS；成功以空 204 响应回收 renderer 的内部请求槽。
fn dns_prefetch(origin: String) -> Receiver<Result<HttpResponse, String>> {
    let result = HttpClient::new().dns_prefetch(origin.clone());
    let (tx, rx) = channel();
    zero_net::client::spawn_network_bridge(move || {
        let response = result
            .recv()
            .map_err(|error| error.to_string())
            .and_then(|result| result.map_err(|error| error.to_string()))
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
}
