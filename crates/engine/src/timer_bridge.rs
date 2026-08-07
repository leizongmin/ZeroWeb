//! P1b S5 定时器 bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持 [`AsyncResolver`]；`register` 在 sandbox 注 `__zw_setTimeout(id, delayMs)` 回调。
//! JS `setTimeout/setInterval`（shim 侧）把回调存 `__zw_pending[id]` 后调本回调；本回调把
//! `(expiry, seq, id)` 入队到**单一协调线程**的优先队列，协调线程按 `(expiry, seq)` 顺序到期时
//! `resolver.resolve(id, "")` 投递回 JS worker → shim `__zwResolveCallback` 取出并调用回调。
//!
//! **R2952 单协调线程**：替代此前「每定时器一子线程 sleep(delay)」模型——多 `setTimeout(fn, 0)`
//! 时各子线程 race 谁先 `resolver.resolve`，致同 delay（尤 0）定时器触发顺序不确定（spec 要求
//! 注册序 FIFO）。单线程 + min-heap（按 `(expiry, seq)`，seq 为注册计数）保证同 delay 严格 FIFO，
//! `setInterval`/`requestIdleCallback` 经 shim re-arm 不变（每 tick 一条新 entry，正确间隔）。
//!
//! `clearTimeout/clearInterval` 由 shim 删 `__zw_pending[id]` 实现——即便协调线程后到 resolve，
//! `__zwResolveCallback` 见无 pending 项即 no-op。

use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use zero_script_sandbox::Sandbox;

use crate::async_resolver::AsyncResolver;

/// 单条定时器——min-heap 项。`Ord` 反序（BinaryHeap 为 max-heap，反序得 min-heap：最早到期先出）。
#[derive(Clone)]
struct TimerEntry {
    expiry: Instant,
    seq: u64,
    id: String,
}
impl PartialEq for TimerEntry {
    fn eq(&self, o: &Self) -> bool {
        self.expiry == o.expiry && self.seq == o.seq
    }
}
impl Eq for TimerEntry {}
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for TimerEntry {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // 反序：max-heap 顶 = (expiry, seq) 最大 → pop 出最小（最早到期 + 最早注册）。
        o.expiry.cmp(&self.expiry).then_with(|| o.seq.cmp(&self.seq))
    }
}

/// 协调器可变状态（队列 + shutdown 标志），经 mutex 保护。
struct CoordInner {
    queue: BinaryHeap<TimerEntry>,
    shutdown: bool,
}

/// P1b S5 定时器 bridge——`setTimeout`/`setInterval` 真实延迟（单协调线程 + 优先队列 + 异步 resolve）。
pub struct TimerBridge {
    inner: Arc<(Mutex<CoordInner>, Condvar)>,
    seq: Arc<AtomicU64>,
    shutdown_flag: Arc<AtomicBool>,
    coord_thread: Mutex<Option<JoinHandle<()>>>,
}

impl TimerBridge {
    /// 构造——`resolver` 用于定时器到期后 resolve（触发 shim 调用 JS 回调）。启动单一协调线程。
    pub fn new(resolver: AsyncResolver) -> Self {
        let inner = Arc::new((
            Mutex::new(CoordInner {
                queue: BinaryHeap::new(),
                shutdown: false,
            }),
            Condvar::new(),
        ));
        let seq = Arc::new(AtomicU64::new(0));
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        // 启动协调线程：循环到期触发 + 等待唤醒。
        let coord_inner = Arc::clone(&inner);
        let coord_resolver = resolver;
        let coord_shutdown = Arc::clone(&shutdown_flag);
        let handle = std::thread::Builder::new()
            .name("zw-timer-coord".into())
            .spawn(move || coordinator_loop(coord_inner, coord_resolver, coord_shutdown))
            .expect("spawn timer coordinator");
        Self {
            inner,
            seq,
            shutdown_flag,
            coord_thread: Mutex::new(Some(handle)),
        }
    }

    /// 注册 `__zw_setTimeout(id, delayMs)` 回调——shim 的 `setTimeout`/`setInterval` 调此。
    /// **非阻塞**：仅入队 `(Instant::now()+delay, seq, id)` + 唤醒协调线程；后者按 (expiry, seq)
    /// 顺序到期 resolve（同 delay 严格 FIFO——修 R2952 前 per-timer 子线程竞态）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let inner = Arc::clone(&self.inner);
        let seq = Arc::clone(&self.seq);
        sandbox.register_callback(
            "__zw_setTimeout",
            Box::new(move |args: &[String]| -> String {
                let id = args.first().cloned().unwrap_or_default();
                let delay_ms: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let entry = TimerEntry {
                    expiry: Instant::now() + Duration::from_millis(delay_ms),
                    seq: seq.fetch_add(1, Ordering::Relaxed),
                    id,
                };
                {
                    let (m, c) = &*inner;
                    if let Ok(mut guard) = m.lock() {
                        guard.queue.push(entry);
                    }
                    c.notify_one();
                }
                String::new()
            }),
        );
    }
}

impl Drop for TimerBridge {
    fn drop(&mut self) {
        // 通知协调线程 shutdown，join 防孤儿线程。
        self.shutdown_flag.store(true, Ordering::Release);
        {
            let (_m, c) = &*self.inner;
            if let Ok(mut guard) = _m.lock() {
                guard.shutdown = true;
            }
            c.notify_all();
        }
        if let Ok(mut slot) = self.coord_thread.lock()
            && let Some(handle) = slot.take()
        {
            let _ = handle.join();
        }
    }
}

/// 协调线程主循环：到期触发（按 min-heap 顺序，同 expiry 按 seq FIFO）+ 无定时器时等待唤醒。
fn coordinator_loop(inner: Arc<(Mutex<CoordInner>, Condvar)>, resolver: AsyncResolver, shutdown_flag: Arc<AtomicBool>) {
    let (m, c) = &*inner;
    let mut guard = m.lock().expect("timer coord lock");
    loop {
        if guard.shutdown || shutdown_flag.load(Ordering::Acquire) {
            return;
        }
        // 收集所有到期定时器（min-heap pop 顺序 = (expiry, seq) 升序 → 同 delay FIFO）。
        let now = Instant::now();
        let mut due: Vec<TimerEntry> = Vec::new();
        while guard.queue.peek().is_some_and(|e| e.expiry <= now) {
            due.push(guard.queue.pop().expect("peek 证非空"));
        }
        if !due.is_empty() {
            // 释放锁后 resolve（避免持锁期间跨线程投递）；按 pop 顺序（升序）resolve 保 FIFO。
            drop(guard);
            for entry in due {
                resolver.resolve(&entry.id, "");
            }
            guard = m.lock().expect("timer coord lock re-acquire");
            continue;
        }
        // 无到期：等到下一个定时器的 expiry，或被唤醒（新定时器 / shutdown）。
        let wait = guard
            .queue
            .peek()
            .map(|e| e.expiry.saturating_duration_since(Instant::now()))
            .unwrap_or_else(|| Duration::from_secs(3600));
        let (g, _) = c.wait_timeout(guard, wait).expect("timer coord wait");
        guard = g;
    }
}
