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

use std::collections::HashMap;
use zero_css_parser::values::DisplayValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// 一个表格单元格的信息。
#[derive(Debug, Clone)]
struct TableCell {
    /// 在行 LayoutBox children 中的索引。
    child_index: usize,
    /// colspan 值（默认 1）。
    colspan: usize,
    /// 单元格跨的列范围 [start, end)。
    col_start: usize,
    col_end: usize,
}

/// 一个表格行的信息。
#[derive(Debug, Clone)]
struct TableRow {
    /// 在 table LayoutBox children 中的索引。
    /// 当行是直接子 table-row 时，这是 table_box.children 中的索引。
    child_index: usize,
    /// 行所在的行组（tbody/thead/tfoot）在 table LayoutBox children 中的索引。
    /// None 表示行是 table 的直接 table-row 子元素。
    row_group_index: Option<usize>,
    /// 行内的单元格列表。
    cells: Vec<TableCell>,
}

/// 解析后的表格网格结构。
#[derive(Debug)]
struct TableGrid {
    /// 行列表。
    rows: Vec<TableRow>,
    /// 总列数。
    col_count: usize,
}

/// 对 LayoutBox 树执行 table 布局后处理。
///
/// 遍历所有 `display: table` 或 `display: inline-table` 的容器，
/// 将其子元素按 table grid 规则重新定位。
pub fn adjust_table_layout(root: &mut LayoutBox, doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) {
    let display = get_display(root, styles);

    if display == Some(DisplayValue::Table) || display == Some(DisplayValue::InlineTable) {
        layout_table(root, doc, styles);
    }

    // 递归处理子节点
    for child in &mut root.children {
        adjust_table_layout(child, doc, styles);
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

/// 从 ComputedStyle 中读取 border-spacing 值。
fn get_border_spacing(style: &ComputedStyle) -> (f32, f32) {
    (style.border_spacing.horizontal, style.border_spacing.vertical)
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
        return;
    }

    // 2. 计算列宽
    let col_widths = compute_column_widths(table_box, &grid);

    // 3. 定位单元格
    position_cells(table_box, &grid, &col_widths, spacing_x, spacing_y);
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

    for (child_idx, child, child_display) in &children_with_priority {
        match child_display {
            Some(d) if is_table_row(d) => {
                // 直接子元素是 table-row
                let row = build_row(*child_idx, child, doc);
                max_cols = max_cols.max(row.cells.last().map(|c| c.col_end).unwrap_or(0));
                if !row.cells.is_empty() {
                    rows.push(TableRow {
                        row_group_index: None,
                        ..row
                    });
                }
            }
            Some(d) if is_row_group(d) => {
                // 直接子元素是 table-row-group (tbody/thead/tfoot)
                // 从 row-group 中提取行
                for (rg_child_idx, rg_child) in child.children.iter().enumerate() {
                    let rg_display = get_display(rg_child, styles);
                    if rg_display.as_ref().is_some_and(is_table_row) {
                        let row = build_row(rg_child_idx, rg_child, doc);
                        max_cols = max_cols.max(row.cells.last().map(|c| c.col_end).unwrap_or(0));
                        if !row.cells.is_empty() {
                            rows.push(TableRow {
                                row_group_index: Some(*child_idx),
                                ..row
                            });
                        }
                    }
                }
            }
            Some(d) if is_table_cell(d) => {
                // 直接子元素是 table-cell — 生成匿名行
                let colspan = get_colspan(child, doc);
                let cell = TableCell {
                    child_index: *child_idx,
                    colspan,
                    col_start: 0,
                    col_end: colspan,
                };
                max_cols = max_cols.max(colspan);
                rows.push(TableRow {
                    child_index: *child_idx, // 匿名行直接引用 cell 的索引
                    row_group_index: None,
                    cells: vec![cell],
                });
            }
            _ => {
                // 其他类型（caption、column 等）— 跳过
            }
        }
    }

    TableGrid {
        rows,
        col_count: max_cols,
    }
}

/// 从一个 table-row 子元素构建 TableRow。
fn build_row(child_idx: usize, row_box: &LayoutBox, doc: &zero_dom::Document) -> TableRow {
    let mut cells = Vec::new();
    let mut col_cursor = 0usize;

    for (cell_idx, cell_child) in row_box.children.iter().enumerate() {
        let colspan = get_colspan(cell_child, doc);
        let col_start = col_cursor;
        let col_end = col_start + colspan;
        cells.push(TableCell {
            child_index: cell_idx,
            colspan,
            col_start,
            col_end,
        });
        col_cursor = col_end;
    }

    TableRow {
        child_index: child_idx,
        row_group_index: None, // 由调用方设置
        cells,
    }
}

/// Auto table layout：根据单元格内容计算每列的宽度。
///
/// 算法：
/// 1. 扫描所有单元格，记录每列的最大内容宽度
/// 2. 如果所有列宽之和小于容器宽度，按比例分配剩余空间
/// 3. 如果所有列宽之和大于容器宽度，保持内容宽度不变
fn compute_column_widths(table_box: &LayoutBox, grid: &TableGrid) -> Vec<f32> {
    let available_width = table_box.content_width;
    let col_count = grid.col_count;

    if col_count == 0 {
        return Vec::new();
    }

    // 收集每列的最大宽度
    let mut col_max_widths = vec![0.0f32; col_count];

    for row in &grid.rows {
        // 获取行盒：可能是直接子元素（table-row），也可能是 row-group 内的行
        let row_box = get_row_box(table_box, row);
        let Some(row_box) = row_box else {
            continue;
        };

        for cell in &row.cells {
            let Some(cell_box) = get_cell_box(row_box, cell) else {
                continue;
            };
            let cell_width = cell_box.width;
            let colspan = cell.colspan;

            if colspan > 1 {
                let per_col = cell_width / colspan as f32;
                for w in col_max_widths
                    .iter_mut()
                    .take(cell.col_end.min(col_count))
                    .skip(cell.col_start)
                {
                    *w = (*w).max(per_col);
                }
            } else if cell.col_start < col_count {
                col_max_widths[cell.col_start] = col_max_widths[cell.col_start].max(cell_width);
            }
        }
    }

    // 计算总宽度
    let total_width: f32 = col_max_widths.iter().sum();

    if total_width < available_width && total_width > 0.0 {
        // 按比例扩展到容器宽度
        let ratio = available_width / total_width;
        for w in &mut col_max_widths {
            *w *= ratio;
        }
    }

    col_max_widths
}

/// 获取行盒 — 处理直接 table-row 和 row-group 内的行两种情况。
///
/// 当 `row.row_group_index` 为 Some 时，行在 row-group 的 children[row.child_index] 中。
/// 当为 None 时，行是 table 的直接 children[row.child_index]。
fn get_row_box<'a>(table_box: &'a LayoutBox, row: &TableRow) -> Option<&'a LayoutBox> {
    match row.row_group_index {
        Some(rg_idx) => {
            // 行在 row-group 内：table_box.children[rg_idx].children[row.child_index]
            let row_group = table_box.children.get(rg_idx)?;
            row_group.children.get(row.child_index)
        }
        None => {
            // 直接 table-row：table_box.children[row.child_index]
            table_box.children.get(row.child_index)
        }
    }
}

/// 获取单元格盒。
fn get_cell_box<'a>(row_box: &'a LayoutBox, cell: &TableCell) -> Option<&'a LayoutBox> {
    row_box.children.get(cell.child_index)
}

/// 根据 grid 结构和列宽定位每个单元格。
fn position_cells(table_box: &mut LayoutBox, grid: &TableGrid, col_widths: &[f32], spacing_x: f32, spacing_y: f32) {
    let mut row_y = 0.0f32;

    for row in &grid.rows {
        // 根据行是否在 row-group 内，定位到正确的行盒
        let row_box = match row.row_group_index {
            Some(rg_idx) => {
                let Some(row_group) = table_box.children.get_mut(rg_idx) else {
                    continue;
                };
                row_group.children.get_mut(row.child_index)
            }
            None => table_box.children.get_mut(row.child_index),
        };
        let Some(row_box) = row_box else {
            continue;
        };

        // 行的高度 = 其所有单元格的最大高度
        let mut row_height = 0.0f32;
        for cell in &row.cells {
            if let Some(cell_box) = row_box.children.get(cell.child_index) {
                row_height = row_height.max(cell_box.height);
            }
        }
        if row_height == 0.0 {
            row_height = 20.0; // 最小行高
        }

        // 设置行盒的位置和尺寸
        row_box.x = table_box.content_x;
        row_box.y = table_box.content_y + row_y;
        row_box.width = table_box.content_width;
        row_box.height = row_height;

        // 定位每个单元格
        let mut cell_x = 0.0f32;
        for cell in &row.cells {
            let Some(cell_box) = row_box.children.get_mut(cell.child_index) else {
                continue;
            };

            // 计算单元格宽度（跨的所有列宽 + spacing）
            let mut cell_width = 0.0f32;
            for col in cell.col_start..cell.col_end {
                if col < col_widths.len() {
                    cell_width += col_widths[col];
                }
            }
            // 加上 colspan-1 个 spacing
            if cell.colspan > 1 {
                cell_width += (cell.colspan - 1) as f32 * spacing_x;
            }

            // 设置单元格位置和尺寸
            cell_box.x = cell_x;
            cell_box.y = 0.0;
            cell_box.width = cell_width;
            cell_box.height = row_height;

            cell_x += cell_width + spacing_x;
        }

        row_y += row_height + spacing_y;
    }
}
