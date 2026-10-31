//! R5358（CSS2 §visufx #visibility）：visibility 的**逐元素**语义——hidden 容器的
//! 后代可用 `visibility: visible` 重置继承、仍须绘制。旧实现在两条走查的内容相位
//!（paint_inline_content_phase / paint_node_in_rect）对 hidden 容器整体早退/整体跳过
//! paint_text，扁平 inline 后代（无独立盒）的片段一并灭绘（driving:
//! CSS2/visufx/visibility-005——div[hidden] 内 span[visible] 绿 X 全白）。修复 =
//! hidden 容器放行 paint_text，片段经 R5098 owner 过滤器逐片段裁决。
//!
//! 独立测试进程锚默认开态（R3858：进程内 set_var 与并行单测互踩）。

use zero_engine::RenderPipeline;

/// hidden div 内 visibility:visible 的 span 绘制字形；同容器 hidden 语义 span
///（继承 hidden）不绘——同一次 paint_text 内逐 owner 裁决。
#[test]
fn visible_descendant_overrides_hidden_ancestor() {
    let html = r#"<body style="margin:0"><div style="visibility:hidden"><span id="v" style="visibility:visible; color: rgb(0,128,0); font-family: Ahem">X</span><span style="color: rgb(255,0,0); font-family: Ahem">Y</span></div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let prims = result.primitives();
    let green: Vec<_> = prims
        .glyphs
        .iter()
        .filter(|g| g.color == zero_render_foundation::color::Color::rgba(0, 128, 0, 255))
        .collect();
    let red: Vec<_> = prims
        .glyphs
        .iter()
        .filter(|g| g.color == zero_render_foundation::color::Color::rgba(255, 0, 0, 255))
        .collect();
    assert_eq!(
        green.len(),
        1,
        "visibility:visible 后代应重置继承绘制 1 个字形，实际 {green:?}"
    );
    assert!(red.is_empty(), "继承 hidden 的 span 不应绘制字形，实际 {red:?}");
}

/// 对照（旧行为保持）：hidden 容器**自有文本**（owner = 容器自身）不绘制。
#[test]
fn hidden_container_own_text_stays_hidden() {
    let html = r#"<body style="margin:0"><div style="visibility:hidden; color: rgb(255,0,0); font-family: Ahem">XY</div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let red: Vec<_> = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.color == zero_render_foundation::color::Color::rgba(255, 0, 0, 255))
        .collect();
    assert!(red.is_empty(), "hidden 容器自有文本不应绘制字形，实际 {red:?}");
}

/// 对照（R5098 行为保持）：visible 容器内 hidden span 不绘制（owner 过滤器原始
/// 驱动形）。
#[test]
fn hidden_span_inside_visible_container_stays_hidden() {
    let html = r#"<body style="margin:0"><div style="color: rgb(255,0,0); font-family: Ahem"><span style="visibility:hidden">XY</span></div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let red: Vec<_> = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.color == zero_render_foundation::color::Color::rgba(255, 0, 0, 255))
        .collect();
    assert!(red.is_empty(), "visible 容器内 hidden span 不应绘制字形，实际 {red:?}");
}
