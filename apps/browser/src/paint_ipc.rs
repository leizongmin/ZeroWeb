//! 多进程 IPC 绘制快照 ↔ 浏览器 TabSnapshot 转换。
//!
//! IPC 图元字段 → `RenderPrimitives` 的映射由公共层 [`zero_paint_convert`]
//! 提供（2026-09-09 收敛 compositor `convert.rs` 双实现）；本模块在其上组合
//! browser 侧副作用：image payload 注入 `TabSnapshot.image_cache`、文档几何
//! 缓存与 hit-test 快照还原。

use zero_engine::{HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64};
use zero_protocol::{IpcHitTestCache, IpcHitTestLayoutNode, PaintSnapshotParams};
// 仅测试用（构造 PaintSnapshotParams 断言）。
#[cfg(test)]
use zero_protocol::{IpcImage, IpcImagePayload};
use zero_render_foundation::image_cache::{ImageData, ImageKey};
use zero_render_foundation::primitive::RenderPrimitives;

use crate::tab_snapshot::{PageRenderResult, TabSnapshot};

/// 将 IPC 绘制快照写入 Tab 快照。
pub fn apply_paint_snapshot(snap: &mut TabSnapshot, params: PaintSnapshotParams) {
    let primitives: RenderPrimitives = zero_paint_convert::to_render_primitives(params.clone());

    for payload in params.image_payloads {
        if let Ok(data) = ImageData::from_rgba(payload.rgba, payload.width, payload.height) {
            snap.image_cache.insert_with_key(ImageKey::new(payload.image_key), data);
        }
    }

    let document_width = crate::page_scroll::primitives_content_width(&primitives);
    let painted_content_height = crate::page_scroll::primitives_scrollable_content_height(
        &primitives,
        params.viewport_width as f32,
        params.viewport_height as f32,
    );
    snap.text_control_boundaries = primitives.text_control_boundaries.clone();
    snap.last_render = Some(PageRenderResult {
        primitives,
        // S3：保留本帧脏区域（IpcRect → (x,y,w,h)），与 engine→webview 的 render_result_to_webview
        // 对齐；browser 侧当前未消费，但保持数据通路完整以便后续增量重绘接入。
        dirty_rects: params
            .dirty_rects
            .iter()
            .map(|r| (r.x, r.y, r.width, r.height))
            .collect(),
    });
    snap.document_height = Some(params.document_height);
    snap.document_generation = params.document_generation;
    // 性能门禁优化 S3（2026-08-08）：快照到达时缓存内容宽度（每快照一次 O(P) 扫描，
    // 替代旧实现的每 mousemove/wheel 扫描）
    snap.document_width = Some(document_width);
    snap.painted_content_height = Some(painted_content_height);
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
    use zero_protocol::{
        IpcColor, IpcGlyph, IpcGlyphSource, IpcGlyphTextRun, IpcHitTestCache, IpcHitTestLayoutNode, IpcHitTestNodeMeta,
        IpcTextControlBoundary,
    };
    use zero_render_foundation::font::OpenTypeVariation;

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

        let primitives = &snap.last_render.as_ref().expect("render result").primitives;
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
                .primitives
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
                rect: zero_protocol::IpcRect {
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
