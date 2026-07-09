//! R1242：纯文本 float shrink-to-fit 回归测试。
//!
//! `float:left; width:auto` 的纯文本元素应 shrink-to-fit 到文本 max-content 宽度
//! （CSS §10.3.5），而非保持 taffy 全宽。`font-size:0` 文本贡献 0 宽度 → float 应
//! 收缩到 0（font-size-zero-3 根因）。旧实现纯文本 float 保持 taffy 全宽（无 block/
//! replaced 子元素的 float 由 adjust_float_positions 跳过），R1242 新增 `shrink_pure_text_floats`
//! 预补 pass 用 `text_content_max_width` 测量并收缩。

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

/// `font-size:0` 纯文本 float 应收缩到 0 宽（text intrinsic contribution = 0）。
///
/// 复现 font-size-zero-3：float div 含长文本但 font-size:0，应 0 宽（不外露红背景）。
#[test]
fn test_pure_text_float_font_size_zero_shrinks_to_zero() {
    let html = r#"<html><body style="margin:0"><div style="float:left;font-size:0">Lorem ipsum dolor sit amet</div><div style="width:200px;height:200px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_div = find_float_left(&result.root).expect("should find a float:left div");
    // font-size:0 文本 max-content = 0 → float shrink-to-fit 到 0（border-box，无 padding/border）。
    assert!(
        float_div.width < 1.0,
        "font-size:0 pure-text float should shrink to 0 width, got {}",
        float_div.width
    );
}

/// Ahem 纯文本 float 应收缩到文本宽（4 字符 × 16px = 64px），而非填满容器（800px）。
#[test]
fn test_pure_text_float_ahem_shrinks_to_text_width() {
    let html = r#"<html><body style="margin:0"><div style="float:left;font:16px Ahem">XXXX</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_div = find_float_left(&result.root).expect("should find a float:left div");
    // Ahem 等宽：4 字符 × 16px = 64px（无 padding/border）。应收缩到 ~64，非 800 全宽。
    assert!(
        float_div.width < 100.0,
        "Ahem pure-text float should shrink to text width (~64px), got {}",
        float_div.width
    );
    assert!(
        float_div.width > 50.0,
        "Ahem pure-text float should be ~64px (4 chars x 16px), got {}",
        float_div.width
    );
}
