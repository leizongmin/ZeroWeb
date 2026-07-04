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
//! - column breaking（子元素超出列高时拆分到多列显示）
//!
//! ## Column Breaking 实现原理
//!
//! 当一个子元素的高度超过列高限制时，需要将其拆分到多个列中显示。
//! 拆分不是真正地将 LayoutBox 树枝剪为多个节点，而是通过「垂直窗口」
//! 机制实现：同一个子元素在多个列中出现，每列显示其不同高度切片。
//!
//! 具体做法：
//! - 分配阶段为超高的子元素创建多个 ColumnFragment（每列一个）
//! - 定位阶段为每个片段设置 y_offset，使子元素在列内向上平移
//! - paint 层通过容器的 overflow 裁剪，每列只显示该片段对应的高度范围

use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{
    BreakValue, ColumnCountComputedValue, ColumnFillComputedValue, ColumnWidthComputedValue,
};

use crate::types::LayoutBox;

/// 列分配中的一个片段。
///
/// 对于普通（未拆分的）子元素，一个子元素对应一个片段。
/// 对于超高子元素的 column breaking，一个子元素可能出现在多列中，
/// 每列一个片段，每片显示子元素的不同垂直范围。
#[derive(Debug, Clone)]
struct ColumnFragment {
    /// 子元素在容器 children 中的索引。
    child_idx: usize,
    /// 该片段对应子元素内容中可见部分的起始 y 偏移。
    /// 定位时子元素 y 坐标 = 列内累积高度 - fragment_y_offset，
    /// 使得只有 fragment_y_offset 到 fragment_y_offset + max_col_height 的内容可见。
    fragment_y_offset: f32,
    /// 该片段在列内占用的视觉高度（= min(child_remaining_height, max_col_height)）。
    visual_height: f32,
}

/// 对 LayoutBox 树执行 multi-column 布局后处理。
///
/// 遍历所有设置了 `column-count` 或 `column-width` 的容器，
/// 将其子元素按多列规则重新定位。
pub fn adjust_multicol_layout(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if let Some(style) = root.node_id.and_then(|id| styles.get(&id)) {
        let col_info = compute_column_info(style, root.content_width);
        if let Some(info) = col_info {
            root.column_gap = info.gap;
            layout_multicol(root, &info, styles);
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
pub(crate) struct ColumnInfo {
    /// 列数。
    pub count: usize,
    /// 单列宽度。
    pub column_width: f32,
    /// 列间距。
    pub gap: f32,
    /// 是否按顺序填充（column-fill: auto）。
    pub sequential_fill: bool,
}

/// 将 LengthValue 转换为像素值。
///
/// `container_width` 用于解析百分比单位；`font_size_px` 用于解析 em 单位。
/// 注意：多数 length 属性的 em/rem 已在 computed style 阶段解析为 Px，此处仅处理
/// 可能残留的百分比和绝对单位。但 `column-width`/`column-gap` 等 multicol 属性的 apply
/// 不解析 em（存 `Length(Em(v))`），故本函数须按 **element font-size** 解析 em——
/// R904 修复：旧实现硬编码 `v*16.0`（root）致 column-width:2em 在 font-size:1.25em(20px)
/// 容器内解析为 32px（应 40px），multicol-break-001 列数 6（应 5）oracle 1.06%。
fn length_to_px(value: &LengthValue, container_width: f32, font_size_px: f32) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Percentage(p) => *p as f32 / 100.0 * container_width,
        LengthValue::Em(v) => *v as f32 * font_size_px,
        LengthValue::Rem(v) => *v as f32 * 16.0,
        LengthValue::Vw(v) => *v as f32 * 8.0,
        LengthValue::Vh(v) => *v as f32 * 6.0,
        LengthValue::Auto | LengthValue::Calc(_) => 0.0,
        LengthValue::Vmin(v) => (*v as f32) * 6.0,
        LengthValue::Vmax(v) => (*v as f32) * 8.0,
        LengthValue::Ch(v) => *v as f32 * 8.0,
        LengthValue::FitContent(inner) => length_to_px(inner, container_width, font_size_px),
        LengthValue::MinContent | LengthValue::MaxContent => 0.0,
    }
}

/// 返回 balance 模式多列容器的（列宽, 列数），供 remeasure 按列宽测量行内内容
/// 并计算分布式高度。仅 `column-fill: balance`（默认）返回 `Some`。
pub(crate) fn balance_column_geometry(style: &ComputedStyle, container_width: f32) -> Option<(f32, usize)> {
    let info = compute_column_info(style, container_width)?;
    if info.sequential_fill || info.count < 2 {
        return None;
    }
    Some((info.column_width, info.count))
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
pub(crate) fn compute_column_info(style: &ComputedStyle, container_width: f32) -> Option<ColumnInfo> {
    // em 单位按 element font-size 解析（R904：column-width/column-gap apply 不解析 em）。
    let font_size_px = match &style.font_size {
        LengthValue::Px(v) => *v as f32,
        _ => 16.0, // computed font_size 应为 Px；防御性回退
    };
    let gap = length_to_px(&style.column_gap, container_width, font_size_px);
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
        ColumnWidthComputedValue::Length(l) => Some(length_to_px(l, container_width, font_size_px)),
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
            // CSS Multi-column Layout §3.4 伪算法（line 13-19）：
            // 当 column-width >= available-width 时，N=1
            // 否则 N = min(column-count, floor((U + gap) / (W + gap)))
            // 即取 column-count 和 column-width 限制列数中的较小值。
            if n == 0 || min_width <= 0.0 {
                return None;
            }
            if min_width >= container_width {
                // column-width 大于等于容器宽度 → 仅一列
                return Some(ColumnInfo {
                    count: 1,
                    column_width: container_width,
                    gap,
                    sequential_fill,
                });
            }
            let count_from_width = compute_column_count(container_width, min_width, gap);
            let count = n.min(count_from_width);
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
fn layout_multicol(container: &mut LayoutBox, info: &ColumnInfo, styles: &HashMap<NodeId, ComputedStyle>) {
    if container.children.is_empty() || info.count == 0 {
        return;
    }

    // 收集非 absolute/fixed 的子元素索引、高度，以及 break-before/after:column 标志
    //（R903：消费此前死值 break_before，强制换列；R1027：mirror 到 break_after）。
    let mut child_info: Vec<(usize, f32)> = Vec::new();
    let mut forced_breaks: Vec<bool> = Vec::new();
    let mut forced_breaks_after: Vec<bool> = Vec::new();
    for (i, c) in container.children.iter().enumerate() {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        child_info.push((i, c.height + c.margin_top + c.margin_bottom));
        let style = c.node_id.and_then(|id| styles.get(&id));
        let force = style.is_some_and(|s| matches!(s.break_before, BreakValue::Column | BreakValue::Page));
        forced_breaks.push(force);
        let force_after = style.is_some_and(|s| matches!(s.break_after, BreakValue::Column | BreakValue::Page));
        forced_breaks_after.push(force_after);
    }

    if child_info.is_empty() {
        return;
    }

    // 列高限制：当 column-fill: auto 且容器有明确高度时生效
    let height_limit = column_height_limit(container, info);

    // 根据 column-fill 模式分配子元素到各列
    let assignments = if info.sequential_fill && height_limit > 0.0 {
        // column-fill: auto — 顺序填充，考虑列高限制（column breaking）
        assign_children_to_columns_with_breaking(
            &child_info,
            info.count,
            height_limit,
            &forced_breaks,
            &forced_breaks_after,
        )
    } else if info.sequential_fill {
        // column-fill: auto 但无明确高度限制
        assign_children_to_columns_sequential(
            &child_info,
            info.count,
            container.content_height,
            &forced_breaks,
            &forced_breaks_after,
        )
    } else {
        // column-fill: balance — 均衡分配（默认行为）
        // CSS Multi-column Layout §3.3：内容应均衡分布在各列中。
        // 使用 shortest-column-first 策略：每个子元素放入当前最短的列。
        // 这自然实现均衡分布，无需人工设置 target_height。
        assign_children_to_columns_balanced(&child_info, info.count, &forced_breaks, &forced_breaks_after)
    };

    // 定位子元素
    position_multicol_children(container, &assignments, info);
}

/// 均衡分配子元素到各列（顺序流 + 目标高度策略）。
///
/// CSS Multi-column Layout §3.3：在 column-fill: balance（默认）模式下，
/// 内容应尽可能均衡地分布在各列中。内容按文档顺序依次填入各列。
///
/// 算法：
/// 1. 计算所有子元素的总高度
/// 2. 目标列高 = 总高度 / 列数
/// 3. 按文档顺序将子元素填入当前列，当列高超过目标时移至下一列
///
/// 这比 shortest-column-first 更符合规范行为：内容按顺序流过各列，
/// 而非被任意分配到最短列。
fn assign_children_to_columns_balanced(
    children: &[(usize, f32)],
    col_count: usize,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    if children.is_empty() || col_count == 0 {
        return vec![Vec::new(); col_count.max(1)];
    }

    // 计算总高度和目标列高
    let total_height: f32 = children.iter().map(|&(_, h)| h).sum();
    let target_height = total_height / col_count as f32;

    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
        // 如果当前列已超过目标高度且还有更多列可用，移到下一列
        if current_col_height >= target_height && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }

        columns[current_col].push(ColumnFragment {
            child_idx,
            fragment_y_offset: 0.0,
            visual_height: child_height,
        });
        current_col_height += child_height;
        // break-after:column：放置完子元素后强制推进到下一列（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
    }

    columns
}

/// 带列高限制的顺序分配（column breaking 实现）。
///
/// 按文档顺序将子元素填入当前列，当子元素超出列高限制时：
/// - 如果子元素可以整体放入下一列，则移动到下一列
/// - 如果子元素本身超过列高（oversized），则拆分为多个片段，
///   每个片段放入连续的列中
///
/// CSS Multi-column Layout §2 "column breaking"：
/// 当一个块级子元素高度超过列高时，内容应自动延续到后续列中。
fn assign_children_to_columns_with_breaking(
    children: &[(usize, f32)],
    col_count: usize,
    max_col_height: f32,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
        let available = max_col_height - current_col_height;

        if child_height <= available {
            // 子元素完全适应当前列剩余空间
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else if child_height <= max_col_height {
            // 子元素可以整体放入下一列（当列剩余不够但列高足够）
            if current_col + 1 < col_count {
                current_col += 1;
                current_col_height = 0.0;
            }
            // 如果没有更多列，保留在当前列（clip 处理）
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height.min(max_col_height),
            });
            current_col_height += child_height.min(max_col_height);
        } else {
            // 子元素超高（> max_col_height）— 需要 column breaking
            // 先消耗当前列剩余空间
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
                // 仅当还有更多列时才推进；单列或末列时保留在当前列（clip 处理），
                // 否则 current_col 越界使后续子元素 columns[current_col].push panic。
                if current_col + 1 < col_count {
                    current_col += 1;
                }
            }

            // 后续片段填满整列。
            // max_col_height > 0.0 守卫：若列高为 0（height:0 multicol 或计算得 0），
            // offset += max_col_height(0) 永不前进会无限循环——此时无法细分，clip 跳出。
            let mut offset = available;
            while offset < child_height && current_col < col_count && max_col_height > 0.0 {
                let remaining = child_height - offset;
                let frag_height = remaining.min(max_col_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += max_col_height;
                current_col_height = frag_height;
                if frag_height >= max_col_height && current_col + 1 < col_count {
                    current_col += 1;
                    current_col_height = 0.0;
                }
            }
        }
        // break-after:column：放置完子元素（含其全部 breaking 片段）后强制推进到下一列
        //（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
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
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
        // 如果当前列放不下，且还有更多列可用，移到下一列
        if current_col_height + child_height > container_height
            && current_col_height > 0.0
            && current_col + 1 < col_count
        {
            current_col += 1;
            current_col_height = 0.0;
        }

        columns[current_col].push(ColumnFragment {
            child_idx,
            fragment_y_offset: 0.0,
            visual_height: child_height,
        });
        current_col_height += child_height;
        // break-after:column：放置完子元素后强制推进到下一列（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
    }

    columns
}

/// 根据列分配结果定位每个子元素。
///
/// 子元素坐标相对于容器 content area（与 taffy/float 后处理一致），
/// 因此列 x 从 0 开始，不需要加 content_x/content_y。
///
/// 对于 column breaking 拆分的片段，使用负 y 偏移（fragment_y_offset）
/// 来显示子元素内容的不同垂直切片。paint 层通过容器的 overflow 裁剪
/// 确保每列只显示对应片段的内容。
///
/// 当一个子元素因 column breaking 出现在多个列中时：
/// - 第一个片段的位置存储在 child.x/y（主位置）
/// - 后续片段存储在 child.column_span_offsets
/// - paint 层对每个额外片段重新绘制子元素，并裁剪到对应列区域
fn position_multicol_children(container: &mut LayoutBox, assignments: &[Vec<ColumnFragment>], info: &ColumnInfo) {
    // 跟踪每个子元素已出现的片段数（用于区分主片段和额外片段）
    let mut child_fragment_count: HashMap<usize, usize> = HashMap::new();

    for (col_idx, col_fragments) in assignments.iter().enumerate() {
        let col_x = col_idx as f32 * (info.column_width + info.gap);
        let mut y_offset = 0.0f32;

        for frag in col_fragments {
            let child = &mut container.children[frag.child_idx];
            let frag_idx = *child_fragment_count
                .entry(frag.child_idx)
                .and_modify(|c| *c += 1)
                .or_insert(0);

            let child_x = col_x + child.margin_left;
            let child_y = y_offset + child.margin_top - frag.fragment_y_offset;

            // 所有片段（包括主片段）存储到 column_span_offsets。
            // paint 层根据 column_span_offsets 的存在跳过正常渲染，
            // 并对每个片段进行独立的列区域裁剪渲染。
            // 格式：(x_in_container, y_in_container, column_x, column_width)
            child
                .column_span_offsets
                .push((child_x, child_y, col_x, info.column_width));

            if frag_idx == 0 {
                // 第一个片段同时设置主位置（用于非 column-breaking 的子元素
                // 和作为后备渲染位置）
                child.x = child_x;
                child.y = child_y;
            }

            y_offset += frag.visual_height;

            // CSS Multi-column Layout：子元素宽度限制到列宽。
            // 仅对第一个片段执行宽度约束（避免重复递归）
            if frag_idx == 0 && child.width > info.column_width {
                let _old_width = child.width;
                child.width = info.column_width;
                let new_content_w = (info.column_width
                    - child.border_left
                    - child.border_right
                    - child.padding_left
                    - child.padding_right)
                    .max(0.0);
                child.content_width = new_content_w;
                child.content_x = child.border_left + child.padding_left;
                constrain_subtree_width(child, new_content_w);
            }
        }
    }
}

/// 递归约束子树中所有元素的宽度不超过指定最大值。
///
/// 用于 multicol 列宽约束：子元素被 taffy 按容器全宽布局，
/// 但实际需要约束到列宽。此函数递归更新所有后代的 width
/// 和 content_width，确保内部布局不会溢出列边界。
fn constrain_subtree_width(box_node: &mut LayoutBox, max_width: f32) {
    if box_node.width > max_width {
        let new_width = max_width;
        let new_content_w =
            (new_width - box_node.border_left - box_node.border_right - box_node.padding_left - box_node.padding_right)
                .max(0.0);
        box_node.width = new_width;
        box_node.content_width = new_content_w;
    }
    // 递归约束子元素
    let child_max = box_node.content_width;
    for child in &mut box_node.children {
        constrain_subtree_width(child, child_max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R904：em 单位按 element font-size 解析（非 root 16）。column-width:2em 在
    /// font-size 20px 容器内 = 40px（旧实现误为 32px = 2×16）。
    #[test]
    fn test_length_to_px_em_uses_element_font_size() {
        use zero_css_parser::values::LengthValue;
        // 2em @ font-size 20px → 40px（非 32px）。
        assert!(
            (length_to_px(&LengthValue::Em(2.0), 800.0, 20.0) - 40.0).abs() < 0.01,
            "em must resolve against element font-size (2em@20px=40), not root 16"
        );
        // 1em @ font-size 16px（默认）→ 16px（不变，零回归）。
        assert!((length_to_px(&LengthValue::Em(1.0), 800.0, 16.0) - 16.0).abs() < 0.01);
        // Px/Percentage 不受 font_size_px 影响。
        assert!((length_to_px(&LengthValue::Px(50.0), 800.0, 20.0) - 50.0).abs() < 0.01);
        assert!((length_to_px(&LengthValue::Percentage(10.0), 800.0, 20.0) - 80.0).abs() < 0.01);
    }

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
    fn test_assign_children_balanced_sequential() {
        // 5 children, each 100px high, 3 columns
        // Total = 500, target = 166.67
        // Sequential:
        // child0(100): col0=100 < 166.67 → col0=[0]
        // child1(100): col0=200 >= 166.67 → col1=[1]
        // child2(100): col1=100 < 166.67 → col1=[1,2]
        // Wait, col1=100+100=200 >= 166.67, so child2 goes to col1
        // Actually: child1 in col1, child2: col1=100 < 166.67 → col1=[1,2]
        // No: after child1 fills col1 to 100, child2: col1=100 < 166.67, so add child2.
        // col1 now = 200 >= 166.67
        // child3: col1=200 >= 166.67 → col2=[3]
        // child4: col2=100 < 166.67 → col2=[3,4]
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0), (4, 100.0)];
        let cols = assign_children_to_columns_balanced(&children, 3, &[false; 5], &[false; 5]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 2); // [0, 1]
        assert_eq!(cols[1].len(), 2); // [2, 3]
        assert_eq!(cols[2].len(), 1); // [4]
    }

    #[test]
    fn test_assign_children_balanced_uneven() {
        // 3 children: 200, 100, 200; 2 columns
        // Total = 500, target = 250
        // child0(200): col0=200 < 250 → col0=[0]
        // child1(100): col0=300 >= 250 → col1=[1]
        // child2(200): col1=100 < 250 → col1=[1,2]
        // Wait: child1(100): col0=200 < 250, so it's added to col0! col0=[0,1], height=300
        // child2(200): 300 >= 250 → col1=[2], height=200
        let children = vec![(0, 200.0), (1, 100.0), (2, 200.0)];
        let cols = assign_children_to_columns_balanced(&children, 2, &[false; 5], &[false; 5]);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].len(), 2); // [0, 1]
        assert_eq!(cols[1].len(), 1); // [2]
    }

    #[test]
    fn test_assign_children_balanced_equal() {
        // 4 children, each 100px high, 2 columns
        // Total = 400, target = 200
        // child0(100): col0=100 < 200 → col0=[0]
        // child1(100): col0=200 >= 200 → col1=[1]
        // Wait: 100 < 200, so child1 is added! col0=[0,1], h=200
        // child2(100): 200 >= 200 → col1=[2], h=100
        // child3(100): 100 < 200 → col1=[2,3], h=200
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
        let cols = assign_children_to_columns_balanced(&children, 2, &[false; 5], &[false; 5]);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].len(), 2); // [0, 1]
        assert_eq!(cols[1].len(), 2); // [2, 3]
    }

    #[test]
    fn test_assign_children_with_breaking() {
        // 4 children, each 100px high, 3 columns, 150px height limit
        // child0(100): col0=100 → col0=[0]
        // child1(100): col0=200 > 150, move to col1 → col1=[1]
        // child2(100): col1=200 > 150, move to col2 → col2=[2]
        // child3(100): col2=200 > 150, no more cols, stays → col2=[2,3]
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 150.0, &[false; 4], &[false; 4]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1);
        assert_eq!(cols[1].len(), 1);
        assert_eq!(cols[2].len(), 2); // last 2 overflow into col2
    }

    #[test]
    fn test_assign_children_with_breaking_oversized() {
        // Single child larger than column height — stays in current column
        let children = vec![(0, 300.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 100.0, &[false; 4], &[false; 4]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1); // oversized child stays in first column
    }

    #[test]
    fn test_assign_children_with_breaking_single_col_oversized_no_panic() {
        // 回归：col_count=1（column-count:1 或计算为单列）+ column-fill:auto + 明确高度 +
        // oversized 子元素后跟另一个子元素。修复前 line 378 无守卫 current_col+=1 越界，
        // 使后续子元素 columns[current_col].push 在 line 350 panic（index OOB, len 1 idx 1）。
        // 修复后：单列时 oversized 内容 clip 到唯一列，后续子元素也落入唯一列，无 panic。
        let children = vec![(0, 300.0), (1, 50.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 1, 100.0, &[false; 4], &[false; 4]);
        assert_eq!(cols.len(), 1);
        // 两子元素都分配到唯一列（clip），不应 panic
        assert!(cols[0].iter().any(|f| f.child_idx == 0));
        assert!(cols[0].iter().any(|f| f.child_idx == 1));
    }

    /// R903：`break-before:column` 强制换列——3 子元素各带 forced break，3 列 → 每列一个
    ///（首个子元素的 break 在空列上 no-op，故仍落 col0；后续两个推进到 col1/col2）。
    /// 对应 multicol-break-001（A/B/C 各入独立列，chromium Oracle 1.22%→1.06% 改善）。
    #[test]
    fn test_break_before_column_forces_new_column_balanced() {
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0)];
        // 全部 forced break（模拟 `div > div { break-before: column }`）
        let cols = assign_children_to_columns_balanced(&children, 3, &[true, true, true], &[false; 3]);
        assert_eq!(cols.len(), 3);
        // 首个子元素 break 在空 col0 上 no-op → col0=[0]；col1=[1]；col2=[2]。
        assert_eq!(cols[0].len(), 1);
        assert_eq!(cols[1].len(), 1);
        assert_eq!(cols[2].len(), 1);
        assert_eq!(cols[0][0].child_idx, 0);
        assert_eq!(cols[1][0].child_idx, 1);
        assert_eq!(cols[2][0].child_idx, 2);
    }

    /// R903：`break-before:column` 在 column-fill:auto + 明确高度路径也生效。
    /// height:3em 容纳多子，但 forced break 使每个强制入新列。
    #[test]
    fn test_break_before_column_forces_new_column_breaking() {
        // max_col_height=200 容纳全部 3 子（各 50），但 forced break 强制换列。
        let children = vec![(0, 50.0), (1, 50.0), (2, 50.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 200.0, &[false, true, true], &[false; 3]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1); // child0 无 forced break → col0
        assert_eq!(cols[1].len(), 1); // child1 forced → col1
        assert_eq!(cols[2].len(), 1); // child2 forced → col2
    }

    /// R903：首个子元素的 forced break 在空列上 no-op（不创建前导空列）。
    #[test]
    fn test_break_before_column_first_child_is_noop() {
        let children = vec![(0, 100.0), (1, 100.0)];
        // 仅首个 forced break → no-op（col0 空，不创建前导空列），child0 仍落 col0。
        let cols = assign_children_to_columns_balanced(&children, 2, &[true, false], &[false; 2]);
        assert_eq!(cols.len(), 2);
        assert!(
            cols[0].iter().any(|f| f.child_idx == 0),
            "first-child break-before must not create a leading empty column"
        );
        // child1 因 target_height（100>=100）推进 col1。
        assert!(cols[1].iter().any(|f| f.child_idx == 1));
    }

    /// R1027：`break-after:column` 强制换列——mirror R903 break-before，但作用于
    /// 「放置完子元素后」。3 子各 100，3 列，target=100：child0 落 col0 后 break-after
    /// 推进 col1；child1 落 col1 后推进 col2；child2 落 col2（末列，break-after no-op）。
    /// 对应 multicol-break-000（`div > div { break-after: column }`，A/B/C 各入独立列）。
    #[test]
    fn test_break_after_column_forces_new_column_balanced() {
        let children = vec![(0, 100.0), (1, 100.0), (2, 100.0)];
        // 全部 break-after（模拟 `div > div { break-after: column }`）
        let cols = assign_children_to_columns_balanced(&children, 3, &[false; 3], &[true, true, true]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1);
        assert_eq!(cols[1].len(), 1);
        assert_eq!(cols[2].len(), 1);
        assert_eq!(cols[0][0].child_idx, 0);
        assert_eq!(cols[1][0].child_idx, 1);
        assert_eq!(cols[2][0].child_idx, 2);
    }

    /// R1027：`break-after:column` 在 column-fill:auto + 明确高度路径也生效。
    /// max_col_height=200 容纳全部 3 子（各 50），但 break-after 使每个强制入新列。
    #[test]
    fn test_break_after_column_forces_new_column_breaking() {
        let children = vec![(0, 50.0), (1, 50.0), (2, 50.0)];
        let cols = assign_children_to_columns_with_breaking(&children, 3, 200.0, &[false; 3], &[true, true, true]);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].len(), 1);
        assert_eq!(cols[1].len(), 1);
        assert_eq!(cols[2].len(), 1);
        assert_eq!(cols[0][0].child_idx, 0);
        assert_eq!(cols[1][0].child_idx, 1);
        assert_eq!(cols[2][0].child_idx, 2);
    }

    /// R1027：末子元素的 break-after 在末列上 no-op（`current_col + 1 < col_count` 守卫
    /// 防止越界，不创建尾随空列）。
    #[test]
    fn test_break_after_column_last_child_in_last_col_is_noop() {
        let children = vec![(0, 100.0), (1, 100.0)];
        // 仅末子 break-after → child0 落 col0 后 break-after 推进 col1；child1 落 col1（末列），
        // 其 break-after 因 current_col+1 >= col_count no-op，不创建尾随空列。
        let cols = assign_children_to_columns_balanced(&children, 2, &[false; 2], &[false, true]);
        assert_eq!(cols.len(), 2);
        assert!(cols[0].iter().any(|f| f.child_idx == 0));
        assert!(cols[1].iter().any(|f| f.child_idx == 1));
    }
}
