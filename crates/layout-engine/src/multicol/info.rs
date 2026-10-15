//! multicol 列几何与信息计算：`ColumnInfo`、`compute_column_info` 族、列高上限与
//! explicit-height 判定。自 multicol.rs 按职责拆出（文件 ≤2000 行债，R5141）。

use zero_css_parser::values::LengthValue;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{ColumnCountComputedValue, ColumnFillComputedValue, ColumnWidthComputedValue};

use crate::types::LayoutBox;

/// 当 `column-fill: auto` 且容器有明确高度时，每列的最大高度等于容器内容高度。
/// 当 `column-fill: balance`（默认）时，列高无限制（均衡分配）。
pub(super) fn column_height_limit(container: &LayoutBox, info: &ColumnInfo) -> f32 {
    if info.sequential_fill && container.content_height > 0.0 {
        container.content_height
    } else {
        0.0 // 无限制
    }
}

/// 多列布局计算信息。
pub struct ColumnInfo {
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
pub(super) fn length_to_px(value: &LengthValue, container_width: f32, font_size_px: f32) -> f32 {
    match value {
        LengthValue::Px(v) => *v as f32,
        LengthValue::Percentage(p) => *p as f32 / 100.0 * container_width,
        LengthValue::Em(v) => *v as f32 * font_size_px,
        LengthValue::Ex(v) => *v as f32 * font_size_px * 0.8,
        LengthValue::Rex(v) => *v as f32 * 16.0 * 0.8,
        LengthValue::Cap(v) => *v as f32 * font_size_px * 0.8,
        LengthValue::Rcap(v) => *v as f32 * 16.0 * 0.8,
        LengthValue::Rem(v) => *v as f32 * 16.0,
        LengthValue::Vw(v) => *v as f32 * 8.0,
        LengthValue::Vh(v) => *v as f32 * 6.0,
        // R3749：column-width math 形式（`columns: 2 calc(6em + 5px)`）在 apply 阶段存为
        // Calc；用容器宽度/字号上下文求值，无法求值（如需 viewport 外部上下文）时回退 0。
        // R3766：lh 单位（css-values-4）——multicol 上下文无 line-height，按 1.2em 近似
        //（与 resolve_length Lh 臂同近似）。
        LengthValue::Lh(v) => *v as f32 * font_size_px * 1.2,
        // R4125：cq 单位（css-conditional-5 container lengths）——multicol 上下文无
        // 查询容器信息，fail-closed 0（column-width: cq 极罕见；有容器上下文的解析在
        // style-system computed 阶段已完成，残留至此视为无上下文）。
        LengthValue::Cqw(v)
        | LengthValue::Cqh(v)
        | LengthValue::Cqi(v)
        | LengthValue::Cqb(v)
        | LengthValue::Cqmin(v)
        | LengthValue::Cqmax(v) => *v as f32 * 0.0,
        LengthValue::Calc(expr) => {
            let ctx = zero_css_parser::values::CalcContext {
                parent_length: Some(container_width as f64),
                font_size: Some(font_size_px as f64),
                root_font_size: Some(16.0),
                ..Default::default()
            };
            zero_css_parser::values::eval_calc_with_context(expr, &ctx)
                .map(|v| v as f32)
                .unwrap_or(0.0)
        }
        LengthValue::Auto => 0.0,
        LengthValue::Vmin(v) => (*v as f32) * 6.0,
        LengthValue::Vmax(v) => (*v as f32) * 8.0,
        LengthValue::Ch(v) => *v as f32 * 8.0,
        LengthValue::Rch(v) => *v as f32 * 8.0,
        LengthValue::Ic(v) => *v as f32 * font_size_px,
        LengthValue::Ric(v) => *v as f32 * 16.0,
        LengthValue::FitContent(inner) => length_to_px(inner, container_width, font_size_px),
        LengthValue::MinContent | LengthValue::MaxContent => 0.0,
        // stretch 需要包含块可用空间（multicol 列环境无该语义），按 0 中性处理
        LengthValue::Stretch => 0.0,
    }
}

/// 返回 balance 模式多列容器的（列宽, 列数），供 remeasure 按列宽测量行内内容
/// 并计算分布式高度。仅 `column-fill: balance`（默认）返回 `Some`。
pub fn balance_column_geometry(style: &ComputedStyle, container_width: f32) -> Option<(f32, usize)> {
    let info = compute_column_info(style, container_width)?;
    if info.sequential_fill || info.count < 2 {
        return None;
    }
    Some((info.column_width, info.count))
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
pub fn compute_column_info(style: &ComputedStyle, container_width: f32) -> Option<ColumnInfo> {
    // em 单位按 element font-size 解析（R904：column-width/column-gap apply 不解析 em）。
    let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None) as f32;
    // R1040：column-gap 初始值 = normal（CSS Multicol §4.1），对 multicol 解析为 1em。
    // default_impl 用 LengthValue::Auto 作 normal sentinel（gap 不接受 auto，无冲突）。
    // 显式 column-gap:<length> 或 column-gap:0 尊重原值。
    let gap = if matches!(style.column_gap, LengthValue::Auto) {
        font_size_px // normal → 1em
    } else {
        length_to_px(&style.column_gap, container_width, font_size_px)
    };
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
            // CSS Multicol §3.1：负值非法 → 无多列；0 合法但 used value 永不小于 1px
            //（zero-column-width-layout：column-width:0 → used 1px → 容器内尽可能多 1px 列）。
            if min_width < 0.0 || container_width <= 0.0 {
                return None;
            }
            let min_width = min_width.max(1.0);
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
            // CSS Multicol §3.1：负值非法；0 合法但 used value ≥1px（zero-column-width-layout）。
            if n == 0 || min_width < 0.0 {
                return None;
            }
            let min_width = min_width.max(1.0);
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
pub(super) fn compute_column_count(container_width: f32, min_width: f32, gap: f32) -> usize {
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
pub(super) fn compute_single_column_width(container_width: f32, count: usize, gap: f32) -> f32 {
    if count == 0 {
        return container_width;
    }
    let total_gap = if count > 1 { gap * (count - 1) as f32 } else { 0.0 };
    ((container_width - total_gap) / count as f32).max(0.0)
}

/// 判断 height 是否为 explicit（length/percentage，非 auto/keyword）。
/// R1037 balance-mode column-breaking gate：CSS Fragmentation monolithic 元素
/// （overflow≠visible 滚动容器 / 替换元素 / auto 高度）不可分；仅 explicit-height
/// 子元素可跨列拆分。排除 Auto/Calc/FitContent/MinContent/MaxContent。
pub(super) fn is_explicit_height(style: &ComputedStyle) -> bool {
    !matches!(
        style.height,
        LengthValue::Auto
            | LengthValue::Calc(_)
            | LengthValue::FitContent(_)
            | LengthValue::MinContent
            | LengthValue::MaxContent
    )
}
