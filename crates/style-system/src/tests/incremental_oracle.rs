//! 增量样式与完整计算双路径对照（incremental-rendering-oracle goal，M1 基线）。
//!
//! 同一 `Document` 上驱动两条样式路径，逐步施加确定性 DOM/CSSOM 最小变更序列，
//! 逐步逐元素比较可观察样式：
//! - 全量路径：每步 [`StyleSystem::compute_styles`]；
//! - 增量路径：首帧全量，此后 [`StyleSystem::compute_styles_incremental`]
//!   （与生产 `paint_only_incremental` 的 cached_styles base 复用方式一致）。
//!
//! 比较契约：[`ComputedStyle`] 结构相等（`PartialEq`，custom properties 为
//! `Arc<HashMap>` 内容等、与插入顺序无关），**不设浮点容差**——两路径在同一
//! 进程对同一输入运行同一计算代码，任何差异都是真实的失效分歧；几何/像素
//! 容差待布局切片按输出契约另行固定（见 goal master 缺口表）。
//!
//! [`Expectation::Divergent`] 步骤把已知 API 契约缺口（changed 集不完备 /
//! 祖先链重算不覆盖伪元素）固化为检测器灵敏度断言：分歧必须被报出，防止
//! 对照器失灵；缺口修复后应把对应步骤改为 [`Expectation::Equal`]。

use super::super::*;
use std::collections::HashMap;
use zero_css_parser::Parser as CssParser;
use zero_dom::{Document, NodeId, NodeKind};

/// 每步期望结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expectation {
    /// 双路径可观察样式必须一致。
    Equal,
    /// 已知失效缺口：对照器必须报出分歧（检测器灵敏度断言）。
    Divergent,
}

/// 一次分歧记录：节点定位 + 差异描述。
struct Divergence {
    /// 人读定位：`tag#id.class`。
    where_: String,
    /// 差异描述（条目丢失/残留，或首个 differing 字段的 Debug 行对）。
    detail: String,
}

/// 基础样式表：覆盖标签/类/ID 选择器与继承（body color）。
const BASE_CSS: &str = "body { color: black; }\n.card { padding: 8px; }\np.text { color: blue; font-size: 14px; }\n#sec-b { color: green; }\n";

/// 加 `.highlight` 规则后的样式表（INC-01 class 变更步，BASE 之上追加）。
const CSS_WITH_HIGHLIGHT: &str = "body { color: black; }\n.card { padding: 8px; }\np.text { color: blue; font-size: 14px; }\n#sec-b { color: green; }\n.highlight { color: red; }\n";

/// `.card` 增 font-size（后代继承步，INC-02 继承序列）。
const CSS_CARD_FONT: &str = "body { color: black; }\n.card { padding: 8px; font-size: 20px; }\np.text { color: blue; font-size: 14px; }\n#sec-b { color: green; }\n";

/// 含 `:has()` 规则的样式表（增量入口按契约退化全量，DC-2 回退行为）。
const CSS_WITH_HAS: &str = "body { color: black; }\n.card { padding: 8px; }\np.text { color: blue; font-size: 14px; }\n#sec-b { color: green; }\n.card:has(.text) { padding: 16px; }\n";

/// 主 fixture 句柄。
struct Fixture {
    html: NodeId,
    card_a: NodeId,
    sec_b: NodeId,
    p1: NodeId,
    p2: NodeId,
    s1: NodeId,
}

/// 构建 html(lang)>body>(div#card-a.card>(p#p1.text+p#p2.text+span#s1)+section#sec-b)。
fn build_main_fixture() -> (Document, Vec<NodeId>, Fixture) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.set_attribute(html, "lang", "en");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let card_a = doc.create_element("div");
    doc.set_attribute(card_a, "id", "card-a");
    doc.set_attribute(card_a, "class", "card");
    doc.append_child(body, card_a).unwrap();
    let p1 = doc.create_element("p");
    doc.set_attribute(p1, "id", "p1");
    doc.set_attribute(p1, "class", "text");
    doc.append_child(card_a, p1).unwrap();
    let p2 = doc.create_element("p");
    doc.set_attribute(p2, "id", "p2");
    doc.set_attribute(p2, "class", "text");
    doc.append_child(card_a, p2).unwrap();
    let s1 = doc.create_element("span");
    doc.set_attribute(s1, "id", "s1");
    doc.append_child(card_a, s1).unwrap();
    let sec_b = doc.create_element("section");
    doc.set_attribute(sec_b, "id", "sec-b");
    doc.append_child(body, sec_b).unwrap();
    let elements = vec![html, body, card_a, p1, p2, s1, sec_b];
    (
        doc,
        elements,
        Fixture {
            html,
            card_a,
            sec_b,
            p1,
            p2,
            s1,
        },
    )
}

/// 节点人读定位：`tag#id.class`；非元素/已删除给出说明。
fn describe_node(doc: &Document, id: NodeId) -> String {
    let Some(n) = doc.get(id) else {
        return format!("{id:?}（已删除）");
    };
    if let NodeKind::Element(e) = &n.kind {
        let mut s = e.local_name().to_string();
        if let Some(i) = doc.get_attribute(id, "id") {
            s.push('#');
            s.push_str(&i);
        }
        if let Some(c) = doc.get_attribute(id, "class") {
            s.push('.');
            s.push_str(&c.replace(' ', "."));
        }
        s
    } else {
        format!("{id:?}（非元素节点）")
    }
}

/// 首个 differing 字段的 Debug 行对，附最近 3 行上下文定位字段路径
/// （首个差异后行对齐不可靠，只报第一处）。
fn first_field_diffs(a: &ComputedStyle, b: &ComputedStyle) -> String {
    let sa = format!("{a:#?}");
    let sb = format!("{b:#?}");
    let la: Vec<&str> = sa.lines().collect();
    let lb: Vec<&str> = sb.lines().collect();
    let mut out = Vec::new();
    for k in 0..la.len().max(lb.len()) {
        match (la.get(k), lb.get(k)) {
            (Some(x), Some(y)) if x != y => {
                let ctx_start = k.saturating_sub(3);
                let ctx: Vec<String> = la[ctx_start..k].iter().map(|l| format!("      ⋮ {l}")).collect();
                if !ctx.is_empty() {
                    out.push(format!("    字段上下文（全量侧）:\n{}", ctx.join("\n")));
                }
                out.push(format!("    - {x}\n    + {y}"));
                break;
            }
            (Some(x), None) => {
                out.push(format!("    - {x}"));
                break;
            }
            (None, Some(y)) => {
                out.push(format!("    + {y}"));
                break;
            }
            _ => {}
        }
    }
    out.join("\n")
}

/// 双路径对照驱动器：同一 Document，两个独立 [`StyleSystem`]（互不污染）。
struct DualPathRun {
    doc: Document,
    elements: Vec<NodeId>,
    full_sys: StyleSystem,
    inc_sys: StyleSystem,
    full_styles: HashMap<NodeId, ComputedStyle>,
    inc_styles: HashMap<NodeId, ComputedStyle>,
    css: String,
}

impl DualPathRun {
    /// 首帧：两路径同为全量，必须一致（基线快照）。
    fn new(doc: Document, elements: Vec<NodeId>, css: &str) -> Self {
        let mut full_sys = StyleSystem::new();
        let mut inc_sys = StyleSystem::new();
        let sheets = vec![CssParser::parse_stylesheet(css)];
        let full_styles = full_sys.compute_styles(&doc, &sheets);
        let inc_styles = inc_sys.compute_styles(&doc, &sheets);
        let run = Self {
            doc,
            elements,
            full_sys,
            inc_sys,
            full_styles,
            inc_styles,
            css: css.to_string(),
        };
        let divs = run.compare();
        assert!(
            divs.is_empty(),
            "首帧全量基线即分歧（双路径输入相同）：\n{}",
            render_divergences(&divs)
        );
        run
    }

    /// 施加一步最小变更并比对双路径。
    ///
    /// - `css` — 本步生效的 CSS 文本；`None` = 沿用上一步（DOM-only 变更）。
    /// - `mutate` — 施加 DOM 变更，返回增量路径的 changed 集合（与生产路由
    ///   一致的最小集：`paint_only_incremental` 只传变更目标）。
    fn apply(
        &mut self,
        step_name: &str,
        css: Option<&str>,
        mutate: impl FnOnce(&mut Document) -> Vec<NodeId>,
        expect: Expectation,
    ) {
        if let Some(c) = css {
            self.css = c.to_string();
        }
        let changed = mutate(&mut self.doc);
        let sheets = vec![CssParser::parse_stylesheet(&self.css)];
        self.full_styles = self.full_sys.compute_styles(&self.doc, &sheets);
        self.inc_sys
            .compute_styles_incremental(&self.doc, &sheets, &changed, &mut self.inc_styles);
        let divs = self.compare();
        match expect {
            Expectation::Equal => assert!(
                divs.is_empty(),
                "步骤「{step_name}」出现双路径分歧（期望一致）：\n{}",
                render_divergences(&divs)
            ),
            Expectation::Divergent => assert!(
                !divs.is_empty(),
                "步骤「{step_name}」期望报出失效分歧，但双路径一致——对照器可能失灵，\
                 或该缺口已被修复（修复后请把本步改为 Equal 并更新 goal 证据）"
            ),
        }
    }

    /// 逐元素比较全量/增量样式表（含条目集合差异——陈旧缓存也是分歧）。
    fn compare(&self) -> Vec<Divergence> {
        let mut out = Vec::new();
        for &id in &self.elements {
            let where_ = describe_node(&self.doc, id);
            match (self.full_styles.get(&id), self.inc_styles.get(&id)) {
                (None, None) => {}
                (Some(_), None) => out.push(Divergence {
                    where_,
                    detail: "增量路径丢失该节点的样式条目".to_string(),
                }),
                (None, Some(_)) => out.push(Divergence {
                    where_,
                    detail: "增量路径残留该节点的样式条目（陈旧缓存）".to_string(),
                }),
                (Some(a), Some(b)) => {
                    if a != b {
                        out.push(Divergence {
                            where_,
                            detail: first_field_diffs(a, b),
                        });
                    }
                }
            }
        }
        out
    }
}

/// 分歧报告文本（失败断言与证据输出共用）。
fn render_divergences(divs: &[Divergence]) -> String {
    divs.iter()
        .map(|d| format!("  - {}: {}\n{}", d.where_, d.detail, ""))
        .collect()
}

// ── INC-01（DC-1）：class/属性变更、移除 CSSOM 声明、移动子树 ──────────

/// INC-01a：class 变更与祖先属性增删——双路径一致。
#[test]
fn inc01_class_and_ancestor_attr_changes_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(doc, elements, BASE_CSS);
    run.apply(
        "p1 class 追加 highlight",
        Some(CSS_WITH_HIGHLIGHT),
        |d| {
            d.set_attribute(f.p1, "class", "text highlight");
            vec![f.p1]
        },
        Expectation::Equal,
    );
    run.apply(
        "祖先 card-a 增 data-x 属性（无选择器依赖）",
        None,
        |d| {
            d.set_attribute(f.card_a, "data-x", "1");
            vec![f.card_a]
        },
        Expectation::Equal,
    );
    run.apply(
        "祖先 card-a 移除 data-x 属性",
        None,
        |d| {
            d.remove_attribute(f.card_a, "data-x");
            vec![f.card_a]
        },
        Expectation::Equal,
    );
}

/// INC-01b：逐条移除 CSSOM 声明直至最后一条——双路径一致。
#[test]
fn inc01_remove_last_declaration_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(doc, elements, "p { color: red; font-size: 14px; }");
    run.apply(
        "移除 p 规则的 font-size 声明",
        Some("p { color: red; }"),
        |_| vec![f.p1, f.p2],
        Expectation::Equal,
    );
    run.apply(
        "移除 p 规则最后声明（规则清空）",
        Some(""),
        |_| vec![f.p1, f.p2],
        Expectation::Equal,
    );
}

/// INC-01c：子树移动——继承来源切换，双路径一致。
#[test]
fn inc01_move_subtree_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(doc, elements, BASE_CSS);
    run.apply(
        "span#s1 移动到 #sec-b（继承 color 切到 green）",
        None,
        |d| {
            d.remove_child(f.card_a, f.s1).unwrap();
            d.append_child(f.sec_b, f.s1).unwrap();
            vec![f.s1]
        },
        Expectation::Equal,
    );
    run.apply(
        "span#s1 移回 .card（继承 color 切回 black）",
        None,
        |d| {
            d.remove_child(f.sec_b, f.s1).unwrap();
            d.append_child(f.card_a, f.s1).unwrap();
            vec![f.s1]
        },
        Expectation::Equal,
    );
}

// ── INC-02（DC-2）starter：继承变更与 :has() 全量回退 ─────────────────

/// INC-02a：祖先 CSS 增可继承属性——增量重算子树后与全量一致。
#[test]
fn inc02_inherited_font_change_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(doc, elements, BASE_CSS);
    run.apply(
        ".card 增 font-size:20px（s1 继承，p1 自身规则覆盖）",
        Some(CSS_CARD_FONT),
        |_| vec![f.card_a],
        Expectation::Equal,
    );
}

/// INC-02b：样式表含 `:has()` 时增量入口按契约退化全量——结果与全量一致，
/// 且回退有计数（DC-2"全量回退有计数"：实例计数器只在真实退化时递增，
/// 无 `:has` 的步保持不变；oracle 据此断言回退确实发生而非静默走错路径）。
#[test]
fn inc02_has_selector_fallback_matches_full() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(doc, elements, CSS_WITH_HAS);
    assert_eq!(
        run.inc_sys.incremental_has_fallback_count(),
        0,
        "首帧两路径同为全量，不应有回退"
    );
    run.apply(
        "移除 :has 规则（CSSOM 变更，changed=p1）",
        Some(BASE_CSS),
        |_| vec![f.p1],
        Expectation::Equal,
    );
    assert_eq!(
        run.inc_sys.incremental_has_fallback_count(),
        0,
        "无 :has 规则的步走真增量，不回退"
    );
    run.apply(
        "恢复 :has 规则（changed=s1）",
        Some(CSS_WITH_HAS),
        |_| vec![f.s1],
        Expectation::Equal,
    );
    assert_eq!(
        run.inc_sys.incremental_has_fallback_count(),
        1,
        ":has 规则在页时增量入口退化全量并计数（DC-2）"
    );
}

/// INC-02c：Shadow DOM 子树内的最小变更序列——双路径一致。
///
/// `attach_shadow` 把 ShadowRoot 节点挂进 host 的 children（shadow.rs：parent 指回
/// host），故全量 DFS 下探 shadow 子树；增量链走自 shadow 内元素向上穿过
/// ShadowRoot（非 Element，跳过）达 host。本序列按内部等价基线固化（Equal）：
/// 两路径对同一 Document 的遍历语义必须一致；shadow 封装/插槽分配的规范正确性
/// 属 rendering-compat 范围，不在本对照断言。
#[test]
fn inc02_shadow_tree_mutations_consistent() {
    // html > body > div#host > (shadow) > p#inner.s-inner
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let shadow = doc.attach_shadow(host, zero_dom::ShadowRootMode::Open).unwrap();
    let inner = doc.create_element("p");
    doc.set_attribute(inner, "id", "inner");
    doc.set_attribute(inner, "class", "s-inner");
    doc.append_child(shadow, inner).unwrap();
    let elements = vec![html, body, host, inner];

    let mut run = DualPathRun::new(doc, elements, "body { color: black; }\n.s-inner { color: purple; }\n");
    run.apply(
        "shadow 内元素 class 追加 hot（链走自 shadow 穿出达 host）",
        Some("body { color: black; }\n.s-inner { color: purple; }\n.hot { color: red; }\n"),
        |d| {
            d.set_attribute(inner, "class", "s-inner hot");
            vec![inner]
        },
        Expectation::Equal,
    );
    run.apply(
        "host 属性变更（变更子树递归下探 shadow 重算 inner）",
        None,
        |d| {
            d.set_attribute(host, "data-x", "1");
            vec![host]
        },
        Expectation::Equal,
    );
    run.apply(
        "CSS 同帧改 .s-inner 色值（shadow 内元素重算不沿用旧值）",
        Some("body { color: black; }\n.s-inner { color: olive; }\n.hot { color: red; }\n"),
        |_| vec![inner],
        Expectation::Equal,
    );
}

/// INC-02d：`lang` 属性变更传播到 `<q>` 自动引号——双路径一致。
///
/// R2856：`quotes: auto` 按父元素有效内容语言解析引号对（`effective_lang` 现读
/// Document、无实例状态）。changed=[html] 时链走重算 html，变更子树递归覆盖 q
/// ——两路径都按新 lang 重解析（en " " → ja 「 」）。
#[test]
fn inc02_lang_change_propagates_to_quotes() {
    // html(lang=en) > body > q#quote
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.set_attribute(html, "lang", "en");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let quote = doc.create_element("q");
    doc.set_attribute(quote, "id", "quote");
    doc.append_child(body, quote).unwrap();
    let elements = vec![html, body, quote];

    let mut run = DualPathRun::new(doc, elements, "body { color: black; }\n");
    run.apply(
        "html lang en→ja（q 自动引号按新语言重解析）",
        None,
        |d| {
            d.set_attribute(html, "lang", "ja");
            vec![html]
        },
        Expectation::Equal,
    );
    run.apply(
        "html lang 置空（空 lang = 无语言，回落英语 root 引号）",
        None,
        |d| {
            d.set_attribute(html, "lang", "");
            vec![html]
        },
        Expectation::Equal,
    );
}

/// INC-02e：根字号度量随 html font-size 变更刷新——根相对单位双路径一致。
///
/// 根度量（root_x_height 等）是 StyleSystem 实例状态：全量入口先 reset、根元素
/// （parent_style=None）计算时回填；增量路径不 reset，靠链走重算 html 回填新值
/// 且先于变更子树递归。`rex` 等根相对单位按该度量解析（rem 恒按常量
/// ROOT_FONT_SIZE，不随 html font-size 变化），故用 `line-height: Nrex` 钉住
/// 度量刷新时序。
#[test]
fn inc02_root_metric_units_refresh_on_html_font_change() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(
        doc,
        elements,
        "body { color: black; }\nhtml { font-size: 16px; }\np { line-height: 10rex; }\n",
    );
    run.apply(
        "html font-size 16→24px（链走回填根度量，p 的 rex 按新值解析）",
        Some("body { color: black; }\nhtml { font-size: 24px; }\np { line-height: 10rex; }\n"),
        |_| vec![f.html],
        Expectation::Equal,
    );
}

// ── 失效反例（检测器灵敏度断言）：已知 API 契约缺口必须被报出 ──────────

/// 反例 A（API 契约缺口，非生产可达）：结构型伪类 `:only-child` 依赖兄弟
/// 存在性；增量 changed 集只含被移动节点时，留存的兄弟不重算——全量判
/// `:only-child` 成立、增量保留旧值。
#[test]
fn inc01_sibling_invalidation_gap_detected() {
    // body > div#host > (p#q1 + p#q2)，body > section#dst
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let q1 = doc.create_element("p");
    doc.set_attribute(q1, "id", "q1");
    doc.append_child(host, q1).unwrap();
    let q2 = doc.create_element("p");
    doc.set_attribute(q2, "id", "q2");
    doc.append_child(host, q2).unwrap();
    let dst = doc.create_element("section");
    doc.set_attribute(dst, "id", "dst");
    doc.append_child(body, dst).unwrap();
    let elements = vec![html, body, host, q1, q2, dst];

    let mut run = DualPathRun::new(
        doc,
        elements,
        "body { color: black; }\np:only-child { color: green; }\n",
    );
    run.apply(
        "q2 移出 #host（q1 变为 :only-child，changed 只含 q2）",
        None,
        |d| {
            d.remove_child(host, q2).unwrap();
            d.append_child(dst, q2).unwrap();
            vec![q2]
        },
        Expectation::Divergent,
    );
}

/// 反例 B 修复回归（原失效缺口，2026-10-23 修复）：祖先链单节点重算与全量
/// 递归共用 `attach_pseudo_elements`——伪样式在链走后保留，且 CSSOM 同帧
/// 变更时同步重算（不沿用旧值）。
#[test]
fn inc01_ancestor_pseudo_preserved_on_chain_recompute() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(
        doc,
        elements,
        "body { color: black; }\nbody::before { content: \"B\"; }\np.text { color: blue; }\n",
    );
    run.apply(
        "p1 inline style 变更触发祖先链重算（body::before 应保留）",
        None,
        |d| {
            d.set_attribute(f.p1, "style", "color: navy");
            vec![f.p1]
        },
        Expectation::Equal,
    );
    run.apply(
        "移除 body::before 规则（CSSOM 同帧变更，祖先伪样式须消失）",
        Some("body { color: black; }\np.text { color: blue; }\n"),
        |_| vec![f.p2],
        Expectation::Equal,
    );
}

/// 反例 C 修复回归（原失效缺口，2026-10-23 修复）：增量链走自根向下压入
/// 祖先容器条目（含 style-only 容器），祖先与变更子树的 @container 条件按
/// 真实祖先容器求值；CSSOM 同帧收窄容器时同步失效。
#[test]
fn inc01_container_context_preserved_on_chain_recompute() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualPathRun::new(
        doc,
        elements,
        "body { color: black; }\n.card { container-type: inline-size; width: 400px; }\n@container (min-width: 300px) { p { color: green; } }\n",
    );
    run.apply(
        "p1 inline margin 变更触发链走（@container 应按 .card 求值，p1 保持 green）",
        None,
        |d| {
            d.set_attribute(f.p1, "style", "margin-top: 4px");
            vec![f.p1]
        },
        Expectation::Equal,
    );
    // changed 集按 API 契约补全：CSS 变更影响所有匹配 `p` 的兄弟（反例 A 同族
    // 契约——调用方须把受影响节点全部纳入 changed）。
    run.apply(
        "CSS 同帧收窄 .card 至 100px（@container 不再成立，p1/p2 变 black）",
        Some("body { color: black; }\n.card { container-type: inline-size; width: 100px; }\n@container (min-width: 300px) { p { color: green; } }\n"),
        |_| vec![f.p1, f.p2],
        Expectation::Equal,
    );
}
