//! R5394 纯原子行内盒行的 strut 行盒（CSS 2.1 §10.8.1）。
//!
//! strut 是块容器每行必参与的假想零宽盒：line-height:140px 容器内单个
//! 100px va:bottom 原子盒行盒 = 140（盒底对齐扩展后行底 [40,140)）。
//! 旧 R5298 接管门仅文本行、原子行缩至原子盒高（盒贴行顶 [0,100)）。
//! chromium oracle 定谳 margin-collapse-clear-014 ref 期望几何
//!（lime line-height:140px + swatch img va:bottom → 行盒 140、img [40,140)）。

use crate::pipeline::{RenderPipeline, RenderResult};

/// 图元清单按色取 fill 矩形（本页仅两盒有背景）。
fn fill_by_color(result: &RenderResult, rgb: [u8; 3]) -> Option<(f32, f32, f32, f32)> {
    let p = result.primitives();
    p.fills
        .iter()
        .find(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
        .map(|f| (f.rect.left(), f.rect.top(), f.rect.size.width, f.rect.size.height))
}

#[test]
fn r5394_atomic_only_line_strut_extends_line_box() {
    // ref 同构：line-height:140px 容器 + 100px inline-block va:bottom 独占行。
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#lime { line-height: 140px; }
#lime > div { display: inline-block; width: 100px; height: 100px; background-color: blue; vertical-align: bottom; }
#yellow { height: 100px; background-color: yellow; }
</style><div id="lime"><div></div></div>
<div id="yellow"></div>"#;
    let result = RenderPipeline::new(300.0, 300.0).render_html(html, "");
    // 行盒 = strut 140，va:bottom 盒底对齐行底 → [40,140)（旧贴行顶 [0,100)）。
    assert_eq!(
        fill_by_color(&result, [0, 0, 255]),
        Some((0.0, 40.0, 100.0, 100.0)),
        "va:bottom 原子盒应底对齐 strut 撑起的行盒底（y=40=140-100）"
    );
    // 后续块随行盒 140 排布（旧行盒 100 时 yellow 顶 = 100）。
    assert_eq!(
        fill_by_color(&result, [255, 255, 0]),
        Some((0.0, 140.0, 300.0, 100.0)),
        "行盒应为 strut 140 而非原子盒高 100"
    );
}

#[test]
fn r5394_atomic_box_taller_than_strut_no_shrink_floor_inert() {
    // 原子盒高 200 > strut 140：strut 下限不生长既有行盒，盒仍贴行底 [0,200)。
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#container { line-height: 140px; }
#container > div { display: inline-block; width: 100px; height: 200px; background-color: blue; vertical-align: bottom; }
</style><div id="container"><div></div></div>"#;
    let result = RenderPipeline::new(300.0, 300.0).render_html(html, "");
    assert_eq!(
        fill_by_color(&result, [0, 0, 255]),
        Some((0.0, 0.0, 100.0, 200.0)),
        "原子盒高于 strut 时行盒 = 原子盒高，strut floor 恒不缩盒"
    );
}
