//! R939 Batch 1：R109 §9.2.1.1 匿名块盒高度回填（spec FR-001）单测。
//!
//! 验证 `backfill_r109_anon_block_heights`：
//! ① 匿名块盒（fragment_node_ids.is_some）从 inline_layout 行盒回填 content_height
//!    （仅增大，不收缩）；
//! ② auto-height 祖先容器按直系匿名块子的 delta 之和扩展自身高度；
//! ③ 非 auto-height 容器（显式 height）不扩展；
//! ④ 非 anon 块不回填。
//!
//! NodeId 是 slotmap opaque key，须经 Document 取有效值；backfill 本身不消费 Document，
//! 仅用 NodeId 作 styles HashMap 的键。

use super::*;
use crate::types::{InlineLayoutFragment, InlineLayoutLine};
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

/// 取一个有效 NodeId（经 Document）。
fn fresh_id() -> NodeId {
    Document::new().create_element("div")
}

/// 构造一行内布局行盒（y, height）。
fn make_line(y: f32, height: f32) -> InlineLayoutLine {
    InlineLayoutLine {
        y,
        height,
        fragments: vec![InlineLayoutFragment {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height,
            font_size: 20.0,
            is_ahem: false,
            is_ahem_font: false,
            text: String::new(),
            node_id: None,
            baseline_y: height,
        }],
        baseline_y: height,
        ascent: height * 0.8,
        descent: height * 0.2,
    }
}

#[test]
fn test_backfill_anon_block_height_from_inline_layout() {
    // 容器（height:auto），content_height=50（taffy 测：仅 div.inserted 高）。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 50.0;
    container.height = 50.0;
    container.is_block_level = true;

    // 匿名块子（fragment_node_ids=Some）：taffy 经 ctx_node 欠计 content_height=20，
    // 但 inline_layout 显示真实 inline run = 2 行 × 20 = 40。
    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 20.0;
    anon.height = 20.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0), make_line(20.0, 20.0)]);

    container.children.push(anon);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Auto;
    styles.insert(cid, s);

    let delta = backfill_r109_anon_block_heights(&mut container, &styles);

    // 匿名块 content_height 从 20 回填到 40（max(0+20, 20+20)=40）。
    let anon = &container.children[0];
    assert!(
        (anon.content_height - 40.0).abs() < 0.01,
        "anon content_height backfilled to 40"
    );
    assert!((anon.height - 40.0).abs() < 0.01, "anon height backfilled to 40");
    // 容器扩展 delta=20（50→70）。
    assert!((delta - 20.0).abs() < 0.01, "returns delta 20");
    assert!(
        (container.content_height - 70.0).abs() < 0.01,
        "container content_height extended to 70"
    );
    assert!(
        (container.height - 70.0).abs() < 0.01,
        "container height extended to 70"
    );
}

#[test]
fn test_backfill_skips_explicit_height_container() {
    // 容器显式 height:100px → 不应被匿名块 delta 扩展。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 100.0;
    container.height = 100.0;
    container.is_block_level = true;

    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 10.0;
    anon.height = 10.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 30.0)]);

    container.children.push(anon);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Px(100.0);
    styles.insert(cid, s);

    let delta = backfill_r109_anon_block_heights(&mut container, &styles);

    // 匿名块自身仍回填（30），但容器（显式 height）不扩展 → 返回 0（不向上传播）。
    assert!(
        (container.children[0].content_height - 30.0).abs() < 0.01,
        "anon still backfilled"
    );
    assert!(
        (delta - 0.0).abs() < 0.01,
        "no propagation to explicit-height container"
    );
    assert!(
        (container.content_height - 100.0).abs() < 0.01,
        "explicit container unchanged"
    );
}

#[test]
fn test_backfill_does_not_shrink() {
    // 匿名块 content_height 已 ≥ inline_layout → 不收缩。
    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 60.0;
    anon.height = 60.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0)]); // inline 仅 20 < 60

    let styles = HashMap::new();
    let mut container = LayoutBox::default();
    container.children.push(anon);
    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!((container.children[0].content_height - 60.0).abs() < 0.01, "not shrunk");
    assert!((container.children[0].height - 60.0).abs() < 0.01, "height not shrunk");
}

#[test]
fn test_backfill_ignores_non_anon_block() {
    // 非 anon 块（fragment_node_ids=None）即使有 inline_layout 也不回填。
    let mut block = LayoutBox::default();
    block.fragment_node_ids = None;
    block.content_height = 10.0;
    block.height = 10.0;
    block.is_block_level = true;
    block.inline_layout = Some(vec![make_line(0.0, 40.0)]);

    let styles = HashMap::new();
    let mut container = LayoutBox::default();
    container.children.push(block);
    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.children[0].content_height - 10.0).abs() < 0.01,
        "non-anon not touched"
    );
}
