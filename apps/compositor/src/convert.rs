//! IPC 图元快照（PaintSnapshotParams）→ 渲染图元（RenderPrimitives）转换。
//!
//! 与 browser 的 paint_ipc::apply_paint_snapshot 保持同一映射（2026-08-07
//! 对照实现）；compositor 在合成器进程内完成光栅化所需的转换。

use std::collections::HashMap;
use std::sync::Arc;
use zero_protocol::paint_snapshot::{
    IpcBlendMode, IpcColor, IpcDrawOp, IpcFilterKind, IpcGlyphSource, IpcGradientColorSpace, IpcGradientInterpolation,
    IpcGradientKind, IpcHueMethod, IpcLineCap, IpcLineStyle, IpcRect, PaintSnapshotParams,
};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, DrawOp, FillPrimitive, FilterKind, FilterPrimitive, GlyphPrimitive,
    GlyphSource, GradientColorSpace, GradientInterpolation, GradientKind, GradientPrimitive, GradientStop, HueMethod,
    ImagePrimitive, LineCap, LineStyle, PathFillPrimitive, PathStrokePrimitive, RenderPrimitives, RoundedRectPrimitive,
    ShadowPrimitive, StrokePrimitive, TransformPrimitive,
};

fn ipc_rect_to_rect(r: IpcRect) -> Rect {
    Rect::new(r.x, r.y, r.width, r.height)
}

fn ipc_color_to_color(c: IpcColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

fn glyph_source_from_ipc(source: &IpcGlyphSource, text_runs: &mut HashMap<u64, Arc<str>>) -> Option<GlyphSource> {
    if source.run_id == 0 {
        return None;
    }
    let text = match text_runs.entry(source.run_id) {
        std::collections::hash_map::Entry::Vacant(entry) => entry.insert(source.text.clone().into()).clone(),
        std::collections::hash_map::Entry::Occupied(entry) if entry.get().as_ref() == source.text => {
            entry.get().clone()
        }
        std::collections::hash_map::Entry::Occupied(_) => return None,
    };
    GlyphSource::new(text, source.start, source.end)
}

fn ipc_gradient_kind_to_kind(k: IpcGradientKind) -> GradientKind {
    match k {
        IpcGradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear { x0, y0, x1, y1 },
        IpcGradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        },
        IpcGradientKind::Conic { cx, cy, start_angle } => GradientKind::Conic { cx, cy, start_angle },
    }
}

fn ipc_interpolation_to_interpolation(i: IpcGradientInterpolation) -> GradientInterpolation {
    let space = match i.space {
        IpcGradientColorSpace::Srgb => GradientColorSpace::Srgb,
        IpcGradientColorSpace::SrgbLinear => GradientColorSpace::SrgbLinear,
        IpcGradientColorSpace::Lab => GradientColorSpace::Lab,
        IpcGradientColorSpace::Oklab => GradientColorSpace::Oklab,
        IpcGradientColorSpace::Lch => GradientColorSpace::Lch,
        IpcGradientColorSpace::Oklch => GradientColorSpace::Oklch,
    };
    let hue = match i.hue {
        IpcHueMethod::Shorter => HueMethod::Shorter,
        IpcHueMethod::Longer => HueMethod::Longer,
        IpcHueMethod::Increasing => HueMethod::Increasing,
        IpcHueMethod::Decreasing => HueMethod::Decreasing,
    };
    GradientInterpolation { space, hue }
}

fn ipc_line_cap(c: IpcLineCap) -> LineCap {
    match c {
        IpcLineCap::Butt => LineCap::Butt,
        IpcLineCap::Round => LineCap::Round,
        IpcLineCap::Square => LineCap::Square,
    }
}

fn ipc_line_style(s: IpcLineStyle) -> LineStyle {
    match s {
        IpcLineStyle::Solid => LineStyle::Solid,
        IpcLineStyle::Dashed => LineStyle::Dashed,
        IpcLineStyle::Dotted => LineStyle::Dotted,
    }
}

fn ipc_filter_kind_to_kind(k: IpcFilterKind) -> FilterKind {
    match k {
        IpcFilterKind::Blur(v) => FilterKind::Blur(v),
        IpcFilterKind::Brightness(v) => FilterKind::Brightness(v),
        IpcFilterKind::Contrast(v) => FilterKind::Contrast(v),
        IpcFilterKind::Grayscale(v) => FilterKind::Grayscale(v),
        IpcFilterKind::HueRotate(v) => FilterKind::HueRotate(v),
        IpcFilterKind::Invert(v) => FilterKind::Invert(v),
        IpcFilterKind::Opacity(v) => FilterKind::Opacity(v),
        IpcFilterKind::Saturate(v) => FilterKind::Saturate(v),
        IpcFilterKind::Sepia(v) => FilterKind::Sepia(v),
        IpcFilterKind::DropShadow {
            offset_x,
            offset_y,
            blur,
            color,
        } => FilterKind::DropShadow(offset_x, offset_y, blur, ipc_color_to_color(color)),
    }
}

fn ipc_blend_mode_to_mode(mode: IpcBlendMode) -> BlendMode {
    match mode {
        IpcBlendMode::Normal => BlendMode::Normal,
        IpcBlendMode::Multiply => BlendMode::Multiply,
        IpcBlendMode::Screen => BlendMode::Screen,
        IpcBlendMode::Overlay => BlendMode::Overlay,
        IpcBlendMode::Darken => BlendMode::Darken,
        IpcBlendMode::Lighten => BlendMode::Lighten,
        IpcBlendMode::ColorDodge => BlendMode::ColorDodge,
        IpcBlendMode::ColorBurn => BlendMode::ColorBurn,
        IpcBlendMode::HardLight => BlendMode::HardLight,
        IpcBlendMode::SoftLight => BlendMode::SoftLight,
        IpcBlendMode::Difference => BlendMode::Difference,
        IpcBlendMode::Exclusion => BlendMode::Exclusion,
        IpcBlendMode::Hue => BlendMode::Hue,
        IpcBlendMode::Saturation => BlendMode::Saturation,
        IpcBlendMode::Color => BlendMode::Color,
        IpcBlendMode::Luminosity => BlendMode::Luminosity,
    }
}

fn ipc_draw_op_to_draw_op(op: IpcDrawOp) -> DrawOp {
    match op {
        IpcDrawOp::Fill(i) => DrawOp::Fill(i),
        IpcDrawOp::RoundedRect(i) => DrawOp::RoundedRect(i),
        IpcDrawOp::Gradient(i) => DrawOp::Gradient(i),
        IpcDrawOp::Shadow(i) => DrawOp::Shadow(i),
        IpcDrawOp::Image(i) => DrawOp::Image(i),
        IpcDrawOp::Stroke(i) => DrawOp::Stroke(i),
        IpcDrawOp::PathFill(i) => DrawOp::PathFill(i),
        IpcDrawOp::PathStroke(i) => DrawOp::PathStroke(i),
        IpcDrawOp::Clip(i) => DrawOp::Clip(i),
        IpcDrawOp::Transform(i) => DrawOp::Transform(i),
        IpcDrawOp::Filter(i) => DrawOp::Filter(i),
        IpcDrawOp::BlendMode(i) => DrawOp::BlendMode(i),
        IpcDrawOp::Glyph(i) => DrawOp::Glyph(i),
    }
}

/// 将 IPC 图元快照转换为渲染图元（合成器进程内光栅化输入）。
pub fn to_render_primitives(params: &PaintSnapshotParams) -> RenderPrimitives {
    let mut primitives = RenderPrimitives::new();

    for fill in &params.fills {
        primitives.fills.push(FillPrimitive {
            rect: ipc_rect_to_rect(fill.rect),
            color: ipc_color_to_color(fill.color),
        });
    }
    for rr in &params.rounded_rects {
        primitives.rounded_rects.push(RoundedRectPrimitive {
            rect: ipc_rect_to_rect(rr.rect),
            color: ipc_color_to_color(rr.color),
            top_left_radius: rr.top_left_radius,
            top_right_radius: rr.top_right_radius,
            bottom_right_radius: rr.bottom_right_radius,
            bottom_left_radius: rr.bottom_left_radius,
        });
    }
    for g in &params.gradients {
        primitives.gradients.push(GradientPrimitive {
            rect: ipc_rect_to_rect(g.rect),
            kind: ipc_gradient_kind_to_kind(g.kind.clone()),
            stops: g
                .stops
                .iter()
                .map(|s| GradientStop {
                    offset: s.offset,
                    color: ipc_color_to_color(s.color),
                })
                .collect(),
            repeating: g.repeating,
            interpolation: ipc_interpolation_to_interpolation(g.interpolation),
        });
    }
    for shadow in &params.shadows {
        primitives.shadows.push(ShadowPrimitive {
            rect: ipc_rect_to_rect(shadow.rect),
            color: ipc_color_to_color(shadow.color),
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur_radius: shadow.blur_radius,
            spread_radius: shadow.spread_radius,
            inset: false,
        });
    }
    for image in &params.images {
        primitives.images.push(ImagePrimitive {
            rect: ipc_rect_to_rect(image.rect),
            image_key: ImageKey::new(image.image_key),
            clip: image.clip.map(ipc_rect_to_rect),
        });
    }
    for stroke in &params.strokes {
        primitives.strokes.push(StrokePrimitive {
            x1: stroke.x1,
            y1: stroke.y1,
            x2: stroke.x2,
            y2: stroke.y2,
            width: stroke.width,
            color: ipc_color_to_color(stroke.color),
            style: ipc_line_style(stroke.style),
            cap: ipc_line_cap(stroke.cap),
        });
    }
    for pf in &params.path_fills {
        primitives.path_fills.push(PathFillPrimitive {
            vertices: pf.vertices.clone(),
            color: ipc_color_to_color(pf.color),
        });
    }
    for ps in &params.path_strokes {
        primitives.path_strokes.push(PathStrokePrimitive {
            vertices: ps.vertices.clone(),
            color: ipc_color_to_color(ps.color),
            line_width: ps.line_width,
            closed: ps.closed,
        });
    }
    for clip in &params.clips {
        primitives.clips.push(ClipPrimitive {
            rect: ipc_rect_to_rect(clip.rect),
        });
    }
    for transform in &params.transforms {
        primitives.transforms.push(TransformPrimitive {
            rect: ipc_rect_to_rect(transform.rect),
            origin_x: transform.origin_x,
            origin_y: transform.origin_y,
            a: transform.a,
            b: transform.b,
            c: transform.c,
            d: transform.d,
            tx: transform.tx,
            ty: transform.ty,
        });
    }
    for filter in &params.filters {
        primitives.filters.push(FilterPrimitive {
            rect: ipc_rect_to_rect(filter.rect),
            filters: filter.filters.iter().cloned().map(ipc_filter_kind_to_kind).collect(),
        });
    }
    for blend in &params.blend_modes {
        primitives.blend_modes.push(BlendModePrimitive {
            rect: ipc_rect_to_rect(blend.rect),
            mode: ipc_blend_mode_to_mode(blend.mode),
        });
    }
    let mut glyph_text_runs = HashMap::new();
    for glyph in &params.glyphs {
        primitives.glyphs.push(GlyphPrimitive {
            x: glyph.x,
            y: glyph.y,
            font_size: glyph.font_size,
            color: ipc_color_to_color(glyph.color),
            glyph_id: glyph.glyph_id,
            font_glyph_index: glyph.font_glyph_index,
            source: glyph
                .source
                .as_ref()
                .and_then(|source| glyph_source_from_ipc(source, &mut glyph_text_runs)),
            font_id: zero_render_foundation::primitive::FontId(glyph.font_id),
            bitmap_width: None,
            bitmap_height: None,
            rotation: glyph.rotation,
            synthetic_italic: false,
        });
    }
    primitives.draw_order = params.draw_order.iter().map(|op| ipc_draw_op_to_draw_op(*op)).collect();
    primitives
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_source_decoder_interns_run_and_rejects_conflicting_text() {
        let source = IpcGlyphSource {
            run_id: 4,
            text: "A\u{301}".to_string(),
            start: 0,
            end: 3,
        };
        let mut text_runs = HashMap::new();
        let first = glyph_source_from_ipc(&source, &mut text_runs).expect("first source");
        let second = glyph_source_from_ipc(&source, &mut text_runs).expect("second source");
        assert!(first.same_cluster(&second));

        let conflicting = IpcGlyphSource {
            text: "different".to_string(),
            ..source
        };
        assert!(glyph_source_from_ipc(&conflicting, &mut text_runs).is_none());
    }
}
