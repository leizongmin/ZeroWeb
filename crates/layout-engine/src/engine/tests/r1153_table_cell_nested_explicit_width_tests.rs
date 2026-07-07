//! R1153（CSS Tables）：table cell 的**嵌套显式宽后代**参与 cell intrinsic width。
//!
//! 驱动 `c5503-mrgn-b-000`（CSS2/css1）：td.control 的直接子是 .teal/.aqua/.yellow div
//! （无显式 width），但**孙**div .blank{width:100px}/.long{width:70px} 有显式宽。
//! `compute_cell_intrinsic_width` 的直接子元素循环只看一层，会漏测嵌套显式宽后代，
//! 致 control 列回落 char_width+padding（~9.6px）→ 表塌缩（119.6px 应 ~216px，
//! control cell 9.6px 应 ~106px）。R1153 在无直接文本/无直接显式宽子且全 block 直接子时，
//! 用 `box_content_max_width` 递归捕获嵌套显式宽后代。
//!
//! ★ gated：仅当 cell 直接子全为 block 级时采用。含 inline 直接子的 cell（如
//! percent-height-replaced-in-percent-cell-004 的 `<span><canvas display:block>`）
//! 走此回退路径时 inline 子 width 不可靠（R109 纠缠），递归会过测致表爆炸（3.38→88%）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
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

/// R1153 驱动案：cell 直接子（无显式宽 block div）内嵌显式宽孙 div → cell 宽应≈孙显式宽。
/// 修复前 cell 回落 char_width+padding（窄），修复后递归测到孙 100px。
#[test]
fn r1153_table_cell_nested_explicit_width_contributes() {
    let html = r#"<html><body style="margin:0">
<table><td><div><div style="width:100px"></div></div></td></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let td_id = doc.get_elements_by_tag_name("td").into_iter().next().expect("td");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, td_id).expect("td box found");
    // 孙 div 显式 width:100px → cell 列宽 ≈ 100px（+ cell padding/border）。
    // 修复前 cell 回落 char_width（默认 16px 字体 × 0.6 ≈ 9.6px）。
    assert!(w >= 100.0, "cell should measure nested 100px grandchild, got {w}");
    assert!(
        w < 150.0,
        "cell should not over-measure (no inline entanglement), got {w}"
    );
}

/// R1153 gated 守卫：cell 直接子含 inline（span > block div）时，不应用递归过测。
/// 驱动 `percent-height-replaced-in-percent-cell-004` 的 inline-containing-block 形态
/// （span 含 block 子）——递归会过测，gated 排除。
#[test]
fn r1153_table_cell_inline_child_not_over_measured() {
    let html = r#"<html><body style="margin:0">
<table><td><span><div style="width:500px"></div></span></td></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let td_id = doc.get_elements_by_tag_name("td").into_iter().next().expect("td");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, td_id).expect("td box found");
    // span（inline）含 500px block div：gated 不触发递归过测，cell 不应被撑到 ~500px。
    // （span 实际布局宽由 IFC 决定，但 cell intrinsic 不会因递归过测到 500。）
    assert!(
        w < 200.0,
        "inline-child cell should not over-measure via recursion, got {w}"
    );
}
