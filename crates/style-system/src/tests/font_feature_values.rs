use super::super::*;
use super::helpers::make_test_dom;
use zero_css_parser::Parser as CssParser;

fn computed_features(css: &str) -> Vec<FontFeatureSetting> {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[CssParser::parse_stylesheet(css)]);
    styles
        .get(&div)
        .expect("div computed style")
        .font_variant_alternates_features
        .clone()
}

#[test]
fn resolves_all_alias_kinds_for_matching_family() {
    let features = computed_features(
        r#"
        @font-feature-values Demo {
            @stylistic { stylistic-name: 2; }
            @styleset { set-names: 1 3; }
            @character-variant { character-name: 4 7; }
            @swash { swash-name: 5; }
            @ornaments { ornament-name: 6; }
            @annotation { annotation-name: 8; }
        }
        div {
            font-family: Demo;
            font-variant-alternates: historical-forms stylistic(stylistic-name)
                styleset(set-names) character-variant(character-name)
                swash(swash-name) ornaments(ornament-name) annotation(annotation-name);
        }
        "#,
    );
    assert_eq!(
        features,
        vec![
            FontFeatureSetting {
                tag: *b"hist",
                value: 1,
            },
            FontFeatureSetting {
                tag: *b"salt",
                value: 2,
            },
            FontFeatureSetting {
                tag: *b"ss01",
                value: 1,
            },
            FontFeatureSetting {
                tag: *b"ss03",
                value: 1,
            },
            FontFeatureSetting {
                tag: *b"cv04",
                value: 7,
            },
            FontFeatureSetting {
                tag: *b"swsh",
                value: 5,
            },
            FontFeatureSetting {
                tag: *b"cswh",
                value: 5,
            },
            FontFeatureSetting {
                tag: *b"ornm",
                value: 6,
            },
            FontFeatureSetting {
                tag: *b"nalt",
                value: 8,
            },
        ]
    );
}

#[test]
fn aliases_are_family_scoped_and_unknown_names_are_ignored() {
    assert!(
        computed_features(
            r#"
        @font-feature-values Other { @stylistic { foo: 1; } }
        div { font-family: Demo; font-variant-alternates: stylistic(foo); }
        "#,
        )
        .is_empty()
    );
    assert!(
        computed_features(
            r#"
        @font-feature-values Demo { @stylistic { foo: 1; } }
        div { font-family: Demo; font-variant-alternates: stylistic(missing); }
        "#,
        )
        .is_empty()
    );
}

#[test]
fn later_rules_and_declared_layer_priority_override_aliases() {
    let features = computed_features(
        r#"
        @layer one, two, three;
        @layer three {
            @font-feature-values Demo { @styleset { foo: 1; bar: 1; } }
        }
        @layer one {
            @font-feature-values Demo { @styleset { foo: 2; bar: 2; baz: 2; } }
        }
        @layer two {
            @font-feature-values Demo { @styleset { baz: 3; } }
        }
        div { font-family: Demo; font-variant-alternates: styleset(foo, bar, baz); }
        "#,
    );
    assert_eq!(
        features,
        vec![
            FontFeatureSetting {
                tag: *b"ss01",
                value: 1,
            },
            FontFeatureSetting {
                tag: *b"ss03",
                value: 1,
            },
        ]
    );

    let overridden = computed_features(
        r#"
        @font-feature-values Demo { @ornaments { foo: 1; } }
        @font-feature-values Demo { @ornaments { foo: 3; } }
        div { font-family: Demo; font-variant-alternates: ornaments(foo); }
        "#,
    );
    assert_eq!(
        overridden,
        vec![FontFeatureSetting {
            tag: *b"ornm",
            value: 3,
        }]
    );
}
