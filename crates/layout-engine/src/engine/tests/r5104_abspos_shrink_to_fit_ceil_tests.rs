//! R5104（CSS2 §10.3.7 shrink-to-fit）：auto 宽 abspos/fixed 叶的单行测量宽度上取整。
//!
//! taffy absolute 臂以 Definite(CB 宽) 约束跑 ContentSize 测量——IFC 排出单行，
//! measured_width = 文本真实 advance；taffy 盒宽经 round_layout 取整后 paint Path B
//! 以盒宽复排，advance > 盒宽即折行 → layout 单行 vs paint 两行
//!（driving: css/CSS2/positioning abspos-block-level-001）。修 = 单行且未溢出时
//! 盒宽 ceil（≥ advance，paint 复排恰容纳）。判别力由 corpus driving 案承担
//!（runner fontdue 度量下 advance 为小数；单测 estimate 度量按词取整恒整数，
//! 本文件断言 ceil 契约本身：盒宽 == max-content 测量的 ceil）。
//! kill-switch: `ZW_ABSPOS_SHRINK_TO_FIT=0`。
//!
//! 注：样式走 inline style 属性——直构管线 compute_styles(&doc, &[]) 不含文档
//! `<style>` 块（R5081 教训同源）。

use crate::engine::LayoutEngine;
use crate::inline_finalization::measure_text_content;
use crate::types::LayoutBox;
use std::collections::HashMap;
use taffy::geometry::Size;
use taffy::prelude::AvailableSpace;
use zero_dom::{Document, NodeId};
use zero_style_system::StyleSystem;

fn find_box_by_id(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_box_by_id(c, id))
}

fn layout(
    html: &str,
) -> (
    Document,
    HashMap<NodeId, zero_style_system::ComputedStyle>,
    crate::engine::LayoutResult,
) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, styles, result)
}

const HTML: &str = r#"<html><body style="margin:0">
<div>
  <span>Inline content</span>
  <div id="abs" style="position: absolute">Block-level abspos after inline content</div>
</div>
</body></html>"#;

/// 契约：单行未溢出的 auto 宽 abspos 盒宽 == max-content 测量的 ceil
///（≥ advance，paint Path B 以盒宽复排恰好容纳，不再折行）。
#[test]
fn r5104_single_line_abspos_width_equals_ceil_of_advance() {
    let (doc, styles, result) = layout(HTML);
    let abs_id = doc.get_element_by_id("abs").expect("abspos div");
    let m = measure_text_content(
        &doc,
        &styles,
        abs_id,
        Size {
            width: None,
            height: None,
        },
        Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
        &HashMap::new(),
        crate::inline_finalization::InlineFontContext {
            metric_provider: None,
            advance_source: None,
            resolver: None,
            font_overrides: None,
        },
        None,
    );
    let b = find_box_by_id(&result.root, abs_id).expect("abspos box");
    assert_eq!(
        b.width,
        m.width.ceil(),
        "R5104: 单行 abspos 盒宽应 == max-content advance 的 ceil（paint 复排恰容纳）"
    );
    // 单行高保持（折行会在 layout 侧表现为两行高）。
    assert!(
        (b.height - m.height).abs() < 1.0,
        "R5104: abspos 盒应保持单行高，盒 {} vs 测量 {}",
        b.height,
        m.height
    );
}

/// 多行臂不动：窄 CB 内文本折行时盒宽 = avail（与 paint 复排一致），不 ceil。
#[test]
fn r5104_multiline_arm_keeps_available_width() {
    let html = r#"<html><body style="margin:0">
<div style="width: 120px; position: relative">
  <div id="abs" style="position: absolute">Block-level abspos after inline content words here</div>
</div>
</body></html>"#;
    let (doc, _styles, result) = layout(html);
    let abs_id = doc.get_element_by_id("abs").expect("abspos div");
    let b = find_box_by_id(&result.root, abs_id).expect("abspos box");
    assert!(
        b.width <= 120.5,
        "R5104: 多行（maxc > avail）臂盒宽应 ≤ avail 120，实际 {}",
        b.width
    );
    assert!(
        b.height > 30.0,
        "R5104: 窄 CB 内文本应折行（多行高），实际 {}",
        b.height
    );
}
