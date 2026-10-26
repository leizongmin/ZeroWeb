//! R5272（css-pseudo-4 §highlight-styling/§highlight-painting）：**祖先链整
//! contents 选区**的 UA 默认高亮面——记录落 html/body（selection-image-001/002
//! 形：selectNode(documentElement)，shim dump 类型选择器回落 R5272 段）时片段
//! owner（p）无记录，盒级 anc_whole 臂（R5256 祖先链）此前门 `selection_pseudo`
//! 整关 → 页零 ::selection 规则时整行零处理。R5272 起盒级臂同 R5269 臂回落
//! UA 默认（Highlight 背景 + HighlightText 前景，css-color-4 §css-system-colors；
//! 同 kill-switch `ZW_SELECTION_HIGHLIGHT_DEFAULT`，独立进程案见
//! crates/engine/tests/r5271_selection_ua_default_kill_switch.rs——开关同源，
//! 机制同面不重复立案）。作者声明面 selection_pseudo 已存不回落。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// selection-image-002 形：选区记录落 html（`selectNode(documentElement)` ≡
/// `("html", 0, 全文长)`）、页无 ::selection 规则 → p 行经盒级 anc_whole 臂铺
/// UA 默认 Highlight fill、字形 HighlightText 白。
#[test]
fn ancestor_whole_contents_selection_gets_ua_default_highlight() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 18)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><p style="margin:0">Selected text here</p></body></html>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "祖先链整 contents 选区应铺 UA 默认 Highlight fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "字形应回落 UA 默认 HighlightText 白，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 祖先链选区 + 部分行（行未完全落入）形不适用 anc_whole（整 contents 判据
/// 锚定祖先全文）——本锚守「anc_whole 仅整 contents 形」边界：选区非全文时
/// 不铺 UA 默认 fill（部分行高亮维持挂账）。
#[test]
fn ancestor_partial_selection_gets_no_ua_default_fill() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("html".to_string(), 0, 3)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><p style="margin:0">Selected text here</p></body></html>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(51, 153, 255, 255)),
        "祖先部分选区不应铺 UA 默认 Highlight fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
