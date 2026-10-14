//! 徽章 pill 背景：`border-radius` 超盒尺寸时 §5.5 重叠收缩保证背景可见。
//!
//! 欢迎页（zero://newtab）v0.1.0 徽章 pill（`.version`：inline-block、`background`
//! 加 `border-radius: 999px`）文字渲染但背景整体缺失。根因：Px 半径在
//! `BorderRadiusSpec::from_style_with_box` 原样透传，未按 css-backgrounds-3 §5.5
//! 收缩相邻圆角；未收缩半径使圆角光栅化的每个像素都落入角部判定分支且距圆心
//! 远超半径，背景整体丢弃。修复 = used-value 解析处等比收缩（共享路径）。
//!
//! 断言锚点：发射的 rounded_rect 图元半径必须 ≤ min(w,h)/2（§5.5 不变量），
//! 修复前该断言在 999 vs 13.5 上必败。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::color::Color;

/// 欢迎页徽章 pill 颜色 #e8eaed。
fn pill_color() -> Color {
    Color::rgb(232, 234, 237)
}

/// 普通块流中的 inline-block span pill（最简 fixture）。
#[test]
fn badge_pill_inline_block_background_emits_clamped_rounded_rect() {
    let html = r#"<html><head><style>
body { font: 16px sans-serif; }
.v { display: inline-block; background: #e8eaed; padding: 4px 10px; border-radius: 999px; }
</style></head><body>
<p>text <span class="v">v0.1.0</span></p>
</body></html>"#;
    let mut pipeline = RenderPipeline::new(920.0, 600.0);
    let result = pipeline.render_html(html, "");
    let rects: Vec<_> = result
        .primitives()
        .rounded_rects
        .iter()
        .filter(|r| r.color == pill_color())
        .collect();
    assert_eq!(rects.len(), 1, "应恰好发射 1 个 pill 色 rounded_rect");
    let rr = rects[0];
    let (w, h) = (rr.rect.size.width, rr.rect.size.height);
    assert!(w > 0.0 && h > 0.0, "pill 盒不得为空");
    // §5.5 不变量：收缩后半径 ≤ 半边长（修复前 999 直通，光栅化背景整体丢失）。
    let half = w.min(h) / 2.0;
    for (name, r) in [
        ("tl", rr.top_left_radius),
        ("tr", rr.top_right_radius),
        ("br", rr.bottom_right_radius),
        ("bl", rr.bottom_left_radius),
    ] {
        assert!(
            r <= half + 1e-3,
            "{name} 半径 {r} 须收缩到 ≤ 半边长 {half}（未收缩则背景光栅化失效）"
        );
    }
}

/// 欢迎页 hero 结构：flex 容器（`.hero-top`）内的徽章 pill 同样须背景可见。
#[test]
fn badge_pill_in_flex_container_emits_clamped_rounded_rect() {
    let html = r#"<html><head><style>
body { font: 16px sans-serif; }
.hero-top { display: flex; align-items: baseline; justify-content: space-between; }
.v { display: inline-block; background: #e8eaed; padding: 4px 10px; border-radius: 999px; }
</style></head><body>
<div class="hero-top"><p>Welcome</p><span class="v">v0.1.0</span></div>
</body></html>"#;
    let mut pipeline = RenderPipeline::new(920.0, 600.0);
    let result = pipeline.render_html(html, "");
    let rects: Vec<_> = result
        .primitives()
        .rounded_rects
        .iter()
        .filter(|r| r.color == pill_color())
        .collect();
    assert_eq!(rects.len(), 1, "flex 容器内的 pill 背景须发射");
    let rr = rects[0];
    let half = rr.rect.size.width.min(rr.rect.size.height) / 2.0;
    assert!(
        rr.top_left_radius <= half + 1e-3,
        "tl={} 须 ≤ {half}",
        rr.top_left_radius
    );
}
