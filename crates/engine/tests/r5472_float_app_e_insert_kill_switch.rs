//! R5472 kill-switch 回退锚（独立测试进程——`ZW_FLOAT_APP_E_INSERT=0` 回退 floats
//! flush 原位旧序；进程内 set_var 会与并行单测互踩，R3858 教训）。

use zero_engine::RenderPipeline;

/// 旧序：float flush 原位（下降末）＝float 盖后继块 bg 与墨。OFF 时断言旧序
/// 成立（splice 不生效），与 src/paint/tests/r5472 断言互为反证。
#[test]
fn kill_switch_restores_flush_in_place_order() {
    unsafe { std::env::set_var("ZW_FLOAT_APP_E_INSERT", "0") };
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#host { background: rgb(255,0,0); }
#flt { float: left; width: 50px; height: 30px; background: rgb(0,128,0); }
#later { background: rgb(0,0,255); }
</style>
<div id="host">
  <div id="flt"></div>
  <div id="later">xxxx</div>
</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let prims = result.primitives();
    let pos = |rgb: [u8; 3]| {
        prims
            .draw_order
            .iter()
            .position(|op| match *op {
                zero_render_foundation::primitive::DrawOp::Fill(i) => prims.fills.get(i).is_some_and(|f| {
                    f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255
                }),
                zero_render_foundation::primitive::DrawOp::RoundedRect(i) => {
                    prims.rounded_rects.get(i).is_some_and(|f| {
                        f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255
                    })
                }
                _ => false,
            })
            .expect("color op")
    };
    let green = pos([0, 128, 0]);
    let blue = pos([0, 0, 255]);
    let glyph = prims
        .draw_order
        .iter()
        .position(|op| matches!(op, zero_render_foundation::primitive::DrawOp::Glyph(_)))
        .expect("glyph op");
    // 旧行为：float 最后施绘 → 后继块 bg 与墨均在 float 之下。
    assert!(
        blue < green && glyph < green,
        "OFF 应回退旧序：float({green}) 最后施绘盖 bg({blue}) 与墨({glyph})"
    );
    unsafe { std::env::remove_var("ZW_FLOAT_APP_E_INSERT") };
}
