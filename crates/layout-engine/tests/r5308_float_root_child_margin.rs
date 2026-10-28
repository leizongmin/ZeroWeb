//! R5308：float BFC 根不与子折叠 margin（CSS2 §8.3.1）回归钉。
//!
//! 根因（taffy-local compute/block.rs float 分支）：float 盒自身布局以
//! `vertical_margins_are_collapsible: Line::TRUE` 执行——float 根被允许与
//! 首流内子折叠 margin，子的 margin-top 被上提进 `first_child_top_margin_set`，
//! 而 float 分支只消费 `item_layout.size` 与 float 自身 margin，上提值无人
//! 接收 → 整体丢失（子 y=0、auto 高 float 根不含子 mt）。§8.3.1：float 建立
//! BFC，「Margins between a floated box and any other box do not collapse
//! (not even between a float and its in-flow children)」——须传 `Line::FALSE`。
//!
//! 驱动案：floats-rule3/7-outside-{left,right}-002/001 的 ref 页结构
//! （root float 500×500 → 块子 margin-top:300，chromium 蓝条渲染在 y=300）。
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5308-float-root-child-margin-2026-10-28.md
//!
//! https://www.w3.org/TR/CSS22/box.html#collapsing-margins

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

/// 块级样式（w/h px，可选 margin-top px）。
fn block(w: f64, h: f64, mt: f64) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        margin_top: LengthValue::Px(mt),
        ..Default::default()
    }
}

/// 块级浮动样式（w/h px，auto_height=true 时 height:auto）。
fn block_float(direction: FloatValue, w: f64, h: Option<f64>) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        float: direction,
        width: LengthValue::Px(w),
        height: h.map(LengthValue::Px).unwrap_or(LengthValue::Auto),
        ..Default::default()
    }
}

/// 按 DOM 节点 ID 在布局树中递归查找对应 LayoutBox。
fn find_box(box_tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if box_tree.node_id == Some(id) {
        return Some(box_tree);
    }
    box_tree.children.iter().find_map(|c| find_box(c, id))
}

/// floats-rule3-outside-left-002-ref 结构：body → root float（left，500×`h`）
/// → 单块子（mt:300，475×10）。返回 (doc, styles, float_root_id, child_id)。
fn build_ref_page(float_h: Option<f64>) -> (Document, HashMap<NodeId, ComputedStyle>, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let float_root = doc.create_element("div");
    doc.append_child(body, float_root).unwrap();
    let child = doc.create_element("div");
    doc.append_child(float_root, child).unwrap();

    let mut styles = HashMap::new();
    styles.insert(float_root, block_float(FloatValue::Left, 500.0, float_h));
    styles.insert(child, block(475.0, 10.0, 300.0));
    (doc, styles, float_root, child)
}

/// 定高 float 根（ref 页形态）：子的 margin-top 不得折叠出 float 根——
/// 子渲染在 float 根内容系 y=300（chromium 蓝条位）。
#[test]
fn fixed_height_float_root_keeps_child_margin_top() {
    let (doc, styles, _float_root, child) = build_ref_page(Some(500.0));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let child_box = find_box(&result.root, child).expect("child 布局盒存在");
    assert_eq!(
        child_box.y, 300.0,
        "float 根（BFC）不与子折叠 margin（§8.3.1）：子 margin-top 300 须保留在根内，子 y=300"
    );
}

/// auto 高 float 根：子 mt 折叠出根会连根高度一起丢——chromium 根高含子 mt
/// （310）；修复前 mt 上提丢失，根高只剩子高（10）。
#[test]
fn auto_height_float_root_contains_child_margin_top() {
    let (doc, styles, float_root, _child) = build_ref_page(None);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let root_box = find_box(&result.root, float_root).expect("float 根布局盒存在");
    assert_eq!(
        root_box.height, 310.0,
        "auto 高 float 根高须含首子 margin-top（300+10=310，§8.3.1 不折叠）"
    );
}
