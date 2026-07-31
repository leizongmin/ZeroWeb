//! apply_property_value 分支覆盖率测试
//!
//! 直接调用 apply_property_value 覆盖所有属性分支，
//! 确保 style-system/src/property/apply.rs 中每个 match arm 都被测试到。

use crate::ComputedStyle;
use crate::property::apply::apply_property_value;

/// 辅助：创建空的 ComputedStyle 并应用单个属性
fn apply(prop: &str, value: &str) -> (bool, ComputedStyle) {
    let mut style = ComputedStyle::default();
    let ok = apply_property_value(&mut style, prop, value);
    (ok, style)
}

// === 布局属性 ===

#[test]
fn test_apply_display() {
    let (ok, s) = apply("display", "block");
    assert!(ok);
    assert!(matches!(s.display, zero_css_parser::values::DisplayValue::Block));
}

#[test]
fn test_apply_position() {
    let (ok, s) = apply("position", "absolute");
    assert!(ok);
    assert!(matches!(s.position, zero_css_parser::values::PositionValue::Absolute));
}

#[test]
fn test_apply_float() {
    let (ok, s) = apply("float", "left");
    assert!(ok);
    assert!(matches!(s.float, zero_css_parser::values::FloatValue::Left));
}

#[test]
fn test_apply_clear() {
    let (ok, s) = apply("clear", "both");
    assert!(ok);
    assert!(matches!(s.clear, zero_css_parser::values::ClearValue::Both));
}

#[test]
fn test_apply_box_sizing() {
    let (ok, _s) = apply("box-sizing", "border-box");
    assert!(ok);
}

#[test]
fn test_apply_invalid_property() {
    let (ok, _) = apply("nonexistent-prop", "value");
    assert!(!ok);
}

// === 尺寸属性 ===

#[test]
fn test_apply_width_height() {
    let (ok, s) = apply("width", "100px");
    assert!(ok);
    assert!(matches!(s.width, zero_css_parser::values::LengthValue::Px(100.0)));

    let (ok, s) = apply("height", "50em");
    assert!(ok);
    assert!(matches!(s.height, zero_css_parser::values::LengthValue::Em(50.0)));
}

#[test]
fn test_apply_inline_block_size_logical() {
    // CSS Logical Properties：inline-size / block-size 在水平书写模式下等价于
    // width / height（垂直模式的轴交换由 converter 的 swap_writing_mode_axes 负责）。
    // 旧实现完全忽略这两个属性（未知属性），导致 firefox-bug-1881495 等用例失效。
    let (ok, s) = apply("inline-size", "1em");
    assert!(ok);
    assert!(matches!(s.width, zero_css_parser::values::LengthValue::Em(1.0)));

    let (ok, s) = apply("block-size", "2em");
    assert!(ok);
    assert!(matches!(s.height, zero_css_parser::values::LengthValue::Em(2.0)));

    // min/max 逻辑尺寸（CSS Logical Properties §7）：水平模式下等价于 min/max-width/height
    //（垂直模式轴交换同 inline-size/block-size，由 converter 负责）。R2301 补齐逻辑尺寸族——
    // 旧 impl 仅 inline-size/block-size，缺 min/max 逻辑变体（不一致缺口）。
    let (ok, s) = apply("min-inline-size", "10px");
    assert!(ok);
    assert!(matches!(s.min_width, zero_css_parser::values::LengthValue::Px(10.0)));
    let (ok, s) = apply("min-block-size", "20px");
    assert!(ok);
    assert!(matches!(s.min_height, zero_css_parser::values::LengthValue::Px(20.0)));
    let (ok, s) = apply("max-inline-size", "300px");
    assert!(ok);
    assert!(matches!(s.max_width, zero_css_parser::values::LengthValue::Px(300.0)));
    let (ok, s) = apply("max-block-size", "none");
    assert!(ok);
    assert!(matches!(
        s.max_height,
        zero_css_parser::values::LengthValue::Px(f64::INFINITY)
    ));
}

#[test]
fn test_apply_min_max_dimensions() {
    let (ok, _) = apply("min-width", "10px");
    assert!(ok);
    let (ok, _) = apply("min-height", "20px");
    assert!(ok);
    let (ok, s) = apply("max-width", "none");
    assert!(ok);
    assert!(matches!(s.max_width, zero_css_parser::values::LengthValue::Px(v) if v == f64::INFINITY));
    let (ok, s) = apply("max-height", "none");
    assert!(ok);
    assert!(matches!(s.max_height, zero_css_parser::values::LengthValue::Px(v) if v == f64::INFINITY));
    let (ok, _) = apply("max-width", "500px");
    assert!(ok);
    let (ok, _) = apply("max-height", "400px");
    assert!(ok);
}

#[test]
fn test_apply_aspect_ratio() {
    let (ok, s) = apply("aspect-ratio", "auto");
    assert!(ok);
    assert!(s.aspect_ratio.is_none());

    let (ok, s) = apply("aspect-ratio", "16 / 9");
    assert!(ok);
    assert_eq!(s.aspect_ratio, Some(16.0 / 9.0));

    let (ok, s) = apply("aspect-ratio", "2");
    assert!(ok);
    assert_eq!(s.aspect_ratio, Some(2.0));

    let (ok, _) = apply("aspect-ratio", "invalid");
    assert!(!ok);
    let (ok, _) = apply("aspect-ratio", "1 / 0");
    assert!(!ok);
}

// === Margin / Padding ===

#[test]
fn test_apply_margins() {
    for prop in ["margin-top", "margin-right", "margin-bottom", "margin-left"] {
        let (ok, _) = apply(prop, "10px");
        assert!(ok, "{} should apply", prop);
    }
}

#[test]
fn test_apply_paddings() {
    for prop in ["padding-top", "padding-right", "padding-bottom", "padding-left"] {
        let (ok, _) = apply(prop, "5px");
        assert!(ok, "{} should apply", prop);
    }
}

// === Border 属性 ===

#[test]
fn test_apply_border_widths() {
    for prop in [
        "border-top-width",
        "border-right-width",
        "border-bottom-width",
        "border-left-width",
    ] {
        let (ok, _) = apply(prop, "2px");
        assert!(ok, "{} should apply", prop);
    }
}

#[test]
fn test_apply_border_colors() {
    for prop in [
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
    ] {
        let (ok, _) = apply(prop, "red");
        assert!(ok, "{} should apply", prop);
    }
}

#[test]
fn test_apply_border_styles() {
    for prop in [
        "border-top-style",
        "border-right-style",
        "border-bottom-style",
        "border-left-style",
    ] {
        let (ok, _) = apply(prop, "solid");
        assert!(ok, "{} should apply", prop);
    }
}

#[test]
fn test_apply_border_radius() {
    for prop in [
        "border-top-left-radius",
        "border-top-right-radius",
        "border-bottom-right-radius",
        "border-bottom-left-radius",
    ] {
        let (ok, _) = apply(prop, "8px");
        assert!(ok, "{} should apply", prop);
    }
}

// === Outline 属性 ===

#[test]
fn test_apply_outline() {
    let (ok, _) = apply("outline-width", "2px");
    assert!(ok);
    let (ok, _) = apply("outline-style", "solid");
    assert!(ok);
    let (ok, _) = apply("outline-color", "blue");
    assert!(ok);
    let (ok, _) = apply("outline-offset", "3px");
    assert!(ok);
}

// === 颜色 / 透明度 / 可见性 ===

#[test]
fn test_apply_color() {
    let (ok, _) = apply("color", "red");
    assert!(ok);
    let (ok, _) = apply("background-color", "#ff0000");
    assert!(ok);
}

#[test]
fn test_apply_opacity() {
    let (ok, s) = apply("opacity", "0.5");
    assert!(ok);
    assert_eq!(s.opacity, 0.5);
}

#[test]
fn test_apply_visibility() {
    let (ok, _) = apply("visibility", "hidden");
    assert!(ok);
}

// === 字体属性 ===

#[test]
fn test_apply_font() {
    let (ok, _) = apply("font-family", "Arial, sans-serif");
    assert!(ok);
    let (ok, _) = apply("font-size", "16px");
    assert!(ok);
    let (ok, _) = apply("font-weight", "bold");
    assert!(ok);
    let (ok, _) = apply("font-style", "italic");
    assert!(ok);
    let (ok, _) = apply("line-height", "1.5");
    assert!(ok);
}

// === 文本属性 ===

#[test]
fn test_apply_text_properties() {
    let (ok, _) = apply("text-align", "center");
    assert!(ok);
    let (ok, _) = apply("text-decoration", "underline");
    assert!(ok);
    let (ok, _) = apply("text-decoration-line", "line-through");
    assert!(ok);
    let (ok, _) = apply("text-transform", "uppercase");
    assert!(ok);
    let (ok, _) = apply("letter-spacing", "2px");
    assert!(ok);
    let (ok, _) = apply("word-spacing", "5px");
    assert!(ok);
    let (ok, _) = apply("white-space", "nowrap");
    assert!(ok);
    let (ok, _) = apply("word-break", "break-all");
    assert!(ok);
    let (ok, _) = apply("writing-mode", "vertical-rl");
    assert!(ok);
    let (ok, _) = apply("text-indent", "20px");
    assert!(ok);
    let (ok, _) = apply("text-overflow", "ellipsis");
    assert!(ok);
    let (ok, _) = apply("vertical-align", "middle");
    assert!(ok);
}

#[test]
fn test_apply_text_transform_full_width_and_kana() {
    use crate::property::types::TextTransformValue;
    // R2327：text-transform full-width / full-size-kana 须解析并写入 ComputedStyle。
    let (ok, s) = apply("text-transform", "full-width");
    assert!(ok, "full-width parses");
    assert!(
        matches!(s.text_transform, TextTransformValue::FullWidth),
        "stored as FullWidth"
    );
    let (ok, s) = apply("text-transform", "full-size-kana");
    assert!(ok, "full-size-kana parses");
    assert!(
        matches!(s.text_transform, TextTransformValue::FullSizeKana),
        "stored as FullSizeKana"
    );
    // 回归：既有值仍工作
    let (ok, s) = apply("text-transform", "uppercase");
    assert!(ok);
    assert!(matches!(s.text_transform, TextTransformValue::Uppercase));
}

#[test]
fn test_apply_text_align_last() {
    let (ok, _) = apply("text-align-last", "justify");
    assert!(ok);
}

#[test]
fn test_apply_font_variant_numeric() {
    let (ok, _) = apply("font-variant-numeric", "ordinal");
    assert!(ok);
}

// === 表格属性 ===

#[test]
fn test_apply_table_properties() {
    let (ok, _) = apply("table-layout", "fixed");
    assert!(ok);
    let (ok, _) = apply("caption-side", "bottom");
    assert!(ok);
    let (ok, _) = apply("border-collapse", "collapse");
    assert!(ok);
    let (ok, _) = apply("resize", "both");
    assert!(ok);
}

// === 列表属性 ===

#[test]
fn test_apply_list_properties() {
    let (ok, _) = apply("list-style-type", "disc");
    assert!(ok);
    let (ok, _) = apply("list-style-position", "inside");
    assert!(ok);
    let (ok, _) = apply("list-style-image", "url(test.png)");
    assert!(ok);
    let (ok, _) = apply("list-style-image", "none");
    assert!(ok);
}

// === Flexbox 属性 ===

#[test]
fn test_apply_flex_properties() {
    let (ok, _) = apply("flex-direction", "row-reverse");
    assert!(ok);
    let (ok, _) = apply("flex-wrap", "wrap");
    assert!(ok);
    let (ok, _) = apply("justify-content", "space-between");
    assert!(ok);
    let (ok, _) = apply("align-items", "center");
    assert!(ok);
    let (ok, _) = apply("align-self", "flex-end");
    assert!(ok);
    let (ok, _) = apply("flex-grow", "2");
    assert!(ok);
    let (ok, _) = apply("flex-shrink", "0");
    assert!(ok);
    let (ok, _) = apply("flex-basis", "200px");
    assert!(ok);
    let (ok, _) = apply("gap", "10px");
    assert!(ok);
    let (ok, _) = apply("column-gap", "5px");
    assert!(ok);
    let (ok, _) = apply("order", "3");
    assert!(ok);
}

// === Grid 属性 ===

#[test]
fn test_apply_grid_properties() {
    let (ok, _) = apply("grid-template-columns", "1fr 1fr");
    assert!(ok);
    let (ok, _) = apply("grid-template-rows", "auto");
    assert!(ok);
    let (ok, _) = apply("grid-auto-flow", "dense");
    assert!(ok);
    let (ok, _) = apply("grid-column-start", "1");
    assert!(ok);
    let (ok, _) = apply("grid-column-end", "3");
    assert!(ok);
    let (ok, _) = apply("grid-row-start", "2");
    assert!(ok);
    let (ok, _) = apply("grid-row-end", "auto");
    assert!(ok);
    let (ok, _) = apply("grid-auto-rows", "100px");
    assert!(ok);
    let (ok, _) = apply("grid-auto-columns", "1fr");
    assert!(ok);
    let (ok, _) = apply("grid-template-areas", "'a b'");
    assert!(ok);
    let (ok, _) = apply("grid-area", "1 / 1 / 3 / 3");
    assert!(ok);
    let (ok, _) = apply("grid-column", "1 / 3");
    assert!(ok);
    let (ok, _) = apply("grid-row", "1 / span 2");
    assert!(ok);
    let (ok, _) = apply("row-gap", "8px");
    assert!(ok);
}

// === 定位 ===

#[test]
fn test_apply_positioning() {
    let (ok, _) = apply("top", "10px");
    assert!(ok);
    let (ok, _) = apply("right", "20px");
    assert!(ok);
    let (ok, _) = apply("bottom", "30px");
    assert!(ok);
    let (ok, _) = apply("left", "40px");
    assert!(ok);
    let (ok, _) = apply("z-index", "10");
    assert!(ok);
}

#[test]
fn test_apply_overflow() {
    let (ok, _) = apply("overflow-x", "hidden");
    assert!(ok);
    let (ok, _) = apply("overflow-y", "scroll");
    assert!(ok);
}

// === Cursor ===

#[test]
fn test_apply_cursor() {
    let (ok, _) = apply("cursor", "pointer");
    assert!(ok);
}

// === Transform 属性 ===

#[test]
fn test_apply_transform() {
    let (ok, _) = apply("transform", "rotate(45deg)");
    assert!(ok);
}

#[test]
fn test_apply_transform_origin() {
    let (ok, _) = apply("transform-origin", "50% 50%");
    assert!(ok);
    let (ok, _) = apply("transform-origin", "10px");
    assert!(ok);
    let (ok, _) = apply("transform-origin", "10px 20px");
    assert!(ok);
}

#[test]
fn test_apply_perspective() {
    let (ok, _) = apply("perspective", "500px");
    assert!(ok);
    let (ok, _) = apply("perspective", "none");
    assert!(ok);
}

#[test]
fn test_apply_perspective_origin() {
    let (ok, _) = apply("perspective-origin", "50% 50%");
    assert!(ok);
}

#[test]
fn test_apply_transform_style() {
    let (ok, s) = apply("transform-style", "flat");
    assert!(ok);
    assert!(matches!(
        s.transform_style,
        crate::property::types::TransformStyleValue::Flat
    ));
    let (ok, s) = apply("transform-style", "preserve-3d");
    assert!(ok);
    assert!(matches!(
        s.transform_style,
        crate::property::types::TransformStyleValue::Preserve3d
    ));
    let (ok, _) = apply("transform-style", "invalid");
    assert!(!ok);
}

#[test]
fn test_apply_backface_visibility() {
    let (ok, _) = apply("backface-visibility", "visible");
    assert!(ok);
    let (ok, _) = apply("backface-visibility", "hidden");
    assert!(ok);
    let (ok, _) = apply("backface-visibility", "invalid");
    assert!(!ok);
}

// === Transition 属性 ===

#[test]
fn test_apply_transition_properties() {
    let (ok, s) = apply("transition-property", "opacity, transform");
    assert!(ok);
    assert_eq!(s.transition_property, vec!["opacity", "transform"]);

    let (ok, _) = apply("transition-property", "none");
    assert!(ok);
    // none → empty list

    let (ok, _) = apply("transition-duration", "0.3s, 0.5s");
    assert!(ok);

    let (ok, _) = apply("transition-timing-function", "ease");
    assert!(ok);

    let (ok, _) = apply("transition-delay", "0.1s");
    assert!(ok);
}

// === 逻辑属性 ===

#[test]
fn test_apply_logical_properties() {
    for (prop, _) in [
        ("margin-block-start", "5px"),
        ("margin-block-end", "5px"),
        ("margin-inline-start", "5px"),
        ("margin-inline-end", "5px"),
        ("padding-block-start", "3px"),
        ("padding-block-end", "3px"),
        ("padding-inline-start", "3px"),
        ("padding-inline-end", "3px"),
        ("inset-block-start", "1px"),
        ("inset-block-end", "1px"),
        ("inset-inline-start", "1px"),
        ("inset-inline-end", "1px"),
    ] {
        let (ok, _) = apply(prop, "5px");
        assert!(ok, "{} should apply", prop);
    }
}

// === Animation 属性 ===

#[test]
fn test_apply_animation_properties() {
    let (ok, s) = apply("animation-name", "fadeIn, fadeOut");
    assert!(ok);
    assert_eq!(s.animation_name, vec!["fadeIn", "fadeOut"]);

    let (ok, _) = apply("animation-name", "none");
    assert!(ok);

    let (ok, _) = apply("animation-duration", "1s, 2s");
    assert!(ok);

    let (ok, _) = apply("animation-timing-function", "linear");
    assert!(ok);

    let (ok, _) = apply("animation-delay", "0.5s");
    assert!(ok);

    let (ok, _) = apply("animation-iteration-count", "3");
    assert!(ok);
    let (ok, _) = apply("animation-iteration-count", "infinite");
    assert!(ok);

    let (ok, _) = apply("animation-direction", "alternate");
    assert!(ok);

    let (ok, _) = apply("animation-fill-mode", "forwards");
    assert!(ok);

    let (ok, _) = apply("animation-play-state", "paused");
    assert!(ok);
}

// === Scroll Snap 属性 ===

#[test]
fn test_apply_scroll_snap() {
    let (ok, _) = apply("scroll-snap-type", "x mandatory");
    assert!(ok);
    let (ok, _) = apply("scroll-snap-align", "center");
    assert!(ok);
    let (ok, _) = apply("scroll-snap-stop", "always");
    assert!(ok);
}

#[test]
fn test_apply_scroll_margin() {
    for prop in [
        "scroll-margin-top",
        "scroll-margin-right",
        "scroll-margin-bottom",
        "scroll-margin-left",
    ] {
        let (ok, _) = apply(prop, "10px");
        assert!(ok, "{} should apply", prop);
    }
}

#[test]
fn test_apply_scroll_padding() {
    for prop in [
        "scroll-padding-top",
        "scroll-padding-right",
        "scroll-padding-bottom",
        "scroll-padding-left",
    ] {
        let (ok, _) = apply(prop, "5px");
        assert!(ok, "{} should apply", prop);
    }
}

// === Container Query 属性 ===

#[test]
fn test_apply_container_properties() {
    let (ok, _) = apply("container-type", "inline-size");
    assert!(ok);
    let (ok, s) = apply("container-name", "sidebar");
    assert!(ok);
    assert_eq!(s.container_name, Some("sidebar".to_string()));
    let (ok, s) = apply("container-name", "none");
    assert!(ok);
    assert!(s.container_name.is_none());
}

// === Counter 属性 ===

#[test]
fn test_apply_counter_properties() {
    let (ok, _) = apply("counter-reset", "section 0");
    assert!(ok);
    let (ok, _) = apply("counter-increment", "section 1");
    assert!(ok);
    let (ok, _) = apply("counter-set", "section 5");
    assert!(ok);
}

// === Content / Quotes ===

#[test]
fn test_apply_content() {
    let (ok, _) = apply("content", "normal");
    assert!(ok);
    let (ok, _) = apply("content", "none");
    assert!(ok);
}

#[test]
fn test_apply_quotes() {
    let (ok, _) = apply("quotes", "none");
    assert!(ok);
    let (ok, _) = apply("quotes", "auto");
    assert!(ok);
}

// === Page Break ===

#[test]
fn test_apply_page_break() {
    let (ok, _) = apply("page-break-before", "always");
    assert!(ok);
    let (ok, _) = apply("page-break-after", "avoid");
    assert!(ok);
    let (ok, _) = apply("page-break-inside", "avoid");
    assert!(ok);
}

// === BoxDecorationBreak / ImageRendering / Isolation ===

#[test]
fn test_apply_box_decoration_break() {
    let (ok, _) = apply("box-decoration-break", "clone");
    assert!(ok);
}

#[test]
fn test_apply_image_rendering() {
    let (ok, _) = apply("image-rendering", "pixelated");
    assert!(ok);
}

#[test]
fn test_apply_isolation() {
    let (ok, _) = apply("isolation", "isolate");
    assert!(ok);
}

// === Break 属性 ===

#[test]
fn test_apply_break() {
    let (ok, _) = apply("break-inside", "avoid");
    assert!(ok);
    let (ok, _) = apply("break-before", "page");
    assert!(ok);
    let (ok, _) = apply("break-after", "column");
    assert!(ok);
}

// === Column Rule ===

#[test]
fn test_apply_column_rule() {
    let (ok, _) = apply("column-rule-width", "2px");
    assert!(ok);
    let (ok, _) = apply("column-rule-style", "dashed");
    assert!(ok);
    let (ok, _) = apply("column-rule-color", "gray");
    assert!(ok);
}

// === Interaction / Performance Hint ===

#[test]
fn test_apply_overscroll_behavior() {
    let (ok, _) = apply("overscroll-behavior-x", "contain");
    assert!(ok);
    let (ok, _) = apply("overscroll-behavior-y", "none");
    assert!(ok);
}

#[test]
fn test_apply_touch_action() {
    let (ok, _) = apply("touch-action", "manipulation");
    assert!(ok);
}

#[test]
fn test_apply_user_select() {
    let (ok, _) = apply("user-select", "none");
    assert!(ok);
}

#[test]
fn test_apply_will_change() {
    let (ok, _) = apply("will-change", "transform");
    assert!(ok);
    let (ok, _) = apply("will-change", "auto");
    assert!(ok);
}

#[test]
fn test_apply_pointer_events() {
    let (ok, _) = apply("pointer-events", "none");
    assert!(ok);
}

// === OverflowWrap ===

#[test]
fn test_apply_overflow_wrap() {
    let (ok, _) = apply("overflow-wrap", "break-word");
    assert!(ok);
}

// === Direction / UnicodeBidi / TabSize ===

#[test]
fn test_apply_direction() {
    let (ok, _) = apply("direction", "rtl");
    assert!(ok);
}

#[test]
fn test_apply_unicode_bidi() {
    let (ok, _) = apply("unicode-bidi", "isolate");
    assert!(ok);
}

#[test]
fn test_apply_tab_size() {
    let (ok, _) = apply("tab-size", "4");
    assert!(ok);
}

// === Columns 简写 ===

#[test]
fn test_apply_columns_shorthand() {
    let (ok, _) = apply("columns", "3 100px");
    assert!(ok);
    let (ok, _) = apply("columns", "100px 3");
    assert!(ok);
    let (ok, _) = apply("columns", "auto");
    assert!(ok);
    let (ok, _) = apply("columns", "4");
    assert!(ok);
}

#[test]
fn test_apply_column_count_width() {
    let (ok, _) = apply("column-count", "3");
    assert!(ok);
    let (ok, _) = apply("column-width", "200px");
    assert!(ok);
}

// === ObjectFit / Filter ===

#[test]
fn test_apply_object_fit() {
    let (ok, _) = apply("object-fit", "cover");
    assert!(ok);
}

#[test]
fn test_apply_filter() {
    let (ok, _) = apply("filter", "blur(5px)");
    assert!(ok);
    let (ok, _) = apply("filter", "none");
    assert!(ok);
}

// === Contain ===

#[test]
fn test_apply_contain() {
    let (ok, _) = apply("contain", "strict");
    assert!(ok);
}

// === Appearance / AccentColor / CaretColor ===

#[test]
fn test_apply_appearance() {
    let (ok, _) = apply("appearance", "none");
    assert!(ok);
}

#[test]
fn test_apply_accent_color() {
    let (ok, _) = apply("accent-color", "auto");
    assert!(ok);
    let (ok, _) = apply("accent-color", "red");
    assert!(ok);
}

#[test]
fn test_apply_caret_color() {
    let (ok, _) = apply("caret-color", "auto");
    assert!(ok);
    let (ok, _) = apply("caret-color", "blue");
    assert!(ok);
}

// === MixBlendMode / ScrollbarWidth / ScrollbarGutter ===

#[test]
fn test_apply_mix_blend_mode() {
    let (ok, _) = apply("mix-blend-mode", "multiply");
    assert!(ok);
}

#[test]
fn test_apply_scrollbar_width() {
    let (ok, _) = apply("scrollbar-width", "thin");
    assert!(ok);
}

#[test]
fn test_apply_scrollbar_gutter() {
    let (ok, _) = apply("scrollbar-gutter", "stable");
    assert!(ok);
}

// === JustifyItems / JustifySelf / AlignContent ===

#[test]
fn test_apply_justify_items() {
    let (ok, _) = apply("justify-items", "center");
    assert!(ok);
}

#[test]
fn test_apply_justify_self() {
    let (ok, _) = apply("justify-self", "start");
    assert!(ok);
}

#[test]
fn test_apply_align_content() {
    let (ok, _) = apply("align-content", "space-between");
    assert!(ok);
}

// === EmptyCells / BorderSpacing ===

#[test]
fn test_apply_empty_cells() {
    let (ok, _) = apply("empty-cells", "hide");
    assert!(ok);
}

#[test]
fn test_apply_border_spacing() {
    let (ok, _) = apply("border-spacing", "5px 10px");
    assert!(ok);
}

// === BorderImage 属性 ===

#[test]
fn test_apply_border_image_properties() {
    let (ok, _) = apply("border-image-source", "url(border.png)");
    assert!(ok);
    let (ok, _) = apply("border-image-slice", "30");
    assert!(ok);
    let (ok, _) = apply("border-image-width", "10px");
    assert!(ok);
    let (ok, _) = apply("border-image-repeat", "round");
    assert!(ok);
    let (ok, _) = apply("border-image-outset", "5px");
    assert!(ok);
}

// === BoxShadow / TextShadow ===

#[test]
fn test_apply_box_shadow() {
    let (ok, _) = apply("box-shadow", "2px 2px 5px black");
    assert!(ok);
}

#[test]
fn test_apply_text_shadow() {
    let (ok, _) = apply("text-shadow", "1px 1px red");
    assert!(ok);
}

// === Background 属性 ===

#[test]
fn test_apply_background_properties() {
    let (ok, _) = apply("background-image", "url(bg.png)");
    assert!(ok);
    let (ok, _) = apply("background-position", "center");
    assert!(ok);
    let (ok, _) = apply("background-repeat", "repeat-x");
    assert!(ok);
    let (ok, _) = apply("background-size", "cover");
    assert!(ok);
    let (ok, _) = apply("background-attachment", "fixed");
    assert!(ok);
    let (ok, _) = apply("background-clip", "content-box");
    assert!(ok);
    let (ok, _) = apply("background-origin", "padding-box");
    assert!(ok);
}

// === Hyphens / LineClamp / TextWrap ===

#[test]
fn test_apply_hyphens() {
    let (ok, _) = apply("hyphens", "auto");
    assert!(ok);
}

#[test]
fn test_apply_line_clamp() {
    let (ok, _) = apply("line-clamp", "3");
    assert!(ok);
}

#[test]
fn test_apply_text_wrap() {
    let (ok, _) = apply("text-wrap", "balance");
    assert!(ok);
}

// === calc / math 函数 ===

#[test]
fn test_apply_calc_values() {
    let (ok, _) = apply("width", "calc(100% - 20px)");
    assert!(ok);
    let (ok, _) = apply("width", "min(100px, 50%)");
    assert!(ok);
    let (ok, _) = apply("width", "max(100px, 50%)");
    assert!(ok);
    let (ok, _) = apply("width", "clamp(100px, 50%, 200px)");
    assert!(ok);
}

// === 无效值测试 ===

#[test]
fn test_apply_invalid_values() {
    assert!(!apply("display", "invalid-value").0);
    assert!(!apply("position", "invalid-value").0);
    assert!(!apply("width", "not-a-length").0);
    assert!(!apply("color", "not-a-color").0);
    assert!(!apply("flex-grow", "not-a-number").0);
    assert!(!apply("order", "not-a-number").0);
    assert!(!apply("opacity", "invalid").0);
}

// === apply_initial_value 覆盖 ===

#[test]
fn test_apply_initial_display() {
    let style = ComputedStyle::default();
    // display 初始值为 inline
    assert!(matches!(style.display, zero_css_parser::values::DisplayValue::Inline));
}

#[test]
fn test_apply_default_computed_style() {
    let style = ComputedStyle::default();
    // 验证默认值不会 panic
    let _ = format!("{:?}", style);
}

// === parse_length_or_math 路径覆盖 ===

#[test]
fn test_apply_em_rem_units() {
    let (ok, _) = apply("width", "2em");
    assert!(ok);
    let (ok, _) = apply("width", "1.5rem");
    assert!(ok);
    let (ok, _) = apply("width", "100%");
    assert!(ok);
    let (ok, _) = apply("width", "50vw");
    assert!(ok);
    let (ok, _) = apply("width", "30vh");
    assert!(ok);
}

// === 确保所有 resize 变体 ===

#[test]
fn test_apply_resize_variants() {
    for v in ["none", "both", "horizontal", "vertical", "block", "inline"] {
        let (ok, _) = apply("resize", v);
        assert!(ok, "resize: {} should apply", v);
    }
}

// === animation-iteration-count infinite ===

#[test]
fn test_apply_animation_iteration_count_infinite() {
    let (ok, s) = apply("animation-iteration-count", "infinite");
    assert!(ok);
    // infinite → None
    assert!(s.animation_iteration_count.len() == 1);
    assert!(s.animation_iteration_count[0].is_none());
}

// === transition-property none ===

#[test]
fn test_apply_transition_property_none() {
    let (ok, s) = apply("transition-property", "none");
    assert!(ok);
    assert!(s.transition_property.is_empty());
}

// === animation-name none ===

#[test]
fn test_apply_animation_name_none() {
    let (ok, s) = apply("animation-name", "none");
    assert!(ok);
    assert!(s.animation_name.is_empty());
}

// === font-family 不区分大小写 ===

#[test]
fn test_apply_font_family_generic() {
    let (ok, _) = apply("font-family", "sans-serif");
    assert!(ok);
    let (ok, _) = apply("font-family", "monospace");
    assert!(ok);
}

// === opacity 边界 ===

#[test]
fn test_apply_opacity_boundary() {
    let (ok, _) = apply("opacity", "0");
    assert!(ok);
    let (ok, _) = apply("opacity", "1");
    assert!(ok);
}

// === visibility 变体 ===

#[test]
fn test_apply_visibility_variants() {
    let (ok, _) = apply("visibility", "visible");
    assert!(ok);
    let (ok, _) = apply("visibility", "hidden");
    assert!(ok);
    let (ok, _) = apply("visibility", "collapse");
    assert!(ok);
}

// === flex-basis special values ===

#[test]
fn test_apply_flex_basis_content() {
    let (ok, _) = apply("flex-basis", "content");
    assert!(ok);
    let (ok, _) = apply("flex-basis", "auto");
    assert!(ok);
}

// === z-index auto ===

#[test]
fn test_apply_z_index_auto() {
    let (ok, _) = apply("z-index", "auto");
    assert!(ok);
}

// === line-height 变体 ===

#[test]
fn test_apply_line_height_variants() {
    let (ok, _) = apply("line-height", "normal");
    assert!(ok);
    let (ok, _) = apply("line-height", "2");
    assert!(ok);
    let (ok, _) = apply("line-height", "24px");
    assert!(ok);
}

// === scroll-padding auto ===

#[test]
fn test_apply_scroll_padding_auto() {
    let (ok, _) = apply("scroll-padding-top", "auto");
    assert!(ok);
}

// === container-type 变体 ===

#[test]
fn test_apply_container_type_variants() {
    let (ok, _) = apply("container-type", "normal");
    assert!(ok);
    let (ok, _) = apply("container-type", "size");
    assert!(ok);
    let (ok, _) = apply("container-type", "inline-size");
    assert!(ok);
}

// === pointer-events 变体 ===

#[test]
fn test_apply_pointer_events_variants() {
    for v in [
        "auto",
        "none",
        "visiblePainted",
        "visibleFill",
        "visibleStroke",
        "visible",
        "painted",
        "fill",
        "stroke",
        "all",
        "inherit",
    ] {
        let (ok, _) = apply("pointer-events", v);
        assert!(ok, "pointer-events: {} should apply", v);
    }
}

// === appearance 变体 ===

#[test]
fn test_apply_appearance_variants() {
    for v in [
        "none",
        "auto",
        "button",
        "checkbox",
        "listbox",
        "menulist",
        "meter",
        "progress-bar",
        "push-button",
        "radio",
        "searchfield",
        "slider-horizontal",
        "square-button",
        "textarea",
        "textfield",
    ] {
        let (ok, _) = apply("appearance", v);
        assert!(ok, "appearance: {} should apply", v);
    }
}

// === contain 变体 ===

#[test]
fn test_apply_contain_variants() {
    let (ok, _) = apply("contain", "none");
    assert!(ok);
    let (ok, _) = apply("contain", "strict");
    assert!(ok);
    let (ok, _) = apply("contain", "content");
    assert!(ok);
    let (ok, _) = apply("contain", "size layout");
    assert!(ok);
}

// === filter 函数变体 ===

#[test]
fn test_apply_filter_functions() {
    let (ok, _) = apply("filter", "blur(5px)");
    assert!(ok);
    let (ok, _) = apply("filter", "brightness(1.5)");
    assert!(ok);
    let (ok, _) = apply("filter", "contrast(0.8)");
    assert!(ok);
    let (ok, _) = apply("filter", "grayscale(1)");
    assert!(ok);
    let (ok, _) = apply("filter", "hue-rotate(90deg)");
    assert!(ok);
    let (ok, _) = apply("filter", "invert(1)");
    assert!(ok);
    let (ok, _) = apply("filter", "opacity(0.5)");
    assert!(ok);
    let (ok, _) = apply("filter", "saturate(2)");
    assert!(ok);
    let (ok, _) = apply("filter", "sepia(1)");
    assert!(ok);
    let (ok, _) = apply("filter", "drop-shadow(1 2 3 black)");
    assert!(ok);
}

// === mix-blend-mode 变体 ===

#[test]
fn test_apply_mix_blend_mode_variants() {
    for v in [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "darken",
        "lighten",
        "color-dodge",
        "color-burn",
        "hard-light",
        "soft-light",
        "difference",
        "exclusion",
        "hue",
        "saturation",
        "color",
        "luminosity",
    ] {
        let (ok, _) = apply("mix-blend-mode", v);
        assert!(ok, "mix-blend-mode: {} should apply", v);
    }
}

// === image-rendering 变体 ===

#[test]
fn test_apply_image_rendering_variants() {
    for v in ["auto", "smooth", "high-quality", "pixelated", "crisp-edges"] {
        let (ok, _) = apply("image-rendering", v);
        assert!(ok, "image-rendering: {} should apply", v);
    }
}
