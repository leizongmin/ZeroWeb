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
/// 生效；缺省两个 env 都关，不落盘不采样）：glibc `malloc_info(3)` 落盘 per-arena
/// 统计——每 arena `system`（已提交）与 fast/normal/large/rest 各 bin 的 free 总量，
/// 用于区分「活分配」与「arena 碎片滞留」。路径 = `<前缀>-<单调序号>.xml`，序号
/// 进程内累计；写失败仅记 warn，不影响主流程。
///
/// 开销口径（如实）：门检查是每次调用一次 env 查表（`enabled_when_true` 走
/// `std::env::var`，无缓存），频率为导航级（settle/reclaim 臂每次导航 2-4 次）、
/// 单次 µs 级，开销可忽略；不引入进程内缓存，保留测试内 `set_var` 后立即读取生效
/// 的语义。
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
    // glibc 符号（musl 无 malloc_info；桌面 glibc 主目标，与 malloc_trim 同判）。
    // 序号与导入仅 gnu 分支引用，随分支走 cfg（非 gnu 目标零警告）。
    #[cfg(target_env = "gnu")]
    {
        use std::os::raw::{c_char, c_int};
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
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
    // 子进程隔离（TV-1/DF-1 返修）：行为钉不在本进程做进程级 smaps 窗口计数——并行
    // 套件下并发测试的 ~4MiB 分配污染窗口计数（HEAD 5 轮 2 红）、arena 大自由块 bin
    // 复用可吸收 4MiB 请求（base=0 during=0 假绿）、隔离单测进程（无前置大分配历史）
    // 对 mallopt 移除假绿（stock 128KB 阈值本就使 4MiB 走 mmap）。改为
    // `current_exe()` 重执行自身：子进程内串行执行固定序列，父进程只断言子进程退出
    // 码，与本进程其他并行测试零串扰；fresh 子进程无 arena 大自由块，红绿两侧确定。
    //
    // 变异灵敏度（双向确定）：子进程序列先 malloc 8MiB+free 把 glibc 动态阈值顶到
    // ≥8MiB（mallopt(3) M_MMAP_THRESHOLD 动态上调，上限 32MB），复现「动态上调已发
    // 生」前置态；此时若封顶步（apply_allocator_tuning）缺失，后续 4MiB < 已上调阈值
    // → arena/brk 承载 → 断言红——修复隔离假绿（TV-1 Break-A 隔离臂）。

    /// 父→子重执行门控 env。
    const PIN_CHILD_ENV: &str = "ZW_T38_PIN_CHILD";
    /// mmap 段 Size 下界（kB）：4MiB 请求 + glibc 块头进位 ≈ 4100kB；只设下界不设
    /// 上界——内核会把相邻匿名 VMA 合并（返修首轮全量套件实测：mmap 承载成立但窗口
    /// 计数 during=0），含 buf 的段 Size ≥ 下界即段形态成立，合并只增不减。
    const VMA_LO_KB: u64 = 4 * 1024 + 1;

    /// 返回包含 `addr` 的 VMA（`(Size kB, 命名)`；匿名段命名 = `None`）。
    /// 地址→VMA 包含判定对并行/合并串扰免疫（DF-1 建议），仅在子进程串行序列内调用。
    fn vma_containing(addr: usize) -> Option<(u64, Option<String>)> {
        let smaps = std::fs::read_to_string("/proc/self/smaps").expect("smaps 可读");
        let mut header: Option<(usize, usize)> = None;
        let mut size_kb = 0u64;
        let mut name: Option<String> = None;
        let mut found = None;
        for line in smaps.lines() {
            if line.starts_with(' ') {
                continue;
            }
            if let Some((lo, hi)) = line.split_once('-') {
                // 新 VMA 头：结算上一段
                if let Some((plo, phi)) = header.replace((
                    usize::from_str_radix(lo, 16).ok()?,
                    usize::from_str_radix(hi.split_whitespace().next()?, 16).ok()?,
                )) && plo <= addr
                    && addr < phi
                {
                    found = Some((size_kb, name.clone()));
                }
                name = line.split_whitespace().nth(5).map(str::to_string);
                size_kb = 0;
            } else if let Some(v) = line.strip_prefix("Size:") {
                size_kb = v.trim().trim_end_matches(" kB").parse().unwrap_or(0);
            }
        }
        if let Some((plo, phi)) = header
            && plo <= addr
            && addr < phi
        {
            found = Some((size_kb, name));
        }
        found
    }

    /// glibc 2.33+ `mallinfo2()` 的 hblkhd 字段——当前经 mmap(2) 承载的字节量
    /// （mallopt(3)），承载/归还判定的主信号（对 VMA 合并、arena bin 复用均免疫：
    /// arena/brk 承载不改变 hblkhd）。
    fn mmap_backed_bytes() -> Option<u64> {
        // SAFETY：mallinfo2 为 glibc 2.33+ 导出符号（gnu target_env 才编译本模块），
        // 只读分配器统计，无指针语义。
        let info = unsafe { libc::mallinfo2() };
        Some(info.hblkhd as u64)
    }

    /// T38/G4 Phase B 行为钉：mmap 阈值封顶必须在「动态阈值已被上调」的前置态下
    /// 可复现——封顶后 4MiB 分配走独立 mmap 段且 free 即 munmap 还 OS。本进程只
    /// 断言子进程退出码（子进程为自身重执行，见模块注释）。
    #[test]
    fn large_alloc_is_mmap_backed_and_released_on_free() {
        let exe = std::env::current_exe().expect("current_exe 可解析");
        let output = std::process::Command::new(exe)
            .arg("pin_child_sequence")
            .arg("--test-threads=1")
            .env(PIN_CHILD_ENV, "1")
            .output()
            .expect("子进程（自身重执行）可启动");
        assert!(
            output.status.success(),
            "子进程序列失败（status={:?}）\n--- 子进程 stdout ---\n{}\n--- 子进程 stderr ---\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    /// 子进程入口：仅在 [`PIN_CHILD_ENV`]`=1` 时执行固定序列（全量套件下 no-op 直接
    /// 绿，不递归 spawn）。串行序列：
    /// ① malloc 8MiB+写+free——stock 阈值下走 mmap，free 触发 glibc 动态阈值上调到
    ///   ≥8MiB，复现「动态上调已发生」前置态；
    /// ② `apply_allocator_tuning()`——封顶 128KB + 禁动态上调（本步缺失即 Break 面）；
    /// ③ malloc 4MiB——断言 mmap 承载：mallinfo2 hblkhd 增量 ≥4MiB（主信号）+ 含
    ///   buf 地址的 VMA 为 ≥[`VMA_LO_KB`] 的匿名段（段形态）；
    /// ④ free——断言归还：hblkhd 回落基线 + 原 VMA 消失或缩到下界以下。
    #[test]
    fn pin_child_sequence() {
        if std::env::var(PIN_CHILD_ENV).as_deref() != Ok("1") {
            return;
        }
        // ① 动态阈值上调前置态
        let trigger = vec![0xA5u8; 8 * 1024 * 1024];
        std::hint::black_box(&trigger);
        drop(trigger);
        // ② 封顶
        assert!(crate::mem_reclaim::apply_allocator_tuning(), "mallopt 必须生效");
        // ③ mmap 承载断言
        let base_hblkhd = mmap_backed_bytes();
        let buf = vec![0xA5u8; 4 * 1024 * 1024];
        std::hint::black_box(&buf);
        let during_hblkhd = mmap_backed_bytes();
        let buf_addr = buf.as_ptr() as usize;
        let during_vma = vma_containing(buf_addr);
        // ④ 归还断言
        drop(buf);
        let after_hblkhd = mmap_backed_bytes();
        let after_vma = vma_containing(buf_addr);
        if let (Some(base), Some(during), Some(after)) = (base_hblkhd, during_hblkhd, after_hblkhd) {
            assert!(
                during.saturating_sub(base) >= 4 * 1024 * 1024,
                "mallinfo2 hblkhd 增量必须 ≥4MiB（mmap 承载主信号，base={base} \
                 during={during}；增量不足说明 4MiB 被 arena/brk 承载——动态阈值未禁用 \
                 或被既有 arena 大自由块吸收）"
            );
            assert_eq!(
                after, base,
                "释放后 hblkhd 必须回落到基线（base={base} after={after}；未回落说明 \
                 mmap 段未 munmap 还 OS）"
            );
        }
        match during_vma {
            Some((size_kb, name)) => {
                assert!(
                    size_kb >= VMA_LO_KB,
                    "含 buf 地址的 VMA Size 必须 ≥{VMA_LO_KB}kB（实际 {size_kb}kB）"
                );
                assert!(
                    !matches!(&name, Some(n) if n.starts_with('[')),
                    "含 buf 地址的 VMA 必须是匿名段而非命名段（实际 {name:?}；[heap] \
                     即 arena/brk 承载）"
                );
            }
            None => panic!("必须存在包含 buf 地址（{buf_addr:#x}）的 VMA"),
        }
        match after_vma {
            None => {}
            Some((size_kb, _)) => assert!(
                size_kb < VMA_LO_KB,
                "释放后原地址所在 VMA 必须消失或缩到 {VMA_LO_KB}kB 以下（实际 {size_kb}kB；\
                 段残留说明未 munmap 还 OS）"
            ),
        }
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
