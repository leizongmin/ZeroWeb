//! R4149：块盒 min/max-width 的 content 关键字（min/max-content/fit-content）=
//! content-based 尺寸（css-sizing-3 §5.2 + csswg #3973）。converter 把 min_width
//! 关键字映射 length(0)、max_width 关键字映射 auto（taffy 无 content-keyword 概念），
//! `apply_intrinsic_content_sizing` pass 在此测 intrinsic 后经 taffy 重跑钳制：
//! min 侧 floor（dynamic-011：`min-width:min-content; width:0px` 内子传宽 100 应撑到
//! 100），max 侧 cap（block-aspect-ratio 域 border-box-and-max-content-002：AR +
//! content-box 盒的 `max-width:max-content` 应按 border-box cap）。
//! driving: WPT css-sizing/intrinsic-percent-replaced-dynamic-011、
//! css-sizing/block-size-with-min-or-max-content-6、css-sizing/border-box-and-max-content-002。

use super::*;

/// `min-width:min-content; width:0px` 的 abspos 盒含 100px 宽子 → 宽须撑到 100
///（converter 把 min_width 关键字塌成 0，无此钳制时盒宽 0）。
#[test]
fn r4149_min_width_min_content_floors_to_content() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.position = zero_css_parser::values::PositionValue::Absolute;
    parent_style.width = LengthValue::Px(0.0);
    parent_style.min_width = LengthValue::MinContent;

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(100.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let p = find(&result.root, parent).expect("parent box");
    assert!(
        (p.width - 100.0).abs() < 1.0,
        "min-width:min-content must floor box width to content width 100, got {}",
        p.width
    );
}

/// 无 AR 的普通块 `max-width:min-content; width:200px`：max 关键字 cap 臂按 R4149
/// 收窄 gate 不触发（intrinsic 按 content-box 求和对普通块高估，cap 会塌盒），
/// 维持 taffy Auto 行为（宽 200）。
#[test]
fn r4149_max_width_keyword_cap_gated_to_ar_content_box() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.width = LengthValue::Px(200.0);
    parent_style.max_width = LengthValue::MinContent;

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let p = find(&result.root, parent).expect("parent box");
    assert!(
        (p.width - 200.0).abs() < 1.0,
        "non-AR block with max-width:min-content keeps taffy Auto behavior (200), got {}",
        p.width
    );
}
