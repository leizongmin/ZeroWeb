//! Shared CSS font-family to ordered face ID resolution.

use std::collections::HashMap;

use zero_css_parser::values::FontWeightValue;
use zero_css_parser::values::types::FontStyleValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_render_foundation::font::{resolve_font_face, resolve_font_faces};
use zero_style_system::ComputedStyle;

const GENERIC_FAMILIES: &[&str] = &[
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "emoji",
    "math",
    "fangsong",
];

fn is_generic_family_name(name: &str) -> bool {
    GENERIC_FAMILIES
        .iter()
        .any(|generic| generic.eq_ignore_ascii_case(name))
}

/// Resolves one available face per CSS family while preserving declaration order.
///
/// Weight and style variants use the same fallback order as the painter. If no
/// declared family resolves, use the available `sans-serif` face before the
/// final no-font sentinel.
pub fn resolve_font_ids_for_style(
    resolver: &HashMap<String, u32>,
    font_family: &[String],
    font_weight: &FontWeightValue,
    font_style: &FontStyleValue,
    font_stretch: f32,
) -> Vec<u32> {
    let want_bold = matches!(font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
        || matches!(font_weight, FontWeightValue::Absolute(weight) if *weight >= 600);
    let want_italic = matches!(font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
    // https://drafts.csswg.org/css-fonts-4/#family-name-value
    let mut ids = Vec::new();
    for family in font_family {
        let is_quoted = family.starts_with('"') || family.starts_with('\'');
        let name = family.trim_matches('"').trim_matches('\'');
        // R3249：quoted generic names 是自定义字体名，不匹配 generic resolver。
        if is_quoted && is_generic_family_name(name) {
            continue;
        }
        if let Some((faces, _)) = resolve_font_faces(resolver, name, want_bold, want_italic, font_stretch) {
            for id in faces {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    if ids.is_empty() {
        ids.push(
            resolve_font_face(resolver, "sans-serif", want_bold, want_italic, font_stretch)
                .map(|face| face.0)
                .unwrap_or(0),
        );
    }
    ids
}

/// Resolves the first available non-generic family for scoped author-font shaping.
///
/// https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
pub(crate) fn resolve_author_font_id_for_style(resolver: &HashMap<String, u32>, style: &ComputedStyle) -> Option<u32> {
    let want_bold = matches!(style.font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
        || matches!(style.font_weight, FontWeightValue::Absolute(weight) if weight >= 600);
    let want_italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
    for family in &style.font_family {
        let name = family.trim_matches('"').trim_matches('\'');
        if is_generic_family_name(name) {
            continue;
        }
        if let Some((faces, _)) = resolve_font_faces(resolver, name, want_bold, want_italic, style.font_stretch) {
            return faces.first().copied();
        }
    }
    None
}

pub(crate) struct FontOverrides {
    pub(crate) ids: std::rc::Rc<HashMap<NodeId, Vec<u32>>>,
    pub(crate) size_adjust: std::rc::Rc<HashMap<NodeId, zero_style_system::FontSizeAdjustValue>>,
    pub(crate) variations: std::rc::Rc<HashMap<NodeId, zero_style_system::FontVariationSettingsValue>>,
}

pub(crate) fn collect_font_overrides(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    root: NodeId,
    resolver: &HashMap<String, u32>,
) -> FontOverrides {
    // OPTIMIZATION: 页面内 font-family/weight/style 组合远少于文本节点数
    // （morning fixture ~1500 节点 vs ~10 组合）——按组合 memo 解析结果，避免
    // 每节点重复 resolve_font_ids_for_style 的 format! 字符串分配（全文档 collect
    // 实测 8ms → <1ms；R3424-F 默认开启 advance source 后该 collect 每 pass 多次）。
    #[derive(PartialEq, Eq, Hash)]
    struct ResolveKey {
        families: Vec<String>,
        bold: bool,
        italic: bool,
        stretch_bits: u32,
    }
    fn visit(
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
        node_id: NodeId,
        resolver: &HashMap<String, u32>,
        collector: &mut Collector,
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
            let bold = matches!(style.font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
                || matches!(style.font_weight, FontWeightValue::Absolute(weight) if weight >= 600);
            let italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
            let key = ResolveKey {
                families: style.font_family.clone(),
                bold,
                italic,
                stretch_bits: style.font_stretch.to_bits(),
            };
            let ids_resolved = collector
                .resolve_cache
                .entry(key)
                .or_insert_with(|| {
                    resolve_font_ids_for_style(
                        resolver,
                        &style.font_family,
                        &style.font_weight,
                        &style.font_style,
                        style.font_stretch,
                    )
                })
                .clone();
            collector.ids.insert(node_id, ids_resolved);
            collector.size_adjust.insert(node_id, style.font_size_adjust);
            collector
                .variations
                .insert(node_id, style.font_variation_settings.clone());
        }
        for child in doc.child_nodes(node_id) {
            visit(doc, styles, child, resolver, collector);
        }
    }

    struct Collector {
        ids: HashMap<NodeId, Vec<u32>>,
        size_adjust: HashMap<NodeId, zero_style_system::FontSizeAdjustValue>,
        variations: HashMap<NodeId, zero_style_system::FontVariationSettingsValue>,
        resolve_cache: HashMap<ResolveKey, Vec<u32>>,
    }
    let mut collector = Collector {
        ids: HashMap::new(),
        size_adjust: HashMap::new(),
        variations: HashMap::new(),
        resolve_cache: HashMap::new(),
    };
    visit(doc, styles, root, resolver, &mut collector);
    FontOverrides {
        ids: std::rc::Rc::new(collector.ids),
        size_adjust: std::rc::Rc::new(collector.size_adjust),
        variations: std::rc::Rc::new(collector.variations),
    }
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
                100.0,
            ),
            vec![7, 9]
        );
    }

    #[test]
    fn preserves_all_faces_for_one_matched_family_variant() {
        let resolver = HashMap::from([
            ("Split:stretch=1000".to_string(), 7),
            ("Split:stretch=1000:face=0".to_string(), 7),
            ("Split:stretch=1000:face=1".to_string(), 9),
        ]);
        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["split".to_string()],
                &FontWeightValue::Normal,
                &FontStyleValue::Normal,
                100.0,
            ),
            vec![7, 9]
        );
    }

    #[test]
    fn deduplicates_faces_and_uses_generic_fallback() {
        let resolver = HashMap::from([
            ("First".to_string(), 3),
            ("Second".to_string(), 3),
            ("sans-serif".to_string(), 4),
            ("sans-serif:700".to_string(), 5),
        ]);

        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["First".to_string(), "Second".to_string()],
                &FontWeightValue::Normal,
                &FontStyleValue::Normal,
                100.0,
            ),
            vec![3]
        );
        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["Missing".to_string()],
                &FontWeightValue::Normal,
                &FontStyleValue::Normal,
                100.0,
            ),
            vec![4]
        );
        assert_eq!(
            resolve_font_ids_for_style(
                &resolver,
                &["Missing".to_string()],
                &FontWeightValue::Bold,
                &FontStyleValue::Normal,
                100.0,
            ),
            vec![5]
        );
    }

    #[test]
    fn resolves_only_explicit_author_face_for_scoped_layout_shaping() {
        let resolver = HashMap::from([("Custom".to_string(), 7), ("sans-serif".to_string(), 0)]);
        let mut author = ComputedStyle::default();
        author.font_family = vec!["Custom".to_string(), "sans-serif".to_string()];
        assert_eq!(resolve_author_font_id_for_style(&resolver, &author), Some(7));

        let mut generic = ComputedStyle::default();
        generic.font_family = vec!["sans-serif".to_string()];
        assert_eq!(resolve_author_font_id_for_style(&resolver, &generic), None);
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
        assert_eq!(
            overrides.variations.get(&text),
            Some(&zero_style_system::FontVariationSettingsValue::Normal)
        );
    }
}
