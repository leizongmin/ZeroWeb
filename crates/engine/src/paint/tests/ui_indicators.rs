//! CSS accent-color / caret-color / scrollbar-width / appearance 渲染测试。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    AccentColorComputedValue, AppearanceComputedValue, CaretColorComputedValue, ComputedStyle,
    ScrollbarWidthComputedValue,
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

/// 辅助函数：创建带 overflow 的 LayoutBox。
fn make_scrollable_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    }
}

// === accent-color 测试 ===

#[test]
fn test_accent_color_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // accent-color: Auto 不应生成额外图元
    let prims = painter.primitives();
    // 只有默认背景（transparent 不生成），不应有 accent-color 指示器
    assert!(
        prims
            .fills
            .iter()
            .all(|f| f.color.a == 0 || f.rect.size.width != 6.0 || f.rect.size.height != 6.0),
        "accent-color: Auto 不应生成指示器"
    );
}

#[test]
fn test_accent_color_custom_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 应生成 accent-color 指示器（6×6 红色方块）
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 6.0 && f.rect.size.height == 6.0 && f.color.r == 255 && f.color.g == 0);
    assert!(has_indicator, "accent-color: red 应生成红色指示器");
}

#[test]
fn test_accent_color_named_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.accent_color = AccentColorComputedValue::Color(ColorValue::Named("blue".to_string()));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_indicator = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 6.0 && f.rect.size.height == 6.0);
    assert!(has_indicator, "accent-color: blue 应生成指示器");
}

// === caret-color 测试 ===

#[test]
fn test_caret_color_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let style = ComputedStyle::default();
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // caret-color: Auto 不应生成光标指示器
    let has_caret = prims.fills.iter().any(|f| f.rect.size.width == 2.0);
    assert!(!has_caret, "caret-color: Auto 不应生成光标指示器");
}

#[test]
fn test_caret_color_custom_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(0, 255, 0, 255));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 应生成 caret-color 指示器（宽 2.0 的绿色竖条）
    let has_caret = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 2.0 && f.color.g == 255 && f.color.r == 0);
    assert!(has_caret, "caret-color: green 应生成绿色光标指示器");
}

#[test]
fn test_caret_color_position_respects_border() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");

    let mut layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);
    layout.border_left = 10.0;

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // caret x 应在 border_left 之后
    let caret = prims.fills.iter().find(|f| f.rect.size.width == 2.0);
    assert!(caret.is_some(), "应生成光标指示器");
    let caret = caret.unwrap();
    assert!(
        caret.rect.origin.x >= 10.0,
        "caret x 应在 border_left (10.0) 之后，实际 {}",
        caret.rect.origin.x
    );
}

// === scrollbar-width 测试 ===

#[test]
fn test_scrollbar_width_none_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_scrollable_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_width = ScrollbarWidthComputedValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // scrollbar-width: none 不应生成滚动条
    let has_scrollbar = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 10.0 || f.rect.size.width == 6.0);
    assert!(!has_scrollbar, "scrollbar-width: none 不应生成滚动条指示器");
}

#[test]
fn test_scrollbar_width_auto_generates_track() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_scrollable_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_width = ScrollbarWidthComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // scrollbar-width: auto 应生成 10px 宽的滚动条轨道
    let has_track = prims.fills.iter().any(|f| f.rect.size.width == 10.0);
    assert!(has_track, "scrollbar-width: auto 应生成滚动条轨道");
}

#[test]
fn test_scrollbar_width_thin_generates_thin_track() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_scrollable_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_width = ScrollbarWidthComputedValue::Thin;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // scrollbar-width: thin 应生成 6px 宽的滚动条轨道
    let has_track = prims.fills.iter().any(|f| f.rect.size.width == 6.0);
    assert!(has_track, "scrollbar-width: thin 应生成细滚动条轨道");
}

#[test]
fn test_scrollbar_width_no_overflow_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 无 overflow — 不应显示滚动条
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.scrollbar_width = ScrollbarWidthComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let has_scrollbar = prims
        .fills
        .iter()
        .any(|f| f.rect.size.width == 10.0 || f.rect.size.width == 6.0);
    assert!(!has_scrollbar, "无 overflow 时不应生成滚动条指示器");
}

// === appearance 测试 ===

#[test]
fn test_appearance_none_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // appearance: none 不应生成原生控件外观
    let prims = painter.primitives();
    let fill_count = prims.fills.len();
    assert_eq!(fill_count, 0, "appearance: none 不应生成原生控件图元");
}

#[test]
fn test_appearance_checkbox_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Checkbox;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(prims.fills.len() >= 2, "未选中 checkbox 应生成边框与白底");
}

#[test]
fn test_appearance_button_does_not_cover_standard_paint() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("button");
    let layout = make_box(Some(elem), 0.0, 0.0, 80.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Button;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert!(
        prims.fills.is_empty(),
        "button appearance 不应覆盖标准背景、边框和文字绘制"
    );
}

#[test]
fn test_appearance_textfield_generates_border() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 120.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Textfield;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // textfield 应生成白色背景 + 4 条边框
    assert!(prims.fills.len() >= 5, "textfield 应生成背景和 4 条边框");

    // 检查白色背景
    let has_white_bg = prims
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 255 && f.color.b == 255);
    assert!(has_white_bg, "textfield 应有白色背景");
}

#[test]
fn test_appearance_radio_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Radio;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    doc.set_attribute(elem, "checked", "");
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(!prims.fills.is_empty(), "选中 radio 应生成抗锯齿外圈与实心圆");
    assert!(
        prims
            .fills
            .iter()
            .any(|fill| fill.color.r == 0 && fill.color.g == 117 && fill.color.b == 255),
        "选中 radio 应包含默认 accent 色"
    );
}

#[test]
fn test_appearance_with_accent_color_override() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Checkbox;
    style.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(0, 128, 0, 255));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    doc.set_attribute(elem, "checked", "");
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // checkbox 内部应使用 accent-color 而非默认蓝色
    let has_green = prims.fills.iter().any(|f| f.color.g == 128 && f.color.r == 0);
    assert!(has_green, "checkbox 应使用 accent-color (绿色)");
}

// === 无节点 ID 不崩溃测试 ===

#[test]
fn test_accent_color_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 不应崩溃
    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 0);
}

#[test]
fn test_scrollbar_no_node_id_no_panic() {
    let mut layout = make_box(None, 0.0, 0.0, 100.0, 200.0);
    layout.overflow_y = OverflowClip::Hidden;
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 不应崩溃
}
