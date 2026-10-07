//! R4986：vertical 写入模式 plain block + definite 块轴（物理宽）+ Auto 内联轴（物理高）
//! 加 `aspect-ratio`——taffy 只做 ratio 传递（w:100 加 AR 2/1 → h=50），不与内容内联尺寸
//! 取大。本 pass（`transfer_aspect_ratio_height` vertical 臂）补 max(transferred, 子
//! inline extent)：child h:100 应胜出 → 100×100。
//! driving: WPT css/css-sizing/aspect-ratio/block-aspect-ratio-017（vertical-lr 100 方块，
//! 曾渲 100×50，fail-list 1414 案）。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_style_system::WritingModeValue;

fn layout_vertical_ar_box() -> crate::LayoutBox {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.writing_mode = WritingModeValue::VerticalLr;
    parent_style.width = LengthValue::Px(100.0);
    parent_style.aspect_ratio = Some(2.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    let mut child_style = ComputedStyle::default();
    child_style.writing_mode = WritingModeValue::VerticalLr;
    child_style.height = LengthValue::Px(100.0);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    find(&result.root, parent).expect("parent box").clone()
}

#[test]
/// vertical-lr AR 2/1 + w:100 盒：子 inline extent 100 胜过 ratio 传递的 50 → 100×100。
fn r4986_vertical_ar_content_inline_extent_wins_over_transferred() {
    let p = layout_vertical_ar_box();
    assert_eq!(
        p.height, 100.0,
        "content-based inline size (child 100px) must win over ratio-transferred 50px in vertical-lr"
    );
    assert_eq!(p.width, 100.0, "definite block-axis width unchanged");
}

#[test]
/// 无子内容时保持 taffy ratio 传递值 50（不虚长）。
fn r4986_vertical_ar_childless_box_keeps_transferred() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.writing_mode = WritingModeValue::VerticalLr;
    parent_style.width = LengthValue::Px(100.0);
    parent_style.aspect_ratio = Some(2.0);
    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let p = find(&result.root, parent).expect("parent box").clone();
    assert_eq!(p.height, 50.0, "childless vertical AR box keeps transferred 100/2=50");
}
