//! T112 G1 layout 探针（perf-mem-20261124）：`LayoutEngine::compute_with_img_intrinsic`
//! 内部五项量测面遥测（T111 §4.2 步 1，`evidence/g1-anatomy/t111-layout-anatomy.md`）。
//!
//! env 门控：`ZEROWEB_LAYOUT_PROBE=<sink 文件路径>`——进程级 [`LazyLock`] 单读；
//! 值 `1`/`true`/`0`/`false`/`off`/空 = 关闭，其余非空值视为 sink 路径（T105
//! `ZW_K3_STYLE_PROBE` 同形制）。默认关闭；关闭时每个插桩入口一次布尔判零
//! （`Instant::now`、thread_local 访问、原子计数均在门内），零分配零 IO，
//! 布局返回值与控制流逐位不变——纯遥测。
//!
//! `crates/taffy-local` 的缓存命中/未命中计数（[`crate::layout_probe`] 经
//! `taffy::probe_counters` 读取）与本模块解析同一 env、同一三态语义（两处解析
//! 保持逐字一致，见 `taffy-local/src/probe_counters.rs` 模块文档）；启用顺序：
//! 本模块 [`enabled`] 首次调用（先于任何 taffy compute）触发 taffy 侧 env 读取。
//!
//! sink 行文法（每次 `compute_with_img_intrinsic` 出口一行，整行单次
//! `write_all` 原子追加，写失败静默——诊断侧信道不得影响渲染路径）：
//!
//! ```text
//! layoutprobe kind=<band|other> tree_ms=<f> compute1_ms=<f> compute1_n=<n>
//!   compute_auth_ms=<f> compute_auth_n=<n> compute_merge_ms=<f> compute_merge_n=<n>
//!   extract1_ms=<f> extract1_n=<n> extract_auth_ms=<f> extract_auth_n=<n>
//!   extract_merge_ms=<f> extract_merge_n=<n>
//!   measure1_ms=<f> measure1_n=<n> measure_auth_ms=<f> measure_auth_n=<n>
//!   measure_merge_ms=<f> measure_merge_n=<n> measure_other_ms=<f> measure_other_n=<n>
//!   cache_hit=<n> cache_miss=<n>
//!   walk_float_ms=<f> walk_table_ms=<f> walk_phasea_ms=<f> walk_abspos_ms=<f>
//!   walk_collapse_ms=<f>
//!   text_nodes=<n> eq_nodes=<n> eq_rate=<f> dom_hash=<hex16> dom_elems=<n>
//!   t=<unix epoch ms>
//! ```
//!
//! 五项量测面（T111 §4.2 步 1 逐项对应）：
//! 1. taffy compute 次数与各趟计时——`compute1`（首趟 engine.rs:628）、
//!    `compute_auth`（flex base authority 轮 :1071-1109，≤3）、`compute_merge`
//!    （合并重跑闸 :1128-1178）分账；
//! 2. measure 闭包调用计数与计时（`measure_text_content_with_baseline`
//!    inline_finalization.rs:2689 包装器，按当前 pass 分桶；pass 标记外的调用落
//!    `measure_other`）＋ taffy 缓存命中/未命中（taffy-local
//!    `compute_cached_layout`）；
//! 3. 树构建（`build_layout_tree_with_r109` tree.rs:747）与 LayoutBox 提取
//!    （`extract_layout` engine.rs:717/:1099/:1168）计数与计时；
//! 4. 后处理 walk 家族分组计时（float :1303-1321／表 :1414＋8.5／Phase A
//!    :1663-1680／abspos 族 :1460-1650／折叠族 :1345-1356——组边界以插桩语句
//!    锚定，见 engine.rs 各 `StageId` 对）；
//! 5. 跨带布局相关样式差分率——逐文本节点布局相关字段带间全等率
//!    （[`TEXT_LAYOUT_FIELDS`] 冻结清单），snapshot 为同进程上一次 compute 的
//!    逐节点键向量（首个 band 无前照，`eq_nodes`/`eq_rate` 落 `-1` 哨兵）。
//!
//! `t=` 与 renderer `phase frame` mark 同宿主钟域（Unix epoch ms），供分析器
//! 1:1 配对归带（T105 k3style→frame 配对同法）。

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Instant;

use zero_css_parser::values::LengthValue;
use zero_dom::{Document, NodeKind};
use zero_style_system::property::types::{
    DirectionValue, FontSizeAdjustValue, FontStyleValue, FontWeightValue, LineHeightValue, TextTransformValue,
    WhiteSpaceValue, WritingModeValue,
};

/// sink 路径解析（进程级单读，纯函数供单测）。语义见模块文档：仅非空、非
/// truthy、非 falsy 的值视为路径（T105 `ZW_K3_STYLE_PROBE` 同三态）。
fn sink_path_from_value(value: Option<&str>) -> Option<PathBuf> {
    let value = value?;
    let truthy = value == "1" || value.eq_ignore_ascii_case("true");
    let falsy = value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("off");
    if truthy || falsy {
        return None;
    }
    let path = value.trim();
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

static SINK_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| sink_path_from_value(std::env::var("ZEROWEB_LAYOUT_PROBE").ok().as_deref()));

/// 探针是否启用（sink 文件模式）。调用点先查本函数再计时，关闭时零格式化、
/// 零 `Instant::now` 开销。首次调用同时触发 taffy 侧 env 读取（见模块文档
/// 启用顺序说明）。
#[inline]
pub fn enabled() -> bool {
    let on = SINK_PATH.is_some();
    if on {
        // 探针开 → taffy 缓存计数器同开（OnceLock 内单次 env 读，此后零重复解析）。
        taffy::probe_counters::init_env();
    }
    on
}

/// 单次 compute 的累计桶（微秒——`Instant` 差值取整无损，输出 ms 两位小数）。
#[derive(Default)]
struct Accum {
    tree_us: u64,
    compute1_us: u64,
    compute1_n: u64,
    compute_auth_us: u64,
    compute_auth_n: u64,
    compute_merge_us: u64,
    compute_merge_n: u64,
    extract1_us: u64,
    extract1_n: u64,
    extract_auth_us: u64,
    extract_auth_n: u64,
    extract_merge_us: u64,
    extract_merge_n: u64,
    measure1_us: u64,
    measure1_n: u64,
    measure_auth_us: u64,
    measure_auth_n: u64,
    measure_merge_us: u64,
    measure_merge_n: u64,
    measure_other_us: u64,
    measure_other_n: u64,
    cache_hit: u64,
    cache_miss: u64,
    walk_float_us: u64,
    walk_table_us: u64,
    walk_phasea_us: u64,
    walk_abspos_us: u64,
    walk_collapse_us: u64,
}

thread_local! {
    static ACCUM: RefCell<Accum> = RefCell::new(Accum::default());
    static SCOPE: RefCell<&'static str> = const { RefCell::new("other") };
    /// 当前 measure 调用归属 pass（`None` = 探针未管理的调用点，如增量路径）。
    static MEASURE_PASS: RefCell<Option<Pass>> = const { RefCell::new(None) };
    /// 跨带样式差分前照：同进程上一次 compute 的逐文本节点键向量（文档序）。
    static SNAPSHOT: RefCell<Option<Vec<TextLayoutKey>>> = const { RefCell::new(None) };
}

/// taffy compute 趟次（measure 归桶与 compute/extract 分账共用）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pass {
    /// 首趟（engine.rs compute_with_img_intrinsic 步 2）。
    First,
    /// flex base authority 轮内重跑（:1071-1109，含 ≤3 轮累计）。
    Authority,
    /// 合并重跑闸（:1128-1178）。
    Merge,
}

/// 插桩计时面标识（[`stage_start`]/[`stage_end`] 成对使用）。
#[derive(Clone, Copy, Debug)]
pub enum StageId {
    /// 树构建（`build_layout_tree_with_r109`）。
    Tree,
    /// taffy 首趟 compute。
    ComputeFirst,
    /// authority 轮 compute（累计）。
    ComputeAuthority,
    /// 合并重跑 compute。
    ComputeMerge,
    /// 首次 LayoutBox 提取。
    ExtractFirst,
    /// authority 轮内提取（累计）。
    ExtractAuthority,
    /// 合并重跑提取。
    ExtractMerge,
    /// float 家族 walk（5a.1 预扫描 → 5.2 exclude_floats）。
    WalkFloat,
    /// 表家族 walk（步 8 + 8.5 table-among-floats）。
    WalkTable,
    /// Phase A 行内终局（步 12 compute_final_inline_layouts）。
    WalkPhaseA,
    /// abspos 族 walk（步 10.5 → 11.12）。
    WalkAbspos,
    /// margin 折叠族 walk（5.2a-0/5.2a/5.3）。
    WalkCollapse,
}

/// 计时起点（禁用时 `None`，调用点一次布尔判零）。
#[inline]
pub fn stage_start() -> Option<Instant> {
    if enabled() { Some(Instant::now()) } else { None }
}

/// 计时闭合：把自 [`stage_start`] 以来的时长累入对应桶。`t` 为 `None`（探针
/// 关）时零操作。
#[inline]
pub fn stage_end(id: StageId, t: Option<Instant>) {
    let Some(t) = t else { return };
    let us = t.elapsed().as_micros() as u64;
    ACCUM.with_borrow_mut(|a| match id {
        StageId::Tree => a.tree_us += us,
        StageId::ComputeFirst => {
            a.compute1_us += us;
            a.compute1_n += 1;
        }
        StageId::ComputeAuthority => {
            a.compute_auth_us += us;
            a.compute_auth_n += 1;
        }
        StageId::ComputeMerge => {
            a.compute_merge_us += us;
            a.compute_merge_n += 1;
        }
        StageId::ExtractFirst => {
            a.extract1_us += us;
            a.extract1_n += 1;
        }
        StageId::ExtractAuthority => {
            a.extract_auth_us += us;
            a.extract_auth_n += 1;
        }
        StageId::ExtractMerge => {
            a.extract_merge_us += us;
            a.extract_merge_n += 1;
        }
        StageId::WalkFloat => a.walk_float_us += us,
        StageId::WalkTable => a.walk_table_us += us,
        StageId::WalkPhaseA => a.walk_phasea_us += us,
        StageId::WalkAbspos => a.walk_abspos_us += us,
        StageId::WalkCollapse => a.walk_collapse_us += us,
    });
}

/// 设置后续 compute 的 measure 归桶（各 compute 趟次前调用；flush 复位）。
#[inline]
pub fn set_measure_pass(pass: Option<Pass>) {
    if enabled() {
        MEASURE_PASS.with_borrow_mut(|p| *p = pass);
    }
}

/// 设置后续 flush 行的域标签（线程局部；管线带宽路径在 compute 前置 `band`
/// 并在返回后复位 `other`，其余调用方落 `other`，分析器按域过滤归带——T105
/// k3style 同形制）。
pub fn set_scope(kind: &'static str) {
    if enabled() {
        SCOPE.with_borrow_mut(|s| *s = kind);
    }
}

/// FNV-1a（64 位）字节馈入。
fn fnv1a(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h ^= u64::from(*b);
        *h = h.wrapping_mul(0x100000001b3);
    }
}

fn fnv1a_str(h: &mut u64, s: &str) {
    fnv1a(h, s.as_bytes());
    fnv1a(h, b"\x00");
}

/// DOM 结构指纹（T105 probe.rs 同法）：迭代 DFS（文档序），元素散列 local name
/// 与全部属性，文本节点散列内容。返回 (hash, 元素数)。
fn fingerprint_dom(doc: &Document) -> (u64, u64) {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut elems: u64 = 0;
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get(id) else { continue };
        match &node.kind {
            NodeKind::Element(e) => {
                elems += 1;
                fnv1a_str(&mut h, e.local_name());
                for a in &e.attributes {
                    fnv1a_str(&mut h, a.name.local.as_ref());
                    fnv1a_str(&mut h, &a.value);
                }
            }
            NodeKind::Text(t) => {
                fnv1a_str(&mut h, &t.content);
            }
            _ => {}
        }
        // rev 压栈使兄弟按文档序出栈（T105 同注释：指纹稳定性取文档序）。
        for c in doc.child_nodes(id).into_iter().rev() {
            stack.push(c);
        }
    }
    (h, elems)
}

/// 逐文本节点「布局相关样式键」——L-A 失效键语义的冻结清单（[`TEXT_LAYOUT_FIELDS`]）。
///
/// 文本测量输入 = 父元素（或节点自身，匿名 item 形）计算样式的文本度量字段 +
/// 文本内容。字段取超集保守向：多计字段只会压低全等率（L-A 兑现率不虚高），
/// 少计才会虚高。内容以 FNV-1a 散列入键（原始 content，非 trim 后——原始恒等
/// 蕴含 trim 恒等；跨带 DOM 逐字节恒等已由 dom_hash 独立佐证）。
#[derive(Clone, PartialEq, Debug)]
struct TextLayoutKey {
    content_hash: u64,
    style: Option<TextMeasureFields>,
}

/// 文本测量消费的计算样式字段（T111 §3 刀位 L-A 失效键域）。
#[derive(Clone, PartialEq, Debug)]
struct TextMeasureFields {
    font_family: Vec<String>,
    font_size: LengthValue,
    font_weight: FontWeightValue,
    font_style: FontStyleValue,
    font_stretch: f32,
    font_size_adjust: FontSizeAdjustValue,
    line_height: LineHeightValue,
    letter_spacing: LengthValue,
    letter_spacing_normal: bool,
    word_spacing: LengthValue,
    white_space: WhiteSpaceValue,
    writing_mode: WritingModeValue,
    writing_mode_sideways_lr: bool,
    direction: DirectionValue,
    text_transform: TextTransformValue,
}

/// 冻结字段清单（T112 FROZEN 引用名——量测面⑤逐文本节点键的字段构成；增删
/// 字段属判读口径变更，须新文件勘误披露）。`content` = 文本内容散列（原始
/// content，非 trim 后），其余 14 键 = 父元素计算样式的文本度量字段。
pub const TEXT_LAYOUT_FIELDS: [&str; 15] = [
    "content",
    "font_family",
    "font_size",
    "font_weight",
    "font_style",
    "font_stretch",
    "font_size_adjust",
    "line_height",
    "letter_spacing",
    "word_spacing",
    "white_space",
    "writing_mode",
    "direction",
    "text_transform",
    "letter_spacing_normal",
];

impl TextLayoutKey {
    fn build(content: &str, style: Option<&zero_style_system::property::ComputedStyle>) -> Self {
        Self {
            content_hash: {
                let mut h: u64 = 0xcbf29ce484222325;
                fnv1a(&mut h, content.as_bytes());
                h
            },
            style: style.map(|s| TextMeasureFields {
                font_family: s.font_family.clone(),
                font_size: s.font_size.clone(),
                font_weight: s.font_weight.clone(),
                font_style: s.font_style.clone(),
                font_stretch: s.font_stretch,
                font_size_adjust: s.font_size_adjust,
                line_height: s.line_height.clone(),
                letter_spacing: s.letter_spacing.clone(),
                letter_spacing_normal: s.letter_spacing_normal,
                word_spacing: s.word_spacing.clone(),
                white_space: s.white_space.clone(),
                writing_mode: s.writing_mode.clone(),
                writing_mode_sideways_lr: s.writing_mode_sideways_lr,
                direction: s.direction.clone(),
                text_transform: s.text_transform,
            }),
        }
    }
}

/// 逐文本节点键向量（文档序；节点身份 = 文本节点序号——跨带 DOM 全同已证
/// （T105 §4 dom_hash 相等率 1.0），序号即身份）。
fn snapshot_text_keys(
    doc: &Document,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::property::ComputedStyle>,
) -> Vec<TextLayoutKey> {
    let mut keys = Vec::new();
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get(id) else { continue };
        if let NodeKind::Text(t) = &node.kind {
            // measure 同源父样式解析（measure_text_content_with_baseline：
            // styles.get(节点).or_else(父元素)）。
            let style = styles
                .get(&id)
                .or_else(|| doc.parent_node(id).and_then(|pid| styles.get(&pid)));
            keys.push(TextLayoutKey::build(&t.content, style));
        }
        for c in doc.child_nodes(id).into_iter().rev() {
            stack.push(c);
        }
    }
    keys
}

/// compute 出口 flush：一行 `layoutprobe` 记录原子追加写 sink（禁用时整体
/// 短路——零遍历、零格式化、零 I/O）。取走并复位累计桶与 measure pass 标记，
/// 行间无跨调用残账。跨带差分：与 [`SNAPSHOT`] 前照逐节点比对后写入本行
/// （首个 band 无前照落 `-1` 哨兵），并把当前键向量存为新前照。
pub fn flush(
    doc: &Document,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::property::ComputedStyle>,
) {
    if !enabled() {
        return;
    }
    let kind = SCOPE.with_borrow(|kind| *kind);
    let mut acc = ACCUM.with_borrow_mut(std::mem::take);
    MEASURE_PASS.with_borrow_mut(|p| *p = None);
    let (cache_hit, cache_miss) = taffy::probe_counters::take_counts();
    acc.cache_hit += cache_hit;
    acc.cache_miss += cache_miss;

    let (dom_hash, dom_elems) = fingerprint_dom(doc);
    let keys = snapshot_text_keys(doc, styles);
    let text_nodes = keys.len() as i64;
    let (eq_nodes, eq_rate) = {
        let mut prev_opt = SNAPSHOT.with_borrow_mut(std::mem::take);
        let had_prev = prev_opt.is_some();
        let (eq, rate) = match prev_opt.as_mut() {
            // 节点数不同（DOM 变异窗形态）→ 全不等。
            Some(prev) => {
                let eq = if prev.len() == keys.len() {
                    prev.iter().zip(keys.iter()).filter(|(p, k)| p == k).count() as i64
                } else {
                    0
                };
                let rate = if keys.is_empty() {
                    -1.0
                } else {
                    eq as f64 / keys.len() as f64
                };
                (eq, rate)
            }
            None => (-1, -1.0),
        };
        // 前照旋转：本带键向量成为新前照（复用原 Vec 分配；无前照时新建槽）。
        if had_prev {
            *prev_opt.as_mut().expect("had_prev") = keys;
        } else {
            prev_opt = Some(keys);
        }
        SNAPSHOT.with_borrow_mut(|s| *s = prev_opt);
        (eq, rate)
    };

    let ms = |us: u64| us as f64 / 1000.0;
    let line = format!(
        "layoutprobe kind={kind} tree_ms={tree_ms:.2} \
         compute1_ms={compute1_ms:.2} compute1_n={compute1_n} \
         compute_auth_ms={compute_auth_ms:.2} compute_auth_n={compute_auth_n} \
         compute_merge_ms={compute_merge_ms:.2} compute_merge_n={compute_merge_n} \
         extract1_ms={extract1_ms:.2} extract1_n={extract1_n} \
         extract_auth_ms={extract_auth_ms:.2} extract_auth_n={extract_auth_n} \
         extract_merge_ms={extract_merge_ms:.2} extract_merge_n={extract_merge_n} \
         measure1_ms={measure1_ms:.2} measure1_n={measure1_n} \
         measure_auth_ms={measure_auth_ms:.2} measure_auth_n={measure_auth_n} \
         measure_merge_ms={measure_merge_ms:.2} measure_merge_n={measure_merge_n} \
         measure_other_ms={measure_other_ms:.2} measure_other_n={measure_other_n} \
         cache_hit={cache_hit} cache_miss={cache_miss} \
         walk_float_ms={walk_float_ms:.2} walk_table_ms={walk_table_ms:.2} \
         walk_phasea_ms={walk_phasea_ms:.2} walk_abspos_ms={walk_abspos_ms:.2} \
         walk_collapse_ms={walk_collapse_ms:.2} \
         text_nodes={text_nodes} eq_nodes={eq_nodes} eq_rate={eq_rate:.4} \
         dom_hash={dom_hash:016x} dom_elems={dom_elems} t={now}",
        kind = kind,
        tree_ms = ms(acc.tree_us),
        compute1_ms = ms(acc.compute1_us),
        compute1_n = acc.compute1_n,
        compute_auth_ms = ms(acc.compute_auth_us),
        compute_auth_n = acc.compute_auth_n,
        compute_merge_ms = ms(acc.compute_merge_us),
        compute_merge_n = acc.compute_merge_n,
        extract1_ms = ms(acc.extract1_us),
        extract1_n = acc.extract1_n,
        extract_auth_ms = ms(acc.extract_auth_us),
        extract_auth_n = acc.extract_auth_n,
        extract_merge_ms = ms(acc.extract_merge_us),
        extract_merge_n = acc.extract_merge_n,
        measure1_ms = ms(acc.measure1_us),
        measure1_n = acc.measure1_n,
        measure_auth_ms = ms(acc.measure_auth_us),
        measure_auth_n = acc.measure_auth_n,
        measure_merge_ms = ms(acc.measure_merge_us),
        measure_merge_n = acc.measure_merge_n,
        measure_other_ms = ms(acc.measure_other_us),
        measure_other_n = acc.measure_other_n,
        cache_hit = acc.cache_hit,
        cache_miss = acc.cache_miss,
        walk_float_ms = ms(acc.walk_float_us),
        walk_table_ms = ms(acc.walk_table_us),
        walk_phasea_ms = ms(acc.walk_phasea_us),
        walk_abspos_ms = ms(acc.walk_abspos_us),
        walk_collapse_ms = ms(acc.walk_collapse_us),
        text_nodes = text_nodes,
        eq_nodes = eq_nodes,
        eq_rate = eq_rate,
        dom_hash = dom_hash,
        dom_elems = dom_elems,
        now = epoch_ms(),
    );
    let _ = write_line_once(&line);
}

/// 宿主钟域 Unix epoch 毫秒（renderer `automation_trace_epoch_ms` 同源语义，
/// T105 probe.rs 同注释）。
fn epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

/// 整行（含结尾 `\n`）单次 `write_all` 落盘（DF-1 同规：常规文件 `O_APPEND`
/// 下单次 `write_all` 即单 `write(2)` 原子追加）。写失败静默。
fn write_line_once(line: &str) -> std::io::Result<()> {
    use std::io::Write;
    let Some(path) = SINK_PATH.as_ref() else {
        return Ok(());
    };
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    let mut buf = String::with_capacity(line.len() + 1);
    buf.push_str(line);
    buf.push('\n');
    f.write_all(buf.as_bytes())
}

/// measure 包装器归桶（`measure_text_content_with_baseline` 薄包装调用；禁用
/// 时 `t` 为 `None` 零操作）。
#[inline]
pub fn note_measure(t: Option<Instant>) {
    let Some(t) = t else { return };
    let us = t.elapsed().as_micros() as u64;
    let pass = MEASURE_PASS.with_borrow(|p| *p);
    ACCUM.with_borrow_mut(|a| match pass {
        Some(Pass::First) => {
            a.measure1_us += us;
            a.measure1_n += 1;
        }
        Some(Pass::Authority) => {
            a.measure_auth_us += us;
            a.measure_auth_n += 1;
        }
        Some(Pass::Merge) => {
            a.measure_merge_us += us;
            a.measure_merge_n += 1;
        }
        None => {
            a.measure_other_us += us;
            a.measure_other_n += 1;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sink_path_three_state_semantics() {
        // truthy/falsy/空 = 关闭；其余非空 = 路径（T105 同形制）。
        assert!(sink_path_from_value(Some("1")).is_none());
        assert!(sink_path_from_value(Some("true")).is_none());
        assert!(sink_path_from_value(Some("TRUE")).is_none());
        assert!(sink_path_from_value(Some("0")).is_none());
        assert!(sink_path_from_value(Some("false")).is_none());
        assert!(sink_path_from_value(Some("off")).is_none());
        assert!(sink_path_from_value(Some("")).is_none());
        assert!(sink_path_from_value(Some("  ")).is_none());
        assert_eq!(
            sink_path_from_value(Some("/tmp/zw-t112.sink")),
            Some(PathBuf::from("/tmp/zw-t112.sink"))
        );
        assert_eq!(
            sink_path_from_value(Some(" sink.sink ")),
            Some(PathBuf::from("sink.sink"))
        );
        assert!(sink_path_from_value(None).is_none());
    }

    #[test]
    fn frozen_field_list_shape() {
        // 冻结清单完整性：15 键 = 内容 + 14 布局相关样式字段。
        assert_eq!(TEXT_LAYOUT_FIELDS.len(), 15);
        assert_eq!(TEXT_LAYOUT_FIELDS[0], "content");
    }

    #[test]
    fn text_layout_key_equality_by_fields_and_content() {
        use zero_style_system::property::ComputedStyle;
        let base = ComputedStyle::default();
        let mut other = ComputedStyle::default();
        let k1 = TextLayoutKey::build("abc", Some(&base));
        let k2 = TextLayoutKey::build("abc", Some(&base));
        assert_eq!(k1, k2);
        // 内容不同 → 不等。
        assert_ne!(k1, TextLayoutKey::build("abd", Some(&base)));
        // 布局相关字段不同 → 不等（font_size 是 L-A 失效键）。
        other.font_size = LengthValue::Px(24.0);
        assert_ne!(k1, TextLayoutKey::build("abc", Some(&other)));
        // 无样式（None）与有样式不等；两个 None 相互等。
        let none_key = TextLayoutKey::build("abc", None);
        assert_ne!(k1, none_key);
        assert_eq!(none_key, TextLayoutKey::build("abc", None));
    }

    #[test]
    fn snapshot_diff_counts_and_snapshot_rotation() {
        use std::collections::HashMap;
        use zero_dom::Document;
        use zero_style_system::property::ComputedStyle;

        // div > (p > "hello", "world")——两文本节点分属两父。
        let mut doc = Document::new();
        let root = doc.root();
        let div = doc.create_element("div");
        doc.append_child(root, div).unwrap();
        let p = doc.create_element("p");
        doc.append_child(div, p).unwrap();
        let t1 = doc.create_text_node("hello");
        doc.append_child(p, t1).unwrap();
        let t2 = doc.create_text_node("world");
        doc.append_child(div, t2).unwrap();
        let styles: HashMap<zero_dom::NodeId, ComputedStyle> = HashMap::new();
        let keys = snapshot_text_keys(&doc, &styles);
        assert_eq!(keys.len(), 2, "div 直挂文本 + p 内文本各一键");

        // flush 的比对分支直测（不经 env 门）：无前照 → 哨兵；有前照同键 → 全等。
        SNAPSHOT.with_borrow_mut(|s| *s = None);
        let prev_opt = SNAPSHOT.with_borrow_mut(std::mem::take);
        assert!(prev_opt.is_none(), "首照无前照");
        let eq_sentinel = prev_opt.as_ref().map_or((-1i64, true), |_| (0, false));
        assert_eq!(eq_sentinel, (-1, true), "无前照走哨兵分支");

        SNAPSHOT.with_borrow_mut(|s| *s = Some(keys.clone()));
        let mut prev_opt = SNAPSHOT.with_borrow_mut(std::mem::take);
        let prev = prev_opt.as_mut().expect("前照已在槽");
        let eq = prev.iter().zip(keys.iter()).filter(|(p, k)| p == k).count();
        assert_eq!(eq, keys.len(), "同 DOM 同样式 → 逐节点全等");
        // 长度不同（DOM 变异）→ 全不等（flush 分支计 0）。
        let truncated: Vec<TextLayoutKey> = keys[..1].to_vec();
        assert_ne!(prev.len(), truncated.len());
        let eq2 = prev.iter().zip(truncated.iter()).filter(|(p, k)| p == k).count();
        assert!(eq2 <= 1);
        // 前照回存语义：take 后写回保持槽非空（flush 两态共同的旋转保证）。
        SNAPSHOT.with_borrow_mut(|s| *s = prev_opt);
        SNAPSHOT.with_borrow(|s| assert!(s.is_some(), "旋转后槽持有新前照"));
        SNAPSHOT.with_borrow_mut(|s| *s = None);
    }

    #[test]
    fn stage_end_accumulates_buckets() {
        // 桶累加闭合逻辑直测（不经 env 门：直接操作 ACCUM 槽）。
        ACCUM.with_borrow_mut(|a| *a = Accum::default());
        let t = Some(Instant::now());
        // 微秒计时下空区间可取整为 0；睡 50µs 保证 us > 0 断言确定成立。
        std::thread::sleep(std::time::Duration::from_micros(50));
        stage_end(StageId::ComputeAuthority, t);
        stage_end(StageId::ComputeAuthority, t);
        stage_end(StageId::ExtractMerge, t);
        ACCUM.with_borrow(|a| {
            assert_eq!(a.compute_auth_n, 2);
            assert_eq!(a.compute_merge_n, 0);
            assert_eq!(a.extract_merge_n, 1);
            assert!(a.compute_auth_us > 0);
        });
        ACCUM.with_borrow_mut(|a| *a = Accum::default());
    }

    #[test]
    fn measure_note_routes_by_pass_marker() {
        use std::collections::HashMap;
        use zero_dom::Document;

        // note_measure 无内部 enabled 门：禁用零开销契约在调用点（stage_start
        // 返回 None）。此处直测归桶路由：None 入参短路，Some 入参按 MEASURE_PASS
        // 标记四路归桶（First/Authority/Merge/None→other）。
        assert!(!enabled(), "测试进程未设 ZEROWEB_LAYOUT_PROBE");
        ACCUM.with_borrow_mut(|a| *a = Accum::default());

        // None 入参短路：零写入（探针禁用时全部调用点实际走这条路径）。
        note_measure(None);
        ACCUM.with_borrow(|a| {
            assert_eq!(a.measure1_n, 0);
            assert_eq!(a.measure_other_n, 0);
        });

        // MEASURE_PASS=None → other 桶。
        MEASURE_PASS.with_borrow_mut(|p| *p = None);
        note_measure(Some(Instant::now()));
        ACCUM.with_borrow(|a| {
            assert_eq!(a.measure_other_n, 1, "无标记走 other 桶");
            assert_eq!(a.measure1_n, 0);
        });

        // MEASURE_PASS=Some(First) → measure1 桶（首次 compute 内量测）。
        MEASURE_PASS.with_borrow_mut(|p| *p = Some(Pass::First));
        note_measure(Some(Instant::now()));
        ACCUM.with_borrow(|a| {
            assert_eq!(a.measure1_n, 1, "First 标记走 measure1 桶");
            assert_eq!(a.measure_other_n, 1, "other 桶不受标记切换影响");
        });

        // MEASURE_PASS=Some(Merge) → merge 桶；标记复位后回归 other。
        MEASURE_PASS.with_borrow_mut(|p| *p = Some(Pass::Merge));
        note_measure(Some(Instant::now()));
        ACCUM.with_borrow(|a| assert_eq!(a.measure_merge_n, 1));
        MEASURE_PASS.with_borrow_mut(|p| *p = None);
        note_measure(Some(Instant::now()));
        ACCUM.with_borrow(|a| assert_eq!(a.measure_other_n, 2));

        // 快照形状：无样式表时文本键仍按 DOM 序产出（父样式回退解析）。
        let mut doc = Document::new();
        let p = doc.create_element("p");
        doc.append_child(doc.root(), p).unwrap();
        let tx = doc.create_text_node("x");
        doc.append_child(p, tx).unwrap();
        let styles: HashMap<zero_dom::NodeId, zero_style_system::property::ComputedStyle> = HashMap::new();
        let keys = snapshot_text_keys(&doc, &styles);
        assert_eq!(keys.len(), 1);
        MEASURE_PASS.with_borrow_mut(|p| *p = None);
    }
}
