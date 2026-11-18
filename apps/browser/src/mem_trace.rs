//! T79 内存归因仪拍：browser 进程 glibc mallinfo2 五秒窗一行（env 门控诊断侧信道）。
//!
//! 用途 = 把 C1 判据窗的 browser 进程增量（by_proc Δ+81MB）劈分到「分配器持有
//! 面 arena/hblkhd vs 在用/空闲」，定位 cache/连接池/IPC 缓冲之外的真实增长桶。
//! 与 renderer 侧 `ZW_RENDERER_MEM_TRACE`（apps/renderer/src/js_worker.rs
//! [`crate::js_worker`] WorkerMemTrace / malloc_stats）同形制；mallinfo2 wrapper
//! 因 crate 边界（browser 不依赖 renderer）本地复制，非 linux-gnu glibc 恒 n/a。
//! 纯观测：默认关闭时每轮仅一次 Instant 比较；值 `1`/`true` → tracing INFO；
//! 其余非空值 → 追加写文件。
//!
//! 另含 T79 S9 分配器滞留收敛接线（[`tune_malloc_retention_policy`]）：
//! browser 进程此前未接 tune（renderer 已接），free 池 65-122MB 为 C1 增量
//! 主桶之一——实现+开关在 zero-runtime-config，两 bin 同刀。

/// 一次分配器快照（字节；`hblkhd` 为 mmap 大块持有面，不含在 arena）。
#[derive(Debug, Clone, Copy, Default)]
struct MallocSnapshot {
    arena_bytes: u64,
    hblkhd_bytes: u64,
    in_use_bytes: u64,
    free_bytes: u64,
}

/// glibc `mallinfo2(3)` 快照；非 linux-gnu 目标恒 `None`。
fn mallinfo2_snapshot() -> Option<MallocSnapshot> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: `mallinfo2` 无参数、读全局分配器簿记并持内部锁；多线程下
        // glibc 自串行化，返回值纯数据无指针语义。
        let info = unsafe { libc::mallinfo2() };
        Some(MallocSnapshot {
            arena_bytes: info.arena as u64,
            hblkhd_bytes: info.hblkhd as u64,
            in_use_bytes: info.uordblks as u64,
            free_bytes: info.fordblks as u64,
        })
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        None
    }
}

/// browser 事件环每轮调用的内存仪拍（`ZW_BROWSER_MEM_TRACE` 启用）。
pub(crate) struct BrowserMemTrace {
    enabled: bool,
    sink: Option<std::path::PathBuf>,
    window_start: std::time::Instant,
    /// T79 诊断：周期 trim 窗（`ZW_BROWSER_MEM_TRIM_S` 非零秒数启用；诊断侧信道，
    /// 非产品路径——量化 glibc free 池 [`malloc_trim`] 净让量后即移除/转产品刀）。
    trim_period: Option<std::time::Duration>,
    last_trim: std::time::Instant,
}

impl BrowserMemTrace {
    pub(crate) fn new() -> Self {
        let value = zero_runtime_config::optional_string("ZW_BROWSER_MEM_TRACE");
        let truthy = value
            .as_deref()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        // falsy 值与 truthy 值都不当作 sink 路径（`MEM_TRACE=0` 不落文件、
        // `MEM_TRACE=1` 不在 cwd 留名为 `1` 的垃圾文件）。
        let sink = value
            .filter(|v| !truthy && !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
            .map(std::path::PathBuf::from);
        let trim_secs = zero_runtime_config::optional_string("ZW_BROWSER_MEM_TRIM_S")
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

    /// 事件环每轮调用：窗口 ≥5s 落一行快照。t=epoch ms（与 renderer 侧
    /// mem trace / automation trace 的 t= 同源，可跨文件对齐时间线）。
    pub(crate) fn maybe_flush(&mut self) {
        if !self.enabled {
            return;
        }
        self.maybe_trim();
        if self.window_start.elapsed() < std::time::Duration::from_secs(5) {
            return;
        }
        self.window_start = std::time::Instant::now();
        let ts = epoch_ms();
        let line = match mallinfo2_snapshot() {
            Some(s) => format!(
                "browser-mem t={ts} mallinfo2=arena={}B hblkhd={}B in_use={}B free={}B",
                s.arena_bytes, s.hblkhd_bytes, s.in_use_bytes, s.free_bytes
            ),
            None => format!("browser-mem t={ts} mallinfo2=n/a"),
        };
        self.write_line(&line);
    }

    /// 到点 `malloc_trim(0)`：trim 前后各落一行 mallinfo2（差 = 净让量）。
    fn maybe_trim(&mut self) {
        let Some(period) = self.trim_period else { return };
        if self.last_trim.elapsed() < period {
            return;
        }
        self.last_trim = std::time::Instant::now();
        let before = mallinfo2_snapshot();
        // SAFETY: `malloc_trim(0)` 只对 malloc 空闲页 madvise 归还，无指针语义、
        // 不触碰活对象；多线程下 glibc 自带 arena 锁。
        let released = unsafe { libc::malloc_trim(0) };
        let after = mallinfo2_snapshot();
        let fmt = |s: &Option<MallocSnapshot>| match s {
            Some(s) => format!(
                "arena={}B hblkhd={}B in_use={}B free={}B",
                s.arena_bytes, s.hblkhd_bytes, s.in_use_bytes, s.free_bytes
            ),
            None => "n/a".to_string(),
        };
        let line = format!(
            "browser-trim t={} released={} before=[{}] after=[{}]",
            epoch_ms(),
            released == 1,
            fmt(&before),
            fmt(&after),
        );
        self.write_line(&line);
    }

    fn write_line(&self, line: &str) {
        if let Some(path) = &self.sink
            && let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            use std::io::Write as _;
            let _ = writeln!(f, "{line}");
        } else if self.sink.is_none() {
            tracing::info!(target: "browser_mem_trace", "{line}");
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

/// T79 S9：browser 进程分配器滞留收敛（启动单次调用，零热路径）。
/// gate+mallopt 实现在 zero-runtime-config（两 bin 同刀，renderer 侧 bin 记录
/// 见 apps/renderer/src/mem_reclaim.rs）；此处保留 bin 侧 tracing 记录，关闭
/// 与非 glibc 目标静默不调。
pub(crate) fn tune_malloc_retention_policy() {
    if let Some((arena, mmap, trim)) = zero_runtime_config::tune_malloc_retention_policy() {
        tracing::info!(
            target: "browser_mem_trace",
            arena_max_ok = arena,
            mmap_threshold_ok = mmap,
            trim_threshold_ok = trim,
            "malloc retention policy tuned"
        );
    }
}
