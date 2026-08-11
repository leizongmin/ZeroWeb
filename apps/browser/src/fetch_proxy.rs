//! Browser 侧 fetch 代理 — 优先级、HTTP 缓存、安全策略、导航 cancel。

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};

use zero_browser_shell::TabId;
use zero_net::{
    CacheLookup, FetchPriority, HttpCache, HttpResponse, PerOriginFetchScheduler, is_file_url, read_file_url,
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
    /// 响应体。
    pub body: Vec<u8>,
}

struct PendingFetch {
    tab_id: TabId,
    request_id: u64,
    url: String,
    request_headers: Vec<(String, String)>,
    rx: Receiver<Result<HttpResponse, String>>,
}

/// 多进程 browser 进程的 fetch 调度状态。
pub struct TabFetchProxy {
    scheduler: Arc<Mutex<PerOriginFetchScheduler>>,
    // 普通缓存统一走 zero_net::shared_http_cache()（性能门禁优化 S6，2026-08-08——
    // webview / fetch_proxy / net_pool 共享一份，同一 URL 不再跨路径反复走网络）；
    // 无痕模式保留独立内存缓存（不写磁盘语义）。
    private_cache: Arc<Mutex<HttpCache>>,
    private_tabs: HashSet<TabId>,
    pending: Vec<PendingFetch>,
    tab_epochs: HashMap<TabId, u64>,
    security: HashMap<TabId, SecurityContext>,
}

impl TabFetchProxy {
    /// 创建 fetch 代理。
    pub fn new() -> Self {
        Self {
            scheduler: PerOriginFetchScheduler::new_shared(),
            private_cache: Arc::new(Mutex::new(HttpCache::new())),
            private_tabs: HashSet::new(),
            pending: Vec::new(),
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
    }

    fn cache_for(&self, tab_id: TabId) -> Arc<Mutex<HttpCache>> {
        if self.private_tabs.contains(&tab_id) {
            Arc::clone(&self.private_cache)
        } else {
            zero_net::shared_http_cache()
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
        let cache = self.cache_for(tab_id);
        if let Ok(mut cache) = cache.lock() {
            cache.remove(url);
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
        let request_headers = params.headers.clone();
        let (priority, resource_type) = FetchPriority::from_fetch_headers(&request_headers, &url);

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
                        request_headers,
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
                request_headers,
                rx,
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
                request_headers,
                rx,
            });
            return;
        }

        tracing::info!(
            "fetch IPC enqueue tab {} req_id={} priority={priority:?} type={resource_type} {url}",
            tab_id.0,
            params.request_id
        );

        let cache = self.cache_for(tab_id);
        let lookup = {
            let mut guard = cache.lock().expect("http cache");
            guard.lookup(&url, &request_headers)
        };
        match lookup {
            CacheLookup::Hit(cached) => {
                tracing::info!("fetch cache hit tab {} req_id={} {url}", tab_id.0, params.request_id);
                let (tx, rx) = channel();
                let _ = tx.send(Ok(cached.into_response()));
                self.pending.push(PendingFetch {
                    tab_id,
                    request_id: params.request_id,
                    url,
                    request_headers,
                    rx,
                });
                return;
            }
            CacheLookup::Revalidate {
                cached,
                conditional_headers,
            } => {
                tracing::info!(
                    "fetch cache revalidate tab {} req_id={} {url}",
                    tab_id.0,
                    params.request_id
                );
                let rx = PerOriginFetchScheduler::submit_shared_with_priority_and_headers(
                    &self.scheduler,
                    &url,
                    priority,
                    conditional_headers,
                );
                self.pending.push(PendingFetch {
                    tab_id,
                    request_id: params.request_id,
                    url,
                    request_headers,
                    rx,
                });
                let _ = cached;
                return;
            }
            CacheLookup::Miss => {}
        }

        let rx = PerOriginFetchScheduler::submit_shared_with_priority_and_headers(
            &self.scheduler,
            &url,
            priority,
            Vec::new(),
        );
        self.pending.push(PendingFetch {
            tab_id,
            request_id: params.request_id,
            url,
            request_headers,
            rx,
        });
    }

    /// 非阻塞轮询 pending fetch。
    pub fn drain(&mut self) -> Vec<CompletedFetch> {
        let mut still_pending = Vec::new();
        let mut completed = Vec::new();
        let drained = std::mem::take(&mut self.pending);
        for pending in drained {
            let tab_cache = self.cache_for(pending.tab_id);
            match pending.rx.try_recv() {
                Ok(Ok(resp)) if resp.status_code == 304 => {
                    let body = if let Some(cached) = tab_cache.lock().expect("http cache").not_modified(
                        &pending.url,
                        &pending.request_headers,
                        &resp,
                    ) {
                        cached.body
                    } else {
                        Vec::new()
                    };
                    tracing::info!(
                        "fetch IPC 304 tab {} req_id={} {} bytes={}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url,
                        body.len()
                    );
                    completed.push(CompletedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 200,
                        body,
                    });
                }
                Ok(Ok(resp)) => {
                    if (200..300).contains(&resp.status_code) {
                        let _ = tab_cache.lock().expect("http cache").put_with_headers(
                            &pending.url,
                            &pending.request_headers,
                            &resp,
                        );
                    }
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

fn immediate_err(msg: String) -> Receiver<Result<HttpResponse, String>> {
    let (tx, rx) = channel();
    let _ = tx.send(Err(msg));
    rx
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
}
