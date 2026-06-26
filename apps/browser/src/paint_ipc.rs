//! 多进程 IPC 绘制快照 ↔ 浏览器 TabSnapshot 转换。

use zero_engine::PipelineTimings;
use zero_protocol::PaintSnapshotParams;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};
use zero_webview::WebViewRenderResult;

use crate::tab_snapshot::TabSnapshot;

/// 将 IPC 绘制快照写入 Tab 快照。
pub fn apply_paint_snapshot(snap: &mut TabSnapshot, params: PaintSnapshotParams) {
    let mut primitives = RenderPrimitives::new();
    for fill in params.fills {
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(fill.rect.x, fill.rect.y, fill.rect.width, fill.rect.height),
            color: Color::rgba(fill.color.r, fill.color.g, fill.color.b, fill.color.a),
        });
    }
    for glyph in params.glyphs {
        primitives.glyphs.push(GlyphPrimitive {
            x: glyph.x,
            y: glyph.y,
            font_size: glyph.font_size,
            color: Color::rgba(glyph.color.r, glyph.color.g, glyph.color.b, glyph.color.a),
            glyph_id: glyph.glyph_id,
            font_id: FontId(glyph.font_id),
            bitmap_width: None,
            bitmap_height: None,
            rotation: glyph.rotation,
        });
    }
    snap.last_render = Some(WebViewRenderResult {
        primitives,
        timings: PipelineTimings::default(),
    });
    snap.document_height = Some(params.document_height);
    snap.loading = false;
}
