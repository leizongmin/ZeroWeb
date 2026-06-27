//! R767：表格单元格内容宽度约束。
//!
//! 列定尺寸（`table::position_cells`）后 cell 已有最终宽度，但 cell content
//!（block 子树）仍是 taffy 初始布局（cell 为 body 宽时）的宽度，未 re-layout。
//! 对 **max-content ≤ cell content width**（非 wrapping，装得下）的 width:auto block，
//! clamp 其 width/content_width 到 cell content width（CSS Tables：width:auto block
//! 填满 cell content）。wrapping 内容（max-content > cell 宽）须真正 re-layout，
//! 此处跳过避免 clip。修 margin-collapse-101 等的 div w=778 溢出 cell（27.5）。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// 递归把 cell 子树中 width:auto 的 block 后代约束到 `cell_content_width`。
///
/// 仅 clamp `width:auto` 且 `max-content ≤ cell_content_width`（非 wrapping）的 block；
/// 显式 width 的 block 不 clamp（尊重作者宽度），wrapping 内容跳过（避 clip）。
pub(crate) fn clamp_cell_subtree_to_content_width(
    box_node: &mut LayoutBox,
    cell_content_width: f32,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    let is_block_auto = style.is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::Block
                | DisplayValue::ListItem
                | DisplayValue::FlowRoot
                | DisplayValue::Flow
                | DisplayValue::Flex
                | DisplayValue::Grid
        ) && matches!(s.width, LengthValue::Auto)
    });
    if is_block_auto && box_node.width > cell_content_width + 0.5 {
        // 仅 max-content 装得下时 clamp（非 wrapping）；否则保留（wrapping 须 re-layout）
        let mc = crate::intrinsic_sizing::box_content_max_width(box_node, doc, styles);
        if mc <= cell_content_width {
            box_node.width = cell_content_width;
            box_node.content_width = (cell_content_width
                - box_node.border_left
                - box_node.border_right
                - box_node.padding_left
                - box_node.padding_right)
                .max(0.0);
        }
    }
    for child in &mut box_node.children {
        clamp_cell_subtree_to_content_width(child, cell_content_width, doc, styles);
    }
}

/// 遍历 box 子树，对每个 table-cell，clamp 其 content 子树到 cell content width。
pub(crate) fn constrain_table_cell_content_widths(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let is_cell = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
    if is_cell {
        let cw = box_node.content_width;
        if cw > 0.0 {
            for child in &mut box_node.children {
                clamp_cell_subtree_to_content_width(child, cw, doc, styles);
            }
        }
    } else {
        for child in &mut box_node.children {
            constrain_table_cell_content_widths(child, doc, styles);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::DisplayValue;
    use zero_dom::Document;
    use zero_style_system::ComputedStyle;

    /// 构造 table > cell(content_width=27.5) > block(width=778) 子树，clamp 后 block
    /// 应被约束到 27.5（max-content=0 ≤ 27.5，非 wrapping）。
    #[test]
    fn test_clamp_overwide_block_to_cell_content_width() {
        let mut doc = Document::new();
        let root = doc.root();
        let table_id = doc.create_element("table");
        let cell_id = doc.create_element("td");
        let block_id = doc.create_element("div");
        let _ = doc.append_child(root, table_id);
        let _ = doc.append_child(table_id, cell_id);

        let mut styles = HashMap::new();
        let mut ts = ComputedStyle::default();
        ts.display = DisplayValue::Table;
        styles.insert(table_id, ts);
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::TableCell;
        styles.insert(cell_id, cs);
        let mut bs = ComputedStyle::default();
        bs.display = DisplayValue::Block;
        bs.width = LengthValue::Auto;
        styles.insert(block_id, bs);

        // cell content_width=27.5，block 子 width=778（body 宽，未约束）
        let mut table_box = LayoutBox {
            node_id: Some(table_id),
            children: vec![LayoutBox {
                node_id: Some(cell_id),
                content_width: 27.5,
                width: 27.5,
                children: vec![LayoutBox {
                    node_id: Some(block_id),
                    width: 778.0,
                    content_width: 778.0,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        constrain_table_cell_content_widths(&mut table_box, &doc, &styles);

        let block = &table_box.children[0].children[0];
        assert!(
            (block.width - 27.5).abs() < 0.01,
            "over-wide block 应 clamp 到 cell content width 27.5，实际 {}",
            block.width
        );
    }

    /// 显式 width 的 block 不应被 clamp（尊重作者宽度）。
    #[test]
    fn test_clamp_skips_explicit_width_block() {
        let mut doc = Document::new();
        let root = doc.root();
        let cell_id = doc.create_element("td");
        let block_id = doc.create_element("div");
        let _ = doc.append_child(root, cell_id);

        let mut styles = HashMap::new();
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::TableCell;
        styles.insert(cell_id, cs);
        let mut bs = ComputedStyle::default();
        bs.display = DisplayValue::Block;
        bs.width = LengthValue::Px(300.0); // 显式 300px
        styles.insert(block_id, bs);

        let mut cell_box = LayoutBox {
            node_id: Some(cell_id),
            content_width: 27.5,
            width: 27.5,
            children: vec![LayoutBox {
                node_id: Some(block_id),
                width: 300.0,
                content_width: 300.0,
                ..Default::default()
            }],
            ..Default::default()
        };

        constrain_table_cell_content_widths(&mut cell_box, &doc, &styles);

        // 直接对 cell 调用（cell 已是 TableCell）
        let block = &cell_box.children[0];
        assert!(
            (block.width - 300.0).abs() < 0.01,
            "显式 width:300px 的 block 不应被 clamp，实际 {}",
            block.width
        );
    }
}
