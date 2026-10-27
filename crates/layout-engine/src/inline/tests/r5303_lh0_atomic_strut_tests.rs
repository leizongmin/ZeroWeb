//! R5303（CSS2 §10.8.1 strut）：line-height:0 容器的仅原子行内盒行 strut 围合。
//!
//! 驱动案 css/CSS2/linebox/line-height-127：`table { font: 10em/1 Ahem }` 内
//! `.test { line-height: 0 }` 独嵌 1px img。strut 盒 0 高、悬于字体中线：
//! A = −fs/2 + 0.8fs = 0.3fs（160px 字体 → 48px），与 img 区间（基线上 [0,1]）
//! 取并集 → 行盒高 48。修复前 R5298 接管被 `container_strut_line_height > 0`
//! 挡住、legacy 兜底（0.8fs = 128）只抬基线不生长行盒（h 恒 1），红 td 整片
//! 露出（reftest ✗1.53%）。

use crate::types::InlineLayoutLine;

/// 收集（首个）存树 IFC 与宿主盒高。
fn find_ifc(node: &crate::LayoutBox) -> Option<(&Vec<InlineLayoutLine>, f32)> {
    if let Some(lines) = &node.inline_layout {
        return Some((lines, node.content_height));
    }
    node.children.iter().find_map(find_ifc)
}

fn layout(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 主判例（127 语义：fs 160 / line-height:0 / 1px 原子盒）：strut A = 0.3fs =
/// 48 围合行盒——行高 48、基线 48、原子盒顶 = 48 − 1 = 47。修复前 h=1、
/// 基线 128（0.8fs legacy，且不生长行盒）。
#[test]
fn r5303_lh0_container_atomic_only_line_strut_bounds_line_box() {
    let css = "div { font: 160px/0 Ahem; width: 400px; } \
               span { display: inline-block; width: 160px; height: 1px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div><span></span></div></body></html>"#);
    let result = layout(&html, css);
    let (lines, div_h) = find_ifc(&result.root).expect("div 应有存树 IFC");

    assert_eq!(lines.len(), 1, "应单行，实得 {lines:?}");
    let l = &lines[0];
    assert!(
        (l.height - 48.0).abs() < 0.01,
        "行盒高应 48（strut A = −80 + 0.8×160），实得 {}",
        l.height
    );
    assert!(
        (l.baseline_y - 48.0).abs() < 0.01,
        "基线应 48（D = −48 负 descent 不拖底边，并集底 = img descent 0），实得 {}",
        l.baseline_y
    );
    assert!((div_h - 48.0).abs() < 0.01, "div 高应 48，实得 {div_h}");
}

/// 对照锚（legacy 校准域零扰动）：line-height 非 0 容器的仅原子行不接管——
/// 行盒高仍由原子盒 run 高决定（inline-block 10px → 行高 10），R5298 文本行
/// 语义与 R5303 新臂均不触发。
#[test]
fn r5303_normal_lh_container_atomic_only_line_unchanged() {
    let css = "div { font: 16px/18px Ahem; width: 400px; } \
               span { display: inline-block; width: 10px; height: 10px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div><span></span></div></body></html>"#);
    let result = layout(&html, css);
    let (lines, div_h) = find_ifc(&result.root).expect("div 应有存树 IFC");

    assert_eq!(lines.len(), 1, "应单行，实得 {lines:?}");
    assert!(
        (lines[0].height - 10.0).abs() < 0.01,
        "行盒高应 10（原子盒 run 高，legacy 不接管），实得 {}",
        lines[0].height
    );
    assert!((div_h - 10.0).abs() < 0.01, "div 高应 10，实得 {div_h}");
}
