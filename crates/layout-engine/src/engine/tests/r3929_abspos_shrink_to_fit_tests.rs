//! R3929（CSS2 §10.3.7/§10.6.4）：abspos 非 replaced 元素 shrink-to-fit 尺寸。
//!
//! taffy 对 width:auto + 全/半 auto 水平 inset 的 abspos 不做内容测量（layout dump
//! 实证 0 宽）——absolute-non-replaced-max-height-002（`&nbsp;` + Ahem 100px 应 100 宽）
//! /009（top:25 定 + bottom:auto）渲 0 宽方块。本 pass 在 intrinsic re-run 组内对
//! taffy 给 0 的 abspos 盒按内容 max-content（≤CB−已定 inset）回填宽度、按行高回填
//! 高度；taffy 已解出非 0 值（stretch、float 内容等）不覆写。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// absolute-non-replaced-max-height-002：全 auto 水平 inset 的 abspos（`&nbsp;` +
/// 100px 字号）宽应收缩适配到内容 100（taffy 旧给 0）。
#[test]
fn r3929_abspos_width_shrinks_to_content() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; width: auto; font: 100px/1 Ahem, sans-serif; max-height: 50px;">&nbsp;</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (w - 100.0).abs() < 1.0,
        "R3929: abspos 全 auto inset 宽应收缩到内容 100，实际 {w}"
    );
}

/// absolute-non-replaced-max-height-009：height:auto + top 定 + bottom:auto 的 abspos
/// 高应收缩到行高（taffy 旧给 0）。
#[test]
fn r3929_abspos_height_shrinks_to_line_height() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; top: 25px; bottom: auto; height: auto; font: 100px/1 Ahem, sans-serif;">&nbsp;</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (_w, h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (h - 100.0).abs() < 1.0,
        "R3929: abspos height:auto 垂直非双定高应收缩到行高 100，实际 {h}"
    );
}

/// taffy 已正确给出宽（水平双定 inset = stretch）时不覆写——margin-applies-to 族
/// 防回归锚（v1 版无此守卫致 23 案回归）。
#[test]
fn r3929_stretch_inset_not_overridden() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="position: absolute; left: 0; right: 0; width: auto; height: 50px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("initial box");
    assert!(
        (w - 200.0).abs() < 1.0,
        "R3929: 双定 inset stretch 宽应保持 200 不被覆写，实际 {w}"
    );
}

/// 内含 float 后代的 abspos 跳过（float 子 max-width 语义 max-content 近似失准，
/// absolute-non-replaced-width-019/020 防回归锚）。
#[test]
fn r3929_float_descendant_skipped() {
    let html = r#"<html><body style="margin:0">
<div style="position: absolute; width: auto; font: 30px/4 Ahem, sans-serif;"><span style="float: left; max-width: 4em;">12345678</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (w, _h) = find_box(&result.root, tid).expect("target box");
    assert!(
        (w - 960.0).abs() > 4.0,
        "R3929: float 后代盒不应被 max-content（≈960）覆写——gate 应跳过，实际 {w}"
    );
}

/// R3930（CSS2 §10.3.7 static position）：流父 direction:rtl（非 positioned）的 abspos
/// 全 auto inset 静态位镜像——absolute-non-replaced-width-021/022（body rtl + inline-block
/// max-width 子）。静态位置由 flow 父 direction 决定，非 positioned 的 rtl 流父同样镜像。
#[test]
fn r3930_rtl_flow_parent_mirrors_static_position() {
    use crate::engine::tests::find_absolute_position_by_node_id;
    let html = r#"<html><body style="margin:0; direction: rtl;">
<div style="position: absolute; width: auto;"><span style="display: inline-block; max-width: 120px; width: 120px; height: 120px;"></span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (x, _y) = find_absolute_position_by_node_id(&result.root, tid).expect("target box");
    // 静态位镜像：左缘贴流父 content 右缘（800−120=680），非 LTR 左贴 x=0。
    assert!(
        x > 400.0,
        "R3930: rtl 流父 abspos 静态位应镜像到右缘（x≈680），实际 x={x}"
    );
}

/// R3930 对照：流父 ltr 时静态位保持左贴（taffy 默认语义不破坏）。
#[test]
fn r3930_ltr_flow_parent_keeps_static_position() {
    use crate::engine::tests::find_absolute_position_by_node_id;
    let html = r#"<html><body style="margin:0; direction: ltr;">
<div style="position: absolute; width: auto;"><span style="display: inline-block; max-width: 120px; width: 120px; height: 120px;"></span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("target div");
    let (x, _y) = find_absolute_position_by_node_id(&result.root, tid).expect("target box");
    assert!(x.abs() < 1.0, "R3930: ltr 流父 abspos 静态位应保持左贴 x=0，实际 x={x}");
}

/// R3932（XML QName + CSS Selectors §6.3 类型选择器）：XHTML+SVG DTD 文档中
/// `<svg:svg>` 的 local_name 须为 "svg"（html5ever HTML 模式存整名 "svg:svg"，
/// CSS 类型选择器 `svg` 不命中 → absolute-replaced-width-038 的 svg 样式全丢）。
/// content_is_xml 文档解析完成后拆 `:` 前缀。
#[test]
fn r3932_xhtml_dtd_prefixed_element_name_split() {
    let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1 plus MathML 2.0 plus SVG 1.1//EN" "http://www.w3.org/2002/04/xhtml-math-svg/xhtml-math-svg.dtd">
<html xmlns="http://www.w3.org/1999/xhtml"><body>
<svg:svg version="1.1" xmlns:svg="http://www.w3.org/2000/svg" height="50"><svg:rect width="10" height="10"/></svg:svg>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    assert!(doc.content_is_xml(), "XHTML DTD 应置位 content_is_xml");
    let hits = doc.get_elements_by_tag_name("svg").len();
    assert_eq!(hits, 1, "R3932: 前缀拆分后 getElementsByTagName('svg') 应命中 1 个元素");
    let id = doc.get_elements_by_tag_name("svg").into_iter().next().unwrap();
    let local = doc.get(id).map(|n| match &n.kind {
        zero_dom::NodeKind::Element(e) => e.local_name().to_string(),
        _ => String::new(),
    });
    assert_eq!(local.as_deref(), Some("svg"), "R3932: local_name 应为 'svg'");
}

/// R3932 守卫对照：纯 HTML 文档（无 XHTML DTD）中 `<svg:svg>` 不拆——chromium
/// text/html 同样按未知元素处理，行为保持。
#[test]
fn r3932_html_doc_prefixed_element_untouched() {
    let html = r#"<html><body>
<svg:svg version="1.1" xmlns:svg="http://www.w3.org/2000/svg" height="50"><svg:rect width="10" height="10"/></svg:svg>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    assert!(!doc.content_is_xml());
    let hits = doc.get_elements_by_tag_name("svg").len();
    assert_eq!(hits, 0, "R3932: HTML 文档不拆前缀，'svg' 查询不命中");
}

/// R3935（inline svg paint 前置锚）：outer_html 对 svg 子树的属性保真——
/// transform / transform-origin 等 SVG presentation attribute 须完整保留
///（paint_svg_element 序列化 → usvg 栅格化的语义输入）。
#[test]
fn r3935_outer_html_preserves_svg_presentation_attrs() {
    let html = r#"<html><body>
<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
<rect x="75" y="75" width="150" height="150" fill="green" transform="rotate(90) translate(0 150)" transform-origin="center right"/>
</svg>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let s = doc.outer_html(sid);
    assert!(
        s.contains(r#"transform="rotate(90) translate(0 150)""#),
        "transform attr 须保留: {s}"
    );
    assert!(
        s.contains(r#"transform-origin="center right""#),
        "transform-origin attr 须保留: {s}"
    );
    assert!(s.contains(r#"fill="green""#), "fill attr 须保留: {s}");
}
