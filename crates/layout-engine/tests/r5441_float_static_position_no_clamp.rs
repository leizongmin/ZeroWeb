//! R5441 kill-switch 锚（独立测试进程——R3858 教训：进程内 set_var 与并行单测互踩，
//! 本文件仅此一测，独占进程环境）：`ZW_FLOAT_ANCHOR_Y=0` 关闭 R3784 钳位后，
//! stack-floats-003 形 float 仍落 chrome 位（静态位地板修复自身到位，不依赖钳位
//! 救场）——R5440 挂账③「钳位退役条件」的本形证据。
//!
//! 修复前本态：walk band_y=0 → border −100，无钳位即泄露
//!（corpus 关臂实证 stack-floats-003 ✗2.35%）。
//!
//! 谱系与几何同 `r5441_float_inline_line_fit_floor.rs` no_fit 测试。

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

#[test]
fn no_fit_float_lands_chrome_position_without_anchor_clamp() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；LazyLock 首读前生效。
    unsafe { std::env::set_var("ZW_FLOAT_ANCHOR_Y", "0") };

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

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
        if tree.node_id == Some(id) {
            return Some(tree);
        }
        tree.children.iter().find_map(|c| find(c, id))
    }
    let float_box = find(&result.root, float).expect("float 布局盒存在");
    assert_eq!(
        float_box.y, 0.0,
        "R3784 钳位关闭后静态位地板独立落位：border = inline 流底 100 + mt (−100) = 0"
    );
}
