//! 渲染进程：RenderPrimitives → IPC 绘制快照。

use zero_engine::{extract_img_srcs, image_resource_key, resolve_document_url};
use zero_protocol::{
    IpcColor, IpcDrawOp, IpcFill, IpcGlyph, IpcGradient, IpcGradientKind, IpcGradientStop, IpcImage, IpcImagePayload,
    IpcLineCap, IpcLineStyle, IpcRect, IpcRoundedRect, IpcShadow, IpcStroke, PaintSnapshotParams,
};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::decode_image_bytes;
use zero_render_foundation::primitive::{
    DrawOp, GradientKind, GradientPrimitive, LineCap, LineStyle, RenderPrimitives, ShadowPrimitive, StrokePrimitive,
};

fn rect_to_ipc(r: &Rect) -> IpcRect {
    IpcRect {
        x: r.origin.x,
        y: r.origin.y,
        width: r.size.width,
        height: r.size.height,
    }
}

fn color_to_ipc(c: Color) -> IpcColor {
    IpcColor {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

fn gradient_kind_to_ipc(k: &GradientKind) -> IpcGradientKind {
    match k {
        GradientKind::Linear { x0, y0, x1, y1 } => IpcGradientKind::Linear {
            x0: *x0,
            y0: *y0,
            x1: *x1,
            y1: *y1,
        },
        GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => IpcGradientKind::Radial {
            cx: *cx,
            cy: *cy,
            inner_radius: *inner_radius,
            outer_radius: *outer_radius,
        },
        GradientKind::Conic { cx, cy, start_angle } => IpcGradientKind::Conic {
            cx: *cx,
            cy: *cy,
            start_angle: *start_angle,
        },
    }
}

fn line_cap_to_ipc(c: LineCap) -> IpcLineCap {
    match c {
        LineCap::Butt => IpcLineCap::Butt,
        LineCap::Round => IpcLineCap::Round,
        LineCap::Square => IpcLineCap::Square,
    }
}

fn line_style_to_ipc(s: LineStyle) -> IpcLineStyle {
    match s {
        LineStyle::Solid => IpcLineStyle::Solid,
        LineStyle::Dashed => IpcLineStyle::Dashed,
        LineStyle::Dotted => IpcLineStyle::Dotted,
    }
}

fn draw_op_to_ipc(op: DrawOp) -> Option<IpcDrawOp> {
    Some(match op {
        DrawOp::Fill(i) => IpcDrawOp::Fill(i),
        DrawOp::RoundedRect(i) => IpcDrawOp::RoundedRect(i),
        DrawOp::Gradient(i) => IpcDrawOp::Gradient(i),
        DrawOp::Shadow(i) => IpcDrawOp::Shadow(i),
        DrawOp::Image(i) => IpcDrawOp::Image(i),
        DrawOp::Stroke(i) => IpcDrawOp::Stroke(i),
        DrawOp::Glyph(i) => IpcDrawOp::Glyph(i),
        DrawOp::PathFill(_)
        | DrawOp::PathStroke(_)
        | DrawOp::Filter(_)
        | DrawOp::BlendMode(_)
        | DrawOp::Transform(_)
        | DrawOp::Clip(_) => return None,
    })
}

/// 抓取 HTML 中 `<img>` 子资源并编码为 IPC 像素块。
pub fn fetch_image_payloads(html: &str, page_url: &str) -> Vec<IpcImagePayload> {
    let client = zero_net::client::HttpClient::new();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for src in extract_img_srcs(html) {
        let key = image_resource_key(&src, Some(page_url));
        if !seen.insert(key) {
            continue;
        }
        if src.starts_with("data:") {
            continue;
        }
        let resolved = resolve_document_url(page_url, &src);
        let Ok(response) = client.get(&resolved) else {
            continue;
        };
        let Ok(data) = decode_image_bytes(&response.body) else {
            continue;
        };
        out.push(IpcImagePayload {
            image_key: key,
            width: data.width,
            height: data.height,
            rgba: data.pixels,
        });
    }
    out
}

/// 从渲染图元构建 IPC 绘制快照。
pub fn paint_snapshot_from_primitives(
    viewport_width: u32,
    viewport_height: u32,
    document_height: f32,
    primitives: &RenderPrimitives,
    image_payloads: Vec<IpcImagePayload>,
) -> PaintSnapshotParams {
    PaintSnapshotParams {
        viewport_width,
        viewport_height,
        document_height,
        fills: primitives
            .fills
            .iter()
            .map(|f| IpcFill {
                rect: rect_to_ipc(&f.rect),
                color: color_to_ipc(f.color),
            })
            .collect(),
        rounded_rects: primitives
            .rounded_rects
            .iter()
            .map(|rr| IpcRoundedRect {
                rect: rect_to_ipc(&rr.rect),
                color: color_to_ipc(rr.color),
                top_left_radius: rr.top_left_radius,
                top_right_radius: rr.top_right_radius,
                bottom_right_radius: rr.bottom_right_radius,
                bottom_left_radius: rr.bottom_left_radius,
            })
            .collect(),
        gradients: primitives
            .gradients
            .iter()
            .map(|g: &GradientPrimitive| IpcGradient {
                rect: rect_to_ipc(&g.rect),
                kind: gradient_kind_to_ipc(&g.kind),
                stops: g
                    .stops
                    .iter()
                    .map(|s| IpcGradientStop {
                        offset: s.offset,
                        color: color_to_ipc(s.color),
                    })
                    .collect(),
                repeating: g.repeating,
            })
            .collect(),
        shadows: primitives
            .shadows
            .iter()
            .map(|s: &ShadowPrimitive| IpcShadow {
                rect: rect_to_ipc(&s.rect),
                color: color_to_ipc(s.color),
                offset_x: s.offset_x,
                offset_y: s.offset_y,
                blur_radius: s.blur_radius,
                spread_radius: s.spread_radius,
            })
            .collect(),
        images: primitives
            .images
            .iter()
            .map(|img| IpcImage {
                rect: rect_to_ipc(&img.rect),
                image_key: img.image_key.0,
                clip: img.clip.as_ref().map(rect_to_ipc),
            })
            .collect(),
        image_payloads,
        strokes: primitives
            .strokes
            .iter()
            .map(|s: &StrokePrimitive| IpcStroke {
                x1: s.x1,
                y1: s.y1,
                x2: s.x2,
                y2: s.y2,
                width: s.width,
                color: color_to_ipc(s.color),
                style: line_style_to_ipc(s.style),
                cap: line_cap_to_ipc(s.cap),
            })
            .collect(),
        glyphs: primitives
            .glyphs
            .iter()
            .map(|g| IpcGlyph {
                x: g.x,
                y: g.y,
                font_size: g.font_size,
                glyph_id: g.glyph_id,
                font_id: g.font_id.0,
                color: color_to_ipc(g.color),
                rotation: g.rotation,
            })
            .collect(),
        draw_order: primitives
            .draw_order
            .iter()
            .filter_map(|op| draw_op_to_ipc(*op))
            .collect(),
    }
}
