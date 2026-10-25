//! T13 S2/S7：导航落定分配器归还与内存探针（设计卡 §3 推荐方案 A）。
//!
//! `malloc_trim` 仅 linux-gnu glibc 提供（musl 无此符号、macOS/Windows 无对应
//! 单调用；桌面 glibc 主目标，设计卡 §8-7 披露），其余目标编译为 no-op。
//! 两个落定点：js_worker Reset 臂尾（JS heap free 归还）与主侧导航落定
//! （DOM/cache free 归还）；两处都只归还 free 页，活对象零触碰。

/// T13 S2：导航 trim 开关——`ZEROWEB_NAV_TRIM=0` 关（缺省开）。
pub(crate) fn nav_trim_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZEROWEB_NAV_TRIM")
}

/// T13 S7：内存探针——`ZEROWEB_MEM_PROBE=1` 开（缺省关；观测腿不进判据样本）。
pub(crate) fn mem_probe_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZEROWEB_MEM_PROBE")
}

/// T13 S2：`malloc_trim(0)` 归还 glibc arena free 页（异步边界落定点调用）。
///
/// `stage` 进 tracing 字段（`worker_trim`/`nav_trim`）——A/B 命中语义按事件计数
/// （每迭代 trim/shrink/clear 事件数 ≥ 导航代数，冻结协议）。探针腿附带
/// trim 前后进程 PSS 采样（`pss_before_kb`/`pss_after_kb`，read 失败记 0）——
/// trim 净让量 = 差值（其他线程并发分配可污染单差值，判读按分布取 median）。
pub(crate) fn malloc_trim_now(stage: &'static str) {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        let started = std::time::Instant::now();
        let pss_before_kb = read_process_pss_kb().unwrap_or(0);
        // SAFETY: `malloc_trim(0)` 只对 malloc 空闲页 madvise 归还，无指针语义、
        // 不触碰活对象；多线程下 glibc 自带 arena 锁。
        let released = unsafe { libc::malloc_trim(0) };
        let pss_after_kb = read_process_pss_kb().unwrap_or(0);
        tracing::info!(
            target: "mem_reclaim",
            stage,
            released = released == 1,
            pss_before_kb,
            pss_after_kb,
            elapsed_us = started.elapsed().as_micros() as u64,
            "malloc_trim done"
        );
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        let _ = stage;
    }
}

/// T13 S7：进程 PSS（kB，`/proc/self/smaps_rollup`）。非 linux 返 `None`
/// （探针日志以 0 占位，判读时对照平台说明）。
pub(crate) fn read_process_pss_kb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").ok()?;
        let line = rollup.lines().find(|line| line.starts_with("Pss:"))?;
        line.split_whitespace().nth(1)?.parse().ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
