//! R5271（css-pseudo-4 §highlight-styling + §highlight-cascading）：选区高亮
//! **UA 默认回落面**——owner 无作者 ::selection 记录时行高亮/字形色取系统色
//! Highlight 背景 + HighlightText 前景（css-color-4 §css-system-colors）。
//! driving: system-color-hightlights-vs-getSelection-001/002（selectNode(span)
//! 形 5.25pp 合计带，ref 页同关键字静态样式 → 双侧同 RGB 必收敛）。
//! 作者声明面（含显式 transparent，selection_pseudo 已存）不回落；
//! kill-switch `ZW_SELECTION_HIGHLIGHT_DEFAULT=0` 回退「无规则无高亮」旧口径
//!（独立进程案见 crates/engine/tests/）。shim selectNode 导出臂（part01.js
//! R5271 段）为 harness 端到端面，判据走定向 reftest。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 块内 inline span 自身被选中（system-color-001 形：selectNode(span)，父块
/// IFC 片段路径）+ 页无 ::selection 规则 → 行高亮铺 Highlight、字形 HighlightText。
#[test]
fn span_owner_without_pseudo_gets_ua_default_line_fill() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#sel".to_string(), 0, 19)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div><span id="sel" style="font: 40px Ahem">With getSelection()</span></div></body>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "应铺 UA 默认 Highlight 背景 fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "字形应为 UA 默认 HighlightText 白，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 作者显式声明 transparent 背景（selection_pseudo 已存）→ **不回落** UA 默认
///（R5065 判例面：作者声明含显式 transparent 即锁定）。
#[test]
fn author_transparent_pseudo_not_overridden_by_ua_default() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#sel".to_string(), 0, 19)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div><span id="sel" style="font: 40px Ahem">With getSelection()</span></div></body>"#,
        "span::selection { background-color: transparent; color: aqua }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "作者显式 transparent 时不应回落 UA 默认 Highlight，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.fills.iter().any(|f| f.color.a > 0),
        "transparent 选区不应铺任何行高亮 fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(0, 255, 255, 255)),
        "字形应按作者伪样式 aqua，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 部分选区行（行未完全落入）→ 不高亮（部分行高亮维持挂账，回落臂同门）。
#[test]
fn partial_selection_line_gets_no_ua_default_fill() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#sel".to_string(), 0, 3)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div><span id="sel" style="font: 40px Ahem">With getSelection()</span></div></body>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "部分选区行不应铺 UA 默认 Highlight fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
