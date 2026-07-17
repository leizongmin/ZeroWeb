//! R1620：table-cell 内容高度 = in-flow 子元素 border-box 底边最大值（非 sum(heights)）回归测试。
//!
//! floats-wrap-bfc-005/007 谱系：`<td>` 含 float(200×20) + BFC/inline-block(width:50%=150, height:20)。
//! BFC 放不下 float 旁可用宽(100) 被 R1369/table_float_fix 推到 float 下方(y=20)，cell 内容底边
//! 应 = max(float 底 20, BFC 底 20+20=40) = 40。旧 `cell_float_aware_content_height` 用
//! `sum(child.height+margins)`（=20，仅 BFC 高）低估——BFC 被推到 y=20 后 sum(heights)≠max(bottom)。
//! R1620 fix：改 `max(c.y + c.height + margin_bottom)`（spec-correct：cell 高 = 内容 extent）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::{DisplayValue, FloatValue};
use zero_style_system::StyleSystem;

/// 找到第一个 TableCell 盒。
fn find_table_cell<'a>(
    root: &'a LayoutBox,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
) -> Option<&'a LayoutBox> {
    let is_cell = root.node_id.is_some_and(|id| {
        styles
            .get(&id)
            .is_some_and(|s| matches!(s.display, DisplayValue::TableCell))
    });
    if is_cell {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_table_cell(c, styles) {
            return Some(b);
        }
    }
    None
}

/// R1620：td 含 float + overflow:hidden BFC（width:50% 放不下 float）→ BFC 推到 float 下方，
/// cell 高度增长到 40（float 20 + BFC 20 堆叠），而非停在 20（BFC 溢出）。
#[test]
fn test_table_cell_grows_for_bfc_pushed_below_float() {
    let html = r#"<html><body style="margin:0"><table width="300" style="border-spacing:0"><tbody><tr><td style="padding:0;vertical-align:top"><div style="float:left;width:200px;height:20px"></div><div style="overflow:hidden;width:50%;height:20px">50%</div></td></tr></tbody></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cell = find_table_cell(&result.root, &styles).expect("should find td");
    assert!(
        cell.height > 35.0,
        "td must grow to contain BFC pushed below float (≈40), got height={}",
        cell.height
    );
    let _ = FloatValue::None;
}
