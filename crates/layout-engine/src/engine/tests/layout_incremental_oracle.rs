//! 布局层双路径对照（incremental-rendering-oracle goal，M1 布局切片）。
//!
//! 同一 `Document` + 同一 CSS 输入驱动两条布局路径，逐步施加确定性变更序列，
//! 逐步逐盒比较几何：
//! - 冷路径（`cold_engine`）：每步先 `invalidate_cache()` 再 [`LayoutEngine::compute`]
//!   ——≡ 生产对布局影响型 mutation 的快照全量重建语义（`render_with_dom_mutations_
//!   persistent`：SetText/结构变更不走增量臂，pipeline/mod.rs slice14 撤用注记）；
//! - 温路径（`warm_engine`）：保留 `cached_state` 跨步 `compute`——钉住 compute()
//!   的输入纯度：cached_state 只写不读（engine.rs 内仅 compute_incremental 消费），
//!   任何未来"全量里偷用缓存"的改动都会被本对照检出。
//!
//! 样式输入走双路径（全量 vs `compute_styles_incremental`），并在几何比对前先断言
//! 两路径样式相等——样式层分歧单独报出，不与布局分歧混淆归因。
//!
//! 比较契约（浮点容差决策，M1 按输出契约固定）：**几何精确相等，不设容差**——
//! 两路径同进程对同一 (doc, styles) 运行同一布局代码，f32 位级相同是应有语义；
//! 任何差异都是真实的缓存泄漏或非确定性，不得用 epsilon 掩盖。
//!
//! [`LayoutResult::paint_skip_node_ids`] 不在本切片比对范围：增量臂恒空是撤用臂的
//! 已知语义差（不重跑 compute_final 孤儿回填），归 M3 生产帧切片对照。

use super::*;
use crate::dirty::LayoutDirtyTracker;
use crate::types::{LayoutBox, LayoutResult, OverflowClip};
use zero_css_parser::Parser as CssParser;
use zero_dom::{Document, NodeId, parse_html};
use zero_style_system::{ComputedStyle, StyleSystem};

/// 单盒最终输出快照：几何（x/y 相对父内容区、边框盒、内容盒）+ 溢出处理枚举
///（M3 拆面扩张：DC-4"溢出符合预期"的布局面可观察量；scroll_x/scroll_y 布局期
/// 恒 0 不入快照，滚动范围由盒树几何蕴含）。
#[derive(Debug, Clone, Copy, PartialEq)]
struct GeoSnapshot {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    cw: f32,
    ch: f32,
    overflow_x: OverflowClip,
    overflow_y: OverflowClip,
}

/// 每步期望结果（冷温双路径序列只承载一致基线；撤用增量臂的已知分歧边界
/// 由独立测试直接断言，不经本 harness）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expectation {
    /// 双路径几何必须一致。
    Equal,
}

/// 基础样式表：定宽容器 + 两段文本 + 兄弟节（几何非平凡：padding/行高/兄弟位移）。
const BASE_CSS: &str = "body { margin: 0; }\n.card { width: 400px; padding: 8px; }\np.text { font-size: 14px; line-height: 1.5; margin: 4px 0; }\n#sec-b { margin-top: 10px; padding: 6px; }\n";

/// `.card` 加宽（布局影响属性变更步）。
const WIDE_CSS: &str = "body { margin: 0; }\n.card { width: 600px; padding: 8px; }\np.text { font-size: 14px; line-height: 1.5; margin: 4px 0; }\n#sec-b { margin-top: 10px; padding: 6px; }\n";

/// `p.text` 字号放大（字体失效步；影响全部 p.text——changed 集须完备）。
const BIGFONT_CSS: &str = "body { margin: 0; }\n.card { width: 400px; padding: 8px; }\np.text { font-size: 20px; line-height: 1.5; margin: 4px 0; }\n#sec-b { margin-top: 10px; padding: 6px; }\n";

/// 主 fixture：html>body>(div#card-a.card>(p#p1.text+p#p2.text+span#s1)+section#sec-b)，
/// 与 style 层 oracle（zero-style-system）同构，含真实文本使行盒几何非平凡。
struct Fixture {
    card_a: NodeId,
    p1: NodeId,
    p2: NodeId,
    s1: NodeId,
    sec_b: NodeId,
}

fn build_main_fixture() -> (Document, Vec<NodeId>, Fixture) {
    let doc = parse_html(
        r#"<html lang="en"><body><div id="card-a" class="card"><p id="p1" class="text">First paragraph text for layout</p><p id="p2" class="text">Second paragraph</p><span id="s1">inline span</span></div><section id="sec-b">section content</section></body></html>"#,
    );
    let card_a = doc.query_selector(doc.root(), "#card-a").expect("#card-a");
    let p1 = doc.query_selector(doc.root(), "#p1").expect("#p1");
    let p2 = doc.query_selector(doc.root(), "#p2").expect("#p2");
    let s1 = doc.query_selector(doc.root(), "#s1").expect("#s1");
    let sec_b = doc.query_selector(doc.root(), "#sec-b").expect("#sec-b");
    let html = doc.query_selector(doc.root(), "html").expect("html");
    let body = doc.query_selector(doc.root(), "body").expect("body");
    let elements = vec![html, body, card_a, p1, p2, s1, sec_b];
    (
        doc,
        elements,
        Fixture {
            card_a,
            p1,
            p2,
            s1,
            sec_b,
        },
    )
}

/// 节点人读定位：`tag#id`。
fn describe_node(doc: &Document, id: NodeId) -> String {
    let Some(n) = doc.get(id) else {
        return format!("{id:?}（已删除）");
    };
    if let zero_dom::NodeKind::Element(e) = &n.kind {
        let mut s = e.local_name().to_string();
        if let Some(i) = doc.get_attribute(id, "id") {
            s.push('#');
            s.push_str(&i);
        }
        s
    } else {
        format!("{id:?}（非元素节点）")
    }
}

/// pre-order 收集 node-keyed 盒最终输出快照（匿名盒无 node_id，跳过——其错位会体现
/// 在兄弟盒几何上，被间接检出）。
fn collect_geometry(box_tree: &LayoutBox, out: &mut Vec<(NodeId, GeoSnapshot)>) {
    if let Some(id) = box_tree.node_id {
        out.push((
            id,
            GeoSnapshot {
                x: box_tree.x,
                y: box_tree.y,
                w: box_tree.width,
                h: box_tree.height,
                cw: box_tree.content_width,
                ch: box_tree.content_height,
                overflow_x: box_tree.overflow_x,
                overflow_y: box_tree.overflow_y,
            },
        ));
    }
    for child in &box_tree.children {
        collect_geometry(child, out);
    }
}

/// 首个最终输出快照分歧的描述；无分歧返 None。序列按序逐盒比较（条目数/顺序/逐字段精确相等）。
fn first_geometry_divergence(
    doc: &Document,
    a: &[(NodeId, GeoSnapshot)],
    b: &[(NodeId, GeoSnapshot)],
) -> Option<String> {
    if a.len() != b.len() {
        return Some(format!(
            "盒序列长度不一致：冷 {} vs 温 {}（首差处：{}）",
            a.len(),
            b.len(),
            a.iter()
                .zip(b.iter())
                .find(|(x, y)| x.0 != y.0)
                .map(|(x, _)| describe_node(doc, x.0))
                .unwrap_or_else(|| "序列尾部".to_string())
        ));
    }
    for ((ida, ga), (idb, gb)) in a.iter().zip(b.iter()) {
        if ida != idb {
            return Some(format!(
                "盒顺序不一致：冷 {} vs 温 {}",
                describe_node(doc, *ida),
                describe_node(doc, *idb)
            ));
        }
        if ga != gb {
            return Some(format!(
                "{} 最终输出快照不一致（x, y, w, h, cw, ch, ox, oy）：冷 {:?} vs 温 {:?}",
                describe_node(doc, *ida),
                ga,
                gb
            ));
        }
    }
    None
}

/// 双路径布局对照驱动器：同一 Document，两个独立 StyleSystem + 两个独立 LayoutEngine。
struct DualLayoutRun {
    doc: Document,
    elements: Vec<NodeId>,
    css: String,
    full_sys: StyleSystem,
    inc_sys: StyleSystem,
    /// 冷路径：每步 invalidate 后全量 compute（生产快照重建语义）。
    cold_engine: LayoutEngine,
    /// 温路径：保留 cached_state 跨步全量 compute（输入纯度护栏）。
    warm_engine: LayoutEngine,
    /// 增量样式路径的累积样式表（与生产 cached_styles base 复用一致）。
    inc_styles: HashMap<NodeId, ComputedStyle>,
}

impl DualLayoutRun {
    /// 首帧：两路径同为全量，样式与几何都必须一致（基线快照）。
    fn new(doc: Document, elements: Vec<NodeId>, css: &str) -> Self {
        let mut full_sys = StyleSystem::new();
        let mut inc_sys = StyleSystem::new();
        full_sys.set_viewport(800.0, 600.0);
        inc_sys.set_viewport(800.0, 600.0);
        let sheets = vec![CssParser::parse_stylesheet(css)];
        let full_styles = full_sys.compute_styles(&doc, &sheets);
        let inc_styles = inc_sys.compute_styles(&doc, &sheets);
        assert!(
            styles_equal(&doc, &elements, &full_styles, &inc_styles).is_none(),
            "首帧全量基线即样式分歧（双路径输入相同）"
        );
        let mut cold_engine = LayoutEngine::new(800.0, 600.0);
        let mut warm_engine = LayoutEngine::new(800.0, 600.0);
        let cold_layout = cold_engine.compute(&doc, &full_styles);
        let warm_layout = warm_engine.compute(&doc, &inc_styles);
        assert!(
            first_geometry_divergence(&doc, &geometry_of(&cold_layout), &geometry_of(&warm_layout)).is_none(),
            "首帧几何基线即分歧（双路径输入相同）"
        );
        Self {
            doc,
            elements,
            css: css.to_string(),
            full_sys,
            inc_sys,
            cold_engine,
            warm_engine,
            inc_styles,
        }
    }

    /// 施加一步最小变更并比对双路径。
    ///
    /// - `css` — 本步生效的 CSS 文本；`None` = 沿用上一步（DOM-only 变更）。
    /// - `mutate` — 施加 DOM 变更，返回增量样式路径的 changed 集合。
    ///
    /// 先断言样式双路径相等（布局分歧归因前置条件），再比对冷/温引擎几何。
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
        let full_styles = self.full_sys.compute_styles(&self.doc, &sheets);
        self.inc_sys
            .compute_styles_incremental(&self.doc, &sheets, &changed, &mut self.inc_styles);
        if let Some(detail) = styles_equal(&self.doc, &self.elements, &full_styles, &self.inc_styles) {
            panic!("步骤「{step_name}」样式双路径分歧（布局对照前置条件）：{detail}");
        }
        // 冷路径：invalidate 后全量——≡ 生产对布局影响型 mutation 的快照重建。
        self.cold_engine.invalidate_cache();
        let cold_layout = self.cold_engine.compute(&self.doc, &full_styles);
        // 温路径：cached_state 保留跨步，全量 compute 不消费它。
        let warm_layout = self.warm_engine.compute(&self.doc, &self.inc_styles);
        let divs = first_geometry_divergence(&self.doc, &geometry_of(&cold_layout), &geometry_of(&warm_layout));
        match expect {
            Expectation::Equal => assert!(
                divs.is_none(),
                "步骤「{step_name}」出现双路径几何分歧（期望一致）：{}",
                divs.unwrap_or_default()
            ),
        }
    }
}

fn geometry_of(result: &LayoutResult) -> Vec<(NodeId, GeoSnapshot)> {
    let mut out = Vec::new();
    collect_geometry(&result.root, &mut out);
    out
}

/// 逐元素样式相等检查（布局对照的前置条件）；返首个分歧描述。
fn styles_equal(
    doc: &Document,
    elements: &[NodeId],
    a: &HashMap<NodeId, ComputedStyle>,
    b: &HashMap<NodeId, ComputedStyle>,
) -> Option<String> {
    for &id in elements {
        match (a.get(&id), b.get(&id)) {
            (None, None) => {}
            (Some(_), None) => return Some(format!("{} 增量样式路径丢条目", describe_node(doc, id))),
            (None, Some(_)) => return Some(format!("{} 增量样式路径残留条目", describe_node(doc, id))),
            (Some(x), Some(y)) if x != y => return Some(format!("{} 计算样式不一致", describe_node(doc, id))),
            (Some(_), Some(_)) => {}
        }
    }
    None
}

// ── INC-03（DC-3）：布局影响属性 / 字体 / 结构变更 / 视口——冷温一致 ──────

/// INC-03a：布局影响属性（width）变更——冷重建与温缓存全量几何一致。
#[test]
fn layout_style_width_change_cold_warm_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualLayoutRun::new(doc, elements, BASE_CSS);
    run.apply(
        ".card 加宽 400→600（changed=card-a）",
        Some(WIDE_CSS),
        |_| vec![f.card_a],
        Expectation::Equal,
    );
    run.apply(
        ".card 缩回 400（同帧连变，changed=card-a）",
        Some(BASE_CSS),
        |_| vec![f.card_a],
        Expectation::Equal,
    );
}

/// INC-03b：字体失效（font-size 变更影响全部 p.text）——changed 集按完备性契约
/// 含两个受影响元素（style oracle 反例 A 同族契约），双路径几何一致。
#[test]
fn layout_font_invalidation_completeness_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualLayoutRun::new(doc, elements, BASE_CSS);
    run.apply(
        "p.text 字号 14→20（changed=p1,p2——CSS 影响两元素，契约须补全）",
        Some(BIGFONT_CSS),
        |_| vec![f.p1, f.p2],
        Expectation::Equal,
    );
}

/// INC-03c：结构变更（子树移动）——生产路由即快照全量重建，冷温几何一致。
#[test]
fn layout_structural_move_cold_warm_consistent() {
    let (doc, elements, f) = build_main_fixture();
    let mut run = DualLayoutRun::new(doc, elements, BASE_CSS);
    run.apply(
        "span#s1 移入 #sec-b（changed=s1）",
        None,
        |d| {
            d.remove_child(f.card_a, f.s1).unwrap();
            d.append_child(f.sec_b, f.s1).unwrap();
            vec![f.s1]
        },
        Expectation::Equal,
    );
    run.apply(
        "span#s1 移回 .card（changed=s1）",
        None,
        |d| {
            d.remove_child(f.sec_b, f.s1).unwrap();
            d.append_child(f.card_a, f.s1).unwrap();
            vec![f.s1]
        },
        Expectation::Equal,
    );
}

/// INC-03d：视口变更——`set_viewport` 失效缓存（温引擎回到冷语义），重建后一致；
/// 同时钉住 viewport 独立 CSS 下样式与布局都不漂移。
#[test]
fn layout_viewport_change_rebuild_consistent() {
    let (doc, elements, _f) = build_main_fixture();
    let mut run = DualLayoutRun::new(doc, elements, BASE_CSS);
    run.cold_engine.set_viewport(400.0, 600.0);
    run.warm_engine.set_viewport(400.0, 600.0);
    let sheets = vec![CssParser::parse_stylesheet(&run.css)];
    let full_styles = run.full_sys.compute_styles(&run.doc, &sheets);
    run.inc_sys
        .compute_styles_incremental(&run.doc, &sheets, &[], &mut run.inc_styles);
    assert!(
        styles_equal(&run.doc, &run.elements, &full_styles, &run.inc_styles).is_none(),
        "视口变更后样式基线分歧"
    );
    let cold_layout = run.cold_engine.compute(&run.doc, &full_styles);
    let warm_layout = run.warm_engine.compute(&run.doc, &run.inc_styles);
    assert!(
        first_geometry_divergence(&run.doc, &geometry_of(&cold_layout), &geometry_of(&warm_layout)).is_none(),
        "视口变更后冷温几何分歧"
    );
}

// ── M3 拆面（DC-4 布局面）：后处理敏感路径的冷温最终输出一致 ───────────
//
// slice14 撤用增量臂的根因是"复用 taffy 缓存、不跑全量后处理族"。本节在
// 后处理最敏感的路径（margin 折叠、float 避让、overflow 滚动容器）上钉住
// 冷温全量一致——未来任何"全量里跳过/重复某后处理 pass"的改动都会被检出。

/// 后处理序列共用：独立 fixture 的 (doc, ids)，ids = [html, body, #a, #b]。
fn postprocess_fixture(html: &str) -> (Document, Vec<NodeId>) {
    let doc = parse_html(html);
    let mut ids = Vec::new();
    for sel in ["html", "body", "#a", "#b"] {
        ids.push(doc.query_selector(doc.root(), sel).expect(sel));
    }
    (doc, ids)
}

/// fixture 有效性自检：三个后处理序列的 CSS 变更对确实改变全量输出——防
/// "双路径一致但都陈旧"的空转基线（对照器对无效果变更同样无感，故序列
/// fixture 须独立证明 layout-effective，同 style oracle 的检测器灵敏度哲学）。
#[test]
fn layout_postprocess_fixtures_are_layout_effective() {
    let cases: [(&str, &str, &str); 3] = [
        (
            "collapse",
            r#"<html><body><div id="a" class="gap-a">top</div><div id="b" class="gap-b">bottom</div></body></html>"#,
            "body { margin: 0; }\n.gap-a { margin-bottom: 30px; }\n.gap-b { margin-top: 20px; }\n|body { margin: 0; }\n.gap-a { margin-bottom: 15px; }\n.gap-b { margin-top: 20px; }\n",
        ),
        (
            "float",
            r#"<html><body><div id="a" class="fl">F</div><div id="b" class="bfc">flow content beside float</div></body></html>"#,
            "body { margin: 0; }\n.fl { float: left; width: 100px; height: 40px; }\n.bfc { display: flow-root; }\n|body { margin: 0; }\n.fl { float: left; width: 250px; height: 40px; }\n.bfc { display: flow-root; }\n",
        ),
        (
            "overflow",
            r#"<html><body><div id="a" class="scroll"><div id="b" class="tall">tall content</div></div></body></html>"#,
            "body { margin: 0; }\n.scroll { overflow: scroll; width: 200px; height: 50px; }\n.tall { height: 200px; }\n|body { margin: 0; }\n.scroll { overflow: visible; width: 200px; height: 50px; }\n.tall { height: 300px; }\n",
        ),
    ];
    for (name, html, pair) in cases {
        let (css1, css2) = pair.split_once('|').expect("CSS 对");
        let (doc, _ids) = postprocess_fixture(html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let mut e1 = LayoutEngine::new(800.0, 600.0);
        let mut e2 = LayoutEngine::new(800.0, 600.0);
        let g1 = geometry_of(&e1.compute(
            &doc,
            &sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css1)]),
        ));
        let g2 = geometry_of(&e2.compute(
            &doc,
            &sys.compute_styles(&doc, &[CssParser::parse_stylesheet(css2)]),
        ));
        assert_ne!(
            g1, g2,
            "后处理序列「{name}」的 CSS 变更对未改变全量输出——序列空转，须修 fixture"
        );
    }
}

/// M3-a：相邻兄弟 margin 折叠（折叠逃逸 + 兄弟位移后处理敏感路径）——
/// 折叠量随 CSS 变更时冷温最终输出一致（折叠本身的规范正确性属 rendering-compat，
/// 本序列只钉双路径一致性）。
#[test]
fn layout_margin_collapse_postprocess_consistent() {
    let (doc, ids) = postprocess_fixture(
        r#"<html><body><div id="a" class="gap-a">top</div><div id="b" class="gap-b">bottom</div></body></html>"#,
    );
    let a = ids[2];
    let mut run = DualLayoutRun::new(
        doc,
        ids,
        "body { margin: 0; }\n.gap-a { margin-bottom: 30px; }\n.gap-b { margin-top: 20px; }\n",
    );
    run.apply(
        ".gap-a 下边距 30→15（折叠值变 max(15,20)=20，#b 上移）",
        Some("body { margin: 0; }\n.gap-a { margin-bottom: 15px; }\n.gap-b { margin-top: 20px; }\n"),
        move |_| vec![a],
        Expectation::Equal,
    );
}

/// M3-b：float 避让（float 收缩/BFC 修正后处理敏感路径）——float 宽高变更时
/// 后续 BFC 块的几何冷温一致。
#[test]
fn layout_float_avoidance_postprocess_consistent() {
    let (doc, ids) = postprocess_fixture(
        r#"<html><body><div id="a" class="fl">F</div><div id="b" class="bfc">flow content beside float</div></body></html>"#,
    );
    let a = ids[2];
    let mut run = DualLayoutRun::new(
        doc,
        ids,
        "body { margin: 0; }\n.fl { float: left; width: 100px; height: 40px; }\n.bfc { display: flow-root; }\n",
    );
    run.apply(
        "float 加宽 100→250（BFC 块可用宽度收缩，changed=#a）",
        Some("body { margin: 0; }\n.fl { float: left; width: 250px; height: 40px; }\n.bfc { display: flow-root; }\n"),
        move |_| vec![a],
        Expectation::Equal,
    );
    run.apply(
        "float 改高 40→120（changed=#a）",
        Some("body { margin: 0; }\n.fl { float: left; width: 250px; height: 120px; }\n.bfc { display: flow-root; }\n"),
        move |_| vec![a],
        Expectation::Equal,
    );
}

/// M3-c：overflow 滚动容器（溢出枚举 + 内容几何）——内容增高与溢出模式切换
/// 冷温一致；溢出枚举入快照（DC-4"溢出符合预期"布局面）。
#[test]
fn layout_overflow_scroll_extent_consistent() {
    let (doc, ids) = postprocess_fixture(
        r#"<html><body><div id="a" class="scroll"><div id="b" class="tall">tall content</div></div></body></html>"#,
    );
    let a = ids[2];
    let b = ids[3];
    let mut run = DualLayoutRun::new(
        doc,
        ids,
        "body { margin: 0; }\n.scroll { overflow: scroll; width: 200px; height: 50px; }\n.tall { height: 200px; }\n",
    );
    run.apply(
        "内容增高 200→300（滚动范围变化，changed=#b）",
        Some("body { margin: 0; }\n.scroll { overflow: scroll; width: 200px; height: 50px; }\n.tall { height: 300px; }\n"),
        move |_| vec![b],
        Expectation::Equal,
    );
    run.apply(
        "溢出模式 scroll→visible（溢出枚举变更 + 容器高度不再钳制，changed=#a）",
        Some("body { margin: 0; }\n.scroll { overflow: visible; width: 200px; height: 50px; }\n.tall { height: 300px; }\n"),
        move |_| vec![a],
        Expectation::Equal,
    );
}

// ── 撤用增量臂（compute_incremental）的几何级边界固化 ─────────────────

/// 已知边界（几何级）：`compute_incremental` 的 taffy 树样式是建树快照——样式属性
/// 变更后 mark_dirty 只重算布局不更新 taffy style，增量几何保留旧值（头注
/// style_change_incremental_keeps_old_taffy_style_boundary 的序列化扩展 + slice14
/// 撤用依据）。本测试固化该边界在逐盒几何上的表现：宽度变更后 card-a 子树
/// 保留旧几何，对照器必须报出。重新启用前置条件见 incremental_parity_experiment 头注。
#[test]
fn layout_incremental_arm_style_change_diverges_boundary() {
    let (doc, _elements, f) = build_main_fixture();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let base = vec![CssParser::parse_stylesheet(BASE_CSS)];
    let styles_v1 = sys.compute_styles(&doc, &base);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let full_v1 = engine.compute(&doc, &styles_v1);
    assert!(engine.has_cached_state(), "compute 后应有缓存供增量臂");

    // 全量基线自检：新引擎同输入必须逐盒一致（对照器灵敏度前提）。
    let mut fresh = LayoutEngine::new(800.0, 600.0);
    let fresh_layout = fresh.compute(&doc, &styles_v1);
    assert!(
        first_geometry_divergence(&doc, &geometry_of(&full_v1), &geometry_of(&fresh_layout)).is_none(),
        "同输入全量基线自检分歧——对照器失灵"
    );

    // 布局影响样式变更：.card 400→600。全量参照走独立引擎（不动增量引擎的 v1 缓存）。
    let wide = vec![CssParser::parse_stylesheet(WIDE_CSS)];
    let styles_v2 = sys.compute_styles(&doc, &wide);
    let mut full_engine = LayoutEngine::new(800.0, 600.0);
    let full_v2 = full_engine.compute(&doc, &styles_v2);
    let mut tracker = LayoutDirtyTracker::new();
    tracker.mark_dirty(f.card_a);
    let (incr_result, stats) =
        engine.compute_incremental(&doc, &styles_v2, &mut tracker, &std::collections::HashMap::new());
    assert!(!stats.was_full_recalc, "本测试须走真增量臂（无缓存退全量则断言无意义）");
    let divs = first_geometry_divergence(&doc, &geometry_of(&full_v2), &geometry_of(&incr_result));
    assert!(
        divs.is_some(),
        "撤用增量臂在样式变更后应保留旧 taffy style 几何（已知边界），但与全量一致——\
         若为修复请更新 incremental_parity_experiment 头注与本断言"
    );
}
