//! T13 S2/S7：导航落定分配器归还与内存探针（设计卡 §3 推荐方案 A）。
//!
//! `malloc_trim` 仅 linux-gnu glibc 提供（musl 无此符号、macOS/Windows 无对应
//! 单调用；桌面 glibc 主目标，设计卡 §8-7 披露），其余目标编译为 no-op。
//! T13 R1（返修）：两落定点改为统一延迟调度——reclaim 触发改 settle+D（主锚 =
//! renderer `LoadComplete` 逐导航渲染完成沿，loading 翻转沿闩为初载相补充，
//! 主循环 16ms tick 消费），S2/S3 回收成本移出首载测窗（M2a/M2b 判据窗，处置
//! 工单主控 M1）；延迟窗证据与命中语义预注册见 `bench/results/t13-rework-frozen.md`。
//!
//! T104 G2：主环 idle trim（主循环静默期 `malloc_trim(0)`）同形态复用——导航
//! trim 只覆盖 settle 窗，close/静默后新滞留（t104-memory-anatomy.md §三①）由
//! [`IdleTrimScheduler`] 在主环静默超 D 后归还。

use std::time::{Duration, Instant};

/// T13 R1：延迟回收窗 D——settle→首帧上界 429ms（1.52×）与末代 settle→C2 采样
/// 1000ms（1.54×）之间，双侧裕度 ≥1.4×（证据表见 t13-rework-frozen.md）。
pub(crate) const RECLAIM_DELAY: Duration = Duration::from_millis(650);

/// T13 S2：导航 trim 开关——`ZEROWEB_NAV_TRIM=0` 关（缺省开）。
pub(crate) fn nav_trim_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZEROWEB_NAV_TRIM")
}

/// T79 S9：glibc 分配器滞留收敛（启动单次调用，零热路径）。
/// gate+mallopt 实现 T79 迁至 zero-runtime-config（browser bin 同刀复用，
/// 劈分证据见该函数文档），此处保留 bin 侧 tracing 记录；关闭与非 glibc
/// 目标静默不调。bin 作用域经 lib.rs 单符号 re-export 调用。
pub fn tune_malloc_retention_policy() {
    if let Some((arena, mmap, trim)) = zero_runtime_config::tune_malloc_retention_policy() {
        tracing::info!(
            target: "mem_reclaim",
            arena_max_ok = arena,
            mmap_threshold_ok = mmap,
            trim_threshold_ok = trim,
            "malloc retention policy tuned"
        );
    }
}

/// T13 S7：内存探针——`ZEROWEB_MEM_PROBE=1` 开（缺省关；观测腿不进判据样本）。
pub(crate) fn mem_probe_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZEROWEB_MEM_PROBE")
}

/// T104 G2：idle trim 开关——`ZEROWEB_IDLE_TRIM=0` 关（缺省开；产品刀，
/// t104-memory-anatomy.md §四单刀位推荐）。
pub(crate) fn idle_trim_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZEROWEB_IDLE_TRIM")
}

/// T109 G2：V8 主动压还开关——`ZEROWEB_V8_PRESSURE_RELEASE=1` 开（缺省关；
/// feature-gate 实验刀，T104 IDLE_TRIM 门制同构）。
pub(crate) fn v8_pressure_release_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZEROWEB_V8_PRESSURE_RELEASE")
}

/// T104 G2：idle trim 静默窗 D——主环连续无活动（消息/busy 待办/媒体/rAF/脚本执行
/// recency）达 D 方触发（防活跃期误 trim）。DF-R1 勘定：busy 脚本活跃判据是 recency
/// 窗读（脚本执行臂 acquire/Drop 双点刷新最近执行时刻，见
/// [`crate::js_worker::script_activity_stamp_within`]），非采样瞬时旗标——任意占空比
/// 的周期脚本（含 1s 级稀疏定时器）均在窗内确定性锚定。静默期按同周期重触（close
/// 后新滞留分批到账，baidu 实测 close 后 ~10s 仍在爬升，见 t104-memory-anatomy.md
/// §三①）。D 取 5s：显著长于帧间/脚本休眠静默，短于 30s 判据持位窗保证窗内生效。
pub(crate) const IDLE_TRIM_DELAY: Duration = Duration::from_secs(5);

/// T104 G2：主环 idle trim 调度器（renderer 主线程单槽，T13
/// [`DeferredReclaimScheduler`] defer＋到期触发同形态——活动沿重锚、tick 消费）。
pub(crate) struct IdleTrimScheduler {
    enabled: bool,
    last_activity: Instant,
    last_fire: Option<Instant>,
}

impl IdleTrimScheduler {
    /// 构造（gate 构造期读取一次；起始时刻视作活动沿——启动相不 trim）。
    pub(crate) fn new() -> Self {
        Self {
            enabled: idle_trim_enabled(),
            last_activity: Instant::now(),
            last_fire: None,
        }
    }

    /// 活动沿（消息到环/busy 待办/媒体播放/rAF 挂起）：重锚静默起点。
    pub(crate) fn note_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// 主环每迭代调用：静默 ≥[`IDLE_TRIM_DELAY`] 且距上次触发 ≥同周期 →
    /// `malloc_trim(0)`（stage=`idle_trim`，gens=0——非文档代数语义）。
    /// 返回是否触发（测试观测面）。
    pub(crate) fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        let now = Instant::now();
        if now.duration_since(self.last_activity) < IDLE_TRIM_DELAY {
            return false;
        }
        if self.last_fire.is_some_and(|t| now.duration_since(t) < IDLE_TRIM_DELAY) {
            return false;
        }
        self.last_fire = Some(now);
        malloc_trim_now("idle_trim", 0);
        true
    }
}

/// T109 G2：V8 压还 idle 沿调度器（renderer 主线程单槽，T104 [`IdleTrimScheduler`]
/// 同构——活动沿重锚、静默 ≥[`IDLE_TRIM_DELAY`] 触发、静默期按同周期重触）。与
/// idle trim 共用主环 busy 判定与静默窗；触发面非 `malloc_trim` 而是「向 js worker
/// 投递 [`crate::js_worker::JsWorkerCommand::V8PressureRelease`]」（isolate 线程命令
/// 边界执行全堆 GC）。gate 关闭恒不触发＝无命令投递，主行为与基线逐分支恒等。
pub(crate) struct V8PressureReleaseScheduler {
    enabled: bool,
    last_activity: Instant,
    last_fire: Option<Instant>,
}

impl V8PressureReleaseScheduler {
    /// 构造（gate 构造期读取一次；起始时刻视作活动沿——启动相不触发）。
    pub(crate) fn new() -> Self {
        Self {
            enabled: v8_pressure_release_enabled(),
            last_activity: Instant::now(),
            last_fire: None,
        }
    }

    /// 活动沿：重锚静默起点（同 busy 判定与 idle trim 共用）。
    pub(crate) fn note_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// 主环每迭代调用：静默 ≥[`IDLE_TRIM_DELAY`] 且距上次触发 ≥同周期 → 返回
    /// true（主环据此投递压还命令）。gate 关闭恒 false。
    pub(crate) fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        let now = Instant::now();
        if now.duration_since(self.last_activity) < IDLE_TRIM_DELAY {
            return false;
        }
        if self.last_fire.is_some_and(|t| now.duration_since(t) < IDLE_TRIM_DELAY) {
            return false;
        }
        self.last_fire = Some(now);
        true
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

#[cfg(test)]
mod tests {
    use super::*;

    /// T79 S9：调参幂等可重入且 gate 关闭时零 mallopt（早退路径）。
    #[test]
    fn malloc_tune_is_idempotent_and_gate_gated() {
        // 缺省开启：gnu 下两次调用不 panic（mallopt 可重入）；非 glibc 零操作。
        crate::mem_reclaim::tune_malloc_retention_policy();
        crate::mem_reclaim::tune_malloc_retention_policy();
    }

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

    /// T104 G2 idle trim 调度门钉：D 内静默不足不触发；静默超 D 触发并按 D 周期
    /// 重触；活动沿重锚静默窗；gate 关闭恒不触发（绕过延迟的同步触发必红）。
    /// 互斥同 TV-F4（tick 走 `malloc_trim_now` 递增进程级计数，防并行串扰）。
    #[test]
    fn idle_trim_scheduler_fires_after_idle_and_reanchors_on_activity() {
        let _guard = test_trim_lock();
        let mut sched = IdleTrimScheduler::new();
        sched.enabled = false;
        assert!(!sched.tick(), "gate 关闭不得触发（含静默超 D 态）");
        sched.last_activity -= IDLE_TRIM_DELAY + Duration::from_millis(1);
        assert!(!sched.tick(), "gate 关闭且静默超 D 仍不得触发");
        sched.enabled = true;
        sched.last_activity = Instant::now();
        assert!(!sched.tick(), "静默不足 D 不得触发");
        sched.last_activity -= IDLE_TRIM_DELAY + Duration::from_millis(1);
        assert!(sched.tick(), "静默超 D 触发");
        assert!(!sched.tick(), "重触周期（D）内不得重复触发");
        sched.last_fire = Some(sched.last_fire.unwrap() - IDLE_TRIM_DELAY);
        assert!(sched.tick(), "静默期按 D 周期重触（close 后滞留分批到账）");
        sched.note_activity();
        assert!(!sched.tick(), "活动沿重锚后静默窗重新计满方触发");
    }

    /// T109 G2 V8 压还调度门钉：gate 关闭恒不触发（含静默超 D 态）；静默不足 D 不
    /// 触发；静默超 D 触发一次并按 D 周期重触；活动沿重锚后静默窗重新计满方触发。
    ///（与 idle trim 调度门钉同构；tick 不触 malloc_trim/进程级计数，无需互斥。）
    #[test]
    fn v8_pressure_release_scheduler_fires_after_idle_and_reanchors() {
        let mut sched = V8PressureReleaseScheduler::new();
        sched.enabled = false;
        assert!(!sched.tick(), "gate 关闭不得触发（含静默超 D 态）");
        sched.last_activity -= IDLE_TRIM_DELAY + Duration::from_millis(1);
        assert!(!sched.tick(), "gate 关闭且静默超 D 仍不得触发");
        sched.enabled = true;
        sched.last_activity = Instant::now();
        assert!(!sched.tick(), "静默不足 D 不得触发");
        sched.last_activity -= IDLE_TRIM_DELAY + Duration::from_millis(1);
        assert!(sched.tick(), "静默超 D 触发（主环投递压还命令）");
        assert!(!sched.tick(), "重触周期（D）内不得重复触发");
        sched.last_fire = Some(sched.last_fire.unwrap() - IDLE_TRIM_DELAY);
        assert!(sched.tick(), "静默期按 D 周期重触");
        sched.note_activity();
        assert!(!sched.tick(), "活动沿重锚后静默窗重新计满方触发");
    }
}
