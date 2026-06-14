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
use zero_style_system::property::types::BorderStyleValue;

use crate::types::LayoutBox;
use crate::types::OverflowClip;

/// 一个表格单元格的信息。
#[derive(Debug, Clone)]
struct TableCell {
    /// 在行 LayoutBox children 中的索引。
    child_index: usize,
    /// colspan 值（默认 1）。
    colspan: usize,
    /// rowspan 值（默认 1）。
    rowspan: usize,
    /// 单元格跨的列范围 [start, end)。
    col_start: usize,
    col_end: usize,
    /// 嵌套行组中的单元格：指向 table_box.children 中的行组索引。
    /// None 表示单元格在 row_box.children[child_index] 中查找（默认）。
    /// Some(rg_idx) 表示单元格在 table_box.children[rg_idx].children[child_index] 中查找。
    /// 用于孤立行组（table_box 本身是行组）中混合嵌套行组和直接子单元格的匿名行。
    parent_rg_idx: Option<usize>,
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
    /// 是否为匿名行（cells 直接在 row-group 中，无包裹 table-row）。
    is_anonymous: bool,
}

/// 解析后的表格网格结构。
#[derive(Debug)]
struct TableGrid {
    /// 行列表。
    rows: Vec<TableRow>,
    /// 总列数。
    col_count: usize,
    /// 每列是否被 visibility:collapse 折叠。
    /// CSS Tables §4.1：visibility:collapse 的列宽度为 0，不参与布局。
    collapsed_cols: Vec<bool>,
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

/// border-collapse: collapse 模式下的边框冲突解决。
///
/// CSS 2.1 §17.6.2.1：当多个边框重叠时，按以下优先级选择"胜出"的边框：
/// 1. border-style：hidden > 其他；none < 其他
/// 2. border-width：越宽越好
/// 3. 来源优先级：cell > row > row-group > col > col-group > table
///
/// 处理两类冲突：
/// - 外边缘：table 边框与 cell 边框
/// - 内部边缘：相邻 cell 之间的边框（包括 hidden/none 样式处理）
fn resolve_collapsed_borders(table_box: &mut LayoutBox, grid: &TableGrid, styles: &HashMap<NodeId, ComputedStyle>) {
    use zero_style_system::BorderCollapseValue;

    let table_style = match table_box.node_id.and_then(|id| styles.get(&id)) {
        Some(s) => s,
        None => return,
    };

    let is_collapsed = matches!(table_style.border_collapse, BorderCollapseValue::Collapse);

    if !is_collapsed {
        return;
    }

    let table_bt = length_to_px(&table_style.border_top_width);
    let table_br = length_to_px(&table_style.border_right_width);
    let table_bb = length_to_px(&table_style.border_bottom_width);
    let table_bl = length_to_px(&table_style.border_left_width);

    let row_count = grid.rows.len();
    if row_count == 0 {
        return;
    }
    let last_row = row_count - 1;
    let last_col = grid.col_count.saturating_sub(1);

    /// 单元格四边边框信息（宽度 + 样式），从 ComputedStyle 读取
    #[derive(Clone)]
    struct CellBorderInfo {
        top_w: f32,
        top_s: BorderStyleValue,
        right_w: f32,
        right_s: BorderStyleValue,
        bottom_w: f32,
        bottom_s: BorderStyleValue,
        left_w: f32,
        left_s: BorderStyleValue,
    }

    // 阶段 1：收集所有单元格的边框信息（从 ComputedStyle 读取）
    // cell_border_data[row_idx][cell_idx] = CellBorderInfo
    // cell_col_map[row_idx][col] = cell_idx（该列对应的单元格在行内的索引）
    let mut cell_border_data: Vec<Vec<Option<CellBorderInfo>>> = Vec::with_capacity(row_count);
    let mut cell_col_map: Vec<Vec<usize>> = Vec::with_capacity(row_count);

    for row in grid.rows.iter() {
        let mut borders = Vec::with_capacity(row.cells.len());
        let mut col_map = vec![0usize; grid.col_count];

        let row_box = match get_row_box(table_box, row) {
            Some(b) => b,
            None => {
                cell_border_data.push(borders);
                cell_col_map.push(col_map);
                continue;
            }
        };

        for (cell_idx, cell) in row.cells.iter().enumerate() {
            let cell_box = match get_cell_box(row_box, cell) {
                Some(b) => b,
                None => {
                    borders.push(None);
                    continue;
                }
            };

            let cell_style = match cell_box.node_id.and_then(|id| styles.get(&id)) {
                Some(s) => s,
                None => {
                    borders.push(None);
                    continue;
                }
            };

            let info = CellBorderInfo {
                top_w: length_to_px(&cell_style.border_top_width),
                top_s: cell_style.border_top_style.clone(),
                right_w: length_to_px(&cell_style.border_right_width),
                right_s: cell_style.border_right_style.clone(),
                bottom_w: length_to_px(&cell_style.border_bottom_width),
                bottom_s: cell_style.border_bottom_style.clone(),
                left_w: length_to_px(&cell_style.border_left_width),
                left_s: cell_style.border_left_style.clone(),
            };

            for slot in col_map
                .iter_mut()
                .take(cell.col_end.min(grid.col_count))
                .skip(cell.col_start)
            {
                *slot = cell_idx;
            }

            borders.push(Some(info));
        }

        cell_border_data.push(borders);
        cell_col_map.push(col_map);
    }

    // 阶段 2：解析所有边框冲突，收集需要覆盖的边框值
    // 格式：((row_idx, cell_idx), side, width, color_override, style_override)
    // side: 0=top, 1=right, 2=bottom, 3=left
    // color_override: Some(rgba_u32) 表示使用此颜色替代 cell 原始颜色
    // style_override: Some(BorderStyleValue) 表示使用此样式替代 cell 原始样式
    #[allow(clippy::type_complexity)]
    let mut overrides: Vec<((usize, usize), u8, f32, Option<u32>, Option<BorderStyleValue>)> = Vec::new();

    for (row_idx, row) in grid.rows.iter().enumerate() {
        for (cell_idx, cell) in row.cells.iter().enumerate() {
            let Some(Some(cb)) = cell_border_data.get(row_idx).and_then(|b| b.get(cell_idx)) else {
                continue;
            };

            let is_first_row = row_idx == 0;
            let _is_last_row = row_idx == last_row;
            let is_first_col = cell.col_start == 0;
            let is_last_col = cell.col_end > last_col;

            // ── Top edge ──
            if is_first_row {
                // 外边缘：table vs rowgroup vs cell（CSS 2.1 §17.6.2.1 多来源解析）
                // 优先级：Cell > Row > RowGroup > Table
                // 先解析低优先级对，再与高优先级比较
                let rg_info = get_row_group_border_info(table_box, grid, row_idx, styles, 0);
                let (mut win_w, mut win_s) = (table_bt, &table_style.border_top_style as &BorderStyleValue);
                let mut win_color = color_value_to_u32(&table_style.border_top_color);
                let mut win_src = BorderSource::Table;
                if let Some((rg_w, rg_s, rg_c)) = rg_info {
                    let winner = resolve_border((win_w, win_s, win_src), (rg_w, rg_s, BorderSource::RowGroup));
                    if winner == BorderSource::RowGroup {
                        win_w = rg_w;
                        win_s = rg_s;
                        win_color = rg_c;
                        win_src = BorderSource::RowGroup;
                    }
                }
                let winner = resolve_border((win_w, win_s, win_src), (cb.top_w, &cb.top_s, BorderSource::Cell));
                if winner != BorderSource::Cell {
                    if matches!(cb.top_s, BorderStyleValue::Hidden) {
                        overrides.push(((row_idx, cell_idx), 0, 0.0, None, None));
                    } else {
                        let style_ov = if win_s != &cb.top_s { Some(win_s.clone()) } else { None };
                        overrides.push(((row_idx, cell_idx), 0, win_w, Some(win_color), style_ov));
                    }
                } else if matches!(cb.top_s, BorderStyleValue::Hidden) {
                    overrides.push(((row_idx, cell_idx), 0, 0.0, None, None));
                }
            } else if row_idx > 0 {
                // 内部边：上一行同列 cell 的 bottom vs 当前 cell 的 top
                let prev_col_map = &cell_col_map[row_idx - 1];
                // 找到上一行中覆盖当前 cell 第一列的单元格
                let col_to_check = cell.col_start.min(grid.col_count.saturating_sub(1));
                let prev_cell_idx = prev_col_map[col_to_check];
                if let Some(Some(prev_cb)) = cell_border_data.get(row_idx - 1).and_then(|b| b.get(prev_cell_idx)) {
                    // hidden 样式优先：强制宽度为 0
                    if matches!(cb.top_s, BorderStyleValue::Hidden)
                        || matches!(prev_cb.bottom_s, BorderStyleValue::Hidden)
                    {
                        // hidden 强制两侧宽度为 0
                        overrides.push(((row_idx, cell_idx), 0, 0.0, None, None));
                        overrides.push(((row_idx - 1, prev_cell_idx), 2, 0.0, None, None));
                    } else {
                        // Cell-vs-Cell 内部边：手动判断哪个 cell 的边框获胜
                        // resolve_border 在两边都是 Cell 时无法区分具体哪个 cell 赢
                        let prev_a_wins = {
                            let prio_a = border_style_priority(&prev_cb.bottom_s);
                            let prio_b = border_style_priority(&cb.top_s);
                            if prio_a != prio_b {
                                prio_a > prio_b
                            } else {
                                prev_cb.bottom_w.floor() > cb.top_w.floor()
                            }
                        };
                        let (win_w, win_style) = if prev_a_wins {
                            (prev_cb.bottom_w, &prev_cb.bottom_s)
                        } else {
                            (cb.top_w, &cb.top_s)
                        };
                        // 覆盖当前 cell 的顶边（side=0）
                        let need_override_cur = (win_w - cb.top_w).abs() > 0.001 || win_style != &cb.top_s;
                        if need_override_cur {
                            let win_color = if prev_a_wins {
                                get_cell_border_color(table_box, grid, row_idx - 1, prev_cell_idx, 2, styles)
                            } else {
                                None
                            };
                            let style_ov = if win_style != &cb.top_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 0, win_w, win_color, style_ov));
                        }
                        // 覆盖上一行 cell 的底边（side=2）—— CSS 2.1 §17.6.2.1 双侧同步
                        let need_override_prev =
                            (win_w - prev_cb.bottom_w).abs() > 0.001 || win_style != &prev_cb.bottom_s;
                        if need_override_prev {
                            let win_color = if !prev_a_wins {
                                get_cell_border_color(table_box, grid, row_idx, cell_idx, 0, styles)
                            } else {
                                None
                            };
                            let style_ov = if win_style != &prev_cb.bottom_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx - 1, prev_cell_idx), 2, win_w, win_color, style_ov));
                        }
                    }
                }
            }

            // ── Bottom edge ──
            // CSS 2.1 §17.6.2：rowspan 单元格的底边在最后跨越行的底部。
            let cell_last_row = row_idx + cell.rowspan - 1;
            let cell_at_table_bottom = cell_last_row >= last_row;

            if cell_at_table_bottom {
                // 单元格底边在表格底部：table vs rowgroup vs cell 多来源解析
                let rg_info = get_row_group_border_info(table_box, grid, cell_last_row, styles, 2);
                let (mut win_w, mut win_s) = (table_bb, &table_style.border_bottom_style as &BorderStyleValue);
                let mut win_color = color_value_to_u32(&table_style.border_bottom_color);
                let mut win_src = BorderSource::Table;
                if let Some((rg_w, rg_s, rg_c)) = rg_info {
                    let winner = resolve_border((win_w, win_s, win_src), (rg_w, rg_s, BorderSource::RowGroup));
                    if winner == BorderSource::RowGroup {
                        win_w = rg_w;
                        win_s = rg_s;
                        win_color = rg_c;
                        win_src = BorderSource::RowGroup;
                    }
                }
                let winner = resolve_border((win_w, win_s, win_src), (cb.bottom_w, &cb.bottom_s, BorderSource::Cell));
                if winner != BorderSource::Cell {
                    if matches!(cb.bottom_s, BorderStyleValue::Hidden) {
                        overrides.push(((row_idx, cell_idx), 2, 0.0, None, None));
                    } else {
                        let style_ov = if win_s != &cb.bottom_s {
                            Some(win_s.clone())
                        } else {
                            None
                        };
                        overrides.push(((row_idx, cell_idx), 2, win_w, Some(win_color), style_ov));
                    }
                } else if matches!(cb.bottom_s, BorderStyleValue::Hidden) {
                    overrides.push(((row_idx, cell_idx), 2, 0.0, None, None));
                }
            }

            // CSS 2.1 §17.6.2.1：行边框参与边框冲突解决。
            // 检查单元格底边所在行的 border-bottom。
            if cell_last_row < row_count
                && let Some(row_at_bottom) = grid.rows.get(cell_last_row)
            {
                let row_box_ref = get_row_box(table_box, row_at_bottom);
                if let Some(rb) = row_box_ref
                    && let Some(rs) = rb.node_id.and_then(|id| styles.get(&id))
                {
                    let row_bb = length_to_px(&rs.border_bottom_width);
                    if row_bb > 0.0
                        && !matches!(
                            rs.border_bottom_style,
                            BorderStyleValue::None | BorderStyleValue::Hidden
                        )
                    {
                        // 行边框与单元格边框冲突解决
                        let winner = resolve_border(
                            (cb.bottom_w, &cb.bottom_s, BorderSource::Cell),
                            (row_bb, &rs.border_bottom_style, BorderSource::Row),
                        );
                        if winner == BorderSource::Row {
                            // 行边框获胜：使用行的颜色和宽度
                            let row_color = color_value_to_u32(&rs.border_bottom_color);
                            overrides.push((
                                (row_idx, cell_idx),
                                2,
                                row_bb,
                                Some(row_color),
                                Some(rs.border_bottom_style.clone()),
                            ));
                        }
                    }
                }
            }

            // ── Left edge ──
            if is_first_col {
                // 外边缘：table vs rowgroup vs cell 多来源解析
                let rg_info = get_row_group_border_info(table_box, grid, row_idx, styles, 3);
                let (mut win_w, mut win_s) = (table_bl, &table_style.border_left_style as &BorderStyleValue);
                let mut win_color = color_value_to_u32(&table_style.border_left_color);
                let mut win_src = BorderSource::Table;
                if let Some((rg_w, rg_s, rg_c)) = rg_info {
                    let winner = resolve_border((win_w, win_s, win_src), (rg_w, rg_s, BorderSource::RowGroup));
                    if winner == BorderSource::RowGroup {
                        win_w = rg_w;
                        win_s = rg_s;
                        win_color = rg_c;
                        win_src = BorderSource::RowGroup;
                    }
                }
                let winner = resolve_border((win_w, win_s, win_src), (cb.left_w, &cb.left_s, BorderSource::Cell));
                if winner != BorderSource::Cell {
                    if matches!(cb.left_s, BorderStyleValue::Hidden) {
                        overrides.push(((row_idx, cell_idx), 3, 0.0, None, None));
                    } else {
                        let style_ov = if win_s != &cb.left_s { Some(win_s.clone()) } else { None };
                        overrides.push(((row_idx, cell_idx), 3, win_w, Some(win_color), style_ov));
                    }
                } else if matches!(cb.left_s, BorderStyleValue::Hidden) {
                    overrides.push(((row_idx, cell_idx), 3, 0.0, None, None));
                }
            } else if cell.col_start > 0 {
                // 内部边：左侧 cell 的 right vs 当前 cell 的 left
                let left_cell_idx = cell_col_map[row_idx][cell.col_start - 1];
                if let Some(Some(left_cb)) = cell_border_data.get(row_idx).and_then(|b| b.get(left_cell_idx)) {
                    if matches!(cb.left_s, BorderStyleValue::Hidden)
                        || matches!(left_cb.right_s, BorderStyleValue::Hidden)
                    {
                        // hidden 强制两侧宽度为 0
                        overrides.push(((row_idx, cell_idx), 3, 0.0, None, None));
                        overrides.push(((row_idx, left_cell_idx), 1, 0.0, None, None));
                    } else {
                        // Cell-vs-Cell 内部边：手动判断哪个 cell 的边框获胜
                        let left_a_wins = {
                            let prio_a = border_style_priority(&left_cb.right_s);
                            let prio_b = border_style_priority(&cb.left_s);
                            if prio_a != prio_b {
                                prio_a > prio_b
                            } else {
                                left_cb.right_w.floor() > cb.left_w.floor()
                            }
                        };
                        let (win_w, win_style) = if left_a_wins {
                            (left_cb.right_w, &left_cb.right_s)
                        } else {
                            (cb.left_w, &cb.left_s)
                        };
                        // 覆盖当前 cell 的左边（side=3）
                        let need_override_cur = (win_w - cb.left_w).abs() > 0.001 || win_style != &cb.left_s;
                        if need_override_cur {
                            let win_color = if left_a_wins {
                                get_cell_border_color(table_box, grid, row_idx, left_cell_idx, 1, styles)
                            } else {
                                None
                            };
                            let style_ov = if win_style != &cb.left_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 3, win_w, win_color, style_ov));
                        }
                        // 覆盖左侧 cell 的右边（side=1）—— CSS 2.1 §17.6.2.1 双侧同步
                        let need_override_left =
                            (win_w - left_cb.right_w).abs() > 0.001 || win_style != &left_cb.right_s;
                        if need_override_left {
                            let win_color = if !left_a_wins {
                                get_cell_border_color(table_box, grid, row_idx, cell_idx, 3, styles)
                            } else {
                                None
                            };
                            let style_ov = if win_style != &left_cb.right_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, left_cell_idx), 1, win_w, win_color, style_ov));
                        }
                    }
                }
            }

            // ── Right edge ──
            if is_last_col {
                // 外边缘：table vs rowgroup vs cell 多来源解析
                let rg_info = get_row_group_border_info(table_box, grid, row_idx, styles, 1);
                let (mut win_w, mut win_s) = (table_br, &table_style.border_right_style as &BorderStyleValue);
                let mut win_color = color_value_to_u32(&table_style.border_right_color);
                let mut win_src = BorderSource::Table;
                if let Some((rg_w, rg_s, rg_c)) = rg_info {
                    let winner = resolve_border((win_w, win_s, win_src), (rg_w, rg_s, BorderSource::RowGroup));
                    if winner == BorderSource::RowGroup {
                        win_w = rg_w;
                        win_s = rg_s;
                        win_color = rg_c;
                        win_src = BorderSource::RowGroup;
                    }
                }
                let winner = resolve_border((win_w, win_s, win_src), (cb.right_w, &cb.right_s, BorderSource::Cell));
                if winner != BorderSource::Cell {
                    if matches!(cb.right_s, BorderStyleValue::Hidden) {
                        overrides.push(((row_idx, cell_idx), 1, 0.0, None, None));
                    } else {
                        let style_ov = if win_s != &cb.right_s {
                            Some(win_s.clone())
                        } else {
                            None
                        };
                        overrides.push(((row_idx, cell_idx), 1, win_w, Some(win_color), style_ov));
                    }
                } else if matches!(cb.right_s, BorderStyleValue::Hidden) {
                    overrides.push(((row_idx, cell_idx), 1, 0.0, None, None));
                }
            }
        }
    }

    // 阶段 3：应用所有边框覆盖到 LayoutBox
    for ((row_idx, cell_idx), side, width, color_override, style_override) in overrides {
        let row = &grid.rows[row_idx];
        let row_box = match get_row_box_mut(table_box, row) {
            Some(b) => b,
            None => continue,
        };
        let cell = &row.cells[cell_idx];
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
        match side {
            0 => {
                cell_box.border_top = width;
                if let Some(c) = color_override {
                    cell_box.collapsed_border_color_overrides[0] = Some(c);
                }
                if let Some(s) = style_override {
                    cell_box.collapsed_border_style_overrides[0] = Some(s);
                }
            }
            1 => {
                cell_box.border_right = width;
                if let Some(c) = color_override {
                    cell_box.collapsed_border_color_overrides[1] = Some(c);
                }
                if let Some(s) = style_override {
                    cell_box.collapsed_border_style_overrides[1] = Some(s);
                }
            }
            2 => {
                cell_box.border_bottom = width;
                if let Some(c) = color_override {
                    cell_box.collapsed_border_color_overrides[2] = Some(c);
                }
                if let Some(s) = style_override {
                    cell_box.collapsed_border_style_overrides[2] = Some(s);
                }
            }
            3 => {
                cell_box.border_left = width;
                if let Some(c) = color_override {
                    cell_box.collapsed_border_color_overrides[3] = Some(c);
                }
                if let Some(s) = style_override {
                    cell_box.collapsed_border_style_overrides[3] = Some(s);
                }
            }
            _ => {}
        }
    }

    // 阶段 4：标记外边缘单元格，供 paint 阶段判断是否减半边框厚度
    // CSS 2.1 §17.6.2：外边缘的边框没有邻居共享，应绘制完整厚度
    for (row_idx, row) in grid.rows.iter().enumerate() {
        for cell in &row.cells {
            let row_box = match get_row_box_mut(table_box, row) {
                Some(b) => b,
                None => continue,
            };
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
            let is_first_row = row_idx == 0;
            let actual_last_row = row_idx + cell.rowspan - 1;
            let cell_at_bottom = actual_last_row >= last_row;
            let is_first_col = cell.col_start == 0;
            let is_last_col = cell.col_end > last_col;
            if is_first_row {
                cell_box.collapsed_border_outer_edge[0] = true;
            }
            if is_last_col {
                cell_box.collapsed_border_outer_edge[1] = true;
            }
            if cell_at_bottom {
                cell_box.collapsed_border_outer_edge[2] = true;
            }
            if is_first_col {
                cell_box.collapsed_border_outer_edge[3] = true;
            }
        }
    }
}

/// 边框来源优先级（CSS 2.1 §17.6.2.1）。
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum BorderSource {
    Table = 0, // 最低优先级
    ColumnGroup = 1,
    Column = 2,
    RowGroup = 3,
    Row = 4,
    Cell = 5, // 最高优先级
}

/// border-style 优先级数值。
///
/// CSS 2.1 §17.6.2.1 中边的优先级顺序：
/// hidden > double > solid > dashed > dotted > ridge > outset > groove > inset > none
fn border_style_priority(style: &BorderStyleValue) -> i32 {
    use BorderStyleValue;
    match style {
        BorderStyleValue::Hidden => 100, // hidden 最高（强制为空）
        BorderStyleValue::None => -1,    // none 最低（等价于无边框）
        BorderStyleValue::Double => 8,
        BorderStyleValue::Solid => 7,
        BorderStyleValue::Dashed => 6,
        BorderStyleValue::Dotted => 5,
        BorderStyleValue::Ridge => 4,
        BorderStyleValue::Outset => 3,
        BorderStyleValue::Groove => 2,
        BorderStyleValue::Inset => 1,
    }
}

/// 解析两条边框的冲突，返回胜出方的来源。
///
/// 比较规则（CSS 2.1 §17.6.2.1）：
/// 1. hidden 样式总是胜出
/// 2. none 样式总是输
/// 3. 更高的 style 优先级胜出
/// 4. 同一 style 时，更宽的边框胜出（宽度先 floor 到整数像素再比较）
/// 5. 同 width 同 style 时，更高来源优先级胜出
fn resolve_border(
    a: (f32, &BorderStyleValue, BorderSource),
    b: (f32, &BorderStyleValue, BorderSource),
) -> BorderSource {
    let (width_a, style_a, source_a) = a;
    let (width_b, style_b, source_b) = b;

    // hidden 总是胜出
    if matches!(style_a, BorderStyleValue::Hidden) {
        return source_a;
    }
    if matches!(style_b, BorderStyleValue::Hidden) {
        return source_b;
    }

    // none 总是输
    let a_is_none = matches!(style_a, BorderStyleValue::None);
    let b_is_none = matches!(style_b, BorderStyleValue::None);
    if a_is_none && !b_is_none {
        return source_b;
    }
    if b_is_none && !a_is_none {
        return source_a;
    }

    // style 优先级
    let prio_a = border_style_priority(style_a);
    let prio_b = border_style_priority(style_b);
    if prio_a > prio_b {
        return source_a;
    }
    if prio_b > prio_a {
        return source_b;
    }

    // 同 style 时比 width（floor 到整数像素再比较，w3c/csswg-drafts#606）
    let fw_a = width_a.floor();
    let fw_b = width_b.floor();
    if fw_a > fw_b {
        return source_a;
    }
    if fw_b > fw_a {
        return source_b;
    }

    // 同 width 同 style 时比来源优先级
    if source_a as i32 >= source_b as i32 {
        source_a
    } else {
        source_b
    }
}

/// 获取行 box 的可变引用。
fn get_row_box_mut<'a>(table_box: &'a mut LayoutBox, row: &TableRow) -> Option<&'a mut LayoutBox> {
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

/// 将 LengthValue 转换为像素值（简化版，不处理百分比和 auto）。
fn length_to_px(value: &zero_css_parser::values::LengthValue) -> f32 {
    use zero_css_parser::values::LengthValue;
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Em(v) => *v as f32 * 16.0,
        LengthValue::Rem(v) => *v as f32 * 16.0,
        _ => 0.0,
    }
}

/// 将 ColorValue 转换为 RGBA u32 格式（0xRRGGBBAA）。
fn color_value_to_u32(color: &zero_css_parser::values::ColorValue) -> u32 {
    match color {
        zero_css_parser::values::ColorValue::Rgba(r, g, b, a) => {
            ((*r as u32) << 24) | ((*g as u32) << 16) | ((*b as u32) << 8) | (*a as u32)
        }
        zero_css_parser::values::ColorValue::Named(name) => {
            // 常见颜色名称到 RGBA 的映射
            match name.to_lowercase().as_str() {
                "green" => 0x008000FF,
                "red" => 0xFF0000FF,
                "blue" => 0x0000FFFF,
                "black" => 0x000000FF,
                "white" => 0xFFFFFFFF,
                "orange" => 0xFFA500FF,
                "yellow" => 0xFFFF00FF,
                "purple" => 0x800080FF,
                "cyan" | "aqua" => 0x00FFFFFF,
                "magenta" | "fuchsia" => 0xFF00FFFF,
                "silver" => 0xC0C0C0FF,
                "gray" | "grey" => 0x808080FF,
                "maroon" => 0x800000FF,
                "olive" => 0x808000FF,
                "navy" => 0x000080FF,
                "teal" => 0x008080FF,
                "lime" => 0x00FF00FF,
                _ => 0x000000FF,
            }
        }
        _ => 0x000000FF,
    }
}

/// 获取指定单元格指定边的边框颜色（RGBA u32）。
/// side: 0=top, 1=right, 2=bottom, 3=left
fn get_cell_border_color(
    table_box: &LayoutBox,
    grid: &TableGrid,
    row_idx: usize,
    cell_idx: usize,
    side: u8,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<u32> {
    let row = grid.rows.get(row_idx)?;
    let row_box = get_row_box(table_box, row)?;
    let cell = row.cells.get(cell_idx)?;
    let cell_box = get_cell_box(row_box, cell)?;
    let cell_style = cell_box.node_id.and_then(|id| styles.get(&id))?;
    let color = match side {
        0 => &cell_style.border_top_color,
        1 => &cell_style.border_right_color,
        2 => &cell_style.border_bottom_color,
        3 => &cell_style.border_left_color,
        _ => return None,
    };
    Some(color_value_to_u32(color))
}

/// 获取行所在的行组（tbody/thead/tfoot）的边框信息。
/// 返回 (width, style, color) 用于参与外边缘的边框冲突解决。
/// 如果行不在行组内或行组没有有效边框，返回 None。
fn get_row_group_border_info<'a>(
    table_box: &LayoutBox,
    grid: &TableGrid,
    row_idx: usize,
    styles: &'a HashMap<NodeId, ComputedStyle>,
    side: u8, // 0=top, 1=right, 2=bottom, 3=left
) -> Option<(f32, &'a BorderStyleValue, u32)> {
    let row = grid.rows.get(row_idx)?;
    let rg_idx = row.row_group_index?;
    let rg_box = table_box.children.get(rg_idx)?;
    let rg_style = rg_box.node_id.and_then(|id| styles.get(&id))?;
    let (width, style, color) = match side {
        0 => (
            length_to_px(&rg_style.border_top_width),
            &rg_style.border_top_style,
            color_value_to_u32(&rg_style.border_top_color),
        ),
        1 => (
            length_to_px(&rg_style.border_right_width),
            &rg_style.border_right_style,
            color_value_to_u32(&rg_style.border_right_color),
        ),
        2 => (
            length_to_px(&rg_style.border_bottom_width),
            &rg_style.border_bottom_style,
            color_value_to_u32(&rg_style.border_bottom_color),
        ),
        3 => (
            length_to_px(&rg_style.border_left_width),
            &rg_style.border_left_style,
            color_value_to_u32(&rg_style.border_left_color),
        ),
        _ => return None,
    };
    // none/hidden 样式的边框不参与冲突解决（已由 border-width zeroing 处理）
    if matches!(style, BorderStyleValue::None) || width <= 0.0 {
        return None;
    }
    Some((width, style, color))
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
                // 直接子元素是 table-cell — 生成匿名行
                let colspan = get_colspan(child, doc);
                let rowspan = get_rowspan(child, doc);
                let cell = TableCell {
                    child_index: *child_idx,
                    colspan,
                    rowspan,
                    col_start: 0,
                    col_end: colspan,
                    parent_rg_idx: None,
                };
                max_cols = max_cols.max(colspan);
                rows.push(TableRow {
                    child_index: *child_idx, // 匿名行直接引用 cell 的索引
                    row_group_index: None,
                    cells: vec![cell],
                    is_anonymous: false,
                });
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

    // 检测 visibility:collapse 的列
    // CSS Tables §4.1：col/colgroup 上 visibility:collapse 的列宽度为 0
    let collapsed_cols = detect_collapsed_columns(table_box, max_cols, styles, doc);

    TableGrid {
        rows,
        col_count: max_cols,
        collapsed_cols,
    }
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

    // 收集每列的最大宽度（两遍算法）
    // CSS Tables §17.5.2.2：列宽首先由非跨列单元格决定（含显式 width），
    // 跨列单元格只把宽度分配给尚未被非跨列单元格约束的列，
    // 这样显式列宽不会被跨列单元格的长内容撑开。
    let mut col_max_widths = vec![0.0f32; col_count];

    // 辅助闭包：计算单元格的宽度
    let cell_used_width = |cell_box: &LayoutBox| -> (f32, bool) {
        let css_width_auto = cell_box
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                use zero_css_parser::values::LengthValue;
                match &s.width {
                    LengthValue::Auto => true,
                    LengthValue::Px(v) => (*v as f32) < 2.0,
                    _ => false,
                }
            })
            .unwrap_or(true);
        let intrinsic = compute_cell_intrinsic_width(cell_box, styles, doc);
        // auto 宽度的单元格：列宽只取内容固有宽度（intrinsic）。
        // taffy 把单元格当 block，cell_box.width = 行/表全宽，不能作为列宽下限
        //（否则每列都撑到全宽，列总和溢出表宽）。无论 table 本身 width 是否 auto，
        // auto 单元格都不应用 cell_box.width 作为下限。
        let w = if css_width_auto || cell_box.width < 2.0 {
            intrinsic
        } else {
            cell_box.width
        };
        (w, css_width_auto)
    };

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

    // 计算总宽度
    let total_width: f32 = col_max_widths.iter().sum();

    // CSS 表格收缩适应（shrink-to-fit）：
    // 当 table 的 CSS width 为 auto 且 table-layout 不为 fixed 时，
    // 表格不应扩展到容器宽度，而是收缩到内容固有宽度。
    // table-layout: fixed 时，列宽由 <col> 或首行决定，仍需扩展。
    let table_style = table_box.node_id.and_then(|id| styles.get(&id));
    let has_explicit_width = table_style.as_ref().is_some_and(|s| {
        use zero_css_parser::values::LengthValue;
        !matches!(s.width, LengthValue::Auto)
    });
    let is_fixed_layout = table_style
        .as_ref()
        .is_some_and(|s| matches!(s.table_layout, zero_style_system::TableLayoutValue::Fixed));

    if (has_explicit_width || is_fixed_layout) && total_width < available_width && total_width > 0.0 {
        // 按比例扩展到容器宽度
        let ratio = available_width / total_width;
        for w in &mut col_max_widths {
            *w *= ratio;
        }
    }

    // visibility:collapse 的列宽度为 0
    // CSS Tables §4.1：折叠列不参与布局，其宽度视为 0
    for (i, w) in col_max_widths.iter_mut().enumerate() {
        if grid.collapsed_cols.get(i).copied().unwrap_or(false) {
            *w = 0.0;
        }
    }

    col_max_widths
}

/// 获取行盒 — 处理直接 table-row、row-group 内的行和匿名行三种情况。
///
/// 当 `row.row_group_index` 为 Some 时，行在 row-group 内。
/// 当 `row.is_anonymous` 为 true 时，行是匿名行，行盒为 row-group 本身。
/// 当为 None 时，行是 table 的直接 children[row.child_index]。
fn get_row_box<'a>(table_box: &'a LayoutBox, row: &TableRow) -> Option<&'a LayoutBox> {
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
fn get_cell_box<'a>(row_box: &'a LayoutBox, cell: &TableCell) -> Option<&'a LayoutBox> {
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

        row_box.x = row_rel_dx;
        row_box.y = local_y + row_rel_dy;
        row_box.width = table_content_width;
        row_box.height = row_height;

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
            cell_box.x = cell_x;
            cell_box.y = 0.0;
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
                let available = cell_box.height - content_height;
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
    _grid: &TableGrid,
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
    table_box.content_width = final_width;
    table_box.width = final_width + padding_border_w;
    table_box.content_height = final_height;
    table_box.height = final_height + padding_border_h;
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
