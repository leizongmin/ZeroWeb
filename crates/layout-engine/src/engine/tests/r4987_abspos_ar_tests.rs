//! R4987：abspos + `aspect-ratio` 三臂传递（css-sizing-4 §5.2 + §4.1 automatic
//! minimum·OOF 语义域）。taffy 0.12 对 OOF 盒 AR 语义不完整：双 auto 塌缩（007）、
//! definite 高 + auto 宽只传比值不取内容地板（013/017）、definite 宽 + auto 高的地板
//! 仅 min-height:auto 时生效（012 取内容 100，018 min-height:0 纯传递 100）。
//! driving: WPT css/css-sizing/aspect-ratio/abspos-007/008/012/013/017/018。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_style_system::PositionValue;

/// 建树：relative 父 > abspos 目标（width/height/min 按参）> 内容子（w×h 定值块）。
fn layout_abspos_ar(min_width: Option<f64>, min_height: Option<f64>) -> crate::LayoutBox {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs = doc.create_element("div");
    doc.append_child(parent, abs).unwrap();
    let kid = doc.create_element("div");
    doc.append_child(abs, kid).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.position = PositionValue::Relative;

    let mut abs_style = ComputedStyle::default();
    abs_style.display = zero_style_system::DisplayValue::Block;
    abs_style.position = PositionValue::Absolute;
    abs_style.aspect_ratio = Some(1.0);
    abs_style.width = LengthValue::Px(100.0);
    if let Some(v) = min_width {
        abs_style.min_width = LengthValue::Px(v);
    }
    if let Some(v) = min_height {
        abs_style.min_height = LengthValue::Px(v);
    }

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(abs, abs_style);
    let mut kid_style = ComputedStyle::default();
    kid_style.display = zero_style_system::DisplayValue::Block;
    kid_style.width = LengthValue::Px(200.0);
    kid_style.height = LengthValue::Px(200.0);
    styles.insert(kid, kid_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    find(&result.root, abs).expect("abspos box").clone()
}

#[test]
/// 臂② + min-height:auto：transferred 50 与内容地板取大——子 200 底边胜出（012 语义）。
fn r4987_abspos_ar_auto_min_height_floors_by_content() {
    // min_height 缺省 = Auto
    let b = layout_abspos_ar(None, None);
    assert_eq!(
        b.height, 200.0,
        "min-height:auto floors transferred height by content bottom (abspos-012 semantics)"
    );
}

#[test]
/// 臂② + min-height:0：无 automatic minimum 地板——纯传递 width/ratio = 100（018 语义）。
fn r4987_abspos_ar_min_height_zero_pure_transfer() {
    let b = layout_abspos_ar(None, Some(0.0));
    assert_eq!(
        b.height, 100.0,
        "min-height:0 disables content floor; height = width/ratio = 100 (abspos-018 semantics)"
    );
}

#[test]
/// 臂①：height definite 100 + AR 1/1 + min-width:0 → 宽纯传递 100，内容 200 不撑（017）。
fn r4987_abspos_ar_min_width_zero_pure_transfer() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs = doc.create_element("div");
    doc.append_child(parent, abs).unwrap();
    let kid = doc.create_element("div");
    doc.append_child(abs, kid).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    let mut abs_style = ComputedStyle::default();
    abs_style.display = zero_style_system::DisplayValue::Block;
    abs_style.position = PositionValue::Absolute;
    abs_style.aspect_ratio = Some(1.0);
    abs_style.height = LengthValue::Px(100.0);
    abs_style.min_width = LengthValue::Px(0.0);
    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(abs, abs_style);
    let mut kid_style = ComputedStyle::default();
    kid_style.display = zero_style_system::DisplayValue::Block;
    kid_style.width = LengthValue::Px(200.0);
    styles.insert(kid, kid_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let b = find(&result.root, abs).expect("abspos box").clone();
    assert_eq!(b.width, 100.0, "min-width:0 → pure transferred width (abspos-017)");
}
