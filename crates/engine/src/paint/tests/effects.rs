#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue, VisibilityValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FontId, GlyphPrimitive};
use zero_style_system::{BorderStyleValue, ComputedStyle, OutlineStyleValue};

use super::super::color::{color_value_to_render, hsla_to_rgba, named_color_to_render};
use super::super::helpers::{apply_transform_offset, clip_fills, clip_glyphs, length_to_f32};
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}
// ── 边界条件测试 ──────────────────────────────────────────

/// 测试 HSL 色相 120（绿色）。
#[test]
fn test_hsla_green_120() {
    let color = hsla_to_rgba(120.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 色相 240（蓝色）。
#[test]
fn test_hsla_blue_240() {
    let color = hsla_to_rgba(240.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 255);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 饱和度 0% 和亮度 0%（黑色）。
#[test]
fn test_hsla_zero_saturation_zero_lightness() {
    let color = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 饱和度 0% 和亮度 100%（白色）。
#[test]
fn test_hsla_zero_saturation_full_lightness() {
    let color = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 255);
    assert_eq!(color.a, 255);
}

/// 测试 border-style: hidden 不产生填充。
#[test]
fn test_border_style_hidden_no_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 5.0, 5.0, 5.0, 5.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Hidden;
    style.border_right_style = BorderStyleValue::Hidden;
    style.border_bottom_style = BorderStyleValue::Hidden;
    style.border_left_style = BorderStyleValue::Hidden;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // border-style: hidden 与 none 行为一致，不绘制边框
    assert_eq!(
        painter.primitives().fills.len(),
        0,
        "hidden border should produce no fills"
    );
}

/// 测试 zero-width border with solid style 不产生填充。
#[test]
fn test_border_zero_width_solid_style_no_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // border_top = 0.0, style = Solid => no fill for top border
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 0.0, 5.0, 5.0, 5.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 只有 3 个边框填充（top border 宽度为 0，不绘制）
    assert_eq!(
        painter.primitives().fills.len(),
        3,
        "zero-width top border should produce no fill"
    );
}

/// 测试 named_color_to_render: lime, purple, maroon, olive, aqua, fuchsia, grey。
#[test]
fn test_named_colors_lime_purple_maroon() {
    assert_eq!(named_color_to_render("lime"), Color::rgb(0, 255, 0));
    assert_eq!(named_color_to_render("purple"), Color::rgb(128, 0, 128));
    assert_eq!(named_color_to_render("maroon"), Color::rgb(128, 0, 0));
    assert_eq!(named_color_to_render("olive"), Color::rgb(128, 128, 0));
    assert_eq!(named_color_to_render("aqua"), Color::rgb(0, 255, 255));
    assert_eq!(named_color_to_render("fuchsia"), Color::rgb(255, 0, 255));
    assert_eq!(named_color_to_render("grey"), Color::rgb(128, 128, 128));
}

/// 测试 outline_width = 0 不产生填充。
#[test]
fn test_outline_zero_width_no_fill() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_width = LengthValue::Px(0.0);
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().is_empty(),
        "zero-width outline should produce no fills"
    );
}

/// R2121：outline 不应用于 table-column / table-column-group（CSS2 §17.4 此二者不生成盒）。
/// driving cluster：outline-applies-to-005/006（4 outline 属性 × 2 display = 8 案）。
/// 非 0 宽 outline 在这两种 display 下应完全不绘制（同 R2108 margin 抑制谱系）。
#[test]
fn test_outline_suppressed_for_table_column_types() {
    use zero_style_system::property::types::DisplayValue;

    for display in [DisplayValue::TableColumn, DisplayValue::TableColumnGroup] {
        let display_dbg = format!("{display:?}");
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.display = display;
        // 非 0 宽、Solid、红 —— 正常 display 会绘制 4 条 fill。
        style.outline_width = LengthValue::Px(10.0);
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        assert!(
            painter.primitives().is_empty(),
            "outline must not apply to {display_dbg} (CSS2 §17.4 no box); got {} primitives",
            painter.primitives().len()
        );
    }
}

/// 测试 paint_text with non-Px font size (Em) — early return, no glyph。
#[test]
fn test_paint_text_em_font_size_no_glyph() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Em(1.0);
    style.color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &styles[&elem], None, None);
    assert!(
        painter.primitives().glyphs.is_empty(),
        "Em font size should produce no glyph"
    );
}

/// 测试 paint_in_rect: parent outside dirty rect, child inside — parent culling should skip subtree。
#[test]
fn test_paint_in_rect_parent_outside_child_inside_skipped() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // child 在 (300, 300) 处
    let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 50.0);
    // parent 在 (300, 300) 处，完全在脏区域外
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 300.0,
        y: 300.0,
        width: 100.0,
        height: 100.0,
        content_x: 300.0,
        content_y: 300.0,
        content_width: 100.0,
        content_height: 100.0,
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
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    // 脏区域在 (0, 0) 处，parent 在 (300, 300) 完全不在脏区域内
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&parent_box, &styles, &dirty_rect, None);

    // parent 完全在脏区域外，整个子树（包括 child）被跳过
    assert!(
        painter.primitives().is_empty(),
        "parent outside dirty rect should skip entire subtree including child"
    );
}

/// 测试 zero-offset translate 不改变位置。
#[test]
fn test_transform_zero_translate_no_offset() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::Translate(0.0, 0.0)]);
    let (dx, dy) = apply_transform_offset(&style, 10.0, 20.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

// ── 边界条件测试：clip_fills / clip_glyphs / color ──────

/// clip_fills 部分重叠：fill 矩形与 clip 矩形部分重叠时，缩小到交集。
#[test]
fn test_clip_fills_partial_overlap() {
    use zero_render_foundation::primitive::FillPrimitive;

    // clip rect: (0, 0, 100, 100)
    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    // fill rect: (50, 50, 100, 100) → 与 clip 交集为 (50, 50, 50, 50)
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(50.0, 50.0, 100.0, 100.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.origin.x, 50.0);
    assert_eq!(fills[0].rect.origin.y, 50.0);
    assert_eq!(fills[0].rect.size.width, 50.0);
    assert_eq!(fills[0].rect.size.height, 50.0);
}

/// clip_fills 完全在外侧：左/右/上/下四个方向均被清零。
#[test]
fn test_clip_fills_outside_each_side() {
    use zero_render_foundation::primitive::FillPrimitive;

    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);

    // 左侧完全在 clip 外
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(-150.0, 0.0, 100.0, 100.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.size.width, 0.0);
    assert_eq!(fills[0].rect.size.height, 0.0);

    // 右侧完全在 clip 外
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(200.0, 0.0, 100.0, 100.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.size.width, 0.0);
    assert_eq!(fills[0].rect.size.height, 0.0);

    // 上侧完全在 clip 外
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(0.0, -200.0, 100.0, 100.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.size.width, 0.0);
    assert_eq!(fills[0].rect.size.height, 0.0);

    // 下侧完全在 clip 外
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 200.0, 100.0, 100.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.size.width, 0.0);
    assert_eq!(fills[0].rect.size.height, 0.0);
}

/// clip_fills start index > 0：只有 start 之后的 fill 被裁剪。
#[test]
fn test_clip_fills_start_index() {
    use zero_render_foundation::primitive::FillPrimitive;

    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    // 第一个 fill 在 clip 内（不受影响），第二个完全在 clip 外
    let mut fills = vec![
        FillPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::BLACK,
        },
        FillPrimitive {
            rect: Rect::new(200.0, 200.0, 50.0, 50.0),
            color: Color::BLACK,
        },
    ];
    clip_fills(&mut fills, 1, &clip);
    // 第一个 fill 不应被裁剪
    assert_eq!(fills[0].rect.size.width, 50.0);
    assert_eq!(fills[0].rect.size.height, 50.0);
    // 第二个 fill 应被清零
    assert_eq!(fills[1].rect.size.width, 0.0);
    assert_eq!(fills[1].rect.size.height, 0.0);
}

/// clip_fills 空 slice 不 panic。
#[test]
fn test_clip_fills_empty_slice() {
    use zero_render_foundation::primitive::FillPrimitive;

    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut fills: Vec<FillPrimitive> = vec![];
    clip_fills(&mut fills, 0, &clip);
    // 应正常返回，不 panic
}

/// clip_fills fill rect 完全匹配 clip rect → 不变。
#[test]
fn test_clip_fills_exact_match() {
    use zero_render_foundation::primitive::FillPrimitive;

    let clip = Rect::new(10.0, 20.0, 80.0, 60.0);
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(10.0, 20.0, 80.0, 60.0),
        color: Color::BLACK,
    }];
    clip_fills(&mut fills, 0, &clip);
    assert_eq!(fills[0].rect.origin.x, 10.0);
    assert_eq!(fills[0].rect.origin.y, 20.0);
    assert_eq!(fills[0].rect.size.width, 80.0);
    assert_eq!(fills[0].rect.size.height, 60.0);
}

/// clip_glyphs 字形在 clip 外侧（左/右/上/下）→ glyph_id 设为 0。
#[test]
fn test_clip_glyphs_outside_rejection() {
    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);

    // 左侧：glyph 在 (-50, 10)，font_size=16 → right = -34，在 clip 左侧
    let mut glyphs = vec![GlyphPrimitive {
        x: -50.0,
        y: 10.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 42,
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    clip_glyphs(&mut glyphs, 0, &clip);
    assert_eq!(glyphs[0].glyph_id, 0);

    // 右侧：glyph 在 (150, 10)，x >= clip right (100)
    let mut glyphs = vec![GlyphPrimitive {
        x: 150.0,
        y: 10.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 42,
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    clip_glyphs(&mut glyphs, 0, &clip);
    assert_eq!(glyphs[0].glyph_id, 0);

    // 上侧：glyph 在 (10, -50)，font_size=16 → bottom = -34
    let mut glyphs = vec![GlyphPrimitive {
        x: 10.0,
        y: -50.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 42,
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    clip_glyphs(&mut glyphs, 0, &clip);
    assert_eq!(glyphs[0].glyph_id, 0);

    // 下侧：glyph 在 (10, 150)，y >= clip bottom (100)
    let mut glyphs = vec![GlyphPrimitive {
        x: 10.0,
        y: 150.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 42,
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    clip_glyphs(&mut glyphs, 0, &clip);
    assert_eq!(glyphs[0].glyph_id, 0);
}

/// clip_glyphs start > 0：只有 start 之后的 glyph 被裁剪。
#[test]
fn test_clip_glyphs_start_index() {
    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut glyphs = vec![
        // glyph[0] 在 clip 外（不应被裁剪，因为 start=1）
        GlyphPrimitive {
            x: 200.0,
            y: 200.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 10,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
        // glyph[1] 在 clip 外（应被裁剪）
        GlyphPrimitive {
            x: 200.0,
            y: 200.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 20,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
    ];
    clip_glyphs(&mut glyphs, 1, &clip);
    // 第一个 glyph 不受影响
    assert_eq!(glyphs[0].glyph_id, 10);
    // 第二个 glyph 被清零
    assert_eq!(glyphs[1].glyph_id, 0);
}

/// clip_glyphs 字形在 clip 内 → 不被裁剪。
#[test]
fn test_clip_glyphs_inside_not_clipped() {
    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut glyphs = vec![GlyphPrimitive {
        x: 10.0,
        y: 10.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    clip_glyphs(&mut glyphs, 0, &clip);
    assert_eq!(glyphs[0].glyph_id, 65);
    assert_eq!(glyphs[0].font_size, 16.0);
}

/// clip_glyphs 空 slice 不 panic。
#[test]
fn test_clip_glyphs_empty_slice() {
    let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut glyphs: Vec<GlyphPrimitive> = vec![];
    clip_glyphs(&mut glyphs, 0, &clip);
    // 应正常返回，不 panic
}

/// color_value_to_render CurrentColor → rgba(0,0,0,255)。
#[test]
fn test_color_value_to_render_current_color() {
    let color = color_value_to_render(&ColorValue::CurrentColor);
    assert_eq!(color, Color::rgba(0, 0, 0, 255));
}

/// hsla_to_rgba(300, 100, 50, 1.0) → 品红区域，验证 RGB 值。
/// hue 300: h'=5.0, 进入 _ => (c, 0.0, x) 分支
/// c=1.0, x=1.0*(1.0-|5.0%2-1.0|)=1.0*(1.0-0.0)=1.0, m=0.0
/// r=255, g=0, b=255
#[test]
fn test_hsla_hue_300_magenta_region() {
    let color = hsla_to_rgba(300.0, 100.0, 50.0, 1.0);
    let Color { r, g, b, a } = color;
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 255);
    assert_eq!(a, 255);
}

/// hsla_to_rgba(330, 100, 50, 1.0) → 验证结果。
/// hue 330: h'=5.5, 进入 _ => (c, 0.0, x)
/// c=1.0, x=1.0*(1.0-|5.5%2-1.0|)=1.0*(1.0-|1.5-1.0|)=1.0*(1.0-0.5)=0.5, m=0.0
/// r=255, g=0, b=128
#[test]
fn test_hsla_hue_330_region() {
    let color = hsla_to_rgba(330.0, 100.0, 50.0, 1.0);
    let Color { r, g, b, a } = color;
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 128);
    assert_eq!(a, 255);
}

/// length_to_f32 对非 Px 单位返回 0.0。
#[test]
fn test_length_to_f32_non_px() {
    assert_eq!(length_to_f32(&LengthValue::Em(2.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Rem(1.5)), 0.0);
}

/// named_color_to_render 扩展颜色测试。
#[test]
fn test_named_color_extended() {
    assert_eq!(named_color_to_render("cyan"), Color::rgb(0, 255, 255));
    assert_eq!(named_color_to_render("aqua"), Color::rgb(0, 255, 255));
    assert_eq!(named_color_to_render("magenta"), Color::rgb(255, 0, 255));
    assert_eq!(named_color_to_render("fuchsia"), Color::rgb(255, 0, 255));
    assert_eq!(named_color_to_render("silver"), Color::rgb(192, 192, 192));
    assert_eq!(named_color_to_render("maroon"), Color::rgb(128, 0, 0));
    assert_eq!(named_color_to_render("olive"), Color::rgb(128, 128, 0));
    assert_eq!(named_color_to_render("lime"), Color::rgb(0, 255, 0));
    assert_eq!(named_color_to_render("purple"), Color::rgb(128, 0, 128));
    assert_eq!(named_color_to_render("teal"), Color::rgb(0, 128, 128));
    assert_eq!(named_color_to_render("navy"), Color::rgb(0, 0, 128));
    assert_eq!(named_color_to_render("orange"), Color::rgb(255, 165, 0));
    assert_eq!(named_color_to_render("pink"), Color::rgb(255, 192, 203));
    assert_eq!(named_color_to_render("brown"), Color::rgb(165, 42, 42));
}

/// named_color_to_render 未知颜色名 → 回退为 rgb(0,0,0)。
#[test]
fn test_named_color_unknown() {
    assert_eq!(named_color_to_render("nonexistent"), Color::rgb(0, 0, 0));
    assert_eq!(named_color_to_render("chartreuse"), Color::rgb(0, 0, 0));
    assert_eq!(named_color_to_render(""), Color::rgb(0, 0, 0));
}

/// 测试子元素 visibility:visible 覆盖父元素 visibility:hidden。
///
/// 父元素设置为 visibility:hidden，子元素设置为 visibility:visible。
/// 父元素不应绘制自身背景，但子元素应正常绘制。
#[test]
fn test_painter_child_visible_overrides_parent_hidden() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
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
    parent_style.visibility = VisibilityValue::Hidden;
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
    child_style.visibility = VisibilityValue::Visible;
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 父元素 visibility:hidden → 不绘制自身背景
    // 子元素 visibility:visible → 正常绘制
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(100, 100, 255));
}

/// 测试 LayoutBox 的 node_id=None 但传入 doc=Some 时退化为 fallback glyph。
///
/// 当布局盒没有关联 DOM 节点（node_id=None），即使传入了 Document，
/// paint_text 也无法使用 InlineFormattingContext，应退化为 glyph_id=0 的占位 glyph。
#[test]
fn test_paint_text_doc_some_node_id_none_fallback() {
    let doc = zero_dom::Document::new();

    // node_id=None 的布局盒
    let layout = make_box(None, 0.0, 0.0, 200.0, 30.0);

    let style = ComputedStyle {
        color: ColorValue::Rgba(0, 0, 0, 255),
        font_size: LengthValue::Px(16.0),
        ..ComputedStyle::default()
    };

    let mut painter = Painter::new();
    painter.paint_text(&layout, 0.0, 0.0, &style, Some(&doc), None);

    // node_id=None → 无法使用 InlineFormattingContext → 走 fallback 路径
    assert_eq!(painter.primitives().glyphs.len(), 1);
    let glyph = &painter.primitives().glyphs[0];
    assert_eq!(glyph.glyph_id, 0, "fallback glyph 应为 glyph_id=0");
    assert_eq!(glyph.font_size, 16.0);
}

/// 测试 visibility:collapse 在非表格元素上表现为 hidden。
///
/// 根据 CSS 规范，visibility:collapse 在非表格行/列元素上
/// 应与 visibility:hidden 行为一致，元素不绘制但保留布局空间。
#[test]
fn test_painter_visibility_collapse_acts_as_hidden() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.visibility = VisibilityValue::Collapse;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // visibility:collapse 应阻止元素绘制（与 hidden 行为一致）
    assert!(
        painter.primitives().fills.is_empty(),
        "visibility:collapse 应阻止元素绘制"
    );
    assert!(
        painter.primitives().glyphs.is_empty(),
        "visibility:collapse 应阻止 glyph 生成"
    );
}

/// 测试 paint_in_rect 对 visibility:hidden 的节点不生成任何图元。
///
/// 增量绘制路径（paint_node_in_rect）同样应遵守 visibility 规则，
/// 隐藏元素不应产生任何填充或 glyph 图元。
#[test]
fn test_paint_in_rect_visibility_hidden_skips_node() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 节点与脏区域相交
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.visibility = VisibilityValue::Hidden;
    styles.insert(elem, style);

    let dirty_rect = Rect::new(0.0, 0.0, 200.0, 200.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

    // visibility:hidden → 节点不应产生任何图元
    assert!(
        painter.primitives().is_empty(),
        "visibility:hidden 在 paint_in_rect 中应跳过节点绘制"
    );
}

/// 测试 paint_in_rect 中 overflow:hidden 裁剪子节点超出父内容区域的部分。
///
/// 父节点设置 overflow_x/overflow_y 为 OverflowClip::Hidden，
/// 子节点（200x200）超出父内容区域（100x100）。
/// paint_in_rect 以覆盖父节点的脏区域调用后，
/// 子节点的填充矩形应被裁剪到 100x100 或更小。
#[test]
fn test_paint_in_rect_overflow_hidden_clips_children() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // 子节点 200x200 超出父节点的 100x100 内容区域
    let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 100.0,
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
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    // 脏区域覆盖整个父节点
    let dirty_rect = Rect::new(0.0, 0.0, 200.0, 200.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&parent_box, &styles, &dirty_rect, None);

    // 子节点填充应被裁剪到父节点的 100x100 内容区域
    let fill = &painter.primitives().fills[0];
    assert!(
        fill.rect.size.width <= 100.0,
        "子节点宽度应被裁剪到 100 或更小，实际 {}",
        fill.rect.size.width
    );
    assert!(
        fill.rect.size.height <= 100.0,
        "子节点高度应被裁剪到 100 或更小，实际 {}",
        fill.rect.size.height
    );
}

/// 测试 paint_in_rect 中与脏区域部分相交的节点会被绘制。
///
/// 节点位于 (50, 50)，大小 100x100，脏区域为 (0, 0, 100, 100)。
/// 节点与脏区域部分相交（重叠区域为 [50,100] x [50,100]），
/// 因此应生成图元。
#[test]
fn test_paint_in_rect_partially_intersecting_node_drawn() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 节点在 (50, 50) 大小 100x100，与脏区域部分重叠
    let layout = make_box(Some(elem), 50.0, 50.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    styles.insert(elem, style);

    // 脏区域 (0, 0, 100, 100)，节点区域 (50, 50, 100, 100)
    // 交集为 (50, 50) 到 (100, 100)，部分相交
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&layout, &styles, &dirty_rect, None);

    // 部分相交的节点应产生填充图元
    assert_eq!(painter.primitives().fills.len(), 1, "部分相交的节点应被绘制");
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(0, 128, 255));
}

/// 测试 paint_in_rect 中兄弟节点独立性：只有与脏区域相交的节点被绘制。
///
/// 两个兄弟节点，一个与脏区域相交（在 (10, 10) 大小 50x50），
/// 另一个完全在脏区域外（在 (300, 300) 大小 50x50）。
/// 只有相交的兄弟应产生图元，不相交的兄弟不应产生任何图元。
#[test]
fn test_paint_in_rect_siblings_independent() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let sibling1 = doc.create_element("span");
    let sibling2 = doc.create_element("span");

    // 兄弟1：在脏区域内 (10, 10, 50x50)
    let s1_box = make_box(Some(sibling1), 10.0, 10.0, 50.0, 50.0);
    // 兄弟2：完全在脏区域外 (300, 300, 50x50)
    let s2_box = make_box(Some(sibling2), 300.0, 300.0, 50.0, 50.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 400.0,
        content_height: 400.0,
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
        children: vec![s1_box, s2_box],
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
    let mut s1_style = ComputedStyle::default();
    s1_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(sibling1, s1_style);

    let mut s2_style = ComputedStyle::default();
    s2_style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    styles.insert(sibling2, s2_style);

    // 脏区域只覆盖 (0, 0, 100, 100)
    let dirty_rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&parent_box, &styles, &dirty_rect, None);

    // 只有兄弟1（相交）应产生填充，兄弟2（不相交）不应产生
    assert_eq!(painter.primitives().fills.len(), 1, "只有与脏区域相交的兄弟应被绘制");
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(255, 0, 0));
}

// ── 边界条件测试：overflow + dirty rect 交互、HSLA 极值、零宽度文本、多层裁剪 ──

/// 测试 paint_in_rect 中 overflow:hidden 父节点与脏区域部分重叠时子节点被裁剪。
///
/// 父节点（overflow:hidden）位于 (0,0) 大小 200x200，内容区域 200x200。
/// 子节点位于 (140,140) 大小 200x200，超出父内容区域。
/// 脏区域为 (0,0,200,200) 完全覆盖父节点，与子节点有交集。
/// 子节点填充被 overflow:hidden 裁剪到父内容区域 (0,0,200,200)，
/// 裁剪后子节点可见区域为 (140,140) 大小 60x60（200-140）。
#[test]
fn test_paint_in_rect_overflow_hidden_clips_child_partially_intersecting() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    // 子节点从 (140,140) 开始 200x200，右下角超出父内容区域
    let child_box = make_box(Some(child), 140.0, 140.0, 200.0, 200.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 200.0,
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
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    // 脏区域覆盖父节点 (0,0,200,200)
    // 子节点绝对坐标 (140,140,200,200) → node_right=340, node_bottom=340
    // 140 < dirty_right(200) 且 140 < dirty_bottom(200) → 不被剔除
    // 子节点进入绘制流程，overflow:hidden 裁剪到父内容 (0,0,200,200)
    // 裁剪后：origin=(140,140), size=(60,60)（200-140=60）
    let dirty_rect = Rect::new(0.0, 0.0, 200.0, 200.0);

    let mut painter = Painter::new();
    painter.paint_in_rect(&parent_box, &styles, &dirty_rect, None);

    assert!(!painter.primitives().fills.is_empty(), "应产生子节点填充");
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 140.0, "子节点裁剪后 x 应为 140");
    assert_eq!(fill.rect.origin.y, 140.0, "子节点裁剪后 y 应为 140");
    assert_eq!(fill.rect.size.width, 60.0, "子节点宽度应被裁剪到 60（200-140）");
    assert_eq!(fill.rect.size.height, 60.0, "子节点高度应被裁剪到 60（200-140）");
}

/// 测试 HSLA 零饱和度零亮度（纯黑）和零饱和度满亮度（纯白）。
///
/// hsla(0, 0, 0, 1.0) → s=0, l=0 → c=0, m=0 → RGB(0,0,0) 纯黑
/// hsla(0, 0, 100, 1.0) → s=0, l=1 → c=0, m=1 → RGB(255,255,255) 纯白
#[test]
fn test_hsla_zero_saturation_and_lightness() {
    // 纯黑：饱和度 0，亮度 0
    let black = hsla_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(black.r, 0, "HSLA 黑色 R 应为 0");
    assert_eq!(black.g, 0, "HSLA 黑色 G 应为 0");
    assert_eq!(black.b, 0, "HSLA 黑色 B 应为 0");
    assert_eq!(black.a, 255, "HSLA 黑色 A 应为 255");

    // 纯白：饱和度 0，亮度 100
    let white = hsla_to_rgba(0.0, 0.0, 100.0, 1.0);
    assert_eq!(white.r, 255, "HSLA 白色 R 应为 255");
    assert_eq!(white.g, 255, "HSLA 白色 G 应为 255");
    assert_eq!(white.b, 255, "HSLA 白色 B 应为 255");
    assert_eq!(white.a, 255, "HSLA 白色 A 应为 255");

    // 验证通过 ColorValue::Hsla 间接调用也正确
    let black_cv = color_value_to_render(&ColorValue::Hsla(0.0, 0.0, 0.0, 1.0));
    assert_eq!(black_cv.r, 0);
    assert_eq!(black_cv.g, 0);
    assert_eq!(black_cv.b, 0);

    let white_cv = color_value_to_render(&ColorValue::Hsla(0.0, 0.0, 100.0, 1.0));
    assert_eq!(white_cv.r, 255);
    assert_eq!(white_cv.g, 255);
    assert_eq!(white_cv.b, 255);
}

/// 测试 width=0 的 LayoutBox 带文本内容不 panic。
///
/// 当布局盒宽度为零时，paint_text 应正常返回而不崩溃。
/// 零宽度容器的 InlineFormattingContext 应安全处理，
/// 退化为 fallback glyph 或直接返回。
#[test]
fn test_paint_text_zero_width_no_panic() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");

    // 零宽度布局盒，但带有文本样式
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 10.0,
        y: 20.0,
        width: 0.0,
        height: 50.0,
        content_x: 10.0,
        content_y: 20.0,
        content_width: 0.0,
        content_height: 50.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    styles.insert(elem, style);

    // 调用 paint 不应 panic
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 零宽度不应产生有效的 glyph（content_width=0 可能导致问题）
    // 关键是整个过程不 panic
    let prims = painter.primitives();
    // 可能产生 0 个 glyph（因为容器宽度为 0）或 1 个 fallback glyph
    assert!(
        prims.glyphs.len() <= 1,
        "零宽度容器应产生 0 或 1 个 glyph，实际 {}",
        prims.glyphs.len()
    );

    // 也测试通过 paint_text 直接调用不 panic
    let mut painter2 = Painter::new();
    painter2.paint_text(&layout, 10.0, 20.0, &styles[&elem], None, None);
    // 不 panic 即通过
}

/// 测试三层 overflow:hidden 嵌套裁剪：最内层子节点被所有祖先裁剪。
///
/// 结构：outer(overflow:hidden, 80x80) > middle(overflow:hidden, 50x50) > inner(overflow:hidden, 30x30) > child(200x200)
/// child(200x200) 被 inner 裁剪到 30x30，
/// inner 的结果(30x30) 在 middle 内不需要进一步裁剪，
/// middle 的结果在 outer 内也不需要进一步裁剪。
/// 最终 child 填充应为 30x30。
#[test]
fn test_multiple_overflow_hidden_nested() {
    let mut doc = zero_dom::Document::new();
    let outer = doc.create_element("div");
    let middle = doc.create_element("div");
    let inner = doc.create_element("div");
    let child = doc.create_element("span");

    // child: 200x200 远超所有祖先
    let child_box = make_box(Some(child), 0.0, 0.0, 200.0, 200.0);
    // inner: overflow:hidden, 30x30 内容区域
    let inner_box = LayoutBox {
        node_id: Some(inner),
        x: 0.0,
        y: 0.0,
        width: 30.0,
        height: 30.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 30.0,
        content_height: 30.0,
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
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };
    // middle: overflow:hidden, 50x50 内容区域
    let middle_box = LayoutBox {
        node_id: Some(middle),
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 50.0,
        content_height: 50.0,
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
        children: vec![inner_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };
    // outer: overflow:hidden, 80x80 内容区域
    let outer_box = LayoutBox {
        node_id: Some(outer),
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 80.0,
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
        children: vec![middle_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&outer_box, &styles, None);

    // child(200x200) → inner 裁剪到 30x30
    // inner 的裁剪结果(30x30) 在 middle(50x50) 内 → 不进一步裁剪
    // middle 的裁剪结果在 outer(80x80) 内 → 不进一步裁剪
    // 最终 child 填充应为 30x30
    assert_eq!(
        painter.primitives().fills.len(),
        1,
        "三层嵌套 overflow:hidden 应产生 1 个子节点填充"
    );
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 30.0, "child 应被 inner 裁剪到 30 宽");
    assert_eq!(fill.rect.size.height, 30.0, "child 应被 inner 裁剪到 30 高");
}

/// 测试三个嵌套 div 的重叠背景绘制顺序（z-order）。
///
/// 结构：outer(灰色背景, 300x200) > middle(蓝色背景, 200x150) > inner(红色背景, 100x80)
/// 绘制顺序应为 outer → middle → inner（父先于子），
/// 在 fills 列表中依次为 fills[0]=outer, fills[1]=middle, fills[2]=inner。
#[test]
fn test_paint_multiple_overlapping_backgrounds() {
    let mut doc = zero_dom::Document::new();
    let outer = doc.create_element("div");
    let middle = doc.create_element("div");
    let inner = doc.create_element("div");

    let inner_box = make_box(Some(inner), 10.0, 10.0, 100.0, 80.0);
    let middle_box = LayoutBox {
        node_id: Some(middle),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 150.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 150.0,
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
        children: vec![inner_box],
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
    let outer_box = LayoutBox {
        node_id: Some(outer),
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 200.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 300.0,
        content_height: 200.0,
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
        children: vec![middle_box],
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
    let mut outer_style = ComputedStyle::default();
    outer_style.background_color = ColorValue::Rgba(200, 200, 200, 255); // 灰色
    styles.insert(outer, outer_style);

    let mut middle_style = ComputedStyle::default();
    middle_style.background_color = ColorValue::Rgba(0, 0, 255, 255); // 蓝色
    styles.insert(middle, middle_style);

    let mut inner_style = ComputedStyle::default();
    inner_style.background_color = ColorValue::Rgba(255, 0, 0, 255); // 红色
    styles.insert(inner, inner_style);

    let mut painter = Painter::new();
    painter.paint(&outer_box, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 3, "三个嵌套 div 应产生 3 个背景填充");

    // z-order: 外层先绘制 → 中层 → 内层后绘制
    assert_eq!(
        prims.fills[0].color,
        Color::rgb(200, 200, 200),
        "第一个填充应为外层灰色背景"
    );
    assert_eq!(prims.fills[0].rect.size.width, 300.0, "外层宽度应为 300");
    assert_eq!(prims.fills[0].rect.size.height, 200.0, "外层高度应为 200");

    assert_eq!(
        prims.fills[1].color,
        Color::rgb(0, 0, 255),
        "第二个填充应为中层蓝色背景"
    );
    assert_eq!(prims.fills[1].rect.size.width, 200.0, "中层宽度应为 200");
    assert_eq!(prims.fills[1].rect.size.height, 150.0, "中层高度应为 150");

    assert_eq!(
        prims.fills[2].color,
        Color::rgb(255, 0, 0),
        "第三个填充应为内层红色背景"
    );
    assert_eq!(prims.fills[2].rect.origin.x, 10.0, "内层 x 偏移应为 10");
    assert_eq!(prims.fills[2].rect.origin.y, 10.0, "内层 y 偏移应为 10");
    assert_eq!(prims.fills[2].rect.size.width, 100.0, "内层宽度应为 100");
    assert_eq!(prims.fills[2].rect.size.height, 80.0, "内层高度应为 80");
}

/// 测试仅有 border-top 和 border-right 时只绘制这两条边框。
///
/// 元素只有顶部和右侧有边框（border-top: 3px solid red, border-right: 5px solid green），
/// 底部和左侧边框宽度为 0。验证只生成 2 个边框填充图元。
#[test]
fn test_paint_border_different_sides() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 只有 top=3 和 right=5 有边框
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 60.0, 3.0, 5.0, 0.0, 0.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255); // 红色顶部
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255); // 绿色右侧
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    // 底部和左侧保持默认 border-style: none
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 只有 2 个边框填充（top + right），bottom 和 left 宽度为 0 不生成
    assert_eq!(prims.fills.len(), 2, "应只绘制 top 和 right 两条边框");

    // 第一个填充：顶部边框
    let top_fill = &prims.fills[0];
    assert_eq!(top_fill.color, Color::rgb(255, 0, 0), "顶部边框应为红色");
    assert_eq!(top_fill.rect.origin.x, 0.0);
    assert_eq!(top_fill.rect.origin.y, 0.0);
    assert_eq!(top_fill.rect.size.width, 100.0, "顶部边框宽度应等于元素宽度");
    assert_eq!(top_fill.rect.size.height, 3.0, "顶部边框高度应为 3");

    // 第二个填充：右侧边框
    let right_fill = &prims.fills[1];
    assert_eq!(right_fill.color, Color::rgb(0, 255, 0), "右侧边框应为绿色");
    assert_eq!(right_fill.rect.size.width, 5.0, "右侧边框宽度应为 5");
    // 右侧边框高度 = 元素高度 - top - bottom = 60 - 3 - 0 = 57
    assert_eq!(right_fill.rect.size.height, 57.0, "右侧边框高度应为 57（60-3-0）");
    // 右侧边框 x = 元素宽度 - right = 100 - 5 = 95
    assert_eq!(right_fill.rect.origin.x, 95.0, "右侧边框 x 应为 95（100-5）");
    assert_eq!(
        right_fill.rect.origin.y, 3.0,
        "右侧边框 y 应为 3（从 top 边框下方开始）"
    );
}
