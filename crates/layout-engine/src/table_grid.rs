//! CSS Table 网格构建（row × column grid）。
//!
//! 从 `table.rs` 抽出（R964，2000 行规则）。本模块仅含 grid 构建逻辑：
//! `build_grid`（从 table 容器子元素构建 `TableGrid`）+ 其叶 helper
//! `count_col_elements` / `detect_collapsed_columns`。其余 table 布局阶段
//! （列宽计算、单元格定位、尺寸约束、列背景收集）仍在 `table.rs`。

use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::table_types::*;
use crate::types::LayoutBox;

/// 从 table 容器的子元素中构建 grid 结构。
///
/// 处理以下结构：
/// - `table > tr > td` — 直接子元素是 table-row
/// - `table > tbody > tr > td` — 直接子元素是 table-row-group
/// - `table > td` — 直接子元素是 table-cell（匿名行生成）
pub(crate) fn build_grid(
    table_box: &LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> TableGrid {
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

    // 检测 visibility:collapse 的行（CSS Tables §4.1，与列折叠对称）
    let collapsed_rows = crate::table_visibility::detect_collapsed_rows(table_box, &rows, styles);

    TableGrid {
        rows,
        col_count: max_cols,
        collapsed_cols,
        collapsed_rows,
    }
}

/// 统计 `<col>`/`<colgroup>` 子元素定义的网格列数。
///
/// CSS Tables §4：colgroup 的 span 属性（默认 1）决定其覆盖的列数；
/// 若 colgroup 内含 `<col>` 子元素，则按内部 col 的 span 之和计算
/// （与 `detect_collapsed_columns` 的 col_cursor 推进逻辑保持一致）。
pub(crate) fn count_col_elements(
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
pub(crate) fn detect_collapsed_columns(
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

/// R1718：收集 `<col>`/`<colgroup>` 的 `width` 属性并映射到列索引（≡ detect_collapsed_columns
/// 遍历模式）。返回每列的 col 指定宽度（px，% 已按 `available_width` 解析）；无 col width 的
/// 列返回 None。col width 行为同显式 cell width（compute_column_widths 据此设列宽 floor +
/// col_explicit，auto 扩展期冻结）。colgroup 含 col 子时按各 col 的 width；否则 colgroup 自身
/// width 平铺到其 span 覆盖的列。
pub(crate) fn collect_col_widths(
    table_box: &LayoutBox,
    col_count: usize,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
    available_width: f32,
) -> Vec<Option<f32>> {
    let mut col_widths = vec![None; col_count];
    let mut col_cursor = 0usize;
    let resolve = |w: &str| -> Option<f32> {
        let w = w.trim();
        if let Some(pct) = w.strip_suffix('%') {
            let p = pct.trim().parse::<f32>().ok()?;
            Some((p / 100.0 * available_width).max(0.0))
        } else {
            let n = w.strip_suffix("px").unwrap_or(w).trim();
            n.parse::<f32>().ok().map(|v| v.max(0.0))
        }
    };
    let apply = |col_widths: &mut Vec<Option<f32>>,
                 nid: Option<NodeId>,
                 span: usize,
                 cursor: usize,
                 resolve: &dyn Fn(&str) -> Option<f32>| {
        let Some(nid) = nid else {
            return;
        };
        // R2045：HTML width 属性优先（<col width="100">/"50%"），否则读 CSS width 属性
        // （ComputedStyle 已解析为 Px/%）。此前仅读 HTML attr → CSS-width col（如
        // `#test{display:table-column;width:1in}`）被当 auto 列，错误吸收剩余宽（175px 而非 96）。
        let px = doc.get_attribute(nid, "width").and_then(|w| resolve(&w)).or_else(|| {
            use zero_css_parser::values::LengthValue;
            styles.get(&nid).and_then(|s| match &s.width {
                LengthValue::Px(v) => Some(*v as f32),
                LengthValue::Percentage(p) => Some((*p as f32 / 100.0 * available_width).max(0.0)),
                _ => None,
            })
        });
        let Some(px) = px else {
            return;
        };
        for i in 0..span {
            let idx = cursor + i;
            if idx < col_count {
                col_widths[idx] = Some(px);
            }
        }
    };
    for child in &table_box.children {
        let child_display = get_display(child, styles);
        match child_display {
            Some(DisplayValue::TableColumnGroup) => {
                let has_col_children = child
                    .children
                    .iter()
                    .any(|c| get_display(c, styles) == Some(DisplayValue::TableColumn));
                if has_col_children {
                    for col_child in &child.children {
                        if get_display(col_child, styles) == Some(DisplayValue::TableColumn) {
                            let col_span = get_span(col_child, doc);
                            apply(&mut col_widths, col_child.node_id, col_span, col_cursor, &resolve);
                            col_cursor += col_span;
                        }
                    }
                    continue;
                }
                let rg_span = get_span(child, doc);
                apply(&mut col_widths, child.node_id, rg_span, col_cursor, &resolve);
                col_cursor += rg_span;
            }
            Some(DisplayValue::TableColumn) => {
                let col_span = get_span(child, doc);
                apply(&mut col_widths, child.node_id, col_span, col_cursor, &resolve);
                col_cursor += col_span;
            }
            _ => {}
        }
    }
    col_widths
}
