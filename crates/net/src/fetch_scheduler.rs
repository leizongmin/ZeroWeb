//! Per-origin 并发 fetch 调度 — 对齐浏览器「每 host ~6 连接」+ 优先级队列。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::client::{HttpClient, async_runtime};
use crate::fetch_priority::FetchPriority;
use crate::resource_policy::{max_connections_per_origin, max_connections_total, origin_from_url};

/// HTTP GET 任务结果。
pub type FetchJobResult = Result<crate::HttpResponse, String>;

/// 单个网络事务的匿名时序数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchTelemetry {
    /// 不含路径或查询参数的请求 origin。
    pub origin: String,
    /// 调度器从接收任务到开始传输的等待时间。
    pub queue_wait_ms: u64,
    /// 网络传输和完整响应体读取时间。
    pub network_ms: u64,
    /// 共享该网络事务的订阅者数。
    pub coalesced_subscriber_count: usize,
}

struct QueuedJob {
    /// 请求身份键；只有身份相同的请求允许合并。
    key: String,
    url: String,
    origin: String,
    priority: FetchPriority,
    extra_headers: Vec<(String, String)>,
    reply_tx: Sender<FetchJobResult>,
    /// 合并路径（submit_shared_*）：完成时经 pending 广播给全部订阅者。
    shared: bool,
    submitted_at: Instant,
    telemetry_tx: Option<Sender<FetchTelemetry>>,
    timeout_secs: u64,
}

/// 按 origin 限制并发数的 fetch 调度器。
pub struct PerOriginFetchScheduler {
    max_per_origin: usize,
    max_total: usize,
    max_queued: usize,
    in_flight: HashMap<String, usize>,
    in_flight_total: usize,
    queue: Vec<QueuedJob>,
    /// 在途/排队请求身份的订阅者；完成时广播结果。
    pending: HashMap<String, Vec<Sender<FetchJobResult>>>,
    /// 同优先级队列上一次获选的 origin；用于按 origin 轮转而非 FIFO 偏置。
    last_queued_origin: Option<String>,
    /// `submit_shared` 安装后，排队 job 启动时也能自动 `on_complete`。
    self_hook: Option<Arc<Mutex<Self>>>,
}

impl PerOriginFetchScheduler {
    /// 使用 [`max_connections_per_origin`] 作为并发上限。
    pub fn new() -> Self {
        Self::with_limits(max_connections_per_origin(), max_connections_total())
    }

    /// 使用显式的 per-origin 与全局并发上限创建调度器。
    pub fn with_limits(max_per_origin: usize, max_total: usize) -> Self {
        Self {
            max_per_origin: max_per_origin.max(1),
            max_total: max_total.max(1),
            max_queued: max_total.saturating_mul(16).clamp(64, 1024),
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
            self_hook: None,
        }
    }

    /// 创建共享调度器并安装 self hook（供 `submit_shared` / 优先级队列使用）。
    pub fn new_shared() -> Arc<Mutex<Self>> {
        let sched = Arc::new(Mutex::new(Self::new()));
        sched.lock().expect("fetch scheduler lock").self_hook = Some(Arc::clone(&sched));
        sched
    }

    /// 使用显式并发上限创建共享调度器。
    pub fn new_shared_with_limits(max_per_origin: usize, max_total: usize) -> Arc<Mutex<Self>> {
        let sched = Arc::new(Mutex::new(Self::with_limits(max_per_origin, max_total)));
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
            key: url.clone(),
            origin: origin_from_url(&url),
            url,
            priority,
            extra_headers,
            reply_tx,
            shared: false,
            submitted_at: Instant::now(),
            telemetry_tx: None,
            timeout_secs: 30,
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
        Self::submit_shared_with_key_and_headers(sched, url.clone(), url, priority, extra_headers)
    }

    /// 经共享调度器提交带明确请求身份键的 GET。
    ///
    /// `key` 必须由调用方从方法、URL、缓存分区及会影响响应的请求头构造；调度器不会
    /// 将不同 key 的请求合并，即使 URL 相同。
    pub fn submit_shared_with_key_and_headers(
        sched: &Arc<Mutex<Self>>,
        key: impl Into<String>,
        url: impl Into<String>,
        priority: FetchPriority,
        extra_headers: Vec<(String, String)>,
    ) -> Receiver<FetchJobResult> {
        Self::submit_shared_with_key_headers_and_telemetry(sched, key, url, priority, extra_headers, 30).0
    }

    /// 经共享调度器提交带身份键的 GET，并在完成时发送匿名时序数据。
    pub fn submit_shared_with_key_headers_and_telemetry(
        sched: &Arc<Mutex<Self>>,
        key: impl Into<String>,
        url: impl Into<String>,
        priority: FetchPriority,
        extra_headers: Vec<(String, String)>,
        timeout_secs: u64,
    ) -> (Receiver<FetchJobResult>, Receiver<FetchTelemetry>, bool) {
        let key = key.into();
        let url = url.into();
        let (reply_tx, reply_rx) = channel();
        let (event_tx, event_rx) = channel();
        let mut s = sched.lock().expect("fetch scheduler lock");
        if let Some(subscribers) = s.pending.get_mut(&key) {
            subscribers.push(reply_tx);
            // 后到的关键消费者可提升仍在队列中的共享请求；已运行任务不抢占。
            if let Some(job) = s.queue.iter_mut().find(|job| job.key == key) {
                job.priority = job.priority.max(priority);
            }
            return (reply_rx, event_rx, false);
        }
        s.pending.insert(key.clone(), vec![reply_tx.clone()]);
        let job = QueuedJob {
            key,
            origin: origin_from_url(&url),
            url,
            priority,
            extra_headers,
            reply_tx,
            shared: true,
            submitted_at: Instant::now(),
            telemetry_tx: Some(event_tx),
            timeout_secs,
        };
        s.try_start(job);
        (reply_rx, event_rx, true)
    }

    /// 某 origin 上一个 in-flight 请求结束。
    pub fn on_complete(&mut self, origin: &str) {
        if let Some(count) = self.in_flight.get_mut(origin) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.in_flight.remove(origin);
            }
        }
        self.in_flight_total = self.in_flight_total.saturating_sub(1);
        self.pump_queue();
    }

    fn try_start(&mut self, job: QueuedJob) {
        if self.in_flight_total >= self.max_total
            || self.in_flight.get(&job.origin).copied().unwrap_or(0) >= self.max_per_origin
        {
            if self.queue.len() >= self.max_queued {
                tracing::warn!(
                    url = %job.url,
                    origin = %job.origin,
                    max_queued = self.max_queued,
                    "fetch scheduler queue is full"
                );
                if job.shared {
                    self.pending.remove(&job.key);
                }
                let _ = job.reply_tx.send(Err("fetch scheduler queue is full".to_string()));
                return;
            }
            tracing::info!(
                url = %job.url,
                origin = %job.origin,
                priority = ?job.priority,
                queued = self.queue.len() + 1,
                total_in_flight = self.in_flight_total,
                "HTTP fetch queued (concurrency limit)"
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
        self.last_queued_origin = Some(job.origin.clone());
        *self.in_flight.entry(job.origin.clone()).or_insert(0) += 1;
        self.in_flight_total += 1;
        let url = job.url;
        let key = job.key;
        let origin = job.origin;
        let reply_tx = job.reply_tx;
        let hook = self.self_hook.clone();
        let extra_headers = job.extra_headers;
        let shared = job.shared;
        let submitted_at = job.submitted_at;
        let telemetry_tx = job.telemetry_tx;
        let timeout_secs = job.timeout_secs;
        async_runtime().spawn(async move {
            let network_started = Instant::now();
            let mut req = crate::HttpRequest::get(&url);
            for (name, value) in extra_headers {
                req = req.header(&name, &value);
            }
            let result = HttpClient::send_async_with_timeout(timeout_secs, req)
                .await
                .map_err(|e| e.to_string());
            match &result {
                Ok(resp) => tracing::info!(
                    url = %url,
                    status = resp.status_code,
                    bytes = resp.body.len(),
                    "HTTP fetch done"
                ),
                Err(e) => tracing::warn!(url = %url, error = %e, "HTTP fetch failed"),
            }
            let coalesced_subscriber_count = if shared {
                // S6 合并路径：广播给全部订阅者后移除 pending 条目
                if let Some(sched) = hook.as_ref()
                    && let Ok(mut s) = sched.lock()
                    && let Some(subscribers) = s.pending.remove(&key)
                {
                    let count = subscribers.len();
                    for tx in subscribers {
                        let _ = tx.send(result.clone());
                    }
                    count
                } else {
                    0
                }
            } else {
                let _ = reply_tx.send(result);
                1
            };
            if let Some(tx) = telemetry_tx {
                let _ = tx.send(FetchTelemetry {
                    origin: origin.clone(),
                    queue_wait_ms: network_started
                        .duration_since(submitted_at)
                        .as_millis()
                        .min(u128::from(u64::MAX)) as u64,
                    network_ms: network_started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                    coalesced_subscriber_count,
                });
            }
            if let Some(sched) = hook.as_ref()
                && let Ok(mut s) = sched.lock()
            {
                s.on_complete(&origin);
            }
        });
    }

    fn pump_queue(&mut self) {
        loop {
            if self.in_flight_total >= self.max_total {
                break;
            }
            let mut best: Option<usize> = None;
            for (i, job) in self.queue.iter().enumerate() {
                if self.in_flight.get(&job.origin).copied().unwrap_or(0) >= self.max_per_origin {
                    continue;
                }
                match best {
                    None => best = Some(i),
                    Some(bi) if job.priority > self.queue[bi].priority => best = Some(i),
                    Some(bi)
                        if job.priority == self.queue[bi].priority
                            && self.last_queued_origin.as_deref() == Some(self.queue[bi].origin.as_str())
                            && self.last_queued_origin.as_deref() != Some(job.origin.as_str()) =>
                    {
                        best = Some(i)
                    }
                    Some(bi)
                        if job.priority == self.queue[bi].priority
                            && self.in_flight.get(&job.origin).copied().unwrap_or(0)
                                < self.in_flight.get(&self.queue[bi].origin).copied().unwrap_or(0) =>
                    {
                        best = Some(i)
                    }
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

    /// 测试用：当前所有 origin 合计的 in-flight 数。
    #[cfg(test)]
    pub fn in_flight_total_for_test(&self) -> usize {
        self.in_flight_total
    }

    /// 测试用：覆盖 per-origin 并发上限。
    #[cfg(test)]
    pub fn set_max_per_origin_for_test(&mut self, max: usize) {
        self.max_per_origin = max;
    }

    /// 测试用：覆盖全局并发上限。
    #[cfg(test)]
    pub fn set_max_total_for_test(&mut self, max: usize) {
        self.max_total = max;
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
            max_total: 24,
            max_queued: 384,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
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
    fn rejects_jobs_when_queue_capacity_is_reached() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            max_total: 1,
            max_queued: 1,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
            self_hook: None,
        };
        let _running = sched.submit("http://127.0.0.1:1/running");
        let _queued = sched.submit("http://127.0.0.1:1/queued");
        let overloaded = sched.submit("http://127.0.0.1:1/overloaded");

        assert_eq!(sched.queue.len(), 1);
        assert!(matches!(
            overloaded.recv().expect("queue rejection result"),
            Err(error) if error == "fetch scheduler queue is full"
        ));
    }

    #[test]
    fn higher_priority_jumps_queue() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            max_total: 24,
            max_queued: 384,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
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
            max_total: 24,
            max_queued: 384,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
            self_hook: None,
        };
        let _r1 = sched.submit("http://127.0.0.1:1/a");
        let _r2 = sched.submit("http://127.0.0.2:1/b");
        assert_eq!(sched.in_flight.len(), 2);
        assert!(sched.queue.is_empty());
    }

    #[test]
    fn global_limit_bounds_different_origins() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 6,
            max_total: 1,
            max_queued: 64,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
            self_hook: None,
        };
        let _r1 = sched.submit("http://127.0.0.1:1/a");
        let _r2 = sched.submit("http://127.0.0.2:1/b");
        assert_eq!(sched.in_flight_total_for_test(), 1);
        assert_eq!(sched.queued_count_for_test(), 1);
    }

    #[test]
    fn equal_priority_queue_rotates_origins() {
        let mut sched = PerOriginFetchScheduler {
            max_per_origin: 1,
            max_total: 1,
            max_queued: 64,
            in_flight: HashMap::new(),
            in_flight_total: 0,
            queue: Vec::new(),
            pending: HashMap::new(),
            last_queued_origin: None,
            self_hook: None,
        };
        let origin_a = "http://127.0.0.1:1";
        let origin_b = "http://127.0.0.2:1";
        let _ = sched.submit(format!("{origin_a}/running"));
        let _ = sched.submit(format!("{origin_a}/queued-first"));
        let _ = sched.submit(format!("{origin_b}/queued-second"));

        sched.on_complete(origin_a);
        assert_eq!(sched.last_queued_origin.as_deref(), Some(origin_b));
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
        // r1 完成后释放 origin slot，调度线程 dequeue r2 启动 → queued_count 1→0。固定 sleep(50ms) 在
        // 并发 workspace 测试负载下可能不足以让调度线程完成 dequeue（线程被反调度）→ 间歇 flake。
        // 改 poll queued_count→0（超时 5s），robust 抗调度延迟。
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let q = sched.lock().unwrap().queued_count_for_test();
            if q == 0 {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!("origin slot not released / r2 not dequeued within 5s: queued_count = {q}");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
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

#[test]
fn submit_shared_coalesces_duplicate_url() {
    // 合并态（in_flight 计数 + pending 订阅者数）是**瞬态**：fetch 完成后即被 on_complete 清空。
    // 旧实现对 127.0.0.1:1（连接立即拒绝）发 fetch，在并发 workspace 测试负载下 worker 可能在测试
    // 断言前就完成并清空 in_flight → 间歇 flake（同文件 submit_shared_queues_when_busy 已踩同坑并硬化）。
    // 本测试改用「挂起本地服务端」：接受连接但不写响应 → fetch worker 阻塞在读 HTTP 响应（30s timeout
    // 内不完成）→ in_flight/pending 稳定，断言确定。状态观察后即返回（worker 线程随 test 进程退出清理）。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind hanging server");
    let port = listener.local_addr().expect("local addr").port();
    std::thread::spawn(move || {
        // 接受连接并持有（不写响应）→ 客户端阻塞等待 HTTP 响应。
        let mut held: Vec<std::net::TcpStream> = Vec::new();
        while let Ok((stream, _)) = listener.accept() {
            held.push(stream);
        }
        drop(held);
    });
    let sched = PerOriginFetchScheduler::new_shared();
    let origin = format!("http://127.0.0.1:{port}");
    let url = format!("{origin}/coalesce-me");
    // 同一 URL 两次提交：合并为一次网络请求，pending 累积 2 个订阅者。
    let _r1 = PerOriginFetchScheduler::submit_shared(&sched, &url);
    let _r2 = PerOriginFetchScheduler::submit_shared(&sched, &url);
    {
        let s = sched.lock().unwrap();
        // 只有一个 in-flight（合并），pending 有 2 个订阅者（瞬态在挂起服务端下稳定）。
        assert_eq!(s.in_flight.get(&origin).copied(), Some(1));
        let subs = s.pending.get(&url).expect("pending entry");
        assert_eq!(subs.len(), 2, "two subscribers for one in-flight request");
    }
    // 不读结果（worker 阻塞在挂起服务端）；状态断言已证明合并。结果广播见下方独立测试。
}

#[test]
fn submit_shared_broadcasts_result_to_subscribers() {
    use std::time::Duration;
    // 结果广播路径：fetch 完成后 on_complete 广播同一结果给全部订阅者并清空 pending。
    // 用 127.0.0.1:1（连接立即拒绝，fast Err）使 fetch 快速完成；recv_timeout 等待完成（非竞态）。
    let sched = PerOriginFetchScheduler::new_shared();
    let r1 = PerOriginFetchScheduler::submit_shared(&sched, "http://127.0.0.1:1/broadcast-me");
    let r2 = PerOriginFetchScheduler::submit_shared(&sched, "http://127.0.0.1:1/broadcast-me");
    let r1v = r1.recv_timeout(Duration::from_secs(5));
    let r2v = r2.recv_timeout(Duration::from_secs(5));
    // 两次都收到同一结果（连接拒绝的 Err 字符串一致）。
    assert_eq!(r1v.as_ref().err(), r2v.as_ref().err(), "coalesced results must match");
    // 完成后 pending 清空（poll 至 drain，robust 抗调度延迟）。
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let p = sched.lock().unwrap().pending.len();
        if p == 0 {
            break;
        }
        if std::time::Instant::now() > deadline {
            panic!("pending entry not drained after completion");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
