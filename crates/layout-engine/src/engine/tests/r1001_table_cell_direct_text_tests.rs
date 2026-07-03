//! R1001（CSS Tables）：table cell 的**直接匿名 inline 文本**参与 cell intrinsic width。
//!
//! 驱动 `table-cell-overflow-explicit-height-001/002`：cell 含一个 block 子（无显式宽）+
//! 直接文本 "Can you see this text?"。box_content_max_width 仅测叶盒文本 + block 子递归，
//! cell 的直接文本（非叶、与 block 子混合生成匿名 block）被漏测，致 cell 塌缩到 block 子
//! border 宽。R1001 在 compute_cell_intrinsic_width 补测 cell 直接文本节点子元素（非全后代）。

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

/// R1001 驱动案：cell 含 block 子 + 直接文本 → cell 宽应≈直接文本宽（非 block 子 border 宽）。
#[test]
fn r1001_table_cell_direct_text_contributes_width() {
    let html = r#"<html><body style="margin:0">
<table><td><div style="background:blue"></div>Can you see this text?</td></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let td_id = doc.get_elements_by_tag_name("td").into_iter().next().expect("td");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, td_id).expect("td box found");
    // 默认 16px 字体，"Can you see this text?" 22 字符 × ~9.6px ≈ 211px。
    // 修复前 cell 塌缩到 empty div border（~0-2px），修复后 ≈ 文本宽。
    assert!(
        w > 150.0,
        "R1001: cell with direct text + empty block child should be ~text width (211), got {w}"
    );
}

/// R1001 安全性：cell 的文本在 block 后代内（无直接文本）→ 不应过计（margin-collapse-101 守卫）。
#[test]
fn r1001_table_cell_text_in_block_descendant_no_overcount() {
    let html = r#"<html><body style="margin:0">
<table><td><div>short</div></td></table>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let td_id = doc.get_elements_by_tag_name("td").into_iter().next().expect("td");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, td_id).expect("td box found");
    // "short" 在 block 子 div 内（非 cell 直接文本），cell 宽 ≈ "short" 宽（~30px），不应过计。
    assert!(
        w < 120.0,
        "R1001: text in block descendant should not over-count cell width, got {w}"
    );
}
