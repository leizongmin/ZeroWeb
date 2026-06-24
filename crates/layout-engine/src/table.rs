//! CSS Table 布局算法。
//!
//! 由于 taffy 没有原生 table 支持，所有 table display types 在 taffy 中
//! 映射为 `Display::Block`。本模块作为后处理步骤，在 taffy 布局完成后
//! 对 `display: table` 容器内的子元素重新定位，实现 table grid 布局。
//!
//! ## 支持的功能
//!
//! - 基本 table grid 构建（row × column）
//! - Auto table layout（列宽自动分配）
//! - colspan 属性支持（通过 DOM 属性读取）
//! - border-spacing 支持
//! - table-row-group / table-header-group / table-footer-group 支持
//! - 匿名 table box 生成（简化版：直接子 table-cell 包装为匿名行）

// R342c：collapsed-border 解析抽出（2000 行规则），经具体 import 调用。
use crate::table_borders::resolve_collapsed_borders;

use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;

use zero_dom::NodeId;

use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

use crate::types::OverflowClip;

/// 一个表格单元格的信息。
#[derive(Debug, Clone)]
pub(crate) struct TableCell {
    /// 在行 LayoutBox children 中的索引。
    pub(crate) child_index: usize,
    /// colspan 值（默认 1）。
    pub(crate) colspan: usize,
    /// rowspan 值（默认 1）。
    pub(crate) rowspan: usize,
    /// 单元格跨的列范围 [start, end)。
    pub(crate) col_start: usize,
    pub(crate) col_end: usize,
    /// 嵌套行组中的单元格：指向 table_box.children 中的行组索引。
    /// None 表示单元格在 row_box.children[child_index] 中查找（默认）。
    /// Some(rg_idx) 表示单元格在 table_box.children[rg_idx].children[child_index] 中查找。
    /// 用于孤立行组（table_box 本身是行组）中混合嵌套行组和直接子单元格的匿名行。
    pub(crate) parent_rg_idx: Option<usize>,
}

/// 一个表格行的信息。
#[derive(Debug, Clone)]
pub(crate) struct TableRow {
    /// 在 table LayoutBox children 中的索引。
    /// 当行是直接子 table-row 时，这是 table_box.children 中的索引。
    pub(crate) child_index: usize,
    /// 行所在的行组（tbody/thead/tfoot）在 table LayoutBox children 中的索引。
    /// None 表示行是 table 的直接 table-row 子元素。
    pub(crate) row_group_index: Option<usize>,
    /// 行内的单元格列表。
    pub(crate) cells: Vec<TableCell>,
    /// 是否为匿名行（cells 直接在 row-group 中，无包裹 table-row）。
    pub(crate) is_anonymous: bool,
}

/// 解析后的表格网格结构。
#[derive(Debug)]
pub(crate) struct TableGrid {
    /// 行列表。
    pub(crate) rows: Vec<TableRow>,
    /// 总列数。
    pub(crate) col_count: usize,
    /// 每列是否被 visibility:collapse 折叠。
    /// CSS Tables §4.1：visibility:collapse 的列宽度为 0，不参与布局。
    pub(crate) collapsed_cols: Vec<bool>,
}

/// 对 LayoutBox 树执行 table 布局后处理。
///
/// 遍历所有 `display: table` 或 `display: inline-table` 的容器，
/// 将其子元素按 table grid 规则重新定位。
///
/// 同时处理孤立的 table 内部元素（如 `display: table-row-group`、
/// `display: table-row`、`display: table-cell`），这些元素缺少
/// 父级 table 容器，CSS 匿名盒修复会为它们生成匿名 table 包装。
pub fn adjust_table_layout(root: &mut LayoutBox, doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) {
    adjust_table_layout_inner(root, doc, styles, false);
}

fn adjust_table_layout_inner(
    root: &mut LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inside_table: bool,
) {
    let display = get_display(root, styles);

    if display == Some(DisplayValue::Table) || display == Some(DisplayValue::InlineTable) {
        layout_table(root, doc, styles);
        // 递归处理子节点（标记为在 table 内部）
        for child in &mut root.children {
            adjust_table_layout_inner(child, doc, styles, true);
        }
    } else if !inside_table && display.as_ref().is_some_and(is_table_internal) {
        // 孤立的 table 内部元素（无父级 table）：
        // CSS 匿名盒修复应生成匿名 table 包装，这里直接对其执行 table 布局。
        layout_table(root, doc, styles);
        for child in &mut root.children {
            adjust_table_layout_inner(child, doc, styles, true);
        }
    } else {
        for child in &mut root.children {
            adjust_table_layout_inner(child, doc, styles, inside_table);
        }
    }
}

/// 获取 LayoutBox 对应的 display 值。
fn get_display(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> Option<DisplayValue> {
    box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| s.display.clone())
}

/// 判断 display 值是否为某种 table-row 类型。
fn is_table_row(display: &DisplayValue) -> bool {
    matches!(display, DisplayValue::TableRow)
}

/// 判断 display 值是否为 table-cell。
fn is_table_cell(display: &DisplayValue) -> bool {
    matches!(display, DisplayValue::TableCell)
}

/// 判断 display 值是否为行组（tbody/thead/tfoot）。
fn is_row_group(display: &DisplayValue) -> bool {
    matches!(
        display,
        DisplayValue::TableRowGroup | DisplayValue::TableHeaderGroup | DisplayValue::TableFooterGroup
    )
}

/// 判断 display 值是否为 table 内部元素（需要匿名 table 包装）。
pub(crate) fn is_table_internal(display: &DisplayValue) -> bool {
    matches!(
        display,
        DisplayValue::TableRowGroup
            | DisplayValue::TableHeaderGroup
            | DisplayValue::TableFooterGroup
            | DisplayValue::TableRow
            | DisplayValue::TableCell
            | DisplayValue::TableColumn
            | DisplayValue::TableColumnGroup
            | DisplayValue::TableCaption
    )
}

/// 行组的排序优先级。
///
/// CSS 规范要求 thead 在 tbody 之前，tbody 在 tfoot 之前，
/// 无论 DOM 顺序如何。
fn row_group_sort_priority(display: &DisplayValue) -> u8 {
    match display {
        DisplayValue::TableHeaderGroup => 0,
        DisplayValue::TableRowGroup => 1,
        DisplayValue::TableFooterGroup => 2,
        _ => 3,
    }
}

/// 从 DOM 中读取元素的 colspan 属性值。
fn get_colspan(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "colspan")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从 DOM 属性中读取 rowspan 值（默认 1）。
fn get_rowspan(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "rowspan")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从 ComputedStyle 中读取 border-spacing 值。
fn get_border_spacing(style: &ComputedStyle) -> (f32, f32) {
    (style.border_spacing.horizontal, style.border_spacing.vertical)
}

/// 从 ComputedStyle 中读取 position: relative 的 inset 偏移量。
///
/// `horizontal` 为 true 时读取 left，否则读取 top。
fn resolve_length_inset(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, horizontal: bool) -> f32 {
    use zero_css_parser::values::LengthValue;
    let Some(node_id) = box_node.node_id else {
        return 0.0;
    };
    let Some(style) = styles.get(&node_id) else {
        return 0.0;
    };
    let value = if horizontal { &style.left } else { &style.top };
    match value {
        LengthValue::Px(v) => *v as f32,
        _ => 0.0,
    }
}

/// 对单个 table 容器执行布局。
///
/// 算法步骤：
/// 1. 扫描子元素，识别 table-row / table-row-group / table-cell
/// 2. 构建 row × column 网格
/// 3. 计算每列的宽度（auto table layout）
/// 4. 定位每个 cell
fn layout_table(table_box: &mut LayoutBox, doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // 读取 border-spacing
    let (spacing_x, spacing_y) = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .map(get_border_spacing)
        .unwrap_or((0.0, 0.0));

    // 1. 收集行和单元格
    let grid = build_grid(table_box, doc, styles);

    if grid.rows.is_empty() || grid.col_count == 0 {
        // 没有正规表格子元素（无 row/cell/row-group），但可能存在 block 级子元素。
        // CSS Tables §2.4：display:table 容器的 block 子元素应生成匿名 row+cell，
        // 使 table 收缩适应到 block 内容宽度（而非填满容器）。
        // 例如 `<html style="display:table">` 应收缩到 body 内容宽度。
        crate::table_shrink::shrink_table_to_block_content(table_box, styles);
        return;
    }

    // 2. 计算列宽
    let col_widths = compute_column_widths(table_box, &grid, styles, doc);

    // 3. 定位单元格
    position_cells(table_box, &grid, &col_widths, spacing_x, spacing_y, styles);

    // 3.5 收集 <col>/<colgroup> 列背景（CSS Tables §17.5.3）
    //     列背景在单元格之下绘制，须在 paint 前把列元素几何写入 table_col_backgrounds。
    collect_table_col_backgrounds(table_box, &grid, &col_widths, spacing_x, styles, doc);

    // 4. border-collapse: collapse 时解析边框冲突
    //    必须在 suppress 之前，因为 resolve 从 ComputedStyle 读取边框
    resolve_collapsed_borders(table_box, &grid, styles);

    // 5. 抑制行组和行的 border/padding/margin
    //    CSS 2.1 规范：在 separated border model 中，
    //    table-row-group / table-row 的 border、padding、margin 无视觉效果
    suppress_row_group_row_box_model(table_box, styles);
}

/// 抑制行组（tbody/thead/tfoot）和行（tr）的 border/padding/margin。
///
/// CSS 2.1 Section 17.5.3 和 17.5.4：
/// - 在 separated border model 中，table-row-group 和 table-row 的
///   border、padding 和 margin 无视觉效果。
/// - 在 collapsed border model 中，border 参与冲突解决（已由
///   resolve_collapsed_borders 从 ComputedStyle 读取），但渲染由
///   单元格边框绘制覆盖，因此 LayoutBox 上的 border 仍归零。
fn suppress_row_group_row_box_model(table_box: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    for child in &mut table_box.children {
        let display = get_display(child, styles);
        match display {
            Some(DisplayValue::TableRowGroup)
            | Some(DisplayValue::TableHeaderGroup)
            | Some(DisplayValue::TableFooterGroup) => {
                // 行组：border/padding/margin 全部归零
                zero_box_model(child);
                // 递归处理行组内的行
                for row in &mut child.children {
                    let row_display = get_display(row, styles);
                    if row_display == Some(DisplayValue::TableRow) {
                        zero_box_model(row);
                    }
                }
            }
            Some(DisplayValue::TableRow) => {
                // 直接子行：border/padding/margin 归零
                zero_box_model(child);
            }
            _ => {}
        }
    }
}

/// 将 LayoutBox 的 border/padding/margin 全部设为 0。
///
/// CSS 2.1 §17.5.3/17.5.4：行组和行的盒模型属性无视觉效果。
/// converter 层已在 taffy 布局前将这些属性归零，
/// 此函数确保 extract_layout 后的 LayoutBox 也保持一致。
fn zero_box_model(box_node: &mut LayoutBox) {
    box_node.border_top = 0.0;
    box_node.border_right = 0.0;
    box_node.border_bottom = 0.0;
    box_node.border_left = 0.0;
    box_node.padding_top = 0.0;
    box_node.padding_right = 0.0;
    box_node.padding_bottom = 0.0;
    box_node.padding_left = 0.0;
    box_node.margin_top = 0.0;
    box_node.margin_right = 0.0;
    box_node.margin_bottom = 0.0;
    box_node.margin_left = 0.0;
}

/// 获取行 box 的可变引用。
pub(crate) fn get_row_box_mut<'a>(table_box: &'a mut LayoutBox, row: &TableRow) -> Option<&'a mut LayoutBox> {
    match row.row_group_index {
        Some(rg_idx) => {
            let rg = table_box.children.get_mut(rg_idx)?;
            if row.is_anonymous {
                // 匿名行：row-group 即为行盒
                Some(rg)
            } else {
                rg.children.get_mut(row.child_index)
            }
        }
        None => {
            if row.is_anonymous {
                // 孤立匿名行：table_box 本身是行组，行盒就是 table_box
                Some(table_box)
            } else {
                table_box.children.get_mut(row.child_index)
            }
        }
    }
}

/// 从 table 容器的子元素中构建 grid 结构。
///
/// 处理以下结构：
/// - `table > tr > td` — 直接子元素是 table-row
/// - `table > tbody > tr > td` — 直接子元素是 table-row-group
/// - `table > td` — 直接子元素是 table-cell（匿名行生成）
fn build_grid(table_box: &LayoutBox, doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) -> TableGrid {
    let mut rows = Vec::new();
    let mut max_cols = 0usize;

    // 检测孤立行组模式：table_box 本身是行组（无外层 table 容器）
    // 此时 table_box.children 中的嵌套行组和直接子单元格需要特殊处理
    let is_orphan = table_box.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::TableRowGroup | DisplayValue::TableHeaderGroup | DisplayValue::TableFooterGroup
        )
    });

    // 收集行组，按 CSS 规范顺序排列（thead → tbody → tfoot）
    // 先收集所有子元素及其类型，按行组排序优先级重排
    let mut children_with_priority: Vec<(usize, &LayoutBox, Option<DisplayValue>)> = Vec::new();
    for (child_idx, child) in table_box.children.iter().enumerate() {
        let child_display = get_display(child, styles);
        children_with_priority.push((child_idx, child, child_display));
    }

    // 按行组排序优先级稳定排序（thead=0, tbody=1, tfoot=2, 其他=3）
    // 稳定排序保留同优先级内的 DOM 顺序
    children_with_priority.sort_by_key(|(_, _, display)| display.as_ref().map_or(3, row_group_sort_priority));

    let mut orphan_anonymous_cells: Vec<TableCell> = Vec::new();
    let mut orphan_first_child_idx = 0usize;
    let mut orphan_col_cursor = 0usize;

    // 直接 table-cell 子元素（无 table-row 包裹）累加器。
    // CSS §17.2.1：连续的直接 table-cell 子元素应合并到同一个匿名 table-row
    // （水平排列为多列），而非每个 cell 各占一行。cell.child_index 指向
    // table_box.children 中的索引，配合 is_anonymous=true 的匿名行使
    // get_row_box 返回 table_box、get_cell_box 返回 table_box.children[idx]。
    let mut direct_cells: Vec<TableCell> = Vec::new();
    let mut direct_first_child_idx = 0usize;
    let mut direct_col_cursor = 0usize;

    for (child_idx, child, child_display) in &children_with_priority {
        if is_orphan {
            match child_display {
                Some(d) if is_table_cell(d) => {
                    let colspan = get_colspan(child, doc);
                    let rowspan = get_rowspan(child, doc);
                    let col_start = orphan_col_cursor;
                    let col_end = col_start + colspan;
                    max_cols = max_cols.max(col_end);
                    orphan_anonymous_cells.push(TableCell {
                        child_index: *child_idx,
                        colspan,
                        rowspan,
                        col_start,
                        col_end,
                        parent_rg_idx: None,
                    });
                    orphan_col_cursor = col_end;
                    if orphan_anonymous_cells.len() == 1 {
                        orphan_first_child_idx = *child_idx;
                    }
                    continue;
                }
                Some(d) if is_row_group(d) => {
                    // 嵌套行组：CSS 表格匿名盒修复
                    // 嵌套行组的单元格应合并到外层行组的同一匿名行
                    for (rg_child_idx, rg_child) in child.children.iter().enumerate() {
                        let rg_display = get_display(rg_child, styles);
                        if rg_display.as_ref().is_some_and(is_table_cell) {
                            let colspan = get_colspan(rg_child, doc);
                            let rowspan = get_rowspan(rg_child, doc);
                            let col_start = orphan_col_cursor;
                            let col_end = col_start + colspan;
                            max_cols = max_cols.max(col_end);
                            orphan_anonymous_cells.push(TableCell {
                                child_index: rg_child_idx,
                                colspan,
                                rowspan,
                                col_start,
                                col_end,
                                parent_rg_idx: Some(*child_idx),
                            });
                            orphan_col_cursor = col_end;
                            if orphan_anonymous_cells.len() == 1 {
                                orphan_first_child_idx = *child_idx;
                            }
                        } else if rg_display.as_ref().is_some_and(is_table_row) {
                            // 嵌套行组中的行：提取其单元格
                            for (cell_idx, cell_child) in rg_child.children.iter().enumerate() {
                                let cell_colspan = get_colspan(cell_child, doc);
                                let cell_rowspan = get_rowspan(cell_child, doc);
                                let col_start = orphan_col_cursor;
                                let col_end = col_start + cell_colspan;
                                max_cols = max_cols.max(col_end);
                                orphan_anonymous_cells.push(TableCell {
                                    child_index: cell_idx,
                                    colspan: cell_colspan,
                                    rowspan: cell_rowspan,
                                    col_start,
                                    col_end,
                                    parent_rg_idx: Some(*child_idx),
                                });
                                orphan_col_cursor = col_end;
                                if orphan_anonymous_cells.len() == 1 {
                                    orphan_first_child_idx = *child_idx;
                                }
                            }
                        }
                    }
                    continue;
                }
                Some(d) if is_table_row(d) => {
                    if !orphan_anonymous_cells.is_empty() {
                        rows.push(TableRow {
                            child_index: orphan_first_child_idx,
                            row_group_index: None,
                            cells: std::mem::take(&mut orphan_anonymous_cells),
                            is_anonymous: true,
                        });
                        orphan_col_cursor = 0;
                    }
                }
                _ => {
                    if !orphan_anonymous_cells.is_empty() {
                        rows.push(TableRow {
                            child_index: orphan_first_child_idx,
                            row_group_index: None,
                            cells: std::mem::take(&mut orphan_anonymous_cells),
                            is_anonymous: true,
                        });
                        orphan_col_cursor = 0;
                    }
                    continue;
                }
            }
        }

        // 遇到非 table-cell 子元素时，flush 已累积的连续直接 table-cell
        //（CSS §17.2.1：连续 cell 合并为一个匿名行）
        if !direct_cells.is_empty() && !child_display.as_ref().is_some_and(is_table_cell) {
            rows.push(TableRow {
                child_index: direct_first_child_idx,
                row_group_index: None,
                cells: std::mem::take(&mut direct_cells),
                is_anonymous: true,
            });
            direct_col_cursor = 0;
        }

        match child_display {
            Some(d) if is_table_row(d) => {
                // 直接子元素是 table-row
                let row = build_row(*child_idx, child, doc);
                max_cols = max_cols.max(row.cells.last().map(|c| c.col_end).unwrap_or(0));
                if !row.cells.is_empty() {
                    rows.push(TableRow {
                        row_group_index: None,
                        is_anonymous: false,
                        ..row
                    });
                }
            }
            Some(d) if is_row_group(d) => {
                // 直接子元素是 table-row-group (tbody/thead/tfoot)
                // 从 row-group 中提取行，同时处理嵌套 row-group 和直接 cell。
                // CSS 表格匿名盒修复：row-group 中没有包裹行的 cell
                // 应收集到同一个匿名行中（而非每个 cell 一个匿名行）。
                let mut anonymous_cells: Vec<TableCell> = Vec::new();
                let mut anonymous_row_group_idx: Option<usize> = None;
                let mut anonymous_first_child_idx: usize = 0;
                let mut col_cursor = 0usize;

                for (rg_child_idx, rg_child) in child.children.iter().enumerate() {
                    let rg_display = get_display(rg_child, styles);
                    if rg_display.as_ref().is_some_and(is_table_row) {
                        // 先 flush 之前收集的匿名 cell
                        if !anonymous_cells.is_empty() {
                            max_cols = max_cols.max(col_cursor);
                            let rg_idx = if is_orphan {
                                None
                            } else {
                                anonymous_row_group_idx.or(Some(*child_idx))
                            };
                            rows.push(TableRow {
                                child_index: anonymous_first_child_idx,
                                row_group_index: rg_idx,
                                cells: std::mem::take(&mut anonymous_cells),
                                is_anonymous: true,
                            });
                            col_cursor = 0;
                        }
                        let row = build_row(rg_child_idx, rg_child, doc);
                        max_cols = max_cols.max(row.cells.last().map(|c| c.col_end).unwrap_or(0));
                        if !row.cells.is_empty() {
                            rows.push(TableRow {
                                row_group_index: Some(*child_idx),
                                is_anonymous: false,
                                ..row
                            });
                        }
                    } else if rg_display.as_ref().is_some_and(is_row_group) {
                        // 嵌套 row-group：递归提取其行/单元格
                        // CSS 表格匿名盒修复：内部 row-group 的内容视为外部 row-group 的直接内容
                        // 先 flush 之前收集的匿名 cell
                        if !anonymous_cells.is_empty() {
                            max_cols = max_cols.max(col_cursor);
                            let rg_idx = if is_orphan {
                                None
                            } else {
                                anonymous_row_group_idx.or(Some(*child_idx))
                            };
                            rows.push(TableRow {
                                child_index: anonymous_first_child_idx,
                                row_group_index: rg_idx,
                                cells: std::mem::take(&mut anonymous_cells),
                                is_anonymous: true,
                            });
                            col_cursor = 0;
                        }
                        for (nested_idx, nested_child) in rg_child.children.iter().enumerate() {
                            let nested_display = get_display(nested_child, styles);
                            if nested_display.as_ref().is_some_and(is_table_row) {
                                let row = build_row(nested_idx, nested_child, doc);
                                max_cols = max_cols.max(row.cells.last().map(|c| c.col_end).unwrap_or(0));
                                if !row.cells.is_empty() {
                                    rows.push(TableRow {
                                        row_group_index: Some(*child_idx),
                                        ..row
                                    });
                                }
                            } else if nested_display.as_ref().is_some_and(is_table_cell) {
                                // 嵌套 row-group 中的直接 cell：收集到匿名行
                                let cell_colspan = get_colspan(nested_child, doc);
                                let cell_rowspan = get_rowspan(nested_child, doc);
                                let col_start = col_cursor;
                                let col_end = col_start + cell_colspan;
                                max_cols = max_cols.max(col_end);
                                // 孤立模式下嵌套行组的单元格需要通过 parent_rg_idx 定位
                                let parent_rg = if is_orphan { Some(*child_idx) } else { None };
                                anonymous_cells.push(TableCell {
                                    child_index: nested_idx,
                                    colspan: cell_colspan,
                                    rowspan: cell_rowspan,
                                    col_start,
                                    col_end,
                                    parent_rg_idx: parent_rg,
                                });
                                col_cursor = col_end;
                                if anonymous_row_group_idx.is_none() {
                                    anonymous_row_group_idx = Some(*child_idx);
                                    anonymous_first_child_idx = rg_child_idx;
                                }
                            }
                        }
                    } else if rg_display.as_ref().is_some_and(is_table_cell) {
                        // row-group 中的直接 cell（无包裹行）：
                        // 收集到匿名行（所有相邻 cell 合并到同一行）
                        let cell_colspan = get_colspan(rg_child, doc);
                        let cell_rowspan = get_rowspan(rg_child, doc);
                        let col_start = col_cursor;
                        let col_end = col_start + cell_colspan;
                        max_cols = max_cols.max(col_end);
                        anonymous_cells.push(TableCell {
                            child_index: rg_child_idx,
                            colspan: cell_colspan,
                            rowspan: cell_rowspan,
                            col_start,
                            col_end,
                            parent_rg_idx: None,
                        });
                        col_cursor = col_end;
                        if anonymous_row_group_idx.is_none() {
                            anonymous_row_group_idx = Some(*child_idx);
                            anonymous_first_child_idx = rg_child_idx;
                        }
                    }
                }
                // flush 最后的匿名 cell
                if !anonymous_cells.is_empty() {
                    max_cols = max_cols.max(col_cursor);
                    // 孤立模式下：匿名行直接使用 table_box 作为行盒
                    // 混合嵌套行组单元格（parent_rg_idx=Some）和直接子单元格（parent_rg_idx=None）
                    let rg_idx = if is_orphan {
                        None
                    } else {
                        Some(anonymous_row_group_idx.unwrap_or(*child_idx))
                    };
                    rows.push(TableRow {
                        child_index: anonymous_first_child_idx,
                        row_group_index: rg_idx,
                        cells: anonymous_cells,
                        is_anonymous: true,
                    });
                }
            }
            Some(d) if is_table_cell(d) => {
                // 直接子元素是 table-cell — 累加到当前匿名行（连续 cell 合并为一行）
                let colspan = get_colspan(child, doc);
                let rowspan = get_rowspan(child, doc);
                let col_start = direct_col_cursor;
                let col_end = col_start + colspan;
                if direct_cells.is_empty() {
                    direct_first_child_idx = *child_idx;
                }
                direct_cells.push(TableCell {
                    child_index: *child_idx,
                    colspan,
                    rowspan,
                    col_start,
                    col_end,
                    parent_rg_idx: None,
                });
                direct_col_cursor = col_end;
                max_cols = max_cols.max(col_end);
            }
            _ => {
                // 其他类型（caption、column 等）— 跳过
            }
        }
    }

    if is_orphan && !orphan_anonymous_cells.is_empty() {
        rows.push(TableRow {
            child_index: orphan_first_child_idx,
            row_group_index: None,
            cells: orphan_anonymous_cells,
            is_anonymous: true,
        });
    }

    // flush 连续的直接 table-cell 子元素为一个匿名行（CSS §17.2.1）
    if !direct_cells.is_empty() {
        rows.push(TableRow {
            child_index: direct_first_child_idx,
            row_group_index: None,
            cells: std::mem::take(&mut direct_cells),
            is_anonymous: true,
        });
    }

    // CSS Tables §4（dimensioning the row/column grid）：`<col>`/`<colgroup>`
    // 元素定义网格列。列数 = max(单元格导出列数, col 元素导出列数)。
    // 对 separated 和 collapsed border model 均生效——collapsed 模式下列宽
    // 语义（border 中心间距）由 compute_column_widths 的 cell-width-as-content
    // 处理，此处只负责网格列数。
    max_cols = max_cols.max(count_col_elements(table_box, styles, doc));

    // 检测 visibility:collapse 的列
    // CSS Tables §4.1：col/colgroup 上 visibility:collapse 的列宽度为 0
    let collapsed_cols = detect_collapsed_columns(table_box, max_cols, styles, doc);

    TableGrid {
        rows,
        col_count: max_cols,
        collapsed_cols,
    }
}

/// 统计 `<col>`/`<colgroup>` 子元素定义的网格列数。
///
/// CSS Tables §4：colgroup 的 span 属性（默认 1）决定其覆盖的列数；
/// 若 colgroup 内含 `<col>` 子元素，则按内部 col 的 span 之和计算
/// （与 `detect_collapsed_columns` 的 col_cursor 推进逻辑保持一致）。
fn count_col_elements(
    table_box: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> usize {
    let mut total = 0usize;
    for child in &table_box.children {
        let child_display = get_display(child, styles);
        match child_display {
            Some(DisplayValue::TableColumnGroup) => {
                let inner: usize = child
                    .children
                    .iter()
                    .filter(|c| get_display(c, styles) == Some(DisplayValue::TableColumn))
                    .map(|c| get_span(c, doc))
                    .sum();
                if inner > 0 {
                    total += inner;
                } else {
                    total += get_span(child, doc);
                }
            }
            Some(DisplayValue::TableColumn) => {
                total += get_span(child, doc);
            }
            _ => {}
        }
    }
    total
}

/// 检测 table 中被 visibility:collapse 折叠的列。
///
/// 遍历 table 的 `<col>` 和 `<colgroup>` 子元素，
/// 检查其 computed style 的 visibility 属性是否为 Collapse。
/// colgroup 的 span 属性决定其覆盖的列数。
fn detect_collapsed_columns(
    table_box: &LayoutBox,
    col_count: usize,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> Vec<bool> {
    use zero_css_parser::values::VisibilityValue;

    let mut collapsed = vec![false; col_count];
    let mut col_cursor = 0usize;

    for child in &table_box.children {
        let child_display = get_display(child, styles);
        match child_display {
            Some(DisplayValue::TableColumnGroup) => {
                // colgroup：检查其 visibility，以及内部 col 的 visibility
                let rg_span = get_span(child, doc);
                let rg_vis = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .map(|s| s.visibility.clone());

                if rg_vis == Some(VisibilityValue::Collapse) {
                    // 整个 colgroup 折叠
                    for i in 0..rg_span {
                        let col_idx = col_cursor + i;
                        if col_idx < col_count {
                            collapsed[col_idx] = true;
                        }
                    }
                } else {
                    // colgroup 未折叠，检查内部 col
                    for col_child in &child.children {
                        let col_display = get_display(col_child, styles);
                        if col_display == Some(DisplayValue::TableColumn) {
                            let col_span = get_span(col_child, doc);
                            let col_vis = col_child
                                .node_id
                                .and_then(|id| styles.get(&id))
                                .map(|s| s.visibility.clone());
                            if col_vis == Some(VisibilityValue::Collapse) {
                                for i in 0..col_span {
                                    let col_idx = col_cursor + i;
                                    if col_idx < col_count {
                                        collapsed[col_idx] = true;
                                    }
                                }
                            }
                            col_cursor += col_span;
                        }
                    }
                    continue; // colgroup 内部已推进 col_cursor
                }
                col_cursor += rg_span;
            }
            Some(DisplayValue::TableColumn) => {
                let col_span = get_span(child, doc);
                let col_vis = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .map(|s| s.visibility.clone());
                if col_vis == Some(VisibilityValue::Collapse) {
                    for i in 0..col_span {
                        let col_idx = col_cursor + i;
                        if col_idx < col_count {
                            collapsed[col_idx] = true;
                        }
                    }
                }
                col_cursor += col_span;
            }
            _ => {}
        }
    }

    let collapsed_indices: Vec<usize> = collapsed
        .iter()
        .enumerate()
        .filter(|(_, c)| **c)
        .map(|(i, _)| i)
        .collect();
    if !collapsed_indices.is_empty() {
        tracing::debug!(
            "detect_collapsed_columns: col_count={}, collapsed={:?}",
            col_count,
            collapsed_indices
        );
    }
    collapsed
}

/// 收集 `<col>`/`<colgroup>` 的列背景几何，写入 `table_box.table_col_backgrounds`。
///
/// CSS Tables §17.5.3：`<col>`/`<colgroup>` 不生成常规流盒，其 `background-color`
/// 须由表格绘制算法在单元格背景**之下**、按列跨满表格高度绘制。本函数在 position_cells
/// 后运行（col_widths + col→column 映射已知），为每个有非透明背景、非全折叠的列元素
/// 记录 `(node_id, x_offset, width)`（相对表格 content box）。
///
/// 列几何（含 border-spacing）与 position_cells 的单元格定位保持一致：相邻非折叠列
/// 之间加 spacing_x，折叠列宽度 0 且不引入 spacing。colgroup 在前（下层）、col 在后
/// （上层），匹配 CSS 列背景堆叠顺序。
fn collect_table_col_backgrounds(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    col_widths: &[f32],
    spacing_x: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) {
    use zero_style_system::ColorValue;

    // 预计算每列 (left, width) —— 镜像 position_cells 的 spacing 累积
    let col_count = grid.col_count;
    let mut col_geo: Vec<(f32, f32)> = vec![(0.0, 0.0); col_count];
    {
        let mut x = 0.0f32;
        let mut nc_count = 0usize; // 已放置的非折叠列数
        for (i, geo) in col_geo.iter_mut().enumerate() {
            let collapsed = grid.collapsed_cols.get(i).copied().unwrap_or(false);
            let w = col_widths.get(i).copied().unwrap_or(0.0);
            if collapsed {
                *geo = (x, 0.0);
            } else {
                if nc_count > 0 {
                    x += spacing_x;
                }
                *geo = (x, w);
                x += w;
                nc_count += 1;
            }
        }
    }

    /// 计算列元素 [col_start, col_end) 跨度的 (left, width)；全折叠返回 None。
    fn span_rect(col_geo: &[(f32, f32)], grid: &TableGrid, col_start: usize, col_end: usize) -> Option<(f32, f32)> {
        let end = col_end.min(col_geo.len());
        if col_start >= end {
            return None;
        }
        let left = col_geo[col_start].0;
        let last = end - 1;
        let right = col_geo[last].0 + col_geo[last].1;
        if right <= left {
            return None;
        }
        // 全折叠（宽度 0）跳过
        let any_visible = (col_start..end).any(|i| !grid.collapsed_cols.get(i).copied().unwrap_or(false));
        if !any_visible {
            return None;
        }
        Some((left, right - left))
    }

    let mut entries: Vec<(NodeId, f32, f32)> = Vec::new();
    let mut col_cursor = 0usize;

    // Pass 1：colgroup（下层先入）
    for child in &table_box.children {
        let child_display = get_display(child, styles);
        if child_display != Some(DisplayValue::TableColumnGroup) {
            continue;
        }
        let rg_span = get_span(child, doc);
        let col_start = col_cursor.min(col_count);
        let col_end = (col_cursor + rg_span).min(col_count);
        let rg_collapsed = child
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.visibility, zero_css_parser::values::VisibilityValue::Collapse));
        let rg_has_bg = child
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| !matches!(s.background_color, ColorValue::Transparent));
        if !rg_collapsed && rg_has_bg {
            if let Some((l, w)) = span_rect(&col_geo, grid, col_start, col_end) {
                if let Some(nid) = child.node_id {
                    entries.push((nid, l, w));
                }
            }
        }
        // col_cursor 推进：colgroup 内含 col 时按内部 col 推进，否则按 rg_span
        let has_inner_cols = child
            .children
            .iter()
            .any(|c| get_display(c, styles) == Some(DisplayValue::TableColumn));
        if has_inner_cols {
            for col_child in &child.children {
                if get_display(col_child, styles) == Some(DisplayValue::TableColumn) {
                    col_cursor += get_span(col_child, doc);
                }
            }
        } else {
            col_cursor += rg_span;
        }
    }

    // Pass 2：col（上层后入）—— 顶层 col + colgroup 内部 col
    let mut col_cursor = 0usize;
    for child in &table_box.children {
        let child_display = get_display(child, styles);
        match child_display {
            Some(DisplayValue::TableColumnGroup) => {
                let rg_span = get_span(child, doc);
                let rg_collapsed = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .is_some_and(|s| matches!(s.visibility, zero_css_parser::values::VisibilityValue::Collapse));
                if rg_collapsed {
                    col_cursor += rg_span;
                    continue;
                }
                let has_inner_cols = child
                    .children
                    .iter()
                    .any(|c| get_display(c, styles) == Some(DisplayValue::TableColumn));
                if has_inner_cols {
                    for col_child in &child.children {
                        if get_display(col_child, styles) == Some(DisplayValue::TableColumn) {
                            let col_span = get_span(col_child, doc);
                            let col_start = col_cursor.min(col_count);
                            let col_end = (col_cursor + col_span).min(col_count);
                            let col_collapsed = col_child.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                                matches!(s.visibility, zero_css_parser::values::VisibilityValue::Collapse)
                            });
                            let col_has_bg = col_child
                                .node_id
                                .and_then(|id| styles.get(&id))
                                .is_some_and(|s| !matches!(s.background_color, ColorValue::Transparent));
                            if !col_collapsed && col_has_bg {
                                if let Some((l, w)) = span_rect(&col_geo, grid, col_start, col_end) {
                                    if let Some(nid) = col_child.node_id {
                                        entries.push((nid, l, w));
                                    }
                                }
                            }
                            col_cursor += col_span;
                        }
                    }
                } else {
                    col_cursor += rg_span;
                }
            }
            Some(DisplayValue::TableColumn) => {
                let col_span = get_span(child, doc);
                let col_start = col_cursor.min(col_count);
                let col_end = (col_cursor + col_span).min(col_count);
                let col_collapsed = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .is_some_and(|s| matches!(s.visibility, zero_css_parser::values::VisibilityValue::Collapse));
                let col_has_bg = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .is_some_and(|s| !matches!(s.background_color, ColorValue::Transparent));
                if !col_collapsed && col_has_bg {
                    if let Some((l, w)) = span_rect(&col_geo, grid, col_start, col_end) {
                        if let Some(nid) = child.node_id {
                            entries.push((nid, l, w));
                        }
                    }
                }
                col_cursor += col_span;
            }
            _ => {}
        }
    }

    table_box.table_col_backgrounds = entries;
}

/// 从 DOM 中读取元素的 span 属性值（用于 col/colgroup）。
fn get_span(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    if let Some(node_id) = box_node.node_id {
        doc.get_attribute(node_id, "span")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1)
    } else {
        1
    }
}

/// 从一个 table-row 子元素构建 TableRow。
fn build_row(child_idx: usize, row_box: &LayoutBox, doc: &zero_dom::Document) -> TableRow {
    let mut cells = Vec::new();
    let mut col_cursor = 0usize;

    for (cell_idx, cell_child) in row_box.children.iter().enumerate() {
        let colspan = get_colspan(cell_child, doc);
        let rowspan = get_rowspan(cell_child, doc);
        let col_start = col_cursor;
        let col_end = col_start + colspan;
        cells.push(TableCell {
            child_index: cell_idx,
            colspan,
            rowspan,
            col_start,
            col_end,
            parent_rg_idx: None,
        });
        col_cursor = col_end;
    }

    TableRow {
        child_index: child_idx,
        row_group_index: None, // 由调用方设置
        cells,
        is_anonymous: false,
    }
}

/// Auto table layout：根据单元格内容计算每列的宽度。
///
/// 算法：
/// 1. 扫描所有单元格，记录每列的最大内容宽度
/// 2. 如果所有列宽之和小于容器宽度，按比例分配剩余空间
/// 3. 如果所有列宽之和大于容器宽度，保持内容宽度不变
fn compute_column_widths(
    table_box: &LayoutBox,
    grid: &TableGrid,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> Vec<f32> {
    let available_width = table_box.content_width;
    let col_count = grid.col_count;

    if col_count == 0 {
        return Vec::new();
    }

    // border-collapse 模式标志（cell-width-as-content 与 fixed 空列裁剪共用）。
    let is_collapsed_border = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.border_collapse, zero_style_system::BorderCollapseValue::Collapse));
    // 收集每列的最大宽度（两遍算法）
    // CSS Tables §17.5.2.2：列宽首先由非跨列单元格决定（含显式 width），
    // 跨列单元格只把宽度分配给尚未被非跨列单元格约束的列，
    // 这样显式列宽不会被跨列单元格的长内容撑开。
    let mut col_max_widths = vec![0.0f32; col_count];
    // R364：记录哪些列含「显式 width 属性」单元格（值非 auto，含 0/2px 等小于 min-content 的值）。
    // 扩展填满容器时这些列冻结（不吸收剩余空间），仅 auto 列吸收——chromium auto 表行为
    //（table-cell-width-0：width:0/2px/20px 列保持其宽，width:auto 的 .normal 列填满剩余）。
    let mut col_explicit = vec![false; col_count];

    // 辅助闭包：计算单元格对其所在列的宽度贡献
    let cell_used_width = |cell_box: &LayoutBox| -> (f32, bool) {
        let cell_style_width = cell_box.node_id.and_then(|id| styles.get(&id)).map(|s| s.width.clone());
        let css_width_auto = match &cell_style_width {
            Some(zero_css_parser::values::LengthValue::Px(v)) => (*v as f32) < 2.0,
            None => true,
            Some(zero_css_parser::values::LengthValue::Auto) => true,
            Some(_) => false,
        };
        let intrinsic = compute_cell_intrinsic_width(cell_box, styles, doc);
        // auto 宽度的单元格：列宽只取内容固有宽度（intrinsic）。
        // taffy 把单元格当 block，cell_box.width = 行/表全宽，不能作为列宽下限
        //（否则每列都撑到全宽，列总和溢出表宽）。无论 table 本身 width 是否 auto，
        // auto 单元格都不应用 cell_box.width 作为下限。
        let w = if css_width_auto || cell_box.width < 2.0 {
            intrinsic
        } else {
            // 显式 width 单元格。border-collapse 模式下 td 的 width 是 content-box，
            // 而列宽是 border 中心间距语义 → 列宽 = content + 水平 borders 的一半
            // （CSS2 §17.6.2：列宽从左 border 中心到右 border 中心）。
            // 这样 `<col width:50px>` 与 `<td style="width:40px>`（border 10px）产生
            // 相同的列宽 50，使 colspan 类用例 test/ref 一致。
            let explicit = match &cell_style_width {
                Some(zero_css_parser::values::LengthValue::Px(v)) => *v as f32,
                _ => cell_box.width,
            };
            let base = if is_collapsed_border {
                explicit + (cell_box.border_left + cell_box.border_right) / 2.0
            } else {
                cell_box.width
            };
            // R364b：显式 width 不小于单元格 min-content（CSS 表格：列宽下限 = 内容
            // min-content；width:2px 但内容 "1" 需 9.6px → 列宽 9.6，内容不溢出列）。
            base.max(intrinsic)
        };
        // R583：cell min-width/max-width 约束其对列的宽度贡献（CSS Tables §17.5.3：
        // 单元格 min-width 贡献列 min-content 下限；§10 max-width 上限，min 优先于 max）。
        // empty cell + min-width:1in → 96px 列（min-width-applies-to-007）。
        // gated on Px（同 resolve_col_min/max）→ 仅影响显式声明 min/max-width 的单元格。
        let cs = cell_box.node_id.and_then(|id| styles.get(&id));
        let cell_max = cs.and_then(|s| match &s.max_width {
            zero_css_parser::values::LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        });
        let cell_min = cs.and_then(|s| match &s.min_width {
            zero_css_parser::values::LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        });
        let w = match cell_max {
            Some(mx) => w.min(mx),
            None => w,
        };
        let w = w.max(cell_min.unwrap_or(0.0));
        (w, css_width_auto)
    };

    // Pass 0：`<col>`/`<colgroup>` 的显式 width 设置列宽（CSS Tables §17.5.2.1/§17.5.2.2）。
    // 对 separated 和 collapsed border model 均生效。colgroup width 作用于其覆盖
    // 的全部列（无内部 col 时）。
    let resolve_col_width = |s: &ComputedStyle| -> Option<f32> {
        use zero_css_parser::values::LengthValue;
        match &s.width {
            // 仅读取绝对像素宽度。百分比在 width:auto（shrink-to-fit）表上解析语义
            // 不明确（参照盒不定），calc/em 等同理——跳过以保持当前同源匹配。
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    };
    // col/colgroup 的 min-width/max-width 约束列宽下限/上限（CSS Tables §17.5 + CSS §10）。
    // 仅读 Px（同 width），gated on 属性设置 → 仅影响显式声明 min/max-width 的列。
    let resolve_col_min = |s: &ComputedStyle| -> Option<f32> {
        use zero_css_parser::values::LengthValue;
        match &s.min_width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    };
    let resolve_col_max = |s: &ComputedStyle| -> Option<f32> {
        use zero_css_parser::values::LengthValue;
        match &s.max_width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        }
    };
    let mut col_cursor = 0usize;
    for child in &table_box.children {
        let child_display = get_display(child, styles);
        match child_display {
            Some(DisplayValue::TableColumnGroup) => {
                let inner_cols: Vec<&LayoutBox> = child
                    .children
                    .iter()
                    .filter(|c| get_display(c, styles) == Some(DisplayValue::TableColumn))
                    .collect();
                if inner_cols.is_empty() {
                    // colgroup span 覆盖的列共用 colgroup width/min-width/max-width
                    let span = get_span(child, doc);
                    let gs = child.node_id.and_then(|id| styles.get(&id));
                    let gw = gs.and_then(resolve_col_width);
                    let gmin = gs.and_then(resolve_col_min);
                    let gmax = gs.and_then(resolve_col_max);
                    for i in 0..span {
                        let idx = col_cursor + i;
                        if idx < col_count {
                            if let Some(w) = gw {
                                col_max_widths[idx] = col_max_widths[idx].max(w);
                            }
                            if let Some(mn) = gmin {
                                col_max_widths[idx] = col_max_widths[idx].max(mn);
                            }
                            if let Some(mx) = gmax {
                                col_max_widths[idx] = col_max_widths[idx].min(mx);
                            }
                        }
                    }
                    col_cursor += span;
                } else {
                    for col_child in inner_cols {
                        let span = get_span(col_child, doc);
                        let cs = col_child.node_id.and_then(|id| styles.get(&id));
                        let cw = cs.and_then(resolve_col_width);
                        let cmin = cs.and_then(resolve_col_min);
                        let cmax = cs.and_then(resolve_col_max);
                        for i in 0..span {
                            let idx = col_cursor + i;
                            if idx < col_count {
                                if let Some(w) = cw {
                                    col_max_widths[idx] = col_max_widths[idx].max(w);
                                }
                                if let Some(mn) = cmin {
                                    col_max_widths[idx] = col_max_widths[idx].max(mn);
                                }
                                if let Some(mx) = cmax {
                                    col_max_widths[idx] = col_max_widths[idx].min(mx);
                                }
                            }
                        }
                        col_cursor += span;
                    }
                }
            }
            Some(DisplayValue::TableColumn) => {
                let span = get_span(child, doc);
                let cs = child.node_id.and_then(|id| styles.get(&id));
                let cw = cs.and_then(resolve_col_width);
                let cmin = cs.and_then(resolve_col_min);
                let cmax = cs.and_then(resolve_col_max);
                for i in 0..span {
                    let idx = col_cursor + i;
                    if idx < col_count {
                        if let Some(w) = cw {
                            col_max_widths[idx] = col_max_widths[idx].max(w);
                        }
                        if let Some(mn) = cmin {
                            col_max_widths[idx] = col_max_widths[idx].max(mn);
                        }
                        if let Some(mx) = cmax {
                            col_max_widths[idx] = col_max_widths[idx].min(mx);
                        }
                    }
                }
                col_cursor += span;
            }
            _ => {}
        }
    }

    // Pass 1：非跨列单元格设置列宽
    for row in &grid.rows {
        let Some(row_box) = get_row_box(table_box, row) else {
            continue;
        };
        for cell in &row.cells {
            if cell.colspan != 1 {
                continue;
            }
            let Some(cell_box) = get_cell_box(row_box, cell) else {
                continue;
            };
            if cell.col_start >= col_count || grid.collapsed_cols.get(cell.col_start).copied().unwrap_or(false) {
                continue;
            }
            let (w, _) = cell_used_width(cell_box);
            col_max_widths[cell.col_start] = col_max_widths[cell.col_start].max(w);
            // R364：记录该列是否有显式 width 属性（用于扩展填满时冻结）。
            if cell_box
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| !matches!(s.width, zero_css_parser::values::LengthValue::Auto))
            {
                col_explicit[cell.col_start] = true;
            }
        }
    }

    // Pass 2：跨列单元格把宽度分配给 span 内**未被 Pass 1 约束**的非折叠列
    for row in &grid.rows {
        let Some(row_box) = get_row_box(table_box, row) else {
            continue;
        };
        for cell in &row.cells {
            if cell.colspan <= 1 {
                continue;
            }
            let Some(cell_box) = get_cell_box(row_box, cell) else {
                continue;
            };
            let (w, _) = cell_used_width(cell_box);
            // span 内被 Pass 1 约束的列数（这些列已有显式/固有宽度，不被撑开）
            let constrained_in_span = (cell.col_start..cell.col_end.min(col_count))
                .filter(|&c| grid.collapsed_cols.get(c).copied().unwrap_or(false) || col_max_widths[c] > 0.0)
                .count();
            let unconstrained_in_span =
                (cell.col_end.min(col_count).saturating_sub(cell.col_start)).saturating_sub(constrained_in_span);
            if unconstrained_in_span == 0 {
                continue;
            }
            let per_col = w / unconstrained_in_span as f32;
            for (i, col_w) in col_max_widths
                .iter_mut()
                .enumerate()
                .take(cell.col_end.min(col_count))
                .skip(cell.col_start)
            {
                if !grid.collapsed_cols.get(i).copied().unwrap_or(false) && *col_w <= 0.0 {
                    *col_w = (*col_w).max(per_col);
                }
            }
        }
    }

    let table_style = table_box.node_id.and_then(|id| styles.get(&id));
    let has_explicit_width = table_style.as_ref().is_some_and(|s| {
        use zero_css_parser::values::LengthValue;
        !matches!(s.width, LengthValue::Auto)
    });
    let is_fixed_layout = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.table_layout, zero_style_system::TableLayoutValue::Fixed));
    // CSS Tables §17.5.2.1：table-layout:fixed 时表格宽度由 width 属性决定，列宽来自
    // <col>/首行而非内容。若内容列宽和 > 显式 width，应收缩列到 width（内容溢出 cell，
    // 由 cell 的 overflow 裁剪），而非让内容撑宽表格（当前 bug：fixed 表渲染成内容宽）。
    let fixed_explicit_px = if is_fixed_layout {
        table_style.as_ref().and_then(|s| {
            use zero_css_parser::values::LengthValue;
            if let LengthValue::Px(v) = s.width {
                Some(v as f32)
            } else {
                None
            }
        })
    } else {
        None
    };

    // CSS Tables §17.5.2.1 fixed 布局空列裁剪：table-layout:fixed 时，无任何单元格
    // 跨越的列不参与渲染（chromium 行为：colspan 用例收缩到 cell extent）。
    // auto 布局保留空列（col-definite-size 等用例保留 `<col>` 定义的空列宽度）。
    // 仅 fixed 布局裁剪 → 爆炸半径限于 colspan 类用例（auto 用例不受影响）。
    if is_fixed_layout {
        let mut cols_with_cells = vec![false; col_count];
        for row in &grid.rows {
            for cell in &row.cells {
                for slot in cols_with_cells
                    .iter_mut()
                    .take(cell.col_end.min(col_count))
                    .skip(cell.col_start)
                {
                    *slot = true;
                }
            }
        }
        for (i, w) in col_max_widths.iter_mut().enumerate() {
            if !cols_with_cells[i] {
                *w = 0.0;
            }
        }
    }

    // 计算总宽度
    let total_width: f32 = col_max_widths.iter().sum();

    // table-layout:fixed + 显式 width：内容列宽和超出 width 时按比例收缩到 width
    //（CSS Tables §17.5.2.1：fixed 布局列宽不由内容决定；内容溢出由 cell overflow 处理）。
    let mut fixed_capped = false;
    if let Some(ew) = fixed_explicit_px
        && ew > 0.0
        && total_width > ew
    {
        let ratio = ew / total_width;
        for w in &mut col_max_widths {
            *w *= ratio;
        }
        fixed_capped = true;
    }

    // CSS 表格收缩适应（shrink-to-fit）：
    // 表格仅在 width 为明确值（Px/% 等）时扩展填满容器。
    // width:auto 的表格（无论 table-layout 是否 fixed）都应收缩到列宽之和，
    // 而非填满容器——CSS Tables §17.5.2.1：fixed 布局下表格宽度 = max(width 属性, 列宽之和)，
    // width:auto 即取列宽之和。table-layout:fixed 仅决定列宽来源（<col>/首行），
    // 不意味着填满容器。（e）扩展条件已从 `has_explicit_width || is_fixed_layout`
    // 收紧为 `has_explicit_width`——fixed 空列裁剪保证 colspan 等用例 test==ref。
    // 仅当 fixed 布局被上面收缩到 width（fixed_capped）时跳过填满扩展——否则会把
    // 收缩后的列再撑回内容宽；未收缩的 fixed 表（内容 fits width）仍正常扩展填满。

    if has_explicit_width && !fixed_capped && total_width < available_width && total_width > 0.0 {
        // R364：显式 width 列冻结（保持其宽），仅 auto 列吸收剩余空间。CSS Tables auto 布局：
        // 显式 width 单元格的列不增长，剩余空间分给 auto 列（按其当前宽度比例）。全部列均显式
        // width 时回退按比例扩展（避免剩余空间留白）。
        let extra = available_width - total_width;
        let auto_idx: Vec<usize> = (0..col_count)
            .filter(|&i| {
                col_max_widths[i] > 0.0 && !col_explicit[i] && !grid.collapsed_cols.get(i).copied().unwrap_or(false)
            })
            .collect();
        if auto_idx.is_empty() {
            let ratio = available_width / total_width;
            for w in &mut col_max_widths {
                *w *= ratio;
            }
        } else {
            let auto_total: f32 = auto_idx.iter().map(|&i| col_max_widths[i]).sum();
            if auto_total > 0.0 {
                for &i in &auto_idx {
                    col_max_widths[i] += extra * (col_max_widths[i] / auto_total);
                }
            } else {
                let per = extra / auto_idx.len() as f32;
                for &i in &auto_idx {
                    col_max_widths[i] = per;
                }
            }
        }
    }

    // visibility:collapse 的列宽度为 0
    // CSS Tables §4.1：折叠列不参与布局，其宽度视为 0
    for (i, w) in col_max_widths.iter_mut().enumerate() {
        if grid.collapsed_cols.get(i).copied().unwrap_or(false) {
            *w = 0.0;
        }
    }

    // R584：table 元素 min-width 下限（CSS Tables §17.5.2 + §10 min-width）。
    // apply_table_size_conditions 已把 table box 宽度 floor 到 min-width，但列宽未同步
    // → 单元格仍按原列宽渲染。min-width-applies-to-013（empty cell + table min-width:1in）
    // 无 96px 黑方。此处当列宽总和 < min-width 时把列按比例放大到 min-width
    // （box-sizing 一致于 apply_table_size_conditions）。仅当所有非折叠列均为 auto（无显式
    // width）时整体放大，避免触碰有显式 width 列的 auto-column 分布语义（须单独按 auto 列分配）。
    let table_min_content = table_style.as_ref().and_then(|s| {
        use zero_css_parser::values::LengthValue;
        if let LengthValue::Px(v) = &s.min_width {
            let mw = *v as f32;
            if matches!(s.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox) {
                let pb =
                    table_box.padding_left + table_box.padding_right + table_box.border_left + table_box.border_right;
                Some((mw - pb).max(0.0))
            } else {
                Some(mw)
            }
        } else {
            None
        }
    });
    if let Some(min_cw) = table_min_content {
        let live: Vec<usize> = (0..col_count)
            .filter(|&i| !grid.collapsed_cols.get(i).copied().unwrap_or(false))
            .collect();
        let cur_total: f32 = live.iter().map(|&i| col_max_widths[i]).sum();
        // 仅当所有非折叠列均为 auto（无显式 width）时整体按比例放大到 min-width，
        // 避免触碰有显式 width 列的 auto-column 分布语义（那须单独按 auto 列分配）。
        let all_auto = live.iter().all(|&i| !col_explicit[i]);
        if min_cw > cur_total && cur_total > 0.0 && all_auto && !live.is_empty() {
            let ratio = min_cw / cur_total;
            for &i in &live {
                col_max_widths[i] *= ratio;
            }
        }
    }

    col_max_widths
}

/// 获取行盒 — 处理直接 table-row、row-group 内的行和匿名行三种情况。
///
/// 当 `row.row_group_index` 为 Some 时，行在 row-group 内。
/// 当 `row.is_anonymous` 为 true 时，行是匿名行，行盒为 row-group 本身。
/// 当为 None 时，行是 table 的直接 children[row.child_index]。
pub(crate) fn get_row_box<'a>(table_box: &'a LayoutBox, row: &TableRow) -> Option<&'a LayoutBox> {
    match row.row_group_index {
        Some(rg_idx) => {
            let row_group = table_box.children.get(rg_idx)?;
            if row.is_anonymous {
                // 匿名行：cells 是 row-group 的直接子元素，row-group 即为行盒
                Some(row_group)
            } else {
                row_group.children.get(row.child_index)
            }
        }
        None => {
            if row.is_anonymous {
                // 孤立匿名行：table_box 本身是行组，行盒就是 table_box
                Some(table_box)
            } else {
                // 直接 table-row：table_box.children[row.child_index]
                table_box.children.get(row.child_index)
            }
        }
    }
}

/// 获取单元格盒。
/// 获取单元格盒（不可变引用）。
///
/// 根据 cell.parent_rg_idx 决定查找路径：
/// - None: 直接在 row_box.children 中查找
/// - Some(rg_idx): 在 row_box.children[rg_idx].children 中查找（嵌套行组场景）
pub(crate) fn get_cell_box<'a>(row_box: &'a LayoutBox, cell: &TableCell) -> Option<&'a LayoutBox> {
    if let Some(rg_idx) = cell.parent_rg_idx {
        row_box
            .children
            .get(rg_idx)
            .and_then(|rg| rg.children.get(cell.child_index))
    } else {
        row_box.children.get(cell.child_index)
    }
}

/// 估算单元格的固有内容宽度。
///
/// 当 CSS width:0 被应用时，taffy 会将单元格布局为 0 宽度。
/// 但 CSS 表格规范要求 width:0 解析为 min-content 宽度。
/// 计算单元格的最小内容宽度。
///
/// 策略：
/// 1. 检查子元素是否有显式 CSS width → 使用这些宽度之和
/// 2. 如果 cell_box.width 接近 0（taffy 将子元素也约束为 0），
///    从 DOM 文本内容和字体大小估算 min-content 宽度
/// 3. 否则用字体大小估算单字符宽度作为最小宽度
fn compute_cell_intrinsic_width(
    cell_box: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) -> f32 {
    let padding = cell_box.padding_left + cell_box.padding_right;
    let is_zero_width = cell_box.width < 2.0;

    // 尝试从子元素计算内容宽度
    let mut content_width = 0.0f32;
    let mut has_explicit_child = false;

    for child in &cell_box.children {
        // 检查子元素是否有显式 CSS width
        let child_has_explicit_width = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                use zero_css_parser::values::LengthValue;
                !matches!(s.width, LengthValue::Auto)
            })
            .unwrap_or(false);

        if child_has_explicit_width {
            // 有显式 width 的子元素：使用其 outer width
            content_width = content_width.max(child.width + child.margin_left + child.margin_right);
            has_explicit_child = true;
        } else if child.width > 0.0 && (!is_zero_width && child.width < cell_box.width * 0.95) {
            // 非 0 宽度单元格：子元素宽度远小于 cell 宽度时使用
            content_width = content_width.max(child.width + child.margin_left + child.margin_right);
            has_explicit_child = true;
        }
    }

    if has_explicit_child && content_width > 0.0 {
        return content_width + padding;
    }

    // 当 cell_box.width 接近 0 时，taffy 将所有子元素也约束为 0，
    // 无法从 layout 结果获取真实内容宽度。
    // 从 DOM 文本内容估算 min-content 宽度。
    let (font_size, is_ahem) = cell_box
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| {
            use zero_css_parser::values::LengthValue;
            let fs = match &s.font_size {
                LengthValue::Px(v) => *v as f32,
                LengthValue::Em(v) => *v as f32,
                LengthValue::Rem(v) => *v as f32,
                _ => 16.0,
            };
            let ahem = s.font_family.contains(&"Ahem".to_string());
            (fs, ahem)
        })
        .unwrap_or((16.0, false));

    let char_width = if is_ahem { font_size } else { font_size * 0.6 };

    // 收集单元格内的文本内容长度
    let text_len = collect_text_length(cell_box, doc);
    if text_len > 0 {
        return char_width * text_len as f32 + padding;
    }

    char_width + padding
}

/// 递归收集 LayoutBox 子树中的文本字符数。
/// 使用 DOM text_content() 方法获取元素的完整文本内容。
/// 注意：使用 .chars().count() 计算字符数而非 .len()（字节数），
/// 因为多字节 Unicode 字符（如 CJK）的字节数不等于字符数。
fn collect_text_length(box_node: &LayoutBox, doc: &zero_dom::Document) -> usize {
    let mut len = 0;
    if let Some(node_id) = box_node.node_id {
        if let Some(text) = doc.text_content(node_id) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                len += trimmed.chars().count();
            }
        }
    }
    len
}

/// 根据 grid 结构和列宽定位每个单元格。
fn position_cells(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    col_widths: &[f32],
    spacing_x: f32,
    spacing_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let table_content_width = table_box.content_width;

    // R89：表格行高分配（CSS 2.1 §17.5.3 — table 的 height 作为最小高度，
    // 额外高度按行均分到各行，使单元格增长、vertical-align 把内容压到分配后位置）。
    // 预计算每行内容高度，再根据 table 指定 height 计算每行的额外分配量。
    let row_extras: Vec<f32> = {
        let content_row_heights: Vec<f32> = grid
            .rows
            .iter()
            .map(|row| {
                let mut h = 0.0f32;
                if let Some(rb) = get_row_box(table_box, row) {
                    for cell in &row.cells {
                        let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                            rb.children.get(rg_idx).and_then(|rg| rg.children.get(cell.child_index))
                        } else {
                            rb.children.get(cell.child_index)
                        };
                        if let Some(cb) = cell_box {
                            h = h.max(cb.height);
                        }
                    }
                }
                if h == 0.0 { 20.0 } else { h }
            })
            .collect();
        let num_rows = content_row_heights.len();
        let target_content_h = table_box
            .node_id
            .and_then(|id| styles.get(&id))
            .and_then(|s| match &s.height {
                zero_css_parser::values::LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            })
            .map(|h| {
                let pb =
                    table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;
                (h - pb).max(0.0)
            });
        match (target_content_h, num_rows > 0) {
            (Some(target), true) => {
                let content_total: f32 = content_row_heights.iter().sum::<f32>() + (num_rows - 1) as f32 * spacing_y;
                let extra = (target - content_total).max(0.0);
                vec![extra / num_rows as f32; num_rows]
            }
            _ => vec![0.0; num_rows],
        }
    };

    let mut row_y = 0.0f32;
    // 判断 table_box 本身是否为 display:table/inline-table（区分「直接 table-cell
    // 子元素的匿名行」与「孤立行组的匿名行」：前者 row_box=table_box 不应覆盖
    // table 几何，后者 row_box=table_box（行组）需要设置行组几何）。
    let table_is_display_table = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::Table | DisplayValue::InlineTable));
    // 跟踪每个行组的起始 row_y，用于计算行在行组内的相对位置
    // 避免 paint 链中行组位置 + 行位置导致的双重计数
    // 跟踪每个行组的起始 row_y，用于计算行在行组内的相对位置
    // 避免 paint 链中行组位置 + 行位置导致的双重计数
    // 使用 (row_group_index, start_y) 元组
    let mut rg_start_y: Option<(usize, f32)> = None;

    for (row_idx, row) in grid.rows.iter().enumerate() {
        // 检测行组切换，记录新行组的起始 row_y
        let rg_key = row.row_group_index;
        if rg_key != rg_start_y.as_ref().map(|(idx, _)| *idx) {
            if let Some(key) = rg_key {
                rg_start_y = Some((key, row_y));
            } else {
                rg_start_y = None;
            }
        }

        // 根据行是否在 row-group 内，定位到正确的行盒
        let row_box = get_row_box_mut(table_box, row);
        let Some(row_box) = row_box else {
            continue;
        };

        // 行的高度 = 其所有单元格的最大高度
        let mut row_height = 0.0f32;
        for cell in &row.cells {
            let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                row_box
                    .children
                    .get(rg_idx)
                    .and_then(|rg| rg.children.get(cell.child_index))
            } else {
                row_box.children.get(cell.child_index)
            };
            if let Some(cell_box) = cell_box {
                row_height = row_height.max(cell_box.height);
            }
        }
        if row_height == 0.0 {
            row_height = 20.0; // 最小行高
        }
        // R89：应用表格指定 height 的行高分配（额外高度均分到行）
        row_height += row_extras.get(row_idx).copied().unwrap_or(0.0);

        // 设置行盒的位置和尺寸
        // 注意：行组的 position:relative 偏移由 update_row_group_positions 处理，
        // 此处仅应用行自身的 relative 偏移，避免 paint 链双重计数
        let mut row_rel_dx = 0.0f32;
        let mut row_rel_dy = 0.0f32;

        // 行自身的 relative 偏移（不包括行组的）
        if row_box.is_relative {
            row_rel_dx = resolve_length_inset(row_box, styles, true);
            row_rel_dy = resolve_length_inset(row_box, styles, false);
        }

        // 计算行在行组内的相对位置（避免 paint 链双重计数）
        // 有 row_group_index 的行：y 相对于行组起始位置
        // 无 row_group_index 的行（直接 table 子元素）：y 相对于 table content
        let local_y = if let Some((_, start_y)) = rg_start_y {
            row_y - start_y
        } else {
            row_y
        };

        // 直接 table-cell 子元素的匿名行：row_box 即 table_box 本身（display:table），
        // 不覆盖 table 的 x/y（table 由正常流定位）和 width/height（由
        // apply_table_size_constraints 设置）。孤立行组的匿名行（table_box 是行组）
        // 仍需设置行组几何。
        let is_direct_cell_row = row.is_anonymous && row.row_group_index.is_none() && table_is_display_table;
        if !is_direct_cell_row {
            row_box.x = row_rel_dx;
            row_box.y = local_y + row_rel_dy;
            row_box.width = table_content_width;
            row_box.height = row_height;
        }

        // 定位每个单元格
        let mut cell_x = 0.0f32;
        for cell in &row.cells {
            // 根据 parent_rg_idx 查找单元格盒
            // 孤立模式下 row_box = table_box，嵌套行组的单元格通过
            // row_box.children[parent_rg_idx].children[child_index] 访问
            let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                row_box
                    .children
                    .get_mut(rg_idx)
                    .and_then(|rg| rg.children.get_mut(cell.child_index))
            } else {
                row_box.children.get_mut(cell.child_index)
            };
            let Some(cell_box) = cell_box else {
                continue;
            };

            // 计算单元格宽度（跨的所有列宽 + spacing）
            // visibility:collapse 的列宽度为 0，不计入单元格宽度
            let mut cell_width = 0.0f32;
            let mut spans_collapsed = false;
            let mut non_collapsed_count = 0usize;
            for col in cell.col_start..cell.col_end {
                if col < col_widths.len() {
                    cell_width += col_widths[col];
                    if grid.collapsed_cols.get(col).copied().unwrap_or(false) {
                        spans_collapsed = true;
                    } else {
                        non_collapsed_count += 1;
                    }
                }
            }
            // 加上 spacing：仅对相邻的非折叠列之间加 spacing
            // 折叠列不占空间，其两侧的 spacing 也不计入
            if non_collapsed_count > 1 {
                cell_width += (non_collapsed_count - 1) as f32 * spacing_x;
            }

            // 设置单元格位置和尺寸
            // CSS-position-3：table-cell (td) 可 position:relative，其 inset 偏移自身。
            // table-cell 由 table.rs 定位，不经 taffy 正常流的 relative-inset 应用
            // （taffy 的 layout.location 仅对正常流 block 生效），故此处须显式应用
            // 单元格自身的 relative inset（镜像行/行组的 row_rel_dx/dy 处理）。
            let (cell_rel_dx, cell_rel_dy) = if cell_box.is_relative {
                (
                    resolve_length_inset(cell_box, styles, true),
                    resolve_length_inset(cell_box, styles, false),
                )
            } else {
                (0.0, 0.0)
            };
            cell_box.x = cell_x + cell_rel_dx;
            cell_box.y = cell_rel_dy;
            cell_box.width = cell_width;

            // CSS Tables §visibility-collapse-cell-rendering：
            // 跨越折叠列的单元格必须裁剪溢出内容。
            // 普通表格单元格不裁剪（CSS 2.1 规定即使 overflow:hidden 也要增长以包含内容），
            // 但跨越折叠列的单元格是例外——它们的内容必须被限制在可见列的宽度内。
            if spans_collapsed {
                cell_box.overflow_x = OverflowClip::Hidden;
            }

            // 同步更新 content_width：paint 系统使用 content_width 来确定
            // 文本渲染容器宽度、背景裁剪等。如果不更新，width:0 单元格的
            // content_width 仍为 taffy 计算的 0，导致文本无法正确渲染。
            let cell_content_w = (cell_width
                - cell_box.border_left
                - cell_box.border_right
                - cell_box.padding_left
                - cell_box.padding_right)
                .max(0.0);
            cell_box.content_width = cell_content_w;

            // 单元格高度：CSS 2.1 规范中，table cell 的 height 属性被视为最小高度。
            // 单元格必须增长以包含其内容，不能裁剪到明确高度。
            // CSS 2.1 规定即使设置了 overflow:hidden，表格单元格仍然必须增长以包含内容。
            // 取 max(行高, 单元格内容的累积高度)。
            // 注意：正常流子元素是垂直堆叠的，应使用 sum 而非 max。
            let cell_content_height: f32 = cell_box
                .children
                .iter()
                .map(|c| c.height + c.margin_top + c.margin_bottom)
                .sum();
            let cell_height = row_height.max(cell_content_height);
            cell_box.height = cell_height;
            // 同步更新 content_height，确保 overflow 裁剪使用增长后的高度
            let cell_content_h = (cell_height
                - cell_box.border_top
                - cell_box.border_bottom
                - cell_box.padding_top
                - cell_box.padding_bottom)
                .max(0.0);
            cell_box.content_height = cell_content_h;

            // 应用 vertical-align 到单元格内的子元素
            // CSS 2.1 表格单元格内的 vertical-align 控制内容垂直对齐
            if let Some(cell_node_id) = cell_box.node_id
                && let Some(cell_style) = styles.get(&cell_node_id)
            {
                let content_height: f32 = cell_box
                    .children
                    .iter()
                    .map(|c| c.height + c.margin_top + c.margin_bottom)
                    .sum();
                // 子元素 y 是相对单元格 content box 度量的，故可用对齐空间应基于
                // content_height（content 区高）而非 height（border-box 高）。
                // 旧实现用 cell_box.height 会多算 border+padding，把 valign:bottom/middle
                // 内容压到 content 区之外（如 background-043 的 img 偏低约 border 之和）。
                let available = cell_box.content_height - content_height;
                if available > 0.0 {
                    let dy = match cell_style.vertical_align {
                        zero_css_parser::values::VerticalAlignValue::Middle => available / 2.0,
                        zero_css_parser::values::VerticalAlignValue::Bottom
                        | zero_css_parser::values::VerticalAlignValue::TextBottom => available,
                        _ => 0.0, // top, baseline, etc.
                    };
                    if dy > 0.0 {
                        for child in &mut cell_box.children {
                            child.y += dy;
                        }
                    }
                }
            }

            // 折叠列的单元格不推进 cell_x（宽度为 0，也不加 spacing）
            // 非折叠列的单元格正常推进
            let is_in_collapsed_col = cell.col_start < grid.collapsed_cols.len() && grid.collapsed_cols[cell.col_start];
            if !is_in_collapsed_col {
                cell_x += cell_width + spacing_x;
            }
        }

        row_y += row_height + spacing_y;
    }

    // 后处理：应用 min-height/max-height/min-width/max-width 约束
    apply_table_size_constraints(table_box, grid, row_y, col_widths, spacing_y, styles);

    // 后处理：更新行组（tbody/thead/tfoot）的位置以包含其所有行
    // 对于 position:relative 的行组，还需应用 inset 偏移
    update_row_group_positions(table_box, grid, styles);
}

/// 应用 min-height/max-height/min-width/max-width 约束到 table 容器。
///
/// 在 position_cells 之后调用，根据 CSS 尺寸约束调整表格的实际尺寸。
fn apply_table_size_constraints(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    total_row_height: f32,
    col_widths: &[f32],
    _spacing_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;

    let Some(node_id) = table_box.node_id else {
        return;
    };
    let Some(style) = styles.get(&node_id) else {
        return;
    };

    // 计算表格内容的固有宽度（所有列宽 + spacing）
    let spacing_x = style.border_spacing.horizontal;
    let total_col_width: f32 = col_widths.iter().sum();
    let spacing_total_x = if col_widths.len() > 1 {
        (col_widths.len() - 1) as f32 * spacing_x
    } else {
        0.0
    };
    let intrinsic_width = total_col_width + spacing_total_x;
    let intrinsic_height = total_row_height;

    // CSS 表格的 min/max 约束应用：
    // - min-height/max-height 始终作用于 border-box（CSS Tables §table-wrapper-box）
    // - min-width/max-width 根据 box-sizing 决定参照盒
    let is_border_box = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
    let padding_border_w =
        table_box.padding_left + table_box.padding_right + table_box.border_left + table_box.border_right;
    let padding_border_h =
        table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;

    // 应用 min-width / max-width（根据 box-sizing）
    let mut final_width = intrinsic_width;
    if let LengthValue::Px(v) = &style.min_width {
        let min_w = *v as f32;
        let min_content = if is_border_box {
            (min_w - padding_border_w).max(0.0)
        } else {
            min_w
        };
        final_width = final_width.max(min_content);
    }
    if let LengthValue::Px(v) = &style.max_width
        && *v != f64::INFINITY
    {
        let max_w = *v as f32;
        let max_content = if is_border_box {
            (max_w - padding_border_w).max(0.0)
        } else {
            max_w
        };
        final_width = final_width.min(max_content);
    }

    // 应用 min-height / max-height（始终 border-box，与 min-height-table 测试一致）
    // CSS Tables §table-wrapper-box：min/max-height 应用到 table wrapper box（border-box 语义）。
    // 注意：即使 box-sizing: content-box，min/max-height 对表格仍按 border-box 解释，
    // 因为 min-height-table WPT 测试验证了这一行为。
    let mut final_height = intrinsic_height;
    if let LengthValue::Px(v) = &style.min_height {
        let min_content = (*v as f32 - padding_border_h).max(0.0);
        final_height = final_height.max(min_content);
    }
    if let LengthValue::Px(v) = &style.max_height
        && *v != f64::INFINITY
    {
        let max_content = (*v as f32 - padding_border_h).max(0.0);
        final_height = final_height.min(max_content);
    }

    // 更新 table 容器尺寸
    // CSS 表格 shrink-to-fit：当 width:auto 时，content_width 应反映
    // 实际内容宽度（所有列宽之和），而非 taffy 分配的容器宽度
    //
    // border-collapse 修正（R291/R292）：collapse 模式下，compute_column_widths
    // 用「border-center 到 border-center」列宽（含半 cell border），position_cells
    // 的行高含整 cell border；而表四边缘的 cell border 与 table border 折叠——
    // table border 胜出（更宽）时覆盖 cell border，后者不应再叠加进表尺寸。
    // 旧实现两者都计入 → 表尺寸偏大（subpixel-collapsed-borders-001 偏 5px）。
    // 扣除外边缘被覆盖的 cell border：宽扣 (左 cell.bl + 右 cell.br)/2，
    // 高扣 (顶 cell.bt + 底 cell.bb)——分别匹配列宽半 border / 行高整 border 的算法。
    let mut edge_w = 0.0_f32;
    let mut edge_h = 0.0_f32;
    if matches!(style.border_collapse, zero_style_system::BorderCollapseValue::Collapse) && !grid.rows.is_empty() {
        let last_col = grid.col_count;
        // 左/右边缘 cell（col_start==0 / col_end==col_count）的 border_left/right
        for row in &grid.rows {
            let Some(rb) = get_row_box(table_box, row) else {
                continue;
            };
            for cell in &row.cells {
                let is_left = cell.col_start == 0;
                let is_right = cell.col_end >= last_col;
                if !is_left && !is_right {
                    continue;
                }
                let Some(cb) = get_cell_box(rb, cell) else { continue };
                // table border 胜出（>=）时 cell border 被覆盖，才扣；否则 cell 胜出保留
                if is_left && table_box.border_left >= cb.border_left {
                    edge_w += cb.border_left;
                }
                if is_right && table_box.border_right >= cb.border_right {
                    edge_w += cb.border_right;
                }
            }
        }
        // 顶/底边缘行（首/末行）的 border_top/bottom
        for (row_idx, row) in grid.rows.iter().enumerate() {
            let is_top = row_idx == 0;
            let is_bottom = row_idx + 1 == grid.rows.len();
            if !is_top && !is_bottom {
                continue;
            }
            let Some(rb) = get_row_box(table_box, row) else {
                continue;
            };
            for cell in &row.cells {
                let Some(cb) = get_cell_box(rb, cell) else { continue };
                if is_top && table_box.border_top >= cb.border_top {
                    edge_h += cb.border_top;
                }
                if is_bottom && table_box.border_bottom >= cb.border_bottom {
                    edge_h += cb.border_bottom;
                }
            }
        }
    }
    let width_correct = (edge_w / 2.0).min(final_width.max(0.0));
    let height_correct = edge_h.min(final_height.max(0.0));

    table_box.content_width = (final_width - width_correct).max(0.0);
    table_box.width = table_box.content_width + padding_border_w;
    table_box.content_height = (final_height - height_correct).max(0.0);
    table_box.height = table_box.content_height + padding_border_h;
}

/// 更新行组的位置，使其包含所有子行。
///
/// 在 position_cells 之后调用，根据行组的视觉位置更新其 LayoutBox。
/// 这确保行组的背景色、边框等绘制在正确的位置。
fn update_row_group_positions(table_box: &mut LayoutBox, grid: &TableGrid, styles: &HashMap<NodeId, ComputedStyle>) {
    // 收集每个行组的视觉行索引
    let mut rg_rows: HashMap<usize, Vec<usize>> = HashMap::new();
    for (visual_idx, row) in grid.rows.iter().enumerate() {
        if let Some(rg_idx) = row.row_group_index {
            rg_rows.entry(rg_idx).or_default().push(visual_idx);
        }
    }

    if rg_rows.is_empty() {
        return;
    }

    // 读取 border-spacing
    let (_, spacing_y) = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .map(get_border_spacing)
        .unwrap_or((0.0, 0.0));

    // 预计算所有行的 y 位置和高度（不可变借用阶段）
    let mut row_positions: Vec<(f32, f32)> = Vec::with_capacity(grid.rows.len());
    let mut row_y = 0.0f32;
    for row in &grid.rows {
        let row_box = get_row_box(table_box, row);
        let row_height = if let Some(rb) = row_box {
            let mut h = 0.0f32;
            for cell in &row.cells {
                let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                    rb.children.get(rg_idx).and_then(|rg| rg.children.get(cell.child_index))
                } else {
                    rb.children.get(cell.child_index)
                };
                if let Some(cell_box) = cell_box {
                    h = h.max(cell_box.height);
                }
            }
            if h == 0.0 { 20.0 } else { h }
        } else {
            20.0
        };
        row_positions.push((row_y, row_height));
        row_y += row_height + spacing_y;
    }

    // 计算需要更新的行组位置
    let mut updates: Vec<(usize, f32, f32, f32, f32)> = Vec::new();

    for (rg_idx, visual_indices) in &rg_rows {
        let Some(row_group) = table_box.children.get(*rg_idx) else {
            continue;
        };

        // 计算行组的视觉起始 y 和总高度（含 spacing）
        let mut first_row_y = 0.0f32;
        let mut total_h = 0.0f32;
        let mut found_first = false;

        for &visual_idx in visual_indices {
            if let Some(&(ry, rh)) = row_positions.get(visual_idx) {
                if !found_first {
                    first_row_y = ry;
                    found_first = true;
                }
                total_h += rh + spacing_y;
            }
        }
        // 最后一行之后不需要额外的 spacing
        if total_h > 0.0 && spacing_y > 0.0 {
            total_h -= spacing_y;
        }

        // 行组的 relative 偏移
        let mut rel_dx = 0.0f32;
        let mut rel_dy = 0.0f32;
        if row_group.is_relative {
            rel_dx = resolve_length_inset(row_group, styles, true);
            rel_dy = resolve_length_inset(row_group, styles, false);
        }

        updates.push((*rg_idx, rel_dx, first_row_y + rel_dy, table_box.content_width, total_h));
    }

    // 应用更新（可变借用阶段）
    for (rg_idx, new_x, new_y, new_w, new_h) in updates {
        if let Some(row_group) = table_box.children.get_mut(rg_idx) {
            row_group.x = new_x;
            row_group.y = new_y;
            row_group.width = new_w;
            row_group.height = new_h;
        }
    }
}

#[cfg(test)]
mod tests;
