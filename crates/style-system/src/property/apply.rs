//! CSS 属性值应用到 ComputedStyle。

use super::parse::*;
use super::types::*;
use zero_css_parser::values;

/// https://drafts.csswg.org/css-fonts-4/#absolute-size-mapping
fn parse_font_size_keyword(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    match v.to_ascii_lowercase().as_str() {
        "xx-small" => Some(LengthValue::Px(9.0)),
        "x-small" => Some(LengthValue::Px(10.0)),
        "small" => Some(LengthValue::Px(13.0)),
        "medium" => Some(LengthValue::Px(16.0)),
        "large" => Some(LengthValue::Px(18.0)),
        "x-large" => Some(LengthValue::Px(24.0)),
        "xx-large" => Some(LengthValue::Px(32.0)),
        "xxx-large" => Some(LengthValue::Px(48.0)),
        "smaller" => Some(LengthValue::Em(0.8333)),
        "larger" => Some(LengthValue::Em(1.2)),
        _ => None,
    }
}

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
    apply_property_value_with_quirks(style, property, value, false, false)
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
    prefers_dark: bool,
) -> bool {
    // 不在此 trim：声明值经 consume_declaration deferred-whitespace 已无首尾空白 token
    // （inline style 经 parse_inline_style 自行 trim，presentational hints 产 clean 值，
    // cascade apply-on-dummy 传 cascaded 值）。此处 trim 会误剥**转义产生的**空白
    //（如 `red\9` → `red\t`），与 parse_color 不再 trim 配合使非法颜色被正确拒绝。
    // driving：escapes-014/015/016（apply 拒绝→cascade R2126 丢弃→下个合法声明胜出）。
    // 颜色解析：light-dark(L, D) 按本元素 used color-scheme（style.color_scheme_dark）取参。
    // color_scheme_dark 由 compute_inherited_style_with_quirks 预解析先行设置（CSS 规定
    // color-scheme 先于其他属性计算），故此处读取已反映显式声明/继承的暗 scheme。
    // quirks 路径忽略 dark（quirks+light-dark 极罕见，保持 parse_color_quirks 原行为）。
    let dark = style.color_scheme_dark;
    let quirks = quirks_mode;
    let parse_color_fn = |value: &str| -> Option<zero_css_parser::values::ColorValue> {
        if quirks {
            values::parse_color_quirks(value)
        } else {
            values::parse_color_with_scheme(value, dark)
        }
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
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.width = v;
                return true;
            }
        }
        "height" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
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
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.width = v;
                return true;
            }
        }
        "block-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.height = v;
                return true;
            }
        }
        // CSS Logical Properties §7：min/max 逻辑尺寸——水平模式等价 min/max-width/height
        //（垂直模式轴交换由 converter 的 swap_writing_mode_axes 负责，同 inline-size/block-size）。
        "min-inline-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.min_width = v;
                return true;
            }
        }
        "min-block-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.min_height = v;
                return true;
            }
        }
        "max-inline-size" => {
            if value.eq_ignore_ascii_case("none") {
                style.max_width = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.max_width = v;
                return true;
            }
        }
        "max-block-size" => {
            if value.eq_ignore_ascii_case("none") {
                style.max_height = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.max_height = v;
                return true;
            }
        }
        "min-width" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.min_width = v;
                return true;
            }
        }
        "min-height" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.min_height = v;
                return true;
            }
        }
        "max-width" => {
            if value.eq_ignore_ascii_case("none") {
                style.max_width = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.max_width = v;
                return true;
            }
        }
        "max-height" => {
            if value.eq_ignore_ascii_case("none") {
                style.max_height = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.max_height = v;
                return true;
            }
        }
        "margin-top" => {
            if let Some(v) = parse_length_fn(value) {
                if margin_length_is_valid(value, &v) {
                    style.margin_top = v;
                    return true;
                }
            }
        }
        "margin-right" => {
            if let Some(v) = parse_length_fn(value) {
                if margin_length_is_valid(value, &v) {
                    style.margin_right = v;
                    return true;
                }
            }
        }
        "margin-bottom" => {
            if let Some(v) = parse_length_fn(value) {
                if margin_length_is_valid(value, &v) {
                    style.margin_bottom = v;
                    return true;
                }
            }
        }
        "margin-left" => {
            if let Some(v) = parse_length_fn(value) {
                if margin_length_is_valid(value, &v) {
                    style.margin_left = v;
                    return true;
                }
            }
        }
        "padding-top" => {
            if let Some(v) = parse_length_fn(value) {
                if !padding_length_is_valid(value, &v) {
                    return false;
                }
                style.padding_top = v;
                return true;
            }
        }
        "padding-right" => {
            if let Some(v) = parse_length_fn(value) {
                if !padding_length_is_valid(value, &v) {
                    return false;
                }
                style.padding_right = v;
                return true;
            }
        }
        "padding-bottom" => {
            if let Some(v) = parse_length_fn(value) {
                if !padding_length_is_valid(value, &v) {
                    return false;
                }
                style.padding_bottom = v;
                return true;
            }
        }
        "padding-left" => {
            if let Some(v) = parse_length_fn(value) {
                if !padding_length_is_valid(value, &v) {
                    return false;
                }
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
                if !border_width_length_is_valid(value, &v) {
                    return false;
                }
                style.border_top_width = v;
                return true;
            }
        }
        "border-right-width" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_width_length_is_valid(value, &v) {
                    return false;
                }
                style.border_right_width = v;
                return true;
            }
        }
        "border-bottom-width" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_width_length_is_valid(value, &v) {
                    return false;
                }
                style.border_bottom_width = v;
                return true;
            }
        }
        "border-left-width" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_width_length_is_valid(value, &v) {
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
                if !border_radius_length_is_valid(value, &v) {
                    return false;
                }
                style.border_top_left_radius = v;
                return true;
            }
        }
        "border-top-right-radius" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_radius_length_is_valid(value, &v) {
                    return false;
                }
                style.border_top_right_radius = v;
                return true;
            }
        }
        "border-bottom-right-radius" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_radius_length_is_valid(value, &v) {
                    return false;
                }
                style.border_bottom_right_radius = v;
                return true;
            }
        }
        "border-bottom-left-radius" => {
            if let Some(v) = parse_length_fn(value) {
                if !border_radius_length_is_valid(value, &v) {
                    return false;
                }
                style.border_bottom_left_radius = v;
                return true;
            }
        }
        // ── Outline 属性 ──
        "outline-width" => {
            if let Some(v) = parse_length_fn(value) {
                if border_width_length_is_valid(value, &v) {
                    style.outline_width = v;
                    return true;
                }
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
            // CSS-UI-4 §4.4: `inset` 关键字——outline 绘制在 border-box 内侧
            // （≡ 负 outline-width 偏移）。长度与 inset 互斥，赋长度时清标记。
            let trimmed = value.trim();
            if trimmed.eq_ignore_ascii_case("inset") {
                style.outline_offset_inset = true;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if outline_offset_length_is_valid(value, &v) {
                    style.outline_offset = v;
                    style.outline_offset_inset = false;
                    return true;
                }
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
        "color-scheme" => {
            // color-scheme 影响本元素 light-dark() 解析，须在颜色属性应用前确定
            //（compute_inherited_style_with_quirks 的预解析已先行设置；此处对显式声明同步）。
            // used-scheme 与 prefers-color-scheme 合成（见 parse_color_scheme_dark）。
            style.color_scheme_dark = parse_color_scheme_dark(value, prefers_dark);
            return true;
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
            // https://drafts.csswg.org/css-fonts-4/#absolute-size-mapping
            if let Some(v) = parse_font_size_keyword(value) {
                style.font_size = v;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if font_size_length_is_valid(value, &v) {
                    style.font_size = v;
                    return true;
                }
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
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis
        "font-synthesis" => {
            if let Some(v) = values::parse_font_synthesis(value) {
                style.font_synthesis = v;
                return true;
            }
        }
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis-weight
        "font-synthesis-weight" => match value.trim().to_ascii_lowercase().as_str() {
            "auto" => {
                style.font_synthesis.weight = true;
                return true;
            }
            "none" => {
                style.font_synthesis.weight = false;
                return true;
            }
            _ => {}
        },
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis-style
        "font-synthesis-style" => match value.trim().to_ascii_lowercase().as_str() {
            "auto" => {
                style.font_synthesis.style = true;
                return true;
            }
            "none" => {
                style.font_synthesis.style = false;
                return true;
            }
            _ => {}
        },
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis-small-caps
        "font-synthesis-small-caps" => match value.trim().to_ascii_lowercase().as_str() {
            "auto" => {
                style.font_synthesis.small_caps = true;
                return true;
            }
            "none" => {
                style.font_synthesis.small_caps = false;
                return true;
            }
            _ => {}
        },
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis-position
        "font-synthesis-position" => match value.trim().to_ascii_lowercase().as_str() {
            "auto" => {
                style.font_synthesis.position = true;
                return true;
            }
            "none" => {
                style.font_synthesis.position = false;
                return true;
            }
            _ => {}
        },
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
                    values::TextDecorationThicknessValue::Length(lv) => {
                        super::types::TextDecorationThicknessValue::Length(lv)
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
        "text-underline-offset" => {
            if let Some(v) = values::parse_text_underline_offset(value) {
                style.text_underline_offset = v;
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
        "text-emphasis-color" => {
            if let Some(v) = parse_color_fn(value) {
                style.text_emphasis_color = v;
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
            if value.trim().eq_ignore_ascii_case("normal") {
                style.letter_spacing = LengthValue::Px(0.0);
                style.letter_spacing_normal = true;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if letter_spacing_length_is_valid(value, &v) {
                    style.letter_spacing = v;
                    style.letter_spacing_normal = false;
                    return true;
                }
            }
        }
        "word-spacing" => {
            if value.trim().eq_ignore_ascii_case("normal") {
                style.word_spacing = LengthValue::Px(0.0);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if text_indent_length_is_valid(value, &v) {
                    style.word_spacing = v;
                    return true;
                }
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
                if text_indent_length_is_valid(value, &v) {
                    style.text_indent = v;
                    return true;
                }
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
        // CSS Flexbox §7.3.1/§7.3.2：flex-grow/flex-shrink 须为非负 <number>，负值/Infinity
        // 非法，按未声明处理（回退到初始值：flex-grow=0、flex-shrink=1，见 default_impl）。
        // R3345 deep-review：旧实现仅检 `v >= 0.0`——`Infinity >= 0.0` 为真 → 无穷大值被存储，
        // 经 `style.flex_grow as f32` 喂入 Taffy flex 算法（converter/mod.rs）可致 flex 分配异常。
        // 加 `is_finite()` 前置，与 `flex` 简写（shorthand/mod.rs is_number 用 is_finite）一致。
        // https://www.w3.org/TR/css-flexbox-1/#flex-grow-property
        "flex-grow" => {
            if let Ok(v) = value.parse::<f64>()
                && v.is_finite()
            {
                if v >= 0.0 {
                    style.flex_grow = v;
                }
                return true;
            }
        }
        "flex-shrink" => {
            if let Ok(v) = value.parse::<f64>()
                && v.is_finite()
            {
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
            if value.eq_ignore_ascii_case("normal") {
                style.gap = LengthValue::Px(0.0);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if gap_length_is_valid(value, &v) {
                    style.gap = v;
                    return true;
                }
            }
        }
        "column-gap" => {
            if value.eq_ignore_ascii_case("normal") {
                style.column_gap = LengthValue::Auto;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !gap_length_is_valid(value, &v) {
                    return false;
                }
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
                if positioned_offset_length_is_valid(value, &v) {
                    style.top = v;
                    return true;
                }
            }
        }
        "right" => {
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.right = v;
                    return true;
                }
            }
        }
        "bottom" => {
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.bottom = v;
                    return true;
                }
            }
        }
        "left" => {
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.left = v;
                    return true;
                }
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
        // CSS Overflow 3 §3：overflow-clip-margin（仅对 overflow:clip 生效，paint 期消费）。
        "overflow-clip-margin" => {
            if let Some(v) = values::parse_overflow_clip_margin(value) {
                style.overflow_clip_margin = v;
                return true;
            }
        }
        // ── Aspect Ratio 属性 ──
        "aspect-ratio" => {
            let v = value.trim();
            if v.eq_ignore_ascii_case("auto") {
                style.aspect_ratio = None;
                style.aspect_ratio_auto = true;
                return true;
            }
            // CSS Aspect Ratio §3：`auto <ratio>` 组合语法。R2440：建模 auto flag——
            // `auto` 优先 replaced 元素固有比，`<ratio>` 仅 fallback（apply_replaced_element_sizing
            // 据 auto + img_intrinsic_sizes 覆盖为固有比）。剥 auto 前缀（须为独立 token）。
            let (ratio_str, has_auto): (&str, bool) = if v.len() >= 5
                && v.as_bytes()[..4].eq_ignore_ascii_case(b"auto")
                && v.as_bytes()[4].is_ascii_whitespace()
            {
                (v[4..].trim(), true)
            } else {
                (v, false)
            };
            // 支持 "16 / 9" 或单个数值
            let ratio: f32 = if let Some(slash_pos) = ratio_str.find('/') {
                let w: f32 = match ratio_str[..slash_pos].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                let h: f32 = match ratio_str[slash_pos + 1..].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                if !w.is_finite() || !h.is_finite() || h == 0.0 {
                    return false;
                }
                w / h
            } else {
                match ratio_str.parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                }
            };
            if !ratio.is_finite() {
                return false;
            }
            style.aspect_ratio = Some(ratio);
            style.aspect_ratio_auto = has_auto;
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
            if value.eq_ignore_ascii_case("normal") {
                style.row_gap = LengthValue::Px(0.0);
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !gap_length_is_valid(value, &v) {
                    return false;
                }
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

fn gap_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite() && *v >= 0.0,
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn sizing_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    let raw = raw.trim().to_ascii_lowercase();
    if matches!(
        raw.as_str(),
        "thin" | "medium" | "thick" | "fit-content(thin)" | "fit-content(medium)" | "fit-content(thick)"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite() && *v >= 0.0,
        LengthValue::FitContent(inner) => sizing_length_is_valid("", inner),
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::Calc(_) => true,
    }
}

pub(crate) fn padding_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite() && *v >= 0.0,
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn border_radius_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    padding_length_is_valid(raw, value)
}

fn font_size_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    padding_length_is_valid(raw, value)
}

pub(crate) fn border_width_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    let raw = raw.trim().to_ascii_lowercase();
    match raw.as_str() {
        "thin" | "medium" | "thick" => true,
        "auto" | "min-content" | "max-content" | "fit-content" => false,
        _ => match value {
            LengthValue::Px(v)
            | LengthValue::Em(v)
            | LengthValue::Ex(v)
            | LengthValue::Rex(v)
            | LengthValue::Cap(v)
            | LengthValue::Rcap(v)
            | LengthValue::Rem(v)
            | LengthValue::Vh(v)
            | LengthValue::Vw(v)
            | LengthValue::Vmin(v)
            | LengthValue::Vmax(v)
            | LengthValue::Ch(v)
            | LengthValue::Rch(v)
            | LengthValue::Ic(v)
            | LengthValue::Ric(v) => v.is_finite() && *v >= 0.0,
            LengthValue::Calc(_) => true,
            _ => false,
        },
    }
}

fn letter_spacing_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(raw.trim().to_ascii_lowercase().as_str(), "thin" | "medium" | "thick") {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v) => v.is_finite(),
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn outline_offset_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v) => v.is_finite(),
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn text_indent_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite(),
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

pub(crate) fn positioned_offset_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Auto => true,
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite(),
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

pub(crate) fn margin_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    positioned_offset_length_is_valid(raw, value)
}

/// 解析 `color-scheme` 描述符为「是否暗 used-scheme」标志（CSS Color Adjust L1 §2.3）。
///
/// used color-scheme 由属性声明的 scheme 列表与用户偏好（`prefers-color-scheme`）合成：
/// - 列表含 `dark` 与 `light`（如 `light dark`、`dark light`、`only light dark`）→ 由
///   `prefers_dark` 决定（两种 scheme 均可用，用户偏好取胜）；
/// - 列表仅含 `dark`（含 `only dark`、`dark only`）→ dark；
/// - 列表仅含 `light`（含 `only light`）→ light；
/// - 列表不含 light/dark（`normal` / 缺省 / 仅 custom-ident）→ 保守取 light（ZW 默认，
///   不让未显式 opt-in 的页面在暗 OS 上整体翻转）。
///
/// `prefers_dark` 来自 `prefers-color-scheme` 媒体查询（自渲染 reftest 恒为 light=false，
/// 故合成对 reftest 字节级等价、零回归；仅真实浏览器暗 OS / 单测 prefers=true 时激活）。
/// driving: css-variables registered-property-light-dark + 全局暗模式 theming。
pub(crate) fn parse_color_scheme_dark(value: &str, prefers_dark: bool) -> bool {
    let mut has_light = false;
    let mut has_dark = false;
    for tok in value.split_whitespace() {
        if tok.eq_ignore_ascii_case("light") {
            has_light = true;
        } else if tok.eq_ignore_ascii_case("dark") {
            has_dark = true;
        }
    }
    match (has_light, has_dark) {
        (true, true) => prefers_dark, // 两种均可用 → 用户偏好决定
        (false, true) => true,        // 仅 dark
        _ => false,                   // 仅 light / normal / 缺省 → light（保守默认）
    }
}
