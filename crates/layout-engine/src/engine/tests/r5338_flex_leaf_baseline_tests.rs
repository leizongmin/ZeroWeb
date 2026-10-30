//! R5338（css-flexbox-1 §8.5 baseline alignment）：flex 叶首行基线通道 pin 测试。
//!
//! 修复前叶节点 `first_baselines` 恒 `None`，taffy 以 border-box 底边合成基线——
//! `align-items: baseline` 退化为底对齐（flexbox_align-items-baseline ✗1.77%：
//! 三项 1em 步进反向阶梯 vs chromium 同基线）。修复后叶测量经
//! `measure_text_content_with_baseline` 产出首行基线（内容系）+ border/padding
//! 顶距换算（engine `leaf_first_baseline_border_relative`）→ taffy 基线对齐生效。
//!
//! 注：`compute_styles(&doc, &[])` 不收集 `<style>` 元素（测试管线按空样式表
//! 参数驱动，见同目录既有用例），故夹具全部用内联 style 属性。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 收集 flex 容器（display:flex，经 styles 判定）直接子盒的 (序号, 顶 y, 高)。
fn flex_item_boxes(
    root: &LayoutBox,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
) -> Vec<(usize, f32, f32)> {
    let mut containers: Vec<&LayoutBox> = Vec::new();
    let mut stack: Vec<&LayoutBox> = vec![root];
    while let Some(box_node) = stack.pop() {
        let is_flex = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.display, zero_style_system::DisplayValue::Flex));
        if is_flex {
            containers.push(box_node);
        }
        stack.extend(&box_node.children);
    }
    let container = containers.first().expect("应有 flex 容器");
    container
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.y, c.height))
        .collect()
}

fn drive(html: &str) -> Vec<(usize, f32, f32)> {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let items = flex_item_boxes(&result.root, &styles);
    assert_eq!(items.len(), 3, "应有三个 flex item");
    items
}

/// 驱动腿：align-items: baseline + 异高（32/48/64px）+ 中项 margin-top 16px——
/// 三项文本皆在盒顶，基线对齐 ⇒ 盒顶 y 相等（chromium 真值：全 33，margin
/// 参与基线偏移；修复前 = 底对齐阶梯 17/33/49）。
#[test]
fn r5338_baseline_alignment_tops_equal() {
    let html = r#"<html><body style="margin:0"><div style="display:flex; align-items:baseline; height:96px; margin:16px 0 0 16px;"><span style="display:inline-block; width:100px; height:32px; margin:0 8px;">one</span><span style="display:inline-block; width:100px; height:48px; margin:16px 8px 0;">two</span><span style="display:inline-block; width:100px; height:64px; margin:0 8px;">three</span></div></body></html>"#;
    let items = drive(html);
    let tops: Vec<f32> = items.iter().map(|(_, y, _)| *y).collect();
    let spread = tops[0].max(tops[1]).max(tops[2]) - tops[0].min(tops[1]).min(tops[2]);
    assert!(
        spread < 0.5,
        "基线对齐下三项盒顶应相等（文本皆在盒顶）；实际 tops={tops:?}"
    );
}

/// 守卫腿：同几何换 align-items: flex-start——margin-box 顶对齐，
/// margin-top 项（s2）独沉 16px，其余两项顶相等；确保基线修复未外溢到
/// 非 baseline 对齐（回归哨）。
#[test]
fn r5338_flex_start_guard_unchanged() {
    let html = r#"<html><body style="margin:0"><div style="display:flex; align-items:flex-start; height:96px; margin:16px 0 0 16px;"><span style="display:inline-block; width:100px; height:32px; margin:0 8px;">one</span><span style="display:inline-block; width:100px; height:48px; margin:16px 8px 0;">two</span><span style="display:inline-block; width:100px; height:64px; margin:0 8px;">three</span></div></body></html>"#;
    let items = drive(html);
    let y1 = items[0].1;
    let y2 = items[1].1;
    let y3 = items[2].1;
    assert!(
        (y1 - y3).abs() < 0.5,
        "flex-start 无 margin 项顶应相等；实际 y1={y1} y3={y3}"
    );
    assert!(
        (y2 - y1 - 16.0).abs() < 0.5,
        "flex-start margin-top 项应独沉 16px；实际 y2={y2} y1={y1}"
    );
}
