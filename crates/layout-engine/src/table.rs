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
// R964：grid 构建（build_grid + 列计数/折叠 helper）抽出到 table_grid.rs（2000 行规则）。
use crate::table_grid::build_grid;

// R832：表数据类型 + 叶 helper 抽出到 table_types.rs（2000 行规则）。本模块的大布局
// 函数（build_grid/compute_column_widths/position_cells 等）调用这些叶 helper；
// `pub(crate) use` 既供本模块调用，又再导出为 `crate::table::*` 供外部模块
//（table_borders/engine 等）访问，保持原 API 路径不变（纯移动，零行为变化）。
pub(crate) use crate::table_types::*;

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue};

use zero_dom::NodeId;

use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::types::LayoutBox;

use crate::types::OverflowClip;

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
        let mut idx = 0usize;
        while idx < root.children.len() {
            let child_display = get_display(&root.children[idx], styles);
            let child_is_orphan_internal = !inside_table && child_display.as_ref().is_some_and(is_table_internal);
            let child_is_real_table =
                child_display == Some(DisplayValue::Table) || child_display == Some(DisplayValue::InlineTable);

            if child_is_orphan_internal {
                // CSS2 §17.2.1.1：连续孤立 table-internal 兄弟应合并到一个匿名 table
                // 包装盒，而非各自独立成表（否则会重叠/不堆叠）。先测 run 长度。
                let run_start = idx;
                while idx < root.children.len() {
                    let d = get_display(&root.children[idx], styles);
                    if !inside_table && d.as_ref().is_some_and(is_table_internal) {
                        idx += 1;
                    } else {
                        break;
                    }
                }
                let run_len = idx - run_start;
                if run_len >= 2 {
                    let merged_idx = merge_orphan_table_run(root, run_start, run_len, doc, styles);
                    for child in &mut root.children[merged_idx].children {
                        adjust_table_layout_inner(child, doc, styles, true);
                    }
                } else {
                    let old_height = root.children[run_start].height;
                    layout_table(&mut root.children[run_start], doc, styles);
                    reflow_siblings_after_table_height_change(root, run_start, old_height);
                    for child in &mut root.children[run_start].children {
                        adjust_table_layout_inner(child, doc, styles, true);
                    }
                }
            } else if child_is_real_table {
                let old_height = root.children[idx].height;
                layout_table(&mut root.children[idx], doc, styles);
                reflow_siblings_after_table_height_change(root, idx, old_height);
                for child in &mut root.children[idx].children {
                    adjust_table_layout_inner(child, doc, styles, true);
                }
                idx += 1;
            } else {
                adjust_table_layout_inner(&mut root.children[idx], doc, styles, inside_table);
                idx += 1;
            }
        }
    }
}

/// CSS2 §17.2.1.1：把连续孤立 table-internal 兄弟合并到一个匿名 table 包装盒。
///
/// `root.children[run_start..run_start+run_len]` 是连续的孤立 table-internal 兄弟
/// （如两个 `display:table-row-group`）。本函数把它们 drain 到一个新的匿名 table
/// `LayoutBox`（无 node_id，`is_anon_table_root=true`），插入到 `run_start` 处，
/// 对其执行 `layout_table`（build_grid 正常路径收集多个 row-group → 多行堆叠），
/// 并按 run 原始垂直 footprint 调整后续兄弟位置 + 父高度。
///
/// 返回插入的包装盒在 `root.children` 中的索引。
fn merge_orphan_table_run(
    root: &mut LayoutBox,
    run_start: usize,
    run_len: usize,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> usize {
    let run_end = run_start + run_len;
    // 1. run 原始垂直 footprint（max bottom − min top，正确反映重叠/堆叠）。
    let (min_top, max_bottom) = root.children[run_start..run_end]
        .iter()
        .fold((f32::MAX, f32::MIN), |(mn, mx), c| {
            (mn.min(c.y), mx.max(c.y + c.height))
        });
    let old_footprint = (max_bottom - min_top).max(0.0);
    // 2. 继承首位 child 的位置/宽 + 父 writing_mode。
    let (x, y, width) = {
        let first = &root.children[run_start];
        (first.x, first.y, first.width)
    };
    let wm = root.writing_mode.clone();
    // 3. drain run → 匿名 table 包装盒。
    let run_children: Vec<LayoutBox> = root.children.drain(run_start..run_end).collect();
    let mut wrapper = LayoutBox::default();
    wrapper.is_anon_table_root = true;
    wrapper.is_block_level = true;
    wrapper.writing_mode = wm;
    wrapper.x = x;
    wrapper.y = y;
    wrapper.width = width;
    wrapper.children = run_children;
    root.children.insert(run_start, wrapper);
    // 4. layout_table 包装盒（build_grid 正常路径：多 row-group → 多行堆叠）。
    let widx = run_start;
    layout_table(&mut root.children[widx], doc, styles);
    // 5. 按 footprint 差异调整后续兄弟 + 父高度。
    let new_height = root.children[widx].height;
    let delta = new_height - old_footprint;
    if delta.abs() > 0.01 {
        for sibling in root.children.iter_mut().skip(widx + 1) {
            if sibling.is_absolute || sibling.is_fixed || !matches!(sibling.float, FloatValue::None) {
                continue;
            }
            sibling.y += delta;
        }
        root.height += delta;
        let pb = root.padding_top + root.padding_bottom + root.border_top + root.border_bottom;
        root.content_height = (root.height - pb).max(0.0);
    }
    widx
}

/// taffy 将 table 映射为 block，高度常大于 table 后处理算出的真实高度。
/// 收缩 table 后需把后续普通流兄弟上移（与 inline_finalization 的 shrink 重排同谱系）。
fn reflow_siblings_after_table_height_change(parent: &mut LayoutBox, table_idx: usize, old_table_height: f32) {
    if !matches!(parent.writing_mode, WritingModeValue::HorizontalTb) {
        return;
    }
    let height_delta = parent.children[table_idx].height - old_table_height;
    if height_delta.abs() <= 0.01 {
        return;
    }
    for sibling in parent.children.iter_mut().skip(table_idx + 1) {
        if sibling.is_absolute || sibling.is_fixed || !matches!(sibling.float, FloatValue::None) {
            continue;
        }
        sibling.y += height_delta;
    }
    parent.height += height_delta;
    let padding_border = parent.padding_top + parent.padding_bottom + parent.border_top + parent.border_bottom;
    parent.content_height = (parent.height - padding_border).max(0.0);
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
    // α-4b-1：vertical-rl/lr 表走转置路径（行沿 x、cell 沿 y），
    // horizontal-tb 走原路径（WM gate，字节一致零回归）。
    if matches!(
        table_box.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    ) {
        position_cells_vertical(table_box, &grid, &col_widths, spacing_x, spacing_y, styles);
    } else {
        position_cells(table_box, &grid, &col_widths, spacing_x, spacing_y, styles);
    }

    // R767: 列定尺寸后，cell content（width:auto block 子树）仍为 taffy 初始（body 宽）
    // 布局宽度，约束到 cell content width（仅 max-content 装得下的非 wrapping 内容安全；
    // wrapping 内容须 re-layout，跳过避 clip）。修 margin-collapse-101 等的 div w=778 溢出。
    crate::table_cell_content::constrain_table_cell_content_widths(table_box, doc, styles);

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

/// 根据 grid 结构和列宽定位每个单元格。
fn position_cells(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    col_widths: &[f32],
    spacing_x: f32,
    spacing_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // 表格内容宽度：auto-width（shrink-to-fit）表须用列宽之和 + spacing，而非
    // table_box.content_width（taffy 对 auto 表拉伸到容器宽，如 784）。apply_table_size_constraints
    // 后表盒会收缩到该值，但行/单元格在此处（收缩前）定位，须用同一收缩值，否则行（背景）
    // 保持拉伸宽而表盒收缩（anonymous-table-cell-margin-collapsing 的绿行 784 vs 表 ~110）。
    // explicit-width 表：compute_column_widths 已把 col 扩展到 explicit，col 和 ≈ content_width，
    // min 取 content_width，无变化。
    let col_sum: f32 = col_widths.iter().sum();
    let spacing_total_x = if col_widths.len() > 1 {
        (col_widths.len() - 1) as f32 * spacing_x
    } else {
        0.0
    };
    // CSS 2.1 §17.6.1（separated borders model）：border-spacing 不仅分隔相邻单元格，
    // 还构成表格四边的「周界 spacing」（外缘 cell 与 table 边缘之间）。旧实现只计列间
    // spacing（spacing_total_x），漏掉了左右周界（各 spacing_x）和上下周界（各 spacing_y），
    // 致使带 border-spacing 的表尺寸偏小、cell 紧贴 table 边缘（visibility-collapse-
    // border-spacing-002：1 列 100px + spacing 50 → 应 200px，旧实现 100px）。仅
    // separated 模式生效（collapse 模式忽略 border-spacing）。
    let separated = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.border_collapse, zero_style_system::BorderCollapseValue::Separate));
    let (perimeter_x, perimeter_y) = if separated { (spacing_x, spacing_y) } else { (0.0, 0.0) };
    let table_content_width = table_box
        .content_width
        .min(col_sum + spacing_total_x + 2.0 * perimeter_x);

    // R89：表格行高分配（CSS 2.1 §17.5.3 — table 的 height 作为最小高度，
    // 额外高度按行均分到各行，使单元格增长、vertical-align 把内容压到分配后位置）。
    // 预计算每行内容高度，再根据 table 指定 height 计算每行的额外分配量。
    let row_extras: Vec<f32> = {
        let content_row_heights: Vec<f32> = grid
            .rows
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
                // visibility:collapse 的行高度为 0（CSS Tables §4.1），不取默认最小行高。
                if grid.collapsed_rows.get(row_idx).copied().unwrap_or(false) {
                    return 0.0;
                }
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
                // 空 cell 行高度 0（无 strut，见 position_cells 同步注释）
                h
            })
            .collect();
        let num_rows = content_row_heights.len();
        // R585/R586：行高分配目标 = table 的 used content height（CSS §10.4 clamp）。
        // 显式 height 受 min/max 约束（max cap、min floor，min 优先于 max）；
        // height:auto 时仅 min-height 作下限分配（max-height 只 cap box 不展开行）。
        // R585：min-height 展开（min-height-applies-to-013）；R586：max-height cap
        //（max-height-applies-to-013：height:3in + max-height:1in → 行展开到 96 而非 288，
        // 避免溢出被 apply_table_size_conditions cap 到 96 的 table box）。
        let target_content_h = table_box.node_id.and_then(|id| styles.get(&id)).and_then(|s| {
            use zero_css_parser::values::LengthValue;
            let h_px = match &s.height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            };
            let mn_px = match &s.min_height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            };
            let mx_px = match &s.max_height {
                LengthValue::Px(v) if *v != f64::INFINITY => Some(*v as f32),
                _ => None,
            };
            let target = match h_px {
                Some(h) => {
                    let mut t = h;
                    if let Some(mx) = mx_px {
                        t = t.min(mx);
                    }
                    if let Some(mn) = mn_px {
                        t = t.max(mn);
                    }
                    Some(t)
                }
                None => mn_px,
            };
            let pb = table_box.padding_top + table_box.padding_bottom + table_box.border_top + table_box.border_bottom;
            target.map(|t| (t - pb).max(0.0))
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

    let mut row_y = perimeter_y;
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

        // visibility:collapse 的行高度为 0（CSS Tables §4.1），不参与布局，
        // 也不分配 table height 的额外高度。
        let row_collapsed = grid.collapsed_rows.get(row_idx).copied().unwrap_or(false);
        let mut row_height = 0.0f32;
        if !row_collapsed {
            // 行的高度 = 其所有单元格的最大高度
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
            // 空行（单元格无内容）高度为 0——chromium 对空 cell 渲染 0px
            // （visibility-collapse-border-spacing-002 chromium-Oracle 实证），旧 20px
            // 最小行高 strut 致带 border-spacing 的空表/折叠行表尺寸偏大。
            // R89：应用表格指定 height 的行高分配（额外高度均分到行）
            row_height += row_extras.get(row_idx).copied().unwrap_or(0.0);
        }

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

        // 定位每个单元格（起始 x 含左侧周界 spacing，§17.6.1）
        let mut cell_x = perimeter_x;
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

            // table-cell 建立新的 BFC（CSS §9.4.1），其首个 in-flow 子元素的
            // margin-top 不应向上穿透单元格（§8.3.1：BFC 的 margin 不与子折叠），
            // 应作为单元格内的顶部留白保留。但 taffy 把单元格按普通 Block 布局，
            // 把首子 margin-top 折叠上提到 cell.margin_top；自定义表格布局忽略
            // 单元格 margin → margin 丢失，内容从 content-box 顶（y=0）开始，
            // 而把等量空白留在了单元格底部（cell_content_height 已计入该 margin）。
            // 将内容子树整体下移 cell.margin_top，把顶部留白从底部移到顶部，
            // 对齐 Chromium。底部 margin（cell.margin_bottom）仍自然留作底部空白。
            let top_gap = cell_box.margin_top;
            if top_gap > 0.0 {
                for child in &mut cell_box.children {
                    child.y += top_gap;
                }
            }

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

        // 折叠行不贡献与相邻行之间的 border-spacing（CSS Tables §4.1）。
        // row_height 对折叠行为 0，故仅折叠行跳过 spacing_y。
        row_y += row_height + if row_collapsed { 0.0 } else { spacing_y };
    }

    // 后处理：应用 min-height/max-height/min-width/max-width 约束
    apply_table_size_constraints(table_box, grid, row_y, col_widths, spacing_y, styles);

    // 后处理：更新行组（tbody/thead/tfoot）的位置以包含其所有行
    // 对于 position:relative 的行组，还需应用 inset 偏移
    update_row_group_positions(table_box, grid, styles);

    // 后处理：caption-side:bottom——将标题移到表格底部
    // 当前 caption 由 taffy 作为 table 子元素布局，默认在顶部。
    // caption-side:bottom 时需移到 row_y（表格内容总高）之后。
    for child in &mut table_box.children {
        if let Some(child_id) = child.node_id
            && let Some(child_style) = styles.get(&child_id)
            && child_style.display == DisplayValue::TableCaption
            && child_style.caption_side == zero_style_system::property::CaptionSideValue::Bottom
        {
            child.y = row_y;
        }
    }
}

/// α-4b-1（RFC `vertical-mode-table-rl-transpose-rfc.md` §4.1）：对 `writing-mode:
/// vertical-rl/lr` 的表，把行/cell 定位从 horizontal-tb **转置** 到 vertical——
/// 行沿 x（vertical-rl 右到左 / vertical-lr 左到右），cell 沿 y 顶到底。
///
/// `col_widths`（逻辑列宽，WM-agnostic）映射为 cell 的 y 高度；行的 x 宽 = 该行
/// cell 内容宽的最大值。WM gate：仅 vertical-rl/lr 触发，horizontal-tb 走原
/// `position_cells`（字节一致零回归）。
///
/// **α-4b-1 范围**：简单表（无 colspan/rowspan）+ border-spacing 轴互换。
/// 有 colspan/rowspan 时回退到 `position_cells`（旧行为），留给 α-4b-2。
/// row-extras（table height 展开）/ vertical-align / caption-side 在转置轴的
/// 处理留给 α-4b-4。
fn position_cells_vertical(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    col_widths: &[f32],
    spacing_x: f32,
    spacing_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // α-4b-2：colspan 已支持（cell.height = Σ col_widths[col_start..col_end]）。
    // rowspan>1：build_grid 不跟踪 rowspan 占位（orphan_col_cursor 每行重置），
    // horizontal position_cells 也不做 rowspan 跨行（cell 仅取首行 row_height）——
    // 即 ZW 的 rowspan 支持本就是 partial（列分配正确、跨行高度不延展）。
    // 故 vertical 路径对 rowspan cell 按单行 x 宽处理（与 horizontal 同局限），
    // 不再回退——转置结构正确（行沿 x、cell 沿 y）比 horizontal fallback（87% 发散）
    // 显著更接近 chromium。
    let is_rl = matches!(table_box.writing_mode, WritingModeValue::VerticalRl);

    // 转置 spacing：vertical 下 spacing_x（inline 轴）→ y 方向（cell 间），
    // spacing_y（block 轴）→ x 方向（行间）。周界 spacing 同步轴换。
    let separated = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.border_collapse, zero_style_system::BorderCollapseValue::Separate));
    let (perim_block, perim_inline) = if separated { (spacing_y, spacing_x) } else { (0.0, 0.0) };

    // 表 inline 跨度（沿 y）= Σ col_widths + cell 间 gap + 上下周界。
    // 这是每行（垂直列）的 y 高度。
    let n_cols = col_widths.len();
    let inline_gaps = if n_cols > 1 {
        (n_cols - 1) as f32 * spacing_x
    } else {
        0.0
    };
    let base_inline_extent: f32 = col_widths.iter().sum::<f32>() + inline_gaps + 2.0 * perim_inline;

    // α-4b-4 row_extras（TBD-4）：vertical 表的 `height` 属性是物理 y = inline 跨度
    //（horizontal 的 height 是 block 跨度，沿 y 分配到行；vertical 的 height 沿 y 分配到
    // cell）。若 style.height > base_inline_extent，把超额均分到各列（cell y 高），
    // 使表填满指定高度（如 row-progression-vrl-002 height:7em=140px，base=60 → 各列+27px）。
    // 仅处理 Px（% 随 WM 语义复杂，defer）；min/max clamp 同 horizontal。
    let target_inline: f32 = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| {
            use zero_css_parser::values::LengthValue;
            let h_px = match &s.height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            };
            let mn_px = match &s.min_height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => None,
            };
            let mx_px = match &s.max_height {
                LengthValue::Px(v) if *v != f64::INFINITY => Some(*v as f32),
                _ => None,
            };
            let mut t = h_px;
            if let Some(mx) = mx_px {
                t = t.map(|v| v.min(mx));
            }
            if let Some(mn) = mn_px {
                t = t.map(|v| v.max(mn));
            }
            // height:auto 时仅 min-height 作下限（同 horizontal apply_table_size_constraints）。
            t.or(mn_px)
        })
        .unwrap_or(base_inline_extent);
    let col_extra: f32 = if n_cols > 0 && target_inline > base_inline_extent {
        (target_inline - 2.0 * perim_inline - inline_gaps - col_widths.iter().sum::<f32>()) / n_cols as f32
    } else {
        0.0
    };
    // α-4b-6 vrl-only cap spike：vertical-**rl** 表 height 作 inline 约束，base>target 时等比缩小
    // col_widths 触发 IFC wrap。原 spike 对 vlr 也 cap 致 net-negative（vlr-003/009 回归）；
    // 本轮收窄到 vrl-only（cap 改善案 vrl-002/004，回归案全 vlr）。vlr 保留 height-as-minimum。
    let col_sum: f32 = col_widths.iter().sum();
    let vrl_cap_scale: Option<f32> = if is_rl && target_inline < base_inline_extent && col_sum > 0.0 {
        Some((target_inline - 2.0 * perim_inline - inline_gaps) / col_sum)
    } else {
        None
    };
    let row_inline_extent: f32 = if let Some(scale) = vrl_cap_scale {
        col_sum * scale + inline_gaps + 2.0 * perim_inline
    } else {
        base_inline_extent + n_cols as f32 * col_extra
    };

    // 先算每行的 block 尺寸（x 宽 = 该行 cell 内容宽最大值）。
    // extract_layout 已对 vertical 容器的 cell 盒做了 size 轴交换，cell.width 是
    // 转置后的 x 宽；取行内最大作为该垂直列的 x 宽。
    let row_block_sizes: Vec<f32> = grid
        .rows
        .iter()
        .enumerate()
        .map(|(row_idx, row)| {
            if grid.collapsed_rows.get(row_idx).copied().unwrap_or(false) {
                return 0.0;
            }
            let mut max_w = 0.0f32;
            if let Some(rb) = get_row_box(table_box, row) {
                for cell in &row.cells {
                    let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                        rb.children.get(rg_idx).and_then(|rg| rg.children.get(cell.child_index))
                    } else {
                        rb.children.get(cell.child_index)
                    };
                    if let Some(cb) = cell_box {
                        max_w = max_w.max(cb.width);
                    }
                }
            }
            max_w
        })
        .collect();

    // 表 block 跨度（沿 x）= Σ row_block_sizes + 行间 gap + 左右周界。
    let block_gaps_total: f32 = grid
        .rows
        .iter()
        .enumerate()
        .map(|(i, _)| if i > 0 { spacing_y } else { 0.0 })
        .sum::<f32>();
    let table_block_extent: f32 = row_block_sizes.iter().sum::<f32>() + block_gaps_total + 2.0 * perim_block;

    // 「直接 cell 的匿名行 = table 自身」标志（避免覆盖 table 盒自身几何）。
    let table_is_display_table = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::Table | DisplayValue::InlineTable));

    // 行沿 x 迭代：vertical-rl 从右到左（首行最右），vertical-lr 从左到右（首行最左）。
    let mut cur_block = perim_block; // vertical-lr 起始
    for (row_idx, row) in grid.rows.iter().enumerate() {
        let row_collapsed = grid.collapsed_rows.get(row_idx).copied().unwrap_or(false);
        let row_block_size = row_block_sizes[row_idx];

        // 行的 x 位置（相对 table content box）。
        let row_x = if is_rl {
            // vertical-rl：首行在最右。cur_block 从右缘递减。
            table_block_extent - cur_block - row_block_size
        } else {
            // vertical-lr：首行在最左。cur_block 从左缘递增。
            cur_block
        };

        // 设置行盒：宽 = 该列 x 宽，高 = inline 跨度（行铺满表高）。
        let row_box = get_row_box_mut(table_box, row);
        if let Some(row_box) = row_box {
            // 行自身 relative inset（沿 x/y，vertical 下 inset 语义已由 converter 交换）。
            let (row_rel_dx, row_rel_dy) = if row_box.is_relative {
                (
                    resolve_length_inset(row_box, styles, true),
                    resolve_length_inset(row_box, styles, false),
                )
            } else {
                (0.0, 0.0)
            };
            let is_direct_cell_row = row.is_anonymous && row.row_group_index.is_none() && table_is_display_table;
            if !is_direct_cell_row {
                row_box.x = row_x + row_rel_dx;
                row_box.y = row_rel_dy;
                row_box.width = row_block_size;
                row_box.height = row_inline_extent;
            }
        }

        // cell 沿 y 迭代：起始 y = 上周界，每个 cell 后 += col_width + cell_gap。
        let mut cell_y = perim_inline;
        for cell in &row.cells {
            let cell_box = if let Some(rg_idx) = cell.parent_rg_idx {
                get_row_box_mut(table_box, row)
                    .and_then(|rb| rb.children.get_mut(rg_idx))
                    .and_then(|rg| rg.children.get_mut(cell.child_index))
            } else {
                get_row_box_mut(table_box, row).and_then(|rb| rb.children.get_mut(cell.child_index))
            };
            let Some(cell_box) = cell_box else {
                continue;
            };

            // cell 在 inline 轴（y）的尺寸 = 跨的列宽之和（colspan>1 时多列）。
            // vertical 下「列」是 inline 轴（y）槽位，colspan 跨多列 → cell 占多段 y。
            // α-4b-6：vrl-only cap 触发时按 scale 缩小（base>height 的 vertical-rl 表）；
            // 否则用原 col_widths，row_extras 的 col_extra 在下方统一加。
            let mut cell_h = 0.0f32;
            let mut spanned_non_collapsed = 0usize;
            for col in cell.col_start..cell.col_end {
                if col < col_widths.len() {
                    cell_h += if let Some(scale) = vrl_cap_scale {
                        col_widths[col] * scale
                    } else {
                        col_widths[col]
                    };
                    if !(grid.collapsed_cols.get(col).copied().unwrap_or(false)) {
                        spanned_non_collapsed += 1;
                    }
                }
            }
            // 相邻非折叠列间加 spacing（沿 y）。
            if spanned_non_collapsed > 1 {
                cell_h += (spanned_non_collapsed - 1) as f32 * spacing_x;
            }
            // α-4b-4 row_extras：把 table height 超额均分到各列，colspan cell 按
            // 跨的非折叠列数累计 col_extra（与它占据的 y 槽位数成正比）。
            cell_h += col_extra * spanned_non_collapsed as f32;

            // cell 自身 relative inset。
            let (cell_rel_dx, cell_rel_dy) = if cell_box.is_relative {
                (
                    resolve_length_inset(cell_box, styles, true),
                    resolve_length_inset(cell_box, styles, false),
                )
            } else {
                (0.0, 0.0)
            };

            cell_box.x = cell_rel_dx; // cell 沿 x 相对行起点（=0），行已定位
            cell_box.y = cell_y + cell_rel_dy;
            cell_box.width = row_block_size; // cell 铺满列 x 宽
            cell_box.height = cell_h; // cell 沿 y 高 = 列宽

            // 同步 content_width/height（paint 用）。
            cell_box.content_width = (cell_box.width
                - cell_box.border_left
                - cell_box.border_right
                - cell_box.padding_left
                - cell_box.padding_right)
                .max(0.0);
            cell_box.content_height = (cell_box.height
                - cell_box.border_top
                - cell_box.border_bottom
                - cell_box.padding_top
                - cell_box.padding_bottom)
                .max(0.0);

            // cell 推进（沿 y）。折叠列不推进。
            let is_in_collapsed_col = cell.col_start < grid.collapsed_cols.len() && grid.collapsed_cols[cell.col_start];
            if !is_in_collapsed_col {
                cell_y += cell_h + spacing_x;
            }
        }

        if !row_collapsed {
            cur_block += row_block_size + spacing_y;
        }
    }

    // 表自身尺寸：content_width（block, x）与 content_height（inline, y）更新为转置值。
    table_box.content_width = table_block_extent;
    table_box.content_height = row_inline_extent;
    // border-box width/height 同步（含 border+padding）。
    table_box.width = table_block_extent
        + table_box.border_left
        + table_box.border_right
        + table_box.padding_left
        + table_box.padding_right;
    table_box.height = row_inline_extent
        + table_box.border_top
        + table_box.border_bottom
        + table_box.padding_top
        + table_box.padding_bottom;

    // 行组位置更新（与 horizontal 路径对称）。
    update_row_group_positions(table_box, grid, styles);

    // 注：caption-side 在 vertical WM 下的语义（物理 top/bottom vs 逻辑 block-start/end）
    // 是 α-4b-4 的 TBD——「physical bottom = y 末端」假设实测 net-negative（破 1 案，
    // caption-side-vlr-005 未改善且 -1 pass），已回退。vertical 表 caption 定位须按
    // 逻辑 block 轴（block-end = vertical-rl 左 / vertical-lr 右）处理，留 α-4b-4。
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
    // §17.6.1 周界 spacing：左右各 spacing_x，仅 separated 模式。intrinsic_height 已含
    // 上下周界（position_cells 的 row_y 起始 = perimeter_y，循环每行 +spacing_y 自然
    // 产生顶部+底部周界，collapsed 行跳过 spacing 故不破坏周界语义）。
    let perimeter_x = if matches!(style.border_collapse, zero_style_system::BorderCollapseValue::Separate) {
        spacing_x
    } else {
        0.0
    };
    let intrinsic_width = total_col_width + spacing_total_x + 2.0 * perimeter_x;
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
    // §17.6.1 周界 spacing（顶部），仅 separated 模式——与 position_cells 的 row_y 起始一致，
    // 否则行组位置会与行位置错位。
    let separated = table_box
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.border_collapse, zero_style_system::BorderCollapseValue::Separate));
    let perimeter_y = if separated { spacing_y } else { 0.0 };

    // 预计算所有行的 y 位置和高度（不可变借用阶段）
    let mut row_positions: Vec<(f32, f32)> = Vec::with_capacity(grid.rows.len());
    let mut row_y = perimeter_y;
    for (row_idx, row) in grid.rows.iter().enumerate() {
        // visibility:collapse 的行高度为 0（CSS Tables §4.1），与 position_cells 对齐。
        let row_collapsed = grid.collapsed_rows.get(row_idx).copied().unwrap_or(false);
        let row_height = if row_collapsed {
            0.0
        } else if let Some(rb) = get_row_box(table_box, row) {
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
            // 空 cell 行高度 0（无 strut，见 position_cells 同步注释）
            h
        } else {
            0.0
        };
        row_positions.push((row_y, row_height));
        // 折叠行不贡献与相邻行之间的 border-spacing（CSS Tables §4.1）。
        row_y += row_height + if row_collapsed { 0.0 } else { spacing_y };
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
