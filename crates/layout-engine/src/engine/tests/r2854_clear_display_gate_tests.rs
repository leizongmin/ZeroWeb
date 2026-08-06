//! R2854：clear display-gate 单元测试。
//!
//! CSS 2.1 §9.10：`clear` 仅适用于 block-level 元素。internal table 元素（caption 等）+
//! inline-table 非 block-level，clear 须忽略（= None）。driving：WPT
//! `css/CSS2/floats-clear/clear-applies-to-015`（display:table-caption + clear:both →
//! clear 须被 gate 忽略；fix 后 reftest PASS，未 fix 前 ZW 误 apply clear 致 3.83% diff）。
//! clear-applies-to-014（inline-table）clear 亦被 gate 忽略，但其 fail 余量来自 inline-table
//! 布局精度（非 clear）故仍 fail。
//!
//! 本测试直接断言 LayoutBox.clear：table-caption/inline-table + clear:both → None（gate 生效），
//! block + clear:both → Both（gate 不误杀 block-level，控制组）。

use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue};
use zero_style_system::ComputedStyle;

/// 构造 body 含 3 个 clear:both 元素：caption（table-caption）/ itbl（inline-table）/ blk（block）。
fn build_clear_gate_doc() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let caption = doc.create_element("div");
    doc.append_child(body, caption).unwrap();
    let itbl = doc.create_element("div");
    doc.append_child(body, itbl).unwrap();
    let blk = doc.create_element("div");
    doc.append_child(body, blk).unwrap();
    let mut styles = HashMap::new();
    let mut cap_s = ComputedStyle::default();
    cap_s.display = DisplayValue::TableCaption;
    cap_s.clear = ClearValue::Both;
    styles.insert(caption, cap_s);
    let mut it_s = ComputedStyle::default();
    it_s.display = DisplayValue::InlineTable;
    it_s.clear = ClearValue::Both;
    styles.insert(itbl, it_s);
    let mut blk_s = ComputedStyle::default();
    blk_s.display = DisplayValue::Block;
    blk_s.clear = ClearValue::Both;
    styles.insert(blk, blk_s);
    (doc, styles, caption, itbl, blk)
}

/// display:table-caption + clear:both → clear 须被 display-gate 忽略（None）。
#[test]
fn r2854_clear_ignored_for_table_caption() {
    let (doc, styles, caption, _itbl, _blk) = build_clear_gate_doc();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let cap_box = find_child_by_node_id(&result.root, caption).expect("caption LayoutBox found");
    assert_eq!(
        cap_box.clear,
        ClearValue::None,
        "display:table-caption + clear:both → clear 须被 gate 忽略（None），CSS 2.1 §9.10 clear 仅适用 block-level"
    );
}

/// display:inline-table + clear:both → clear 须被 gate 忽略（None）。inline-table 为 inline-level。
#[test]
fn r2854_clear_ignored_for_inline_table() {
    let (doc, styles, _caption, itbl, _blk) = build_clear_gate_doc();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let it_box = find_child_by_node_id(&result.root, itbl).expect("inline-table LayoutBox found");
    assert_eq!(
        it_box.clear,
        ClearValue::None,
        "display:inline-table + clear:both → clear 须被 gate 忽略（None），inline-table 非 block-level"
    );
}

/// 控制组：display:block + clear:both → clear 须正常应用（Both）。防 gate 误杀 block-level。
#[test]
fn r2854_clear_applies_to_block_control() {
    let (doc, styles, _caption, _itbl, blk) = build_clear_gate_doc();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let blk_box = find_child_by_node_id(&result.root, blk).expect("block LayoutBox found");
    assert_eq!(
        blk_box.clear,
        ClearValue::Both,
        "display:block + clear:both → clear 须应用（Both）—— 控制组，gate 不可误杀 block-level"
    );
}
