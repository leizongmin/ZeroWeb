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
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
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
    for child in &mut b.children {
        walk(child, styles);
    }
}
