//! R5412：replaced AR flex item 的 definite min-cross 传递修复（css-flexbox §4.5
//! automatic minimum size + transferred size suggestion + §9.4）。
//!
//! 案形（flex-minimum-width-flex-items-015）：外层 column flex-basis 给 row flexbox
//! definite 高 100，img min-height:100%——automatic minimum = min(transferred
//!（min-cross × ratio）, content suggestion)，收缩穿透内容建议地板 200 → 100×100。
//! 修复前 % min-cross 的 flex 求解基准静态树构建期不可见（「只抬不降」floor 留
//! 200×200）；修复在本首趟 laid 修复层解析（taffy 父节点 content cross），并释放
//! R3794 both-auto 设备的 definite cross（阻断 §9.4 stretch/min 解析的尾差）。
//!
//! kill-switch `ZW_FLEX_MIN_CROSS_TRANSFER=0`（OFF 精确回退由 corpus A/B 双窗
//! 覆盖，R3858 口径）。

use std::collections::HashMap;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;

// 2×2 PNG data URI（解码固有由 runner 同语义 map 钉为 200×200，对齐 corpus
// support/200x200-green.png）。
const IMG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAEElEQVR4nGP8z8DwnwEPYMInFXw8aAAAAABJRU5ErkJggg==";

/// corpus 015 同语义渲染：runner map 钉固有 200×200 + img green 背景（填充几何
/// 即 img 盒几何）。
fn render_case(img_extra: &str) -> (f32, f32, f32, f32) {
    let html = format!(
        r#"<div style="display: flex; flex-direction: column; width: 100px; height: 444px;">
  <div style="flex-basis: 100px; display: flex; min-height: 0px;">
    <img src='{IMG}' style="{img_extra} background: green;">
  </div>
</div>"#,
        IMG = IMG,
        img_extra = img_extra
    );
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let key = simple_hash(IMG.trim());
    let mut sizes = HashMap::new();
    sizes.insert(key, (200.0f32, 200.0f32));
    pipeline.set_image_sizes(sizes);
    let mut natural = HashMap::new();
    natural.insert(key, (200.0f32, 200.0f32));
    pipeline.set_image_natural_sizes(natural);
    let result = pipeline.render_html(&html, "");
    let green = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.color.a == 255)
        .expect("green fill");
    (
        green.rect.origin.x,
        green.rect.origin.y,
        green.rect.size.width,
        green.rect.size.height,
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

/// ★load-bearing（corpus 015 形态）：min-height:100% × ratio 1 → automatic min
/// main = 100，shrink 200→100；交叉轴释放 both-auto 设备后 stretch/min 解析
/// → 高 100。ref-filled-green-100px-square = 精确 100×100。
#[test]
fn r5412_min_cross_pct_transfers_and_caps_automatic_min() {
    let rect = render_case("min-height: 100%;");
    assert_rect(rect, (8.0, 8.0, 100.0, 100.0), "015 形态");
}

/// 无 min-cross 让位：automatic minimum = 内容建议 200（classic flex img
/// no-shrink 行为），cap 不降、设备不释放——200×200 维持。
#[test]
fn r5412_no_min_cross_keeps_content_suggestion() {
    let rect = render_case("");
    assert_rect(rect, (8.0, 8.0, 200.0, 200.0), "无 min 形态");
}
