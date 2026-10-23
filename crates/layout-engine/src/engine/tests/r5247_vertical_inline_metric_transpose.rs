//! R5247：vertical 帧 inline_element_metrics 行高槽转置锚点。
//!
//! R4440 已把 `text_node_line_heights` 槽在 vertical 下取 frag.width（列宽=行高），
//! 但 `inline_element_metrics` 的行高分量漏转置——vertical frag.height = 行内 advance，
//! 扁平化 run（node_id = span 元素自身）经 Path B/measure 重建的三级回退把 advance
//! 当行高：`<h1><span>6</span>月</h1>`（fs 32，span run lh=32 vs 直系月 run lh=37）
//! 在 vertical_text_runs_are_compatible 判 false 拆列推进，字形同位叠画
//!（css-text text-transform-fullwidth-004/005 ✗0.14/0.19 实证）。
//!
//! 锚：vertical Ahem 100px + line-height:1.5（行高 150 ≠ advance 100）——修前行高
//! 槽存 advance 100，修后存行高 150。

use super::*;

/// 在布局树中找持有 span metric 的盒，返回其行高槽值。
fn find_box_with_metric(box_: &LayoutBox, span_id: zero_dom::NodeId) -> Option<f32> {
    if let Some(&(_, lh)) = box_.inline_element_metrics.get(&span_id) {
        return Some(lh);
    }
    box_.children.iter().find_map(|c| find_box_with_metric(c, span_id))
}

/// R5247 锚：vertical 容器内 span 扁平化 run 的行高槽须存行高（150），非 advance（100）。
#[test]
fn r5247_vertical_inline_element_metrics_line_height_transposed() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"writing-mode:vertical-rl; font:100px Ahem; line-height:1.5\"><span>ab</span></div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找 span 元素 id（div 直下唯一 inline 子）。
    let div = result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| b.is_block_level)
        .expect("块容器存在");
    let span_id = doc
        .child_nodes(div.node_id.expect("div 有 node id"))
        .iter()
        .copied()
        .find(|&id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "span"))
        })
        .expect("span 元素存在");

    // 修前行高槽 = frag.height = advance（100px Ahem 单字 advance）；修后 = 150。
    // metric 存于持有 IFC 的容器盒（div），非 root。
    let lh = (find_box_with_metric(&result.root, span_id))
        .unwrap_or_else(|| panic!("span 扁平化 run 的 metric 须存储于某容器盒"));
    assert!(
        (lh - 150.0).abs() < 1.0,
        "vertical 下 inline_element_metrics 行高槽应存行高 150（R4440 同款转置），实 {lh}"
    );
}
