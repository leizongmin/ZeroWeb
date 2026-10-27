//! CSS accent-color / caret-color / scrollbar-width / appearance 渲染测试。
//!
//! 指示器在生产渲染中默认关闭（`Painter::skip_indicators` 默认 true），本模块
//! 每个用例显式置 false 开启，以验证对应 CSS 值确实会生成指示器。

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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, None);

    // appearance: none 不应生成原生控件外观
    let prims = painter.primitives();
    let fill_count = prims.fills.len();
    assert_eq!(fill_count, 0, "appearance: none 不应生成原生控件图元");
}

#[test]
fn test_appearance_auto_bare_input_no_native_paint() {
    // R5304 回归守卫：裸 input（computed auto，无 type）两态解析后 ZW 无对应 widget
    // chrome → 按普通盒渲染，不得生成任何原生图元。原 catchall 灰色指示器
    // rgba(230,230,230,200) 曾在此类元素上成片扰动 corpus（grouped-fallback/trailing-space
    // 等 21 案），已删；此断言防其回归。
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 120.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(prims.fills.is_empty(), "裸 input auto 不应生成原生控件图元（≡ 普通盒）");
}

#[test]
fn test_appearance_auto_checkbox_resolves_native() {
    // R5304 两态模型判例：作者 auto 覆盖 UA sheet 后仍按元素解析原生外观
    //（≡ appearance-auto-ref 断言组：auto 与全部 legacy 值互为兼容别名）。
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    doc.set_attribute(elem, "type", "checkbox");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(
        prims.fills.len() >= 2,
        "input[type=checkbox] auto 应解析为 checkbox 原生外观（边框+白底）"
    );
}

#[test]
fn test_appearance_checkbox_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    // R5304 两态模型：绘制侧按元素解析原生 appearance，checkbox chrome 须 type=checkbox。
    doc.set_attribute(elem, "type", "checkbox");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Checkbox;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(prims.fills.len() >= 2, "未选中 checkbox 应生成边框与白底");
    assert!(
        prims
            .fills
            .iter()
            .any(|fill| fill.color.r == 179 && fill.color.g == 179 && fill.color.b == 179),
        "未选中 checkbox 应生成抗锯齿角点"
    );
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
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert!(
        prims.fills.is_empty(),
        "button appearance 不应覆盖标准背景、边框和文字绘制"
    );
}

#[test]
fn test_appearance_textfield_no_native_paint() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    let layout = make_box(Some(elem), 0.0, 0.0, 120.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Textfield;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // R5304 两态模型：textfield 无专属 ZW 原生 chrome——按普通盒渲染（UA 样式素盒），
    // 不再叠加白底+#787878 边框（原臂覆盖 UA 样式成双边框，corpus 扰动已删）。
    assert!(prims.fills.is_empty(), "textfield 不应生成原生控件图元（≡ 普通盒）");
}

#[test]
fn test_appearance_radio_generates_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    // R5304 两态模型：radio chrome 须 type=radio（绘制侧按元素解析）。
    doc.set_attribute(elem, "type", "radio");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Radio;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
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
fn test_appearance_13px_radio_floors_fractional_y() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    doc.set_attribute(elem, "type", "radio");
    let layout = make_box(Some(elem), 0.0, 0.8, 13.0, 13.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Radio;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    doc.set_attribute(elem, "checked", "");
    painter.paint(&layout, &styles, Some(&doc));

    let top = painter
        .primitives()
        .fills
        .iter()
        .map(|fill| fill.rect.origin.y)
        .fold(f32::INFINITY, f32::min);
    assert_eq!(top, 0.0, "13px radio 模板应向下取整分数 y 坐标");
}

#[test]
fn test_appearance_with_accent_color_override() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("input");
    doc.set_attribute(elem, "type", "checkbox");
    let layout = make_box(Some(elem), 0.0, 0.0, 20.0, 20.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::Checkbox;
    style.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(0, 128, 0, 255));
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.skip_indicators = false;
    doc.set_attribute(elem, "checked", "");
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // checkbox 内部应使用 accent-color 而非默认蓝色
    let has_green = prims.fills.iter().any(|f| f.color.g == 128 && f.color.r == 0);
    assert!(has_green, "checkbox 应使用 accent-color (绿色)");
    assert!(
        !prims
            .fills
            .iter()
            .any(|fill| fill.rect.size.width == 6.0 && fill.rect.size.height == 6.0),
        "native checkbox 不应叠加通用 accent-color 色块"
    );

    // Chrome 真值（R3877，2026-08-31 GUI Chrome 151 探针）：焦点态不调暗 accent 填充。
    let mut focused_painter = Painter::new();
    focused_painter.skip_indicators = false;
    focused_painter.set_focused_node(Some(elem));
    focused_painter.paint(&layout, &styles, Some(&doc));
    assert!(
        focused_painter
            .primitives()
            .fills
            .iter()
            .any(|fill| fill.color.g == 128 && fill.color.r == 0),
        "focused checkbox 应保持原 accent-color（不调暗）"
    );
    assert!(
        !focused_painter
            .primitives()
            .fills
            .iter()
            .any(|fill| fill.color.g == 92 && fill.color.r == 0),
        "focused checkbox 不应出现 ×18/25 调暗色"
    );
}

// === 无节点 ID 不崩溃测试 ===

#[test]
fn test_accent_color_no_node_id_no_panic() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.skip_indicators = false;
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
    painter.skip_indicators = false;
    painter.paint(&layout, &styles, None);

    // 不应崩溃
}
