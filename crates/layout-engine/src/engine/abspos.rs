//! 绝对/固定定位后处理（abspos post-processing）。
//!
//! R831 从 `engine.rs` 抽出（2000 行规则）：5 个自包含的定位后处理函数，
//! 在 taffy 布局完成后对 `position: absolute/fixed` 元素的 LayoutBox 树做坐标修正
//!（fixed→视口相对、absolute 百分比→视口、abspos 根 CB 解析等）。零私有 helper
//! 依赖（仅用 LayoutBox + LengthValue），经 engine.rs 的 `use abspos::*` 调用
//!（18 处 call site 不变）。纯移动，零行为变化。

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// 递归调整 fixed 定位元素的坐标为视口相对。
///
/// taffy 将 `position: fixed` 当作 `absolute` 处理，坐标是相对于包含块的。
/// 此函数在布局完成后遍历布局树，将 fixed 元素的坐标加上祖先累积偏移，
/// 使其变为相对于视口的绝对坐标。
pub(super) fn adjust_fixed_to_viewport(box_node: &mut LayoutBox, parent_offset_x: f32, parent_offset_y: f32) {
    if box_node.is_fixed {
        // R324：fixed 元素须视口相对。taffy 0.7 把 fixed 当 absolute 处理（containing
        // block = 最近 positioned 祖先），故 box.x/y 编码的是相对该祖先的 left/top。
        // 视口相对 = 同一 left/top 数值但相对视口 → 需从累积祖先偏移中【扣除】
        //（而非旧实现的「加上」——旧实现仅在 parent_offset==0 时碰巧正确，对有偏移
        // positioned 祖先的 fixed 会 over-correct，如 fixed-inside-relative-ancestor）。
        box_node.x -= parent_offset_x;
        box_node.y -= parent_offset_y;
    }

    let offset_x = if box_node.is_fixed {
        0.0
    } else {
        parent_offset_x + box_node.x
    };
    let offset_y = if box_node.is_fixed {
        0.0
    } else {
        parent_offset_y + box_node.y
    };

    for child in &mut box_node.children {
        adjust_fixed_to_viewport(child, offset_x, offset_y);
    }
}

/// 将没有 positioned ancestor 的 absolute 元素修正为相对于初始包含块。
///
/// 仅对 `position:absolute` 且路径上不存在 `position != static` 祖先的元素生效。
/// 这避免 body 的外边距或静态祖先的偏移被重复计入 abs-pos 元素坐标。
///
/// 注意：此功能当前导致多个回归，暂不启用。
#[allow(dead_code)]
pub(super) fn adjust_absolute_to_initial_containing_block(
    box_node: &mut LayoutBox,
    current_content_origin_x: f32,
    current_content_origin_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    has_positioned_ancestor: bool,
) {
    let child_has_positioned_ancestor = has_positioned_ancestor
        || box_node.is_absolute
        || box_node.is_fixed
        || box_node.is_relative
        || box_node.is_sticky;

    for child in &mut box_node.children {
        // 使用 child_has_positioned_ancestor 而非 has_positioned_ancestor，
        // 因为当前节点自身（如 position:relative）也是 positioned ancestor。
        if child.is_absolute && !child_has_positioned_ancestor {
            child.x -= current_content_origin_x;
            child.y -= current_content_origin_y;

            if let Some(style) = child.node_id.and_then(|node_id| styles.get(&node_id)) {
                if matches!(style.width, zero_css_parser::values::LengthValue::Auto) {
                    child.width += (viewport_width - box_node.content_width).max(0.0);
                }
                if matches!(style.height, zero_css_parser::values::LengthValue::Auto) {
                    child.height += (viewport_height - box_node.content_height).max(0.0);
                }
            }
        }

        let child_content_origin_x = current_content_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_content_origin_y = current_content_origin_y + box_node.border_top + box_node.padding_top + child.y;
        adjust_absolute_to_initial_containing_block(
            child,
            child_content_origin_x,
            child_content_origin_y,
            viewport_width,
            viewport_height,
            styles,
            child_has_positioned_ancestor || child.is_absolute,
        );
    }
}

/// 修正无 positioned ancestor 的 absolute 元素的**百分比** inset 与尺寸。
///
/// CSS 2.1 §10.1：absolute 元素无 positioned ancestor 时，containing block 是
/// 初始包含块（视口）。但 taffy 用静态父作为 containing block，导致 `width:50%`、
/// `left:50%` 等百分比按父宽度（而非视口宽度）解析。
///
/// 本函数**只重解析百分比**（Length/Percent::Auto 不动），避免历史上
/// `adjust_absolute_to_initial_containing_block` 因同时调整 x/y 偏移和 auto 宽高
/// 导致的回归（static-inside-inline-block、background-329 等）。
///
/// 坐标系：LayoutBox.x/y 相对父内容盒原点。paint 链逐层累加得到视口绝对坐标。
/// `current_content_origin_x/y` 是当前盒内容盒原点的视口绝对坐标。
pub(super) fn adjust_absolute_pct_to_viewport(
    box_node: &mut LayoutBox,
    current_content_origin_x: f32,
    current_content_origin_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    has_positioned_ancestor: bool,
) {
    use zero_css_parser::values::LengthValue;
    let child_has_positioned_ancestor = has_positioned_ancestor
        || box_node.is_absolute
        || box_node.is_fixed
        || box_node.is_relative
        || box_node.is_sticky;

    for child in &mut box_node.children {
        if child.is_absolute
            && !child_has_positioned_ancestor
            && let Some(style) = child.node_id.and_then(|node_id| styles.get(&node_id))
        {
            // R880：`current_content_origin_x/y` 是父盒（box_node）的 **border-box**
            // 视口原点（见下方递归 line：传给子的是 border-box origin），而子盒的
            // `child.x/y` 是相对父盒 **content box**（= border-box + border + padding）
            // 的偏移（taffy 约定）。viewport-CB abspos 的目标视口坐标须转回父 content
            // 相对坐标，故减父 content origin（非 border-box origin）——否则当 CB 链含
            // border/padding 时位置偏移（abspos-containing-block-010：body border+padding
            // 1em 致 abspos div 落 (32,32) 而非视口 (0,0)）。无 border/padding 的 CB 链
            // 二者相等，行为不变（R98/R872 测试均 borderless CB 故此前未暴露）。
            let parent_content_origin_x = current_content_origin_x + box_node.border_left + box_node.padding_left;
            let parent_content_origin_y = current_content_origin_y + box_node.border_top + box_node.padding_top;
            // 仅当 width 为百分比时按视口重解析
            if let LengthValue::Percentage(p) = &style.width {
                child.width = *p as f32 / 100.0 * viewport_width;
            }
            if let LengthValue::Percentage(p) = &style.height {
                child.height = *p as f32 / 100.0 * viewport_height;
            }
            // auto 尺寸 + 全长度 inset → stretch（CSS §10.3.18 / §10.6.4，仅非替换）。
            // 仅当 left+right（或 top+bottom）均为长度且尺寸为 auto 时按视口 CB
            // stretch；与历史 adjust_absolute_to_initial_containing_block 的「无条件
            // 扩张 auto 宽高」（width += viewport - content，致 static-inside-inline-block
            // / background-329 回归）不同——本块严格匹配 spec 的「双 inset 才 stretch」，
            // 不动 x/y（位置已由下方 Px left/top 块设好）。`!is_replaced` 仅守卫本块：
            // §10.3.8 替换元素 auto 尺寸按固有尺寸解析（非 stretch），但 §10.1.4 的
            // viewport-CB 定位与百分比尺寸对替换/非替换同等适用，故守卫不扩到整分支
            // （避免误关 R98 位置/百分比尺寸解析，致替换 abspos 定位回退）。
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (viewport_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (viewport_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // left/top 百分比：目标视口绝对坐标 = p/100 * viewport，转回父 content 相对坐标
            if let LengthValue::Percentage(p) = &style.left {
                let target_viewport_x = *p as f32 / 100.0 * viewport_width;
                child.x = target_viewport_x - parent_content_origin_x;
            }
            if let LengthValue::Percentage(p) = &style.top {
                let target_viewport_y = *p as f32 / 100.0 * viewport_height;
                child.y = target_viewport_y - parent_content_origin_y;
            }
            // left/top 为长度（Px）时：CSS 2.1 §10.1 规定无 positioned ancestor 的
            // absolute 元素以初始包含块（视口）为 containing block。taffy 用静态父
            // 作 containing block，导致 top:118px 解析为静态父相对坐标。此处把目标
            // 视口坐标（= px 值）转回父 content 相对坐标，与百分比路径同机制（不调整
            // auto 宽高，避免历史上 auto 宽高扩张导致的回归）。
            if let LengthValue::Px(px) = &style.left {
                child.x = (*px as f32) - parent_content_origin_x;
            }
            if let LengthValue::Px(px) = &style.top {
                child.y = (*px as f32) - parent_content_origin_y;
            }
            // right/bottom 为长度且 left/top 为 auto 时：CSS 2.1 §10.1 无 positioned
            // ancestor 的 absolute 元素 CB=视口。left:auto + right:Px → 右边对齐视口
            // 右缘，由已解析的 width 反解 left（§10.3.18 rule 2）：
            // target_x = viewport_w - right - width。须在 width/height 解析后执行
            // （上方百分比/auto-stretch 块已设好 child.width/height）。left/top 已为
            // Px 时由上方块处理；双 inset 全 Px 的 over-constrained（LTR）忽略 right。
            // right/bottom 百分比仅当对应尺寸为 auto 时才影响位置，当前不处理。
            if matches!(style.left, LengthValue::Auto)
                && let LengthValue::Px(right) = &style.right
            {
                let target_viewport_x = viewport_width - (*right as f32) - child.width;
                child.x = target_viewport_x - parent_content_origin_x;
            }
            if matches!(style.top, LengthValue::Auto)
                && let LengthValue::Px(bottom) = &style.bottom
            {
                let target_viewport_y = viewport_height - (*bottom as f32) - child.height;
                child.y = target_viewport_y - parent_content_origin_y;
            }
            // §10.3.7：width:auto + 全长度 left+right 填满后，max-width 钳制，再把
            // over-constrained 方程的 leftover 重分配到 auto-margin（abspos 无 positioned
            // 祖先时 CB=viewport）。taffy 0.7 不钳 abspos inset-fill 宽。须在 width
            // stretch + left/right 定位之后执行（覆盖上方 x 定位）。target_viewport_x
            // 转回父 content 相对坐标（与上方各块同机制）。
            if matches!(style.width, LengthValue::Auto)
                && let (LengthValue::Px(left_v), LengthValue::Px(right_v)) = (&style.left, &style.right)
                && let LengthValue::Px(mw_v) = &style.max_width
                && child.width > *mw_v as f32 + 0.5
            {
                let (left, right, mw) = (*left_v as f32, *right_v as f32, *mw_v as f32);
                let leftover = (viewport_width - left - right - mw).max(0.0);
                let ml_auto = matches!(style.margin_left, LengthValue::Auto);
                let mr_auto = matches!(style.margin_right, LengthValue::Auto);
                let target_viewport_x = if ml_auto && mr_auto {
                    // 两侧 auto → 居中
                    let m = leftover / 2.0;
                    child.margin_left = m;
                    child.margin_right = m;
                    left + m
                } else if ml_auto {
                    // 仅 margin-left auto → 吸收 leftover（右对齐）
                    child.margin_left = leftover;
                    left + leftover
                } else if mr_auto {
                    // 仅 margin-right auto → x 留在 left（左对齐）
                    child.margin_right = leftover;
                    left
                } else {
                    // 无 auto margin，over-constrained → 忽略 right，x=left
                    left
                };
                child.width = mw;
                child.content_width =
                    (mw - child.border_left - child.border_right - child.padding_left - child.padding_right).max(0.0);
                child.x = target_viewport_x - parent_content_origin_x;
            }
        }

        // 递归：用（可能已修改的）child 位置计算其内容盒原点
        let child_content_origin_x = current_content_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_content_origin_y = current_content_origin_y + box_node.border_top + box_node.padding_top + child.y;
        adjust_absolute_pct_to_viewport(
            child,
            child_content_origin_x,
            child_content_origin_y,
            viewport_width,
            viewport_height,
            styles,
            child_has_positioned_ancestor || child.is_absolute,
        );
    }
}

/// 对 position:fixed 元素的全-inset stretch 尺寸后处理（CSS §10.3.18 / §10.6.4）。
///
/// fixed 元素的 containing block 是视口。当 top+bottom 均为长度且 height:auto 时，
/// height = viewport_h - top - bottom；left+right 均为长度且 width:auto 时，
/// width = viewport_w - left - right。taffy 0.7 把 fixed 当 absolute 处理
/// （CB=最近 positioned 祖先），尺寸按该祖先而非视口 stretch，导致全-inset fixed
/// 元素尺寸不足（典型：全 0 inset 应覆盖视口却塌缩为内容固有尺寸）。
///
/// 仅处理 fixed（CB=视口无条件已知，零位置风险——位置已由 adjust_fixed_to_viewport
/// 修正）。不处理 absolute（CB=positioned 祖先，layout 后方知；历史
/// adjust_absolute_to_initial_containing_block 同调 auto 宽高致多回归故禁用）。
/// R1139：root 元素自身 abspos/fixed 的全-inset stretch 在本函数之外（见
/// [`stretch_root_abspos_to_viewport`]），因本函数只递归 `box_node.children`，
/// root 自身（无父）不被触。
pub(super) fn stretch_fixed_to_viewport_size(
    box_node: &mut LayoutBox,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if child.is_fixed
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // height: auto + 全长度 top+bottom → stretch
            if matches!(style.height, LengthValue::Auto)
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (viewport_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // width: auto + 全长度 left+right → stretch
            if matches!(style.width, LengthValue::Auto)
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (viewport_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            // 百分比尺寸：fixed 的 CB 恒为视口（CSS §10.1），百分比相对视口解析。
            // taffy 按 positioned 祖先解析（如 body CB），此处按视口重算。
            if let LengthValue::Percentage(p) = &style.height {
                child.height = (*p as f32 / 100.0) * viewport_height;
            }
            if let LengthValue::Percentage(p) = &style.width {
                child.width = (*p as f32 / 100.0) * viewport_width;
            }
        }
        stretch_fixed_to_viewport_size(child, viewport_width, viewport_height, styles);
    }
}

/// R1139：root 元素自身 `position:absolute`/`fixed` + 全长度 inset + auto 尺寸的 stretch
/// 后处理（CSS §10.3.18 / §10.6.4）。root 元素的 CB = initial containing block（视口）。
///
/// [`stretch_fixed_to_viewport_size`] 只递归 `box_node.children`，root 自身（LayoutBox 树
/// 顶层、无父）不被触；且历史 absolute stretch 被禁用（CB=positioned 祖先，layout 后方知，
/// 同调 auto 宽高致回归）。但**root 元素自身** abspos/fixed 的 CB 恒为视口（与 fixed 同语义），
/// stretch 安全——`position-{absolute,fixed}-root-element-{flex,grid}` 4 案（html root 全
/// inset，应 stretch 到视口减 inset，旧实现 height 塌缩到内容 ~65px ≠ 应 530px，diff 4.46%）。
///
/// 仅处理 root 自身（gated `is_absolute || is_fixed`），全长度 inset + auto 尺寸时 stretch；
/// 位置（x/y）按 left/top inset 设（CB 原点 = 视口 0,0）。非 abspos/fixed root 零影响。
pub(super) fn stretch_root_abspos_to_viewport(
    root: &mut LayoutBox,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;
    if !(root.is_absolute || root.is_fixed) {
        return;
    }
    let Some(style) = root.node_id.and_then(|nid| styles.get(&nid)) else {
        return;
    };
    // 位置：root CB 原点 = 视口 (0,0)，left/top Px → 绝对坐标。
    if let LengthValue::Px(left) = &style.left {
        root.x = *left as f32;
    }
    if let LengthValue::Px(top) = &style.top {
        root.y = *top as f32;
    }
    // 尺寸 stretch：auto + 全长度对边 inset → viewport - inset（§10.3.18/§10.6.4）。
    if matches!(style.width, LengthValue::Auto)
        && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
    {
        root.width = (viewport_width - (*left as f32) - (*right as f32)).max(0.0);
        let pb = root.padding_left + root.padding_right + root.border_left + root.border_right;
        root.content_width = (root.width - pb).max(0.0);
    }
    if matches!(style.height, LengthValue::Auto)
        && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
    {
        root.height = (viewport_height - (*top as f32) - (*bottom as f32)).max(0.0);
        let pb = root.padding_top + root.padding_bottom + root.border_top + root.border_bottom;
        root.content_height = (root.height - pb).max(0.0);
    }
}

/// 对「containing block = 根元素（positioned root）」的 abspos 元素按根 padding-box
/// 重解析百分比尺寸与 Px/百分比 inset 位置（CSS §10.1.2/§10.3.18/§10.6.4）。
///
/// taffy 0.7 的 root quirk：当根元素（如 `<html style="position:relative">`）是 abspos
/// 后代的最近 positioned 祖先时，taffy 不把根当作 CB，而是误用静态父（如 body），
/// 致 abspos 百分比尺寸按父宽度解析、位置偏移（abspos-containing-block-005/006 实证，
/// 对照 bottom-offset-percentage-001 的**非根** positioned 祖先 `#div1` taffy 正确）。
/// 与 R123（根 relative inset 不应用）同属 taffy root quirk 谱系。本 pass 在 extract 后
/// 按根 padding-box 补解析。
///
/// 仅处理「最近 positioned 祖先 = 根」的 abspos（`nearest_pos_ancestor_is_root`）：
/// 非根 positioned 祖先（如 `#div1`）由 taffy 正确处理，本 pass 通过递归把
/// `nearest_pos_ancestor_is_root` 在遇到任何非根 positioned 元素时置 false，不介入。
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_abspos_against_root_cb(
    box_node: &mut LayoutBox,
    current_box_origin_x: f32,
    current_box_origin_y: f32,
    cb_origin_x: f32,
    cb_origin_y: f32,
    cb_width: f32,
    cb_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    nearest_pos_ancestor_is_root: bool,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if child.is_absolute
            && nearest_pos_ancestor_is_root
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // 百分比尺寸：相对根 padding-box（CB）
            if let LengthValue::Percentage(p) = &style.width {
                child.width = *p as f32 / 100.0 * cb_width;
            }
            if let LengthValue::Percentage(p) = &style.height {
                child.height = *p as f32 / 100.0 * cb_height;
            }
            // auto 尺寸 + 全长度 inset → stretch（§10.3.18/§10.6.4，仅非替换）
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(left), LengthValue::Px(right)) = (&style.left, &style.right)
            {
                child.width = (cb_width - (*left as f32) - (*right as f32)).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (LengthValue::Px(top), LengthValue::Px(bottom)) = (&style.top, &style.bottom)
            {
                child.height = (cb_height - (*top as f32) - (*bottom as f32)).max(0.0);
            }
            // left/top 百分比：目标视口绝对坐标 = cb_origin + p% * cb，转回父相对坐标
            if let LengthValue::Percentage(p) = &style.left {
                let target_x = cb_origin_x + *p as f32 / 100.0 * cb_width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let LengthValue::Percentage(p) = &style.top {
                let target_y = cb_origin_y + *p as f32 / 100.0 * cb_height;
                child.y = target_y - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // left/top Px：目标视口绝对坐标 = cb_origin + px
            if let LengthValue::Px(px) = &style.left {
                child.x =
                    cb_origin_x + (*px as f32) - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let LengthValue::Px(px) = &style.top {
                child.y =
                    cb_origin_y + (*px as f32) - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // right/bottom Px 且 left/top 为 auto：右/下边对齐 CB 右/下缘（§10.3.18 rule 2）
            if matches!(style.left, LengthValue::Auto)
                && let LengthValue::Px(right) = &style.right
            {
                let target_x = cb_origin_x + cb_width - (*right as f32) - child.width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if matches!(style.top, LengthValue::Auto)
                && let LengthValue::Px(bottom) = &style.bottom
            {
                let target_y = cb_origin_y + cb_height - (*bottom as f32) - child.height;
                child.y = target_y - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
        }

        // 递归：遇到非根 positioned 元素时，其后代的最近 positioned 祖先不再是根 → false
        let child_nearest_is_root = if child.is_absolute || child.is_fixed || child.is_relative || child.is_sticky {
            false
        } else {
            nearest_pos_ancestor_is_root
        };
        let child_box_origin_x = current_box_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_box_origin_y = current_box_origin_y + box_node.border_top + box_node.padding_top + child.y;
        resolve_abspos_against_root_cb(
            child,
            child_box_origin_x,
            child_box_origin_y,
            cb_origin_x,
            cb_origin_y,
            cb_width,
            cb_height,
            styles,
            child_nearest_is_root,
        );
    }
}
