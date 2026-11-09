//! R5441：float 静态位地板——inline 级前序占据参与落位（CSS 2.1 §9.5）回归钉。
//!
//! 根因链（stack-floats-003 形）：float_positioning walk 中 R1277② 跳过 inline 级
//! 子的 flow_bottom 推进（floats-006 行顶旁落不退），float 落位带试探只看 float 带
//! → 满宽 inline 后的 float 落 band_y=0（行顶）→ `child.y = band_y + margin_top`
//! 负 margin 直出容器顶（border −100），靠 R3784 钳位救场回 0。
//!
//! 修复：R1277② 跳过的 inline 级前序子记宽度占据 Σ 与行内容流底——float 旁放不下
//! （宽 > 容器宽 − 占据 Σ）时静态位地板（inline 流底）抬升落位起点（fit 判定仅用
//! 宽度，rtl 免疫：floats-placement-007 rtl float-left 贴左缘旁落不受扰）。
//! chrome 落位：贴行顶旁放得下 → beside（floats-placement-006）；放不下 → 下移到
//! 行内容流底（静态位；stack-floats-003：流底 100 + mt −100 → border 0）。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5440-float-stack-ref-attribution-2026-11-07.md
//!
//! https://www.w3.org/TR/CSS22/visuren.html#float-position

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

fn find_box(box_tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if box_tree.node_id == Some(id) {
        return Some(box_tree);
    }
    box_tree.children.iter().find_map(|c| find_box(c, id))
}

/// stack-floats-003 形：container(100) → inline div（含 100×100 块子）→
/// float(100×100, mt:−100)。返回 (doc, styles, float_id)。
fn build_stack_floats_page() -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let inline = doc.create_element("div");
    doc.append_child(container, inline).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(inline, inner).unwrap();
    let float = doc.create_element("div");
    doc.append_child(container, float).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            ..Default::default()
        },
    );
    styles.insert(
        inline,
        ComputedStyle {
            display: DisplayValue::Inline,
            ..Default::default()
        },
    );
    styles.insert(
        inner,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            height: LengthValue::Px(100.0),
            ..Default::default()
        },
    );
    styles.insert(
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(100.0),
            height: LengthValue::Px(100.0),
            margin_top: LengthValue::Px(-100.0),
            ..Default::default()
        },
    );
    (doc, styles, float)
}

/// 不 fit 形（stack-floats-003）：满宽 inline 后 mt:−100 float——旁放不下，静态位
/// 地板（inline 流底 100）抬升落位 → border = 100 + (−100) = 0（chrome 位；修复前
/// band_y=0 → border −100，借 R3784 钳位救场）。
#[test]
fn no_fit_float_descends_to_inline_flow_bottom() {
    let (doc, styles, float) = build_stack_floats_page();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let float_box = find_box(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, 0.0,
        "满宽 inline 后 mt:−100 float 静态位落位：border = inline 流底 100 + mt (−100) = 0（§9.5，非行顶 −100）"
    );
}

/// fit 形（floats-placement-006）：50 宽 inline-block 后 50 宽 float——旁放得下，
/// 行顶旁落不受地板扰动（修复前墙式带建模曾误下探 rtl/旁落形：r5441 轮 A/B 实证）。
#[test]
fn fit_float_stays_at_line_top() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let inline_block = doc.create_element("div");
    doc.append_child(container, inline_block).unwrap();
    let float = doc.create_element("div");
    doc.append_child(container, float).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            ..Default::default()
        },
    );
    styles.insert(
        inline_block,
        ComputedStyle {
            display: DisplayValue::InlineBlock,
            width: LengthValue::Px(50.0),
            height: LengthValue::Px(100.0),
            ..Default::default()
        },
    );
    styles.insert(
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(50.0),
            height: LengthValue::Px(100.0),
            ..Default::default()
        },
    );

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let float_box = find_box(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, 0.0,
        "50+50 旁放得下：float 贴行顶旁落（y=0），不因 inline 占据下探"
    );
}

/// 纯文本 inline 占据不可信形（floats-clear/floats-006）：auto 宽无块级子的 inline 的
/// taffy 宽是全行宽（r109/remeasure 收缩在后），不计入占据 Σ——float 不因虚高占据
/// 误判放不下而下探（修复首版 corpus 实证 ✗10.42% 回归根因）。
#[test]
fn text_only_inline_does_not_trigger_descent() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let inline = doc.create_element("span");
    doc.append_child(container, inline).unwrap();
    let text = doc.create_text_node("X");
    doc.append_child(inline, text).unwrap();
    let float = doc.create_element("div");
    doc.append_child(container, float).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(300.0),
            ..Default::default()
        },
    );
    styles.insert(
        inline,
        ComputedStyle {
            display: DisplayValue::Inline,
            ..Default::default()
        },
    );
    styles.insert(
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(100.0),
            height: LengthValue::Px(100.0),
            ..Default::default()
        },
    );

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let float_box = find_box(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, 0.0,
        "纯文本 inline 占据不可信：float 不因虚高占据下探，贴行顶旁落（y=0）"
    );
}
