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
        return;
    }

    // 2. 计算列宽
    let col_widths = compute_column_widths(table_box, &grid, styles);

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
/// 在 separated border model 中，table-row-group 和 table-row 的
/// border、padding 和 margin 无视觉效果。
/// 在 collapsed border model 中，只有 border 有意义（用于冲突解决），
/// padding 和 margin 仍然无效。
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
            let cell_box = match row_box.children.get(cell.child_index) {
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
                // 外边缘：table vs cell
                let winner = resolve_border(
                    (table_bt, &table_style.border_top_style, BorderSource::Table),
                    (cb.top_w, &cb.top_s, BorderSource::Cell),
                );
                if winner == BorderSource::Table {
                    let table_color = color_value_to_u32(&table_style.border_top_color);
                    overrides.push((
                        (row_idx, cell_idx),
                        0,
                        table_bt,
                        Some(table_color),
                        Some(table_style.border_top_style.clone()),
                    ));
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
                        overrides.push(((row_idx, cell_idx), 0, 0.0, None, None));
                    } else {
                        let winner = resolve_border(
                            (prev_cb.bottom_w, &prev_cb.bottom_s, BorderSource::Cell),
                            (cb.top_w, &cb.top_s, BorderSource::Cell),
                        );
                        // 始终应用获胜宽度（之前错误地丢弃了结果）
                        let (win_w, win_style) = if winner == BorderSource::Cell {
                            // 当前 cell 的 top 赢
                            (cb.top_w, &cb.top_s)
                        } else {
                            // 上方 cell 的 bottom 赢
                            (prev_cb.bottom_w, &prev_cb.bottom_s)
                        };
                        // 需要覆盖：宽度不同，或样式不同
                        let need_override = (win_w - cb.top_w).abs() > 0.001 || win_style != &cb.top_s;
                        if need_override {
                            let win_color = if winner == BorderSource::Cell {
                                None
                            } else {
                                get_cell_border_color(table_box, grid, row_idx - 1, prev_cell_idx, 2, styles)
                            };
                            let style_ov = if win_style != &cb.top_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 0, win_w, win_color, style_ov));
                        }
                    }
                }
            }

            // ── Bottom edge ──
            // CSS 2.1 §17.6.2：rowspan 单元格的底边在最后跨越行的底部。
            let cell_last_row = row_idx + cell.rowspan - 1;
            let cell_at_table_bottom = cell_last_row >= last_row;

            if cell_at_table_bottom {
                // 单元格底边在表格底部：与 table border 冲突解决
                let winner = resolve_border(
                    (table_bb, &table_style.border_bottom_style, BorderSource::Table),
                    (cb.bottom_w, &cb.bottom_s, BorderSource::Cell),
                );
                if winner == BorderSource::Table {
                    let table_color = color_value_to_u32(&table_style.border_bottom_color);
                    overrides.push((
                        (row_idx, cell_idx),
                        2,
                        table_bb,
                        Some(table_color),
                        Some(table_style.border_bottom_style.clone()),
                    ));
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
                let winner = resolve_border(
                    (table_bl, &table_style.border_left_style, BorderSource::Table),
                    (cb.left_w, &cb.left_s, BorderSource::Cell),
                );
                if winner == BorderSource::Table {
                    let table_color = color_value_to_u32(&table_style.border_left_color);
                    overrides.push((
                        (row_idx, cell_idx),
                        3,
                        table_bl,
                        Some(table_color),
                        Some(table_style.border_left_style.clone()),
                    ));
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
                        overrides.push(((row_idx, cell_idx), 3, 0.0, None, None));
                    } else {
                        let winner = resolve_border(
                            (left_cb.right_w, &left_cb.right_s, BorderSource::Cell),
                            (cb.left_w, &cb.left_s, BorderSource::Cell),
                        );
                        let (win_w, win_style) = if winner == BorderSource::Cell {
                            (cb.left_w, &cb.left_s)
                        } else {
                            (left_cb.right_w, &left_cb.right_s)
                        };
                        let need_override = (win_w - cb.left_w).abs() > 0.001 || win_style != &cb.left_s;
                        if need_override {
                            let win_color = if winner == BorderSource::Cell {
                                None
                            } else {
                                get_cell_border_color(table_box, grid, row_idx, left_cell_idx, 1, styles)
                            };
                            let style_ov = if win_style != &cb.left_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 3, win_w, win_color, style_ov));
                        }
                    }
                }
            }

            // ── Right edge ──
            if is_last_col {
                let winner = resolve_border(
                    (table_br, &table_style.border_right_style, BorderSource::Table),
                    (cb.right_w, &cb.right_s, BorderSource::Cell),
                );
                if winner == BorderSource::Table {
                    let table_color = color_value_to_u32(&table_style.border_right_color);
                    overrides.push((
                        (row_idx, cell_idx),
                        1,
                        table_br,
                        Some(table_color),
                        Some(table_style.border_right_style.clone()),
                    ));
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
        let Some(cell_box) = row_box.children.get_mut(cell.child_index) else {
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
            // 行组内的行按 DOM 顺序排列
            // 行组的 children 中，找到与 row.child_index 对应的行
            rg.children.get_mut(row.child_index)
        }
        None => table_box.children.get_mut(row.child_index),
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
    let cell_box = row_box.children.get(cell.child_index)?;
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
                let rowspan = get_rowspan(child, doc);
                let cell = TableCell {
                    child_index: *child_idx,
                    colspan,
                    rowspan,
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
        let rowspan = get_rowspan(cell_child, doc);
        let col_start = col_cursor;
        let col_end = col_start + colspan;
        cells.push(TableCell {
            child_index: cell_idx,
            colspan,
            rowspan,
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
fn compute_column_widths(table_box: &LayoutBox, grid: &TableGrid, styles: &HashMap<NodeId, ComputedStyle>) -> Vec<f32> {
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

            // CSS 表格规则：width:0 的单元格应使用固有内容宽度，
            // 而非 taffy 计算的 0 宽度。检查 CSS width 属性。
            let css_width_is_small = cell_box
                .node_id
                .and_then(|id| styles.get(&id))
                .map(|s| {
                    use zero_css_parser::values::LengthValue;
                    match &s.width {
                        LengthValue::Px(v) => (*v as f32) < 2.0,
                        _ => false,
                    }
                })
                .unwrap_or(false);
            let cell_width = if css_width_is_small || cell_box.width < 2.0 {
                compute_cell_intrinsic_width(cell_box, styles).max(cell_box.width)
            } else {
                cell_box.width
            };
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

/// 估算单元格的固有内容宽度。
///
/// 当 CSS width:0 被应用时，taffy 会将单元格布局为 0 宽度。
/// 但 CSS 表格规范要求 width:0 解析为 min-content 宽度。
/// 使用字体大小估算单字符宽度作为最小内容宽度。
fn compute_cell_intrinsic_width(cell_box: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    // 从 ComputedStyle 读取字体大小作为字符宽度估算
    let font_size = cell_box
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| {
            use zero_css_parser::values::LengthValue;
            match &s.font_size {
                LengthValue::Px(v) => *v as f32,
                LengthValue::Em(v) => *v as f32,
                LengthValue::Rem(v) => *v as f32,
                _ => 16.0,
            }
        })
        .unwrap_or(16.0);

    // 估算 min-content 宽度：一个字符宽度约为字体大小的 0.6 倍
    // 加上 padding
    let char_width = font_size * 0.6;
    let padding = cell_box.padding_left + cell_box.padding_right;
    char_width + padding
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
    let mut row_y = 0.0f32;

    for row in &grid.rows {
        // 预计算行组的 relative 偏移（需要在可变借用前解析）
        let mut rg_rel_dx = 0.0f32;
        let mut rg_rel_dy = 0.0f32;
        if let Some(rg_idx) = row.row_group_index
            && let Some(row_group) = table_box.children.get(rg_idx)
            && row_group.is_relative
        {
            rg_rel_dx = resolve_length_inset(row_group, styles, true);
            rg_rel_dy = resolve_length_inset(row_group, styles, false);
        }

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
        // 合并行自身和行组的 position:relative inset 偏移
        let mut rel_dx = rg_rel_dx;
        let mut rel_dy = rg_rel_dy;

        // 行自身的 relative 偏移
        if row_box.is_relative {
            rel_dx += resolve_length_inset(row_box, styles, true);
            rel_dy += resolve_length_inset(row_box, styles, false);
        }

        row_box.x = table_box.content_x + rel_dx;
        row_box.y = table_box.content_y + row_y + rel_dy;
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

            cell_x += cell_width + spacing_x;
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

    // 应用 min-width / max-width
    let mut final_width = intrinsic_width;
    if let LengthValue::Px(v) = &style.min_width {
        final_width = final_width.max(*v as f32);
    }
    if let LengthValue::Px(v) = &style.max_width
        && *v != f64::INFINITY
    {
        final_width = final_width.min(*v as f32);
    }

    // 应用 min-height / max-height
    // 注意：min-height/max-height 应用到整个 border box（包含 padding + border）
    let padding_border_h =
        table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;
    let mut final_height = intrinsic_height;
    if let LengthValue::Px(v) = &style.min_height {
        // min-height 包含 padding+border，需减去后得到内容高度
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
    table_box.width = final_width;
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
                if let Some(cell_box) = rb.children.get(cell.child_index) {
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

        updates.push((
            *rg_idx,
            table_box.content_x + rel_dx,
            table_box.content_y + first_row_y + rel_dy,
            table_box.content_width,
            total_h,
        ));
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
