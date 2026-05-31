//! Engine-core 基准测试 — 绘制、脏区域追踪、合成层分析、端到端管线。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::Document;
use zero_engine::{DirtyTracker, Painter, RenderPipeline, promote_compositing_layers};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

// ── 辅助函数 ──────────────────────────────────────────────────────

/// 创建扁平的 LayoutBox 树（N 个子节点）。
fn make_flat_layout(n: usize, with_background: bool) -> (LayoutBox, HashMap<zero_dom::NodeId, ComputedStyle>) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let mut styles = HashMap::new();
    let mut children = Vec::with_capacity(n);

    for i in 0..n {
        let elem = doc.create_element("div");
        doc.append_child(body, elem).unwrap();

        let mut style = ComputedStyle::default();
        if with_background {
            style.background_color =
                ColorValue::Rgba((i % 256) as u8, ((i * 3) % 256) as u8, ((i * 7) % 256) as u8, 255);
        }
        styles.insert(elem, style);

        let x = (i as f32 % 10.0) * 80.0;
        let y = (i as f32 / 10.0).floor() * 50.0;
        children.push(LayoutBox {
            node_id: Some(elem),
            x,
            y,
            width: 80.0,
            height: 50.0,
            content_x: x,
            content_y: y,
            content_width: 80.0,
            content_height: 50.0,
            border_top: if with_background { 1.0 } else { 0.0 },
            border_right: if with_background { 1.0 } else { 0.0 },
            border_bottom: if with_background { 1.0 } else { 0.0 },
            border_left: if with_background { 1.0 } else { 0.0 },
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        });
    }

    let root_box = LayoutBox {
        node_id: Some(body),
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children,
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
    };

    (root_box, styles)
}

// ── 基准测试 ──────────────────────────────────────────────────────

/// bench_paint_simple_page: 绘制 100 个元素的布局。
fn bench_paint_simple_page(c: &mut Criterion) {
    let (layout, styles) = make_flat_layout(100, true);
    c.bench_function("paint_simple_page_100_elements", |b| {
        b.iter(|| {
            let mut painter = Painter::new();
            painter.paint(black_box(&layout), black_box(&styles));
            black_box(painter.into_primitives());
        })
    });
}

/// bench_paint_complex_page: 绘制 500 个带背景和边框的元素。
fn bench_paint_complex_page(c: &mut Criterion) {
    let (layout, styles) = make_flat_layout(500, true);
    c.bench_function("paint_complex_page_500_elements", |b| {
        b.iter(|| {
            let mut painter = Painter::new();
            painter.paint(black_box(&layout), black_box(&styles));
            black_box(painter.into_primitives());
        })
    });
}

/// bench_dirty_tracking: 标记 100 个节点为脏并合并。
fn bench_dirty_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("dirty_tracking");
    group.bench_function("mark_100_nodes_and_merge", |b| {
        b.iter(|| {
            let mut tracker = DirtyTracker::new();
            for i in 0..100 {
                let x = (i as f32 % 10.0) * 80.0;
                let y = (i as f32 / 10.0).floor() * 50.0;
                tracker.mark_dirty(Rect::new(x, y, 80.0, 50.0));
            }
            tracker.merge_overlapping();
            black_box(tracker.dirty_area());
        })
    });
    group.finish();
}

/// bench_compositing_layer_analysis: 分析 200 个元素的合成层。
fn bench_compositing_layer_analysis(c: &mut Criterion) {
    let (layout, styles) = make_flat_layout(200, false);
    c.bench_function("compositing_layer_analysis_200", |b| {
        b.iter(|| {
            black_box(promote_compositing_layers(black_box(&layout), black_box(&styles)));
        })
    });
}

/// bench_end_to_end: 完整的 HTML→Style→Layout→Paint 管线。
fn bench_end_to_end(c: &mut Criterion) {
    let html = "<html><body>\
        <div style=\"background-color: red; width: 200px; height: 100px;\">Block 1</div>\
        <div style=\"background-color: blue; width: 200px; height: 100px;\">Block 2</div>\
        <div style=\"background-color: green; width: 200px; height: 100px;\">Block 3</div>\
        <div><p><span>Nested content</span></p></div>\
        </body></html>";
    let css = "div { display: block; margin: 10px; padding: 5px; }
               span { color: black; }
               p { margin: 5px; }";

    c.bench_function("end_to_end_pipeline", |b| {
        b.iter(|| {
            let mut pipeline = RenderPipeline::new(800.0, 600.0);
            black_box(pipeline.render_html(black_box(html), black_box(css)));
        })
    });
}

criterion_group!(
    benches,
    bench_paint_simple_page,
    bench_paint_complex_page,
    bench_dirty_tracking,
    bench_compositing_layer_analysis,
    bench_end_to_end,
);
criterion_main!(benches);
