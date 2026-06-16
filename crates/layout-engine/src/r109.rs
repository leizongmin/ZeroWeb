//! R109（CSS2 §9.2.1.1）匿名块片段收缩 + fragment border 边选择后处理。
//!
//! split inline 的匿名块片段（由 `tree.rs` 在 `r109_wired()` 时生成，标记
//! `LayoutBox.fragment_node_ids`）默认被 taffy 拉伸到可用宽（全宽），致 inline 的
//! border/background 落在全宽而非文本宽。本模块的后处理把片段收缩到文本宽并应用
//! fragment border 边选择，使 border/background 落在 inline 级（文本宽）。
//!
//! 从 `engine.rs` 抽出以控制单文件行数（engine.rs 已超 2000 行）。

use std::collections::HashMap;

use zero_dom::{Document, NodeId};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::intrinsic_sizing::fragment_inline_max_width;
use crate::types::LayoutBox;

/// 把 split inline 的匿名块片段收缩到文本宽 + 应用 fragment border 边选择。
///
/// 处理：
/// 1. **边选择**——首 Inline 片段开放右分裂边（`border_right=0`），末片段开放左分裂边
///    （`border_left=0`）。CSS2 §9.2.1.1：被拆分 inline 的边框不在分裂边绘制。
/// 2. **收缩**——用 `fragment_inline_max_width` 测片段文本 max-content 宽（与 paint
///    IFC 同用 estimate_char_width，故收缩宽=渲染宽，自洽），仅在可测且更窄时把
///    `width`/`content_width` 收缩到 文本宽 + frame（frame 用边选择后的 border）。
///
/// 不可测（纯空白/无文本，返回 0）的片段跳过——保持全宽（中性，避免误收缩）。
pub(crate) fn shrink_r109_anon_blocks(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if box_node.fragment_node_ids.is_some()
        && matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)
        && !box_node.is_absolute
        && !box_node.is_fixed
    {
        // 1. fragment border 边选择（先于收缩，使 frame 用开放后的 border）。
        if box_node.r109_first_fragment {
            box_node.border_right = 0.0;
        }
        if box_node.r109_last_fragment {
            box_node.border_left = 0.0;
        }

        // 2. 收缩到文本宽。字体度量取自 split inline 自身（node_id=inline）。
        let measurable = box_node.node_id.is_some_and(|id| styles.get(&id).is_some());
        if measurable
            && let Some(ref frag_ids) = box_node.fragment_node_ids
            && let Some(inline_style) = box_node.node_id.and_then(|id| styles.get(&id))
        {
            let text_w = fragment_inline_max_width(inline_style, frag_ids, doc);
            if std::env::var("R109_DBG").ok().as_deref() == Some("1") {
                eprintln!(
                    "R109_DBG shrink: node={:?} first={} last={} text_w={} cur_w={} bl={} br={}",
                    box_node.node_id,
                    box_node.r109_first_fragment,
                    box_node.r109_last_fragment,
                    text_w,
                    box_node.width,
                    box_node.border_left,
                    box_node.border_right
                );
            }
            if text_w > 0.0 {
                let frame =
                    box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
                let shrink_border_box = text_w + frame;
                if shrink_border_box + 0.5 < box_node.width {
                    box_node.width = shrink_border_box;
                    box_node.content_width = text_w;
                }
            }
        }
    }

    for child in &mut box_node.children {
        shrink_r109_anon_blocks(child, doc, styles);
    }
}
