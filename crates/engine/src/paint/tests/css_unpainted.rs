//! CSS quotes / text-wrap / line-clamp / scrollbar-gutter / background-attachment / hyphens 渲染测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    BackgroundAttachmentComputedValue, ComputedStyle, HyphensComputedValue, LineClampComputedValue,
    QuotesComputedValue, ScrollbarGutterComputedValue, ScrollbarWidthComputedValue, TextWrapComputedValue,
};

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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

// === scrollbar-gutter 测试 ===

#[test]
fn test_scrollbar_gutter_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // scrollbar-gutter: auto 不应生成 gutter 指示器
    let prims = painter.primitives();
    let has_gutter = prims
        .fills
        .iter()
        .any(|f| f.color.r == 245 && f.color.g == 245 && f.color.b == 245 && f.color.a == 120);
    assert!(!has_gutter, "scrollbar-gutter: auto 不应生成 gutter 指示器");
}

#[test]
fn test_scrollbar_gutter_stable_generates_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // stable 应生成右侧 10px 宽的 gutter
    let has_gutter = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 10.0 && f.color.r == 245 && f.color.a == 120);
    assert!(has_gutter, "scrollbar-gutter: stable 应生成右侧 gutter");
}

#[test]
fn test_scrollbar_gutter_stable_both_edges_generates_two() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::StableBothEdges;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let gutter_count = prims
        .fills
        .iter()
        .filter(|f| f.color.r == 245 && f.color.a == 120 && f.rect.size.width == 10.0)
        .count();
    assert_eq!(
        gutter_count, 2,
        "scrollbar-gutter: stable both-edges 应生成左右两个 gutter"
    );
}

#[test]
fn test_scrollbar_gutter_thin_scrollbar_width() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
    style.scrollbar_width = ScrollbarWidthComputedValue::Thin;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // thin scrollbar 应生成 6px 宽的 gutter
    let has_thin_gutter = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 6.0 && f.color.r == 245 && f.color.a == 120);
    assert!(has_thin_gutter, "scrollbar-gutter: stable + thin 应生成 6px gutter");
}

// === background-attachment 测试 ===

#[test]
fn test_background_attachment_scroll_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(!has_pin, "background-attachment: scroll 不应生成固定背景指示器");
}

#[test]
fn test_background_attachment_fixed_generates_pin() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_attachment = BackgroundAttachmentComputedValue::Fixed;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 固定背景应生成蓝色图钉指示器
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(has_pin, "background-attachment: fixed 应生成图钉指示器");
}

#[test]
fn test_background_attachment_local_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_attachment = BackgroundAttachmentComputedValue::Local;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_pin = prims
        .fills
        .iter()
        .any(|f| f.color.r == 100 && f.color.b == 200 && f.color.a == 180);
    assert!(!has_pin, "background-attachment: local 不应生成固定背景指示器");
}

// === hyphens 测试 ===

#[test]
fn test_hyphens_auto_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.hyphens = HyphensComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // hyphens: auto 应在底部生成横线指示器
    let has_hyphen_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 8.0 && f.rect.size.height == 1.0 && f.color.a == 160);
    assert!(has_hyphen_indicator, "hyphens: auto 应生成横线指示器");
}

#[test]
fn test_hyphens_none_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("p");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_hyphen_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 8.0 && f.rect.size.height == 1.0 && f.color.a == 160);
    assert!(!has_hyphen_indicator, "hyphens: none 不应生成指示器");
}

// === quotes 测试 ===

#[test]
fn test_quotes_pairs_generates_glyphs() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("q");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.quotes = QuotesComputedValue::Pairs(vec![("\u{201C}".to_string(), "\u{201D}".to_string())]);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 应生成开引号和闭引号 glyph
    let open_quote_code = '\u{201C}' as u32;
    let close_quote_code = '\u{201D}' as u32;
    let has_open = prims.glyphs.iter().any(|g| g.glyph_id == open_quote_code);
    let has_close = prims.glyphs.iter().any(|g| g.glyph_id == close_quote_code);
    assert!(has_open, "quotes: Pairs 应生成开引号 glyph");
    assert!(has_close, "quotes: Pairs 应生成闭引号 glyph");
}

#[test]
fn test_quotes_none_no_glyphs() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("q");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.quotes = QuotesComputedValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let open_quote_code = '\u{201C}' as u32;
    let close_quote_code = '\u{201D}' as u32;
    let has_open = prims.glyphs.iter().any(|g| g.glyph_id == open_quote_code);
    let has_close = prims.glyphs.iter().any(|g| g.glyph_id == close_quote_code);
    assert!(!has_open, "quotes: none 不应生成引号 glyph");
    assert!(!has_close, "quotes: none 不应生成引号 glyph");
}

#[test]
fn test_quotes_auto_no_glyphs() {
    // quotes: auto 是默认值，不应为非 <q> 元素生成引号
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // Auto 不应生成引号 glyph（仅 Pairs 时渲染）
    // 检查不应有 « » 或 " " 引号 glyph
    let auto_open = '\u{00AB}' as u32; // «
    let auto_close = '\u{00BB}' as u32; // »
    let has_auto_quotes = prims
        .glyphs
        .iter()
        .any(|g| g.glyph_id == auto_open || g.glyph_id == auto_close);
    assert!(!has_auto_quotes, "quotes: auto (默认) 不应生成 « » 引号");
}

// === text-wrap 测试 ===

#[test]
fn test_text_wrap_nowrap_override() {
    let style = ComputedStyle::default();
    assert!(
        Painter::resolve_text_wrap(&style).is_none(),
        "text-wrap: wrap (默认) 不应覆盖换行设置"
    );
}

// === line-clamp 测试 ===

#[test]
fn test_line_clamp_none_default() {
    let style = ComputedStyle::default();
    assert!(
        Painter::resolve_line_clamp(&style).is_none(),
        "line-clamp: none (默认) 不应限制行数"
    );
}

#[test]
fn test_line_clamp_count_returns_value() {
    let mut style = ComputedStyle::default();
    style.line_clamp = LineClampComputedValue::Count(3);
    let result = Painter::resolve_line_clamp(&style);
    assert_eq!(result, Some(3), "line-clamp: 3 应返回 Some(3)");
}

// === 无节点 ID 不崩溃测试 ===

#[test]
fn test_scrollbar_gutter_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 200.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_background_attachment_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_hyphens_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 30.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}

#[test]
fn test_quotes_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 30.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);
    // 不应崩溃
}
