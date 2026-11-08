//! R5472：floats flush 段逐容器 App E step4 < step5 splice（draw_order 重排钉子）。
//!
//! 规范语义：同一 SC 内 step3（in-flow 块级后代装饰）< step4（floats）< step5
//!（行内级墨）。R5420 flush 在下降末施绘＝float 盖全部墨（step5 违例，c5525-
//! fltmrgn test6 img mr−30 红格）；单点 splice 已证不可行（earliest-ink 实验把
//! float 插到后继块 bg 之前——pattern bg 盖 img）。本钉断言 draw_order 真实插
//! 入序（typed fills 位序不随重排变化，R5420 的 fills 位序钉不敏感本域）：
//! float 绘于宿主及**后继兄弟块**装饰之上、首个相关墨主之下。目标位＝「首个 ≥
//! 宿主 pre-ink 位的真墨主 pre-ink 位」（R5472b：纯结构宿主 div 须越过——
//! fltblck ✗1.42% 实证）。kill-switch `ZW_FLOAT_APP_E_INSERT=0` 回退 flush 原位
//! 旧序（反证由独立进程锚 tests/r5472_float_app_e_insert_kill_switch.rs 承担，
//! R3858 口径）。

use crate::pipeline::{RenderPipeline, RenderResult};
use zero_render_foundation::primitive::DrawOp;

/// draw_order 内首命中色图元的绘制位（Fill/RoundedRect 双桶查）。
fn fill_draw_pos(result: &RenderResult, rgb: [u8; 3]) -> Option<usize> {
    let prims = result.primitives();
    prims.draw_order.iter().position(|op| {
        let idx = match *op {
            DrawOp::Fill(i) => i,
            DrawOp::RoundedRect(i) => i,
            _ => return false,
        };
        prims
            .fills
            .get(idx)
            .is_some_and(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
            || prims
                .rounded_rects
                .get(idx)
                .is_some_and(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
    })
}

/// draw_order 首个 Glyph op 位（页面唯一墨源时的 step5 锚）。
fn first_glyph_pos(result: &RenderResult) -> Option<usize> {
    result
        .primitives()
        .draw_order
        .iter()
        .position(|op| matches!(op, DrawOp::Glyph(_)))
}

/// ★load-bearing（c5525-fltblck 形）：宿主 div（红 bg，无墨）内 float（绿）+
/// 后继兄弟块（蓝 bg + 文本墨）。App E：红 < 蓝（step3 树序）< 绿（step4 覆盖
/// 后继块 bg）< 墨（step5 盖 float）。旧序 flush 原位＝绿最后（墨被盖）；v1 宿主
/// pre-ink 位＝绿在蓝 bg 之前（蓝 bg 盖 float）——双缺陷均被 green/blue 断言判死。
#[test]
fn r5472_float_over_later_sibling_block_bg() {
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
    let red = fill_draw_pos(&result, [255, 0, 0]).expect("host red bg");
    let green = fill_draw_pos(&result, [0, 128, 0]).expect("float green bg");
    let blue = fill_draw_pos(&result, [0, 0, 255]).expect("later blue bg");
    let glyph = first_glyph_pos(&result).expect("later text ink");
    assert!(red < blue, "宿主 bg({red}) < 后继块 bg({blue})（step3 树序，sanity）");
    assert!(
        blue < green,
        "float({green}) 应绘于后继块 bg({blue}) 之上（App E step4 > step3；v1 宿主位 splice 缺陷 fltblck ✗1.42%）"
    );
    assert!(
        green < glyph,
        "墨({glyph}) 应盖 float({green})（step5 > step4；旧 flush 原位缺陷 fltmrgn test6 同面）"
    );
}

/// ★fltmrgn test6 形：float 宿主即红 bg 墨主（p[bg red] > img float + 自身文本）。
/// 目标位＝宿主自身 pre-ink（首个 ≥ 宿主位的墨主＝宿主自身）——float 绘于宿主
/// bg 之上、宿主墨之下（img 负 margin 外悬区不被宿主 bg 盖、文本仍浮于 img 上）。
#[test]
fn r5472_float_between_host_bg_and_host_ink() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#host { background: rgb(255,0,0); }
#flt { float: left; width: 50px; height: 30px; background: rgb(0,128,0); margin-bottom: -15px; }
</style>
<div id="host">
  <div id="flt"></div>
  xxxx
</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let red = fill_draw_pos(&result, [255, 0, 0]).expect("host red bg");
    let green = fill_draw_pos(&result, [0, 128, 0]).expect("float green bg");
    let glyph = first_glyph_pos(&result).expect("host text ink");
    assert!(red < green, "float({green}) 应绘于宿主 bg({red}) 之上（step4 > step3）");
    assert!(
        green < glyph,
        "宿主墨({glyph}) 应盖 float({green})（step5 > step4；驱动案 c5525-fltmrgn test6 img mr−30 红格）"
    );
}

/// ★R5420 树序保持（overhanging-float-paint-order 语义）：跨宿主两 float 树序
/// 不因逐容器目标位分拆倒置——前树 float（绿，宿主 a 无墨）与后树 float（品红，
/// 宿主 b 带墨）目标位同落 b pre-ink，同目标按树序保序输出。
#[test]
fn r5472_floats_keep_tree_order_across_hosts() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#a { background: rgb(255,255,0); }
#b { background: rgb(0,0,255); }
.f { float: left; width: 50px; height: 30px; }
</style>
<div id="a"><div class="f" style="background: rgb(0,128,0)"></div></div>
<div id="b"><div class="f" style="background: rgb(255,0,255)"></div>xxxx</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let green = fill_draw_pos(&result, [0, 128, 0]).expect("float 1 green bg");
    let magenta = fill_draw_pos(&result, [255, 0, 255]).expect("float 2 magenta bg");
    assert!(
        green < magenta,
        "跨宿主 float 树序应保持 green({green}) < magenta({magenta})（R5420 驱动案 overhanging-float-paint-order 语义）"
    );
}
