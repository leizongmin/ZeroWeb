//! R5409：visibility:collapse flex item 多线 strut 迁移（css-flexbox-1 §4 Collapsed
//! Items，flexbox-collapsed-item-horiz-002）。
//!
//! 规范语义：collapsed item 先按可见参与断线（strut 所在线的交叉尺寸被记为该 item
//! 的 strut），随后以零主尺寸重跑断线——item 可能迁入其他线，strut 随之迁移并贡献
//! 给落线（「carries the strut between lines」）。单线容器 strut 主尺寸语义由
//! converter `ZW_VC_NONFLEX_STRUT`（非 flexible collapsed 保留 flex-basis）承担
//! （flexbox-collapsed-item-horiz-001 判例）；多线（flex-wrap）容器下保留 basis 会
//! 阻断迁移（002 Row2/Row3）。
//!
//! 真值口径（chromium oracle 2026-11-04 探针定谳）：chromium **不实现** collapsed
//! strut 迁移——case 页两案（001 单线全尺寸保留 / 002 wrap 形态首趟布局原样）
//! 与各自 ref 页（编码规范几何）在 chromium 内即不一致，上游双挂；本仓 corpus 为
//! self-source（ZW case 页 vs ZW ref 页），002 修复目标 = ref 几何（002 ref 逐行
//! 解码 = 本 pass 两趟语义：R2a 迁移 25 strut、R3a/R3b 迁移 + 40 strut 全吻合）。
//! 单线零主尺寸候选面（001 ref = 主 0 + strut 40，现 ZW = basis-keep 1.42% 宽限
//! 过案）另案挂账。
//!
//! 实现（taffy 单树两趟近似）：第一趟布局即 converter strut-basis 形态（=规范首趟），
//! 按 taffy 输出对容器子项 y 分组得首趟线，每线交叉尺寸 = 线内最大高（strut 源）；
//! 随后 patch 每个 collapsed 子项 flex_basis→0（零主尺寸断线）+ 高度→strut（交叉
//! 贡献换轨为 strut，taffy 原生按 align-content 堆线），mark_dirty 入既有重跑组。
//! kill-switch `ZW_FLEX_COLLAPSE_STRUT_MIGRATE=0`（default-on）。
//!
//! 限定：水平书写模式（R4076 同门：vertical 主/交叉轴互换另案）；仅 wrap 容器
//! （单线容器 horiz-001 strut 主尺寸语义不受扰动）；strut 仅计非 flexible collapsed
//! （flexible collapsed 的首趟 flexing 语义 FIXME 挂账，converter 已对其置零 basis）；
//! strut 取线内最大 border-box 高，不含 margin（corpus 案 margin 全 0，边缘另案）。
use std::collections::HashMap;

use crate::types::LayoutBox;
use taffy::TaffyTree;
use zero_css_parser::values::{DisplayValue, FlexWrapValue, VisibilityValue};
use zero_dom::NodeId;
use zero_style_system::{ComputedStyle, WritingModeValue};

/// R5409 kill-switch 读取（进程级缓存）：`ZW_FLEX_COLLAPSE_STRUT_MIGRATE=0` 回退
/// 多线 strut 迁移臂（guideline #29 按趟开关，OnceLock 同 R5408 口径）。
pub(crate) fn flex_collapse_strut_migrate_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_FLEX_COLLAPSE_STRUT_MIGRATE").as_deref() != Ok("0"))
}

/// converter `ZW_VC_NONFLEX_STRUT` 前提读取：`=0` 时 collapsed basis 已全零，本臂
/// 的「当前布局 = 规范首趟」前提不成立，整 pass 跳过（趟级单读，guideline #9）。
fn converter_strut_premise_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_VC_NONFLEX_STRUT").as_deref() != Ok("0"))
}

/// 扫描水平书写 wrap flex 容器内的 collapsed 子项，按首趟线注入 strut 迁移
/// （详见模块文档）。返回是否有 taffy style 修改（调用方据此 mark_dirty 重跑）。
pub(crate) fn apply_flex_collapsed_strut_migration(
    taffy_tree: &mut TaffyTree<NodeId>,
    root: &LayoutBox,
    dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    if !flex_collapse_strut_migrate_enabled() || !converter_strut_premise_enabled() {
        return false;
    }
    fn walk(
        b: &LayoutBox,
        taffy_tree: &mut TaffyTree<NodeId>,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> bool {
        if b.writing_mode != WritingModeValue::HorizontalTb {
            return false;
        }
        let mut changed = false;
        if let Some(id) = b.node_id
            && let Some(cs) = styles.get(&id)
            && matches!(cs.display, DisplayValue::Flex | DisplayValue::InlineFlex)
            && !matches!(cs.flex_wrap, FlexWrapValue::Nowrap)
            && !b.children.is_empty()
        {
            changed |= migrate_container(b, taffy_tree, dom_to_taffy, styles);
        }
        for c in &b.children {
            changed |= walk(c, taffy_tree, dom_to_taffy, styles);
        }
        changed
    }
    walk(root, taffy_tree, dom_to_taffy, styles)
}

/// 单容器迁移：子项按首趟布局 y 分组断线，collapsed 子项 strut = 所在线交叉尺寸，
/// patch `flex_basis→0` + `height→strut`。
fn migrate_container(
    b: &LayoutBox,
    taffy_tree: &mut TaffyTree<NodeId>,
    dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    // 子项（taffy id, 首趟 y, 首趟 border-box 高, 是否非 flexible collapsed）。
    // 任一子项缺 taffy 节点/样式/布局输出即整容器跳过（分组不完整会错算 strut）。
    struct Child {
        tid: taffy::NodeId,
        y: f32,
        h: f32,
        collapsed: bool,
    }
    let mut children = Vec::with_capacity(b.children.len());
    for c in &b.children {
        let Some(nid) = c.node_id else { return false };
        let Some(cs) = styles.get(&nid) else { return false };
        let Some(tid) = dom_to_taffy.get(&nid).copied() else {
            return false;
        };
        let Ok(l) = taffy_tree.layout(tid) else { return false };
        // 非 growable 限定（converter strut basis 域同门，converter/mod.rs flex_basis
        // 分支同判 flex_grow>0）：growable collapsed 首趟参与 flexing，strut 语义不同
        // （FIXME：css-flexbox §4 flexible strut 另案）。flex_shrink 不影响 strut
        // basis（basis 为收缩前主尺寸，ComputedStyle 保留声明值，默认 shrink=1）。
        let collapsed = matches!(cs.visibility, VisibilityValue::Collapse) && cs.flex_grow <= 0.0;
        children.push(Child {
            tid,
            y: l.location.y,
            h: l.size.height,
            collapsed,
        });
    }
    if !children.iter().any(|c| c.collapsed) {
        return false;
    }
    // 首趟线分组：align-items/flex-start 或 stretch 下同线子项共享线顶 y（0.5px 容差）。
    // 线交叉尺寸 = 线内最大 border-box 高（strut 源；margin 未计入，见模块 FIXME）。
    let mut line_cross: Vec<f32> = Vec::new();
    let mut line_of: Vec<usize> = Vec::with_capacity(children.len());
    for c in &children {
        let line = line_cross.iter().position(|y0| (*y0 - c.y).abs() <= 0.5);
        let line = match line {
            Some(i) => i,
            None => {
                line_cross.push(c.y);
                line_cross.len() - 1
            }
        };
        line_of.push(line);
    }
    let mut line_max_h: Vec<f32> = line_cross.iter().map(|_| 0.0_f32).collect();
    for (c, line) in children.iter().zip(&line_of) {
        line_max_h[*line] = line_max_h[*line].max(c.h);
    }
    let mut changed = false;
    for (c, line) in children.iter().zip(&line_of) {
        if !c.collapsed {
            continue;
        }
        let strut = line_max_h[*line];
        let Ok(mut st) = taffy_tree.style(c.tid).cloned() else {
            continue;
        };
        let mut dirty = false;
        // 零主尺寸断线：converter 保留的 strut basis 归零（css-flexbox §4「post-collapse
        // zero main-size」）。basis 已零（长度解析 < 0.5）时免写；Auto/percent 不作
        // 零判定（percent 以 0 上下文解析的边缘无案驱动，FIXME）。
        let basis_zero = st.flex_basis.into_option().is_some_and(|v| v < 0.5);
        if !basis_zero {
            st.flex_basis = taffy::style::Dimension::length(0.0_f32);
            dirty = true;
        }
        // 交叉贡献换轨 strut：strut ≥ 自身首趟高（线 max 恒含自身），仅长高不缩。
        // 相等时免写（Row1/Row2 形态 strut == 自身高，保留 CSS height 语义）。
        if strut > c.h + 0.5 {
            st.size.height = taffy::style::Dimension::length(strut);
            dirty = true;
        }
        if dirty {
            let _ = taffy_tree.set_style(c.tid, st);
            let _ = taffy_tree.mark_dirty(c.tid);
            changed = true;
        }
    }
    changed
}
