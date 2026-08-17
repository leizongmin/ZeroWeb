//! Stores IFC fragment metrics required by the paint fallback path.

use std::collections::HashMap;
use std::sync::OnceLock;

use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

use crate::inline::{InlineFormattingContext, TextFragment};
use crate::types::LayoutBox;

fn metric_dedup_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_IFC_METRIC_DEDUP").as_deref() != Ok("0"))
}

/// Stores per-node metrics from the final IFC fragments.
pub(crate) fn store_font_sizes_from_ifc(
    inline_ctx: &InlineFormattingContext,
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    store_font_sizes_from_ifc_mode(inline_ctx, box_node, doc, styles, metric_dedup_enabled());
}

fn store_font_sizes_from_ifc_mode(
    inline_ctx: &InlineFormattingContext,
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dedup: bool,
) {
    let mut store = |frag: &TextFragment| {
        box_node.text_node_font_sizes.insert(frag.node_id, frag.font_size);
        // R1464: paint Path B has no style map, so retain the fragment owner's
        // family and font-size-adjust values under the fragment NodeId.
        let font_owner = if doc
            .get(frag.node_id)
            .is_some_and(|node| matches!(node.kind, NodeKind::Element(_)))
        {
            Some(frag.node_id)
        } else {
            doc.parent_node(frag.node_id)
        };
        let font_style = font_owner.and_then(|owner| styles.get(&owner));
        box_node.text_node_font_families.insert(
            frag.node_id,
            font_style.map(|style| style.font_family.clone()).unwrap_or_default(),
        );
        if let Some(style) = font_style {
            box_node
                .text_node_font_size_adjust
                .insert(frag.node_id, style.font_size_adjust);
            if matches!(style.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext)
                && let Some(owner) = font_owner
            {
                box_node.plaintext_bidi_nodes.insert(owner);
            }
        }
        box_node.text_node_is_ahem.insert(frag.node_id, frag.is_ahem);
        box_node
            .text_node_letter_spacing
            .insert(frag.node_id, frag.letter_spacing);
        box_node.text_node_line_heights.insert(frag.node_id, frag.height);
        // R1012: text-transform belongs to the text node's parent style but is
        // restored by fragment NodeId when paint reruns IFC with empty styles.
        if doc
            .get(frag.node_id)
            .is_some_and(|node| matches!(node.kind, NodeKind::Text(_)))
            && let Some(parent) = doc.parent_node(frag.node_id)
        {
            let transform = styles
                .get(&parent)
                .map(|style| style.text_transform)
                .unwrap_or(zero_style_system::TextTransformValue::None);
            box_node.text_node_text_transform.insert(frag.node_id, transform);
        }
        box_node
            .inline_element_metrics
            .insert(frag.node_id, (frag.font_size, frag.height));
        box_node
            .inline_element_margins
            .insert(frag.node_id, (frag.margin_left, frag.margin_right));
    };

    if dedup {
        // OPTIMIZATION: split words and wrapped lines commonly form adjacent
        // runs for one text node. Delaying one fragment keeps the last value
        // in each run without adding another NodeId hash table.
        let mut pending: Option<&TextFragment> = None;
        for line in &inline_ctx.lines {
            for frag in &line.runs {
                if pending.is_some_and(|previous| previous.node_id != frag.node_id) {
                    store(pending.take().expect("pending fragment"));
                }
                pending = Some(frag);
            }
        }
        if let Some(frag) = pending {
            store(frag);
        }
    } else {
        for line in &inline_ctx.lines {
            for frag in &line.runs {
                store(frag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::{LineBox, TextFragment};
    use zero_css_parser::values::VerticalAlignValue;

    fn fragment(node_id: NodeId, font_size: f32, height: f32) -> TextFragment {
        TextFragment {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height,
            text: "word".to_string(),
            source: None,
            node_id,
            font_size,
            vertical_align: VerticalAlignValue::Baseline,
            is_ahem: font_size > 10.0,
            letter_spacing: font_size / 10.0,
            margin_left: font_size,
            margin_right: height,
            margin_top: 0.0,
            baseline: font_size,
        }
    }

    #[test]
    fn dedup_preserves_last_fragment_values() {
        let mut doc = Document::new();
        let root = doc.root();
        let owner = doc.create_element("div");
        doc.append_child(root, owner).unwrap();
        let text = doc.create_text_node("first second");
        doc.append_child(owner, text).unwrap();
        let other_text = doc.create_text_node("middle");
        doc.append_child(owner, other_text).unwrap();

        let mut style = ComputedStyle::default();
        style.font_family = vec!["Ahem".to_string()];
        style.unicode_bidi = zero_style_system::UnicodeBidiValue::Plaintext;
        let styles = HashMap::from([(owner, style)]);

        let mut inline_ctx = InlineFormattingContext::new(20.0);
        inline_ctx.lines = vec![
            LineBox {
                y: 0.0,
                height: 12.0,
                runs: vec![fragment(text, 10.0, 12.0)],
                baseline_y: 10.0,
                ascent: 10.0,
                descent: 2.0,
            },
            LineBox {
                y: 12.0,
                height: 16.0,
                runs: vec![fragment(other_text, 14.0, 16.0)],
                baseline_y: 14.0,
                ascent: 14.0,
                descent: 2.0,
            },
            LineBox {
                y: 28.0,
                height: 24.0,
                runs: vec![fragment(text, 20.0, 24.0)],
                baseline_y: 20.0,
                ascent: 20.0,
                descent: 4.0,
            },
        ];

        let mut legacy = LayoutBox::default();
        let mut optimized = LayoutBox::default();
        store_font_sizes_from_ifc_mode(&inline_ctx, &mut legacy, &doc, &styles, false);
        store_font_sizes_from_ifc_mode(&inline_ctx, &mut optimized, &doc, &styles, true);

        assert_eq!(optimized.text_node_font_sizes, legacy.text_node_font_sizes);
        assert_eq!(optimized.text_node_is_ahem, legacy.text_node_is_ahem);
        assert_eq!(optimized.text_node_letter_spacing, legacy.text_node_letter_spacing);
        assert_eq!(optimized.text_node_line_heights, legacy.text_node_line_heights);
        assert_eq!(optimized.text_node_text_transform, legacy.text_node_text_transform);
        assert_eq!(optimized.plaintext_bidi_nodes, legacy.plaintext_bidi_nodes);
        assert_eq!(optimized.text_node_font_families, legacy.text_node_font_families);
        assert_eq!(optimized.text_node_font_size_adjust, legacy.text_node_font_size_adjust);
        assert_eq!(optimized.inline_element_metrics, legacy.inline_element_metrics);
        assert_eq!(optimized.inline_element_margins, legacy.inline_element_margins);
    }
}
