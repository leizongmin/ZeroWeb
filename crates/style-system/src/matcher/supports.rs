//! `@supports` declaration-condition property checks.

use zero_css_parser::values::{self, PageBreakValue};

/// Checks extended visual/layout declarations that already have direct apply parsers.
///
/// https://www.w3.org/TR/css-conditional-3/#at-supports
pub(super) fn extended_visual_or_layout_property_supported(property: &str, value: &str) -> Option<bool> {
    Some(match property {
        // https://drafts.csswg.org/css-ui-4/#appearance-switching
        "appearance" => values::parse_appearance(value).is_some(),
        "accent-color" => values::parse_accent_color(value).is_some(),
        "caret-color" => values::parse_caret_color(value).is_some(),
        // https://drafts.csswg.org/compositing-1/#mix-blend-mode
        "mix-blend-mode" => values::parse_mix_blend_mode(value).is_some(),
        "isolation" => values::parse_isolation(value).is_some(),
        // https://drafts.csswg.org/css-overflow-4/
        "scrollbar-width" => values::parse_scrollbar_width(value).is_some(),
        "scrollbar-gutter" => values::parse_scrollbar_gutter(value).is_some(),
        // https://drafts.csswg.org/css-overflow-3/#overflow-clip-margin
        "overflow-clip-margin" => values::parse_overflow_clip_margin(value).is_some(),
        // https://drafts.csswg.org/css-text-4/
        "text-wrap" => values::parse_text_wrap(value).is_some(),
        "hyphens" => values::parse_hyphens(value).is_some(),
        "overflow-wrap" => values::parse_overflow_wrap(value).is_some(),
        "tab-size" => values::parse_tab_size(value).is_some(),
        // https://drafts.csswg.org/css-overflow-4/#line-clamp
        "line-clamp" | "-webkit-line-clamp" => values::parse_line_clamp(value).is_some(),
        // https://drafts.csswg.org/css-images-4/#the-object-fit
        "object-fit" => values::parse_object_fit(value).is_some(),
        "object-position" => values::parse_background_position(value).is_some(),
        // https://drafts.fxtf.org/css-masking-1/#the-mask-image
        "mask-image" => values::parse_mask_image_layers(value).is_some(),
        "mask-mode" => values::parse_mask_mode(value).is_some(),
        // https://drafts.fxtf.org/filter-effects-1/
        "filter" | "backdrop-filter" => values::parse_filter_list(value).is_some(),
        // https://drafts.csswg.org/css-backgrounds-3/#shadow-layers
        "text-shadow" => values::parse_text_shadow_list(value).is_some(),
        "box-shadow" => values::parse_box_shadow_list(value).is_some(),
        // https://drafts.csswg.org/css-contain-2/#contain-property
        "contain" => values::parse_contain(value).is_some(),
        // https://drafts.csswg.org/css2/#page-break-props
        "page-break-before" | "page-break-after" => values::parse_page_break(value).is_some(),
        "page-break-inside" => matches!(
            values::parse_page_break(value),
            Some(PageBreakValue::Auto | PageBreakValue::Avoid)
        ),
        // https://drafts.csswg.org/css-break-4/
        "box-decoration-break" => values::parse_box_decoration_break(value).is_some(),
        "break-inside" => values::parse_break_inside(value).is_some(),
        "break-before" => values::parse_break_before(value).is_some(),
        "break-after" => values::parse_break_after(value).is_some(),
        // https://drafts.csswg.org/css-images-3/#the-image-rendering
        "image-rendering" => values::parse_image_rendering(value).is_some(),
        // https://drafts.csswg.org/css-multicol-1/
        "columns" => columns_supported(value),
        "column-count" => values::parse_column_count(value).is_some(),
        "column-width" => values::parse_column_width(value).is_some(),
        "column-fill" => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "balance" | "balance-all" | "auto"
        ),
        "column-span" => matches!(value.trim().to_ascii_lowercase().as_str(), "none" | "all"),
        "column-rule-width" => values::parse_column_rule_width(value).is_some(),
        "column-rule-style" => values::parse_column_rule_style(value).is_some(),
        "column-rule-color" => values::parse_color(value).is_some(),
        // https://drafts.csswg.org/css-backgrounds-3/#border-images
        "border-image" => crate::shorthand::border_image_shorthand_supported(value),
        "border-image-source" => values::parse_border_image_source(value).is_some(),
        "border-image-slice" => values::parse_border_image_slice(value).is_some(),
        "border-image-width" => values::parse_border_image_width(value).is_some(),
        "border-image-repeat" => values::parse_border_image_repeat(value).is_some(),
        "border-image-outset" => values::parse_border_image_outset(value).is_some(),
        // https://drafts.csswg.org/css-transforms-1/#transform-origin-property
        "transform-origin" | "perspective-origin" => origin_pair_supported(value),
        "perspective" => perspective_supported(value),
        "transform-style" => matches!(value.trim().to_ascii_lowercase().as_str(), "flat" | "preserve-3d"),
        "backface-visibility" => matches!(value.trim().to_ascii_lowercase().as_str(), "visible" | "hidden"),
        // https://drafts.csswg.org/css-overscroll-1/#overscroll-behavior-properties
        "overscroll-behavior-x" | "overscroll-behavior-y" => values::parse_overscroll_behavior(value).is_some(),
        // https://w3c.github.io/pointerevents/#the-touch-action-css-property
        "touch-action" => values::parse_touch_action(value).is_some(),
        "pointer-events" => values::parse_pointer_events(value).is_some(),
        "user-select" => values::parse_user_select(value).is_some(),
        // https://drafts.csswg.org/css-will-change-1/#will-change
        "will-change" => values::parse_will_change_list(value).is_some(),
        // https://drafts.fxtf.org/css-masking-1/#the-clip-path
        "clip-path" => values::parse_clip_path(value).is_some(),
        // https://www.w3.org/TR/CSS22/visufx.html#clipping
        "clip" => values::parse_clip(value).is_some(),
        _ => return None,
    })
}

fn columns_supported(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [single] => values::parse_column_count(single).is_some() || values::parse_column_width(single).is_some(),
        [first, second] => {
            (values::parse_column_count(first).is_some() && values::parse_column_width(second).is_some())
                || (values::parse_column_width(first).is_some() && values::parse_column_count(second).is_some())
        }
        _ => false,
    }
}

fn origin_pair_supported(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [single] => origin_component_kind(single).is_some(),
        [first, second] => {
            let first = origin_component_kind(first);
            let second = origin_component_kind(second);
            matches!(
                (first, second),
                (
                    Some(OriginComponentKind::Horizontal),
                    Some(OriginComponentKind::Vertical)
                ) | (
                    Some(OriginComponentKind::Horizontal),
                    Some(OriginComponentKind::LengthPercentage)
                ) | (
                    Some(OriginComponentKind::LengthPercentage),
                    Some(OriginComponentKind::Vertical)
                ) | (
                    Some(OriginComponentKind::Vertical),
                    Some(OriginComponentKind::Horizontal)
                ) | (
                    Some(OriginComponentKind::Vertical),
                    Some(OriginComponentKind::LengthPercentage)
                ) | (Some(OriginComponentKind::Horizontal), Some(OriginComponentKind::Center))
                    | (Some(OriginComponentKind::Center), Some(OriginComponentKind::Vertical))
                    | (Some(OriginComponentKind::Center), Some(OriginComponentKind::Horizontal))
                    | (Some(OriginComponentKind::Vertical), Some(OriginComponentKind::Center))
                    | (
                        Some(OriginComponentKind::LengthPercentage),
                        Some(OriginComponentKind::LengthPercentage)
                    )
                    | (
                        Some(OriginComponentKind::LengthPercentage),
                        Some(OriginComponentKind::Center)
                    )
                    | (
                        Some(OriginComponentKind::Center),
                        Some(OriginComponentKind::LengthPercentage)
                    )
                    | (Some(OriginComponentKind::Center), Some(OriginComponentKind::Center))
            )
        }
        _ => false,
    }
}

fn perspective_supported(value: &str) -> bool {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return true;
    }
    let Some(length) = parse_length_or_math(value) else {
        return false;
    };
    match length {
        values::LengthValue::Px(v)
        | values::LengthValue::Em(v)
        | values::LengthValue::Ex(v)
        | values::LengthValue::Rex(v)
        | values::LengthValue::Cap(v)
        | values::LengthValue::Rcap(v)
        | values::LengthValue::Rem(v)
        | values::LengthValue::Vh(v)
        | values::LengthValue::Vw(v)
        | values::LengthValue::Vmin(v)
        | values::LengthValue::Vmax(v)
        | values::LengthValue::Ch(v)
        | values::LengthValue::Rch(v)
        | values::LengthValue::Ic(v)
        | values::LengthValue::Ric(v) => v.is_finite() && v >= 0.0,
        values::LengthValue::Calc(_) => true,
        _ => false,
    }
}

#[derive(Clone, Copy)]
enum OriginComponentKind {
    Horizontal,
    Vertical,
    Center,
    LengthPercentage,
}

fn origin_component_kind(value: &str) -> Option<OriginComponentKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" | "right" => Some(OriginComponentKind::Horizontal),
        "top" | "bottom" => Some(OriginComponentKind::Vertical),
        "center" => Some(OriginComponentKind::Center),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content" => None,
        _ => match parse_length_or_math(value)? {
            values::LengthValue::Px(v)
            | values::LengthValue::Em(v)
            | values::LengthValue::Ex(v)
            | values::LengthValue::Rex(v)
            | values::LengthValue::Cap(v)
            | values::LengthValue::Rcap(v)
            | values::LengthValue::Rem(v)
            | values::LengthValue::Vh(v)
            | values::LengthValue::Vw(v)
            | values::LengthValue::Vmin(v)
            | values::LengthValue::Vmax(v)
            | values::LengthValue::Ch(v)
            | values::LengthValue::Rch(v)
            | values::LengthValue::Ic(v)
            | values::LengthValue::Ric(v)
            | values::LengthValue::Percentage(v)
                if v.is_finite() =>
            {
                Some(OriginComponentKind::LengthPercentage)
            }
            values::LengthValue::Calc(_) => Some(OriginComponentKind::LengthPercentage),
            _ => None,
        },
    }
}

fn parse_length_or_math(value: &str) -> Option<values::LengthValue> {
    values::parse_length(value).or_else(|| {
        values::parse_math_function(value)
            .map(Box::new)
            .map(values::LengthValue::Calc)
    })
}
