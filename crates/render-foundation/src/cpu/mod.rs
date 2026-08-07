//! CPU 软件渲染 — 将所有图元类型光栅化到 RGBA 帧缓冲。
//!
//! 支持的图元类型（13 种）：
//! - FillPrimitive — 纯色矩形填充
//! - RoundedRectPrimitive — 圆角矩形填充
//! - GradientPrimitive — 线性/径向/锥形渐变
//! - ShadowPrimitive — 高斯模糊阴影
//! - ImagePrimitive — 图片渲染
//! - StrokePrimitive — 线段渲染（实线/虚线/点线）
//! - PathFillPrimitive — 多边形填充
//! - PathStrokePrimitive — 多边形描边
//! - TransformPrimitive — 2D 仿射变换
//! - ClipPrimitive — 矩形裁剪
//! - FilterPrimitive — CSS 滤镜（blur/opacity 等）
//! - BlendModePrimitive — 混合模式合成
//! - GlyphPrimitive / GlyphDraw — 文字渲染

mod effects;
mod gradient;
mod shadow;

use crate::geometry::Rect;
mod stroke;

use crate::color::Color;
use crate::font::cache::{GlyphCache, GlyphKey};
use crate::font::loader::FontLoader;
use crate::gpu::renderer::GlyphDraw;
use crate::image_cache::ImageCache;
use crate::primitive::{
    ClipPrimitive, DrawOp, FillPrimitive, ImagePrimitive, RenderPrimitives, RoundedRectPrimitive, TransformPrimitive,
};
use crate::surface::FrameBuffer;

pub(crate) fn glyph_top_left(x: f32, baseline_y: f32, x_offset: i16, y_offset: i16, height: u16) -> (f32, f32) {
    (x + x_offset as f32, baseline_y - y_offset as f32 - height as f32)
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scale_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32 * scale).round() as u32).max(1)
}

/// 在独立工作线程中执行 `render_full_scene`（#3 渲染线程化 RFC S2 基础设施）。
///
/// 使用 `std::thread::scope` 安全借用 `&mut` 状态（scope 保证线程在借用
/// 结束前 join）；调用方同步等待结果。当前为「线程内光栅化 + 同步等待」
///（正确性/架构准备，无并行收益）——异步化（发起后不阻塞）为后续切片。
#[allow(clippy::too_many_arguments)] // 光栅化全参数（本文件多处同款）
pub fn render_full_scene_threaded(
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: Option<&mut ImageCache>,
    ui_glyphs: &[GlyphDraw],
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
    overlay_rounded_rects: &[RoundedRectPrimitive],
) -> FrameBuffer {
    std::thread::scope(|s| {
        s.spawn(|| {
            render_full_scene(
                width,
                height,
                scale_factor,
                primitives,
                font_loader,
                glyph_cache,
                image_cache,
                ui_glyphs,
                overlay_fills,
                overlay_glyphs,
                overlay_rounded_rects,
            )
        })
        .join()
        .expect("渲染线程 panic")
    })
}

/// 将所有 RenderPrimitives 图元渲染到 CPU 帧缓冲。
///
/// 这是 M7 新增的完整渲染入口，接受 `RenderPrimitives` 并渲染所有 13 种图元类型。
#[allow(clippy::too_many_arguments)] // 光栅化全参数（文件内多处同款）
pub fn render_full_scene(
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: Option<&mut ImageCache>,
    ui_glyphs: &[GlyphDraw],
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
    overlay_rounded_rects: &[RoundedRectPrimitive],
) -> FrameBuffer {
    render_full_scene_region(
        width,
        height,
        scale_factor,
        primitives,
        font_loader,
        glyph_cache,
        image_cache,
        ui_glyphs,
        overlay_fills,
        overlay_glyphs,
        overlay_rounded_rects,
        None,
    )
}

/// 区域光栅化（S3 增量重绘）——只绘制与 `region`（CSS 逻辑像素）相交的图元。
///
/// `region=None` 时与 `render_full_scene` 完全一致（全量）。增量消费方
///（RFC S3）把 RenderStats.dirty_rects 传入，跳过区域外图元的光栅化。
#[allow(clippy::too_many_arguments)] // 光栅化全参数（文件内多处同款）
pub fn render_full_scene_region(
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: Option<&mut ImageCache>,
    ui_glyphs: &[GlyphDraw],
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
    overlay_rounded_rects: &[RoundedRectPrimitive],
    region: Option<Rect>,
) -> FrameBuffer {
    let scale = normalize_scale_factor(scale_factor);
    let physical_width = scale_dimension(width, scale);
    let physical_height = scale_dimension(height, scale);
    let mut fb = FrameBuffer::new(physical_width, physical_height);
    fb.clear(255, 255, 255, 255);

    // DC-10 CSS painting order：默认按图元真实插入顺序（draw_order）渲染，
    // 修复「父背景图画在子内容之上」的类型分桶缺陷（render_typed_buckets 把所有
    // images 画在所有 fills 之后，违反 CSS painting order）。
    // draw_order 由 cull_invisible 重建保留（见 ops.rs），生产路径可用。
    // 逃生舱：`ZERO_DRAW_ORDER=0` 回退到类型分桶（旧行为，用于诊断/回归对比）。
    // draw_order 为空时（旧代码路径未填充）自动回退到类型分桶。
    let use_draw_order = std::env::var("ZERO_DRAW_ORDER").as_deref() != Ok("0") && !primitives.draw_order.is_empty();

    if use_draw_order {
        render_draw_order(
            &mut fb,
            primitives,
            scale,
            font_loader,
            glyph_cache,
            image_cache,
            region,
        );
    } else {
        render_typed_buckets(
            &mut fb,
            primitives,
            scale,
            font_loader,
            glyph_cache,
            image_cache,
            region,
        );
    }

    // Chrome / WebView 文字（GlyphDraw，在 overlay 之前）
    for glyph in ui_glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    // Overlay 层（始终在最后，独立于主体绘制顺序）
    for fill in overlay_fills {
        fill_rect(&mut fb, fill, scale);
    }

    for rr in overlay_rounded_rects {
        fill_rounded_rect(&mut fb, rr, scale);
    }

    for glyph in overlay_glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    fb
}

/// 按类型分桶渲染（旧行为，`ZERO_DRAW_ORDER=0` 时回退）。
///
/// 所有同类型图元连续渲染：fills → rounded_rects → gradients → images →
/// strokes → ... → glyphs。违反 CSS painting order（父背景图覆盖子内容），
/// 保留作为逃生舱用于诊断/回归对比。
#[allow(clippy::too_many_arguments)]
fn render_typed_buckets(
    fb: &mut FrameBuffer,
    primitives: &RenderPrimitives,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    mut image_cache: Option<&mut ImageCache>,
    region: Option<Rect>,
) {
    // 渲染顺序遵循 CSS painting order:
    // shadows → backgrounds (fills/rounded_rects/gradients) → images →
    // borders (strokes/path_fills/path_strokes) → content (glyphs) →
    // overlay → filters → blend_modes

    // S3 区域裁剪：图元矩形与 region 不相交则跳过（region=None 全量）
    let in_region = |rect: Rect| region.is_none_or(|r| r.intersects(&rect));

    // 1. 阴影（绘制在最底层）
    for shadow in &primitives.shadows {
        if !in_region(shadow.rect) {
            continue;
        }
        shadow::render_shadow(fb, shadow, scale);
    }

    // 2. 填充矩形（背景色）
    for fill in &primitives.fills {
        if !in_region(fill.rect) {
            continue;
        }
        fill_rect(fb, fill, scale);
    }

    // 3. 圆角矩形
    for rr in &primitives.rounded_rects {
        if !in_region(rr.rect) {
            continue;
        }
        fill_rounded_rect(fb, rr, scale);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        if !in_region(gradient.rect) {
            continue;
        }
        gradient::render_gradient(fb, gradient, scale);
    }

    // 5. 图片
    if let Some(ref mut cache) = image_cache {
        for image in &primitives.images {
            if !in_region(image.rect) {
                continue;
            }
            render_image(fb, image, scale, cache);
        }
    }

    // 6. 线段（边框等）
    for stroke in &primitives.strokes {
        if !in_region(Rect::new(
            stroke.x1.min(stroke.x2),
            stroke.y1.min(stroke.y2),
            (stroke.x1 - stroke.x2).abs(),
            (stroke.y1 - stroke.y2).abs(),
        )) {
            continue;
        }
        stroke::render_stroke(fb, stroke, scale);
    }

    // 7. 路径填充
    for path_fill in &primitives.path_fills {
        if !in_region(path_vertices_rect(&path_fill.vertices)) {
            continue;
        }
        stroke::render_path_fill(fb, path_fill, scale);
    }

    // 8. 路径描边
    for path_stroke in &primitives.path_strokes {
        if !in_region(path_vertices_rect(&path_stroke.vertices)) {
            continue;
        }
        stroke::render_path_stroke(fb, path_stroke, scale);
    }

    // 9. 文字
    for glyph in &primitives.glyphs {
        draw_glyph_primitive(fb, glyph, scale, font_loader, glyph_cache);
    }

    // 10. 裁剪 — 后处理像素级裁剪
    for clip in &primitives.clips {
        apply_clip(fb, clip, scale);
    }

    // 11. 变换 — 后处理像素级变换
    for transform in &primitives.transforms {
        apply_transform_post(fb, transform, scale);
    }

    // 12. 滤镜 — 后处理效果
    for filter in &primitives.filters {
        effects::apply_filter(fb, filter, scale);
    }

    // 13. 混合模式 — 后处理合成
    for blend in &primitives.blend_modes {
        effects::apply_blend_mode(fb, blend, scale);
    }
}

/// 路径图元包围盒（区域裁剪辅助；vertices 为扁平 (x, y) 序列）。
fn path_vertices_rect(vertices: &[f32]) -> Rect {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for pair in vertices.chunks_exact(2) {
        min_x = min_x.min(pair[0]);
        min_y = min_y.min(pair[1]);
        max_x = max_x.max(pair[0]);
        max_y = max_y.max(pair[1]);
    }
    if min_x > max_x {
        return Rect::ZERO;
    }
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// 按插入顺序渲染（DC-10 默认路径，满足 CSS painting order）。
///
/// 按 `draw_order` 记录的真实插入顺序逐个渲染图元。背景、边框、子内容、
/// 文字按 paint_node 的深度优先序列交错，父背景图正确画在子内容之下。
#[allow(clippy::too_many_arguments)]
fn render_draw_order(
    fb: &mut FrameBuffer,
    primitives: &RenderPrimitives,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    mut image_cache: Option<&mut ImageCache>,
    region: Option<Rect>,
) {
    // S3 区域裁剪：图元矩形与 region 不相交则跳过（region=None 全量）
    let in_region = |rect: Rect| region.is_none_or(|r| r.intersects(&rect));

    for op in &primitives.draw_order {
        match op {
            DrawOp::Shadow(i) => {
                if let Some(p) = primitives.shadows.get(*i)
                    && in_region(p.rect)
                {
                    shadow::render_shadow(fb, p, scale);
                }
            }
            DrawOp::Fill(i) => {
                if let Some(p) = primitives.fills.get(*i)
                    && in_region(p.rect)
                {
                    fill_rect(fb, p, scale);
                }
            }
            DrawOp::RoundedRect(i) => {
                if let Some(p) = primitives.rounded_rects.get(*i)
                    && in_region(p.rect)
                {
                    fill_rounded_rect(fb, p, scale);
                }
            }
            DrawOp::Gradient(i) => {
                if let Some(p) = primitives.gradients.get(*i)
                    && in_region(p.rect)
                {
                    gradient::render_gradient(fb, p, scale);
                }
            }
            DrawOp::Image(i) => {
                if let Some(p) = primitives.images.get(*i)
                    && let Some(ref mut cache) = image_cache
                    && in_region(p.rect)
                {
                    render_image(fb, p, scale, cache);
                }
            }
            DrawOp::Stroke(i) => {
                if let Some(p) = primitives.strokes.get(*i)
                    && in_region(Rect::new(
                        p.x1.min(p.x2),
                        p.y1.min(p.y2),
                        (p.x1 - p.x2).abs(),
                        (p.y1 - p.y2).abs(),
                    ))
                {
                    stroke::render_stroke(fb, p, scale);
                }
            }
            DrawOp::PathFill(i) => {
                if let Some(p) = primitives.path_fills.get(*i)
                    && in_region(path_vertices_rect(&p.vertices))
                {
                    stroke::render_path_fill(fb, p, scale);
                }
            }
            DrawOp::PathStroke(i) => {
                if let Some(p) = primitives.path_strokes.get(*i) {
                    stroke::render_path_stroke(fb, p, scale);
                }
            }
            DrawOp::Glyph(i) => {
                if let Some(p) = primitives.glyphs.get(*i) {
                    draw_glyph_primitive(fb, p, scale, font_loader, glyph_cache);
                }
            }
            DrawOp::Filter(i) => {
                if let Some(p) = primitives.filters.get(*i) {
                    effects::apply_filter(fb, p, scale);
                }
            }
            DrawOp::BlendMode(i) => {
                if let Some(p) = primitives.blend_modes.get(*i) {
                    effects::apply_blend_mode(fb, p, scale);
                }
            }
            DrawOp::Transform(i) => {
                if let Some(p) = primitives.transforms.get(*i) {
                    apply_transform_post(fb, p, scale);
                }
            }
            DrawOp::Clip(i) => {
                if let Some(p) = primitives.clips.get(*i) {
                    apply_clip(fb, p, scale);
                }
            }
        }
    }
}

/// 旧版兼容入口 — 仅渲染 fills + rounded_rects + glyphs。
///
/// 保留此函数是为了向后兼容。新代码应使用 `render_full_scene()`。
#[allow(clippy::too_many_arguments)]
pub fn render_scene_to_framebuffer(
    width: u32,
    height: u32,
    scale_factor: f32,
    fills: &[FillPrimitive],
    rounded_rects: &[RoundedRectPrimitive],
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    glyphs: &[GlyphDraw],
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
    overlay_rounded_rects: &[RoundedRectPrimitive],
) -> FrameBuffer {
    let scale = normalize_scale_factor(scale_factor);
    let physical_width = scale_dimension(width, scale);
    let physical_height = scale_dimension(height, scale);
    let mut fb = FrameBuffer::new(physical_width, physical_height);
    fb.clear(255, 255, 255, 255);

    for fill in fills {
        fill_rect(&mut fb, fill, scale);
    }

    for rr in rounded_rects {
        fill_rounded_rect(&mut fb, rr, scale);
    }

    for glyph in glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    for fill in overlay_fills {
        fill_rect(&mut fb, fill, scale);
    }

    for rr in overlay_rounded_rects {
        fill_rounded_rect(&mut fb, rr, scale);
    }

    for glyph in overlay_glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    fb
}

// ─── 基础图元渲染 ───

fn fill_rect(fb: &mut FrameBuffer, fill: &FillPrimitive, scale: f32) {
    let left = (fill.rect.left() * scale).floor().max(0.0) as u32;
    let top = (fill.rect.top() * scale).floor().max(0.0) as u32;
    let right = (fill.rect.right() * scale).ceil().max(0.0).min(fb.width as f32) as u32;
    let bottom = (fill.rect.bottom() * scale).ceil().max(0.0).min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    for y in top..bottom {
        for x in left..right {
            let src_a = fill.color.a as f32 / 255.0;
            if src_a >= 1.0 {
                fb.set_pixel(x, y, [fill.color.r, fill.color.g, fill.color.b, 255]);
            } else if src_a > 0.0 {
                blend_pixel(fb, x, y, fill.color, 255);
            }
        }
    }
}

/// 光栅化圆角矩形 — 对每个像素判断是否在圆角矩形内。
fn fill_rounded_rect(fb: &mut FrameBuffer, rr: &RoundedRectPrimitive, scale: f32) {
    let left = (rr.rect.left() * scale).floor().max(0.0) as u32;
    let top = (rr.rect.top() * scale).floor().max(0.0) as u32;
    let right = (rr.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (rr.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    let tl_r = rr.top_left_radius * scale;
    let tr_r = rr.top_right_radius * scale;
    let br_r = rr.bottom_right_radius * scale;
    let bl_r = rr.bottom_left_radius * scale;

    let x0 = rr.rect.left() * scale;
    let y0 = rr.rect.top() * scale;
    let x1 = rr.rect.right() * scale;
    let y1 = rr.rect.bottom() * scale;

    // 与 fill_rect 一致的 alpha 处理：不透明直接 set_pixel，半透明 blend_pixel
    // 与背景合成。旧实现硬编码 alpha=255 + set_pixel，致 rgba() 半透明圆角背景
    //（如 morning.work .item-tag 的 var(--color-primary-alpha-05)）被渲染为实色。
    let src_a = rr.color.a as f32 / 255.0;

    for y in top..bottom {
        let fy = y as f32 + 0.5;
        for x in left..right {
            let fx = x as f32 + 0.5;

            if !is_inside_rounded_rect(fx, fy, x0, y0, x1, y1, tl_r, tr_r, br_r, bl_r) {
                continue;
            }

            if src_a >= 1.0 {
                fb.set_pixel(x, y, [rr.color.r, rr.color.g, rr.color.b, 255]);
            } else if src_a > 0.0 {
                blend_pixel(fb, x, y, rr.color, 255);
            }
        }
    }
}

/// 判断像素 (fx, fy) 是否在圆角矩形内。
#[allow(clippy::too_many_arguments)]
fn is_inside_rounded_rect(
    fx: f32,
    fy: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    tl_r: f32,
    tr_r: f32,
    br_r: f32,
    bl_r: f32,
) -> bool {
    if fx < x0 + tl_r && fy < y0 + tl_r {
        let dx = fx - (x0 + tl_r);
        let dy = fy - (y0 + tl_r);
        return dx * dx + dy * dy <= tl_r * tl_r;
    }
    if fx > x1 - tr_r && fy < y0 + tr_r {
        let dx = fx - (x1 - tr_r);
        let dy = fy - (y0 + tr_r);
        return dx * dx + dy * dy <= tr_r * tr_r;
    }
    if fx > x1 - br_r && fy > y1 - br_r {
        let dx = fx - (x1 - br_r);
        let dy = fy - (y1 - br_r);
        return dx * dx + dy * dy <= br_r * br_r;
    }
    if fx < x0 + bl_r && fy > y1 - bl_r {
        let dx = fx - (x0 + bl_r);
        let dy = fy - (y1 - bl_r);
        return dx * dx + dy * dy <= bl_r * bl_r;
    }
    true
}

fn draw_glyph(
    fb: &mut FrameBuffer,
    glyph: &GlyphDraw,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
) {
    let Some(bitmap) = resolve_glyph_bitmap(glyph, scale, font_loader, glyph_cache) else {
        return;
    };

    if bitmap.width == 0 || bitmap.height == 0 {
        return;
    }

    let (gx, gy) = glyph_top_left(
        glyph.x * scale,
        glyph.baseline_y * scale,
        bitmap.x_offset,
        bitmap.y_offset,
        bitmap.height,
    );
    let start_x = gx.round() as i32;
    let start_y = gy.round() as i32;

    for row in 0..bitmap.height {
        for col in 0..bitmap.width {
            let px = start_x + col as i32;
            let py = start_y + row as i32;
            if px < 0 || py < 0 || px >= fb.width as i32 || py >= fb.height as i32 {
                continue;
            }

            let alpha = bitmap.data[row as usize * bitmap.width as usize + col as usize];
            if alpha == 0 {
                continue;
            }
            blend_pixel(fb, px as u32, py as u32, glyph.color, alpha);
        }
    }
}

/// 从 GlyphPrimitive 渲染文字（使用 font_id + glyph_id 而非 char）。
fn draw_glyph_primitive(
    fb: &mut FrameBuffer,
    glyph: &crate::primitive::GlyphPrimitive,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
) {
    let physical_font_size = glyph.font_size * scale;
    let raw_font_id = glyph.font_id.0;
    let key = GlyphKey::new(raw_font_id, glyph.glyph_id, physical_font_size);

    // 尝试从缓存获取
    if let Some(cached) = glyph_cache.get(&key) {
        if cached.width > 0 && cached.height > 0 {
            let color = glyph.color;
            let x = glyph.x * scale;
            let y = glyph.y * scale;
            blit_glyph_bitmap(fb, cached, x, y, color, glyph.rotation, glyph.synthetic_italic);
        }
        return;
    }

    // 通过 font_loader 渲染 glyph
    let ch = char::from_u32(glyph.glyph_id).unwrap_or('\0');
    if ch == '\0' {
        return;
    }

    if let Ok((resolved_id, bitmap)) = font_loader.rasterize_glyph_with_fallback(raw_font_id, ch, physical_font_size)
        && bitmap.width > 0
        && bitmap.height > 0
    {
        let cache_key = GlyphKey::new(resolved_id, glyph.glyph_id, physical_font_size);
        let _ = glyph_cache.get_or_insert_with(cache_key, || Ok(bitmap.clone()));
        let color = glyph.color;
        let x = glyph.x * scale;
        let y = glyph.y * scale;
        blit_glyph_bitmap(fb, &bitmap, x, y, color, glyph.rotation, glyph.synthetic_italic);
    }
}

/// 将字形位图合成到帧缓冲。
///
/// `rotation` 为弧度，0.0 表示不旋转，FRAC_PI_2 表示顺时针 90°。
/// 对于垂直书写模式，字形需要旋转 90° 使文字从上到下排列。
/// `synthetic_italic`（R2497）：true 时对非旋转字形应用 ~14° 水平 shear
/// （系统字体无 italic face 时的合成斜体，对齐 chromium）。
///
/// kill-switch：env `ZW_SYNTHETIC_ITALIC=0` 关闭（强制不 shear，回退现状 upright）。
fn blit_glyph_bitmap(
    fb: &mut FrameBuffer,
    bitmap: &crate::font::GlyphBitmap,
    x: f32,
    y: f32,
    color: Color,
    rotation: f32,
    synthetic_italic: bool,
) {
    // R2497 kill-switch：env 关闭时强制 synthetic_italic=false（不 shear）。
    let synthetic_italic = synthetic_italic && !matches!(std::env::var("ZW_SYNTHETIC_ITALIC"), Ok(v) if v == "0");
    let start_x = x.round() as i32;
    let start_y = y.round() as i32;

    // 判断是否为 ~90° 旋转（容差 ±0.1 弧度）
    let is_rotated_90 = (rotation - std::f32::consts::FRAC_PI_2).abs() < 0.1;

    if is_rotated_90 {
        // 顺时针旋转 90°：原始 (col, row) → 旋转后 (row, width - 1 - col)
        // 旋转后位图尺寸：width × height → height × width
        let bmp_w = bitmap.width as i32;
        let bmp_h = bitmap.height as i32;
        for row in 0..bmp_h {
            for col in 0..bmp_w {
                // 顺时针 90° 旋转后的坐标
                let rotated_col = row;
                let rotated_row = bmp_w - 1 - col;

                let px = start_x + rotated_col;
                let py = start_y + rotated_row;
                if px < 0 || py < 0 || px >= fb.width as i32 || py >= fb.height as i32 {
                    continue;
                }

                let alpha = bitmap.data[row as usize * bitmap.width as usize + col as usize];
                if alpha == 0 {
                    continue;
                }
                blend_pixel(fb, px as u32, py as u32, color, alpha);
            }
        }
    } else {
        // R2497 synthetic italic：每行水平偏移 shear_dx = (row - height/2) * ITALIC_SKEW。
        // 锚 height/2 使 shear 上下对称（近似 chromium 基线锚；A/B 后可调）。
        // ITALIC_SKEW = tan(14°) ≈ 0.249。
        const ITALIC_SKEW: f32 = 0.249;
        let anchor = bitmap.height as f32 / 2.0;
        for row in 0..bitmap.height {
            let shear_dx = if synthetic_italic {
                ((row as f32 - anchor) * ITALIC_SKEW).round() as i32
            } else {
                0
            };
            for col in 0..bitmap.width {
                let px = start_x + col as i32 + shear_dx;
                let py = start_y + row as i32;
                if px < 0 || py < 0 || px >= fb.width as i32 || py >= fb.height as i32 {
                    continue;
                }

                let alpha = bitmap.data[row as usize * bitmap.width as usize + col as usize];
                if alpha == 0 {
                    continue;
                }
                blend_pixel(fb, px as u32, py as u32, color, alpha);
            }
        }
    }
}

fn resolve_glyph_bitmap(
    glyph: &GlyphDraw,
    scale: f32,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
) -> Option<crate::font::GlyphBitmap> {
    let physical_font_size = glyph.font_size * scale;
    let primary_key = GlyphKey::new(glyph.font_id, glyph.ch as u32, physical_font_size);
    if let Some(cached) = glyph_cache.get(&primary_key) {
        return Some(cached.clone());
    }

    let (resolved_id, bitmap) = font_loader
        .rasterize_glyph_with_fallback(glyph.font_id, glyph.ch, physical_font_size)
        .ok()?;
    let key = GlyphKey::new(resolved_id, glyph.ch as u32, physical_font_size);
    glyph_cache.get_or_insert_with(key, || Ok(bitmap)).ok().cloned()
}

fn blend_pixel(fb: &mut FrameBuffer, x: u32, y: u32, color: Color, alpha: u8) {
    let dst = fb.get_pixel(x, y);
    let src_a = alpha as f32 / 255.0 * (color.a as f32 / 255.0);
    let inv_a = 1.0 - src_a;
    let r = color.r as f32 * src_a + dst[0] as f32 * inv_a;
    let g = color.g as f32 * src_a + dst[1] as f32 * inv_a;
    let b = color.b as f32 * src_a + dst[2] as f32 * inv_a;
    fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
}

/// 渲染图片图元 — 将 RGBA 像素数据合成到帧缓冲。
fn render_image(fb: &mut FrameBuffer, image: &ImagePrimitive, scale: f32, image_cache: &mut ImageCache) {
    let Some(data) = image_cache.get(&image.image_key) else {
        return;
    };

    // source 映射基 = **完整 image.rect**（保持原始分辨率，不因裁剪缩放）。
    // crop 语义（R294）：clip 窗口只收窄绘制区域，source 仍按完整 rect 映射。
    let rect_left = image.rect.left() * scale;
    let rect_top = image.rect.top() * scale;
    let rect_w = (image.rect.right() - image.rect.left()) * scale;
    let rect_h = (image.rect.bottom() - image.rect.top()) * scale;
    if rect_w <= 0.0 || rect_h <= 0.0 {
        image_cache.release(&image.image_key);
        return;
    }

    // 实际绘制区域 = rect ∩ clip（None 时 = rect），再裁到 framebuffer 边界
    let mut draw_left = rect_left;
    let mut draw_top = rect_top;
    let mut draw_right = rect_left + rect_w;
    let mut draw_bottom = rect_top + rect_h;
    if let Some(clip) = &image.clip {
        draw_left = draw_left.max(clip.left() * scale);
        draw_top = draw_top.max(clip.top() * scale);
        draw_right = draw_right.min(clip.right() * scale);
        draw_bottom = draw_bottom.min(clip.bottom() * scale);
    }
    let left = draw_left.floor().max(0.0) as u32;
    let top = draw_top.floor().max(0.0) as u32;
    let right = draw_right.ceil().min(fb.width as f32) as u32;
    let bottom = draw_bottom.ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        image_cache.release(&image.image_key);
        return;
    }

    // 纯色图片优化：跳过双线性插值，直接填充目标矩形
    // 消除小尺寸 swatch 图片（如 1x1-green.png）缩放到大尺寸时的边缘伪影
    if let Some(color) = data.solid_color() {
        let [sr, sg, sb, sa] = color;
        if sa == 0 {
            image_cache.release(&image.image_key);
            return; // 全透明，跳过
        }
        for y in top..bottom {
            for x in left..right {
                if x >= fb.width || y >= fb.height {
                    continue;
                }
                if sa == 255 {
                    fb.set_pixel(x, y, [sr, sg, sb, 255]);
                } else {
                    let c = Color::rgba(sr, sg, sb, sa);
                    blend_pixel(fb, x, y, c, 255);
                }
            }
        }
        image_cache.release(&image.image_key);
        return;
    }

    // 双线性插值缩放采样：source 映射到完整 rect，仅绘制 [left,right)×[top,bottom)
    let src_w_f = data.width as f32;
    let src_h_f = data.height as f32;
    for py in top..bottom {
        // 映射到源图像坐标（中心对齐，相对完整 rect）
        let src_y = ((py as f32 + 0.5 - rect_top) / rect_h) * src_h_f - 0.5;
        let src_y0 = (src_y.floor().max(0.0) as u32).min(data.height.saturating_sub(1));
        let src_y1 = (src_y0 + 1).min(data.height.saturating_sub(1));
        let fy = src_y - src_y0 as f32;

        for px in left..right {
            let src_x = ((px as f32 + 0.5 - rect_left) / rect_w) * src_w_f - 0.5;
            let src_x0 = (src_x.floor().max(0.0) as u32).min(data.width.saturating_sub(1));
            let src_x1 = (src_x0 + 1).min(data.width.saturating_sub(1));
            let fx = src_x - src_x0 as f32;

            // 双线性插值：4 个邻近像素加权平均
            let [r00, g00, b00, a00] = data.get_pixel(src_x0, src_y0);
            let [r10, g10, b10, a10] = data.get_pixel(src_x1, src_y0);
            let [r01, g01, b01, a01] = data.get_pixel(src_x0, src_y1);
            let [r11, g11, b11, a11] = data.get_pixel(src_x1, src_y1);

            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;

            let src_r = (r00 as f32 * w00 + r10 as f32 * w10 + r01 as f32 * w01 + r11 as f32 * w11 + 0.5) as u8;
            let src_g = (g00 as f32 * w00 + g10 as f32 * w10 + g01 as f32 * w01 + g11 as f32 * w11 + 0.5) as u8;
            let src_b = (b00 as f32 * w00 + b10 as f32 * w10 + b01 as f32 * w01 + b11 as f32 * w11 + 0.5) as u8;
            let src_a = (a00 as f32 * w00 + a10 as f32 * w10 + a01 as f32 * w01 + a11 as f32 * w11 + 0.5) as u8;

            if px >= fb.width || py >= fb.height {
                continue;
            }

            if src_a == 255 {
                fb.set_pixel(px, py, [src_r, src_g, src_b, 255]);
            } else if src_a > 0 {
                let color = Color::rgba(src_r, src_g, src_b, src_a);
                blend_pixel(fb, px, py, color, 255);
            }
        }
    }
    image_cache.release(&image.image_key);
}

/// 应用矩形裁剪 — 将裁剪区域外的像素清除为透明。
fn apply_clip(fb: &mut FrameBuffer, clip: &ClipPrimitive, scale: f32) {
    let clip_left = (clip.rect.left() * scale).floor().max(0.0) as u32;
    let clip_top = (clip.rect.top() * scale).floor().max(0.0) as u32;
    let clip_right = (clip.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let clip_bottom = (clip.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    // 清除裁剪区域上方的像素
    for y in 0..clip_top.min(fb.height) {
        for x in 0..fb.width {
            fb.set_pixel(x, y, [255, 255, 255, 255]);
        }
    }

    // 清除裁剪区域下方的像素
    for y in clip_bottom..fb.height {
        for x in 0..fb.width {
            fb.set_pixel(x, y, [255, 255, 255, 255]);
        }
    }

    // 清除裁剪区域左右两侧的像素
    for y in clip_top..clip_bottom {
        for x in 0..clip_left {
            fb.set_pixel(x, y, [255, 255, 255, 255]);
        }
        for x in clip_right..fb.width {
            fb.set_pixel(x, y, [255, 255, 255, 255]);
        }
    }
}

/// 应用变换后处理 — 对 TransformPrimitive 区域内的像素执行仿射变换。
///
/// 在 CPU 渲染器中，变换通过「反向采样」实现：
/// 对输出区域中的每个像素，通过逆变换映射到源像素位置，然后复制颜色。
fn apply_transform_post(fb: &mut FrameBuffer, transform: &TransformPrimitive, scale: f32) {
    let rect = transform.rect;
    let left = (rect.left() * scale).floor().max(0.0) as u32;
    let top = (rect.top() * scale).floor().max(0.0) as u32;
    let right = (rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    // 仿射矩阵: | a  c  tx |   逆矩阵: 1/det * | d  -c  (c*ty - d*tx) |
    //            | b  d  ty |                     | -b  a  (b*tx - a*ty) |
    //            | 0  0   1 |                     |  0  0       det       |
    let det = transform.a * transform.d - transform.b * transform.c;
    if det.abs() < 1e-10 {
        return; // 奇异矩阵，跳过
    }

    let inv_det = 1.0 / det;
    let inv_a = transform.d * inv_det;
    let inv_c = -transform.c * inv_det;
    let inv_tx = (transform.c * transform.ty - transform.d * transform.tx) * inv_det;
    let inv_b = -transform.b * inv_det;
    let inv_d = transform.a * inv_det;
    let inv_ty = (transform.b * transform.tx - transform.a * transform.ty) * inv_det;

    let ox = transform.origin_x * scale;
    let oy = transform.origin_y * scale;

    // 保存原始像素
    let w = (right - left) as usize;
    let h = (bottom - top) as usize;
    let mut src_pixels = vec![[255u8; 4]; w * h];
    for y in 0..h {
        for x in 0..w {
            src_pixels[y * w + x] = fb.get_pixel(left + x as u32, top + y as u32);
        }
    }

    // 清除区域为白色
    for y in top..bottom {
        for x in left..right {
            fb.set_pixel(x, y, [255, 255, 255, 255]);
        }
    }

    // 反向采样
    for y in top..bottom {
        for x in left..right {
            let px = x as f32 - ox;
            let py = y as f32 - oy;

            // 应用逆变换
            let src_x = inv_a * px + inv_c * py + inv_tx + ox;
            let src_y = inv_b * px + inv_d * py + inv_ty + oy;

            let sx = src_x.round() as i32;
            let sy = src_y.round() as i32;

            if sx >= left as i32 && sx < right as i32 && sy >= top as i32 && sy < bottom as i32 {
                let src_color = src_pixels[(sy as usize - top as usize) * w + (sx as usize - left as usize)];
                fb.set_pixel(x, y, src_color);
            }
        }
    }
}

#[cfg(test)]
mod tests;
