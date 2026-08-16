//! 多进程 IPC 绘制快照 ↔ 浏览器 TabSnapshot 转换。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use zero_engine::{
    HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, PipelineTimings, node_id_from_u64,
};
use zero_protocol::{
    IpcBlendMode, IpcColor, IpcDrawOp, IpcFilterKind, IpcGlyphSource, IpcGlyphTextRun, IpcGradientColorSpace,
    IpcGradientInterpolation, IpcGradientKind, IpcHitTestCache, IpcHitTestLayoutNode, IpcHueMethod, IpcLineCap,
    IpcLineStyle, IpcRect, PaintSnapshotParams,
};
// 仅测试用（构造 PaintSnapshotParams 断言）。
#[cfg(test)]
use zero_protocol::{IpcImage, IpcImagePayload};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::OpenTypeVariation;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::{ImageData, ImageKey};
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, DrawOp, FillPrimitive, FilterKind, FilterPrimitive, FontId,
    FontVariationId, GlyphPrimitive, GlyphSource, GradientColorSpace, GradientInterpolation, GradientKind,
    GradientPrimitive, GradientStop, HueMethod, ImagePrimitive, LineCap, LineStyle, PathFillPrimitive,
    PathStrokePrimitive, RenderPrimitives, RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive, TextControlBoundary,
    TransformPrimitive,
};
use zero_webview::WebViewRenderResult;

use crate::tab_snapshot::TabSnapshot;

fn ipc_rect_to_rect(r: IpcRect) -> Rect {
    Rect::new(r.x, r.y, r.width, r.height)
}

fn ipc_color_to_color(c: IpcColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

fn glyph_text_runs_from_ipc(runs: Vec<IpcGlyphTextRun>) -> HashMap<u64, Arc<str>> {
    let mut text_runs = HashMap::new();
    let mut invalid_ids = HashSet::new();
    for run in runs {
        if run.run_id == 0 || invalid_ids.contains(&run.run_id) {
            continue;
        }
        if text_runs.insert(run.run_id, run.text.into()).is_some() {
            text_runs.remove(&run.run_id);
            invalid_ids.insert(run.run_id);
        }
    }
    text_runs
}

fn glyph_source_from_ipc(source: IpcGlyphSource, text_runs: &HashMap<u64, Arc<str>>) -> Option<GlyphSource> {
    let text = text_runs.get(&source.run_id)?.clone();
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
        IpcGradientColorSpace::Hsl => GradientColorSpace::Hsl,
        IpcGradientColorSpace::Hwb => GradientColorSpace::Hwb,
        IpcGradientColorSpace::Xyz => GradientColorSpace::Xyz,
        IpcGradientColorSpace::XyzD50 => GradientColorSpace::XyzD50,
        IpcGradientColorSpace::ProphotoRgb => GradientColorSpace::ProphotoRgb,
        IpcGradientColorSpace::DisplayP3 => GradientColorSpace::DisplayP3,
        IpcGradientColorSpace::DisplayP3Linear => GradientColorSpace::DisplayP3Linear,
        IpcGradientColorSpace::A98Rgb => GradientColorSpace::A98Rgb,
        IpcGradientColorSpace::Rec2020 => GradientColorSpace::Rec2020,
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

/// 将 IPC 绘制快照写入 Tab 快照。
pub fn apply_paint_snapshot(snap: &mut TabSnapshot, params: PaintSnapshotParams) {
    let mut primitives = RenderPrimitives::new();

    for fill in params.fills {
        primitives.fills.push(FillPrimitive {
            rect: ipc_rect_to_rect(fill.rect),
            color: ipc_color_to_color(fill.color),
        });
    }
    for rr in params.rounded_rects {
        primitives.rounded_rects.push(RoundedRectPrimitive {
            rect: ipc_rect_to_rect(rr.rect),
            color: ipc_color_to_color(rr.color),
            top_left_radius: rr.top_left_radius,
            top_right_radius: rr.top_right_radius,
            bottom_right_radius: rr.bottom_right_radius,
            bottom_left_radius: rr.bottom_left_radius,
        });
    }
    for g in params.gradients {
        primitives.gradients.push(GradientPrimitive {
            rect: ipc_rect_to_rect(g.rect),
            kind: ipc_gradient_kind_to_kind(g.kind),
            stops: g
                .stops
                .into_iter()
                .map(|s| GradientStop {
                    offset: s.offset,
                    color: ipc_color_to_color(s.color),
                })
                .collect(),
            repeating: g.repeating,
            interpolation: ipc_interpolation_to_interpolation(g.interpolation),
        });
    }
    for shadow in params.shadows {
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
    for image in params.images {
        primitives.images.push(ImagePrimitive {
            rect: ipc_rect_to_rect(image.rect),
            image_key: ImageKey::new(image.image_key),
            clip: image.clip.map(ipc_rect_to_rect),
        });
    }
    for stroke in params.strokes {
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
    for pf in params.path_fills {
        primitives.path_fills.push(PathFillPrimitive {
            vertices: pf.vertices,
            color: ipc_color_to_color(pf.color),
        });
    }
    for ps in params.path_strokes {
        primitives.path_strokes.push(PathStrokePrimitive {
            vertices: ps.vertices,
            color: ipc_color_to_color(ps.color),
            line_width: ps.line_width,
            closed: ps.closed,
        });
    }
    for clip in params.clips {
        primitives.clips.push(ClipPrimitive {
            rect: ipc_rect_to_rect(clip.rect),
        });
    }
    for transform in params.transforms {
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
    for filter in params.filters {
        primitives.filters.push(FilterPrimitive {
            rect: ipc_rect_to_rect(filter.rect),
            filters: filter.filters.into_iter().map(ipc_filter_kind_to_kind).collect(),
        });
    }
    for blend in params.blend_modes {
        primitives.blend_modes.push(BlendModePrimitive {
            rect: ipc_rect_to_rect(blend.rect),
            mode: ipc_blend_mode_to_mode(blend.mode),
        });
    }
    primitives.font_variations = params
        .font_variations
        .into_iter()
        .map(|variations| -> Arc<[OpenTypeVariation]> {
            if variations
                .iter()
                .copied()
                .all(zero_protocol::IpcFontVariation::is_valid)
            {
                Arc::from(
                    variations
                        .into_iter()
                        .map(|variation| OpenTypeVariation::new(variation.tag, variation.value))
                        .collect::<Vec<_>>(),
                )
            } else {
                Arc::from([])
            }
        })
        .collect();
    let glyph_text_runs = glyph_text_runs_from_ipc(params.glyph_text_runs);
    for glyph in params.glyphs {
        let font_variation_id = glyph
            .font_variation_id
            .filter(|id| usize::try_from(*id).is_ok_and(|index| index < primitives.font_variations.len()))
            .map(FontVariationId);
        primitives.glyphs.push(GlyphPrimitive {
            x: glyph.x,
            y: glyph.y,
            font_size: glyph.font_size,
            color: ipc_color_to_color(glyph.color),
            glyph_id: glyph.glyph_id,
            font_glyph_index: glyph.font_glyph_index,
            source: glyph
                .source
                .and_then(|source| glyph_source_from_ipc(source, &glyph_text_runs)),
            font_id: FontId(glyph.font_id),
            font_variation_id,
            bitmap_width: None,
            bitmap_height: None,
            rotation: glyph.rotation,
            synthetic_italic: glyph.synthetic_italic,
        });
    }
    primitives.text_control_boundaries = params
        .text_control_boundaries
        .into_iter()
        .map(|boundary| TextControlBoundary {
            node_handle: boundary.node_handle,
            utf16_offset: boundary.utf16_offset,
            x: boundary.x,
            y: boundary.y,
            height: boundary.height,
        })
        .collect();
    primitives.draw_order = params.draw_order.into_iter().map(ipc_draw_op_to_draw_op).collect();

    for payload in params.image_payloads {
        if let Ok(data) = ImageData::from_rgba(payload.rgba, payload.width, payload.height) {
            snap.image_cache.insert_with_key(ImageKey::new(payload.image_key), data);
        }
    }

    let document_width = crate::page_scroll::primitives_content_width(&primitives);
    snap.text_control_boundaries = primitives.text_control_boundaries.clone();
    snap.last_render = Some(WebViewRenderResult {
        primitives,
        // S3：保留本帧脏区域（IpcRect → (x,y,w,h)），与 engine→webview 的 render_result_to_webview
        // 对齐；browser 侧当前未消费，但保持数据通路完整以便后续增量重绘接入。
        dirty_rects: params
            .dirty_rects
            .iter()
            .map(|r| (r.x, r.y, r.width, r.height))
            .collect(),
        timings: PipelineTimings::default(),
    });
    snap.document_height = Some(params.document_height);
    snap.document_generation = params.document_generation;
    // 性能门禁优化 S3（2026-08-08）：快照到达时缓存内容宽度（每快照一次 O(P) 扫描，
    // 替代旧实现的每 mousemove/wheel 扫描）
    snap.document_width = Some(document_width);
    snap.hit_test = params.hit_test.and_then(hit_test_cache_from_ipc);
}

/// 从 IPC 命中测试快照还原成 engine 主线程可消费的 `HitTestCache`。
///
/// 多进程模式下，渲染进程在每帧 `ViewPainted` 中携带 hit-test 缓存；
/// 浏览器主线程用它完成本地 hover / 点击命中查询，避免每次交互都发起同步 IPC。
fn hit_test_cache_from_ipc(cache: IpcHitTestCache) -> Option<HitTestCache> {
    let doc_root = node_id_from_u64(cache.doc_root);
    let layout_root = ipc_layout_to_snapshot(&cache.layout_root)?;
    let mut nodes = Vec::with_capacity(cache.nodes.len());
    for (id_u64, meta) in cache.nodes {
        let id = node_id_from_u64(id_u64);
        nodes.push((
            id,
            HitTestNodeSnapshot {
                tag_name: meta.tag_name,
                id: meta.id,
                class_name: meta.class_name,
                selector: meta.selector,
                href: meta.href,
                src: meta.src,
            },
        ));
    }
    let parents = cache
        .parents
        .into_iter()
        .map(|(c, p)| (node_id_from_u64(c), node_id_from_u64(p)))
        .collect::<Vec<_>>();
    Some(HitTestCache::from_snapshot(HitTestCacheSnapshot {
        doc_root,
        layout_root,
        nodes,
        parents,
    }))
}

fn ipc_layout_to_snapshot(node: &IpcHitTestLayoutNode) -> Option<HitTestLayoutSnapshot> {
    let node_id = node.node_id.map(node_id_from_u64);
    let children = node
        .children
        .iter()
        .filter_map(ipc_layout_to_snapshot)
        .collect::<Vec<_>>();
    Some(HitTestLayoutSnapshot {
        node_id,
        x: node.x,
        y: node.y,
        width: node.width,
        height: node.height,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::{IpcGlyph, IpcHitTestCache, IpcHitTestLayoutNode, IpcHitTestNodeMeta, IpcTextControlBoundary};

    #[test]
    fn apply_paint_snapshot_restores_hit_test_cache() {
        let mut snap = TabSnapshot::default();
        let params = PaintSnapshotParams {
            document_height: 42.0,
            document_generation: 9,
            hit_test: Some(IpcHitTestCache {
                doc_root: 1,
                layout_root: IpcHitTestLayoutNode {
                    node_id: Some(1),
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                    children: Vec::new(),
                },
                nodes: std::iter::once((
                    1,
                    IpcHitTestNodeMeta {
                        tag_name: "a".to_string(),
                        id: None,
                        class_name: None,
                        selector: "a".to_string(),
                        href: Some("https://example.com".to_string()),
                        src: None,
                    },
                ))
                .collect(),
                parents: Default::default(),
            }),
            ..Default::default()
        };

        apply_paint_snapshot(&mut snap, params);

        assert!(snap.last_render.is_some(), "frame data should still be applied");
        assert_eq!(snap.document_generation, 9);
        let hit = snap
            .hit_test
            .as_ref()
            .and_then(|cache| cache.hit_test_element(1.0, 1.0))
            .expect("hit");
        assert_eq!(
            hit.node_handle,
            zero_engine::node_id_to_u64(zero_engine::node_id_from_u64(1))
        );
        assert!(
            snap.hit_test.is_some(),
            "browser should restore hit-test cache from IPC snapshot"
        );
    }

    #[test]
    fn apply_paint_snapshot_restores_glyph_source_and_variations() {
        let source = |run_id| IpcGlyphSource {
            run_id,
            start: 0,
            end: 3,
        };
        let glyph = |run_id, glyph_id| IpcGlyph {
            x: 0.0,
            y: 16.0,
            font_size: 16.0,
            glyph_id,
            font_glyph_index: Some(1),
            source: Some(source(run_id)),
            font_id: 1,
            font_variation_id: Some(0),
            color: IpcColor {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            rotation: 0.0,
            synthetic_italic: true,
        };
        let params = PaintSnapshotParams {
            font_variations: vec![vec![zero_protocol::IpcFontVariation {
                tag: *b"wdth",
                value: 125.0,
            }]],
            glyph_text_runs: vec![
                IpcGlyphTextRun {
                    run_id: 7,
                    text: "A\u{301}".to_string(),
                },
                IpcGlyphTextRun {
                    run_id: 8,
                    text: "A\u{301}".to_string(),
                },
            ],
            glyphs: vec![glyph(7, 'A' as u32), glyph(7, '\u{301}' as u32), glyph(8, 'A' as u32)],
            text_control_boundaries: vec![IpcTextControlBoundary {
                node_handle: 4,
                utf16_offset: 2,
                x: 12.5,
                y: 8.0,
                height: 18.0,
            }],
            ..Default::default()
        };
        let mut snap = TabSnapshot::default();

        apply_paint_snapshot(&mut snap, params);

        let primitives = snap.last_render.as_ref().expect("render result").primitives();
        let glyphs = &primitives.glyphs;
        let first = glyphs[0].source.as_ref().expect("first source");
        let second = glyphs[1].source.as_ref().expect("second source");
        let independent = glyphs[2].source.as_ref().expect("independent source");
        assert!(glyphs[0].synthetic_italic);
        assert_eq!(
            primitives.glyph_font_variations(&glyphs[0]),
            &[OpenTypeVariation::new(*b"wdth", 125.0)]
        );
        assert!(first.same_cluster(second));
        assert!(!first.same_cluster(independent));
        assert_eq!(
            snap.last_render
                .as_ref()
                .expect("render result")
                .primitives()
                .text_control_boundaries[0]
                .utf16_offset,
            2
        );
    }

    #[test]
    fn apply_paint_snapshot_keeps_new_image_payload_available() {
        let mut snap = TabSnapshot::default();
        let params = PaintSnapshotParams {
            images: vec![IpcImage {
                rect: IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: 4.0,
                    height: 4.0,
                },
                image_key: 99,
                clip: None,
            }],
            image_payloads: vec![IpcImagePayload {
                image_key: 99,
                width: 2,
                height: 2,
                rgba: [255u8, 0, 0, 255].repeat(4),
            }],
            ..Default::default()
        };

        apply_paint_snapshot(&mut snap, params);

        assert!(
            snap.image_cache.get(&ImageKey::new(99)).is_some(),
            "newly injected image payload should remain available for immediate rendering"
        );
    }
}
