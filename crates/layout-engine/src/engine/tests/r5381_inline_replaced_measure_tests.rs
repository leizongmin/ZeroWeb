//! R5381 回归测试：inline 级替换子（canvas，s24 UA flip 后 display:inline）对
//! inline-block shrink-to-fit max-content 的贡献（WPT
//! intrinsic-percent-replaced-014 ✗1.04% 驱动）：
//! css-sizing-3 §5.1 替换子的 max-content contribution = 自身固有/transferred 宽；
//! 旧路径 R1298 空盒臂对无子无文本的 canvas 误判 0 → shrink 臂写 0（先于 R5348
//! grow 臂求值）→ inline-block 塌 0。修复后臂按第一趟 laid-out outer_w 入账。
//! 守卫：display:block canvas（flip 前等价面）与 attr 直读尺寸不变。

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

fn compute(html: &str) -> (NodeId, NodeId, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let inner_id = doc.get_element_by_id("inner").expect("#inner");
    let canvas_id = doc.get_elements_by_tag_name("canvas")[0];
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (inner_id, canvas_id, result)
}

/// 裸 inline canvas（attr 40×10，无 CSS 尺寸）在 auto inline-block 内：max-content
/// 贡献 = attr 固有宽 40 → inner shrink-to-fit 40（旧路径测 0 → inner 塌 0）。
#[test]
fn r5381_inline_canvas_counts_toward_inline_block_max_content() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px; height:50px;">
  <div id="inner" style="display:inline-block; box-sizing:border-box;">
    <canvas width="40" height="10"></canvas>
  </div>
</div>
</body></html>"#;
    let (inner_id, canvas_id, result) = compute(html);
    let inner = find(&result.root, inner_id).unwrap();
    let canvas = find(&result.root, canvas_id).unwrap();
    assert_eq!(
        inner.width, 40.0,
        "inline-block 应按 inline canvas 固有宽 40 shrink-to-fit，实际 {}",
        inner.width
    );
    assert_eq!(canvas.width, 40.0, "canvas attr 固有宽 40，实际 {}", canvas.width);
}

/// WPT 案面：inline canvas `height:100%`（内层 content 高 25）经 ratio 4 transferred
/// 宽 100 → inner shrink-to-fit 100（第一格 display:block canvas 已绿，此为第二格
/// inline 面——回归前 inner 塌 0，第二格 100×50 绿区消失）。
#[test]
fn r5381_transferred_inline_canvas_widths_inline_block() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px; height:50px;">
  <div id="inner" style="display:inline-block; box-sizing:border-box; height:100%; padding-top:100%; background:green;">
    <canvas width="40" height="10" style="height:100%;"></canvas>
  </div>
</div>
</body></html>"#;
    let (inner_id, canvas_id, result) = compute(html);
    let inner = find(&result.root, inner_id).unwrap();
    let canvas = find(&result.root, canvas_id).unwrap();
    assert_eq!(
        inner.width, 100.0,
        "inner 应按 transferred 宽 100 shrink-to-fit，实际 {}",
        inner.width
    );
    assert_eq!(
        canvas.width, 100.0,
        "canvas transferred 宽 = 25×ratio4 = 100，实际 {}",
        canvas.width
    );
}

/// 守卫：display:block canvas（第一格面，回归前已绿）不变——inner 100。
#[test]
fn r5381_block_canvas_face_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px; height:50px;">
  <div id="inner" style="display:inline-block; box-sizing:border-box; height:100%; padding-top:100%; background:green;">
    <canvas width="40" height="10" style="display:block; height:100%;"></canvas>
  </div>
</div>
</body></html>"#;
    let (inner_id, _canvas_id, result) = compute(html);
    let inner = find(&result.root, inner_id).unwrap();
    assert_eq!(
        inner.width, 100.0,
        "block canvas 面 inner 应维持 100，实际 {}",
        inner.width
    );
}
