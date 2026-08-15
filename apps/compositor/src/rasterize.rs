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
    let dirty = ipc_dirty_rects(&paint.dirty_rects);
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
        // caller 已 copy_front_to_back；若未复制则清底
    } else {
        back.clear(255, 255, 255, 255);
    }

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
        render_full_scene_region_into(
            back,
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
