// Auto-generated test file — split from property.rs
use super::super::*;

#[test]
/// page-break-before 应用各值
fn test_apply_page_break_before() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "page-break-before", "always"));
    assert_eq!(style.page_break_before, PageBreakValue::Always);

    assert!(apply_property_value(&mut style, "page-break-before", "avoid"));
    assert_eq!(style.page_break_before, PageBreakValue::Avoid);

    assert!(apply_property_value(&mut style, "page-break-before", "left"));
    assert_eq!(style.page_break_before, PageBreakValue::Left);

    assert!(apply_property_value(&mut style, "page-break-before", "right"));
    assert_eq!(style.page_break_before, PageBreakValue::Right);

    assert!(apply_property_value(&mut style, "page-break-before", "auto"));
    assert_eq!(style.page_break_before, PageBreakValue::Auto);

    assert!(!apply_property_value(&mut style, "page-break-before", "invalid"));
}

#[test]
/// page-break-after 应用各值
fn test_apply_page_break_after() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "page-break-after", "always"));
    assert_eq!(style.page_break_after, PageBreakValue::Always);
}

#[test]
/// page-break-inside 仅接受 auto/avoid
fn test_apply_page_break_inside() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "page-break-inside", "avoid"));
    assert_eq!(style.page_break_inside, PageBreakValue::Avoid);

    assert!(apply_property_value(&mut style, "page-break-inside", "auto"));
    assert_eq!(style.page_break_inside, PageBreakValue::Auto);

    // always/left/right 对 page-break-inside 无效
    assert!(!apply_property_value(&mut style, "page-break-inside", "always"));
}

#[test]
/// box-decoration-break 应用 slice/clone
fn test_apply_box_decoration_break() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "box-decoration-break", "clone"));
    assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Clone);

    assert!(apply_property_value(&mut style, "box-decoration-break", "slice"));
    assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);

    assert!(!apply_property_value(&mut style, "box-decoration-break", "invalid"));
}

#[test]
/// image-rendering 应用各值
fn test_apply_image_rendering() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "image-rendering", "pixelated"));
    assert_eq!(style.image_rendering, ImageRenderingValue::Pixelated);

    assert!(apply_property_value(&mut style, "image-rendering", "crisp-edges"));
    assert_eq!(style.image_rendering, ImageRenderingValue::CrispEdges);

    assert!(apply_property_value(&mut style, "image-rendering", "smooth"));
    assert_eq!(style.image_rendering, ImageRenderingValue::Smooth);

    assert!(apply_property_value(&mut style, "image-rendering", "high-quality"));
    assert_eq!(style.image_rendering, ImageRenderingValue::HighQuality);

    assert!(!apply_property_value(&mut style, "image-rendering", "invalid"));
}

#[test]
/// isolation 应用 auto/isolate
fn test_apply_isolation() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "isolation", "isolate"));
    assert_eq!(style.isolation, IsolationValue::Isolate);

    assert!(apply_property_value(&mut style, "isolation", "auto"));
    assert_eq!(style.isolation, IsolationValue::Auto);

    assert!(!apply_property_value(&mut style, "isolation", "invalid"));
}

#[test]
/// 新属性不在继承列表中
fn test_new_properties_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("page-break-before"));
    assert!(!PropertyRegistry::is_inherited("page-break-after"));
    assert!(!PropertyRegistry::is_inherited("page-break-inside"));
    assert!(!PropertyRegistry::is_inherited("box-decoration-break"));
    assert!(!PropertyRegistry::is_inherited("image-rendering"));
    assert!(!PropertyRegistry::is_inherited("isolation"));
}

#[test]
/// 新属性在 known_properties 中注册
fn test_new_properties_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"page-break-before"));
    assert!(props.contains(&"page-break-after"));
    assert!(props.contains(&"page-break-inside"));
    assert!(props.contains(&"box-decoration-break"));
    assert!(props.contains(&"image-rendering"));
    assert!(props.contains(&"isolation"));
}

#[test]
/// 新属性的 initial_value 存在
fn test_new_properties_initial_values() {
    assert!(PropertyRegistry::initial_value("page-break-before").is_some());
    assert!(PropertyRegistry::initial_value("page-break-after").is_some());
    assert!(PropertyRegistry::initial_value("page-break-inside").is_some());
    assert!(PropertyRegistry::initial_value("box-decoration-break").is_some());
    assert!(PropertyRegistry::initial_value("image-rendering").is_some());
    assert!(PropertyRegistry::initial_value("isolation").is_some());
}

#[test]
/// apply_initial_value 对新属性
fn test_apply_initial_value_new_round5_properties() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "page-break-before", "always");
    apply_property_value(&mut style, "page-break-after", "avoid");
    apply_property_value(&mut style, "box-decoration-break", "clone");
    apply_property_value(&mut style, "image-rendering", "pixelated");
    apply_property_value(&mut style, "isolation", "isolate");

    assert!(apply_initial_value(&mut style, "page-break-before"));
    assert_eq!(style.page_break_before, PageBreakValue::Auto);

    assert!(apply_initial_value(&mut style, "page-break-after"));
    assert_eq!(style.page_break_after, PageBreakValue::Auto);

    assert!(apply_initial_value(&mut style, "box-decoration-break"));
    assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);

    assert!(apply_initial_value(&mut style, "image-rendering"));
    assert_eq!(style.image_rendering, ImageRenderingValue::Auto);

    assert!(apply_initial_value(&mut style, "isolation"));
    assert_eq!(style.isolation, IsolationValue::Auto);
}

// ── Interaction / Performance Hint 属性测试 ──

#[test]
/// overscroll-behavior-x/y 默认值为 Auto
fn test_overscroll_behavior_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.overscroll_behavior_x, OverscrollBehaviorValue::Auto);
    assert_eq!(style.overscroll_behavior_y, OverscrollBehaviorValue::Auto);
}

#[test]
/// overscroll-behavior-x/y apply
fn test_overscroll_behavior_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "overscroll-behavior-x", "contain"));
    assert_eq!(style.overscroll_behavior_x, OverscrollBehaviorValue::Contain);
    assert!(apply_property_value(&mut style, "overscroll-behavior-y", "none"));
    assert_eq!(style.overscroll_behavior_y, OverscrollBehaviorValue::None);
    // 无效值
    assert!(!apply_property_value(&mut style, "overscroll-behavior-x", "invalid"));
}

#[test]
/// touch-action apply
fn test_touch_action_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "touch-action", "none"));
    assert_eq!(style.touch_action, TouchActionValue::None);
    assert!(apply_property_value(&mut style, "touch-action", "pan-x"));
    assert_eq!(style.touch_action, TouchActionValue::PanX);
    assert!(apply_property_value(&mut style, "touch-action", "manipulation"));
    assert_eq!(style.touch_action, TouchActionValue::Manipulation);
    assert!(!apply_property_value(&mut style, "touch-action", "invalid"));
}

#[test]
/// user-select apply
fn test_user_select_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "user-select", "text"));
    assert_eq!(style.user_select, UserSelectValue::Text);
    assert!(apply_property_value(&mut style, "user-select", "none"));
    assert_eq!(style.user_select, UserSelectValue::None);
    assert!(apply_property_value(&mut style, "user-select", "all"));
    assert_eq!(style.user_select, UserSelectValue::All);
}

#[test]
/// will-change apply（R2308：`auto | scroll-position | contents | <custom-ident>+`，空格或逗号分隔多 ident）
fn test_will_change_apply() {
    let mut style = ComputedStyle::default();
    // auto = 空 Vec（默认值）
    assert!(apply_property_value(&mut style, "will-change", "auto"));
    assert_eq!(style.will_change, Vec::<WillChangeValue>::new());
    // 单 ident
    assert!(apply_property_value(&mut style, "will-change", "scroll-position"));
    assert_eq!(style.will_change, vec![WillChangeValue::ScrollPosition]);
    assert!(apply_property_value(&mut style, "will-change", "transform"));
    assert_eq!(
        style.will_change,
        vec![WillChangeValue::Custom("transform".to_string())]
    );
    // R2308：多 ident 空格分隔（CSS Will Change 规范 `<custom-ident>+`）
    assert!(apply_property_value(&mut style, "will-change", "transform opacity"));
    assert_eq!(
        style.will_change,
        vec![
            WillChangeValue::Custom("transform".to_string()),
            WillChangeValue::Custom("opacity".to_string()),
        ]
    );
    // R2308：多 ident 逗号分隔（容忍 `will-change: transform, opacity` 写法）
    assert!(apply_property_value(
        &mut style,
        "will-change",
        "scroll-position, contents"
    ));
    assert_eq!(
        style.will_change,
        vec![WillChangeValue::ScrollPosition, WillChangeValue::Contents]
    );
}

#[test]
/// pointer-events apply 和继承
fn test_pointer_events_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "pointer-events", "none"));
    assert_eq!(style.pointer_events, PointerEventsValue::None);
    assert!(apply_property_value(&mut style, "pointer-events", "visiblePainted"));
    assert_eq!(style.pointer_events, PointerEventsValue::VisiblePainted);
    // 继承性
    assert!(PropertyRegistry::is_inherited("pointer-events"));
}

#[test]
/// 新属性不在继承列表中（除 pointer-events）
fn test_interaction_properties_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("overscroll-behavior-x"));
    assert!(!PropertyRegistry::is_inherited("overscroll-behavior-y"));
    assert!(!PropertyRegistry::is_inherited("touch-action"));
    assert!(!PropertyRegistry::is_inherited("user-select"));
    assert!(!PropertyRegistry::is_inherited("will-change"));
}

#[test]
/// 新属性在 known_properties 中注册
fn test_interaction_properties_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"overscroll-behavior-x"));
    assert!(props.contains(&"overscroll-behavior-y"));
    assert!(props.contains(&"touch-action"));
    assert!(props.contains(&"user-select"));
    assert!(props.contains(&"will-change"));
    assert!(props.contains(&"pointer-events"));
}

#[test]
/// 新属性的 initial_value 存在
fn test_interaction_properties_initial_values() {
    assert!(PropertyRegistry::initial_value("overscroll-behavior-x").is_some());
    assert!(PropertyRegistry::initial_value("overscroll-behavior-y").is_some());
    assert!(PropertyRegistry::initial_value("touch-action").is_some());
    assert!(PropertyRegistry::initial_value("user-select").is_some());
    assert!(PropertyRegistry::initial_value("will-change").is_some());
    assert!(PropertyRegistry::initial_value("pointer-events").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// overflow-wrap / text-align-last / font-variant-numeric 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 overflow-wrap apply_property_value
fn test_apply_overflow_wrap() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);

    assert!(apply_property_value(&mut style, "overflow-wrap", "break-word"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::BreakWord);

    assert!(apply_property_value(&mut style, "overflow-wrap", "anywhere"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::Anywhere);

    assert!(apply_property_value(&mut style, "overflow-wrap", "normal"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);

    assert!(!apply_property_value(&mut style, "overflow-wrap", "invalid"));
}

#[test]
/// 测试 overflow-wrap 继承性
fn test_overflow_wrap_inherited() {
    assert!(PropertyRegistry::is_inherited("overflow-wrap"));
}

#[test]
/// 测试 overflow-wrap initial_value
fn test_overflow_wrap_initial_value() {
    assert!(PropertyRegistry::initial_value("overflow-wrap").is_some());
    let mut style = ComputedStyle::default();
    style.overflow_wrap = OverflowWrapValue::BreakWord;
    assert!(apply_initial_value(&mut style, "overflow-wrap"));
    assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);
}

#[test]
/// 测试 overflow-wrap 继承
fn test_overflow_wrap_inherit() {
    let mut parent = ComputedStyle::default();
    parent.overflow_wrap = OverflowWrapValue::Anywhere;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "overflow-wrap"));
    assert_eq!(child.overflow_wrap, OverflowWrapValue::Anywhere);
}

#[test]
/// 测试 text-align-last apply_property_value
fn test_apply_text_align_last() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.text_align_last, TextAlignLastValue::Auto);

    assert!(apply_property_value(&mut style, "text-align-last", "left"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Left);

    assert!(apply_property_value(&mut style, "text-align-last", "right"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Right);

    assert!(apply_property_value(&mut style, "text-align-last", "center"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Center);

    assert!(apply_property_value(&mut style, "text-align-last", "justify"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Justify);

    assert!(apply_property_value(&mut style, "text-align-last", "start"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Start);

    assert!(apply_property_value(&mut style, "text-align-last", "end"));
    assert_eq!(style.text_align_last, TextAlignLastValue::End);

    assert!(!apply_property_value(&mut style, "text-align-last", "invalid"));
}

#[test]
/// 测试 text-align-last 继承性
fn test_text_align_last_inherited() {
    assert!(PropertyRegistry::is_inherited("text-align-last"));
}

#[test]
/// 测试 text-align-last initial_value
fn test_text_align_last_initial_value() {
    assert!(PropertyRegistry::initial_value("text-align-last").is_some());
    let mut style = ComputedStyle::default();
    style.text_align_last = TextAlignLastValue::Justify;
    assert!(apply_initial_value(&mut style, "text-align-last"));
    assert_eq!(style.text_align_last, TextAlignLastValue::Auto);
}

#[test]
/// 测试 text-align-last 继承
fn test_text_align_last_inherit() {
    let mut parent = ComputedStyle::default();
    parent.text_align_last = TextAlignLastValue::Center;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "text-align-last"));
    assert_eq!(child.text_align_last, TextAlignLastValue::Center);
}

#[test]
/// 测试 font-variant-numeric apply_property_value
fn test_apply_font_variant_numeric() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Normal);

    assert!(apply_property_value(&mut style, "font-variant-numeric", "ordinal"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Ordinal);

    assert!(apply_property_value(&mut style, "font-variant-numeric", "slashed-zero"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::SlashedZero);

    assert!(apply_property_value(&mut style, "font-variant-numeric", "lining-nums"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::LiningNums);

    assert!(apply_property_value(
        &mut style,
        "font-variant-numeric",
        "oldstyle-nums"
    ));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::OldstyleNums);

    assert!(apply_property_value(
        &mut style,
        "font-variant-numeric",
        "proportional-nums"
    ));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::ProportionalNums);

    assert!(apply_property_value(&mut style, "font-variant-numeric", "tabular-nums"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::TabularNums);

    assert!(apply_property_value(
        &mut style,
        "font-variant-numeric",
        "diagonal-fractions"
    ));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::DiagonalFractions);

    assert!(apply_property_value(
        &mut style,
        "font-variant-numeric",
        "stacked-fractions"
    ));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::StackedFractions);

    assert!(!apply_property_value(&mut style, "font-variant-numeric", "invalid"));
}

#[test]
/// 测试 font-variant-numeric 继承性
fn test_font_variant_numeric_inherited() {
    assert!(PropertyRegistry::is_inherited("font-variant-numeric"));
}

#[test]
/// 测试 font-variant-numeric initial_value
fn test_font_variant_numeric_initial_value() {
    assert!(PropertyRegistry::initial_value("font-variant-numeric").is_some());
    let mut style = ComputedStyle::default();
    style.font_variant_numeric = FontVariantNumericValue::Ordinal;
    assert!(apply_initial_value(&mut style, "font-variant-numeric"));
    assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Normal);
}

#[test]
/// 测试 font-variant-numeric 继承
fn test_font_variant_numeric_inherit() {
    let mut parent = ComputedStyle::default();
    parent.font_variant_numeric = FontVariantNumericValue::TabularNums;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "font-variant-numeric"));
    assert_eq!(child.font_variant_numeric, FontVariantNumericValue::TabularNums);
}

#[test]
fn test_font_variant_alternates_apply_initial_and_inherit() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "font-variant-alternates",
        "historical-forms"
    ));
    assert_eq!(
        style.font_variant_alternates,
        FontVariantAlternatesValue::HistoricalForms
    );
    assert!(!apply_property_value(
        &mut style,
        "font-variant-alternates",
        "stylistic(foo)"
    ));
    assert!(PropertyRegistry::is_inherited("font-variant-alternates"));

    let mut child = ComputedStyle::default();
    assert!(inherit_property(&style, &mut child, "font-variant-alternates"));
    assert_eq!(
        child.font_variant_alternates,
        FontVariantAlternatesValue::HistoricalForms
    );
    assert!(apply_initial_value(&mut child, "font-variant-alternates"));
    assert_eq!(child.font_variant_alternates, FontVariantAlternatesValue::Normal);
}

#[test]
fn test_font_feature_settings_apply_initial_and_inherit() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "font-feature-settings",
        "'liga' off, \"kern\" 2"
    ));
    assert_eq!(
        style.font_feature_settings,
        FontFeatureSettingsValue::Features(vec![
            FontFeatureSetting {
                tag: *b"liga",
                value: 0,
            },
            FontFeatureSetting {
                tag: *b"kern",
                value: 2,
            },
        ])
    );
    assert!(PropertyRegistry::is_inherited("font-feature-settings"));

    let mut child = ComputedStyle::default();
    assert!(inherit_property(&style, &mut child, "font-feature-settings"));
    assert_eq!(child.font_feature_settings, style.font_feature_settings);
    assert!(apply_initial_value(&mut child, "font-feature-settings"));
    assert_eq!(child.font_feature_settings, FontFeatureSettingsValue::Normal);
}

#[test]
fn test_font_variant_ligatures_apply_initial_and_inherit() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "font-variant-ligatures",
        "common-ligatures no-discretionary-ligatures"
    ));
    assert_eq!(
        style.font_variant_ligatures,
        FontVariantLigaturesValue {
            common: Some(true),
            discretionary: Some(false),
            historical: None,
            contextual: None,
        }
    );
    assert!(PropertyRegistry::is_inherited("font-variant-ligatures"));

    let mut child = ComputedStyle::default();
    assert!(inherit_property(&style, &mut child, "font-variant-ligatures"));
    assert_eq!(child.font_variant_ligatures, style.font_variant_ligatures);
    assert!(apply_initial_value(&mut child, "font-variant-ligatures"));
    assert_eq!(child.font_variant_ligatures, FontVariantLigaturesValue::default());
}

#[test]
/// 测试新属性在 known_properties 中（overflow-wrap、text-align-last、font-variant-numeric）
fn test_text_new_properties_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"overflow-wrap"));
    assert!(props.contains(&"text-align-last"));
    assert!(props.contains(&"font-variant-numeric"));
}

#[test]
/// 测试新属性 apply_initial_value_all_properties 覆盖
fn test_new_properties_apply_initial_value() {
    for prop in &["overflow-wrap", "text-align-last", "font-variant-numeric"] {
        let mut style = ComputedStyle::default();
        assert!(
            apply_initial_value(&mut style, prop),
            "apply_initial_value should handle: {prop}"
        );
    }
}

#[test]
/// 测试 pointer-events 继承（inherit_property）
fn test_pointer_events_inherit() {
    let mut parent = ComputedStyle::default();
    parent.pointer_events = PointerEventsValue::None;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "pointer-events"));
    assert_eq!(child.pointer_events, PointerEventsValue::None);
}

// ═══════════════════════════════════════════════════════════════════
// direction / unicode-bidi / tab-size 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 direction 默认值为 ltr
fn test_direction_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.direction, DirectionValue::Ltr);
}

#[test]
/// 测试 direction apply_property_value
fn test_direction_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "direction", "rtl"));
    assert_eq!(style.direction, DirectionValue::Rtl);

    assert!(apply_property_value(&mut style, "direction", "ltr"));
    assert_eq!(style.direction, DirectionValue::Ltr);

    assert!(!apply_property_value(&mut style, "direction", "invalid"));
}

#[test]
/// 测试 direction 继承性（inherited）
fn test_direction_inherited() {
    assert!(PropertyRegistry::is_inherited("direction"));
}

#[test]
/// 测试 direction initial_value
fn test_direction_initial_value() {
    assert!(PropertyRegistry::initial_value("direction").is_some());
    let mut style = ComputedStyle::default();
    style.direction = DirectionValue::Rtl;
    assert!(apply_initial_value(&mut style, "direction"));
    assert_eq!(style.direction, DirectionValue::Ltr);
}

#[test]
/// 测试 direction 继承
fn test_direction_inherit() {
    let mut parent = ComputedStyle::default();
    parent.direction = DirectionValue::Rtl;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "direction"));
    assert_eq!(child.direction, DirectionValue::Rtl);
}

#[test]
/// 测试 unicode-bidi 默认值为 normal
fn test_unicode_bidi_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Normal);
}

#[test]
/// 测试 unicode-bidi apply_property_value
fn test_unicode_bidi_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "unicode-bidi", "embed"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Embed);

    assert!(apply_property_value(&mut style, "unicode-bidi", "isolate"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Isolate);

    assert!(apply_property_value(&mut style, "unicode-bidi", "bidi-override"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::BidiOverride);

    assert!(apply_property_value(&mut style, "unicode-bidi", "isolate-override"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::IsolateOverride);

    assert!(apply_property_value(&mut style, "unicode-bidi", "plaintext"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Plaintext);

    assert!(!apply_property_value(&mut style, "unicode-bidi", "invalid"));
}

#[test]
/// 测试 unicode-bidi 不继承
fn test_unicode_bidi_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("unicode-bidi"));
}

#[test]
/// 测试 unicode-bidi initial_value
fn test_unicode_bidi_initial_value() {
    assert!(PropertyRegistry::initial_value("unicode-bidi").is_some());
    let mut style = ComputedStyle::default();
    style.unicode_bidi = UnicodeBidiValue::Embed;
    assert!(apply_initial_value(&mut style, "unicode-bidi"));
    assert_eq!(style.unicode_bidi, UnicodeBidiValue::Normal);
}

#[test]
/// 测试 tab-size 默认值为 8
fn test_tab_size_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.tab_size, TabSizeValue::Number(8));
}

#[test]
/// 测试 tab-size apply_property_value
fn test_tab_size_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "tab-size", "4"));
    assert_eq!(style.tab_size, TabSizeValue::Number(4));

    assert!(apply_property_value(&mut style, "tab-size", "20px"));
    assert_eq!(style.tab_size, TabSizeValue::Length(LengthValue::Px(20.0)));

    assert!(apply_property_value(&mut style, "tab-size", "2em"));
    assert_eq!(style.tab_size, TabSizeValue::Length(LengthValue::Em(2.0)));

    assert!(!apply_property_value(&mut style, "tab-size", "invalid"));
}

#[test]
/// 测试 tab-size 继承性（inherited）
fn test_tab_size_inherited() {
    assert!(PropertyRegistry::is_inherited("tab-size"));
}

#[test]
/// 测试 tab-size initial_value
fn test_tab_size_initial_value() {
    assert!(PropertyRegistry::initial_value("tab-size").is_some());
    let mut style = ComputedStyle::default();
    style.tab_size = TabSizeValue::Number(2);
    assert!(apply_initial_value(&mut style, "tab-size"));
    assert_eq!(style.tab_size, TabSizeValue::Number(8));
}

#[test]
/// 测试 tab-size 继承
fn test_tab_size_inherit() {
    let mut parent = ComputedStyle::default();
    parent.tab_size = TabSizeValue::Number(4);
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "tab-size"));
    assert_eq!(child.tab_size, TabSizeValue::Number(4));
}

#[test]
/// 测试新属性在 known_properties 中
fn test_direction_unicode_bidi_tab_size_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"direction"));
    assert!(props.contains(&"unicode-bidi"));
    assert!(props.contains(&"tab-size"));
}

// ═══════════════════════════════════════════════════════════════════
// contain + column-rule-color 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// contain 默认值为 None
fn test_contain_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.contain, ContainComputedValue::None);
}

#[test]
/// contain 应用关键字值
fn test_apply_contain_keywords() {
    let mut style = ComputedStyle::default();

    assert!(apply_property_value(&mut style, "contain", "strict"));
    assert_eq!(style.contain, ContainComputedValue::Strict);

    assert!(apply_property_value(&mut style, "contain", "content"));
    assert_eq!(style.contain, ContainComputedValue::Content);

    assert!(apply_property_value(&mut style, "contain", "none"));
    assert_eq!(style.contain, ContainComputedValue::None);

    assert!(apply_property_value(&mut style, "contain", "size"));
    assert_eq!(style.contain, ContainComputedValue::Size);

    assert!(apply_property_value(&mut style, "contain", "layout"));
    assert_eq!(style.contain, ContainComputedValue::Layout);

    assert!(apply_property_value(&mut style, "contain", "style"));
    assert_eq!(style.contain, ContainComputedValue::Style);

    assert!(apply_property_value(&mut style, "contain", "paint"));
    assert_eq!(style.contain, ContainComputedValue::Paint);

    assert!(!apply_property_value(&mut style, "contain", "invalid"));
}

#[test]
/// contain 支持多值空格分隔
fn test_apply_contain_multi_value() {
    let mut style = ComputedStyle::default();

    assert!(apply_property_value(&mut style, "contain", "layout style paint"));
    match &style.contain {
        ContainComputedValue::Custom(flags) => {
            let expected =
                ContainComputedValue::FLAG_LAYOUT | ContainComputedValue::FLAG_STYLE | ContainComputedValue::FLAG_PAINT;
            assert_eq!(*flags, expected);
        }
        _ => panic!("expected Custom, got {:?}", style.contain),
    }

    // "layout style paint size" 等价于 content 的位组合
    assert!(apply_property_value(&mut style, "contain", "layout style paint size"));
    match &style.contain {
        ContainComputedValue::Custom(flags) => {
            let expected = ContainComputedValue::FLAG_LAYOUT
                | ContainComputedValue::FLAG_STYLE
                | ContainComputedValue::FLAG_PAINT
                | ContainComputedValue::FLAG_SIZE;
            assert_eq!(*flags, expected);
        }
        _ => panic!("expected Custom, got {:?}", style.contain),
    }
}

#[test]
/// contain 不继承
fn test_contain_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("contain"));
}

#[test]
/// contain 在 known_properties 中
fn test_contain_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"contain"));
}

#[test]
/// contain 有 initial_value
fn test_contain_initial_value() {
    assert!(PropertyRegistry::initial_value("contain").is_some());
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Strict;
    assert!(apply_initial_value(&mut style, "contain"));
    assert_eq!(style.contain, ContainComputedValue::None);
}

#[test]
/// CSS Multi-column §4.3：column-rule-color 初始值 = currentColor（与 border-color 同）。
fn test_column_rule_color_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.column_rule_color, ColorValue::CurrentColor);
}

#[test]
/// column-rule-color 应用颜色值
fn test_apply_column_rule_color() {
    let mut style = ComputedStyle::default();

    assert!(apply_property_value(&mut style, "column-rule-color", "red"));
    assert_eq!(style.column_rule_color, ColorValue::Rgba(255, 0, 0, 255));

    assert!(apply_property_value(&mut style, "column-rule-color", "#00ff00"));
    assert_eq!(style.column_rule_color, ColorValue::Rgba(0, 255, 0, 255));

    assert!(apply_property_value(&mut style, "column-rule-color", "transparent"));
    assert_eq!(style.column_rule_color, ColorValue::Transparent);

    assert!(apply_property_value(&mut style, "column-rule-color", "currentColor"));
    assert_eq!(style.column_rule_color, ColorValue::CurrentColor);

    assert!(!apply_property_value(&mut style, "column-rule-color", "not-a-color"));
}

#[test]
/// column-rule-color 不继承
fn test_column_rule_color_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("column-rule-color"));
}

#[test]
/// column-rule-color 在 known_properties 中
fn test_column_rule_color_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"column-rule-color"));
}

#[test]
/// column-rule-color 有 initial_value
fn test_column_rule_color_initial_value() {
    assert!(PropertyRegistry::initial_value("column-rule-color").is_some());
    let mut style = ComputedStyle::default();
    style.column_rule_color = ColorValue::Rgba(255, 0, 0, 255);
    assert!(apply_initial_value(&mut style, "column-rule-color"));
    // CSS Multi-column §4.3：初始 = currentColor（与 border-color 同）。
    assert_eq!(style.column_rule_color, ColorValue::CurrentColor);
}

// ── appearance 属性测试 ──

#[test]
fn test_apply_property_appearance_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "appearance", "none"));
    assert_eq!(style.appearance, AppearanceComputedValue::None);
}

#[test]
fn test_apply_property_appearance_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "appearance", "auto"));
    assert_eq!(style.appearance, AppearanceComputedValue::Auto);
}

#[test]
fn test_apply_property_appearance_button() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "appearance", "button"));
    assert_eq!(style.appearance, AppearanceComputedValue::Button);
}

#[test]
fn test_apply_property_appearance_checkbox() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "appearance", "checkbox"));
    assert_eq!(style.appearance, AppearanceComputedValue::Checkbox);
}

#[test]
fn test_apply_property_appearance_textfield() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "appearance", "textfield"));
    assert_eq!(style.appearance, AppearanceComputedValue::Textfield);
}

#[test]
fn test_apply_property_appearance_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "appearance", "invalid"));
}

#[test]
fn test_appearance_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("appearance"));
}

#[test]
fn test_appearance_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"appearance"));
}

#[test]
fn test_appearance_initial_value() {
    assert!(PropertyRegistry::initial_value("appearance").is_some());
    let mut style = ComputedStyle::default();
    style.appearance = AppearanceComputedValue::None;
    assert!(apply_initial_value(&mut style, "appearance"));
    assert_eq!(style.appearance, AppearanceComputedValue::Auto);
}

// ── accent-color 属性测试 ──

#[test]
fn test_apply_property_accent_color_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "accent-color", "auto"));
    assert_eq!(style.accent_color, AccentColorComputedValue::Auto);
}

#[test]
fn test_apply_property_accent_color_named() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "accent-color", "red"));
    assert_eq!(
        style.accent_color,
        AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255))
    );
}

#[test]
fn test_apply_property_accent_color_hex() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "accent-color", "#00ff00"));
    assert_eq!(
        style.accent_color,
        AccentColorComputedValue::Color(ColorValue::Rgba(0, 255, 0, 255))
    );
}

#[test]
fn test_apply_property_accent_color_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "accent-color", "not-a-color"));
}

#[test]
fn test_accent_color_is_inherited() {
    assert!(PropertyRegistry::is_inherited("accent-color"));
}

#[test]
fn test_accent_color_inherit() {
    let mut parent = ComputedStyle::default();
    parent.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "accent-color"));
    assert_eq!(
        child.accent_color,
        AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255))
    );
}

#[test]
fn test_accent_color_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"accent-color"));
}

#[test]
fn test_accent_color_initial_value() {
    assert!(PropertyRegistry::initial_value("accent-color").is_some());
    let mut style = ComputedStyle::default();
    style.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(0, 128, 0, 255));
    assert!(apply_initial_value(&mut style, "accent-color"));
    assert_eq!(style.accent_color, AccentColorComputedValue::Auto);
}

// ── caret-color 属性测试 ──

#[test]
fn test_apply_property_caret_color_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "caret-color", "auto"));
    assert_eq!(style.caret_color, CaretColorComputedValue::Auto);
}

#[test]
fn test_apply_property_caret_color_named() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "caret-color", "blue"));
    assert_eq!(
        style.caret_color,
        CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255))
    );
}

#[test]
fn test_apply_property_caret_color_hex() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "caret-color", "#abcdef"));
    assert_eq!(
        style.caret_color,
        CaretColorComputedValue::Color(ColorValue::Rgba(0xAB, 0xCD, 0xEF, 255))
    );
}

#[test]
fn test_apply_property_caret_color_transparent() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "caret-color", "transparent"));
    assert_eq!(
        style.caret_color,
        CaretColorComputedValue::Color(ColorValue::Transparent)
    );
}

#[test]
fn test_apply_property_caret_color_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "caret-color", "not-a-color"));
}

#[test]
fn test_caret_color_is_inherited() {
    assert!(PropertyRegistry::is_inherited("caret-color"));
}

#[test]
fn test_caret_color_inherit() {
    let mut parent = ComputedStyle::default();
    parent.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255));
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "caret-color"));
    assert_eq!(
        child.caret_color,
        CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255))
    );
}

#[test]
fn test_caret_color_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"caret-color"));
}

#[test]
fn test_caret_color_initial_value() {
    assert!(PropertyRegistry::initial_value("caret-color").is_some());
    let mut style = ComputedStyle::default();
    style.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
    assert!(apply_initial_value(&mut style, "caret-color"));
    assert_eq!(style.caret_color, CaretColorComputedValue::Auto);
}

// ── mix-blend-mode 属性测试 ──

#[test]
fn test_apply_property_mix_blend_mode_normal() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "normal"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Normal);
}

#[test]
fn test_apply_property_mix_blend_mode_multiply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "multiply"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Multiply);
}

#[test]
fn test_apply_property_mix_blend_mode_screen() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "screen"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Screen);
}

#[test]
fn test_apply_property_mix_blend_mode_overlay() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "overlay"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Overlay);
}

#[test]
fn test_apply_property_mix_blend_mode_darken() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "darken"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Darken);
}

#[test]
fn test_apply_property_mix_blend_mode_lighten() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "lighten"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Lighten);
}

#[test]
fn test_apply_property_mix_blend_mode_color_dodge() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "color-dodge"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::ColorDodge);
}

#[test]
fn test_apply_property_mix_blend_mode_color_burn() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "color-burn"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::ColorBurn);
}

#[test]
fn test_apply_property_mix_blend_mode_hard_light() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "hard-light"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::HardLight);
}

#[test]
fn test_apply_property_mix_blend_mode_soft_light() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "soft-light"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::SoftLight);
}

#[test]
fn test_apply_property_mix_blend_mode_difference() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "difference"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Difference);
}

#[test]
fn test_apply_property_mix_blend_mode_exclusion() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "exclusion"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Exclusion);
}

#[test]
fn test_apply_property_mix_blend_mode_hue() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "hue"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Hue);
}

#[test]
fn test_apply_property_mix_blend_mode_saturation() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "saturation"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Saturation);
}

#[test]
fn test_apply_property_mix_blend_mode_color() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "color"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Color);
}

#[test]
fn test_apply_property_mix_blend_mode_luminosity() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "mix-blend-mode", "luminosity"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Luminosity);
}

#[test]
fn test_apply_property_mix_blend_mode_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "mix-blend-mode", "invalid"));
}

#[test]
fn test_mix_blend_mode_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("mix-blend-mode"));
}

#[test]
fn test_mix_blend_mode_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"mix-blend-mode"));
}

#[test]
fn test_mix_blend_mode_initial_value() {
    assert!(PropertyRegistry::initial_value("mix-blend-mode").is_some());
    let mut style = ComputedStyle::default();
    style.mix_blend_mode = MixBlendModeComputedValue::Multiply;
    assert!(apply_initial_value(&mut style, "mix-blend-mode"));
    assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Normal);
}

// ── scrollbar-width 属性测试 ──

#[test]
fn test_apply_property_scrollbar_width_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scrollbar-width", "auto"));
    assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Auto);
}

#[test]
fn test_apply_property_scrollbar_width_thin() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scrollbar-width", "thin"));
    assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Thin);
}

#[test]
fn test_apply_property_scrollbar_width_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scrollbar-width", "none"));
    assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::None);
}

#[test]
fn test_apply_property_scrollbar_width_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "scrollbar-width", "thick"));
}

#[test]
fn test_scrollbar_width_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("scrollbar-width"));
}

#[test]
fn test_scrollbar_width_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"scrollbar-width"));
}

#[test]
fn test_scrollbar_width_initial_value() {
    assert!(PropertyRegistry::initial_value("scrollbar-width").is_some());
    let mut style = ComputedStyle::default();
    style.scrollbar_width = ScrollbarWidthComputedValue::Thin;
    assert!(apply_initial_value(&mut style, "scrollbar-width"));
    assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Auto);
}

// ── scrollbar-gutter 属性测试 ──

#[test]
fn test_apply_property_scrollbar_gutter_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scrollbar-gutter", "auto"));
    assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Auto);
}

#[test]
fn test_apply_property_scrollbar_gutter_stable() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scrollbar-gutter", "stable"));
    assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Stable);
}

#[test]
fn test_apply_property_scrollbar_gutter_stable_both_edges() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "scrollbar-gutter",
        "stable both-edges"
    ));
    assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::StableBothEdges);
}

#[test]
fn test_apply_property_scrollbar_gutter_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "scrollbar-gutter", "both-edges"));
    assert!(!apply_property_value(&mut style, "scrollbar-gutter", "invalid"));
}

#[test]
fn test_scrollbar_gutter_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("scrollbar-gutter"));
}

#[test]
fn test_scrollbar_gutter_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"scrollbar-gutter"));
}

#[test]
fn test_scrollbar_gutter_initial_value() {
    assert!(PropertyRegistry::initial_value("scrollbar-gutter").is_some());
    let mut style = ComputedStyle::default();
    style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
    assert!(apply_initial_value(&mut style, "scrollbar-gutter"));
    assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Auto);
}

// ── text-wrap 属性测试 ──

#[test]
fn test_apply_property_text_wrap_wrap() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "wrap"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Wrap);
}

#[test]
fn test_apply_property_text_wrap_nowrap() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "nowrap"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Nowrap);
}

#[test]
fn test_apply_property_text_wrap_balance() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "balance"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Balance);
}

#[test]
fn test_apply_property_text_wrap_pretty() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "pretty"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Pretty);
}

#[test]
fn test_apply_property_text_wrap_stable() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-wrap", "stable"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Stable);
}

#[test]
fn test_apply_property_text_wrap_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "text-wrap", "auto"));
    assert!(!apply_property_value(&mut style, "text-wrap", "invalid"));
}

#[test]
fn test_text_wrap_is_inherited() {
    assert!(PropertyRegistry::is_inherited("text-wrap"));
}

#[test]
fn test_text_wrap_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"text-wrap"));
}

#[test]
fn test_text_wrap_initial_value() {
    assert!(PropertyRegistry::initial_value("text-wrap").is_some());
    let mut style = ComputedStyle::default();
    style.text_wrap = TextWrapComputedValue::Nowrap;
    assert!(apply_initial_value(&mut style, "text-wrap"));
    assert_eq!(style.text_wrap, TextWrapComputedValue::Wrap);
}

#[test]
fn test_text_wrap_inherit() {
    let mut parent = ComputedStyle::default();
    parent.text_wrap = TextWrapComputedValue::Balance;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "text-wrap"));
    assert_eq!(child.text_wrap, TextWrapComputedValue::Balance);
}

// ── hyphens 属性测试 ──

#[test]
fn test_apply_property_hyphens_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "hyphens", "none"));
    assert_eq!(style.hyphens, HyphensComputedValue::None);
}

#[test]
fn test_apply_property_hyphens_manual() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "hyphens", "manual"));
    assert_eq!(style.hyphens, HyphensComputedValue::Manual);
}

#[test]
fn test_apply_property_hyphens_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "hyphens", "auto"));
    assert_eq!(style.hyphens, HyphensComputedValue::Auto);
}

#[test]
fn test_apply_property_hyphens_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "hyphens", "all"));
    assert!(!apply_property_value(&mut style, "hyphens", "invalid"));
}

#[test]
fn test_hyphens_is_inherited() {
    assert!(PropertyRegistry::is_inherited("hyphens"));
}

#[test]
fn test_hyphens_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"hyphens"));
}

#[test]
fn test_hyphens_initial_value() {
    assert!(PropertyRegistry::initial_value("hyphens").is_some());
    let mut style = ComputedStyle::default();
    style.hyphens = HyphensComputedValue::Auto;
    assert!(apply_initial_value(&mut style, "hyphens"));
    assert_eq!(style.hyphens, HyphensComputedValue::None);
}

#[test]
fn test_hyphens_inherit() {
    let mut parent = ComputedStyle::default();
    parent.hyphens = HyphensComputedValue::Auto;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "hyphens"));
    assert_eq!(child.hyphens, HyphensComputedValue::Auto);
}

// ── line-clamp 属性测试 ──

#[test]
fn test_apply_property_line_clamp_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-clamp", "none"));
    assert_eq!(style.line_clamp, LineClampComputedValue::None);
}

#[test]
fn test_apply_property_line_clamp_count() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-clamp", "3"));
    assert_eq!(style.line_clamp, LineClampComputedValue::Count(3));
}

/// R2296：`-webkit-line-clamp`（web-compat 遗留语法，多数页面/测试用此）应别名到 `line-clamp`。
#[test]
fn test_apply_property_webkit_line_clamp_alias() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "-webkit-line-clamp", "2"));
    assert_eq!(style.line_clamp, LineClampComputedValue::Count(2));
    // none 也应工作。
    let mut style2 = ComputedStyle::default();
    assert!(apply_property_value(&mut style2, "-webkit-line-clamp", "none"));
    assert_eq!(style2.line_clamp, LineClampComputedValue::None);
}

#[test]
fn test_apply_property_line_clamp_count_one() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-clamp", "1"));
    assert_eq!(style.line_clamp, LineClampComputedValue::Count(1));
}

#[test]
fn test_apply_property_line_clamp_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "line-clamp", "0"));
    assert!(!apply_property_value(&mut style, "line-clamp", "-1"));
    assert!(!apply_property_value(&mut style, "line-clamp", "auto"));
    assert!(!apply_property_value(&mut style, "line-clamp", "invalid"));
}

#[test]
fn test_line_clamp_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("line-clamp"));
}

#[test]
fn test_line_clamp_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"line-clamp"));
}

#[test]
fn test_line_clamp_initial_value() {
    assert!(PropertyRegistry::initial_value("line-clamp").is_some());
    let mut style = ComputedStyle::default();
    style.line_clamp = LineClampComputedValue::Count(5);
    assert!(apply_initial_value(&mut style, "line-clamp"));
    assert_eq!(style.line_clamp, LineClampComputedValue::None);
}

// ── background-image 属性测试 ──

#[test]
fn test_apply_property_background_image_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "none"));
    // "none" generates vec![None], representing an explicit no-image layer
    assert_eq!(style.background_image, vec![BackgroundImageComputedValue::None]);
}

#[test]
fn test_apply_property_background_image_url() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "url(bg.png)"));
    assert_eq!(
        style.background_image,
        vec![BackgroundImageComputedValue::Url("bg.png".to_string())]
    );
}

#[test]
fn test_apply_property_background_image_url_quoted() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-image", "url(\"bg.png\")"));
    assert_eq!(
        style.background_image,
        vec![BackgroundImageComputedValue::Url("bg.png".to_string())]
    );
}

#[test]
fn test_apply_property_background_image_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-image", "invalid"));
}

#[test]
fn test_background_image_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-image"));
}

#[test]
fn test_background_image_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-image"));
}

#[test]
fn test_background_image_initial_value() {
    assert!(PropertyRegistry::initial_value("background-image").is_some());
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("test.png".to_string())];
    assert!(apply_initial_value(&mut style, "background-image"));
    assert!(style.background_image.is_empty());
}

// ── background-position 属性测试 ──

#[test]
fn test_apply_property_background_position_center() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "center"));
    assert_eq!(style.background_position, vec![BackgroundPositionComputedValue::Center]);
}

#[test]
fn test_apply_property_background_position_left() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "left"));
    assert_eq!(style.background_position, vec![BackgroundPositionComputedValue::Left]);
}

#[test]
fn test_apply_property_background_position_percent() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "50%"));
    assert_eq!(
        style.background_position,
        vec![BackgroundPositionComputedValue::Percent(50.0)]
    );
}

#[test]
fn test_apply_property_background_position_length() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "10px"));
    assert_eq!(
        style.background_position,
        vec![BackgroundPositionComputedValue::Length(10.0)]
    );
}

/// R1417：em/rem 单位 background-position 此前被 parser 拒绝（仅匹配 LengthValue::Px），
/// 现保留 LengthValue 经 apply 按 font-size 解析为 px。默认 font-size=16px → 2em=32px。
#[test]
fn test_apply_property_background_position_em_resolves_to_px() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "2em"));
    assert_eq!(
        style.background_position,
        vec![BackgroundPositionComputedValue::Length(32.0)]
    );
    // -0em（负零 em，驱动案 background-position-076）应解析为 0px。
    let mut style2 = ComputedStyle::default();
    assert!(apply_property_value(&mut style2, "background-position", "-0em"));
    assert_eq!(
        style2.background_position,
        vec![BackgroundPositionComputedValue::Length(0.0)]
    );
}

/// R1417：默认 background-position 应为 0% 0%（top-left，CSS initial），非单值（旧实现
/// 经 resolve_background_position 单值规则把垂直 default 到 center）。
#[test]
fn test_apply_property_background_position_default_is_zero_zero() {
    let style = ComputedStyle::default();
    match &style.background_position[0] {
        BackgroundPositionComputedValue::TwoValue(h, v) => {
            assert_eq!(**h, BackgroundPositionComputedValue::Percent(0.0));
            assert_eq!(**v, BackgroundPositionComputedValue::Percent(0.0));
        }
        other => panic!("default background-position 应为 TwoValue(0%,0%), got {other:?}"),
    }
}

#[test]
fn test_apply_property_background_position_two_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-position", "left top"));
    match &style.background_position[0] {
        BackgroundPositionComputedValue::TwoValue(h, v) => {
            assert_eq!(**h, BackgroundPositionComputedValue::Left);
            assert_eq!(**v, BackgroundPositionComputedValue::Top);
        }
        other => panic!("Expected TwoValue, got {:?}", other),
    }
}

#[test]
fn test_apply_property_background_position_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-position", "invalid"));
}

#[test]
fn test_background_position_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-position"));
}

#[test]
fn test_background_position_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-position"));
}

#[test]
fn test_background_position_initial_value() {
    assert!(PropertyRegistry::initial_value("background-position").is_some());
    let mut style = ComputedStyle::default();
    style.background_position = vec![BackgroundPositionComputedValue::Center];
    assert!(apply_initial_value(&mut style, "background-position"));
    // R1417：CSS initial = 0% 0%（top-left，双值），非单值 Percent(0.0)。
    assert_eq!(
        style.background_position,
        vec![BackgroundPositionComputedValue::TwoValue(
            Box::new(BackgroundPositionComputedValue::Percent(0.0)),
            Box::new(BackgroundPositionComputedValue::Percent(0.0)),
        )]
    );
}

// ── background-repeat 属性测试 ──

#[test]
fn test_apply_property_background_repeat_repeat() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "repeat"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::Repeat]);
}

#[test]
fn test_apply_property_background_repeat_no_repeat() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "no-repeat"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::NoRepeat]);
}

#[test]
fn test_apply_property_background_repeat_repeat_x() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "repeat-x"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::RepeatX]);
}

#[test]
fn test_apply_property_background_repeat_repeat_y() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "repeat-y"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::RepeatY]);
}

#[test]
fn test_apply_property_background_repeat_space() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "space"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::Space]);
}

#[test]
fn test_apply_property_background_repeat_round() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-repeat", "round"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::Round]);
}

#[test]
fn test_apply_property_background_repeat_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "background-repeat", "invalid"));
}

#[test]
fn test_background_repeat_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("background-repeat"));
}

#[test]
fn test_background_repeat_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"background-repeat"));
}

#[test]
fn test_background_repeat_initial_value() {
    assert!(PropertyRegistry::initial_value("background-repeat").is_some());
    let mut style = ComputedStyle::default();
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    assert!(apply_initial_value(&mut style, "background-repeat"));
    assert_eq!(style.background_repeat, vec![BackgroundRepeatComputedValue::Repeat]);
}

// ── background-size 属性测试 ──

#[test]
fn test_apply_property_background_size_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "auto"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Auto]);
}

#[test]
fn test_apply_property_background_size_cover() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "cover"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Cover]);
}

#[test]
fn test_apply_property_background_size_contain() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "contain"));
    assert_eq!(style.background_size, vec![BackgroundSizeComputedValue::Contain]);
}

// R2311：background 多层 longhand apply（`<position>#` / `<repeat-style>#` / `<bg-size>#`）

/// R2311：多层 background-position/repeat/size 正确展开为 Vec；单层 byte-identical。
#[test]
fn test_apply_property_background_multi_layer_longhands() {
    let mut style = ComputedStyle::default();
    // background-position 多层
    assert!(apply_property_value(
        &mut style,
        "background-position",
        "center, left top"
    ));
    assert_eq!(style.background_position.len(), 2);
    // background-repeat 多层
    assert!(apply_property_value(
        &mut style,
        "background-repeat",
        "repeat, no-repeat, space"
    ));
    assert_eq!(
        style.background_repeat,
        vec![
            BackgroundRepeatComputedValue::Repeat,
            BackgroundRepeatComputedValue::NoRepeat,
            BackgroundRepeatComputedValue::Space,
        ]
    );
    // background-size 多层
    assert!(apply_property_value(&mut style, "background-size", "cover, 50%"));
    assert_eq!(
        style.background_size,
        vec![
            BackgroundSizeComputedValue::Cover,
            BackgroundSizeComputedValue::Percent(50.0)
        ]
    );
    // 任一层失败 → 整条不应用（保持上一次的值）
    assert!(!apply_property_value(&mut style, "background-repeat", "repeat, bogus"));
    assert_eq!(style.background_repeat.len(), 3); // 未被覆盖
}
