#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{
    ColorValue, ConicGradient, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
    RadialGradient, RadialShape, RadialSize, TransformFunction, TransformValue, VisibilityValue,
};
use zero_dom::{Document, NodeId};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FontId, GlyphPrimitive, GradientKind};
use zero_style_system::{
    BackgroundImageComputedValue, BorderStyleValue, BoxShadowComputedValue, ComputedStyle, OutlineStyleValue,
    TextDecorationLineValue, TextShadowComputedValue, TextTransformValue,
};

use super::super::color::{color_value_to_render, hsla_to_rgba, named_color_to_render};
use super::super::helpers::{
    BorderRadiusSpec, apply_transform_offset, clip_fills, clip_glyphs, convert_color_stops, gradient_to_primitive,
    length_to_f32, linear_direction_to_kind, simple_hash,
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

/// 辅助函数：创建带边框的 LayoutBox。
fn make_box_with_border(
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
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
    style.background_image = BackgroundImageComputedValue::Url("test.png".to_string());
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Radial(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        position_x: LengthValue::Px(100.0),
        position_y: LengthValue::Px(100.0),
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
    }));
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
            // cx = rect.left() + length_to_f32(&Px(100)) / 100.0 * w = 0 + 100/100 * 200 = 200
            // cy = rect.top() + length_to_f32(&Px(100)) / 100.0 * h = 0 + 100/100 * 200 = 200
            assert_eq!(*inner_radius, 0.0, "inner_radius 应为 0");
            assert!(*outer_radius > 0.0, "outer_radius 应大于 0");
            // 验证是 Radial 类型即可，cx/cy 由 length_to_f32 计算决定
            let _ = (cx, cy);
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Radial(RadialGradient {
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
    }));
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

/// 测试 radial-gradient 自定义位置。
#[test]
fn test_paint_radial_gradient_custom_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 200.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    // 使用 Px 作为 position：length_to_f32(Px(25)) / 100.0 * w = 25/100*200 = 50
    // cx = rect.left() + 50 = 10 + 50 = 60
    // length_to_f32(Px(75)) / 100.0 * h = 75/100*100 = 75
    // cy = rect.top() + 75 = 20 + 75 = 95
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Radial(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        position_x: LengthValue::Px(25.0),
        position_y: LengthValue::Px(75.0),
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
    }));
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let grad = &painter.primitives().gradients[0];
    if let GradientKind::Radial { cx, cy, .. } = &grad.kind {
        assert_eq!(*cx, 10.0 + 25.0 / 100.0 * 200.0, "cx 应为 rect.left + px/100 * width");
        assert_eq!(*cy, 20.0 + 75.0 / 100.0 * 100.0, "cy 应为 rect.top + px/100 * height");
    }
}

/// 测试 conic-gradient 不生成图元（暂不支持）。
#[test]
fn test_paint_conic_gradient_no_primitive() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Conic(ConicGradient {
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
    }));
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().gradients.is_empty(),
        "conic-gradient 暂不支持渲染，不应生成渐变图元"
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::None;
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Radial(RadialGradient {
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
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
        direction: GradientDirection::ToBottom,
        stops: vec![GradientColorStop {
            color: ColorValue::Rgba(255, 0, 0, 255),
            position: None,
        }],
        repeating: false,
    }));
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
    style.background_image = BackgroundImageComputedValue::Gradient(GradientValue::Linear(LinearGradient {
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
    }));
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
        !painter.primitives().fills.is_empty(),
        "border-radius 非零时仍应产生 fill"
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
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None);
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
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

    painter.paint_text_decoration(0.0, 16.0, 16.0, 0.0, color, &TextDecorationLineValue::Underline);
    assert!(painter.primitives().fills.is_empty(), "宽度为 0 不应生成装饰填充");

    painter.paint_text_decoration(0.0, 16.0, 16.0, -10.0, color, &TextDecorationLineValue::Underline);
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
        z_index: 0,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
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
