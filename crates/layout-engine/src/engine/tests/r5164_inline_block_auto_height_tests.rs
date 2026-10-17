//! R5164（css-tables §consecutive-boxes / csswg#8358 + CSS2 §10.6.3）：auto 高
//! inline-block 的父容器 remeasure 高度与 table 内 inline-block 行位。
//!
//! 双缺陷：
//! 1. **auto 高 inline-block 空白换行塌高**：父容器 remeasure 期子盒 LayoutBox 高
//!    尚未落定（0）→ `collect_items` 对「无有效尺寸」原子盒降级为零宽 TextRun
//!    （`w>0 && h>0` 门）→ 换行机会丢失、`lines=1` → shrink 臂把已正确的父高砍到
//!    单行（css-tables/whitespace-001 ref 侧 38→20）。修复：`h<=0` 且子自身有
//!    inline 内容时以**子自身行高 strut** 估计（auto 高 inline-block 的内容高 ≈
//!    其首行 strut）；空 spacer 与 `contain:size` 维持 0（内容高 0 是语义）。
//!    kill-switch `ZW_IB_AUTO_H_ATOMIC=0`。
//! 2. **table 容器 inline-block 行位 claim 被 step 8 打回**：remeasure 的
//!    position-reuse claim 对 Table/InlineTable 过早生效——step 8
//!    `adjust_table_layout_with_fonts` 随后重排表内子盒打回行位，而 step 10
//!    （跳过名单不含 Table）因 claim 跳过不再纠正 → 匿名表 cell 内第二个
//!    inline-block 叠回 (0,0)（whitespace-001 test 侧；显式高变体预存于 HEAD）。
//!    修复：Table/InlineTable 不 claim，行位交回 step 10 既有路径。
//!    kill-switch `ZW_TABLE_POS_CLAIM=0`。
//!
//! 探针无 FontLoader：断言用关系（b.y == a.height、容器高 = 2×行高 + 边框）而非
//! 绝对钉值，行高随环境字体浮动时关系恒成立。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 按 id 在布局树中查找 LayoutBox。
fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

/// #outer/#a/#b 三盒几何 (x, y, width, height)。
struct Geom {
    outer: (f32, f32, f32, f32),
    a: (f32, f32, f32, f32),
    b: (f32, f32, f32, f32),
}

fn geom_of(outer: &LayoutBox, a: &LayoutBox, b: &LayoutBox) -> Geom {
    let rect = |x: &LayoutBox| (x.x, x.y, x.width, x.height);
    Geom {
        outer: rect(outer),
        a: rect(a),
        b: rect(b),
    }
}

/// 布局 whitespace-001 形：#outer（指定 display/宽）内两个 50% inline-block 子
/// （中间空白 + 注释文本节点），可选 .half 追加样式（如显式高）。返回三盒几何。
fn probe_wrap_form(outer_display: &str, extra: &str) -> Geom {
    let html = format!(
        r#"<html><body style="margin:0">
        <div id="outer" style="{outer_display}">
          <div id="a" class="half">A</div>
          <!-- White space here should take up space -->
          <div id="b" class="half">B</div>
        </div>
        </body></html>"#
    );
    let css = format!("#outer {{ border: 1px solid green; }}\n.half {{ display: inline-block; width: 50%; {extra} }}");
    let doc = zero_dom::parse_html(&html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(&css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    geom_of(
        find("outer", &doc, &result.root).expect("outer box"),
        find("a", &doc, &result.root).expect("a box"),
        find("b", &doc, &result.root).expect("b box"),
    )
}

/// whitespace-001 ref 形（display:block 外壳）：两个 auto 高 50% inline-block 间
/// 空白产生换行机会 → B 落第二行（y == a.height），容器高 = 2 行 + 上下边框
/// （修复前 shrink 臂砍到单行：容器高 ≈ 1 行 + 边框）。
#[test]
fn test_block_auto_height_inline_blocks_wrap_keeps_two_line_height() {
    let g = probe_wrap_form("display: block; width: 500px; background: purple", "");
    assert!(g.a.3 > 0.0, "auto-height inline-block must keep strut height");
    assert_eq!(g.b.1, g.a.3, "B must start on the second line (break opportunity)");
    assert_eq!(
        g.outer.3,
        g.a.3 * 2.0 + 2.0,
        "container height must cover both lines + 1px borders"
    );
}

/// whitespace-001 test 形（display:table 外壳）：匿名 cell 内空白必须占位
/// （css-tables §consecutive-boxes / csswg#8358）——B 落第二行且不被 step 8 表格
/// pass 打回 (0,0) 叠 A（claim 链坑；显式高变体预存于 HEAD）。
#[test]
fn test_anon_table_cell_whitespace_takes_space_b_not_overlapping() {
    let g = probe_wrap_form("display: table; width: 500px; background: purple", "");
    assert!(g.a.3 > 0.0, "auto-height inline-block must keep strut height");
    assert_eq!(g.b.1, g.a.3, "whitespace inside anonymous cell must take up space");
    assert_eq!(
        g.outer.3,
        g.a.3 * 2.0 + 2.0,
        "table container height must cover both lines + 1px borders"
    );
}

/// table + 显式高 inline-block（h:20）：与 strut 估计无关，纯 claim 链回归——
/// B 不得被 step 8 打回 (0,0)（修复前 HEAD 即红：B 叠 A）。
#[test]
fn test_table_explicit_height_inline_blocks_survive_table_pass() {
    let g = probe_wrap_form("display: table; width: 500px; background: purple", "height: 20px;");
    assert_eq!(g.a.3, 20.0, "explicit height must be honored");
    assert_eq!(g.b.1, 20.0, "B must sit on second line, not clobbered back to (0,0)");
    assert_eq!(g.outer.3, 42.0, "container = 2 lines of 20 + 1px borders");
}

/// 最小形（无 border/padding/背景）：151px 容器两个 100px auto 高 span——
/// shrink 臂不得把正确两行高砍回单行（修复前 36→18）。
#[test]
fn test_shrink_arm_not_cutting_wrapped_auto_height_line() {
    let html = r#"<html><body style="margin:0">
        <div id="outer" style="width: 151px"><span id="a" style="display:inline-block;width:100px">A</span> <span id="b" style="display:inline-block;width:100px">B</span></div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let g = geom_of(
        find("outer", &doc, &result.root).expect("outer box"),
        find("a", &doc, &result.root).expect("a box"),
        find("b", &doc, &result.root).expect("b box"),
    );
    assert_eq!(g.b.1, g.a.3, "B on second line at exactly one line height");
    assert_eq!(g.outer.3, g.a.3 * 2.0, "container = two lines (no frame)");
}

/// 对照组：两个 auto 高 inline-block **无空白**——无换行机会，同行排布，
/// 容器单行高（shrink/strut 路径不得误增高度）。
#[test]
fn test_no_whitespace_keeps_single_line() {
    let html = r#"<html><body style="margin:0">
        <div id="outer" style="width: 500px"><div id="a" style="display:inline-block;width:50%">A</div><div id="b" style="display:inline-block;width:50%">B</div></div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let g = geom_of(
        find("outer", &doc, &result.root).expect("outer box"),
        find("a", &doc, &result.root).expect("a box"),
        find("b", &doc, &result.root).expect("b box"),
    );
    assert_eq!(
        (g.b.0, g.b.1),
        (g.a.2, 0.0),
        "no break opportunity: B beside A on line 1"
    );
    assert_eq!(g.outer.3, g.a.3, "container = single line");
}
