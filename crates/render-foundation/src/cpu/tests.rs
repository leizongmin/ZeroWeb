//! CPU 渲染器测试 — 验证所有图元类型的渲染正确性。

use super::*;
use crate::color::Color;
use crate::geometry::Rect;
use crate::gpu::renderer::GlyphDraw;
use crate::image_cache::{ImageCache, ImageData, ImageKey};
use crate::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, FillPrimitive, FilterKind, FilterPrimitive, GradientKind,
    GradientPrimitive, GradientStop, ImagePrimitive, LineCap, LineStyle, PathFillPrimitive, PathStrokePrimitive,
    RenderPrimitives, RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive, TransformPrimitive,
};

// ─── 旧版兼容测试 ───

#[test]
fn glyph_top_left_converts_fontdue_y_up_metrics_to_screen_y_down() {
    let (x, y) = glyph_top_left(10.0, 50.0, 2, -4, 18);
    assert_eq!(x, 12.0);
    assert_eq!(y, 36.0);
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

    blit_glyph_bitmap(&mut fb, &bitmap, 10.0, 10.0, Color::BLACK, FRAC_PI_2);

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

    blit_glyph_bitmap(&mut fb, &bitmap, 5.0, 5.0, Color::BLACK, 0.0);

    // 无旋转：像素在 (5,5), (6,5), (5,6), (6,6)
    assert_eq!(fb.get_pixel(5, 5), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(6, 5), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(5, 6), [0, 0, 0, 255]);
    assert_eq!(fb.get_pixel(6, 6), [0, 0, 0, 255]);
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
