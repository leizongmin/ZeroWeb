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

use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue};

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

/// 多列布局计算信息。
struct ColumnInfo {
    /// 列数。
    count: usize,
    /// 单列宽度。
    column_width: f32,
    /// 列间距。
    gap: f32,
}

/// 将 LengthValue 转换为像素值（简化版，不处理百分比和 auto）。
fn length_to_px(value: &LengthValue) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Em(v) => *v as f32 * 16.0, // 假设基准字号 16px
        LengthValue::Rem(v) => *v as f32 * 16.0,
        LengthValue::Vw(v) => *v as f32 * 8.0, // 简化：假设 800px 视口
        LengthValue::Vh(v) => *v as f32 * 6.0, // 简化：假设 600px 视口
        LengthValue::Percentage(_) | LengthValue::Auto | LengthValue::Calc(_) => 0.0,
        LengthValue::Vmin(v) => (*v as f32) * 6.0,
        LengthValue::Vmax(v) => (*v as f32) * 8.0,
        LengthValue::Ch(v) => *v as f32 * 8.0, // 简化：假设 ch ≈ 8px
        LengthValue::FitContent(inner) => length_to_px(inner),
        LengthValue::MinContent | LengthValue::MaxContent => 0.0,
    }
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
fn compute_column_info(style: &ComputedStyle, container_width: f32) -> Option<ColumnInfo> {
    let gap = length_to_px(&style.column_gap);

    // CSS Multi-column spec: column-width 是最小列宽（理想宽度）
    // column-count 是理想列数
    // 两者同时设置时，取能容纳的最大列数（不小于 column-count，不小于 column-width）

    let col_count_from_count = match &style.column_count {
        ColumnCountComputedValue::Auto => None,
        ColumnCountComputedValue::Number(n) => Some(*n as usize),
    };

    let col_width_hint = match &style.column_width {
        ColumnWidthComputedValue::Auto => None,
        ColumnWidthComputedValue::Length(l) => Some(length_to_px(l)),
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
/// 2. 将子元素均衡分配到各列（min-height-first 策略）
/// 3. 定位每个子元素的 x/y 坐标
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

    // 均衡分配子元素到各列
    let assignments = assign_children_to_columns(&child_info, info.count);

    // 定位子元素
    position_multicol_children(container, &assignments, info);
}

/// 将子元素均衡分配到各列。
///
/// 使用"最矮列优先"（shortest-column-first）策略：
/// 依次将每个子元素放入当前总高度最小的列。
fn assign_children_to_columns(children: &[(usize, f32)], col_count: usize) -> Vec<Vec<(usize, f32)>> {
    let mut columns: Vec<Vec<(usize, f32)>> = vec![Vec::new(); col_count];
    let mut col_heights = vec![0.0f32; col_count];

    for &(child_idx, child_height) in children {
        // 找到最矮的列
        let shortest_col = col_heights
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        columns[shortest_col].push((child_idx, child_height));
        col_heights[shortest_col] += child_height;
    }

    columns
}

/// 根据列分配结果定位每个子元素。
fn position_multicol_children(container: &mut LayoutBox, assignments: &[Vec<(usize, f32)>], info: &ColumnInfo) {
    let content_x = container.content_x;
    let content_y = container.content_y;

    for (col_idx, col_children) in assignments.iter().enumerate() {
        let col_x = content_x + col_idx as f32 * (info.column_width + info.gap);
        let mut y_offset = 0.0f32;

        for &(child_idx, child_total_height) in col_children {
            let child = &mut container.children[child_idx];

            // 设置子元素的 x 位置为列的 x
            // y 位置保持原有的流布局基础上，加列内偏移
            child.x = col_x + child.margin_left;
            // y 位置：列内累积
            child.y = content_y + y_offset + child.margin_top;

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
        // 4 children, each 100px high, 2 columns
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
        let cols = assign_children_to_columns(&children, 2);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].len(), 2);
        assert_eq!(cols[1].len(), 2);
    }

    #[test]
    fn test_assign_children_balanced_uneven() {
        // 3 children: 100, 200, 100 → best is col1: [100, 100], col2: [200]
        let children = vec![(0, 100.0), (1, 200.0), (2, 100.0)];
        let cols = assign_children_to_columns(&children, 2);
        assert_eq!(cols.len(), 2);
        // 最矮列优先：col1=[100], col2=[200], then col1=[100,100], col2=[200]
        let col1_total: f32 = cols[0].iter().map(|(_, h)| *h).sum();
        let col2_total: f32 = cols[1].iter().map(|(_, h)| *h).sum();
        assert!((col1_total - 200.0).abs() < 0.01);
        assert!((col2_total - 200.0).abs() < 0.01);
    }
}
