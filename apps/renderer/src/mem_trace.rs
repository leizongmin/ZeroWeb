//! T104 G2 半刀：renderer mallinfo/V8 时序仪拍（`ZW_RENDERER_MEM_TRACE` 启用）
//! 自 js worker 线程迁 renderer 主环（零行为变化——纯观测仪器路径迁移）。
//!
//! 迁移原因：worker 线程随 tab 退出（窗闭合 ~3-4s）消亡，5s 汇出窗 post-close
//! 永不满足（原 `worker-mem` 行结构性为空，t104-memory-anatomy.md §六-2）；
//! 主环存活全程，稳态 mallinfo 时序与 trim 诊断窗在 close 后持续可得。
//! 形制承继：env 三态（`1`/`true` → tracing INFO；`0`/`false`/`off` → 关；
//! 其余非空值 → 追加写文件）与 5s 汇出窗不变；线名 `worker-mem`/`worker-trim`
//! 更名 `renderer-mem`/`renderer-trim`（采样点语义迁移，t=epoch ms 时间线口径
//! 不变）。V8 已用堆自报改经 ProbeHeap 共享槽读数（worker 活跃期至多滞后一个
//! 命令队列深度；worker 退出后冻结在末次读数——isolate 已消亡，无更新源）。
//! mallinfo2 wrapper 复用 [`crate::malloc_stats`]；周期 trim 窗
//! `ZW_RENDERER_MEM_TRIM_S` 诊断侧信道随迁（原 worker 侧同窗结构性饥饿）。

/// renderer 主环 mallinfo/V8 时序仪拍（原 js_worker `WorkerMemTrace` 迁移形制）。
pub(crate) struct RendererMemTrace {
    enabled: bool,
    sink: Option<std::path::PathBuf>,
    window_start: std::time::Instant,
    /// T79 诊断：周期 trim 窗（`ZW_RENDERER_MEM_TRIM_S` 非零秒数启用；诊断侧信
    /// 道，非产品路径——产品刀走 idle trim/nav trim 生命周期）。
    trim_period: Option<std::time::Duration>,
    last_trim: std::time::Instant,
}

impl RendererMemTrace {
    pub(crate) fn new() -> Self {
        let value = zero_runtime_config::optional_string("ZW_RENDERER_MEM_TRACE");
        let truthy = value
            .as_deref()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        // 同 WorkerCensus：falsy 值与 truthy 值都不当作 sink 路径（`MEM_TRACE=0`
        // 不落文件、`MEM_TRACE=1` 不在 cwd 留名为 `1` 的垃圾文件）。
        let sink = value
            .filter(|v| !truthy && !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
            .map(std::path::PathBuf::from);
        let trim_secs = zero_runtime_config::optional_string("ZW_RENDERER_MEM_TRIM_S")
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|s| *s > 0);
        Self {
            enabled: truthy || sink.is_some(),
            sink,
            window_start: std::time::Instant::now(),
            trim_period: trim_secs.map(std::time::Duration::from_secs),
            last_trim: std::time::Instant::now(),
        }
    }

    fn write_sink(&self, line: &str) {
        if let Some(path) = &self.sink
            && let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            use std::io::Write as _;
            if let Err(e) = writeln!(f, "{line}") {
                // 诊断侧信道写失败可见化：静默吞错会掩盖 sink 病因。
                tracing::warn!(target: "renderer_mem_trace", "sink write failed: {e}");
            }
        }
    }

    /// 主环每迭代调用：窗口 ≥5s 落一行快照（mallinfo2 + V8 堆自报槽读数）。
    /// `worker` 经 ProbeHeap 补发请求（活跃期读数至多滞后一个命令队列深度）；
    /// t=epoch ms（与 automation trace / browser-mem 行的 t= 同源，可跨文件对齐
    /// 时间线）。
    pub(crate) fn maybe_flush(&mut self, worker: &crate::js_worker::RendererJsWorker) {
        if !self.enabled {
            return;
        }
        self.maybe_trim();
        if self.window_start.elapsed() < std::time::Duration::from_secs(5) {
            return;
        }
        self.window_start = std::time::Instant::now();
        let _ = worker.request_v8_heap_probe();
        let js = worker.latest_v8_heap_bytes();
        let ts = epoch_ms();
        let line = format!(
            "renderer-mem t={ts} mallinfo2={} js={}",
            match crate::malloc_stats::mallinfo2_snapshot() {
                Some(s) => format!(
                    "arena={}B hblkhd={}B in_use={}B free={}B",
                    s.arena_bytes, s.hblkhd_bytes, s.in_use_bytes, s.free_bytes
                ),
                None => "n/a".to_string(),
            },
            match js {
                Some(v) => format!("used={v}B"),
                None => "n/a".to_string(),
            },
        );
        if self.sink.is_some() {
            self.write_sink(&line);
        } else {
            tracing::info!(target: "renderer_mem_trace", "{line}");
        }
    }

    /// 到点 `malloc_trim(0)`：trim 前后各落一行 mallinfo2（差 = 净让量；诊断
    /// 量化用——trim 后残量即「部分空闲页段」收不回的部分，读数直读）。
    fn maybe_trim(&mut self) {
        let Some(period) = self.trim_period else { return };
        if self.last_trim.elapsed() < period {
            return;
        }
        self.last_trim = std::time::Instant::now();
        let ts = epoch_ms();
        let fmt = |s: Option<crate::malloc_stats::MallocSnapshot>| match s {
            Some(s) => format!(
                "arena={}B hblkhd={}B in_use={}B free={}B",
                s.arena_bytes, s.hblkhd_bytes, s.in_use_bytes, s.free_bytes
            ),
            None => "n/a".to_string(),
        };
        let before = crate::malloc_stats::mallinfo2_snapshot();
        // SAFETY: `malloc_trim(0)` 只对 malloc 空闲页 madvise 归还，无指针语义、
        // 不触碰活对象；多线程下 glibc 自带 arena 锁。符号仅 linux-gnu glibc
        // 提供，其余目标编译为 no-op（与 mallinfo2_snapshot 同 gate，恒记 false）。
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        let released = unsafe { libc::malloc_trim(0) };
        #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
        let released = 0;
        let after = crate::malloc_stats::mallinfo2_snapshot();
        let line = format!(
            "renderer-trim t={ts} released={} before=[{}] after=[{}]",
            released == 1,
            fmt(before),
            fmt(after),
        );
        if self.sink.is_some() {
            self.write_sink(&line);
        } else {
            tracing::info!(target: "renderer_mem_trace", "{line}");
        }
    }
}

/// epoch 毫秒（跨文件时间线对齐用；时钟回退时退化为 0，仅诊断侧信道可接受）。
fn epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
