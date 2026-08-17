use std::borrow::Cow;
use std::collections::HashMap;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

pub(super) fn computed_style_for_layout(
    styles: &HashMap<NodeId, ComputedStyle>,
    node_id: NodeId,
) -> Cow<'_, ComputedStyle> {
    static BORROW_ENABLED: std::sync::LazyLock<bool> =
        std::sync::LazyLock::new(|| std::env::var("ZW_LAYOUT_STYLE_BORROW").as_deref() != Ok("0"));
    computed_style_for_layout_with_mode(styles, node_id, *BORROW_ENABLED)
}

fn computed_style_for_layout_with_mode(
    styles: &HashMap<NodeId, ComputedStyle>,
    node_id: NodeId,
    borrow_enabled: bool,
) -> Cow<'_, ComputedStyle> {
    if borrow_enabled {
        styles
            .get(&node_id)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(ComputedStyle::default()))
    } else {
        Cow::Owned(styles.get(&node_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::Document;

    #[test]
    fn borrows_present_styles_and_owns_fallbacks() {
        let mut doc = Document::new();
        let node_id = doc.create_element("div");
        let styles = HashMap::from([(node_id, ComputedStyle::default())]);

        assert!(matches!(
            computed_style_for_layout_with_mode(&styles, node_id, true),
            Cow::Borrowed(_)
        ));
        assert!(matches!(
            computed_style_for_layout_with_mode(&styles, node_id, false),
            Cow::Owned(_)
        ));

        let missing_id = doc.create_element("span");
        assert!(matches!(
            computed_style_for_layout_with_mode(&styles, missing_id, true),
            Cow::Owned(_)
        ));
    }
}
