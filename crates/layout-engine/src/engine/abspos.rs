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

pub(super) fn resolve_abspos_real_length(
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

/// R4122（csswg #10544 + css-position-3）：本元素是否为 fixed 后代的 containment
/// 包含块——layout/paint containment（含 strict/content）使元素成为 absolute/fixed
/// 后代的包含块（R3902 同款谓词的 containment 臂）。**不含** container-type 隐含的
/// containment：csswg #10544 决议（driving: css-conditional/container-queries
/// no-layout-containment-fixedpos，测试标题「fixed elements should propagate from a
/// container-type subtree」）——container-type 查询容器**不**捕获 position:fixed 后代，
/// fixed 继续向上传播到下一个真实 containment 祖先。table 内部盒 / inline 排除臂与
/// R3902 is_abspos_cb 同表。
pub(super) fn is_fixed_cb_containment(s: &ComputedStyle) -> bool {
    (s.contain.has_layout() || s.contain.has_paint())
        && !matches!(
            s.display,
            zero_css_parser::values::DisplayValue::TableRow
                | zero_css_parser::values::DisplayValue::TableColumn
                | zero_css_parser::values::DisplayValue::TableColumnGroup
                | zero_css_parser::values::DisplayValue::TableRowGroup
                | zero_css_parser::values::DisplayValue::TableHeaderGroup
                | zero_css_parser::values::DisplayValue::TableFooterGroup
                | zero_css_parser::values::DisplayValue::Inline
                | zero_css_parser::values::DisplayValue::Contents
                | zero_css_parser::values::DisplayValue::None
        )
}

/// R4295（filter-effects-1 §3 / filter-effects-2 #BackdropFilterProperty / CSS Transforms
/// §3 / css-will-change §3）：非 none 的 `filter`/`backdrop-filter`/`transform`/
/// `perspective`，及 will-change 提示这些属性，使元素成为 absolute/fixed 后代的包含块
///（同 positioned 祖先语义）。driving: backdrop-filter-containing-block（backdrop-filter
/// 容器捕获 fixed/absolute 子）、filter-cb-abspos-inline-001/002/003（filter/perspective
/// 的 inline span）、同测试 ref 页 will-change:transform。
pub(super) fn creates_cb_for_abspos_descendants(s: &ComputedStyle) -> bool {
    !s.filter.is_empty()
        || !s.backdrop_filter.is_empty()
        || !matches!(s.transform, zero_css_parser::values::TransformValue::None)
        || !matches!(s.perspective, zero_css_parser::values::LengthValue::Px(0.0))
        || s.will_change.iter().any(|w| {
            matches!(
                w,
                zero_style_system::WillChangeValue::Custom(c)
                    if matches!(c.as_str(), "transform" | "perspective" | "filter" | "backdrop-filter")
            )
        })
}

/// fixed 后代 CB 判定（树遍历点用）：containment（R4122/csswg #10544）∪ 视觉 CB
///（R4295）。根元素检查点（engine.rs step 4/11.6 的 root_under_containment）保持
/// containment-only——filter-effects-1 §3 根元素例外（root 的 CB 本就是 ICB）。
pub(super) fn is_fixed_cb_ancestor(s: &ComputedStyle) -> bool {
    is_fixed_cb_containment(s) || creates_cb_for_abspos_descendants(s)
}

/// 递归调整 fixed 定位元素的坐标为视口相对。
///
/// taffy 将 `position: fixed` 当作 `absolute` 处理，坐标是相对于包含块的。
/// 此函数在布局完成后遍历布局树，将 fixed 元素的坐标加上祖先累积偏移，
/// 使其变为相对于视口的绝对坐标。
///
/// R4122：`under_containment_cb` 沿树向下传递「祖先链上存在 containment 包含块」——
/// 其 CB 为该 containment 祖先而非视口（csswg #10544），本 pass 的「扣除祖先偏移 →
/// 视口相对」改写不得触达（box.x/y 保留树相对坐标，由 paint 链正常累加得 CB 内位置），
/// 且偏移累积不再归零（fixed 在页面空间内的子树照常从其位置累加）。
pub(super) fn adjust_fixed_to_viewport(
    box_node: &mut LayoutBox,
    parent_offset_x: f32,
    parent_offset_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    under_containment_cb: bool,
) {
    let gated = under_containment_cb && box_node.is_fixed;
    if box_node.is_fixed && !gated {
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

    let offset_x = if box_node.is_fixed && !gated {
        0.0
    } else {
        parent_offset_x + box_node.x
    };
    let offset_y = if box_node.is_fixed && !gated {
        0.0
    } else {
        parent_offset_y + box_node.y
    };

    for child in &mut box_node.children {
        let child_under = under_containment_cb
            || child
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(is_fixed_cb_ancestor);
        adjust_fixed_to_viewport(child, offset_x, offset_y, styles, child_under);
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
    let child_has_positioned_ancestor = has_positioned_ancestor || box_node.is_abspos_cb;

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
#[allow(clippy::too_many_arguments)]
pub(super) fn adjust_absolute_pct_to_viewport(
    box_node: &mut LayoutBox,
    current_content_origin_x: f32,
    current_content_origin_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    has_positioned_ancestor: bool,
    under_containment_cb: bool,
) {
    use zero_css_parser::values::LengthValue;
    let child_has_positioned_ancestor = has_positioned_ancestor || box_node.is_abspos_cb;
    // R4122：本节点自身为 containment 包含块 → 其下 fixed 后代的 CB 是它而非视口。
    // R4295：视觉 CB（filter/backdrop-filter/transform/perspective/will-change）同表。
    let child_under_containment = under_containment_cb
        || box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(is_fixed_cb_ancestor);

    for child in &mut box_node.children {
        // R1308：fixed 元素 CB 恒为视口（CSS §10.1），其 inset/百分比应恒对视口解析
        //（同 absolute-no-positioned-ancestor 路径）。旧 gate 仅 is_absolute，致
        // `position:fixed + bottom:0` 不解析 bottom（盒落视口顶外 abs_y=-height 而非视口底）。
        // kill-switch ZW_FIXED_INSET=0 回退（仅 absolute）。
        // R4122：fixed 的祖先链上有 containment 包含块时（csswg #10544），CB 是该祖先
        // 而非视口 → 本视口重解析臂不触（几何由 stretch pass 的 containment 臂按 CB 解析）。
        let is_abs_viewport_cb = child.is_absolute && !child_has_positioned_ancestor;
        let is_fixed_cb =
            child.is_fixed && std::env::var("ZW_FIXED_INSET").as_deref() != Ok("0") && !child_under_containment;
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
            // R4047：margin 计入定位方程（CSS2 §10.3.7：left + margin-left 定盒缘）——
            // taffy 0.7 对视口-CB（顶层）absolute item 的 location 不含 margin（nested-CB
            // 含），containing-block-008 div1 margin:50px + top:0 应落 y=50 实测 y=0。
            // margin auto/百分比不计（auto margin 另有 R2062 居中；% margin 相对 CB 宽）。
            if let Some(px) = resolve_abspos_real_length(&style.left, &style.font_size, viewport_width, viewport_height)
            {
                let ml =
                    resolve_abspos_real_length(&style.margin_left, &style.font_size, viewport_width, viewport_height)
                        .unwrap_or(0.0);
                child.x = px + ml - parent_content_origin_x;
            }
            if let Some(px) = resolve_abspos_real_length(&style.top, &style.font_size, viewport_width, viewport_height)
            {
                let mt =
                    resolve_abspos_real_length(&style.margin_top, &style.font_size, viewport_width, viewport_height)
                        .unwrap_or(0.0);
                child.y = px + mt - parent_content_origin_y;
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
                // R4047：margin-right 计入方程（盒右缘 = 视口右 - right - margin-right）
                let mr =
                    resolve_abspos_real_length(&style.margin_right, &style.font_size, viewport_width, viewport_height)
                        .unwrap_or(0.0);
                let target_viewport_x = viewport_width - right - mr - child.width;
                child.x = target_viewport_x - parent_content_origin_x;
            }
            if matches!(style.top, LengthValue::Auto)
                && let Some(bottom) =
                    resolve_abspos_real_length(&style.bottom, &style.font_size, viewport_width, viewport_height)
            {
                // R4047：margin-bottom 计入方程（盒下缘 = 视口下 - bottom - margin-bottom）
                let mb =
                    resolve_abspos_real_length(&style.margin_bottom, &style.font_size, viewport_width, viewport_height)
                        .unwrap_or(0.0);
                let target_viewport_y = viewport_height - bottom - mb - child.height;
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
            child_under_containment
                || child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .is_some_and(is_fixed_cb_ancestor),
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
///
/// R4122：`under_containment_cb` 沿树向下传递「祖先链上存在 containment 包含块」
///（csswg #10544：layout/paint containment 祖先捕获 fixed 后代为其 CB；container-type
/// 不捕获）。gated fixed 的 auto + 全长度 inset stretch / 百分比尺寸改按该 containment
/// 祖先的 **padding-box**（进入子树时捕获为 `cb_w`/`cb_h`）解析，非视口。
pub(super) fn stretch_fixed_to_viewport_size(
    box_node: &mut LayoutBox,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    under_containment_cb: bool,
    cb: Option<(f32, f32)>,
) {
    use zero_css_parser::values::LengthValue;
    // R4122：本节点自身为 containment 包含块 → 其 padding-box 成为子树内 fixed 的最近 CB
    //（嵌套 containment 取最近：子帧捕获覆盖传入值）。
    let self_cb: Option<(f32, f32)> = if under_containment_cb {
        cb
    } else {
        box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .filter(|s| is_fixed_cb_ancestor(s))
            .map(|_| {
                (
                    (box_node.width - box_node.border_left - box_node.border_right).max(0.0),
                    (box_node.height - box_node.border_top - box_node.border_bottom).max(0.0),
                )
            })
    };
    let child_under = under_containment_cb || self_cb.is_some();
    for child in &mut box_node.children {
        let gated = child_under && child.is_fixed;
        if child.is_fixed
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // R4122：containment-CB 下按该 CB 尺寸解析；视口语义仅保留给真视口-CB fixed。
            // child 的 CB = 父链最近 containment 祖先的 padding-box（self_cb 所在帧捕获）。
            let (cb_w, cb_h) = if gated {
                self_cb.unwrap_or((viewport_width, viewport_height))
            } else {
                (viewport_width, viewport_height)
            };
            // height: auto + 全长度 top+bottom → stretch
            if matches!(style.height, LengthValue::Auto)
                && let (Some(top), Some(bottom)) = (
                    resolve_abspos_real_length(&style.top, &style.font_size, cb_w, cb_h),
                    resolve_abspos_real_length(&style.bottom, &style.font_size, cb_w, cb_h),
                )
            {
                child.height = (cb_h - top - bottom).max(0.0);
            }
            // width: auto + 全长度 left+right → stretch
            if matches!(style.width, LengthValue::Auto)
                && let (Some(left), Some(right)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, cb_w, cb_h),
                    resolve_abspos_real_length(&style.right, &style.font_size, cb_w, cb_h),
                )
            {
                child.width = (cb_w - left - right).max(0.0);
            }
            // 百分比尺寸：fixed 的 CB 恒为视口（CSS §10.1），百分比相对视口解析。
            // taffy 按 positioned 祖先解析（如 body CB），此处按视口重算。R1227：box-sizing
            // 感知（content-box 须加 border），见 resolve_abspos_pct。
            // R4122：containment-CB 下按 CB 尺寸解析。
            if let LengthValue::Percentage(p) = &style.height {
                let is_bb = matches!(style.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox);
                let (h, ch) = resolve_abspos_pct(
                    *p as f32,
                    cb_h,
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
                    cb_w,
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
        // R4122：向下传递最近 containment CB 尺寸（本帧捕获则用本帧值；否则透传传入值）。
        stretch_fixed_to_viewport_size(child, viewport_width, viewport_height, styles, child_under, self_cb);
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
            // left/top real length：目标视口绝对坐标 = cb_origin + used length + 对应 margin
            //（CSS2 §10.3.7/§10.6.4 定位方程：left + margin-left（top + margin-top）定盒缘；
            // taffy 0.7 对根级 absolute item 的 location 不含 margin（nested-CB 含），导致
            // root-CB abspos 的 margin 丢失——containing-block-008 div1 margin:50px + top:0
            // 应落 y=50 实测 y=0）。margin auto/百分比不计（auto margin 另有 R2062 居中）。
            let margin_left_px =
                resolve_abspos_real_length(&style.margin_left, &style.font_size, cb_width, cb_height).unwrap_or(0.0);
            let margin_top_px =
                resolve_abspos_real_length(&style.margin_top, &style.font_size, cb_width, cb_height).unwrap_or(0.0);
            if let Some(px) = resolve_abspos_real_length(&style.left, &style.font_size, cb_width, cb_height) {
                child.x = cb_origin_x + px + margin_left_px
                    - current_box_origin_x
                    - box_node.border_left
                    - box_node.padding_left;
            }
            if let Some(px) = resolve_abspos_real_length(&style.top, &style.font_size, cb_width, cb_height) {
                child.y = cb_origin_y + px + margin_top_px
                    - current_box_origin_y
                    - box_node.border_top
                    - box_node.padding_top;
            }
            // right/bottom real length 且 left/top 为 auto：右/下边对齐 CB 右/下缘（§10.3.18
            // rule 2），margin-right/margin-bottom 同须计入方程（盒右缘 = CB 右 - right -
            // margin-right）
            if matches!(style.left, LengthValue::Auto)
                && let Some(right) = resolve_abspos_real_length(&style.right, &style.font_size, cb_width, cb_height)
            {
                let margin_right_px =
                    resolve_abspos_real_length(&style.margin_right, &style.font_size, cb_width, cb_height)
                        .unwrap_or(0.0);
                let target_x = cb_origin_x + cb_width - right - margin_right_px - child.width;
                child.x = target_x - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if matches!(style.top, LengthValue::Auto)
                && let Some(bottom) = resolve_abspos_real_length(&style.bottom, &style.font_size, cb_width, cb_height)
            {
                let margin_bottom_px =
                    resolve_abspos_real_length(&style.margin_bottom, &style.font_size, cb_width, cb_height)
                        .unwrap_or(0.0);
                let target_y = cb_origin_y + cb_height - bottom - margin_bottom_px - child.height;
                child.y = target_y - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
        }

        // 递归：遇到非根 positioned 元素时，其后代的最近 positioned 祖先不再是根 → false
        let child_nearest_is_root = if child.is_abspos_cb {
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

/// R3858：abspos 最近 positioned 祖先为**非根**元素时的 inset 重解析（CSS §10.1.2）。
///
/// taffy 0.7 把 absolute 子的 inset 相对其**静态父**解析；当 static 中间层隔在 abspos
/// 与最近 positioned 祖先之间时（如 `div.relative > div > p > span{position:absolute;
/// bottom:0}`），CB 应为 positioned 祖先的 padding-box，taffy 却按静态父（p）解析——
/// `bottom:0` 落到 p 底而非 positioned 祖先底（driving：inline-replaced-width-015 的
/// 绿色覆盖 span 落 y=166.6 而非 216.6，红 img 露出）。
///
/// 机制与 `resolve_abspos_against_root_cb` 同谱系：递归携带最近 positioned 祖先的
/// padding-box（origin + size）；对「直接父非 positioned」的 absolute 子重解析
/// left/top/right/bottom + 百分比尺寸，再转回父 content 相对坐标。直接父即 positioned
/// 祖先的（taffy 已正确，inline-replaced-width-014 `top:0` 直连场景）不触，防回归。
/// 根 positioned 场景由 11.7 root-CB 专项 pass 处理——本 pass 仅在根非 positioned 时
/// 启用（engine.rs 调用点 gate），避免双应用。视口 CB（无 positioned 祖先）由 11.5
/// `adjust_absolute_pct_to_viewport` 处理，cb=None 不触。
///
/// kill-switch `ZW_ABSPOS_NESTED_CB=0` 回退（入口单次读取，避免逐节点 env 查询——
/// bench-gate block_layout_1000_elements 曾因递归内每层 env::var 读取 ~40% 回归）。
pub(super) fn resolve_abspos_against_nested_cb(
    box_node: &mut LayoutBox,
    current_box_origin_x: f32,
    current_box_origin_y: f32,
    cb: Option<(f32, f32, f32, f32)>,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let enabled = std::env::var("ZW_ABSPOS_NESTED_CB").as_deref() != Ok("0");
    resolve_abspos_against_nested_cb_inner(
        box_node,
        current_box_origin_x,
        current_box_origin_y,
        cb,
        styles,
        enabled,
    );
}

/// R4017（CSS2 §10.3.7 static position）：block-level abspos 元素 top/bottom 均 auto
/// 时的垂直静态位置重算。
///
/// taffy 对「前驱 in-flow 兄弟高度在布局期随后续 pass 增长」的 abspos 静态位用**过期值**
/// （absolute-replaced-width-037 族探针实证：两行 `<p>` h=37.2 的兄弟，abspos 静态位
/// 却按单行 18.6 算——static_position 在 sibling 尺寸增长前定格）。spec：静态位置 =
/// 假设 position:static 时盒的位置，即前 in-flow 兄弟的 **margin-edge bottom**
///（含 margin 折叠 max(prev_mb, my_mt)）。
///
/// 修：每容器内对（top/bottom CSS 均 auto 的 absolute 直接子），取其**前一个 in-flow
/// block-level 兄弟**的 `y + height + max(mb, mt)` 重算 y（兄弟坐标即本容器 content 坐标，
/// abspos 子 taffy 输出同基）。等价时与 taffy 值一致（幂等），仅修正过期场景。
/// gate 保守：无前 in-flow block 兄弟（首子/行内语境）不碰（行盒内静态位是另一域）；
/// fixed 不在此域（CB=视口，adjust_fixed_to_viewport 处理）。
pub(super) fn fix_abspos_static_position_y(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    use zero_css_parser::values::LengthValue;

    // 先收集（static-position, 目标 y），再统一写回——避免借用冲突。
    let mut fixes: Vec<(usize, f32)> = Vec::new();
    for (idx, child) in box_node.children.iter().enumerate() {
        if !child.is_absolute {
            continue;
        }
        let Some(style) = child.node_id.and_then(|id| styles.get(&id)) else {
            continue;
        };
        if !matches!(style.top, LengthValue::Auto) || !matches!(style.bottom, LengthValue::Auto) {
            continue;
        }
        // R4017 gate：abspos 自带 margin-top 非零时不介入——taffy absolute 布局对 static
        // 另加 margin.top，其非零 mt 折叠语义有自己的处理链（multicol-spanner-007 翻红
        // 实证：mt:60 的 abspos 被本公式重算后 diff 变差）；本轮修复面（037 族）mt 均 0。
        if resolve_abspos_real_length(&style.margin_top, &style.font_size, 0.0, 0.0).unwrap_or(0.0) > 0.5 {
            continue;
        }
        // 前一个 in-flow block-level 兄弟。
        let Some(prev) = box_node.children[..idx]
            .iter()
            .rev()
            .find(|c| c.is_block_level && !c.is_absolute && !c.is_fixed)
        else {
            // R5017（css-tables §17.5.3 × css-position-3 §static-position）：无前 in-flow
            // 兄弟且容器为 table-cell 时，静态位 = valign 后的内容流原点——cell 的 in-flow
            // 内容经 vertical-align 居中/沉底后，假设 static 的盒落在流原点（padding box
            // 顶 + valign 位移）。chromium 空内容 cell（middle）静态位 = content 顶 +50，
            // abspos translate 后绿块恰盖红（position-absolute-dynamic-static-position-
            // table-cell 实证：taffy 静态位滞留 cell 顶；table.rs 期的 valign 位移会被
            // taffy absolute 覆写，故在本 post-pass 补）。
            if let Some(cell_node_id) = box_node.node_id
                && let Some(cell_style) = styles.get(&cell_node_id)
                && cell_style.display == zero_css_parser::values::DisplayValue::TableCell
            {
                // 与 table.rs valign 同口径：仅 in-flow 子计入内容高。
                let in_flow_h: f32 = box_node
                    .children
                    .iter()
                    .filter(|c| !c.is_absolute && !c.is_fixed)
                    .map(|c| c.height + c.margin_top + c.margin_bottom)
                    .sum();
                let available = box_node.content_height - in_flow_h;
                if available > 0.0 {
                    let dy = match cell_style.vertical_align {
                        zero_css_parser::values::VerticalAlignValue::Middle => available / 2.0,
                        zero_css_parser::values::VerticalAlignValue::Bottom
                        | zero_css_parser::values::VerticalAlignValue::TextBottom => available,
                        _ => 0.0,
                    };
                    if dy > 0.0 {
                        let pad_top = box_node.padding_top;
                        fixes.push((idx, pad_top + dy));
                    }
                }
            }
            continue;
        };
        // margin 折叠：max(prev mb, my mt)（Px/长度解析，% margin 对 abspos 静态位记 0）。
        let my_mt = resolve_abspos_real_length(&style.margin_top, &style.font_size, 0.0, 0.0).unwrap_or(0.0);
        let prev_mb = prev
            .node_id
            .and_then(|id| styles.get(&id))
            .and_then(|s| resolve_abspos_real_length(&s.margin_bottom, &s.font_size, 0.0, 0.0))
            .unwrap_or(0.0);
        let collapsed = prev_mb.max(my_mt);
        fixes.push((idx, prev.y + prev.height + collapsed));
    }
    for (idx, y) in fixes {
        box_node.children[idx].y = y;
    }
    for child in &mut box_node.children {
        // R4017 gate（A/B 实证）：容器子树含 float 时不介入——float 语境的 abspos 静态位
        // 由 float 避让参与（position-absolute-dynamic-static-position-floats-001/004、
        // multicol-spanner-007 翻红实证），简单「前 block 兄弟 margin-box 底」公式不适用。
        if !subtree_has_float(child) {
            fix_abspos_static_position_y(child, styles);
        }
    }
}

/// R4155b：直挂 abspos（parent 即 CB）的 `width/height: stretch` 收尾——taffy 直接
/// 管理 CB=parent 的 abspos，converter Stretch→auto 后走进 replaced attr 回退
///（positioned-replaced-2：canvas `width:stretch; height:stretch; top:50; left:50`
/// 在 150×150 CB 内应 100×100，旧 0×0）。语义（css-sizing-4 §6.2）：stretch =
/// 可用空间填充——inset 一侧 definite 时 = cb − definite inset；双侧 auto =
/// 静态位置到对侧缘（由 nested-CB pass 的 margin 链臂处理，此处补直接子）。
pub(super) fn stretch_abspos_direct_cb(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_ABSPOS_STRETCH").as_deref() == Ok("0") {
        return;
    }
    fn walk(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
        use zero_css_parser::values::LengthValue;
        let cb_w = (box_node.width - box_node.border_left - box_node.border_right).max(0.0);
        let cb_h = (box_node.height - box_node.border_top - box_node.border_bottom).max(0.0);
        let mut changed = false;
        for child in &mut box_node.children {
            if !(child.is_absolute || child.is_fixed) {
                continue;
            }
            let Some(style) = child.node_id.and_then(|nid| styles.get(&nid)) else {
                continue;
            };
            if matches!(style.width, LengthValue::Stretch)
                && (resolve_abspos_real_length(&style.left, &style.font_size, cb_w, cb_h).is_some()
                    || resolve_abspos_real_length(&style.right, &style.font_size, cb_w, cb_h).is_some())
            {
                // inset 至少一侧 definite——stretch = cb − left − right（auto inset = 0；
                // inset:50px 双侧 definite 时 200−50−50=100）。双侧全 auto 的直挂 abspos
                // taffy 静态位求解已正确，不介入（防塌 attr 回退）。
                let left_px = resolve_abspos_real_length(&style.left, &style.font_size, cb_w, cb_h).unwrap_or(0.0);
                let right_px = resolve_abspos_real_length(&style.right, &style.font_size, cb_w, cb_h).unwrap_or(0.0);
                let new_w = (cb_w - left_px - right_px).max(0.0);
                if (child.width - new_w).abs() > 0.5 {
                    if std::env::var("R4155_TRACE").is_ok() {
                        eprintln!("[r4155b] W child={:?} {}->{}", child.node_id, child.width, new_w);
                    }
                    child.width = new_w;
                    child.content_width =
                        (new_w - child.border_left - child.border_right - child.padding_left - child.padding_right)
                            .max(0.0);
                    changed = true;
                }
            }
            if matches!(style.height, LengthValue::Stretch)
                && (resolve_abspos_real_length(&style.top, &style.font_size, cb_w, cb_h).is_some()
                    || resolve_abspos_real_length(&style.bottom, &style.font_size, cb_w, cb_h).is_some())
            {
                let top_px = resolve_abspos_real_length(&style.top, &style.font_size, cb_w, cb_h).unwrap_or(0.0);
                let bottom_px = resolve_abspos_real_length(&style.bottom, &style.font_size, cb_w, cb_h).unwrap_or(0.0);
                let new_h = (cb_h - top_px - bottom_px).max(0.0);
                if (child.height - new_h).abs() > 0.5 {
                    if std::env::var("R4155_TRACE").is_ok() {
                        eprintln!("[r4155b] H child={:?} {}->{}", child.node_id, child.height, new_h);
                    }
                    // bottom definite（top auto）时盒底对齐 cb_bottom−bottom——h 变化后
                    // y 须回移 delta（taffy 按旧 h 定位；abspos-2 h 0→100 应上移 100）。
                    if matches!(style.top, LengthValue::Auto)
                        && resolve_abspos_real_length(&style.bottom, &style.font_size, cb_w, cb_h).is_some()
                    {
                        child.y -= new_h - child.height;
                    }
                    child.height = new_h;
                    child.content_height =
                        (new_h - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                    changed = true;
                }
            }
        }
        changed
    }
    // 自顶向下：parent 先定型（taffy 输出已终态，此处只读尺寸），再处理其 abspos 子。
    fn rec(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
        walk(box_node, styles);
        for child in &mut box_node.children {
            rec(child, styles);
        }
    }
    rec(box_node, styles);
}

/// 子树是否含 float 盒（R3929 同款谓词语义——float 参与 static position/可用宽计算）。
pub(super) fn subtree_has_float(b: &LayoutBox) -> bool {
    if b.float != zero_css_parser::values::FloatValue::None {
        return true;
    }
    b.children.iter().any(subtree_has_float)
}

pub(super) fn resolve_abspos_against_nested_cb_inner(
    box_node: &mut LayoutBox,
    current_box_origin_x: f32,
    current_box_origin_y: f32,
    cb: Option<(f32, f32, f32, f32)>,
    styles: &HashMap<NodeId, ComputedStyle>,
    enabled: bool,
) {
    resolve_abspos_against_nested_cb_inner_m(
        box_node,
        current_box_origin_x,
        current_box_origin_y,
        (0.0, 0.0),
        cb,
        styles,
        enabled,
    )
}

/// R4155：带 margin 链的变体——current_box_origin 累积只含 border/padding/x（margin
/// 在盒外），静态位置语义（css-sizing §static position）须含祖先 margin；margin 链仅
/// Stretch 臂消费（其余 inset 算术保持既有口径防回归）。
fn resolve_abspos_against_nested_cb_inner_m(
    box_node: &mut LayoutBox,
    current_box_origin_x: f32,
    current_box_origin_y: f32,
    margin_chain: (f32, f32),
    cb: Option<(f32, f32, f32, f32)>,
    styles: &HashMap<NodeId, ComputedStyle>,
    enabled: bool,
) {
    let (margin_chain_x, margin_chain_y) = margin_chain;
    use zero_css_parser::values::LengthValue;
    if !enabled {
        return;
    }
    let box_is_positioned = box_node.is_abspos_cb;
    // R4408：**inline CB** 直接子重解析臂（实验态 ZW_MIXED_BARE_TEXT=1 同门，随门开走
    // default）——taffy 对 abs 子的 CB 解析发生在**第一趟**，而 inline CB 盒（span[relative]
    // 等）在 taffy 后还会收缩（R372/IFC shrink 仅作用于 inline 族；块级盒 taffy 宽即终值
    // 无过期问题），直接子的 taffy 值基于过期拉伸宽（slice-inline-fragmentation-002：
    // span[relative] 176，其 abs div taffy 按 784 stretch = 789 实证）。
    // 首轮无差别重解析实测 −20（34 案翻红）：块级 CB 的 taffy 基与 walk CB 基在 margin
    // 链/盒形上并不重合，重算反而打掉正确值——收窄为仅 inline CB（过期域恰为 inline 族），
    // 块级 CB 维持旧跳过。重解析公式从 cb+insets 重算。
    let direct_cb_reresolve = std::env::var("ZW_MIXED_BARE_TEXT").as_deref() != Ok("0")
        && box_node
            .node_id
            .and_then(|nid| styles.get(&nid))
            .is_some_and(|s| matches!(s.display, zero_css_parser::values::DisplayValue::Inline));
    for child in &mut box_node.children {
        // 仅当已有 positioned 祖先 CB 时重解析；default 下本盒即 CB（直接子场景）维持
        // 旧跳过（taffy 基解析）。R3902：box_is_positioned 实为 is_abspos_cb——
        // contain:layout/paint 盒同样是 CB，其间层不再被视为「static 中间层」。
        if let Some((cb_origin_x, cb_origin_y, cb_width, cb_height)) = cb
            && (direct_cb_reresolve || !box_is_positioned)
            && child.is_absolute
            && let Some(style) = child.node_id.and_then(|nid| styles.get(&nid))
        {
            // 百分比尺寸：相对 CB（R1227 box-sizing 感知，同 root-CB pass）。
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
            // auto 尺寸 + 全长度 inset → stretch（§10.3.18/§10.6.4，仅非替换）。
            if matches!(style.width, LengthValue::Auto)
                && !child.is_replaced
                && let (Some(left), Some(right)) = (
                    resolve_abspos_real_length(&style.left, &style.font_size, cb_width, cb_height),
                    resolve_abspos_real_length(&style.right, &style.font_size, cb_width, cb_height),
                )
            {
                child.width = (cb_width - left - right).max(0.0);
                child.content_width =
                    (child.width - child.border_left - child.border_right - child.padding_left - child.padding_right)
                        .max(0.0);
            }
            if matches!(style.height, LengthValue::Auto)
                && !child.is_replaced
                && let (Some(top), Some(bottom)) = (
                    resolve_abspos_real_length(&style.top, &style.font_size, cb_width, cb_height),
                    resolve_abspos_real_length(&style.bottom, &style.font_size, cb_width, cb_height),
                )
            {
                child.height = (cb_height - top - bottom).max(0.0);
                child.content_height =
                    (child.height - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                        .max(0.0);
            }
            // left/top（% 或长度）：目标视口绝对坐标 = cb_origin + inset，转回父相对坐标。
            if let LengthValue::Percentage(p) = &style.left {
                child.x = cb_origin_x + *p as f32 / 100.0 * cb_width
                    - current_box_origin_x
                    - box_node.border_left
                    - box_node.padding_left;
            }
            if let LengthValue::Percentage(p) = &style.top {
                child.y = cb_origin_y + *p as f32 / 100.0 * cb_height
                    - current_box_origin_y
                    - box_node.border_top
                    - box_node.padding_top;
            }
            if let Some(px) = resolve_abspos_real_length(&style.left, &style.font_size, cb_width, cb_height) {
                child.x = cb_origin_x + px - current_box_origin_x - box_node.border_left - box_node.padding_left;
            }
            if let Some(px) = resolve_abspos_real_length(&style.top, &style.font_size, cb_width, cb_height) {
                child.y = cb_origin_y + px - current_box_origin_y - box_node.border_top - box_node.padding_top;
            }
            // right/bottom 且 left/top 为 auto：右/下边对齐 CB 右/下缘（§10.3.18 rule 2）。
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
            // R4155（css-sizing-4 §6.2 stretch）：abspos 的 `width/height: stretch`——
            // 尺寸 = 从**静态位置**到 CB 对侧缘的可用空间（insets 全 auto 时静态位置
            // 即盒位）。converter 把 Stretch 映射 auto（taffy 无 stretch 关键字），taffy
            // 后此处补解：positioned-replaced-1/2 canvas `width:stretch; height:stretch`
            // 在 150×150 CB 内静态位 (50,50) → 100×100（旧 0×0——replaced 的 auto-size
            // stretch 臂被 `!is_replaced` 排除，converter 又塌 attr 50×25 为 0）。
            if matches!(style.width, LengthValue::Stretch)
                && matches!(style.left, LengthValue::Auto)
                && matches!(style.right, LengthValue::Auto)
            {
                let child_abs_x = current_box_origin_x
                    + margin_chain_x
                    + box_node.border_left
                    + box_node.padding_left
                    + child.margin_left
                    + child.x;
                let static_offset = child_abs_x - cb_origin_x;
                let new_w = (cb_width - static_offset).max(0.0);
                if (child.width - new_w).abs() > 0.5 {
                    child.width = new_w;
                    child.content_width =
                        (new_w - child.border_left - child.border_right - child.padding_left - child.padding_right)
                            .max(0.0);
                }
            }
            if matches!(style.height, LengthValue::Stretch)
                && matches!(style.top, LengthValue::Auto)
                && matches!(style.bottom, LengthValue::Auto)
            {
                let child_abs_y = current_box_origin_y
                    + margin_chain_y
                    + box_node.border_top
                    + box_node.padding_top
                    + child.margin_top
                    + child.y;
                let static_offset = child_abs_y - cb_origin_y;
                let new_h = (cb_height - static_offset).max(0.0);
                if (child.height - new_h).abs() > 0.5 {
                    child.height = new_h;
                    child.content_height =
                        (new_h - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                }
            }
        }

        // 递归：positioned 子成为其后代的最近 positioned 祖先（padding-box = border-box
        // 内缘）；否则继承当前 CB。child 坐标是相对本盒 content 的偏移。
        let child_box_origin_x = current_box_origin_x + box_node.border_left + box_node.padding_left + child.x;
        let child_box_origin_y = current_box_origin_y + box_node.border_top + box_node.padding_top + child.y;
        let child_cb = if child.is_abspos_cb {
            Some((
                child_box_origin_x + child.border_left,
                child_box_origin_y + child.border_top,
                (child.width - child.border_left - child.border_right).max(0.0),
                (child.height - child.border_top - child.border_bottom).max(0.0),
            ))
        } else {
            cb
        };
        resolve_abspos_against_nested_cb_inner_m(
            child,
            child_box_origin_x,
            child_box_origin_y,
            (margin_chain_x + child.margin_left, margin_chain_y + child.margin_top),
            child_cb,
            styles,
            enabled,
        );
    }
}
