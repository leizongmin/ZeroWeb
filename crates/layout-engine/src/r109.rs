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
        // R4338：block-mixed 宿主的匿名块片段**不收缩**——§9.2.1.1 匿名块盒是块容器
        // 的块级子（满宽语义），文本贴宽收缩只适用于 split inline 的片段（其 inline
        // border/background 须落在文本宽）。block-mixed 片段 dom_id = 宿主容器
        //（taffy_to_dom 映射），is_r109_block_mixed 由此为 true；split inline 片段的
        // dom_id = inline 元素自身，该标志为 false（run-in-contains-block-001：片段被
        // 缩到 107.2 后 R3770 片段 remeasure 以缩宽跑 IFC 折 2 行 → 父盒 19→37.2，
        // 后继整体下移 18.6px = 3.95% 签名）。
        && !box_node.is_r109_block_mixed
    {
        // 1. fragment border 边选择（先于收缩，使 frame 用开放后的 border）。
        // R5335：CSS2 §9.2.1.1 切片边框三态——首片段承担 left（notend 清 right）、
        // 末片段承担 right（notstart 清 left）、**首末合一的单片段保留全边**、
        // middle 片段（首末皆非）左右全清（nosplit-ref 的 notstart+notend 形）。
        // 旧二态清法把单片段错清 right、middle 片段错留全边（insert-009 Six 段）。
        if box_node.r109_first_fragment && box_node.r109_last_fragment {
            // 单片段（首末合一）：left+right 全保留。
        } else if box_node.r109_first_fragment {
            box_node.border_right = 0.0;
        } else if box_node.r109_last_fragment {
            box_node.border_left = 0.0;
        } else {
            box_node.border_left = 0.0;
            box_node.border_right = 0.0;
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

/// R5335：剥离 split inline 壳盒 frame 对切片内容的布局推进。
///
/// CSS2 §9.2.1.1：inline 被块级后代切片后，切片序列（匿名块片段 + 块级子）在
/// 块层排布——壳盒（split inline 宿主）的 border/padding 已随切片下放到各片段
/// （边选择 + 收缩），不再占行内推进；taffy 层切片内容仍相对壳 content 原点
/// 定位（继承壳 `frame_left` 推进），本 pass 按目标语义平移：
/// - 匿名块片段（`fragment_node_ids` 非 None）：x 剥**壳自身** frame_left，保留
///   祖先 inline 链推进——外层 inline 的 border 仍水平推进切片起点（nosplit-ref
///   slice1 的 blue/cyan 双左带错开 3px 实证）；
/// - 块级子（Block 段 div）：x 剥**全部** inline 链推进——块级子的包含块是外层
///   块容器（§9.2.1.1），左缘回到块层（insert-009 div Four 左缘 = body 缘）；
/// - 进入块容器（片段/块子）后累计清零（新块层重新累计）；
/// - abs/fixed 盒定位基准独立于流内推进，不平移。
pub(crate) fn unshift_r109_slice_content_frames(box_node: &mut LayoutBox, inline_frame_acc: f32) {
    let is_split_shell = box_node.is_r109_split && box_node.fragment_node_ids.is_none();
    if is_split_shell {
        let frame = box_node.padding_left + box_node.border_left;
        let acc_with_self = inline_frame_acc + frame;
        let children = &mut box_node.children;
        for c in children.iter_mut() {
            if c.is_absolute || c.is_fixed {
                continue;
            }
            if c.fragment_node_ids.is_some() {
                c.x -= frame;
            } else if c.is_block_level {
                c.x -= acc_with_self;
            }
        }
        // 片段/块子均为块容器：其子树从零累计重新起算。
        for c in children.iter_mut() {
            unshift_r109_slice_content_frames(c, 0.0);
        }
        return;
    }
    let frame = box_node.padding_left + box_node.border_left;
    let child_acc = if box_node.is_block_level {
        0.0
    } else {
        inline_frame_acc + frame
    };
    for c in &mut box_node.children {
        unshift_r109_slice_content_frames(c, child_acc);
    }
}
