//! R4059 回归：blur box-shadow 的裁剪窗口（css-contain-1 §paint containment / overflow）。
//!
//! blur（3σ 外扩）会越过 shadow.rect 渲染，paint 侧 rect ∩ clip 挡不住溢出——
//! clip_all_primitives_to_rect 现把窗口写 `shadow.clip`，renderer（cpu/gpu）按窗口
//! 硬裁 blur 输出。driving: css-contain/contain-paint-026（contain:paint 容器内
//! `box-shadow: 0 0 100px 100px red` 子元素，溢出晕染应被裁剪）。

use crate::pipeline::RenderPipeline;

/// contain:paint 容器内的 blur 阴影：shadow.clip 应被写入（窗口 = 容器 padding-box）。
#[test]
fn r4059_contain_paint_child_shadow_gets_clip_window() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"contain: paint; width:100px; height:100px; background:red\">\
<div style=\"background:green; box-shadow:0 0 100px 100px red; width:100px; height:100px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(!p.shadows.is_empty(), "R4059: 子元素 blur 阴影应生成 shadow 图元");
    // 容器 padding-box = (0,0,100,100)（无 border/padding）——blur 溢出部分须按此窗口裁。
    for (i, s) in p.shadows.iter().enumerate() {
        let clip = s.clip.expect("R4059: 被裁剪容器内的阴影应有 clip 窗口");
        assert!(
            (clip.size.width - 100.0).abs() < 0.5 && (clip.size.height - 100.0).abs() < 0.5,
            "R4059: shadow#{} clip 窗口应为容器 padding-box 100×100，got {}×{}",
            i,
            clip.size.width,
            clip.size.height
        );
    }
}

/// 对照：无裁剪容器内的 blur 阴影 clip = None（溢出晕染照常渲染）。
#[test]
fn r4059_unclipped_shadow_has_no_clip_window() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"width:100px; height:100px\">\
<div style=\"background:green; box-shadow:0 0 100px 100px red; width:100px; height:100px\"></div></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(!p.shadows.is_empty(), "R4059 对照: 阴影应生成");
    assert!(
        p.shadows.iter().all(|s| s.clip.is_none()),
        "R4059 对照: 无裁剪容器的阴影 clip 应为 None"
    );
}
