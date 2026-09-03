//! R3992（CSS Display 3 §2.3 run-in box）：并入态 run-in 的端到端布局断言。
//!
//! 并入成功时 run-in 自身不留独立块盒：taffy leaf 测量 0（measure gate）+ 后续
//! postprocess 不回填高度，后继块贴紧前文（run-in-basic-001：target 顶 = body 顶）。

use super::*;

fn find_box_by_id(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 并入态 run-in：自身盒高 0、后继块 y = 容器内容顶（无 run-in 占位行）。
#[test]
fn r3992_merged_run_in_leaf_has_zero_height_and_target_not_pushed() {
    let html = r#"<html><body style="margin:0">
<div style="display: run-in; font-weight: bold">Run-in header</div><div id="target">Start of block.</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let run_in_id = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("run-in div");
    let target_id = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("target div");
    let body_id = doc.get_elements_by_tag_name("body").into_iter().next().expect("body");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 并入态：run-in 自身 0 高（旧测量按其 DOM 文本测出单行高 ~18.6，把 target 推下一行）。
    // 经 body 直接子顺序取真实 run-in leaf（target 内另有 hit-test 代理盒 node_id 相同，
    // DFS 需按层级序而非栈序）。
    let body_box = result
        .root
        .children
        .iter()
        .find(|c| c.node_id == Some(body_id))
        .expect("body box");
    let run_in_h = body_box
        .children
        .iter()
        .find(|c| c.node_id == Some(run_in_id))
        .map(|b| b.height)
        .expect("run-in leaf as direct body child");
    assert!(
        run_in_h < 0.5,
        "R3992: merged run-in leaf should have zero height, got {run_in_h}"
    );
    // 后继块不被 run-in 占位行推下：target 顶 = body 内容顶（margin:0）。
    let target_y = body_box
        .children
        .iter()
        .find(|c| c.node_id == Some(target_id))
        .map(|b| b.y)
        .expect("target as body child");
    assert!(
        target_y < 1.0,
        "R3992: target should sit at body content top (no run-in row), got y={target_y}"
    );
}

/// 降级态（后继非块级）：run-in 保持普通块盒占一行。
#[test]
fn r3992_fallback_run_in_keeps_block_height() {
    let html = r#"<html><body style="margin:0">
<div style="display: run-in">Run-in header</div><span>Some text.</span>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let run_in_id = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("run-in div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (_w, run_in_h) = find_box_by_id(&result.root, run_in_id).expect("run-in box");
    assert!(
        run_in_h > 10.0,
        "R3992: fallback run-in should keep a line of height, got {run_in_h}"
    );
}
