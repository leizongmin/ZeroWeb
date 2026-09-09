//! R4176：float 提升钳制的基准 = 假设流位置（margin 应用前）回归测试。
//!
//! negative-block-margin-pushing-float-out-of-block-formatting-context：BFC（abspos
//! 容器）内唯一 float `margin-top:-100px` 应渲染于流位上移 100（chromium 同——负
//! margin 是 float 相对流位的合法偏移，§9.5.1 rule 1/2 的「不得高于」约束针对
//! 假设流位置，非 margin 应用后的 border-box 位）。旧实现拿 mt 应用后的 y 与
//! flow_bottom 比较，负 mt 恒触发钳制把 y 拉回 flow_bottom（负 margin 整体失效，
//! abs_y 恒 = 容器顶，LAYOUT_DUMP 实证）。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

/// 找到 node_id 指定 DOM 元素的 LayoutBox（按 attributes id 匹配不可用，此处用
/// 几何特征定位：width=50 的 float 盒）。
fn find_float_box(root: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
    if !matches!(root.float, zero_css_parser::values::FloatValue::None) && (root.width - 50.0).abs() < 0.5 {
        return Some(root);
    }
    root.children.iter().find_map(find_float_box)
}

/// BFC 内唯一 float `mt:-100`：final y 应含负 margin 偏移（rel = -100），
/// 旧实现被钳回 0（负 margin 失效）。
#[test]
fn test_negative_margin_float_not_clamped_to_flow_top() {
    let html = r#"<html><body style="margin:0"><div style="height:100px"></div><div style="position:absolute"><div style="float:left;width:50px;height:50px;margin-top:-100px;background:green"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = find_float_box(&result.root).expect("should find 50px-wide float box");
    // abs y = 容器 top（body mt 8 + 100 高 = 108）+ rel -100 = 8（chromium 一致）。
    // 旧实现 rel 被钳 0 → abs 108。
    assert!(
        (float_box.y - (-100.0)).abs() < 0.5,
        "negative-margin float rel y must be -100 (margin applied), got {}",
        float_box.y
    );
}
