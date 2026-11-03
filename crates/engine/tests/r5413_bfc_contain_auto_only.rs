//! R5413：BFC float containment 补扩仅 auto 高——显式高度 float 容器不被 float 子
//! 撑高（CSS2 §10.5 used height = 显式值；§10.6.7「含最低 float 外底边」高度公式
//! 限定 auto 高 BFC root，R1277 ④ 非 BFC 收缩臂同判守卫的对称补齐）。
//!
//! 案形（flexbox-overflow-horiz-001 ref 页）：70×70 显式高 float 容器 + 200px
//! float 子——修复前容器被补扩到 200（ref 页整体下移 130px 与 test 页错位，
//! 6.15% 像素差主项）。
//!
//! kill-switch `ZW_BFC_CONTAIN_AUTO_ONLY=0`（OFF 精确回退由 corpus A/B 双窗
//! 覆盖；OnceLock 进程级缓存不支撑同进程双极性测试）。

use zero_engine::RenderPipeline;

/// ref 页容器形态（float 容器 + float 子），返回容器紫背景填充矩形 (x, y, w, h)。
fn render_container_rect(container_css: &str) -> (f32, f32, f32, f32) {
    let html = format!(
        r#"<div style="{container_css}">
  <div style="display: inline-block; background: blue; height: 200px; width: 75%; float: left;"></div>
  <div style="background: teal; width: 25%; height: calc(100% - 10px); float: left;"></div>
</div>"#
    );
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(&html, "");
    let purple = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 128 && f.color.g == 0 && f.color.b == 128 && f.color.a == 255)
        .expect("purple container fill");
    (
        purple.rect.origin.x,
        purple.rect.origin.y,
        purple.rect.size.width,
        purple.rect.size.height,
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

/// ★load-bearing（corpus 001 ref 页形态）：显式高 70 的 float 容器（BFC root）+
/// 200px float 子 → 容器保持 70，float 溢出。修复前补扩到 200。
#[test]
fn r5413_definite_height_float_container_not_grown() {
    let rect = render_container_rect("background: purple; width: 70px; height: 70px; float: left;");
    assert_rect(rect, (8.0, 8.0, 70.0, 70.0), "显式高 float 容器");
}

/// auto 高 float 容器维持 §10.6.7 containment 补扩（float 子 200px → 容器 200）。
#[test]
fn r5413_auto_height_float_container_still_contains_floats() {
    let rect = render_container_rect("background: purple; width: 70px; float: left;");
    assert_rect(rect, (8.0, 8.0, 70.0, 200.0), "auto 高 float 容器");
}
