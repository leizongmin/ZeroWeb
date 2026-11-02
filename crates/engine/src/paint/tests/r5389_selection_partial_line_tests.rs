//! R5389（css-pseudo-4 §highlight-painting）：部分行选区分段绘制——行与 owner
//! 选区相交但未整行覆盖时，段内字形与 originating decoration 以 ::selection
//! color 重绘（assert「decorations should only recolor to ::selection color in
//! the selected part」），段外维持 originating 色。整行/不相交行走既有
//! R5257/R5269/R5272 臂字节不变；账平门拦空白塌缩形态（渲染序累计口径与
//! textContent 端点错位，保守不染）；UA 默认回落不拆段（作者 ::selection
//! 声明面才入表）。
//! driving: selection-originating-decoration-color（1.30%→0.00%）、
//! selection-originating-underline-order（1.68%→0.35% 近似通过）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

fn blue() -> Color {
    Color::rgba(0, 0, 255, 255)
}
fn red() -> Color {
    Color::rgba(255, 0, 0, 255)
}
fn black() -> Color {
    Color::rgba(0, 0, 0, 255)
}

/// 段内字形 ::selection 色、段外字形维持容器色（x 序：先 blue 后 black）。
#[test]
fn partial_selection_glyphs_recolor_to_selection_color() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">XX</div></body></html>"#,
        "div::selection { background-color: transparent; color: blue; }",
    );
    let prims = result.primitives();
    let mut blue_x = None;
    let mut black_x = None;
    for g in &prims.glyphs {
        if g.color == blue() {
            blue_x = Some(g.x);
        } else if g.color == black() {
            black_x = Some(g.x);
        }
    }
    assert!(blue_x.is_some(), "选区内字形应为 ::selection blue，实际无 blue glyph");
    assert!(black_x.is_some(), "选区外字形应维持容器黑，实际无 black glyph");
    assert!(
        blue_x.unwrap() < black_x.unwrap(),
        "blue 段应在首个字形域（x {:?}），black 在次字形域（x {:?}）",
        blue_x,
        black_x
    );
}

/// originating decoration 选中段以 ::selection 色重涂、段外维持声明色：
/// blue 段 fill（选中字形域）+ red 整条 fill，段宽小于整条。
#[test]
fn partial_selection_decoration_recolors_selected_span() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0;text-decoration: 4px solid red underline">XX</div></body></html>"#,
        "div::selection { background-color: transparent; color: blue; }",
    );
    let prims = result.primitives();
    let blue_underline = prims
        .fills
        .iter()
        .find(|f| f.color == blue() && f.rect.size.height <= 8.0);
    let red_underline = prims
        .fills
        .iter()
        .find(|f| f.color == red() && f.rect.size.height <= 8.0);
    assert!(
        blue_underline.is_some(),
        "选中段 decoration 应以 ::selection blue 重涂，实际无 blue 细 fill"
    );
    assert!(
        red_underline.is_some(),
        "段外 decoration 应维持声明 red，实际无 red 细 fill"
    );
    assert!(
        blue_underline.unwrap().rect.size.width < red_underline.unwrap().rect.size.width,
        "blue 段宽（{:?}）应小于整条 red 线宽（{:?}）",
        blue_underline.unwrap().rect.size.width,
        red_underline.unwrap().rect.size.width
    );
}

/// 账平门：空白塌缩形态保守不染——源码双空格 textContent utf16=7（"a  bcd"）
/// vs 渲染序折叠 6（"a bcd"），部分选区 [3,4) 的 utf16 端点与渲染序 glyph 序
/// 错位 → 段臂 owner 空白敏感门拦下（零 blue glyph；整行臂不覆盖非整行）。
#[test]
fn collapsed_whitespace_partial_selection_stays_uncolored() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 3, 4)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">a  bcd</div></body></html>"#,
        "div::selection { background-color: transparent; color: blue; }",
    );
    let prims = result.primitives();
    assert!(
        !prims.glyphs.iter().any(|g| g.color == blue()),
        "塌缩空白形态账面错位应保守不染，实际出现 blue glyph"
    );
}

/// 高亮面完整度门：::selection 自带非透明 bg（fill 分段未做）→ 保守不拆段
///（只染字形会放大合成差，highlight-painting-004 0.24%→0.96% 实证；零 blue
/// glyph 且 bg 分段缺席）。
#[test]
fn selection_with_author_bg_partial_stays_uncolored() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">XX</div></body></html>"#,
        "div::selection { background-color: rgb(0, 128, 0); color: blue; }",
    );
    let prims = result.primitives();
    assert!(
        !prims.glyphs.iter().any(|g| g.color == blue()),
        "::selection 自带 bg 时段臂应保守不拆段（整行臂也不覆盖部分行），实际出现 blue glyph"
    );
}

/// UA 默认回落不拆段：作者零 ::selection 规则 + 部分行选区 → 无 Highlight
/// fill、无 HighlightText 字形（R5272 臂部分行 all_selected=false 不铺）。
#[test]
fn ua_default_partial_selection_no_highlight_fill() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">XX</div></body></html>"#,
        "",
    );
    let prims = result.primitives();
    let highlight = Color::rgba(51, 153, 255, 255);
    assert!(
        !prims.fills.iter().any(|f| f.color == highlight),
        "部分行选区 UA 默认回落不应铺 Highlight fill"
    );
    assert!(
        !prims.glyphs.iter().any(|g| g.color == highlight),
        "部分行选区 UA 默认回落不应重染字形"
    );
}

/// 整行选区零触碰：全 contents 选区字形全 ::selection 色（既有
/// $frag_sel/R5272 臂，R5389 段臂不介入）。
#[test]
fn whole_line_selection_still_recolors_all_glyphs() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 2)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">XX</div></body></html>"#,
        "div::selection { background-color: transparent; color: blue; }",
    );
    let prims = result.primitives();
    assert!(
        prims.glyphs.iter().all(|g| g.color == blue()),
        "整行选区字形应全部 ::selection blue（既有臂），实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}
