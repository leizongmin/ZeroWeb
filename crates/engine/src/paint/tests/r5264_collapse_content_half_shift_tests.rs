//! R5264（collapse 格线模型内容原点——CSS2 §17.6.2）：cell 内容盒 = 盒跨（格线到
//! 格线）内缩**胜出边框宽的一半**/缘，非整框。共享带以格线为中心跨画双邻，整框
//! 内缩会留 ½框护城河且内容错位 +½框（001d 白字母 +10 咬带、ref 马赛克槽位算术
//! + p001/p002 chromium 位图取证）。半框修正过整框口翻红。

use crate::pipeline::RenderPipeline;

/// collapse cell 内容原点 = 格线 + win/2（chromium 001d 槽位型，2-cell 版）。
#[test]
fn collapse_cell_content_inset_is_half_border() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body><table style="border-collapse: collapse; border: 20px solid red; margin: 0"><tr><td style="border: 20px solid blue; padding: 0"><div style="width: 10px; height: 10px; background: green"></div></td><td style="border: 20px solid yellow; padding: 0"></td></tr></table></body>"#,
        "",
    );
    let prims = result.primitives();
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    // 表盒 [8,108]（UA body margin 8 + 2 cell×40 格线跨 + 外缘带 2×10，同 R5263
    // 判例）；左外带 [8,28]（win = max(表框 20, cell 框 20)）、格线 18 → 蓝 cell
    // 盒 [18,58]。内容原点 = 盒 + win/2 = 28（chromium 槽位 [28,38]）；旧码整框
    // 内缩落 38，与带 [8,28] 间露 10px 护城河。
    let green_at = |x: f32, y: f32| {
        prims.fills.iter().any(|f| {
            near(f.rect.origin.x, x)
                && near(f.rect.origin.y, y)
                && near(f.rect.size.width, 10.0)
                && near(f.rect.size.height, 10.0)
                && (f.color.r, f.color.g, f.color.b) == (0, 128, 0)
        })
    };
    assert!(
        green_at(28.0, 28.0),
        "绿块应落在 cell 内容原点 (28,28) = 盒 [18,58] + 半框 10——\
         旧码整框内缩落 (38,38)，[28,38] 护城河露白、内容错位 +10"
    );
}
