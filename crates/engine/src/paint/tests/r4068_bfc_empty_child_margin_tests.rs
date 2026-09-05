//! R4068 回归：BFC 父内空子 collapse-through 边距计账（chromium 151 oracle 实证修订）。
//!
//! R3754 曾把「BFC 父内空块的 mb 计入父高」当作 chromium 语义（父高 60）；2026-09-06
//! chromium 151 oracle 实测 `overflow:hidden 父 > 空子(margin:30px 0)` 父高 = **30**
//! （childY=30，mb 与父底边折叠不计高；content-002 链 lime=30/blue=90/orange=150 同证；
//! 带 border 空子才全计 62）。R3754 记录的「Chromium 60」为误测。
//!
//! 修 = R1743 回填 fold：BFC 父内空块仍不排除（mt 计入 child.y），但底边贡献去掉
//! margin_bottom（R4068）。driving 翻绿：margin-collapse-102/103（abspos BFC 父内
//! 空子 m:5em 双边距，父高只含 mt）。
//!
//! 余账（本轮定位未修）：contain:layout/paint/content 父（R3755 taffy overflow:Hidden
//! 注入路径）的空子 LayoutBox.height 在回填前已被合成 mt+h+mb（60），回填只增不缩
//! → contain-content-002 lime 仍 60（chromium 30）。overflow:hidden 父无此合成路径。

use crate::pipeline::RenderPipeline;

fn depth_metrics(result: &crate::pipeline::RenderResult) -> (f32, f32) {
    // 返回 (depth2 块容器 content 高, depth3 子 y)。结构：html(0) / head+body(1) / 父(2) / 空子(3)。
    let mut parent_h = 0.0;
    let mut child_y = 0.0;
    fn walk(b: &zero_layout_engine::types::LayoutBox, depth: usize, parent_h: &mut f32, child_y: &mut f32) {
        if depth == 2 {
            *parent_h = b.content_height;
        }
        if depth == 3 {
            *child_y = b.y;
        }
        for c in &b.children {
            walk(c, depth + 1, parent_h, child_y);
        }
    }
    walk(&result.layout.root, 0, &mut parent_h, &mut child_y);
    (parent_h, child_y)
}

/// overflow:hidden 父 + 空子(margin:30 0)：父高 30（chromium oracle），空子 y=30。
#[test]
fn r4068_bfc_parent_empty_child_height_30() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow:hidden\"><div style=\"margin:30px 0\"></div></div></body></html>";
    let (parent_h, child_y) = depth_metrics(&pipeline.render_html(html, ""));
    assert!(
        (parent_h - 30.0).abs() < 0.5,
        "R4068: BFC 父内空子 collapse-through 后父高应只含 mt=30（chromium oracle），got {parent_h}"
    );
    assert!((child_y - 30.0).abs() < 0.5, "空子 y=mt=30，got {child_y}");
}

/// 带 border 空子不 collapse-through：父高 = mt+高+mb = 62（chromium oracle）。
#[test]
fn r4068_nonempty_child_still_counts_mb() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow:hidden\"><div style=\"margin:30px 0;border:1px solid black\"></div></div></body></html>";
    let (parent_h, _child_y) = depth_metrics(&pipeline.render_html(html, ""));
    assert!(
        (parent_h - 62.0).abs() < 0.5,
        "R4068 对照: 有边框空子不折叠，父高应 30+2+30=62，got {parent_h}"
    );
}
