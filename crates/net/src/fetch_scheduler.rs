//! Per-origin 并发 fetch 调度 — 对齐浏览器「每 host ~6 连接」策略。

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::client::HttpClient;
use crate::resource_policy::{max_connections_per_origin, origin_from_url};

/// HTTP GET 任务结果：`(status_code, body)` 或网络错误。
pub type FetchJobResult = Result<(u16, Vec<u8>), String>;

struct QueuedJob {
    url: String,
    origin: String,
    reply_tx: Sender<FetchJobResult>,
}

/// 按 origin 限制并发数的 fetch 调度器；复用单个 [`HttpClient`]（keep-alive）。
pub struct PerOriginFetchScheduler {
    max_per_origin: usize,
    client: HttpClient,
    in_flight: HashMap<String, usize>,
    queue: VecDeque<QueuedJob>,
}

impl PerOriginFetchScheduler {
    /// 使用 [`max_connections_per_origin`] 作为并发上限。
    pub fn new() -> Self {
        Self {
            max_per_origin: max_connections_per_origin(),
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    /// 发起 GET，立即返回接收端；超出 per-origin 上限的请求进入队列。
    pub fn submit(&mut self, url: impl Into<String>) -> Receiver<FetchJobResult> {
        let url = url.into();
        let (reply_tx, reply_rx) = channel();
        let job = QueuedJob {
            origin: origin_from_url(&url),
            url,
            reply_tx,
        };
        self.try_start(job, None);
        reply_rx
    }

    /// 经共享 [`Arc<Mutex<Self>>`] 提交；worker 完成后自动释放槽位（供全局 net pool 使用）。
    pub fn submit_shared(sched: &Arc<Mutex<Self>>, url: impl Into<String>) -> Receiver<FetchJobResult> {
        let url = url.into();
        let (reply_tx, reply_rx) = channel();
        let job = QueuedJob {
            origin: origin_from_url(&url),
            url,
            reply_tx,
        };
        let mut s = sched.lock().expect("fetch scheduler lock");
        s.try_start(job, Some(Arc::clone(sched)));
        reply_rx
    }

    /// 某 origin 上一个 in-flight 请求结束；由宿主在收到响应后调用。
    pub fn on_complete(&mut self, origin: &str) {
        if let Some(count) = self.in_flight.get_mut(origin) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.in_flight.remove(origin);
            }
        }
        self.pump_queue();
    }

    fn try_start(&mut self, job: QueuedJob, hook: Option<Arc<Mutex<Self>>>) {
        if self.in_flight.get(&job.origin).copied().unwrap_or(0) >= self.max_per_origin {
            tracing::info!(
                url = %job.url,
                origin = %job.origin,
                queued = self.queue.len() + 1,
                "HTTP fetch queued (per-origin limit)"
            );
            self.queue.push_back(job);
            return;
        }
        self.start(job, hook);
    }

    fn start(&mut self, job: QueuedJob, hook: Option<Arc<Mutex<Self>>>) {
        tracing::info!(url = %job.url, origin = %job.origin, "HTTP fetch start");
        *self.in_flight.entry(job.origin.clone()).or_insert(0) += 1;
        let client = self.client.clone();
        let url = job.url;
        let origin = job.origin;
        let reply_tx = job.reply_tx;
        thread::spawn(move || {
            let result = client
                .get(&url)
                .map(|resp| (resp.status_code, resp.body))
                .map_err(|e| e.to_string());
            match &result {
                Ok((status, body)) => tracing::info!(
                    url = %url,
                    status,
                    bytes = body.len(),
                    "HTTP fetch done"
                ),
                Err(e) => tracing::warn!(url = %url, error = %e, "HTTP fetch failed"),
            }
            let _ = reply_tx.send(result);
            if let Some(sched) = hook {
                if let Ok(mut s) = sched.lock() {
                    s.on_complete(&origin);
                }
            }
        });
    }

    fn pump_queue(&mut self) {
        let mut i = 0;
        while i < self.queue.len() {
            let origin = self.queue[i].origin.clone();
            if self.in_flight.get(&origin).copied().unwrap_or(0) >= self.max_per_origin {
                i += 1;
                continue;
            }
            let job = self.queue.remove(i).expect("queue index");
            self.start(job, None);
        }
    }

    /// 测试用：队列中等待槽位的 job 数。
    #[cfg(test)]
    pub fn queued_count_for_test(&self) -> usize {
        self.queue.len()
    }

    /// 测试用：指定 origin 当前 in-flight 数。
    #[cfg(test)]
    pub fn in_flight_for_test(&self, origin: &str) -> usize {
        self.in_flight.get(origin).copied().unwrap_or(0)
    }

    /// 测试用：覆盖 per-origin 并发上限。
    #[cfg(test)]
    pub fn set_max_per_origin_for_test(&mut self, max: usize) {
        self.max_per_origin = max;
    }
}

impl Default for PerOriginFetchScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::TryRecvError;

    #[test]
    fn queues_beyond_per_origin_limit() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 2,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: VecDeque::new(),
        };
        let _r1 = sched.submit("http://127.0.0.1:1/a");
        let _r2 = sched.submit("http://127.0.0.1:1/b");
        let _r3 = sched.submit("http://127.0.0.1:1/c");
        assert_eq!(sched.in_flight.get("http://127.0.0.1:1").copied(), Some(2));
        assert_eq!(sched.queue.len(), 1);
        sched.on_complete("http://127.0.0.1:1");
        assert_eq!(sched.in_flight.get("http://127.0.0.1:1").copied(), Some(2));
        assert!(sched.queue.is_empty());
    }

    #[test]
    fn different_origins_do_not_share_limit() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: VecDeque::new(),
        };
        let _r1 = sched.submit("http://127.0.0.1:1/a");
        let _r2 = sched.submit("http://127.0.0.2:1/b");
        assert_eq!(sched.in_flight.len(), 2);
        assert!(sched.queue.is_empty());
    }

    #[test]
    fn submit_returns_receiver_before_worker_finishes() {
        let mut sched = PerOriginFetchScheduler::new();
        let rx = sched.submit("http://127.0.0.1:1/unreachable");
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
    }

    #[test]
    fn on_complete_starts_queued_job_for_same_origin() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: VecDeque::new(),
        };
        let r1 = sched.submit("http://127.0.0.1:1/first");
        let r2 = sched.submit("http://127.0.0.1:1/second");
        assert_eq!(sched.queued_count_for_test(), 1);
        sched.on_complete("http://127.0.0.1:1");
        assert_eq!(sched.queued_count_for_test(), 0);
        assert_eq!(sched.in_flight_for_test("http://127.0.0.1:1"), 1);
        let _ = r1;
        let _ = r2;
    }

    #[test]
    fn submit_shared_auto_releases_origin_slot() {
        use std::time::Duration;
        let sched = Arc::new(Mutex::new(PerOriginFetchScheduler {
            max_per_origin: 1,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: VecDeque::new(),
        }));
        let r1 = PerOriginFetchScheduler::submit_shared(&sched, "http://127.0.0.1:1/a");
        let r2 = PerOriginFetchScheduler::submit_shared(&sched, "http://127.0.0.1:1/b");
        assert_eq!(sched.lock().unwrap().queued_count_for_test(), 1);
        let _ = r1.recv_timeout(Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(sched.lock().unwrap().queued_count_for_test(), 0);
        let _ = r2.recv_timeout(Duration::from_secs(5));
    }

    #[test]
    fn invalid_url_uses_fallback_origin_key() {
        let mut sched = PerOriginFetchScheduler::new();
        sched.set_max_per_origin_for_test(1);
        let _r1 = sched.submit("not-a-valid-url");
        let _r2 = sched.submit("not-a-valid-url");
        assert_eq!(sched.queued_count_for_test(), 1);
    }
}
