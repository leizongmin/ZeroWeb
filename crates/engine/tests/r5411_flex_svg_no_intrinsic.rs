//! R5411：no-ratio SVG（无确定固有尺寸/比，CSS §10.3.2）作 flex item 的双缺陷修复
//!（css-flexbox §9.4 align-stretch + §4.5 content size suggestion + css-sizing-3
//! §5.1.1 default object size）。
//!
//! 案形（flex-svg-no-intrinsic-column-001）：column 容器 150×0 + `<img>`（无尺寸
//! 声明）。修复前两处断点：① no-ratio 臂把 default 300 写死交叉轴 → align-stretch
//! 被阻断（300 ≠ 150）；② R4996 内容建议读 bogus 解码 pixmap（usvg 默认 100×100，
//! 与 no_ratio 信号并存）→ min_size.height=100 → shrink 地板 100（≠ default object
//! size 150）。修复：交叉轴仅在 align-stretch 确将生效时留 auto 让位；no-ratio 权威
//! 信号下内容建议 = default object size 主轴维（宽 300 / 高 150）。
//!
//! kill-switch `ZW_FLEX_NO_RATIO_ITEM=0`（OFF 轮由 corpus 001 ✗4.69% 精确回退覆盖，
//! R3858 口径）。

use std::collections::HashMap;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;

const SRC: &str = "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" />";

/// runner 同语义填充（extract_image_metrics 实测输出：no-ratio SVG 的 bogus pixmap
/// sizes 与 no_ratio 信号并存——修复须在 sizes 遮蔽下仍取 no-ratio 权威值）。
fn render_with_runner_maps(img_extra: &str) -> (f32, f32, f32, f32) {
    let html = format!(
        r#"<!DOCTYPE html><style>
.container {{ display: flex; flex-direction: column; width: 150px; height: 0; }}
img {{ background: green; {img_extra} }}
</style>
<p>Test passes if there is a filled green square.</p>
<div class="container"><img src='{SRC}'></div>"#,
        SRC = SRC,
        img_extra = img_extra
    );
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let key = simple_hash(SRC.trim());
    let mut sizes = HashMap::new();
    sizes.insert(key, (100.0f32, 100.0f32));
    pipeline.set_image_sizes(sizes);
    let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = HashMap::new();
    no_ratio.insert(key, (None, None));
    pipeline.set_image_no_ratio(no_ratio);
    let mut natural = HashMap::new();
    natural.insert(key, (100.0f32, 100.0f32));
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

/// ★load-bearing（corpus 001 形态）：交叉轴让位 align-stretch → 150；主轴 shrink 被
/// default object size 150 内容建议地板托住（bogus 100 pixmap 不再遮蔽）。
#[test]
fn r5411_no_ratio_column_stretch_to_container() {
    let rect = render_with_runner_maps("");
    assert_rect(rect, (8.0, 50.0, 150.0, 150.0), "stretch 形态");
}

/// 非 stretch 守卫：align-self:flex-start 时交叉轴保持 default object size 宽 300
///（CSS2 §10.3.2，不得因让位逻辑塌 0——taffy 非 stretch 的 auto cross = content 0）。
#[test]
fn r5411_no_ratio_non_stretch_keeps_default_cross() {
    let rect = render_with_runner_maps("align-self: flex-start;");
    assert_rect(rect, (8.0, 50.0, 300.0, 150.0), "flex-start 形态");
}
