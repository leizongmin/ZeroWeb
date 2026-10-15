//! T35 回归：视口 resize 后的增量重绘（`paint_in_rect` 路径）须保留 `<input>` 的
//! placeholder/value 文本——此前 `paint_node_in_rect` 缺 `paint_input_value` 绘制族，
//! 窗口打开时 renderer 初始视口（800x600）首帧后收到真实窗口尺寸，resize 重发布
//! 走该路径，LoadHtml welcome 页搜索框 placeholder 被静默丢弃。

use crate::pipeline::RenderPipeline;

fn glyph_text(primitives: &zero_render_foundation::primitive::RenderPrimitives) -> String {
    primitives.glyphs.iter().filter_map(|g| g.code_point()).collect()
}

/// 复现窗口打开序列：初始视口首帧（placeholder 在）→ set_viewport 到真实窗口
/// 尺寸 → `repaint_cached_viewport` 增量重发布（`incremental_paint` →
/// `paint_in_rect`）。重发布帧必须保留 placeholder 字形。
#[test]
fn resize_repaint_keeps_input_placeholder() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><form action="https://www.google.com/search" method="get"><input type="text" name="q" value="" placeholder="Search the web" aria-label="Search the web"></form></body></html>"#;
    let first = pipeline.render_html(html, "");
    assert!(
        glyph_text(first.primitives()).contains("Search the web"),
        "首帧应绘制 placeholder"
    );

    pipeline.set_viewport(1602.0, 1292.0);
    let repainted = pipeline
        .repaint_cached_viewport("")
        .expect("cached viewport repaint after resize");
    assert!(
        glyph_text(repainted.primitives()).contains("Search the web"),
        "resize 重绘帧应保留 placeholder（T35：paint_in_rect 路径此前缺 input value 绘制）"
    );
}

/// 同路径的预填 value：resize 重绘不得丢失已填值文本（同一绘制族）。
#[test]
fn resize_repaint_keeps_input_value() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><input type="text" value="hello"></body></html>"#;
    let first = pipeline.render_html(html, "");
    assert!(
        glyph_text(first.primitives()).contains("hello"),
        "首帧应绘制 input value"
    );

    pipeline.set_viewport(1024.0, 768.0);
    let repainted = pipeline
        .repaint_cached_viewport("")
        .expect("cached viewport repaint after resize");
    assert!(
        glyph_text(repainted.primitives()).contains("hello"),
        "resize 重绘帧应保留 input value"
    );
}
