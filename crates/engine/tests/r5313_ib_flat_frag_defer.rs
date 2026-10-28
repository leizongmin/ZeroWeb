//! R5313（CSS2.1 App E 绘制序）回归钉——FLAT_CHILD_WALK 无有效尺寸 shrink-to-fit
//! inline-block 降级路径（collect_items「降级为零宽度 TextRun」）：ib 自身文本被
//! 扁平进父容器 stored IFC，容器 paint_text 先绘字形，子盒遍历后绘 ib 背景盖字，
//! ib 自身 paint_text 再被 painted_inline_nodes dedup 跳过（outline-028 ref 页
//! 白底压字形实证）。修复 = 容器 stored 趟让位（`ZW_IB_FLAT_FRAG_DEFER`），交
//! ib 自身盒路径按 App E 序（bg → 自绘文本）完成。
//!
//! 判别信号（R5086 同源 draw_order 口径）：ib 背景 Fill 位点 **先于** ib 文本
//! Glyph 位点。页内 span 行文本染蓝、ib 文本黑、ib 背景品红——三色互斥锁定
//! 归因；驱动案 = outline-028-ref 第 4 行片段（grid + 前置 span 行 + ib 行），
//! 最小 probe 形不带 grid 不触发降级。mutant（kill-switch=0 / 源级短路 helper）
//! 旧序 bg 后置，红在案。

use zero_engine::RenderPipeline;
use zero_render_foundation::primitive::DrawOp;

/// 渲染驱动案，返回 (品红 ib 背景 Fill 首位点, 黑 ib 文本 Glyph 首位点, 蓝 span Glyph 数, 绿 probe Glyph 数)。
fn render_ib_paint_order() -> (Option<usize>, Option<usize>, usize, usize) {
    let html = r#"<body style="margin:0">
<style>
.outline-container {
  width: max-content;
  margin: 30px 0px;
  outline: auto;
  font: 10px/1 Ahem;
  height: 10px;
  color: blue;
}
.inline { display: inline-block; color: black; }
</style>
<p style="color: rgb(0, 128, 0)">probe</p>
<div style="display: grid; grid-template-columns: repeat(4, 100px);">
  <div>
    <div class="outline-container">
      <span>XX</span>
    </div>
    <div class="outline-container" style="width: 20px;">
      <div class="inline" style="padding-top: 5px; background: rgb(255, 0, 255)">X</div>
    </div>
  </div>
</div>
</body>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let prims = result.primitives();
    let mut magenta_fill = None;
    let mut black_glyph = None;
    let mut blue_glyphs = 0usize;
    let mut green_glyphs = 0usize;
    for (pos, op) in prims.draw_order.iter().enumerate() {
        match *op {
            DrawOp::Fill(i) => {
                let c = &prims.fills[i].color;
                if (c.r, c.g, c.b, c.a) == (255, 0, 255, 255) {
                    magenta_fill = magenta_fill.or(Some(pos));
                }
            }
            DrawOp::Glyph(i) => {
                let c = &prims.glyphs[i].color;
                match (c.r, c.g, c.b) {
                    (0, 0, 0) => black_glyph = black_glyph.or(Some(pos)),
                    (0, 0, 255) => blue_glyphs += 1,
                    (0, 128, 0) => green_glyphs += 1,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (magenta_fill, black_glyph, blue_glyphs, green_glyphs)
}

#[test]
fn ib_background_paints_before_own_text() {
    // 修复态（默认开）：ib 自身盒路径 bg → 文本，品红 Fill 位点在黑 Glyph 之前。
    let (bg, text, blue, green) = render_ib_paint_order();
    assert_eq!(blue, 2, "span 行两枚蓝 Glyph 须存在（归因锚）");
    assert_eq!(green, 5, "probe 段五枚绿 Glyph 须存在（黑 Glyph 归因独占前提）");
    let (bg, text) = match (bg, text) {
        (Some(bg), Some(text)) => (bg, text),
        _ => panic!("ib 背景 Fill 或文本 Glyph 缺失：bg={bg:?} text={text:?}（文本消失 = 让位后无人绘）"),
    };
    assert!(
        bg < text,
        "App E：ib 背景（位点 {bg}）须先于自身文本（位点 {text}）；bg 后置 = 降级扁平旧序回归"
    );
}
