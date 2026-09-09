//! R3994（css-sizing-4 §4.2 transferred size suggestion）：plain block 的
//! aspect-ratio 传递后处理。
//!
//! taffy 0.12 block 布局对「CSS aspect-ratio + width/height 双 Auto」不做 auto→auto
//! 比传递（仅 replaced/flex 路径有语义），普通 block 盒高塌 0 或停留在内容高。本 pass
//! 在 float 定位/BFC 收缩**之后**运行——此时 inline 轴宽度已是最终值（含 float 避让、
//! shrink-to-fit），对双 auto + 有 ratio 的非替换水平块按 `height = width / ratio`
//! 传递（双向钳制：内容高与传递值取大者，min/max 由既有钳制路径处理）。
//!
//! 布局期（engine/sizing.rs `apply_aspect_ratio_container_cross_size`）只覆盖 flex/grid
//! 容器：first-pass 的宽度未含 float 避让（floats-aspect-ratio-001 会传 200 而非避让
//! 后 40），plain block 必须在 postprocess 最终宽度上做。
//!
//! kill-switch `ZW_AR_TRANSFER=0`。

use crate::LayoutBox;
use zero_css_parser::values::{
    AlignmentValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue, LengthValue,
};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::WritingModeValue;

use std::collections::HashMap;

pub(crate) fn transfer_aspect_ratio_height(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_AR_TRANSFER").as_deref() == Ok("0") {
        return;
    }
    walk(root, styles);
}

fn walk(b: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    // R4076（css-sizing-4 §4.2 "specified but indefinite"）：float 盒 height:100% 在
    // indefinite CB 下不可解析——高由 min-height 地板撑起（b.height>0），但 shrink-to-fit
    // 宽塌 0；transferred min-width = main×ratio 需要传递到宽（block-aspect-ratio-025：
    // `float:left; aspect-ratio:1/1; height:100%; min-height:100px` 应 100×100，ZW 0×100）。
    // 仅 float（width 真 indefinite）：height 百分比在 definite-CB 块上已解析，不得反推宽。
    if let Some(id) = b.node_id
        && let Some(style) = styles.get(&id)
        && matches!(b.writing_mode, WritingModeValue::HorizontalTb)
        && let Some(ratio) = style.aspect_ratio.filter(|&r| r > 0.0)
        && matches!(style.display, DisplayValue::Block | DisplayValue::FlowRoot)
        && !b.is_replaced
        && !b.is_flex_grid_item
        && !b.is_absolute
        && !b.is_fixed
        && matches!(style.width, LengthValue::Auto)
        && matches!(style.height, LengthValue::Percentage(_))
        && !matches!(style.float, FloatValue::None)
    {
        let main = b.height - b.padding_top - b.padding_bottom - b.border_top - b.border_bottom;
        let transferred_w = main * ratio;
        if main > 0.5 && transferred_w > b.width + 0.5 {
            let frame = b.padding_left + b.padding_right + b.border_left + b.border_right;
            b.width = transferred_w + frame;
            b.content_width = transferred_w;
        }
    }
    if let Some(id) = b.node_id
        && let Some(style) = styles.get(&id)
        && matches!(b.writing_mode, WritingModeValue::HorizontalTb)
        && let Some(ratio) = style.aspect_ratio.filter(|&r| r > 0.0)
        && matches!(style.display, DisplayValue::Block | DisplayValue::FlowRoot)
        && !b.is_replaced
        && !b.is_flex_grid_item
        && !b.is_absolute
        && !b.is_fixed
        && matches!(style.width, LengthValue::Auto)
        && matches!(style.height, LengthValue::Auto)
    {
        let frame = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
        let content_w = (b.width - b.padding_left - b.padding_right - b.border_left - b.border_right).max(0.0);
        // 传递高 = content 宽 / ratio（css-sizing-4：ratio 作用在 box-sizing 指定盒，
        // content-box 默认下按 content 宽传 content 高）。传递值与现有高取大者——
        // 内容（子块流）高于传递值时以内容为准（§4.1 automatic minimum 的近似，
        // 精确 min-content 测量为 RFC 域）；taffy 塌 0 时传递值生效。
        let transferred = content_w / ratio;
        let target = transferred.max(b.content_height);
        if (b.height - (target + frame)).abs() > 0.5 && content_w > 0.5 {
            b.content_height = target;
            b.height = target + frame;
        }
    }
    // R4152（css-sizing-4 §4.1 automatic content-based minimum）：flex/grid 容器
    // definite inline size + AR + Auto block size——R3912 tree build 期按 transferred
    // 写死 taffy size.height（040：w:100 + AR 2/1 → 50），内容高于传递值时（item
    // h:100 in-flow）§4.1 内容最小尺寸不被 transferred 钳——chromium 容器高 = 内容
    // 100。本臂对齐既有块级臂语义：target = max(transferred, content_height)。
    if let Some(id) = b.node_id
        && let Some(style) = styles.get(&id)
        && matches!(b.writing_mode, WritingModeValue::HorizontalTb)
        && let Some(ratio) = style.aspect_ratio.filter(|&r| r > 0.0)
        && matches!(
            style.display,
            DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
        )
        && !b.is_replaced
        && !b.is_absolute
        && !b.is_fixed
        && resolve_definite(&style.width).is_some()
        && matches!(style.height, LengthValue::Auto)
    {
        let main = resolve_definite(&style.width).unwrap_or(0.0);
        let frame_v = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
        // content_height 以容器内容盒顶为原点；无子内容时 R3912 的 transferred 保持。
        let transferred = main / ratio;
        let content = b.children.iter().map(|c| c.y + c.height).fold(0.0_f32, f32::max);
        // R4152b：max-height 上限参与——CSS2 §10.4 max 胜过内容/transferred
        //（flex-aspect-ratio-043/044：max-block-size:100px 钳内容 200 → 容器 100）。
        let target = transferred
            .max(content)
            .min(resolve_definite(&style.max_height).unwrap_or(f32::INFINITY));
        if content > transferred + 0.5 && (b.height - (target + frame_v)).abs() > 0.5 {
            b.content_height = target;
            b.height = target + frame_v;
        }
    }
    for child in &mut b.children {
        walk(child, styles);
    }
}

/// 定值 real length 解析（Px/Em/Rem/Ch；Auto/百分比/关键字 → None）。
fn resolve_definite(value: &LengthValue) -> Option<f32> {
    match value {
        LengthValue::Auto | LengthValue::Percentage(_) | LengthValue::MinContent | LengthValue::MaxContent => None,
        LengthValue::Px(v) if *v == f64::INFINITY => None,
        other => {
            let px = zero_style_system::computed::resolve_length(other, 16.0, None, None);
            px.is_finite().then_some(px.max(0.0) as f32)
        }
    }
}

/// R4185（css-flexbox-1 §algo-cross-line step 4）：单行 flex 容器的 line cross 钳制——
/// 「If the flex container is single-line, then clamp the line's cross-size to be within
/// the container's computed min and max cross sizes」。
///
/// taffy 0.12.1 对 row 方向单行 flex 不做该钳制：容器 max-height:200 被遵守（容器盒
/// 200），但 stretch 对齐的 item（cross Auto）保持内容高 402 溢出容器（flexbox-single-line-
/// clamp-1 ref：panel 应 200，tall-child 400 定高溢出）。本 pass 在布局后对最终
/// LayoutBox 收缩 stretch 对齐 item 的 border-box 高到钳制 line cross（= 容器 content-box
/// max-height 减 item 自身 margin）——定高 item（tall-child height:400）不动（spec：
/// 钳制的是 line，非 item 的 definite cross）。
///
/// 范围限定：row/row-reverse（column 的 cross=inline 轴，taffy 已正确，-2/-3 现绿）；
/// flex_wrap: Nowrap（wrap 按 line 各自钳制，未建线模型）；水平书写模式；
/// max-height definite（Px/可解析长度）；仅收缩（item 高 ≤ 钳制值不动）。
///
/// kill-switch `ZW_FLEX_LINE_CLAMP=0`。
pub(crate) fn clamp_single_line_flex_cross(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_FLEX_LINE_CLAMP").as_deref() == Ok("0") {
        return;
    }
    walk_flex_cross(root, styles);
}

fn walk_flex_cross(b: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if let Some(id) = b.node_id
        && let Some(style) = styles.get(&id)
        && matches!(b.writing_mode, WritingModeValue::HorizontalTb)
        && matches!(style.display, DisplayValue::Flex | DisplayValue::InlineFlex)
        && matches!(
            style.flex_direction,
            FlexDirectionValue::Row | FlexDirectionValue::RowReverse
        )
        && matches!(style.flex_wrap, FlexWrapValue::Nowrap)
        && matches!(
            style.align_items,
            AlignmentValue::Auto | AlignmentValue::Normal | AlignmentValue::Stretch
        )
    {
        let max_cross = match &style.max_height {
            LengthValue::Auto
            | LengthValue::Percentage(_)
            | LengthValue::MinContent
            | LengthValue::MaxContent
            | LengthValue::FitContent(_) => None,
            LengthValue::Px(p) if *p == f64::INFINITY => None,
            other => {
                let fs = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
                let px = zero_style_system::computed::resolve_length(other, fs, None, None);
                (px.is_finite() && px > 0.0).then_some(px as f32)
            }
        };
        if let Some(max_cross) = max_cross {
            if std::env::var("ZW_DBG4185").is_ok() {
                eprintln!(
                    "DBG4185 container={:?} max_cross={} children={}",
                    b.node_id,
                    max_cross,
                    b.children.len()
                );
            }
            for c in b.children.iter_mut() {
                let in_flow = !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None);
                if !in_flow || c.is_replaced {
                    continue;
                }
                let self_stretch = c.node_id.and_then(|cid| styles.get(&cid)).is_some_and(|s| {
                    matches!(
                        s.align_self,
                        AlignmentValue::Auto | AlignmentValue::Normal | AlignmentValue::Stretch
                    )
                }) || {
                    // LayoutBox 无 align_self 快照时的保守回退：item 样式缺失按 stretch。
                    c.node_id.and_then(|cid| styles.get(&cid)).is_none()
                };
                let definite_cross = c
                    .node_id
                    .and_then(|cid| styles.get(&cid))
                    .is_some_and(|s| !matches!(s.height, LengthValue::Auto));
                if self_stretch && !definite_cross && c.height > max_cross + 0.5 {
                    let target = (max_cross - c.margin_top - c.margin_bottom).max(0.0);
                    if target > 0.5 {
                        let frame_v = c.padding_top + c.padding_bottom + c.border_top + c.border_bottom;
                        c.height = target;
                        c.content_height = (target - frame_v).max(0.0);
                    }
                }
            }
        }
    }
    for c in b.children.iter_mut() {
        walk_flex_cross(c, styles);
    }
}
