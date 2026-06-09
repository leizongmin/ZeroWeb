//! CSS Multi-column 布局算法。
//!
//! 由于 taffy 没有原生 multicol 支持，所有 multicol 容器在 taffy 中
//! 映射为 `Display::Block`。本模块作为后处理步骤，在 taffy 布局完成后
//! 对设置了 `column-count` 或 `column-width` 的容器内的子元素重新定位，
//! 实现多列布局。
//!
//! ## 支持的功能
//!
//! - `column-count` 固定列数
//! - `column-width` 最小列宽自动计算列数
//! - `column-gap` 列间距
//! - 子元素按列分配（均衡分配策略）
//! - `column-fill: auto` 顺序填充 + 列高限制
//! - 基础 column breaking（子元素超出列高时移至下一列）

use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{ColumnCountComputedValue, ColumnFillComputedValue, ColumnWidthComputedValue};

use crate::types::LayoutBox;

/// 对 LayoutBox 树执行 multi-column 布局后处理。
///
/// 遍历所有设置了 `column-count` 或 `column-width` 的容器，
/// 将其子元素按多列规则重新定位。
pub fn adjust_multicol_layout(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if let Some(style) = root.node_id.and_then(|id| styles.get(&id)) {
        let col_info = compute_column_info(style, root.content_width);
        if let Some(info) = col_info {
            layout_multicol(root, &info);
        }
    }

    // 递归处理子节点
    for child in &mut root.children {
        adjust_multicol_layout(child, styles);
    }
}

/// 计算列高限制（用于 column breaking 判断）。
///
/// 当 `column-fill: auto` 且容器有明确高度时，每列的最大高度等于容器内容高度。
/// 当 `column-fill: balance`（默认）时，列高无限制（均衡分配）。
fn column_height_limit(container: &LayoutBox, info: &ColumnInfo) -> f32 {
    if info.sequential_fill && container.content_height > 0.0 {
        container.content_height
    } else {
        0.0 // 无限制
    }
}

/// 多列布局计算信息。
struct ColumnInfo {
    /// 列数。
    count: usize,
    /// 单列宽度。
    column_width: f32,
    /// 列间距。
    gap: f32,
    /// 是否按顺序填充（column-fill: auto）。
    sequential_fill: bool,
}

/// 将 LengthValue 转换为像素值。
///
/// `container_width` 用于解析百分比单位。
/// 注意：em/rem/viewport 单位已在 computed style 阶段解析为 Px，
/// 此处仅处理可能残留的百分比和绝对单位。
fn length_to_px(value: &LengthValue, container_width: f32) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Percentage(p) => *p as f32 / 100.0 * container_width,
        LengthValue::Em(v) => *v as f32 * 16.0, // 回退：已由 computed.rs 解析
        LengthValue::Rem(v) => *v as f32 * 16.0,
        LengthValue::Vw(v) => *v as f32 * 8.0,
        LengthValue::Vh(v) => *v as f32 * 6.0,
        LengthValue::Auto | LengthValue::Calc(_) => 0.0,
        LengthValue::Vmin(v) => (*v as f32) * 6.0,
        LengthValue::Vmax(v) => (*v as f32) * 8.0,
        LengthValue::Ch(v) => *v as f32 * 8.0,
        LengthValue::FitContent(inner) => length_to_px(inner, container_width),
        LengthValue::MinContent | LengthValue::MaxContent => 0.0,
    }
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
fn compute_column_info(style: &ComputedStyle, container_width: f32) -> Option<ColumnInfo> {
    let gap = length_to_px(&style.column_gap, container_width);
    let sequential_fill = matches!(style.column_fill, ColumnFillComputedValue::Auto);

    // CSS Multi-column spec: column-width 是最小列宽（理想宽度）
    // column-count 是理想列数
    // 两者同时设置时，取能容纳的最大列数（不小于 column-count，不小于 column-width）

    let col_count_from_count = match &style.column_count {
        ColumnCountComputedValue::Auto => None,
        ColumnCountComputedValue::Number(n) => Some(*n as usize),
    };

    let col_width_hint = match &style.column_width {
        ColumnWidthComputedValue::Auto => None,
        ColumnWidthComputedValue::Length(l) => Some(length_to_px(l, container_width)),
    };

    match (col_count_from_count, col_width_hint) {
        (None, None) => None, // auto + auto → 无多列
        (Some(n), None) => {
            // 仅 column-count: N
            if n == 0 {
                return None;
            }
            let count = n;
            let column_width = compute_single_column_width(container_width, count, gap);
            Some(ColumnInfo {
                count,
                column_width,
                gap,
                sequential_fill,
            })
        }
        (None, Some(min_width)) => {
            // 仅 column-width: W
            if min_width <= 0.0 || container_width <= 0.0 {
                return None;
            }
            let count = compute_column_count(container_width, min_width, gap);
            if count <= 1 {
                return None;
            }
            let column_width = compute_single_column_width(container_width, count, gap);
            Some(ColumnInfo {
                count,
                column_width,
                gap,
                sequential_fill,
            })
        }
        (Some(n), Some(min_width)) => {
            // 两者都有：取较大列数，但列宽不小于 min_width
            if n == 0 || min_width <= 0.0 {
                return None;
            }
            let count_from_width = compute_column_count(container_width, min_width, gap);
            let count = n.max(count_from_width);
            if count == 0 {
                return None;
            }
            let column_width = compute_single_column_width(container_width, count, gap);
            Some(ColumnInfo {
                count,
                column_width,
                gap,
                sequential_fill,
            })
        }
    }
}

/// 计算列数：在 container_width 内能放多少列（每列至少 min_width 宽）。
fn compute_column_count(container_width: f32, min_width: f32, gap: f32) -> usize {
    if gap <= 0.0 {
        return (container_width / min_width).floor() as usize;
    }
    // n 列需要 (n-1) 个 gap
    // container_width >= n * min_width + (n-1) * gap
    // container_width + gap >= n * (min_width + gap)
    // n <= (container_width + gap) / (min_width + gap)
    let n = ((container_width + gap) / (min_width + gap)).floor() as usize;
    n.max(1)
}

/// 计算单列宽度：将容器宽度均分给 n 列（含 gap）。
fn compute_single_column_width(container_width: f32, count: usize, gap: f32) -> f32 {
    if count == 0 {
        return container_width;
    }
    let total_gap = if count > 1 { gap * (count - 1) as f32 } else { 0.0 };
    ((container_width - total_gap) / count as f32).max(0.0)
}

/// 对单个 multicol 容器执行布局。
///
/// 算法：
/// 1. 计算每个子元素的高度
/// 2. 将子元素分配到各列（考虑 column breaking）
/// 3. 定位每个子元素的 x/y 坐标
/// 4. 对超出列高的子元素进行 clip 处理
fn layout_multicol(container: &mut LayoutBox, info: &ColumnInfo) {
    if container.children.is_empty() || info.count == 0 {
        return;
    }

    // 收集非 absolute/fixed 的子元素索引和高度
    let child_info: Vec<(usize, f32)> = container
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_absolute && !c.is_fixed)
        .map(|(i, c)| (i, c.height + c.margin_top + c.margin_bottom))
        .collect();

    if child_info.is_empty() {
        return;
    }

    // 列高限制：当 column-fill: auto 且容器有明确高度时生效
    let height_limit = column_height_limit(container, info);

    // 根据 column-fill 模式分配子元素到各列
    let assignments = if info.sequential_fill && height_limit > 0.0 {
        // column-fill: auto — 顺序填充，考虑列高限制（column breaking）
        assign_children_to_columns_with_breaking(&child_info, info.count, height_limit)
    } else if info.sequential_fill {
        // column-fill: auto 但无明确高度限制
        assign_children_to_columns_sequential(&child_info, info.count, container.content_height)
    } else {
        // column-fill: balance — 均衡分配（默认行为）
        // 也应用 breaking，但使用内容总高度除以列数作为每列目标高度
        let target_height = if height_limit > 0.0 {
            height_limit
        } else {
            // 均衡模式：计算总内容高度，平均分配
            let total_height: f32 = child_info.iter().map(|(_, h)| *h).sum();
            let per_col = total_height / info.count as f32;
            // 给一些余量避免不必要的 breaking
            per_col * 1.1
        };
        assign_children_to_columns_with_breaking(&child_info, info.count, target_height)
    };

    // 定位子元素
    position_multicol_children(container, &assignments, info);
}

/// 带列高限制的顺序分配（column breaking 基础实现）。
///
/// 按文档顺序将子元素填入当前列，当子元素超出列高限制时移至下一列。
/// 这是 CSS Multi-column Layout §2 "column breaking" 的简化实现：
/// - 子元素整体移动到下一列（不拆分单个块级元素的内容）
/// - 单个超过列高的子元素保留在当前列（clip 处理）
fn assign_children_to_columns_with_breaking(
    children: &[(usize, f32)],
    col_count: usize,
    max_col_height: f32,
) -> Vec<Vec<(usize, f32)>> {
    let mut columns: Vec<Vec<(usize, f32)>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for &(child_idx, child_height) in children {
        // 如果当前列放不下这个子元素，且还有更多列可用，移到下一列
        if current_col_height + child_height > max_col_height && current_col_height > 0.0 && current_col + 1 < col_count
        {
            current_col += 1;
            current_col_height = 0.0;
        }

        columns[current_col].push((child_idx, child_height));
        current_col_height += child_height;
    }

    columns
}

/// 按顺序填充列（column-fill: auto）。
///
/// 子元素按文档顺序依次填入当前列，当列高度达到容器高度时移至下一列。
fn assign_children_to_columns_sequential(
    children: &[(usize, f32)],
    col_count: usize,
    container_height: f32,
) -> Vec<Vec<(usize, f32)>> {
    let mut columns: Vec<Vec<(usize, f32)>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for &(child_idx, child_height) in children {
        // 如果当前列放不下，且还有更多列可用，移到下一列
        if current_col_height + child_height > container_height
            && current_col_height > 0.0
            && current_col + 1 < col_count
        {
            current_col += 1;
            current_col_height = 0.0;
        }

        columns[current_col].push((child_idx, child_height));
        current_col_height += child_height;
    }

    columns
}

/// 根据列分配结果定位每个子元素。
///
/// 子元素坐标相对于容器 content area（与 taffy/float 后处理一致），
/// 因此列 x 从 0 开始，不需要加 content_x/content_y。
fn position_multicol_children(container: &mut LayoutBox, assignments: &[Vec<(usize, f32)>], info: &ColumnInfo) {
    for (col_idx, col_children) in assignments.iter().enumerate() {
        let col_x = col_idx as f32 * (info.column_width + info.gap);
        let mut y_offset = 0.0f32;

        for &(child_idx, child_total_height) in col_children {
            let child = &mut container.children[child_idx];

            // 设置子元素的 x 位置为列的 x（相对于 content area）
            child.x = col_x + child.margin_left;
            // y 位置：列内累积（相对于 content area）
            child.y = y_offset + child.margin_top;

            y_offset += child_total_height;

            // 限制子元素宽度不超过列宽
            if child.width > info.column_width {
                child.width = info.column_width;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_column_count_basic() {
        // 800px 容器, 200px 最小列宽, 0 gap → 4 列
        assert_eq!(compute_column_count(800.0, 200.0, 0.0), 4);
    }

    #[test]
    fn test_compute_column_count_with_gap() {
        // 800px 容器, 200px 最小列宽, 20px gap
        // n <= (800 + 20) / (200 + 20) = 820 / 220 = 3.72 → 3
        assert_eq!(compute_column_count(800.0, 200.0, 20.0), 3);
    }

    #[test]
    fn test_compute_single_column_width() {
        // 800px / 3 列, 20px gap
        // total_gap = 2 * 20 = 40
        // column_width = (800 - 40) / 3 = 253.33
        let w = compute_single_column_width(800.0, 3, 20.0);
        assert!((w - 253.333).abs() < 1.0);
    }

    #[test]
    fn test_assign_children_balanced() {
        // 4 children, each 100px high, 2 columns, large height limit
        // Sequential fill: all fit in col0 since total (400) < limit (1000)
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 2, 1000.0);
        assert_eq!(cols.len(), 2);
        // All in col0 since they all fit
        assert_eq!(cols[0].len(), 4);
        assert_eq!(cols[1].len(), 0);
    }

    #[test]
    fn test_assign_children_with_breaking() {
        // 4 children, each 100px high, 3 columns, 150px height limit
        // child0(100): col0=100 → col0=[0]
        // child1(100): col0=200 > 150, move to col1 → col1=[1]
        // child2(100): col1=200 > 150, move to col2 → col2=[2]
        // child3(100): col2=200 > 150, no more cols, stays → col2=[2,3]
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 150.0);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1);
        assert_eq!(cols[1].len(), 1);
        assert_eq!(cols[2].len(), 2); // last 2 overflow into col2
    }

    #[test]
    fn test_assign_children_with_breaking_oversized() {
        // Single child larger than column height — stays in current column
        let children = vec![(0, 300.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 100.0);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1); // oversized child stays in first column
    }
}
