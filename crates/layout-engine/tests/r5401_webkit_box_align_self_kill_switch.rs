//! R5401 kill-switch 回退锚（独立测试进程——`ZW_WEBKIT_BOX_ALIGN_SELF_IGNORE=0`
//! 时 legacy `-webkit-box` 容器不再抑制 item align-self，safe-center 溢出回退重新
//! 生效；进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 `r5401_webkit_box_align_self.rs`（006 形：容器 90×90 align-items:end
//! + item 100×100 align-self:safe center）。

use std::collections::HashMap;
use zero_css_parser::values::{AlignmentValue, DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

#[test]
fn kill_switch_restores_align_self_in_legacy_box() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_WEBKIT_BOX_ALIGN_SELF_IGNORE", "0") };

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Flex,
            width: LengthValue::Px(90.0),
            height: LengthValue::Px(90.0),
            align_items: AlignmentValue::End,
            legacy_webkit_box_flex: true,
            ..Default::default()
        },
    );
    styles.insert(
        item,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            height: LengthValue::Px(100.0),
            align_self: AlignmentValue::SafeCenter,
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
    let c = find(&result.root, container).expect("container 布局盒存在");
    let i = find(&result.root, item).expect("item 布局盒存在");
    let dy = i.y - c.y;
    assert!(
        dy.abs() <= 0.5,
        "开关关闭：align-self 不被抑制 → safe-center 溢出回退 start（dy=0）；dy={dy}"
    );
    unsafe { std::env::remove_var("ZW_WEBKIT_BOX_ALIGN_SELF_IGNORE") };
}
