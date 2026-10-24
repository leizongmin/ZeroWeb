//! R5256（整contents 选区的后代 owner 行）：R5054 后代 owner FIXME 首片——
//! `getSelection().addRange()` / `selectNodeContents(盒)` 形下，片段行可归
//! inline 后代 owner（`<div 选区><span>text</span></div>` 的 span 文本），
//! 此前 owner==box_id 过滤整行跳过（driving: text-decoration-skip-spaces-008
//! ref 页形态 0.119%——选区字形色未及 span 载体）。判据 start==0 且
//! end ≥ owner textContent utf16 长 → 全部片段行选中；部分选区语义不变。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 整contents 选区 + 后代 owner（span 载体）→ 后代字形用伪样式 color
///（transparent 背景 → 零高亮 fill，纯色面判别）。
#[test]
fn descendant_owner_whole_contents_selection_colors_span() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#d".to_string(), 0, 5)]);
    let result = pipeline.render_html(
        r#"<body><div id="d" style="color: yellow; font: 50px Ahem;"><span>Hello</span></div></body>"#,
        "::selection { background-color: transparent; color: white }",
    );
    let prims = result.primitives();
    assert!(
        !prims.glyphs.is_empty() && prims.glyphs.iter().all(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "整contents 选区下后代 owner 字形应为伪样式色，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 部分选区（非整contents）→ 后代 owner 行不选中（本切片不扩展部分选区语义，
/// 既有 owner==box_id 行子集检查不变）。
#[test]
fn partial_selection_on_owner_leaves_descendant_uncolored() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#d".to_string(), 0, 2)]);
    let result = pipeline.render_html(
        r#"<body><div id="d" style="color: yellow; font: 50px Ahem;"><span>Hello</span></div></body>"#,
        "::selection { background-color: transparent; color: white }",
    );
    let prims = result.primitives();
    assert!(
        prims.glyphs.iter().all(|g| g.color == Color::rgba(255, 255, 0, 255)),
        "部分选区下后代 owner 字形应保持元素色，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}
