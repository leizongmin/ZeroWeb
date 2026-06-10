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
mod stroke;

use crate::color::Color;
use crate::font::cache::{GlyphCache, GlyphKey};
use crate::font::loader::FontLoader;
use crate::gpu::renderer::GlyphDraw;
use crate::image_cache::ImageCache;
use crate::primitive::{
    ClipPrimitive, FillPrimitive, ImagePrimitive, RenderPrimitives, RoundedRectPrimitive, TransformPrimitive,
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

/// 将所有 RenderPrimitives 图元渲染到 CPU 帧缓冲。
///
/// 这是 M7 新增的完整渲染入口，接受 `RenderPrimitives` 并渲染所有 13 种图元类型。
#[allow(clippy::too_many_arguments)]
pub fn render_full_scene(
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    mut image_cache: Option<&mut ImageCache>,
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
) -> FrameBuffer {
    let scale = normalize_scale_factor(scale_factor);
    let physical_width = scale_dimension(width, scale);
    let physical_height = scale_dimension(height, scale);
    let mut fb = FrameBuffer::new(physical_width, physical_height);
    fb.clear(255, 255, 255, 255);

    // 渲染顺序遵循 CSS painting order:
    // shadows → backgrounds (fills/rounded_rects/gradients) → images →
    // borders (strokes/path_fills/path_strokes) → content (glyphs) →
    // overlay → filters → blend_modes

    // 1. 阴影（绘制在最底层）
    for shadow in &primitives.shadows {
        shadow::render_shadow(&mut fb, shadow, scale);
    }

    // 2. 填充矩形（背景色）
    for fill in &primitives.fills {
        fill_rect(&mut fb, fill, scale);
    }

    // 3. 圆角矩形
    for rr in &primitives.rounded_rects {
        fill_rounded_rect(&mut fb, rr, scale);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        gradient::render_gradient(&mut fb, gradient, scale);
    }

    // 5. 图片
    if let Some(ref mut cache) = image_cache {
        for image in &primitives.images {
            render_image(&mut fb, image, scale, cache);
        }
    }

    // 6. 线段（边框等）
    for stroke in &primitives.strokes {
        stroke::render_stroke(&mut fb, stroke, scale);
    }

    // 7. 路径填充
    for path_fill in &primitives.path_fills {
        stroke::render_path_fill(&mut fb, path_fill, scale);
    }

    // 8. 路径描边
    for path_stroke in &primitives.path_strokes {
        stroke::render_path_stroke(&mut fb, path_stroke, scale);
    }

    // 9. 文字
    for glyph in &primitives.glyphs {
        draw_glyph_primitive(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    // 10. 裁剪 — 后处理像素级裁剪
    for clip in &primitives.clips {
        apply_clip(&mut fb, clip, scale);
    }

    // 11. 变换 — 后处理像素级变换
    for transform in &primitives.transforms {
        apply_transform_post(&mut fb, transform, scale);
    }

    // 12. 滤镜 — 后处理效果
    for filter in &primitives.filters {
        effects::apply_filter(&mut fb, filter, scale);
    }

    // 13. 混合模式 — 后处理合成
    for blend in &primitives.blend_modes {
        effects::apply_blend_mode(&mut fb, blend, scale);
    }

    // Overlay 层
    for fill in overlay_fills {
        fill_rect(&mut fb, fill, scale);
    }

    for glyph in overlay_glyphs {
        draw_glyph(&mut fb, glyph, scale, font_loader, glyph_cache);
    }

    fb
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

    let color = [rr.color.r, rr.color.g, rr.color.b, 255];

    for y in top..bottom {
        let fy = y as f32 + 0.5;
        for x in left..right {
            let fx = x as f32 + 0.5;

            if !is_inside_rounded_rect(fx, fy, x0, y0, x1, y1, tl_r, tr_r, br_r, bl_r) {
                continue;
            }

            fb.set_pixel(x, y, color);
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
            blit_glyph_bitmap(fb, cached, x, y, color, glyph.rotation);
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
        blit_glyph_bitmap(fb, &bitmap, x, y, color, glyph.rotation);
    }
}

/// 将字形位图合成到帧缓冲。
///
/// `rotation` 为弧度，0.0 表示不旋转，FRAC_PI_2 表示顺时针 90°。
/// 对于垂直书写模式，字形需要旋转 90° 使文字从上到下排列。
fn blit_glyph_bitmap(
    fb: &mut FrameBuffer,
    bitmap: &crate::font::GlyphBitmap,
    x: f32,
    y: f32,
    color: Color,
    rotation: f32,
) {
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
        // 无旋转 — 正常渲染
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

    let left = (image.rect.left() * scale).floor().max(0.0) as u32;
    let top = (image.rect.top() * scale).floor().max(0.0) as u32;
    let right = (image.rect.right() * scale).ceil().min(fb.width as f32) as u32;
    let bottom = (image.rect.bottom() * scale).ceil().min(fb.height as f32) as u32;

    if left >= right || top >= bottom {
        return;
    }

    let dst_w = right - left;
    let dst_h = bottom - top;

    // 双线性插值缩放采样
    let src_w_f = data.width as f32;
    let src_h_f = data.height as f32;
    for y in 0..dst_h {
        // 映射到源图像坐标（中心对齐）
        let src_y = (y as f32 + 0.5) / dst_h as f32 * src_h_f - 0.5;
        let src_y0 = src_y.floor().max(0.0) as u32;
        let src_y1 = (src_y0 + 1).min(data.height - 1);
        let fy = src_y - src_y0 as f32;

        for x in 0..dst_w {
            let src_x = (x as f32 + 0.5) / dst_w as f32 * src_w_f - 0.5;
            let src_x0 = src_x.floor().max(0.0) as u32;
            let src_x1 = (src_x0 + 1).min(data.width - 1);
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

            let dst_x = left + x;
            let dst_y = top + y;

            if dst_x >= fb.width || dst_y >= fb.height {
                continue;
            }

            if src_a == 255 {
                fb.set_pixel(dst_x, dst_y, [src_r, src_g, src_b, 255]);
            } else if src_a > 0 {
                let color = Color::rgba(src_r, src_g, src_b, src_a);
                blend_pixel(fb, dst_x, dst_y, color, 255);
            }
        }
    }
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
