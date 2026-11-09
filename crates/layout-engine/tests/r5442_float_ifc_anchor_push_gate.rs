//! R5442：R3784 锚位推压 no-preceding-content 门（CSS 2.1 §9.5.1 规则 6）回归钉。
//!
//! 根因（R5440 死锁表 test4 行 bug 源）：锚位 0 = IFC 起点（float 行首且容器无前文
//! 内容行）时规则 6 无更早行盒可约束——静态位（adjust_float_positions band+mt）已
//! 合规；旧推压 `anchor > child.y` 无条件触发，把负 mt float 从 chrome 位抬回容器顶
//!（c5525-fltmrgn test4：img mt −30 walk 位 = chrome 位，推后高 30px）。
//!
//! 修复：仅前文有完整内容行（行槽顶锚 > 0，line-clamp-auto-015 形 L1-L4 后 128）
//! 推压；锚位 0 不推。layout-dump 双态实证：test4 img 关钳位 278 abs（=walk 静态位
//! =chrome 位）、默认 308（推坏）；auto-015 float 默认 136 abs（推压保留）。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5442-ifc-float-anchor-gate-2026-11-07.md
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

/// c5525-fltmrgn test4 形：container(100) → float(45×45, mt:−30, ml:−30 行首) → 文本
/// "x"（同行后随）。返回 (doc, styles, float_id)。
fn build_ifc_start_float_page() -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float = doc.create_element("img");
    doc.append_child(container, float).unwrap();
    let text = doc.create_text_node("x");
    doc.append_child(container, text).unwrap();

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
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(45.0),
            height: LengthValue::Px(45.0),
            margin_top: LengthValue::Px(-30.0),
            margin_left: LengthValue::Px(-30.0),
            ..Default::default()
        },
    );
    (doc, styles, float)
}

/// IFC 起点 float（无前文内容行）负 mt 不被锚位推压：静态位 mt −30 保留（旧推压
/// 无条件触发 → 抬回 0，chrome 位 278 abs 变 308；dump 双态实证）。
#[test]
fn ifc_start_negative_margin_float_keeps_static_position() {
    let (doc, styles, float) = build_ifc_start_float_page();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let float_box = find_box(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, -30.0,
        "锚位 0（无前文内容行）不推压：静态位 band 0 + mt (−30) = −30 保留（规则 6 无更早行盒可约束）"
    );
}

/// auto-015 保护面：前文完整内容行（pre-wrap 4 行，行槽顶锚 128 > 0）推压保留——
/// walk 缺前文行高知识落 0，锚位推压把 float 抬到行槽顶 128（钳位仍必要面）。
#[test]
fn float_after_complete_lines_still_pushed_to_anchor() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let text = doc.create_text_node("L1\nL2\nL3\nL4\n");
    doc.append_child(container, text).unwrap();
    let float = doc.create_element("div");
    doc.append_child(container, float).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(200.0),
            // pre-wrap：源码 \n 折行成 4 行；行高显式 32px（auto-015 形 16px/32px 同构）
            white_space: zero_style_system::property::types::WhiteSpaceValue::PreWrap,
            line_height: zero_style_system::property::types::LineHeightValue::Length(LengthValue::Px(32.0)),
            ..Default::default()
        },
    );
    styles.insert(
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(300.0),
            height: LengthValue::Px(32.0),
            ..Default::default()
        },
    );

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let float_box = find_box(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, 128.0,
        "前文 4 完整行（行槽顶锚 128 > 0）推压保留：float 抬到行槽顶（walk 无行高知识落 0）"
    );
}

/// R5442 排除带可见段：IFC 起点 float mt −30 静态位 −30 后，margin-box 带 [0,15]
/// 跨容器顶——不可折长词绕不进带侧剩余宽（100 − 15），整行被推到带底 15 起排
///（容器高 = 15 + 行高 32 = 47）；旧 rel_y<0 整带丢弃 → 行停 0（容器高 32）。
#[test]
fn ifc_start_negative_margin_float_exclusion_bands_visible_segment() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float = doc.create_element("img");
    doc.append_child(container, float).unwrap();
    // 100×'x' 单词 @16px ≫ 带侧剩余宽 85px：不可折 → 整行推到带底。
    let long_word = "x".repeat(100);
    let text = doc.create_text_node(&long_word);
    doc.append_child(container, text).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            line_height: zero_style_system::property::types::LineHeightValue::Length(LengthValue::Px(32.0)),
            ..Default::default()
        },
    );
    styles.insert(
        float,
        ComputedStyle {
            display: DisplayValue::Block,
            float: FloatValue::Left,
            width: LengthValue::Px(45.0),
            height: LengthValue::Px(45.0),
            margin_top: LengthValue::Px(-30.0),
            margin_left: LengthValue::Px(-30.0),
            ..Default::default()
        },
    );

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let container_box = find_box(&result.root, container).expect("容器布局盒存在");
    assert_eq!(
        container_box.height, 47.0,
        "跨顶带裁顶保留可见段 [0,15]：不可折长词绕不进带侧 → 整行推到带底 15，容器高 15+32=47（整带丢弃时为 32）"
    );
}
