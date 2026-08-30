//! R3818 回归：canvas 传播背景的 positioning area = 根元素（html）**padding-box**
//! （CSS2 §14.2——无论背景实际来自 html 传播还是 body fallback）。
//!
//! 旧实现 anchor = html border-box 原点（layout.x/y）；html 有 border 时相位错
//! border 值：background-root-007 test（html border 3px + body pos 0,0）tile 相位
//! 16 vs ref（html 无 border + body pos 2px）2 → 37.83% diff。修 = anchor 加 border。
//! chromium CDP 实测（2026-08-30）：007 两页 tile 网格相位一致 (2,2)；008/010 同规则。

use crate::pipeline::RenderPipeline;

/// body fallback + html border：positioning area 仍 = html padding-box（非 body 盒、
/// 非画布原点）。驱动 background-root-007/008/010（4 案翻绿）。
#[test]
fn r3818_body_fallback_bg_anchors_root_padding_box() {
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    // html{margin:16px; border:3px solid blue}（padding-box 原点 (19,19)）
    // body{background: linear-gradient(...) ; position 0,0; margin:0}
    // 若 anchor = html border-box (16,16)：gradient primitive 落 x=16；
    // 修复后 anchor = padding-box (19,19)：落 x=19。
    let html = "<html style=\"margin:16px; border:3px solid blue\">\
                <body style=\"margin:0; background-image:linear-gradient(to right, red, blue)\">\
                <p>x</p></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(!grads.is_empty(), "R3818: body fallback gradient 应传播到画布");
    // gradient positioned = anchor_x + offset（pos 默认 0%）→ left 应为 19（padding-box 原点）。
    let left = grads[0].rect.left();
    assert!(
        (left - 19.0).abs() < 1.0,
        "R3818: canvas bg anchor 应为根 padding-box 原点 x=19（border-box 16 + border 3），got {}",
        left
    );
}

/// html 传播 + html border：同一 padding-box 规则（对称面；html 自身有背景时）。
#[test]
fn r3818_html_propagation_bg_anchors_root_padding_box() {
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    // html{margin:16px; border:3px solid blue; background:linear-gradient(...)}
    let html = "<html style=\"margin:16px; border:3px solid blue; \
                background-image:linear-gradient(to right, red, blue)\">\
                <body style=\"margin:0\"><p>x</p></body></html>";
    let result = pipeline.render_html(html, "");
    let grads = &result.primitives().gradients;
    assert!(!grads.is_empty(), "R3818: html propagation gradient 应传播到画布");
    let left = grads[0].rect.left();
    assert!(
        (left - 19.0).abs() < 1.0,
        "R3818: html 传播 anchor 同为根 padding-box 原点 x=19，got {}",
        left
    );
}
