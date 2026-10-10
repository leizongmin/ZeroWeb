//! R5047：flex AR 叶条目 flexing 驱动 main（csswg #10997）。
//! taffy 把 definite max-cross 经比反传为 transferred max-main 钳 flexed main
//!（transferred-max-size：flex:1 + ratio 1/2 + max-height:100 在 definite 100
//! 容器渲 50×100；interop 应 main=100（flex 求解胜）+ cross = clamp(200,100)=100）。
//! driving: WPT css-flexbox/aspect-ratio-transferred-max-size.html。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;

fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find(c, id))
}

/// 建树：row flex 容器（w:100）+ 条目（AR 1/2 + max-height:100 + flex:1）。
fn transferred_max_size_tree() -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(100.0);
    container_style.border_top_width = LengthValue::Px(0.0);
    container_style.border_bottom_width = LengthValue::Px(0.0);
    container_style.border_left_width = LengthValue::Px(0.0);
    container_style.border_right_width = LengthValue::Px(0.0);

    // flex item 块化（css-flexbox §3.2）：div → display:block；真实样式链 border 0。
    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;
    item_style.border_top_width = LengthValue::Px(0.0);
    item_style.border_bottom_width = LengthValue::Px(0.0);
    item_style.border_left_width = LengthValue::Px(0.0);
    item_style.border_right_width = LengthValue::Px(0.0);
    item_style.aspect_ratio = Some(0.5);
    item_style.max_height = LengthValue::Px(100.0);
    item_style.flex_grow = 1.0;
    item_style.flex_basis = zero_style_system::FlexBasisValue::Length(LengthValue::Percentage(0.0));

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);
    (doc, styles, item)
}

#[test]
/// csswg #10997：flex 求解的 main 不受 transferred cross-max 反传钳——100×100。
fn r5047_flexed_main_wins_over_transferred_cross_max() {
    let (doc, styles, item) = transferred_max_size_tree();
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find(&result.root, item).expect("item box").clone();
    assert_eq!(
        (b.width, b.height),
        (100.0, 100.0),
        "flex-resolved main must win over transferred max-cross clamp; got {:?}",
        (b.width, b.height)
    );
}

/// R5047 替换元素 sizing 臂：块轴 definite（height % 解析于 definite 父高）+ 比信号
/// → 内联轴 transferred 定值，内联 min/max 不参与（css-sizing-4 §4.4 + css-sizing-3
/// §5.1）。driving: WPT css-sizing/replaced-aspect-ratio-stretch-fit-003。
#[test]
fn r5047_svg_definite_block_transfer_ignores_inline_minmax() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let svg = doc.create_element("svg");
    {
        let elem = doc.get_mut(svg).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("viewBox", "0 0 1 1");
        }
    }
    doc.append_child(container, svg).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Block;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);

    let mut svg_style = ComputedStyle::default();
    svg_style.display = zero_style_system::DisplayValue::Block;
    svg_style.height = LengthValue::Percentage(100.0);
    // max-width:50px 应被 transferred 定值绕过（inline min/max 不参与）。
    svg_style.max_width = LengthValue::Px(50.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(svg, svg_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find(&result.root, svg).expect("svg box").clone();
    assert_eq!(
        (b.width, b.height),
        (100.0, 100.0),
        "definite block + ratio must transfer inline size past inline min/max; got {:?}",
        (b.width, b.height)
    );
}
