//! Shared CSS font-family to ordered face ID resolution.

use std::collections::HashMap;

use zero_css_parser::values::FontWeightValue;
use zero_css_parser::values::types::FontStyleValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

fn lookup_face(resolver: &HashMap<String, u32>, key: &str) -> Option<u32> {
    resolver.get(key).copied().or_else(|| {
        resolver
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, id)| *id)
    })
}

/// Resolves one available face per CSS family while preserving declaration order.
///
/// Weight and style variants use the same fallback order as the painter. If no
/// declared family resolves, bold text falls back to `sans-serif:700`; all
/// other text uses face ID `0`.
pub fn resolve_font_ids_for_style(
    resolver: &HashMap<String, u32>,
    font_family: &[String],
    font_weight: &FontWeightValue,
    font_style: &FontStyleValue,
) -> Vec<u32> {
    let want_bold = matches!(font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
        || matches!(font_weight, FontWeightValue::Absolute(weight) if *weight >= 600);
    let want_italic = matches!(font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
    let suffixes: &[&str] = match (want_bold, want_italic) {
        (true, true) => &[":700:italic", ":700", ":italic", ""],
        (true, false) => &[":700", ""],
        (false, true) => &[":italic", ""],
        (false, false) => &[""],
    };

    // https://drafts.csswg.org/css-fonts-4/#family-name-value
    const GENERIC_FAMILIES: &[&str] = &[
        "serif", "sans-serif", "monospace", "cursive", "fantasy",
        "system-ui", "ui-serif", "ui-sans-serif", "ui-monospace", "ui-rounded",
        "emoji", "math", "fangsong",
    ];
    let mut ids = Vec::new();
    for family in font_family {
        let is_quoted = family.starts_with('"') || family.starts_with('\'');
        let name = family.trim_matches('"').trim_matches('\'');
        // R3249：quoted generic names 是自定义字体名，不匹配 generic resolver。
        if is_quoted && GENERIC_FAMILIES.iter().any(|g| g.eq_ignore_ascii_case(name)) {
            continue;
        }
        let id = suffixes
            .iter()
            .find_map(|suffix| lookup_face(resolver, &format!("{name}{suffix}")));
        if let Some(id) = id
            && !ids.contains(&id)
        {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        ids.push(
            want_bold
                .then(|| lookup_face(resolver, "sans-serif:700"))
                .flatten()
                .unwrap_or(0),
        );
    }
    ids
}

pub(crate) struct FontOverrides {
    pub(crate) ids: HashMap<NodeId, Vec<u32>>,
    pub(crate) size_adjust: HashMap<NodeId, zero_style_system::FontSizeAdjustValue>,
}

pub(crate) fn collect_font_overrides(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    root: NodeId,
    resolver: &HashMap<String, u32>,
) -> FontOverrides {
    fn visit(
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        node_id: NodeId,
        resolver: &HashMap<String, u32>,
        overrides: &mut FontOverrides,
    ) {
        let style_id = doc
            .get(node_id)
            .and_then(|node| {
                matches!(node.kind, NodeKind::Text(_))
                    .then(|| doc.parent_node(node_id))
                    .flatten()
            })
            .unwrap_or(node_id);
        if let Some(style) = styles.get(&style_id) {
            overrides.ids.insert(
                node_id,
                resolve_font_ids_for_style(resolver, &style.font_family, &style.font_weight, &style.font_style),
            );
            overrides.size_adjust.insert(node_id, style.font_size_adjust);
        }
        for child in doc.child_nodes(node_id) {
            visit(doc, styles, child, resolver, overrides);
        }
    }

    let mut overrides = FontOverrides {
        ids: HashMap::new(),
        size_adjust: HashMap::new(),
    };
    visit(doc, styles, root, resolver, &mut overrides);
    overrides
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_family_order_and_resolves_face_variants_case_insensitively() {
        let resolver = HashMap::from([("Primary:700:italic".to_string(), 7), ("secondary:700".to_string(), 9)]);

        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["primary".to_string(), "Secondary".to_string()],
                &FontWeightValue::Bold,
                &FontStyleValue::Italic,
            ),
            vec![7, 9]
        );
    }

    #[test]
    fn deduplicates_faces_and_uses_bold_generic_fallback() {
        let resolver = HashMap::from([
            ("First".to_string(), 3),
            ("Second".to_string(), 3),
            ("sans-serif:700".to_string(), 5),
        ]);

        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["First".to_string(), "Second".to_string()],
                &FontWeightValue::Normal,
                &FontStyleValue::Normal,
            ),
            vec![3]
        );
        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["Missing".to_string()],
                &FontWeightValue::Bold,
                &FontStyleValue::Normal,
            ),
            vec![5]
        );
    }

    #[test]
    fn collects_ordered_faces_for_element_and_text_run_ids() {
        let mut doc = Document::new();
        let root = doc.create_element("div");
        doc.append_child(doc.root(), root).unwrap();
        let text = doc.create_text_node("xA");
        doc.append_child(root, text).unwrap();

        let mut style = ComputedStyle::default();
        style.font_family = vec!["Primary".to_string(), "Secondary".to_string()];
        let styles = HashMap::from([(root, style)]);
        let resolver = HashMap::from([("Primary".to_string(), 7), ("Secondary".to_string(), 9)]);

        let overrides = collect_font_overrides(&doc, &styles, root, &resolver);
        assert_eq!(overrides.ids.get(&root), Some(&vec![7, 9]));
        assert_eq!(overrides.ids.get(&text), Some(&vec![7, 9]));
        assert_eq!(
            overrides.size_adjust.get(&text),
            Some(&zero_style_system::FontSizeAdjustValue::None)
        );
    }
}
