//! R5263（collapse 带端归属——CSS2 §17.6.2 边框带以格线为中心）：横带端部随
//! 相邻竖格线胜者伸缩：自胜端外延 win/2、败者端（阶段 3/R1626 写入胜者色
//! override）收进同量。旧码横带恒跨盒宽 [box_x, box_x+w]——001e 蓝 cell 顶带
//! 左端 [表盒, 盒缘] 半厚缝隙露表红框、邻 cell 顶带在共享格线处越权叠色。

use crate::pipeline::RenderPipeline;

/// 横带端部按格线胜者伸缩（chromium 001e 横带 [蓝60|…] 型，2-cell 版）。
#[test]
fn collapse_band_ends_stretch_to_gridline_winners() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body><table style="border-collapse: collapse; border: 20px solid red; margin: 0"><tr><td style="border: 20px solid blue; padding: 10px"></td><td style="border: 20px solid yellow; padding: 10px"></td></tr></table></body>"#,
        "",
    );
    let prims = result.primitives();
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    // 表盒 [8,108]（UA body margin 8 + 2 cell×40 + 外缘带 2×10），顶带行 y [8,28]：
    // 蓝 cell 盒 [18,58]，顶带自胜左右格线 → [8,68]（左外延至表盒缘、右跨格线
    // +win/2）；黄 cell 盒 [58,98]，顶带左端败于蓝（R1626 蓝色 override）→ 收进
    // 半厚起于 68，右端自胜表右缘外延 → [68,108]。
    let band_row = |x: f32, w: f32, rgb: (u8, u8, u8)| {
        prims.fills.iter().any(|f| {
            near(f.rect.origin.y, 8.0)
                && near(f.rect.size.height, 20.0)
                && near(f.rect.origin.x, x)
                && near(f.rect.size.width, w)
                && (f.color.r, f.color.g, f.color.b) == rgb
        })
    };
    assert!(
        band_row(8.0, 60.0, (0, 0, 255)),
        "蓝 cell 顶带应外延至表盒左缘 8、右跨共享格线至 68（60×20 蓝）——\
         旧码自盒缘 18 起绘，[8,18] 露表红框、右端止于 58"
    );
    assert!(
        band_row(68.0, 40.0, (255, 255, 0)),
        "黄 cell 顶带左端应收进半厚让位蓝带（起 68），右端外延至表右缘 108（40×20 黄）——\
         旧码自 58 起绘与蓝带在 [58,68] 叠色"
    );
}
