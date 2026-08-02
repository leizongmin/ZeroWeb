//! primitive 模块单元测试

use super::*;
use crate::geometry::Point;

#[test]
fn test_primitives_empty() {
    let p = RenderPrimitives::new();
    assert!(p.is_empty());
    assert_eq!(p.len(), 0);
    assert!(p.bounding_box().is_none());
}

#[test]
fn test_primitives_add_fill() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
    assert!(!p.is_empty());
    assert_eq!(p.fills.len(), 1);
    assert_eq!(p.glyphs.len(), 0);
}

#[test]
fn test_draw_order_records_insertion_order() {
    // DC-10: draw_order 必须按 add_* 的真实调用顺序记录，使渲染器能按
    // 插入序（而非类型分桶）渲染，修复父背景图覆盖子内容的 painting-order 缺陷。
    use crate::image_cache::ImageKey;
    use crate::primitive::{FontId, GlyphPrimitive, ImagePrimitive, LineCap, LineStyle, StrokePrimitive};

    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED); // parent bg fill
    p.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        image_key: ImageKey::new(1),
        clip: None,
    }); // parent bg image — must paint BEFORE child fill
    p.add_fill(Rect::new(2.0, 2.0, 4.0, 4.0), Color::GREEN); // child content fill
    p.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 0.0,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    p.add_glyph(GlyphPrimitive {
        x: 3.0,
        y: 3.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });

    // draw_order 必须与调用顺序一一对应：Fill(0), Image(0), Fill(1), Stroke(0), Glyph(0)。
    assert_eq!(
        p.draw_order,
        vec![
            DrawOp::Fill(0),
            DrawOp::Image(0),
            DrawOp::Fill(1),
            DrawOp::Stroke(0),
            DrawOp::Glyph(0),
        ]
    );
}

#[test]
fn test_primitives_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(10.0, 20.0, 100.0, 50.0), Color::BLACK);
    p.add_fill(Rect::new(200.0, 100.0, 50.0, 50.0), Color::BLACK);

    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.origin, Point::new(10.0, 20.0));
    // 右边界 250, 下边界 150
    assert_eq!(bb.right(), 250.0);
    assert_eq!(bb.bottom(), 150.0);
}

#[test]
fn test_fill_primitive_fields() {
    let fill = FillPrimitive {
        rect: Rect::new(1.0, 2.0, 3.0, 4.0),
        color: Color::BLUE,
    };
    assert_eq!(fill.rect.origin.x, 1.0);
    assert_eq!(fill.color, Color::BLUE);
}

#[test]
fn test_rounded_rect_uniform() {
    let rr = RoundedRectPrimitive::uniform(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED, 10.0);
    assert_eq!(rr.top_left_radius, 10.0);
    assert_eq!(rr.top_right_radius, 10.0);
    assert_eq!(rr.bottom_right_radius, 10.0);
    assert_eq!(rr.bottom_left_radius, 10.0);
}

#[test]
fn test_rounded_rect_in_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_rounded_rect(RoundedRectPrimitive::uniform(
        Rect::new(10.0, 10.0, 80.0, 80.0),
        Color::GREEN,
        15.0,
    ));
    assert_eq!(p.rounded_rects.len(), 1);
    assert!(!p.is_empty());
}

#[test]
fn test_stroke_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
        width: 2.0,
        color: Color::BLACK,
        style: LineStyle::Dashed,
        cap: LineCap::Butt,
    });
    assert_eq!(p.strokes.len(), 1);
    assert!(!p.is_empty());
}

#[test]
fn test_clip_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_clip(Rect::new(0.0, 0.0, 200.0, 200.0));
    assert_eq!(p.clips.len(), 1);
    assert!(!p.is_empty());
}

#[test]
fn test_gradient_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 100.0,
            y1: 0.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::RED,
            },
            GradientStop {
                offset: 1.0,
                color: Color::BLUE,
            },
        ],
        repeating: false,
    });
    assert_eq!(p.gradients.len(), 1);
}

#[test]
fn test_shadow_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(10.0, 10.0, 80.0, 80.0),
        color: Color::rgba(0, 0, 0, 128),
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        inset: false,
    });
    assert_eq!(p.shadows.len(), 1);
}

#[test]
fn test_image_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        image_key: ImageKey::new(42),
        clip: None,
    });
    assert_eq!(p.images.len(), 1);
}

#[test]
fn test_path_fill_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_path_fill(vec![0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0, 50.0], Color::RED);
    assert_eq!(p.path_fills.len(), 1);
    assert!(!p.is_empty());
}

#[test]
fn test_path_stroke_primitive() {
    let mut p = RenderPrimitives::new();
    p.add_path_stroke(vec![0.0, 0.0, 100.0, 100.0], Color::BLACK, 2.0, false);
    assert_eq!(p.path_strokes.len(), 1);
}

#[test]
fn test_bounding_box_with_rounded_rect() {
    let mut p = RenderPrimitives::new();
    p.add_rounded_rect(RoundedRectPrimitive::uniform(
        Rect::new(5.0, 5.0, 50.0, 50.0),
        Color::BLACK,
        10.0,
    ));
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 5.0);
    assert_eq!(bb.top(), 5.0);
    assert_eq!(bb.right(), 55.0);
    assert_eq!(bb.bottom(), 55.0);
}

#[test]
fn test_bounding_box_with_stroke() {
    let mut p = RenderPrimitives::new();
    p.add_stroke(StrokePrimitive {
        x1: 10.0,
        y1: 20.0,
        x2: 50.0,
        y2: 60.0,
        width: 4.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 8.0); // 10 - 2
    assert_eq!(bb.top(), 18.0); // 20 - 2
    assert_eq!(bb.right(), 52.0); // 50 + 2
    assert_eq!(bb.bottom(), 62.0); // 60 + 2
}

#[test]
fn test_bounding_box_with_shadow() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(10.0, 10.0, 50.0, 50.0),
        color: Color::BLACK,
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 3.0,
        spread_radius: 2.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 10.0);
    assert_eq!(bb.top(), 10.0);
    assert_eq!(bb.right(), 70.0);
    assert_eq!(bb.bottom(), 70.0);
}

#[test]
fn test_len_counts_all_types() {
    let mut p = RenderPrimitives::new();
    p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
    p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED);
    p.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 10.0,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    assert!(p.len() >= 3);
}

#[test]
fn test_line_style_equality() {
    assert_eq!(LineStyle::Solid, LineStyle::Solid);
    assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
}

#[test]
fn test_line_cap_equality() {
    assert_eq!(LineCap::Round, LineCap::Round);
    assert_ne!(LineCap::Butt, LineCap::Square);
}

#[test]
fn test_gradient_kind_radial() {
    let kind = GradientKind::Radial {
        cx: 50.0,
        cy: 50.0,
        inner_radius: 0.0,
        outer_radius: 50.0,
    };
    if let GradientKind::Radial { outer_radius, .. } = kind {
        assert_eq!(outer_radius, 50.0);
    } else {
        panic!("Expected Radial");
    }
}

#[test]
fn test_glyph_primitive_creation() {
    let g = GlyphPrimitive {
        x: 10.0,
        y: 20.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 42,
        font_id: FontId(1),
        bitmap_width: Some(12),
        bitmap_height: Some(16),
        rotation: 0.0,
    };
    assert_eq!(g.x, 10.0);
    assert_eq!(g.font_id, FontId(1));
    assert_eq!(g.bitmap_width, Some(12));
}

#[test]
fn test_glyph_in_render_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_glyph(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    assert_eq!(p.glyphs.len(), 1);
    assert!(!p.is_empty());
}

#[test]
fn test_font_id_equality() {
    assert_eq!(FontId(1), FontId(1));
    assert_ne!(FontId(1), FontId(2));
}

#[test]
fn test_bounding_box_with_glyphs() {
    let mut p = RenderPrimitives::new();
    p.add_glyph(GlyphPrimitive {
        x: 5.0,
        y: 10.0,
        font_size: 16.0,
        color: Color::BLACK,
        glyph_id: 0,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 5.0);
    assert_eq!(bb.top(), 10.0);
    assert_eq!(bb.right(), 21.0); // x + font_size
    assert_eq!(bb.bottom(), 26.0); // y + font_size
}

#[test]
fn test_bounding_box_with_images() {
    let mut p = RenderPrimitives::new();
    p.add_image(ImagePrimitive {
        rect: Rect::new(50.0, 60.0, 100.0, 80.0),
        image_key: ImageKey::new(1),
        clip: None,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 50.0);
    assert_eq!(bb.top(), 60.0);
    assert_eq!(bb.right(), 150.0);
    assert_eq!(bb.bottom(), 140.0);
}

#[test]
fn test_bounding_box_with_gradient() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 200.0, 100.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 200.0,
            y1: 0.0,
        },
        stops: vec![],
        repeating: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.right(), 200.0);
    assert_eq!(bb.bottom(), 100.0);
}

#[test]
fn test_bounding_box_with_path_fill() {
    let mut p = RenderPrimitives::new();
    p.add_path_fill(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0], Color::RED);
    let bb = p.bounding_box().unwrap();
    // Points: (10,20), (30,40), (50,60)
    assert_eq!(bb.left(), 10.0);
    assert_eq!(bb.top(), 20.0);
    assert_eq!(bb.right(), 50.0);
    assert_eq!(bb.bottom(), 60.0);
}

#[test]
fn test_render_primitives_mixed_types_count() {
    let mut p = RenderPrimitives::new();
    p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
    p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::BLUE);
    p.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 10.0,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Round,
    });
    p.add_glyph(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 0,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    assert_eq!(p.len(), 5);
    assert!(!p.is_empty());
}

#[test]
fn test_rounded_rect_individual_radii() {
    let rr = RoundedRectPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        color: Color::GREEN,
        top_left_radius: 5.0,
        top_right_radius: 10.0,
        bottom_right_radius: 15.0,
        bottom_left_radius: 20.0,
    };
    assert_eq!(rr.top_left_radius, 5.0);
    assert_eq!(rr.top_right_radius, 10.0);
    assert_eq!(rr.bottom_right_radius, 15.0);
    assert_eq!(rr.bottom_left_radius, 20.0);
}

// -- 边界条件测试 --

/// 测试 bounding_box 只包含 clips 时返回 None
#[test]
fn test_bounding_box_clips_only_returns_none() {
    let mut p = RenderPrimitives::new();
    p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
    p.add_clip(Rect::new(50.0, 50.0, 100.0, 100.0));
    // clips 不参与 bounding_box 计算
    assert!(p.bounding_box().is_none());
}

/// 测试 RenderPrimitives::len 包含所有类型
#[test]
fn test_len_all_primitive_types() {
    let mut p = RenderPrimitives::new();
    p.add_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
    p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::BLACK);
    p.add_rounded_rect(RoundedRectPrimitive::uniform(
        Rect::new(0.0, 0.0, 10.0, 10.0),
        Color::BLACK,
        5.0,
    ));
    p.add_path_fill(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK);
    p.add_path_stroke(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK, 1.0, false);
    p.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 10.0,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 0.0,
        },
        stops: vec![],
        repeating: false,
    });
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: Color::BLACK,
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        inset: false,
    });
    p.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        image_key: ImageKey::new(0),
        clip: None,
    });
    p.add_glyph(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 0,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    assert_eq!(p.len(), 10);
}

/// 测试 bounding_box 包含负坐标
#[test]
fn test_bounding_box_negative_coordinates() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(-50.0, -30.0, 100.0, 60.0), Color::BLACK);
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), -50.0);
    assert_eq!(bb.top(), -30.0);
    assert_eq!(bb.right(), 50.0);
    assert_eq!(bb.bottom(), 30.0);
}

/// 透明度 alpha=0.0 的图元应不可见（预乘 alpha 后所有通道为零）。
#[test]
fn test_composite_primitive_opacity_zero() {
    let invisible_color = Color::rgba(255, 0, 0, 0);
    let premultiplied = invisible_color.premultiplied();
    assert!(premultiplied[0].abs() < f32::EPSILON, "R 通道预乘后应为 0");
    assert!(premultiplied[1].abs() < f32::EPSILON, "G 通道预乘后应为 0");
    assert!(premultiplied[2].abs() < f32::EPSILON, "B 通道预乘后应为 0");
    assert!(premultiplied[3].abs() < f32::EPSILON, "A 通道预乘后应为 0");

    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), invisible_color);
    assert_eq!(p.fills.len(), 1);
    assert_eq!(p.fills[0].color.a, 0, "alpha 应为 0");

    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(10.0, 10.0, 50.0, 50.0),
        color: Color::TRANSPARENT,
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 3.0,
        spread_radius: 0.0,
        inset: false,
    });
    let shadow = &p.shadows[0];
    assert_eq!(shadow.color.a, 0);
    let shadow_premul = shadow.color.premultiplied();
    assert!(shadow_premul.iter().all(|&c| c.abs() < f32::EPSILON));
}

/// 测试 path_fill 空 vertices 的 bounding_box
#[test]
fn test_bounding_box_empty_path_fill_vertices() {
    let mut p = RenderPrimitives::new();
    p.add_path_fill(vec![], Color::BLACK);
    assert!(p.bounding_box().is_none());
}

/// 测试 StrokePrimitive width=0.0
#[test]
fn test_stroke_primitive_zero_width() {
    let s = StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 10.0,
        y2: 10.0,
        width: 0.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    };
    assert_eq!(s.width, 0.0);

    let mut p = RenderPrimitives::new();
    p.add_stroke(s);
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 0.0);
    assert_eq!(bb.top(), 0.0);
    assert_eq!(bb.right(), 10.0);
    assert_eq!(bb.bottom(), 10.0);
}

/// 测试 bounding_box 在 GlyphPrimitive 含 bitmap_width/bitmap_height 时
/// 仍基于 font_size 计算包围盒（不使用 bitmap 尺寸）。
#[test]
fn test_bounding_box_glyph_with_bitmap_dims() {
    let mut p = RenderPrimitives::new();
    p.add_glyph(GlyphPrimitive {
        x: 100.0,
        y: 200.0,
        font_size: 24.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: Some(12),
        bitmap_height: Some(16),
        rotation: 0.0,
    });

    let bb = p.bounding_box().expect("glyph 应产生包围盒");
    assert_eq!(bb.left(), 100.0, "left 应为 glyph.x");
    assert_eq!(bb.top(), 200.0, "top 应为 glyph.y");
    assert_eq!(bb.right(), 124.0, "right 应为 x + font_size = 124");
    assert_eq!(bb.bottom(), 224.0, "bottom 应为 y + font_size = 224");
}

/// 测试 ShadowPrimitive 大模糊半径 bounding_box 计算。
#[test]
fn test_edge_shadow_large_blur_radius_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(100.0, 100.0, 50.0, 50.0),
        color: Color::BLACK,
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 200.0,
        spread_radius: 0.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), -100.0);
    assert_eq!(bb.top(), -100.0);
    assert_eq!(bb.right(), 350.0);
    assert_eq!(bb.bottom(), 350.0);
}

/// 测试 ShadowPrimitive 负偏移 bounding_box 计算。
#[test]
fn test_edge_shadow_negative_offset_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(50.0, 50.0, 40.0, 40.0),
        color: Color::BLACK,
        offset_x: -10.0,
        offset_y: -20.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 40.0);
    assert_eq!(bb.top(), 30.0);
    assert_eq!(bb.right(), 80.0);
    assert_eq!(bb.bottom(), 70.0);
}

/// 测试 ShadowPrimitive 大扩展半径 bounding_box 计算。
#[test]
fn test_edge_shadow_large_spread_radius_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(20.0, 20.0, 30.0, 30.0),
        color: Color::BLACK,
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 50.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), -30.0);
    assert_eq!(bb.top(), -30.0);
    assert_eq!(bb.right(), 100.0);
    assert_eq!(bb.bottom(), 100.0);
}

/// 测试多个 ShadowPrimitive bounding_box 合并计算。
#[test]
fn test_edge_multiple_shadows_bounding_box_merge() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        color: Color::BLACK,
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 2.0,
        spread_radius: 1.0,
        inset: false,
    });
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(200.0, 200.0, 50.0, 50.0),
        color: Color::BLACK,
        offset_x: -5.0,
        offset_y: -5.0,
        blur_radius: 10.0,
        spread_radius: 0.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 2.0);
    assert_eq!(bb.top(), 2.0);
    assert_eq!(bb.right(), 255.0);
    assert_eq!(bb.bottom(), 255.0);
}

/// 测试 ImagePrimitive 不同 ImageKey 区分。
#[test]
fn test_edge_image_primitive_different_keys() {
    let mut p = RenderPrimitives::new();
    let key_a = ImageKey::new(100);
    let key_b = ImageKey::new(200);
    p.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        image_key: key_a,
        clip: None,
    });
    p.add_image(ImagePrimitive {
        rect: Rect::new(10.0, 10.0, 50.0, 50.0),
        image_key: key_b,
        clip: None,
    });
    assert_eq!(p.images.len(), 2);
    assert_ne!(p.images[0].image_key, p.images[1].image_key);
    assert_eq!(p.images[0].image_key, ImageKey::new(100));
    assert_eq!(p.images[1].image_key, ImageKey::new(200));
    assert_eq!(p.images[0].rect.origin.x, 0.0);
    assert_eq!(p.images[1].rect.origin.x, 10.0);
}

/// 测试 RenderPrimitives 包含阴影和图片时的 len 计数。
#[test]
fn test_edge_len_with_shadows_and_images() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        color: Color::BLACK,
        offset_x: 3.0,
        offset_y: 3.0,
        blur_radius: 5.0,
        spread_radius: 0.0,
        inset: false,
    });
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(50.0, 50.0, 100.0, 100.0),
        color: Color::rgba(0, 0, 0, 80),
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 10.0,
        spread_radius: 2.0,
        inset: false,
    });
    p.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 200.0, 200.0),
        image_key: ImageKey::new(1),
        clip: None,
    });
    p.add_image(ImagePrimitive {
        rect: Rect::new(10.0, 10.0, 150.0, 150.0),
        image_key: ImageKey::new(2),
        clip: None,
    });
    p.add_image(ImagePrimitive {
        rect: Rect::new(20.0, 20.0, 100.0, 100.0),
        image_key: ImageKey::new(3),
        clip: None,
    });
    assert_eq!(p.shadows.len(), 2);
    assert_eq!(p.images.len(), 3);
    assert_eq!(p.len(), 5);
    assert!(!p.is_empty());
}

/// 测试 ShadowPrimitive 零尺寸矩形。
#[test]
fn test_edge_shadow_zero_size_rect() {
    let mut p = RenderPrimitives::new();
    p.add_shadow(ShadowPrimitive {
        rect: Rect::new(50.0, 50.0, 0.0, 0.0),
        color: Color::BLACK,
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 5.0,
        spread_radius: 3.0,
        inset: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 42.0);
    assert_eq!(bb.top(), 42.0);
    assert_eq!(bb.right(), 58.0);
    assert_eq!(bb.bottom(), 58.0);
}

/// 测试 GradientPrimitive::Linear 加入 RenderPrimitives。
#[test]
fn test_gradient_primitive_linear_in_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 100.0,
            y1: 0.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::RED,
            },
            GradientStop {
                offset: 1.0,
                color: Color::BLUE,
            },
        ],
        repeating: false,
    });
    assert_eq!(p.gradients.len(), 1);
}

/// 测试 GradientPrimitive::Radial 加入 RenderPrimitives。
#[test]
fn test_gradient_primitive_radial_in_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        kind: GradientKind::Radial {
            cx: 50.0,
            cy: 50.0,
            inner_radius: 10.0,
            outer_radius: 50.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::WHITE,
            },
            GradientStop {
                offset: 1.0,
                color: Color::BLACK,
            },
        ],
        repeating: false,
    });
    assert_eq!(p.gradients.len(), 1);
    if let GradientKind::Radial {
        cx,
        cy,
        inner_radius,
        outer_radius,
    } = &p.gradients[0].kind
    {
        assert_eq!(*cx, 50.0);
        assert_eq!(*cy, 50.0);
        assert_eq!(*inner_radius, 10.0);
        assert_eq!(*outer_radius, 50.0);
    } else {
        panic!("Expected Radial gradient kind");
    }
}

/// 测试渐变 bounding_box 计算。
#[test]
fn test_gradient_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(10.0, 20.0, 100.0, 80.0),
        kind: GradientKind::Linear {
            x0: 10.0,
            y0: 20.0,
            x1: 110.0,
            y1: 20.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::RED,
            },
            GradientStop {
                offset: 1.0,
                color: Color::BLUE,
            },
        ],
        repeating: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 10.0);
    assert_eq!(bb.top(), 20.0);
    assert_eq!(bb.right(), 110.0);
    assert_eq!(bb.bottom(), 100.0);
}

/// 测试多个渐变图元 bounding_box 合并。
#[test]
fn test_multiple_gradients_bounding_box() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(10.0, 20.0, 100.0, 80.0),
        kind: GradientKind::Linear {
            x0: 10.0,
            y0: 20.0,
            x1: 110.0,
            y1: 20.0,
        },
        stops: vec![],
        repeating: false,
    });
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(200.0, 150.0, 50.0, 50.0),
        kind: GradientKind::Radial {
            cx: 225.0,
            cy: 175.0,
            inner_radius: 0.0,
            outer_radius: 25.0,
        },
        stops: vec![],
        repeating: false,
    });
    let bb = p.bounding_box().unwrap();
    assert_eq!(bb.left(), 10.0);
    assert_eq!(bb.top(), 20.0);
    assert_eq!(bb.right(), 250.0);
    assert_eq!(bb.bottom(), 200.0);
}

/// 测试 GradientStop 顺序。
#[test]
fn test_gradient_stops_order() {
    let mut p = RenderPrimitives::new();
    p.add_gradient(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 100.0,
            y1: 0.0,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::RED,
            },
            GradientStop {
                offset: 0.5,
                color: Color::GREEN,
            },
            GradientStop {
                offset: 1.0,
                color: Color::BLUE,
            },
        ],
        repeating: false,
    });
    let stops = &p.gradients[0].stops;
    assert_eq!(stops.len(), 3);
    assert_eq!(stops[0].offset, 0.0);
    assert_eq!(stops[1].offset, 0.5);
    assert_eq!(stops[2].offset, 1.0);
    for i in 1..stops.len() {
        assert!(
            stops[i].offset > stops[i - 1].offset,
            "stops should be in ascending order"
        );
    }
}

/// 测试 RenderPrimitives::default 等价于 new
#[test]
fn test_render_primitives_default_equals_new() {
    let p1 = RenderPrimitives::new();
    let p2 = RenderPrimitives::default();
    assert!(p1.is_empty());
    assert!(p2.is_empty());
    assert_eq!(p1.len(), 0);
    assert_eq!(p2.len(), 0);
}

/// 测试 bounding_box 包含重合点时返回 None
#[test]
fn test_bounding_box_coincident_points() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(10.0, 10.0, 0.0, 0.0), Color::BLACK);
    assert!(p.bounding_box().is_none(), "零面积矩形不应产生包围盒");
}

/// 测试 path_stroke 空 vertices 不影响 bounding_box
#[test]
fn test_bounding_box_empty_path_stroke_vertices() {
    let mut p = RenderPrimitives::new();
    p.add_path_stroke(vec![], Color::BLACK, 1.0, false);
    assert!(p.bounding_box().is_none(), "空 path_stroke 不应产生包围盒");
}

/// 测试 add_glyph 多次添加后 len 正确
#[test]
fn test_add_glyph_multiple() {
    let mut p = RenderPrimitives::new();
    for i in 0..10 {
        p.add_glyph(GlyphPrimitive {
            x: i as f32,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: i,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
        });
    }
    assert_eq!(p.glyphs.len(), 10);
    assert_eq!(p.len(), 10);
    assert!(!p.is_empty());
}

// ── RenderStats + batch_fills + cull_invisible 测试 ──

#[test]
fn test_stats_empty_primitives() {
    let p = RenderPrimitives::new();
    let stats = p.stats();
    assert_eq!(stats.total_primitives(), 0);
    assert_eq!(stats.estimated_draw_calls, 0);
}

#[test]
fn test_stats_single_fill() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
    let stats = p.stats();
    assert_eq!(stats.fill_count, 1);
    assert_eq!(stats.estimated_draw_calls, 1);
}

#[test]
fn test_stats_same_color_fills_batched_draw_calls() {
    let mut p = RenderPrimitives::new();
    for i in 0..5 {
        p.add_fill(Rect::new(i as f32 * 100.0, 0.0, 50.0, 50.0), Color::RED);
    }
    let stats = p.stats();
    assert_eq!(stats.fill_count, 5);
    assert_eq!(stats.estimated_draw_calls, 1);
}

#[test]
fn test_stats_different_color_fills_separate_draw_calls() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
    p.add_fill(Rect::new(100.0, 0.0, 50.0, 50.0), Color::BLUE);
    p.add_fill(Rect::new(200.0, 0.0, 50.0, 50.0), Color::GREEN);
    let stats = p.stats();
    assert_eq!(stats.fill_count, 3);
    assert_eq!(stats.estimated_draw_calls, 3);
}

#[test]
fn test_stats_mixed_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
    p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
    p.add_glyph(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    p.add_glyph(GlyphPrimitive {
        x: 10.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 66,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    let stats = p.stats();
    assert_eq!(stats.total_primitives(), 4);
    assert_eq!(stats.estimated_draw_calls, 2);
}

#[test]
fn test_batch_fills_no_merge_different_colors() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::BLUE);
    let batched = p.batch_fills();
    assert_eq!(batched.fills.len(), 2);
}

#[test]
fn test_batch_fills_merge_adjacent_same_color() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 50.0, 100.0, 50.0), Color::RED);
    // 合并优化仅服务于 render_typed_buckets 回退路径（draw_order 为空）。
    // draw_order 非空时（生产 render_draw_order 默认路径）batch_fills 直接跳过——
    // 颜色分组会破坏 draw_order 的真实绘制顺序（见 batch_fills 注释）。
    p.draw_order.clear();
    let batched = p.batch_fills();
    assert_eq!(batched.fills.len(), 1);
    let merged = &batched.fills[0];
    assert_eq!(merged.rect.origin.y, 0.0);
    assert_eq!(merged.rect.size.height, 100.0);
}

#[test]
fn test_batch_fills_no_merge_non_adjacent() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 200.0, 100.0, 50.0), Color::RED);
    let batched = p.batch_fills();
    assert_eq!(batched.fills.len(), 2);
}

#[test]
fn test_batch_fills_preserves_other_primitives() {
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 50.0, 100.0, 50.0), Color::RED);
    p.add_glyph(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    // 合并优化仅服务于 render_typed_buckets 回退路径（draw_order 为空）；见 batch_fills 注释。
    p.draw_order.clear();
    let batched = p.batch_fills();
    assert_eq!(batched.fills.len(), 1);
    assert_eq!(batched.glyphs.len(), 1);
}

#[test]
fn test_batch_fills_skips_when_draw_order_present() {
    // 生产默认走 render_draw_order 路径（draw_order 非空）。batch_fills 的颜色分组
    // 会重排 fills 但不更新 draw_order 索引，破坏 CSS painting order（如 flex-grow-003：
    // position:relative 的 cover 被同色分组提前到 in-flow flex items 之下）。
    // 故 draw_order 非空时 batch_fills 须直接跳过，保持 fills 与 draw_order 的一致性。
    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
    p.add_fill(Rect::new(0.0, 50.0, 100.0, 50.0), Color::RED);
    let n_draw_order = p.draw_order.len();
    assert!(n_draw_order > 0, "add_fill 应填充 draw_order");
    let batched = p.batch_fills();
    // 跳过：fills 数量不变（未合并）、draw_order 不变
    assert_eq!(batched.fills.len(), 2);
    assert_eq!(batched.draw_order.len(), n_draw_order);
}

#[test]
fn test_cull_invisible_removes_offscreen_fills() {
    let mut p = RenderPrimitives::new();
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
    p.add_fill(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED);
    p.add_fill(Rect::new(900.0, 10.0, 50.0, 50.0), Color::RED);
    let (culled, stats) = p.cull_invisible(viewport);
    assert_eq!(culled.fills.len(), 1);
    assert_eq!(stats.culled_count, 1);
}

#[test]
fn test_cull_invisible_keeps_clips_and_glyphs() {
    let mut p = RenderPrimitives::new();
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
    p.add_clip(Rect::new(0.0, 0.0, 1000.0, 1000.0));
    p.add_glyph(GlyphPrimitive {
        x: 900.0,
        y: 10.0,
        font_size: 12.0,
        color: Color::BLACK,
        glyph_id: 65,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    let (culled, _) = p.cull_invisible(viewport);
    assert_eq!(culled.clips.len(), 1);
    assert_eq!(culled.glyphs.len(), 1);
}

#[test]
fn test_cull_invisible_nothing_removed_when_all_visible() {
    let mut p = RenderPrimitives::new();
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
    p.add_fill(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED);
    p.add_fill(Rect::new(100.0, 100.0, 50.0, 50.0), Color::BLUE);
    let (culled, stats) = p.cull_invisible(viewport);
    assert_eq!(culled.fills.len(), 2);
    assert_eq!(stats.culled_count, 0);
}

#[test]
fn test_cull_invisible_partial_overlap_kept() {
    let mut p = RenderPrimitives::new();
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
    p.add_fill(Rect::new(750.0, 10.0, 100.0, 50.0), Color::RED);
    let (culled, stats) = p.cull_invisible(viewport);
    assert_eq!(culled.fills.len(), 1);
    assert_eq!(stats.culled_count, 0);
}
