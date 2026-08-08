//! 渲染进程：RenderPrimitives → IPC 绘制快照。

use zero_engine::{HitTestCache, HitTestLayoutSnapshot, node_id_to_u64};
use zero_engine::{extract_img_srcs, image_resource_key, resolve_document_url};
use zero_protocol::{
    IpcBlendMode, IpcBlendModePrimitive, IpcClip, IpcColor, IpcDrawOp, IpcFill, IpcFilter, IpcFilterKind, IpcGlyph,
    IpcGradient, IpcGradientColorSpace, IpcGradientInterpolation, IpcGradientKind, IpcGradientStop, IpcHitTestCache,
    IpcHitTestLayoutNode, IpcHitTestNodeMeta, IpcHueMethod, IpcImage, IpcImagePayload, IpcLineCap, IpcLineStyle,
    IpcPathFill, IpcPathStroke, IpcRect, IpcRoundedRect, IpcShadow, IpcStroke, IpcTransform, PaintSnapshotParams,
};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::{decode_data_uri, decode_image_bytes};
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, DrawOp, FilterKind, FilterPrimitive, GradientColorSpace,
    GradientKind, GradientPrimitive, HueMethod, LineCap, LineStyle, PathFillPrimitive, PathStrokePrimitive,
    RenderPrimitives, ShadowPrimitive, StrokePrimitive, TransformPrimitive,
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

fn filter_kind_to_ipc(k: &FilterKind) -> IpcFilterKind {
    match k {
        FilterKind::Blur(v) => IpcFilterKind::Blur(*v),
        FilterKind::Brightness(v) => IpcFilterKind::Brightness(*v),
        FilterKind::Contrast(v) => IpcFilterKind::Contrast(*v),
        FilterKind::Grayscale(v) => IpcFilterKind::Grayscale(*v),
        FilterKind::HueRotate(v) => IpcFilterKind::HueRotate(*v),
        FilterKind::Invert(v) => IpcFilterKind::Invert(*v),
        FilterKind::Opacity(v) => IpcFilterKind::Opacity(*v),
        FilterKind::Saturate(v) => IpcFilterKind::Saturate(*v),
        FilterKind::Sepia(v) => IpcFilterKind::Sepia(*v),
        FilterKind::DropShadow(x, y, blur, color) => IpcFilterKind::DropShadow {
            offset_x: *x,
            offset_y: *y,
            blur: *blur,
            color: color_to_ipc(*color),
        },
    }
}

fn blend_mode_to_ipc(mode: BlendMode) -> IpcBlendMode {
    match mode {
        BlendMode::Normal => IpcBlendMode::Normal,
        BlendMode::Multiply => IpcBlendMode::Multiply,
        BlendMode::Screen => IpcBlendMode::Screen,
        BlendMode::Overlay => IpcBlendMode::Overlay,
        BlendMode::Darken => IpcBlendMode::Darken,
        BlendMode::Lighten => IpcBlendMode::Lighten,
        BlendMode::ColorDodge => IpcBlendMode::ColorDodge,
        BlendMode::ColorBurn => IpcBlendMode::ColorBurn,
        BlendMode::HardLight => IpcBlendMode::HardLight,
        BlendMode::SoftLight => IpcBlendMode::SoftLight,
        BlendMode::Difference => IpcBlendMode::Difference,
        BlendMode::Exclusion => IpcBlendMode::Exclusion,
        BlendMode::Hue => IpcBlendMode::Hue,
        BlendMode::Saturation => IpcBlendMode::Saturation,
        BlendMode::Color => IpcBlendMode::Color,
        BlendMode::Luminosity => IpcBlendMode::Luminosity,
    }
}

fn draw_op_to_ipc(op: DrawOp) -> IpcDrawOp {
    match op {
        DrawOp::Fill(i) => IpcDrawOp::Fill(i),
        DrawOp::RoundedRect(i) => IpcDrawOp::RoundedRect(i),
        DrawOp::Gradient(i) => IpcDrawOp::Gradient(i),
        DrawOp::Shadow(i) => IpcDrawOp::Shadow(i),
        DrawOp::Image(i) => IpcDrawOp::Image(i),
        DrawOp::Stroke(i) => IpcDrawOp::Stroke(i),
        DrawOp::PathFill(i) => IpcDrawOp::PathFill(i),
        DrawOp::PathStroke(i) => IpcDrawOp::PathStroke(i),
        DrawOp::Clip(i) => IpcDrawOp::Clip(i),
        DrawOp::Transform(i) => IpcDrawOp::Transform(i),
        DrawOp::Filter(i) => IpcDrawOp::Filter(i),
        DrawOp::BlendMode(i) => IpcDrawOp::BlendMode(i),
        DrawOp::Glyph(i) => IpcDrawOp::Glyph(i),
    }
}

/// 优先从已解码 `ImageCache` 取图，缺失时再经 fetch 回调抓取。
///
/// `sent_keys`（性能门禁优化 S8，2026-08-08）：已发送过且 browser 端 ImageCache
/// 已存的 key 不再重传像素——DOM 变更后每次 publish 全量 clone 图片像素是
/// ViewPainted IPC 体积的大头。仅「成功取到数据」的 key 标记 sent（fetch 失败
/// 的不标，下次仍重试——负缓存由 fetch 层处理）。
pub fn fetch_image_payloads_with_cache<F>(
    html: &str,
    page_url: &str,
    cache: &mut zero_render_foundation::image_cache::ImageCache,
    fetch: &mut F,
    sent_keys: &mut std::collections::HashSet<u64>,
) -> Vec<IpcImagePayload>
where
    F: FnMut(&str) -> Option<Vec<u8>>,
{
    use zero_render_foundation::image_cache::ImageKey;

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for src in extract_img_srcs(html) {
        let resolved = resolve_document_url(page_url, &src);
        let key = image_resource_key(&resolved, None);
        if !seen.insert(key) {
            continue;
        }
        let data = if let Some(img) = cache.get(&ImageKey::new(key)) {
            img.clone()
        } else if src.starts_with("data:") {
            // R1705：data URI 自包含（无文件系统/网络），inline 解码（不经 fetch 回调）。
            match decode_data_uri(&resolved) {
                Ok(data) => data,
                Err(_) => continue,
            }
        } else if let Some(body) = fetch(&resolved) {
            match decode_image_bytes(&body) {
                Ok(data) => data,
                Err(_) => continue,
            }
        } else {
            continue;
        };
        if sent_keys.contains(&key) {
            // S8：browser 端 ImageCache 已有该 key 像素，不重传
            continue;
        }
        sent_keys.insert(key);
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
    hit_test: Option<HitTestCache>,
    navigation_epoch: u64,
) -> PaintSnapshotParams {
    PaintSnapshotParams {
        viewport_width,
        viewport_height,
        document_height,
        navigation_epoch,
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
                interpolation: IpcGradientInterpolation {
                    space: match g.interpolation.space {
                        GradientColorSpace::Srgb => IpcGradientColorSpace::Srgb,
                        GradientColorSpace::SrgbLinear => IpcGradientColorSpace::SrgbLinear,
                        GradientColorSpace::Lab => IpcGradientColorSpace::Lab,
                        GradientColorSpace::Oklab => IpcGradientColorSpace::Oklab,
                        GradientColorSpace::Lch => IpcGradientColorSpace::Lch,
                        GradientColorSpace::Oklch => IpcGradientColorSpace::Oklch,
                    },
                    hue: match g.interpolation.hue {
                        HueMethod::Shorter => IpcHueMethod::Shorter,
                        HueMethod::Longer => IpcHueMethod::Longer,
                        HueMethod::Increasing => IpcHueMethod::Increasing,
                        HueMethod::Decreasing => IpcHueMethod::Decreasing,
                    },
                },
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
        path_fills: primitives
            .path_fills
            .iter()
            .map(|pf: &PathFillPrimitive| IpcPathFill {
                vertices: pf.vertices.clone(),
                color: color_to_ipc(pf.color),
            })
            .collect(),
        path_strokes: primitives
            .path_strokes
            .iter()
            .map(|ps: &PathStrokePrimitive| IpcPathStroke {
                vertices: ps.vertices.clone(),
                color: color_to_ipc(ps.color),
                line_width: ps.line_width,
                closed: ps.closed,
            })
            .collect(),
        clips: primitives
            .clips
            .iter()
            .map(|c: &ClipPrimitive| IpcClip {
                rect: rect_to_ipc(&c.rect),
            })
            .collect(),
        transforms: primitives
            .transforms
            .iter()
            .map(|t: &TransformPrimitive| IpcTransform {
                rect: rect_to_ipc(&t.rect),
                origin_x: t.origin_x,
                origin_y: t.origin_y,
                a: t.a,
                b: t.b,
                c: t.c,
                d: t.d,
                tx: t.tx,
                ty: t.ty,
            })
            .collect(),
        filters: primitives
            .filters
            .iter()
            .map(|f: &FilterPrimitive| IpcFilter {
                rect: rect_to_ipc(&f.rect),
                filters: f.filters.iter().map(filter_kind_to_ipc).collect(),
            })
            .collect(),
        blend_modes: primitives
            .blend_modes
            .iter()
            .map(|b: &BlendModePrimitive| IpcBlendModePrimitive {
                rect: rect_to_ipc(&b.rect),
                mode: blend_mode_to_ipc(b.mode),
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
        draw_order: primitives.draw_order.iter().copied().map(draw_op_to_ipc).collect(),
        hit_test: hit_test.map(hit_test_cache_to_ipc),
    }
}

fn hit_test_cache_to_ipc(cache: HitTestCache) -> IpcHitTestCache {
    let snap = cache.snapshot();
    IpcHitTestCache {
        doc_root: node_id_to_u64(snap.doc_root),
        layout_root: hit_test_layout_to_ipc(&snap.layout_root),
        nodes: snap
            .nodes
            .into_iter()
            .map(|(id, meta)| {
                (
                    node_id_to_u64(id),
                    IpcHitTestNodeMeta {
                        tag_name: meta.tag_name,
                        id: meta.id,
                        class_name: meta.class_name,
                        href: meta.href,
                        src: meta.src,
                    },
                )
            })
            .collect(),
        parents: snap
            .parents
            .into_iter()
            .map(|(child, parent)| (node_id_to_u64(child), node_id_to_u64(parent)))
            .collect(),
    }
}

fn hit_test_layout_to_ipc(node: &HitTestLayoutSnapshot) -> IpcHitTestLayoutNode {
    IpcHitTestLayoutNode {
        node_id: node.node_id.map(node_id_to_u64),
        x: node.x,
        y: node.y,
        width: node.width,
        height: node.height,
        children: node.children.iter().map(hit_test_layout_to_ipc).collect(),
    }
}
