//! R5413：taffy scrollbar_width 槽位恒 0——ZW 不绘制 classic scrollbar，声明
//! `overflow:scroll` 的容器不再预留 15px 死槽位（css-scrollbars-1 §scrollbar-width
//! × UA 渲染模型；R4265 的 auto 抑制判例对齐到全部声明值）。
//!
//! 案形（flexbox-overflow-horiz-001）：70×70 flex 容器 + bigItem（flex:3,
//! height:200px）+ smallItem（flex:1, margin-bottom:10px）——ref 以 75%/25% 宽 +
//! calc(100%−10px) 高模拟 scrollbar-less 几何，四容器（无声明/hidden/scroll/auto）
//! 几何必须一致。修复前 scroll 容器 15px 双轴内缩 → smallItem 13.8×45（应 17.5×60）。
//!
//! kill-switch `ZW_SCROLLBAR_SLOT_SUPPRESS=0`（OFF 精确回退由 corpus A/B 双窗
//! 覆盖，R3858 口径；OnceLock 进程级缓存不支撑同进程双极性测试）。

use zero_engine::RenderPipeline;

/// 单容器渲染，返回 teal（smallItem）填充矩形 (x, y, w, h)——smallItem 全程未被
/// 容器裁剪（y=0..60 含 10px margin 恰好 fit），是 scroll/hidden/auto 三声明的
/// 公共无裁剪几何探针。
fn render_small_item_rect(overflow_css: &str) -> (f32, f32, f32, f32) {
    let html = format!(
        r#"<div style="display: flex; width: 70px; height: 70px; {overflow_css}">
  <div style="background: blue; height: 200px; flex: 3;"></div>
  <div style="background: teal; margin-bottom: 10px; flex: 1;"></div>
</div>"#
    );
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(&html, "");
    let teal = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 128 && f.color.a == 255)
        .expect("teal fill");
    (
        teal.rect.origin.x,
        teal.rect.origin.y,
        teal.rect.size.width,
        teal.rect.size.height,
    )
}

fn assert_rect(actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32), tag: &str) {
    for (a, e, axis) in [
        (actual.0, expected.0, "x"),
        (actual.1, expected.1, "y"),
        (actual.2, expected.2, "w"),
        (actual.3, expected.3, "h"),
    ] {
        assert!(
            (a - e).abs() < 0.5,
            "{tag}: {axis} = {a} 应 {e}（实际 {actual:?} 应 {expected:?}）"
        );
    }
}

/// ★load-bearing（corpus 001 scroll 容器形态）：声明 `overflow:scroll` 零槽位——
/// smallItem 拉伸满 cross（60 = 70 − 10 margin），main 占 3/4（17.5）。
/// 修复前 15px 双轴内缩 → 13.75×45。填充矩形像素吸附（60.5→61、17.5→17）。
#[test]
fn r5413_scroll_container_keeps_scrollbarless_geometry() {
    let rect = render_small_item_rect("overflow: scroll;");
    assert_rect(rect, (61.0, 8.0, 17.0, 60.0), "scroll 容器");
}

/// R4265 判例回归守卫：overflow:auto 在恒 0 端点下几何不变（槽位本就不预留）。
#[test]
fn r5413_overflow_auto_geometry_unchanged() {
    let rect = render_small_item_rect("overflow: auto;");
    assert_rect(rect, (61.0, 8.0, 17.0, 60.0), "auto 容器");
}
