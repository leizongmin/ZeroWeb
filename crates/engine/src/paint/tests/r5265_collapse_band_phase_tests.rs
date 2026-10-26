//! R5265（collapse 带 z 序相位间插——CSS2.1 Appendix E / Blink
//! `PaintPhase::kCollapsedTableBorders`）：带 = 「后代块背景 < 带 < inline 内容」。
//! deco 相位两点式无法两全（R5264 实验：带后置则 002 绿 001 红、带前置反之），
//! 本轮 cell 子树拆 sweep A（后代 plain-block 装饰）/ 带 / 内容相位 / sweep B
//!（其余整体）三明治。锚走 draw_order 序断言（判据 = 常驻 WPT
//! collapsed-border-paint-phase-001/002）。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::primitive::DrawOp;

/// 收集 draw_order 中红（带）/绿（内容）fill 的序位。
fn band_vs_content_order(html: &str) -> (Vec<usize>, Vec<usize>) {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let prims = result.primitives();
    let mut red = Vec::new();
    let mut green = Vec::new();
    for (i, op) in prims.draw_order.iter().enumerate() {
        if let DrawOp::Fill(fi) = op {
            let c = &prims.fills[*fi].color;
            if (c.r, c.g, c.b) == (255, 0, 0) {
                red.push(i);
            } else if (c.r, c.g, c.b) == (0, 128, 0) {
                green.push(i);
            }
        }
    }
    (red, green)
}

/// paint-phase-002 型：后代 plain-block 背景画在带之下（旧行 = 带 deco 相位在
/// 后代之前，块背景盖带——corpus 002 2.08% 残差域）。
#[test]
fn collapse_band_above_descendant_block_background() {
    let (red, green) = band_vs_content_order(
        r#"<body style="margin:0"><table style="border-collapse: collapse; border-spacing: 0"><tr><td style="border: 20px solid red; padding: 0"><div style="width: 20px; height: 20px; background: rgb(0,128,0)"></div></td></tr></table></body>"#,
    );
    assert!(!red.is_empty() && !green.is_empty(), "collapse 带与后代块背景均须施画");
    let first_green = *green.iter().min().unwrap();
    assert!(
        red.iter().all(|&r| r > first_green),
        "collapse 带（draw_order {red:?}）须画在后代块背景（{first_green}）之上\
         （CSS2 App E：后代块背景 < 带）"
    );
}

/// paint-phase-001 型：后代 inline 内容（inline-block 背景）画在带之上（旧行 =
/// 带前置已满足；本轮 sweep B 不得回退——R5264 带后置实验曾翻红此域）。
#[test]
fn collapse_band_below_descendant_inline_content() {
    let (red, green) = band_vs_content_order(
        r#"<body style="margin:0"><table style="border-collapse: collapse; border-spacing: 0"><tr><td style="border-right: 50px solid red; padding: 0"><div style="width: 10px"><div style="display: inline-block; width: 20px; height: 20px; background: rgb(0,128,0)"></div></div></td></tr></table></body>"#,
    );
    assert!(
        !red.is_empty() && !green.is_empty(),
        "collapse 带与 inline 内容均须施画"
    );
    let last_red = *red.iter().max().unwrap();
    assert!(
        green.iter().all(|&g| g > last_red),
        "inline 内容（draw_order {green:?}）须画在 collapse 带（{last_red}）之上\
         （CSS2 App E：带 < inline 内容）"
    );
}
