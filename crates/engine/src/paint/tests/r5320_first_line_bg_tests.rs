//! R5320（::first-line 首格式行 background-color，切片 2 bg-color 位——CSS2
//! §5.12.1）：块容器 `::first-line` 伪样式的 background-color 铺到首格式行
//! （行组 line_top == 组内最小）的片段横向范围并集带；次行不铺；bg-only 页
//!（伪样式 color 与元素相同）也存储伪样式（style-system 存储门扩位——
//! 双向可判别：仅回 paint 臂 → 本文件全红；仅回存储门 → bg-only 案红）。
//! driving: css1/c25-pseudo-elmnt-000（2.80%）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 首行铺伪样式 background-color 带，次行不铺。
#[test]
fn first_line_bg_band_first_line_only() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="color: red; font: 50px Ahem; width: 300px">AAAAA BBBBB</div></body>"#,
        "div::first-line { background: green }",
    );
    let prims = result.primitives();
    let bands: Vec<_> = prims
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 128, 0, 255))
        .collect();
    assert_eq!(bands.len(), 1, "恰一条首行绿带，实际 {} 条", bands.len());
    assert!(
        bands[0].rect.size.width > 0.0 && bands[0].rect.size.height > 0.0,
        "绿带应有正面积，实际 {:?}",
        bands[0].rect
    );
    assert!(
        prims.glyphs.iter().all(|g| g.color == Color::rgba(255, 0, 0, 255)),
        "bg-only 伪样式不改字形 color，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// bg-only 页（伪样式 color == 元素 color）也存储并铺带——style-system 存储
/// 门扩位腿：门未扩时 first_line_pseudo = None → line_ranges 不构建 → 零带。
#[test]
fn first_line_bg_only_page_still_stores_pseudo_style() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="font: 50px Ahem">Hello</div></body>"#,
        "div::first-line { background: green }",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "bg-only 伪样式应铺首行绿带，fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 透明 background 不铺带（伪样式因 color 差异仍存储，字形 color 照常应用）。
#[test]
fn first_line_transparent_bg_pushes_no_band() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="color: red; font: 50px Ahem">Hello</div></body>"#,
        "div::first-line { color: green; background: transparent }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color.a > 0),
        "透明 background 不应产生任何带，fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.glyphs.is_empty() && prims.glyphs.iter().all(|g| g.color == Color::rgba(0, 128, 0, 255)),
        "color 位照常应用（R5258 切片 1 不回退）"
    );
}
