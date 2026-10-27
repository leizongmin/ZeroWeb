//! R5298（CSS2 §10.8.1 leading/strut）：混合 line-height 行盒 strut 参与。
//!
//! 驱动案 c42-ibx-ht-000：div `font: 10px/12px Ahem` 折 5 行，em `font: 1em/1`
//! （lh 10px）词独占 L2。修复前 strut 被近似为「本行 run 的 line-height max」，
//! 纯小行高 run 行 strut 蒸发：行盒缩至 10、基线 8，行距 11 vs chromium 12
//!（div 高 58 vs 60）。修复后 strut 由容器自身 font/line-height 决定
//!（`layout()` 捕获，run 侧同源解析器），参与每一行盒。

use crate::types::InlineLayoutLine;

/// c42-ibx-ht-000 页面原样（宽 200 = 20em×10px）。
fn build_c42() -> (
    zero_dom::Document,
    std::collections::HashMap<zero_dom::NodeId, zero_style_system::property::types::ComputedStyle>,
) {
    let css = "div { width: 20em; font: 10px/12px Ahem; color: green; } \
               span { border: solid 10px lime; padding: 2px 0 2px 10px; margin: 10px 10px 0 0; } \
               em { font: 1em/1 Ahem; color: red; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<p>There should be no red below, only green lines and dots.</p>
<div>
 z z <em> x x x x x x x x x x </em> <span> <em> x x x x x x x x x x
 x x x </em> y y y y y y y y y </span> y y y y y y y y y y z z z z
</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    (doc, styles)
}

/// R5298 主判例：纯小行高 run 行 strut 不蒸发——每行 h=12、基线 9（行距 12），
/// div 高 60。L2（y=12）全部片段属 em（lh 10），修复前行盒 10/基线 8。
#[test]
fn r5298_strut_holds_line_box_on_small_lh_only_lines() {
    let (doc, styles) = build_c42();
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    /// 收集（首个）存树 IFC 与宿主盒高。
    fn find_ifc(node: &crate::LayoutBox) -> Option<(&Vec<InlineLayoutLine>, f32)> {
        if let Some(lines) = &node.inline_layout {
            return Some((lines, node.content_height));
        }
        node.children.iter().find_map(find_ifc)
    }
    let (lines, div_h) = find_ifc(&result.root).expect("div 应有存树 IFC");

    assert_eq!(lines.len(), 5, "应折 5 行，实得 {lines:?}");
    for (i, l) in lines.iter().enumerate() {
        assert!(
            (l.height - 12.0).abs() < 0.01,
            "L{i} 行盒高应 12（strut 参与），实得 {}",
            l.height
        );
        assert!(
            (l.baseline_y - 9.0).abs() < 0.01,
            "L{i} 基线应 9（half-leading 1 + ascent 8），实得 {}",
            l.baseline_y
        );
        assert!(
            (l.y - 12.0 * i as f32).abs() < 0.01,
            "L{i} 行顶应 {}，实得 {}",
            12 * i,
            l.y
        );
    }
    assert!((div_h - 60.0).abs() < 0.01, "div 高应 60（5×12），实得 {div_h}");
}

/// 对照：容器 strut 行高走 run 侧同源解析器（normal 整数化同口径）——普通
/// normal 行高段落（16px→18）行盒高度逐字节不变（corpus 零扰动锚）。
#[test]
fn r5298_normal_line_height_paragraph_unchanged() {
    let css = "p { font: 16px Ahem; width: 300px; margin: 0 }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body><p>hello world enough text to wrap two lines here</p></body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find_ifc(node: &crate::LayoutBox) -> Option<(&Vec<InlineLayoutLine>, f32)> {
        if let Some(lines) = &node.inline_layout {
            return Some((lines, node.content_height));
        }
        node.children.iter().find_map(find_ifc)
    }
    // Ahem 16px：词宽 5×16=80、空格 16，300px 容器折 3 行
    //（hello world enough=288 / text to wrap two=272 / lines here=160）。
    let (lines, p_h) = find_ifc(&result.root).expect("p 应有存树 IFC");
    assert_eq!(lines.len(), 3, "应折 3 行，实得 {lines:?}");
    for l in lines {
        assert!(
            (l.height - 16.0).abs() < 0.01,
            "Ahem normal 行盒高应 16（ratio 1.0），实得 {}",
            l.height
        );
    }
    assert!((p_h - 48.0).abs() < 0.01, "p 高应 48（3×16），实得 {p_h}");
}
