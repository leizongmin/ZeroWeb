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

/// R4016（CSS2 §10.3.8 + css-sizing-3 default object size）：abspos 无尺寸 svg 的
/// taffy definite 尺寸（(b) 臂 300×150）不得被 inline-only remeasure 清 0——
/// absolute-replaced-width-004 h 面（R4015b 遗留）。svg 子树（rect 等 SVG 元素）
/// 不产行盒几何 → remeasure total_height=0 旧把盒从 150 减回 0，蓝色方块整体消失。
#[test]
fn r4016_abspos_svg_default_size_survives_inline_remeasure() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<div style="height: 110px; width: 288px;">
<svg version="1.1" xmlns="http://www.w3.org/2000/svg" style="position: absolute;">
<rect x="0" y="0" width="200" height="100" fill="blue" />
</svg>
</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (w, h) = find_box(&result.root, sid).expect("svg box");
    assert!(
        (w - 300.0).abs() < 1.0 && (h - 150.0).abs() < 1.0,
        "R4016: abspos 无尺寸 svg 应保持 default object size 300×150，实际 {w}×{h}"
    );
}

/// R4016 对照：attr 固有高（height=50）的 abspos svg 盒高保持 50——009/023/030
/// （2.08% 簇）h 面（taffy attr 臂已设高，同被旧 remeasure 清 0）。
#[test]
fn r4016_abspos_svg_attr_height_survives_inline_remeasure() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 200px;">
<svg version="1.1" xmlns="http://www.w3.org/2000/svg" height="50" style="position: absolute;">
<rect x="0" y="0" width="200" height="100" fill="blue" />
</svg>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (w, h) = find_box(&result.root, sid).expect("svg box");
    assert!(
        (h - 50.0).abs() < 1.0,
        "R4016: abspos attr height=50 的 svg 盒高应保持 50，实际 h={h} w={w}"
    );
}

/// R4017（CSS2 §10.3.7 static position）：block-level abspos top/bottom 均 auto 的
/// 垂直静态位 = 前 in-flow 兄弟 margin-box 底。taffy static_position 对「前驱兄弟
/// 高度后续增长」（两行 p）用单行过期值——absolute-replaced-width-037 族 y 偏上 18px。
#[test]
fn r4017_abspos_static_position_uses_final_sibling_height() {
    use crate::engine::tests::find_absolute_position_by_node_id;
    let html = r#"<html><body style="margin:0">
<p style="margin:16px 0;">Test passes if the blue and orange rectangles have the same width and are horizontally centered in an hollow black square.</p>
<div style="position: absolute; width: 100px; height: 50px;"></div>
<div style="width: 100px; height: 100px;"></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (_x, y) = find_absolute_position_by_node_id(&result.root, tid).expect("abspos box");
    // p 两行 ≈ 37.2 + p.mb 16（mt 0）→ 静态位 ≈ 53.2（taffy 旧值 ≈ 35 单行）。
    assert!(
        y > 45.0,
        "R4017: abspos 静态位应随前驱兄弟最终高度（两行 p ≈ 53），实际 y={y}"
    );
}

/// R4017 gate 对照：abspos 自带非零 margin-top 时不介入（taffy absolute 布局对 static
/// 另加 margin.top，非零 mt 折叠语义归既有链）——multicol-spanner-007 防回归锚。
#[test]
fn r4017_abspos_with_margin_top_not_touched() {
    use crate::engine::tests::find_absolute_position_by_node_id;
    let html = r#"<html><body style="margin:0">
<p style="margin:16px 0;">prefix line one</p>
<div style="position: absolute; margin-top: 60px; width: 100px; height: 20px;"></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (_x, y) = find_absolute_position_by_node_id(&result.root, tid).expect("abspos box");
    // taffy 既有值（static + mt 链）保持不动——锚定 R4017 公式值未应用：
    // 公式介入会给出 p margin-box 底 + max(p.mb, mt) = 34.6 + 60 = 94.6；
    // taffy 既有链（static + margin.top 叠加语义）实测 ≈111（taffy 内部值，不锚定具体数）。
    assert!(
        (y - 94.6).abs() > 1.0,
        "R4017: mt 非零的 abspos 应保持 taffy 既有静态位（gate 跳过，公式值 94.6 不应出现），实际 y={y}"
    );
}

/// R4018（CSS2 §10.6.6 + SVG2 sizing）：abspos svg 的 % attr 固有高——
/// `height="50%"` 是存在的百分比声明，used 高 = % × CB padding-box
///（absolute-replaced-height-027/034：50% × 192 = 96，旧落 default 150）。
#[test]
fn r4018_abspos_svg_pct_attr_height_resolves_against_cb() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 192px; border-top: 3px solid black;">
<svg version="1.1" xmlns="http://www.w3.org/2000/svg" height="50%" style="position: absolute; top: 0; left: 0;">
<rect x="0" y="0" width="200" height="100" fill="blue" />
</svg>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (_w, h) = find_box(&result.root, sid).expect("svg box");
    // 50% × CB padding-box（192 = 195 border-box − 3 border-top）= 96。
    assert!(
        (h - 96.0).abs() < 1.0,
        "R4018: abspos svg height=50% 应按 CB padding-box 解析为 96，实际 {h}"
    );
}

/// R4018（CSS2 §10.3 + §10.6.4）：abspos 元素（含 UA display:inline 的 replaced 类）
/// 垂直 margin **参与定位方程**——converter 的 R1058 inline 垂直 margin 清零须排除
/// abspos/fixed（blockify 语义）。
#[test]
fn r4018_abspos_inline_element_margin_top_participates() {
    use crate::engine::tests::find_absolute_position_by_node_id;
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 200px; height: 192px;">
<svg version="1.1" xmlns="http://www.w3.org/2000/svg" style="position: absolute; top: 48px; left: 0; margin-top: 48px;">
<rect x="0" y="0" width="200" height="100" fill="blue" />
</svg>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (_x, y) = find_absolute_position_by_node_id(&result.root, sid).expect("svg box");
    // §10.6.4：盒顶 = top(48) + margin-top(48) = 96（CB=div1 padding-box，body margin 0）。
    assert!(
        (y - 96.0).abs() < 1.0,
        "R4018: abspos 元素垂直 margin 应参与定位（top+mt=96），实际 y={y}"
    );
}

/// R4054（CSS2 §10.8.1 + SVG2 §7.2）：替换 inline 元素（svg，UA display:inline）的
/// 垂直 padding 参与 taffy 布局——converter 的 R1442 inline 垂直 padding 清零 gate
/// 语义是「**非替换** inline 元素不影响 line box 高度」，替换元素的 margin-box 参与
/// 高度合成（contain-size-replaced-002：`padding: 50px 0` + contain:size + width=100
/// → 应 100 高，旧清 0 后 svg 消失）。锚定方式：带 padding 变体比无 padding 变体
/// 高 100（content 部分两变体同源）。
#[test]
fn r4054_inline_svg_vertical_padding_participates() {
    let (doc, result) = layout(
        r#"<html><body style="margin:0">
<svg width="100" style="padding: 50px 0;"><rect width="50" height="50" fill="red"/></svg>
</body></html>"#,
    );
    let sid = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (_w, h_pad) = find_box(&result.root, sid).expect("svg box");

    let (doc2, result2) = layout(
        r#"<html><body style="margin:0">
<svg width="100"><rect width="50" height="50" fill="red"/></svg>
</body></html>"#,
    );
    let sid2 = doc2.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
    let (_w2, h_bare) = find_box(&result2.root, sid2).expect("svg box");

    assert!(
        (h_pad - h_bare - 100.0).abs() < 1.0,
        "R4054: 替换 inline svg 垂直 padding 应全额参与（Δ=100），实际 pad={h_pad} bare={h_bare}"
    );
}

/// R4054 对照：非替换 inline（span）的垂直 padding 不参与（R1442 gate 主语义不回退——
/// `<span padding:16px>` 不得撑高段落 32px）。
#[test]
fn r4054_inline_span_vertical_padding_still_zero() {
    let html = r#"<html><body style="margin:0">
<p style="margin:0">a<span style="padding: 16px 0;">b</span>c</p>
</body></html>"#;
    let (doc, result) = layout(html);
    let pid = doc.get_elements_by_tag_name("p").into_iter().next().expect("p");
    let (_w, h) = find_box(&result.root, pid).expect("p box");
    // 单行 line-height ≈ 18.6，垂直 padding 不计入。
    assert!(h < 30.0, "R4054: 非替换 inline 垂直 padding 不应撑高（<30），实际 {h}");
}
