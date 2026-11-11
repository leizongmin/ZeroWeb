//! T62/G4 先导分配仪器：renderer 全堆分配按「分配点」聚合的观测层。
//!
//! 目标是为 G4（②→③ 交互增长，T35/T37 记账的 renderer anon ~850 MiB 归因缺口）
//! 提供 malloc 路径的 site 级归因前置件：环形（有界）聚合 + checkpoint dump
//! Top-N。本模块**只观测不改变分配行为**——所有真实分配仍走 [`System`]。
//!
//! ## 开关与零开销边界（T62 任务卡判据：默认关、零开销）
//!
//! 三层状态，逐层披露：
//!
//! 1. **默认构建**（未开 `alloc-trace` feature）：本模块整体不编译，产品二进制
//!    零新增代码、零数据、分配路径不变。
//! 2. **feature 构建且未启用**（`ZW_ALLOC_TRACE` ≠ `1`）：每次分配/释放只多一次
//!    `AtomicBool` Relaxed 读后直通 [`System`]；进程快照在 `init_from_env`
//!    一次性读取（进程级开关构造期读取范式，learnings
//!    `2026-08-17-snapshot-process-lifetime-paint-flags`），热路径无 getenv。
//! 3. **feature 构建且启用**：记录路径仅追加固有 [`System`] 调用之后的簿记
//!    （采样回溯 + 有界表更新），不改变分配结果、大小或布局。仪器自身内存
//!    有界：site 表 ≤ `max_sites` 条（丢弃计数封口，紧凑记录 ≲10MiB）、
//!    net 表 ≤ `max_live_entries` 条（≲3MiB），合计最坏 ~12MiB，dump
//!    totals 中如实申报。
//!
//! ## 环境变量（`init_from_env` 进程快照，一次性）
//!
//! - `ZW_ALLOC_TRACE=1`：启用记录（默认关）。
//! - `ZW_ALLOC_TRACE_SAMPLE_EVERY`：每 N 次分配采样一次回溯（默认 256）。
//! - `ZW_ALLOC_TRACE_MAX_SITES`：site 聚合表上限（默认 65536；满后新 site 计入
//!   `dropped_sites`，不驱逐——保 Top-N 稳定且表有界）。紧凑 site 记录
//!   （原始 IP 窗数组，dump 时渲染偏移）使 65536 条 ≲10MiB、live 表 ≲3MiB，
//!   合计最坏 ~12MiB，dump totals 如实申报；T62 attempt3 实证 2048 表在
//!   启动期即被 V8/JIT 唯一调用点灌满（dropped 53.6 万），故取 65536。
//! - `ZW_ALLOC_TRACE_NET_MIN`：进入 net（存活）表的块大小下限字节（默认 64 KiB；
//!   覆盖 glibc mmap 阈值之上、arena/位图大块面，net 出「增长」而非「流量」）。
//! - `ZW_ALLOC_TRACE_MAX_LIVE`：net 表条目上限（默认 65536；满后新大块计入
//!   `live_untracked`）。
//! - `ZW_ALLOC_TRACE_DUMP_DIR`：dump 目录；设置后启动 dump 线程。
//! - `ZW_ALLOC_TRACE_DUMP_PERIOD_SECS`：周期 dump 间隔秒（默认 10）。
//!
//! dump 触发：周期 + 触发文件 `<dump_dir>/trigger`（首行为 reason，读后即删；
//! 外部测量驱动在检查点写入，触发延迟语义 = 1s 轮询粒度）。
//!
//! ## 回溯采集与离线符号解析
//!
//! 采样走 `backtrace` crate 原始 IP 展开（`trace` 帧回调，不符号化，微秒级）；
//! site 签名 = 机器帧之后固定窗口原始 IP 的哈希（进程内稳定）。摘要与 dump 落
//! **模块相对偏移**（运行时 IP − 主映像 PIE 加载基址，`/proc/self/maps` 一次性
//! 解析），符号名由测量侧离线 `addr2line`/`llvm-symbolizer` 批量解析。不走
//! std `Backtrace` 稳定面：其 Debug 格式化在格式化时对全部帧符号化，fat 二进制
//! （v8 + debuginfo）下病态开销——T62 libtest 实测挂起超 60s（运行目录
//! evidence/t62-libtest-c1-stddbg-hang.log）。
//!
//! ## 与引擎面的边界
//!
//! 只覆盖 malloc 路径（含 glibc 大块 mmap）。V8 cage/页分配器直接 mmap 的
//! 保留区不经 [`System`]，本仪器不捕获——该面由测量协议的 smaps 差分覆盖
//! （T37 Phase A 先例）；JS 堆在用量已有 `ZEROWEB_MEM_PROBE=1` 的
//! `v8_heap_bytes` 打点（js_worker reset 臂），仪器不重复建设。

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// feature 构建下进程级启用位（`init_from_env` 置位后不再翻转）。
///
/// 分配热路径唯一额外成本：一次 Relaxed load。初始化次序保证见
/// [`init_with_config`]（CONFIG 先行，ENABLED 后行）。
static ENABLED: AtomicBool = AtomicBool::new(false);

/// 进程快照配置（`init_from_env` 一次性读取，此后只读）。
static CONFIG: OnceLock<TraceConfig> = OnceLock::new();

/// 分配调用序号（采样计数器；与状态锁分离，采样判定无锁）。
static SEQ: AtomicU64 = AtomicU64::new(0);

/// dump 落盘序号（进程内单调递增）：文件名去重成分，同毫秒同 reason 的
/// 触发不再碰撞覆写（T62 DF F3）。
static DUMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// 聚合状态（site 表 + net 表）；懒初始化，锁内首次使用时构造。
static STATE: Mutex<Option<TraceState>> = Mutex::new(None);

// 重入守卫（thread_local）：记录路径自身的分配（回溯捕获、格式化、表扩容）
// 直通 [`System`]，防止无限递归。const 初始化不分配。
thread_local! {
    static RECORDING: Cell<bool> = const { Cell::new(false) };
    #[cfg(test)]
    static TEST_MAX_SITES: Cell<Option<usize>> = const { Cell::new(None) };
}

/// 回溯捕获窗口上限（帧）。
const MAX_CAPTURE_FRAMES: usize = 12;

/// 哈希窗口跳过的机器帧数（展开器内部 + 本仪器记录路径 + 分配器包装链）。
/// 经验常数，测量轮 dump 产物核验；完整原始窗口始终随摘要落盘，常数标定
/// 偏差只降归属粒度（帧漂移进签名），不损聚合正确性。
const SKIP_FRAMES: usize = 3;

/// site 签名取跳过机器帧之后的原始 IP 帧数。
const SIGNATURE_FRAMES: usize = 6;

/// dump 输出的 site 条目数（traffic 与 live 各取 Top-N）。
const DUMP_TOP_N: usize = 16;

/// 仪器配置（进程快照，`init_from_env` 构造；测试经注入形给定）。
#[derive(Clone, Debug)]
pub(crate) struct TraceConfig {
    /// 每 N 次分配采样一次回溯。
    pub(crate) sample_every: usize,
    /// site 聚合表上限；满后新 site 丢弃计数。
    pub(crate) max_sites: usize,
    /// 进入 net（存活）表的块大小下限（字节）。
    pub(crate) net_min_bytes: usize,
    /// net 表条目上限；满后新大块丢弃计数。
    pub(crate) max_live_entries: usize,
    /// dump 目录；None 时 dump 仅可显式调用且必须给路径。
    pub(crate) dump_dir: Option<PathBuf>,
    /// dump 线程周期 dump 间隔。
    pub(crate) dump_period: Duration,
}

/// 单 site 的流量聚合（采样口径，外推系数 = `sample_every`）。
///
/// 帧窗存原始 IP 数组（紧凑定长 96B）而非字符串摘要：V8/JIT 运行期的唯一
/// 调用点数量级在数十万，紧凑记录把表容量从 ~2Ki 条提到 64Ki 条仍 ≲10MiB
/// （T62 attempt3 实证 2048 表启动期即满，dropped_sites 53.6 万——页面期
/// 新 site 全丢会让 traffic 面偏 startup）。偏移摘要在 dump 时按 exe 基址
/// 现渲染，符号仍由分析侧离线解析。
#[derive(Clone, Debug)]
struct SiteStats {
    /// 采到的分配次数。
    allocs: u64,
    /// 采到的分配字节合计。
    bytes: u64,
    /// 首采时完整原始 IP 窗（≤[`MAX_CAPTURE_FRAMES`] 帧；不足部分为 0）。
    frames: [usize; MAX_CAPTURE_FRAMES],
    /// 首采时实际捕获帧数。
    captured: usize,
}

/// net 表条目（按指针跟踪的大块存活记录）。
#[derive(Clone, Copy, Debug)]
struct LiveEntry {
    /// 归属 site 哈希；0 = 未归属（`unattributed`）。
    site: u64,
    /// 块大小（字节）。
    size: usize,
}

/// 按 site 的存活净量聚合。
#[derive(Clone, Copy, Debug, Default)]
struct LiveAgg {
    /// 存活块数。
    blocks: u64,
    /// 存活字节合计。
    bytes: u64,
}

/// 聚合状态（全部有界；预算上限见模块文档）。
#[derive(Debug, Default)]
struct TraceState {
    /// 采样到的分配次数。
    sampled_allocs: u64,
    /// 采样到的分配字节。
    sampled_bytes: u64,
    /// site 表满后被丢弃的新 site 数。
    dropped_sites: u64,
    /// site 流量表：hash → 聚合。
    sites: HashMap<u64, SiteStats>,
    /// net 表：指针 → 条目。
    live: HashMap<usize, LiveEntry>,
    /// net 表满后被丢弃的大块数。
    live_untracked: u64,
    /// 按 site 的存活净量（track/untrack 增量维护）。
    live_sites: HashMap<u64, LiveAgg>,
}

impl TraceState {
    fn new() -> Self {
        Self {
            sites: HashMap::new(),
            live: HashMap::new(),
            live_sites: HashMap::new(),
            sampled_allocs: 0,
            sampled_bytes: 0,
            dropped_sites: 0,
            live_untracked: 0,
        }
    }

    /// site 表上限（测试可注入覆盖；生产读 CONFIG，未初始化视为无界——
    /// 上限缺位时仍受真实站点数自然封口，dump 如实申报）。
    fn max_sites_limit(&self) -> usize {
        #[cfg(test)]
        {
            TEST_MAX_SITES
                .with(|m| m.get())
                .unwrap_or_else(|| CONFIG.get().map(|c| c.max_sites).unwrap_or(usize::MAX))
        }
        #[cfg(not(test))]
        {
            CONFIG.get().map(|c| c.max_sites).unwrap_or(usize::MAX)
        }
    }

    /// 记录一次采样到的分配（流量 + site 表）。
    fn record_sample(&mut self, size: usize, hash: u64, frames: [usize; MAX_CAPTURE_FRAMES], captured: usize) {
        self.sampled_allocs += 1;
        self.sampled_bytes += size as u64;
        match self.sites.get_mut(&hash) {
            Some(stats) => {
                stats.allocs += 1;
                stats.bytes += size as u64;
            }
            None => {
                if self.sites.len() < self.max_sites_limit() {
                    self.sites.insert(hash, SiteStats {
                        allocs: 1,
                        bytes: size as u64,
                        frames,
                        captured,
                    });
                } else {
                    self.dropped_sites += 1;
                }
            }
        }
    }

    /// net 表 track（分配侧）；表满丢弃计数。
    fn track_live(&mut self, ptr: usize, size: usize, site: u64, max_entries: usize) {
        if self.live.len() >= max_entries {
            self.live_untracked += 1;
            return;
        }
        if self.live.insert(ptr, LiveEntry { site, size }).is_none() {
            let agg = self.live_sites.entry(site).or_default();
            agg.blocks += 1;
            agg.bytes += size as u64;
        }
    }

    /// net 表 untrack（释放侧）；未知指针（小块或溢出块）静默略过。
    fn untrack_live(&mut self, ptr: usize) {
        let _ = self.take_live(ptr);
    }

    /// 摘下 live 条目（realloc 起始或显式 untrack）；指针仍唯一持有者时调用。
    fn take_live(&mut self, ptr: usize) -> Option<LiveEntry> {
        if let Some(entry) = self.live.remove(&ptr) {
            let mut remove_agg = false;
            if let Some(agg) = self.live_sites.get_mut(&entry.site) {
                agg.blocks = agg.blocks.saturating_sub(1);
                agg.bytes = agg.bytes.saturating_sub(entry.size as u64);
                remove_agg = agg.blocks == 0;
            }
            // 先取完值再删键：判别式内不留锁 guard 临时（learnings #38 同源纪律）。
            if remove_agg {
                self.live_sites.remove(&entry.site);
            }
            Some(entry)
        } else {
            None
        }
    }

    /// realloc 失败回滚：挂回 begin 前摘下的条目（不因表满丢弃——槽位在 begin
    /// 已让出，与 `System.realloc` 之间并发分配可能占满预算）。
    fn restore_live(&mut self, ptr: usize, entry: LiveEntry) {
        let site = entry.site;
        let size = entry.size;
        if self.live.insert(ptr, entry).is_none() {
            let agg = self.live_sites.entry(site).or_default();
            agg.blocks += 1;
            agg.bytes += size as u64;
        }
    }

    /// realloc 成功：挂回替换块（同 `restore_live`，不二次卡 `max_live_entries`）。
    fn track_live_realloc(&mut self, ptr: usize, size: usize, site: u64) {
        if self.live.insert(ptr, LiveEntry { site, size }).is_none() {
            let agg = self.live_sites.entry(site).or_default();
            agg.blocks += 1;
            agg.bytes += size as u64;
        }
    }
}

/// 状态锁访问（poison 容忍：聚合观测状态不因 panic 丢弃）。
fn state() -> MutexGuard<'static, Option<TraceState>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// 重入守卫 RAII：作用域内本线程记录路径直通（内部分配不再被采样）。
struct RecordingGuard;

impl RecordingGuard {
    fn enter() -> Self {
        RECORDING.with(|c| c.set(true));
        Self
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        RECORDING.with(|c| c.set(false));
    }
}

/// 重入判定（分配热路径在 feature 构建的第二道便宜检查）。
fn recording() -> bool {
    RECORDING.with(Cell::get)
}

/// `init_from_env` 的注入形（测试与未来宿主定制入口）。
///
/// 次序不变量：CONFIG 先落，ENABLED 后置 Release——读者以 Relaxed 读到
/// `ENABLED=true` 后最坏短暂见到 `CONFIG.get()==None`，`record_alloc` 对
/// None 直接返回，无未定义行为。
pub(crate) fn init_with_config(config: TraceConfig, enabled: bool) {
    let _ = CONFIG.set(config);
    if enabled {
        ENABLED.store(true, Ordering::Release);
        if let Some(cfg) = CONFIG.get()
            && cfg.dump_dir.is_some()
        {
            spawn_dump_thread();
        }
        // 局限（T62 DF F2）：bin 路径本事件先于 tracing 订阅者初始化（bin
        // main() 首句调用 init_from_env），实际不可观测——保留供注入形/宿主
        // 定制入口（订阅者可能已就绪）；启用确认走 dump 产物与测量 driver
        // 判据。
        tracing::info!(target: "alloc_trace", enabled = true, "alloc trace enabled");
    }
}

/// 进程快照初始化（`zero-renderer` bin 入口在最早时机调用一次；幂等）。
///
/// 默认构建（无 `alloc-trace` feature）本函数不存在，产品路径零变化。
pub fn init_from_env() {
    if CONFIG.get().is_some() {
        return;
    }
    let enabled = zero_runtime_config::enabled_when_true("ZW_ALLOC_TRACE");
    let config = TraceConfig {
        sample_every: zero_runtime_config::positive_usize("ZW_ALLOC_TRACE_SAMPLE_EVERY", 256),
        // 65536 × 紧凑记录（hash 8 + counters 16 + 窗 96 + 表开销）≈ <10MiB：
        // V8/JIT 运行期唯一调用点量级数十万（T62 attempt3：2048 表在启动期
        // 即满、dropped_sites 53.6 万），表太小会让 traffic 面偏 startup。
        max_sites: zero_runtime_config::positive_usize("ZW_ALLOC_TRACE_MAX_SITES", 65536),
        net_min_bytes: zero_runtime_config::positive_usize("ZW_ALLOC_TRACE_NET_MIN", 64 * 1024),
        max_live_entries: zero_runtime_config::positive_usize("ZW_ALLOC_TRACE_MAX_LIVE", 1 << 16),
        dump_dir: zero_runtime_config::optional_path("ZW_ALLOC_TRACE_DUMP_DIR"),
        dump_period: Duration::from_secs(
            zero_runtime_config::positive_usize("ZW_ALLOC_TRACE_DUMP_PERIOD_SECS", 10) as u64,
        ),
    };
    init_with_config(config, enabled);
}

/// 分配侧记录入口（`TracingAllocator` 成功分配后调用）。
///
/// `ptr` 仅取整做 net 表键；采样路径在锁外完成回溯与解析，锁内只做表更新。
///
/// net 表 site 归属为同调用直通两态（T62 DF F1 契约）：本次调用采样命中→
/// 采样哈希经局部变量直通入表；未采样一律记 unattributed 哨兵 0——不继承
/// 线程历史采样 site，杜绝跨调用栈错归属；采样命中但捕获不足落哨兵键 0 者
/// 同归 unattributed 桶。dump 侧 0 哨兵渲染为 `unattributed`，与直击条目
/// （site 表实帧窗）可区分。
pub(crate) fn record_alloc(size: usize, ptr: usize) {
    if recording() {
        return;
    }
    let _guard = RecordingGuard::enter();
    let Some(config) = CONFIG.get() else { return };
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    // site 提示只在本次调用内传递：采样哈希是局部变量，调用返回即消失，
    // 线程上不残留任何历史 site 状态（无跨调用栈继承面）。
    let mut site_hint = 0u64;
    if seq.is_multiple_of(config.sample_every as u64) {
        let (ips, captured) = capture_frame_ips();
        let hash = site_signature(&ips[..captured]);
        let mut outer = state();
        outer
            .get_or_insert_with(TraceState::new)
            .record_sample(size, hash, ips, captured);
        site_hint = hash;
    }
    if size >= config.net_min_bytes {
        let mut outer = state();
        outer
            .get_or_insert_with(TraceState::new)
            .track_live(ptr, size, site_hint, config.max_live_entries);
    }
}

/// 释放侧 net 表回收入口（`TracingAllocator` 释放前调用）。
pub(crate) fn record_dealloc(ptr: usize) {
    if recording() {
        return;
    }
    let _guard = RecordingGuard::enter();
    let mut outer = state();
    if let Some(active) = outer.as_mut() {
        active.untrack_live(ptr);
    }
}

/// realloc 起始：旧指针仍唯一持有，先出 net 表（与 `record_dealloc` 同序）。
pub(crate) fn record_realloc_begin(old_ptr: usize) -> Option<LiveEntry> {
    if recording() {
        return None;
    }
    let _guard = RecordingGuard::enter();
    let mut outer = state();
    let active = outer.as_mut()?;
    active.take_live(old_ptr)
}

/// realloc 成功：新块入 net 表（无回溯采样，site 哨兵 0——T62 DF F1）。
///
/// `replacement` 为 true 表示 begin 已摘下旧块（槽位已让出），可不经
/// `max_live_entries` 卡；否则与 `record_alloc` 同样受 net 表上限约束。
pub(crate) fn record_realloc_finish(new_ptr: usize, new_size: usize, replacement: bool) {
    if recording() {
        return;
    }
    let _guard = RecordingGuard::enter();
    let Some(config) = CONFIG.get() else { return };
    let mut outer = state();
    let Some(active) = outer.as_mut() else { return };
    if new_size >= config.net_min_bytes {
        if replacement {
            active.track_live_realloc(new_ptr, new_size, 0);
        } else {
            active.track_live(new_ptr, new_size, 0, config.max_live_entries);
        }
    }
}

/// realloc 失败回滚：`System.realloc` 返 null 时原块仍在，挂回 begin 摘下条目。
pub(crate) fn record_realloc_abort(old_ptr: usize, entry: LiveEntry) {
    if recording() {
        return;
    }
    let _guard = RecordingGuard::enter();
    let Some(config) = CONFIG.get() else { return };
    let mut outer = state();
    let Some(active) = outer.as_mut() else { return };
    if entry.size >= config.net_min_bytes {
        active.restore_live(old_ptr, entry);
    }
}

/// 原始 IP 回溯捕获（帧回调展开，不符号化——符号解析全部离线）。
/// 返回（IP 数组，实捕获数）；展开器失败时捕获数为 0。
fn capture_frame_ips() -> ([usize; MAX_CAPTURE_FRAMES], usize) {
    let mut ips = [0usize; MAX_CAPTURE_FRAMES];
    let mut count = 0usize;
    backtrace::trace(|frame| {
        if count >= MAX_CAPTURE_FRAMES {
            return false;
        }
        ips[count] = frame.ip() as usize;
        count += 1;
        true
    });
    (ips, count)
}

/// site 签名：跳过机器帧后取 [`SIGNATURE_FRAMES`] 个原始 IP 哈希（进程内
/// 稳定键）。偏移摘要改在 dump 时由存储的原始窗现渲染（见
/// [`render_frames`]）——采样热路径不再构造字符串（V8/JIT 运行期唯一
/// 调用点量级数十万，attempt3 实证 2048 表即满）。哈希键为原始 IP 而非
/// 符号名：同 site 跨线程/跨次捕获逐字恒等，代价是二进制变化（重编译）
/// 间键不迁移——仪器按次进程生命周期运转，该边界可接受。
fn site_signature(ips: &[usize]) -> u64 {
    if ips.len() <= SKIP_FRAMES {
        // 捕获不足（极端：展开器直接失败）——固定键聚合并如实标注。
        return 0;
    }
    let end = (SKIP_FRAMES + SIGNATURE_FRAMES).min(ips.len());
    let mut hasher = DefaultHasher::new();
    for ip in &ips[SKIP_FRAMES..end] {
        ip.hash(&mut hasher);
    }
    hasher.finish()
}

/// dump 时把 site 存储的原始 IP 窗渲染为空格分隔 `0x…` 模块相对偏移摘要
/// （`rel = ip − exe_base` 即 ELF 静态 vaddr，addr2line 直接可用；基址解析
/// 失败退回绝对 IP 并由 dump `exe_base: null` 如实标注）。captured=0 渲染
/// 为 `unresolved`。
fn render_frames(stats: &SiteStats) -> String {
    if stats.captured == 0 {
        return String::from("unresolved");
    }
    let base = exe_load_base();
    let mut excerpt = String::new();
    for ip in &stats.frames[..stats.captured] {
        if !excerpt.is_empty() {
            excerpt.push(' ');
        }
        let rel = match base {
            Some(base) => ip.wrapping_sub(base),
            None => *ip,
        };
        excerpt.push_str(&format!("{rel:#x}"));
    }
    excerpt
}

/// 主映像加载基址（PIE 运行基址；进程内 OnceLock 缓存）：`/proc/self/maps`
/// 中路径等于当前可执行文件的首个映射段起始地址。`rel = ip − base` 即 ELF
/// 静态 vaddr，`addr2line -e <bin>` 直接可用。解析失败返回 None（摘要退回
/// 绝对 IP，dump `exe_base` 如实标 null）。
fn exe_load_base() -> Option<usize> {
    static BASE: OnceLock<Option<usize>> = OnceLock::new();
    *BASE.get_or_init(|| {
        let exe = std::fs::canonicalize(std::env::current_exe().ok()?).ok()?;
        let exe_str = exe.to_string_lossy().into_owned();
        let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
        for line in maps.lines() {
            if !line.ends_with(&exe_str) {
                continue;
            }
            // pathname 是行尾字段：后缀命中且紧邻空白才算精确路径匹配。
            let before = &line[..line.len() - exe_str.len()];
            if !before.ends_with(' ') {
                continue;
            }
            let start = line.split('-').next()?;
            return usize::from_str_radix(start, 16).ok();
        }
        None
    })
}

/// 当前进程 PSS（kB）——复用 `mem_reclaim` 探针读法（同 crate 私有面）。
fn process_pss_kb() -> Option<u64> {
    crate::mem_reclaim::read_process_pss_kb()
}

/// dump 快照为 JSON 并写 `dir/alloc-trace-<ms>-<seq>-<reason>.json`；返回写入路径。
///
/// `dir` 显式给定时优先（测试入口）；否则回落配置 `dump_dir`；两者皆无返回 Err。
/// 落盘原子（同目录 temp 写 + rename，T62 DF F3）：中断留下的是 temp 残件，
/// 目标名要么完整要么不出现；序号成分保证同毫秒同 reason 不碰撞覆写。
/// 整体处于重入守卫内：锁内格式化/写盘的自身分配直通 [`System`]，不自锁。
pub(crate) fn dump_snapshot_to(dir: Option<&Path>, reason: &str) -> Result<PathBuf, String> {
    if recording() {
        return Err(String::from("dump inside recording path is not allowed"));
    }
    let _guard = RecordingGuard::enter();
    let config = CONFIG.get().ok_or("alloc trace not initialized")?;
    let Some(dir) = dir.or(config.dump_dir.as_deref()) else {
        return Err(String::from("no dump dir configured"));
    };
    std::fs::create_dir_all(dir).map_err(|e| format!("create dump dir failed: {e}"))?;
    let reason = sanitize_reason(reason);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let seq = DUMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("alloc-trace-{ms}-{seq}-{reason}.json"));
    let json = snapshot_json(config, &reason);
    let tmp = dir.join(format!("alloc-trace-{ms}-{seq}-{reason}.json.tmp"));
    std::fs::write(&tmp, json).map_err(|e| format!("write dump {} failed: {e}", tmp.display()))?;
    // 同目录 rename 同文件系统原子：目标要么缺要么完整，无撕裂 JSON。
    if let Err(error) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!(
            "rename dump {} -> {} failed: {error}",
            tmp.display(),
            path.display()
        ));
    }
    tracing::info!(target: "alloc_trace", path = %path.display(), reason, "alloc trace dumped");
    Ok(path)
}

/// reason 白名单净化（触发文件内容不可信：测量驱动是本进程外部输入）。
fn sanitize_reason(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .take(24)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        String::from("trigger")
    } else {
        cleaned
    }
}

/// 快照 JSON 文本（serde_json 手拼；分析侧按 schema_version 判读）。
fn snapshot_json(config: &TraceConfig, reason: &str) -> String {
    let mut state = state();
    let state = state.get_or_insert_with(TraceState::new);
    let mut traffic: Vec<(&u64, &SiteStats)> = state.sites.iter().collect();
    traffic.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes).then(a.0.cmp(b.0)));
    let mut live: Vec<(&u64, &LiveAgg)> = state.live_sites.iter().collect();
    live.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes).then(a.0.cmp(b.0)));
    let live_total: u64 = state.live_sites.values().map(|a| a.bytes).sum();

    let traffic_top: Vec<serde_json::Value> = traffic
        .iter()
        .take(DUMP_TOP_N)
        .map(|(hash, stats)| {
            serde_json::json!({
                "hash": format!("{hash:016x}"),
                "sampled_allocs": stats.allocs,
                "sampled_bytes": stats.bytes,
                "frames": render_frames(stats),
            })
        })
        .collect();
    let live_top: Vec<serde_json::Value> = live
        .iter()
        .take(DUMP_TOP_N)
        .map(|(hash, agg)| {
            let frames = state.sites.get(hash).map(render_frames).unwrap_or_else(|| {
                if **hash == 0 {
                    String::from("unattributed")
                } else {
                    String::from("no-sampled-signature")
                }
            });
            serde_json::json!({
                "hash": format!("{hash:016x}"),
                "live_blocks": agg.blocks,
                "live_bytes": agg.bytes,
                "frames": frames,
            })
        })
        .collect();

    serde_json::json!({
        "schema_version": 1,
        "reason": reason,
        "captured_unix_ms": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        "pid": std::process::id(),
        "pss_kb": process_pss_kb(),
        // 离线符号解析锚点：`frames` 内偏移 = 运行时 IP − exe_base（即 ELF
        // 静态 vaddr，addr2line 直接可用）；exe_base 解析失败为 null 且摘要
        // 已退回绝对 IP（分析侧按 exe_path 与同构建二进制判读）。
        "exe_path": std::env::current_exe().ok().map(|p| p.display().to_string()),
        "exe_base": exe_load_base().map(|b| format!("{b:#x}")),
        "config": {
            "sample_every": config.sample_every,
            "max_sites": config.max_sites,
            "net_min_bytes": config.net_min_bytes,
            "max_live_entries": config.max_live_entries,
        },
        "totals": {
            "alloc_calls": SEQ.load(Ordering::Relaxed),
            "sampled_allocs": state.sampled_allocs,
            "sampled_bytes": state.sampled_bytes,
            "dropped_sites": state.dropped_sites,
            "live_tracked_blocks": state.live.len() as u64,
            "live_bytes_total": live_total,
            "live_untracked": state.live_untracked,
        },
        "traffic_top": traffic_top,
        "live_top": live_top,
    })
    .to_string()
}

/// dump 线程：1s 轮询触发文件；周期到点落 `periodic` dump。
///
/// 线程随进程退出终止（renderer 退出路径 `std::process::exit`，末次 dump 与
/// 退出间窗口由测量协议吸收）。// FIXME: 进程退出前 final dump 挂钩未建——
/// 当前以周期/触发 dump 覆盖，退出窗不落专门快照。
fn spawn_dump_thread() {
    let Some(config) = CONFIG.get() else { return };
    let Some(dir) = config.dump_dir.clone() else { return };
    let period = config.dump_period;
    let trigger = dir.join("trigger");
    let builder = std::thread::Builder::new().name(String::from("alloc-trace-dump"));
    let spawned = builder.spawn(move || {
        let mut last_periodic = SystemTime::now();
        loop {
            std::thread::sleep(Duration::from_secs(1));
            // 触发文件优先（检查点精确绑定）；读后即删（claim 语义）。
            if let Ok(reason) = std::fs::read_to_string(&trigger) {
                let _ = std::fs::remove_file(&trigger);
                let reason = reason.lines().next().unwrap_or("trigger");
                // 检查点失败不静默（T62 DF F2：磁盘满/权限等要有线索）；
                // 订阅者此时已初始化，事件有效。不 panic——dump 线程必须
                // 存活到下一检查点。
                if let Err(error) = dump_snapshot_to(Some(&dir), reason) {
                    tracing::warn!(target: "alloc_trace", reason = %reason, error = %error, "alloc trace trigger dump failed");
                }
                continue;
            }
            if last_periodic.elapsed().is_ok_and(|elapsed| elapsed >= period) {
                last_periodic = SystemTime::now();
                if let Err(error) = dump_snapshot_to(Some(&dir), "periodic") {
                    tracing::warn!(target: "alloc_trace", error = %error, "alloc trace periodic dump failed");
                }
            }
        }
    });
    if let Err(error) = spawned {
        // 局限同 init_with_config：bin 路径先于订阅者，本事件不可观测
        // （dump 缺失由测量 driver 的检查点判据兜底检出）。
        tracing::warn!(target: "alloc_trace", %error, "alloc trace dump thread spawn failed");
    }
}

/// 全局分配器包装：所有请求无条件转发 [`System`]（分配语义逐字不变），
/// 成功之后按进程快照开关追加观测簿记。
///
/// 默认构建不含本类型；feature 构建未启用时簿记入口在 `AtomicBool` 快速门
/// 处直接返回。SAFETY（`GlobalAlloc` 契约）：转发保持 `System` 的全部布局/
/// 对齐/别名保证；簿记只读指针整数值，不触碰块内容、不改返回值；记录路径
/// 经 `RecordingGuard` 防重入，内部分配直通 `System`，不会在持有状态锁时
/// 再入本包装的状态锁（无重入死锁面）。
pub struct TracingAllocator;

unsafe impl GlobalAlloc for TracingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 调用方满足 GlobalAlloc 契约（合法 layout、配对释放）；
        // System.alloc 对合法 layout 恒安全。
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && ENABLED.load(Ordering::Relaxed) && !recording() {
            record_alloc(layout.size(), ptr as usize);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 同 alloc；清零语义由 System 保持。
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && ENABLED.load(Ordering::Relaxed) && !recording() {
            record_alloc(layout.size(), ptr as usize);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ENABLED.load(Ordering::Relaxed) && !recording() {
            record_dealloc(ptr as usize);
        }
        // SAFETY: ptr 由 System 通道产出且 layout 与分配时一致（调用方契约）。
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // 旧块出表须先于 System.realloc（同 dealloc 序）：扩缩容后旧地址可能
        // 立即被并发分配复用，延后 untrack 会误摘新占有者条目。
        let preserved = if ENABLED.load(Ordering::Relaxed) && !recording() {
            record_realloc_begin(ptr as usize)
        } else {
            None
        };
        // SAFETY: System.realloc 按 GlobalAlloc 契约处理扩缩容；失败返 null
        // 时原块不动，须回滚 begin 摘表。
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if new.is_null() {
            if let Some(entry) = preserved {
                record_realloc_abort(ptr as usize, entry);
            }
            return new;
        }
        if ENABLED.load(Ordering::Relaxed) && !recording() {
            record_realloc_finish(new as usize, new_size, preserved.is_some());
        }
        new
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// site 签名（纯函数面）：同一原始窗口哈希恒等；签名窗内任一帧变化即
    /// 换键（调用点差异产生不同签名是产品语义——T62 探针实证：两次捕获
    /// ips[0..4] 恒等、ips[4] 差 48 字节 = 测试函数两个不同调用点的返回
    /// 地址）；捕获不足 SKIP_FRAMES 落哨兵键 0。
    #[test]
    fn site_signature_is_stable_per_call_site() {
        let ips = [0x1110, 0x2220, 0x3330, 0x4440, 0x5550, 0x6660, 0x7770, 0x8880];
        let h1 = site_signature(&ips);
        let h2 = site_signature(&ips);
        assert_eq!(h1, h2, "同一原始窗口签名哈希必须恒等");

        let mut shifted = ips;
        shifted[3] = 0x9990; // 签名窗（≥SKIP_FRAMES）内帧变化
        let h3 = site_signature(&shifted);
        assert_ne!(h3, h1, "签名窗内帧变化必须换键");

        let short = [0x1110, 0x2220];
        assert_eq!(site_signature(&short), 0, "捕获不足落哨兵键 0");
    }

    /// dump 渲染：原始窗按 captured 渲染偏移，相邻注入帧差值保持（有无
    /// 基址都成立——基址是公共减数）；captured=0 落 unresolved 哨兵。
    #[test]
    fn render_frames_renders_window_with_relative_offsets() {
        let mut frames = [0usize; MAX_CAPTURE_FRAMES];
        frames[0] = 0x7f00;
        frames[1] = 0x7f08;
        let stats = SiteStats {
            allocs: 1,
            bytes: 9,
            frames,
            captured: 2,
        };
        let rendered = render_frames(&stats);
        let tokens: Vec<&str> = rendered.split(' ').collect();
        assert_eq!(tokens.len(), 2, "captured=2 只渲染两帧：{rendered}");
        let v0 = usize::from_str_radix(tokens[0].trim_start_matches("0x"), 16).unwrap();
        let v1 = usize::from_str_radix(tokens[1].trim_start_matches("0x"), 16).unwrap();
        assert_eq!(v1.wrapping_sub(v0), 8, "相邻帧差值必须保持（公共减数）");

        let empty = SiteStats {
            allocs: 0,
            bytes: 0,
            frames: [0; MAX_CAPTURE_FRAMES],
            captured: 0,
        };
        assert_eq!(render_frames(&empty), "unresolved");
    }

    /// 真实捕获面：展开器在测试二进制里取到非零帧（回溯链路可用性）。
    #[test]
    fn capture_frame_ips_returns_real_stack() {
        let (ips, captured) = capture_frame_ips();
        assert!(captured > 0, "回溯必须捕获到帧");
        assert!(ips[..captured].iter().all(|ip| *ip != 0), "捕获帧 IP 必须非零");
    }

    /// 主映像基址：Linux 下 /proc/self/maps 必含当前测试二进制，基址可解析
    /// 且落在合理空间（非零）。/proc 不可用的平台该函数优雅返回 None，不设测。
    #[cfg(target_os = "linux")]
    #[test]
    fn exe_load_base_resolves_on_linux() {
        assert!(exe_load_base().is_some(), "/proc/self/maps 必含当前 exe 映射");
    }

    /// 有界 site 表：超限新 site 计入 dropped，不驱逐、不无界增长。
    #[test]
    fn site_table_is_bounded_and_counts_drops() {
        let mut state = TraceState::new();
        TEST_MAX_SITES.with(|m| m.set(Some(2)));
        for i in 0..5u64 {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let mut frames = [0usize; MAX_CAPTURE_FRAMES];
            frames[0] = i as usize + 0x10;
            state.record_sample(100, hasher.finish(), frames, 1);
        }
        TEST_MAX_SITES.with(|m| m.set(None));
        assert_eq!(state.sites.len(), 2, "表长必须钉在上限");
        assert_eq!(state.dropped_sites, 3, "超限 site 计入丢弃计数");
    }

    /// net 表：track/untrack 增量一致，释放后归零；未知指针不 panic。
    #[test]
    fn live_netting_tracks_and_releases() {
        let mut state = TraceState::new();
        state.track_live(0x1000, 200_000, 0xABCD, 16);
        state.track_live(0x2000, 50_000, 0xABCD, 16);
        assert_eq!(state.live_sites.get(&0xABCD).map(|a| a.bytes), Some(250_000));
        state.untrack_live(0x1000);
        assert_eq!(state.live_sites.get(&0xABCD).map(|a| a.bytes), Some(50_000));
        state.untrack_live(0x2000);
        assert!(!state.live_sites.contains_key(&0xABCD), "全释放后 site 聚合清位");
        state.untrack_live(0xDEAD);
    }

    /// net 表溢出：超 max_live_entries 的大块计入 live_untracked。
    #[test]
    fn live_table_overflow_counts_untracked() {
        let mut state = TraceState::new();
        state.track_live(0x1000, 200_000, 7, 1);
        state.track_live(0x2000, 200_000, 7, 1);
        assert_eq!(state.live.len(), 1);
        assert_eq!(state.live_untracked, 1);
    }

    /// reason 白名单净化：不可信字符归 `_`，空串回落固定值。
    #[test]
    fn reason_is_sanitized() {
        assert_eq!(sanitize_reason("C1"), "C1");
        assert_eq!(sanitize_reason("a/b\nC2"), "a_b_C2");
        assert_eq!(sanitize_reason(""), "trigger");
        let long = "x".repeat(64);
        assert_eq!(sanitize_reason(&long).len(), 24);
    }

    /// 全局单例链路（OnceLock 每测试进程仅可初始化一次，全局相关断言合并进
    /// 本测试；其余用例只触纯状态面，避免跨用例进程态互扰——learnings #19）。
    /// 覆盖：注入形初始化 → net 归属同调用直通两态（T62 DF F1 回归：采样
    /// 命中调用的 ≥net_min 条目直通本次采样 site；未采样调用落 unattributed
    /// 哨兵 0，不得继承线程历史采样）→ 记录入口前进 totals → 注入态 →
    /// dump 落盘合法 JSON 且 totals/哨兵渲染一致（site=0 条目标注
    /// `unattributed`，与直击条目可区分）。ENABLED 保持 false：不污染同进程
    /// 其他用例的分配路径（全局接线的行为验证走测量轮 dump 产物与二进制
    /// 符号核对）。
    #[test]
    fn global_init_record_and_dump_roundtrip() {
        let dir = std::env::temp_dir().join(format!("zw-alloc-trace-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = CONFIG.set(TraceConfig {
            sample_every: 2,
            max_sites: 64,
            net_min_bytes: 1024,
            max_live_entries: 64,
            dump_dir: Some(dir.clone()),
            dump_period: Duration::from_secs(3600),
        });
        {
            // 状态锁 guard 不跨 record_* 调用：Mutex 不可重入，持锁进入
            // record_dealloc 的内层 state() 即同线程自死锁（T62 gdb 栈实证）。
            // SEQ 只由测试驱动的 record_* 前进（ENABLED=false，分配器不入
            // 记录路径），单线程顺序调用可钉住采样窗命中/错过。
            SEQ.store(0, Ordering::Relaxed);
            let before = {
                let outer = state();
                outer.as_ref().map(|s| s.sampled_allocs).unwrap_or(0)
            };
            // 直通正例（T62 DF F1）：seq=0 命中采样窗（2 的倍数），同一次
            // 调用 ≥net_min 入 net 表——条目 site 必须是本次采样哈希（site
            // 表实键），而非哨兵 0。
            record_alloc(4096, 0x9000);
            {
                let outer = state();
                let active = outer.as_ref().expect("record_alloc 后状态必已构造");
                assert_eq!(active.sampled_allocs, before + 1, "seq=0 命中采样窗必须前进");
                let live_site = active.live.get(&0x9000).expect("≥net_min 必入 net 表").site;
                assert_ne!(live_site, 0, "采样命中调用的 net 条目必须直通 site 而非哨兵");
                assert!(
                    active.sites.contains_key(&live_site),
                    "直通 site 必须是同调用入 site 表的实键"
                );
            }
            record_dealloc(0x9000);
            // 未采样负例（T62 DF F1）：seq=1 错过采样窗，≥net_min 入表——
            // site 必须落 unattributed 哨兵 0，不得继承上一调用的采样哈希。
            record_alloc(8192, 0x9100);
            {
                let outer = state();
                let active = outer.as_ref().expect("状态必已构造");
                assert_eq!(active.sampled_allocs, before + 1, "未采样调用不得前进采样计数");
                let live_site = active.live.get(&0x9100).expect("≥net_min 必入 net 表").site;
                assert_eq!(
                    live_site, 0,
                    "未采样 ≥net_min 必落 unattributed 哨兵，不得继承历史采样 site"
                );
            }
            record_dealloc(0x9100);
            // 注入确定性状态供 dump 断言（独立锁作用域）：原始 IP 窗含注入值
            // 0x7f00/0x7f08，exe 基址解析失败时渲染绝对 IP、成功时渲染相对
            // 偏移——两种形态都以 "0x7f" 前缀锚定同一注入值。
            {
                let mut outer = state();
                let mut fresh = TraceState::new();
                let mut hasher = DefaultHasher::new();
                "dumper".hash(&mut hasher);
                let mut frames = [0usize; MAX_CAPTURE_FRAMES];
                frames[0] = 0x7f00;
                frames[1] = 0x7f08;
                fresh.record_sample(4096, hasher.finish(), frames, 2);
                fresh.track_live(0x7000, 8192, hasher.finish(), 16);
                // unattributed 腿（T62 DF F1）：site=0 哨兵条目在 dump
                // live_top 渲染 `unattributed`，与直击条目可区分。
                fresh.track_live(0x7001, 4096, 0, 16);
                *outer = Some(fresh);
            }
        }
        let path = dump_snapshot_to(Some(&dir), "test").expect("dump 必须成功（目录已建）");
        let text = std::fs::read_to_string(&path).expect("dump 文件可读");
        let value: serde_json::Value = serde_json::from_str(&text).expect("dump 必须是合法 JSON");
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["totals"]["live_tracked_blocks"], 2);
        let rendered = value["traffic_top"][0]["frames"].as_str().unwrap();
        let tokens: Vec<&str> = rendered.split(' ').collect();
        assert_eq!(tokens.len(), 2, "captured=2 只渲染两帧：{rendered}");
        let v0 = usize::from_str_radix(tokens[0].trim_start_matches("0x"), 16).unwrap();
        let v1 = usize::from_str_radix(tokens[1].trim_start_matches("0x"), 16).unwrap();
        // 注入帧 0x7f00/0x7f08：公共减数（exe 基址，有无皆可）下差值必为 8。
        assert_eq!(v1.wrapping_sub(v0), 8, "注入帧差值必须保持：{rendered}");
        assert_eq!(value["live_top"][0]["live_bytes"], 8192);
        // live_top 按 bytes desc 排序：8192 直击条目在前、4096 哨兵条目在后。
        assert_eq!(
            value["live_top"][1]["frames"], "unattributed",
            "site=0 哨兵条目必须如实标注，与直击条目可区分",
        );
        assert_eq!(value["live_top"][1]["hash"], "0000000000000000");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
