//! Matcher 额外覆盖率测试：is_property_supported、container、supports 等。

use super::super::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
    ContainerCondition, ContainerRule, ContainerSizeCondition, Declaration, LayerRule, NthPattern, PseudoClassSelector,
    PseudoElementSelector, Rule, Selector, StyleRule, SubclassSelector, TypeSelector,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_dom::Document;

// ═══════════════════════════════════════════════════════════════════════
// is_property_supported 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_property_supported_display() {
    assert!(is_property_supported("display", "block"));
    assert!(is_property_supported("display", "flex"));
    assert!(is_property_supported("display", "grid"));
    assert!(!is_property_supported("display", "invalid"));
}

#[test]
fn test_property_supported_position() {
    assert!(is_property_supported("position", "relative"));
    assert!(is_property_supported("position", "absolute"));
    assert!(!is_property_supported("position", "invalid"));
}

#[test]
fn test_property_supported_overflow() {
    assert!(is_property_supported("overflow", "hidden"));
    assert!(is_property_supported("overflow-x", "scroll"));
    assert!(!is_property_supported("overflow", "invalid"));
}

#[test]
fn test_property_supported_visibility() {
    assert!(is_property_supported("visibility", "visible"));
    assert!(!is_property_supported("visibility", "invalid"));
}

#[test]
fn test_property_supported_box_sizing() {
    assert!(is_property_supported("box-sizing", "border-box"));
    assert!(!is_property_supported("box-sizing", "invalid"));
}

#[test]
fn test_property_supported_flex() {
    assert!(is_property_supported("flex-direction", "row"));
    assert!(is_property_supported("flex-wrap", "wrap"));
    assert!(!is_property_supported("flex-direction", "invalid"));
}

#[test]
fn test_property_supported_alignment() {
    assert!(is_property_supported("justify-content", "center"));
    assert!(is_property_supported("align-items", "flex-start"));
    assert!(is_property_supported("align-self", "stretch"));
    assert!(is_property_supported("justify-self", "start"));
    assert!(!is_property_supported("justify-content", "invalid"));
}

#[test]
fn test_property_supported_font() {
    assert!(is_property_supported("font-weight", "bold"));
    assert!(is_property_supported("font-style", "italic"));
    assert!(!is_property_supported("font-weight", "invalid"));
}

#[test]
fn test_property_supported_flow_and_list_style() {
    assert!(is_property_supported("float", "inline-start"));
    assert!(is_property_supported("clear", "both"));
    assert!(is_property_supported("list-style-type", "\"Step\""));
    assert!(is_property_supported("list-style-type", "Custom-Style"));
    assert!(is_property_supported("list-style-position", "inside"));
    assert!(is_property_supported("list-style-image", "url(\"my marker.png\")"));
    assert!(is_property_supported("list-style", "square inside url(marker.png)"));
    assert!(is_property_supported("list-style", "URL(\"my marker.png\") inside"));

    assert!(!is_property_supported("float", "both"));
    assert!(!is_property_supported("clear", "inline"));
    assert!(!is_property_supported("list-style-type", "123bad"));
    assert!(!is_property_supported("list-style-position", "center"));
    assert!(!is_property_supported("list-style-image", "url(my marker.png)"));
    assert!(!is_property_supported("list-style", "inside outside"));
    assert!(!is_property_supported("list-style", "disc url(\"marker.png\" extra)"));
}

#[test]
fn test_property_supported_table_and_ui_text_properties() {
    assert!(is_property_supported("text-align", "match-parent"));
    assert!(is_property_supported("text-align-last", "justify"));
    assert!(is_property_supported("white-space", "break-spaces"));
    assert!(is_property_supported("vertical-align", "middle"));
    assert!(is_property_supported("table-layout", "fixed"));
    assert!(is_property_supported("caption-side", "bottom"));
    assert!(is_property_supported("border-collapse", "collapse"));
    assert!(is_property_supported("empty-cells", "hide"));
    assert!(is_property_supported("border-spacing", "1em 2ch"));
    assert!(is_property_supported("resize", "inline"));
    assert!(is_property_supported("direction", "rtl"));
    assert!(is_property_supported("unicode-bidi", "plaintext"));

    assert!(!is_property_supported("text-align", "sparkle"));
    assert!(!is_property_supported("text-align-last", "sparkle"));
    assert!(!is_property_supported("white-space", "wrap"));
    assert!(!is_property_supported("vertical-align", "center"));
    assert!(!is_property_supported("table-layout", "grid"));
    assert!(!is_property_supported("caption-side", "left"));
    assert!(!is_property_supported("border-collapse", "hidden"));
    assert!(!is_property_supported("empty-cells", "visible"));
    assert!(!is_property_supported("border-spacing", "10%"));
    assert!(!is_property_supported("border-spacing", "-1px"));
    assert!(!is_property_supported("resize", "diagonal"));
    assert!(!is_property_supported("direction", "auto"));
    assert!(!is_property_supported("unicode-bidi", "isolate isolate"));
}

#[test]
fn test_property_supported_ui_and_text_decoration_properties() {
    assert!(is_property_supported("outline-width", "medium"));
    assert!(is_property_supported("outline-style", "auto"));
    assert!(is_property_supported("outline-color", "currentColor"));
    assert!(is_property_supported("outline-offset", "inset"));
    assert!(is_property_supported("outline-offset", "-0.25em"));
    assert!(is_property_supported("content-visibility", "auto"));
    assert!(is_property_supported("opacity", "50%"));
    assert!(is_property_supported("cursor", "zoom-in"));
    assert!(is_property_supported("text-decoration", "underline"));
    assert!(is_property_supported("text-decoration-line", "underline overline"));
    assert!(is_property_supported(
        "text-decoration-color",
        "light-dark(black, white)"
    ));
    assert!(is_property_supported("text-decoration-style", "wavy"));
    assert!(is_property_supported("text-emphasis-style", "open sesame"));
    assert!(is_property_supported("text-emphasis-position", "under left"));
    assert!(is_property_supported("text-emphasis-color", "blue"));
    assert!(is_property_supported("text-transform", "full-width"));
    assert!(is_property_supported("word-break", "keep-all"));
    assert!(is_property_supported(
        "text-autospace",
        "ideograph-alpha ideograph-numeric"
    ));
    assert!(is_property_supported("line-break", "anywhere"));
    assert!(is_property_supported("writing-mode", "sideways-rl"));
    assert!(is_property_supported("text-overflow", "\"...\""));

    assert!(!is_property_supported("outline-width", "10%"));
    assert!(!is_property_supported("outline-style", "hidden"));
    assert!(!is_property_supported("outline-offset", "auto"));
    assert!(!is_property_supported("content-visibility", "collapse"));
    assert!(!is_property_supported("opacity", "sparkle"));
    assert!(!is_property_supported("cursor", "diagonal"));
    assert!(!is_property_supported("text-decoration", "underline overline"));
    assert!(!is_property_supported("text-decoration-line", "underline sparkle"));
    assert!(!is_property_supported("text-decoration-style", "groove"));
    assert!(!is_property_supported("text-emphasis-style", "filled open"));
    assert!(!is_property_supported("text-emphasis-position", "over under"));
    assert!(!is_property_supported("text-transform", "small-caps"));
    assert!(!is_property_supported("word-break", "break-word break-all"));
    assert!(!is_property_supported("text-autospace", "ideograph-alpha sparkle"));
    assert!(!is_property_supported("line-break", "soft"));
    assert!(!is_property_supported("writing-mode", "vertical"));
    assert!(!is_property_supported("text-overflow", "fade"));
}

#[test]
fn test_property_supported_extended_visual_and_layout_properties() {
    assert!(is_property_supported("appearance", "textfield"));
    assert!(is_property_supported("accent-color", "auto"));
    assert!(is_property_supported("caret-color", "currentColor"));
    assert!(is_property_supported("mix-blend-mode", "multiply"));
    assert!(is_property_supported("scrollbar-width", "thin"));
    assert!(is_property_supported("scrollbar-gutter", "stable both-edges"));
    assert!(is_property_supported("text-wrap", "balance"));
    assert!(is_property_supported("hyphens", "manual"));
    assert!(is_property_supported("line-clamp", "3"));
    assert!(is_property_supported("-webkit-line-clamp", "none"));
    assert!(is_property_supported("object-fit", "scale-down"));
    assert!(is_property_supported("object-position", "left 20%"));
    assert!(is_property_supported("filter", "blur(2px) brightness(120%)"));
    assert!(is_property_supported("backdrop-filter", "none"));
    assert!(is_property_supported("contain", "layout paint"));
    assert!(is_property_supported("page-break-before", "left"));
    assert!(is_property_supported("page-break-inside", "avoid"));
    assert!(is_property_supported("box-decoration-break", "clone"));
    assert!(is_property_supported("image-rendering", "pixelated"));
    assert!(is_property_supported("isolation", "isolate"));
    assert!(is_property_supported("break-inside", "avoid-column"));
    assert!(is_property_supported("break-before", "page"));
    assert!(is_property_supported("break-after", "avoid-page"));
    assert!(is_property_supported("column-rule-width", "thin"));
    assert!(is_property_supported("column-rule-style", "dashed"));
    assert!(is_property_supported("column-rule-color", "red"));
    assert!(is_property_supported("overscroll-behavior-x", "contain"));
    assert!(is_property_supported("overscroll-behavior-y", "none"));
    assert!(is_property_supported("touch-action", "pan-x pan-y"));
    assert!(is_property_supported("user-select", "all"));
    assert!(is_property_supported("will-change", "opacity, transform"));
    assert!(is_property_supported("pointer-events", "visiblePainted"));
    assert!(is_property_supported("overflow-wrap", "anywhere"));
    assert!(is_property_supported("tab-size", "4"));
    assert!(is_property_supported("columns", "12em 3"));
    assert!(is_property_supported("column-count", "2"));
    assert!(is_property_supported("column-width", "12em"));
    assert!(is_property_supported("column-fill", "auto"));
    assert!(is_property_supported("column-span", "all"));
    assert!(is_property_supported("clip-path", "inset(10px 20%)"));
    assert!(is_property_supported("clip", "rect(0px, auto, 10px, 0px)"));

    assert!(!is_property_supported("appearance", "sparkle"));
    assert!(!is_property_supported("accent-color", "sparkle"));
    assert!(!is_property_supported("caret-color", "sparkle"));
    assert!(!is_property_supported("mix-blend-mode", "plus-lighter"));
    assert!(!is_property_supported("scrollbar-width", "wide"));
    assert!(!is_property_supported("scrollbar-gutter", "both-edges"));
    assert!(!is_property_supported("text-wrap", "auto"));
    assert!(!is_property_supported("hyphens", "all"));
    assert!(!is_property_supported("line-clamp", "0"));
    assert!(!is_property_supported("object-fit", "stretch"));
    assert!(!is_property_supported("object-position", "left top extra"));
    assert!(!is_property_supported("filter", "blur(-1px)"));
    assert!(!is_property_supported("contain", "layout sparkle"));
    assert!(!is_property_supported("page-break-inside", "left"));
    assert!(!is_property_supported("box-decoration-break", "split"));
    assert!(!is_property_supported("image-rendering", "nearest"));
    assert!(!is_property_supported("isolation", "none"));
    assert!(!is_property_supported("break-before", "always"));
    assert!(!is_property_supported("column-rule-width", "-1px"));
    assert!(!is_property_supported("column-rule-style", "auto"));
    assert!(!is_property_supported("overscroll-behavior-x", "scroll"));
    assert!(!is_property_supported("touch-action", "pinch-zoom"));
    assert!(!is_property_supported("user-select", "visible"));
    assert!(!is_property_supported("will-change", "auto, opacity"));
    assert!(!is_property_supported("pointer-events", "paint"));
    assert!(!is_property_supported("overflow-wrap", "wrap"));
    assert!(!is_property_supported("tab-size", "auto"));
    assert!(!is_property_supported("columns", "2 3"));
    assert!(!is_property_supported("column-count", "0"));
    assert!(!is_property_supported("column-width", "50%"));
    assert!(!is_property_supported("column-fill", "balance-all auto"));
    assert!(!is_property_supported("column-span", "none all"));
    assert!(!is_property_supported("clip-path", "polygon(0 0, , 100% 100%)"));
    assert!(!is_property_supported("clip", "rect(10px, 20px)"));
}

#[test]
fn test_property_supported_border_image_properties() {
    assert!(is_property_supported(
        "border-image",
        "url(border.png) 25 / 2 / 3 repeat"
    ));
    assert!(is_property_supported("border-image-source", "url(border.png)"));
    assert!(is_property_supported("border-image-slice", "25 fill"));
    assert!(is_property_supported("border-image-width", "auto"));
    assert!(is_property_supported("border-image-repeat", "repeat round"));
    assert!(is_property_supported("border-image-outset", "2"));

    assert!(!is_property_supported("border-image", "url(border.png) 25 / 2 / 3 / 4"));
    assert!(!is_property_supported("border-image-source", "invalid"));
    assert!(!is_property_supported("border-image-slice", "-1"));
    assert!(!is_property_supported("border-image-width", "-1px"));
    assert!(!is_property_supported("border-image-repeat", "repeat round stretch"));
    assert!(!is_property_supported("border-image-outset", "-1px"));
}

#[test]
fn test_property_supported_color() {
    assert!(is_property_supported("color", "red"));
    assert!(is_property_supported("background-color", "#fff"));
    assert!(is_property_supported("border-color", "rgb(0,0,0)"));
    assert!(is_property_supported("border-top-color", "blue"));
    assert!(is_property_supported("border-right-color", "green"));
    assert!(is_property_supported("border-bottom-color", "yellow"));
    assert!(is_property_supported("border-left-color", "purple"));
    assert!(!is_property_supported("color", "invalid-color-xyz"));
}

#[test]
fn test_property_supported_length_properties() {
    assert!(is_property_supported("width", "100px"));
    assert!(is_property_supported("inline-size", "100px"));
    assert!(is_property_supported("height", "50em"));
    assert!(is_property_supported("block-size", "50em"));
    assert!(is_property_supported("min-width", "10px"));
    assert!(is_property_supported("min-inline-size", "10px"));
    assert!(is_property_supported("max-width", "200px"));
    assert!(is_property_supported("max-width", "none"));
    assert!(is_property_supported("min-height", "10px"));
    assert!(is_property_supported("min-block-size", "10px"));
    assert!(is_property_supported("max-height", "200px"));
    assert!(is_property_supported("max-height", "none"));
    assert!(is_property_supported("max-block-size", "none"));
    assert!(is_property_supported("margin", "10px"));
    assert!(is_property_supported("margin-top", "5px"));
    assert!(is_property_supported("margin-right", "5px"));
    assert!(is_property_supported("margin-bottom", "5px"));
    assert!(is_property_supported("margin-left", "5px"));
    assert!(is_property_supported("margin-block", "1px auto"));
    assert!(is_property_supported("margin-inline-start", "2%"));
    assert!(is_property_supported("padding", "10px"));
    assert!(is_property_supported("padding-top", "5px"));
    assert!(is_property_supported("padding-right", "5px"));
    assert!(is_property_supported("padding-bottom", "5px"));
    assert!(is_property_supported("padding-left", "5px"));
    assert!(is_property_supported("padding-inline", "1px 2%"));
    assert!(is_property_supported("padding-block-end", "3em"));
    assert!(is_property_supported("gap", "10px"));
    assert!(is_property_supported("row-gap", "normal"));
    assert!(is_property_supported("column-gap", "2em"));
    assert!(is_property_supported("top", "0px"));
    assert!(is_property_supported("right", "0px"));
    assert!(is_property_supported("bottom", "0px"));
    assert!(is_property_supported("left", "0px"));
    assert!(is_property_supported("inset-block", "1px auto"));
    assert!(is_property_supported("inset-inline-end", "2%"));
    assert!(is_property_supported("border-top-width", "1px"));
    assert!(is_property_supported("border-right-width", "1px"));
    assert!(is_property_supported("border-bottom-width", "1px"));
    assert!(is_property_supported("border-left-width", "1px"));
    assert!(is_property_supported("border-inline-width", "thin 2px"));
    assert!(is_property_supported("border-block-start-width", "medium"));
    assert!(is_property_supported("border-top-left-radius", "5px"));
    assert!(is_property_supported("border-top-right-radius", "5px"));
    assert!(is_property_supported("border-bottom-right-radius", "5px"));
    assert!(is_property_supported("border-bottom-left-radius", "5px"));
    assert!(is_property_supported("margin", "1px auto 2% 3em"));
    assert!(is_property_supported("padding", "1px 2% 3em 4px"));
    assert!(is_property_supported("border-width", "thin 2px medium 4px"));
    assert!(is_property_supported("border-radius", "1px 2% 3em 4px"));
    assert!(is_property_supported("gap", "normal 2em"));
    assert!(is_property_supported("inset", "1px auto 2% 3em"));
    assert!(!is_property_supported("width", "invalid"));
    assert!(!is_property_supported("width", "thin"));
    assert!(!is_property_supported("padding", "-1px"));
    assert!(!is_property_supported("padding-top", "auto"));
    assert!(!is_property_supported("border-width", "10%"));
    assert!(!is_property_supported("border-top-width", "-1px"));
    assert!(!is_property_supported("border-inline-width", "thin 2px 3px"));
    assert!(!is_property_supported("border-radius", "-1px"));
    assert!(!is_property_supported("gap", "-1px"));
    assert!(!is_property_supported("column-gap", "-1px"));
    assert!(!is_property_supported("top", "thin"));
    assert!(!is_property_supported("padding-block", "1px 2px 3px"));
}

#[test]
fn test_property_supported_css_text_length_properties() {
    assert!(is_property_supported("line-height", "normal"));
    assert!(is_property_supported("line-height", "1.5"));
    assert!(is_property_supported("line-height", "24px"));
    assert!(is_property_supported("line-height", "120%"));
    assert!(is_property_supported("letter-spacing", "normal"));
    assert!(is_property_supported("letter-spacing", "-0.5em"));
    assert!(is_property_supported("word-spacing", "normal"));
    assert!(is_property_supported("word-spacing", "10%"));
    assert!(is_property_supported("text-indent", "-2em"));
    assert!(is_property_supported("text-decoration-thickness", "from-font"));
    assert!(is_property_supported("text-decoration-thickness", "2px"));
    assert!(is_property_supported("text-decoration-inset", "0.25em -0.5em"));
    assert!(is_property_supported("text-underline-offset", "auto"));
    assert!(is_property_supported("text-underline-offset", "-3px"));

    assert!(!is_property_supported("line-height", "-1"));
    assert!(!is_property_supported("line-height", "-2px"));
    assert!(!is_property_supported("line-height", "thin"));
    assert!(!is_property_supported("letter-spacing", "10%"));
    assert!(!is_property_supported("letter-spacing", "thin"));
    assert!(!is_property_supported("word-spacing", "auto"));
    assert!(!is_property_supported("word-spacing", "thin"));
    assert!(!is_property_supported("text-indent", "auto"));
    assert!(!is_property_supported("text-indent", "thin"));
    assert!(!is_property_supported("text-decoration-thickness", "-1px"));
    assert!(!is_property_supported("text-decoration-thickness", "thin"));
    assert!(!is_property_supported("text-decoration-inset", "auto"));
    assert!(!is_property_supported("text-decoration-inset", "thin"));
    assert!(!is_property_supported("text-underline-offset", "thin"));
}

#[test]
fn test_property_supported_font_feature_properties() {
    assert!(is_property_supported("font-size-adjust", "none"));
    assert!(is_property_supported("font-size-adjust", "0.5"));
    assert!(is_property_supported("font-size-adjust", "cap-height from-font"));
    assert!(is_property_supported("font-feature-settings", "normal"));
    assert!(is_property_supported("font-feature-settings", "\"liga\" off, 'kern' 2"));
    assert!(is_property_supported("font-variation-settings", "normal"));
    assert!(is_property_supported(
        "font-variation-settings",
        "\"wght\" 600.7, 'slnt' -12"
    ));
    assert!(is_property_supported(
        "font-variant-ligatures",
        "common-ligatures no-discretionary-ligatures"
    ));
    assert!(is_property_supported("font-variant-numeric", "tabular-nums"));
    assert!(is_property_supported("font-variant-caps", "small-caps"));
    assert!(is_property_supported("font-variant-east-asian", "jis78"));
    assert!(is_property_supported("font-variant-position", "sub"));
    assert!(is_property_supported("font-variant", "small-caps tabular-nums"));

    assert!(!is_property_supported("font-size-adjust", "auto"));
    assert!(!is_property_supported("font-size-adjust", "-0.5"));
    assert!(!is_property_supported("font-feature-settings", "sparkle"));
    assert!(!is_property_supported("font-feature-settings", "\"toolong\" 1"));
    assert!(!is_property_supported("font-variation-settings", "\"wght\" nan"));
    assert!(!is_property_supported(
        "font-variant-ligatures",
        "common-ligatures common-ligatures"
    ));
    assert!(!is_property_supported("font-variant-numeric", "sparkle"));
    assert!(!is_property_supported("font-variant-caps", "small-caps all-small-caps"));
    assert!(!is_property_supported("font-variant-east-asian", "sparkle"));
    assert!(!is_property_supported("font-variant-position", "baseline"));
    assert!(!is_property_supported("font-variant", "small-caps sparkle"));
}

#[test]
fn test_property_supported_transform() {
    assert!(is_property_supported("transform", "translate(10px, 20px)"));
    assert!(!is_property_supported("transform", "invalid-transform"));
}

#[test]
fn test_property_supported_background() {
    assert!(is_property_supported("background", "red"));
    assert!(is_property_supported(
        "background",
        "red url(img.png) no-repeat center / auto 10px"
    ));
    assert!(is_property_supported("background-image", "linear-gradient(red, blue)"));
    assert!(is_property_supported("background-image", "url(\"my image.png\"), none"));
    assert!(is_property_supported("background-repeat", "repeat-x, no-repeat"));
    assert!(is_property_supported("background-position", "left top, 10px 20px"));
    assert!(is_property_supported("background-size", "cover, auto 10px"));
    assert!(is_property_supported("background-attachment", "fixed"));
    assert!(is_property_supported("background-clip", "text"));
    assert!(is_property_supported("background-origin", "content-box"));
    assert!(is_property_supported("mask-image", "url(mask.png), none"));
    assert!(is_property_supported("mask-mode", "luminance"));

    assert!(!is_property_supported("background", "red url(my image.png)"));
    assert!(!is_property_supported("background", "center url(\"image.png\" extra)"));
    assert!(!is_property_supported("background-image", "url(my image.png)"));
    assert!(!is_property_supported("background-image", "url(\"image.png\" extra)"));
    assert!(!is_property_supported("background-repeat", "repeat, sparkle"));
    assert!(!is_property_supported("background-position", "left right"));
    assert!(!is_property_supported("background-size", "cover contain"));
    assert!(!is_property_supported("background-attachment", "sticky"));
    assert!(!is_property_supported("background-clip", "sparkle"));
    assert!(!is_property_supported("background-origin", "text"));
    assert!(!is_property_supported("mask-image", "url(mask.png) sparkle"));
    assert!(!is_property_supported("mask-mode", "alpha luminance"));
}

#[test]
fn test_property_supported_scroll_snap() {
    assert!(is_property_supported("scroll-snap-type", "mandatory"));
    assert!(is_property_supported("scroll-snap-align", "center"));
    assert!(is_property_supported("scroll-snap-stop", "always"));
    assert!(is_property_supported("scroll-margin", "1px -2px 3em 4px"));
    assert!(!is_property_supported("scroll-margin", "1px 2%"));
    assert!(!is_property_supported("scroll-margin-top", "auto"));
    assert!(!is_property_supported("scroll-margin-right", "10%"));
    assert!(!is_property_supported("scroll-margin-bottom", "thin"));
    assert!(!is_property_supported("scroll-margin-left", "infpx"));
    assert!(is_property_supported("scroll-margin-top", "10px"));
    assert!(is_property_supported("scroll-margin-right", "10px"));
    assert!(is_property_supported("scroll-margin-bottom", "10px"));
    assert!(is_property_supported("scroll-margin-left", "10px"));
    assert!(is_property_supported("scroll-padding", "auto 2px 3% 4px"));
    assert!(!is_property_supported("scroll-padding", "-1px"));
    assert!(!is_property_supported("scroll-padding", "thin"));
    assert!(!is_property_supported("scroll-padding", "calc(1px + 2px)"));
    assert!(!is_property_supported("scroll-padding-top", "-1px"));
    assert!(!is_property_supported("scroll-padding-right", "thin"));
    assert!(!is_property_supported("scroll-padding-bottom", "infpx"));
    assert!(is_property_supported("scroll-padding-top", "auto"));
    assert!(is_property_supported("scroll-padding-right", "10px"));
    assert!(is_property_supported("scroll-padding-bottom", "10px"));
    assert!(is_property_supported("scroll-padding-left", "10px"));
    assert!(!is_property_supported("scroll-snap-type", "invalid"));
}

#[test]
fn test_property_supported_container() {
    assert!(is_property_supported("container-type", "inline-size"));
    assert!(is_property_supported("container-name", "sidebar"));
    assert!(is_property_supported("container-name", "anything"));
    assert!(!is_property_supported("container-type", "invalid"));
}

#[test]
fn test_property_supported_unknown() {
    assert!(!is_property_supported("unknown-property", "value"));
    assert!(!is_property_supported("custom-var", "red"));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 无容器上下文
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_condition_no_context() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "100px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    assert!(!evaluate_container_condition(&rule, None));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 比较运算符全覆盖
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_comparison_gte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some(">=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
    let ctx_exact = ContainerContext::with_size(300.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx_exact)));
    let ctx_small = ContainerContext::with_size(200.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_small)));
}

#[test]
fn test_container_comparison_lt() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some("<".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(200.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
    let ctx_equal = ContainerContext::with_size(300.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_equal)));
}

#[test]
fn test_container_comparison_lte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "height".to_string(),
            value: "500px".to_string(),
            operator: Some("<=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 400.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — inline-size / block-size 特性
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_inline_size_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-inline-size".to_string(),
            value: "300px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_block_size_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-block-size".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 500.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_unknown_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "unknown-feature".to_string(),
            value: "300px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_invalid_length_value() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "not-a-length".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// PseudoElement 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_pseudo_element_always_false() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoElement(PseudoElementSelector::Standard(
                        "before".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :lang() 伪类返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_lang_no_lang_attribute_does_not_match() {
    // :lang(en) 对无 lang 属性（且无 lang 祖先）的元素不匹配（CSS 2.1 §5.11.4）。
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Lang(vec![
                        "en".to_string(),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 选择器条件
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_selector_condition() {
    use zero_css_parser::ast::SupportsCondition;

    // 有效选择器
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        "div".to_string()
    )));
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        ".class".to_string()
    )));
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        "#id".to_string()
    )));

    // 无效选择器（以组合器开头）
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "> div".to_string()
    )));
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "+ span".to_string()
    )));

    // 空选择器
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "".to_string()
    )));
}

// ═══════════════════════════════════════════════════════════════════════
// @container 规则在 collect_from_rules 中
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_rule_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    doc.append_child(root, span).unwrap();

    let container_rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![super::uncovered_paths::make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    }];

    // 有容器上下文且匹配
    let ctx = ContainerContext::with_size(500.0, 600.0);
    let decls = collect_matching_declarations_with_media(
        &doc,
        span,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        Some(&ctx),
    );
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].0, "color");

    // 无容器上下文 → 不匹配
    let decls_no_ctx = collect_matching_declarations(&doc, span, &stylesheets);
    assert!(decls_no_ctx.is_empty());

    // 容器上下文尺寸不足 → 不匹配
    let ctx_small = ContainerContext::with_size(200.0, 600.0);
    let decls_small = collect_matching_declarations_with_media(
        &doc,
        span,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        Some(&ctx_small),
    );
    assert!(decls_small.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 规则在 collect_from_rules 中
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_rule_in_collect() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{SupportsCondition, SupportsRule};
    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("display".to_string(), "grid".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![super::uncovered_paths::make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "grid".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].0, "display");
}

#[test]
fn test_supports_rule_not_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{SupportsCondition, SupportsRule};
    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("display".to_string(), "invalid-display".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![super::uncovered_paths::make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "grid".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 属性不存在 → matches_attribute 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_selector_missing_attr() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-missing".to_string(),
                        matcher: AttributeMatcher::Exists,
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 非元素节点 → matches_type 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_matches_type_text_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("hello");
    doc.append_child(root, text).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, text, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// Universal 选择器匹配
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_universal_selector_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :nth-child, :nth-last-child, :nth-of-type, :nth-last-of-type 集成
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let li3 = doc.create_element("li");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, li1).unwrap();
    doc.append_child(parent, li2).unwrap();
    doc.append_child(parent, li3).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        NthPattern { a: 2, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, li1, &sel)); // 2*0+1=1 ✓
    assert!(!matches_selector(&doc, li2, &sel)); // 2 ≠ 2n+1
    assert!(matches_selector(&doc, li3, &sel)); // 2*1+1=3 ✓
}

#[test]
fn test_nth_last_child_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let li3 = doc.create_element("li");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, li1).unwrap();
    doc.append_child(parent, li2).unwrap();
    doc.append_child(parent, li3).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastChild(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, li1, &sel)); // 第3个从末尾
    assert!(!matches_selector(&doc, li2, &sel)); // 第2个从末尾
    assert!(matches_selector(&doc, li3, &sel)); // 第1个从末尾 ✓
}

#[test]
fn test_nth_last_of_type_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let span = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    // p:nth-last-of-type(1) → 最后一个 p 类型
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, p1, &sel)); // 第2个 p 从末尾
    assert!(matches_selector(&doc, p2, &sel)); // 第1个 p 从末尾 ✓
}

// ═══════════════════════════════════════════════════════════════════════
// @media 规则带匹配上下文
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_rule_matching_context() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("span");
    doc.append_child(root, el).unwrap();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 400px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![super::uncovered_paths::make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let media_ctx = MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    };
    let decls = collect_matching_declarations_with_media(
        &doc,
        el,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        Some(&media_ctx),
        None,
    );
    assert_eq!(decls.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 非 media 的 AtRule（如 @font-face）无条件递归
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_non_media_at_rule_unconditional_recurse() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "font-face".to_string(),
            prelude: "".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![super::uncovered_paths::make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    // 非 @media 的通用 AtRule（未知 at-rule）body 不得作为样式应用（CSS：未知 at-rule
    // 整体忽略）。R2142 修正：旧实现无条件递归→body 内规则泄漏应用；现 body 不参与 cascade。
    // 注：真实 @font-face 经解析为 Rule::FontFace 专属变体，不进此通用 Rule::At 分支；
    // 此处构造通用 Rule::At(name="font-face") 模拟未知 at-rule 带 body 的场景。
    assert_eq!(decls.len(), 0, "unknown/non-media generic at-rule body must not apply");
}

// ═══════════════════════════════════════════════════════════════════════
// :not() 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_not_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(root, span).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, div, &sel)); // div 被 :not(div) 排除
    assert!(matches_selector(&doc, span, &sel)); // span 不被排除
}

// ═══════════════════════════════════════════════════════════════════════
// :is() 和 :where() 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                        super::uncovered_paths::make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, div, &sel));
    assert!(matches_selector(&doc, span, &sel));
    assert!(!matches_selector(&doc, p, &sel));
}

#[test]
fn test_where_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :first-of-type 和 :last-of-type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_first_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, p1, &sel)); // 第一个 p
    assert!(!matches_selector(&doc, p2, &sel)); // 第二个 p
    assert!(matches_selector(&doc, span, &sel)); // 唯一的 span
}

#[test]
fn test_last_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "last-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, p1, &sel)); // 第一个 p
    assert!(matches_selector(&doc, p2, &sel)); // 最后一个 p
    assert!(matches_selector(&doc, span, &sel)); // 唯一的 span
}

// ═══════════════════════════════════════════════════════════════════════
// 属性不存在 → matches_class 返回 false（非元素节点）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_class_on_text_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("hello");
    doc.append_child(root, text).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("test".to_string())],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, text, &sel));
}
