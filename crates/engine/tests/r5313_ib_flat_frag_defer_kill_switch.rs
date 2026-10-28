//! R5313 kill-switch 回退锚（独立测试进程——`ZW_IB_FLAT_FRAG_DEFER=0` 恢复降级
//! 扁平旧序；开关经 LazyLock 构造期单次读取，set_var 必须先于本进程首次渲染，
//! 与并行单测互踩由独立测试二进制隔离，R3858/R5267 教训）。旧序 = 容器
//! paint_text 先绘 ib 文本 Glyph、子盒遍历后绘 ib 背景 Fill 盖字（驱动案与判别
//! 口径见 [`r5313_ib_flat_frag_defer`]）。

use zero_engine::RenderPipeline;
use zero_render_foundation::primitive::DrawOp;

#[test]
fn kill_switch_restores_legacy_flat_paint_order() {
    unsafe { std::env::set_var("ZW_IB_FLAT_FRAG_DEFER", "0") };
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
                if (c.r, c.g, c.b) == (0, 0, 0) {
                    black_glyph = black_glyph.or(Some(pos));
                }
            }
            _ => {}
        }
    }
    match (magenta_fill, black_glyph) {
        (Some(bg), Some(text)) => assert!(
            bg > text,
            "kill-switch 关闭应恢复旧序（文本位点 {text} 先于 bg 位点 {bg}）——门未生效"
        ),
        _ => panic!("kill-switch 关闭后 ib bg/文本图元缺失：bg={magenta_fill:?} text={black_glyph:?}"),
    }
}
