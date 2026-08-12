//! `unicode-bidi: plaintext` 行断与 paint Path B 恢复测试。

use super::super::*;
use crate::LayoutBox;
use std::collections::HashMap;
use zero_css_parser::values::VerticalAlignValue;
use zero_dom::{NodeKind, parse_html};
use zero_style_system::{ComputedStyle, UnicodeBidiValue};

#[test]
fn plaintext_breaks_logically_and_aligns_soft_lines_by_paragraph_direction() {
    let mut doc = zero_dom::Document::new();
    let rtl_node = doc.create_text_node("");
    let ltr_node = doc.create_text_node("");
    let mut rtl = TextRun::simple(
        "אבגדהוזח MMMMM".to_string(),
        rtl_node,
        16.0,
        20.0,
        VerticalAlignValue::Baseline,
    );
    rtl.is_plaintext_bidi = true;
    let mut ltr = TextRun::simple("abc".to_string(), ltr_node, 16.0, 20.0, VerticalAlignValue::Baseline);
    ltr.is_plaintext_bidi = true;

    let mut ctx = InlineFormattingContext::new(80.0).with_plaintext_bidi(true, true);
    ctx.break_items_into_lines(vec![InlineItem::Text(rtl), InlineItem::Br, InlineItem::Text(ltr)]);

    assert_eq!(ctx.lines.len(), 3);
    assert_eq!(ctx.lines[0].runs[0].text, "חזוהדגבא");
    assert_eq!(ctx.lines[1].runs[0].text, "MMMMM");
    assert!(ctx.lines[0].runs[0].x > 0.0);
    assert!(ctx.lines[1].runs[0].x > 0.0);
    assert_eq!(ctx.lines[2].runs[0].x, 0.0);

    let mut explicit_left = InlineFormattingContext::new(80.0).with_plaintext_bidi(true, false);
    let mut rtl = TextRun::simple(
        "אבגדהוזח MMMMM".to_string(),
        rtl_node,
        16.0,
        20.0,
        VerticalAlignValue::Baseline,
    );
    rtl.is_plaintext_bidi = true;
    explicit_left.break_into_lines(vec![rtl]);
    assert_eq!(explicit_left.lines[0].runs[0].text, "חזוהדגבא");
    assert_eq!(explicit_left.lines[0].runs[0].x, 0.0);
}

/// R3289-F：paint Path B 空 styles IFC 须按 inline owner 恢复 plaintext。
#[test]
fn plaintext_inline_owner_is_restored_via_override_set() {
    let doc = parse_html("<p>&gt; <span>אבג דהו</span> &gt;</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();
    let span = doc
        .child_nodes(p)
        .iter()
        .copied()
        .find(|id| {
            doc.get(*id)
                .is_some_and(|node| matches!(node.kind, NodeKind::Element(_)))
        })
        .expect("span");

    let mut span_style = ComputedStyle::default();
    span_style.unicode_bidi = UnicodeBidiValue::Plaintext;
    let styles = HashMap::from([(span, span_style)]);
    let mut layout_ctx = InlineFormattingContext::new(800.0);
    layout_ctx.layout(&doc, p, &styles);
    let mut box_node = LayoutBox::default();
    crate::inline_finalization::store_font_sizes_from_ifc(&layout_ctx, &mut box_node, &doc, &styles);
    assert!(box_node.plaintext_bidi_nodes.contains(&span));

    let mut ctx = InlineFormattingContext::new(800.0).with_plaintext_bidi_overrides(box_node.plaintext_bidi_nodes);
    ctx.layout(&doc, p, &HashMap::new());

    let span_fragments = ctx
        .all_fragments()
        .into_iter()
        .filter(|fragment| fragment.node_id == span)
        .collect::<Vec<_>>();
    assert_eq!(span_fragments.len(), 1);
    assert_eq!(span_fragments[0].text, "והד גבא");
}
