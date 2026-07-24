//! CSS 高级属性值应用（Transforms、Animations、Scroll Snap、Container 等）。
//!
//! 将 ComputedStyle 的高级属性匹配从 apply.rs 拆分出来，保持文件在 2000 行以内。

use super::apply::parse_length_or_math;
use super::computed_style::ComputedStyle;
use super::parse::*;
use super::types::*;
use zero_css_parser::values;

/// 将高级 CSS 属性字符串值设置到 ComputedStyle。
///
/// 处理 Transforms、Transitions、Animations、Scroll Snap、Container、
/// Counter、Content、Break、Column、Appearance 等高级属性。
/// 返回 true 表示成功设置。
/// R1417：把 background-position 的长度值（任意单位）解析为 px。
/// em/rem/ex/ch 按元素 font-size 解析；vh/vw/vmin/vmax 按 viewport（apply 无 viewport
/// 上下文，传 None 用 resolve_length 默认，vh/vw 在 bg-position 极罕见）。
fn resolve_bg_pos_length(lv: zero_css_parser::values::LengthValue, style: &ComputedStyle) -> f32 {
    let fs = match &style.font_size {
        zero_css_parser::values::LengthValue::Px(v) => *v,
        _ => 16.0,
    };
    crate::computed::resolve_length(&lv, fs, None, None) as f32
}

pub fn apply_advanced_property_value(style: &mut ComputedStyle, property: &str, value: &str) -> bool {
    let value = value.trim();
    match property {
        // ── Transforms ──
        "transform" => {
            if let Some(v) = values::parse_transform(value) {
                style.transform = v;
                return true;
            }
        }
        "transform-origin" => {
            // 解析 "x y" 或单个值（y 默认为 50%）
            let parts: Vec<&str> = value.split_whitespace().collect();
            if let Some(x) = parse_length_or_math(parts[0]) {
                style.transform_origin_x = x;
                style.transform_origin_y = if parts.len() > 1 {
                    parse_length_or_math(parts[1]).unwrap_or(LengthValue::Percentage(50.0))
                } else {
                    LengthValue::Percentage(50.0)
                };
                return true;
            }
        }
        "perspective" => {
            if value.eq_ignore_ascii_case("none") {
                style.perspective = LengthValue::Px(0.0);
                return true;
            }
            if let Some(v) = parse_length_or_math(value) {
                style.perspective = v;
                return true;
            }
        }
        "perspective-origin" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if let Some(x) = parse_length_or_math(parts[0]) {
                style.perspective_origin_x = x;
                style.perspective_origin_y = if parts.len() > 1 {
                    parse_length_or_math(parts[1]).unwrap_or(LengthValue::Percentage(50.0))
                } else {
                    LengthValue::Percentage(50.0)
                };
                return true;
            }
        }
        "transform-style" => match value.trim() {
            "flat" => {
                style.transform_style = TransformStyleValue::Flat;
                return true;
            }
            "preserve-3d" => {
                style.transform_style = TransformStyleValue::Preserve3d;
                return true;
            }
            _ => {}
        },
        "backface-visibility" => match value.trim() {
            "visible" => {
                style.backface_visibility = BackfaceVisibilityValue::Visible;
                return true;
            }
            "hidden" => {
                style.backface_visibility = BackfaceVisibilityValue::Hidden;
                return true;
            }
            _ => {}
        },
        // ── Transitions ──
        "transition-property" => {
            // transition-property: none 表示无过渡属性，结果为空列表
            style.transition_property = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| s != "none")
                .collect();
            return true;
        }
        "transition-duration" => {
            let durations = value.split(',').filter_map(|s| values::parse_time(s.trim())).collect();
            style.transition_duration = durations;
            return true;
        }
        "transition-timing-function" => {
            // 简化解析：按逗号分割，但注意 cubic-bezier() 和 steps() 内部也有逗号
            let funcs = parse_comma_separated_timing_functions(value);
            if !funcs.is_empty() {
                style.transition_timing_function = funcs;
                return true;
            }
        }
        "transition-delay" => {
            let delays = value.split(',').filter_map(|s| values::parse_time(s.trim())).collect();
            style.transition_delay = delays;
            return true;
        }

        // ── 逻辑属性 ──
        // margin/padding/inset 的 inline/block × start/end 逻辑属性，按元素 computed
        // writing-mode 映射物理边（CSS Logical Properties §1 + Writing Modes §6）。
        // R143 起支持，R1049 由静态 horizontal-tb 升级为 writing-mode-aware：horizontal-tb
        // 下与原静态映射字节一致（零回归），vertical-rl/lr 下映射到正确物理边。
        // inline 轴 direction 暂按 ltr（vertical 模式 inline-start=top）。
        "margin-block-start" => return apply_logical_margin(style, false, true, value),
        "margin-block-end" => return apply_logical_margin(style, false, false, value),
        "margin-inline-start" => return apply_logical_margin(style, true, true, value),
        "margin-inline-end" => return apply_logical_margin(style, true, false, value),
        "padding-block-start" => return apply_logical_padding(style, false, true, value),
        "padding-block-end" => return apply_logical_padding(style, false, false, value),
        "padding-inline-start" => return apply_logical_padding(style, true, true, value),
        "padding-inline-end" => return apply_logical_padding(style, true, false, value),
        "inset-block-start" => return apply_logical_inset(style, false, true, value),
        "inset-block-end" => return apply_logical_inset(style, false, false, value),
        "inset-inline-start" => return apply_logical_inset(style, true, true, value),
        "inset-inline-end" => return apply_logical_inset(style, true, false, value),

        // ── border 逻辑属性 longhand（CSS Logical Properties §3 + Writing Modes §6）──
        // border-inline-start / border-block-end 等简写经 shorthand 模块展开为这些
        // logical longhand。此处按元素 computed writing-mode 映射到物理边。
        //   horizontal-tb（ltr）：inline-start=left, inline-end=right,
        //                        block-start=top, block-end=bottom
        //   vertical-rl：inline-start=top, inline-end=bottom,
        //                block-start=right, block-end=left
        //   vertical-lr：inline-start=top, inline-end=bottom,
        //                block-start=left, block-end=right
        // inline 轴的 direction 暂按 ltr（vertical 模式 inline-start=top）；这是
        // logical-props-001 等 vertical-rl 用例的预期。
        "border-inline-start-width"
        | "border-inline-end-width"
        | "border-block-start-width"
        | "border-block-end-width" => {
            if let Some(v) = parse_length_or_math(value) {
                if let LengthValue::Px(px) = v
                    && px < 0.0
                {
                    return false;
                }
                let side = logical_border_physical_side(property, &style.writing_mode);
                set_border_width_field(style, side, v);
                return true;
            }
        }
        "border-inline-start-style"
        | "border-inline-end-style"
        | "border-block-start-style"
        | "border-block-end-style" => {
            if let Some(v) = parse_border_style(value) {
                let side = logical_border_physical_side(property, &style.writing_mode);
                set_border_style_field(style, side, v);
                return true;
            }
        }
        "border-inline-start-color"
        | "border-inline-end-color"
        | "border-block-start-color"
        | "border-block-end-color" => {
            if let Some(v) = values::parse_color(value) {
                let side = logical_border_physical_side(property, &style.writing_mode);
                set_border_color_field(style, side, v);
                return true;
            }
        }

        // ── Animation 属性 ──
        "animation-name" => {
            // animation-name: none 表示无动画，结果为空列表
            style.animation_name = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| s != "none")
                .collect();
            return true;
        }
        "animation-duration" => {
            style.animation_duration = value.split(',').filter_map(|s| values::parse_time(s.trim())).collect();
            return true;
        }
        "animation-timing-function" => {
            let funcs = parse_comma_separated_timing_functions(value);
            if !funcs.is_empty() {
                style.animation_timing_function = funcs;
                return true;
            }
        }
        "animation-delay" => {
            style.animation_delay = value.split(',').filter_map(|s| values::parse_time(s.trim())).collect();
            return true;
        }
        "animation-iteration-count" => {
            let counts = value
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    if s.eq_ignore_ascii_case("infinite") {
                        None
                    } else {
                        s.parse::<f64>().ok()
                    }
                })
                .collect();
            style.animation_iteration_count = counts;
            return true;
        }
        "animation-direction" => {
            let dirs: Vec<_> = value
                .split(',')
                .filter_map(|s| values::parse_animation_direction(s.trim()))
                .collect();
            if !dirs.is_empty() {
                style.animation_direction = dirs;
                return true;
            }
        }
        "animation-fill-mode" => {
            let modes: Vec<_> = value
                .split(',')
                .filter_map(|s| values::parse_animation_fill_mode(s.trim()))
                .collect();
            if !modes.is_empty() {
                style.animation_fill_mode = modes;
                return true;
            }
        }
        "animation-play-state" => {
            let states: Vec<_> = value
                .split(',')
                .filter_map(|s| values::parse_animation_play_state(s.trim()))
                .collect();
            if !states.is_empty() {
                style.animation_play_state = states;
                return true;
            }
        }
        // ── Scroll Snap 属性 ──
        "scroll-snap-type" => {
            if let Some(v) = parse_scroll_snap_type_computed(value) {
                style.scroll_snap_type = v;
                return true;
            }
        }
        "scroll-snap-align" => {
            if let Some(v) = parse_scroll_snap_align_computed(value) {
                style.scroll_snap_align = v;
                return true;
            }
        }
        "scroll-snap-stop" => {
            if let Some(v) = parse_scroll_snap_stop_computed(value) {
                style.scroll_snap_stop = v;
                return true;
            }
        }
        "scroll-margin-top" => {
            if let Some(v) = parse_length_or_math(value) {
                style.scroll_margin_top = resolve_length_to_px(v);
                return true;
            }
        }
        "scroll-margin-right" => {
            if let Some(v) = parse_length_or_math(value) {
                style.scroll_margin_right = resolve_length_to_px(v);
                return true;
            }
        }
        "scroll-margin-bottom" => {
            if let Some(v) = parse_length_or_math(value) {
                style.scroll_margin_bottom = resolve_length_to_px(v);
                return true;
            }
        }
        "scroll-margin-left" => {
            if let Some(v) = parse_length_or_math(value) {
                style.scroll_margin_left = resolve_length_to_px(v);
                return true;
            }
        }
        "scroll-padding-top" => {
            if let Some(v) = parse_scroll_padding(value) {
                style.scroll_padding_top = v;
                return true;
            }
        }
        "scroll-padding-right" => {
            if let Some(v) = parse_scroll_padding(value) {
                style.scroll_padding_right = v;
                return true;
            }
        }
        "scroll-padding-bottom" => {
            if let Some(v) = parse_scroll_padding(value) {
                style.scroll_padding_bottom = v;
                return true;
            }
        }
        "scroll-padding-left" => {
            if let Some(v) = parse_scroll_padding(value) {
                style.scroll_padding_left = v;
                return true;
            }
        }
        // ── Container Query 属性 ──
        "container-type" => {
            if let Some(v) = parse_container_type_computed(value) {
                style.container_type = v;
                return true;
            }
        }
        "container-name" => {
            let trimmed = value.trim();
            if trimmed.eq_ignore_ascii_case("none") {
                style.container_name = None;
            } else {
                style.container_name = Some(trimmed.to_string());
            }
            return true;
        }
        // ── Counters 属性 ──
        "counter-reset" => {
            if let Some(v) = values::parse_counter_list(value) {
                style.counter_reset = v;
                return true;
            }
        }
        "counter-increment" => {
            if let Some(v) = values::parse_counter_list(value) {
                style.counter_increment = v;
                return true;
            }
        }
        "counter-set" => {
            if let Some(v) = values::parse_counter_set(value) {
                style.counter_set = match v {
                    values::CounterSetValue::None => vec![],
                    values::CounterSetValue::Actions(actions) => actions,
                };
                return true;
            }
        }
        // ── Content 属性 ──
        "content" => {
            if let Some(v) = values::parse_content(value) {
                style.content = match v {
                    ContentValue::Normal => ContentComputedValue::Normal,
                    ContentValue::None => ContentComputedValue::None,
                    ContentValue::String(s) => ContentComputedValue::String(s),
                    ContentValue::Attr(a) => ContentComputedValue::Attr(a),
                    ContentValue::Counter { name, style } => ContentComputedValue::Counter { name, style },
                    ContentValue::Url(u) => ContentComputedValue::Url(u),
                };
                return true;
            }
        }
        // ── Quotes 属性 ──
        "quotes" => {
            if let Some(v) = values::parse_quotes(value) {
                style.quotes = match v {
                    QuotesValue::None => QuotesComputedValue::None,
                    QuotesValue::Auto => QuotesComputedValue::Auto,
                    QuotesValue::Pairs(p) => QuotesComputedValue::Pairs(p),
                };
                return true;
            }
        }
        // ── Page Break 属性 ──
        "page-break-before" => {
            if let Some(v) = values::parse_page_break(value) {
                style.page_break_before = match v {
                    zero_css_parser::values::PageBreakValue::Auto => PageBreakValue::Auto,
                    zero_css_parser::values::PageBreakValue::Always => PageBreakValue::Always,
                    zero_css_parser::values::PageBreakValue::Avoid => PageBreakValue::Avoid,
                    zero_css_parser::values::PageBreakValue::Left => PageBreakValue::Left,
                    zero_css_parser::values::PageBreakValue::Right => PageBreakValue::Right,
                };
                return true;
            }
        }
        "page-break-after" => {
            if let Some(v) = values::parse_page_break(value) {
                style.page_break_after = match v {
                    zero_css_parser::values::PageBreakValue::Auto => PageBreakValue::Auto,
                    zero_css_parser::values::PageBreakValue::Always => PageBreakValue::Always,
                    zero_css_parser::values::PageBreakValue::Avoid => PageBreakValue::Avoid,
                    zero_css_parser::values::PageBreakValue::Left => PageBreakValue::Left,
                    zero_css_parser::values::PageBreakValue::Right => PageBreakValue::Right,
                };
                return true;
            }
        }
        "page-break-inside" => {
            if let Some(v) = values::parse_page_break(value) {
                style.page_break_inside = match v {
                    zero_css_parser::values::PageBreakValue::Auto => PageBreakValue::Auto,
                    zero_css_parser::values::PageBreakValue::Avoid => PageBreakValue::Avoid,
                    _ => return false,
                };
                return true;
            }
        }
        // ── BoxDecorationBreak 属性 ──
        "box-decoration-break" => {
            if let Some(v) = values::parse_box_decoration_break(value) {
                style.box_decoration_break = match v {
                    zero_css_parser::values::BoxDecorationBreakValue::Slice => BoxDecorationBreakValue::Slice,
                    zero_css_parser::values::BoxDecorationBreakValue::Clone => BoxDecorationBreakValue::Clone,
                };
                return true;
            }
        }
        // ── ImageRendering 属性 ──
        "image-rendering" => {
            if let Some(v) = values::parse_image_rendering(value) {
                style.image_rendering = match v {
                    zero_css_parser::values::ImageRenderingValue::Auto => ImageRenderingValue::Auto,
                    zero_css_parser::values::ImageRenderingValue::Smooth => ImageRenderingValue::Smooth,
                    zero_css_parser::values::ImageRenderingValue::HighQuality => ImageRenderingValue::HighQuality,
                    zero_css_parser::values::ImageRenderingValue::Pixelated => ImageRenderingValue::Pixelated,
                    zero_css_parser::values::ImageRenderingValue::CrispEdges => ImageRenderingValue::CrispEdges,
                };
                return true;
            }
        }
        // ── Isolation 属性 ──
        "isolation" => {
            if let Some(v) = values::parse_isolation(value) {
                style.isolation = match v {
                    zero_css_parser::values::IsolationValue::Auto => IsolationValue::Auto,
                    zero_css_parser::values::IsolationValue::Isolate => IsolationValue::Isolate,
                };
                return true;
            }
        }
        // ── Break 属性 ──
        "break-inside" => {
            if let Some(v) = values::parse_break_inside(value) {
                style.break_inside = match v {
                    zero_css_parser::values::BreakInsideValue::Auto => BreakInsideValue::Auto,
                    zero_css_parser::values::BreakInsideValue::Avoid => BreakInsideValue::Avoid,
                    zero_css_parser::values::BreakInsideValue::AvoidPage => BreakInsideValue::AvoidPage,
                    zero_css_parser::values::BreakInsideValue::AvoidColumn => BreakInsideValue::AvoidColumn,
                };
                return true;
            }
        }
        "break-before" => {
            if let Some(v) = values::parse_break_before(value) {
                style.break_before = match v {
                    zero_css_parser::values::BreakValue::Auto => BreakValue::Auto,
                    zero_css_parser::values::BreakValue::Avoid => BreakValue::Avoid,
                    zero_css_parser::values::BreakValue::Column => BreakValue::Column,
                    zero_css_parser::values::BreakValue::Page => BreakValue::Page,
                    zero_css_parser::values::BreakValue::AvoidPage => BreakValue::AvoidPage,
                    zero_css_parser::values::BreakValue::AvoidColumn => BreakValue::AvoidColumn,
                };
                return true;
            }
        }
        "break-after" => {
            if let Some(v) = values::parse_break_after(value) {
                style.break_after = match v {
                    zero_css_parser::values::BreakValue::Auto => BreakValue::Auto,
                    zero_css_parser::values::BreakValue::Avoid => BreakValue::Avoid,
                    zero_css_parser::values::BreakValue::Column => BreakValue::Column,
                    zero_css_parser::values::BreakValue::Page => BreakValue::Page,
                    zero_css_parser::values::BreakValue::AvoidPage => BreakValue::AvoidPage,
                    zero_css_parser::values::BreakValue::AvoidColumn => BreakValue::AvoidColumn,
                };
                return true;
            }
        }
        // ── Column Rule 属性 ──
        "column-rule-width" => {
            if let Some(v) = values::parse_column_rule_width(value) {
                style.column_rule_width = match v {
                    zero_css_parser::values::ColumnRuleWidthValue::Medium => ColumnRuleWidthComputedValue::Medium,
                    zero_css_parser::values::ColumnRuleWidthValue::Thin => ColumnRuleWidthComputedValue::Thin,
                    zero_css_parser::values::ColumnRuleWidthValue::Thick => ColumnRuleWidthComputedValue::Thick,
                    zero_css_parser::values::ColumnRuleWidthValue::Length(l) => ColumnRuleWidthComputedValue::Length(l),
                };
                return true;
            }
        }
        "column-rule-style" => {
            if let Some(v) = values::parse_column_rule_style(value) {
                style.column_rule_style = match v {
                    zero_css_parser::values::ColumnRuleStyleValue::None => ColumnRuleStyleComputedValue::None,
                    zero_css_parser::values::ColumnRuleStyleValue::Hidden => ColumnRuleStyleComputedValue::Hidden,
                    zero_css_parser::values::ColumnRuleStyleValue::Dotted => ColumnRuleStyleComputedValue::Dotted,
                    zero_css_parser::values::ColumnRuleStyleValue::Dashed => ColumnRuleStyleComputedValue::Dashed,
                    zero_css_parser::values::ColumnRuleStyleValue::Solid => ColumnRuleStyleComputedValue::Solid,
                    zero_css_parser::values::ColumnRuleStyleValue::Double => ColumnRuleStyleComputedValue::Double,
                    zero_css_parser::values::ColumnRuleStyleValue::Groove => ColumnRuleStyleComputedValue::Groove,
                    zero_css_parser::values::ColumnRuleStyleValue::Ridge => ColumnRuleStyleComputedValue::Ridge,
                    zero_css_parser::values::ColumnRuleStyleValue::Inset => ColumnRuleStyleComputedValue::Inset,
                    zero_css_parser::values::ColumnRuleStyleValue::Outset => ColumnRuleStyleComputedValue::Outset,
                };
                return true;
            }
        }
        // ── Interaction / Performance Hint 属性 ──
        "overscroll-behavior-x" => {
            if let Some(v) = values::parse_overscroll_behavior(value) {
                style.overscroll_behavior_x = match v {
                    zero_css_parser::values::OverscrollBehaviorValue::Auto => OverscrollBehaviorValue::Auto,
                    zero_css_parser::values::OverscrollBehaviorValue::Contain => OverscrollBehaviorValue::Contain,
                    zero_css_parser::values::OverscrollBehaviorValue::None => OverscrollBehaviorValue::None,
                };
                return true;
            }
        }
        "overscroll-behavior-y" => {
            if let Some(v) = values::parse_overscroll_behavior(value) {
                style.overscroll_behavior_y = match v {
                    zero_css_parser::values::OverscrollBehaviorValue::Auto => OverscrollBehaviorValue::Auto,
                    zero_css_parser::values::OverscrollBehaviorValue::Contain => OverscrollBehaviorValue::Contain,
                    zero_css_parser::values::OverscrollBehaviorValue::None => OverscrollBehaviorValue::None,
                };
                return true;
            }
        }
        "touch-action" => {
            if let Some(v) = values::parse_touch_action(value) {
                style.touch_action = match v {
                    zero_css_parser::values::TouchActionValue::Auto => TouchActionValue::Auto,
                    zero_css_parser::values::TouchActionValue::None => TouchActionValue::None,
                    zero_css_parser::values::TouchActionValue::PanX => TouchActionValue::PanX,
                    zero_css_parser::values::TouchActionValue::PanY => TouchActionValue::PanY,
                    zero_css_parser::values::TouchActionValue::PanXPanY => TouchActionValue::PanXPanY,
                    zero_css_parser::values::TouchActionValue::Manipulation => TouchActionValue::Manipulation,
                };
                return true;
            }
        }
        "user-select" => {
            if let Some(v) = values::parse_user_select(value) {
                style.user_select = match v {
                    zero_css_parser::values::UserSelectValue::Auto => UserSelectValue::Auto,
                    zero_css_parser::values::UserSelectValue::Text => UserSelectValue::Text,
                    zero_css_parser::values::UserSelectValue::None => UserSelectValue::None,
                    zero_css_parser::values::UserSelectValue::All => UserSelectValue::All,
                    zero_css_parser::values::UserSelectValue::Contain => UserSelectValue::Contain,
                };
                return true;
            }
        }
        "will-change" => {
            if let Some(v) = values::parse_will_change(value) {
                style.will_change = match v {
                    zero_css_parser::values::WillChangeValue::Auto => WillChangeValue::Auto,
                    zero_css_parser::values::WillChangeValue::ScrollPosition => WillChangeValue::ScrollPosition,
                    zero_css_parser::values::WillChangeValue::Contents => WillChangeValue::Contents,
                    zero_css_parser::values::WillChangeValue::Custom(s) => WillChangeValue::Custom(s),
                };
                return true;
            }
        }
        "pointer-events" => {
            if let Some(v) = values::parse_pointer_events(value) {
                style.pointer_events = match v {
                    zero_css_parser::values::PointerEventsValue::Auto => PointerEventsValue::Auto,
                    zero_css_parser::values::PointerEventsValue::None => PointerEventsValue::None,
                    zero_css_parser::values::PointerEventsValue::VisiblePainted => PointerEventsValue::VisiblePainted,
                    zero_css_parser::values::PointerEventsValue::VisibleFill => PointerEventsValue::VisibleFill,
                    zero_css_parser::values::PointerEventsValue::VisibleStroke => PointerEventsValue::VisibleStroke,
                    zero_css_parser::values::PointerEventsValue::Visible => PointerEventsValue::Visible,
                    zero_css_parser::values::PointerEventsValue::Painted => PointerEventsValue::Painted,
                    zero_css_parser::values::PointerEventsValue::Fill => PointerEventsValue::Fill,
                    zero_css_parser::values::PointerEventsValue::Stroke => PointerEventsValue::Stroke,
                    zero_css_parser::values::PointerEventsValue::All => PointerEventsValue::All,
                    zero_css_parser::values::PointerEventsValue::Inherit => PointerEventsValue::Inherit,
                };
                return true;
            }
        }
        // ── OverflowWrap 属性 ──
        "overflow-wrap" => {
            if let Some(v) = values::parse_overflow_wrap(value) {
                style.overflow_wrap = match v {
                    zero_css_parser::values::OverflowWrapValue::Normal => OverflowWrapValue::Normal,
                    zero_css_parser::values::OverflowWrapValue::BreakWord => OverflowWrapValue::BreakWord,
                    zero_css_parser::values::OverflowWrapValue::Anywhere => OverflowWrapValue::Anywhere,
                };
                return true;
            }
        }
        // ── TextAlignLast 属性 ──
        "text-align-last" => {
            if let Some(v) = values::parse_text_align_last(value) {
                style.text_align_last = match v {
                    zero_css_parser::values::TextAlignLastValue::Auto => TextAlignLastValue::Auto,
                    zero_css_parser::values::TextAlignLastValue::Start => TextAlignLastValue::Start,
                    zero_css_parser::values::TextAlignLastValue::End => TextAlignLastValue::End,
                    zero_css_parser::values::TextAlignLastValue::Left => TextAlignLastValue::Left,
                    zero_css_parser::values::TextAlignLastValue::Right => TextAlignLastValue::Right,
                    zero_css_parser::values::TextAlignLastValue::Center => TextAlignLastValue::Center,
                    zero_css_parser::values::TextAlignLastValue::Justify => TextAlignLastValue::Justify,
                };
                return true;
            }
        }
        // ── FontVariantNumeric 属性 ──
        "font-variant-numeric" => {
            if let Some(v) = values::parse_font_variant_numeric(value) {
                style.font_variant_numeric = match v {
                    zero_css_parser::values::FontVariantNumericValue::Normal => FontVariantNumericValue::Normal,
                    zero_css_parser::values::FontVariantNumericValue::Ordinal => FontVariantNumericValue::Ordinal,
                    zero_css_parser::values::FontVariantNumericValue::SlashedZero => {
                        FontVariantNumericValue::SlashedZero
                    }
                    zero_css_parser::values::FontVariantNumericValue::LiningNums => FontVariantNumericValue::LiningNums,
                    zero_css_parser::values::FontVariantNumericValue::OldstyleNums => {
                        FontVariantNumericValue::OldstyleNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::ProportionalNums => {
                        FontVariantNumericValue::ProportionalNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::TabularNums => {
                        FontVariantNumericValue::TabularNums
                    }
                    zero_css_parser::values::FontVariantNumericValue::DiagonalFractions => {
                        FontVariantNumericValue::DiagonalFractions
                    }
                    zero_css_parser::values::FontVariantNumericValue::StackedFractions => {
                        FontVariantNumericValue::StackedFractions
                    }
                };
                return true;
            }
        }
        // ── Direction 属性 ──
        "direction" => {
            if let Some(v) = values::parse_direction(value) {
                style.direction = match v {
                    zero_css_parser::values::DirectionValue::Ltr => DirectionValue::Ltr,
                    zero_css_parser::values::DirectionValue::Rtl => DirectionValue::Rtl,
                };
                return true;
            }
        }
        // ── UnicodeBidi 属性 ──
        "unicode-bidi" => {
            if let Some(v) = values::parse_unicode_bidi(value) {
                style.unicode_bidi = match v {
                    zero_css_parser::values::UnicodeBidiValue::Normal => UnicodeBidiValue::Normal,
                    zero_css_parser::values::UnicodeBidiValue::Embed => UnicodeBidiValue::Embed,
                    zero_css_parser::values::UnicodeBidiValue::Isolate => UnicodeBidiValue::Isolate,
                    zero_css_parser::values::UnicodeBidiValue::BidiOverride => UnicodeBidiValue::BidiOverride,
                    zero_css_parser::values::UnicodeBidiValue::IsolateOverride => UnicodeBidiValue::IsolateOverride,
                    zero_css_parser::values::UnicodeBidiValue::Plaintext => UnicodeBidiValue::Plaintext,
                };
                return true;
            }
        }
        // ── TabSize 属性 ──
        "tab-size" => {
            if let Some(v) = values::parse_tab_size(value) {
                style.tab_size = match v {
                    zero_css_parser::values::TabSizeValue::Number(n) => TabSizeValue::Number(n),
                    zero_css_parser::values::TabSizeValue::Length(l) => TabSizeValue::Length(l),
                };
                return true;
            }
        }
        // ── Columns 简写属性 ──
        // columns: <column-width> <column-count>
        // 单值时按类型判断：纯数字 → column-count，带单位 → column-width
        "columns" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.len() == 2 {
                // 尝试两种顺序
                if let Some(v) = values::parse_column_count(parts[0]) {
                    style.column_count = match v {
                        ColumnCountValue::Auto => ColumnCountComputedValue::Auto,
                        ColumnCountValue::Number(n) => ColumnCountComputedValue::Number(n),
                    };
                    if let Some(w) = values::parse_column_width(parts[1]) {
                        style.column_width = match w {
                            ColumnWidthValue::Auto => ColumnWidthComputedValue::Auto,
                            ColumnWidthValue::Length(l) => ColumnWidthComputedValue::Length(l),
                        };
                        return true;
                    }
                }
                if let Some(v) = values::parse_column_width(parts[0]) {
                    style.column_width = match v {
                        ColumnWidthValue::Auto => ColumnWidthComputedValue::Auto,
                        ColumnWidthValue::Length(l) => ColumnWidthComputedValue::Length(l),
                    };
                    if let Some(w) = values::parse_column_count(parts[1]) {
                        style.column_count = match w {
                            ColumnCountValue::Auto => ColumnCountComputedValue::Auto,
                            ColumnCountValue::Number(n) => ColumnCountComputedValue::Number(n),
                        };
                        return true;
                    }
                }
            } else if parts.len() == 1 {
                // CSS Multi-column spec: 单值时，正整数优先解析为 column-count，
                // 长度值解析为 column-width。先尝试 column-count，再尝试 column-width。
                // 这确保 `columns: 3` 设置 column-count: 3 而非 column-width: 3px。
                if let Some(v) = values::parse_column_count(parts[0]) {
                    style.column_count = match v {
                        ColumnCountValue::Auto => ColumnCountComputedValue::Auto,
                        ColumnCountValue::Number(n) => ColumnCountComputedValue::Number(n),
                    };
                    style.column_width = ColumnWidthComputedValue::Auto;
                    return true;
                }
                if let Some(v) = values::parse_column_width(parts[0]) {
                    style.column_width = match v {
                        ColumnWidthValue::Auto => ColumnWidthComputedValue::Auto,
                        ColumnWidthValue::Length(l) => ColumnWidthComputedValue::Length(l),
                    };
                    style.column_count = ColumnCountComputedValue::Auto;
                    return true;
                }
            }
        }
        // ── ColumnCount 属性 ──
        "column-count" => {
            if let Some(v) = values::parse_column_count(value) {
                style.column_count = match v {
                    ColumnCountValue::Auto => ColumnCountComputedValue::Auto,
                    ColumnCountValue::Number(n) => ColumnCountComputedValue::Number(n),
                };
                return true;
            }
        }
        // ── ColumnWidth 属性 ──
        "column-width" => {
            if let Some(v) = values::parse_column_width(value) {
                style.column_width = match v {
                    ColumnWidthValue::Auto => ColumnWidthComputedValue::Auto,
                    ColumnWidthValue::Length(l) => ColumnWidthComputedValue::Length(l),
                };
                return true;
            }
        }
        // ── ColumnFill 属性 ──
        "column-fill" => {
            let v = value.trim().to_ascii_lowercase();
            match v.as_str() {
                "balance" | "balance-all" => {
                    style.column_fill = ColumnFillComputedValue::Balance;
                    return true;
                }
                "auto" => {
                    style.column_fill = ColumnFillComputedValue::Auto;
                    return true;
                }
                _ => {}
            }
        }
        // ── ColumnSpan 属性（§6.1：none 留在列流，all 跨越全宽成 spanner）──
        "column-span" => {
            let v = value.trim().to_ascii_lowercase();
            match v.as_str() {
                "none" => {
                    style.column_span = ColumnSpanComputedValue::None;
                    return true;
                }
                "all" => {
                    style.column_span = ColumnSpanComputedValue::All;
                    return true;
                }
                _ => {}
            }
        }
        // ── ObjectFit 属性 ──
        "object-fit" => {
            if let Some(v) = values::parse_object_fit(value) {
                style.object_fit = match v {
                    ObjectFitValue::Fill => ObjectFitComputedValue::Fill,
                    ObjectFitValue::Contain => ObjectFitComputedValue::Contain,
                    ObjectFitValue::Cover => ObjectFitComputedValue::Cover,
                    ObjectFitValue::None => ObjectFitComputedValue::None,
                    ObjectFitValue::ScaleDown => ObjectFitComputedValue::ScaleDown,
                };
                return true;
            }
        }
        // ── Filter 属性 ──
        "filter" => {
            if let Some(v) = values::parse_filter(value) {
                style.filter = match v {
                    FilterValue::None => FilterComputedValue::None,
                    FilterValue::Blur(n) => FilterComputedValue::Blur(n),
                    FilterValue::Brightness(n) => FilterComputedValue::Brightness(n),
                    FilterValue::Contrast(n) => FilterComputedValue::Contrast(n),
                    FilterValue::Grayscale(n) => FilterComputedValue::Grayscale(n),
                    FilterValue::HueRotate(n) => FilterComputedValue::HueRotate(n),
                    FilterValue::Invert(n) => FilterComputedValue::Invert(n),
                    FilterValue::Opacity(n) => FilterComputedValue::Opacity(n),
                    FilterValue::Saturate(n) => FilterComputedValue::Saturate(n),
                    FilterValue::Sepia(n) => FilterComputedValue::Sepia(n),
                    FilterValue::DropShadow(x, y, b, c) => FilterComputedValue::DropShadow(x, y, b, c),
                };
                return true;
            }
        }
        "backdrop-filter" => {
            if let Some(v) = values::parse_filter(value) {
                style.backdrop_filter = match v {
                    FilterValue::None => FilterComputedValue::None,
                    FilterValue::Blur(n) => FilterComputedValue::Blur(n),
                    FilterValue::Brightness(n) => FilterComputedValue::Brightness(n),
                    FilterValue::Contrast(n) => FilterComputedValue::Contrast(n),
                    FilterValue::Grayscale(n) => FilterComputedValue::Grayscale(n),
                    FilterValue::HueRotate(n) => FilterComputedValue::HueRotate(n),
                    FilterValue::Invert(n) => FilterComputedValue::Invert(n),
                    FilterValue::Opacity(n) => FilterComputedValue::Opacity(n),
                    FilterValue::Saturate(n) => FilterComputedValue::Saturate(n),
                    FilterValue::Sepia(n) => FilterComputedValue::Sepia(n),
                    FilterValue::DropShadow(x, y, b, c) => FilterComputedValue::DropShadow(x, y, b, c),
                };
                return true;
            }
        }
        // ── Column Rule Color 属性 ──
        "column-rule-color" => {
            if let Some(v) = values::parse_color(value) {
                style.column_rule_color = v;
                return true;
            }
        }
        // ── Contain 属性 ──
        "contain" => {
            if let Some(v) = values::parse_contain(value) {
                style.contain = match v {
                    ContainValue::None => ContainComputedValue::None,
                    ContainValue::Strict => ContainComputedValue::Strict,
                    ContainValue::Content => ContainComputedValue::Content,
                    ContainValue::Size => ContainComputedValue::Size,
                    ContainValue::Layout => ContainComputedValue::Layout,
                    ContainValue::Style => ContainComputedValue::Style,
                    ContainValue::Paint => ContainComputedValue::Paint,
                    ContainValue::Custom(flags) => ContainComputedValue::Custom(flags),
                };
                return true;
            }
        }
        // ── UI Appearance 属性 ──
        "appearance" => {
            if let Some(v) = values::parse_appearance(value) {
                style.appearance = match v {
                    zero_css_parser::values::AppearanceValue::None => AppearanceComputedValue::None,
                    zero_css_parser::values::AppearanceValue::Auto => AppearanceComputedValue::Auto,
                    zero_css_parser::values::AppearanceValue::Button => AppearanceComputedValue::Button,
                    zero_css_parser::values::AppearanceValue::Checkbox => AppearanceComputedValue::Checkbox,
                    zero_css_parser::values::AppearanceValue::Listbox => AppearanceComputedValue::Listbox,
                    zero_css_parser::values::AppearanceValue::Menulist => AppearanceComputedValue::Menulist,
                    zero_css_parser::values::AppearanceValue::Meter => AppearanceComputedValue::Meter,
                    zero_css_parser::values::AppearanceValue::ProgressBar => AppearanceComputedValue::ProgressBar,
                    zero_css_parser::values::AppearanceValue::PushButton => AppearanceComputedValue::PushButton,
                    zero_css_parser::values::AppearanceValue::Radio => AppearanceComputedValue::Radio,
                    zero_css_parser::values::AppearanceValue::Searchfield => AppearanceComputedValue::Searchfield,
                    zero_css_parser::values::AppearanceValue::SliderHorizontal => {
                        AppearanceComputedValue::SliderHorizontal
                    }
                    zero_css_parser::values::AppearanceValue::SquareButton => AppearanceComputedValue::SquareButton,
                    zero_css_parser::values::AppearanceValue::Textarea => AppearanceComputedValue::Textarea,
                    zero_css_parser::values::AppearanceValue::Textfield => AppearanceComputedValue::Textfield,
                };
                return true;
            }
        }
        "accent-color" => {
            if let Some(v) = values::parse_accent_color(value) {
                style.accent_color = match v {
                    zero_css_parser::values::AccentColorValue::Auto => AccentColorComputedValue::Auto,
                    zero_css_parser::values::AccentColorValue::Color(c) => AccentColorComputedValue::Color(c),
                };
                return true;
            }
        }
        "caret-color" => {
            if let Some(v) = values::parse_caret_color(value) {
                style.caret_color = match v {
                    zero_css_parser::values::CaretColorValue::Auto => CaretColorComputedValue::Auto,
                    zero_css_parser::values::CaretColorValue::Color(c) => CaretColorComputedValue::Color(c),
                };
                return true;
            }
        }
        // ── Compositing / Scrolling 属性 ──
        "mix-blend-mode" => {
            if let Some(v) = values::parse_mix_blend_mode(value) {
                style.mix_blend_mode = match v {
                    zero_css_parser::values::MixBlendModeValue::Normal => MixBlendModeComputedValue::Normal,
                    zero_css_parser::values::MixBlendModeValue::Multiply => MixBlendModeComputedValue::Multiply,
                    zero_css_parser::values::MixBlendModeValue::Screen => MixBlendModeComputedValue::Screen,
                    zero_css_parser::values::MixBlendModeValue::Overlay => MixBlendModeComputedValue::Overlay,
                    zero_css_parser::values::MixBlendModeValue::Darken => MixBlendModeComputedValue::Darken,
                    zero_css_parser::values::MixBlendModeValue::Lighten => MixBlendModeComputedValue::Lighten,
                    zero_css_parser::values::MixBlendModeValue::ColorDodge => MixBlendModeComputedValue::ColorDodge,
                    zero_css_parser::values::MixBlendModeValue::ColorBurn => MixBlendModeComputedValue::ColorBurn,
                    zero_css_parser::values::MixBlendModeValue::HardLight => MixBlendModeComputedValue::HardLight,
                    zero_css_parser::values::MixBlendModeValue::SoftLight => MixBlendModeComputedValue::SoftLight,
                    zero_css_parser::values::MixBlendModeValue::Difference => MixBlendModeComputedValue::Difference,
                    zero_css_parser::values::MixBlendModeValue::Exclusion => MixBlendModeComputedValue::Exclusion,
                    zero_css_parser::values::MixBlendModeValue::Hue => MixBlendModeComputedValue::Hue,
                    zero_css_parser::values::MixBlendModeValue::Saturation => MixBlendModeComputedValue::Saturation,
                    zero_css_parser::values::MixBlendModeValue::Color => MixBlendModeComputedValue::Color,
                    zero_css_parser::values::MixBlendModeValue::Luminosity => MixBlendModeComputedValue::Luminosity,
                };
                return true;
            }
        }
        "scrollbar-width" => {
            if let Some(v) = values::parse_scrollbar_width(value) {
                style.scrollbar_width = match v {
                    zero_css_parser::values::ScrollbarWidthValue::Auto => ScrollbarWidthComputedValue::Auto,
                    zero_css_parser::values::ScrollbarWidthValue::Thin => ScrollbarWidthComputedValue::Thin,
                    zero_css_parser::values::ScrollbarWidthValue::None => ScrollbarWidthComputedValue::None,
                };
                return true;
            }
        }
        "scrollbar-gutter" => {
            if let Some(v) = values::parse_scrollbar_gutter(value) {
                style.scrollbar_gutter = match v {
                    zero_css_parser::values::ScrollbarGutterValue::Auto => ScrollbarGutterComputedValue::Auto,
                    zero_css_parser::values::ScrollbarGutterValue::Stable => ScrollbarGutterComputedValue::Stable,
                    zero_css_parser::values::ScrollbarGutterValue::StableBothEdges => {
                        ScrollbarGutterComputedValue::StableBothEdges
                    }
                };
                return true;
            }
        }
        "text-wrap" => {
            if let Some(v) = values::parse_text_wrap(value) {
                style.text_wrap = match v {
                    zero_css_parser::values::TextWrapValue::Wrap => TextWrapComputedValue::Wrap,
                    zero_css_parser::values::TextWrapValue::Nowrap => TextWrapComputedValue::Nowrap,
                    zero_css_parser::values::TextWrapValue::Balance => TextWrapComputedValue::Balance,
                    zero_css_parser::values::TextWrapValue::Pretty => TextWrapComputedValue::Pretty,
                    zero_css_parser::values::TextWrapValue::Stable => TextWrapComputedValue::Stable,
                };
                return true;
            }
        }
        "hyphens" => {
            if let Some(v) = values::parse_hyphens(value) {
                style.hyphens = match v {
                    zero_css_parser::values::HyphensValue::None => HyphensComputedValue::None,
                    zero_css_parser::values::HyphensValue::Manual => HyphensComputedValue::Manual,
                    zero_css_parser::values::HyphensValue::Auto => HyphensComputedValue::Auto,
                };
                return true;
            }
        }
        "line-clamp" => {
            if let Some(v) = values::parse_line_clamp(value) {
                style.line_clamp = match v {
                    zero_css_parser::values::LineClampValue::None => LineClampComputedValue::None,
                    zero_css_parser::values::LineClampValue::Count(n) => LineClampComputedValue::Count(n),
                };
                return true;
            }
        }
        "background-image" => {
            if let Some(layers) = values::parse_background_image_layers(value) {
                style.background_image = layers
                    .into_iter()
                    .map(|v| match v {
                        zero_css_parser::values::BackgroundImageValue::None => BackgroundImageComputedValue::None,
                        zero_css_parser::values::BackgroundImageValue::Url(url) => {
                            BackgroundImageComputedValue::Url(url)
                        }
                        zero_css_parser::values::BackgroundImageValue::Gradient(g) => {
                            BackgroundImageComputedValue::Gradient(g)
                        }
                    })
                    .collect();
                return true;
            }
        }
        "mask-image" => {
            if let Some(layers) = values::parse_mask_image_layers(value) {
                style.mask_image = layers
                    .into_iter()
                    .map(|v| match v {
                        zero_css_parser::values::BackgroundImageValue::None => BackgroundImageComputedValue::None,
                        zero_css_parser::values::BackgroundImageValue::Url(url) => {
                            BackgroundImageComputedValue::Url(url)
                        }
                        zero_css_parser::values::BackgroundImageValue::Gradient(g) => {
                            BackgroundImageComputedValue::Gradient(g)
                        }
                    })
                    .collect();
                return true;
            }
        }
        "mask-mode" => {
            if let Some(v) = values::parse_mask_mode(value) {
                style.mask_mode = match v {
                    zero_css_parser::values::MaskModeValue::Alpha => MaskModeComputedValue::Alpha,
                    zero_css_parser::values::MaskModeValue::Luminance => MaskModeComputedValue::Luminance,
                    zero_css_parser::values::MaskModeValue::MatchSource => MaskModeComputedValue::MatchSource,
                };
                return true;
            }
        }
        "background-position" => {
            if let Some(v) = values::parse_background_position(value) {
                style.background_position = match v {
                    zero_css_parser::values::BackgroundPositionValue::Center => BackgroundPositionComputedValue::Center,
                    zero_css_parser::values::BackgroundPositionValue::Left => BackgroundPositionComputedValue::Left,
                    zero_css_parser::values::BackgroundPositionValue::Right => BackgroundPositionComputedValue::Right,
                    zero_css_parser::values::BackgroundPositionValue::Top => BackgroundPositionComputedValue::Top,
                    zero_css_parser::values::BackgroundPositionValue::Bottom => BackgroundPositionComputedValue::Bottom,
                    zero_css_parser::values::BackgroundPositionValue::Length(lv) => {
                        BackgroundPositionComputedValue::Length(resolve_bg_pos_length(lv, style))
                    }
                    zero_css_parser::values::BackgroundPositionValue::Percent(pct) => {
                        BackgroundPositionComputedValue::Percent(pct)
                    }
                    zero_css_parser::values::BackgroundPositionValue::TwoValue(h, v) => {
                        let hc = match *h {
                            zero_css_parser::values::BackgroundPositionValue::Center => {
                                BackgroundPositionComputedValue::Center
                            }
                            zero_css_parser::values::BackgroundPositionValue::Left => {
                                BackgroundPositionComputedValue::Left
                            }
                            zero_css_parser::values::BackgroundPositionValue::Right => {
                                BackgroundPositionComputedValue::Right
                            }
                            zero_css_parser::values::BackgroundPositionValue::Top => {
                                BackgroundPositionComputedValue::Top
                            }
                            zero_css_parser::values::BackgroundPositionValue::Bottom => {
                                BackgroundPositionComputedValue::Bottom
                            }
                            zero_css_parser::values::BackgroundPositionValue::Length(lv) => {
                                BackgroundPositionComputedValue::Length(resolve_bg_pos_length(lv, style))
                            }
                            zero_css_parser::values::BackgroundPositionValue::Percent(pct) => {
                                BackgroundPositionComputedValue::Percent(pct)
                            }
                            zero_css_parser::values::BackgroundPositionValue::TwoValue(_, _) => return false,
                        };
                        let vc = match *v {
                            zero_css_parser::values::BackgroundPositionValue::Center => {
                                BackgroundPositionComputedValue::Center
                            }
                            zero_css_parser::values::BackgroundPositionValue::Left => {
                                BackgroundPositionComputedValue::Left
                            }
                            zero_css_parser::values::BackgroundPositionValue::Right => {
                                BackgroundPositionComputedValue::Right
                            }
                            zero_css_parser::values::BackgroundPositionValue::Top => {
                                BackgroundPositionComputedValue::Top
                            }
                            zero_css_parser::values::BackgroundPositionValue::Bottom => {
                                BackgroundPositionComputedValue::Bottom
                            }
                            zero_css_parser::values::BackgroundPositionValue::Length(lv) => {
                                BackgroundPositionComputedValue::Length(resolve_bg_pos_length(lv, style))
                            }
                            zero_css_parser::values::BackgroundPositionValue::Percent(pct) => {
                                BackgroundPositionComputedValue::Percent(pct)
                            }
                            zero_css_parser::values::BackgroundPositionValue::TwoValue(_, _) => return false,
                        };
                        BackgroundPositionComputedValue::TwoValue(Box::new(hc), Box::new(vc))
                    }
                };
                return true;
            }
        }
        "background-repeat" => {
            if let Some(v) = values::parse_background_repeat(value) {
                style.background_repeat = match v {
                    zero_css_parser::values::BackgroundRepeatValue::Repeat => BackgroundRepeatComputedValue::Repeat,
                    zero_css_parser::values::BackgroundRepeatValue::RepeatX => BackgroundRepeatComputedValue::RepeatX,
                    zero_css_parser::values::BackgroundRepeatValue::RepeatY => BackgroundRepeatComputedValue::RepeatY,
                    zero_css_parser::values::BackgroundRepeatValue::NoRepeat => BackgroundRepeatComputedValue::NoRepeat,
                    zero_css_parser::values::BackgroundRepeatValue::Space => BackgroundRepeatComputedValue::Space,
                    zero_css_parser::values::BackgroundRepeatValue::Round => BackgroundRepeatComputedValue::Round,
                };
                return true;
            }
        }
        "background-size" => {
            if let Some(v) = values::parse_background_size(value) {
                style.background_size = match v {
                    zero_css_parser::values::BackgroundSizeValue::Auto => BackgroundSizeComputedValue::Auto,
                    zero_css_parser::values::BackgroundSizeValue::Cover => BackgroundSizeComputedValue::Cover,
                    zero_css_parser::values::BackgroundSizeValue::Contain => BackgroundSizeComputedValue::Contain,
                    zero_css_parser::values::BackgroundSizeValue::Length(n) => BackgroundSizeComputedValue::Length(n),
                    zero_css_parser::values::BackgroundSizeValue::Percent(n) => BackgroundSizeComputedValue::Percent(n),
                };
                return true;
            }
        }
        "background-attachment" => {
            if let Some(v) = values::parse_background_attachment(value) {
                style.background_attachment = match v {
                    zero_css_parser::values::BackgroundAttachmentValue::Scroll => {
                        BackgroundAttachmentComputedValue::Scroll
                    }
                    zero_css_parser::values::BackgroundAttachmentValue::Fixed => {
                        BackgroundAttachmentComputedValue::Fixed
                    }
                    zero_css_parser::values::BackgroundAttachmentValue::Local => {
                        BackgroundAttachmentComputedValue::Local
                    }
                };
                return true;
            }
        }
        "background-clip" => {
            if let Some(v) = values::parse_background_clip(value) {
                style.background_clip = match v {
                    zero_css_parser::values::BackgroundClipValue::BorderBox => BackgroundClipComputedValue::BorderBox,
                    zero_css_parser::values::BackgroundClipValue::PaddingBox => BackgroundClipComputedValue::PaddingBox,
                    zero_css_parser::values::BackgroundClipValue::ContentBox => BackgroundClipComputedValue::ContentBox,
                    zero_css_parser::values::BackgroundClipValue::Text => BackgroundClipComputedValue::Text,
                };
                return true;
            }
        }
        "background-origin" => {
            if let Some(v) = values::parse_background_origin(value) {
                style.background_origin = match v {
                    zero_css_parser::values::BackgroundOriginValue::PaddingBox => {
                        BackgroundOriginComputedValue::PaddingBox
                    }
                    zero_css_parser::values::BackgroundOriginValue::BorderBox => {
                        BackgroundOriginComputedValue::BorderBox
                    }
                    zero_css_parser::values::BackgroundOriginValue::ContentBox => {
                        BackgroundOriginComputedValue::ContentBox
                    }
                };
                return true;
            }
        }
        "border-image-source" => {
            if let Some(v) = values::parse_border_image_source(value) {
                style.border_image_source = match v {
                    zero_css_parser::values::BorderImageSourceValue::None => BorderImageSourceComputedValue::None,
                    zero_css_parser::values::BorderImageSourceValue::Url(url) => {
                        BorderImageSourceComputedValue::Url(url)
                    }
                };
                return true;
            }
        }
        "border-image-slice" => {
            if let Some(v) = values::parse_border_image_slice(value) {
                fn convert_comp(
                    c: &zero_css_parser::values::BorderImageSliceComponent,
                ) -> BorderImageSliceComputedComponent {
                    match c {
                        zero_css_parser::values::BorderImageSliceComponent::Number(n) => {
                            BorderImageSliceComputedComponent::Number(*n)
                        }
                        zero_css_parser::values::BorderImageSliceComponent::Percent(p) => {
                            BorderImageSliceComputedComponent::Percent(*p)
                        }
                    }
                }
                style.border_image_slice = BorderImageSliceComputedValue {
                    top: convert_comp(&v.top),
                    right: convert_comp(&v.right),
                    bottom: convert_comp(&v.bottom),
                    left: convert_comp(&v.left),
                    fill: v.fill,
                };
                return true;
            }
        }
        "border-image-width" => {
            if let Some(v) = values::parse_border_image_width(value) {
                fn convert_comp(
                    c: &zero_css_parser::values::BorderImageWidthComponent,
                ) -> BorderImageWidthComputedComponent {
                    match c {
                        zero_css_parser::values::BorderImageWidthComponent::Auto => {
                            BorderImageWidthComputedComponent::Auto
                        }
                        zero_css_parser::values::BorderImageWidthComponent::Number(n) => {
                            BorderImageWidthComputedComponent::Number(*n)
                        }
                        zero_css_parser::values::BorderImageWidthComponent::Length(
                            zero_css_parser::values::LengthValue::Px(px),
                        ) => BorderImageWidthComputedComponent::Length(*px as f32),
                        zero_css_parser::values::BorderImageWidthComponent::Percent(p) => {
                            BorderImageWidthComputedComponent::Percent(*p)
                        }
                        _ => BorderImageWidthComputedComponent::Number(1.0),
                    }
                }
                style.border_image_width = BorderImageWidthComputedValue {
                    top: convert_comp(&v.top),
                    right: convert_comp(&v.right),
                    bottom: convert_comp(&v.bottom),
                    left: convert_comp(&v.left),
                };
                return true;
            }
        }
        "border-image-repeat" => {
            if let Some(v) = values::parse_border_image_repeat(value) {
                fn convert_mode(m: &zero_css_parser::values::BorderImageRepeatMode) -> BorderImageRepeatComputedMode {
                    match m {
                        zero_css_parser::values::BorderImageRepeatMode::Stretch => {
                            BorderImageRepeatComputedMode::Stretch
                        }
                        zero_css_parser::values::BorderImageRepeatMode::Repeat => BorderImageRepeatComputedMode::Repeat,
                        zero_css_parser::values::BorderImageRepeatMode::Round => BorderImageRepeatComputedMode::Round,
                        zero_css_parser::values::BorderImageRepeatMode::Space => BorderImageRepeatComputedMode::Space,
                    }
                }
                style.border_image_repeat = BorderImageRepeatComputedValue {
                    horizontal: convert_mode(&v.horizontal),
                    vertical: convert_mode(&v.vertical),
                };
                return true;
            }
        }
        "border-image-outset" => {
            if let Some(v) = values::parse_border_image_outset(value) {
                fn convert_comp(
                    c: &zero_css_parser::values::BorderImageOutsetComponent,
                ) -> BorderImageOutsetComputedComponent {
                    match c {
                        zero_css_parser::values::BorderImageOutsetComponent::Number(n) => {
                            BorderImageOutsetComputedComponent::Number(*n)
                        }
                        zero_css_parser::values::BorderImageOutsetComponent::Length(
                            zero_css_parser::values::LengthValue::Px(px),
                        ) => BorderImageOutsetComputedComponent::Length(*px as f32),
                        _ => BorderImageOutsetComputedComponent::Number(0.0),
                    }
                }
                style.border_image_outset = BorderImageOutsetComputedValue {
                    top: convert_comp(&v.top),
                    right: convert_comp(&v.right),
                    bottom: convert_comp(&v.bottom),
                    left: convert_comp(&v.left),
                };
                return true;
            }
        }
        "text-shadow" => {
            if let Some(v) = zero_css_parser::values::parse_text_shadow(value) {
                style.text_shadow = TextShadowComputedValue {
                    offset_x: match v.offset_x {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    offset_y: match v.offset_y {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    blur_radius: match v.blur_radius {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    color: v.color,
                };
                return true;
            }
        }
        "box-shadow" => {
            if let Some(v) = zero_css_parser::values::parse_box_shadow(value) {
                style.box_shadow = BoxShadowComputedValue {
                    offset_x: match v.offset_x {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    offset_y: match v.offset_y {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    blur_radius: match v.blur_radius {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    spread_radius: match v.spread_radius {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    color: v.color,
                    inset: v.inset,
                };
                return true;
            }
        }
        "clip-path" => {
            if let Some(v) = zero_css_parser::values::parse_clip_path(value) {
                style.clip_path = v;
                return true;
            }
        }
        "clip" => {
            if let Some(v) = zero_css_parser::values::parse_clip(value) {
                style.clip = v;
                return true;
            }
        }
        "justify-items" => {
            let lower = value.to_ascii_lowercase();
            let v = match lower.as_str() {
                "auto" => JustifyItemsValue::Auto,
                "normal" => JustifyItemsValue::Normal,
                "start" => JustifyItemsValue::Start,
                "end" => JustifyItemsValue::End,
                "center" => JustifyItemsValue::Center,
                "stretch" => JustifyItemsValue::Stretch,
                "baseline" => JustifyItemsValue::Baseline,
                _ => return false,
            };
            style.justify_items = v;
            return true;
        }
        "justify-self" => {
            let lower = value.to_ascii_lowercase();
            let v = match lower.as_str() {
                "auto" => JustifySelfValue::Auto,
                "normal" => JustifySelfValue::Normal,
                "start" => JustifySelfValue::Start,
                "end" => JustifySelfValue::End,
                "center" => JustifySelfValue::Center,
                "stretch" => JustifySelfValue::Stretch,
                "baseline" => JustifySelfValue::Baseline,
                _ => return false,
            };
            style.justify_self = v;
            return true;
        }
        "align-content" => {
            let lower = value.to_ascii_lowercase();
            let v = match lower.as_str() {
                "auto" => AlignContentValue::Auto,
                "normal" => AlignContentValue::Normal,
                "start" => AlignContentValue::Start,
                "end" => AlignContentValue::End,
                // R1412：flex-start/flex-end 此前未解析（fall through → return false →
                // 默认 Normal → taffy 默认 flex-start pack），致 align-content:flex-end 被
                // 当作 flex-start（lines 在顶不在底）。CSS css-align-3：flex 容器的 block 轴
                // 上 flex-start/flex-end 等价 start/end（horizontal-tb 下）。vertical/RTL 差异
                // 属 R109 territory。驱动 css-flexbox/flex-align-content-end 簇。
                "flex-start" => AlignContentValue::Start,
                "flex-end" => AlignContentValue::End,
                "center" => AlignContentValue::Center,
                "stretch" => AlignContentValue::Stretch,
                "baseline" => AlignContentValue::Baseline,
                "space-between" => AlignContentValue::SpaceBetween,
                "space-around" => AlignContentValue::SpaceAround,
                "space-evenly" => AlignContentValue::SpaceEvenly,
                _ => return false,
            };
            style.align_content = v;
            return true;
        }
        "empty-cells" => {
            if let Some(v) = zero_css_parser::values::parse_empty_cells(value) {
                style.empty_cells = match v {
                    zero_css_parser::values::EmptyCellsValue::Show => EmptyCellsComputedValue::Show,
                    zero_css_parser::values::EmptyCellsValue::Hide => EmptyCellsComputedValue::Hide,
                };
                return true;
            }
        }
        "border-spacing" => {
            if let Some(v) = zero_css_parser::values::parse_border_spacing(value) {
                style.border_spacing = BorderSpacingComputedValue {
                    horizontal: match v.horizontal {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                    vertical: match v.vertical {
                        zero_css_parser::values::LengthValue::Px(px) => px as f32,
                        _ => 0.0,
                    },
                };
                return true;
            }
        }
        _ => {}
    }
    false
}

// ── border 逻辑属性辅助（CSS Logical Properties §3 + Writing Modes §6）──

// ── 逻辑属性辅助（CSS Logical Properties §1/§3 + Writing Modes §6）──
// margin / padding / inset / border 的 inline/block × start/end 逻辑属性共用此映射。

/// 物理边标签（top/right/bottom/left）。
#[derive(Clone, Copy)]
enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}

/// 按 logical 轴 + 起/止 + 元素 writing-mode 解析物理边。
///
/// - `axis_inline=true` 表示 inline 轴（inline-start/inline-end），`false` 表示 block 轴。
/// - `start=true` 表示 start 侧，`false` 表示 end 侧。
/// - inline 轴 direction 暂按 ltr（vertical 模式 inline-start=top）。
///
/// 映射（CSS Writing Modes §6）：
///   horizontal-tb：inline-start=left, inline-end=right, block-start=top, block-end=bottom
///   vertical-rl：  inline-start=top,  inline-end=bottom, block-start=right, block-end=left
///   vertical-lr：  inline-start=top,  inline-end=bottom, block-start=left, block-end=right
fn logical_physical_side(axis_inline: bool, start: bool, wm: &WritingModeValue) -> PhysicalSide {
    use PhysicalSide as P;
    use WritingModeValue as Wm;
    match (axis_inline, start) {
        // inline 轴：horizontal-tb 水平（start=left/end=right），vertical 垂直（start=top/end=bottom）
        (true, true) => match wm {
            Wm::HorizontalTb => P::Left,
            _ => P::Top,
        },
        (true, false) => match wm {
            Wm::HorizontalTb => P::Right,
            _ => P::Bottom,
        },
        // block 轴：horizontal-tb 垂直（start=top/end=bottom）；
        // vertical-rl 水平 start=right/end=left；vertical-lr 水平 start=left/end=right
        (false, true) => match wm {
            Wm::HorizontalTb => P::Top,
            Wm::VerticalRl => P::Right,
            Wm::VerticalLr => P::Left,
        },
        (false, false) => match wm {
            Wm::HorizontalTb => P::Bottom,
            Wm::VerticalRl => P::Left,
            Wm::VerticalLr => P::Right,
        },
    }
}

/// 按 logical 属性名（如 `border-inline-start-width`）与元素 writing-mode 解析物理边。
///
/// 属性名形如 `border-{axis}-{side}-{kind}`，axis ∈ {inline, block}，side ∈ {start, end}。
fn logical_border_physical_side(property: &str, wm: &WritingModeValue) -> PhysicalSide {
    logical_physical_side(property.contains("-inline-"), property.contains("-start-"), wm)
}

fn set_border_width_field(style: &mut ComputedStyle, side: PhysicalSide, v: LengthValue) {
    match side {
        PhysicalSide::Top => style.border_top_width = v,
        PhysicalSide::Right => style.border_right_width = v,
        PhysicalSide::Bottom => style.border_bottom_width = v,
        PhysicalSide::Left => style.border_left_width = v,
    }
}

fn set_border_style_field(style: &mut ComputedStyle, side: PhysicalSide, v: BorderStyleValue) {
    match side {
        PhysicalSide::Top => style.border_top_style = v,
        PhysicalSide::Right => style.border_right_style = v,
        PhysicalSide::Bottom => style.border_bottom_style = v,
        PhysicalSide::Left => style.border_left_style = v,
    }
}

fn set_border_color_field(style: &mut ComputedStyle, side: PhysicalSide, v: ColorValue) {
    match side {
        PhysicalSide::Top => style.border_top_color = v,
        PhysicalSide::Right => style.border_right_color = v,
        PhysicalSide::Bottom => style.border_bottom_color = v,
        PhysicalSide::Left => style.border_left_color = v,
    }
}

// ── margin / padding / inset 逻辑属性应用（R1049：writing-mode-aware）──

/// 应用 logical margin（margin-block-start 等）。horizontal-tb 下与原 R143 静态映射字节一致。
fn apply_logical_margin(style: &mut ComputedStyle, axis_inline: bool, start: bool, value: &str) -> bool {
    if let Some(v) = parse_length_or_math(value) {
        let side = logical_physical_side(axis_inline, start, &style.writing_mode);
        match side {
            PhysicalSide::Top => style.margin_top = v,
            PhysicalSide::Right => style.margin_right = v,
            PhysicalSide::Bottom => style.margin_bottom = v,
            PhysicalSide::Left => style.margin_left = v,
        }
        return true;
    }
    false
}

/// 应用 logical padding。horizontal-tb 下与原 R143 静态映射字节一致。
fn apply_logical_padding(style: &mut ComputedStyle, axis_inline: bool, start: bool, value: &str) -> bool {
    if let Some(v) = parse_length_or_math(value) {
        let side = logical_physical_side(axis_inline, start, &style.writing_mode);
        match side {
            PhysicalSide::Top => style.padding_top = v,
            PhysicalSide::Right => style.padding_right = v,
            PhysicalSide::Bottom => style.padding_bottom = v,
            PhysicalSide::Left => style.padding_left = v,
        }
        return true;
    }
    false
}

/// 应用 logical inset（inset-block-start 等）。horizontal-tb 下与原 R143 静态映射字节一致。
fn apply_logical_inset(style: &mut ComputedStyle, axis_inline: bool, start: bool, value: &str) -> bool {
    if let Some(v) = parse_length_or_math(value) {
        let side = logical_physical_side(axis_inline, start, &style.writing_mode);
        match side {
            PhysicalSide::Top => style.top = v,
            PhysicalSide::Right => style.right = v,
            PhysicalSide::Bottom => style.bottom = v,
            PhysicalSide::Left => style.left = v,
        }
        return true;
    }
    false
}
