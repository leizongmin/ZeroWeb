#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{
    ColorValue, ConicGradient, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
    RadialGradient, RadialShape, RadialSize, VisibilityValue,
};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::GradientKind;
use zero_style_system::{
    BackgroundImageComputedValue, BorderStyleValue, BoxShadowComputedValue, ComputedStyle, TextDecorationLineValue,
    TextShadowComputedValue,
};

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
pub(super) fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
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

/// 辅助函数：创建带边框的 LayoutBox。
pub(super) fn make_box_with_border(
    node_id: Option<NodeId>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    border_left: f32,
) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: border_left,
        content_y: border_top,
        content_width: width - border_left - border_right,
        content_height: height - border_top - border_bottom,
        border_top,
        border_right,
        border_bottom,
        border_left,
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
// ── 新增测试：组合渲染 ──────────────────────────────────

/// 测试 box-shadow + background-color + border + text-shadow 全组合。
#[test]
fn test_paint_combined_box_shadow_background_border_text_shadow() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    };
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 1.0,
        offset_y: 1.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(128, 128, 128, 128),
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 1 background fill + 4 border fills = 5 fills
    assert_eq!(prims.fills.len(), 5, "应生成 5 个填充（1 背景 + 4 边框）");
    // 1 shadow
    assert_eq!(prims.shadows.len(), 1, "应生成 1 个 box-shadow");
    // 2 glyphs (shadow glyph + main glyph)
    assert_eq!(prims.glyphs.len(), 2, "应生成 2 个 glyph（text-shadow + 主文本）");
}

/// box-shadow 颜色为 `currentColor` 时须解析为元素自身 `color`，而非回落黑色。
/// driving: WPT box-shadow-currentcolor（`color:transparent` 时 box-shadow 应透明，
/// 旧实现 color_value_to_render 无元素上下文 → 黑色 alpha=255 实心阴影）。
#[test]
fn test_paint_box_shadow_currentcolor_resolves_to_element_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.color = ColorValue::Transparent; // 元素 color = 透明
    style.font_size = LengthValue::Px(16.0);
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 10.0,
        offset_y: 5.0,
        blur_radius: 5.0,
        spread_radius: 0.0,
        color: ColorValue::CurrentColor, // currentColor → 须解析为元素 color(transparent)
        inset: false,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.shadows.len(), 1, "应生成 1 个 box-shadow");
    // shadow 颜色须为 transparent（alpha=0），非黑色回落（alpha=255）。
    assert_eq!(
        prims.shadows[0].color.a, 0,
        "box-shadow currentColor 应解析为元素 color(transparent, alpha=0)，非黑色回落"
    );

    // 对照：显式 limegreen 元素 color + currentColor box-shadow → 阴影应为 limegreen。
    let mut styles2 = HashMap::new();
    let mut style2 = ComputedStyle::default();
    style2.color = ColorValue::Rgba(50, 205, 50, 255); // limegreen（具体值，避免 named 表耦合）
    style2.font_size = LengthValue::Px(16.0);
    style2.box_shadow = BoxShadowComputedValue {
        offset_x: 10.0,
        offset_y: 5.0,
        blur_radius: 5.0,
        spread_radius: 0.0,
        color: ColorValue::CurrentColor,
        inset: false,
    };
    styles2.insert(elem, style2);
    let mut painter2 = Painter::new();
    painter2.paint(&layout, &styles2, None);
    let c = painter2.primitives().shadows[0].color;
    assert_eq!((c.r, c.g, c.b), (50, 205, 50), "currentColor 应解析为元素 color");
}

/// 测试 visibility:hidden 时 box-shadow 和 background-image 不绘制。
#[test]
fn test_paint_visibility_hidden_no_shadow_no_image() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.visibility = VisibilityValue::Hidden;
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    };
    style.background_image = vec![BackgroundImageComputedValue::Url("test.png".to_string())];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert!(prims.shadows.is_empty(), "visibility:hidden 不应生成阴影");
    assert!(prims.images.is_empty(), "visibility:hidden 不应生成图片");
    assert!(prims.fills.is_empty(), "visibility:hidden 不应生成填充");
}

/// 测试 paint_in_rect 正确绘制 box-shadow。
#[test]
fn test_paint_in_rect_with_box_shadow() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    };
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    // 脏区域完全覆盖节点
    let dirty_rect = Rect::new(0.0, 0.0, 200.0, 200.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

    assert_eq!(painter.primitives().shadows.len(), 1, "paint_in_rect 应生成 box-shadow");
}

// ── 新增测试：渐变渲染 ──────────────────────────────────

/// 测试 linear-gradient (to bottom) 生成 GradientPrimitive。
#[test]
fn test_paint_linear_gradient_to_bottom() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.gradients.len(), 1, "应生成 1 个渐变图元");
    let grad = &prims.gradients[0];
    match &grad.kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!(*x0, 100.0, "x0 应为水平中心 100");
            assert_eq!(*y0, 0.0, "y0 应为顶部 0");
            assert_eq!(*x1, 100.0, "x1 应为水平中心 100");
            assert_eq!(*y1, 100.0, "y1 应为底部 100");
        }
        other => panic!("期望 Linear 类型，实际 {:?}", other),
    }
}

/// 测试 linear-gradient (to right) 方向正确。
#[test]
fn test_paint_linear_gradient_to_right() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::ToRight,
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    match &grad.kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!(*x0, 0.0, "x0 应为左侧 0");
            assert_eq!(*y0, 50.0, "y0 应为垂直中心 50");
            assert_eq!(*x1, 200.0, "x1 应为右侧 200");
            assert_eq!(*y1, 50.0, "y1 应为垂直中心 50");
        }
        other => panic!("期望 Linear 类型，实际 {:?}", other),
    }
}

/// 测试 linear-gradient 角度方向。
#[test]
fn test_paint_linear_gradient_angle() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::Angle(90.0),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    assert!(
        matches!(grad.kind, GradientKind::Linear { .. }),
        "Angle(90) 应生成 Linear 类型"
    );
}

/// 测试 linear-gradient 色标正确传递。
#[test]
fn test_paint_linear_gradient_color_stops() {
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
                    color: ColorValue::Rgba(0, 255, 0, 255),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    assert_eq!(grad.stops.len(), 3, "应有 3 个色标");
    assert_eq!(grad.stops[0].color, Color::rgb(255, 0, 0), "第 1 个色标应为红色");
    assert_eq!(grad.stops[1].offset, 0.5, "第 2 个色标 offset 应为 0.5（均匀分布）");
}

/// 测试 linear-gradient 带百分比位置色标。
#[test]
fn test_paint_linear_gradient_with_position() {
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
                    position: Some(LengthValue::Percentage(25.0)),
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
            repeating: false,
        },
    ))];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    assert_eq!(grad.stops[0].offset, 0.25, "带百分比位置的色标 offset 应为 0.25");
}

/// 测试 radial-gradient 生成 GradientPrimitive。
#[test]
fn test_paint_radial_gradient_basic() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 使用 Px 作为 position，因为 length_to_f32 只处理 Px
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Radial(
        RadialGradient {
            shape: RadialShape::Circle,
            size: RadialSize::FarthestCorner,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 255, 255, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 0, 255),
                    position: None,
                },
            ],
            repeating: false,
        },
    ))];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.gradients.len(), 1, "应生成 1 个渐变图元");
    let grad = &prims.gradients[0];
    match &grad.kind {
        GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => {
            // 50% of 200 = 100 — 中心在容器中心
            assert!((*cx - 100.0).abs() < 0.1, "cx 应约为 100，实际 {}", cx);
            assert!((*cy - 100.0).abs() < 0.1, "cy 应约为 100，实际 {}", cy);
            assert_eq!(*inner_radius, 0.0, "inner_radius 应为 0");
            assert!(*outer_radius > 0.0, "outer_radius 应大于 0");
        }
        other => panic!("期望 Radial 类型，实际 {:?}", other),
    }
}

/// 测试 radial-gradient closest-side 尺寸计算。
#[test]
fn test_paint_radial_gradient_closest_side() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Radial(
        RadialGradient {
            shape: RadialShape::Circle,
            size: RadialSize::ClosestSide,
            position_x: LengthValue::Px(100.0),
            position_y: LengthValue::Px(100.0),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    if let GradientKind::Radial { cx, outer_radius, .. } = &grad.kind {
        // cx = 0 + 100/100 * 200 = 200（at right edge）
        // closest-side from (200,200) in (0,0,200,200): min(200-0, 200-200, 200-0, 200-200) = 0
        // 但 outer_radius 有 .max(0.01) 保底
        assert!(*outer_radius >= 0.01, "outer_radius 应 >= 0.01");
        let _ = cx;
    }
}

/// 测试 radial-gradient 自定义位置（百分比）。
#[test]
fn test_paint_radial_gradient_custom_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    // 使用 Percentage 作为 position：25% of 200 = 50, 75% of 100 = 75
    // cx = rect.left() + 50 = 60, cy = rect.top() + 75 = 95
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Radial(
        RadialGradient {
            shape: RadialShape::Circle,
            size: RadialSize::FarthestCorner,
            position_x: LengthValue::Percentage(25.0),
            position_y: LengthValue::Percentage(75.0),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    if let GradientKind::Radial { cx, cy, .. } = &grad.kind {
        assert_eq!(*cx, 10.0 + 25.0 / 100.0 * 200.0, "cx 应为 rect.left + 25% * width");
        assert_eq!(*cy, 20.0 + 75.0 / 100.0 * 100.0, "cy 应为 rect.top + 75% * height");
    }
}

/// 测试 conic-gradient 生成渐变图元。
#[test]
fn test_paint_conic_gradient_no_primitive() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Conic(
        ConicGradient {
            from_angle: 0.0,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().gradients.is_empty(),
        "conic-gradient 应生成渐变图元"
    );
}

/// 测试渐变与背景色同时生成。
#[test]
fn test_paint_gradient_with_background_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "应生成 1 个背景色填充");
    assert_eq!(prims.gradients.len(), 1, "应生成 1 个渐变图元");
}

/// 测试 BackgroundImageComputedValue::None 不生成渐变。
#[test]
fn test_paint_gradient_none_no_output() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::None];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().gradients.is_empty(),
        "BackgroundImageComputedValue::None 不应生成渐变图元"
    );
}

/// 测试 linear-gradient (to top left) 对角方向。
#[test]
fn test_paint_linear_gradient_to_top_left() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::ToTopLeft,
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    match &grad.kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!(*x0, 200.0, "x0 应为 rect.right = 200");
            assert_eq!(*y0, 100.0, "y0 应为 rect.bottom = 100");
            assert_eq!(*x1, 0.0, "x1 应为 rect.left = 0");
            assert_eq!(*y1, 0.0, "y1 应为 rect.top = 0");
        }
        other => panic!("期望 Linear 类型，实际 {:?}", other),
    }
}

/// 测试 linear-gradient (to bottom right) 对角方向。
#[test]
fn test_paint_linear_gradient_to_bottom_right() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::ToBottomRight,
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    match &grad.kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!(*x0, 0.0, "x0 应为 rect.left = 0");
            assert_eq!(*y0, 0.0, "y0 应为 rect.top = 0");
            assert_eq!(*x1, 200.0, "x1 应为 rect.right = 200");
            assert_eq!(*y1, 100.0, "y1 应为 rect.bottom = 100");
        }
        other => panic!("期望 Linear 类型，实际 {:?}", other),
    }
}

/// 测试 radial-gradient length size。
#[test]
fn test_paint_radial_gradient_length_size() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 200.0, 200.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Radial(
        RadialGradient {
            shape: RadialShape::Circle,
            size: RadialSize::Length(LengthValue::Px(50.0)),
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
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
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    if let GradientKind::Radial { outer_radius, .. } = &grad.kind {
        assert_eq!(*outer_radius, 50.0, "Length(Px(50)) 的 outer_radius 应为 50");
    }
}

/// 测试单个色标的 linear-gradient。
#[test]
fn test_paint_linear_gradient_single_stop() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            }],
            repeating: false,
        },
    ))];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    assert_eq!(grad.stops.len(), 1, "应有 1 个色标");
    assert_eq!(grad.stops[0].offset, 0.0, "单个色标 position=None 时 offset 应为 0.0");
}

// ── 新增测试：opacity + text-decoration + text-transform ──

/// 测试 opacity=0.5 降低 fill alpha。
#[test]
fn test_paint_opacity_halves_alpha() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.opacity = 0.5;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().fills[0].color.a,
        128,
        "opacity=0.5 应将 fill alpha 从 255 降到 128"
    );
}

/// 测试 opacity=1.0 不影响 alpha。
#[test]
fn test_paint_opacity_full() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.opacity = 1.0;
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().fills[0].color.a,
        255,
        "opacity=1.0 不应改变 fill alpha"
    );
}

/// 测试 opacity=0.0 使 fill 完全透明。
#[test]
fn test_paint_opacity_zero() {
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
        "opacity=0.0 应将 fill alpha 设为 0"
    );
}

/// 测试 opacity 影响 glyph alpha。
#[test]
fn test_paint_opacity_affects_glyphs() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.opacity = 0.5;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().glyphs[0].color.a,
        128,
        "opacity=0.5 应将 glyph alpha 从 255 降到 128"
    );
}

/// 测试 opacity 影响 shadow alpha。
#[test]
fn test_paint_opacity_affects_shadow() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
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

    assert_eq!(
        painter.primitives().shadows[0].color.a,
        128,
        "opacity=0.5 应将 shadow alpha 从 255 降到 128"
    );
}

/// 测试 opacity 不影响无样式节点。
#[test]
fn test_paint_opacity_no_style() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let mut painter = Painter::new();
    let styles = HashMap::new();
    painter.paint(&layout, &styles, None);
    assert!(painter.primitives().is_empty(), "无样式节点不应产生任何图元");
}

/// 测试 text-decoration: underline 生成填充图元。
#[test]
fn test_paint_text_decoration_underline() {
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
    assert!(prims.fills.len() >= 1, "underline 应生成至少 1 个装饰填充图元");
    assert!(prims.glyphs.len() >= 1, "underline 应同时生成至少 1 个 glyph");
}

/// 测试 text-decoration: overline 生成填充图元。
#[test]
fn test_paint_text_decoration_overline() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::Overline;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().fills.len() >= 1,
        "overline 应生成至少 1 个装饰填充图元"
    );
}

/// 测试 text-decoration: line-through 生成填充图元。
#[test]
fn test_paint_text_decoration_line_through() {
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

    assert!(
        painter.primitives().fills.len() >= 1,
        "line-through 应生成至少 1 个装饰填充图元"
    );
}

/// 测试 text-decoration: none 不生成填充图元。
#[test]
fn test_paint_text_decoration_none() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_decoration_line = TextDecorationLineValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 0, "text-decoration: none 不应生成额外填充图元");
    assert_eq!(prims.glyphs.len(), 1, "应有 1 个 glyph");
}
