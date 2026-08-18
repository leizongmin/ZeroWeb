//! Inline-content predicates shared by layout postprocessing passes.

use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

pub(crate) fn has_direct_text(doc: &Document, dom_id: NodeId) -> bool {
    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}

/// Checks for direct text or inline-level element children.
pub(crate) fn has_inline_content(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    if has_direct_text(doc, dom_id) {
        return true;
    }

    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            (doc.get(*child_id), styles.get(child_id)),
            (Some(node), Some(style))
                if matches!(&node.kind, NodeKind::Element(_))
                    && matches!(style.display, DisplayValue::Inline | DisplayValue::InlineBlock)
        )
    })
}
