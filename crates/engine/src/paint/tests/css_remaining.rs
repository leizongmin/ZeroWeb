//! CSS contain / unicode-bidi / box-decoration-break / overflow-wrap / text-align-last /
//! break / scroll-area / scroll-snap-stop / container-type 渲染指示器单元测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::Document;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::*;

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<zero_dom::NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: 0.0,
        content_y: 0.0,
        content_width: width,
        content_height: height,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    }
}

/// 创建指定样式的 Painter 并渲染一个节点。
fn paint_with_style(style: &ComputedStyle) -> Painter {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    styles.insert(elem, style.clone());

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    painter
}

// ──────────────────────────────────────────────────────
// CSS contain 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_contain_none_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    // contain: None 是默认值，不应产生额外填充
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_contain_strict_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Strict;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_paint_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Paint;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_content_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Content;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_size_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_contain_layout_indicator() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Layout;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS unicode-bidi 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_unicode_bidi_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_unicode_bidi_embed_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Embed;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_isolate_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Isolate;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_bidi_override_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::BidiOverride;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_unicode_bidi_plaintext_indicator() {
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Plaintext;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS box-decoration-break 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_box_decoration_break_slice_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_box_decoration_break_clone_indicator() {
    let mut style = ComputedStyle::default();
    style.box_decoration_break = BoxDecorationBreakValue::Clone;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS overflow-wrap 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_overflow_wrap_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_overflow_wrap_break_word_indicator() {
    let mut style = ComputedStyle::default();
    style.overflow_wrap = OverflowWrapValue::BreakWord;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_overflow_wrap_anywhere_indicator() {
    let mut style = ComputedStyle::default();
    style.overflow_wrap = OverflowWrapValue::Anywhere;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS text-align-last 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_text_align_last_auto_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_text_align_last_center_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Center;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_justify_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Justify;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_right_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Right;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_text_align_last_left_indicator() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Left;
    let painter = paint_with_style(&style);
    // Left 映射到 1 条横线指示器
    assert!(painter.primitives().fills.len() >= 1);
}

// ──────────────────────────────────────────────────────
// CSS break 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_break_all_auto_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_break_before_column_indicator() {
    let mut style = ComputedStyle::default();
    style.break_before = BreakValue::Column;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_break_after_page_indicator() {
    let mut style = ComputedStyle::default();
    style.break_after = BreakValue::Page;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_break_inside_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.break_inside = BreakInsideValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_before_always_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_before = PageBreakValue::Always;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_after_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_after = PageBreakValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_page_break_inside_avoid_indicator() {
    let mut style = ComputedStyle::default();
    style.page_break_inside = PageBreakValue::Avoid;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS scroll-margin/padding 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_scroll_margin_zero_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_scroll_margin_top_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 10.0;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_margin_all_sides_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 5.0;
    style.scroll_margin_right = 5.0;
    style.scroll_margin_bottom = 5.0;
    style.scroll_margin_left = 5.0;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_padding_length_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_padding_top = ScrollPadding::Length(8.0);
    style.scroll_padding_bottom = ScrollPadding::Length(8.0);
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_scroll_padding_auto_no_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_padding_top = ScrollPadding::Auto;
    style.scroll_padding_bottom = ScrollPadding::Auto;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

// ──────────────────────────────────────────────────────
// CSS scroll-snap-stop 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_scroll_snap_stop_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_scroll_snap_stop_always_indicator() {
    let mut style = ComputedStyle::default();
    style.scroll_snap_stop = ScrollSnapStop::Always;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// CSS container-type 指示器
// ──────────────────────────────────────────────────────

#[test]
fn test_container_type_normal_no_indicator() {
    let style = ComputedStyle::default();
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() <= 1);
}

#[test]
fn test_container_type_size_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_container_type_inline_size_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::InlineSize;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

#[test]
fn test_container_type_with_name_extra_indicator() {
    let mut style = ComputedStyle::default();
    style.container_type = ContainerType::Size;
    style.container_name = Some("sidebar".to_string());
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 2);
}

// ──────────────────────────────────────────────────────
// 组合测试
// ──────────────────────────────────────────────────────

#[test]
fn test_contain_plus_unicode_bidi_combined() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Strict;
    style.unicode_bidi = UnicodeBidiValue::BidiOverride;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_break_before_plus_after_combined() {
    let mut style = ComputedStyle::default();
    style.break_before = BreakValue::Page;
    style.break_after = BreakValue::Column;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_scroll_margin_plus_padding_combined() {
    let mut style = ComputedStyle::default();
    style.scroll_margin_top = 5.0;
    style.scroll_margin_bottom = 5.0;
    style.scroll_padding_top = ScrollPadding::Length(8.0);
    style.scroll_padding_bottom = ScrollPadding::Length(8.0);
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 4);
}

#[test]
fn test_all_indicators_combined() {
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Layout;
    style.overflow_wrap = OverflowWrapValue::BreakWord;
    style.text_align_last = TextAlignLastValue::Center;
    style.unicode_bidi = UnicodeBidiValue::Embed;
    style.box_decoration_break = BoxDecorationBreakValue::Clone;
    style.break_before = BreakValue::Column;
    style.scroll_margin_top = 5.0;
    style.scroll_snap_stop = ScrollSnapStop::Always;
    style.container_type = ContainerType::Size;
    let painter = paint_with_style(&style);
    assert!(painter.primitives().fills.len() >= 10);
}
