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
        // https://drafts.csswg.org/css-text-decor-3/#text-emphasis-property
        "text-emphasis" => shorthand_supported(property, value),
        // https://www.w3.org/TR/css-flexbox-1/#flex-property
        "flex" => flex_shorthand_supported(value),
        "flex-flow" => shorthand_supported(property, value),
        // https://drafts.csswg.org/css-color-adjust-1/#color-scheme-prop
        "color-scheme" => color_scheme_supported(value),
        // https://drafts.csswg.org/css-transitions-1/
        "transition-property" => transition_property_list_supported(value),
        "transition-duration" => non_negative_time_list_supported(value),
        "transition-timing-function" => timing_function_list_supported(value),
        "transition-delay" => time_list_supported(value),
        // https://drafts.csswg.org/css-animations-1/
        "animation-name" => animation_name_list_supported(value),
        "animation-duration" => non_negative_time_list_supported(value),
        "animation-timing-function" => timing_function_list_supported(value),
        "animation-delay" => time_list_supported(value),
        "animation-iteration-count" => animation_iteration_count_list_supported(value),
        "animation-direction" => animation_direction_list_supported(value),
        "animation-fill-mode" => animation_fill_mode_list_supported(value),
        "animation-play-state" => animation_play_state_list_supported(value),
        // https://drafts.csswg.org/css-contain-2/#contain-property
        "contain" => values::parse_contain(value).is_some(),
        // https://drafts.csswg.org/css-sizing-4/#intrinsic-size-override
        "contain-intrinsic-size" => contain_intrinsic_size_supported(value),
        "contain-intrinsic-width"
        | "contain-intrinsic-height"
        | "contain-intrinsic-inline-size"
        | "contain-intrinsic-block-size" => contain_intrinsic_longhand_supported(value),
        // https://drafts.csswg.org/css-align-3/#justify-items-property
        // https://drafts.csswg.org/css-align-3/#justify-self-property
        "justify-items" | "justify-self" => justify_items_supported(value),
        // https://www.w3.org/TR/css-position-3/#propdef-z-index
        "z-index" => crate::property::parse_z_index(value).is_some(),
        // https://drafts.csswg.org/css-sizing-4/#aspect-ratio
        "aspect-ratio" => aspect_ratio_supported(value),
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
        "column-rule" => shorthand_supported(property, value),
        // https://drafts.csswg.org/css-box-4/#margin-trim
        "margin-trim" => values::parse_margin_trim(value).is_some(),
        // https://drafts.csswg.org/css-backgrounds-3/#border-style
        "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
            crate::property::parse_border_style(value).is_some()
        }
        // https://drafts.csswg.org/css-logical-1/#border-shorthands
        "border-inline-style" | "border-block-style" => border_axis_style_supported(value),
        "border-inline-start-style"
        | "border-inline-end-style"
        | "border-block-start-style"
        | "border-block-end-style" => crate::property::parse_border_style(value).is_some(),
        "border-inline-color" | "border-block-color" => border_axis_color_supported(value),
        "border-inline-start-color"
        | "border-inline-end-color"
        | "border-block-start-color"
        | "border-block-end-color" => values::parse_color(value).is_some(),
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
        // https://drafts.csswg.org/css-ui-4/#outline-props
        "outline" => shorthand_supported(property, value),
        _ => return None,
    })
}

fn shorthand_supported(property: &str, value: &str) -> bool {
    if value.trim().is_empty() {
        return false;
    }
    let decl = vec![(property.to_string(), value.to_string(), false, (0, 0, 0))];
    !crate::shorthand::expand_shorthands(&decl).is_empty()
}

fn flex_shorthand_supported(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("auto") {
        return true;
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [single] => flex_number_supported(single) || crate::property::parse_flex_basis(single).is_some(),
        [grow, second] => {
            flex_number_supported(grow)
                && (flex_number_supported(second) || crate::property::parse_flex_basis(second).is_some())
        }
        [grow, shrink, basis] => {
            flex_number_supported(grow)
                && flex_number_supported(shrink)
                && crate::property::parse_flex_basis(basis).is_some()
        }
        _ => false,
    }
}

fn flex_number_supported(value: &str) -> bool {
    value
        .parse::<f64>()
        .is_ok_and(|number| number.is_finite() && number >= 0.0)
}

fn color_scheme_supported(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    if parts.len() == 1 && parts[0].eq_ignore_ascii_case("normal") {
        return true;
    }

    let mut has_scheme = false;
    let mut has_only = false;
    for part in parts {
        let lower = part.to_ascii_lowercase();
        match lower.as_str() {
            "normal" => return false,
            "only" => {
                if has_only {
                    return false;
                }
                has_only = true;
            }
            "inherit" | "initial" | "unset" | "revert" | "revert-layer" => return false,
            "light" | "dark" => has_scheme = true,
            _ => {
                if !css_ident_supported(part) {
                    return false;
                }
                has_scheme = true;
            }
        }
    }
    has_scheme
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

fn contain_intrinsic_size_supported(value: &str) -> bool {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return true;
    }
    let mut lengths = 0;
    for token in value.split_whitespace() {
        if token.eq_ignore_ascii_case("auto") {
            continue;
        }
        if !contain_intrinsic_length_supported(token) {
            return false;
        }
        lengths += 1;
    }
    matches!(lengths, 1 | 2)
}

fn contain_intrinsic_longhand_supported(value: &str) -> bool {
    let value = value.trim();
    let value = if value.len() >= 5
        && value.as_bytes()[..4].eq_ignore_ascii_case(b"auto")
        && value.as_bytes()[4].is_ascii_whitespace()
    {
        value[4..].trim()
    } else {
        value
    };
    value.eq_ignore_ascii_case("none") || contain_intrinsic_length_supported(value)
}

fn contain_intrinsic_length_supported(value: &str) -> bool {
    if matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    matches!(
        values::parse_length(value),
        Some(values::LengthValue::Px(v))
            | Some(values::LengthValue::Em(v))
            | Some(values::LengthValue::Ex(v))
            | Some(values::LengthValue::Rex(v))
            | Some(values::LengthValue::Cap(v))
            | Some(values::LengthValue::Rcap(v))
            | Some(values::LengthValue::Rem(v))
            | Some(values::LengthValue::Vh(v))
            | Some(values::LengthValue::Vw(v))
            | Some(values::LengthValue::Vmin(v))
            | Some(values::LengthValue::Vmax(v))
            | Some(values::LengthValue::Ch(v))
            | Some(values::LengthValue::Rch(v))
            | Some(values::LengthValue::Ic(v))
            | Some(values::LengthValue::Ric(v))
            if v.is_finite() && v >= 0.0
    )
}

fn justify_items_supported(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "auto" | "normal" | "start" | "end" | "center" | "stretch" | "baseline" | "left" | "right"
    )
}

fn border_axis_style_supported(value: &str) -> bool {
    axis_pair_supported(value, crate::property::parse_border_style)
}

fn border_axis_color_supported(value: &str) -> bool {
    axis_pair_supported(value, values::parse_color)
}

fn axis_pair_supported<T>(value: &str, parse_component: fn(&str) -> Option<T>) -> bool {
    let mut count = 0;
    for part in value.split_whitespace() {
        count += 1;
        if count > 2 || parse_component(part).is_none() {
            return false;
        }
    }
    count > 0
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

fn aspect_ratio_supported(value: &str) -> bool {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return true;
    }
    let ratio = if value.len() >= 5
        && value.as_bytes()[..4].eq_ignore_ascii_case(b"auto")
        && value.as_bytes()[4].is_ascii_whitespace()
    {
        value[4..].trim()
    } else {
        value
    };
    parse_aspect_ratio_value(ratio).is_some()
}

fn parse_aspect_ratio_value(value: &str) -> Option<f32> {
    if let Some(slash_pos) = value.find('/') {
        let width: f32 = value[..slash_pos].trim().parse().ok()?;
        let height: f32 = value[slash_pos + 1..].trim().parse().ok()?;
        if !width.is_finite() || !height.is_finite() || height == 0.0 {
            return None;
        }
        Some(width / height)
    } else {
        value.parse().ok()
    }
    .filter(|ratio: &f32| ratio.is_finite())
}

fn comma_list_supported(value: &str, mut is_valid: impl FnMut(&str) -> bool) -> bool {
    let Some(items) = split_top_level_commas(value) else {
        return false;
    };
    !items.is_empty() && items.into_iter().all(|item| is_valid(item.trim()))
}

fn split_top_level_commas(value: &str) -> Option<Vec<&str>> {
    let mut start = 0;
    let mut depth = 0i32;
    let mut items = Vec::new();
    for (index, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                let item = value[start..index].trim();
                if item.is_empty() {
                    return None;
                }
                items.push(item);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let item = value[start..].trim();
    if item.is_empty() {
        return None;
    }
    items.push(item);
    Some(items)
}

fn non_negative_time_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| {
        values::parse_animation_duration(item)
            .is_some_and(|time| matches!(time, values::AnimationDurationValue::Time(_, _)))
    })
}

fn time_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_time(item).is_some())
}

fn timing_function_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_timing_function(item).is_some())
}

fn animation_iteration_count_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_animation_iteration_count(item).is_some())
}

fn animation_direction_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_animation_direction(item).is_some())
}

fn animation_fill_mode_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_animation_fill_mode(item).is_some())
}

fn animation_play_state_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_animation_play_state(item).is_some())
}

fn animation_name_list_supported(value: &str) -> bool {
    comma_list_supported(value, |item| values::parse_animation_name(item).is_some())
}

fn transition_property_list_supported(value: &str) -> bool {
    comma_list_supported(value, transition_property_ident_supported)
}

fn transition_property_ident_supported(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("all") || value.eq_ignore_ascii_case("none") || css_ident_supported(value)
}

fn css_ident_supported(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first == '-' {
        let Some(second) = chars.next() else {
            return false;
        };
        if second != '-' && !css_name_start_supported(second) {
            return false;
        }
    } else if !css_name_start_supported(first) {
        return false;
    }
    chars.all(css_name_char_supported)
}

fn css_name_start_supported(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic() || !c.is_ascii()
}

fn css_name_char_supported(c: char) -> bool {
    css_name_start_supported(c) || c.is_ascii_digit() || c == '-'
}
