use crate::ast::{FontFeatureValueKind, Rule};
use crate::parser::Parser;
use crate::values::{FontVariantAlternatesValue, parse_font_variant_alternates};

#[test]
fn parses_all_font_feature_value_kinds_and_families() {
    let stylesheet = Parser::parse_stylesheet(
        r#"
        @font-feature-values "Demo One", Demo Two {
            @stylistic { salt: 2; }
            @styleset { sets: 1 3; }
            @character-variant { variant: 4 7; }
            @swash { swash: 5; }
            @ornaments { ornament: 6; }
            @annotation { note: 8; }
        }
        "#,
    );
    let Rule::FontFeatureValues(rule) = &stylesheet.rules[0] else {
        panic!("expected @font-feature-values");
    };
    assert_eq!(rule.families, ["Demo One", "Demo Two"]);
    assert_eq!(rule.definitions.len(), 6);
    assert_eq!(rule.definitions[0].kind, FontFeatureValueKind::Stylistic);
    assert_eq!(rule.definitions[1].values, [1, 3]);
    assert_eq!(rule.definitions[2].values, [4, 7]);
    assert_eq!(rule.definitions[5].kind, FontFeatureValueKind::Annotation);
}

#[test]
fn parses_font_variant_alternates_combination_and_rejects_duplicates() {
    let parsed = parse_font_variant_alternates(
        "historical-forms stylistic(foo) styleset(one, two) character-variant(cv) swash(sw) ornaments(or) annotation(an)",
    )
    .expect("valid combination");
    let FontVariantAlternatesValue::Values(values) = parsed else {
        panic!("expected values");
    };
    assert!(values.historical_forms);
    assert_eq!(values.stylistic.as_deref(), Some("foo"));
    assert_eq!(values.styleset, ["one", "two"]);
    assert_eq!(values.character_variant.as_deref(), Some("cv"));
    assert_eq!(values.swash.as_deref(), Some("sw"));
    assert_eq!(values.ornaments.as_deref(), Some("or"));
    assert_eq!(values.annotation.as_deref(), Some("an"));
    assert!(parse_font_variant_alternates("stylistic(one) stylistic(two)").is_none());
    assert!(parse_font_variant_alternates("stylistic(foo,)").is_none());
    assert!(parse_font_variant_alternates("styleset(foo,)").is_none());
    assert!(parse_font_variant_alternates("styleset(foo,,bar)").is_none());
}

#[test]
fn preserves_comma_separated_layer_order_statement() {
    let stylesheet = Parser::parse_stylesheet("@layer one, two, three;");
    let Rule::Layer(layer) = &stylesheet.rules[0] else {
        panic!("expected layer rule");
    };
    assert_eq!(layer.name, "one,two,three");
    assert!(layer.rules.is_empty());
}
