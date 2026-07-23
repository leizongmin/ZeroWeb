//! 垂直书写模式（vertical-rl / vertical-lr）的 native block-flow 布局算法。
//!
//! 本模块是 R1541（vertical empirical ground truth）+ R1544（设计文档）建立的
//! **vertical block-flow 模型**，经 2 个受控 chromium ground-truth 变体（V2/V3，
//! `docs/goal/rendering-compat/empirical/vertical-mode/`）验证。
//!
//! **状态：dormant（Phase 2 wiring 待定）**。当前生产路径对 vertical-rl/lr 容器的
//! block-level 子仍用 taffy 默认（horizontal-tb 垂直堆叠，R1542 实证 sizing-entangled，
//! postprocess 路三证 net-negative）。本模块编码 native block-flow 的**位置 + 容器
//! content-size 一致计算**，供 Phase 2 wiring（engine.rs block-layout 主循环的 vertical
//! 分支 + parent propagation）复用。类 R885/R1350 的 de-risking 基础设施。
//!
//! ## 模型（R1541 PIL measured）
//!
//! 对一个 `writing-mode: vertical-rl/lr` 容器的 block-level 子（DOM 序，已知各自 outer
//! width/height）：
//!
//! - **container content-width（inline-size）** = Σ child outer-width + (n−1)×gap。
//! - **container content-height（block-size）** = max child outer-height。
//! - **child 位置**（相对 container content-box top-left）：
//!   - vertical-rl：DOM 序子从右到左。child[i].x = content_width − Σ_{0..=i} width[i]
//!     − i×gap；child[i].y = 0（block-start = top）。
//!   - vertical-lr：DOM 序子从左到右。child[i].x = Σ_{0..i} width[i] + i×gap；y = 0。
//!
//! 关键：layout 期算 container content-size = Σ/max（非 taffy 的 max/Σ），父用此值 sizing，
//! 消除 postprocess 的 sizing 矛盾（R1043/R1047/R1542 net-negative 根因）。

#![allow(dead_code)]

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::WritingModeValue;

use crate::types::LayoutBox;

/// vertical block-flow 布局结果（container content-box 系）。
#[derive(Clone, Debug, PartialEq)]
pub struct VerticalBlockFlowLayout {
    /// 各子（DOM 序）相对 container content-box top-left 的 (x, y) 偏移。
    pub child_offsets: Vec<(f32, f32)>,
    /// container content-width（inline-size）= Σ child width + gaps。
    pub content_width: f32,
    /// container content-height（block-size）= max child height。
    pub content_height: f32,
}

/// 计算 vertical-rl/lr 容器的 native block-flow（R1541 ground truth 模型）。
///
/// - `children_outer_sizes`：各 block-level 子的 (outer-width, outer-height)，DOM 序。
/// - `writing_mode`：须为 VerticalRl 或 VerticalLr（HorizontalTb 调用方不应进入此函数）。
/// - `gap`：block-flow 方向（水平）的列间距。
pub fn compute_vertical_block_flow(
    children_outer_sizes: &[(f32, f32)],
    writing_mode: WritingModeValue,
    gap: f32,
) -> VerticalBlockFlowLayout {
    let n = children_outer_sizes.len();
    if n == 0 {
        return VerticalBlockFlowLayout {
            child_offsets: Vec::new(),
            content_width: 0.0,
            content_height: 0.0,
        };
    }
    let total_width: f32 = children_outer_sizes.iter().map(|(w, _)| *w).sum();
    let content_width = (total_width + (n as f32 - 1.0) * gap).max(0.0);
    let content_height = children_outer_sizes.iter().map(|(_, h)| *h).fold(0.0_f32, f32::max);

    let mut child_offsets = Vec::with_capacity(n);
    match writing_mode {
        WritingModeValue::VerticalRl => {
            // DOM 序从右到左：child[i].x = content_width − Σ_{0..=i} width − i×gap。
            //（R1785：sideways-rl 在 parse 时规范化为 VerticalRl，故同此分支。）
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                cum += w;
                let x = (content_width - cum - i as f32 * gap).max(0.0);
                child_offsets.push((x, 0.0));
            }
        }
        WritingModeValue::VerticalLr => {
            // DOM 序从左到右：child[i].x = Σ_{0..i} width + i×gap。
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                let x = cum + i as f32 * gap;
                child_offsets.push((x, 0.0));
                cum += w;
            }
        }
        // HorizontalTb 不应进入此函数；兜底按 vertical-lr（左到右）以避免 panic。
        WritingModeValue::HorizontalTb => {
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                let x = cum + i as f32 * gap;
                child_offsets.push((x, 0.0));
                cum += w;
            }
        }
    }

    VerticalBlockFlowLayout {
        child_offsets,
        content_width,
        content_height,
    }
}

/// R1544 Phase 2：vertical-rl/lr 容器的 native block-flow 后处理接线。
///
/// 对 `writing-mode:vertical-rl/lr` 容器的 block-level in-flow 子，taffy 仍按
/// horizontal-tb 垂直堆叠（R1541 V2/V3 实证：子对角排列——B2 既右移又下移，应同 y
/// 水平排列：rl 右到左 / lr 左到右）。本 pass 用 [`compute_vertical_block_flow`] 重算
/// 子位置，并把容器 block 方向的 content-size（物理 width = block-size = Σ 子宽）修正。
///
/// **安全性论证（区别 R1542 三证 net-negative 的 postprocess 路径）**：
/// - 仅重定位子 + 设容器**物理 width**（block-size）。物理 width 在 HorizontalTb 块父中
///   不传播（块子垂直堆叠，sibling 位置不依赖兄弟宽度），故无需 two-pass / mark_dirty。
/// - **不动物理 height**（inline-size）：definite 时已正确；auto 时若改需 ancestor 高度
///   传播（R1542 墙），故本 pass 不处理容器物理 height——auto-inline-size 容器的高度
///   残留（taffy 按 Σ 子高而非 max）留待后续 two-pass 演进。
/// - gate：vertical 容器 + **HorizontalTb 父** + ≥2 block-level in-flow 子（排除 abspos/
///   fixed/float/table-internal/flex/grid/multicol 子树）。
///
/// env `ZW_VERTICAL_BLOCK_FLOW`（**default-on**；`=0` 关闭 kill-switch）。R1545 全量
/// writing-modes reftest-oracle A/B 实测 **net +1**（91/784 vs 90/784，chr<1%）+ 14 大改善
///（block-flow/line-box-direction-vrl/vlr 簇 −62~−66pp）/ 0 大回归 + Σ z_vs_chr −422pp；
/// V2/V3 ground truth 像素级匹配 chromium。kill-switch 留作回退兜底。
pub fn apply_vertical_block_flow(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_VERTICAL_BLOCK_FLOW").as_deref() == Ok("0") {
        return;
    }
    apply_inner(root, styles, &WritingModeValue::HorizontalTb);
}

/// R1544 Phase 2 layout-time 两阶段 content-size 传播：第一趟 taffy 布局后，对 vertical
/// 容器（gate 同 [`apply_vertical_block_flow`]）用 [`compute_vertical_block_flow`] 算正确
/// content-size（width=Σ 子宽 / height=max 子高），把 **Auto** 维度的 taffy `style.size`
/// 改为 Length + mark_dirty，由 engine 重跑 taffy → 父级按正确 container size re-layout。
///
/// **与 postprocess（[`apply_vertical_block_flow`]）的区别 / 互补**：postprocess 仅改物理
/// width（HorizontalTb 块父中不传播），**不碰 height**（避 R1542 高度传播 net-negative 墙）。
/// 本 layout-time pass 把正确 size 喂回 taffy，让父级 re-layout 时按正确 container height
/// 定位后续 in-flow 兄弟 / sizing 自身——**解 R1545 forward 残余的 height 传播**（taffy 按
/// vertical-stack 算 container height=Σ 子高，应为 max）。子位置仍由 postprocess 在重提取
/// 后重定位（taffy 仍按 vertical-stack 排子，位置错误）。
///
/// **安全性**：仅覆盖 **Auto** 维度（`style.width/height == Auto`）。definite 维度
///（如 `height:8em`）已正确，覆盖会破坏（vrl-013 inline-block height:8em 实证）。HorizontalTb
/// 块父中 width 传播为 benign（同 postprocess）；height 经 taffy 重跑正确传播到父 content_height。
///
/// **A/B 实测（css-writing-modes reftest-oracle 784 案）**：width-only（height-set 关）+ 排除
/// float 容器 = **net +2（134→136 chr<1%）**，19 改善 / 1 小回归（vrl-011 +1.85pp，与改善案
/// vrl-012 结构同体，疑噪声）。大改善 = line-box-direction/block-flow-direction-vrl/vlr 簇
///（−13~−22pp，多案 flip 到 pass）+ percent-padding 簇（−6pp×4）。height-set（`ZW_VERTICAL_BLOCK_FLOW_HEIGHT=1`）
/// 对 float 容器（vlr-008 +12.86pp）与 auto-height inline-block（orthogonal +6~8pp）回归，故
/// **default-off**。
///
/// env `ZW_VERTICAL_BLOCK_FLOW_LAYOUT` **default-on**（`=0` kill-switch）；height-set 独立 env
/// `ZW_VERTICAL_BLOCK_FLOW_HEIGHT=1`（default-off，A/B 实测回归）。
pub fn apply_vertical_block_flow_sizing(
    taffy_tree: &mut taffy::TaffyTree<NodeId>,
    root: &LayoutBox,
    dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    if std::env::var("ZW_VERTICAL_BLOCK_FLOW_LAYOUT").as_deref() == Ok("0") {
        return false;
    }
    let height_enabled = std::env::var("ZW_VERTICAL_BLOCK_FLOW_HEIGHT").as_deref() == Ok("1");
    apply_vertical_block_flow_sizing_inner(taffy_tree, root, dom_to_taffy, styles, height_enabled)
}

/// env-free 核心（供单测直接调用，避免 set_var 在并行测试中的竞态）。
/// `height_enabled` 控制 height-set（实测对 float/auto-height inline-block 回归，故独立门控）。
fn apply_vertical_block_flow_sizing_inner(
    taffy_tree: &mut taffy::TaffyTree<NodeId>,
    root: &LayoutBox,
    dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
    styles: &HashMap<NodeId, ComputedStyle>,
    height_enabled: bool,
) -> bool {
    let mut changed = false;
    // 自顶向下遍历，跟踪 parent_wm（root 父 = HorizontalTb 视口）。
    let mut stack: Vec<(&LayoutBox, WritingModeValue)> = vec![(root, WritingModeValue::HorizontalTb)];
    while let Some((b, parent_wm)) = stack.pop() {
        let own_wm = b.writing_mode.clone();
        for child in &b.children {
            stack.push((child, own_wm.clone()));
        }
        let Some(block_indices) = vertical_block_child_indices(b, styles, &parent_wm) else {
            continue;
        };
        // 排除 float 容器：float 有自有 shrink-to-fit 宽度算法（§10.3.5），taffy 重跑强制宽度
        // 破坏 float 流定位（§9.5）——block-flow-direction-vlr-007/008 float 容器各
        // +1.97/+12.86pp 回归实证。postprocess（apply_vertical_block_flow）仍处理 float 容器
        // width（final 期，无重跑，R1545 net+1 不变）。
        if !matches!(b.float, FloatValue::None) {
            continue;
        }
        let outer_sizes: Vec<(f32, f32)> = block_indices
            .iter()
            .map(|&i| {
                let c = &b.children[i];
                (
                    c.width + c.margin_left + c.margin_right,
                    c.height + c.margin_top + c.margin_bottom,
                )
            })
            .collect();
        let layout = compute_vertical_block_flow(&outer_sizes, b.writing_mode.clone(), 0.0);
        let Some(id) = b.node_id else { continue };
        let Some(&taffy_id) = dom_to_taffy.get(&id) else {
            continue;
        };
        let Some(s) = styles.get(&id) else { continue };
        let Ok(mut style) = taffy_tree.style(taffy_id).cloned() else {
            continue;
        };
        let frame_w = b.border_left + b.border_right + b.padding_left + b.padding_right;
        let frame_h = b.border_top + b.border_bottom + b.padding_top + b.padding_bottom;
        let new_w = layout.content_width + frame_w;
        let new_h = layout.content_height + frame_h;
        // 仅覆盖 Auto 维度（definite 已正确）；仅在显著不同时 set（避 float 抖动）。
        let width_auto = matches!(s.width, LengthValue::Auto);
        let height_auto = matches!(s.height, LengthValue::Auto);
        let width_diff = width_auto && (new_w - b.width).abs() > 0.5;
        // height-set 经 A/B 实测对 float 容器（vlr-008 +12.61pp）与 auto-height inline-block
        // 容器（orthogonal-vrl/vlr +6~8pp）回归——height 传播破坏 float 流（§9.5）/ inline-block
        // sizing。故 height-set 独立 env（default-off）；width-only 是安全基线（width 传播 benign，
        // 同 R1545 postprocess 但经 taffy 重跑解 vertical-in-vertical）。
        let height_diff = height_enabled && height_auto && (new_h - b.height).abs() > 0.5;
        if width_diff {
            style.size.width = taffy::style::Dimension::length(new_w);
        }
        if height_diff {
            style.size.height = taffy::style::Dimension::length(new_h);
        }
        if width_diff || height_diff {
            let _ = taffy_tree.set_style(taffy_id, style);
            let _ = taffy_tree.mark_dirty(taffy_id);
            changed = true;
        }
    }
    changed
}

/// R1545 gate（共享）：判定一个容器是否可做 native vertical block-flow，返回其
/// block-level in-flow 子的 DOM 序索引（≥2）。`None` = gate 不通过。
///
/// 复用于 postprocess 子重定位（[`apply_inner`]）与 layout-time 两阶段 sizing
///（engine `apply_vertical_block_flow_sizing`），保证两条路径 gate 一致。
///
/// gate（每条均有 per-case A/B measured 回归实证）：
/// 1. 容器 `writing-mode` ∈ {VerticalRl, VerticalLr}；父 `writing-mode` == HorizontalTb
///    （HorizontalTb 块父中改容器物理 width 不传播——postprocess 安全性论证；
///    layout-time sizing 经 taffy 重跑解 height 传播）。
/// 2. 容器非 abspos/fixed（§10.3.7 shrink-to-fit 自有尺寸；vrl-009 +23.60pp 回归）。
/// 3. 容器非 table/table-internal（table step 8 自有算法；vlr-018/020 +5.83pp 回归）。
/// 4. ≥2 个 block-level in-flow 子（display:Block，排除 abspos/fixed/float/
///    table/flex/grid/multicol 子树）。
/// 5. 无 Percentage margin 子（§7.2 dimensional-mapping 独立缺口；vrl-004/006 +8pp 回归）。
pub fn vertical_block_child_indices(
    b: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_wm: &WritingModeValue,
) -> Option<Vec<usize>> {
    let is_vertical = b.writing_mode.is_vertical_block_flow();
    if !is_vertical || !matches!(parent_wm, WritingModeValue::HorizontalTb) {
        return None;
    }
    if b.is_absolute || b.is_fixed {
        return None;
    }
    let container_is_table = b.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::Table
                | DisplayValue::InlineTable
                | DisplayValue::TableRow
                | DisplayValue::TableCell
                | DisplayValue::TableCaption
                | DisplayValue::TableColumn
                | DisplayValue::TableColumnGroup
                | DisplayValue::TableRowGroup
                | DisplayValue::TableHeaderGroup
                | DisplayValue::TableFooterGroup
        )
    });
    if container_is_table {
        // R1844：vertical-RL/sideways-RL table-cell BFC 允许 native RL block flow
        //（block-flow-direction-vrl-017 / -srl-057 等 RL table-cell 需 RL 子排列；旧 blanket
        // 排除致其子保 LR）。vertical-LR table-cell 仍排除（gate #3 measured vlr-018/020
        // +5.83pp 回归——table step-8 与本 pass 在 LR 方向冲突）。RL-only 窄放宽：避开 LR 回归。
        // A/B（writing-modes）：vrl-017/srl-057 各 −7.48pp，vlr-018/020 不变，0 回归。
        let is_vrl_table_cell = b.writing_mode == WritingModeValue::VerticalRl
            && b.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
        if !is_vrl_table_cell {
            return None;
        }
    }
    let block_indices: Vec<usize> = b
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if c.is_absolute || c.is_fixed || !matches!(c.float, FloatValue::None) {
                return false;
            }
            c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.display, DisplayValue::Block))
        })
        .map(|(i, _)| i)
        .collect();
    let any_pct_margin = block_indices.iter().any(|&i| {
        b.children[i].node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
            matches!(s.margin_left, LengthValue::Percentage(_))
                || matches!(s.margin_right, LengthValue::Percentage(_))
                || matches!(s.margin_top, LengthValue::Percentage(_))
                || matches!(s.margin_bottom, LengthValue::Percentage(_))
        })
    });
    if any_pct_margin {
        return None;
    }
    if block_indices.len() < 2 {
        return None;
    }
    Some(block_indices)
}

fn apply_inner(b: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, parent_wm: &WritingModeValue) {
    // 先递归子（用本盒 wm 作子父 wm），再处理本盒——自底向上。
    let own_wm = b.writing_mode.clone();
    for child in &mut b.children {
        apply_inner(child, styles, &own_wm);
    }

    let Some(block_indices) = vertical_block_child_indices(b, styles, parent_wm) else {
        return;
    };

    // 各子 outer (width, height) = border-box + margin（DOM 序）。
    let outer_sizes: Vec<(f32, f32)> = block_indices
        .iter()
        .map(|&i| {
            let c = &b.children[i];
            (
                c.width + c.margin_left + c.margin_right,
                c.height + c.margin_top + c.margin_bottom,
            )
        })
        .collect();

    let layout = compute_vertical_block_flow(&outer_sizes, b.writing_mode.clone(), 0.0);

    // 重定位子：offset 相对 content-box top-left；LayoutBox.x/y 相对父 content area
    //（与 multicol position_multicol_children 约定一致：child.x = col_x + margin_left）。
    for (k, &i) in block_indices.iter().enumerate() {
        let (ox, oy) = layout.child_offsets[k];
        let ml = b.children[i].margin_left;
        let mt = b.children[i].margin_top;
        b.children[i].x = ox + ml;
        b.children[i].y = oy + mt;
    }

    // 修正容器物理 width（block-size）= Σ 子宽 + frame。物理 width 在 HorizontalTb 块父
    // 中不传播，安全。仅在显著变化时写（避免 float 抖动 / 无意义覆盖）。
    let frame_w = b.border_left + b.border_right + b.padding_left + b.padding_right;
    let new_width = layout.content_width + frame_w;
    if (new_width - b.width).abs() > 0.5 {
        b.width = new_width;
        b.content_width = layout.content_width.max(0.0);
    }
}

/// R1972 vertical child inline-fill（experimental，gate `ZW_VERTICAL_CHILD_FILL` default-off）。
///
/// **与 R1965/R1968/R1970（改 *容器* height）的根本区别**：本 pass 改 *子* 的物理 height
///（= inline-size），不改容器几何。容器 height（definite inline-size，如 height:8em=180）
/// 与 width（block-size=Σ 子宽，apply_vertical_block_flow 设）都不变 → **horizontal 父兄弟
/// 定位不受影响**（容器 outer size 不变）。子 height 改在 vertical 子树内部，contained。
///
/// **修的 bug（R1971 定位）**：vertical 容器（horizontal 父）未 axis-swap 但子 swap →
/// 容器 definite inline-size（物理 height）在错误 taffy 轴 → 子物理 height=inf。本 pass 把
/// inf 子的 height 设为容器 content_height（子填满 inline 轴，CSS block 子语义）。
///
/// **gate**：vertical 容器（复用 vertical_block_child_indices）+ 容器 height **definite**
///（非 Auto，content_height 是正确 fill 值）+ content_height finite + 子 height inf。
/// 仅 inf 子（不覆盖 finite definite 子）。
pub fn apply_vertical_child_inline_fill(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_VERTICAL_CHILD_FILL").as_deref() != Ok("1") {
        return;
    }
    apply_child_fill_inner(root, styles, &WritingModeValue::HorizontalTb);
}

fn apply_child_fill_inner(b: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, parent_wm: &WritingModeValue) {
    // 自底向上：先递归子。
    let own_wm = b.writing_mode.clone();
    for child in &mut b.children {
        apply_child_fill_inner(child, styles, &own_wm);
    }
    // 复用 vertical_block_child_indices gate（vertical 容器 + horizontal 父 + ≥2 block 子 + ...）。
    let Some(block_indices) = vertical_block_child_indices(b, styles, parent_wm) else {
        return;
    };
    // 容器 inline-size 须 definite（height 非 Auto）——否则 content_height 非正确 fill 值。
    let container_height_definite = b
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| !matches!(s.height, LengthValue::Auto));
    if !container_height_definite {
        return;
    }
    let inline_size = b.content_height;
    if !inline_size.is_finite() || inline_size <= 0.0 {
        return;
    }
    for &i in &block_indices {
        let child = &mut b.children[i];
        // 仅修 inf 子（clear bug）；finite 子（definite 或已正确）不动。
        if child.height.is_infinite() {
            let margins = child.margin_top + child.margin_bottom;
            let box_h = (inline_size - margins).max(0.0);
            child.height = box_h;
            let frame_h = child.border_top + child.border_bottom + child.padding_top + child.padding_bottom;
            child.content_height = (box_h - frame_h).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R1541 V2：vertical-rl，2 个 50×150 block，gap 0。
    /// chromium 实测：B1 x[52-101]（右），B2 x[2-51]（左），container 100×150（+border）。
    #[test]
    fn test_v2_vertical_rl_two_blocks() {
        let r = compute_vertical_block_flow(&[(50.0, 150.0), (50.0, 150.0)], WritingModeValue::VerticalRl, 0.0);
        assert!(
            (r.content_width - 100.0).abs() < 0.5,
            "content_width {}",
            r.content_width
        );
        assert!(
            (r.content_height - 150.0).abs() < 0.5,
            "content_height {}",
            r.content_height
        );
        // B1 右（x=50），B2 左（x=0）。
        assert!(
            (r.child_offsets[0].0 - 50.0).abs() < 0.5,
            "B1.x {}",
            r.child_offsets[0].0
        );
        assert!((r.child_offsets[1].0).abs() < 0.5, "B2.x {}", r.child_offsets[1].0);
        assert!(
            r.child_offsets[0].1.abs() < 0.5 && r.child_offsets[1].1.abs() < 0.5,
            "y=0"
        );
    }

    /// R1541 V3：vertical-lr，2 个 50×150 block，gap 0。
    /// chromium 实测：B1 x[2-51]（左），B2 x[52-101]（右）。
    #[test]
    fn test_v3_vertical_lr_two_blocks() {
        let r = compute_vertical_block_flow(&[(50.0, 150.0), (50.0, 150.0)], WritingModeValue::VerticalLr, 0.0);
        assert!((r.content_width - 100.0).abs() < 0.5);
        assert!((r.content_height - 150.0).abs() < 0.5);
        // B1 左（x=0），B2 右（x=50）。
        assert!((r.child_offsets[0].0).abs() < 0.5, "B1.x {}", r.child_offsets[0].0);
        assert!(
            (r.child_offsets[1].0 - 50.0).abs() < 0.5,
            "B2.x {}",
            r.child_offsets[1].0
        );
    }

    /// rl 三子：验证 right-to-left 累计（content_width=150，B1 x=100/B2 x=50/B3 x=0）。
    #[test]
    fn test_vertical_rl_three_blocks() {
        let r = compute_vertical_block_flow(
            &[(50.0, 100.0), (50.0, 100.0), (50.0, 100.0)],
            WritingModeValue::VerticalRl,
            0.0,
        );
        assert!((r.content_width - 150.0).abs() < 0.5);
        assert!((r.content_height - 100.0).abs() < 0.5);
        let xs: Vec<f32> = r.child_offsets.iter().map(|(x, _)| *x).collect();
        assert!((xs[0] - 100.0).abs() < 0.5 && (xs[1] - 50.0).abs() < 0.5 && xs[2].abs() < 0.5);
    }

    /// gap 非零：content_width 含 (n−1)×gap，子位置含累计 gap。
    #[test]
    fn test_vertical_lr_with_gap() {
        let r = compute_vertical_block_flow(&[(50.0, 100.0), (50.0, 100.0)], WritingModeValue::VerticalLr, 10.0);
        assert!(
            (r.content_width - 110.0).abs() < 0.5,
            "content_width {}",
            r.content_width
        );
        assert!((r.child_offsets[0].0).abs() < 0.5);
        assert!(
            (r.child_offsets[1].0 - 60.0).abs() < 0.5,
            "B2.x with gap {}",
            r.child_offsets[1].0
        );
    }

    /// content_height = max（非 sum）。
    #[test]
    fn test_content_height_is_max() {
        let r = compute_vertical_block_flow(&[(50.0, 100.0), (50.0, 200.0)], WritingModeValue::VerticalRl, 0.0);
        assert!((r.content_height - 200.0).abs() < 0.5);
    }

    /// 空子：零宽高，无 panic。
    #[test]
    fn test_empty_children() {
        let r = compute_vertical_block_flow(&[], WritingModeValue::VerticalRl, 0.0);
        assert!(r.child_offsets.is_empty());
        assert_eq!(r.content_width, 0.0);
        assert_eq!(r.content_height, 0.0);
    }

    // ── Phase 2 接线（apply_inner）测试 ───────────────────────────────
    // 用真实 Document 取 NodeId，构造 LayoutBox 树模拟 R1541 V2/V3 ground truth，
    // 验证 apply_inner 把 taffy 的对角堆叠重定位到 chromium 正确的水平 block-flow。

    use zero_css_parser::values::DisplayValue;
    use zero_css_parser::values::LengthValue;
    use zero_dom::Document;

    /// 构造一个 vertical 容器 + 2 个 50×150 block 子（border 2px，无 margin），
    /// writing_mode 由参数指定。返回 (container_box, styles)。
    fn build_v2v3_tree(wm: WritingModeValue) -> (LayoutBox, HashMap<zero_dom::NodeId, ComputedStyle>) {
        let mut doc = Document::new();
        let container_id = doc.create_element("div");
        let b1 = doc.create_element("div");
        let b2 = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::Block;
        cs.writing_mode = wm.clone();
        styles.insert(container_id, cs);
        let mut bs = ComputedStyle::default();
        bs.display = DisplayValue::Block;
        bs.width = LengthValue::Px(50.0);
        bs.height = LengthValue::Px(150.0);
        styles.insert(b1, bs.clone());
        styles.insert(b2, bs);

        let mut container = LayoutBox::default();
        container.node_id = Some(container_id);
        container.writing_mode = wm;
        container.border_left = 2.0;
        container.border_right = 2.0;
        container.width = 800.0; // taffy 拉伸到满宽（待修正）
        container.content_width = 796.0;

        for id in [b1, b2] {
            let mut child = LayoutBox::default();
            child.node_id = Some(id);
            child.writing_mode = WritingModeValue::HorizontalTb;
            child.width = 50.0;
            child.height = 150.0;
            // 模拟 taffy 对角堆叠：B2 既右移又下移（R1541 V2 ZW 实测）。
            container.children.push(child);
        }
        // 给 B2 一个错误的「对角」初值，验证 apply_inner 覆盖之。
        container.children[1].x = 50.0;
        container.children[1].y = 150.0;

        (container, styles)
    }

    /// V2 vertical-rl：apply_inner 后 B1 右（x≈50）、B2 左（x≈0）、同 y=0，
    /// 容器物理 width 修正为 Σ(100) + frame(4) = 104。
    #[test]
    fn test_apply_inner_v2_vertical_rl() {
        let (mut container, styles) = build_v2v3_tree(WritingModeValue::VerticalRl);
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        let b1 = &container.children[0];
        let b2 = &container.children[1];
        assert!((b1.x - 50.0).abs() < 0.5, "B1.x 右侧应≈50，实 {}", b1.x);
        assert!(b2.x.abs() < 0.5, "B2.x 左侧应≈0，实 {}", b2.x);
        assert!(b1.y.abs() < 0.5 && b2.y.abs() < 0.5, "同 y=0（{} {}）", b1.y, b2.y);
        assert!(
            (container.width - 104.0).abs() < 0.5,
            "容器 width 应≈104，实 {}",
            container.width
        );
        assert!(
            (container.content_width - 100.0).abs() < 0.5,
            "content_width 应≈100，实 {}",
            container.content_width
        );
    }

    /// V3 vertical-lr：apply_inner 后 B1 左（x≈0）、B2 右（x≈50）、同 y=0。
    #[test]
    fn test_apply_inner_v3_vertical_lr() {
        let (mut container, styles) = build_v2v3_tree(WritingModeValue::VerticalLr);
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        let b1 = &container.children[0];
        let b2 = &container.children[1];
        assert!(b1.x.abs() < 0.5, "B1.x 左侧应≈0，实 {}", b1.x);
        assert!((b2.x - 50.0).abs() < 0.5, "B2.x 右侧应≈50，实 {}", b2.x);
        assert!(b1.y.abs() < 0.5 && b2.y.abs() < 0.5, "同 y=0");
    }

    /// gate：HorizontalTb 容器不触发（apply_inner 应 no-op）。
    #[test]
    fn test_apply_inner_skips_horizontal_container() {
        let (mut container, styles) = build_v2v3_tree(WritingModeValue::HorizontalTb);
        // build 用 HorizontalTb 时容器非 vertical，apply_inner 应早返回不改 B2 对角初值。
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        assert_eq!(container.children[1].x, 50.0, "HorizontalTb 容器不应被改");
        assert_eq!(container.width, 800.0, "HorizontalTb 容器 width 不变");
    }

    /// gate：仅 1 个 block 子不触发（避免误移单子）。
    #[test]
    fn test_apply_inner_skips_single_block_child() {
        let (mut container, styles) = build_v2v3_tree(WritingModeValue::VerticalRl);
        container.children.truncate(1); // 仅留 B1
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        assert_eq!(container.width, 800.0, "单子容器不应触发 width 修正");
    }

    /// gate：abspos 容器不触发（vrl-009 回归实证，§10.3.7 shrink-to-fit 自有尺寸）。
    #[test]
    fn test_apply_inner_skips_abspos_container() {
        let (mut container, styles) = build_v2v3_tree(WritingModeValue::VerticalRl);
        container.is_absolute = true;
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        assert_eq!(container.children[1].x, 50.0, "abspos 容器不应被改");
        assert_eq!(container.width, 800.0, "abspos 容器 width 不变");
    }

    /// gate：table-cell 容器不触发（vlr-018 回归实证，table 布局自有算法）。
    #[test]
    fn test_apply_inner_skips_table_cell_container() {
        let (mut container, mut styles) = build_v2v3_tree(WritingModeValue::VerticalLr);
        // 把容器样式改为 table-cell。
        if let Some(id) = container.node_id {
            if let Some(s) = styles.get_mut(&id) {
                s.display = DisplayValue::TableCell;
            }
        }
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        assert_eq!(container.width, 800.0, "table-cell 容器 width 不变");
    }

    /// gate：任一 block 子带 Percentage margin → 跳过整个容器（percent-margin-vrl-004/006 回归）。
    #[test]
    fn test_apply_inner_skips_percent_margin_children() {
        let (mut container, mut styles) = build_v2v3_tree(WritingModeValue::VerticalRl);
        // 给 B2 加一个百分比 margin。
        if let Some(id) = container.children[1].node_id {
            if let Some(s) = styles.get_mut(&id) {
                s.margin_top = LengthValue::Percentage(12.5);
            }
        }
        apply_inner(&mut container, &styles, &WritingModeValue::HorizontalTb);
        // 容器未被重定位（B2 保留错误的「对角」初值 x=50，width 不变）。
        assert_eq!(container.children[1].x, 50.0, "percent-margin 容器不应被重定位");
        assert_eq!(container.width, 800.0, "percent-margin 容器 width 不变");
    }

    // ── Phase 2 layout-time sizing（apply_vertical_block_flow_sizing_inner）测试 ──
    // 验证两阶段 sizing 把 vertical 容器的 Auto 维度喂回 taffy（width=Σ / height=max），
    // 且尊重 definite 维度（不覆盖 height:8em 等）。

    use taffy::TaffyTree;

    /// sizing 测试夹具：taffy 树 + LayoutBox + dom_to_taffy + styles + container NodeId。
    struct SizingFixture {
        taffy: TaffyTree<zero_dom::NodeId>,
        container: LayoutBox,
        dom_to_taffy: HashMap<zero_dom::NodeId, taffy::NodeId>,
        styles: HashMap<zero_dom::NodeId, ComputedStyle>,
        container_id: zero_dom::NodeId,
    }

    /// 构造 taffy 树 + LayoutBox + dom_to_taffy，模拟 vertical-rl 容器（width/height Auto）
    /// 含 2 个 50×150 block 子（border 2px 左右）。container_style_height 控制 height 维度。
    fn build_sizing_fixture(wm: WritingModeValue, container_height: LengthValue) -> SizingFixture {
        let mut doc = Document::new();
        let container_id = doc.create_element("div");
        let b1 = doc.create_element("div");
        let b2 = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut cs = ComputedStyle::default();
        cs.display = DisplayValue::Block;
        cs.writing_mode = wm.clone();
        cs.width = LengthValue::Auto;
        cs.height = container_height;
        styles.insert(container_id, cs);
        let mut bs = ComputedStyle::default();
        bs.display = DisplayValue::Block;
        bs.width = LengthValue::Px(50.0);
        bs.height = LengthValue::Px(150.0);
        styles.insert(b1, bs.clone());
        styles.insert(b2, bs);

        // taffy 树：2 leaf 子 + 1 容器（with_children）。
        let mut taffy_tree: TaffyTree<zero_dom::NodeId> = TaffyTree::new();
        let b1_t = taffy_tree.new_leaf_with_context(taffy::Style::default(), b1).unwrap();
        let b2_t = taffy_tree.new_leaf_with_context(taffy::Style::default(), b2).unwrap();
        let container_t = taffy_tree
            .new_with_children(taffy::Style::default(), &[b1_t, b2_t])
            .unwrap();
        let mut dom_to_taffy = HashMap::new();
        dom_to_taffy.insert(container_id, container_t);
        dom_to_taffy.insert(b1, b1_t);
        dom_to_taffy.insert(b2, b2_t);

        // LayoutBox 树（模拟第一趟 taffy：容器 800 宽 / Σ=300 高堆叠，子 50×150）。
        let mut container = LayoutBox::default();
        container.node_id = Some(container_id);
        container.writing_mode = wm;
        container.border_left = 2.0;
        container.border_right = 2.0;
        container.width = 800.0;
        container.height = 300.0; // taffy vertical-stack Σ
        container.content_width = 796.0;
        for id in [b1, b2] {
            let mut child = LayoutBox::default();
            child.node_id = Some(id);
            child.writing_mode = WritingModeValue::HorizontalTb;
            child.width = 50.0;
            child.height = 150.0;
            container.children.push(child);
        }

        SizingFixture {
            taffy: taffy_tree,
            container,
            dom_to_taffy,
            styles,
            container_id,
        }
    }

    /// Auto w/h + height_enabled：sizing 把容器 width→Σ(100)+frame(4)=104、
    /// height→max(150)+frame(0)=150 喂回 taffy。
    #[test]
    fn test_sizing_sets_auto_dimensions() {
        let mut f = build_sizing_fixture(WritingModeValue::VerticalRl, LengthValue::Auto);
        let changed =
            apply_vertical_block_flow_sizing_inner(&mut f.taffy, &f.container, &f.dom_to_taffy, &f.styles, true);
        assert!(changed, "Auto 容器应触发 sizing");
        let size = f.taffy.style(f.dom_to_taffy[&f.container_id]).unwrap().size;
        // Dimension 是 newtype struct，用相等比较（length/auto 构造子）。
        assert_eq!(
            size.width,
            taffy::style::Dimension::length(104.0),
            "width 应=Σ(100)+frame(4)=104"
        );
        assert_eq!(
            size.height,
            taffy::style::Dimension::length(150.0),
            "height 应=max(150)+frame(0)=150"
        );
    }

    /// definite height（height:8em→Px）：sizing **不覆盖** height（保留 definite），仅覆盖 width。
    #[test]
    fn test_sizing_respects_definite_height() {
        let mut f = build_sizing_fixture(WritingModeValue::VerticalRl, LengthValue::Px(80.0));
        let changed =
            apply_vertical_block_flow_sizing_inner(&mut f.taffy, &f.container, &f.dom_to_taffy, &f.styles, true);
        assert!(changed, "width Auto 仍应触发 sizing");
        let size = f.taffy.style(f.dom_to_taffy[&f.container_id]).unwrap().size;
        // width 被设为 Σ+frame；height 保持 Auto（definite Px 不覆盖）。
        assert_eq!(size.width, taffy::style::Dimension::length(104.0), "width 应≈104");
        assert_eq!(
            size.height,
            taffy::style::Dimension::auto(),
            "definite height 不应被覆盖（应保持 Auto 由 taffy 按 definite style 解析）"
        );
    }

    /// height_enabled=false（default）：width-only，不设 height（避 float/inline-block 回归）。
    #[test]
    fn test_sizing_width_only_when_height_disabled() {
        let mut f = build_sizing_fixture(WritingModeValue::VerticalRl, LengthValue::Auto);
        let changed =
            apply_vertical_block_flow_sizing_inner(&mut f.taffy, &f.container, &f.dom_to_taffy, &f.styles, false);
        assert!(changed, "width Auto 应触发 sizing");
        let size = f.taffy.style(f.dom_to_taffy[&f.container_id]).unwrap().size;
        assert_eq!(size.width, taffy::style::Dimension::length(104.0), "width 应≈104");
        assert_eq!(
            size.height,
            taffy::style::Dimension::auto(),
            "height_disabled 时 height 保持 Auto"
        );
    }

    /// env kill-switch：`ZW_VERTICAL_BLOCK_FLOW_LAYOUT=0` 禁用 sizing（default-on，=0 关闭）。
    #[test]
    fn test_sizing_env_kill_switch() {
        // env 操作在 edition 2024 为 unsafe。SAFETY: 测试串行段，set 后立即读。
        unsafe {
            std::env::set_var("ZW_VERTICAL_BLOCK_FLOW_LAYOUT", "0");
        }
        let mut f = build_sizing_fixture(WritingModeValue::VerticalRl, LengthValue::Auto);
        let changed = apply_vertical_block_flow_sizing(&mut f.taffy, &f.container, &f.dom_to_taffy, &f.styles);
        assert!(!changed, "kill-switch=0 时不应触发 sizing");
        unsafe {
            std::env::remove_var("ZW_VERTICAL_BLOCK_FLOW_LAYOUT");
        }
    }
}
