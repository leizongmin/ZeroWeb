//! Fail-closed reuse policy for containers finalized by the post-order IFC pass.

use std::collections::HashMap;
use std::sync::LazyLock;

use zero_css_parser::values::FloatValue;
use zero_dom::{Document, NodeId};
use zero_style_system::property::types::LineClampComputedValue;
use zero_style_system::{ComputedStyle, FontSizeAdjustValue};

use crate::NodeIdSet;
use crate::types::LayoutBox;

pub(crate) fn can_skip_final_ifc(
    root: &LayoutBox,
    node_id: NodeId,
    style: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    finalized: &NodeIdSet,
    has_float_exclusions: bool,
) -> bool {
    static ENABLED: LazyLock<bool> =
        LazyLock::new(|| std::env::var("ZW_FINAL_IFC_REUSE_REMEASURED").as_deref() != Ok("0"));
    if !*ENABLED || !finalized.contains(&node_id) {
        return false;
    }

    const GENERIC_FAMILIES: [&str; 6] = ["sans-serif", "serif", "monospace", "cursive", "fantasy", "system-ui"];
    let generic_font = style.font_family.len() == 1
        && GENERIC_FAMILIES
            .iter()
            .any(|generic| style.font_family[0].trim_matches('"').eq_ignore_ascii_case(generic));
    let orphan_candidates = doc
        .child_nodes(node_id)
        .into_iter()
        .filter(|child| crate::tree::phasea_multi_inline_eligible(doc, styles, *child))
        .take(2)
        .count();

    reuse_policy(
        true,
        true,
        !has_float_exclusions && matches!(style.float, FloatValue::None),
        !root.is_multicol,
        matches!(style.line_clamp, LineClampComputedValue::None),
        matches!(style.font_size_adjust, FontSizeAdjustValue::None),
        generic_font,
        orphan_candidates < 2,
        root.inline_layout.is_none(),
        !root.text_node_font_sizes.is_empty() && !root.text_node_line_heights.is_empty(),
    )
}

#[allow(clippy::too_many_arguments)]
fn reuse_policy(
    enabled: bool,
    finalized: bool,
    no_floats: bool,
    no_multicol: bool,
    no_line_clamp: bool,
    no_size_adjust: bool,
    generic_font: bool,
    no_orphan_backfill: bool,
    no_stored_layout: bool,
    has_metrics: bool,
) -> bool {
    enabled
        && finalized
        && no_floats
        && no_multicol
        && no_line_clamp
        && no_size_adjust
        && generic_font
        && no_orphan_backfill
        && no_stored_layout
        && has_metrics
}

#[cfg(test)]
mod tests {
    use super::reuse_policy;

    #[test]
    fn final_ifc_reuse_is_fail_closed() {
        assert!(reuse_policy(true, true, true, true, true, true, true, true, true, true));
        for rejected in 0..10 {
            let mut conditions = [true; 10];
            conditions[rejected] = false;
            assert!(!reuse_policy(
                conditions[0],
                conditions[1],
                conditions[2],
                conditions[3],
                conditions[4],
                conditions[5],
                conditions[6],
                conditions[7],
                conditions[8],
                conditions[9],
            ));
        }
    }
}
