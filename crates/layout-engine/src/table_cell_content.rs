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

/// R4298（CSS2 §17.5.2 列压缩的组成步骤）：列宽被压缩后（`table::compute_column_widths_inner`
/// 收缩臂 / R4227 约束臂），cell content 仍是压缩前宽度的 taffy 布局——包裹性内容（文本）
/// 须按最终列宽重排：①把 cell 子树中 width:auto 的 block 宽度 clamp 到 cell content width
/// （含 wrapping 内容——R767 的「跳过避 clip」以不重排为前提，此处紧随重测高度，clip 消除）；
/// ②经 `measure_text_content` 按新宽重测各 block 与 cell 的内容高并增高（paint 侧 IFC 以
/// box content_width 重排文本，宽/高就位后文字即按列宽换行）。
/// img_intrinsic_sizes 以空表传入（table 调用链不穿 engine 侧 intrinsic 表；仅影响压缩表内
/// img 的固有比解析，属性/CSS 尺寸路径不受影响）。
/// 须在 `position_cells` 之前调用（行高取 cell 高）。multicol-basic ref 页：
/// width:360 表 3×120 列压缩后，td 文本仍按 280 宽换 3 行溢出窄列，重测后按 120 宽换行。
pub(crate) fn remeasure_cells_at_compressed_widths(
    table_box: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
) {
    fn walk(
        box_node: &mut LayoutBox,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
    ) {
        let is_cell = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
        if is_cell {
            let cw = box_node.content_width;
            if cw > 0.5 {
                // ① 强制 clamp：width:auto 的 block 子（含 wrapping 内容）收到 cell 内容宽。
                // 显式 width 的 block 不 clamp（尊重作者宽度，同 R767 口径）。
                for child in box_node.children.iter_mut() {
                    clamp_cell_subtree_to_content_width_forced(child, cw, styles);
                }
                // ② 重测：直接 block 子先按自身新宽增高，cell 再整体重测增高。
                let vertical_ext =
                    box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                for child in box_node.children.iter_mut() {
                    let Some(cid) = child.node_id else { continue };
                    let Some(cs) = styles.get(&cid) else { continue };
                    if !matches!(
                        cs.display,
                        DisplayValue::Block
                            | DisplayValue::ListItem
                            | DisplayValue::FlowRoot
                            | DisplayValue::Flow
                            | DisplayValue::Inline
                    ) {
                        continue;
                    }
                    let inner_w = child.content_width;
                    if inner_w <= 0.5 {
                        continue;
                    }
                    let measured = crate::inline_finalization::measure_text_content(
                        doc,
                        styles,
                        cid,
                        taffy::geometry::Size {
                            width: Some(inner_w),
                            height: None,
                        },
                        taffy::geometry::Size {
                            width: taffy::style::AvailableSpace::Definite(inner_w),
                            height: taffy::style::AvailableSpace::MaxContent,
                        },
                        &HashMap::new(),
                        inline_fonts,
                        // R4332：table cell 内容非 run-in 后继块，无前缀可并入。
                        None,
                    );
                    let vext = child.padding_top + child.padding_bottom + child.border_top + child.border_bottom;
                    let new_h = measured.height + vext;
                    if new_h > child.height + 0.5 {
                        child.height = new_h;
                        child.content_height = measured.height;
                    }
                }
                let Some(cell_id) = box_node.node_id else { return };
                let measured = crate::inline_finalization::measure_text_content(
                    doc,
                    styles,
                    cell_id,
                    taffy::geometry::Size {
                        width: Some(cw),
                        height: None,
                    },
                    taffy::geometry::Size {
                        width: taffy::style::AvailableSpace::Definite(cw),
                        height: taffy::style::AvailableSpace::MaxContent,
                    },
                    &HashMap::new(),
                    inline_fonts,
                    // R4332：table cell 内容非 run-in 后继块，无前缀可并入。
                    None,
                );
                let new_h = measured.height + vertical_ext;
                if new_h > box_node.height + 0.5 {
                    box_node.height = new_h;
                    box_node.content_height = measured.height;
                }
            }
            return;
        }
        for child in &mut box_node.children {
            walk(child, doc, styles, inline_fonts);
        }
    }
    walk(table_box, doc, styles, inline_fonts);
}

/// `clamp_cell_subtree_to_content_width` 的强制变体：wrapping 内容（max-content 超宽）也
/// clamp（R4298 列压缩重排路径——紧随 measure 增高，无 clip 风险）。
pub(crate) fn clamp_cell_subtree_to_content_width_forced(
    box_node: &mut LayoutBox,
    cell_content_width: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    // R4298：display:inline 一并纳入——Phase A 把 inline 子 block 化建盒（converter
    // `DisplayValue::Inline => taffy Block`），其 LayoutBox 同样被 taffy 以初始宽布局
    // （multicol-basic ref 页 `.multicol-basic-ref-item{display:inline}` div 280 宽实证），
    // 压缩重排路径同样需要 clamp。inline 的 CSS width 不适用（CSS2 §10.3.1 width
    // applies to non-replaced inline ⇒ no）——width 声明按 auto 处理（multicol-basic
    // ref 页 div width:120px 声明仍被 taffy 布局为 280 实证）；替换元素除外（width 适用）。
    let width_ignored = style.is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::Block
                | DisplayValue::ListItem
                | DisplayValue::FlowRoot
                | DisplayValue::Flow
                | DisplayValue::Flex
                | DisplayValue::Grid
                | DisplayValue::Inline
        ) && (matches!(s.width, LengthValue::Auto) || matches!(s.display, DisplayValue::Inline))
    });
    let is_block_auto = width_ignored && !box_node.is_replaced;
    if is_block_auto && box_node.width > cell_content_width + 0.5 {
        box_node.width = cell_content_width;
        box_node.content_width = (cell_content_width
            - box_node.border_left
            - box_node.border_right
            - box_node.padding_left
            - box_node.padding_right)
            .max(0.0);
    }
    for child in &mut box_node.children {
        clamp_cell_subtree_to_content_width_forced(child, cell_content_width, styles);
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
