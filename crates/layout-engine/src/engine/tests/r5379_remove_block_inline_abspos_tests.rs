//! R5379 回归测试：块级隔断 display:none 后 inline 流 abspos 静态位两症状
//!（WPT remove-block-between-inline-and-abspos ✗1.25% 驱动）：
//! ① 锚 y 缺混合容器块流偏移——断行层锚是 IFC 局部系行顶（current_y 0 基），
//!   容器前驱块级子使 IFC 宿主位 = 前驱块流底（CSS2 §10.3.7 static position）；
//! ② 锚 x 含行尾可折叠空白——phase-2 行尾空白移除后假想盒位 = 塌缩后光标
//!   （CSS Text §4.1.2）。
//! 守卫：纯 inline 容器（无块级前驱）锚位不折算、行中空白（锚后有词）不扣。

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

fn compute(html: &str) -> (zero_dom::NodeId, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (abs_id, result)
}

/// WPT 案面骨架（removeMe 保留 DOM、display:none，与 reftest 脚本后终态同构）：
/// abspos 与 inline-block 同行顶（y=50 = 前驱块高）+ 行尾空白塌缩后光标（x=100）。
#[test]
fn r5379_remove_block_inline_abspos_static_position() {
    let html = r#"<html><body style="margin:0">
<div style="width:200px;">
  <div style="height:50px;"></div>
  <div id="ib" style="display:inline-block; vertical-align:top; width:100px; height:150px;"></div>
  <div id="removeMe" style="height:100px; display:none;"></div>
  <span id="abs" style="position:absolute; width:100px; height:150px;"></span>
</div>
</body></html>"#;
    let (abs_id, result) = compute(html);
    let abs = find(&result.root, abs_id).unwrap();
    assert_eq!(
        abs.y, 50.0,
        "abspos 静态位 y 应同行顶 50（含前驱块流偏移），实际 {}",
        abs.y
    );
    assert_eq!(abs.x, 100.0, "abspos 静态位 x 应行尾空白塌缩后光标 100，实际 {}", abs.x);
}

/// 守卫：纯 inline 容器（无块级前驱）锚位不折算——IFC 原点 = 容器顶，既有
/// line-breaking-019 面不变（锚 x = 行内光标、y = 行顶 0）。
#[test]
fn r5379_pure_inline_container_offset_zero() {
    let html = r#"<html><body style="margin:0">
<div style="width:400px;"><span id="abs" style="position:absolute; width:10px; height:10px;"></span>X</div>
</body></html>"#;
    let (abs_id, result) = compute(html);
    let abs = find(&result.root, abs_id).unwrap();
    assert_eq!(abs.y, 0.0, "纯 inline 容器锚 y 应行顶 0（无块流偏移），实际 {}", abs.y);
    assert_eq!(abs.x, 0.0, "纯 inline 容器锚 x 应行首光标 0，实际 {}", abs.x);
}

/// 守卫：行中空白（锚后有词内容）不扣 advance——锚位含前导空格宽（行中空白
/// phase-2 不移除；假想盒位在空白之后）。
#[test]
fn r5379_mid_line_whitespace_not_collapsed() {
    let html = r#"<html><body style="margin:0">
<div style="width:400px; font:16px Ahem;"><span id="abs" style="position:absolute; width:10px; height:10px;"></span> XX</div>
</body></html>"#;
    let (abs_id, result) = compute(html);
    let abs = find(&result.root, abs_id).unwrap();
    assert_eq!(abs.x, 0.0, "行首锚（无前导空白）x 应 0，实际 {}", abs.x);
    // 对照组：锚前有 inline 内容 + 空白 + 锚后有词 → 行中语义锚 x 含空白 advance。
    let html2 = r#"<html><body style="margin:0">
<div style="width:400px; font:16px Ahem;"><span style="display:inline-block; width:30px; height:10px;"></span> <span id="abs" style="position:absolute; width:10px; height:10px;"></span> YY</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html2);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs2_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result2 = engine.compute(&doc, &styles);
    let abs2 = find(&result2.root, abs2_id).unwrap();
    assert_eq!(
        abs2.x, 46.0,
        "行中空白锚 x 应 30+16（Ahem 空格 advance 保留），实际 {}",
        abs2.x
    );
}
