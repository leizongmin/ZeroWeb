//! R3755：BFC 元素（flow-root / inline-block / contain:layout|paint / overflow:hidden|clip）
//! 的垂直 margin 不与子元素折叠（CSS 2.1 §8.3.1 + CSS Contain §3.1）。
//!
//! converter 把 computed style 的 BFC-ness 注入 taffy `margin_collapse_isolation` 旗标，
//! taffy block 据此停用 own_margins_collapse_with_children 并阻止 collapse-through。
//! driving: css/css-contain/contain-content-002（嵌套 contain:content 链，子 mt 全被
//! 折叠出父盒 → 三层背景同 y 重叠）。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_style_system::property::types::ContainComputedValue;

/// 构造 body > parent > child（child margin:30px 0，height 20px），返回 (engine result, parent_id, child_id)。
fn layout_bfc_parent_with_margin_child(
    parent_setup: impl FnOnce(&mut ComputedStyle),
) -> (crate::LayoutResult, NodeId, NodeId) {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.width = LengthValue::Px(200.0);
    parent_setup(&mut parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.margin_top = LengthValue::Px(30.0);
    child_style.margin_bottom = LengthValue::Px(30.0);
    child_style.height = LengthValue::Px(20.0);
    child_style.width = LengthValue::Px(100.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (result, parent, child)
}

fn find_by_id(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_by_id(c, id))
}

#[test]
/// R4251：flow-root 父建立 BFC——子 mt 不再折叠出父盒（CSS2 §9.4.1）。R3755 曾因
/// float-adjacent 收缩几何回归收窄；R4251 在**无 float 文档**（ctx.has_any_float gate）
/// 启用 taffy overflow:Hidden 隔离臂，float-adjacent 回归面（bfc-next-to-float-2 /
/// margin-trim block-in-inline-005）被 gate 整体排除。本测试文档无 float → 隔离生效。
fn r4251_flow_root_parent_does_not_collapse_child_margin_top() {
    let (result, _parent, child) = layout_bfc_parent_with_margin_child(|s| {
        s.display = zero_style_system::DisplayValue::FlowRoot;
    });
    let root = &result.root;
    let c = find_by_id(root, child).expect("child box");
    assert!(
        (c.y - 30.0).abs() < 0.5,
        "flow-root 父内子 margin-top 应保留在内容盒内（child.y={}, 期望 30）",
        c.y
    );
}

#[test]
fn r3755_contain_content_parent_does_not_collapse_child_margin_top() {
    let (result, _parent, child) = layout_bfc_parent_with_margin_child(|s| {
        s.contain = ContainComputedValue::Content;
    });
    let root = &result.root;
    let c = find_by_id(root, child).expect("child box");
    assert!(
        (c.y - 30.0).abs() < 0.5,
        "contain:content 父内子 margin-top 应保留在内容盒内（child.y={}, 期望 30）",
        c.y
    );
}

#[test]
fn r3755_overflow_hidden_parent_does_not_collapse_child_margin_top() {
    let (result, _parent, child) = layout_bfc_parent_with_margin_child(|s| {
        s.overflow_x = zero_style_system::OverflowValue::Hidden;
    });
    let root = &result.root;
    let c = find_by_id(root, child).expect("child box");
    assert!(
        (c.y - 30.0).abs() < 0.5,
        "overflow:hidden 父内子 margin-top 应保留在内容盒内（child.y={}, 期望 30）",
        c.y
    );
}

#[test]
/// 对照组：非 BFC 父维持折叠语义（子 mt 与父 mt 折叠，child.y 相对父内容盒顶 = 0）。
fn r3755_plain_block_parent_still_collapses_child_margin_top() {
    let (result, _parent, child) = layout_bfc_parent_with_margin_child(|_s| {});
    let root = &result.root;
    let c = find_by_id(root, child).expect("child box");
    assert!(c.y.abs() < 0.5, "非 BFC 父子 mt 应折叠（child.y={}, 期望 0）", c.y);
}
