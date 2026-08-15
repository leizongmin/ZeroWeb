//! RFC 4.2-S2：compositor 侧滚动变换。
//!
//! 滚动必须在图元坐标系中完成：front buffer 只包含一个视口，直接平移它会使
//! 滚动后新露出的区域越界为空。合成器保留最近的绘制快照，并以滚动后的坐标
//! 重光栅化当前视口。

use zero_protocol::paint_snapshot::{IpcGradientKind, IpcRect, PaintSnapshotParams};

fn translate_rect(rect: &mut IpcRect, x: f32, y: f32) {
    rect.x += x;
    rect.y += y;
}

/// 返回适用于当前视口的绘制快照。
///
/// `scroll_x`/`scroll_y` 是 CSS 像素；文档坐标向反方向平移，使滚动后的文档
/// 内容落入固定的 viewport backing store。
// https://drafts.csswg.org/cssom-view/#scrolling
pub fn paint_for_viewport(paint: &PaintSnapshotParams, scroll_x: f32, scroll_y: f32) -> PaintSnapshotParams {
    let mut out = paint.clone();
    let x = -scroll_x;
    let y = -scroll_y;

    for fill in &mut out.fills {
        translate_rect(&mut fill.rect, x, y);
    }
    for rect in &mut out.rounded_rects {
        translate_rect(&mut rect.rect, x, y);
    }
    for gradient in &mut out.gradients {
        translate_rect(&mut gradient.rect, x, y);
        match &mut gradient.kind {
            IpcGradientKind::Linear { x0, y0, x1, y1 } => {
                *x0 += x;
                *y0 += y;
                *x1 += x;
                *y1 += y;
            }
            IpcGradientKind::Radial { cx, cy, .. } | IpcGradientKind::Conic { cx, cy, .. } => {
                *cx += x;
                *cy += y;
            }
        }
    }
    for shadow in &mut out.shadows {
        translate_rect(&mut shadow.rect, x, y);
    }
    for image in &mut out.images {
        translate_rect(&mut image.rect, x, y);
        if let Some(clip) = &mut image.clip {
            translate_rect(clip, x, y);
        }
    }
    for stroke in &mut out.strokes {
        stroke.x1 += x;
        stroke.y1 += y;
        stroke.x2 += x;
        stroke.y2 += y;
    }
    for path in &mut out.path_fills {
        for point in path.vertices.chunks_exact_mut(2) {
            point[0] += x;
            point[1] += y;
        }
    }
    for path in &mut out.path_strokes {
        for point in path.vertices.chunks_exact_mut(2) {
            point[0] += x;
            point[1] += y;
        }
    }
    for clip in &mut out.clips {
        translate_rect(&mut clip.rect, x, y);
    }
    for transform in &mut out.transforms {
        translate_rect(&mut transform.rect, x, y);
        transform.origin_x += x;
        transform.origin_y += y;
    }
    for filter in &mut out.filters {
        translate_rect(&mut filter.rect, x, y);
    }
    for blend in &mut out.blend_modes {
        translate_rect(&mut blend.rect, x, y);
    }
    for glyph in &mut out.glyphs {
        glyph.x += x;
        glyph.y += y;
    }

    // 滚动会让任意文档区域进入视口，不能复用文档原坐标下的局部 dirty rect。
    out.dirty_rects = vec![IpcRect {
        x: 0.0,
        y: 0.0,
        width: out.viewport_width.max(1) as f32,
        height: out.viewport_height.max(1) as f32,
    }];
    out
}
