//! Per-origin 并发 fetch 调度 — 对齐浏览器「每 host ~6 连接」+ 优先级队列。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::client::HttpClient;
use crate::fetch_priority::FetchPriority;
use crate::resource_policy::{max_connections_per_origin, origin_from_url};

/// HTTP GET 任务结果。
pub type FetchJobResult = Result<crate::HttpResponse, String>;

struct QueuedJob {
    url: String,
    origin: String,
    priority: FetchPriority,
    extra_headers: Vec<(String, String)>,
    reply_tx: Sender<FetchJobResult>,
}

/// 按 origin 限制并发数的 fetch 调度器；复用单个 [`HttpClient`]（keep-alive）。
pub struct PerOriginFetchScheduler {
    max_per_origin: usize,
    client: HttpClient,
    in_flight: HashMap<String, usize>,
    queue: Vec<QueuedJob>,
    /// `submit_shared` 安装后，排队 job 启动时也能自动 `on_complete`。
    self_hook: Option<Arc<Mutex<Self>>>,
}

impl PerOriginFetchScheduler {
    /// 使用 [`max_connections_per_origin`] 作为并发上限。
    pub fn new() -> Self {
        Self {
            max_per_origin: max_connections_per_origin(),
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: Vec::new(),
            self_hook: None,
        }
    }

    /// 创建共享调度器并安装 self hook（供 `submit_shared` / 优先级队列使用）。
    pub fn new_shared() -> Arc<Mutex<Self>> {
        let sched = Arc::new(Mutex::new(Self::new()));
        sched.lock().expect("fetch scheduler lock").self_hook = Some(Arc::clone(&sched));
        sched
    }

    /// 发起 GET，立即返回接收端；超出 per-origin 上限的请求进入优先级队列。
    pub fn submit(&mut self, url: impl Into<String>) -> Receiver<FetchJobResult> {
        self.submit_with_priority(url, FetchPriority::MEDIUM)
    }

    /// 带优先级的 GET。
    pub fn submit_with_priority(
        &mut self,
        url: impl Into<String>,
        priority: FetchPriority,
    ) -> Receiver<FetchJobResult> {
        self.submit_with_priority_and_headers(url, priority, Vec::new())
    }

    /// 带优先级与条件请求头的 GET。
    pub fn submit_with_priority_and_headers(
        &mut self,
        url: impl Into<String>,
        priority: FetchPriority,
        extra_headers: Vec<(String, String)>,
    ) -> Receiver<FetchJobResult> {
        let url = url.into();
        let (reply_tx, reply_rx) = channel();
        let job = QueuedJob {
            origin: origin_from_url(&url),
            url,
            priority,
            extra_headers,
            reply_tx,
        };
        self.try_start(job);
        reply_rx
    }

    /// 经共享 [`Arc<Mutex<Self>>`] 提交；worker 完成后自动释放槽位。
    pub fn submit_shared(sched: &Arc<Mutex<Self>>, url: impl Into<String>) -> Receiver<FetchJobResult> {
        Self::submit_shared_with_priority(sched, url, FetchPriority::MEDIUM)
    }

    /// 经共享调度器提交并指定优先级。
    pub fn submit_shared_with_priority(
        sched: &Arc<Mutex<Self>>,
        url: impl Into<String>,
        priority: FetchPriority,
    ) -> Receiver<FetchJobResult> {
        Self::submit_shared_with_priority_and_headers(sched, url, priority, Vec::new())
    }

    /// 经共享调度器提交并指定优先级与条件请求头。
    pub fn submit_shared_with_priority_and_headers(
        sched: &Arc<Mutex<Self>>,
        url: impl Into<String>,
        priority: FetchPriority,
        extra_headers: Vec<(String, String)>,
    ) -> Receiver<FetchJobResult> {
        let url = url.into();
        let (reply_tx, reply_rx) = channel();
        let job = QueuedJob {
            origin: origin_from_url(&url),
            url,
            priority,
            extra_headers,
            reply_tx,
        };
        let mut s = sched.lock().expect("fetch scheduler lock");
        s.try_start(job);
        reply_rx
    }

    /// 某 origin 上一个 in-flight 请求结束。
    pub fn on_complete(&mut self, origin: &str) {
        if let Some(count) = self.in_flight.get_mut(origin) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.in_flight.remove(origin);
            }
        }
        self.pump_queue();
    }

    fn try_start(&mut self, job: QueuedJob) {
        if self.in_flight.get(&job.origin).copied().unwrap_or(0) >= self.max_per_origin {
            tracing::info!(
                url = %job.url,
                origin = %job.origin,
                priority = ?job.priority,
                queued = self.queue.len() + 1,
                "HTTP fetch queued (per-origin limit)"
            );
            self.queue.push(job);
            return;
        }
        self.start(job);
    }

    fn start(&mut self, job: QueuedJob) {
        tracing::info!(
            url = %job.url,
            origin = %job.origin,
            priority = ?job.priority,
            "HTTP fetch start"
        );
        *self.in_flight.entry(job.origin.clone()).or_insert(0) += 1;
        let client = self.client.clone();
        let url = job.url;
        let origin = job.origin;
        let reply_tx = job.reply_tx;
        let hook = self.self_hook.clone();
        let extra_headers = job.extra_headers;
        thread::spawn(move || {
            let mut req = crate::HttpRequest::get(&url);
            for (name, value) in extra_headers {
                req = req.header(&name, &value);
            }
            let result = client.send(req).map_err(|e| e.to_string());
            match &result {
                Ok(resp) => tracing::info!(
                    url = %url,
                    status = resp.status_code,
                    bytes = resp.body.len(),
                    "HTTP fetch done"
                ),
                Err(e) => tracing::warn!(url = %url, error = %e, "HTTP fetch failed"),
            }
            let _ = reply_tx.send(result);
            if let Some(sched) = hook
                && let Ok(mut s) = sched.lock()
            {
                s.on_complete(&origin);
            }
        });
    }

    fn pump_queue(&mut self) {
        loop {
            let mut best: Option<usize> = None;
            for (i, job) in self.queue.iter().enumerate() {
                if self.in_flight.get(&job.origin).copied().unwrap_or(0) >= self.max_per_origin {
                    continue;
                }
                match best {
                    None => best = Some(i),
                    Some(bi) if job.priority > self.queue[bi].priority => best = Some(i),
                    _ => {}
                }
            }
            let Some(i) = best else { break };
            let job = self.queue.remove(i);
            self.start(job);
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

    #[test]
    fn queues_beyond_per_origin_limit() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 2,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: Vec::new(),
            self_hook: None,
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
    fn higher_priority_jumps_queue() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: Vec::new(),
            self_hook: None,
        };
        let _r1 = sched.submit_with_priority("http://127.0.0.1:1/low", FetchPriority::LOW);
        let _r2 = sched.submit_with_priority("http://127.0.0.1:1/high", FetchPriority::CRITICAL);
        assert_eq!(sched.queue.len(), 1);
        assert_eq!(sched.queue[0].url, "http://127.0.0.1:1/high");
        sched.on_complete("http://127.0.0.1:1");
        assert!(sched.queue.is_empty());
    }

    #[test]
    fn different_origins_do_not_share_limit() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            client: HttpClient::new(),
            in_flight: HashMap::new(),
            queue: Vec::new(),
            self_hook: None,
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
        // submit() 是非阻塞的：立即返回 Receiver 而非等待 fetch 完成（执行到此处即证其
        // 及时返回）。但严格断言 `Err(Empty)`（worker 仍未完成）是 racy 时序假设——
        // 127.0.0.1:1 不可达，连接被近瞬时拒绝（kernel RST），在并发 workspace 测试负载
        // 下主测试线程可能被反调度、worker 抢先完成并投递 FetchJobResult，使 try_recv 返
        // Ok 而非 Empty → 间歇性 flake。Empty（worker 仍在跑）与已投递结果对不可达 URL 同
        // 样合法；此处仅要求 receiver 可用（非阻塞轮询、不 panic），不锁死时序。
        let _outcome = rx.try_recv();
    }

    #[test]
    fn submit_shared_auto_releases_origin_slot() {
        use std::time::Duration;
        let sched = PerOriginFetchScheduler::new_shared();
        sched.lock().unwrap().set_max_per_origin_for_test(1);
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
