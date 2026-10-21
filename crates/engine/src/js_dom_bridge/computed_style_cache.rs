//! getComputedStyle per-generation `(Document, styles)` 缓存——真站 js worker 停摆（E14）
//! 根因修复。从 [`super::computed_style`] 拆出以控制主文件行数。
//!
//! [`super::computed_style::compute_document_styles_with_inline_overrides`] 每次全量
//! parse+cascade（真实大页面单次 ~600ms）；生产回调 `__zw_get_computed_style` 因
//! `Document` 非 `Send`（observer/listener 闭包 + html5ever tendril `Cell`）不能把
//! parsed doc 放进 `Send + Sync` 回调闭包缓存，退化为**每新 selector 重算一次**——
//! 真站（bilibili 首页）轮播/懒加载每 tick 量测几十个新元素，每个 30s 臂 15-48 次
//! 全量重算，js worker 被 30s V8 watchdog 击杀占死，CDP evaluate 全部排队超时
//! （E14 诊断证据 hang-diag-r9~r13：170 次慢调用均值 628ms，合计 106.8s/轮）。
//!
//! **解法**：host 回调只在 js worker 单线程执行（`register_callback` 闭包经
//! `host_callback_invoke` 于脚本执行期同步调用）——`thread_local` 持有
//! `(html, drain_gen, style_version)` 代际键下的 parsed doc + 全文档计算样式，规避
//! `Send` 约束。同代际内所有 selector 查询摊销为一次 parse+cascade。
//!
//! **键契约**（与视图缓存精确命中键同构，见 callbacks.rs `MUT_DRAIN_GEN` 注）：
//! `drain_gen`（mutation 队列 drain 代际）把「drain 后重长回同 len 但内容不同」的
//! 批次隔到不同代；`style_version`（R3030 mutation 计数）在单个 drain 代内单调递增，
//! len 相等 ⇒ 队列前缀相同；`html` 全文兜底快照换代。
//!
//! **三级换代（t6 reflow-storm）**：轮询站点（suggest 面板等）每 tick 写一两个
//! attr/style 后即 gCS——旧实现任一键变化都全量 parse+cascade（8k 节点页单次
//! ~250ms），40Hz 轮询把 js worker 打满、页面 evaluate 挂死。现按变化来源分三级：
//! - 代际未变 → 纯命中（0.01ms 级）；
//! - html 同、drain_gen / style_version 前进（仅 pending attr/style 批次变化）→
//!   复用 cached doc：幂等 replay（apply_inline_style_overrides 四类 apply 均幂等，
//!   latest-wins 由顺序保持）+ 变更子树增量 cascade；
//! - html 变（结构变化 / drain 后快照重序列化——**轮询站点的每 tick 主通道**）→
//!   先试 drain 记录同步：`apply_recorded_mutations` 发布「本批 drained mutations」，
//!   gCS 侧把 cached doc 经同一权威 applier（apply_dom_mutations）推进到新代 +
//!   变更子树增量 cascade，免全量 re-parse；记录缺失/不衔接/批次含不支持的变体 →
//!   全量 parse 兜底（正确性不依赖同步成功）。
//!
//! 增量 cascade 用 [`StyleSystem::compute_styles_incremental`]（祖先链恢复继承，
//! `:has()` 内部自动退化全量）+ 固有尺寸覆盖重跑。
//!
//! webview 文档换代就地 clear 且不 bump drain_gen 的已知例外，由
//! [`register_dom_callbacks`] 每次注册时调 [`clear_generation_cache`] 覆盖（注册装新
//! 快照 → 旧代际不得复用；同线程先后多沙箱同理隔离）。
//!
//! **内存驻留**：单槽有界、换代即 drop 旧代际，但在 js worker 线程存活期内持有最后
//! 一代的整页 `Document` + 全文档 styles + html 全文拷贝（重页页级内存常驻，直到该
//! 线程下一次 gCS 或线程结束）——相对旧实现（仅 per-selector 值）的内存形态阶跃，
//! 换取 E14 停摆消除。

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_style_system::{ComputedStyle, StyleSystem};

use super::{
    DomMutation,
    computed_style::{
        apply_inline_style_overrides, apply_intrinsic_size_overrides, compute_document_styles_with_inline_overrides_css,
    },
};
use crate::pipeline::collect_stylesheets_with_parsed_external;

/// drain 应用记录（t6）：`apply_recorded_mutations` 成功路径发布，gCS 代际缓存于
/// js 线程消费——把「cached doc + 本批 drained mutations」推进到新代，免全量 re-parse
///（轮询站点每 tick 结构性写 DOM → 快照重序列化 → html 键必变 → 旧实现每 tick 全量
/// parse ~200ms 打满 js worker）。`Arc<str>` 避免 40Hz drain 下整页字符串双份克隆。
pub struct GcsDrainRecord {
    /// 应用前快照（应与 gCS 槽当前 html 一致才可同步）。
    pub old_html: Arc<str>,
    /// 应用后新快照（发布方产出的序列化结果）。
    pub new_html: Arc<str>,
    /// 本批 drained mutations（已滤 FocusChanged）。
    pub mutations: Vec<DomMutation>,
}

static DRAIN_RECORD: Mutex<Option<Arc<GcsDrainRecord>>> = Mutex::new(None);

thread_local! {
    /// 本代外部样式表文本（P9-9/R5107）：页面 `<link rel=stylesheet>` 聚合 CSS
    /// （webview `cached_css`），经 `SetDomSnapshot` 钉版下发、js worker 线程装填。
    /// getComputedStyle 合成源（[`crate::pipeline::collect_stylesheets`] 的 `css`
    /// 参数）与布局管线（`session.css` 同一文本）同源——JS 读样式与布局级联一致。
    /// 空串 = 无外部 CSS（纯快照/测试线程、LoadHtml 内联形态），行为与修复前逐位一致。
    /// 与代际槽同架构（host 回调仅 js worker 单线程执行，thread_local 规避 `Send`）。
    static ACTIVE_EXTERNAL_CSS: RefCell<Arc<str>> = RefCell::new(Arc::from(""));
}

/// js worker `SetDomSnapshot` 臂调用：装本代外部 CSS。每次快照无条件覆盖
///（含清空——跨文档残留不可能存活），先于同代 gCS 查询生效（同通道 FIFO）。
pub fn set_active_external_css(css: &str) {
    ACTIVE_EXTERNAL_CSS.with(|cell| *cell.borrow_mut() = Arc::from(css));
}

thread_local! {
    /// reftest harness（wpt-runner）case 级 adopted 覆盖（R-t2qq 返修 F2）：rayon 并行下
    /// 每案在**单 worker 线程**内完成「页面脚本 + 渲染」（`render_with_layout_inner` /
    /// webview `load_html` 均同步执行），adopted 文本经本覆盖直达同线程 gCS 与 collect，
    /// **不写**进程级 [`ACTIVE_ADOPTED_CSS`]——并行 case 互不可见、跨案零残留
    ///（adopted reftest 案用裸 `div` 选择器，若经进程级槽，脚本窗口内会污染并行案的
    /// 渲染级联、本案值也会被并行案的首清截断）。生产路径恒 `None`（仅 wpt-runner
    /// 调用 [`set_active_adopted_css_render_override`]），读进程级槽（D1 共享语义）。
    static ADOPTED_CSS_RENDER_OVERRIDE: RefCell<Option<Arc<str>>> = const { RefCell::new(None) };
}

/// reftest harness（wpt-runner）case 级装填：`Some(text)` = 本线程后续 gCS 合成与
/// collect 级联的 adopted 源；`None` = 清除（case 首尾清空），回落进程级
/// [`ACTIVE_ADOPTED_CSS`]。线程局部 ⇒ 并行 reftest case 天然隔离。
pub fn set_active_adopted_css_render_override(css: Option<&str>) {
    ADOPTED_CSS_RENDER_OVERRIDE.with(|cell| *cell.borrow_mut() = css.map(Arc::from));
}

/// 本代 adopted 样式表规则文本（constructable stylesheets）的**进程级共享槽**
///（R-t2qq 返修 D1：原 thread_local 形态下写入方 `__zw_set_adopted_css` 宿主回调在
/// js worker 线程执行，而生产级联 [`crate::pipeline::collect_stylesheets`] 在
/// renderer 渲染线程执行——thread_local 令渲染级联永不含 adopted 规则，gCS 与
/// paint 系统性分歧，reftest 全绿只是 harness 同线程执行的假象）。`RwLock<Arc<str>>`
/// 换取跨线程可见；值仍 `Arc<str>`（generation/cs_cache 缓存键 Arc 比对逻辑不变）。
/// 页面 `document.adoptedStyleSheets` / `ShadowRoot.adoptedStyleSheets` 内全部规则
/// cssText 由 shim 汇总，经 `__zw_set_adopted_css` 回调下发。getComputedStyle 合成源
/// 在 document `<style>` 表**之后**级联（CSSOM author 源顺序，
/// https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets）。
/// 空串 = 无 adopted 表；跨代残留清空与 [`ACTIVE_EXTERNAL_CSS`] 同点同法
///（js worker `SetDomSnapshot` 无条件覆盖，tab worker 同点镜像）。
static ACTIVE_ADOPTED_CSS: LazyLock<RwLock<Arc<str>>> = LazyLock::new(|| RwLock::new(Arc::from("")));

/// js worker `__zw_set_adopted_css` 回调：装本代 adopted 规则文本。无条件覆盖
///（shim 每次数组/规则变更全量重推，非增量）。进程级槽——js worker 线程写、
/// 渲染线程读（含清空，[`ACTIVE_ADOPTED_CSS`] 文档注）。
pub fn set_active_adopted_css(css: &str) {
    if let Ok(mut slot) = ACTIVE_ADOPTED_CSS.write() {
        *slot = Arc::from(css);
    }
}

/// gCS 合成源 / collect 级联消费点读取当前代 adopted 规则文本：case 级线程局部覆盖
/// 优先（reftest harness，见 [`ADOPTED_CSS_RENDER_OVERRIDE`]），否则进程级槽。
/// 空串 = 无 adopted 表。
pub(crate) fn active_adopted_css() -> Arc<str> {
    if let Some(v) = ADOPTED_CSS_RENDER_OVERRIDE.with(|cell| cell.borrow().clone()) {
        return v;
    }
    ACTIVE_ADOPTED_CSS
        .read()
        .map(|slot| Arc::clone(&slot))
        .unwrap_or_else(|_| Arc::from(""))
}

/// M2-S2a 查询覆盖提供者槽（goal compositor-animation-continuity，RFC
/// `docs/goal/compositor-animation-continuity/m2-slice-query-consistency.md` §2.1）：
/// flag 开启且动画经 compositor 采样时，`getComputedStyle` 对已导出通道返回与合成
/// 视觉一致的采样值。engine 不引 zero-protocol 依赖——提供者由 renderer 以
/// `Arc<dyn Fn>` 注册（输入元素结构路径 + 属性名，输出**整体替代**的序列化 CSS 值；
/// 与 compositor「干净基线上采样」镜像，禁止与底层级联混合）。未注册（flag 关）
/// 时查询路径行为与现状逐字节一致（S2 FR-9 回滚纪律）。
type QueryStyleOverrideFn = dyn Fn(&str, &str) -> Option<String> + Send + Sync;

static QUERY_STYLE_OVERRIDE: LazyLock<RwLock<Option<Arc<QueryStyleOverrideFn>>>> = LazyLock::new(|| RwLock::new(None));

/// renderer 侧装填/撤除查询覆盖提供者（`None` = 撤除：deactivate、导航换代清表、
/// flag 关初始化）。进程级槽——renderer 线程写、js worker gCS 回调线程读，
/// 与 [`ACTIVE_ADOPTED_CSS`] 同型跨线程可见模式。
pub fn set_query_style_override_provider(provider: Option<Arc<QueryStyleOverrideFn>>) {
    if let Ok(mut slot) = QUERY_STYLE_OVERRIDE.write() {
        *slot = provider;
    }
}

/// gCS 回调顶部咨询：命中返回采样值（先于级联与缓存机制——动画属性零底层计算），
/// 未命中 `None` 回落既有级联路径零干扰。
pub(crate) fn query_style_override(selector: &str, property: &str) -> Option<String> {
    let provider = QUERY_STYLE_OVERRIDE.read().ok().and_then(|slot| slot.clone())?;
    provider(selector, property)
}

/// gCS 合成源消费点读取当前代外部 CSS。`Arc` 克隆——持有跨 collect 过程，避免
/// 借用跨 re-entrant 调用（collect 内部无回流，防御性约束与代际槽 `f` 契约同向）。
pub(crate) fn active_external_css() -> Arc<str> {
    ACTIVE_EXTERNAL_CSS.with(|cell| cell.borrow().clone())
}

thread_local! {
    /// 外部 CSS 解析记忆化（P9-9/R5107 定向门禁缓解）：全量兜底路径与 sheets 脏
    /// 重建每代重跑 [`collect_stylesheets`]，css 文本不变也重复 tokenize+parse
    /// （200KB 量级 parse ~9ms/代 → 代际耗时 4.9x 超阈值；克隆 AST ~1ms）。按精确
    /// css 文本存单份解析结果，命中克隆注入。与 ACTIVE_EXTERNAL_CSS 同架构。
    static EXTERNAL_CSS_SHEET_MEMO: RefCell<Option<(Arc<str>, Arc<zero_css_parser::Stylesheet>)>> =
        const { RefCell::new(None) };
}

/// 取（或建立）css 文本对应的已解析外部表：命中克隆 AST，未命中解析并存档再克隆
/// （每 css 版本多付一次克隆，换同版本后续代免 parse）。空串返 `None`（无外部表，
/// collect 侧同语义）；键为精确文本，css 换版即重解析，无 stale 面。
pub(super) fn memoized_external_sheet(css: &str) -> Option<zero_css_parser::Stylesheet> {
    if css.is_empty() {
        return None;
    }
    EXTERNAL_CSS_SHEET_MEMO.with(|cell| {
        let mut memo = cell.borrow_mut();
        if let Some((key, sheet)) = memo.as_ref()
            && **key == *css
        {
            return Some((**sheet).clone());
        }
        let sheet = Arc::new(zero_css_parser::Parser::parse_stylesheet(css));
        *memo = Some((Arc::from(css), Arc::clone(&sheet)));
        Some((*sheet).clone())
    })
}

/// drain 站点（`apply_recorded_mutations` webview/HTML 回写两成功路径）发布本批
/// 应用记录。只留最新一条（gCS 只消费「从自己当前代出发」的一次推进；中间多代
/// 未被消费则该槽错过同步，走全量 parse 兜底——正确性不变，只慢一次）。
pub fn publish_gcs_drain_record(old_html: &str, new_html: &str, mutations: &[DomMutation]) {
    let record = Arc::new(GcsDrainRecord {
        old_html: Arc::from(old_html),
        new_html: Arc::from(new_html),
        mutations: mutations.to_vec(),
    });
    if let Ok(mut slot) = DRAIN_RECORD.lock() {
        *slot = Some(record);
    }
}

/// 消费 drain 记录并把代际槽推进到新 html（t6）。返回 `false` = 不可/不宜同步
///（记录缺失或不衔接、批次含 v1 不支持的变体、解析失败、变更节点解析失败），
/// 调用方回退全量 parse。
///
/// **v1 sync-safe 变体集**（批次含白名单外变体一律回退，宁慢勿错）：
/// - 无样式效果（changed 不计）：SetFormValue / SetFormComposition / FocusChanged /
///   Create*（本批登记，目标节点随 append 父子树 cascade 覆盖）。SelectOption 因
///   applier 改写兄弟 selected 属性（:checked 匹配面）不在白名单；
/// - selector 目标（pre-apply 解析，身份稳定）：SetAttr / RemoveAttr / SetStyle /
///   RemoveStyle / SetText / SetInnerHtml（目标子树）、SetChildText / RemoveChildAt
///   （selector 字段即父）、Remove / SetOuterHtml（解析目标后取父——nth-child 位移面）、
///   AppendChild / InsertBefore（parent_selector）；
/// - 同批 handle（post-apply 经 apply 返出的 ephemeral handle→selector 解析）：OnHandle
///   六族（目标）、AppendChildByHandle / InsertBeforeByHandle / InsertBeforeByHandleHandle
///   （父）。跨批 handle、handle-path / fragment / InsertAdjacent 族 → 回退。
///
/// **正确性**：cached doc 与 `parse(new_html)` 的等价性由「同一 applier
///（apply_dom_mutations）+ 幂等 replay + drain 时 style 类 mutation 必伴随快照重序列化」
/// 保持——与 webview 渲染路径 M3-S9 活 doc 机制同一信任基座；记录不衔接即全量兜底，
/// 漂移不可能跨过 html 键检查存活。批次触及样式表面（结构性写 / 目标为 style·meta）
/// 时换代收尾重收集 stylesheets 并以根元素扩大重算覆盖全文档（见
/// [`finish_generation_update`]）。
fn try_sync_with_drain_record(
    slot: &mut (
        String,
        usize,
        usize,
        Document,
        HashMap<NodeId, ComputedStyle>,
        Vec<Stylesheet>,
        Arc<str>,
        Arc<str>,
    ),
    html: &str,
    drain_gen: usize,
    style_version: usize,
    pending: &[DomMutation],
) -> bool {
    let Some(record) = DRAIN_RECORD.lock().ok().and_then(|mut r| r.take()) else {
        return false;
    };
    if record.old_html.as_ref() != slot.0.as_str() || record.new_html.as_ref() != html {
        return false;
    }
    // 批次分类 + pre-apply 解析（身份稳定：attr/选择器突变前先锚节点）。
    // 样式面判脏（PR #88 复核 D1）：结构性变体（fragment/append 子树可携带
    // `<style>`/`<meta>`、删除可移除样式元素）无条件脏；attr 变体目标为 meta 时脏
    //（color-scheme 合成规则）；文本变体目标为 style 时脏（规则文本）。
    let mut changed: Vec<NodeId> = Vec::new();
    let mut sheets_dirty = false;
    let mut post_handle: Vec<(String, bool)> = Vec::new(); // (handle, 无条件结构性脏)
    for m in &record.mutations {
        match m {
            DomMutation::SetAttr { selector, .. } | DomMutation::RemoveAttr { selector, .. } => {
                match find_in_doc(&slot.3, selector) {
                    Some(n) => {
                        if is_style_or_meta(&slot.3, n) {
                            sheets_dirty = true;
                        }
                        changed.push(n);
                    }
                    None => return false,
                }
            }
            DomMutation::SetInnerHtml { selector, .. } => match find_in_doc(&slot.3, selector) {
                Some(n) => {
                    sheets_dirty = true;
                    changed.push(n);
                }
                None => return false,
            },
            DomMutation::SetStyle { selector, .. } | DomMutation::RemoveStyle { selector, .. } => {
                match find_in_doc(&slot.3, selector) {
                    Some(n) => changed.push(n),
                    None => return false,
                }
            }
            DomMutation::SetText { selector, .. } => match find_in_doc(&slot.3, selector) {
                Some(n) => {
                    if is_style_or_meta(&slot.3, n) {
                        sheets_dirty = true;
                    }
                    changed.push(n);
                }
                None => return false,
            },
            // selector 字段即父；Remove/SetOuterHtml 解析目标后取父（目标本身将被删/换）。
            DomMutation::SetChildText { parent_selector, .. } => match find_in_doc(&slot.3, parent_selector) {
                Some(n) => {
                    if is_style_or_meta(&slot.3, n) {
                        sheets_dirty = true;
                    }
                    changed.push(n);
                }
                None => return false,
            },
            // 删除子节点可移除 `<style>` 的规则文本子节点（styleEl.firstChild.remove()）
            // → 与 SetChildText 同判父 tag。
            DomMutation::RemoveChildAt { parent_selector, .. } => match find_in_doc(&slot.3, parent_selector) {
                Some(n) => {
                    if is_style_or_meta(&slot.3, n) {
                        sheets_dirty = true;
                    }
                    changed.push(n);
                }
                None => return false,
            },
            DomMutation::Remove { selector } | DomMutation::SetOuterHtml { selector, .. } => {
                let target = match find_in_doc(&slot.3, selector) {
                    Some(n) => n,
                    None => return false,
                };
                // 删除/外层替换可移除 `<style>`/`<meta>` → 无条件脏。
                sheets_dirty = true;
                match slot.3.parent_node(target) {
                    Some(p) => changed.push(p),
                    None => return false,
                }
            }
            DomMutation::AppendChild { parent_selector, .. } | DomMutation::InsertBefore { parent_selector, .. } => {
                match find_in_doc(&slot.3, parent_selector) {
                    // append/insert 子树可携带 `<style>`/`<meta>` → 无条件脏。
                    Some(n) => {
                        sheets_dirty = true;
                        changed.push(n);
                    }
                    None => return false,
                }
            }
            DomMutation::AppendChildByHandle { parent_handle, .. }
            | DomMutation::InsertBeforeByHandle { parent_handle, .. }
            | DomMutation::InsertBeforeByHandleHandle { parent_handle, .. } => {
                post_handle.push((parent_handle.clone(), true));
            }
            // SetInnerHtmlOnHandle 与非 handle 版同语义（fragment 可携带 style/meta）
            // → 无条件脏；其余 OnHandle 变体 post-apply 按目标 tag 判脏。
            DomMutation::SetInnerHtmlOnHandle { handle, .. } => {
                post_handle.push((handle.clone(), true));
            }
            DomMutation::SetAttrOnHandle { handle, .. }
            | DomMutation::RemoveAttrOnHandle { handle, .. }
            | DomMutation::SetTextOnHandle { handle, .. }
            | DomMutation::SetStyleOnHandle { handle, .. }
            | DomMutation::RemoveStyleOnHandle { handle, .. } => {
                post_handle.push((handle.clone(), false));
            }
            // 无样式效果 / 本批登记（create 的节点随其 append 父子树 cascade）。
            // SelectOption 不在此列（PR #88 复核 D2）：权威 applier 会改写目标 option
            // 的 selected 属性并 deselect 兄弟，:checked / option[selected] 匹配面
            // 超出目标子树 → 回退全量。
            DomMutation::SetFormValue { .. }
            | DomMutation::SetFormComposition { .. }
            | DomMutation::FocusChanged { .. }
            | DomMutation::CreateElement { .. }
            | DomMutation::CreateElementNS { .. }
            | DomMutation::CreateTextNode { .. }
            | DomMutation::CreateComment { .. }
            | DomMutation::CreateProcessingInstruction { .. }
            | DomMutation::CreateDocumentFragment { .. } => {}
            // v1 白名单外（SelectOption、跨批 handle、path/fragment/InsertAdjacent 族、
            // RemoveHandle、其余表单外变体）→ 回退全量。
            _ => return false,
        }
    }
    // 权威 applier 推进 cached doc；Err → 回退。
    let handle_selectors = match super::apply_dom_mutations(&mut slot.3, &record.mutations) {
        Ok(map) => map,
        Err(_) => return false,
    };
    // post-apply 解析同批 handle（OnHandle 目标 / ByHandle 父——均随目标或父子树
    // cascade 覆盖）；结构父无条件脏，OnHandle 目标按 tag 判脏（attr/text 写到
    // meta/style 上会改样式面；SetStyle 类 inline 写误判为脏仅多一次级联，无害）。
    for (handle, structural) in post_handle {
        let Some(sel) = handle_selectors.get(&handle) else {
            return false;
        };
        let Some(n) = find_in_doc(&slot.3, sel) else {
            return false;
        };
        if structural || is_style_or_meta(&slot.3, n) {
            sheets_dirty = true;
        }
        changed.push(n);
    }
    // 变更节点必须全部仍然存在（批内被删的目标/父 → 回退，宁慢勿错）。
    for &nid in &changed {
        if !matches!(slot.3.get(nid).map(|n| &n.kind), Some(zero_dom::NodeKind::Element(_))) {
            return false;
        }
    }
    // 当前 pending 批 replay（attr/style 子集，既有语义）并入变更集；pending 写到
    // meta/style 上同样改样式面 → 与同 html 路径镜像判脏。
    changed.extend(apply_inline_style_overrides(&mut slot.3, pending));
    if changed.iter().any(|&n| is_style_or_meta(&slot.3, n)) {
        sheets_dirty = true;
    }
    finish_generation_update(slot, html, drain_gen, style_version, changed, sheets_dirty)
}

fn find_in_doc(doc: &Document, selector: &str) -> Option<NodeId> {
    super::find_by_selector(doc, selector)
}

/// 目标元素是否可能牵动样式表面（`<style>` 规则文本 / `<meta color-scheme>` 合成
/// 规则）——供增量换代判脏（PR #88 复核 D1）。
fn is_style_or_meta(doc: &Document, nid: NodeId) -> bool {
    matches!(
        doc.get(nid).map(|n| &n.kind),
        Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "style" || e.local_name() == "meta"
    )
}

/// 增量 cascade 收尾：去重变更集 → （样式面脏时重收集 + 全文档覆盖）→ 变更子树
/// 重算 → 固有尺寸覆盖 → 换代键前进。返回 `false` = 无法安全收尾（样式面脏但无
/// html 根可锚），调用方回退全量 parse。
fn finish_generation_update(
    slot: &mut (
        String,
        usize,
        usize,
        Document,
        HashMap<NodeId, ComputedStyle>,
        Vec<Stylesheet>,
        Arc<str>,
        Arc<str>,
    ),
    html: &str,
    drain_gen: usize,
    style_version: usize,
    mut changed: Vec<NodeId>,
    sheets_dirty: bool,
) -> bool {
    changed.sort_unstable();
    changed.dedup();
    // 样式面脏（PR #88 复核 D1）：本批可能改变 `<style>` 文本或 meta color-scheme
    // 合成规则——沿用旧代 stylesheets 会让脏规则贯穿整个会话（sync 通道持续成功则
    // 全量 parse 一次都不发生，无自愈点）。重收集后还需把根元素推入变更集：新规则
    // 可匹配任意元素、根级声明经继承传播，影响面不限于变更子树——根子树=全文档，
    // 免 HTML 重 parse 的全量级联。
    // P9-9：外部 CSS 变化（同 html 晚到/更新样式表）同判脏——css 是级联源之一，
    // 沿用旧代表会 stale 整个会话。当前 css 经 active_external_css() 读取（快照臂
    // 先于同代 gCS 装填，同代内稳定）。
    // R-t2qq：adopted 样式表文本（constructable stylesheets，shim 经
    // `__zw_set_adopted_css` 全量重推）同为级联源键分量——变化判脏重收集。
    let css = active_external_css();
    let adopted = active_adopted_css();
    let sheets_dirty = sheets_dirty || slot.6 != css || slot.7 != adopted;
    if sheets_dirty {
        let parsed = memoized_external_sheet(&css);
        slot.5 = collect_stylesheets_with_parsed_external(&slot.3, &css, parsed.as_ref());
        slot.6 = css;
        slot.7 = adopted;
        match slot.3.get_elements_by_tag_names(&["html"]).first().copied() {
            Some(root) => changed.push(root),
            None => return false,
        }
    }
    let mut sys = StyleSystem::new();
    // 视口与全量路径同源（compute_styles_for_doc 的 1280×800 默认）。
    sys.set_viewport(1280.0, 800.0);
    sys.compute_styles_incremental(&slot.3, &slot.5, &changed, &mut slot.4);
    apply_intrinsic_size_overrides(&slot.3, &mut slot.4);
    slot.0.clear();
    slot.0.push_str(html);
    slot.1 = drain_gen;
    slot.2 = style_version;
    true
}

thread_local! {
    /// `(html 快照, drain 代际, style_version, parsed doc, NodeId→ComputedStyle,
    /// stylesheets, 外部 CSS, adopted CSS)` 代际槽。stylesheets 随 doc 缓存（html 未变的
    /// 增量换代不重收集/重解析 CSS）；外部 CSS 与 adopted CSS 作键分量（P9-9 / R-t2qq：
    /// 变化 ⇒ sheets 重收集 + 全文档重算）。非 `Send` 类型（`Document`）仅存于创建线程
    ///（js worker），无跨线程共享。
    static GENERATION_CACHE: RefCell<
        Option<(String, usize, usize, Document, HashMap<NodeId, ComputedStyle>, Vec<Stylesheet>, Arc<str>, Arc<str>)>,
    > = const { RefCell::new(None) };

    /// 测试观测点：本线程换代重算次数（全量或增量；仅 `cfg(test)` 编译，生产零开销）。
    /// `clear_generation_cache` 的直接单测靠它判定「清槽后同键查询重算」——计数与
    /// 槽同线程，天然隔离并行测试。
    #[cfg(test)]
    pub static GENERATION_RECOMPUTE_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };

    /// 测试观测点：本线程全量 parse 兜底次数（增量 / sync 换代不计；`cfg(test)` only）。
    /// 与换代总计数分离，用于钉「同 html 增量路径零全量 parse」——换代粒度修复的
    /// revert 检出锚点（PR #88 复核）。
    #[cfg(test)]
    pub static GENERATION_FULL_PARSE_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// 清空本线程的代际槽（换代即 drop 旧代际的 `Document` 与 styles）+ 全局 drain 记录
/// （跨文档残留记录不可能衔接新槽，清掉防误消费）。
/// `register_dom_callbacks` 注册时调用：新注册装新快照，旧代际一律不复用
///（覆盖 webview 就地 clear 不 bump drain_gen 的例外与同线程多沙箱隔离）。
pub fn clear_generation_cache() {
    GENERATION_CACHE.with(|cell| *cell.borrow_mut() = None);
    if let Ok(mut r) = DRAIN_RECORD.lock() {
        *r = None;
    }
}

/// 在 `(html, drain_gen, style_version)` 代际缓存的 `(Document, NodeId→ComputedStyle)`
/// 上执行 `f`。
///
/// 代际未变 → 复用 parsed doc（不重算）；html 同而 drain_gen / style_version 前进 →
/// 复用 doc 幂等 replay pending + 变更子树增量 cascade；html 变 → 先试 drain 记录
/// 同步（t6，cached doc + 本批 drained mutations 推进，免重 parse），记录缺失或不
/// 衔接 → 全量 parse + inline override + cascade 后换代。返回 `f` 的结果。
///
/// **契约**：同代际调用必须传等价的 `mutations` 内容（键不含内容摘要——由键契约
/// 保证同代际队列前缀相同）；`f` 不得重入本函数（RefCell 借用期内；host 回调单线程
/// 执行且 `f` 体无 JS/宿主回流，现状唯一调用方满足）。
///
/// **已知偏差**（PR #88 复核 D3，接受项）：二级 replay 隐含「html 同 + 键前进 ⇒
/// drained 批无可见 DOM 效果」。drain 时 applier Err（队列已消费、gen 已 bump、
/// html 未变、无记录）或 `clear_mutations_fresh` 丢弃未 drain 批次时，cached doc
/// 保留此前 replay 进去的 inline style 效果，而权威全量语义（parse(html)+新队列）
/// 不含——「JS 写入可见 vs 快照权威」取前者（R3030 意图方向），偏离本函数
/// 「增量 ≡ 全量逐位一致」合同；闭合需 drain 失败路径发作废信号，暂不做。
pub fn with_cached_document_styles<T>(
    html: &str,
    drain_gen: usize,
    style_version: usize,
    mutations: &[DomMutation],
    f: impl FnOnce(&Document, &HashMap<NodeId, ComputedStyle>) -> T,
) -> T {
    GENERATION_CACHE.with(|cell| {
        // P9-9：当前代外部 CSS（快照臂装填）作键分量——css 变化即换代（走下方
        // html 同分支，finish_generation_update 经 css 对比判 sheets 脏重收集）。
        // R-t2qq：adopted 文本同为键分量（页面 setter 全量重推，变化即换代）。
        let css = active_external_css();
        let adopted = active_adopted_css();
        {
            let mut slot = cell.borrow_mut();
            let same_gen = matches!(
                &*slot,
                Some((h, g, v, .., c, a)) if h == html && *g == drain_gen && *v == style_version && **c == *css && **a == *adopted
            );
            if !same_gen {
                #[cfg(test)]
                GENERATION_RECOMPUTE_COUNT.with(|c| c.set(c.get() + 1));
                let advanced = if matches!(&*slot, Some((h, ..)) if h == html) {
                    // html 同、键前进（drain_gen / style_version / 外部 CSS / adopted）：
                    // 幂等 replay pending 收集变更节点 → 增量换代。css 变化由
                    // finish_generation_update 的 css 对比判 sheets 脏（重收集 + 根全文档重算）。
                    let s = slot.as_mut().expect("html 匹配守卫已保证槽为 Some");
                    let changed = apply_inline_style_overrides(&mut s.3, mutations);
                    // pending attr 写到 meta/style 上会改样式面 → 判脏。
                    let dirty = changed.iter().any(|&n| is_style_or_meta(&s.3, n));
                    finish_generation_update(s, html, drain_gen, style_version, changed, dirty)
                } else {
                    // html 变：先试 drain 记录同步（cached doc 推进到新代）。
                    slot.is_some()
                        && try_sync_with_drain_record(
                            slot.as_mut().expect("守卫保证 Some"),
                            html,
                            drain_gen,
                            style_version,
                            mutations,
                        )
                };
                if !advanced {
                    // 冷槽 / 记录不衔接 / 批次含不支持变体 / 收尾失败：全量 parse 兜底。
                    #[cfg(test)]
                    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(c.get() + 1));
                    // P9-9：css 变体一次 parse 同时产出级联与代际槽 sheets。
                    let (doc, styles, sheets) =
                        compute_document_styles_with_inline_overrides_css(html, mutations, &css);
                    *slot = Some((html.to_string(), drain_gen, style_version, doc, styles, sheets, css, adopted));
                }
            }
        }
        // 不变式：上方 borrow_mut 块保证槽必为 Some（命中沿用或换代重填），故 expect 不可达。
        let binding = cell.borrow();
        let (_, _, _, doc, styles, ..) = binding.as_ref().expect("generation cache populated");
        f(doc, styles)
    })
}
