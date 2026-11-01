//! R5380 回归测试：svg default 尺寸臂 content 派生高与行向 flex 交叉轴的兼容
//!（WPT display-flex-svg-overflow-default ✗1.04% 驱动）：
//! default 臂（宽高 attr/CSS 全无来源）definite 高写入使 flex item 交叉轴恒非
//! auto → stretch 跳过 → svg 150 高；css-flexbox-1 §8.3 单行 definite 容器
//! line cross = 容器 inner cross + §9.6 stretch（§4.5 automatic min 经 SVG2 §7.2
//! UA overflow:hidden = 0）应 100×100。
//! 守卫：非 flex（R4090 default 150×150）与显式高 attr（R4091 definite 交叉）不变。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<LayoutBox> {
    let mut stack = vec![root.clone()];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter().cloned());
    }
    None
}

fn compute(html: &str) -> (NodeId, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let svg_id = doc.get_elements_by_tag_name("svg")[0];
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (svg_id, result)
}

/// WPT 案面：row flex 容器 100×100 + svg flex-grow:1（rect 150 子）→ stretch 后
/// 100×100（body margin 8 → abs 8,8）。
#[test]
fn r5380_svg_flex_row_stretch_cross_auto() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:100px; height:100px;">
  <svg style="flex-grow:1;"><rect width="150" height="150" fill="green"></rect></svg>
</div>
</body></html>"#;
    let (svg_id, result) = compute(html);
    let svg = find(&result.root, svg_id).unwrap();
    assert_eq!(svg.width, 100.0, "svg 主轴应 flex-grow+shrink 100，实际 {}", svg.width);
    assert_eq!(
        svg.height, 100.0,
        "svg 交叉轴应 stretch 至 line cross 100（default 高保 auto），实际 {}",
        svg.height
    );
}

/// 守卫：非 flex（普通流）同 svg → R4090 default 150×150 不变。
#[test]
fn r5380_svg_block_flow_default_size_unchanged() {
    let html = r#"<html><body style="margin:0">
<svg><rect width="150" height="150" fill="green"></rect></svg>
</body></html>"#;
    let (svg_id, result) = compute(html);
    let svg = find(&result.root, svg_id).unwrap();
    assert_eq!(
        svg.width, 150.0,
        "非 flex svg default 宽应 150（R4090），实际 {}",
        svg.width
    );
    assert_eq!(
        svg.height, 150.0,
        "非 flex svg default 高应 150（R4090），实际 {}",
        svg.height
    );
}

/// 守卫：flex item svg 带**显式**高 attr（R4091 臂）→ definite 交叉保留，stretch
/// 跳过（height 50；主轴 shrink 100）。
#[test]
fn r5380_svg_flex_explicit_height_definite_kept() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:100px; height:100px;">
  <svg style="flex-grow:1;" height="50"><rect width="150" height="150" fill="green"></rect></svg>
</div>
</body></html>"#;
    let (svg_id, result) = compute(html);
    let svg = find(&result.root, svg_id).unwrap();
    assert_eq!(
        svg.height, 50.0,
        "显式高 attr svg 交叉应 definite 50，实际 {}",
        svg.height
    );
    assert_eq!(svg.width, 100.0, "主轴仍应 shrink 100，实际 {}", svg.width);
}
