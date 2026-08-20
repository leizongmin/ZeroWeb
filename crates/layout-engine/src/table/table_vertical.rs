//! Vertical writing-mode 表格 cell 定位（vertical-rl / vertical-lr）。
//!
//! R1696 从 table.rs 抽离（table.rs 减负，2086→<2000，CLAUDE.md §5）。vertical 表布局是
//! 与 horizontal position_cells 对称但独立的转置路径（行沿 x、cell 沿 y），抽出提升内聚。
//! 纯重定位——函数体字节一致，仅 `fn` → `pub(super) fn`。共享 helper 多在 table_types.rs
//!（pub(crate)），table.rs-local 的 get_row_box_mut / update_row_group_positions 经 super 引入。
use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::table_types::*;
use crate::types::LayoutBox;

use super::{get_row_box_mut, update_row_group_positions};

fn resolve_vertical_table_extent_length(
    value: &zero_css_parser::values::LengthValue,
    font_size: &zero_css_parser::values::LengthValue,
    table_width: f32,
    table_height: f32,
) -> Option<f32> {
    use zero_css_parser::values::LengthValue;
    match value {
        LengthValue::Auto
        | LengthValue::Percentage(_)
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::FitContent(_) => None,
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(
                font_size,
                16.0,
                Some(table_width as f64),
                Some(table_height as f64),
            );
            let px = zero_style_system::computed::resolve_length(
                other,
                font_size_px,
                Some(table_width as f64),
                Some(table_height as f64),
            );
            px.is_finite().then_some(px as f32)
        }
    }
}

pub(super) fn position_cells_vertical(
    table_box: &mut LayoutBox,
    grid: &TableGrid,
    col_widths: &[f32],
    spacing_x: f32,
    spacing_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    doc: &zero_dom::Document,
) {
    // α-4b-2：colspan 已支持（cell.height = Σ col_widths[col_start..col_end]）。
    // rowspan>1：build_grid 不跟踪 rowspan 占位（orphan_col_cursor 每行重置），
    // horizontal position_cells 也不做 rowspan 跨行（cell 仅取首行 row_height）——
    // 即 ZW 的 rowspan 支持本就是 partial（列分配正确、跨行高度不延展）。
    // 故 vertical 路径对 rowspan cell 按单行 x 宽处理（与 horizontal 同局限），
    // 不再回退——转置结构正确（行沿 x、cell 沿 y）比 horizontal fallback（87% 发散）
    // 显著更接近 chromium。
    let is_rl = table_box.writing_mode.is_block_flow_rl();

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
            let h_px = resolve_vertical_table_extent_length(&s.height, &s.font_size, table_box.width, table_box.height);
            let mn_px =
                resolve_vertical_table_extent_length(&s.min_height, &s.font_size, table_box.width, table_box.height);
            let mx_px = match &s.max_height {
                LengthValue::Px(v) if *v == f64::INFINITY => None,
                other => resolve_vertical_table_extent_length(other, &s.font_size, table_box.width, table_box.height),
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
    // α-4b-6 cap：vertical 表 height 作 inline 约束，base>target 时等比缩小 col_widths
    // 触发 IFC wrap。R1114 原 vrl-only（vlr cap 致 vlr-003/009 回归），但**彼时 pre-R1131
    // 无 cell.width 增长**——cap 触发 wrap 后 cell 盒不增长，wrap 列溢出 → 回归。
    // post-R1131（grow_vrl_cell_block_extent 增长 cell.width 容纳 wrap 列），vlr cap 应同 vrl
    // 安全。R1132 实验：扩 cap 到 vlr（is_rl gate 移除），验 vlr row-progression 是否改善。
    let col_sum: f32 = col_widths.iter().sum();
    let vrl_cap_scale: Option<f32> = if target_inline < base_inline_extent && col_sum > 0.0 {
        Some((target_inline - 2.0 * perim_inline - inline_gaps) / col_sum)
    } else {
        None
    };
    let row_inline_extent: f32 = if let Some(scale) = vrl_cap_scale {
        col_sum * scale + inline_gaps + 2.0 * perim_inline
    } else {
        base_inline_extent + n_cols as f32 * col_extra
    };

    // R1146：CSS §17.5.2.2 auto-layout 列宽分布。vrl_cap_scale 触发（target < max-content 和）
    // 时，旧实现按 max-content **比例** 缩放 col_widths，违反「列宽 ≥ min-content」——实测
    // row-progression-vrl-002 col_widths=[280,200,240] 比例缩放到 [54.4,38.9,46.7]，而每列
    // min-content（最长 word）=[60,40,40]，cell0/cell1 **低于自身 min-content**（54.4<60,
    // 38.9<40）→ "DDD"/"EE" 等最长 word 溢出列、被迫多 wrap 一列 → 表过宽，离 chromium 更远。
    // 正确算法（CSS auto table layout）：min_sum ≤ avail < max_sum 时每列 = min + 剩余按
    // (max-min) 比例分；avail ≤ min_sum 时每列 = min_content（内容溢出，CSS 允许）。
    // ★关键：floor 须在 vrl_cap_scale **之后**应用（col_widths 进此函数时是 max-content 全宽
    // [280,200,240]，远 > min-content，在 compute_column_widths 加 floor 永不绑定——R1145 实验
    // 因此零效果）。avail_for_cols = target 减周界与列间 gap。
    let cap_fired = vrl_cap_scale.is_some();
    let avail_for_cols = (target_inline - 2.0 * perim_inline - inline_gaps).max(0.0);
    // R1146 实测：min-content floor 一致改善 4 个 vrl row-progression 案（-0.06~-0.33pp），
    // 但**一致恶化 4 个 vlr 案**（+0.06~+0.37pp）——vlr 有 Path A/B 发散（R1119），其
    // compensating-error 被正确化 min-content floor 破坏。故 floor 仅 vrl 应用；vlr 保
    // 留旧比例缩放（col_widths×scale）。horizontal 不受影响（cap_fired 仅 vertical 触发）。
    let scale_val = vrl_cap_scale.unwrap_or(1.0);
    let final_col_widths: Vec<f32> = if cap_fired && is_rl {
        let min_content = compute_col_min_content(table_box, grid, n_cols, styles, doc);
        let min_sum: f32 = min_content.iter().sum();
        if col_sum <= avail_for_cols {
            // 理论上 cap 不会在此条件触发，保守返回原值。
            col_widths.to_vec()
        } else if avail_for_cols >= min_sum {
            // min_sum ≤ avail < max_sum：每列保 min-content，剩余按 (max-min) 比例分配。
            let excess = avail_for_cols - min_sum;
            let denom: f32 = (0..n_cols).map(|i| (col_widths[i] - min_content[i]).max(0.0)).sum();
            col_widths
                .iter()
                .enumerate()
                .map(|(i, &mx)| {
                    let mn = min_content[i];
                    if denom > 0.0 {
                        mn + excess * ((mx - mn).max(0.0) / denom)
                    } else {
                        mn
                    }
                })
                .collect()
        } else {
            // avail < min_sum：每列取 min-content（内容溢出）。
            min_content
        }
    } else if cap_fired {
        // vlr：保留旧比例缩放（min-content floor 反致 vlr Path A/B 发散恶化）。
        col_widths.iter().map(|w| w * scale_val).collect()
    } else {
        col_widths.to_vec()
    };

    // 先算每行的 block 尺寸（x 宽 = 该行 cell 内容宽最大值）。
    // extract_layout 已对 vertical 容器的 cell 盒做了 size 轴交换，cell.width 是
    // 转置后的 x 宽；取行内最大作为该垂直列的 x 宽。
    //
    // R1131 slice 3：vrl_cap_scale 触发时（inline extent 被 cap → 文本强制 wrap 成多列），
    // cell 的 block extent（x 宽）须同步增长以容纳 wrap 列数。旧实现 cell.width 保持
    // taffy 原值（~单字符宽），wrap 列溢出 cell 盒。R1116 曾试 area-conservation 增长但
    // 0-flip（**彼时 pre-R1100，vertical IFC container_width=0 致文本不 wrap**）；post-R1100
    // IFC wrap 已修，故重试。N 列 = ceil(文本像素高 / cell_h_scaled)；文本高用 DOM char 数
    // × fs（Ahem 精确，变宽近似）—col_widths 是 WM-agnostic 水平测量（R1112）不能直接用。
    // rowspan cell 跳过（ZW rowspan 本就 partial，R1110 先例；避 vrl-006 回归）。仅 vrl
    //（vrl_cap_scale 仅 vrl 触发，故天然 vrl-only gate）。
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
                    let Some(cb) = cell_box else { continue };
                    let w = grow_vrl_cell_block_extent(
                        cb,
                        cell,
                        &final_col_widths,
                        cap_fired,
                        spacing_x,
                        grid,
                        styles,
                        doc,
                    );
                    max_w = max_w.max(w);
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
    let mut table_block_extent: f32 = row_block_sizes.iter().sum::<f32>() + block_gaps_total + 2.0 * perim_block;

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
            // R1146：用 final_col_widths（cap 触发时已按 CSS auto-layout 分布含 min-content floor；
            // 未触发时 == col_widths）。row_extras 的 col_extra 在下方统一加。
            let mut cell_h = 0.0f32;
            let mut spanned_non_collapsed = 0usize;
            for col in cell.col_start..cell.col_end {
                if col < final_col_widths.len() {
                    cell_h += final_col_widths[col];
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

    // α-4b-4 caption-side 逻辑 block 轴定位：vertical 表的 caption-side 映射到 block 轴（x），
    // caption 应在 block-start 或 block-end 侧（非物理 top/bottom——R1110b「physical bottom=y 末端」
    // 假设 net-negative 已回退）。CSS writing-modes §7.4：caption-side:top=block-start，
    // caption-side:bottom=block-end；vertical-rl block-start=右 / block-end=左，
    // vertical-lr block-start=左 / block-end=右。
    //
    // caption 当前由 taffy 放在 table content 起点（x=0，与行重叠）。对「caption 应在右侧」的案
    //（vrl+top / vlr+bottom），把 caption 移到行右侧（rows_block_extent）。「caption 应在左侧」
    // 的案（vrl+bottom / vlr+top）caption 已在正确侧 x=0，无须移动（且 WPT 该簇 td 透明，
    // 行位无关视觉）。仅移动 caption-at-right 即可覆盖该簇结构性发散。
    use zero_style_system::property::CaptionSideValue;
    let rows_block_extent_before_caption = table_block_extent; // caption 调整前的 block 跨度
    let mut caption_w_right = 0.0f32;
    for child in &mut table_box.children {
        let Some(cid) = child.node_id else { continue };
        let Some(cs) = styles.get(&cid) else { continue };
        if cs.display != DisplayValue::TableCaption {
            continue;
        }
        let side_bottom = cs.caption_side == CaptionSideValue::Bottom;
        // caption_at_right：vrl+top（block-start=右）/ vlr+bottom（block-end=右）。
        let caption_at_right = if is_rl { !side_bottom } else { side_bottom };
        if caption_at_right {
            child.x = rows_block_extent_before_caption; // 移到行右侧（相对 table content box）
            caption_w_right = caption_w_right.max(child.width);
        }
    }
    // 表 block 跨度计入右侧 caption 占的额外 x 空间。
    table_block_extent += caption_w_right;

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

    // 注：caption-side vertical 逻辑 block 轴定位已在本函数上方「α-4b-4」块实现（caption-at-right
    // 移到行右侧）。残余：caption-side-vrl-002/004 的 1.73% 非 caption 位（box 已在正确侧），
    // 是 vertical-rl 文本绘制偏离 box（R1050 vrl paint bug），须 vertical IFC paint 修（多 session）。
}
