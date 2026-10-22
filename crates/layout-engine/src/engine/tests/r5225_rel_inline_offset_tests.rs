//! R5225（css-position-3 §relative-positioning）display:inline 非替换盒 relative inset
//! 单次施加锚点。converter 对 display:inline 保留 taffy inset（R109 split 族依赖），
//! 后处理臂再施加 = 双重计数（abspos-inline-008 span 位移 2×−100px 实证）；本臂只对
//! inline-block（R5054：converter 已清 inset）与替换元素（inline layout 定位、taffy
//! inset 不消费）施加。LayoutBox.x 为父相对坐标——锚点以祖先链累加断言绝对位。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_abs_positions(root: &LayoutBox, node_id: zero_dom::NodeId) -> Vec<(f32, f32)> {
    let mut hits = Vec::new();
    fn walk(b: &LayoutBox, abs_x: f32, abs_y: f32, node_id: zero_dom::NodeId, hits: &mut Vec<(f32, f32)>) {
        let ax = abs_x + b.x;
        let ay = abs_y + b.y;
        if b.node_id == Some(node_id) {
            hits.push((ax, ay));
        }
        for c in &b.children {
            walk(c, ax, ay, node_id, hits);
        }
    }
    walk(root, 0.0, 0.0, node_id, &mut hits);
    hits
}

fn assert_abs(box_positions: &[(f32, f32)], expect_x: f32, label: &str) {
    assert!(!box_positions.is_empty(), "{label} 盒应存在于布局树");
    for (i, (ax, _)) in box_positions.iter().enumerate() {
        assert!(
            (ax - expect_x).abs() <= 0.5,
            "{label} 第 {i} 个盒 abs_x 应为 {expect_x}（relative 偏移单次施加），实际 {ax}"
        );
    }
}

/// display:inline 相对定位 + 仅 OOF 块级子（abspos-inline-008 形态）：taffy 单次施加，
/// 臂不重复——span 与 abspos 子（静态位随 CB）abs_x 均 = 100−100 = 0。
#[test]
fn r5225_rel_inline_oof_child_single_offset() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:100px; height:20px;">
  <span id=s style="position:relative; left:-100px;">
    <div id=abs style="position:absolute; width:50px; height:50px;"></div>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let s = doc.get_element_by_id("s").expect("span#s");
    let abs = doc.get_element_by_id("abs").expect("div#abs");
    let spos = find_abs_positions(&result.root, s);
    let apos = find_abs_positions(&result.root, abs);
    assert_abs(&spos, 0.0, "span#s");
    assert_abs(&apos, 0.0, "div#abs（静态位随 CB）");
}

/// R109 split inline 相对定位守卫：taffy 单次施加 + 臂跳过的既有行为不变
/// （block-in-inline-relpos-001 谱系），文本 + 块级子形态 wrapper abs_x = 0。
#[test]
fn r5225_rel_inline_split_offset_preserved() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:200px; height:40px;">
  <span id=s style="position:relative; left:-100px;">text
    <div style="width:50px; height:10px;"></div>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let s = doc.get_element_by_id("s").expect("span#s");
    let spos = find_abs_positions(&result.root, s);
    assert_abs(&spos, 0.0, "split inline span wrapper");
}

/// block-level 相对定位守卫：taffy 单次施加路径不变，嵌套块 abs_x = 0。
#[test]
fn r5225_rel_block_nested_offset_preserved() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:200px; height:20px;">
  <div id=inner style="position:relative; left:-100px; width:50px; height:10px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let inner = doc.get_element_by_id("inner").expect("div#inner");
    let ipos = find_abs_positions(&result.root, inner);
    assert_abs(&ipos, 0.0, "block-in-block inner");
}
