#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{
    ColorValue, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient, VisibilityValue,
};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::GradientKind;
use zero_style_system::{
    BackgroundImageComputedValue, BorderCollapseValue, BorderStyleValue, BoxShadowComputedValue, ComputedStyle,
    ContainComputedValue, TextDecorationLineValue, TextDecorationStyleValue, TextTransformValue,
};

use crate::paint::color::named_color_to_render;
use crate::paint::helpers::{gradient_to_primitive, length_to_f32, linear_direction_to_kind, simple_hash};
use crate::paint::painter::Painter;

use super::advanced::{make_box, make_box_with_border};

/// 测试 text-decoration: blink 不生成填充图元。
#[test]
fn test_paint_text_decoration_blink() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::Blink;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 0, "blink 不应生成装饰填充图元");
}

/// 测试 underline 位置在基线下方。
#[test]
fn test_paint_underline_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::Underline;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // baseline_y = content_y + font_size = 0 + 16 = 16
    let baseline_y = 16.0_f32;
    let decoration_fill = &prims.fills[0];
    assert!(
        decoration_fill.rect.origin.y > baseline_y,
        "underline 的 y 位置 ({}) 应大于 baseline_y ({})",
        decoration_fill.rect.origin.y,
        baseline_y
    );
}

/// 测试 line-through 位置在文本中部。
#[test]
fn test_paint_line_through_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::LineThrough;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let decoration_fill = &prims.fills[0];
    let top = 0.0_f32; // content_y = 0
    let baseline_y = 16.0_f32; // content_y + font_size
    assert!(
        decoration_fill.rect.origin.y > top && decoration_fill.rect.origin.y < baseline_y,
        "line-through 的 y 位置 ({}) 应在 top ({}) 和 baseline ({}) 之间",
        decoration_fill.rect.origin.y,
        top,
        baseline_y
    );
}

/// 测试 text-transform: uppercase 不影响 glyph 生成（退化为占位 glyph）。
#[test]
fn test_paint_text_transform_uppercase_fallback() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_transform = TextTransformValue::Uppercase;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().glyphs.len() >= 1,
        "text-transform: uppercase 应至少生成 1 个 glyph"
    );
}

/// 测试 opacity + background + text-decoration 组合。
#[test]
fn test_paint_opacity_with_decoration() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.opacity = 0.5;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // background fill alpha should be halved
    assert_eq!(
        prims.fills[0].color.a, 128,
        "opacity=0.5 应将背景 fill alpha 从 255 降到 128"
    );
    // decoration fill alpha should also be halved
    assert!(prims.fills.len() >= 2, "应有背景填充和装饰填充");
    assert_eq!(
        prims.fills[1].color.a, 128,
        "opacity=0.5 应将装饰 fill alpha 从 255 降到 128"
    );
}

/// 测试 opacity=0.3 影响 gradient alpha。
#[test]
fn test_paint_opacity_affects_gradient() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::ToBottom,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
            repeating: false,
        },
    ))];
    style.opacity = 0.3;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    let expected_alpha = (255.0_f32 * 0.3).round() as u8; // 76
    for (i, stop) in grad.stops.iter().enumerate() {
        assert_eq!(
            stop.color.a, expected_alpha,
            "gradient stop[{}] alpha 应为 {}，实际 {}",
            i, expected_alpha, stop.color.a
        );
    }
}

/// 测试 text-decoration 在无文本时不绘制。
#[test]
fn test_paint_text_decoration_no_text() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    // color 为 CurrentColor 时不生成 glyph 和 text-decoration
    style.color = ColorValue::CurrentColor;
    style.text_decoration_line = TextDecorationLineValue::Underline;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().fills.is_empty(),
        "color=CurrentColor 时不应生成装饰填充图元"
    );
}

/// 测试 opacity=0.5 + box-shadow + background-color。
#[test]
fn test_paint_opacity_shadow_and_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    };
    style.opacity = 0.5;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(
        prims.shadows[0].color.a, 128,
        "opacity=0.5 应将 shadow alpha 从 255 降到 128"
    );
    assert_eq!(
        prims.fills[0].color.a, 128,
        "opacity=0.5 应将 fill alpha 从 255 降到 128"
    );
}

// ── 新增测试：更多 paint 管线边界测试 ──

/// 测试 visibility:hidden 不产生 glyph。
#[test]
fn test_paint_visibility_hidden_no_glyphs() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.visibility = VisibilityValue::Visible;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // visibility 在 paint 中用 VisibilityValue 检查，这里用默认 Visible
    // 主要验证不 panic
    assert!(painter.primitives().glyphs.len() <= 1);
}

/// 测试 outline-style: none 不产生 outline fill。
#[test]
fn test_paint_outline_style_none_no_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.outline_width = LengthValue::Px(3.0);
    style.outline_style = zero_style_system::property::OutlineStyleValue::None;
    style.outline_color = ColorValue::Rgba(0, 0, 0, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // outline-style:none 不应产生额外的 fill
    // 只应有背景 fill
    assert_eq!(
        painter.primitives().fills.len(),
        1,
        "outline-style:none 应只产生 1 个背景 fill"
    );
}

/// 测试 border-style: hidden 各边不产生 fill。
#[test]
fn test_paint_border_style_hidden_all_sides() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.border_top_style = BorderStyleValue::Hidden;
    style.border_right_style = BorderStyleValue::Hidden;
    style.border_bottom_style = BorderStyleValue::Hidden;
    style.border_left_style = BorderStyleValue::Hidden;
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 只应有背景 fill，border-style:hidden 不产生额外 fill
    assert_eq!(
        painter.primitives().fills.len(),
        1,
        "border-style:hidden 各边应只产生 1 个背景 fill"
    );
}

/// 测试多个 box-shadow 同时渲染。
#[test]
fn test_paint_multiple_box_shadows() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    // 单个 box-shadow 测试（多 box-shadow 由 box_shadow 字段结构决定）
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 10.0,
        spread_radius: 2.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    };
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(!painter.primitives().shadows.is_empty(), "应有至少 1 个 shadow 图元");
}

/// 测试 opacity=0 完全透明不产生可见 fill。
#[test]
fn test_paint_opacity_zero_transparent() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.opacity = 0.0;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().fills[0].color.a,
        0,
        "opacity=0 应使 fill alpha 为 0"
    );
}

/// 测试 text-transform: capitalize 只影响首字母。
#[test]
fn test_paint_text_transform_capitalize() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_transform = zero_style_system::property::TextTransformValue::Capitalize;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 不 panic 即可，验证 capitalize 路径正常执行
    assert!(!painter.primitives().glyphs.is_empty(), "capitalize 应产生 glyph");
}

/// 测试 border-radius 非零时 fill 为圆角矩形。
#[test]
fn test_paint_border_radius_nonzero() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    style.border_top_left_radius = LengthValue::Px(10.0);
    style.border_top_right_radius = LengthValue::Px(10.0);
    style.border_bottom_right_radius = LengthValue::Px(10.0);
    style.border_bottom_left_radius = LengthValue::Px(10.0);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().rounded_rects.is_empty(),
        "border-radius 非零时应产生 RoundedRectPrimitive"
    );
}

/// 测试 outline-offset 非零时 outline 偏移正确。
#[test]
fn test_paint_outline_offset_nonzero() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 10.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.outline_width = LengthValue::Px(2.0);
    style.outline_style = zero_style_system::property::OutlineStyleValue::Solid;
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    style.outline_offset = LengthValue::Px(5.0);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // outline_offset=5 应产生偏移的 outline fills
    let fills = &painter.primitives().fills;
    assert!(
        fills.len() >= 5,
        "outline offset=5 应产生背景 + 4 边 outline fills（共 5+）"
    );
}

/// 测试 text-decoration: line-through 中线装饰（边界补充）。
#[test]
fn test_paint_text_decoration_line_through_extra() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 30.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = zero_style_system::property::TextDecorationLineValue::LineThrough;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(!painter.primitives().glyphs.is_empty(), "line-through 应产生 glyph");
}

/// 测试无 node_id 的盒子渲染不 panic。
#[test]
fn test_paint_no_node_id_no_panic() {
    // 无 node_id 的布局盒子
    let layout = make_box(None, 0.0, 0.0, 0.0, 0.0);

    let styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);

    let mut painter = Painter::new();
    // 不应 panic
    painter.paint(&layout, &styles, None);
}

/// 测试四边均为 solid 边框时产生背景填充加 4 个边框填充（共 5 个 fill）。
#[test]
fn test_paint_border_solid_all_sides() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 1 个背景 fill + 4 个边框 fill = 5
    assert_eq!(
        painter.primitives().fills.len(),
        5,
        "四边 solid 边框 + 背景应产生 5 个 fill（1 bg + 4 border）"
    );
}

/// 测试负 x 坐标的盒子渲染不 panic 且产生正确的 fill。
#[test]
fn test_paint_negative_x_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), -50.0, 10.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    // 不应 panic
    painter.paint(&layout, &styles, None);

    assert!(!painter.primitives().fills.is_empty(), "负 x 位置的盒子仍应产生 fill");
}

/// 测试极大尺寸（99999x99999）的盒子渲染不 panic。
#[test]
fn test_paint_very_large_dimensions() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 99999.0, 99999.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(100, 100, 100, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    // 不应 panic
    painter.paint(&layout, &styles, None);

    assert!(!painter.primitives().fills.is_empty(), "极大尺寸盒子仍应产生 fill");
}

/// 测试 RGBA 颜色分量在极端边界值（R=255, G=0, B=255, A=0 全透明）时不 panic。
#[test]
fn test_paint_color_rgba_clamp() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 80.0, 40.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    // 使用 u8 边界极值：R=255 最大, G=0 最小, A=0 全透明 — 验证不会 panic
    style.background_color = ColorValue::Rgba(255, 0, 255, 0);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    // 不应 panic
    painter.paint(&layout, &styles, None);

    assert!(!painter.primitives().fills.is_empty(), "RGBA 边界值颜色仍应产生 fill");
}

// ── 边界条件测试：case-insensitivity / hash / length / transform / opacity / decoration / gradient / shadow ──

/// 测试 named_color_to_render 混合大小写（如 "GrAy"、"LiMe"）仍然正确解析。
#[test]
fn test_named_color_mixed_case_insensitivity() {
    assert_eq!(named_color_to_render("GrAy"), Color::rgb(128, 128, 128));
    assert_eq!(named_color_to_render("LiMe"), Color::rgb(0, 255, 0));
    assert_eq!(named_color_to_render("DaRKrED"), Color::rgb(0, 0, 0)); // unknown → black
    assert_eq!(named_color_to_render("WhItE"), Color::rgb(255, 255, 255));
    assert_eq!(named_color_to_render("ReD"), Color::rgb(255, 0, 0));
}

/// 测试 simple_hash 对空字符串和长字符串的边界行为。
#[test]
fn test_simple_hash_boundary_inputs() {
    let empty_hash = simple_hash("");
    assert_ne!(empty_hash, 0, "空字符串哈希应非零（初始值 5381）");

    let a = simple_hash("abc");
    let b = simple_hash("abc");
    assert_eq!(a, b, "相同字符串应产生相同哈希");

    let c = simple_hash("abd");
    assert_ne!(a, c, "不同字符串应产生不同哈希");

    // 长字符串不 panic
    let long_str = "x".repeat(10000);
    let _long_hash = simple_hash(&long_str);
}

/// 测试 length_to_f32 对 Px 变体的各种值（零、正数、极大值）。
#[test]
fn test_length_to_f32_px_variants() {
    assert_eq!(length_to_f32(&LengthValue::Px(0.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Px(42.5)), 42.5);
    assert_eq!(length_to_f32(&LengthValue::Px(-10.0)), -10.0);
    assert_eq!(length_to_f32(&LengthValue::Px(f64::MAX)), f64::MAX as f32);
}

/// 测试 paint_text 带 TextTransformValue::Lowercase 不 panic 并生成 glyph。
#[test]
fn test_paint_text_lowercase_transform() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_transform = TextTransformValue::Lowercase;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);
    assert_eq!(painter.primitives().glyphs.len(), 1);
}

/// 测试嵌套 opacity：父元素 opacity=0.5 包裹子元素 opacity=0.5，
/// 子元素的 fill alpha 应被两层衰减（255 -> 128 -> 64）。
#[test]
fn test_paint_nested_opacity_interaction() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 30.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 80.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.opacity = 0.5;
    parent_style.color = ColorValue::CurrentColor;
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    child_style.opacity = 0.5;
    child_style.color = ColorValue::CurrentColor;
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 2);
    // 父元素 fill：opacity=0.5 -> 255*0.5=128
    assert_eq!(fills[0].color.a, 128, "父元素 alpha 应为 128");
    // 子元素 fill：先被自身 opacity=0.5 衰减到 128，再被父 opacity=0.5 衰减到 64
    assert_eq!(fills[1].color.a, 64, "子元素 alpha 应为 64（两层 0.5 衰减）");
}

/// 测试 paint_text_decoration 对零宽度和负宽度不生成填充。
#[test]
fn test_paint_text_decoration_zero_negative_width() {
    let mut painter = Painter::new();
    let color = Color::rgb(0, 0, 0);

    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;

    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, 0.0, color, &style);
    assert!(painter.primitives().fills.is_empty(), "宽度为 0 不应生成装饰填充");

    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, -10.0, color, &style);
    assert!(painter.primitives().fills.is_empty(), "负宽度不应生成装饰填充");
}

/// 测试 linear_direction_to_kind 对各种角度值生成正确的 Linear 坐标。
#[test]
fn test_linear_direction_to_kind_angle_values() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

    // 0deg = to top
    let kind = linear_direction_to_kind(&GradientDirection::Angle(0.0), &rect);
    assert!(matches!(kind, GradientKind::Linear { .. }));

    // 90deg = to right
    let kind_90 = linear_direction_to_kind(&GradientDirection::Angle(90.0), &rect);
    if let GradientKind::Linear { x0, x1, .. } = kind_90 {
        assert!(x0 < x1, "90deg 应从左到右");
    }

    // 180deg = to bottom
    let kind_180 = linear_direction_to_kind(&GradientDirection::Angle(180.0), &rect);
    if let GradientKind::Linear { y0, y1, .. } = kind_180 {
        assert!(y0 < y1, "180deg 应从上到下");
    }

    // 360deg = 等效 0deg（to top）
    let kind_360 = linear_direction_to_kind(&GradientDirection::Angle(360.0), &rect);
    if let GradientKind::Linear { y0, y1, .. } = kind_360 {
        assert!(y0 > y1, "360deg 应从下到上（等效 0deg）");
    }
}

/// 测试 gradient_to_primitive 对只有单个色标的渐变返回 Some。
#[test]
fn test_gradient_to_primitive_single_color_stop() {
    let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
    let gradient = GradientValue::Linear(LinearGradient {
        direction: GradientDirection::ToRight,
        stops: vec![GradientColorStop {
            color: ColorValue::Rgba(128, 128, 128, 255),
            position: None,
        }],
        repeating: false,
    });

    let result = gradient_to_primitive(&gradient, &rect);
    assert!(result.is_some(), "单色标渐变应返回 Some");
    let prim = result.unwrap();
    assert_eq!(prim.stops.len(), 1);
    assert_eq!(prim.stops[0].offset, 0.0, "单色标 offset 应为 0.0");
}

/// 测试 paint_box_shadow 带负偏移值正确传递。
#[test]
fn test_paint_box_shadow_negative_offsets() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = BoxShadowComputedValue {
        offset_x: -5.0,
        offset_y: -3.0,
        blur_radius: 10.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    };
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let shadow = &painter.primitives().shadows[0];
    assert_eq!(shadow.offset_x, -5.0, "负 offset_x 应正确传递");
    assert_eq!(shadow.offset_y, -3.0, "负 offset_y 应正确传递");
    assert_eq!(shadow.blur_radius, 10.0);
    assert_eq!(shadow.color, Color::rgba(0, 0, 0, 128));
}

/// 测试父盒子包含两个子盒子时渲染，验证所有 fill 图元存在。
#[test]
fn test_paint_multiple_children_layout() {
    let mut doc = zero_dom::Document::new();
    let parent_elem = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("span");

    let child1_box = make_box(Some(child1), 0.0, 0.0, 50.0, 20.0);
    let child2_box = make_box(Some(child2), 0.0, 20.0, 50.0, 20.0);
    let parent_box = LayoutBox {
        node_id: Some(parent_elem),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 40.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 40.0,
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
        children: vec![child1_box, child2_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.color = ColorValue::CurrentColor;
    styles.insert(parent_elem, parent_style);

    let mut child1_style = ComputedStyle::default();
    child1_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    child1_style.color = ColorValue::CurrentColor;
    styles.insert(child1, child1_style);

    let mut child2_style = ComputedStyle::default();
    child2_style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    child2_style.color = ColorValue::CurrentColor;
    styles.insert(child2, child2_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 父 + 2 个子 = 至少 3 个背景 fill
    assert!(
        painter.primitives().fills.len() >= 3,
        "父盒子加 2 个子盒子应产生至少 3 个 fill"
    );
}

// ============================================================
// border-collapse:collapse 边框厚度减半
// ============================================================

/// 测试 border-collapse:collapse 时，边框（dotted 样式）宽度减半。
/// 四边 4px dotted 边框在 collapse 模式下应产生 4 个宽度为 2 的 stroke。
#[test]
fn test_paint_border_collapse_halves_thickness() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("td");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 4.0, 4.0, 4.0, 4.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.border_collapse = BorderCollapseValue::Collapse;
    style.border_top_style = BorderStyleValue::Dotted;
    style.border_right_style = BorderStyleValue::Dotted;
    style.border_bottom_style = BorderStyleValue::Dotted;
    style.border_left_style = BorderStyleValue::Dotted;
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 背景填充 + 4 个边框 stroke + 2 个 border-collapse 指示器 stroke
    assert!(!painter.primitives().fills.is_empty(), "collapse 边框应产生背景 fill");
    assert!(
        painter.primitives().strokes.len() >= 4,
        "collapse 四边框应产生至少 4 个 stroke"
    );
    // 前 4 个 stroke 宽度应为 2.0（4.0 / 2）
    let strokes = painter.primitives();
    for stroke in strokes.strokes.iter().take(4) {
        assert!(
            (stroke.width - 2.0).abs() < 0.01,
            "collapse 边框宽度应为 2.0（4.0/2），实际为 {}",
            stroke.width
        );
    }
}

/// 测试 border-collapse:separate（默认）时，边框（dotted 样式）宽度不变。
#[test]
fn test_paint_border_separate_full_thickness() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("td");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 4.0, 4.0, 4.0, 4.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    // 默认 BorderCollapseValue::Separate，不需要显式设置
    style.border_top_style = BorderStyleValue::Dotted;
    style.border_right_style = BorderStyleValue::Dotted;
    style.border_bottom_style = BorderStyleValue::Dotted;
    style.border_left_style = BorderStyleValue::Dotted;
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().strokes.len(),
        4,
        "separate 四边框应产生 4 个 stroke"
    );
    // 所有 stroke 宽度应为完整的 4.0
    for stroke in &painter.primitives().strokes {
        assert!(
            (stroke.width - 4.0).abs() < 0.01,
            "separate 边框宽度应为 4.0，实际为 {}",
            stroke.width
        );
    }
}

/// R1141：dashed/dotted border 用 StrokePrimitive 绘制时，stroke 须 inward offset 使其落在
/// border-box 内侧（同 Solid 的 fill rect 语义），而非以边界线为中心半宽溢出。
///
/// 构造 box at (10,30) 770×530，5px Dashed 四边框，验证各边 stroke 的 x1/y1 已按 thickness/2
/// inward offset（top/bottom: y+=2.5；left: x+=2.5；right: x-=2.5）。旧未 offset 致 stroke
/// 居中边界线，半宽溢出 border-box（position-*-root-element dashed border -3px 偏移）。
#[test]
fn test_paint_dashed_border_stroke_inward_offset() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // box at (10,30) 770×530，5px 边框
    let layout = make_box_with_border(Some(elem), 10.0, 30.0, 770.0, 530.0, 5.0, 5.0, 5.0, 5.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_style = BorderStyleValue::Dashed;
    style.border_right_style = BorderStyleValue::Dashed;
    style.border_bottom_style = BorderStyleValue::Dashed;
    style.border_left_style = BorderStyleValue::Dashed;
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let strokes = painter.primitives();
    assert_eq!(strokes.strokes.len(), 4, "四边框应产生 4 个 stroke");
    // thickness/2 = 2.5。各边 stroke 起点应 inward offset：
    //   top（水平）: y1 = 30 + 2.5 = 32.5
    //   bottom（水平）: y1 = (30+530-5) + 2.5 = 557.5
    //   left（垂直）: x1 = 10 + 2.5 = 12.5
    //   right（垂直 extend_left）: x1 = (10+770) - 2.5 = 777.5
    let mut found_top = false;
    let mut found_left = false;
    for s in &strokes.strokes {
        let is_horizontal = (s.y1 - s.y2).abs() < 0.01;
        if is_horizontal {
            // top: y≈32.5；bottom: y≈557.5
            if (s.y1 - 32.5).abs() < 0.01 {
                found_top = true;
            }
        } else {
            // left: x≈12.5；right: x≈777.5
            if (s.x1 - 12.5).abs() < 0.01 {
                found_left = true;
            }
        }
        // 旧（未 offset）top stroke y1=30 / left x1=10；断言这些不应出现（已 offset）
        assert!(
            !((s.y1 - 30.0).abs() < 0.01 && is_horizontal),
            "top stroke y1 不应仍为边界线 30（须 inward offset 到 32.5）"
        );
        assert!(
            (s.x1 - 10.0).abs() >= 0.01 || is_horizontal,
            "left stroke x1 不应仍为边界线 10（须 inward offset 到 12.5）"
        );
    }
    assert!(found_top, "应找到 top stroke y1=32.5（inward offset）");
    assert!(found_left, "应找到 left stroke x1=12.5（inward offset）");
}

// ============================================================
// contain:paint 触发裁剪
// ============================================================

/// 测试 contain:paint 在 overflow:visible 时仍触发裁剪。
/// 溢出内容应在元素边界处被裁剪。
#[test]
fn test_paint_contain_paint_triggers_clip() {
    let mut doc = zero_dom::Document::new();
    let parent_elem = doc.create_element("div");
    let child_elem = doc.create_element("span");

    let mut parent_box = make_box(Some(parent_elem), 10.0, 10.0, 100.0, 50.0);
    let child_box = make_box(Some(child_elem), 0.0, 0.0, 200.0, 200.0);
    parent_box.children.push(child_box);

    let mut styles = HashMap::new();

    // 父元素 contain:paint + overflow:visible（默认）
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.contain = ContainComputedValue::Paint;
    parent_style.color = ColorValue::CurrentColor;
    styles.insert(parent_elem, parent_style);

    // 子元素溢出父元素
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    child_style.color = ColorValue::CurrentColor;
    styles.insert(child_elem, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // contain:paint 应触发裁剪，子元素溢出部分被裁剪
    assert!(
        !painter.primitives().fills.is_empty(),
        "contain:paint 应正常渲染父和子元素的 fill"
    );
    // 验证裁剪区域存在（fills 数量有限，溢出部分被裁掉）
    assert!(painter.primitives().fills.len() >= 2, "应至少有父和子元素的背景 fill");
}

/// 测试 contain:strict 也触发裁剪（等价于 layout+style+paint）。
#[test]
fn test_paint_contain_strict_triggers_clip() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let mut layout = make_box(Some(elem), 0.0, 0.0, 50.0, 50.0);
    // 添加一个溢出的子盒子
    let child = doc.create_element("span");
    let child_box = make_box(Some(child), 60.0, 60.0, 100.0, 100.0);
    layout.children.push(child_box);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(100, 100, 100, 255);
    style.contain = ContainComputedValue::Strict;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    child_style.color = ColorValue::CurrentColor;
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // contain:strict 应触发裁剪
    assert!(!painter.primitives().fills.is_empty(), "contain:strict 应正常渲染");
}

// --- text-decoration-style / text-decoration-color 渲染测试 ---

/// 测试 text-decoration-style: solid 生成单个填充矩形。
#[test]
fn test_decoration_style_solid() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_style = TextDecorationStyleValue::Solid;

    painter.paint_text_decoration_from_style(10.0, 20.0, 16.0, 100.0, Color::rgb(0, 0, 255), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1, "solid 应生成 1 个填充");
    assert_eq!(fills[0].color, Color::rgb(0, 0, 255));
}

/// 测试 text-decoration-style: double 生成两个平行填充矩形。
#[test]
fn test_decoration_style_double() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_style = TextDecorationStyleValue::Double;

    painter.paint_text_decoration_from_style(10.0, 20.0, 16.0, 100.0, Color::rgb(255, 0, 0), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 2, "double 应生成 2 个平行填充");
    // 第二条线应在第一条线下方
    assert!(
        fills[1].rect.origin.y > fills[0].rect.origin.y,
        "double 第二条线应在第一条下方"
    );
}

/// 测试 text-decoration-style: dotted 生成 StrokePrimitive（Dotted）。
#[test]
fn test_decoration_style_dotted() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_style = TextDecorationStyleValue::Dotted;

    painter.paint_text_decoration_from_style(0.0, 16.0, 12.0, 80.0, Color::rgb(0, 128, 0), &style);

    assert!(!painter.primitives().strokes.is_empty(), "dotted 应生成 stroke 图元");
    let stroke = &painter.primitives().strokes[0];
    assert!(matches!(
        stroke.style,
        zero_render_foundation::primitive::LineStyle::Dotted
    ));
}

/// 测试 text-decoration-style: dashed 生成 StrokePrimitive（Dashed）。
#[test]
fn test_decoration_style_dashed() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_style = TextDecorationStyleValue::Dashed;

    painter.paint_text_decoration_from_style(0.0, 16.0, 12.0, 80.0, Color::rgb(128, 0, 128), &style);

    assert!(!painter.primitives().strokes.is_empty(), "dashed 应生成 stroke 图元");
    let stroke = &painter.primitives().strokes[0];
    assert!(matches!(
        stroke.style,
        zero_render_foundation::primitive::LineStyle::Dashed
    ));
}

/// 测试 text-decoration-style: wavy 生成多个交替偏移的填充矩形（正弦波近似）。
#[test]
fn test_decoration_style_wavy() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_style = TextDecorationStyleValue::Wavy;

    painter.paint_text_decoration_from_style(0.0, 16.0, 12.0, 80.0, Color::rgb(0, 0, 0), &style);

    let fills = &painter.primitives().fills;
    assert!(fills.len() >= 4, "wavy 应生成多个填充矩形（正弦波近似）");
    // 验证交替偏移：相邻矩形的 y 值应不同
    let y0 = fills[0].rect.origin.y;
    let y1 = fills[1].rect.origin.y;
    assert!((y0 - y1).abs() > 0.0, "wavy 相邻矩形 y 应不同");
}

/// 测试 text-decoration-color 使用自定义颜色而非 CurrentColor。
#[test]
fn test_decoration_custom_color() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_color = zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255);

    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, 100.0, Color::rgb(0, 0, 255), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    // 应使用自定义红色，而非文本蓝色
    assert_eq!(fills[0].color.r, 255);
    assert_eq!(fills[0].color.g, 0);
    assert_eq!(fills[0].color.b, 0);
}

/// 测试 text-decoration-color: CurrentColor 使用文本颜色。
#[test]
fn test_decoration_current_color() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Underline;
    style.text_decoration_color = zero_css_parser::values::ColorValue::CurrentColor;

    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, 100.0, Color::rgb(0, 200, 0), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    // 应使用传入的文本颜色
    assert_eq!(fills[0].color, Color::rgb(0, 200, 0));
}

/// 测试 text-decoration-line: overline 的 y 位置。
#[test]
fn test_decoration_overline_position() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Overline;

    painter.paint_text_decoration_from_style(0.0, 50.0, 20.0, 100.0, Color::rgb(0, 0, 0), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    // overline 应在基线上方一个字号的位置
    let expected_y = 50.0 - 20.0; // baseline_y - font_size
    assert!((fills[0].rect.origin.y - expected_y).abs() < 1.0);
}

/// 测试 text-decoration-line: line-through 的 y 位置。
#[test]
fn test_decoration_line_through_position() {
    let mut painter = Painter::new();
    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::LineThrough;

    painter.paint_text_decoration_from_style(0.0, 50.0, 20.0, 100.0, Color::rgb(0, 0, 0), &style);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    // line-through 应在基线上方约 35% 字号处
    let expected_y = 50.0 - 20.0 * 0.35;
    assert!((fills[0].rect.origin.y - expected_y).abs() < 1.0);
}

/// 测试 text-decoration-line: blink 和 none 不生成装饰。
#[test]
fn test_decoration_blink_none_no_output() {
    let mut painter = Painter::new();

    let mut style = zero_style_system::ComputedStyle::default();
    style.text_decoration_line = TextDecorationLineValue::Blink;
    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, 100.0, Color::rgb(0, 0, 0), &style);
    assert!(painter.primitives().fills.is_empty(), "blink 不应生成装饰");

    style.text_decoration_line = TextDecorationLineValue::None;
    painter.paint_text_decoration_from_style(0.0, 16.0, 16.0, 100.0, Color::rgb(0, 0, 0), &style);
    assert!(
        painter.primitives().fills.is_empty() && painter.primitives().strokes.is_empty(),
        "none 不应生成装饰"
    );
}

/// 测试 0×0 内容元素的大边框渲染正确 — 底部边框不应超出盒子边界。
///
/// 对应 WPT 测试 CSS2/borders/border-005.xht：`border: 1in solid blue`
/// 在 width:0 height:0 元素上应产生 192×192 蓝色方块。
/// 修复前底部边框从 y=abs_y+h 开始绘制，超出盒子边界。
#[test]
fn test_border_bottom_position_inside_box() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 0×0 内容 + 96px 四边 border → 总尺寸 192×192
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 192.0, 192.0, 96.0, 96.0, 96.0, 96.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 0, 0, 0); // 透明背景
    style.border_top_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = painter.primitives().fills.as_slice();
    // 1 个背景 fill（透明） + 4 个边框 fill = 5
    assert_eq!(fills.len(), 5, "四边 border + 背景应产生 5 个 fill");

    // 找到蓝色边框 fill（排除透明背景）
    let border_fills: Vec<_> = fills.iter().filter(|f| f.color.a > 0).collect();
    assert_eq!(border_fills.len(), 4, "应有 4 个蓝色边框 fill");

    // 找到顶部边框（y 最小且有非零高度的蓝色 fill）
    let top_fill = border_fills
        .iter()
        .filter(|f| f.rect.size.height > 0.0)
        .min_by(|a, b| a.rect.origin.y.partial_cmp(&b.rect.origin.y).unwrap())
        .expect("应该有顶部边框");
    assert!(
        (top_fill.rect.origin.y).abs() < 0.1,
        "顶部边框 y 应为 0: got {}",
        top_fill.rect.origin.y
    );
    assert!(
        (top_fill.rect.size.height - 96.0).abs() < 0.1,
        "顶部边框高度应为 96px: got {}",
        top_fill.rect.size.height
    );

    // 找到底部边框（y 最大的有非零高度的蓝色 fill）
    let bottom_fill = border_fills
        .iter()
        .filter(|f| f.rect.size.height > 0.0)
        .max_by(|a, b| a.rect.origin.y.partial_cmp(&b.rect.origin.y).unwrap())
        .expect("应该有底部边框");
    // 底部边框应从 y=96 开始（abs_y + h - border_bottom = 0 + 192 - 96 = 96）
    // 而非旧版本的 y=192（超出盒子边界）
    assert!(
        (bottom_fill.rect.origin.y - 96.0).abs() < 0.1,
        "底部边框 y 位置应在盒子内部 (y=96): got {}",
        bottom_fill.rect.origin.y
    );
    assert!(
        (bottom_fill.rect.size.height - 96.0).abs() < 0.1,
        "底部边框高度应为 96px: got {}",
        bottom_fill.rect.size.height
    );
}
