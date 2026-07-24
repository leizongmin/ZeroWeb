//! R2013 layout-width-for-print：Print 模式根布局 containing block 宽 = 页内容盒宽。
//!
//! 验证 `LayoutEngine::effective_root_layout_width`：
//! - Screen 模式 → `viewport_width`（旧行为，零变更）。
//! - Print 模式 → `print_content_width(page_width, margin_left, margin_right)`
//!   （default A4 宽 − 水平 margin）。

use crate::LayoutEngine;
use zero_css_parser::media_query::MediaType;

/// Screen 模式：`effective_root_layout_width` 返回 `viewport_width`（不受 Print 页几何影响）。
#[test]
fn r2013_screen_mode_returns_viewport_width() {
    let mut engine = LayoutEngine::new(1200.0, 800.0);
    // 即使注入了 Print 页几何，Screen 模式也不消费。
    engine.set_print_page_width(500.0);
    engine.set_print_horizontal_margins(100.0, 100.0);
    assert_eq!(engine.effective_root_layout_width(), 1200.0, "Screen → viewport_width");
}

/// Print 模式 default（无 @page 注入）：根宽 = A4 默认宽 ≈ 793.7（与页高默认 A4 一致）。
/// 旧行为是 viewport_width；本切片使 Print 文档按页宽 reflow。
#[test]
fn r2013_print_mode_defaults_to_a4_width() {
    let mut engine = LayoutEngine::new(1200.0, 800.0);
    engine.set_media_type(MediaType::Print);
    let a4_w = 210.0 / 25.4 * 96.0;
    let w = engine.effective_root_layout_width();
    assert!((w - a4_w).abs() < 0.01, "Print default → A4 width {a4_w}, got {w}");
}

/// Print 模式 + @page size + 水平 margin：根宽 = 页宽 − 左右 margin。
#[test]
fn r2013_print_mode_subtracts_horizontal_margins() {
    let mut engine = LayoutEngine::new(1200.0, 800.0);
    engine.set_media_type(MediaType::Print);
    engine.set_print_page_width(1000.0);
    engine.set_print_horizontal_margins(150.0, 250.0);
    // 1000 − 150 − 250 = 600
    assert_eq!(
        engine.effective_root_layout_width(),
        600.0,
        "Print → page content width"
    );
}

/// Print 模式退化：水平 margin 吃光页宽 → 回退页宽本身（边距归零），不返回负值/零塌缩。
#[test]
fn r2013_print_mode_degenerate_margins_fall_back_to_page_width() {
    let mut engine = LayoutEngine::new(1200.0, 800.0);
    engine.set_media_type(MediaType::Print);
    engine.set_print_page_width(100.0);
    engine.set_print_horizontal_margins(60.0, 60.0); // usable = -20 < 1
    assert_eq!(
        engine.effective_root_layout_width(),
        100.0,
        "degenerate → page width 100 (no collapse)"
    );
}

/// set_print_page_width ≤0 忽略（保默认 A4）；set_print_horizontal_margins 负值忽略（保 0）。
#[test]
fn r2013_setters_ignore_invalid_values() {
    let mut engine = LayoutEngine::new(1200.0, 800.0);
    engine.set_media_type(MediaType::Print);
    let a4_w = 210.0 / 25.4 * 96.0;
    engine.set_print_page_width(0.0); // ignored
    engine.set_print_page_width(-50.0); // ignored
    engine.set_print_horizontal_margins(-10.0, -20.0); // ignored → stay 0
    let w = engine.effective_root_layout_width();
    assert!((w - a4_w).abs() < 0.01, "invalid → A4 default {a4_w}, got {w}");
}

/// 切回 Screen 后即使保留 Print 页几何，根宽仍 = viewport_width（模式门控）。
#[test]
fn r2013_toggle_back_to_screen_uses_viewport_width() {
    let mut engine = LayoutEngine::new(980.0, 600.0);
    engine.set_media_type(MediaType::Print);
    engine.set_print_page_width(1000.0);
    engine.set_print_horizontal_margins(50.0, 50.0);
    assert_eq!(engine.effective_root_layout_width(), 900.0, "Print → 900");
    engine.set_media_type(MediaType::Screen);
    assert_eq!(
        engine.effective_root_layout_width(),
        980.0,
        "toggle Screen → viewport_width 980"
    );
}
