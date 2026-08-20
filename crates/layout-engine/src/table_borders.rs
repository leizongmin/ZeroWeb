//! 表格 collapsed-border 解析。
//!
//! 从 `table.rs` 抽出（R342c，2000 行规则）。包含 resolve_collapsed_borders（CSS §17.6.2
//! collapsed 边框冲突解析）及其辅助：BorderSource 优先级、resolve_border、边框颜色读取、
//! length/colour 工具。共享 table.rs 的 TableCell/TableRow/TableGrid 类型与 get_row_box_mut。

use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::BorderStyleValue;

use crate::table::{TableGrid, get_cell_box, get_row_box, get_row_box_mut};
use crate::types::LayoutBox;

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
pub(crate) fn resolve_collapsed_borders(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_style_system::BorderCollapseValue;

    let table_style = match table_box.node_id.and_then(|id| styles.get(&id)) {
        Some(s) => s,
        None => return,
    };

    let is_collapsed = matches!(table_style.border_collapse, BorderCollapseValue::Collapse);

    if !is_collapsed {
        return;
    }

    let table_bt = length_to_px(&table_style.border_top_width, table_style);
    let table_br = length_to_px(&table_style.border_right_width, table_style);
    let table_bb = length_to_px(&table_style.border_bottom_width, table_style);
    let table_bl = length_to_px(&table_style.border_left_width, table_style);

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
                top_w: length_to_px(&cell_style.border_top_width, cell_style),
                top_s: cell_style.border_top_style.clone(),
                right_w: length_to_px(&cell_style.border_right_width, cell_style),
                right_s: cell_style.border_right_style.clone(),
                bottom_w: length_to_px(&cell_style.border_bottom_width, cell_style),
                bottom_s: cell_style.border_bottom_style.clone(),
                left_w: length_to_px(&cell_style.border_left_width, cell_style),
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
                        // CSS 2.1 §17.6.2.1：同 style 同 width 平局时，最左/最上格胜出（用 >=）。
                        let prev_a_wins = {
                            let prio_a = border_style_priority(&prev_cb.bottom_s);
                            let prio_b = border_style_priority(&cb.top_s);
                            if prio_a != prio_b {
                                prio_a > prio_b
                            } else {
                                prev_cb.bottom_w.floor() >= cb.top_w.floor()
                            }
                        };
                        let (win_w, win_style) = if prev_a_wins {
                            (prev_cb.bottom_w, &prev_cb.bottom_s)
                        } else {
                            (cb.top_w, &cb.top_s)
                        };
                        // 提前取两侧颜色：§17.6.2.1 平局异色时也须把胜出色 propagate 到败者侧
                        //（旧触发条件只看宽/style 差，同宽同 style 异色平局不推 override → 共享边
                        // 被相邻两格各画半宽异色，border-conflict-element-001a 谱系，R1626）。
                        let prev_bottom_color =
                            get_cell_border_color(table_box, grid, row_idx - 1, prev_cell_idx, 2, styles);
                        let cur_top_color = get_cell_border_color(table_box, grid, row_idx, cell_idx, 0, styles);
                        let colors_differ = matches!(
                            (prev_bottom_color, cur_top_color),
                            (Some(wc), Some(cc)) if wc != cc
                        );
                        // 覆盖当前 cell 的顶边（side=0）
                        let need_override_cur = (win_w - cb.top_w).abs() > 0.001
                            || win_style != &cb.top_s
                            || (prev_a_wins && colors_differ);
                        if need_override_cur {
                            let win_color = if prev_a_wins { prev_bottom_color } else { None };
                            let style_ov = if win_style != &cb.top_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 0, win_w, win_color, style_ov));
                        }
                        // 覆盖上一行 cell 的底边（side=2）—— CSS 2.1 §17.6.2.1 双侧同步
                        let need_override_prev = (win_w - prev_cb.bottom_w).abs() > 0.001
                            || win_style != &prev_cb.bottom_s
                            || (!prev_a_wins && colors_differ);
                        if need_override_prev {
                            let win_color = if !prev_a_wins { cur_top_color } else { None };
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
                    let row_bb = length_to_px(&rs.border_bottom_width, rs);
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

            // CSS 2.1 §17.6.2.1：行边框参与顶边冲突解决（镜像下方 BOTTOM 的 ROW 处理，
            // 补 TOP 对称缺口——原仅 BOTTOM 读 row border-bottom，TOP 漏 row border-top）。
            // 单元格顶边所在起始行（row_idx）的 border-top 与单元格 border-top 冲突解决。
            if row_idx < row_count
                && let Some(row_at_top) = grid.rows.get(row_idx)
            {
                let row_box_ref = get_row_box(table_box, row_at_top);
                if let Some(rb) = row_box_ref
                    && let Some(rs) = rb.node_id.and_then(|id| styles.get(&id))
                {
                    let row_bt = length_to_px(&rs.border_top_width, rs);
                    if row_bt > 0.0 && !matches!(rs.border_top_style, BorderStyleValue::None | BorderStyleValue::Hidden)
                    {
                        // 行边框与单元格顶边冲突解决
                        let winner = resolve_border(
                            (cb.top_w, &cb.top_s, BorderSource::Cell),
                            (row_bt, &rs.border_top_style, BorderSource::Row),
                        );
                        if winner == BorderSource::Row {
                            // 行边框获胜：使用行的颜色和宽度
                            let row_color = color_value_to_u32(&rs.border_top_color);
                            overrides.push((
                                (row_idx, cell_idx),
                                0,
                                row_bt,
                                Some(row_color),
                                Some(rs.border_top_style.clone()),
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
                        // CSS 2.1 §17.6.2.1：同 style 同 width 平局时，最左/最上格胜出（用 >=）。
                        let left_a_wins = {
                            let prio_a = border_style_priority(&left_cb.right_s);
                            let prio_b = border_style_priority(&cb.left_s);
                            if prio_a != prio_b {
                                prio_a > prio_b
                            } else {
                                left_cb.right_w.floor() >= cb.left_w.floor()
                            }
                        };
                        let (win_w, win_style) = if left_a_wins {
                            (left_cb.right_w, &left_cb.right_s)
                        } else {
                            (cb.left_w, &cb.left_s)
                        };
                        // 提前取两侧颜色：平局异色时也须把胜出色 propagate 到败者侧（R1626）。
                        let left_right_color =
                            get_cell_border_color(table_box, grid, row_idx, left_cell_idx, 1, styles);
                        let cur_left_color = get_cell_border_color(table_box, grid, row_idx, cell_idx, 3, styles);
                        let colors_differ = matches!(
                            (left_right_color, cur_left_color),
                            (Some(wc), Some(cc)) if wc != cc
                        );
                        // 覆盖当前 cell 的左边（side=3）
                        let need_override_cur = (win_w - cb.left_w).abs() > 0.001
                            || win_style != &cb.left_s
                            || (left_a_wins && colors_differ);
                        if need_override_cur {
                            let win_color = if left_a_wins { left_right_color } else { None };
                            let style_ov = if win_style != &cb.left_s {
                                Some(win_style.clone())
                            } else {
                                None
                            };
                            overrides.push(((row_idx, cell_idx), 3, win_w, win_color, style_ov));
                        }
                        // 覆盖左侧 cell 的右边（side=1）—— CSS 2.1 §17.6.2.1 双侧同步
                        let need_override_left = (win_w - left_cb.right_w).abs() > 0.001
                            || win_style != &left_cb.right_s
                            || (!left_a_wins && colors_differ);
                        if need_override_left {
                            let win_color = if !left_a_wins { cur_left_color } else { None };
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
pub(crate) enum BorderSource {
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
pub(crate) fn border_style_priority(style: &BorderStyleValue) -> i32 {
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
pub(crate) fn resolve_border(
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

/// 将 border width LengthValue 转换为像素值。
///
/// Percent/auto/intrinsic 不属于 border-width used value；direct-style 残留时按 0 处理。
pub(crate) fn length_to_px(value: &zero_css_parser::values::LengthValue, style: &ComputedStyle) -> f32 {
    use zero_css_parser::values::LengthValue;
    match value {
        LengthValue::Auto
        | LengthValue::Percentage(_)
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::FitContent(_) => 0.0,
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            zero_style_system::computed::resolve_length(other, font_size_px, None, None) as f32
        }
    }
}

/// 将 ColorValue 转换为 RGBA u32 格式（0xRRGGBBAA）。
pub(crate) fn color_value_to_u32(color: &zero_css_parser::values::ColorValue) -> u32 {
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
pub(crate) fn get_cell_border_color(
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
pub(crate) fn get_row_group_border_info<'a>(
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
            length_to_px(&rg_style.border_top_width, rg_style),
            &rg_style.border_top_style,
            color_value_to_u32(&rg_style.border_top_color),
        ),
        1 => (
            length_to_px(&rg_style.border_right_width, rg_style),
            &rg_style.border_right_style,
            color_value_to_u32(&rg_style.border_right_color),
        ),
        2 => (
            length_to_px(&rg_style.border_bottom_width, rg_style),
            &rg_style.border_bottom_style,
            color_value_to_u32(&rg_style.border_bottom_color),
        ),
        3 => (
            length_to_px(&rg_style.border_left_width, rg_style),
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

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::LengthValue;

    #[test]
    fn collapsed_border_length_uses_source_font_size_for_em() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);

        assert_eq!(length_to_px(&LengthValue::Em(2.0), &style), 40.0);
    }

    #[test]
    fn collapsed_border_length_resolves_non_px_real_units() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);

        assert_eq!(length_to_px(&LengthValue::Ch(4.0), &style), 40.0);
        assert_eq!(length_to_px(&LengthValue::Percentage(50.0), &style), 0.0);
    }
}
