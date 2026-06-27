//! Browser 侧 fetch 代理 — per-origin 并发上限 + pending 轮询（对齐主流浏览器连接策略）。

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};

use zero_browser_shell::TabId;
use zero_net::{PerOriginFetchScheduler, origin_from_url};
use zero_protocol::message::FetchParams;

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
    origin: String,
    rx: Receiver<Result<(u16, Vec<u8>), String>>,
}

struct DrainedFetch {
    tab_id: TabId,
    request_id: u64,
    status: u16,
    body: Vec<u8>,
    origin: String,
}

/// 多进程 browser 进程的 fetch 调度状态。
pub struct TabFetchProxy {
    scheduler: Arc<Mutex<PerOriginFetchScheduler>>,
    pending: Vec<PendingFetch>,
}

impl TabFetchProxy {
    /// 创建 fetch 代理（默认 per-origin 并发上限见 [`zero_net::max_connections_per_origin`]）。
    pub fn new() -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(PerOriginFetchScheduler::new())),
            pending: Vec::new(),
        }
    }

    /// 受理 renderer 的 `FetchRequest`。
    pub fn enqueue(&mut self, tab_id: TabId, params: &FetchParams) {
        let url = params.url.clone();
        tracing::info!(
            "fetch IPC enqueue tab {} req_id={} {url}",
            tab_id.0,
            params.request_id
        );
        let rx = PerOriginFetchScheduler::submit_shared(&self.scheduler, &url);
        self.pending.push(PendingFetch {
            tab_id,
            request_id: params.request_id,
            url,
            origin: origin_from_url(&params.url),
            rx,
        });
    }

    /// 非阻塞轮询 pending fetch；完成的条目经 `on_complete` 释放 origin 槽位。
    pub fn drain(&mut self) -> Vec<CompletedFetch> {
        let mut still_pending = Vec::new();
        let mut completed = Vec::new();
        for pending in self.pending.drain(..) {
            match pending.rx.try_recv() {
                Ok(Ok((status, body))) => {
                    tracing::info!(
                        "fetch IPC done tab {} req_id={} {} status={status} bytes={}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url,
                        body.len()
                    );
                    completed.push(DrainedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status,
                        body,
                        origin: pending.origin,
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!(
                        "fetch IPC failed tab {} req_id={} {}: {e}",
                        pending.tab_id.0,
                        pending.request_id,
                        pending.url
                    );
                    completed.push(DrainedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 0,
                        body: format!("网络请求失败: {e}").into_bytes(),
                        origin: pending.origin,
                    });
                }
                Err(TryRecvError::Empty) => still_pending.push(pending),
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!(
                        "Browser fetch proxy thread dropped tab {}: {}",
                        pending.tab_id.0,
                        pending.url
                    );
                    completed.push(DrainedFetch {
                        tab_id: pending.tab_id,
                        request_id: pending.request_id,
                        status: 0,
                        body: "网络请求失败: fetch worker exited".as_bytes().to_vec(),
                        origin: pending.origin,
                    });
                }
            }
        }
        self.pending = still_pending;
        completed
            .into_iter()
            .map(|c| CompletedFetch {
                tab_id: c.tab_id,
                request_id: c.request_id,
                status: c.status,
                body: c.body,
            })
            .collect()
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
        assert!(proxy.drain().is_empty(), "HTTP 尚未完成时不应产出 CompletedFetch");
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
        assert_eq!(proxy.pending_count(), 0);
    }

    #[test]
    fn multiple_concurrent_fetches_keep_distinct_request_ids() {
        let mut proxy = TabFetchProxy::new();
        for (id, path) in [(1_u64, "a.css"), (2, "b.css"), (3, "c.png")] {
            proxy.enqueue(
                TabId(1),
                &params(id, &format!("http://127.0.0.1:1/{path}")),
            );
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
