//! R5446：嵌套 inline float 收集器（inline wrapper 子树内的 float 并入容器 IFC float
//! 机制）。从 `inline_finalization.rs` 抽出（R5448，2000 行规则）。
//!
//! 覆盖面：①锚行顶 y ②§9.5.1 rule 6 方向极值缘 x ③exclusion 带（文本绕排）④BFC
//! 根 auto 高计入。vertical 书写容器不入（guidelines #29：vertical 轴模型另一套，
//! 另案挂账）。kill-switch `ZW_NESTED_INLINE_FLOAT=0` 回退旧行为。

use std::collections::{HashMap, HashSet};

use zero_css_parser::values::{DisplayValue, FloatValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// R5446：嵌套 float kill-switch（collect 侧 FloatAnchor 补发臂同门——放置侧与收集侧
/// 必须同一开关回退，半开状态会产生有锚无放置/有放置无锚的分裂行为）。
static NESTED_INLINE_FLOAT: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_NESTED_INLINE_FLOAT").as_deref() != Ok("0"));

/// R5446：嵌套 float kill-switch 查询（collect 侧 FloatAnchor 补发臂同门——放置侧与
/// 收集侧必须同一开关回退，半开状态会产生有锚无放置/有放置无锚的分裂行为）。
pub(crate) fn nested_inline_float_enabled() -> bool {
    *NESTED_INLINE_FLOAT
}

/// R5446：嵌套 float 摘要（collector 产出；锚 pass 后回填 `target`）。
pub(crate) struct NestedInlineFloatInfo {
    pub(crate) fid: NodeId,
    pub(crate) is_left: bool,
    pub(crate) width: f32,
    pub(crate) margin_left: f32,
    pub(crate) margin_right: f32,
    pub(crate) margin_top: f32,
    pub(crate) margin_bottom: f32,
    pub(crate) height: f32,
    /// 写入目标（容器 content-box 相对）：Some((y, x))——y=锚行顶（§9.5.1 rule 5/6
    /// 尽高；嵌套 float 从未获得有效静态位：步 5 walk 对 inline 子 `continue`、taffy
    /// 以 wrapper 帧堆叠，故无条件取锚，R5442 静态位保留门不适用）；x=rule 6 方向
    /// 极值缘（Left→左缘 +ml，Right→右缘 −mr）。None=锚缺失，该 float 退回旧行为。
    pub(crate) target: Option<(f32, f32)>,
}

/// R5446：inline 级可下降 wrapper 判定——非块级、非原子（inline-block 族/replaced）、
/// 非 BFC（其 float 子树归自身容器，各有 remeasure 递归）。
pub(crate) fn nested_float_wrapper_descendable(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    if child.is_absolute || child.is_fixed || child.is_block_level || child.is_replaced {
        return false;
    }
    if crate::margin_collapse::establishes_bfc(child) {
        return false;
    }
    !child.node_id.is_some_and(|id| {
        styles.get(&id).is_some_and(|s| {
            matches!(
                s.display,
                DisplayValue::InlineBlock
                    | DisplayValue::InlineTable
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
            )
        })
    })
}

/// R5446：收集 wrapper 子树内嵌套 float 的 margin-box 排除带（root 盒坐标系）——
/// 供 `compute_final_inline_layouts` 并入 `own_floats`。调用时机在 step 6 remeasure
/// 写回之后：嵌套 float 盒几何已是终位（§9.5.1 rule 6 极值缘 + 锚行顶），构造公式
/// 与直接 float 子臂一致（R5312 margin-box 口径：x−ml / y−mt / w+ml+mr / h+mt+mb）。
/// 排除带非空使 float-aware fail-closed 门成立，容器落 re-break 存储路径（否则
/// `can_skip_final_ifc` 早退 → `inline_layout` 缺存储 → paint Path B 无锚重跑分段
/// 错乱）。几何无效（w/h≤0 或 y<0）的 float 不产带（同直接臂过滤）。vertical 书写
/// 容器不入（guidelines #29）。kill-switch `ZW_NESTED_INLINE_FLOAT=0`。
pub(crate) fn collect_nested_float_exclusions(
    box_node: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Vec<crate::inline::FloatExclusion> {
    fn walk(
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        dx: f32,
        dy: f32,
        depth: usize,
        out: &mut Vec<crate::inline::FloatExclusion>,
    ) {
        for child in &box_node.children {
            if child.is_absolute || child.is_fixed {
                continue;
            }
            if !matches!(child.float, FloatValue::None) {
                // depth=0（容器直接 float 子）不入——`own_floats` 直接臂已收集，重复
                // 入带会双计（带重复对 min/max 收窄幂等，但 fail-closed 门与后续按带
                // 消费的路径不受控）。
                if depth == 0 {
                    continue;
                }
                let rx = dx + child.x;
                let ry = dy + child.y;
                if ry < 0.0 || child.width <= 0.0 || child.height <= 0.0 {
                    continue;
                }
                let Some(_) = child.node_id else {
                    continue;
                };
                out.push(crate::inline::FloatExclusion {
                    y: ry - child.margin_top,
                    height: child.margin_top + child.height + child.margin_bottom,
                    width: child.width + child.margin_left + child.margin_right,
                    x: rx - child.margin_left,
                    is_left: matches!(child.float, FloatValue::Left),
                });
            } else if nested_float_wrapper_descendable(child, styles) {
                walk(
                    child,
                    styles,
                    dx + child.x + child.border_left + child.padding_left,
                    dy + child.y + child.border_top + child.padding_top,
                    depth + 1,
                    out,
                );
            }
        }
    }
    let mut out = Vec::new();
    walk(box_node, styles, 0.0, 0.0, 0, &mut out);
    out
}

/// R5446：收集 inline wrapper 子树下的嵌套 float（只沿 [`nested_float_wrapper_descendable`]
/// 子下降；直接 float 子归既有路径不入本列）。同时给出已认领的顶层子索引集。
pub(crate) fn collect_nested_inline_floats(
    box_node: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> (Vec<NestedInlineFloatInfo>, HashSet<usize>) {
    fn walk(
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        top_idx: Option<usize>,
        out: &mut Vec<NestedInlineFloatInfo>,
        claimed: &mut HashSet<usize>,
    ) {
        for (i, child) in box_node.children.iter().enumerate() {
            let ti = top_idx.unwrap_or(i);
            if child.is_absolute || child.is_fixed {
                continue;
            }
            if !matches!(child.float, FloatValue::None) {
                // 顶层（top_idx=None）float 是直接子，归既有 direct 路径（步 5 锚移 +
                // direct 排除带 + float_bottom），不入本列——否则双重处置（嵌套目标把
                // 堆叠 float 二次搬到极值缘/锚顶，floats-006 蓝块错位实证）。
                if top_idx.is_none() {
                    continue;
                }
                let Some(fid) = child.node_id else {
                    continue;
                };
                claimed.insert(ti);
                out.push(NestedInlineFloatInfo {
                    fid,
                    is_left: matches!(child.float, FloatValue::Left),
                    width: child.width,
                    margin_left: child.margin_left,
                    margin_right: child.margin_right,
                    margin_top: child.margin_top,
                    margin_bottom: child.margin_bottom,
                    height: child.height,
                    target: None,
                });
                continue;
            }
            if nested_float_wrapper_descendable(child, styles) {
                walk(child, styles, Some(ti), out, claimed);
            }
        }
    }
    let mut out = Vec::new();
    let mut claimed = HashSet::new();
    walk(box_node, styles, None, &mut out, &mut claimed);
    (out, claimed)
}

/// R5446：嵌套 float 存在性早退检查（`remeasure_inline_only_containers` 的
/// has_floats 门同步——否则 inline-only 重测会覆写容器级刚写入的 float 高）。
pub(crate) fn has_nested_inline_float(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    fn walk(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
        for child in &box_node.children {
            if child.is_absolute || child.is_fixed {
                continue;
            }
            if !matches!(child.float, FloatValue::None) {
                return true;
            }
            if nested_float_wrapper_descendable(child, styles) && walk(child, styles) {
                return true;
            }
        }
        false
    }
    walk(box_node, styles)
}

/// R5446：sync 后按父链偏移回写嵌套 float 目标位——目标为容器 content-box 相对，
/// 盒字段相对其直接父级，故逐层累计 (x+bl+pl, y+bt+pt) 链偏移后反解
///（wrapper 盒几何已被 IFC sync 重写，链偏移必须读 sync 后现值）。
pub(crate) fn apply_nested_float_writeback(
    box_node: &mut LayoutBox,
    targets: &[(NodeId, f32, f32)],
    chain_x: f32,
    chain_y: f32,
) {
    for child in &mut box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        if let Some(fid) = child.node_id
            && let Some((_, tx, ty)) = targets.iter().find(|t| t.0 == fid)
        {
            child.x = tx - chain_x;
            child.y = ty - chain_y;
            continue;
        }
        apply_nested_float_writeback(
            child,
            targets,
            chain_x + child.x + child.border_left + child.padding_left,
            chain_y + child.y + child.border_top + child.padding_top,
        );
    }
}
