//! Browser 侧 fetch 代理 — 优先级、HTTP 缓存、安全策略、导航 cancel。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};

use zero_browser_shell::TabId;
use zero_net::{FetchPriority, HttpCache, HttpResponse, PerOriginFetchScheduler};
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
    /// 响应体。
    pub body: Vec<u8>,
}

struct PendingFetch {
    tab_id: TabId,
    request_id: u64,
    url: String,
    rx: Receiver<Result<(u16, Vec<u8>), String>>,
}

/// 多进程 browser 进程的 fetch 调度状态。
pub struct TabFetchProxy {
    scheduler: Arc<Mutex<PerOriginFetchScheduler>>,
    http_cache: Arc<Mutex<HttpCache>>,
    pending: Vec<PendingFetch>,
    tab_epochs: HashMap<TabId, u64>,
    security: HashMap<TabId, SecurityContext>,
}

impl TabFetchProxy {
    /// 创建 fetch 代理。
    pub fn new() -> Self {
        Self {
            scheduler: PerOriginFetchScheduler::new_shared(),
            http_cache: Arc::new(Mutex::new(HttpCache::new())),
            pending: Vec::new(),
            tab_epochs: HashMap::new(),
            security: HashMap::new(),
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

    /// 取消 Tab 当前 pending fetch。
    pub fn cancel_tab(&mut self, tab_id: TabId) {
        let epoch = self.tab_epochs.entry(tab_id).or_insert(0);
        *epoch += 1;
        let before = self.pending.len();
        self.pending.retain(|p| p.tab_id != tab_id);
        let dropped = before.saturating_sub(self.pending.len());
        if dropped > 0 {
            tracing::info!("fetch cancel tab {} dropped {dropped} pending IPC fetches", tab_id.0);
        }
    }

    /// 受理 renderer 的 `FetchRequest`。
    pub fn enqueue(&mut self, tab_id: TabId, params: &FetchParams) {
        self.ensure_tab(tab_id);
        let mut url = params.url.clone();
        let (priority, resource_type) = FetchPriority::from_fetch_headers(&params.headers, &url);

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

        tracing::info!(
            "fetch IPC enqueue tab {} req_id={} priority={priority:?} type={resource_type} {url}",
            tab_id.0,
            params.request_id
        );

        if let Some(cached) = self.http_cache.lock().expect("http cache").get(&url) {
            tracing::info!("fetch cache hit tab {} req_id={} {url}", tab_id.0, params.request_id);
            let (tx, rx) = channel();
            let _ = tx.send(Ok((cached.status_code, cached.body)));
            self.pending.push(PendingFetch {
                tab_id,
                request_id: params.request_id,
                url,
                rx,
            });
            return;
        }

        let rx = PerOriginFetchScheduler::submit_shared_with_priority(&self.scheduler, &url, priority);
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
        let cache = Arc::clone(&self.http_cache);
        for pending in self.pending.drain(..) {
            match pending.rx.try_recv() {
                Ok(Ok((status, body))) => {
                    if (200..300).contains(&status) {
                        store_cache(&cache, &pending.url, status, &body);
                    }
                    tracing::info!(
                        "fetch IPC done tab {} req_id={} {} status={status} bytes={}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url,
                        body.len()
                    );
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status,
                        body,
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
                        body: format!("网络请求失败: {e}").into_bytes(),
                    });
                }
                Err(TryRecvError::Empty) => still_pending.push(pending),
                Err(TryRecvError::Disconnected) => {
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 0,
                        body: "网络请求失败: fetch worker exited".as_bytes().to_vec(),
                    });
                }
            }
        }
        self.pending = still_pending;
        completed
    }

    /// 仍在等待 HTTP 结果的 in-flight 数量。
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for TabFetchProxy {
    fn default() -> Self {
        Self::new()
    }
}

fn immediate_err(msg: String) -> Receiver<Result<(u16, Vec<u8>), String>> {
    let (tx, rx) = channel();
    let _ = tx.send(Err(msg));
    rx
}

fn store_cache(cache: &Arc<Mutex<HttpCache>>, url: &str, status: u16, body: &[u8]) {
    let resp = HttpResponse {
        status_code: status,
        headers: vec![("Cache-Control".into(), "max-age=300".into())],
        body: body.to_vec(),
        url: url.to_string(),
        redirect_count: 0,
    };
    let _ = cache.lock().expect("http cache").put(url, &resp);
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
