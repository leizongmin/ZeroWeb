//! R5187 回归：SVG 文档 CSS 呈现属性注入加固（SVG2 presentation attributes）。
//!
//! 旧行为：R4280/R3938 序列化改写的元素定位 needle 要求 id/class **紧跟标签名**
//!（`<rect class="s"`）——序列化源 class 前有 x/y/width/height 等属性时 find 失配
//! → fill/geometry/transform 注入集体静默丢失（border-shape-shadow 判例 ref 侧
//! SVG 全黑、10.71% 差异根因）；`stroke` 色值无 style-system 管道（rect 落回
//! SVG 缺省 none）；`stroke-width` 计算值不注入（ref 的 spread 环/边框带缺失）。

use super::super::painter::{
    apply_css_svg_presentation_to_source, collect_css_svg_presentation, locate_element_open_tag,
    merge_style_decls_in_tag,
};
use zero_style_system::StyleSystem;

/// 定位：id/class 属性全文指纹在开标签内**任意位置**匹配（属性序无关）。
#[test]
fn test_r5187_locate_tag_attrs_before_class() {
    let source =
        r#"<svg><rect x="0" y="0" width="220" height="220" class="shadow"></rect><rect class="other"></rect></svg>"#;
    let (lt, tag_end) =
        locate_element_open_tag(source, "rect", None, Some("shadow"), true).expect("class 在非首位属性时也应命中");
    assert!(
        source[lt..=tag_end].starts_with(r#"<rect x="0""#),
        "命中 shadow rect 开标签"
    );
    assert!(source[lt..=tag_end].contains(r#"class="shadow""#));
    // id 优先。
    let src2 = r#"<rect class="a" id="b" x="1"/>"#;
    let (lt2, te2) = locate_element_open_tag(src2, "rect", Some("b"), Some("a"), true).expect("id 命中");
    assert!(src2[lt2..=te2].contains(r#"id="b""#));
}

/// 定位：无指纹回退首个同标签；有指纹无匹配 → None；`<rect` 不误配 `<rectX`。
#[test]
fn test_r5187_locate_tag_fallback_and_boundary() {
    let source = r#"<svg><circle/><rect x="1"/></svg>"#;
    let (lt, te) = locate_element_open_tag(source, "rect", None, None, true).expect("无指纹回退首个");
    assert!(
        source[lt..=te].starts_with("<rect"),
        "回退首个 rect，实际 {}",
        &source[lt..=te]
    );
    assert!(
        locate_element_open_tag(source, "rect", None, Some("nope"), true).is_none(),
        "指纹无匹配 → None"
    );
    let tricky = r#"<rectx class="s"></rectx>"#;
    assert!(
        locate_element_open_tag(tricky, "rect", None, Some("s"), true).is_none(),
        "标签名边界：rect 不得误配 rectx"
    );
    // fallback_first = false（transform 非根无标识语义）→ 不回退。
    assert!(locate_element_open_tag(source, "rect", None, None, false).is_none());
}

/// style attr 合并：无 style 新增；有 style 前插；同名替换；属性名边界（`fill`
/// 不误配 `transform: fill-box`；`stroke` 不误配 `stroke-width`）。
#[test]
fn test_r5187_merge_style_decls() {
    let merged = merge_style_decls_in_tag(r#"<rect x="1"/>"#, &["fill: red".to_string()]);
    assert_eq!(merged, r#"<rect x="1" style="fill: red"/>"#);
    let merged = merge_style_decls_in_tag(
        r#"<rect style="stroke: blue" x="1">"#,
        &["fill: red".to_string(), "stroke: green".to_string()],
    );
    assert!(
        merged.contains(r#"style="fill: red; stroke: green""#),
        "前插且同名替换（旧 stroke: blue 被换出），实际 {merged}"
    );
    let merged = merge_style_decls_in_tag(r#"<rect style="stroke-width: 2px"/>"#, &["stroke: red".to_string()]);
    assert!(
        merged.contains("stroke: red") && merged.contains("stroke-width: 2px"),
        "stroke 不得替换 stroke-width，实际 {merged}"
    );
    let merged = merge_style_decls_in_tag(r#"<rect style="transform: fill-box"/>"#, &["fill: red".to_string()]);
    assert!(
        merged.contains("transform: fill-box") && merged.contains("fill: red"),
        "fill 不得误配 transform: fill-box，实际 {merged}"
    );
}

/// 端到端合成：border-shape-shadow ref 页复刻——文档 CSS（class 选择器）fill/
/// stroke/stroke-width/transform 全部注入序列化源，几何属性注入 svg 根。
#[test]
fn test_r5187_svg_css_presentation_synthesis() {
    let css = "body { margin: 0 } \
               svg { position: absolute; top: 0; left: 0; width: 230px; height: 240px; overflow: visible } \
               .shadow { fill: purple; stroke: purple; stroke-width: 20px; transform: translate(10px, 20px) } \
               .border { fill: green; stroke: black; stroke-width: 10px }";
    let html = r#"<!DOCTYPE html><html><head><style></style></head><body><svg viewBox="0 0 230 240" xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0" width="220" height="220" class="shadow"/><rect x="5" y="5" width="210" height="210" class="border"/></svg></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);

    // 子树根 = svg 元素。
    let mut svg_id = None;
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        if let Some(node) = doc.get(id) {
            if let zero_dom::NodeKind::Element(el) = &node.kind
                && el.local_name() == "svg"
            {
                svg_id = Some(id);
                break;
            }
            for c in doc.child_nodes(id) {
                stack.push(c);
            }
        }
    }
    let Some(svg_id) = svg_id else {
        panic!("doc 应含 svg 元素");
    };

    let mut presentations = Vec::new();
    let mut geometry = Vec::new();
    collect_css_svg_presentation(&doc, svg_id, &styles, &mut presentations, &mut geometry);
    // ref 页 `svg { overflow: visible }`（author）须覆盖 UA hidden（R4174 注入）——
    // 越界 stroke 外环走扩展画布路径的入口。
    let svg_style = styles.get(&svg_id).expect("svg 样式存在");
    assert!(
        matches!(svg_style.overflow_x, zero_css_parser::values::OverflowValue::Visible),
        "author overflow: visible 应覆盖 UA hidden（R4174 注入），实际 {:?}",
        svg_style.overflow_x
    );
    let mut transforms = Vec::new();
    super::super::painter::collect_css_transforms(&doc, svg_id, &styles, &mut transforms);
    let mut source = doc.outer_html(svg_id);
    source = apply_css_svg_presentation_to_source(&source, &presentations, &geometry, &doc);
    source = super::super::painter::apply_css_transforms_to_source(&source, &transforms, &doc, svg_id);

    // shadow rect：fill/stroke/stroke-width/transform 全注入。
    let shadow_start = source.find(r#"class="shadow""#).expect("shadow rect 在源中");
    let tag_start = source[..shadow_start].rfind("<rect").expect("开标签");
    let tag_end = source[shadow_start..].find('>').expect("标签闭合") + shadow_start;
    let shadow_tag = &source[tag_start..=tag_end];
    assert!(
        shadow_tag.contains("fill: rgb(128, 0, 128)"),
        "purple fill 注入，实际 {shadow_tag}"
    );
    assert!(
        shadow_tag.contains("stroke: rgb(128, 0, 128)"),
        "purple stroke 注入，实际 {shadow_tag}"
    );
    assert!(
        shadow_tag.contains("stroke-width: 20px"),
        "stroke-width 20px 注入，实际 {shadow_tag}"
    );
    assert!(
        shadow_tag.contains(r#"transform=""#) && shadow_tag.contains("translate"),
        "CSS transform 注入"
    );

    // border rect：green fill + black stroke + 10px。
    let border_start = source.find(r#"class="border""#).expect("border rect 在源中");
    let tag_start = source[..border_start].rfind("<rect").expect("开标签");
    let tag_end = source[border_start..].find('>').expect("标签闭合") + border_start;
    let border_tag = &source[tag_start..=tag_end];
    assert!(
        border_tag.contains("fill: rgb(0, 128, 0)"),
        "green fill 注入，实际 {border_tag}"
    );
    assert!(border_tag.contains("stroke: rgb(0, 0, 0)"), "black stroke 注入");
    assert!(border_tag.contains("stroke-width: 10px"), "stroke-width 10px 注入");
    assert!(
        !border_tag.contains(r#"transform=""#),
        "border rect 无 CSS transform 勿注入"
    );

    // svg 根：CSS width/height（无 authored attr）→ 几何属性注入。
    let svg_end = source.find('>').expect("svg 开标签闭合");
    let svg_tag = &source[..=svg_end];
    assert!(svg_tag.contains(r#"width="230""#), "svg 根 width 注入，实际 {svg_tag}");
    assert!(svg_tag.contains(r#"height="240""#), "svg 根 height 注入");
}
