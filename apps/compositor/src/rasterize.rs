//! compositor 光栅化：全量 / S3 脏区域增量 + 持久 RenderingThread。

use zero_protocol::paint_snapshot::{IpcRect, PaintSnapshotParams};
use zero_render_foundation::cpu::render_full_scene_region_into;
use zero_render_foundation::display_list::DisplayList;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageCache;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_render_foundation::rendering_thread::{RenderingThread, render_threading_enabled};
use zero_render_foundation::surface::FrameBuffer;

pub fn device_scale_factor(paint: &PaintSnapshotParams) -> f32 {
    if paint.device_scale_factor.is_finite() && paint.device_scale_factor > 0.0 {
        paint.device_scale_factor
    } else {
        1.0
    }
}

pub fn physical_viewport_size(paint: &PaintSnapshotParams) -> (u32, u32) {
    let scale = device_scale_factor(paint);
    (
        ((paint.viewport_width.max(1) as f32 * scale).round() as u32).max(1),
        ((paint.viewport_height.max(1) as f32 * scale).round() as u32).max(1),
    )
}

fn ipc_dirty_rects(rects: &[IpcRect]) -> Vec<(f32, f32, f32, f32)> {
    rects.iter().map(|r| (r.x, r.y, r.width, r.height)).collect()
}

/// T60：脏区共享归一——整数像素对齐 + 四边外扩 1px 墨迹余量（GPU scissor 路径与
/// CPU 区域渲染两个入口都经此函数，保证两臂消费同一像素集合）。
///
/// - 外扩 1px：字形 AA/子像素边缘的光栅外溢保守界（抗锯齿覆盖 ≤1 物理像素）；
/// - floor/ceil 对齐：消除小数坐标脏区（真实页面百分比宽度常态）下 GPU scissor
///   与 blit 区域边界错位的触发面；
/// - 视口钳位：脏区越界（节点部分在视口外）不产生越界 blit。
///
/// 视口外或退化矩形被过滤；返回逻辑 CSS 像素坐标（与 IPC 脏区同坐标系）。
pub fn normalize_dirty_rects(
    rects: &[(f32, f32, f32, f32)],
    viewport_w: f32,
    viewport_h: f32,
) -> Vec<(f32, f32, f32, f32)> {
    const INK_MARGIN: f32 = 1.0;
    rects
        .iter()
        .filter_map(|&(x, y, w, h)| {
            if w <= 0.0 || h <= 0.0 {
                return None;
            }
            let x0 = (x - INK_MARGIN).floor().max(0.0);
            let y0 = (y - INK_MARGIN).floor().max(0.0);
            let x1 = (x + w + INK_MARGIN).ceil().min(viewport_w);
            let y1 = (y + h + INK_MARGIN).ceil().min(viewport_h);
            if x1 <= x0 || y1 <= y0 {
                return None;
            }
            Some((x0, y0, x1 - x0, y1 - y0))
        })
        .collect()
}

/// 光栅化一帧到 `back` 缓冲（S3：部分 dirty 时 `copy_front_to_back` 后只重绘脏区）。
#[allow(clippy::too_many_arguments)]
pub fn rasterize_into_back(
    paint: &PaintSnapshotParams,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    render_thread: Option<&RenderingThread>,
    image_cache: &mut ImageCache,
    back: &mut FrameBuffer,
    copy_front: bool,
) {
    let scale = device_scale_factor(paint);
    let (w, h) = physical_viewport_size(paint);
    let logical_w = paint.viewport_width.max(1);
    let logical_h = paint.viewport_height.max(1);
    let dirty = normalize_dirty_rects(&ipc_dirty_rects(&paint.dirty_rects), logical_w as f32, logical_h as f32);
    let display_list = DisplayList::new(primitives.clone(), dirty);
    let vw = paint.viewport_width.max(1) as f32;
    let vh = paint.viewport_height.max(1) as f32;

    if display_list.is_full_viewport(vw, vh) {
        *back = rasterize_full(
            logical_w,
            logical_h,
            scale,
            primitives,
            font_loader,
            glyph_cache,
            render_thread,
            image_cache,
        );
        return;
    }

    if copy_front {
        // caller 已 copy_front_to_back：区外保留旧帧（未变内容）；区内重铺白底
        // 语义由下方两臂的「白底 patch 渲染 + blit_region 精确列拷回」承载（部分
        // 重绘若直接在旧帧上叠画，半透明图元会与自身上一帧渲染结果二次 src-over
        // 混合、逐帧漂移，goal M3-c）。GPU 臂同语义：主 pass `LoadOp::Clear(WHITE)`
        // 全目标清白后按 scissor 绘制。
    } else {
        back.clear(255, 255, 255, 255);
    }

    // 白底 scratch（懒分配、跨脏区复用）：直绘臂若在 back 上 region 渲染，跨界
    // 图元（圆角/阴影/渐变/图片/描边/路径/跨界字形，均无逐像素裁剪）的区外越界
    // 写入会叠上旧帧（半透明/AA 逐帧漂移，M3-c 残留族）。改为白底 scratch 渲染
    // 后按 region 精确列拷回——越界写入落在 scratch 被丢弃；与 worker patch 臂
    // （rasterize_sync 白底整帧）及 browser 条带（scratch + 拷行）同款隔离。
    let mut scratch: Option<FrameBuffer> = None;

    for (x, y, rw, rh) in &display_list.dirty_rects {
        if *rw <= 0.0 || *rh <= 0.0 {
            continue;
        }
        let region = Rect::new(*x * scale, *y * scale, *rw * scale, *rh * scale);
        if primitives.images.is_empty()
            && render_threading_enabled()
            && let Some(rt) = render_thread
        {
            let patch = rt.rasterize_sync(
                logical_w,
                logical_h,
                scale,
                primitives,
                &[],
                &[],
                &[],
                &[],
                Some(region),
            );
            blit_region(back, &patch, w, h, region);
            continue;
        }
        let buf = scratch.get_or_insert_with(|| FrameBuffer::new_filled(w, h, 255, 255, 255, 255));
        // 上一脏区的越界残留不得串区（blit 只拷 region 列，scratch 其余部分弃用）。
        buf.clear(255, 255, 255, 255);
        render_full_scene_region_into(
            buf,
            primitives,
            font_loader,
            glyph_cache,
            Some(image_cache),
            &[],
            &[],
            &[],
            &[],
            Some(region),
            scale,
        );
        blit_region(back, buf, w, h, region);
    }
}

#[allow(clippy::too_many_arguments)] // 光栅化需要 surface、缩放、图元和两类缓存。
fn rasterize_full(
    w: u32,
    h: u32,
    scale: f32,
    primitives: &RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    render_thread: Option<&RenderingThread>,
    image_cache: &mut ImageCache,
) -> FrameBuffer {
    if primitives.images.is_empty()
        && render_threading_enabled()
        && let Some(rt) = render_thread
    {
        return rt.rasterize_sync(w, h, scale, primitives, &[], &[], &[], &[], None);
    }
    zero_render_foundation::cpu::render_full_scene(
        w,
        h,
        scale,
        primitives,
        font_loader,
        glyph_cache,
        Some(image_cache),
        &[],
        &[],
        &[],
        &[],
    )
}

fn blit_region(dst: &mut FrameBuffer, src: &FrameBuffer, width: u32, _height: u32, region: Rect) {
    let x0 = region.origin.x.max(0.0).floor() as u32;
    let y0 = region.origin.y.max(0.0).floor() as u32;
    let x1 = (region.origin.x + region.size.width).ceil() as u32;
    let y1 = (region.origin.y + region.size.height).ceil() as u32;
    let x1 = x1.min(width);
    let row = (width * 4) as usize;
    for y in y0..y1 {
        let src_off = (y as usize * row) + (x0 as usize * 4);
        let dst_off = src_off;
        let len = ((x1 - x0) * 4) as usize;
        if src_off + len <= src.data.len() && dst_off + len <= dst.data.len() {
            dst.data[dst_off..dst_off + len].copy_from_slice(&src.data[src_off..src_off + len]);
        }
    }
}
