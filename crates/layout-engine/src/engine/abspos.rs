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

/// R1227：abspos/fixed 百分比尺寸（`width`/`height: %`）按 CB 重解析为 border-box +
/// content 一对值。
///
/// `LayoutBox.width`/`height` 是 **border-box**（taffy `layout.size` 语义，见
/// engine.rs extract_layout）。CSS `width:%` 对 content-box 指 **content**、对
/// border-box 指 **border-box**。故 content-box 须 `border-box = content + border`，
/// border-box 直接用。`content_*` 同步重算防 taffy 按「错误 CB」（静态父）解析后陈旧。
///
/// 修 abspos-containing-block-initial-009e/009f（body abspos `width:50%` + `border:10px`
/// 旧渲 border-box 400 而非 420——旧代码把 `%` 当 border-box 丢 border 调整）。
fn resolve_abspos_pct(
    pct: f32,
    cb_size: f32,
    border_a: f32,
    border_b: f32,
    padding_a: f32,
    padding_b: f32,
    is_border_box: bool,
) -> (f32, f32) {
    let resolved = pct / 100.0 * cb_size;
    let border_box = if is_border_box {
        resolved
    } else {
        resolved + border_a + border_b
    };
    let content = (border_box - border_a - border_b - padding_a - padding_b).max(0.0);
    (border_box, content)
}

fn resolve_abspos_real_length(
    value: &zero_css_parser::values::LengthValue,
    font_size: &zero_css_parser::values::LengthValue,
    viewport_width: f32,
    viewport_height: f32,
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
                Some(viewport_width as f64),
                Some(viewport_height as f64),
            );
            Some(zero_style_system::computed::resolve_length(
                other,
                font_size_px,
                Some(viewport_width as f64),
                Some(viewport_height as f64),
            ) as f32)
        }
    }
}

fn resolve_abspos_vcenter_inset(
    value: &zero_css_parser::values::LengthValue,
    font_size: &zero_css_parser::values::LengthValue,
    percentage_basis: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<f32> {
    use zero_css_parser::values::LengthValue;
    match value {
        LengthValue::Percentage(p) => Some(*p as f32 / 100.0 * percentage_basis),
        other => resolve_abspos_real_length(other, font_size, viewport_width, viewport_height),
    }
}

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
        // R1874：四 inset 全 auto 的 fixed，位置应为静态位置（§10.3.7/§10.6.4），
        // taffy 已置其于静态坐标（视口正确），扣除祖先偏移反将其误移到 (0,0)，故跳过。
        // R2084 dim-aware：per-dim 判定（旧单一 fixed_insets_all_auto 过粗）。仅当该维有
        // explicit inset（即 !fixed_{x,y}_insets_all_auto）才扣该维偏移；该维全 auto 的
        // fixed 静态位置已是视口正确，扣除会误零化（partial-auto 如 top:auto+left:10px：
        // x 维 left explicit→扣 x✓，y 维 top/bottom 全 auto→不扣 y✓，保静态 y）。
        if !box_node.fixed_x_insets_all_auto {
            box_node.x -= parent_offset_x;
        }
        if !box_node.fixed_y_insets_all_auto {
            box_node.y -= parent_offset_y;
        }
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
        // R1308：fixed 元素 CB 恒为视口（CSS §10.1），其 inset/百分比应恒对视口解析
        //（同 absolute-no-positioned-ancestor 路径）。旧 gate 仅 is_absolute，致
        // `position:fixed + bottom:0` 不解析 bottom（盒落视口顶外 abs_y=-height 而非视口底）。
        // kill-switch ZW_FIXED_INSET=0 回退（仅 absolute）。
        let is_abs_viewport_cb = child.is_absolute && !child_has_positioned_ancestor;
        let is_fixed_cb = child.is_fixed && std::env::var("ZW_FIXED_INSET").as_deref() != Ok("0");
        if (is_abs_viewport_cb || is_fixed_cb)
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
            // 仅当 width 为百分比时按视口重解析。R1227：box-sizing 感知（content-box
            // 须加 border），见 resolve_abspos_pct。
            if let LengthValue::Percentage(p) = &style.width {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (w, cw) = resolve_abspos_pct(
                    *p as f32,
                    viewport_width,
                    child.border_left,
                    child.border_right,
                    child.padding_left,
                    child.padding_right,
                    is_bb,
                );
                child.width = w;
                child.content_width = cw;
            }
            if let LengthValue::Percentage(p) = &style.height {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (h, ch) = resolve_abspos_pct(
                    *p as f32,
                    viewport_height,
                    child.border_top,
                    child.border_bottom,
                    child.padding_top,
                    child.padding_bottom,
                    is_bb,
                );
                child.height = h;
                child.content_height = ch;
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
                && let (Some(left), Some(right)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.right, &style.font_size, viewport_width, viewport_height),
                )
            {
                child.width = (viewport_width - left - right).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (Some(top), Some(bottom)) = (
                    resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.bottom, &style.font_size, viewport_width, viewport_height),
                )
            {
                child.height = (viewport_height - top - bottom).max(0.0);
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
            // left/top 为真实长度时：CSS 2.1 §10.1 规定无 positioned ancestor 的
            // absolute 元素以初始包含块（视口）为 containing block。taffy 用静态父
            // 作 containing block，导致 top:118px 解析为静态父相对坐标。此处把目标
            // 视口坐标（= used length）转回父 content 相对坐标，与百分比路径同机制（不调整
            // auto 宽高，避免历史上 auto 宽高扩张导致的回归）。
            if let Some(px) = resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height)
            {
                child.x = px - parent_content_origin_x;
            }
            if let Some(px) = resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height)
            {
                child.y = px - parent_content_origin_y;
            }
            // right/bottom 为长度且 left/top 为 auto 时：CSS 2.1 §10.1 无 positioned
            // ancestor 的 absolute 元素 CB=视口。left:auto + right:Px → 右边对齐视口
            // 右缘，由已解析的 width 反解 left（§10.3.18 rule 2）：
            // target_x = viewport_w - right - width。须在 width/height 解析后执行
            // （上方百分比/auto-stretch 块已设好 child.width/height）。left/top 已为
            // Px 时由上方块处理；双 inset 全 Px 的 over-constrained（LTR）忽略 right。
            // right/bottom 百分比仅当对应尺寸为 auto 时才影响位置，当前不处理。
            if matches!(style.left, LengthValue::Auto)
                && let Some(right) =
                    resolve_abspos_real_length(&style.right, &style.font_size, viewport_width, viewport_height)
            {
                let target_viewport_x = viewport_width - right - child.width;
                child.x = target_viewport_x - parent_content_origin_x;
            }
            if matches!(style.top, LengthValue::Auto)
                && let Some(bottom) =
                    resolve_abspos_real_length(&style.bottom, &style.font_size, viewport_width, viewport_height)
            {
                let target_viewport_y = viewport_height - bottom - child.height;
                child.y = target_viewport_y - parent_content_origin_y;
            }
            // §10.3.7：width:auto + 全长度 left+right 填满后，max-width 钳制，再把
            // over-constrained 方程的 leftover 重分配到 auto-margin（abspos 无 positioned
            // 祖先时 CB=viewport）。taffy 0.7 不钳 abspos inset-fill 宽。须在 width
            // stretch + left/right 定位之后执行（覆盖上方 x 定位）。target_viewport_x
            // 转回父 content 相对坐标（与上方各块同机制）。
            if matches!(style.width, LengthValue::Auto)
                && let (Some(left), Some(right), Some(mw)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.right, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.max_width, &style.font_size, viewport_width, viewport_height),
                )
                && child.width > mw + 0.5
            {
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
                && let (Some(top), Some(bottom)) = (
                    resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.bottom, &style.font_size, viewport_width, viewport_height),
                )
            {
                child.height = (viewport_height - top - bottom).max(0.0);
            }
            // width: auto + 全长度 left+right → stretch
            if matches!(style.width, LengthValue::Auto)
                && let (Some(left), Some(right)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height),
                    resolve_abspos_real_length(&style.right, &style.font_size, viewport_width, viewport_height),
                )
            {
                child.width = (viewport_width - left - right).max(0.0);
            }
            // 百分比尺寸：fixed 的 CB 恒为视口（CSS §10.1），百分比相对视口解析。
            // taffy 按 positioned 祖先解析（如 body CB），此处按视口重算。R1227：box-sizing
            // 感知（content-box 须加 border），见 resolve_abspos_pct。
            if let LengthValue::Percentage(p) = &style.height {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (h, ch) = resolve_abspos_pct(
                    *p as f32,
                    viewport_height,
                    child.border_top,
                    child.border_bottom,
                    child.padding_top,
                    child.padding_bottom,
                    is_bb,
                );
                child.height = h;
                child.content_height = ch;
            }
            if let LengthValue::Percentage(p) = &style.width {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (w, cw) = resolve_abspos_pct(
                    *p as f32,
                    viewport_width,
                    child.border_left,
                    child.border_right,
                    child.padding_left,
                    child.padding_right,
                    is_bb,
                );
                child.width = w;
                child.content_width = cw;
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
    // 位置：root CB 原点 = 视口 (0,0)，left/top real length → 绝对坐标。
    if let Some(left) = resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height) {
        root.x = left;
    }
    if let Some(top) = resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height) {
        root.y = top;
    }
    // 尺寸 stretch：auto + 全长度对边 inset → viewport - inset（§10.3.18/§10.6.4）。
    if matches!(style.width, LengthValue::Auto)
        && let (Some(left), Some(right)) = (
            resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height),
            resolve_abspos_real_length(&style.right, &style.font_size, viewport_width, viewport_height),
        )
    {
        root.width = (viewport_width - left - right).max(0.0);
        let pb = root.padding_left + root.padding_right + root.border_left + root.border_right;
        root.content_width = (root.width - pb).max(0.0);
    }
    if matches!(style.height, LengthValue::Auto)
        && let (Some(top), Some(bottom)) = (
            resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height),
            resolve_abspos_real_length(&style.bottom, &style.font_size, viewport_width, viewport_height),
        )
    {
        root.height = (viewport_height - top - bottom).max(0.0);
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
            // 百分比尺寸：相对根 padding-box（CB）。R1227：box-sizing 感知（content-box
            // 须加 border），见 resolve_abspos_pct。
            if let LengthValue::Percentage(p) = &style.width {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (w, cw) = resolve_abspos_pct(
                    *p as f32,
                    cb_width,
                    child.border_left,
                    child.border_right,
                    child.padding_left,
                    child.padding_right,
                    is_bb,
                );
                child.width = w;
                child.content_width = cw;
            }
            if let LengthValue::Percentage(p) = &style.height {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (h, ch) = resolve_abspos_pct(
                    *p as f32,
                    cb_height,
                    child.border_top,
                    child.border_bottom,
                    child.padding_top,
                    child.padding_bottom,
                    is_bb,
                );
                child.height = h;
                child.content_height = ch;
            }
            // auto 尺寸 + 全长度 inset → stretch（§10.3.18/§10.6.4，仅非替换）
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (Some(left), Some(right)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, cb_width, cb_height),
                    resolve_abspos_real_length(&style.right, &style.font_size, cb_width, cb_height),
                )
            {
                child.width = (cb_width - left - right).max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (Some(top), Some(bottom)) = (
                    resolve_abspos_real_length(&style.top, &style.font_size, cb_width, cb_height),
                    resolve_abspos_real_length(&style.bottom, &style.font_size, cb_width, cb_height),
                )
            {
                child.height = (cb_height - top - bottom).max(0.0);
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
            // left/top real length：目标视口绝对坐标 = cb_origin + used length
            if let Some(px) = resolve_abspos_real_length(&style.left, &style.font_size, cb_width, cb_height) {
                child.x = cb_origin_x + px - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let Some(px) = resolve_abspos_real_length(&style.top, &style.font_size, cb_width, cb_height) {
                child.y = cb_origin_y + px - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // right/bottom real length 且 left/top 为 auto：右/下边对齐 CB 右/下缘（§10.3.18 rule 2）
            if matches!(style.left, LengthValue::Auto)
                && let Some(right) = resolve_abspos_real_length(&style.right, &style.font_size, cb_width, cb_height)
            {
                let target_x = cb_origin_x + cb_width - right - child.width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if matches!(style.top, LengthValue::Auto)
                && let Some(bottom) = resolve_abspos_real_length(&style.bottom, &style.font_size, cb_width, cb_height)
            {
                let target_y = cb_origin_y + cb_height - bottom - child.height;
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

/// R2062：abspos 元素垂直 margin:auto 居中（CSS §10.6.4 over-constrained 方程）。
///
/// taffy 0.12 不对 positioned-ancestor-CB 的 abspos（top+bottom 均 Px + height 非 auto
/// 且 margin-top/bottom 含 auto）做垂直居中——元素被放在 CB 顶部（top 处）而非居中。
/// 实证 position-absolute-replaced-no-intrinsic-size：img 的 height 为 max-content→100，
/// 配 `top:0 bottom:0 margin:auto`、CB=relative div 200px，ZW 落 y=0，应居中 y=50。
///
/// R2061 的 `recenter_abspos_table_vertically`（table.rs）仅处理 **table + height:auto**
/// 的 taffy-stretch 场景；本 pass 补 **definite-height** 通用情况（replaced/非 replaced 均适用）。
/// 水平 margin:auto 居中（margin-left/right）已由 `adjust_absolute_pct_to_viewport` 处理
/// root-CB；本 pass 仅垂直。
///
/// 递归携带最近 positioned 祖先的 padding-box 高度（CB height）。遇到 positioned 元素时
/// 更新 CB 为其 padding-box。shift 以 delta 形式加到 `child.y`（任意祖先坐标系下「下移
/// delta」等价，因父子 y 轴同向同尺度）。
pub(super) fn recenter_abspos_margin_auto_vertically(
    box_node: &mut LayoutBox,
    cb_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if std::env::var("ZW_ABSPOS_VCENTER").as_deref() == Ok("0") {
        return;
    }
    recenter_abspos_vcenter_inner(box_node, cb_height, viewport_width, viewport_height, styles);
}

fn recenter_abspos_vcenter_inner(
    box_node: &mut LayoutBox,
    cb_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if (child.is_absolute || child.is_fixed)
            && let Some(s) = child.node_id.and_then(|id| styles.get(&id))
        {
            let mt_auto = matches!(s.margin_top, LengthValue::Auto);
            let mb_auto = matches!(s.margin_bottom, LengthValue::Auto);
            // R2068：仅处理「两侧 margin 均 auto」的垂直居中（taffy 不解此场景，absolute-tables-016
            // 等 driving test 依赖）。单边 auto（mt_auto xor mb_auto）的 §10.6.4 margin 分配
            // **taffy 已正确求解**——禁用本 pass（ZW_ABSPOS_VCENTER=0）实证 max-height-004 单边
            // auto 0.06% PASS；启用则双重应用（taffy 已下移 + 本 pass 再叠加 leftover）致元素
            // 贴底（absolute-non-replaced-max-height-002/003/004/007/009/011 簇）。故单边 auto
            // 交回 taffy，本 pass 仅 both-auto。
            // R2072：放开 height_definite gate——height:auto + top+bottom Px + both-auto 也处理。
            // SET（top_px+half）对两种 height:auto 子场景都对：① 无 max-height stretch 填满 CB
            //（child.height=CB-top-bottom，leftover=0，half=0，SET=top_px no-op，元素已填满 ✓）；
            // ② max-height cap（child.height=capped<stretch，leftover>0，SET 下移居中 ✓）。解
            // max-height-002/007/009/011 簇（height:auto+cap+both-auto，旧 gate 跳过）。
            // R2083：扩到 position:fixed——§10.6.4 对 fixed 同样适用（both-auto + top+bottom Px
            // 垂直居中），CB = 初始包含块（视口，§10.1；transform 祖先例外 ZW 暂不处理，同
            // adjust_fixed_to_viewport 假设）。taffy 对 fixed both-auto 给不一致结果（probe 实证
            // abs_y=0 + mt=100 未居中），故本 pass 接管。effective_cb_height：fixed→viewport，
            // absolute→inherited cb_height（positioned 祖先 padding-box 或 root ICB）。
            // R2085：扩到 Percentage inset——CSS2.1 §10.6.4 百分比 top/bottom 相对 CB height 解析后
            // 与 Px 等价参与方程（absolute-non-replaced-height-013：top/bottom:50%+height:100+
            // margin:auto in 100px CB）。旧 both_v_inset 仅 Px 漏 Percentage → 013 recenter no-op。
            // R3596：扩到 residual real length（em/ch/rem/vw/...），但仍显式拒绝 auto 和 intrinsic；
            // Percentage 继续按 effective_cb_height，而非 resolve_length 的 viewport basis。
            let effective_cb_height = if child.is_fixed { viewport_height } else { cb_height };
            if mt_auto
                && mb_auto
                && let (Some(top_px), Some(bottom_px)) = (
                    resolve_abspos_vcenter_inset(
                        &s.top,
                        &s.font_size,
                        effective_cb_height,
                        viewport_width,
                        viewport_height,
                    ),
                    resolve_abspos_vcenter_inset(
                        &s.bottom,
                        &s.font_size,
                        effective_cb_height,
                        viewport_width,
                        viewport_height,
                    ),
                )
            {
                // §10.6.4：leftover = CB_height − top − bottom − element border-box height
                //（child.height 是 border-box，已含 border/padding，对 box-sizing 均正确）。
                // 两侧 margin 均 auto → 各取 leftover/2。R2085：leftover **不钳零**——CSS2.1
                // §10.6.4 "solve the equation under the constraint that the two margins get equal
                // values" 不限符号；over-constrained（显式 height > CB−insets）时 margin 取负值仍
                // 居中。旧 `.max(0.0)` 把负 leftover 钳零致元素贴 top（013：leftover=−100 钳零→
                // y=50 留上半红；应 mt=mb=−50→y=0 填满 CB）。height:auto stretch/cap 场景 leftover
                // 恒 ≥0，去钳零对其无影响（max-height-002/003/007/009/011 簇零回归）。
                let leftover = effective_cb_height - top_px - bottom_px - child.height;
                let half = leftover / 2.0;
                child.margin_top = half;
                child.margin_bottom = half;
                // R2069：SET（非 +=）目标居中位 child.y = top_px + half。旧 `+= half` 假设 taffy
                // 把元素放在静态位（child.y = top_px），仅对 taffy 不居中的场景（height keyword
                // stretch / table）正确；对 height:Px regular div，taffy 已居中（child.y = top_px
                // + half），+= half 双重应用致贴底（max-height-003）。SET 对两种 taffy 起点都对。
                child.y = top_px + half;
            }
        }
        // 递归：若 child 自身 positioned，其后代 CB = child padding-box height（§10.1）。
        let child_cb_height = if child.is_absolute || child.is_fixed || child.is_relative || child.is_sticky {
            (child.height - child.border_top - child.border_bottom).max(0.0)
        } else {
            cb_height
        };
        recenter_abspos_vcenter_inner(child, child_cb_height, viewport_width, viewport_height, styles);
    }
}

#[cfg(test)]
mod r2062_tests {
    use super::*;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    /// 构造 abspos img（definite height 100）作为 positioned 父（CB height 200）的子。
    /// 函数处理 `box_node.children`，故 img 必须包在父盒里再对父盒调用。
    fn make_parent_with_abspos_img(
        margin_top: LengthValue,
        margin_bottom: LengthValue,
        height: LengthValue,
    ) -> (LayoutBox, HashMap<NodeId, ComputedStyle>) {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = height;
        si.margin_top = margin_top;
        si.margin_bottom = margin_bottom;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        (parent_box, styles)
    }

    /// R2062：abspos + top+bottom Px + height definite + 两侧 auto margin → 居中下移 leftover/2。
    #[test]
    fn r2062_abspos_definite_height_both_auto_margins_center() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::Px(100.0));
        // 顶层 cb_height = 父 padding-box = 200（父 positioned，其子的 CB）。
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        // leftover = 200 − 100 = 100；mt=mb 均 auto → 各 50；img 下移 50。
        assert_eq!(img.y, 50.0, "img should shift down by leftover/2 = 50");
        assert_eq!(img.margin_top, 50.0);
        assert_eq!(img.margin_bottom, 50.0);
        assert_eq!(img.height, 100.0, "img height unchanged");
    }

    /// R2068：仅 margin-top auto（mb 非 auto）→ 本 pass **不再处理**（交回 taffy）。
    /// taffy 已正确解 §10.6.4 单边 auto margin 分配；旧实现双重应用致 max-height-004 簇
    /// 贴底（ZW_ABSPOS_VCENTER=0 实证 taffy 单独 0.06% PASS）。守 recenter 对单边 auto no-op。
    #[test]
    fn r2068_abspos_single_auto_margin_left_to_taffy_top() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Px(0.0), LengthValue::Px(100.0));
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        assert_eq!(
            img.y, 0.0,
            "single-auto (mt auto, mb=0) is left to taffy — recenter no-op"
        );
        assert_eq!(img.margin_top, 0.0);
        assert_eq!(img.margin_bottom, 0.0);
    }

    /// R2068：仅 margin-bottom auto → 同上，recenter no-op（交回 taffy）。
    #[test]
    fn r2068_abspos_single_auto_margin_left_to_taffy_bottom() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Px(0.0), LengthValue::Auto, LengthValue::Px(100.0));
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        assert_eq!(
            img.y, 0.0,
            "single-auto (mt=0, mb auto) is left to taffy — recenter no-op"
        );
        assert_eq!(img.margin_top, 0.0);
        assert_eq!(img.margin_bottom, 0.0);
    }

    /// R2072：height:auto 现也处理（放开 height_definite gate）。synthetic config
    ///（top=0, bottom=0, height=100, cb=200）→ leftover=200-0-0-100=100, half=50,
    /// SET child.y = top_px(0) + half(50) = 50。真实 stretch 场景 taffy 会让 child.height
    /// = CB-top-bottom（填满）→ leftover=0 → SET=top_px no-op（无中心化需要）。
    #[test]
    fn r2072_abspos_height_auto_now_processed() {
        let (mut parent, styles) = make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::Auto);
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        // top=0, bottom=0, child.height=100, cb=200 → leftover=100, half=50, SET y=0+50=50.
        assert_eq!(img.y, 50.0, "height:auto both-auto now centered via SET (R2072)");
        assert_eq!(img.margin_top, 50.0);
        assert_eq!(img.margin_bottom, 50.0);
    }

    /// R2062：递归——positioned 祖先的 padding-box（border-box − border）成为后代 CB。
    #[test]
    fn r2062_recursive_cb_uses_positioned_ancestor_padding_box() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let container = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, container);
        let _ = doc.append_child(container, img);
        let mut styles = HashMap::new();
        // container: relative，border-box height 220，border_top/bottom 各 10 → padding-box 200。
        let mut cs = ComputedStyle::default();
        cs.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(container, cs);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let container_box = LayoutBox {
            node_id: Some(container),
            is_relative: true,
            height: 220.0,
            border_top: 10.0,
            border_bottom: 10.0,
            children: vec![img_box],
            ..Default::default()
        };
        // root（非 positioned）→ container。顶层 cb_height=999（模拟 viewport），
        // 递归进 container（positioned）后其子（img）CB 应为 container padding-box 200。
        let mut root_box = LayoutBox {
            children: vec![container_box],
            ..Default::default()
        };
        recenter_abspos_margin_auto_vertically(&mut root_box, 999.0, 800.0, 600.0, &styles);
        let img = &root_box.children[0].children[0];
        assert_eq!(
            img.y, 50.0,
            "img centered against padding-box CB 200 (not border-box 220)"
        );
        assert_eq!(img.margin_top, 50.0);
    }

    /// R2083：position:fixed + top+bottom Px + 两侧 auto margin → 垂直居中，CB = 视口
    ///（§10.6.4 + §10.1，非父 cb_height）。R2082 probe 实证旧实现（recenter 仅 is_absolute）
    /// 对 fixed both-auto 给不一致结果（abs_y=0 + mt=100 未居中）。本测试守 fixed 走 viewport CB。
    #[test]
    fn r2083_fixed_both_auto_margins_center_against_viewport() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Fixed;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_fixed: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        // parent height 300；顶层 cb_height=300（模拟 positioned 父），viewport=600（fixed CB）。
        // 若误用 cb_height(300)：leftover=300-0-0-100=200, half=100, y=100。
        // 正确（viewport 600）：leftover=600-0-0-100=500, half=250, y=250。
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            height: 300.0,
            children: vec![img_box],
            ..Default::default()
        };
        recenter_abspos_margin_auto_vertically(&mut parent_box, 300.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(
            img.y, 250.0,
            "fixed both-auto centers against viewport (600), not parent cb_height (300)"
        );
        assert_eq!(img.margin_top, 250.0);
        assert_eq!(img.margin_bottom, 250.0);
    }

    /// R3593：position:fixed + auto size + real-length opposing insets stretch against viewport.
    /// The fixed stretch pass previously only accepted Px, so residual `em` insets left the
    /// Taffy-sized fallback box unchanged.
    #[test]
    fn r3593_fixed_auto_size_stretches_with_relative_insets() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let div = doc.create_element("div");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, div);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.position = zero_style_system::property::types::PositionValue::Fixed;
        style.font_size = LengthValue::Px(20.0);
        style.width = LengthValue::Auto;
        style.height = LengthValue::Auto;
        style.left = LengthValue::Em(1.0);
        style.right = LengthValue::Em(2.0);
        style.top = LengthValue::Em(0.5);
        style.bottom = LengthValue::Em(1.0);
        styles.insert(div, style);

        let div_box = LayoutBox {
            node_id: Some(div),
            is_fixed: true,
            width: 10.0,
            height: 10.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            children: vec![div_box],
            ..Default::default()
        };

        stretch_fixed_to_viewport_size(&mut parent_box, 800.0, 600.0, &styles);

        let div = &parent_box.children[0];
        assert_eq!(div.width, 740.0, "800 - 20px - 40px");
        assert_eq!(div.height, 570.0, "600 - 10px - 20px");
    }

    /// R2085：Percentage top/bottom inset 被接受并参与居中（相对 effective_cb_height 解析）。
    /// absolute-non-replaced-height-013 谱系：top/bottom:25% + height:50 + margin:auto in CB 200
    /// → top=bottom=50, leftover=200-50-50-50=50, half=25, y=75。旧实现 both_v_inset 仅 Px → no-op。
    #[test]
    fn r2085_abspos_percentage_inset_centers() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Percentage(25.0);
        si.bottom = LengthValue::Percentage(25.0);
        si.height = LengthValue::Px(50.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 50.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        // cb_height=200：top=bottom=50（25% of 200），leftover=200-50-50-50=50，half=25，y=75。
        recenter_abspos_margin_auto_vertically(&mut parent_box, 200.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(
            img.y, 75.0,
            "percentage inset 25% of 200 = 50; centers at top(50)+half(25)=75"
        );
        assert_eq!(img.margin_top, 25.0);
        assert_eq!(img.margin_bottom, 25.0);
    }

    /// R3596：real-length top/bottom inset should participate in abspos vertical
    /// both-auto margin centering after resolving against the element font context.
    #[test]
    fn r3596_abspos_relative_length_inset_centers() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.font_size = LengthValue::Px(20.0);
        si.top = LengthValue::Em(1.0);
        si.bottom = LengthValue::Em(2.0);
        si.height = LengthValue::Px(50.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 50.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        // top=20, bottom=40, leftover=200-20-40-50=90, half=45, y=65.
        recenter_abspos_margin_auto_vertically(&mut parent_box, 200.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(img.y, 65.0);
        assert_eq!(img.margin_top, 45.0);
        assert_eq!(img.margin_bottom, 45.0);
    }

    /// R3596：fixed uses the viewport as its vertical centering CB while resolving
    /// residual real-length insets with the element font context.
    #[test]
    fn r3596_fixed_relative_length_inset_centers_against_viewport() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut si = ComputedStyle::default();
        si.font_size = LengthValue::Px(20.0);
        si.top = LengthValue::Em(0.5);
        si.bottom = LengthValue::Em(1.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Fixed;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_fixed: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            height: 300.0,
            children: vec![img_box],
            ..Default::default()
        };
        // viewport=600: top=10, bottom=20, leftover=470, half=235, y=245.
        recenter_abspos_margin_auto_vertically(&mut parent_box, 300.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(img.y, 245.0);
        assert_eq!(img.margin_top, 235.0);
        assert_eq!(img.margin_bottom, 235.0);
    }

    /// R2085：over-constrained（显式 height > CB−insets）both-auto 居中——CSS2.1 §10.6.4
    /// "solve with equal margins" 不钳零；leftover 为负时 margin 取负值仍居中。
    /// absolute-non-replaced-height-013 精确复现：CB 100, top/bottom:50%→50/50, height:100
    /// → leftover=100-50-50-100=−100, half=−50, y=0（填满 CB 顶部）。
    #[test]
    fn r2085_abspos_over_constrained_centers_with_negative_margins() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let div = doc.create_element("div");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, div);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Percentage(50.0);
        si.bottom = LengthValue::Percentage(50.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(div, si);
        let div_box = LayoutBox {
            node_id: Some(div),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 100.0,
            children: vec![div_box],
            ..Default::default()
        };
        // cb_height=100：top=bottom=50（50%），height=100，leftover=100-50-50-100=−100，
        // half=−50，y=50+(−50)=0（green 填满 100×100 red CB 顶部，no red 可见）。旧 .max(0.0)
        // 钳零 → half=0 → y=50（仅覆盖下半，上半红可见 → 013 FAIL）。
        recenter_abspos_margin_auto_vertically(&mut parent_box, 100.0, 800.0, 600.0, &styles);
        let child = &parent_box.children[0];
        assert_eq!(
            child.y, 0.0,
            "over-constrained both-auto centers via negative margins: y=0"
        );
        assert_eq!(child.margin_top, -50.0);
        assert_eq!(child.margin_bottom, -50.0);
        assert_eq!(child.height, 100.0, "height unchanged");
    }
}
