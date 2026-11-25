//! T108 三通道改码仪拍（env 门控诊断，默认关——门关时主环行为与基线一致）。
//! 闭口对象＝T107 anon 下钻遗留的三类身份未判结构（t107-anon-drill.md §六）：
//!
//! - **A** `ZW_RENDERER_V8_SPACE_TRACE`：V8 isolate per-space 页级统计（5s 窗
//!   JSONL；`ProbeHeapSpaces` 命令臂在 isolate 线程自报，主环仪拍窗读共享缓冲）
//!   ——闭口大单段对（45.3-46.9 MiB）的 V8 堆页归属与长尾中 V8 页份额。
//! - **B** `ZW_RENDERER_MALLOC_STATS_DUMP`：glibc `malloc_info` 分 arena 账
//!   （5s 窗向路径追加 XML 文档，仅 linux-gnu）——闭口 arena 三簇构成与
//!   hblkhd（mmap 大块）明细。
//! - **C** `ZW_RENDERER_MMAP_TRACE`：`/proc/self/maps` 周期差分（1s 窗，anon
//!   ≥512 KiB 段新现/消失/伸缩事件 JSONL）——闭口大 mmap 出现时点与 64 MiB
//!   对齐族归属（glibc arena 家族判别，同 t107 镜头 B 口径）。
//!
//! 方案取舍（t107 §六 仪拍 C 提示的 LD_PRELOAD 备选被弃）：mmap interposition
//! 需仓库外新增 C shim 构件且改变全部 mmap 调用的符号解析面（扰动不可测），
//! 改为进程内 maps 差分——读+解析 <1ms/次（1Hz），代价可量化；但两次采样间
//! 出现又释放的短命 mmap 不可见（T107 靶目标均为稳态驻留结构，不受影响）。
//! 「栈上下文线索」由同窗跨通道对轴替代（A 的 per-space 增量/B 的 hblkhd
//! 增量与 C 的事件在同一 epoch ms 时间线上比对），真调用栈不在本通道范围。
//!
//! 阈值 512 KiB（brief 草拟 16 MiB 的考古修正）：T107 实测大单段对成员
//! 12.56-33.53 MiB、固定散簇成员 0.61-2.45 MiB——16 MiB 阈值会漏散簇全体与
//! 半数大段对成员，按实测最小成员（0.61 MiB）下探到 512 KiB。
//!
//! env 三态语义（A/C，同 [`crate::mem_trace`] / census 形制）：`1`/`true` →
//! tracing INFO；`0`/`false`/`off` → 关；其余非空值 → 追加写该路径。B 仅接受
//! 路径值（XML 全文走 tracing 会灌爆日志，truthy/falsy 一律视为关闭）。
//!
//! 写失败语义（三通道一致：诊断侧信道不逐窗重试刷日志）：A/C sink 打开或写失败
//! warn 一次后停用（`sink` 置 None，通道静默）；B 任一写失败（fwrite/`malloc_info`/
//! fclose）置 `failed` 闩 warn 一次后永久停写。C 的 maps 读失败为瞬时处理（warn
//! 一次、保留 prev 下窗重试，不停用）；maps 读为有损读（`from_utf8_lossy`）——
//! file-backed 路径字段含非 UTF-8 字节时仅路径呈现受影响，读本身永不失败。

use std::time::Duration;
use std::time::Instant;

/// C 通道差分阈值：512 KiB（T107 靶目标最小成员 0.61 MiB，见模块注释）。
const VMA_TRACE_THRESHOLD: u64 = 512 * 1024;
/// C 通道伸缩事件阈值：两次采样间 end 移动 ≥1 MiB 才记 grow/shrink
/// （更小的移动是 arena 顶常规扩展，记满会淹没事件流）。
const VMA_RESIZE_DELTA: u64 = 1024 * 1024;
/// 64 MiB（2^26）对齐掩码：glibc HEAP_MAX_SIZE 对齐父址判别面（t107 镜头 B 同口径）。
const ARENA_ALIGN: u64 = 64 * 1024 * 1024;

/// A/C 共用的输出形态：tracing 或追加写文件（路径）。
#[derive(Debug)]
enum TraceSink {
    /// tracing INFO（`1`/`true`）。
    Log,
    /// 追加写文件（其余非空值）。
    Sink(std::path::PathBuf),
}

/// env 值 → 输出形态（同 [`crate::js_worker`] census_sink_path 三态判读：truthy
/// 值与 falsy 值都不当作 sink 路径，防 `TRACE=0` 意外落文件、`TRACE=1` 在 cwd
/// 留名为 `1` 的垃圾文件）。
fn trace_sink(name: &str) -> Option<TraceSink> {
    let value = zero_runtime_config::optional_string(name)?;
    let truthy = value == "1" || value.eq_ignore_ascii_case("true");
    let falsy = value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("off");
    if truthy {
        return Some(TraceSink::Log);
    }
    if falsy {
        return None;
    }
    Some(TraceSink::Sink(std::path::PathBuf::from(value)))
}

/// 落一行 JSONL：sink 模式追加写文件（逐行开合，崩溃安全），log 模式 tracing INFO。
/// sink 打开或写失败：warn 一次后停用（`*sink` 置 None）——诊断侧信道不逐行重试
/// 刷日志（B 通道 `MallocInfoDump::fail` 同语义）。
fn write_line(sink: &mut Option<TraceSink>, line: &str) {
    match sink {
        Some(TraceSink::Sink(path)) => {
            let mut f = match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    // 诊断侧信道失败可见化：静默吞错会掩盖 sink 病因（mem_trace 同款）。
                    tracing::warn!(target: "renderer_mem_diag", "sink open failed, disabling sink: {e}");
                    *sink = None;
                    return;
                }
            };
            use std::io::Write as _;
            if let Err(e) = writeln!(f, "{line}") {
                tracing::warn!(target: "renderer_mem_diag", "sink write failed, disabling sink: {e}");
                *sink = None;
            }
        }
        // None＝已停用（同批后续事件静默丢弃，不得转落 tracing），Log 才走 INFO。
        Some(TraceSink::Log) => tracing::info!(target: "renderer_mem_diag", "{line}"),
        None => {}
    }
}

/// T108 三通道仪拍门面（主环 [`crate::mem_trace::RendererMemTrace`] 采样点旁
/// 一并驱动；三通道各自 env 门控，全关时 tick 为三次布尔判断）。
pub(crate) struct MemDiagChannels {
    /// A：V8 per-space 页级统计。
    v8_spaces: V8SpaceTrace,
    /// B：malloc_info 分 arena 账。
    malloc_info: MallocInfoDump,
    /// C：maps 差分大段标记。
    vma: VmaTrace,
}

impl MemDiagChannels {
    pub(crate) fn new() -> Self {
        Self {
            v8_spaces: V8SpaceTrace::new(),
            malloc_info: MallocInfoDump::new(),
            vma: VmaTrace::new(),
        }
    }

    /// 主环每迭代调用：三通道各按自己的 env 门与窗节奏采样（全关零开销）。
    pub(crate) fn maybe_tick(&mut self, worker: &crate::js_worker::RendererJsWorker) {
        self.v8_spaces.tick(worker);
        self.malloc_info.tick();
        self.vma.tick();
    }
}

/// T108 仪拍 A：V8 isolate per-space 页级统计（5s 窗；与
/// [`crate::mem_trace::RendererMemTrace`] 同节奏，两线可按 epoch ms 对轴）。
struct V8SpaceTrace {
    sink: Option<TraceSink>,
    window_start: Instant,
}

impl V8SpaceTrace {
    fn new() -> Self {
        Self {
            sink: trace_sink("ZW_RENDERER_V8_SPACE_TRACE"),
            window_start: Instant::now(),
        }
    }

    fn tick(&mut self, worker: &crate::js_worker::RendererJsWorker) {
        if self.sink.is_none() {
            return;
        }
        if self.window_start.elapsed() < Duration::from_secs(5) {
            return;
        }
        self.window_start = Instant::now();
        // 快照自报走 worker 命令边界（isolate 线程纪律，同 ProbeHeap）：本窗发起
        // 的请求在下一窗读得（读数至多滞后一个命令队列深度，probe_t 与 t= 之差
        // 即真实滞后）。
        let _ = worker.request_v8_space_probe();
        let Some(stats) = worker.latest_v8_space_line() else {
            // 首窗（读数未达）或 QuickJS 构建（恒无读数）——无行可落。
            return;
        };
        let ts = crate::mem_trace::epoch_ms();
        let line = format!("{{\"kind\":\"renderer-v8spaces\",\"t\":{ts},{stats}}}");
        write_line(&mut self.sink, &line);
    }
}

/// T108 仪拍 B：glibc `malloc_info` 分 arena 账（5s 窗；仅 linux-gnu glibc，
/// 其余目标 dump() 为空 no-op）。仅路径模式：每窗向路径追加一个
/// `<!-- zw-malloc-info t=<epoch ms> seq=<n> -->` 标记行 + 一个完整 XML 文档
/// （`<malloc version="1">…</malloc>`，逐 arena `<heap>` 节含 system/mmap
/// 账与 bins 明细）——单文件多文档，按标记行切分。
struct MallocInfoDump {
    enabled: bool,
    path: std::path::PathBuf,
    window_start: Instant,
    seq: u64,
    /// 打开/写入失败后停用（防逐窗重试刷日志），首败记一次 warn。
    failed: bool,
}

impl MallocInfoDump {
    fn new() -> Self {
        // 仅路径值启用；truthy 值记一次 warn（本通道无 tracing 形态——XML 全文走
        // tracing 会灌爆日志），falsy 值静默关闭。
        let raw = zero_runtime_config::optional_string("ZW_RENDERER_MALLOC_STATS_DUMP");
        let truthy = raw
            .as_deref()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        let falsy = raw
            .as_deref()
            .is_some_and(|v| v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off"));
        if truthy {
            tracing::warn!(target: "renderer_mem_diag",
                "ZW_RENDERER_MALLOC_STATS_DUMP needs a filesystem path; truthy value is disabled (no tracing form for XML)");
        }
        Self {
            enabled: raw.is_some() && !truthy && !falsy,
            path: raw.map(std::path::PathBuf::from).unwrap_or_default(),
            window_start: Instant::now(),
            seq: 0,
            failed: false,
        }
    }

    fn tick(&mut self) {
        if !self.enabled || self.failed {
            return;
        }
        if self.window_start.elapsed() < Duration::from_secs(5) {
            return;
        }
        self.window_start = Instant::now();
        self.dump();
    }

    /// 一窗一文档：append 打开 → 写标记行 → `malloc_info` 落 XML → 关闭。
    /// fopen/malloc_info 走 glibc 内部锁（malloc_info 遍历各 arena 时串行化），
    /// 5s 窗频下与分配热路径的锁竞争可忽略。
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    fn dump(&mut self) {
        let Ok(path) = std::ffi::CString::new(self.path.as_os_str().as_encoded_bytes()) else {
            self.fail("path contains interior NUL");
            return;
        };
        let ts = crate::mem_trace::epoch_ms();
        let marker = format!("<!-- zw-malloc-info t={ts} seq={} -->\n", self.seq);
        // SAFETY: 三个指针参数均指向本栈/本结构持有的合法内存；FILE 由 fopen 创建
        // 并在全部读写后 fclose（含错误路径——malloc_info 失败仍须 fclose 防泄漏）。
        // glibc 内部自持分配器锁，无并发 UB 面。
        unsafe {
            let mode = b"a\0";
            let f = libc::fopen(path.as_ptr(), mode.as_ptr().cast());
            if f.is_null() {
                self.fail("fopen failed");
                return;
            }
            // 写失败停用（`failed` 字段契约）：标记行/`malloc_info`/收尾换行/fclose
            // 任一失败即置闩（warn-once 后永久停写），seq 不推进——失败文档不占序号。
            let mut ok = libc::fwrite(marker.as_ptr().cast(), 1, marker.len(), f) == marker.len();
            let rc = libc::malloc_info(0, f);
            if rc != 0 {
                ok = false;
            }
            ok &= libc::fwrite(b"\n".as_ptr().cast(), 1, 1, f) == 1;
            let closed = libc::fclose(f) == 0;
            if !ok || !closed {
                let why = if rc != 0 {
                    "malloc_info failed"
                } else {
                    "write/close failed"
                };
                self.fail(why);
                return;
            }
        }
        self.seq += 1;
    }

    /// 非 glibc 目标：构造期不禁用（`new()` 不分 target），本臂空 no-op——env 设
    /// 路径值时每窗空转一次，无输出无副作用。
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    fn dump(&mut self) {}

    fn fail(&mut self, why: &str) {
        self.failed = true;
        tracing::warn!(target: "renderer_mem_diag",
            "ZW_RENDERER_MALLOC_STATS_DUMP disabled after failure: {why}");
    }
}

/// C 通道跟踪的单段（/proc/self/maps 一行；匿名段且 ≥
/// [`VMA_TRACE_THRESHOLD`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct VmaEntry {
    start: u64,
    end: u64,
    /// 4 字符权限串（如 `rw-p`；maps 恒 4 字符，解析失败记 b"????"）。
    perms: [u8; 4],
    /// 内核具名匿名段（`[anon:…]`）；无名段为空串。
    name: String,
}

impl VmaEntry {
    fn size(&self) -> u64 {
        self.end - self.start
    }

    /// 64 MiB 对齐父址（glibc arena 家族判别面，t107 镜头 B 口径）。
    fn parent64(&self) -> u64 {
        self.start & !(ARENA_ALIGN - 1)
    }
}

/// C 通道事件（新现/消失/伸缩）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmaEvent {
    New,
    Gone,
    Grow,
    Shrink,
}

impl VmaEvent {
    fn as_str(self) -> &'static str {
        match self {
            VmaEvent::New => "new",
            VmaEvent::Gone => "gone",
            VmaEvent::Grow => "grow",
            VmaEvent::Shrink => "shrink",
        }
    }
}

/// T108 仪拍 C：`/proc/self/maps` 周期差分（1s 窗）。只跟踪匿名段（无名或
/// `[anon:…]` 具名）≥[`VMA_TRACE_THRESHOLD`]；file-backed/`[heap]`/`[stack…]`
/// 等具名段不进差分集（B/mallinfo 已覆盖 [heap]，线程栈被 t107 排除）。
struct VmaTrace {
    sink: Option<TraceSink>,
    window_start: Instant,
    /// 上一采样点的跟踪集（按 start 升序，maps 天然有序）。
    prev: Vec<VmaEntry>,
    /// 当前采样点缓冲（与 prev 换手复用容量，稳态零新增分配）。
    cur: Vec<VmaEntry>,
    /// maps 原文缓冲（read_to_end 复用容量；路径字段可含非 UTF-8 字节，按有损读转 str）。
    buf: Vec<u8>,
    /// 打开/读 maps 失败只记一次 warn（瞬时失败不永久停用；sink 写失败的停用
    /// 在 [`write_line`]，两者独立）。
    warned: bool,
}

impl VmaTrace {
    fn new() -> Self {
        Self {
            sink: trace_sink("ZW_RENDERER_MMAP_TRACE"),
            window_start: Instant::now(),
            prev: Vec::new(),
            cur: Vec::new(),
            buf: Vec::new(),
            warned: false,
        }
    }

    fn tick(&mut self) {
        if self.sink.is_none() {
            return;
        }
        if self.window_start.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.window_start = Instant::now();
        if !self.sample() {
            return;
        }
        let events = diff_vmas(&self.prev, &self.cur);
        if !events.is_empty() {
            let ts = crate::mem_trace::epoch_ms();
            let mut out = String::new();
            for (event, entry, prev_size) in &events {
                out.clear();
                vma_event_line(&mut out, ts, *event, entry, *prev_size);
                write_line(&mut self.sink, &out);
            }
        }
        std::mem::swap(&mut self.prev, &mut self.cur);
        self.cur.clear();
    }

    /// 读并解析 maps 进 `cur`；读失败返 false（保留 prev，下一窗重试）。
    fn sample(&mut self) -> bool {
        use std::io::Read as _;
        self.buf.clear();
        let Ok(mut f) = std::fs::File::open("/proc/self/maps") else {
            if !self.warned {
                self.warned = true;
                tracing::warn!(target: "renderer_mem_diag", "ZW_RENDERER_MMAP_TRACE: cannot open /proc/self/maps");
            }
            return false;
        };
        // 有损读（永不失败）：maps 的 file-backed 路径字段可含非 UTF-8 字节，整缓冲
        // UTF-8 校验会让该构建上本通道永久静默——read_to_end + from_utf8_lossy，
        // 地址/perms 字段纯 ASCII 不受影响，lossy 只改写路径字段尾部的坏字节呈现。
        if f.read_to_end(&mut self.buf).is_err() {
            if !self.warned {
                self.warned = true;
                tracing::warn!(target: "renderer_mem_diag", "ZW_RENDERER_MMAP_TRACE: cannot read /proc/self/maps");
            }
            return false;
        }
        let text = String::from_utf8_lossy(&self.buf);
        parse_tracked_vmas(&text, &mut self.cur);
        true
    }
}

/// 解析 maps 文本，收匿名 ≥[`VMA_TRACE_THRESHOLD`] 段进 `out`（升序，按 start）。
fn parse_tracked_vmas(text: &str, out: &mut Vec<VmaEntry>) {
    out.clear();
    for line in text.lines() {
        let Some(entry) = parse_vma_line(line) else { continue };
        if entry.size() >= VMA_TRACE_THRESHOLD {
            out.push(entry);
        }
    }
}

/// 解析一行 maps；匿名段（无名或 `[anon:…]`）返 Some，file-backed 等具名段返 None。
fn parse_vma_line(line: &str) -> Option<VmaEntry> {
    // maps 行形制：`start-end perms offset dev inode [pathname]`——内核单空格
    // 分隔、路径字段可能含空格，splitn(6) 后第 6 段取整段路径。
    let mut fields = line.splitn(6, ' ');
    let range = fields.next()?;
    let perms = fields.next()?;
    fields.next()?; // offset
    fields.next()?; // dev
    fields.next()?; // inode
    let name = fields.next().unwrap_or("").trim_start();
    if !name.is_empty() && !name.starts_with("[anon:") {
        return None;
    }
    let (start_s, end_s) = range.split_once('-')?;
    let start = u64::from_str_radix(start_s, 16).ok()?;
    let end = u64::from_str_radix(end_s, 16).ok()?;
    let mut perm_bytes = [b'?'; 4];
    for (slot, b) in perm_bytes.iter_mut().zip(perms.bytes()) {
        *slot = b;
    }
    Some(VmaEntry {
        start,
        end,
        perms: perm_bytes,
        name: name.to_string(),
    })
}

/// 差分新旧跟踪集（按 `(start, perms, name)` 认同一段：同起址但权限/名字不连续
/// 是 munmap 后同址重生，按 gone+new 两事件报，不误报 grow/shrink；new/gone 必报，
/// 伸缩按 [`VMA_RESIZE_DELTA`] 报）。返 (事件, 段, 前值大小) 列表；首采样 prev 空
/// → 全 new（基线快照）。
fn diff_vmas(prev: &[VmaEntry], cur: &[VmaEntry]) -> Vec<(VmaEvent, VmaEntry, Option<u64>)> {
    let mut events = Vec::new();
    for c in cur {
        match prev
            .iter()
            .find(|p| p.start == c.start && p.perms == c.perms && p.name == c.name)
        {
            Some(p) => {
                let delta = c.end.abs_diff(p.end);
                if delta >= VMA_RESIZE_DELTA {
                    let event = if c.end > p.end {
                        VmaEvent::Grow
                    } else {
                        VmaEvent::Shrink
                    };
                    events.push((event, c.clone(), Some(p.size())));
                }
            }
            None => events.push((VmaEvent::New, c.clone(), None)),
        }
    }
    for p in prev {
        if !cur
            .iter()
            .any(|c| c.start == p.start && c.perms == p.perms && c.name == p.name)
        {
            events.push((VmaEvent::Gone, p.clone(), Some(p.size())));
        }
    }
    events
}

/// 一条 VMA 事件 JSONL（`parent64`/`off64` 供 arena 家族判别，同 t107 镜头 B）。
fn vma_event_line(out: &mut String, ts: u128, event: VmaEvent, entry: &VmaEntry, prev_size: Option<u64>) {
    use std::fmt::Write as _;
    out.clear();
    let prev_field = prev_size.map_or_else(|| "null".to_string(), |v| v.to_string());
    let _ = write!(
        out,
        "{{\"kind\":\"renderer-vma\",\"t\":{ts},\"event\":\"{}\",\"start\":\"0x{:x}\",\"end\":\"0x{:x}\",\"size\":{},\"prev_size\":{prev_field},\"perms\":\"{}\",\"name\":\"",
        event.as_str(),
        entry.start,
        entry.end,
        entry.size(),
        String::from_utf8_lossy(&entry.perms),
    );
    write_json_escaped(out, &entry.name);
    let _ = write!(
        out,
        "\",\"parent64\":\"0x{:x}\",\"off64\":{}}}",
        entry.parent64(),
        entry.start - entry.parent64(),
    );
}

/// T108 仪拍 A：把 per-space stats 写成 JSON 对象体（`"key":val,…` 形式；调用方
/// 外层包 `{"kind":"renderer-v8spaces","t":…,{体}}`）。js_worker worker 线程
/// 命令臂调用（复用共享缓冲容量，稳态零新增分配）。
pub(crate) fn format_v8_space_stats(buf: &mut String, stats: &zero_script_sandbox::HeapSpaceStats, probe_t: u128) {
    use std::fmt::Write as _;
    buf.clear();
    let _ = write!(
        buf,
        "\"probe_t\":{probe_t},\"total_heap_size\":{},\"total_physical_size\":{},\"used_heap_size\":{},\
         \"malloced_memory\":{},\"external_memory\":{},\"peak_malloced_memory\":{},\"total_allocated_bytes\":{}",
        stats.total_heap_size,
        stats.total_physical_size,
        stats.used_heap_size,
        stats.malloced_memory,
        stats.external_memory,
        stats.peak_malloced_memory,
        stats.total_allocated_bytes,
    );
    match stats.code_and_metadata_size {
        Some(v) => {
            let _ = write!(buf, ",\"code_and_metadata_size\":{v}");
        }
        None => buf.push_str(",\"code_and_metadata_size\":null"),
    }
    buf.push_str(",\"spaces\":[");
    for (index, space) in stats.spaces.iter().enumerate() {
        if index > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"name\":\"{}\",\"size\":{},\"used\":{},\"available\":{},\"physical\":{}}}",
            // 名字原位转义写进 buf（V8 空间名为固定标识符，转义是防御面）。
            {
                let mut escaped = String::new();
                write_json_escaped(&mut escaped, &space.name);
                escaped
            },
            space.size,
            space.used,
            space.available,
            space.physical,
        );
    }
    buf.push(']');
}

/// JSON 字符串体转义（`"`/`\`/控制字符），原位写入 `out`。
/// T109 G2：压还账随行字段（`,"v8_release":{…}` 追加到 v8spaces JSON 对象体尾）。
/// `count == 0`（gate 关闭或尚未触发）不追加——仪拍行与基线字节恒等。
/// js_worker worker 线程 `ProbeHeapSpaces` 命令臂调用（账为同线程本地态）。
pub(crate) fn append_v8_release_fields(buf: &mut String, account: &super::js_worker::V8ReleaseAccount) {
    if account.count == 0 {
        return;
    }
    use std::fmt::Write as _;
    let source = account
        .last_source
        .map_or("none", super::js_worker::V8ReleaseSource::as_str);
    let _ = write!(
        buf,
        ",\"v8_release\":{{\"count\":{},\"last_source\":\"{}\",\"last_elapsed_us\":{},\"last_t\":{}}}",
        account.count, source, account.last_elapsed_us, account.last_t,
    );
}

fn write_json_escaped(out: &mut String, s: &str) {
    use std::fmt::Write as _;
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 解析：匿名 ≥512KiB 进集（含 ---p 守卫——匿名保留，glibc arena 预约同形，
    /// t107 guards 面）；file-backed/小段/[heap]/线程栈具名段排除；`[anon:…]`
    /// 具名匿名段保留。
    #[test]
    fn parse_tracked_vmas_filters_threshold_and_named() {
        let text = "\
00900000-01100000 rw-p 00000000 00:00 0\n\
01100000-01210000 rw-p 00000000 00:00 0 [anon:scudo_named]\n\
02000000-02400000 rw-p 00000000 03:01 12345 /usr/lib/libfoo.so\n\
03000000-03019000 rw-p 00000000 00:00 0\n\
7f0000000-7f4000000 ---p 00000000 00:00 0\n\
7f5000000-7fd000000 rw-p 00000000 00:00 0 [heap]\n\
7fe0000000-7fe800000 rw-p 00000000 00:00 0 [stack:1234]\n";
        let mut out = Vec::new();
        parse_tracked_vmas(text, &mut out);
        let sizes: Vec<u64> = out.iter().map(|v| v.size()).collect();
        // 命中：8 MiB 无名 + 1.0625 MiB [anon:…] + 64 MiB ---p 守卫；排除：4 MiB
        // file-backed、100 KiB 小段、128 MiB [heap]、8 MiB [stack:…]（具名）。
        assert_eq!(out.len(), 3, "tracked: {out:?}");
        assert_eq!(sizes, vec![8 * 1024 * 1024, 0x1210000 - 0x1100000, 0x4000000]);
        assert_eq!(out[1].name, "[anon:scudo_named]");
        assert_eq!(out[0].perms, *b"rw-p");
        assert_eq!(out[2].perms, *b"---p");
    }

    /// 差分：首采样全 new；随后 new/gone/grow/shrink 按阈值与 Δ 阈值各就各位。
    #[test]
    fn diff_vmas_reports_all_event_kinds() {
        let base = |start: u64, end: u64| VmaEntry {
            start,
            end,
            perms: *b"rw-p",
            name: String::new(),
        };
        let v0 = vec![
            base(0x1000, 0x1000 + 20 * 1024 * 1024),
            base(0x8000, 0x8000 + 1024 * 1024),
        ];
        // 首采样：prev 空 → 全 new。
        let first = diff_vmas(&[], &v0);
        assert_eq!(
            first.iter().map(|(e, _, _)| *e).collect::<Vec<_>>(),
            vec![VmaEvent::New; 2]
        );
        // 第二采样：段 1 长大 ≥1MiB（grow）、段 2 不动（无行）、段 3 新现、段 4 缩 ≥1MiB、段 5 消失。
        let v1 = vec![
            base(0x1000, 0x1000 + 24 * 1024 * 1024),
            base(0x8000, 0x8000 + 1024 * 1024),
            base(0x90000, 0x90000 + 2 * 1024 * 1024),
            base(0xa0000, 0xa0000 + 3 * 1024 * 1024),
        ];
        let mut prev2 = vec![base(0xa0000, 0xa0000 + 5 * 1024 * 1024)];
        prev2.extend(v0.iter().cloned());
        let events = diff_vmas(&prev2, &v1);
        let mut by_start: std::collections::HashMap<u64, (VmaEvent, Option<u64>)> =
            events.into_iter().map(|(e, v, p)| (v.start, (e, p))).collect();
        assert_eq!(by_start.remove(&0x1000), Some((VmaEvent::Grow, Some(20 * 1024 * 1024))));
        assert_eq!(by_start.remove(&0x8000), None, "未动段无事件");
        assert_eq!(by_start.remove(&0x90000), Some((VmaEvent::New, None)));
        assert_eq!(
            by_start.remove(&0xa0000),
            Some((VmaEvent::Shrink, Some(5 * 1024 * 1024)))
        );
        assert!(by_start.is_empty(), "残留事件 {by_start:?}");
        // 消失段（prev 独有）在 v1 缺位 → gone。
        let events2 = diff_vmas(&prev2, &[base(0x1000, 0x1000 + 20 * 1024 * 1024)]);
        assert!(
            events2
                .iter()
                .any(|(e, v, _)| *e == VmaEvent::Gone && v.start == 0x8000)
        );
    }

    /// 伸缩 Δ <1MiB 不报（arena 顶常规移动不淹没事件流）。
    #[test]
    fn diff_vmas_ignores_subthreshold_resize() {
        let base = |start: u64, end: u64| VmaEntry {
            start,
            end,
            perms: *b"rw-p",
            name: String::new(),
        };
        let prev = vec![base(0x1000, 0x1000 + 4 * 1024 * 1024)];
        let cur = vec![base(0x1000, 0x1000 + 4 * 1024 * 1024 + 512 * 1024)];
        assert!(diff_vmas(&prev, &cur).is_empty());
    }

    /// 同起址重生（munmap 后同址重 mmap）按身份不连续降级为 gone+new 两事件：
    /// 换 perms、换名各一场景，均不误报 grow/shrink（含起止恰同时零伸缩歧义）。
    #[test]
    fn diff_vmas_same_start_rebirth_reports_gone_and_new() {
        let base = |start: u64, end: u64, perms: [u8; 4], name: &str| VmaEntry {
            start,
            end,
            perms,
            name: name.to_string(),
        };
        let start = 0x1000;
        let end = start + 4 * 1024 * 1024;
        // 场景一：同起止、perms 变（rw-p → ---p 守卫段重生）。
        let prev = vec![base(start, end, *b"rw-p", "")];
        let cur = vec![base(start, end, *b"---p", "")];
        let events = diff_vmas(&prev, &cur);
        assert_eq!(
            events.iter().map(|(e, _, _)| *e).collect::<Vec<_>>(),
            vec![VmaEvent::New, VmaEvent::Gone],
            "同址换 perms 须 gone+new，不得 grow/shrink：{events:?}"
        );
        assert_eq!(events[0].1.perms, *b"---p", "new 事件携带新段");
        assert_eq!(events[1].1.perms, *b"rw-p", "gone 事件携带旧段");
        // 场景二：同起止、具名换名（[anon:a] → [anon:b]），Δ=0 无伸缩面。
        let prev = vec![base(start, end, *b"rw-p", "[anon:a]")];
        let cur = vec![base(start, end, *b"rw-p", "[anon:b]")];
        let events = diff_vmas(&prev, &cur);
        assert_eq!(
            events.iter().map(|(e, _, _)| *e).collect::<Vec<_>>(),
            vec![VmaEvent::New, VmaEvent::Gone],
            "同址换名须 gone+new：{events:?}"
        );
        assert_eq!(events[0].1.name, "[anon:b]");
        assert_eq!(events[1].1.name, "[anon:a]");
        // 反向锁：身份连续 + 亚阈值伸缩仍无事件（守卫不误伤常规路径）。
        let cur = vec![base(start, end + 512 * 1024, *b"rw-p", "[anon:a]")];
        assert!(diff_vmas(&prev, &cur).is_empty());
    }

    /// 有损读（DF-F1 解析面近似）：缓冲含非 UTF-8 字节（file-backed 路径字段）
    /// 时解析不失败——file-backed 段照常排除、同缓冲匿名段照常入集，路径字段
    /// 坏字节以替换字符呈现。
    #[test]
    fn parse_tracked_vmas_tolerates_non_utf8_path_bytes() {
        let mut raw = b"02000000-02400000 rw-p 00000000 00:00 0\n".to_vec();
        raw.extend_from_slice(b"00900000-01100000 rw-p 00000000 00:00 0 /tmp/\xff\xfe.so\n");
        let text = String::from_utf8_lossy(&raw);
        let mut out = Vec::new();
        parse_tracked_vmas(&text, &mut out);
        assert_eq!(out.len(), 1, "匿名段不受同缓冲坏字节影响：{out:?}");
        assert_eq!(out[0].start, 0x02000000);
        // file-backed 行（坏字节在路径字段）仍按无名判定排除，不进跟踪集。
        assert!(!out.iter().any(|v| v.start == 0x00900000));
    }

    /// sink 写失败停用闩（DF-F3）：打开失败一次 warn 后 `sink` 置 None（通道停写，
    /// 不逐行重试刷日志）；成功路径 sink 保持可用且行真实落盘。
    #[test]
    fn write_line_disables_sink_on_failure() {
        // 成功路径：追加写真实文件，sink 不被停用。
        let dir = std::env::temp_dir().join("zeroweb-mem-diag-test");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("write_line_ok.jsonl");
        let mut ok = Some(TraceSink::Sink(path.clone()));
        write_line(&mut ok, "{\"k\":1}");
        assert!(ok.is_some(), "写成功不得停用 sink");
        let written = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(written, "{\"k\":1}\n");
        let _ = std::fs::remove_file(&path);
        // 打开失败（父目录已移除）：sink 被置 None，后续 tick 走 is_none 早退。
        let _ = std::fs::remove_dir(&dir);
        let mut bad = Some(TraceSink::Sink(dir.join("write_line_bad.jsonl")));
        write_line(&mut bad, "{}");
        assert!(bad.is_none(), "打开失败须停用 sink");
    }

    /// 事件行是合法 JSON 且带 arena 判别面字段。
    #[test]
    fn vma_event_line_is_valid_json() {
        // 起点 0x720e24100000：64MiB 对齐父址 0x720e24000000、块内偏移 0x100000
        // （非对齐样本，判别面字段有区分度）。
        let entry = VmaEntry {
            start: 0x720e24100000,
            end: 0x720e24100000 + 16_204_800,
            perms: *b"rw-p",
            name: "[anon:test\"q]".to_string(),
        };
        let mut out = String::new();
        vma_event_line(&mut out, 12345, VmaEvent::New, &entry, None);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(parsed["kind"], "renderer-vma");
        assert_eq!(parsed["event"], "new");
        assert_eq!(parsed["t"], 12345);
        assert_eq!(parsed["size"], 16_204_800u64);
        assert_eq!(parsed["prev_size"], serde_json::Value::Null);
        assert_eq!(parsed["parent64"], "0x720e24000000");
        assert_eq!(parsed["off64"], 0x100000u64);
        assert_eq!(parsed["name"], "[anon:test\"q]");
    }

    /// stats 对象体格式化后可拼成合法 JSON 且逐字段可读（含 code=None 与转义名）。
    #[test]
    fn format_v8_space_stats_wraps_into_valid_json() {
        let stats = zero_script_sandbox::HeapSpaceStats {
            total_heap_size: 100,
            total_physical_size: 90,
            used_heap_size: 80,
            malloced_memory: 70,
            external_memory: 60,
            peak_malloced_memory: 50,
            total_allocated_bytes: 40,
            code_and_metadata_size: None,
            spaces: vec![zero_script_sandbox::HeapSpaceStat {
                name: "new_space".to_string(),
                size: 16,
                used: 15,
                available: 1,
                physical: 14,
            }],
        };
        let mut body = String::new();
        format_v8_space_stats(&mut body, &stats, 999);
        let line = format!("{{\"kind\":\"renderer-v8spaces\",\"t\":123,{body}}}");
        let parsed: serde_json::Value = serde_json::from_str(&line).expect("valid JSON");
        assert_eq!(parsed["probe_t"], 999);
        assert_eq!(parsed["total_physical_size"], 90);
        assert_eq!(parsed["code_and_metadata_size"], serde_json::Value::Null);
        assert_eq!(parsed["spaces"][0]["name"], "new_space");
        assert_eq!(parsed["spaces"][0]["physical"], 14);
    }

    /// JSON 转义覆盖引号/反斜杠/控制字符。
    #[test]
    fn write_json_escaped_handles_specials() {
        let mut out = String::new();
        write_json_escaped(&mut out, "a\"b\\c\u{1}d");
        assert_eq!(out, "a\\\"b\\\\c\\u0001d");
    }

    /// T109 G2 压还账随行：count=0 不追加（gate 关闭＝仪拍行与基线字节恒等）；
    /// count>0 追加合法 JSON 字段且源标识就位。
    #[test]
    fn append_v8_release_fields_gates_on_count() {
        let mut buf = String::from("\"total_physical_size\":1");
        append_v8_release_fields(&mut buf, &Default::default());
        assert_eq!(buf, "\"total_physical_size\":1", "count=0 不追加任何字段");
        let account = crate::js_worker::V8ReleaseAccount {
            count: 3,
            last_source: Some(crate::js_worker::V8ReleaseSource::IdleEdge),
            last_elapsed_us: 4211,
            last_t: 12345,
        };
        append_v8_release_fields(&mut buf, &account);
        let line = format!("{{\"kind\":\"renderer-v8spaces\",\"t\":1,{buf}}}");
        let parsed: serde_json::Value = serde_json::from_str(&line).expect("valid JSON");
        assert_eq!(parsed["v8_release"]["count"], 3);
        assert_eq!(parsed["v8_release"]["last_source"], "idle");
        assert_eq!(parsed["v8_release"]["last_elapsed_us"], 4211);
        assert_eq!(parsed["v8_release"]["last_t"], 12345);
    }
}
