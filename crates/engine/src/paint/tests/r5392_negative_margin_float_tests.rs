//! R5392 负 margin float 的 earliest-fit 下探（CSS 2.1 §9.5.1）。
//!
//! 负总 margin（mt+mb<0）float 的 margin 盒高为负，fit 试探带倒置为空，
//! 此前永不避让更早 float：候选带高取 max(margin 盒高, border 盒高)。
//! chromium oracle（headless 探针）定谳 overhanging-float-paint-order 期望几何：
//! red y=0 h=60 / blue y=40 h=60（带顶 60 = red 底，mt −20 回拉）/
//! green y=0 h=80（带顶 100 = blue margin 盒底，mt −100 回拉）。

use crate::pipeline::{RenderPipeline, RenderResult};

/// 图元清单按色取 fill 矩形（本页仅三 float 有背景）。
fn fill_by_color(result: &RenderResult, rgb: [u8; 3]) -> Option<(f32, f32, f32, f32)> {
    let p = result.primitives();
    p.fills
        .iter()
        .find(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
        .map(|f| (f.rect.left(), f.rect.top(), f.rect.size.width, f.rect.size.height))
}

#[test]
fn r5392_negative_margin_float_stacks_after_earlier_float_band() {
    let html = r#"<!DOCTYPE html><style>
#red-first-float { width: 100px; height: 60px; background-color: red; float: left; }
#container { width: 100px; height: 80px; }
#blue-second-float-overhanging { width: 100px; height: 60px; background-color: blue; float: left; margin-top: -20px; }
#green-third-float { width: 100px; height: 80px; background-color: green; float: left; margin-top: -100px; }
</style><body style="margin:0"><div id="red-first-float"></div>
<div id="container">
  <div id="blue-second-float-overhanging"></div>
  <div id="green-third-float"></div>
</div></body>"#;
    let result = RenderPipeline::new(300.0, 300.0).render_html(html, "");
    // blue：放不下 red 旁（等宽）→ 下探 red margin 盒底 60，mt −20 → border top 40。
    assert_eq!(
        fill_by_color(&result, [0, 0, 255]),
        Some((0.0, 40.0, 100.0, 60.0)),
        "blue float margin-top:-20 应在 red 底 60 处堆叠后回拉至 y=40"
    );
    // green：候选带顶 60 时 border 盒高 80 与 blue margin 带 [60,100) 相交
    // → 下探 100，mt −100 → border top 0（修复前带高 −20 恒不相交，落在 −40）。
    assert_eq!(
        fill_by_color(&result, [0, 128, 0]),
        Some((0.0, 0.0, 100.0, 80.0)),
        "green float margin-top:-100 应下探 blue margin 盒底 100 后回拉至 y=0"
    );
    assert_eq!(
        fill_by_color(&result, [255, 0, 0]),
        Some((0.0, 0.0, 100.0, 60.0)),
        "red float 无负 margin，落位不受影响"
    );
}

#[test]
fn r5392_negative_margin_float_alone_no_earlier_band() {
    // 无更早 float 时负 margin float 从流位直接回拉（带集为空，无下探）。
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#container { width: 100px; height: 80px; }
#float-neg { width: 100px; height: 60px; background-color: blue; float: left; margin-top: -20px; }
</style><div id="container"><div id="float-neg"></div></div>"#;
    let result = RenderPipeline::new(300.0, 300.0).render_html(html, "");
    assert_eq!(
        fill_by_color(&result, [0, 0, 255]),
        Some((0.0, -20.0, 100.0, 60.0)),
        "无更早 float 带约束时负 margin float 应停在流位回拉 y=-20"
    );
}
