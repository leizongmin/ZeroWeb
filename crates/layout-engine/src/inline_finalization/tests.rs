use super::{
    ComputedStyle, InlineFormattingContext, LayoutBox, TextAlign, resolve_text_align, resolve_text_align_last,
    resolve_text_indent, sync_inline_block_positions_from_ifc, vertical_decoration_free_with_mode,
};
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::Document;
use zero_style_system::property::{DirectionValue, TextAlignLastValue, TextAlignValue};

#[test]
fn test_resolve_text_align_start_end_direction_aware() {
    let mut style = ComputedStyle::default();
    style.direction = DirectionValue::Ltr;
    style.text_align = TextAlignValue::Start;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    style.text_align = TextAlignValue::End;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
    style.text_align = TextAlignValue::Left;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    style.direction = DirectionValue::Rtl;
    style.text_align = TextAlignValue::Start;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Right);
    style.text_align = TextAlignValue::End;
    assert_eq!(resolve_text_align(Some(&style)), TextAlign::Left);
    assert_eq!(resolve_text_align(None), TextAlign::Left);
}

#[test]
fn test_resolve_text_align_last_mapping() {
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Auto;
    assert_eq!(resolve_text_align_last(Some(&style)), None);
    style.text_align_last = TextAlignLastValue::Justify;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Justify));
    style.text_align_last = TextAlignLastValue::Right;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.text_align_last = TextAlignLastValue::Center;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Center));
    style.text_align_last = TextAlignLastValue::Left;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
    assert_eq!(resolve_text_align_last(None), None);
    style.direction = DirectionValue::Ltr;
    style.text_align_last = TextAlignLastValue::Start;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
    style.text_align_last = TextAlignLastValue::End;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.direction = DirectionValue::Rtl;
    style.text_align_last = TextAlignLastValue::Start;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Right));
    style.text_align_last = TextAlignLastValue::End;
    assert_eq!(resolve_text_align_last(Some(&style)), Some(TextAlign::Left));
}

#[test]
fn test_resolve_text_indent_px_em_percentage() {
    assert_eq!(
        resolve_text_indent(&LengthValue::Px(40.0), &LengthValue::Px(16.0), 800.0),
        40.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Em(5.0), &LengthValue::Px(16.0), 800.0),
        80.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Percentage(50.0), &LengthValue::Px(16.0), 800.0),
        400.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Auto, &LengthValue::Px(16.0), 800.0),
        0.0
    );
}

#[test]
fn test_resolve_text_indent_relative_lengths() {
    assert_eq!(
        resolve_text_indent(&LengthValue::Ch(4.0), &LengthValue::Px(20.0), 800.0),
        40.0
    );
    assert_eq!(
        resolve_text_indent(&LengthValue::Rem(2.0), &LengthValue::Px(20.0), 800.0),
        32.0
    );
}

#[test]
fn horizontal_decoration_gate_skips_subtree_scan() {
    let scans = std::cell::Cell::new(0);
    assert!(vertical_decoration_free_with_mode(true, false, || {
        scans.set(scans.get() + 1);
        true
    }));
    assert_eq!(scans.get(), 0);

    assert!(!vertical_decoration_free_with_mode(true, true, || {
        scans.set(scans.get() + 1);
        true
    }));
    assert_eq!(scans.get(), 1);
}

#[test]
fn inline_block_position_reuse_is_complete_and_fail_closed() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    let text = doc.create_text_node("prefix");
    let inline_block = doc.create_element("span");
    doc.append_child(container, text).unwrap();
    doc.append_child(container, inline_block).unwrap();

    let mut styles = HashMap::new();
    styles.insert(container, ComputedStyle::default());
    let mut inline_block_style = ComputedStyle::default();
    inline_block_style.display = DisplayValue::InlineBlock;
    styles.insert(inline_block, inline_block_style);

    let mut sizes = HashMap::new();
    sizes.insert(inline_block, (40.0, 2.0));
    let mut context = InlineFormattingContext::new(200.0).with_inline_block_sizes(sizes);
    context.layout(&doc, container, &styles);
    let stale_y = context
        .all_fragments_with_line_y()
        .into_iter()
        .find(|fragment| fragment.node_id == inline_block)
        .unwrap()
        .y;

    let mut root = LayoutBox {
        node_id: Some(container),
        children: vec![LayoutBox {
            node_id: Some(inline_block),
            width: 40.0,
            height: 25.0,
            ..LayoutBox::default()
        }],
        ..LayoutBox::default()
    };
    let mut final_sizes = HashMap::new();
    final_sizes.insert(inline_block, (40.0, 25.0));
    assert!(context.refresh_reused_inline_block_metrics(&doc, &styles, &final_sizes));
    assert!(sync_inline_block_positions_from_ifc(&mut root, &context, &doc, &styles));
    assert!(root.children[0].x > 0.0);
    assert!(root.children[0].y < stale_y);

    styles.get_mut(&inline_block).unwrap().display = DisplayValue::InlineFlex;
    assert!(!context.refresh_reused_inline_block_metrics(&doc, &styles, &final_sizes));
    assert!(!sync_inline_block_positions_from_ifc(
        &mut root, &context, &doc, &styles
    ));
}
