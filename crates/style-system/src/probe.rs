//! K3 style 探针（T105 perf-mem-20261124 G1-open-speed）：`compute_styles`
//! 内部三分计时遥测（selector matching / cascade / computed value 分相）。
//!
//! env 门控：`ZW_K3_STYLE_PROBE=<sink 文件路径>`——进程级 LazyLock 单读（与
//! `ZW_SELECT_LISTBOX_FORM` 同形制）；值 `1`/`true`/`0`/`false`/`off`/空 = 关闭，
//! 其余非空值视为 sink 路径（census sink [`crate` 外同族] 三态语义的去 tracing
//! 简化：本探针无 truthy→tracing 模式，`1` 只表示关闭，避免在 cwd 落名为 `1`
//! 的垃圾文件）。默认关闭，关闭时每元素仅一次 bool 判断（`Instant::now` 与
//! thread_local 访问均在门内），样式计算返回值与控制流逐位不变——纯遥测。
//!
//! sink 行文法（每次全量/增量 compute 出口一行，整行单次 `write_all` 原子追加，
//! 写失败静默——诊断侧信道不得影响渲染路径）：
//!
//! ```text
//! k3style kind=<域标签> wall_ms=<f> match_ms=<f> cascade_ms=<f> computed_ms=<f>
//!          elements=<n> pseudo=<n> style_cache_hit=<n> cascade_hit=<n> cascade_miss=<n>
//!          sheets=<n> rules=<n> sheet_hash=<hex16> dom_hash=<hex16> dom_elems=<n>
//!          text_nodes=<n> text_chars=<n> t=<unix epoch ms>
//! ```
//!
//! 分相边界（[`StyleSystem::compute_element_style_internal`] 内三处闭合 mark）：
//! - `match_ms` — 入口 → 声明收集完（matcher 主匹配 + 宿主段 + slotted 段）；
//! - `cascade_ms` — 上一边界 → `CascadeOutcome` 产出（表现提示/内联/UA 声明装配、
//!   attr()/var() 代入、级联缓存查找、简写展开、`cascade_with_orders`）；
//! - `computed_ms` — 上一边界 → 元素出口（自定义属性 gather/var 解析、继承、
//!   计算值解析与修正）。伪元素调用经 `pseudo` 计数单独披露，金额并入三桶。
//! `wall_ms` = 三桶分相累计之和（`match+cascade+computed`，即探针计时覆盖面；
//! 恒等三桶和，不含树走查粘合与规则索引构建等桶外成本——桶外粘合面规模可由
//! 渲染带 `style_ms` 与本值之差间接估计）。`t=` 与 renderer `phase` mark 同宿主
//! 钟域（Unix epoch ms，禁沙箱 JS 钟），供分析器与 `phase frame` 行配对归带。
//!
//! `dom_hash`/`dom_elems`/`text_nodes`/`text_chars` = flush 时 DOM 结构指纹
//! （tag+attrs+文本内容 FNV-1a，文档序迭代 DFS）；`sheets`/`rules`/`sheet_hash`
//! = 样式表集签名。两者合成本任务「跨带输入恒等面」的逐带测量：skeleton 带
//! 与 styled 带对同一 DOM 先后全量重算，指纹相等性直接刻画可复用面。

use std::cell::RefCell;
use std::time::Instant;

/// sink 路径解析（进程级单读）。语义见模块文档：仅非空、非 truthy、非 falsy
/// 的值视为路径。
fn sink_path_from_env() -> Option<std::path::PathBuf> {
    let value = std::env::var("ZW_K3_STYLE_PROBE").ok()?;
    let truthy = value == "1" || value.eq_ignore_ascii_case("true");
    let falsy = value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("off");
    if truthy || falsy {
        return None;
    }
    let path = value.trim();
    if path.is_empty() {
        return None;
    }
    Some(std::path::PathBuf::from(path))
}

static SINK_PATH: std::sync::LazyLock<Option<std::path::PathBuf>> = std::sync::LazyLock::new(sink_path_from_env);

/// 探针是否启用（sink 文件模式）。调用点先查本函数再计时，关闭时零格式化、
/// 零 `Instant::now` 开销。
#[inline]
pub fn enabled() -> bool {
    SINK_PATH.is_some()
}

/// 单次 compute 的分相累计桶（微秒——`Instant` 差值取整无损，输出 ms 两位小数）。
#[derive(Default)]
struct Accum {
    match_us: u64,
    cascade_us: u64,
    computed_us: u64,
    elements: u64,
    pseudo: u64,
    style_cache_hit: u64,
    cascade_hit: u64,
    cascade_miss: u64,
}

thread_local! {
    static ACCUM: RefCell<Accum> = RefCell::new(Accum::default());
    static SCOPE: RefCell<&'static str> = const { RefCell::new("other") };
}

/// 单元素三分计时器（[`enabled`] 关闭时恒 `None`，调用方 `if let Some` 跳过）。
/// `last` 槽实现分段闭合：每次 mark 把自上一 mark 以来的时长累入对应桶。
pub struct ElemTimer {
    last: Instant,
}

impl ElemTimer {
    fn close(&mut self) -> u64 {
        let now = Instant::now();
        let us = now.saturating_duration_since(self.last).as_micros() as u64;
        self.last = now;
        us
    }

    /// match 相闭合（声明收集完；入口构造与 matcher 调用同桶）。
    pub fn mark_match(&mut self) {
        let us = self.close();
        ACCUM.with_borrow_mut(|a| a.match_us += us);
    }

    /// cascade 相闭合（`CascadeOutcome` 产出）。
    pub fn mark_cascade(&mut self) {
        let us = self.close();
        ACCUM.with_borrow_mut(|a| a.cascade_us += us);
    }

    /// 元素出口：computed 相闭合 + 元素计数（伪元素调用单独计数，金额同入桶）。
    pub fn finish(mut self, pseudo: bool) {
        let us = self.close();
        ACCUM.with_borrow_mut(|a| {
            a.computed_us += us;
            if pseudo {
                a.pseudo += 1;
            } else {
                a.elements += 1;
            }
        });
    }
}

/// 元素计算入口取计时器（禁用时返回 `None`，零 `Instant::now` 开销）。
#[inline]
pub fn elem_enter() -> Option<ElemTimer> {
    if enabled() {
        Some(ElemTimer { last: Instant::now() })
    } else {
        None
    }
}

/// S11 元素样式键缓存命中（跳过了整段元素计算，不进 `elements`）。
pub fn note_style_cache_hit() {
    if enabled() {
        ACCUM.with_borrow_mut(|a| a.style_cache_hit += 1);
    }
}

/// 级联缓存查找结果（命中/未命中各一桶；`author_attr_active` 强制跳过查找的
/// 臂不计，由调用方门控）。
pub fn note_cascade_cache(hit: bool) {
    if enabled() {
        ACCUM.with_borrow_mut(|a| {
            if hit {
                a.cascade_hit += 1;
            } else {
                a.cascade_miss += 1;
            }
        });
    }
}

/// 设置后续 flush 行的域标签（线程局部，跨 flush 保持）。管线带宽路径在调用
/// `compute_styles` 前置 `band` 并在返回后复位 `other`——gCS 桥/pending view/
/// 有限矩形重排等其余全量调用方落 `other` 域，分析器按域过滤归带。
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

/// DOM 结构指纹：迭代 DFS（文档序），元素散列 local name 与全部属性
///（name/value），文本节点散列内容。返回 (hash, 元素数, 文本节点数, 文本字符数)。
fn fingerprint_dom(doc: &zero_dom::Document) -> (u64, u64, u64, u64) {
    let mut h: u64 = 0xcbf29ce484222325;
    let (mut elems, mut tnodes, mut tchars): (u64, u64, u64) = (0, 0, 0);
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get(id) else {
            continue;
        };
        match &node.kind {
            zero_dom::NodeKind::Element(e) => {
                elems += 1;
                fnv1a_str(&mut h, e.local_name());
                for a in &e.attributes {
                    fnv1a_str(&mut h, a.name.local.as_ref());
                    fnv1a_str(&mut h, &a.value);
                }
            }
            zero_dom::NodeKind::Text(t) => {
                tnodes += 1;
                tchars += t.content.len() as u64;
                fnv1a_str(&mut h, &t.content);
            }
            _ => {}
        }
        // rev 压栈使兄弟按文档序出栈——同构 DOM 同指纹与遍历序约定无关，
        // 文档序仅为指纹稳定性（同页两带对比）取正序。
        for c in doc.child_nodes(id).into_iter().rev() {
            stack.push(c);
        }
    }
    (h, elems, tnodes, tchars)
}

/// 样式表集签名：递归走 Style/At/Keyframes/Layer/Supports/Container 规则树，
/// 散列声明 property/value/important 与每规则一条结构记号；选择器以条数入账
///（签名目标 = 集合变体区分，非逐字序列化）。返回 (表数, 规则数, hash)。
fn fingerprint_sheets(stylesheets: &[zero_css_parser::Stylesheet]) -> (u64, u64, u64) {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut rules: u64 = 0;
    fnv1a(&mut h, &stylesheets.len().to_le_bytes());
    for s in stylesheets {
        hash_rules(&mut h, &s.rules, &mut rules);
    }
    (stylesheets.len() as u64, rules, h)
}

fn hash_rules(h: &mut u64, rules: &[zero_css_parser::Rule], count: &mut u64) {
    use zero_css_parser::Rule;
    for r in rules {
        *count += 1;
        match r {
            Rule::Style(s) => {
                fnv1a(h, b"S");
                fnv1a(h, &s.selectors.len().to_le_bytes());
                for d in &s.declarations {
                    fnv1a_str(h, &d.property);
                    fnv1a_str(h, &d.value);
                    if d.important {
                        fnv1a(h, b"!");
                    }
                }
            }
            Rule::At(a) => {
                // AtRule 体两形：Block(嵌套规则) 递归；Statement（@import 等）只记名。
                fnv1a(h, b"A");
                fnv1a_str(h, &a.name);
                if let zero_css_parser::ast::AtRuleBody::Block(rs) = &a.body {
                    hash_rules(h, rs, count);
                }
            }
            Rule::Keyframes(k) => {
                fnv1a(h, b"K");
                fnv1a_str(h, &k.name);
                for kf in &k.keyframes {
                    fnv1a(h, &kf.selectors.len().to_le_bytes());
                    for d in &kf.declarations {
                        fnv1a_str(h, &d.property);
                        fnv1a_str(h, &d.value);
                    }
                }
            }
            Rule::Layer(l) => {
                fnv1a(h, b"L");
                hash_rules(h, &l.rules, count);
            }
            Rule::Supports(s) => {
                fnv1a(h, b"@");
                hash_rules(h, &s.rules, count);
            }
            Rule::Container(c) => {
                fnv1a(h, b"C");
                hash_rules(h, &c.rules, count);
            }
            _ => {
                fnv1a(h, b"?");
            }
        }
    }
}

/// compute 出口 flush：一行 `k3style` 记录原子追加写 sink（禁用时整体短路——
/// 零遍历、零格式化、零 I/O）。取走并复位累计桶，行间无跨调用残账。
pub fn flush(doc: &zero_dom::Document, stylesheets: &[zero_css_parser::Stylesheet]) {
    if !enabled() {
        return;
    }
    let kind = SCOPE.with_borrow(|kind| *kind);
    let acc = ACCUM.with_borrow_mut(std::mem::take);
    let (dom_hash, dom_elems, text_nodes, text_chars) = fingerprint_dom(doc);
    let (sheets, rules, sheet_hash) = fingerprint_sheets(stylesheets);
    let ms = |us: u64| us as f64 / 1000.0;
    let wall_ms = ms(acc.match_us + acc.cascade_us + acc.computed_us);
    let match_ms = ms(acc.match_us);
    let cascade_ms = ms(acc.cascade_us);
    let computed_ms = ms(acc.computed_us);
    let line = format!(
        "k3style kind={kind} wall_ms={wall_ms:.2} match_ms={match_ms:.2} cascade_ms={cascade_ms:.2} \
         computed_ms={computed_ms:.2} elements={elements} pseudo={pseudo} \
         style_cache_hit={style_cache_hit} cascade_hit={cascade_hit} cascade_miss={cascade_miss} \
         sheets={sheets} rules={rules} sheet_hash={sheet_hash:016x} dom_hash={dom_hash:016x} \
         dom_elems={dom_elems} text_nodes={text_nodes} text_chars={text_chars} t={now}",
        kind = kind,
        wall_ms = wall_ms,
        match_ms = match_ms,
        cascade_ms = cascade_ms,
        computed_ms = computed_ms,
        elements = acc.elements,
        pseudo = acc.pseudo,
        style_cache_hit = acc.style_cache_hit,
        cascade_hit = acc.cascade_hit,
        cascade_miss = acc.cascade_miss,
        sheets = sheets,
        rules = rules,
        sheet_hash = sheet_hash,
        dom_hash = dom_hash,
        dom_elems = dom_elems,
        text_nodes = text_nodes,
        text_chars = text_chars,
        now = epoch_ms(),
    );
    let _ = write_line_once(&line);
}

/// 宿主钟域 Unix epoch 毫秒（renderer `automation_trace_epoch_ms` 同源语义——
/// 该函数在 apps/renderer，crate 依赖不可达，此处按同一定义现取）。
fn epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

/// 整行（含结尾 `\n`）单次 `write_all` 落盘（DF-1 同规：`writeln!` 拆两次
/// `write` 在多写者并发下可产生撕裂合并行；常规文件 `O_APPEND` 下单次
/// `write_all` 即单 `write(2)` 原子追加）。写失败静默。
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
