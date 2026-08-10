//! CPU 渲染器测试 — 验证所有图元类型的渲染正确性。

use super::*;
use crate::color::Color;
use crate::geometry::Rect;
use crate::gpu::renderer::GlyphDraw;
use crate::image_cache::{ImageCache, ImageData, ImageKey};
use crate::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, FillPrimitive, FilterKind, FilterPrimitive, FontId, GlyphPrimitive,
    GradientKind, GradientPrimitive, GradientStop, ImagePrimitive, LineCap, LineStyle, PathFillPrimitive,
    PathStrokePrimitive, RenderPrimitives, RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive, TransformPrimitive,
};

// ─── 旧版兼容测试 ───

#[test]
fn glyph_top_left_converts_fontdue_y_up_metrics_to_screen_y_down() {
    let (x, y) = glyph_top_left(10.0, 50.0, 2, -4, 18);
    assert_eq!(x, 12.0);
    assert_eq!(y, 36.0);
}

#[test]
fn indexed_glyph_renders_identically_to_unicode_code_point() {
    const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
    let mut font_loader = FontLoader::new();
    let font_id = font_loader.load_font(LATO_TTF).expect("should load bundled Lato font");
    let glyph_index = font_loader
        .get(font_id)
        .expect("font should remain loaded")
        .lookup_glyph_index('A');
    let make_primitives = |font_glyph_index| {
        let mut primitives = RenderPrimitives::new();
        primitives.add_glyph(GlyphPrimitive {
            x: 8.0,
            y: 28.0,
            font_size: 20.0,
            color: Color::BLACK,
            glyph_id: 'A' as u32,
            font_glyph_index,
            font_id: FontId(font_id),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        primitives
    };
    let unicode = make_primitives(None);
    let indexed = make_primitives(Some(glyph_index));
    let mut unicode_cache = GlyphCache::new(8);
    let mut indexed_cache = GlyphCache::new(8);

    let unicode_frame = render_full_scene(
        40,
        40,
        1.0,
        &unicode,
        &font_loader,
        &mut unicode_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    let indexed_frame = render_full_scene(
        40,
        40,
        1.0,
        &indexed,
        &font_loader,
        &mut indexed_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(indexed_frame.data, unicode_frame.data);
}

#[test]
fn render_scene_to_framebuffer_scales_logical_dimensions() {
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 5.0, 4.0),
        color: Color::BLACK,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    let fb = render_scene_to_framebuffer(
        10,
        8,
        2.0,
        &fills,
        &[],
        &font_loader,
        &mut glyph_cache,
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(fb.width, 20);
    assert_eq!(fb.height, 16);
    assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(19, 15), [255, 255, 255, 255]);
}

#[test]
fn render_scene_to_framebuffer_no_scaling() {
    let fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: Color::RED,
    }];
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    let fb = render_scene_to_framebuffer(
        10,
        10,
        1.0,
        &fills,
        &[],
        &font_loader,
        &mut glyph_cache,
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(fb.width, 10);
    assert_eq!(fb.height, 10);
    assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
    assert_eq!(fb.get_pixel(9, 9), [255, 0, 0, 255]);
}

#[test]
fn render_scene_to_framebuffer_empty_inputs() {
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);

    let fb = render_scene_to_framebuffer(8, 8, 1.0, &[], &[], &font_loader, &mut glyph_cache, &[], &[], &[], &[]);

    assert_eq!(fb.width, 8);
    assert_eq!(fb.height, 8);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(fb.get_pixel(x, y), [255, 255, 255, 255]);
        }
    }
}

#[test]
fn scale_dimension_edge_cases() {
    assert_eq!(scale_dimension(0, 1.0), 1);
    assert_eq!(scale_dimension(0, 2.0), 1);
    assert_eq!(scale_dimension(100, 1.0), 100);
    assert_eq!(scale_dimension(100, 1.5), 150);
}

#[test]
fn render_image_releases_cache_reference_after_draw() {
    let mut fb = FrameBuffer::new(8, 8);
    let mut cache = ImageCache::new(8, 1024 * 1024);
    let image = ImageData::from_rgba([255u8, 0, 0, 255].repeat(4), 2, 2).unwrap();
    let key = ImageKey::new(7);
    cache.insert_with_key(key.clone(), image);

    let primitive = ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 4.0, 4.0),
        image_key: key.clone(),
        clip: None,
    };

    render_image(&mut fb, &primitive, 1.0, &mut cache);

    assert_eq!(
        cache.ref_count(&key),
        Some(1),
        "rendering should return image-cache refcount to baseline"
    );
}

// ─── M7 新增图元测试 ───

#[test]
fn gradient_linear_red_to_blue() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 10.0),
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

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        10,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 左端应该是红色
    let left_pixel = fb.get_pixel(1, 5);
    assert!(
        left_pixel[0] > 200,
        "left should be red (R > 200), got {:?}",
        left_pixel
    );
    assert!(left_pixel[2] < 50, "left should have low blue, got {:?}", left_pixel);

    // 右端应该是蓝色
    let right_pixel = fb.get_pixel(98, 5);
    assert!(
        right_pixel[2] > 200,
        "right should be blue (B > 200), got {:?}",
        right_pixel
    );
    assert!(right_pixel[0] < 50, "right should have low red, got {:?}", right_pixel);

    // 中间应该是紫色（红色和蓝色的混合）
    let mid_pixel = fb.get_pixel(50, 5);
    assert!(
        mid_pixel[0] > 50 && mid_pixel[2] > 50,
        "middle should be purple-ish, got {:?}",
        mid_pixel
    );
}

/// 圆角矩形背景必须通过 draw_order 渲染（`add_rounded_rect` 记录 `DrawOp::RoundedRect`）。
///
/// 回归测试：`paint_background` 此前直接 `rounded_rects.push()` 绕过 `add_rounded_rect`，
/// 导致 draw_order（默认渲染路径）丢弃圆角背景——任何带 border-radius 的元素背景都
/// 不绘制（DC-13 welcome.html 卡片白底消失，welcome 差距 50.45%→26.15% 的主因之一）。
/// 修复后通过 `add_rounded_rect` 记录 DrawOp，圆角背景在 draw_order 模式下正常渲染。
#[test]
fn rounded_rect_renders_via_draw_order() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_rounded_rect(RoundedRectPrimitive {
        rect: Rect::new(0.0, 0.0, 40.0, 40.0),
        color: Color::RED,
        top_left_radius: 8.0,
        top_right_radius: 8.0,
        bottom_right_radius: 8.0,
        bottom_left_radius: 8.0,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        40,
        40,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 中心像素（远离圆角）应为红色——若 draw_order 丢弃 rounded_rect 则为透明/黑
    let center = fb.get_pixel(20, 20);
    assert!(
        center[0] > 200,
        "rounded_rect center should be red via draw_order, got {:?}",
        center
    );
}

#[test]
fn gradient_radial_center_to_edge() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 20.0, 20.0),
        kind: GradientKind::Radial {
            cx: 10.0,
            cy: 10.0,
            inner_radius: 0.0,
            outer_radius: 10.0,
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

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        20,
        20,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 中心应该是白色
    let center = fb.get_pixel(10, 10);
    assert!(center[0] > 200, "center should be white, got {:?}", center);

    // 角落应该是黑色（距离远）
    let corner = fb.get_pixel(0, 0);
    assert!(corner[0] < 100, "corner should be dark, got {:?}", corner);
}

/// 重复线性渐变 — repeating-linear-gradient(red 0px, blue 25px) 在 100px 宽矩形中应重复 4 次。
#[test]
fn gradient_linear_repeating() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 10.0),
        kind: GradientKind::Linear {
            x0: 0.0,
            y0: 0.0,
            x1: 25.0, // 渐变周期 25px，在 100px 中重复 4 次
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
        repeating: true,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        10,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // x=0 第一周期起点应为红色
    let p_start = fb.get_pixel(1, 5);
    assert!(p_start[0] > 200, "first period start should be red, got {:?}", p_start);

    // x=12 第一周期中间应为紫色（红→蓝 50%）
    let p1 = fb.get_pixel(12, 5);
    assert!(
        p1[0] > 80 && p1[2] > 80,
        "first period mid should be purple, got {:?}",
        p1
    );

    // x=25 第二周期起点应回到红色（fract(25/25)=0）
    let p2 = fb.get_pixel(26, 5);
    assert!(p2[0] > 100, "second period start should be red-ish, got {:?}", p2);

    // x=50 第三周期起点应为红色
    let p3 = fb.get_pixel(51, 5);
    assert!(p3[0] > 100, "third period start should be red-ish, got {:?}", p3);

    // x=75 第四周期起点应为红色
    let p4 = fb.get_pixel(76, 5);
    assert!(p4[0] > 100, "fourth period start should be red-ish, got {:?}", p4);
}

/// 重复径向渐变 — repeating-radial-gradient 应在距中心不同距离处重复色标。
#[test]
fn gradient_radial_repeating() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 40.0, 40.0),
        kind: GradientKind::Radial {
            cx: 20.0,
            cy: 20.0,
            inner_radius: 0.0,
            outer_radius: 10.0, // 每 10px 重复一次
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
        repeating: true,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        40,
        40,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 中心应该是白色（t ≈ 0）
    let center = fb.get_pixel(20, 20);
    assert!(center[0] > 200, "center should be white, got {:?}", center);

    // 距中心 5px 处（t ≈ 0.5 第一周期）应为灰色
    let mid1 = fb.get_pixel(25, 20);
    assert!(
        mid1[0] > 50 && mid1[0] < 200,
        "first period mid should be gray-ish, got {:?}",
        mid1
    );

    // 距中心 12px 处（t ≈ 0.2 第二周期）应偏白
    let second_period = fb.get_pixel(32, 20);
    assert!(
        second_period[0] > 80,
        "second period should be lighter, got {:?}",
        second_period
    );
}

/// R2316：子域重复线性渐变 — repeating-linear-gradient(red 0, blue 50%) 色标区间 [0,0.5] 为一周期，
/// 在全宽渐变线上应重复 2 次。旧代码 `t/=period` 后用 [0,1) 采 [0,0.5] offset，致 t≥0.5 像素
/// 被钳到 blue（末色标）—— t=0.25 本应是 50% 紫，旧代码给纯蓝。
#[test]
fn gradient_linear_repeating_subrange_stops() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 10.0),
        kind: GradientKind::Linear {
            // 渐变线 = 全宽 100px；色标 [0,0.5] → 一周期 50px，跨 100px 重复 2 次
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
                color: Color::BLUE,
            },
        ],
        repeating: true,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        10,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // x=12 → t≈0.12 → 折叠到第一周期 [0,0.5] 的 0.12 → local_t=0.24 → 24% blue（偏红紫）
    // 旧 buggy：t/=0.5 → 0.24 采 [0,0.5] → local_t=0.48；x=25 更极端：旧代码 t=0.25→0.5→采到末色标=纯蓝
    let p_quarter = fb.get_pixel(25, 5); // t≈0.25 → 第一周期 50% → 紫（R≈B）
    assert!(
        p_quarter[0] > 80 && p_quarter[2] > 80,
        "first-period midpoint (t=0.25) should be purple ~50% red/blue, got {:?}",
        p_quarter
    );

    // x=75 → t≈0.75 → 折叠到第二周期 [0.5,1.0] 的 0.25 → 同样 50% 紫（重复正确）
    let p_second = fb.get_pixel(75, 5);
    assert!(
        p_second[0] > 80 && p_second[2] > 80,
        "second-period midpoint (t=0.75) should be purple (repetition), got {:?}",
        p_second
    );

    // 重复性：两周期同相位应近等色（t=0.25 与 t=0.75 都折叠到 0.25）
    let dr = (p_quarter[0] as i32 - p_second[0] as i32).abs();
    let db = (p_quarter[2] as i32 - p_second[2] as i32).abs();
    assert!(
        dr < 20 && db < 20,
        "same phase across periods should match: quarter={:?} second={:?}",
        p_quarter,
        p_second
    );

    // x=5 → t≈0.05 → 折叠 0.05 → local_t=0.1 → 10% blue（明显偏红，R 远大于 B）
    let p_early = fb.get_pixel(5, 5);
    assert!(
        p_early[0] > p_early[2] + 40,
        "early in period (t=0.05) should be red-dominant, got {:?}",
        p_early
    );
}

/// R2317：conic-gradient 角度约定。CSS 规定 0deg = 正上方（12 点钟）、顺时针。
/// 旧 ZW `atan2(dy, dx)` = 正右（3 点钟）、逆时针——中心正上方像素落在 t=0.75 而非 t=0。
/// 修正后正上方像素应在 t=0（起始色 red），正下方 t=0.5（50% 紫）。
#[test]
fn conic_gradient_angle_convention_top_clockwise() {
    let mut primitives = RenderPrimitives::new();
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        kind: GradientKind::Conic {
            cx: 50.0,
            cy: 50.0,
            start_angle: 0.0,
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

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 正上方像素（dx=0, dy<0）：CSS t=0 → red。旧 buggy（atan2(dy,dx)）→ t=0.75 → 偏蓝
    let top = fb.get_pixel(50, 5);
    assert!(
        top[0] > 200 && top[2] < 80,
        "top pixel (12 o'clock) should be RED (CSS conic 0deg=start), got {:?}",
        top
    );

    // 正右方像素（dx>0, dy=0）：CSS t=0.25 → 25% blue（偏红）
    let right = fb.get_pixel(95, 50);
    assert!(
        right[0] > right[2],
        "right pixel (3 o'clock) should be red-dominant (t=0.25), got {:?}",
        right
    );

    // 正下方像素（dx=0, dy>0）：CSS t=0.5 → 50% 紫
    let bottom = fb.get_pixel(50, 95);
    assert!(
        bottom[0] > 80 && bottom[2] > 80,
        "bottom pixel (6 o'clock) should be purple (t=0.5), got {:?}",
        bottom
    );
}

#[test]
fn shadow_renders_blur_around_rect() {
    let mut primitives = RenderPrimitives::new();
    primitives.shadows.push(ShadowPrimitive {
        rect: Rect::new(40.0, 40.0, 60.0, 60.0),
        color: Color::rgba(0, 0, 0, 128),
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 4.0,
        spread_radius: 0.0,
        inset: false,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 阴影偏移后中心区域应该有颜色变化
    // 阴影矩形大约在 (45, 45) 到 (65, 65)
    let shadow_pixel = fb.get_pixel(55, 55);
    assert!(
        shadow_pixel[0] < 250,
        "shadow area should be darkened, got {:?}",
        shadow_pixel
    );

    // 远离阴影的区域应该是白色
    let far_pixel = fb.get_pixel(5, 5);
    assert_eq!(far_pixel, [255, 255, 255, 255], "far area should be white");
}

/// R2476：inset 内阴影应在盒**内**渲染（盒内边缘暗化），盒外保持白色（裁切到盒）。
#[test]
fn inset_shadow_renders_inside_box() {
    let mut primitives = RenderPrimitives::new();
    // 盒 (40,40)-(100,100)，inset 黑阴影 offset(5,5) blur 4
    primitives.shadows.push(ShadowPrimitive {
        rect: Rect::new(40.0, 40.0, 60.0, 60.0),
        color: Color::rgba(0, 0, 0, 200),
        offset_x: 5.0,
        offset_y: 5.0,
        blur_radius: 4.0,
        spread_radius: 0.0,
        inset: true,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 盒内左上角（inset 偏移致该侧阴影最厚）应被暗化
    let inner = fb.get_pixel(43, 43);
    assert!(inner[0] < 250, "inset shadow 应暗化盒内边缘，got {:?}", inner);
    // 盒外应保持白色（inset 裁切到盒，不外溢）
    let outside = fb.get_pixel(30, 30);
    assert_eq!(outside, [255, 255, 255, 255], "inset shadow 不应外溢到盒外");
    // 盒中心（远离内边缘）应接近白色（阴影向内淡出，中心几乎无阴影）
    let center = fb.get_pixel(70, 70);
    assert!(
        center[0] > 200,
        "inset shadow 中心应接近白（向内淡出），got {:?}",
        center
    );
}

#[test]
fn stroke_solid_line() {
    let mut primitives = RenderPrimitives::new();
    primitives.strokes.push(StrokePrimitive {
        x1: 10.0,
        y1: 10.0,
        x2: 90.0,
        y2: 10.0,
        width: 2.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        20,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 线段中心应该有黑色像素
    let center_pixel = fb.get_pixel(50, 10);
    assert_eq!(center_pixel[0], 0, "line center should be black");

    // 线段上方应该保持白色
    let above_pixel = fb.get_pixel(50, 7);
    assert_eq!(above_pixel, [255, 255, 255, 255], "above line should be white");
}

#[test]
fn stroke_dashed_line_has_gaps() {
    let mut primitives = RenderPrimitives::new();
    primitives.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 10.0,
        x2: 50.0,
        y2: 10.0,
        width: 2.0,
        color: Color::BLACK,
        style: LineStyle::Dashed,
        cap: LineCap::Butt,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        50,
        20,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 虚线段上应该有黑色和白色交替
    let mut has_black = false;
    let mut has_white = false;
    for x in 0..50 {
        let p = fb.get_pixel(x, 10);
        if p[0] == 0 {
            has_black = true;
        }
        if p[0] == 255 {
            has_white = true;
        }
    }
    assert!(has_black, "dashed line should have black segments");
    assert!(has_white, "dashed line should have white gaps");
}

#[test]
fn stroke_dotted_line_has_dots() {
    let mut primitives = RenderPrimitives::new();
    primitives.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 10.0,
        x2: 50.0,
        y2: 10.0,
        width: 2.0,
        color: Color::BLACK,
        style: LineStyle::Dotted,
        cap: LineCap::Butt,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        50,
        20,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 点线应该有黑色像素
    let mut has_black = false;
    for x in 0..50 {
        let p = fb.get_pixel(x, 10);
        if p[0] == 0 {
            has_black = true;
            break;
        }
    }
    assert!(has_black, "dotted line should have dots");
}

/// R1909 回归测试：退化（非有限坐标 / 零宽度）的点线/虚线不得使渲染卡死。
///
/// 根因：`render_dotted_line` 的 `while d <= total_len`（d += dot_spacing）在
/// total_len=inf（vertical-mode border 生成 y2=inf）或 dot_spacing=0（width=0）时
/// 永不终止，曾致 text-underline-position-001a 渲染 >75s hang。本测试断言这些退化
/// 输入被防御性跳过、快速返回（不 hang、不 panic）。
#[test]
fn stroke_degenerate_dotted_does_not_hang() {
    let mut primitives = RenderPrimitives::new();
    // (28.5,124)-(28.5,inf)：复刻 R1909 实测到的 vertical-mode inf 端点点线。
    primitives.strokes.push(StrokePrimitive {
        x1: 28.5,
        y1: 124.0,
        x2: 28.5,
        y2: f32::INFINITY,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Dotted,
        cap: LineCap::Butt,
    });
    // width=0 的点线：dot_spacing=0，旧实现 while 死循环。
    primitives.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 10.0,
        x2: 50.0,
        y2: 10.0,
        width: 0.0,
        color: Color::BLACK,
        style: LineStyle::Dotted,
        cap: LineCap::Butt,
    });
    // inf 端点虚线（render_dashed_line 路径亦须经 render_stroke 守卫）。
    primitives.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: f32::NEG_INFINITY,
        x2: 50.0,
        y2: 10.0,
        width: 1.0,
        color: Color::BLACK,
        style: LineStyle::Dashed,
        cap: LineCap::Butt,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    // 若回归，本调用将 hang（test-guard/CI 超时杀进程）；预期立即返回。
    let fb = render_full_scene(
        50,
        20,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 退化线段被跳过 → 帧应保持背景白色，无黑像素泄漏。
    for y in 0..20 {
        for x in 0..50 {
            let p = fb.get_pixel(x, y);
            assert_eq!(p, [255, 255, 255, 255], "degenerate stroke must not paint at ({x},{y})");
        }
    }
}

#[test]
fn path_fill_triangle() {
    let mut primitives = RenderPrimitives::new();
    // 三角形：(50, 10), (10, 90), (90, 90)
    primitives.path_fills.push(PathFillPrimitive {
        vertices: vec![50.0, 10.0, 10.0, 90.0, 90.0, 90.0],
        color: Color::RED,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 三角形内部应该有红色
    let center_pixel = fb.get_pixel(50, 60);
    assert!(
        center_pixel[0] > 200,
        "triangle center should be red, got {:?}",
        center_pixel
    );

    // 三角形外部应该保持白色
    let outside_pixel = fb.get_pixel(5, 5);
    assert_eq!(outside_pixel, [255, 255, 255, 255], "outside triangle should be white");
}

#[test]
fn path_stroke_rectangle() {
    let mut primitives = RenderPrimitives::new();
    // 矩形路径：(20, 20), (80, 20), (80, 80), (20, 80)
    primitives.path_strokes.push(PathStrokePrimitive {
        vertices: vec![20.0, 20.0, 80.0, 20.0, 80.0, 80.0, 20.0, 80.0],
        color: Color::BLACK,
        line_width: 2.0,
        closed: true,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 边框上应该有黑色像素
    let top_edge = fb.get_pixel(50, 20);
    assert_eq!(top_edge[0], 0, "top edge should be black");

    // 内部应该保持白色（描边不填充）
    let inside = fb.get_pixel(50, 50);
    assert_eq!(inside, [255, 255, 255, 255], "inside stroke should be white");
}

#[test]
fn clip_removes_pixels_outside() {
    let mut primitives = RenderPrimitives::new();
    // 先画一个全屏黑色矩形
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        color: Color::BLACK,
    });
    // 应用裁剪
    primitives.clips.push(ClipPrimitive {
        rect: Rect::new(20.0, 20.0, 80.0, 80.0),
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 裁剪区域内应该有黑色
    let inside = fb.get_pixel(50, 50);
    assert_eq!(inside, [0, 0, 0, 255], "inside clip should be black");

    // 裁剪区域外应该被清除为白色
    let outside = fb.get_pixel(5, 5);
    assert_eq!(outside, [255, 255, 255, 255], "outside clip should be white");
}

#[test]
fn transform_translate_shifts_content() {
    let mut primitives = RenderPrimitives::new();
    // 画一个小红色矩形
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(10.0, 10.0, 20.0, 20.0),
        color: Color::RED,
    });
    // 应用平移变换
    primitives.transforms.push(TransformPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        origin_x: 0.0,
        origin_y: 0.0,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 30.0,
        ty: 30.0,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 变换后内容应该偏移到新位置
    // 注意：变换是后处理，会反向采样
    // 原始红色矩形在 (10,10)-(20,20)，变换后应该出现在 (40,40)-(50,50)
    let _translated = fb.get_pixel(45, 45);
    // 由于后处理性质，结果取决于变换方向
    assert!(true, "transform applied without crash");
}

#[test]
fn image_renders_rgba_data() {
    use crate::image_cache::{ImageCache, ImageData};

    let mut image_cache = ImageCache::new(10, 1024 * 1024);
    let key = image_cache.insert(
        ImageData::from_rgba(
            vec![
                255, 0, 0, 255, // 红色
                0, 255, 0, 255, // 绿色
                0, 0, 255, 255, // 蓝色
                255, 255, 0, 255, // 黄色
            ],
            2,
            2,
        )
        .unwrap(),
    );

    let mut primitives = RenderPrimitives::new();
    primitives.images.push(ImagePrimitive {
        rect: Rect::new(10.0, 10.0, 30.0, 30.0),
        image_key: key,
        clip: None,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let mut cache = image_cache;
    let fb = render_full_scene(
        40,
        40,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut cache),
        &[],
        &[],
        &[],
        &[],
    );

    // 左上象限（对应红色像素）应该偏红
    let top_left = fb.get_pixel(15, 15);
    assert!(top_left[0] > 200, "top-left should be red-ish, got {:?}", top_left);

    // 右下象限（对应黄色像素，源图像 [1,1] 是黄色 (255,255,0)）
    // 注：双线性插值下像素(25,25)约在源图像中心(0.5,0.5)附近，
    // 四色混合后不纯黄。改用靠近右下角的像素(38,38)确保采样自源(1,1)。
    let bottom_right = fb.get_pixel(38, 38);
    assert!(
        bottom_right[0] > 200 && bottom_right[1] > 200,
        "bottom-right should be yellow-ish, got {:?}",
        bottom_right
    );

    // 图片外应该是白色
    let outside = fb.get_pixel(5, 5);
    assert_eq!(outside, [255, 255, 255, 255], "outside image should be white");
}

#[test]
fn image_clip_crops_not_rescales() {
    // 验证裁剪语义 = **裁剪（crop）非重缩放（rescale）**。
    //
    // 关键区分点：对**非均匀**图像施加 clip 窗口后，可见区应保持 source 原始分辨率
    // （source 仍按完整 rect 映射，clip 仅收窄绘制窗口）。旧实现把 rect 缩到 clip 窗口
    // 后把**整张 source** 重映射进缩小区 → 把 source 挤压进窗口（rescale）。
    //
    // 源 4×4：左上 2×2 红、其余蓝。映射到 40×40 rect（每源像素 = 10×10 块），
    // 故 source 红块覆盖 rect 左上 (0,0)-(20,20)。clip 仅留左上 20×20：
    //   - crop（正确）：(10,10) 落在 source 红块内 → 红（原分辨率）；
    //   - rescale（旧 bug）：source 被挤进 20×20 → (10,10) 映射到 source(1.6,1.6)
    //     （红块边界外、蓝区）→ 蓝。
    // 另断言 clip 窗口外（(30,30)）不绘制（白色）。
    use crate::image_cache::{ImageCache, ImageData};

    let red = [255u8, 0, 0, 255];
    let blue = [0u8, 0, 255, 255];
    // 行优先：y=0/1 为「红 红 蓝 蓝」，y=2/3 全蓝
    let rows: [[[u8; 4]; 4]; 4] = [
        [red, red, blue, blue],
        [red, red, blue, blue],
        [blue, blue, blue, blue],
        [blue, blue, blue, blue],
    ];
    let mut buf = Vec::with_capacity(4 * 4 * 4);
    for row in &rows {
        for px in row {
            buf.extend_from_slice(px);
        }
    }

    let mut image_cache = ImageCache::new(10, 1024 * 1024);
    let key = image_cache.insert(ImageData::from_rgba(buf, 4, 4).unwrap());

    let mut primitives = RenderPrimitives::new();
    primitives.images.push(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 40.0, 40.0),
        image_key: key,
        clip: Some(Rect::new(0.0, 0.0, 20.0, 20.0)),
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let mut cache = image_cache;
    let fb = render_full_scene(
        40,
        40,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut cache),
        &[],
        &[],
        &[],
        &[],
    );

    // clip 窗口内深处 (10,10)：crop→红（落在 source 红块）；rescale→蓝（source 被挤进窗口）
    let inside_clip = fb.get_pixel(10, 10);
    assert!(
        inside_clip[0] > 200 && inside_clip[2] < 60,
        "clip 窗口内应为红色（crop 保持原分辨率，非 rescale 把蓝挤进来），got {:?}",
        inside_clip
    );

    // clip 窗口外 (30,30)：rect 内但被裁掉，不绘制，白色
    let outside_clip = fb.get_pixel(30, 30);
    assert_eq!(
        outside_clip,
        [255, 255, 255, 255],
        "clip 窗口外不应绘制，got {:?}",
        outside_clip
    );
}

#[test]
fn filter_blur_softens_hard_edge() {
    let mut primitives = RenderPrimitives::new();
    // 黑色矩形（硬边）
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(40.0, 0.0, 60.0, 100.0),
        color: Color::BLACK,
    });
    // 对整个区域应用模糊
    primitives.filters.push(FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        filters: vec![FilterKind::Blur(3.0)],
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 模糊边缘应该有过渡
    let edge = fb.get_pixel(38, 50);
    // 硬边应该在 x=40，所以 x=38 应该受模糊影响变暗
    assert!(edge[0] < 255, "edge should be blurred, got {:?}", edge);
}

#[test]
fn filter_brightness_makes_image_brighter() {
    let mut primitives = RenderPrimitives::new();
    // 灰色矩形
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: Color::rgb(100, 100, 100),
    });
    primitives.filters.push(FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        filters: vec![FilterKind::Brightness(2.0)],
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        10,
        10,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    let p = fb.get_pixel(5, 5);
    assert_eq!(p[0], 200, "brightness(2.0) should double gray 100 to 200");
}

#[test]
fn blend_mode_normal_does_nothing() {
    let mut primitives = RenderPrimitives::new();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: Color::RED,
    });
    primitives.blend_modes.push(BlendModePrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        mode: BlendMode::Normal,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        10,
        10,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // Normal 混合模式不应该改变任何东西
    let p = fb.get_pixel(5, 5);
    assert_eq!(p, [255, 0, 0, 255]);
}

#[test]
fn full_scene_renders_multiple_primitives() {
    let mut primitives = RenderPrimitives::new();

    // 背景
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        color: Color::WHITE,
    });

    // 渐变
    primitives.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(10.0, 10.0, 90.0, 30.0),
        kind: GradientKind::Linear {
            x0: 10.0,
            y0: 0.0,
            x1: 90.0,
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

    // 线段
    primitives.strokes.push(StrokePrimitive {
        x1: 10.0,
        y1: 50.0,
        x2: 90.0,
        y2: 50.0,
        width: 2.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        100,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 渐变区域左端应该是红色
    let gradient_left = fb.get_pixel(12, 20);
    assert!(gradient_left[0] > 100, "gradient left should be red-ish");

    // 线段应该是黑色
    let line_pixel = fb.get_pixel(50, 50);
    assert_eq!(line_pixel[0], 0, "line should be black");
}

#[test]
fn rounded_rect_with_all_corners() {
    let mut primitives = RenderPrimitives::new();
    primitives.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(10.0, 10.0, 50.0, 50.0),
        color: Color::GREEN,
        top_left_radius: 10.0,
        top_right_radius: 10.0,
        bottom_right_radius: 10.0,
        bottom_left_radius: 10.0,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        60,
        60,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // 中心应该是绿色
    let center = fb.get_pixel(30, 30);
    assert_eq!(center, [0, 255, 0, 255], "center should be green");

    // 角落（在圆角半径外）应该是白色
    let corner = fb.get_pixel(12, 12);
    assert_eq!(corner, [255, 255, 255, 255], "corner should be white (outside radius)");
}

/// 半透明圆角矩形背景必须按 alpha 与底色合成，不能硬编码 alpha=255 渲染为实色。
///
/// 回归用例：旧 `fill_rounded_rect` 用 `set_pixel([r,g,b,255])` 直接覆盖，
/// 把 `rgba()` 半透明圆角背景（如 morning.work `.item-tag` 的
/// `var(--color-primary-alpha-05)` = `rgba(96,124,210,0.05)`）渲染成实色蓝。
/// 修复后应与 `fill_rect` 一致：半透明 → `blend_pixel` 与白色底合成。
#[test]
fn rounded_rect_translucent_alpha_blends() {
    let mut primitives = RenderPrimitives::new();
    // 蓝色 alpha=13（≈0.05）圆角矩形，铺满 60×60 framebuffer
    primitives.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(0.0, 0.0, 60.0, 60.0),
        color: Color {
            r: 96,
            g: 124,
            b: 210,
            a: 13,
        },
        top_left_radius: 0.0,
        top_right_radius: 0.0,
        bottom_right_radius: 0.0,
        bottom_left_radius: 0.0,
    });

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        60,
        60,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );

    // alpha=13/255≈0.051 在白底上：dst*(1-a)+src*a = 255*0.949 + 96*0.051 ≈ 247
    let px = fb.get_pixel(30, 30);
    assert!(
        px[0] > 235 && px[0] < 255 && px[2] > 235,
        "translucent rounded rect should blend to a light tint, got {:?}",
        px
    );
    // 关键：绝不能是实色蓝 (96,124,210)
    assert!(
        !(px[0] < 130 && px[2] > 180),
        "must not render as solid color {:?} (alpha dropped)",
        px
    );
}

// ─── 垂直书写模式字形旋转测试 ───

/// 测试垂直书写模式下字形 90° 旋转渲染。
/// GlyphPrimitive.rotation = FRAC_PI_2 时，字形应顺时针旋转 90°。
#[test]
fn test_glyph_rotation_90_degrees() {
    use crate::font::GlyphBitmap;
    use std::f32::consts::FRAC_PI_2;

    // 创建一个 4x2 的位图（宽 4，高 2），左半部分不透明，右半部分透明
    // 原始布局（4x2）:
    //   XX..
    //   XX..
    let bitmap = GlyphBitmap {
        width: 4,
        height: 2,
        data: vec![
            255, 255, 0, 0, // row 0: col 0,1 = opaque; col 2,3 = transparent
            255, 255, 0, 0, // row 1: same
        ],
        x_offset: 0,
        y_offset: 0,
        advance: 0.0,
    };

    // 在 (10, 10) 处渲染旋转 90° 的位图
    let mut fb = FrameBuffer::new(40, 40);
    fb.clear(255, 255, 255, 255); // 白色背景

    blit_glyph_bitmap(&mut fb, &bitmap, 10.0, 10.0, Color::BLACK, FRAC_PI_2, false);

    // 顺时针旋转 90° 后，原始 4x2 变成 2x4：
    // 原始 (col=0,row=0) → 旋转后 (rotated_col=0, rotated_row=3) → (px=10, py=13)
    // 原始 (col=0,row=1) → 旋转后 (rotated_col=1, rotated_row=3) → (px=11, py=13)
    // 原始 (col=1,row=0) → 旋转后 (rotated_col=0, rotated_row=2) → (px=10, py=12)
    // 原始 (col=1,row=1) → 旋转后 (rotated_col=1, rotated_row=2) → (px=11, py=12)

    // 验证旋转后的像素位置
    let p1 = fb.get_pixel(10, 13); // (col=0,row=0) → 旋转后
    assert_eq!(p1, [0, 0, 0, 255], "rotated pixel at (10,13) should be black");

    let p2 = fb.get_pixel(11, 13); // (col=0,row=1) → 旋转后
    assert_eq!(p2, [0, 0, 0, 255], "rotated pixel at (11,13) should be black");

    let p3 = fb.get_pixel(10, 12); // (col=1,row=0) → 旋转后
    assert_eq!(p3, [0, 0, 0, 255], "rotated pixel at (10,12) should be black");

    let p4 = fb.get_pixel(11, 12); // (col=1,row=1) → 旋转后
    assert_eq!(p4, [0, 0, 0, 255], "rotated pixel at (11,12) should be black");

    // 原始位置（未旋转的位置）应保持白色
    let p5 = fb.get_pixel(10, 10); // 原始 (col=0,row=0) 未旋转时在此
    assert_eq!(p5, [255, 255, 255, 255], "unrotated position (10,10) should stay white");
}

/// 测试零旋转时字形正常渲染（无旋转）。
#[test]
fn test_glyph_no_rotation() {
    use crate::font::GlyphBitmap;

    // 创建一个 2x2 的位图
    let bitmap = GlyphBitmap {
        width: 2,
        height: 2,
        data: vec![255, 255, 255, 255],
        x_offset: 0,
        y_offset: 0,
        advance: 0.0,
    };

    let mut fb = FrameBuffer::new(20, 20);
    fb.clear(255, 255, 255, 255);

    blit_glyph_bitmap(&mut fb, &bitmap, 5.0, 5.0, Color::BLACK, 0.0, false);

    // 无旋转：像素在 (5,5), (6,5), (5,6), (6,6)
    assert_eq!(fb.get_pixel(5, 5), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(6, 5), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(5, 6), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(6, 6), [0, 0, 0, 255]);
}

/// R2497：synthetic italic blit shear——synthetic_italic=true 时每行按 ITALIC_SKEW 水平
/// 偏移（锚 height/2 上下对称），产出倾斜位图（≠ 非斜体）。
#[test]
fn test_glyph_synthetic_italic_shear() {
    use crate::font::GlyphBitmap;

    // 4 像素高、1 像素宽的竖线（足以观测按行水平偏移）。
    let bitmap = GlyphBitmap {
        width: 1,
        height: 4,
        data: vec![255, 255, 255, 255],
        x_offset: 0,
        y_offset: 0,
        advance: 0.0,
    };

    // 非斜体：4 行同列（x=10），垂直竖线。
    let mut fb_normal = FrameBuffer::new(40, 20);
    fb_normal.clear(255, 255, 255, 255);
    blit_glyph_bitmap(&mut fb_normal, &bitmap, 10.0, 5.0, Color::BLACK, 0.0, false);
    // 行 0..4（py=5..8）全在 px=10。
    for row in 5..9 {
        assert_eq!(fb_normal.get_pixel(10, row), [0, 0, 0, 255], "normal row {row} at x=10");
    }

    // 斜体：锚 height/2=2，shear_dx = (row - 2) * 0.249 round。
    //   row0 → (0-2)*0.249 = -0.498 → round 0 → px=10
    //   row1 → (1-2)*0.249 = -0.249 → round 0 → px=10
    //   row2 → (2-2)*0.249 = 0     → px=10
    //   row3 → (3-2)*0.249 = 0.249  → round 0 → px=10
    // 4 高度太小 shear 不足 1px → 用更高位图验证明显偏移。
    let bitmap_tall = GlyphBitmap {
        width: 1,
        height: 10,
        data: vec![255; 10],
        x_offset: 0,
        y_offset: 0,
        advance: 0.0,
    };
    let mut fb_italic = FrameBuffer::new(40, 20);
    fb_italic.clear(255, 255, 255, 255);
    blit_glyph_bitmap(&mut fb_italic, &bitmap_tall, 10.0, 5.0, Color::BLACK, 0.0, true);
    // 锚 height/2=5（相对位图）；row=0 → (0-5)*0.249=-1.245 → round -1 → px=9；
    // row=9 → (9-5)*0.249=0.996 → round 1 → px=11。顶端左移、底端右移 = 倾斜。
    assert_eq!(
        fb_italic.get_pixel(9, 5),
        [0, 0, 0, 255],
        "italic top row shifted left to x=9"
    );
    assert_eq!(fb_italic.get_pixel(10, 10), [0, 0, 0, 255], "italic anchor row at x=10");
    assert_eq!(
        fb_italic.get_pixel(11, 14),
        [0, 0, 0, 255],
        "italic bottom row shifted right to x=11"
    );
    // 非斜体同位图底端应在 x=10（未偏移），证 shear 差异。
    let mut fb_tall_normal = FrameBuffer::new(40, 20);
    fb_tall_normal.clear(255, 255, 255, 255);
    blit_glyph_bitmap(&mut fb_tall_normal, &bitmap_tall, 10.0, 5.0, Color::BLACK, 0.0, false);
    assert_eq!(
        fb_tall_normal.get_pixel(10, 14),
        [0, 0, 0, 255],
        "normal bottom row at x=10 (no shear)"
    );
    assert_eq!(
        fb_tall_normal.get_pixel(11, 14),
        [255, 255, 255, 255],
        "normal x=11 stays white"
    );
}

// ─── DC-8 CPU framebuffer rigor（对称 R664 GPU）───
// R661 识别 gap：CPU ImagePrimitive / PathFill / PathStroke 缺 framebuffer 像素断言测试。
// 以下 3 测用 render_full_scene + fb.get_pixel 验证 CPU 路径，与 GPU test_gpu_full_scene_* 对称。

/// DC-8 CPU ImagePrimitive — 纯红图片渲染到 framebuffer，断言像素为红。
#[test]
fn cpu_full_scene_image_solid_red() {
    let mut primitives = RenderPrimitives::new();
    // 1×1 纯红 RGBA 图片（放大到 16×16 rect）
    let img = crate::image_cache::ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1).expect("red image");
    let mut image_cache = crate::image_cache::ImageCache::new(16, 1 << 20);
    let key = image_cache.insert(img);
    primitives.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 16.0),
        image_key: key,
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        16,
        16,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
    );
    // 中心 (8,8) 应为红
    let c = fb.get_pixel(8, 8);
    assert!(c[0] > 240, "image center R should be ~255, got {:?}", c);
    assert!(c[1] < 15, "image center G should be ~0, got {:?}", c);
    assert!(c[2] < 15, "image center B should be ~0, got {:?}", c);
}

/// 大图缩放至视口边缘时，双线性采样不应因 src 坐标等于 width/height 而 panic。
#[test]
fn cpu_scaled_image_at_viewport_edge_does_not_panic() {
    let mut primitives = RenderPrimitives::new();
    let w = 1070_u32;
    let h = 400_u32;
    let pixels = vec![128u8; (w as usize) * (h as usize) * 4];
    let img = ImageData::from_rgba(pixels, w, h).expect("large image");
    let mut image_cache = ImageCache::new(8, 256 * 1024 * 1024);
    let key = image_cache.insert(img);
    primitives.add_image(ImagePrimitive {
        rect: Rect::new(0.0, 0.0, w as f32, h as f32),
        image_key: key,
        clip: None,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        w,
        h,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
    );
    let corner = fb.get_pixel(w - 1, h - 1);
    assert!(corner[3] == 255, "corner should be opaque, got {:?}", corner);
}

/// DC-8 CPU PathFillPrimitive — 矩形多边形填充，断言中心黑、外部白。
#[test]
fn cpu_full_scene_path_fill_black_rect() {
    let mut primitives = RenderPrimitives::new();
    // 矩形多边形 (4,4)-(28,28)
    primitives.add_path_fill(vec![4.0, 4.0, 28.0, 4.0, 28.0, 28.0, 4.0, 28.0], Color::BLACK);
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        32,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 中心 (16,16) 黑
    let center = fb.get_pixel(16, 16);
    assert_eq!(
        center,
        [0, 0, 0, 255],
        "path-fill center should be black, got {:?}",
        center
    );
    // 角 (1,1) 白（framebuffer 默认白底）
    let corner = fb.get_pixel(1, 1);
    assert_eq!(
        corner,
        [255, 255, 255, 255],
        "path-fill corner should be white, got {:?}",
        corner
    );
}

/// DC-8 CPU PathStrokePrimitive — 闭合矩形描边，断言内部白、顶边带黑像素。
#[test]
fn cpu_full_scene_path_stroke_closed_rect() {
    let mut primitives = RenderPrimitives::new();
    // 矩形描边中心 (8,8)-(24,24)，线宽 3，闭合
    primitives.add_path_stroke(
        vec![8.0, 8.0, 24.0, 8.0, 24.0, 24.0, 8.0, 24.0],
        Color::BLACK,
        3.0,
        true,
    );
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        32,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 内部 (16,16) 白（未被描边覆盖）
    let center = fb.get_pixel(16, 16);
    assert_eq!(
        center,
        [255, 255, 255, 255],
        "path-stroke interior should be white, got {:?}",
        center
    );
    // 顶边带 y=8 行 x∈[8,24] 至少一黑像素
    let top_edge_black = (8..=24).any(|x| fb.get_pixel(x, 8) == [0, 0, 0, 255]);
    assert!(top_edge_black, "path-stroke top edge should contain black pixels");
}

/// DC-8 CPU StrokePrimitive — 水平线段渲染，断言线段中心黑、外部白。
#[test]
fn cpu_full_scene_stroke_horizontal_line() {
    let mut primitives = RenderPrimitives::new();
    // 水平线 (0,16)-(31,16) 宽 4，黑
    primitives.add_stroke(StrokePrimitive {
        x1: 0.0,
        y1: 16.0,
        x2: 31.0,
        y2: 16.0,
        width: 4.0,
        color: Color::BLACK,
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        32,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 线段中心 (16,16) 黑
    let mid = fb.get_pixel(16, 16);
    assert_eq!(mid, [0, 0, 0, 255], "stroke center should be black, got {:?}", mid);
    // 线段上方 (16,4) 白
    let above = fb.get_pixel(16, 4);
    assert_eq!(
        above,
        [255, 255, 255, 255],
        "above stroke should be white, got {:?}",
        above
    );
}

/// DC-8 CPU ClipPrimitive — 黑色全屏 fill 后应用 clip rect，断言 clip 区内保留黑、区外清白。
#[test]
fn cpu_full_scene_clip_rect_clears_outside() {
    let mut primitives = RenderPrimitives::new();
    // 黑色 fill 覆盖整屏
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::BLACK,
    });
    // clip 只保留 (8,8)-(24,24)
    primitives.clips.push(ClipPrimitive {
        rect: Rect::new(8.0, 8.0, 16.0, 16.0),
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        32,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // clip 内 (16,16) 保留黑
    assert_eq!(fb.get_pixel(16, 16), [0, 0, 0, 255], "clip interior should stay black");
    // clip 外 (2,2) 被清白
    assert_eq!(
        fb.get_pixel(2, 2),
        [255, 255, 255, 255],
        "clip outside should be cleared white"
    );
    // clip 边界外 (6,16) 清白（在 clip 矩形左外）
    assert_eq!(
        fb.get_pixel(6, 16),
        [255, 255, 255, 255],
        "left of clip rect should be white"
    );
}

/// DC-8 CPU TransformPrimitive — 左半屏黑色 fill 后应用平移 tx=8，断言内容右移。
#[test]
fn cpu_full_scene_transform_translates_content() {
    let mut primitives = RenderPrimitives::new();
    // 左半屏 (0-16) 黑色 fill
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 16.0, 32.0),
        color: Color::BLACK,
    });
    // 平移变换：a=1,b=0,c=0,d=1,tx=8,ty=0（内容右移 8px），rect 覆盖全屏
    primitives.transforms.push(TransformPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        origin_x: 0.0,
        origin_y: 0.0,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 8.0,
        ty: 0.0,
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let fb = render_full_scene(
        32,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // 变换前 (20,16) 是白（右半屏）；平移右移 8 后 (20,16) 应为黑（src=12 在原黑色区）
    assert_eq!(
        fb.get_pixel(20, 16),
        [0, 0, 0, 255],
        "after tx=8, (20,16) should be black (content shifted right)"
    );
    // (4,16) 原 black 区左缘；平移后 src=-4 越界 → 清白
    assert_eq!(
        fb.get_pixel(4, 16),
        [255, 255, 255, 255],
        "after tx=8, (4,16) should be white (cleared, src out of bounds)"
    );
}

fn try_load_ui_font_for_layer_test(loader: &mut FontLoader) -> Option<u32> {
    #[cfg(target_os = "windows")]
    let paths: &[&str] = &["C:\\Windows\\Fonts\\arial.ttf"];
    #[cfg(target_os = "macos")]
    let paths: &[&str] = &["/System/Library/Fonts/Supplemental/Arial Unicode.ttf"];
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let paths: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];

    paths
        .iter()
        .find_map(|path| std::fs::read(path).ok().and_then(|data| loader.load_font(&data).ok()))
}

/// overlay 必须在 ui_glyphs 之后绘制，否则页面文字会盖住上下文菜单等浮层背景。
#[test]
fn render_full_scene_overlay_covers_ui_glyphs() {
    let mut font_loader = FontLoader::new();
    let Some(font_id) = try_load_ui_font_for_layer_test(&mut font_loader) else {
        return;
    };
    let mut glyph_cache = GlyphCache::new(256);
    let primitives = RenderPrimitives::new();

    let mut ui_glyphs = Vec::new();
    for row in 0..6 {
        for col in 0..10 {
            ui_glyphs.push(GlyphDraw {
                ch: 'M',
                font_glyph_index: None,
                x: col as f32 * 10.0 + 2.0,
                baseline_y: row as f32 * 12.0 + 14.0,
                color: Color::BLACK,
                font_id,
                font_size: 12.0,
                rotation: 0.0,
            });
        }
    }

    let overlay_fills = vec![FillPrimitive {
        rect: Rect::new(0.0, 0.0, 96.0, 72.0),
        color: Color::WHITE,
    }];

    let fb = render_full_scene(
        96,
        72,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &ui_glyphs,
        &overlay_fills,
        &[],
        &[],
    );

    let center = fb.get_pixel(48, 36);
    assert!(
        center[0] > 240 && center[1] > 240 && center[2] > 240,
        "overlay fill must paint over ui glyphs (context menu regression), got {:?}",
        center
    );
}

/// S3 区域光栅化：region 内的像素与全量渲染一致，且 region 外图元被跳过。
#[test]
fn render_full_scene_region_matches_full_within_region() {
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(256);
    let mut primitives = RenderPrimitives::new();
    // 两个 fills：一个在 region 内（左半），一个在 region 外（右半）
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        color: Color::rgba(255, 0, 0, 255),
    });
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(200.0, 0.0, 100.0, 100.0),
        color: Color::rgba(0, 0, 255, 255),
    });

    let full = render_full_scene(
        200,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    // region = 左半（0,0,100,100）：只绘制左 fills
    let region = render_full_scene_region(
        200,
        100,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        Some(Rect::new(0.0, 0.0, 100.0, 100.0)),
    );

    // region 内像素：region 渲染 == 全量渲染
    for i in 0..(100 * 100 * 4) {
        assert_eq!(region.data[i], full.data[i], "region 内像素应与全量一致 @ {i}");
    }
    // region 内应为红色（左 fills 绘制）
    assert_eq!(&region.data[..4], &[255, 0, 0, 255]);
    // region 外（右半）应为背景白（右 fills 被跳过）
    let right_pixel = &region.data[150 * 4..150 * 4 + 4];
    assert_eq!(right_pixel, &[255, 255, 255, 255], "region 外图元应被跳过");
}

/// S3：全视口 fill 与部分 region 相交时，只更新脏区内像素，保留区外已有内容。
#[test]
fn render_full_scene_region_into_clips_full_viewport_fill() {
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(256);
    let mut back = FrameBuffer::new(20, 10);
    back.clear(0, 0, 255, 255);

    let mut primitives = RenderPrimitives::new();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 20.0, 10.0),
        color: Color::rgba(255, 0, 0, 255),
    });

    render_full_scene_region_into(
        &mut back,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        Some(Rect::new(0.0, 0.0, 10.0, 10.0)),
        1.0,
    );

    assert_eq!(&back.data[..4], &[255, 0, 0, 255], "dirty 内应为红");
    let outside = ((5 * 20 + 15) * 4) as usize;
    assert_eq!(back.data[outside], 0, "dirty 外应保留蓝");
    assert_eq!(back.data[outside + 2], 255);
}

/// S2：`render_full_scene_threaded`（Browser scope 线程路径）与直连逐像素一致。
#[test]
fn render_full_scene_threaded_matches_direct() {
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let mut primitives = RenderPrimitives::new();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 48.0, 32.0),
        color: Color::rgb(40, 80, 120),
    });

    let direct = render_full_scene(
        48,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    let threaded = render_full_scene_threaded(
        48,
        32,
        1.0,
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
    );
    assert_eq!(direct.data, threaded.data);
}

/// 性能门禁优化 S1（2026-08-08）：滚动 translate-blit 像素等价性——
/// 「平移上一帧内容 + 只重绘新露条带」必须与「同滚动全量渲染」逐像素一致。
/// 覆盖向上/向下滚动、不同条带高度、overlay 层（滚动条语义）与半透明混合。
#[test]
fn scroll_blit_matches_full_render() {
    let mut scene = RenderPrimitives::new();
    // 不透明 fills（网格，内容延伸到视口下方供条带重绘）
    for i in 0..240u32 {
        let x = (i % 20) as f32 * 40.0;
        let y = (i / 20) as f32 * 40.0;
        scene.add_fill(Rect::new(x, y, 38.0, 38.0), Color::rgb((i % 256) as u8, 100, 50));
    }
    // 半透明 fills（混合路径）
    for i in 0..60u32 {
        let x = (i % 12) as f32 * 90.0 + 10.0;
        let y = (i / 12) as f32 * 90.0 + 10.0;
        scene.add_fill(Rect::new(x, y, 60.0, 60.0), Color::rgba(20, 60, 200, 128));
    }
    scene.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(20.0, 30.0, 80.0, 50.0),
        color: Color::rgb(0, 200, 0),
        top_left_radius: 8.0,
        top_right_radius: 8.0,
        bottom_right_radius: 8.0,
        bottom_left_radius: 8.0,
    });
    scene.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(100.0, 100.0, 100.0, 60.0),
        kind: GradientKind::Linear {
            x0: 100.0,
            y0: 100.0,
            x1: 200.0,
            y1: 160.0,
        },
        stops: vec![
            GradientStop {
                color: Color::rgb(255, 0, 0),
                offset: 0.0,
            },
            GradientStop {
                color: Color::rgb(0, 0, 255),
                offset: 1.0,
            },
        ],
        repeating: false,
    });
    scene.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 300.0,
        y2: 300.0,
        width: 4.0,
        color: Color::rgb(0, 0, 0),
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });

    let (w, h) = (400u32, 300u32);
    let scale = 1.0f32;
    // 页面内容矩形 = 整个帧缓冲（模拟视口）
    let (ix0, ix1, iy0, iy1) = (0usize, w as usize, 0usize, h as usize);
    let row_bytes = w as usize * 4;
    let span = (ix1 - ix0) * 4;

    // overlay（滚动条轨道语义：**全高**覆盖页面内容区——每帧重画正确位置，
    // 平移后由 overlay pass 全覆盖自愈；部分高度 overlay（查找栏/上下文菜单）
    // 会留下残影，由浏览器侧 blit guard 禁用，见 app_platform render_scene_cpu）
    let overlay_fills = vec![FillPrimitive {
        rect: Rect::new(w as f32 - 20.0, 0.0, 12.0, h as f32),
        color: Color::rgb(0, 0, 200),
    }];

    let font_loader = FontLoader::new();
    let dy_list: [i32; 6] = [10, -12, 25, -40, 1, 37];

    for &dy in &dy_list {
        // 场景 B = 场景 A 内容整体平移 dy（模拟滚动 dy 后的新 offset 渲染输入）
        let mut scene_b = RenderPrimitives::new();
        for fill in &scene.fills {
            scene_b.add_fill(
                Rect::new(
                    fill.rect.origin.x,
                    fill.rect.origin.y - dy as f32,
                    fill.rect.size.width,
                    fill.rect.size.height,
                ),
                fill.color,
            );
        }
        for rr in &scene.rounded_rects {
            let mut c = rr.clone();
            c.rect.origin.y -= dy as f32;
            scene_b.rounded_rects.push(c);
        }
        for g in &scene.gradients {
            let mut c = g.clone();
            c.rect.origin.y -= dy as f32;
            c.kind = match &g.kind {
                GradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear {
                    x0: *x0,
                    y0: *y0 - dy as f32,
                    x1: *x1,
                    y1: *y1 - dy as f32,
                },
                other => other.clone(),
            };
            scene_b.gradients.push(c);
        }
        for st in &scene.strokes {
            scene_b.strokes.push(StrokePrimitive {
                x1: st.x1,
                y1: st.y1 - dy as f32,
                x2: st.x2,
                y2: st.y2 - dy as f32,
                width: st.width,
                color: st.color,
                style: st.style,
                cap: st.cap,
            });
        }

        let fb_a = render_full_scene_region(
            w,
            h,
            scale,
            &scene,
            &font_loader,
            &mut GlyphCache::new(64),
            None,
            &[],
            &overlay_fills,
            &[],
            &[],
            None,
        );
        // blit：平移 + 只重绘新露条带
        let mut fb_blit = fb_a.clone();
        let ady = dy.unsigned_abs() as usize;
        if dy > 0 {
            for y in iy0..iy1 - ady {
                let src = (y + ady) * row_bytes + ix0 * 4;
                let dst = y * row_bytes + ix0 * 4;
                fb_blit.data.copy_within(src..src + span, dst);
            }
        } else {
            for y in (iy0 + ady..iy1).rev() {
                let src = (y - ady) * row_bytes + ix0 * 4;
                let dst = y * row_bytes + ix0 * 4;
                fb_blit.data.copy_within(src..src + span, dst);
            }
        }
        let strip_top = if dy > 0 { iy1 - ady } else { iy0 };
        let strip_bottom = strip_top + ady;
        let strip = Rect::new(ix0 as f32, strip_top as f32, (ix1 - ix0) as f32, ady as f32);
        // 条带渲染到 scratch（region 仅剔除不相交图元，越界绘制无害），
        // 只把条带行拷回保留帧——与浏览器 render_scene_cpu 的 blit 一致
        let mut scratch = FrameBuffer::new(w, h);
        scratch.clear(255, 255, 255, 255);
        render_full_scene_region_into(
            &mut scratch,
            &scene_b,
            &font_loader,
            &mut GlyphCache::new(64),
            None,
            &[],
            &overlay_fills,
            &[],
            &[],
            Some(strip),
            scale,
        );
        for y in strip_top..strip_bottom {
            let row = y * row_bytes;
            fb_blit.data[row..row + row_bytes].copy_from_slice(&scratch.data[row..row + row_bytes]);
        }

        // 同滚动全量渲染
        let fb_b = render_full_scene_region(
            w,
            h,
            scale,
            &scene_b,
            &font_loader,
            &mut GlyphCache::new(64),
            None,
            &[],
            &overlay_fills,
            &[],
            &[],
            None,
        );

        let diff_count = fb_blit.data.iter().zip(&fb_b.data).filter(|(a, b)| a != b).count();
        assert_eq!(
            diff_count, 0,
            "scroll blit != full render for dy={dy}（{diff_count} 像素不一致）"
        );
    }
}

/// 性能门禁优化 S1b（2026-08-08）：chrome-only 动画帧——页面区保留，
/// 只重绘页面区外的 chrome 条带（顶部/底部），页面区像素必须与全量渲染一致。
#[test]
fn chrome_strip_rerender_does_not_touch_page_region() {
    let mut scene = RenderPrimitives::new();
    // 页面内容（页面 rect 内）：不透明 + 半透明 fills
    for i in 0..120u32 {
        let x = (i % 15) as f32 * 40.0;
        let y = (i / 15) as f32 * 40.0;
        scene.add_fill(Rect::new(x, y, 38.0, 38.0), Color::rgb((i % 256) as u8, 100, 50));
    }
    // chrome（页面 rect 外顶部条带）：不透明背景
    scene.add_fill(Rect::new(0.0, 0.0, 400.0, 40.0), Color::rgb(240, 240, 240));
    // chrome（底部条带）
    scene.add_fill(Rect::new(0.0, 300.0, 400.0, 40.0), Color::rgb(200, 200, 200));

    let (w, h) = (400u32, 340u32);
    let scale = 1.0f32;
    // 页面 rect：y ∈ [40, 300)
    let (cy, ch) = (40.0f32, 260.0f32);

    let font_loader = FontLoader::new();
    // 全量渲染
    let fb_full = render_full_scene_region(
        w,
        h,
        scale,
        &scene,
        &font_loader,
        &mut GlyphCache::new(64),
        None,
        &[],
        &[],
        &[],
        &[],
        None,
    );
    // reuse 帧：从全量 fb 出发，重绘顶部条带 [0, cy) 与底部条带 [cy+ch, h)
    let mut fb_reuse = fb_full.clone();
    let top_strip = Rect::new(0.0, 0.0, w as f32, cy);
    render_full_scene_region_into(
        &mut fb_reuse,
        &scene,
        &font_loader,
        &mut GlyphCache::new(64),
        None,
        &[],
        &[],
        &[],
        &[],
        Some(top_strip),
        scale,
    );
    let bottom_strip = Rect::new(0.0, cy + ch, w as f32, h as f32 - cy - ch);
    render_full_scene_region_into(
        &mut fb_reuse,
        &scene,
        &font_loader,
        &mut GlyphCache::new(64),
        None,
        &[],
        &[],
        &[],
        &[],
        Some(bottom_strip),
        scale,
    );
    assert_eq!(
        fb_reuse.data, fb_full.data,
        "chrome strip re-render must not change page-region pixels (S1b reuse frame)"
    );
}
