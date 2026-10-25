//! R5275：preserve 纯空白 float 收缩回归测试（active-selection-063 宽片）。
//!
//! `white-space:pre` 容器的纯空白文本是保留内容（CSS Text 3 §3）——measure_text_content
//! 的空叶早退（has_inline_content 按 is_collapsible_ws 判空，R4034 口径不看 white-space
//! 上下文）曾把纯 tab div 判「无 inline 内容」回报 0 宽 → taffy 给 float 0 宽，且
//! shrink_pure_text_floats 只缩不扩（扩臂要求非文本 Σ>0）→ 盒 0×0 整缺。修复：preserve
//! 容器含直接文本（has_preserved_ws_text）时放行走 IFC 测量，tab 片段 advance 参与宽。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::FloatValue;
use zero_style_system::StyleSystem;

/// 深度优先找到第一个 `float:left` 的 LayoutBox。
fn find_float_left(root: &LayoutBox) -> Option<&LayoutBox> {
    if matches!(root.float, FloatValue::Left) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_float_left(child) {
            return Some(f);
        }
    }
    None
}

/// pre 纯 tab float 收缩到 tab stop advance（tab-size:4 × 25px 单字 advance = 100），
/// 高 = 单保留空白行行高 25（R5274 preserve 行计高前置片）。
#[test]
fn r5275_pre_tab_only_float_shrinks_to_tab_advance() {
    let html = r#"<html><body style="margin:0"><div style="float:left;font:25px/25px Ahem;white-space:pre;tab-size:4">&Tab;</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_div = find_float_left(&result.root).expect("should find pre tab float");
    assert!(
        (float_div.width - 100.0).abs() < 0.01,
        "pre 纯 tab float 应收缩到 tab advance 100，实测 {}",
        float_div.width
    );
    assert!(
        (float_div.height - 25.0).abs() < 0.01,
        "保留空白行计高 25，实测 {}",
        float_div.height
    );
}
