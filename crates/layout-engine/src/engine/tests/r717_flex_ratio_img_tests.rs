//! R717（CSS §10.3.2 + Flexbox §4.5）：ratio-only SVG `<img>`（%-dim / viewBox-only）
//! 在 flex 容器内按 transferred-size 推导 main 尺寸的回归测试。
//!
//! 驱动案 `aspect-ratio-intrinsic-size-007`：`<svg width="100%" height="100%" viewBox="0 0 7500 3750">`
//! 经 `<img>` 嵌入 `<div style="display:flex;flex-direction:column">`。SVG 无确定固有尺寸、仅有
//! viewBox 比 2:1。期望 img 在 flex column 内 width 拉伸到容器宽（800）、height = width/ratio = 400。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 在布局树中查找指定 DOM NodeId 的盒尺寸 (width, height)。
fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// R717 驱动案：flex column + ratio-only SVG img（ratio 2:1）→ width 拉伸 800、height=400。
#[test]
fn r717_flex_column_ratio_only_img_derives_height() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column"><img src="large-green-rectangle.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    // ratio-only 信号：ratio = 2.0（viewBox 7500×3750），无确定固有尺寸。
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // 容器宽 800（flex column align-stretch cross=width）；height = width / ratio = 400。
    assert!(
        (w - 800.0).abs() < 2.0,
        "img width should stretch to container 800, got {w}"
    );
    assert!(
        (h - 400.0).abs() < 2.0,
        "R717: img height should ratio-derive to 400 (800/2), got {h}"
    );
}

/// R717 flex row 对称：ratio-only img 在 flex row（明确 height 200）→ width = height × ratio。
#[test]
fn r717_flex_row_ratio_only_img_derives_width() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;height:200px"><img src="r.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // flex row：cross=height 拉伸到容器 200；main=width = height × ratio = 400。
    assert!(
        (h - 200.0).abs() < 2.0,
        "img height should stretch to container 200, got {h}"
    );
    assert!(
        (w - 400.0).abs() < 2.0,
        "R717: img width should ratio-derive to 400 (200×2), got {w}"
    );
}

/// R717 非 flex 父（block）不应触发 ratio-derivation——img 保持无确定尺寸（不 collapse 也不强推）。
/// 此前该 img 同样无 size（ratio-only SVG 从不在 image_sizes 中），故不构成回归。
#[test]
fn r717_block_parent_ratio_only_img_no_force() {
    let html = r#"<html><body style="margin:0">
<div><img src="r.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    // 仅断言不 panic + img 存在；非 flex 块上下文 ZW 暂未实现 300×150 默认（独立 gap）。
    let (_w, _h) = find_box(&result.root, img_id).expect("img box found");
}
