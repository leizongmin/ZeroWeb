//! R3895：嵌套 inline 包装层不得双绘文本（quotes-028/029/031 回归守卫）。
//!
//! 根因（三层叠加）：layout IFC 把嵌套 inline 的子树文本扁平化到**外层**元素 id 的
//! 单个 TextRun；paint 渲染该片段后 `painted_inline_nodes` 只标记了外层 id；内层
//! span 的 paint_text dedup 查不到自身 id → 放行 → 重跑无度量覆盖的 Path B IFC
//!（text_node_font_sizes 只存了外层键）→ 以默认 16px 在错误位置双绘文本。
//! 修复 = 片段渲染后沿 inline 包装链补标（`mark_inline_wrapper_chain_painted`）。

use crate::pipeline::RenderPipeline;

/// `<p>One <span><span>two</span></span>` 须只绘一行 "One two"（无 fs16 幽灵带）。
#[test]
fn r3895_nested_inline_no_ghost_row() {
    let html = r#"<html><head><style>body { font: 32px serif; }</style></head><body>
<p>One <span><span>two</span></span></body></html>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");

    let glyphs = &result.primitives().glyphs;
    assert!(!glyphs.is_empty(), "应产生 glyph");
    // 全部 glyph 须在同一行基线（32px 行高 → 主带 y≈64.3）。
    // 修复前：内层 span 自绘的幽灵带在 y≈48.2 且 fs=16。
    let first_y = glyphs[0].y;
    for g in glyphs.iter() {
        assert!(
            (g.y - first_y).abs() < 1.0,
            "嵌套 inline 文本不得双绘到另一行：ch={:?} y={} (首行 y={first_y})",
            char::from_u32(g.glyph_id).unwrap_or('?'),
            g.y
        );
        assert_eq!(
            g.font_size,
            32.0,
            "嵌套 inline 文本不得以默认 16px 重绘：ch={:?}",
            char::from_u32(g.glyph_id).unwrap_or('?')
        );
    }
    // 文本恰好一份："Onetwo" 6 glyph（双绘则 "two" ×2 = 9；折叠后空格不成 glyph）。
    assert_eq!(glyphs.len(), 6, "文本须恰好渲染一次（无双绘），得 {}", glyphs.len());
}

/// `<q>` 嵌套（quotes: Pairs 注入引号）同样不得双绘引号——quotes-028/031 主残差。
#[test]
fn r3895_nested_q_no_ghost_quote_glyphs() {
    let html = r#"<html lang="en"><head><style>
body { font: 32px serif; quotes: "“" "”" "‘" "’" "«" "»" "‹" "›"; }
</style></head><body>
<p>One <q>two <q>three</q></q></body></html>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");

    let glyphs = &result.primitives().glyphs;
    // 每种引号码点在整页恰好出现其配对次数（depth1 “”+ depth2 ‘’= 4 个引号 glyph）。
    let count = |cp: char| glyphs.iter().filter(|g| g.glyph_id == cp as u32).count();
    assert_eq!(count('\u{201C}'), 1, "“ 须恰好 1 个（双绘则 2+）");
    assert_eq!(count('\u{201D}'), 1, "” 须恰好 1 个");
    assert_eq!(count('\u{2018}'), 1, "‘ 须恰好 1 个");
    assert_eq!(count('\u{2019}'), 1, "’ 须恰好 1 个");
}
