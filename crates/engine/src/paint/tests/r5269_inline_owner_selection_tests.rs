//! R5269（css-pseudo-4 §highlight-cascading 起源元素独立级联——owner 自持选区形）：
//! inline 元素自身被选中（`selectNodeContents(inline 元素)`，driving:
//! active-selection-012 `div[inline]::selection` 2.57%）时其文本经父块
//! paint_text 片段路径绘制，盒级 sel_owned（盒/祖先链）查不到 owner 记录 →
//! 整行零处理。新臂按片段 owner 自身选区记录判定行选中、字形色按 owner 自持
//! ::selection 级联；行完全落在 owner 选区内才着色（部分行维持挂账）。
//! 行高亮 fill 由 R5270 接入（与 inline 片段 bg 同几何同站，见 r5270 测试文件）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// active-selection-012 形：display:inline div 自身被选中 → 字形用自持伪样式
/// color（aqua）。修复前字形保持 fuchsia。行高亮 fill 同站铺自持伪样式 bg
///（R5270，green 矩形几何断言见 r5270 测试文件）。
#[test]
fn inline_owner_own_selection_colors_glyphs() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 8)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 50px Ahem">Selected</div></body>"#,
        "div::selection { background-color: green; color: aqua }",
    );
    let prims = result.primitives();
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .count()
            >= 8
            && prims
                .glyphs
                .iter()
                .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
                .all(|g| g.color == Color::rgba(0, 255, 255, 255)),
        "inline owner 被选中时字形应为自持伪样式色 aqua，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "owner 自持选区行高亮 fill 应铺 green（R5270 同站 fill），实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 块形（无 display:inline）保持盒级臂行为：aqua 字形 + green 行高亮（零回归锚）。
#[test]
fn block_owner_selection_unchanged() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 8)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div id="test" style="color: fuchsia; font: 50px Ahem">Selected</div></body>"#,
        "div::selection { background-color: green; color: aqua }",
    );
    let prims = result.primitives();
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .count()
            >= 8
            && prims
                .glyphs
                .iter()
                .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
                .all(|g| g.color == Color::rgba(0, 255, 255, 255)),
        "块形被选中时字形应为伪样式色 aqua，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "块形被选中时应铺 green 行高亮，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// inline owner 无自持 ::selection 记录（页无任何 ::selection 规则）→ R5271 起
/// 回落 **UA 默认高亮面**（Highlight 背景 + HighlightText 前景，css-color-4
/// §css-system-colors 系统色；driving: system-color-hightlights-vs-getSelection
/// 族——ref 页同关键字静态样式，双侧同 RGB）。
#[test]
fn inline_owner_without_pseudo_gets_ua_default_highlight() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 8)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 50px Ahem">Selected</div></body>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "无伪样式时字形应回落 UA 默认 HighlightText 白，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "无伪样式时应铺 UA 默认 Highlight 背景，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 部分选区（行未完全落入 owner 选区）→ 不着色（部分行高亮维持挂账）。
#[test]
fn partial_selection_on_inline_owner_leaves_line_uncolored() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 3)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 50px Ahem">Selected</div></body>"#,
        "div::selection { background-color: green; color: aqua }",
    );
    let prims = result.primitives();
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(255, 0, 255, 255)),
        "部分选区下字形应保持元素色 fuchsia，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "部分选区下不应铺 green 行高亮，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
