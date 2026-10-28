//! t-p116：块容器「流内文本 + 原子行内级盒（inline-block/button）」同行续排。
//!
//! CSS 2 §9.2.1.1（原子行内级盒参与父 IFC 行盒）+ CSS2 §10.8（行盒内基线对齐）：
//! 前置文本与原子盒之间无软换行机会（无空白）时同一行盒内续排，原子盒 x =
//! 文本推进之后、y 落首行带内（chromium 同）。修前原子盒被当作块级子堆叠到
//! 文本行之后（y = 文本行底、x = 容器原点；探针实测 div>hi+span[inline-block
//! 50×20] 修前 span y−div y = 24、x = 0）。
//! 相邻原子盒无文本场景（同行且 x 递增）与容器单行高度为既有行为守护。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "body { margin: 0 } div { margin: 0 } .ib { display: inline-block; width: 50px; height: 20px } button { border: 0; padding: 0; margin: 0 }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 文本 + inline-block（带内容，镜像探针 v5：width:max-content 收缩容器）：
/// 原子盒落首行带内（y−容器 y < 18 = 默认行盒高）且 x 在文本推进之后（0 < x < 30）。
/// 探针实测修前 y−容器 y = 24（块堆叠到文本行后）、x = 0。
#[test]
fn p116_text_then_inline_block_same_line() {
    let (doc, result) =
        layout("<html><body><div style=\"width:max-content\">hi<span class=ib>ab cd</span></div></body></html>");
    let span_id = doc.get_elements_by_tag_name("span").into_iter().next().expect("span");
    let (x, y, _w, _h) = find_box(&result.root, span_id).expect("span box");
    assert!(
        (0.0..30.0).contains(&x),
        "原子盒 x 应在文本推进之后（0<x<30），got {x}（0 = 块堆叠原点，t-p116 缺陷）"
    );
    assert!(
        y < 18.0,
        "原子盒 y 应落首行带内（<18），got {y}（≥文本行高 = 块堆叠到第二行，t-p116 缺陷）"
    );
}

/// 宽度守护：收缩容器宽/高按行盒口径（宽 = 文本+原子推进和 ≈70、高 = 单行盒
/// ≈20；p115 已修宽，高度不得因原子块堆叠回翻倍）。
#[test]
fn p116_mixed_line_container_box_single_line() {
    let (doc, result) =
        layout("<html><body><div style=\"width:max-content\">hi<span class=ib>ab cd</span></div></body></html>");
    let div_id = doc.get_elements_by_tag_name("div").into_iter().next().expect("div");
    let (_x, _y, w, h) = find_box(&result.root, div_id).expect("div box");
    assert!(
        (60.0..80.0).contains(&w),
        "容器宽应为文本+原子推进和（≈70，p115 修复口径），got {w}"
    );
    assert!(h < 30.0, "容器高应为单行盒（≈20，<30），got {h}（≥38 = 两行块堆叠）");
}

/// 镜像探针 minG（浏览器实证复现形）：monospace 20px/1 + **shrink-to-fit** 原子盒
/// （无声明宽）+ width:max-content 容器——修前原子盒折到第二行（y=20、x=0）。
#[test]
fn p116_shrink_to_fit_ib_same_line_monospace() {
    let doc = zero_dom::parse_html(
        "<html><body><div style=\"width:max-content\">hi<span class=ib>ab cd</span></div></body></html>",
    );
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "body { margin: 0; font: 20px/1 monospace } div { margin: 0 } .ib { display: inline-block; background: #0f0 }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let span_id = doc.get_elements_by_tag_name("span").into_iter().next().expect("span");
    let (x, y, _w, _h) = find_box(&result.root, span_id).expect("span box");
    assert!(
        (0.0..30.0).contains(&x),
        "原子盒 x 应在文本推进之后（0<x<30），got {x}（0 = 第二行块堆叠原点，t-p116/minG 缺陷）"
    );
    assert!(
        y < 18.0,
        "原子盒 y 应落首行带内（<18），got {y}（≥行高 = 折到第二行，t-p116/minG 缺陷）"
    );
}

/// 相邻原子盒（无前置文本）同行且 x 递增——既有行为守护（v6 形态，防修复回归）。
#[test]
fn p116_adjacent_inline_blocks_same_line() {
    let (doc, result) = layout("<html><body><div><span class=ib></span><span class=ib></span></div></body></html>");
    let span_ids = doc.get_elements_by_tag_name("span");
    assert_eq!(span_ids.len(), 2, "两 span");
    let (x0, y0, _, _) = find_box(&result.root, span_ids[0]).expect("span0 box");
    let (x1, y1, _, _) = find_box(&result.root, span_ids[1]).expect("span1 box");
    assert!((y0 - y1).abs() < 1.0, "两原子盒应同行，got {y0} vs {y1}");
    assert!(x1 > x0 + 49.0, "第二原子盒 x 应在首盒推进后，got {x0} vs {x1}");
}
