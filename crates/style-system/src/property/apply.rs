//! CSS 属性值应用到 ComputedStyle。

use super::parse::*;
use super::types::*;
use zero_css_parser::values;

/// 尝试解析 CSS 长度值，支持简单值和数学函数（calc/min/max/clamp）。
///
/// 先尝试简单解析（parse_length），失败时尝试数学函数（parse_math_function）。
/// 数学函数在属性应用阶段存储为 `LengthValue::Calc`，后续由 `resolve_computed_style` 求值。
pub(crate) fn parse_length_or_math(value: &str) -> Option<LengthValue> {
    if let Some(v) = values::parse_length(value) {
        return Some(v);
    }
    // 尝试解析 calc/min/max/clamp 数学表达式
    values::parse_math_function(value).map(|expr| LengthValue::Calc(Box::new(expr)))
}

/// 尝试解析 CSS 长度值（quirks mode）。
///
/// 与 `parse_length_or_math` 类似，但标准解析失败时尝试将裸数字视为 px。
pub(crate) fn parse_length_or_math_quirks(value: &str) -> Option<LengthValue> {
    if let Some(v) = values::parse_length_quirks(value) {
        return Some(v);
    }
    // 尝试解析 calc/min/max/clamp 数学表达式
    values::parse_math_function(value).map(|expr| LengthValue::Calc(Box::new(expr)))
}

/// 将属性字符串值设置到 ComputedStyle 的对应字段（非 quirks mode）。
///
/// 返回 true 表示成功设置。
pub fn apply_property_value(style: &mut ComputedStyle, property: &str, value: &str) -> bool {
    apply_property_value_with_quirks(style, property, value, false)
}

/// 将属性字符串值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
///
/// 当 `quirks_mode` 为 true 时，颜色解析使用 quirks mode 宽容规则
/// （如接受不带 # 的十六进制、纯数字颜色值），长度解析接受裸数字视为 px。
pub fn apply_property_value_with_quirks(
    style: &mut ComputedStyle,
    property: &str,
    value: &str,
    quirks_mode: bool,
) -> bool {
    // 不在此 trim：声明值经 consume_declaration deferred-whitespace 已无首尾空白 token
    // （inline style 经 parse_inline_style 自行 trim，presentational hints 产 clean 值，
    // cascade apply-on-dummy 传 cascaded 值）。此处 trim 会误剥**转义产生的**空白
    //（如 `red\9` → `red\t`），与 parse_color 不再 trim 配合使非法颜色被正确拒绝。
    // driving：escapes-014/015/016（apply 拒绝→cascade R2126 丢弃→下个合法声明胜出）。
    let parse_color_fn = if quirks_mode {
        values::parse_color_quirks
    } else {
        values::parse_color
    };

    // 长度解析函数：quirks mode 将裸数字视为 px
    let parse_length_fn = if quirks_mode {
        parse_length_or_math_quirks
    } else {
        parse_length_or_math
    };

    match property {
        "display" => {
            if let Some(v) = values::parse_display(value) {
                style.display = v;
                return true;
            }
        }
        "position" => {
            if let Some(v) = values::parse_position(value) {
                style.position = v;
                return true;
            }
        }
        "float" => {
            if let Some(v) = values::parse_float(value) {
                style.float = v;
                return true;
            }
        }
        "clear" => {
            if let Some(v) = values::parse_clear(value) {
                style.clear = v;
                return true;
            }
        }
        "list-style-type" => {
            if let Some(v) = values::parse_list_style_type(value) {
                style.list_style_type = v;
                return true;
            }
        }
        "list-style-position" => {
            if let Some(v) = values::parse_list_style_position(value) {
                style.list_style_position = v;
                return true;
            }
        }
        "list-style-image" => {
            if let Some(v) = zero_css_parser::values::parse_list_style_image(value) {
                style.list_style_image = match v {
                    zero_css_parser::values::ListStyleImageValue::None => ListStyleImageComputedValue::None,
                    zero_css_parser::values::ListStyleImageValue::Url(url) => ListStyleImageComputedValue::Url(url),
                };
                return true;
            }
        }
        "width" => {
            if let Some(v) = parse_length_fn(value) {
                style.width = v;
                return true;
            }
        }
        "height" => {
            if let Some(v) = parse_length_fn(value) {
                style.height = v;
                return true;
            }
        }
        // 逻辑尺寸属性 inline-size / block-size（CSS Logical Properties §1）。
        // 这里映射到水平书写模式的物理等价（inline-size→width、block-size→height）；
        // 垂直书写模式的轴交换由 converter 的 swap_writing_mode_axes 负责
        //（width↔height 互换），故无需在此感知 writing-mode。
        "inline-size" => {
            if let Some(v) = parse_length_fn(value) {
                style.width = v;
                return true;
            }
        }
        "block-size" => {
            if let Some(v) = parse_length_fn(value) {
                style.height = v;
                return true;
            }
        }
        "min-width" => {
            if let Some(v) = parse_length_fn(value) {
                style.min_width = v;
                return true;
            }
        }
        "min-height" => {
            if let Some(v) = parse_length_fn(value) {
                style.min_height = v;
                return true;
            }
        }
        "max-width" => {
            if value == "none" {
                style.max_width = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                style.max_width = v;
                return true;
            }
        }
        "max-height" => {
            if value == "none" {
                style.max_height = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                style.max_height = v;
                return true;
            }
        }
        "margin-top" => {
            if let Some(v) = parse_length_fn(value) {
                style.margin_top = v;
                return true;
            }
        }
        "margin-right" => {
            if let Some(v) = parse_length_fn(value) {
                style.margin_right = v;
                return true;
            }
        }
        "margin-bottom" => {
            if let Some(v) = parse_length_fn(value) {
                style.margin_bottom = v;
                return true;
            }
        }
        "margin-left" => {
            if let Some(v) = parse_length_fn(value) {
                style.margin_left = v;
                return true;
            }
        }
        "padding-top" => {
            if let Some(v) = parse_length_fn(value) {
                style.padding_top = v;
                return true;
            }
        }
        "padding-right" => {
            if let Some(v) = parse_length_fn(value) {
                style.padding_right = v;
                return true;
            }
        }
        "padding-bottom" => {
            if let Some(v) = parse_length_fn(value) {
                style.padding_bottom = v;
                return true;
            }
        }
        "padding-left" => {
            if let Some(v) = parse_length_fn(value) {
                style.padding_left = v;
                return true;
            }
        }
        "box-sizing" => {
            if let Some(v) = values::parse_box_sizing(value) {
                style.box_sizing = v;
                return true;
            }
        }
        "border-top-width" => {
            if let Some(v) = parse_length_fn(value) {
                // CSS 规范：border-width 不允许负值，负值视为无效
                if let LengthValue::Px(px) = v
                    && px < 0.0
                {
                    return false;
                }
                style.border_top_width = v;
                return true;
            }
        }
        "border-right-width" => {
            if let Some(v) = parse_length_fn(value) {
                if let LengthValue::Px(px) = v
                    && px < 0.0
                {
                    return false;
                }
                style.border_right_width = v;
                return true;
            }
        }
        "border-bottom-width" => {
            if let Some(v) = parse_length_fn(value) {
                if let LengthValue::Px(px) = v
                    && px < 0.0
                {
                    return false;
                }
                style.border_bottom_width = v;
                return true;
            }
        }
        "border-left-width" => {
            if let Some(v) = parse_length_fn(value) {
                if let LengthValue::Px(px) = v
                    && px < 0.0
                {
                    return false;
                }
                style.border_left_width = v;
                return true;
            }
        }
        "border-top-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.border_top_color = v;
                return true;
            }
        }
        "border-right-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.border_right_color = v;
                return true;
            }
        }
        "border-bottom-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.border_bottom_color = v;
                return true;
            }
        }
        "border-left-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.border_left_color = v;
                return true;
            }
        }
        "border-top-style" => {
            if let Some(v) = parse_border_style(value) {
                style.border_top_style = v;
                return true;
            }
        }
        "border-right-style" => {
            if let Some(v) = parse_border_style(value) {
                style.border_right_style = v;
                return true;
            }
        }
        "border-bottom-style" => {
            if let Some(v) = parse_border_style(value) {
                style.border_bottom_style = v;
                return true;
            }
        }
        "border-left-style" => {
            if let Some(v) = parse_border_style(value) {
                style.border_left_style = v;
                return true;
            }
        }
        "border-top-left-radius" => {
            if let Some(v) = parse_length_fn(value) {
                style.border_top_left_radius = v;
                return true;
            }
        }
        "border-top-right-radius" => {
            if let Some(v) = parse_length_fn(value) {
                style.border_top_right_radius = v;
                return true;
            }
        }
        "border-bottom-right-radius" => {
            if let Some(v) = parse_length_fn(value) {
                style.border_bottom_right_radius = v;
                return true;
            }
        }
        "border-bottom-left-radius" => {
            if let Some(v) = parse_length_fn(value) {
                style.border_bottom_left_radius = v;
                return true;
            }
        }
        // ── Outline 属性 ──
        "outline-width" => {
            if let Some(v) = parse_length_fn(value) {
                style.outline_width = v;
                return true;
            }
        }
        "outline-style" => {
            if let Some(v) = parse_outline_style(value) {
                style.outline_style = v;
                return true;
            }
        }
        "outline-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.outline_color = v;
                return true;
            }
        }
        "outline-offset" => {
            if let Some(v) = parse_length_fn(value) {
                style.outline_offset = v;
                return true;
            }
        }
        "color" => {
            if let Some(v) = parse_color_fn(value) {
                style.color = v;
                return true;
            }
        }
        "background-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.background_color = v;
                return true;
            }
        }
        "opacity" => {
            if let Some(v) = values::parse_opacity(value) {
                style.opacity = v;
                return true;
            }
        }
        "visibility" => {
            if let Some(v) = values::parse_visibility(value) {
                style.visibility = v;
                return true;
            }
        }
        "content-visibility" => {
            if let Some(v) = values::parse_content_visibility(value) {
                style.content_visibility = v;
                return true;
            }
        }
        "font-family" => {
            style.font_family = parse_font_family(value);
            return true;
        }
        "font-size" => {
            if let Some(v) = parse_length_fn(value) {
                style.font_size = v;
                return true;
            }
        }
        "font-weight" => {
            if let Some(v) = values::parse_font_weight(value) {
                style.font_weight = v;
                return true;
            }
        }
        "font-style" => {
            if let Some(v) = values::parse_font_style(value) {
                style.font_style = v;
                return true;
            }
        }
        "line-height" => {
            if let Some(v) = parse_line_height(value) {
                style.line_height = v;
                return true;
            }
        }
        "font-size-adjust" => {
            if let Some(v) = parse_font_size_adjust(value) {
                style.font_size_adjust = v;
                return true;
            }
        }
        "text-align" => {
            if let Some(v) = parse_text_align(value) {
                style.text_align = v;
                return true;
            }
        }
        "text-decoration" => {
            if let Some(v) = parse_text_decoration(value) {
                style.text_decoration = v;
                return true;
            }
        }
        "text-decoration-line" => {
            if let Some(v) = parse_text_decoration_line(value) {
                style.text_decoration_line = v;
                return true;
            }
        }
        "text-decoration-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.text_decoration_color = v;
                return true;
            }
        }
        "text-decoration-style" => {
            if let Some(v) = values::parse_text_decoration_style(value) {
                style.text_decoration_style = match v {
                    values::TextDecorationStyleValue::Solid => super::types::TextDecorationStyleValue::Solid,
                    values::TextDecorationStyleValue::Double => super::types::TextDecorationStyleValue::Double,
                    values::TextDecorationStyleValue::Dotted => super::types::TextDecorationStyleValue::Dotted,
                    values::TextDecorationStyleValue::Dashed => super::types::TextDecorationStyleValue::Dashed,
                    values::TextDecorationStyleValue::Wavy => super::types::TextDecorationStyleValue::Wavy,
                };
                return true;
            }
        }
        "text-decoration-thickness" => {
            if let Some(v) = values::parse_text_decoration_thickness(value) {
                style.text_decoration_thickness = match v {
                    values::TextDecorationThicknessValue::Auto | values::TextDecorationThicknessValue::FromFont => {
                        super::types::TextDecorationThicknessValue::Auto
                    }
                    values::TextDecorationThicknessValue::Length(n) => {
                        super::types::TextDecorationThicknessValue::Length(n)
                    }
                };
                return true;
            }
        }
        "text-decoration-inset" => {
            if let Some(v) = values::parse_text_decoration_inset(value) {
                style.text_decoration_inset = v;
                return true;
            }
        }
        "text-emphasis-style" => {
            if let Some(v) = values::parse_text_emphasis_style(value) {
                style.text_emphasis_style = v;
                return true;
            }
        }
        "text-emphasis-position" => {
            if let Some(v) = values::parse_text_emphasis_position(value) {
                style.text_emphasis_position = v;
                return true;
            }
        }
        "text-transform" => {
            if let Some(v) = parse_text_transform(value) {
                style.text_transform = v;
                return true;
            }
        }
        "letter-spacing" => {
            if let Some(v) = parse_length_fn(value) {
                style.letter_spacing = v;
                return true;
            }
        }
        "word-spacing" => {
            if let Some(v) = parse_length_fn(value) {
                style.word_spacing = v;
                return true;
            }
        }
        "white-space" => {
            if let Some(v) = parse_white_space(value) {
                style.white_space = v;
                return true;
            }
        }
        "word-break" => {
            if let Some(v) = parse_word_break(value) {
                style.word_break = v;
                return true;
            }
        }
        "text-autospace" => {
            if let Some(v) = parse_text_autospace(value) {
                style.text_autospace = v;
                return true;
            }
        }
        "line-break" => {
            if let Some(v) = parse_line_break(value) {
                style.line_break = v;
                return true;
            }
        }
        "writing-mode" => {
            if let Some(v) = parse_writing_mode(value) {
                style.writing_mode = v;
                return true;
            }
        }
        "text-indent" => {
            if let Some(v) = parse_length_fn(value) {
                style.text_indent = v;
                return true;
            }
        }
        "table-layout" => {
            if let Some(v) = values::parse_table_layout(value) {
                style.table_layout = match v {
                    zero_css_parser::values::TableLayoutValue::Auto => TableLayoutValue::Auto,
                    zero_css_parser::values::TableLayoutValue::Fixed => TableLayoutValue::Fixed,
                };
                return true;
            }
        }
        "caption-side" => {
            if let Some(v) = values::parse_caption_side(value) {
                style.caption_side = match v {
                    zero_css_parser::values::CaptionSideValue::Top => CaptionSideValue::Top,
                    zero_css_parser::values::CaptionSideValue::Bottom => CaptionSideValue::Bottom,
                };
                return true;
            }
        }
        "border-collapse" => {
            if let Some(v) = values::parse_border_collapse(value) {
                style.border_collapse = match v {
                    zero_css_parser::values::BorderCollapseValue::Separate => BorderCollapseValue::Separate,
                    zero_css_parser::values::BorderCollapseValue::Collapse => BorderCollapseValue::Collapse,
                };
                return true;
            }
        }
        "resize" => {
            if let Some(v) = values::parse_resize(value) {
                style.resize = match v {
                    zero_css_parser::values::ResizeValue::None => ResizeValue::None,
                    zero_css_parser::values::ResizeValue::Both => ResizeValue::Both,
                    zero_css_parser::values::ResizeValue::Horizontal => ResizeValue::Horizontal,
                    zero_css_parser::values::ResizeValue::Vertical => ResizeValue::Vertical,
                    zero_css_parser::values::ResizeValue::Block => ResizeValue::Block,
                    zero_css_parser::values::ResizeValue::Inline => ResizeValue::Inline,
                };
                return true;
            }
        }
        "margin-trim" => {
            if let Some(v) = values::parse_margin_trim(value) {
                style.margin_trim = MarginTrimValue {
                    block_start: v.block_start,
                    block_end: v.block_end,
                    inline_start: v.inline_start,
                    inline_end: v.inline_end,
                };
                return true;
            }
        }
        "text-overflow" => {
            if let Some(v) = parse_text_overflow(value) {
                style.text_overflow = v;
                return true;
            }
        }
        "vertical-align" => {
            if let Some(v) = values::parse_vertical_align(value) {
                style.vertical_align = v;
                return true;
            }
        }
        "flex-direction" => {
            if let Some(v) = values::parse_flex_direction(value) {
                style.flex_direction = v;
                return true;
            }
        }
        "flex-wrap" => {
            if let Some(v) = values::parse_flex_wrap(value) {
                style.flex_wrap = v;
                return true;
            }
        }
        "justify-content" => {
            if let Some(v) = values::parse_alignment(value) {
                style.justify_content = v;
                return true;
            }
        }
        "align-items" => {
            if let Some(v) = values::parse_alignment(value) {
                style.align_items = v;
                return true;
            }
        }
        "align-self" => {
            if let Some(v) = values::parse_alignment(value) {
                style.align_self = v;
                return true;
            }
        }
        // CSS Flexbox §7.3.1/§7.3.2：flex-grow/flex-shrink 负值非法，按未声明处理
        // （回退到初始值：flex-grow=0、flex-shrink=1，见 default_impl）。
        "flex-grow" => {
            if let Ok(v) = value.parse::<f64>() {
                if v >= 0.0 {
                    style.flex_grow = v;
                }
                return true;
            }
        }
        "flex-shrink" => {
            if let Ok(v) = value.parse::<f64>() {
                if v >= 0.0 {
                    style.flex_shrink = v;
                }
                return true;
            }
        }
        "flex-basis" => {
            if let Some(v) = parse_flex_basis(value) {
                style.flex_basis = v;
                return true;
            }
        }
        "gap" => {
            // gap 简写仅设置 style.gap（legacy 字段）
            // column_gap / row_gap 由各自的 longhand handler 设置，
            // 通过 shorthand expansion 生成的 "row-gap" / "column-gap" 声明。
            if let Some(v) = parse_length_fn(value) {
                style.gap = v;
                return true;
            }
        }
        "column-gap" => {
            if let Some(v) = parse_length_fn(value) {
                style.column_gap = v;
                return true;
            }
        }
        "order" => {
            if let Ok(v) = value.parse::<i32>() {
                style.order = v;
                return true;
            }
        }
        "top" => {
            if let Some(v) = parse_length_fn(value) {
                style.top = v;
                return true;
            }
        }
        "right" => {
            if let Some(v) = parse_length_fn(value) {
                style.right = v;
                return true;
            }
        }
        "bottom" => {
            if let Some(v) = parse_length_fn(value) {
                style.bottom = v;
                return true;
            }
        }
        "left" => {
            if let Some(v) = parse_length_fn(value) {
                style.left = v;
                return true;
            }
        }
        "z-index" => {
            if let Some(v) = parse_z_index(value) {
                style.z_index = v;
                return true;
            }
        }
        "overflow-x" => {
            if let Some(v) = values::parse_overflow(value) {
                style.overflow_x = v;
                return true;
            }
        }
        "overflow-y" => {
            if let Some(v) = values::parse_overflow(value) {
                style.overflow_y = v;
                return true;
            }
        }
        // ── Aspect Ratio 属性 ──
        "aspect-ratio" => {
            if value == "auto" {
                style.aspect_ratio = None;
                return true;
            }
            // 支持 "16 / 9" 或单个数值
            let ratio: f32 = if let Some(slash_pos) = value.find('/') {
                let w: f32 = match value[..slash_pos].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                let h: f32 = match value[slash_pos + 1..].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                if h == 0.0 {
                    return false;
                }
                w / h
            } else {
                match value.parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                }
            };
            style.aspect_ratio = Some(ratio);
            return true;
        }
        // ── Cursor 属性 ──
        "cursor" => {
            if let Some(v) = values::parse_cursor(value) {
                style.cursor = map_css_cursor(v);
                return true;
            }
        }
        // ── Grid 属性 ──
        "grid-template-columns" => {
            style.grid_template_columns = Some(value.to_string());
            return true;
        }
        "grid-template-rows" => {
            style.grid_template_rows = Some(value.to_string());
            return true;
        }
        "grid-auto-flow" => {
            if let Some(v) = parse_grid_auto_flow(value) {
                style.grid_auto_flow = v;
                return true;
            }
        }
        "grid-column-start" => {
            if let Some(v) = parse_grid_line(value) {
                style.grid_column_start = v;
                return true;
            }
        }
        "grid-column-end" => {
            if let Some(v) = parse_grid_line(value) {
                style.grid_column_end = v;
                return true;
            }
        }
        "grid-row-start" => {
            if let Some(v) = parse_grid_line(value) {
                style.grid_row_start = v;
                return true;
            }
        }
        "grid-row-end" => {
            if let Some(v) = parse_grid_line(value) {
                style.grid_row_end = v;
                return true;
            }
        }
        "grid-auto-rows" => {
            style.grid_auto_rows = Some(value.to_string());
            return true;
        }
        "grid-auto-columns" => {
            style.grid_auto_columns = Some(value.to_string());
            return true;
        }
        "grid-template-areas" => {
            style.grid_template_areas = Some(value.to_string());
            return true;
        }
        // ── Grid 简写属性 ──
        "grid-area" => {
            if let Some((rs, re, cs, ce)) = parse_grid_area_shorthand(value) {
                style.grid_row_start = rs;
                style.grid_row_end = re;
                style.grid_column_start = cs;
                style.grid_column_end = ce;
                return true;
            }
        }
        "grid-column" => {
            if let Some((start, end)) = parse_grid_line_shorthand(value) {
                style.grid_column_start = start;
                style.grid_column_end = end;
                return true;
            }
        }
        "grid-row" => {
            if let Some((start, end)) = parse_grid_line_shorthand(value) {
                style.grid_row_start = start;
                style.grid_row_end = end;
                return true;
            }
        }
        "row-gap" => {
            if let Some(v) = parse_length_fn(value) {
                style.row_gap = v;
                return true;
            }
        }
        _ => {
            // 高级属性委托给 apply_advanced 模块
            if super::apply_advanced::apply_advanced_property_value(style, property, value) {
                return true;
            }
        }
    }
    false
}
