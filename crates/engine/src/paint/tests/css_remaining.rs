//! CSS contain / unicode-bidi / box-decoration-break / overflow-wrap / text-align-last /
//! break / scroll-area / scroll-snap-stop / container-type 渲染指示器单元测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
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

/// R3933（CSS2 replaced elements + SVG2）：inline `<svg>` paint 栅格化。
/// kill-switch `ZW_INLINE_SVG_PAINT=1` 激活后：svg 元素产 ImagePrimitive +
/// canvas_images 像素（canvas 同款两段式通路）；默认关（svg transform/viewport
/// 语义切片前 102 案双白假绿 unmask 域）。
#[test]
fn r3933_inline_svg_paint_rasterizes_under_kill_switch() {
    let html = r#"<html><body style="margin:0"><div style="position: relative; width: 200px; height: 100px;"><svg width="100" height="50" xmlns="http://www.w3.org/2000/svg" style="position: absolute; left: 0; top: 0; width: 100px; height: 50px;"><rect width="100" height="50" fill="blue"/></svg></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let svg_id = doc
        .get_elements_by_tag_name("svg")
        .into_iter()
        .next()
        .expect("svg element");

    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, c))
    }
    let box_node = find(svg_id, &result.root).expect("svg box");

    // 默认关：无图元。
    let mut painter = Painter::new();
    let style = ComputedStyle::default();
    painter.paint_svg_element(box_node, 0.0, 0.0, &doc);
    assert_eq!(painter.primitives().images.len(), 0, "默认关应无 svg 图元");
    assert!(painter.canvas_images.is_empty(), "默认关应无像素注入");

    // 开关开：1 图元 + canvas_images 像素（100x50 盒）。
    // SAFETY：单线程测试环境（cargo test 默认多线程但本断言窗口内无其他读线程）。
    // 2024 edition 起 set_var/remove_var 为 unsafe。
    unsafe { std::env::set_var("ZW_INLINE_SVG_PAINT", "1") };
    let mut painter_on = Painter::new();
    painter_on.paint_svg_element(box_node, 0.0, 0.0, &doc);
    unsafe { std::env::remove_var("ZW_INLINE_SVG_PAINT") };
    assert_eq!(painter_on.primitives().images.len(), 1, "开关开应有 1 个 svg 图元");
    assert_eq!(painter_on.canvas_images.len(), 1, "canvas_images 应携带栅格化像素");
    let (key, w, h, rgba) = &painter_on.canvas_images[0];
    assert_eq!((*w, *h), (100, 50), "栅格化尺寸应等于盒尺寸");
    assert!(!rgba.is_empty(), "像素非空");
    assert_ne!(*key, 0, "哈希键非零");
}
