//! T13 S2/S7：导航落定分配器归还与内存探针（设计卡 §3 推荐方案 A）。
//!
//! `malloc_trim` 仅 linux-gnu glibc 提供（musl 无此符号、macOS/Windows 无对应
//! 单调用；桌面 glibc 主目标，设计卡 §8-7 披露），其余目标编译为 no-op。
//! T13 R1（返修）：两落定点改为统一延迟调度——reclaim 触发改 settle+D（主锚 =
//! renderer `LoadComplete` 逐导航渲染完成沿，loading 翻转沿闩为初载相补充，
//! 主循环 16ms tick 消费），S2/S3 回收成本移出首载测窗（M2a/M2b 判据窗，处置
//! 工单主控 M1）；延迟窗证据与命中语义预注册见 `bench/results/t13-rework-frozen.md`。

use std::time::{Duration, Instant};

/// T13 R1：延迟回收窗 D——settle→首帧上界 429ms（1.52×）与末代 settle→C2 采样
/// 1000ms（1.54×）之间，双侧裕度 ≥1.4×（证据表见 t13-rework-frozen.md）。
pub(crate) const RECLAIM_DELAY: Duration = Duration::from_millis(650);

/// T13 S2：导航 trim 开关——`ZEROWEB_NAV_TRIM=0` 关（缺省开）。
pub(crate) fn nav_trim_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZEROWEB_NAV_TRIM")
}

/// T13 S7：内存探针——`ZEROWEB_MEM_PROBE=1` 开（缺省关；观测腿不进判据样本）。
pub(crate) fn mem_probe_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZEROWEB_MEM_PROBE")
}

/// T38/G4 归因仪器（`ZEROWEB_MEM_PROBE=1` 且 `ZEROWEB_MEM_PROBE_XML` 设前缀路径时
/// 生效；缺省两个 env 都关，零调用零开销）：glibc `malloc_info(3)` 落盘 per-arena
/// 统计——每 arena `system`（已提交）与 fast/normal/large/rest 各 bin 的 free 总量，
/// 用于区分「活分配」与「arena 碎片滞留」。路径 = `<前缀>-<单调序号>.xml`，序号
/// 进程内累计；写失败仅记 warn，不影响主流程。
pub(crate) fn malloc_info_dump_if_enabled() {
    if !mem_probe_enabled() {
        return;
    }
    if let Ok(prefix) = std::env::var("ZEROWEB_MEM_PROBE_XML")
        && !prefix.is_empty()
    {
        malloc_info_dump_to(&prefix);
    }
}

/// [`malloc_info_dump_if_enabled`] 的核心（env 无关，便于单测）：glibc
/// `malloc_info(3)` 写 `<前缀>-<单调序号>.xml`，返回落盘路径；非 glibc 目标或
/// fopen/malloc_info 失败返 `None`（仅 warn，不影响主流程）。
pub(crate) fn malloc_info_dump_to(prefix: &str) -> Option<String> {
    use std::os::raw::{c_char, c_int};
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    // glibc 符号（musl 无 malloc_info；桌面 glibc 主目标，与 malloc_trim 同判）。
    #[cfg(target_env = "gnu")]
    {
        unsafe extern "C" {
            fn fopen(path: *const c_char, mode: *const c_char) -> *mut libc::c_void;
            fn malloc_info(options: c_int, stream: *mut libc::c_void) -> c_int;
            fn fclose(stream: *mut libc::c_void) -> c_int;
        }
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = format!("{prefix}-{n}.xml");
        let Ok(c_path) = std::ffi::CString::new(path.clone()) else {
            return None;
        };
        let c_mode = std::ffi::CString::new("w").expect("mode 常量");
        // SAFETY：fopen/malloc_info/fclose 均为 glibc 导出符号；FILE* 以不透明指针
        // 传递，路径来自本次调用的 CString（调用期间存活）。
        unsafe {
            let fp = fopen(c_path.as_ptr(), c_mode.as_ptr());
            if fp.is_null() {
                tracing::warn!(path = %path, "mem_probe malloc_info fopen failed");
                return None;
            }
            let rc = malloc_info(0, fp);
            let _ = fclose(fp);
            if rc != 0 {
                tracing::warn!(path = %path, rc = rc as i64, "mem_probe malloc_info failed");
                return None;
            }
        }
        Some(path)
    }
    #[cfg(not(target_env = "gnu"))]
    {
        let _ = prefix;
        None
    }
}

#[cfg(all(test, target_env = "gnu"))]
mod mem_probe_malloc_info_tests {
    /// T38/G4 仪器钉：malloc_info 落盘必须产出 `<malloc` 开头的 XML（glibc 格式），
    /// 且包含 per-arena `<heap nr=` 与 `<system type="current"` 统计——归因面
    /// （活分配 vs arena 碎片）依赖这两个字段存在。
    #[test]
    fn malloc_info_dump_writes_per_arena_xml() {
        let dir = std::env::temp_dir().join(format!("zw-t38-malloc-info-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir 创建");
        let prefix = dir.join("probe").display().to_string();
        let path = crate::mem_reclaim::malloc_info_dump_to(&prefix).expect("glibc malloc_info 落盘");
        let txt = std::fs::read_to_string(&path).expect("落盘文件可读");
        assert!(
            txt.starts_with("<malloc"),
            "malloc_info XML 头，实际 {}",
            &txt[..40.min(txt.len())]
        );
        assert!(txt.contains("<heap nr="), "必须有 per-arena heap 段");
        assert!(txt.contains("type=\"current\""), "必须有 system current 统计");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// T13 R1：延迟回收调度器（renderer 主线程单槽）。
///
/// webview loading 翻转沿经 [`Self::arm`] 重锚 deadline（单槽天然合并相邻导航）；
/// `reclaim_after_document_reset` 每次 [`Self::note_reset`] 累计待回收代数；
/// 主循环 tick 经 [`Self::due`] 到点取走（gens 归零）。进程提前退出则 pending
/// 丢弃（M2 b 侧多此形态，无内存判据，预注册披露）。
pub(crate) struct DeferredReclaimScheduler {
    deadline: Option<Instant>,
    pending_gens: u32,
    delay: Duration,
}

impl Default for DeferredReclaimScheduler {
    fn default() -> Self {
        Self {
            deadline: None,
            pending_gens: 0,
            delay: RECLAIM_DELAY,
        }
    }
}

impl DeferredReclaimScheduler {
    /// loading 翻转沿（任一方向）：重锚 deadline（合并语义——后到沿覆盖未触发的旧 deadline）。
    pub(crate) fn arm(&mut self, now: Instant) {
        self.deadline = Some(now + self.delay);
    }

    /// 文档复位（reclaim_after_document_reset）：待回收代数 +1。
    pub(crate) fn note_reset(&mut self) {
        self.pending_gens += 1;
    }

    /// 主循环 tick：到点返回并清空（gens 一并取走）；未到点返回 `None`。
    pub(crate) fn due(&mut self, now: Instant) -> Option<u32> {
        let deadline = self.deadline.take()?;
        if now < deadline {
            self.deadline = Some(deadline);
            return None;
        }
        Some(std::mem::take(&mut self.pending_gens))
    }

    /// 待回收代数（测试观测面）。
    #[cfg(test)]
    pub(crate) fn pending_gens(&self) -> u32 {
        self.pending_gens
    }

    /// deadline 是否已布防（测试观测面）。
    #[cfg(test)]
    pub(crate) fn armed(&self) -> bool {
        self.deadline.is_some()
    }
}

/// T13 S2：`malloc_trim(0)` 归还 glibc arena free 页（延迟到点触发调用）。
///
/// `stage` 进 tracing 字段（`worker_trim`/`nav_trim`）；`gens` 为本次 fire 覆盖的
/// 文档代数（合并 fire >1，预注册命中语义按 Σgens ≥ reset_count 判定）。
/// 探针腿附带 trim 前后进程 PSS 采样（`pss_before_kb`/`pss_after_kb`，read 失败
/// 记 0）——trim 净让量 = 前后差值；T13 R4 DEF-F2：PSS 双读仅在
/// `ZEROWEB_MEM_PROBE=1` 时执行（观测腿，不随常开 trim 每次付 /proc 读开销）。
pub(crate) fn malloc_trim_now(stage: &'static str, gens: u32) {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        let started = std::time::Instant::now();
        let probe = mem_probe_enabled();
        let pss_before_kb = if probe { read_process_pss_kb().unwrap_or(0) } else { 0 };
        // SAFETY: `malloc_trim(0)` 只对 malloc 空闲页 madvise 归还，无指针语义、
        // 不触碰活对象；多线程下 glibc 自带 arena 锁。
        let released = unsafe { libc::malloc_trim(0) };
        let pss_after_kb = if probe { read_process_pss_kb().unwrap_or(0) } else { 0 };
        tracing::info!(
            target: "mem_reclaim",
            stage,
            gens = gens as u64,
            released = released == 1,
            pss_before_kb,
            pss_after_kb,
            elapsed_us = started.elapsed().as_micros() as u64,
            "malloc_trim done"
        );
        #[cfg(test)]
        {
            TEST_TRIM_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        let _ = (stage, gens);
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

/// T13 R1 TV-F4 测试面：trim 触发计数与测试互斥（js_worker 钉用；仅测试构建编译）。
#[cfg(test)]
pub(crate) static TEST_TRIM_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// T13 R1 TV-F4 测试面：trim 计数当前值。
#[cfg(test)]
pub(crate) fn test_trim_count() -> usize {
    TEST_TRIM_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}

/// T13 R1 TV-F4 测试面：trim 相关测试互斥（共享进程级计数，防并行串扰）。
#[cfg(test)]
pub(crate) fn test_trim_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// T38/G4 Phase B：mmap 阈值封顶（linux-gnu glibc 专属，其余平台 no-op 返 false）。
///
/// glibc 缺省 mmap 阈值 128KB，但 free 一块 mmap 段后阈值动态上调（上限 32MB）：
/// 几次大块 free 之后，所有 <32MB 的分配改走 per-thread arena——图片解码缓冲、
/// 网络响应体、JS ArrayBuffer 底存等多 MB 级瞬态分配在 arena 内碎裂成洞，
/// `malloc_trim`（只缩各 arena 顶）无法归还（T38 Phase A 定谳：settle 时 91 个
/// arena 滞留 396MiB 自由页，其中 15+ 个 arena ≥98% 空闲仍不还 OS）。
///
/// `mallopt(M_MMAP_THRESHOLD, 128KB)` 恢复 stock 缺省并禁用动态上调：≥128KB
/// 分配改走 mmap，free 即 munmap 还 OS。纯分配器调参，执行语义零变化。
/// 必须在进程早期（大块分配/free 循环开始前）调用。
///
/// 返回 linux-gnu 是否生效；musl/macOS/Windows 返 `false`（函数内已记日志，调用方可忽略）。
pub fn apply_allocator_tuning() -> bool {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        /// glibc 缺省 mmap 阈值（恢复 stock 面而非新限制）。
        const MMAP_THRESHOLD_BYTES: libc::c_int = 128 * 1024;
        // SAFETY: `mallopt` 只改写 glibc 分配器参数（数值常量，无指针语义），
        // 线程安全由 glibc 内部锁保证；文档要求在并发分配前调用最佳，晚调仍生效。
        let applied = unsafe { libc::mallopt(libc::M_MMAP_THRESHOLD, MMAP_THRESHOLD_BYTES) };
        tracing::info!(
            target: "mem_reclaim",
            applied = applied == 1,
            mmap_threshold_bytes = MMAP_THRESHOLD_BYTES as u64,
            "allocator tuning: mmap threshold capped (T38 G4)"
        );
        applied == 1
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        tracing::info!(target: "mem_reclaim", "allocator tuning: not available on this platform (T38 G4)");
        false
    }
}

#[cfg(all(test, target_os = "linux", target_env = "gnu"))]
mod allocator_tuning_tests {
    /// 统计 /proc/self/smaps 中 Size 落在 `[lo_kb, hi_kb]` 窗口的 VMA 数。
    fn vmas_in_range(lo_kb: u64, hi_kb: u64) -> u32 {
        let smaps = std::fs::read_to_string("/proc/self/smaps").expect("smaps 可读");
        let mut count = 0u32;
        let mut cur_size = 0u64;
        for line in smaps.lines() {
            if let Some(rest) = line.strip_suffix(" kB") {
                if let Some(v) = rest.strip_prefix("Size:") {
                    cur_size = v.trim().parse().unwrap_or(0);
                }
            } else if line.contains('-') && !line.starts_with(' ') {
                // 新 VMA 头：结算上一段
                if (lo_kb..=hi_kb).contains(&cur_size) {
                    count += 1;
                }
                cur_size = 0;
            }
        }
        if (lo_kb..=hi_kb).contains(&cur_size) {
            count += 1;
        }
        count
    }

    /// T38/G4 Phase B 行为钉：mmap 阈值封顶后，≥128KB 分配必须走独立 mmap 段
    /// （free 即 munmap 还 OS）。4MiB 请求的 mmap 段 = 4MiB + glibc 块头进位
    /// ≈ 4100kB，窗口上界容许页宽/实现出入；若回归为 arena 承载（动态阈值
    /// 未禁用），分配表现为既有 arena VMA 增长，不出现该窗口内的独立段，
    /// 或释放后段残留。
    #[test]
    fn large_alloc_is_mmap_backed_and_released_on_free() {
        assert!(crate::mem_reclaim::apply_allocator_tuning(), "mallopt 必须生效");
        let (lo_kb, hi_kb) = (4 * 1024 + 1, 4 * 1024 + 128);
        let base = vmas_in_range(lo_kb, hi_kb);
        let buf = vec![0xA5u8; 4 * 1024 * 1024];
        std::hint::black_box(&buf);
        let during = vmas_in_range(lo_kb, hi_kb);
        drop(buf);
        let after = vmas_in_range(lo_kb, hi_kb);
        assert!(
            during > base,
            "分配期间必须出现 ~4MiB mmap VMA（base={base} during={during}）"
        );
        assert_eq!(after, base, "释放后 mmap VMA 必须归还 OS（base={base} after={after}）");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T13 R1 TV-F4 调度门钉（1/3）：布防后 D 内不触发——绕过延迟（同步触发）必红。
    #[test]
    fn scheduler_does_not_fire_before_delay() {
        let mut sched = DeferredReclaimScheduler::default();
        let t0 = Instant::now();
        assert!(!sched.armed());
        sched.arm(t0);
        sched.note_reset();
        assert!(sched.armed());
        assert_eq!(sched.pending_gens(), 1);
        assert!(sched.due(t0).is_none(), "D 内到点必须为 None（同步触发即违反延迟调度）");
        assert!(sched.armed(), "未到点不得消费 deadline");
        assert_eq!(sched.pending_gens(), 1, "未到点不得取走 gens");
    }

    /// T13 R1 TV-F4 调度门钉（2/3）：D 后到点触发一次，gens 全额取走并归零。
    #[test]
    fn scheduler_fires_once_after_delay_with_full_gens() {
        let mut sched = DeferredReclaimScheduler::default();
        let t0 = Instant::now();
        sched.arm(t0);
        sched.note_reset();
        sched.note_reset();
        let fired_at = t0 + RECLAIM_DELAY + Duration::from_millis(1);
        assert_eq!(sched.due(fired_at), Some(2), "到点须返回全部待回收代数");
        assert!(!sched.armed());
        assert_eq!(sched.pending_gens(), 0);
        assert_eq!(sched.due(fired_at), None, "二次 due 不得重复触发");
    }

    /// T13 R1 TV-F4 调度门钉（3/3）：翻转沿重锚（合并语义）——旧 deadline 被后到沿
    /// 覆盖，窗口顺延；旧窗口内不触发。
    #[test]
    fn scheduler_rearm_on_later_flip_extends_deadline() {
        let mut sched = DeferredReclaimScheduler::default();
        let t0 = Instant::now();
        sched.arm(t0);
        let t1 = t0 + Duration::from_millis(400);
        sched.arm(t1);
        assert!(sched.due(t0 + RECLAIM_DELAY + Duration::from_millis(1)).is_none());
        assert!(sched.armed(), "重锚后须按新 deadline 继续布防");
        assert_eq!(
            sched.due(t1 + RECLAIM_DELAY + Duration::from_millis(1)),
            Some(0),
            "新 deadline 到点触发（工厂态文档 gens=0 合法）"
        );
    }
}
