//! R4988：flex AR 条目 automatic minimum 地板（css-flexbox §4.5 + css-sizing-4 §4.1）。
//! 容器 main definite 且小于 transferred 值时，taffy flex-shrink 把 size.main 缩穿
//! （045：容器 w:0 h:100 + item AR 1 → 应溢出 100 方，ZW 收缩 0 宽）。地板仅
//! min-*:auto（014 回归案：min-height:0 禁用 automatic minimum）且无 auto margin
//!（auto-margins-002 回归案：margin:auto 条目走 max 钳 + 居中路径）。
//! driving: WPT css-sizing/aspect-ratio/flex-aspect-ratio-045/053/054、flex-aspect-ratio-014。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;

fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find(c, id))
}

/// 建树：flex 容器（w:0 h:100）+ AR 1/1 条目；返回 (doc, styles, item id)。
fn flex_container_with_ar_item(min_height: Option<f64>) -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(0.0);
    container_style.height = LengthValue::Px(100.0);

    let mut item_style = ComputedStyle::default();
    item_style.aspect_ratio = Some(1.0);
    if let Some(v) = min_height {
        item_style.min_height = LengthValue::Px(v);
    }

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);
    (doc, styles, item)
}

#[test]
/// 臂 1（auto main）：容器 main 0 收缩竞争下 transferred minimum 地板——item 100×100。
fn r4988_flex_ar_min_floors_against_shrink() {
    let (doc, styles, item) = flex_container_with_ar_item(None);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find(&result.root, item).expect("item box").clone();
    assert_eq!(
        (b.width, b.height),
        (100.0, 100.0),
        "transferred minimum (cross 100 × ratio 1) must floor flex-shrink in 0-width container"
    );
}

#[test]
/// min-height:0 守卫：显式零禁用 automatic minimum，不得地板撑爆（014 语义域）。
fn r4988_flex_ar_min_height_zero_disables_floor() {
    let (doc, styles, item) = flex_container_with_ar_item(Some(0.0));
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find(&result.root, item).expect("item box").clone();
    assert!(
        b.height <= 100.5 && b.width <= 100.5,
        "min-height:0 disables the transferred-minimum floor; box must stay within container cross"
    );
}
