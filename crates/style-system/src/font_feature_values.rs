//! `@font-feature-values` family-scoped alias resolution.

use crate::{ComputedStyle, FontFeatureSetting};
use std::collections::HashMap;
use zero_css_parser::Stylesheet;
use zero_css_parser::ast::{FontFeatureValueKind, Rule};
use zero_css_parser::values::FontVariantAlternatesValue;

type FontFeatureAliasKey = (FontFeatureValueKind, String);

/// 按 family 索引的已级联合并 alias 表。
pub(crate) type FontFeatureValuesRegistry = HashMap<String, HashMap<FontFeatureAliasKey, Vec<u32>>>;

/// 扫描并按 family / cascade layer / 源序合并 `@font-feature-values` alias。
///
/// 无层规则优先于有层规则；有层规则按声明层顺序后层优先；同优先级后声明覆盖。
pub(crate) fn collect_font_feature_values(stylesheets: &[Stylesheet]) -> FontFeatureValuesRegistry {
    if std::env::var("ZW_FONT_FEATURE_VALUES").as_deref() == Ok("0") {
        return HashMap::new();
    }
    let mut layer_order = Vec::<String>::new();
    for stylesheet in stylesheets {
        collect_layer_names(&stylesheet.rules, &mut layer_order);
    }
    let layer_priority: HashMap<String, usize> = layer_order
        .iter()
        .enumerate()
        .map(|(index, name)| (name.to_ascii_lowercase(), index))
        .collect();
    let unlayered_priority = layer_order.len();

    type Candidate = (usize, usize, Vec<u32>);
    let mut candidates: HashMap<(String, FontFeatureAliasKey), Candidate> = HashMap::new();
    let mut source_order = 0usize;
    for stylesheet in stylesheets {
        collect_candidates(
            &stylesheet.rules,
            None,
            &layer_priority,
            unlayered_priority,
            &mut source_order,
            &mut candidates,
        );
    }

    let mut registry = FontFeatureValuesRegistry::new();
    for ((family, key), (_, _, values)) in candidates {
        registry.entry(family).or_default().insert(key, values);
    }
    registry
}

fn collect_layer_names(rules: &[Rule], order: &mut Vec<String>) {
    for rule in rules {
        if let Rule::Layer(layer) = rule {
            for name in layer.name.split(',').map(str::trim).filter(|name| !name.is_empty()) {
                if !order.iter().any(|existing| existing.eq_ignore_ascii_case(name)) {
                    order.push(name.to_string());
                }
            }
            collect_layer_names(&layer.rules, order);
        }
    }
}

fn collect_candidates(
    rules: &[Rule],
    layer: Option<&str>,
    layer_priority: &HashMap<String, usize>,
    unlayered_priority: usize,
    source_order: &mut usize,
    candidates: &mut HashMap<(String, FontFeatureAliasKey), (usize, usize, Vec<u32>)>,
) {
    for rule in rules {
        match rule {
            Rule::FontFeatureValues(feature_values) => {
                let priority = layer
                    .and_then(|name| layer_priority.get(&name.to_ascii_lowercase()).copied())
                    .unwrap_or(unlayered_priority);
                for family in &feature_values.families {
                    for definition in &feature_values.definitions {
                        *source_order += 1;
                        let key = (family.to_ascii_lowercase(), (definition.kind, definition.name.clone()));
                        let should_replace = candidates.get(&key).is_none_or(|(old_priority, old_order, _)| {
                            priority > *old_priority || (priority == *old_priority && *source_order > *old_order)
                        });
                        if should_replace {
                            candidates.insert(key, (priority, *source_order, definition.values.clone()));
                        }
                    }
                }
            }
            Rule::Layer(layer_rule) if !layer_rule.rules.is_empty() => {
                collect_candidates(
                    &layer_rule.rules,
                    Some(&layer_rule.name),
                    layer_priority,
                    unlayered_priority,
                    source_order,
                    candidates,
                );
            }
            _ => {}
        }
    }
}

/// 按 computed font-family 将 alias 解析成最终 OpenType feature。
pub(crate) fn resolve_font_variant_alternates(style: &mut ComputedStyle, registry: &FontFeatureValuesRegistry) {
    let FontVariantAlternatesValue::Values(alternates) = &style.font_variant_alternates else {
        style.font_variant_alternates_features.clear();
        return;
    };
    let mut features = Vec::new();
    if alternates.historical_forms {
        features.push(FontFeatureSetting {
            tag: *b"hist",
            value: 1,
        });
    }

    let aliases = style.font_family.iter().find_map(|family| {
        let name = family.trim_matches('"').trim_matches('\'').to_ascii_lowercase();
        registry.get(&name)
    });
    let Some(aliases) = aliases else {
        style.font_variant_alternates_features = features;
        return;
    };
    let lookup = |kind, name: &str| aliases.get(&(kind, name.to_string()));
    let mut push = |tag, value| {
        if let Some(existing) = features.iter_mut().find(|feature| feature.tag == tag) {
            existing.value = value;
        } else {
            features.push(FontFeatureSetting { tag, value });
        }
    };

    if let Some(name) = &alternates.stylistic
        && let Some(values) = lookup(FontFeatureValueKind::Stylistic, name)
    {
        push(*b"salt", values[0]);
    }
    for name in &alternates.styleset {
        if let Some(values) = lookup(FontFeatureValueKind::Styleset, name) {
            for value in values {
                if let Some(tag) = indexed_feature_tag(b"ss", *value) {
                    push(tag, 1);
                }
            }
        }
    }
    if let Some(name) = &alternates.character_variant
        && let Some(values) = lookup(FontFeatureValueKind::CharacterVariant, name)
        && let Some(tag) = indexed_feature_tag(b"cv", values[0])
    {
        push(tag, values.get(1).copied().unwrap_or(1));
    }
    if let Some(name) = &alternates.swash
        && let Some(values) = lookup(FontFeatureValueKind::Swash, name)
    {
        push(*b"swsh", values[0]);
        push(*b"cswh", values[0]);
    }
    if let Some(name) = &alternates.ornaments
        && let Some(values) = lookup(FontFeatureValueKind::Ornaments, name)
    {
        push(*b"ornm", values[0]);
    }
    if let Some(name) = &alternates.annotation
        && let Some(values) = lookup(FontFeatureValueKind::Annotation, name)
    {
        push(*b"nalt", values[0]);
    }
    style.font_variant_alternates_features = features;
}

fn indexed_feature_tag(prefix: &[u8; 2], value: u32) -> Option<[u8; 4]> {
    (1..=99).contains(&value).then(|| {
        [
            prefix[0],
            prefix[1],
            b'0' + (value / 10) as u8,
            b'0' + (value % 10) as u8,
        ]
    })
}
