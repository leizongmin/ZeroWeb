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
    BreakValue, ColumnCountComputedValue, ColumnFillComputedValue, ColumnSpanComputedValue, ColumnWidthComputedValue,
};

use crate::types::{LayoutBox, OverflowClip};

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
/// R1820：forced-break overflow column + auto-height recompute kill-switch（LANDED default-on）。
///
/// 承接 R1817/R1818（code-trace 诊断未经验证属假阳性，A/B 零效果已 revert）。R1820 经
/// REFTEST_DEBUG 实证确证真根因：multicol-fill-auto-005 ZW 输出容器高 160px（= 自然高度和），
/// chromium 100px（forced breaks > column-count 时创建溢出列，容器高 = max 列高）。两层：
/// (1) 主路径 `let _ = position_multicol_children`（multicol.rs:927）丢弃 region_height →
/// 容器高永不按列分配结果重算；(2) 末列 forced break 不创建溢出列（`current_col+1<col_count`
/// 守卫）。本开关同时启用两层；全 multicol corpus A/B net +1 零 pass 回归后 LANDED default-on。
/// `ZW_MULTICOL_FORCED_OVERFLOW=0` 可紧急关闭。
fn forced_overflow_enabled() -> bool {
    // R1820 LANDED default-on（全 multicol corpus A/B net +1 零 pass 回归；
    // multicol-fill-auto-005 1.87%→0.62% flip，multicol-nested-027 0.62→0.87 仍 pass）。
    // ZW_MULTICOL_FORCED_OVERFLOW=0 可紧急关闭。
    std::env::var("ZW_MULTICOL_FORCED_OVERFLOW").as_deref() != Ok("0")
}

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
fn length_to_px(value: &LengthValue, container_width: f32, font_size_px: f32) -> f32 {
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
        LengthValue::Auto | LengthValue::Calc(_) => 0.0,
        LengthValue::Vmin(v) => (*v as f32) * 6.0,
        LengthValue::Vmax(v) => (*v as f32) * 8.0,
        LengthValue::Ch(v) => *v as f32 * 8.0,
        LengthValue::Rch(v) => *v as f32 * 8.0,
        LengthValue::Ic(v) => *v as f32 * font_size_px,
        LengthValue::Ric(v) => *v as f32 * 16.0,
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
pub fn compute_column_info(style: &ComputedStyle, container_width: f32) -> Option<ColumnInfo> {
    // em 单位按 element font-size 解析（R904：column-width/column-gap apply 不解析 em）。
    let font_size_px = match &style.font_size {
        LengthValue::Px(v) => *v as f32,
        _ => 16.0, // computed font_size 应为 Px；防御性回退
    };
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

/// R1340：检测一个盒的**后代**（非自身）中是否存在 column-span:all 元素。
///
/// multicol 容器的**直接子** spanner 由 `layout_multicol` 主循环检测并经
/// `layout_multicol_with_spanners` 处理（R1028）。嵌套 spanner（multicol >
/// 非 multicol wrapper > ... > spanner）目前未实现 fragmentation（R1336 诊断）。
/// 本函数 DFS 检测此类嵌套 spanner，是 wrapper-fragmentation 的检测基础
/// （当前仅用于量化日志，见 `layout_multicol` 的 ZW_MULTICOL_DEBUG_NESTED）。
///
/// 注意：检查的是后代（不含 `box_node` 自身），故对 multicol 的直接子调用时，
/// 返回 true 表示该子内部含 spanner 后代（即嵌套 spanner 场景）。
fn has_descendant_spanner(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    for c in &box_node.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        let style = c.node_id.and_then(|id| styles.get(&id));
        let is_spanner = style.is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
        if is_spanner {
            return true;
        }
        // 不下钻嵌套 multicol 容器：其内部 spanner 属于该嵌套 multicol（由其自身
        // layout_multicol 处理），非外层 wrapper-fragmentation 场景。
        let is_nested_multicol = style.is_some_and(|s| {
            matches!(s.column_count, ColumnCountComputedValue::Number(_))
                || matches!(s.column_width, ColumnWidthComputedValue::Length(_))
        });
        if !is_nested_multicol && has_descendant_spanner(c, styles) {
            return true;
        }
    }
    false
}

/// R1352：检测一个盒的**直接子**（非后代）中是否存在 column-span:all 元素（in-flow）。
///
/// 与 `has_descendant_spanner` 区别：仅查一层直接子，不下钻。用于 R1341
/// `try_layout_nested_spanner` 的 gate——仅当 spanner 是 wrapper 的**直接子**时才触发
/// synthetic 碎片化，因为 painter 的 nested-spanner-wrapper 列循环（`paint_as_multicol`）
/// 把 wrapper 直接子当列片段绘；spanner 嵌在更深层（如 `wrapper > div > div(spanner)`）
/// 时 painter 找不到直接子 spanner 全宽插入 → 渲染错（R1351
/// remove-transform-descendant-becomes-spanner 回归）。tight gate 把这类 deep-nesting
/// 案留给后续多会话，先正确处理 direct-child-spanner（004a/004b 结构）。
fn has_direct_spanner_child(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    box_node.children.iter().any(|c| {
        if c.is_absolute || c.is_fixed {
            return false;
        }
        c.node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All))
    })
}

/// R1341：嵌套 spanner wrapper-fragmentation（首版：synthetic-container + 位置回填）。
///
/// 当 multicol 容器的**非 spanner 直接子**（wrapper）含 column-span:all 后代时，wrapper
/// 应被「碎片化」——其内容参与 multicol 列流，spanner 跨全宽（CSS Multicol §6.1）。
/// R1336 诊断 ZW 当前把 wrapper 当单个非 spanner 子整体平衡（嵌套 spanner 不检测）。
///
/// **首版（gate 紧）**：仅当 wrapper 是容器的**唯一 in-flow 直接子**。建 synthetic
/// container（clone article，children 替换为 wrapper 的 in-flow 子 clone），跑现有
/// `layout_multicol_with_spanners`（区域分割 + 列平衡 + spanner 全宽插入），再把位置
/// 回填到真实 wrapper 子（补偿 wrapper 偏移 dx=wrapper.x+content_x）。spanner 子宽设为
/// article content_width（taffy 未在 article 全宽拉伸过）。
///
/// **已知局限**：wrapper 背景不分列铺（painter 侧 bg 仍按 wrapper 单盒涂）→ bg-less
/// wrapper 案可 flip；含 bg 案（如 004a pink）残余 bg diff，须后续 painter region 分段。
/// 多 wrapper / wrapper 非唯一子 / 深层嵌套未处理（gate 排除）。
///
/// 返回 true 表示已处理（`layout_multicol` 跳过常规路径）。
/// kill-switch default-on（`ZW_MULTICOL_NESTED_SPANNER=0` 关闭回退常规行为）。
fn try_layout_nested_spanner(
    container: &mut LayoutBox,
    info: &ColumnInfo,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    if std::env::var("ZW_MULTICOL_NESTED_SPANNER").as_deref() == Ok("0") {
        return false;
    }
    // R1341 gate：fragmentation 仅 2+ 列（1 列无须碎片化；multicol-span-all-children-height-008
    // column-count:1 回归）+ balance 模式（sequential fill + spanner 未支持，同 R1028/R1035
    // balance-only；multicol-span-all-017/parallel-flow-after-spanner-001 回归）。
    if info.count < 2 || info.sequential_fill {
        return false;
    }
    // gate：唯一 in-flow 直接子。
    let in_flow: Vec<usize> = container
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_absolute && !c.is_fixed)
        .map(|(i, _)| i)
        .collect();
    if in_flow.len() != 1 {
        return false;
    }
    let wrapper_idx = in_flow[0];
    // 收集 wrapper 信息（作用域隔离借用）：含嵌套 spanner？in-flow 子 idx？偏移？
    let nested: Option<(Vec<usize>, f32, f32)> = {
        let wrapper = &container.children[wrapper_idx];
        let ws = wrapper.node_id.and_then(|id| styles.get(&id));
        let wrapper_is_spanner = ws.is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
        // wrapper 自身是 multicol 容器（nested multicol）→ 其后代 spanner 属于该嵌套
        // multicol（由其自身 layout_multicol 处理），非外层 wrapper-fragmentation 场景。
        // 否则会误把嵌套 multicol 的 spanner 提升到外层（spanner-fragmentation-* 回归）。
        let wrapper_is_multicol = ws.is_some_and(|s| {
            matches!(s.column_count, ColumnCountComputedValue::Number(_))
                || matches!(s.column_width, ColumnWidthComputedValue::Length(_))
        });
        // R1341 回归规避（synthetic 碎片化是首版，无 bg/border 分列铺，须排除会回归的 wrapper）：
        // - clean_block：block-level 且非 R109 split（避 inline span wrapper 如 parallel-flow，
        //   span 含 block 子被 R109 拆成匿名块，synthetic clone 破坏其拓扑）。
        // - no_box：无 border/padding（避 styled wrapper 如 008 border:20px；bg/border 须分列铺，
        //   首版未实现，styled wrapper 现有整体渲染更接近 chromium）。
        // - no_transform：wrapper 无 transform（transform 内 column-span:all 非真 spanner，
        //   CSS Multicol：transform 建立-containing-block 中和 spanner，避 multicol-span-all-017）。
        let clean_block = wrapper.is_block_level && !wrapper.is_r109_split;
        let no_box = wrapper.border_top < 1.0
            && wrapper.border_bottom < 1.0
            && wrapper.border_left < 1.0
            && wrapper.border_right < 1.0
            && wrapper.padding_top < 1.0
            && wrapper.padding_bottom < 1.0;
        let no_transform = ws.is_some_and(|s| matches!(s.transform, zero_css_parser::values::TransformValue::None));
        // R1473 step1（RFC bordered-wrapper-multicol-fragmentation）：允许 bordered wrapper 进
        // synthetic fragmentation（使 column-span:all spanner 跨全宽 + 内容拆 region）。仅 no-box
        // fast-path 之外的 bordered 备选：须 direct-child-spanner（painter-core 路径，cso 传播）+
        // border 各侧 < 阈值（避过厚 border 几何）。A/B（css-multicol 177→177 零回归，006 17.75→
        // 10.07%）。border-skip（painter 侧）= step2 未实现，本步 border 仍全周绘（006 残余 =
        // border 全周 + R1357 c=0 column-breaking）。kill-switch `ZW_BORDERED_FRAG=0`。
        let bordered_frag = std::env::var("ZW_BORDERED_FRAG").as_deref() != Ok("0")
            && has_direct_spanner_child(wrapper, styles)
            && wrapper.border_top < 40.0
            && wrapper.border_bottom < 40.0
            && wrapper.border_left < 40.0
            && wrapper.border_right < 40.0;
        if wrapper_is_spanner
            || wrapper_is_multicol
            || !has_descendant_spanner(wrapper, styles)
            || has_abspos_descendant(wrapper)
            || !clean_block
            || !(no_box || bordered_frag)
            || !no_transform
        {
            None
        } else {
            let eff: Vec<usize> = wrapper
                .children
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.is_absolute && !c.is_fixed)
                .map(|(i, _)| i)
                .collect();
            if eff.is_empty() {
                None
            } else {
                Some((eff, wrapper.x + wrapper.content_x, wrapper.y + wrapper.content_y))
            }
        }
    };
    let (eff_indices, dx, dy) = match nested {
        Some(x) => x,
        None => return false,
    };

    // 建 synthetic：clone container，children 替换为 wrapper in-flow 子 clone；
    // spanner 子宽设为 container content_width（全宽）。
    let article_cw = container.content_width;
    let mut synth = container.clone();
    synth.children = eff_indices
        .iter()
        .map(|&i| {
            let mut c = container.children[wrapper_idx].children[i].clone();
            let is_spanner = c
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
            if is_spanner {
                c.width = article_cw;
            }
            c
        })
        .collect();

    // 跑现有 spanner 布局（synthetic 上：区域分割 + 列平衡 + spanner 全宽插入）。
    layout_multicol_with_spanners(&mut synth, info, styles);

    // 回填位置到真实 wrapper 子（补偿 wrapper 偏移 dx/dy）。
    let wrapper = &mut container.children[wrapper_idx];
    // R1352：painter 列循环（paint_as_multicol）+ column_span_offsets 传播**仅当 spanner 是
    // wrapper 的直接子**时启用。direct-child-spanner（004a/004b 结构）painter 能把直接子当列
    // 片段正确绘；deep-nesting（spanner 是孙辈，如 remove-transform-descendant 的 #elm>div>div）
    // painter 找不到直接子 spanner 全宽插入 → 渲染错。故 deep-nesting 走 baseline 路径（仅
    // x/y 回填，不设 flag、不传 cso），保留 R1341 的 0.63% 行为；direct-child 才启用 R1343
    // painter-core 改进。
    let enable_painter_core = has_direct_spanner_child(wrapper, styles);
    wrapper.is_nested_spanner_wrapper = enable_painter_core;
    for (synth_i, &real_i) in eff_indices.iter().enumerate() {
        let (sx, sy, sw, is_spanner, cso) = {
            let s = &synth.children[synth_i];
            let is_spanner = s
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|st| matches!(st.column_span, ColumnSpanComputedValue::All));
            (s.x, s.y, s.width, is_spanner, s.column_span_offsets.clone())
        };
        let r = &mut wrapper.children[real_i];
        r.x = sx - dx;
        r.y = sy - dy;
        if is_spanner {
            // spanner 脱离列流：宽设为 article 全宽（synthetic 已算），清 column_span_offsets
            // 按 block 渲染（同 layout_multicol_with_spanners line 597）。
            r.width = sw;
            r.column_span_offsets.clear();
        } else if enable_painter_core {
            // R1352 R1343：breaking 子（跨列拆分的 block）须把 synthetic 算出的
            // column_span_offsets 传播给真实 wrapper 子，否则 painter 只绘首片段（col0）。
            // 坐标：wrapper 经 R1341 no_box gate（无 border/padding），synthetic 为 article
            // clone；片段位置按 wrapper 偏移 (-dx/-dy) 平移到 wrapper-content 系（与
            // r.x = sx - dx 同变换）；col_w/col_h 不变。
            r.column_span_offsets = cso
                .iter()
                .map(|&(fx, fy, cx, cw, ct, ch)| (fx - dx, fy - dy, cx - dx, cw, ct - dy, ch))
                .collect();
        }
        // else（deep-nesting, !enable_painter_core）：保留 baseline——仅 x/y 回填，不动 cso。
    }

    // R1357：cap wrapper effective height（chromium "just enough" allocation）。
    // PIL 实证（004a/004b）：chromium 把 definite-height wrapper 的显式高分配给 span 分割的
    // sections，末 section 取 squeeze 值 `c = min(last_balanced, max(0, container − total_balanced − spans))`
    // 致 wrapper effective < CSS explicit（004a 450→350, 004b 350→300）。ZW 用 CSS explicit 全涂
    // bg 致 pink over-render（004a 12.76% / 004b 14.68% 残余主因）。此处 post-backfill 改
    // wrapper.height = effective 使 painter bg 止于正确高。gated to enable_painter_core（同 R1352
    // direct-child-spanner）+ 2+ 列 + **wrapper 显式 definite height**（排除 column-height-013 等
    // auto-height / Level-2 column-height 案——其 wrapper 无显式高，公式误 cap 致 pass→fail 回归）。
    // 安全：span-all-children-height family 13/14 案现全 FAIL，无 flip 可失；全量 A/B 守回归。
    let wrapper_definite_h = wrapper
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.height, LengthValue::Px(_)));
    if enable_painter_core && info.count >= 2 && wrapper_definite_h {
        let col_count = info.count as f32;
        let mut section_content: Vec<f32> = vec![0.0];
        let mut spans_total = 0.0_f32;
        for &ci in &eff_indices {
            let child = &wrapper.children[ci];
            let is_span = child
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
            if is_span {
                spans_total += child.height;
                section_content.push(0.0);
            } else {
                *section_content.last_mut().unwrap() += child.height;
            }
        }
        let total_balanced: f32 = section_content.iter().map(|&h| h / col_count).sum();
        let last_balanced = section_content.last().map(|&h| h / col_count).unwrap_or(0.0);
        let c = last_balanced.min((wrapper.height - total_balanced - spans_total).max(0.0));
        let effective = (total_balanced - last_balanced) + c + spans_total;
        let capped_h = if effective > 0.0 && effective < wrapper.height {
            wrapper.height = effective;
            effective
        } else {
            wrapper.height
        };
        // R1359：per-column bg regions。末列 height = capped_h − c（block3 overflow 致末列容器
        // 只覆盖到内容止点，col1 section c 应露 article bg），其余列 = capped_h（全高）。PIL
        // 实证 004a：col0 pink 到 358（350 全高）+ col1 到 308（300，缺末段 c=50）。
        // 注：c=0 case（004b）末列真值 = block2-col1 end（sequential 50px 致 col1 pink 到 208），
        // 非 capped_h−c（300）亦非 first_section（100，R1361 试过太短）。须 column-breaking 算法
        // （block2-col1 = 50 vs ZW balance 100）才能算出，当前 R1035 unsolved，c=0 残余接受。
        let n = info.count;
        let cw = info.column_width;
        let gap = info.gap;
        // R1535：末列 bg height。col0 全高 = capped_h（painter 经 column clip 自然截到列内容
        // 止点）；末列 col1 只在非末 region 有内容（末 region block 全进 col0，col1 空），故
        // 末列 bg 止于末 span 之前 = capped_h − spans_total。PIL 实证：004a col1=250（=350−100）、
        // 004b col1=200（=300−100），与 chromium oracle 逐像素一致（span1 绘于 bg 之上，故单
        // 矩形覆盖 region1+span1+region2 与 oracle 的非连续 pink 等效）。替旧 capped_h−c（004b
        // 误给 300 致末列 over-paint 50px = 1.51% 残余）。kill-switch ZW_R1535_LASTH_SPANS=0 回退。
        let r1535 = std::env::var("ZW_R1535_LASTH_SPANS").as_deref() != Ok("0");
        let last_h = if r1535 {
            (capped_h - spans_total).max(0.0)
        } else {
            (capped_h - c).max(0.0)
        };
        let mut regions = Vec::with_capacity(n);
        for i in 0..n {
            let offset = i as f32 * (cw + gap);
            let h = if i + 1 < n { capped_h } else { last_h };
            regions.push((offset, cw, h));
        }
        wrapper.nested_spanner_col_bg = regions;
    }

    // R1358：article（container/multicol 容器）content_height = 真实内容 extent（含 block3
    // overflow 到 article bg）。R1355 实证：仅 cap wrapper pink 不 flip（article green 仍
    // over-render 500 vs CHR 407）；须双 cap。container.content_height（taffy 算=500，wrap
    // wrapper CSS 450）改 = synth 实际内容 extent（max fragment end，block3 overflow 止点），
    // 使 article bg（lightgreen）止于内容（~407）非 CSS wrap（500）。clipping 用 container_h
    // 但 content_extent = max fragment end 故无内容被裁（定义上无内容超出）。gated 同 R1357。
    // 注意：breaking 子用 cso 片段范围（ct+ch）非 child.y+height（后者含全 content extent 200
    // 致 overestimate 700 > 500，guard 不 fire）；span 子 cso 空，用 y+height。
    if wrapper_definite_h {
        let content_extent: f32 = synth
            .children
            .iter()
            .map(|c| {
                if c.column_span_offsets.is_empty() {
                    c.y + c.height
                } else {
                    c.column_span_offsets
                        .iter()
                        .map(|&(_, _, _, _, ct, ch)| ct + ch)
                        .fold(0.0_f32, f32::max)
                }
            })
            .fold(0.0_f32, f32::max);
        if content_extent > 0.0 && content_extent < container.content_height {
            // R1360：wrapper box height/content_height = content_extent（非 R1357 的 effective）。
            // painter column loop 把 breaking 子（block3）裁到 `container_h = wrapper.content_height`；
            // 若留 CSS 值（004b=350），block3（col_top=300,ch=100）被裁到 [300,350]=50px（abs 358），
            // CHR 应到 404（full overflow）。改 content_height=content_extent（400）使 block3 full 渲。
            // bg 由 R1359 regions（effective）独立处理，wrapper box 高 = content_extent 不影响 bg
            //（regions 用 capped_h，非 wrapper.height）。004a 不受影响（content_extent 400 >= block3 需求）。
            let wrapper = &mut container.children[wrapper_idx];
            wrapper.height = content_extent;
            wrapper.content_height = content_extent;
            container.content_height = content_extent;
            container.height = content_extent;
        }
    }

    true
}

/// R1341：检测一个盒的后代中是否存在 absolute/fixed 元素。
///
/// nested-spanner synthetic-container 碎片化会 clone wrapper 的子树；若含 abspos/fixed
/// 后代，clone 会重复定位/破坏 containing-block 关系（abspos-containing-block-outside-
/// spanner 回归）。此类 wrapper 跳过碎片化。
fn has_abspos_descendant(box_node: &LayoutBox) -> bool {
    for c in &box_node.children {
        if c.is_absolute || c.is_fixed {
            return true;
        }
        if has_abspos_descendant(c) {
            return true;
        }
    }
    false
}

/// R1869 Slice 1（RFC `multicol-block-fragmentation-rfc-2026-07-23.md`）：multicol 单子块
/// 「透明展开」分片。
///
/// 当 multicol 唯一 in-flow 子是块（wrapper，无 border/padding、非 spanner、非 monolithic、
/// 非 nested multicol）且其直接子含 forced column break 时，把 wrapper 的直接子当 multicol
/// fragmentable units 跨列分配（复用 [`assign_children_to_columns_with_breaking`]，forced
/// breaks 驱动每子入新列），定位到列 x，并把 wrapper 宽设为 multicol 内容宽、高设为 max
/// 列内容高——使 wrapper 背景填满各列（chromium fragmented-bg 近似；multicol-fill-auto-004
/// green div bg 填满 100×100）。env `ZW_MULTICOL_BLOCKFRAG=0` 关闭（kill-switch，default-on）。
/// gate 严格：仅单 in-flow 块子（无 box）+ 直接子 forced break，不触现有 direct-children 路径。
fn try_layout_single_child_block_frag(
    container: &mut LayoutBox,
    info: &ColumnInfo,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    if std::env::var("ZW_MULTICOL_BLOCKFRAG").as_deref() == Ok("0") {
        return false;
    }
    // 1. 找唯一 in-flow 子（wrapper）
    let wrapper_idx = {
        let mut idx: Option<usize> = None;
        for (i, c) in container.children.iter().enumerate() {
            if c.is_absolute || c.is_fixed {
                continue;
            }
            if idx.is_some() {
                return false; // 多于一个 in-flow 子 → 非本 gate
            }
            idx = Some(i);
        }
        match idx {
            Some(i) => i,
            None => return false,
        }
    };
    // 2. gate + 收集 wrapper 直接子（immutable 借用，块内结束供后续 mutable）
    let (child_info, forced_breaks, forced_breaks_after) = {
        let wrapper = &container.children[wrapper_idx];
        let Some(wid) = wrapper.node_id else {
            return false;
        };
        let Some(ws) = styles.get(&wid) else {
            return false;
        };
        if !wrapper.is_block_level
            || matches!(ws.column_span, ColumnSpanComputedValue::All)
            || wrapper.overflow_x != OverflowClip::Visible
            || wrapper.overflow_y != OverflowClip::Visible
            || matches!(ws.column_count, ColumnCountComputedValue::Number(_))
            || matches!(ws.column_width, ColumnWidthComputedValue::Length(_))
            // wrapper 须无自身 box（Slice 1 简化：border/padding 与列坐标系互斥）
            || wrapper.border_top + wrapper.border_bottom + wrapper.padding_top + wrapper.padding_bottom
                > 0.5
        {
            return false;
        }
        let mut child_info: Vec<(usize, f32)> = Vec::new();
        let mut forced_breaks: Vec<bool> = Vec::new();
        let mut forced_breaks_after: Vec<bool> = Vec::new();
        let mut has_forced = false;
        for (i, c) in wrapper.children.iter().enumerate() {
            if c.is_absolute || c.is_fixed {
                continue;
            }
            child_info.push((i, c.height + c.margin_top + c.margin_bottom));
            let st = c.node_id.and_then(|id| styles.get(&id));
            let fb = st.is_some_and(|s| matches!(s.break_before, BreakValue::Column | BreakValue::Page));
            let fa = st.is_some_and(|s| matches!(s.break_after, BreakValue::Column | BreakValue::Page));
            if fb || fa {
                has_forced = true;
            }
            forced_breaks.push(fb);
            forced_breaks_after.push(fa);
        }
        if !has_forced || child_info.is_empty() {
            return false;
        }
        (child_info, forced_breaks, forced_breaks_after)
    };
    // 3. 分配：max_col_height 用大 sentinel 使子总适应当前列，由 forced breaks 驱动换列。
    let assignments = assign_children_to_columns_with_breaking(
        &child_info,
        info.count,
        100_000.0,
        &forced_breaks,
        &forced_breaks_after,
    );
    // 4. 定位 wrapper 子到列 x + 设 wrapper 宽/高（bg 填满）。
    let wrapper = &mut container.children[wrapper_idx];
    let col_stride = info.column_width + info.gap;
    let mut max_col_h = 0.0f32;
    for (col_idx, col_frags) in assignments.iter().enumerate() {
        let col_x = col_idx as f32 * col_stride;
        let mut y = 0.0f32;
        for frag in col_frags {
            let child = &mut wrapper.children[frag.child_idx];
            child.x = col_x + child.margin_left;
            child.y = y + child.margin_top - frag.fragment_y_offset;
            if child.width > info.column_width {
                child.width = info.column_width;
            }
            y += frag.visual_height;
        }
        max_col_h = max_col_h.max(y);
    }
    // wrapper 宽 = multicol 内容宽（子列 x 偏移有效）；高 = max 列内容高（bg 填各列）。
    wrapper.width = info.column_width * info.count as f32 + info.gap * (info.count - 1) as f32;
    wrapper.content_width = wrapper.width;
    if max_col_h > 0.0 {
        wrapper.height = max_col_h;
        wrapper.content_height = max_col_h;
    }
    true
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

    // 本函数可在列宽收窄后的文本重测量后再次运行。旧的 fragment offsets
    // 已不再对应新的列分配，必须先清除，避免 painter 重复绘制过期片段。
    for child in &mut container.children {
        child.column_span_offsets.clear();
    }

    // R1340：嵌套 spanner 检测（multicol wrapper-fragmentation 基础）。
    // layout_multicol 主循环只检测直接子 column-span:all；嵌套 spanner（multicol >
    // 非 multicol wrapper > spanner）当前未实现 fragmentation（R1336 诊断：wrapper 被
    // 当单个非 spanner 子整体平衡）。此处仅检测 + 量化日志（env ZW_MULTICOL_DEBUG_NESTED），
    // 不改变行为（log-only，零回归）。fragmentation 实现是后续多 session 架构工作。
    if std::env::var("ZW_MULTICOL_DEBUG_NESTED").is_ok() {
        for c in &container.children {
            if c.is_absolute || c.is_fixed {
                continue;
            }
            let self_spanner = c
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
            if !self_spanner && has_descendant_spanner(c, styles) {
                eprintln!(
                    "R1341 nested-spanner detected: multicol child node={:?} contains \
                     column-span:all descendant (handled by try_layout_nested_spanner if gate passes)",
                    c.node_id
                );
            }
        }
    }

    // R1341：嵌套 spanner wrapper-fragmentation（synthetic-container 首版）。命中则跳过常规路径。
    if try_layout_nested_spanner(container, info, styles) {
        return;
    }

    // R1869 Slice 1（RFC multicol-block-fragmentation）：单子块 + 后代 forced break 透明展开。
    // env ZW_MULTICOL_BLOCKFRAG=0 关闭（kill-switch，default-on）。命中则跳过常规 direct-children 路径。
    if try_layout_single_child_block_frag(container, info, styles) {
        return;
    }

    // 收集非 absolute/fixed 的子元素索引、高度，以及 break-before/after:column 标志
    //（R903：消费此前死值 break_before，强制换列；R1027：mirror 到 break_after）。
    // 同时检测 column-span:all spanner（R1028：spanner 脱离列流成全宽元素）。
    let mut child_info: Vec<(usize, f32)> = Vec::new();
    let mut forced_breaks: Vec<bool> = Vec::new();
    let mut forced_breaks_after: Vec<bool> = Vec::new();
    let mut explicit_height: Vec<bool> = Vec::new();
    let mut has_spanner = false;
    for (i, c) in container.children.iter().enumerate() {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        child_info.push((i, c.height + c.margin_top + c.margin_bottom));
        let style = c.node_id.and_then(|id| styles.get(&id));
        if style.is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All)) {
            has_spanner = true;
        }
        let force = style.is_some_and(|s| matches!(s.break_before, BreakValue::Column | BreakValue::Page));
        forced_breaks.push(force);
        let force_after = style.is_some_and(|s| matches!(s.break_after, BreakValue::Column | BreakValue::Page));
        forced_breaks_after.push(force_after);
        // R1037：explicit-height 标志（balance-breaking gate）。
        explicit_height.push(style.is_some_and(is_explicit_height));
    }

    if child_info.is_empty() {
        return;
    }

    // column-span:all spanner 路径：独立处理（区域分割 + 全宽 spanner），不走走单区域路径。
    if has_spanner {
        layout_multicol_with_spanners(container, info, styles);
        return;
    }

    // 列高限制：当 column-fill: auto 且容器有明确高度时生效
    let height_limit = column_height_limit(container, info);

    // 根据 column-fill 模式分配子元素到各列
    let assignments = if info.sequential_fill && height_limit > 0.0 {
        // column-fill: auto — 顺序填充，考虑列高限制（column breaking）
        //
        // R1076：definite 高度 + 内容超 col_count×列高时走 **inline 列溢出**（chromium 实测确认：
        // column-wrap:auto 默认下，列高 cap 容器高度，超出内容向右生成额外 column box，非丢弃）。
        // 用 assign_children_to_columns_multirow（以 height_limit 作 max_col_height 顺序填，超 col_count
        // 自动 push 新列）。gate 排除：① monolithic（不可分，同 R1075）；② forced breaks
        //（break-before/after:column 须 _with_breaking 尊重，multirow 不消费）；③ **nested multicol**
        //（子元素自身 column-count/width → nested fragmentation 须独立模型，同 R1035 守卫）。
        // 注：column-wrap:wrap（css-multicol-2 draft，ZW 未解析）的垂直换行语义不被本 gate 覆盖，
        // 这类案（column-height-004/025/026/027）仍 FAIL（unsupported feature，非本路径可解）。
        let total_child_height_seq: f32 = child_info.iter().map(|&(_, h)| h).sum();
        let has_monolithic_child_seq = child_info.iter().any(|&(idx, _)| {
            let c = &container.children[idx];
            c.overflow_x != OverflowClip::Visible || c.overflow_y != OverflowClip::Visible
        });
        let has_forced_break = forced_breaks.iter().any(|&b| b) || forced_breaks_after.iter().any(|&b| b);
        let has_nested_multicol_seq = child_info.iter().any(|&(idx, _)| {
            container.children[idx]
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| {
                    matches!(s.column_count, ColumnCountComputedValue::Number(_))
                        || matches!(s.column_width, ColumnWidthComputedValue::Length(_))
                })
        });
        if total_child_height_seq > info.count as f32 * height_limit + 1.0
            && !has_monolithic_child_seq
            && !has_forced_break
            && !has_nested_multicol_seq
        {
            assign_children_to_columns_multirow(&child_info, info.count, height_limit)
        } else {
            assign_children_to_columns_with_breaking(
                &child_info,
                info.count,
                height_limit,
                &forced_breaks,
                &forced_breaks_after,
            )
        }
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
        // R1037：balance-breaking 仅在容器有 definite 高度时启用（避 zero-height 容器
        // 误触——zero-height-002 height:0 容器 + explicit 子，breaking 强回归 +5.51pp）。
        //
        // R1075：definite 高度 balance 容器内容超 col_count×列高时，走 **inline 列溢出**
        //（chromium 实测确认：列高 cap 在容器高度，超出内容生成额外 column box 溢出到
        // 容器右外侧，非向下堆叠/丢弃）。用 assign_children_to_columns_multirow（以
        // container_height 作 max_col_height 把内容拆成列高片段，超出 col_count 自动 push
        // 新列）替代 balanced（balanced 在 col_count 处 break 丢弃 overflow）。定位仍
        // row_height=0（下方 position_multicol_children 调用），溢出列落 col_idx×(col_w+gap)
        // 的 x（容器右外侧）。同 R1074 spanner 路径的 inline-overflow 语义。
        let total_child_height: f32 = child_info.iter().map(|&(_, h)| h).sum();
        // `content_height` 对 height:auto 只是当前内容测量结果，并不是列高上限。
        // 只有 CSS 给出明确高度时，超出 column-count × 列高的内容才应产生行内溢出列。
        // https://drafts.csswg.org/css-multicol/#column-height
        let has_definite_height = container
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(is_explicit_height);
        let col_height = if has_definite_height {
            container.content_height
        } else {
            0.0
        };
        // monolithic（overflow≠visible）子元素不可分（CSS Fragmentation）——multirow 会拆分
        // 超高子元素，对 monolithic（如 overflow-unsplittable 的 overflow:scroll 滚动容器）是错的；
        // 有 monolithic 子元素时退回 balanced（balanced 的 R1037 gate 不拆 auto-height/monolithic）。
        let has_monolithic_child = child_info.iter().any(|&(idx, _)| {
            let c = &container.children[idx];
            c.overflow_x != OverflowClip::Visible || c.overflow_y != OverflowClip::Visible
        });
        let overflow_inline =
            col_height > 0.0 && !has_monolithic_child && total_child_height > info.count as f32 * col_height + 1.0;
        if overflow_inline {
            assign_children_to_columns_multirow(&child_info, info.count, col_height)
        } else {
            let explicit_for_break: &[bool] = if container.content_height > 0.0 {
                &explicit_height
            } else {
                &[]
            };
            assign_children_to_columns_balanced(
                &child_info,
                info.count,
                &forced_breaks,
                &forced_breaks_after,
                explicit_for_break,
            )
        }
    };

    // 定位子元素（y_base=0：单区域，整个 multicol 内容在一行列内）
    let region_height = position_multicol_children(container, &assignments, info, 0.0, 0.0);
    // R1820：auto-height column-fill:auto 容器高度重算。主路径此前丢弃 region_height（let _），
    // 容器高保持 taffy 预算的自然高度和，对 forced-break / 跨列分配给出错误（过高）容器高
    //（multicol-fill-auto-005：160px 自然和 vs chromium 100px max 列高）。仅当
    // column-fill:auto + 容器 auto-height（非 explicit）+ 列分配结果比自然和短时收紧容器高
    //（只缩不胀），限 blast radius（balance / explicit-height 路径不变）。随 R1820 kill-switch default-on。
    if forced_overflow_enabled()
        && info.sequential_fill
        && region_height > 0.0
        && region_height < container.content_height
    {
        let container_explicit = container
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(is_explicit_height);
        if !container_explicit {
            let delta = container.content_height - region_height;
            container.content_height = region_height;
            container.height -= delta;
        }
    }
}

/// column-span:all spanner 布局（R1028）。
///
/// CSS Multi-column Layout §6.1：`column-span: all` 的直接子元素（spanner）脱离列流，
/// 跨越 multicol 容器全宽，把内容按 spanner 分成多段独立平衡的列区域：
/// region 0（spanner[0] 之前）→ spanner 0（全宽）→ region 1 → spanner 1 → ... → region N。
///
/// 限制（R1028 初版）：每段区域用 balanced 分配（多数 span-all 测试用 column-fill:balance
/// 默认）。`column-fill:auto` + spanner 的 sequential row-fill 是更复杂的 multi-column
/// row 模型，暂不支持。
fn layout_multicol_with_spanners(
    container: &mut LayoutBox,
    info: &ColumnInfo,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let col_count = info.count;
    if col_count == 0 {
        return;
    }

    // 1. 划分区域：遍历直接子元素（非 abspos/fixed），spanner 作区域边界。
    //    regions[i] = 第 i 段非 spanner 子元素索引；spanners[i] = 第 i 个 spanner 索引。
    //    区域数 = spanner 数 + 1（首尾各一段，相邻 spanner 间可能有空区域）。
    let mut regions: Vec<Vec<usize>> = vec![Vec::new()];
    let mut spanners: Vec<usize> = Vec::new();
    for (i, c) in container.children.iter().enumerate() {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        let is_spanner = c
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
        if is_spanner {
            spanners.push(i);
            regions.push(Vec::new());
        } else {
            regions.last_mut().unwrap().push(i);
        }
    }

    // 2. 逐区域分配 + 定位，spanner 全宽插入其後。
    let mut y_base = 0.0f32;
    for (region_idx, region_children) in regions.iter().enumerate() {
        // 该区域子元素高度信息（break 标志暂不传递——spanner 区域内 break-before/after:column 罕见）。
        let region_child_info: Vec<(usize, f32)> = region_children
            .iter()
            .map(|&i| {
                let c = &container.children[i];
                (i, c.height + c.margin_top + c.margin_bottom)
            })
            .collect();

        // R1035：multi-row 列模型（CSS Multicol §3）。当容器有明确高度时，每个区域有
        // 「可用 leftover 高度」= 容器内容高 − 当前 y_base。若区域内容超过 col_count ×
        // region_available，溢出部分换到下一行（而非均衡单行溢出容器）。这是
        // span-all-children-height 簇（percentage-height 子 + spanner）的真解锁路径。
        //
        // 紧 gate（避 R1034 nested/fragmentation 回归 + auto 路径语义差异）：
        // (a) column-fill: balance（!sequential_fill）——auto 路径 sequential fill 语义不同，
        //     multirow 会回归 fill-auto-block-children 谱系（R1035 A/B 实测 +0.12pp）；
        // (b) 容器 content_height > 0（definite 高，否则 region_available 无意义）；
        // (c) 区域非空 && 总高 > col_count × region_available（真溢出）；
        // (d) 区域内无 nested multicol 子（nested multicol 的 fragmentation 须独立 fragmentation
        //     模型，简单 row-wrap 会回归 multicol-nested-019 谱系）。
        let region_available = if container.content_height > 0.0 {
            (container.content_height - y_base).max(0.0)
        } else {
            0.0
        };
        let total_region_height: f32 = region_child_info.iter().map(|&(_, h)| h).sum();
        let has_nested_multicol = region_children.iter().any(|&i| {
            container.children[i]
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| {
                    matches!(s.column_count, ColumnCountComputedValue::Number(_))
                        || matches!(s.column_width, ColumnWidthComputedValue::Length(_))
                })
        });
        let use_multirow = !region_child_info.is_empty()
            && !info.sequential_fill
            && region_available > 0.0
            && total_region_height > col_count as f32 * region_available + 1.0
            && !has_nested_multicol;

        let (assignments, row_height) = if use_multirow {
            // R1074：overflow 列走 **inline 方向**（水平向右），非垂直 multi-row。
            // assign 仍以 region_available 作列高（max_col_height）把超高子元素拆成 50px 片段，
            // 但定位传 row_height=0.0 → position_multicol_children 把超出 col_count 的列放在
            // col_idx ≥ col_count 的 x（容器右外侧），同 y_base 单行，而非堆到下方行。
            // CSS Multicol：definite 高度容器内容超 col_count×列高时，额外 column box 在 inline
            // 方向溢出（向右），不增加 block 方向高度。修 span-all-children-height-002：block2
            // 4 列（2 in-article + 2 右溢出）匹配 chromium；旧 multi-row 把第 2 行放到容器下方再被
            // R1039 slice-clip 隐藏 → block2[100:200] 丢失（z_vs_chr 3.99%）。
            (
                assign_children_to_columns_multirow(&region_child_info, col_count, region_available),
                0.0,
            )
        } else if region_child_info.is_empty() {
            (vec![Vec::new(); col_count.max(1)], 0.0)
        } else {
            // R1037：region 子元素 explicit-height 标志（balance-breaking gate）。
            // 仅 balance 模式（!sequential_fill）+ 容器 definite 高度时启用：
            // (a) 避 zero-height 容器误触（同非 spanner 路径）；
            // (b) column-fill:auto 的 spanner **之后** 区域应 sequential fill（region_idx>0），
            //     不做 balance-breaking（no-balancing-after-column-span：auto+spanner 后不应 balance）；
            //     region_idx==0（spanner 之前）保留 breaking（always-balancing-before-column-span）。
            let region_explicit: Vec<bool> =
                if container.content_height > 0.0 && (!info.sequential_fill || region_idx == 0) {
                    region_children
                        .iter()
                        .map(|&i| {
                            container.children[i]
                                .node_id
                                .and_then(|id| styles.get(&id))
                                .is_some_and(is_explicit_height)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
            (
                assign_children_to_columns_balanced(&region_child_info, col_count, &[], &[], &region_explicit),
                0.0,
            )
        };

        // 定位该区域子元素（列内 y 从 y_base 起），返回该区域高度。
        let region_height = position_multicol_children(container, &assignments, info, y_base, row_height);
        y_base += region_height;

        // 该区域之后插入对应 spanner（region_idx 与 spanner_idx 一一对应；末区域无 spanner）。
        if region_idx < spanners.len() {
            let spanner = &mut container.children[spanners[region_idx]];
            // spanner 脱离列流：清空 column_span_offsets 使其按正常 block 渲染（非列片段），
            // 定位到当前 y_base。width 不强制——taffy 已按 block 子元素规则把 auto 宽拉伸到
            // 容器 content_width（全宽），显式 width（如 `width:100px; column-span:all`）须尊重。
            spanner.column_span_offsets.clear();
            spanner.x = spanner.margin_left;
            spanner.y = y_base + spanner.margin_top;
            y_base += spanner.height + spanner.margin_top + spanner.margin_bottom;
        }
    }
}

/// 判断 height 是否为 explicit（length/percentage，非 auto/keyword）。
/// R1037 balance-mode column-breaking gate：CSS Fragmentation monolithic 元素
/// （overflow≠visible 滚动容器 / 替换元素 / auto 高度）不可分；仅 explicit-height
/// 子元素可跨列拆分。排除 Auto/Calc/FitContent/MinContent/MaxContent。
fn is_explicit_height(style: &ComputedStyle) -> bool {
    !matches!(
        style.height,
        LengthValue::Auto
            | LengthValue::Calc(_)
            | LengthValue::FitContent(_)
            | LengthValue::MinContent
            | LengthValue::MaxContent
    )
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
/// 4. **R1037 balance-mode column-breaking**：单个子元素超过目标列高时跨列拆分
///    （每片填满 target 边界），避免超大子元素整体留单列破坏均衡。**仅对 explicit-height
///    子元素启用**（`explicit_height[i]=true`）——CSS Fragmentation monolithic 元素
///    （overflow≠visible 滚动容器 / 替换元素 / auto 高度）不可分。
///
/// 这比 shortest-column-first 更符合规范行为：内容按顺序流过各列，
/// 而非被任意分配到最短列。
fn assign_children_to_columns_balanced(
    children: &[(usize, f32)],
    col_count: usize,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
    explicit_height: &[bool],
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

        // R1037：balance-mode column-breaking，仅对 explicit-height 子元素启用。
        // CSS Fragmentation：monolithic 元素（overflow≠visible 滚动容器 / 替换元素 / auto 高度）
        // 不可分。explicit-height（length/percentage）子元素可跨列拆分以均衡分布。
        // gate 排除 overflow-unsplittable（overflow:scroll + auto height → 不拆）等回归案，
        // 捕获 span-all-children-height（height:200px/100% explicit → 拆）目标簇。
        // R1036 通用应用 net -12（18 回归），本 gate 避免回归。
        let is_explicit = explicit_height.get(i).copied().unwrap_or(false);
        if is_explicit && child_height > target_height && target_height > 0.0 {
            // 先消耗当前列剩余空间。
            let available = (target_height - current_col_height).max(0.0);
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
            }
            // 后续片段按 target_height 填充连续列；末列之外 clip（与 with_breaking 一致）。
            let mut offset = available;
            while offset < child_height {
                if current_col + 1 < col_count {
                    current_col += 1;
                } else {
                    break;
                }
                let remaining = child_height - offset;
                let frag_height = remaining.min(target_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += frag_height;
                current_col_height = frag_height;
            }
            if current_col + 1 >= col_count {
                current_col_height = target_height;
            }
        } else {
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        }
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
    // R1820：forced break 命中末列时创建溢出列（chromium parity，env-gated default-off）。
    let forced_overflow = forced_overflow_enabled();

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 {
            if current_col + 1 < col_count {
                current_col += 1;
                current_col_height = 0.0;
            } else if forced_overflow {
                // R1820：末列 forced break → 创建溢出列（chromium 对「forced breaks 多于
                // column-count」创建额外 column box 于 inline 方向溢出，非堆末列）。
                columns.push(Vec::new());
                current_col = columns.len() - 1;
                current_col_height = 0.0;
            }
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

/// 多行列模型分配（CSS Multi-column Layout §3 multi-row）。
///
/// 内容溢出 `col_count` 列后创建额外行（row 2, 3...）。当 `current_col` 超过
/// `col_count` 时**不截断到末列**，而是换行到 row+1 col 0（动态 `push` 新列）。
/// 返回扁平 `Vec`（`len = rows × col_count`），由 `position_multicol_children` 据
/// `row_height` 还原 row/col。
///
/// R1035：用于 spanner 路径每个区域的 overflow 处理（region 内容超过 region 可用高度时）。
fn assign_children_to_columns_multirow(
    children: &[(usize, f32)],
    col_count: usize,
    max_col_height: f32,
) -> Vec<Vec<ColumnFragment>> {
    if children.is_empty() || col_count == 0 || max_col_height <= 0.0 {
        return vec![Vec::new(); col_count.max(1)];
    }
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;
    macro_rules! advance_col {
        () => {{
            current_col += 1;
            current_col_height = 0.0;
            if current_col >= columns.len() {
                columns.push(Vec::new());
            }
        }};
    }

    for &(child_idx, child_height) in children.iter() {
        let available = max_col_height - current_col_height;
        if child_height <= available {
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else if child_height <= max_col_height {
            advance_col!();
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else {
            // 子元素超高（> max_col_height）— column breaking 跨多列（含跨行）。
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
                advance_col!();
            }
            let mut offset = available;
            while offset < child_height && max_col_height > 0.0 {
                let remaining = child_height - offset;
                let frag_height = remaining.min(max_col_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += max_col_height;
                current_col_height = frag_height;
                if frag_height >= max_col_height && offset < child_height {
                    advance_col!();
                }
            }
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
fn position_multicol_children(
    container: &mut LayoutBox,
    assignments: &[Vec<ColumnFragment>],
    info: &ColumnInfo,
    y_base: f32,
    row_height: f32,
) -> f32 {
    // 跟踪每个子元素已出现的片段数（用于区分主片段和额外片段）
    let mut child_fragment_count: HashMap<usize, usize> = HashMap::new();
    let mut region_height = 0.0f32;
    // R1035：multi-row 列模型（CSS Multicol §3）。assignments 为扁平布局
    // （索引 = row × col_count + col_in_row）。row_height=0.0 表示单行（向后兼容，
    // assignments.len() ≤ col_count 时 row 恒为 0）。
    let col_count = info.count.max(1);

    for (col_idx, col_fragments) in assignments.iter().enumerate() {
        let col_in_row = if row_height > 0.0 { col_idx % col_count } else { col_idx };
        let row = if row_height > 0.0 { col_idx / col_count } else { 0 };
        let col_x = col_in_row as f32 * (info.column_width + info.gap);
        let mut y_offset = y_base + (row as f32) * row_height;

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
            // R1039：扩存 col_top（片段列顶 y_offset）+ col_h（片段视觉高 visual_height），
            // 供 paint 把 breaking 片段裁到自己的 slice 范围 [col_top, col_top+col_h]，
            // 而非容器全高（修 span-all-children-height-002 block1 全 200px 覆盖 spanner 区）。
            // 格式：(x_in_container, y_in_container, column_x, column_width, col_top, col_h)
            child
                .column_span_offsets
                .push((child_x, child_y, col_x, info.column_width, y_offset, frag.visual_height));

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
        // region 高度 = 各列达到的最大 y（相对 y_base）。各列 balance 后高度相近。
        if y_offset - y_base > region_height {
            region_height = y_offset - y_base;
        }
    }
    region_height
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
mod tests;
