//! CSS 属性值应用到 ComputedStyle。

use super::parse::*;
use super::types::*;
use crate::computed::calc_contains_percentage;
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
    apply_property_value_with_quirks(style, property, value, false, false, None)
}

/// 将属性字符串值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
///
/// 当 `quirks_mode` 为 true 时，颜色解析使用 quirks mode 宽容规则
/// （如接受不带 # 的十六进制、纯数字颜色值），长度解析接受裸数字视为 px。
///
/// `sibling`：源元素 `(兄弟序, 兄弟总数)`（1 基；CSS Values 5 §tree-counting），
/// 供 sibling-index()/sibling-count() 树求值；None = 探测/无 DOM 视图（求值
/// fail-closed）。探测面（cascade 合法性/CSS.supports）传序数无关哨兵
/// `Some((1, 1))`——合法性判定不依赖真值（范围敏感属性除外，在册局限）。
pub fn apply_property_value_with_quirks(
    style: &mut ComputedStyle,
    property: &str,
    value: &str,
    quirks_mode: bool,
    // meta color-scheme 落地后 used-scheme 判定收口到 inheritance 预解析臂（含
    // normal→文档 scheme 回落），此处的 prefers 不再直接参与——保留参数以兼容
    // 调用点签名。
    _prefers_dark: bool,
    sibling: Option<(u64, u64)>,
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
                // light-dark() 图像上下文选支（css-color-5 §funcdef-light-dark）：
                // 按 used color-scheme dark flag computed 期展开；支内 none 计算
                // 为透明图像（#valdef-light-dark-none）。
                fn resolve(
                    v: zero_css_parser::values::ListStyleImageValue,
                    dark: bool,
                    in_branch: bool,
                ) -> ListStyleImageComputedValue {
                    use zero_css_parser::values::ListStyleImageValue;
                    match v {
                        ListStyleImageValue::None => {
                            if in_branch {
                                ListStyleImageComputedValue::TransparentImage
                            } else {
                                ListStyleImageComputedValue::None
                            }
                        }
                        ListStyleImageValue::Url(url) => ListStyleImageComputedValue::Url(url),
                        ListStyleImageValue::LightDark {
                            light,
                            dark: dark_branch,
                        } => {
                            let pick = if dark { *dark_branch } else { *light };
                            resolve(pick, dark, true)
                        }
                    }
                }
                let dark = style.color_scheme_dark;
                style.list_style_image = resolve(v, dark, false);
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
        // 逻辑尺寸属性 inline-size / block-size 及 min/max 变体（CSS Logical Properties
        // §1/§7 + css-writing-modes-3 §7.1）。R5128：按**元素最终 writing-mode** 定轴——
        // inline 轴 = horizontal-tb 物理 width / 垂直模式物理 height；block 轴反之。
        // 旧实现钉死 horizontal-tb（级联 canonical 化）+ converter 轴交换把映射值当物理值
        // 再交换 → 垂直模式落行内轴（multicol-span-all-rule-002 block-size 全隐实证）。
        // R4448/R4449 延迟臂保证调用点 writing-mode 已定态；kill-switch ZW_LOGICAL_SIZE=0
        // 时级联已 canonical 化，本组臂仅剩 horizontal-tb 路径可达（语义不变）。
        "inline-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.height = v;
                } else {
                    style.width = v;
                }
                return true;
            }
        }
        "block-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.width = v;
                } else {
                    style.height = v;
                }
                return true;
            }
        }
        "min-inline-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.min_height = v;
                } else {
                    style.min_width = map_bare_fit_content_avail(value, v);
                }
                return true;
            }
        }
        "min-block-size" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.min_width = v;
                } else {
                    style.min_height = v;
                }
                return true;
            }
        }
        "max-inline-size" => {
            if value.eq_ignore_ascii_case("none") {
                if style.writing_mode.is_vertical_block_flow() {
                    style.max_height = LengthValue::Px(f64::INFINITY);
                } else {
                    style.max_width = LengthValue::Px(f64::INFINITY);
                }
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.max_height = v;
                } else {
                    style.max_width = map_bare_fit_content_avail(value, v);
                }
                return true;
            }
        }
        "max-block-size" => {
            if value.eq_ignore_ascii_case("none") {
                if style.writing_mode.is_vertical_block_flow() {
                    style.max_width = LengthValue::Px(f64::INFINITY);
                } else {
                    style.max_height = LengthValue::Px(f64::INFINITY);
                }
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                if style.writing_mode.is_vertical_block_flow() {
                    style.max_width = v;
                } else {
                    style.max_height = v;
                }
                return true;
            }
        }
        "min-width" => {
            if let Some(v) = parse_length_fn(value) {
                if !sizing_length_is_valid(value, &v) {
                    return false;
                }
                style.min_width = map_bare_fit_content_avail(value, v);
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
                style.max_width = map_bare_fit_content_avail(value, v);
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
            if let Some((rx, ry)) = parse_radius_two_axis(value, parse_length_fn) {
                set_radius_y_slot(style, 0, ry, &rx);
                style.border_top_left_radius = rx;
                return true;
            }
        }
        "border-top-right-radius" => {
            if let Some((rx, ry)) = parse_radius_two_axis(value, parse_length_fn) {
                set_radius_y_slot(style, 1, ry, &rx);
                style.border_top_right_radius = rx;
                return true;
            }
        }
        "border-bottom-right-radius" => {
            if let Some((rx, ry)) = parse_radius_two_axis(value, parse_length_fn) {
                set_radius_y_slot(style, 2, ry, &rx);
                style.border_bottom_right_radius = rx;
                return true;
            }
        }
        "border-bottom-left-radius" => {
            if let Some((rx, ry)) = parse_radius_two_axis(value, parse_length_fn) {
                set_radius_y_slot(style, 3, ry, &rx);
                style.border_bottom_left_radius = rx;
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
                // R5383：级联胜出的作者声明（含显式 none）压制 UA 焦点环
                //（css-ui-4 §6.1 级联序；outline 简写展开亦经此处）。
                style.outline_style_declared = true;
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
            // used-scheme 已由 compute_inherited_style_with_quirks 的预解析臂权威设定
            //（同一 cascaded 值先行流经该臂，含 normal→文档 scheme 回落与 inherit 语义；
            // color-scheme 影响本元素 light-dark() 解析故须在颜色属性应用前确定）。
            // 此臂只确认属性可应用——覆写会丢 normal/inherit 语义（语料
            // normal-descendant-change 钉死），decl 同步亦在预解析臂完成。
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
            let families = parse_font_family(value);
            if !families.is_empty() {
                style.font_family = families;
                return true;
            }
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
        "text-decoration-skip-spaces" => {
            if let Some(v) = values::parse_text_decoration_skip_spaces(value) {
                style.text_decoration_skip_spaces = v;
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
        "ruby-position" => {
            if let Some(v) = values::parse_ruby_position(value) {
                style.ruby_position = v;
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
        "transform-box" => {
            if let Some(v) = parse_transform_box(value) {
                style.transform_box = v;
                return true;
            }
        }
        "stroke-width" => {
            if let Some(v) = parse_stroke_width(value) {
                style.stroke_width = v;
                return true;
            }
        }
        "fill" => {
            // R4280：SVG2 presentation `fill`（V1 色值子集；`none` → Transparent——
            // 视觉等价于不绘制；url() paint server 未支持 → 声明丢弃）。
            // system color 按 used color-scheme 在 computed 值时间解析为绝对色
            //（css-color-4 §resolving-color-values；dark 传播同 apply_advanced 四臂
            // parse_color_with_scheme 形态）。继承见 inherit_property fill/stroke 臂。
            // driving: system-color-compute fill 差分断言（parent light / child dark）。
            let v = value.trim();
            if v.eq_ignore_ascii_case("none") {
                style.fill = Some(zero_css_parser::values::ColorValue::Transparent);
                return true;
            }
            if let Some(c) = zero_css_parser::values::parse_color_with_scheme(v, style.color_scheme_dark) {
                style.fill = Some(c);
                return true;
            }
        }
        "stroke" => {
            // R5187：SVG2 presentation `stroke`（V1 色值子集，与 fill R4280 同型——
            // `none` → Transparent（视觉等价于不描边）；url() paint server 未支持 →
            // 声明丢弃）。system color dark 传播同 fill。继承见 inherit_property 臂。
            // driving: system-color-compute stroke 差分断言。
            let v = value.trim();
            if v.eq_ignore_ascii_case("none") {
                style.stroke = Some(zero_css_parser::values::ColorValue::Transparent);
                return true;
            }
            if let Some(c) = zero_css_parser::values::parse_color_with_scheme(v, style.color_scheme_dark) {
                style.stroke = Some(c);
                return true;
            }
        }
        "writing-mode" => {
            if let Some(v) = parse_writing_mode(value) {
                style.writing_mode = v;
                // R4443：sideways-lr 旁路标记（声明值区分，供 paint 字形取向/行内方向）。
                style.writing_mode_sideways_lr = value.trim().eq_ignore_ascii_case("sideways-lr");
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
        // CSS Flexbox §7.3.1/§7.3.2：flex-grow/flex-shrink 值域 `<number [0,∞]>`——负值
        // 非法按未声明处理（回退到初始值：flex-grow=0、flex-shrink=1，见 default_impl）。
        // R5333（css-values-4 math functions × driving flex-grow-009）：math 表达式先求值
        // ——`calc(infinity)` 是合法 flex-grow（旧实现 `parse::<f64>()` 对 calc 字符串直接
        // 失败整条拒收）。R3345 旧定性「Infinity 非法」证伪：值域上界含 ∞，√∞ 因子由
        // taffy R5333 分配臂实现「∞ 均分全部 free space」语义（flex-grow 放行 +∞）；
        // flex-shrink 值域同含 ∞ 但 taffy shrink 分配臂无 ∞ 语义（scaled factor ∞/∞ NaN），
        // 维持拒 ∞（挂账：taffy shrink ∞ 臂），普通 math 表达式放行。
        // https://www.w3.org/TR/css-flexbox-1/#flex-grow-property
        "flex-grow" => {
            let v = value.parse::<f64>().ok().or_else(|| {
                zero_css_parser::values::parse_math_function(value)
                    .and_then(|e| zero_css_parser::values::eval_calc(&e, None))
            });
            // `v >= 0.0` 同时拒负值、-∞ 与 NaN；+∞ 是值域合法上界
            if let Some(v) = v
                && v >= 0.0
            {
                style.flex_grow = v;
                return true;
            }
        }
        "flex-shrink" => {
            let v = value.parse::<f64>().ok().or_else(|| {
                zero_css_parser::values::parse_math_function(value)
                    .and_then(|e| zero_css_parser::values::eval_calc(&e, None))
            });
            if let Some(v) = v
                && v.is_finite()
                && v >= 0.0
            {
                style.flex_shrink = v;
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
            // <integer> 文法位接受顶层数学函数（CSS Values 4 §10.5 型别表：
            // math function 可用于对应类型文法位；tree-counting 页 `order:
            // calc(2 * sibling-count())` 形态驱动）。型别须整体 <number>，
            // 求值四舍五入取整 + i32 clamp（与 parse_z_index 同口径）。
            // https://drafts.csswg.org/css-values-4/#calc-type-checking
            if let Some(v) = super::parse::eval_int_math_value(value, sibling) {
                style.order = v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                return true;
            }
        }
        "top" => {
            // CSS Anchor Positioning §anchor()：inset 槽 anchor() 形原文承载
            //（布局期 anchor_pos pass 求值回填，槽位暂存 Auto=静态位基线）。
            // "anchor-size(" 无 "anchor(" 子串不误捕（inset 属性本不接受
            // anchor-size，非法形走既有 parse 拒面）。
            if value.contains("anchor(") {
                style.anchor_inset_top = Some(value.trim().to_ascii_lowercase().into_boxed_str());
                style.top = LengthValue::Auto;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.top = v;
                    return true;
                }
            }
        }
        "right" => {
            if value.contains("anchor(") {
                style.anchor_inset_right = Some(value.trim().to_ascii_lowercase().into_boxed_str());
                style.right = LengthValue::Auto;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.right = v;
                    return true;
                }
            }
        }
        "bottom" => {
            if value.contains("anchor(") {
                style.anchor_inset_bottom = Some(value.trim().to_ascii_lowercase().into_boxed_str());
                style.bottom = LengthValue::Auto;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.bottom = v;
                    return true;
                }
            }
        }
        "left" => {
            if value.contains("anchor(") {
                style.anchor_inset_left = Some(value.trim().to_ascii_lowercase().into_boxed_str());
                style.left = LengthValue::Auto;
                return true;
            }
            if let Some(v) = parse_length_fn(value) {
                if positioned_offset_length_is_valid(value, &v) {
                    style.left = v;
                    return true;
                }
            }
        }
        "z-index" => {
            // sibling ctx 注入：sibling-index()/sibling-count() 树随元素真值
            // 求值（tree-counting 页 zIndex '2' 断言驱动）；None = 探测路径
            // （哨兵由调用方注入）。
            if let Some(v) = parse_z_index_with_sibling(value, sibling) {
                style.z_index = v;
                return true;
            }
        }
        // ── CSS Anchor Positioning（css-anchor-position-1）四属性 ──
        // 原文小写存储（文法校验/区域解析单点在 engine dom_bindings，声明面批
        // anchor_props/position_area 同矩阵；样式承载只存原文供布局期消费）。
        // 空值声明（`prop: ;`）parse 层不达此处；`none`/`auto` 存 None 语义同未声明。
        "anchor-name" => {
            let v = value.trim().to_ascii_lowercase();
            if v.is_empty() || v == "none" {
                style.anchor_name = None;
            } else {
                style.anchor_name = Some(v.into_boxed_str());
            }
            return true;
        }
        "position-anchor" => {
            let v = value.trim().to_ascii_lowercase();
            if v.is_empty() || v == "auto" {
                style.position_anchor = None;
            } else {
                style.position_anchor = Some(v.into_boxed_str());
            }
            return true;
        }
        "position-area" => {
            let v = value.trim().to_ascii_lowercase();
            if v.is_empty() || v == "none" {
                style.position_area = None;
            } else {
                style.position_area = Some(v.into_boxed_str());
            }
            return true;
        }
        "position-try-fallbacks" => {
            // FIXME(css-anchor-position-1 §position-try-prop)：`position-try` 简写
            // 拆分臂未接（stylesheet 声明面；shim CSSOM 面已拆，part03）。
            let v = value.trim().to_ascii_lowercase();
            if v.is_empty() || v == "none" {
                style.position_try_fallbacks = None;
            } else {
                style.position_try_fallbacks = Some(v.into_boxed_str());
            }
            return true;
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
            // https://drafts.csswg.org/css-sizing-4/#aspect-ratio
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
                if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
                    // R3797：w==0 退化比（aspect-ratio:0/1，csswg #4572）——transferred
                    // height = width/0 = inf，taffy 全链 NaN 污染（zero-or-infinity-001：
                    // 红盒 100×inf，绿方块 y=NaN 不绘制）。css-sizing-4 §4.2 退化比按
                    // 无 ratio 处理（content-based sizing）——解析拒绝 = aspect_ratio None。
                    return false;
                }
                w / h
            } else {
                match ratio_str.parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                }
            };
            if !ratio.is_finite() || ratio < 0.0 {
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
            if let Some(v) = parse_grid_template_track_list(value) {
                style.grid_template_columns = v;
                return true;
            }
        }
        "grid-template-rows" => {
            if let Some(v) = parse_grid_template_track_list(value) {
                style.grid_template_rows = v;
                return true;
            }
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
            if let Some(v) = parse_grid_auto_track_list(value) {
                style.grid_auto_rows = v;
                return true;
            }
        }
        "grid-auto-columns" => {
            if let Some(v) = parse_grid_auto_track_list(value) {
                style.grid_auto_columns = v;
                return true;
            }
        }
        "grid-template-areas" => {
            if let Some(v) = parse_grid_template_areas_value(value) {
                style.grid_template_areas = v;
                return true;
            }
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
            if super::apply_advanced::apply_advanced_property_value(style, property, value, sibling) {
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Rlh(v)
        | LengthValue::Cqw(v)
        | LengthValue::Cqh(v)
        | LengthValue::Cqi(v)
        | LengthValue::Cqb(v)
        | LengthValue::Cqmin(v)
        | LengthValue::Cqmax(v)
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
        // g24：viewport 单位族 provenance 变体（此前 parse 期塌缩基类单位，
        // 接受面等价——消费方 sv*/lv*/dv* ≡ 基类、vi ≡ vw、vb ≡ vh）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Rlh(v)
        | LengthValue::Cqw(v)
        | LengthValue::Cqh(v)
        | LengthValue::Cqi(v)
        | LengthValue::Cqb(v)
        | LengthValue::Cqmin(v)
        | LengthValue::Cqmax(v)
        | LengthValue::Percentage(v) => v.is_finite() && *v >= 0.0,
        LengthValue::FitContent(inner) => sizing_length_is_valid("", inner),
        LengthValue::Auto
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::Calc(_)
        | LengthValue::Stretch => true,
    }
}

/// R5508（css-sizing-3 §5.2）：min/max-width 裸 `fit-content` 关键字保留 fit-content
/// 公式语义。共享 `parse_length` 把裸 fit-content 映射 MaxContent（R1018，width/height
/// 域 layout trigger 等价、保持不动），但 min/max-width 的 fit-content =
/// min(W_max, max(W_min, stretch-fit))——`max(W_min, avail)` 地板与 MaxContent（纯
/// W_max cap）在窄 CB 域分叉（chromium oracle 定谳：50px 容器内 width:300px +
/// max-width:fit-content + 不可断子 90px → used 90 溢出容器；MaxContent 别名经
/// sizing pass 写入走 CB 地板塌 50）。映射 `FitContent(100%)` ＝
/// fit-content(stretch-fit)（% arg 由 sizing pass 公式臂按 CB 解析），与显式
/// `fit-content(100%)` 拼写收敛（§5.2 同值）。仅物理水平轴槽位调用；块轴镜像
///（min/max-height）别名不动（独立子问题挂账）。
fn map_bare_fit_content_avail(raw: &str, value: LengthValue) -> LengthValue {
    if raw.eq_ignore_ascii_case("fit-content") {
        LengthValue::FitContent(Box::new(LengthValue::Percentage(100.0)))
    } else {
        value
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Rlh(v)
        | LengthValue::Cqw(v)
        | LengthValue::Cqh(v)
        | LengthValue::Cqi(v)
        | LengthValue::Cqb(v)
        | LengthValue::Cqmin(v)
        | LengthValue::Cqmax(v)
        | LengthValue::Percentage(v) => v.is_finite() && *v >= 0.0,
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn border_radius_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    padding_length_is_valid(raw, value)
}

/// R4574：border-*-radius 长hand 双轴值——`<h-radius> [ <v-radius> ]?`（CSS Backgrounds
/// §5.5 椭圆角；单值 = 两轴同值）。返 (rx, ry)；ry = None 表垂直分量未声明，槽位存声明 rx 副本。
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
fn parse_radius_two_axis(
    value: &str,
    parse_length_fn: impl Fn(&str) -> Option<LengthValue>,
) -> Option<(LengthValue, Option<LengthValue>)> {
    let parts = crate::shorthand::split_top_level_whitespace(value)?;
    let parse_axis = |raw: &str| -> Option<LengthValue> {
        let v = parse_length_fn(raw)?;
        border_radius_length_is_valid(raw, &v).then_some(v)
    };
    match parts.len() {
        1 => {
            let rx = parse_axis(parts[0])?;
            Some((rx, None))
        }
        2 => {
            let rx = parse_axis(parts[0])?;
            let ry = parse_axis(parts[1])?;
            Some((rx, Some(ry)))
        }
        _ => None,
    }
}

/// R4574：写四角垂直轴槽位（角序 [tl, tr, br, bl]）。ry=None（单值声明）回**水平声明值
/// 副本**（R5341——css-backgrounds-3 §5.1 两轴百分比各相对对应边，单值声明的垂直
/// 半径 = 同声明值重按高解析，消费侧按声明值直读无哨兵分支；原 Px(0) 哨兵使
/// `50%` 于槽位存在时垂直轴误 0）；显式 `h 0` 的 Some(Px(0)) 与哨兵不再歧义。
/// 双侧皆缺省零分配（None 常态零开销——ComputedStyle 每节点一份，R3867 帧体量纪律）。
fn set_radius_y_slot(style: &mut ComputedStyle, index: usize, ry: Option<LengthValue>, rx_declared: &LengthValue) {
    match (ry, style.border_radius_y.as_mut()) {
        (None, None) => {}
        (ry, Some(slot)) => slot[index] = ry.unwrap_or_else(|| rx_declared.clone()),
        (Some(ry), None) => {
            // 建槽时其余角存**当前水平声明值副本**（语义：槽位值恒为该角垂直声明值；
            // 早前单值声明角不因建槽丢垂直轴）。Px(0) 默认仅当该角声明亦为 0。
            let declared = [
                &style.border_top_left_radius,
                &style.border_top_right_radius,
                &style.border_bottom_right_radius,
                &style.border_bottom_left_radius,
            ];
            let slot = style.border_radius_y.get_or_insert_with(|| {
                Box::new([
                    declared[0].clone(),
                    declared[1].clone(),
                    declared[2].clone(),
                    declared[3].clone(),
                ])
            });
            slot[index] = ry;
        }
    }
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
            // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
            | LengthValue::Svh(v)
            | LengthValue::Svw(v)
            | LengthValue::Svmin(v)
            | LengthValue::Svmax(v)
            | LengthValue::Svb(v)
            | LengthValue::Svi(v)
            | LengthValue::Lvh(v)
            | LengthValue::Lvw(v)
            | LengthValue::Lvmin(v)
            | LengthValue::Lvmax(v)
            | LengthValue::Lvb(v)
            | LengthValue::Lvi(v)
            | LengthValue::Dvh(v)
            | LengthValue::Dvw(v)
            | LengthValue::Dvmin(v)
            | LengthValue::Dvmax(v)
            | LengthValue::Dvb(v)
            | LengthValue::Dvi(v)
            | LengthValue::Vb(v)
            | LengthValue::Vi(v)
            | LengthValue::Vh(v)
            | LengthValue::Vw(v)
            | LengthValue::Vmin(v)
            | LengthValue::Vmax(v)
            | LengthValue::Ch(v)
            | LengthValue::Rch(v)
            | LengthValue::Ic(v)
            | LengthValue::Ric(v)
            | LengthValue::Rlh(v) => v.is_finite() && *v >= 0.0,
            LengthValue::Calc(_) => true,
            _ => false,
        },
    }
}

pub(crate) fn letter_spacing_length_is_valid(raw: &str, value: &LengthValue) -> bool {
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Rlh(v) => v.is_finite(),
        // R4228（css-text-4 #letter-spacing）：`normal | <length-percentage>`——百分比
        // 相对 used font-size（消费方 resolve 期按 font-size 解析），合法值接受。
        LengthValue::Percentage(v) => v.is_finite(),
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Lh(v)
        | LengthValue::Rlh(v) => v.is_finite(),
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

pub(crate) fn text_indent_length_is_valid(raw: &str, value: &LengthValue) -> bool {
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Rlh(v)
        | LengthValue::Percentage(v) => v.is_finite(),
        // calc 树须 <length-percentage> 族（§10.5 型别一致）：纯 number 式
        //（`calc(5)`、calc-mix 全 number 项）拒——calc-mix-invalid 语料
        // `test_invalid_length_percentage('calc-mix(1 50%, 3 50%)')` 驱动。
        LengthValue::Calc(expr) => values::calc_expr_is_length_percentage(expr),
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
        // g24：viewport 单位族 provenance 变体，接受面等价（此前 parse 期塌缩基类单位）。
        | LengthValue::Svh(v)
        | LengthValue::Svw(v)
        | LengthValue::Svmin(v)
        | LengthValue::Svmax(v)
        | LengthValue::Svb(v)
        | LengthValue::Svi(v)
        | LengthValue::Lvh(v)
        | LengthValue::Lvw(v)
        | LengthValue::Lvmin(v)
        | LengthValue::Lvmax(v)
        | LengthValue::Lvb(v)
        | LengthValue::Lvi(v)
        | LengthValue::Dvh(v)
        | LengthValue::Dvw(v)
        | LengthValue::Dvmin(v)
        | LengthValue::Dvmax(v)
        | LengthValue::Dvb(v)
        | LengthValue::Dvi(v)
        | LengthValue::Vb(v)
        | LengthValue::Vi(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Rlh(v)
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
pub(crate) fn color_scheme_tokens(value: &str) -> (bool, bool) {
    let mut has_light = false;
    let mut has_dark = false;
    for tok in value.split_whitespace() {
        if tok.eq_ignore_ascii_case("light") {
            has_light = true;
        } else if tok.eq_ignore_ascii_case("dark") {
            has_dark = true;
        }
    }
    (has_light, has_dark)
}

pub(crate) fn parse_color_scheme_dark(value: &str, prefers_dark: bool) -> bool {
    let (has_light, has_dark) = color_scheme_tokens(value);
    match (has_light, has_dark) {
        (true, true) => prefers_dark, // 两种均可用 → 用户偏好决定
        (false, true) => true,        // 仅 dark
        _ => false,                   // 仅 light / normal / 缺省 → light（保守默认）
    }
}

/// inline style 写面值文法校验（cssom-1 §CSSStyleDeclaration 属性写 + css-values-4
/// 文法）——`el.style.rotate = 'asin()'` 等非法形须整体拒绝（赋值忽略保留原值，
/// WPT invalid 面 `acos-asin-atan-atan2-invalid.html` 63 子测试驱动）。
///
/// **白名单口径**（FIXME: 逐 parser 覆盖扩面）：仅校验引擎文法已验证支持的属性；
/// 表外属性一律放行——registry 认但 parser 有假阴性缺口的属性（grid 子格形、
/// 未接线的函数形等）全量收紧会把合法值误拒（活站 inline style 兼容面回退）。
/// shorthand 属性（apply 链无展开臂，展开在 cascade 层）与 css-wide keyword
/// （Cascading 4 §7.3 上下文无关）同样放行。空值 = 声明移除，恒真。
pub fn validate_decl_value(property: &str, value: &str) -> bool {
    const VALIDATED_PROPERTIES: [&str; 13] = [
        "transform",
        "rotate",
        "scale",
        "translate",
        "z-index",
        "margin-left",
        // text-indent（<length-percentage> && hanging? && each-line?——css-text-3
        // §text-indent）与 transform-origin-z（<length>——css-transforms-2
        // §transform-origin）：calc-mix-invalid 语料 6F 收口（number/angle/混族
        // calc-mix 此前白名单外放行 store 原文），分派臂见文末。
        "text-indent",
        "transform-origin-z",
        // <time> 族写面文法门（css-transitions-1 §transition-delay-property /
        // §transition-duration-property + css-animations-1 §animation-delay/
        // §animation-duration）——apply_advanced 四臂既存（parse_time_list/
        // parse_non_negative_time_list 逗号列表 + 负 delay 合法、duration 非负），
        // 白名单接入即启用 min/max/round 等数学形与畸形参拒收（driving:
        // minmax-time-invalid 52F——`min()`/`min(1s, )`/`min(0px)` 等写面此前
        // 无校验原文透传）。
        "transition-delay",
        "transition-duration",
        "animation-delay",
        "animation-duration",
        // random() 写面文法门槽（CSS Values 5 §random，tentative）：invalid 页
        // 39 形全在 width 槽——random 臂见 [`validate_decl_value`] 中段。
        "width",
    ];
    let prop = property.trim().to_ascii_lowercase();
    // sibling-index()/sibling-count() 写面门（CSS Values 5 §tree-counting，
    // tentative）：值含 tree-counting 函数形时整值走槽族门——calc 树（裸函数形
    // wrap 统一 parse）+ [`tree_counting_slot_valid`]（槽文法族一致），invalid →
    // 整拒；valid 放行走 [`fold_decl_specified`] canonical 串化（left/z-index
    // 分派：空白归一/静态积规范单位/单叶 calc 解包）。位于白名单检查之前——
    // left 等表外槽同样收口（槽族表外条目放行，complex 槽无语料钉面）。math
    // 形（calc 前缀/裸函数前缀）parse 失败 = 文法 invalid → 整拒（带参
    // `sibling-index(100px)`、非时槽 `calc(1s * sibling-index())`——time 叶共享
    // calc 入口拒，槽族门亦必拒）；非 math 形（`content: "sibling-index()"`
    // 字符串字面、color() 嵌套等复合上下文）放行——apply oracle 通道既有拒面
    // 兜底，不在此重复收口（FIXME: color() 实参槽型别门挂账，tree-counting
    // parsing 页 2F）。
    // https://drafts.csswg.org/css-values-5/#tree-counting
    {
        let v = value.trim();
        let lower = v.to_ascii_lowercase();
        if lower.contains("sibling-index(") || lower.contains("sibling-count(") {
            let is_time_slot = matches!(
                prop.as_str(),
                "transition-delay" | "transition-duration" | "animation-delay" | "animation-duration"
            );
            // math 形判定：calc 包裹或裸函数起始（引号字符串字面等非 math 形
            // 不收 parse-fail 拒面——content 等槽的字符串值合法含函数名文本）。
            let math_shaped = lower.starts_with("calc(")
                || lower.starts_with("sibling-index(")
                || lower.starts_with("sibling-count(");
            // 裸函数形（无 calc 包裹）wrap 成单叶 calc 树统一 parse（z-index
            // <integer> 槽语料形态；裸 sibling 叶产出 <integer> = number 型别，
            // 槽族门同样裁决）。
            let probe = if lower.starts_with("calc(") {
                v.to_string()
            } else {
                format!("calc({v})")
            };
            // Time 叶仅 allow_time 入口可达（共享 calc 入口拒 time 原子——
            // CalcExpr::Time 注）。
            let parsed = if is_time_slot {
                values::parse_time_calc(&probe)
            } else {
                values::parse_math_function(&probe)
            };
            match parsed {
                Some(expr) => {
                    if !values::tree_counting_slot_valid(&prop, &expr) {
                        return false;
                    }
                }
                // math 形 parse 失败：无参文法/槽族文法下唯一可能是 invalid
                // 形（带参/时槽 time 叶）→ 拒；非 math 形放行（oracle 兜底）。
                None if math_shaped => return false,
                None => {}
            }
        }
    }
    // random-item() 写面文法门（CSS Values 5 §funcdef-random-item，tentative）：
    // 任意替换函数 parse 期仅实参文法裁决（属性族型别在替换后承担）——
    // 位于白名单检查之前，font-family 等表外槽同样收口（random-item-invalid
    // 语料 14F 驱动：`(auto, !)`/未闭合花括号/块残片形此前原文透传 store）。
    // invalid → 整拒；valid 放行 store 原文（compute 期播种替换承担求值，
    // §9.4；与 cascade/supports 站同函数单实现——eval_random_item_call 臂）。
    // random() 家族型别门保持白名单槽位（下方既有臂——表外属性无族表）。
    // https://drafts.csswg.org/css-values-5/#funcdef-random-item
    {
        let v = value.trim();
        let lower = v.to_ascii_lowercase();
        if lower.contains("random-item(") && !random_item_calls_valid(v, &lower) {
            return false;
        }
    }
    // math 函数形通用文法+槽族型别门（CSS Values 4 §10.5 calc-type-checking，
    // 写面）：值顶层为数学函数形（22 名单，[`values::is_math_function_value`]）
    // 时整值过 parse（文法裁决——min() 空/残参/裸单参 round 等拒）+ 槽族型别
    // 表（[`math_slot_family_valid`]）。位于白名单检查之前——border-left-width/
    // height/letter-spacing/margin-top/opacity 等 minmax/clamp/round-mod-rem
    // invalid 语料 155F 槽此前无门原文透传 store；替换函数形（var()/env()/
    // attr()/random() 族/progress()/if()/calc-size()/url()）parse 树不可达，
    // 豁免交各自既有门；parse 失败时非单函数平衡跨度（双 calc 拐角——
    // [`is_single_spanning_math_value`]）与未知函数实参（inherit() 类——
    // [`has_unknown_function_token`]）同豁免交 oracle（g22 零漂移）；
    // custom property 值任意文法（CSS Parsing 3 §custom-properties）豁免。
    // 未知槽族仅文法门放行（FIXME: 逐 parser 族扩面）。
    // https://drafts.csswg.org/css-values-4/#calc-type-checking
    {
        let v = value.trim();
        if !prop.starts_with("--")
            && !v.is_empty()
            && values::is_math_function_value(v)
            && !value_has_substitution_function(v)
        {
            let is_time_slot = matches!(
                prop.as_str(),
                "transition-delay" | "transition-duration" | "animation-delay" | "animation-duration"
            );
            // Time 叶仅 allow_time 入口可达（共享 calc 入口拒 time 原子——
            // CalcExpr::Time 注）。
            let parsed = if is_time_slot {
                values::parse_time_calc(v)
            } else {
                values::parse_math_function(v)
            };
            match parsed {
                // 文法 invalid → 整拒；两豁免交 oracle 既有裁决（g22 零漂移）：
                // 非单函数平衡跨度（`calc(50%) calc(25%)` 双 calc 拐角语法——
                // [`is_single_spanning_math_value`]，border-radius-002 钉面）、
                // 未知函数实参（css-values-5 inherit() 类——
                // [`has_unknown_function_token`]）。
                None => {
                    if is_single_spanning_math_value(v) && !has_unknown_function_token(v) {
                        return false;
                    }
                }
                Some(expr) => {
                    if !math_slot_family_valid(&prop, &expr, v) {
                        return false;
                    }
                }
            }
        }
    }
    if !VALIDATED_PROPERTIES.contains(&prop.as_str()) {
        return true;
    }
    let v = value.trim();
    if v.is_empty() {
        return true;
    }
    match v.to_ascii_lowercase().as_str() {
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" => return true,
        _ => {}
    }
    // random() 文法门（CSS Values 5 §random，tentative）：值内每个 `random(`
    // 调用逐处过 css-parser 文法（identification 组合/节数门/族一致）+ 族随
    // 属性槽校验，任一 invalid → 整拒；全 valid → 放行 store 原文（apply 链
    // 未实现 random()，经 oracle 会误杀——serialize 页 71 接受形零漂移锚）。
    // invalid 页 39 形全 width 槽。
    // https://drafts.csswg.org/css-values-5/#random
    {
        let mut random_valid = true;
        let mut had_random = false;
        for args in extract_random_args(v) {
            had_random = true;
            let ok = match values::parse_random_args(&args) {
                Some(spec) => values::random_family_allowed(&prop, spec.family),
                None => false,
            };
            if !ok {
                random_valid = false;
                break;
            }
        }
        if had_random {
            return random_valid;
        }
    }
    // 值内含引擎未实现函数（tentative/未接线：progress()/calc-mix()/
    // sibling-index()/if()/var() 等）→ 放行 store 原文——apply oracle 对未实现形
    // 恒拒，会误拒引擎 store+serialize 路径本已正常的合法值（values corpus v5e
    // progress/calc-mix/random serialize 页 -12 回归根因）。文法断言只对实现面
    // 函数调用生效（引擎实现集见 [`IMPLEMENTED_VALUE_FUNCTIONS`]）。
    if value_has_unimplemented_function(v) {
        return true;
    }
    // css-animations-2 §propdef-animation-duration（`<time [0s,∞]># | auto#`，
    // csswg#6530 滚动驱动 auto 时长；项列表化——shorthand `auto, 1s` 形合法）：
    // 逐 top-level 逗号项校验，`auto` 项放行、其余项走原 apply 裁决。引擎
    // resolved-value 通道透传原文（animation-duration-computed 期望
    // getComputedStyle 恒 `auto`），apply 语义臂本批不扩面。
    if prop == "animation-duration" {
        return match super::apply_advanced::split_top_level_commas(v) {
            Some(items) => items.iter().all(|item| {
                let item = item.trim();
                item.eq_ignore_ascii_case("auto") || {
                    let mut scratch = ComputedStyle::default();
                    apply_property_value(&mut scratch, &prop, item)
                }
            }),
            // 括号/引号不闭合等病态形 → 原 apply 裁决。
            None => {
                let mut scratch = ComputedStyle::default();
                apply_property_value(&mut scratch, &prop, v)
            }
        };
    }
    // text-indent 关键字形（css-text-3 §text-indent：`<length-percentage> && hanging?
    // && each-line?`——`&&` 组合分量任意序、关键字各至多一次）。apply 链 TextIndent
    // 槽仅收 <length-percentage>，关键字形在此剥离裁决：裸关键字（长度位必在）/重复
    // 关键字拒；余下 token 重新过 apply oracle（css-text parsing 语料
    // text-indent-valid/invalid 驱动；纯 <length-percentage> 形不经此臂，走文末 oracle）。
    // https://drafts.csswg.org/css-text-3/#text-indent-property
    if prop == "text-indent"
        && v.split_whitespace()
            .any(|t| t.eq_ignore_ascii_case("hanging") || t.eq_ignore_ascii_case("each-line"))
    {
        let mut hanging = false;
        let mut each_line = false;
        let mut core = Vec::new();
        for token in v.split_whitespace() {
            match token.to_ascii_lowercase().as_str() {
                "hanging" if !hanging => hanging = true,
                "each-line" if !each_line => each_line = true,
                _ => core.push(token),
            }
        }
        if core.is_empty() {
            return false;
        }
        let mut scratch = ComputedStyle::default();
        return apply_property_value(&mut scratch, &prop, &core.join(" "));
    }
    // transform-origin-z（css-transforms-2 §transform-origin：z 分量 `<length>`——
    // 无 %，含 % 的 calc 树同拒）。registry 未收该属性（apply 链无臂）→ apply
    // oracle 不可用，parser 型别门直裁（calc-mix-invalid 语料 test_invalid_length
    // 三形驱动：number Mix / angle Mix / %·number 混族 Mix）。
    // https://drafts.csswg.org/css-transforms-2/#transform-origin-property
    if prop == "transform-origin-z" {
        return match parse_length_or_math(v) {
            Some(value) => {
                !matches!(value, LengthValue::Percentage(_))
                    && match &value {
                        LengthValue::Calc(expr) => !calc_contains_percentage(expr),
                        _ => true,
                    }
                    && text_indent_length_is_valid(v, &value)
            }
            None => false,
        };
    }
    let mut scratch = ComputedStyle::default();
    apply_property_value(&mut scratch, &prop, v)
}

/// 引擎已实现、文法可断言的函数名集：CSS Values 4 数学函数（css-parser
/// `parse_math_function` 面）+ transform 函数族。值内出现集外 `fn(` 调用即视为
/// 未实现面放行。
const IMPLEMENTED_VALUE_FUNCTIONS: [&str; 39] = [
    "calc",
    "min",
    "max",
    "clamp",
    "round",
    "mod",
    "rem",
    "hypot",
    "pow",
    "atan2",
    "abs",
    "sign",
    "sqrt",
    "exp",
    "log",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    // CSS Values 5 calc-mix()（tentative 实现面：parse/fold/eval 全链）——
    // 从 passthrough 集转入文法断言集（invalid 页空项/字面权重范围/跨型拒）。
    "calc-mix",
    "matrix",
    "matrix3d",
    "translate",
    "translate3d",
    "translatex",
    "translatey",
    "translatez",
    "scale",
    "scale3d",
    "scalex",
    "scaley",
    "scalez",
    "rotate",
    "rotate3d",
    "rotatex",
    "rotatey",
    "rotatez",
];

/// 提取值内全部 `random(` 调用的实参原文（ASCII 大小写不敏感、括号配平——
/// 嵌套 calc 形如 `random(calc(1px + 2px), 3px)` 取到最外层配对为止）。
fn extract_random_args(value: &str) -> Vec<String> {
    let lower = value.to_ascii_lowercase();
    let mut out = Vec::new();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i + 7 <= bytes.len() {
        if &lower[i..i + 7] != "random(" {
            i += 1;
            continue;
        }
        // ident 边界：前随字母/数字/-/_ 的 `random(` 属更长 ident（`foorandom(`
        // 类），非本函数调用。
        if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_') {
            i += 1;
            continue;
        }
        let start = i + 7;
        let mut depth = 1usize;
        let mut j = start;
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        if depth == 0 {
            out.push(value[start..j].trim().to_string());
            i = j + 1;
        } else {
            // 括号不闭合（病态形，random-invalid `random(1px, )` 类的极端面
            // 不在此列——闭括号缺失由整体文法门兜底拒）。
            break;
        }
    }
    out
}

/// 值内替换/未接线函数形检测（math 门豁免集）：var()/env()/attr()（替换
/// 函数——parse 树不可达，substitution 后型别在 computed 期）；random()/
/// random-item()（自有文法门，validate_decl_value 既有臂）；progress()/if()/
/// calc-size()/url()（引擎未接线，写面 oracle/通道既有兜底）。子串扫描：
/// math 形值无字符串字面量误报面（content 等槽非 math 形不达本门）。
fn value_has_substitution_function(value: &str) -> bool {
    const NEEDLES: [&str; 9] = [
        "var(",
        "env(",
        "attr(",
        "random(",
        "random-item(",
        "progress(",
        "if(",
        "calc-size(",
        "url(",
    ];
    let lower = value.to_ascii_lowercase();
    NEEDLES.iter().any(|n| lower.contains(n))
}

/// math 门豁免二：值整体为单个括号平衡的已知数学函数调用判定——
/// [`values::is_math_function_value`] 是前缀 + 尾括号的朴素跨度检查
///（不感知配平，`calc(50%) calc(25%)` 双 calc 拐角语法也判真），parse
/// 前先经本函数收紧：配平点非整值末字符即多函数/混排形（getComputedStyle-
/// border-radius-002 语料钉面），交 oracle 既有裁决。
fn is_single_spanning_math_value(value: &str) -> bool {
    // 与 css-parser `is_math_function_value` 分派名单同步（22 数学函数）。
    const NAMES: [&str; 22] = [
        "calc", "min", "max", "clamp", "round", "mod", "rem", "hypot", "pow", "atan2", "abs", "sign", "sqrt", "exp",
        "log", "sin", "cos", "tan", "asin", "acos", "atan", "calc-mix",
    ];
    let v = value.trim();
    let lower = v.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for name in NAMES {
        if !lower.starts_with(name) || bytes.get(name.len()) != Some(&b'(') {
            continue;
        }
        let mut depth = 0usize;
        for (idx, &b) in bytes.iter().enumerate().skip(name.len()) {
            if b == b'(' {
                depth += 1;
            } else if b == b')' {
                depth -= 1;
                if depth == 0 {
                    // 配平点必须为整值末字符（尾随空白容许）。
                    return v[idx + 1..].trim().is_empty();
                }
            }
        }
    }
    false
}

/// math 门豁免三：值内未知函数 token（[`IMPLEMENTED_VALUE_FUNCTIONS`] 集外
/// `fn(` 调用——css-values-5 inherit() 类 parse 树不可达面）交 oracle 既有
/// 裁决（g22 零漂移）。ident 边界同 [`extract_random_args`]（`-`/alnum/`_`
/// 前随者属更长 ident）。
fn has_unknown_function_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_alphabetic() && bytes[i] != b'-' && bytes[i] != b'_' {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            let ident = &lower[start..i];
            if !IMPLEMENTED_VALUE_FUNCTIONS.contains(&ident) {
                return true;
            }
        }
    }
    false
}

/// number 槽文本级角度单位 token 扫描（§10.5 number 槽 angle-provenance
/// 补面）：角度叶 parse 期弧度 Number 化（CalcParser 注——`0deg` 产出纯
/// number 树），型别表失去角度 provenance，文本扫描补 `min(0deg)`/`round(1,
/// 0deg)` 类 number 槽拒绝面（minmax-number-invalid 26 型别行）。简化子串
/// 扫描：math 形值无字符串字面量，单位串即命中。
fn has_angle_unit_token(value: &str) -> bool {
    const UNITS: [&str; 4] = ["deg", "grad", "rad", "turn"];
    UNITS.iter().any(|u| value.contains(u))
}

/// 剥离三角函数调用实参跨度（sin/cos/tan/asin/acos/atan/atan2 的括号平衡
/// 段）——角度是其合法定义域（§trig-funcs：`sin(30deg)` 返回 <number>），
/// [`has_angle_unit_token`] 扫描前先剥，避免误伤三角实参角
/// （sin-cos-tan-serialize 语料 270P 面）；calc 族裸角度叶不受扰
/// （`round(1, 0deg)` 仍拒）。非 ASCII 字节按原样保留（UTF-8 安全）。
fn strip_trig_argument_spans(value: &str) -> String {
    const TRIG: [&str; 7] = ["sin", "cos", "tan", "asin", "acos", "atan", "atan2"];
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(value.len());
    let mut i = 0;
    while i < bytes.len() {
        let mut span_len = None;
        for name in TRIG {
            let ident_start =
                i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_');
            if lower[i..].starts_with(name) && ident_start && bytes.get(i + name.len()) == Some(&b'(') {
                span_len = Some(name.len());
                break;
            }
        }
        if let Some(n) = span_len {
            let mut depth = 0usize;
            let mut j = i + n;
            while j < bytes.len() {
                if bytes[j] == b'(' {
                    depth += 1;
                } else if bytes[j] == b')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            i = j + 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// math 形槽族型别门（CSS Values 4 §10.5 型别表 × 属性文法族）：calc 树结果
/// 文法族与属性槽一致时放行，不一致拒。槽族三档：
///
/// - **LengthOnly**（% 不可）：border 宽度族/outline-width/column-rule-width
///   （<line-width> = <length [0,∞]>）+ transform-origin-z（<length>，既有臂
///   同口径——calc-mix-invalid 语料收口面保零漂移）；
/// - **LengthPercentage**（% 可）：尺寸/边距/缩进/字距/left；
/// - **Number**（<number>）：opacity/scale——角度单位文本扫描补 provenance
///   （[`has_angle_unit_token`]）；rotate（<angle> 槽）容 number（角度产出
///   弧度 Number 化与 bare-number-as-deg 同口径）。
///
/// 表外槽放行（仅文法门已过——FIXME: 逐 parser 族扩面）。Invalid 型别二分：
/// 无除法拒（混族和树——`min(1px, 0s)` 类，无代数抵消路径），含除法放行交
/// fold/computed 链（除法单位幂抵消——getComputedStyle-calc-mixed-units-003
/// 钉面）；Time 仅 time 槽可达（共享 calc 入口拒 time 原子），落未知臂文法
/// 门已过即放行。
/// https://drafts.csswg.org/css-values-4/#calc-type-checking
fn math_slot_family_valid(prop: &str, expr: &values::CalcExpr, raw: &str) -> bool {
    use values::CalcMathType::{self, Angle, Invalid, LengthPercentage, Number};
    let mt = values::calc_expr_math_type(expr);
    // Invalid 二分（getComputedStyle-calc-mixed-units-003 语料钉面）：线性
    // 型别在内层乘积节点判 Invalid（`calc(5px * 10lh / 1px)` 的 px*lh），
    // 但除法单位代数幂可抵消（px·lh/px → lh，Chrome 同口径计算值 1000px）
    // ——含除法放行交 fold/computed 链裁决（后续槽族臂按 mt 匹配，Invalid
    // 不落臂）；无除法的混族（`min(1px, 0s)`）无抵消路径，保持拒绝。
    if mt == Invalid {
        return raw.contains('/');
    }
    // 按槽族分派（树型 × 槽文法族一致才放行）；Time 树仅 time 槽可达
    //（parse_time_calc 入口）——落未知臂文法门已过即放行。
    match prop {
        // <line-width>（<length [0,∞]>，% 不可）+ transform-origin-z（<length>，
        // 既有臂同口径——calc-mix-invalid 语料收口面保零漂移）。
        "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width"
        | "border-block-start-width"
        | "border-block-end-width"
        | "border-inline-start-width"
        | "border-inline-end-width"
        | "outline-width"
        | "column-rule-width"
        | "transform-origin-z" => mt == CalcMathType::LengthPercentage && !calc_contains_percentage(expr),
        // <length-percentage> 槽（% 可）：尺寸/边距/缩进/字距/left。
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" | "margin-top" | "margin-left"
        | "text-indent" | "letter-spacing" | "left" => mt == LengthPercentage,
        // <number>|<percentage> 槽（opacity/scale，css-transforms-2 §individual-
        // transform-properties）：number 树角度单位 token 扫描补 provenance
        //（角度叶 parse 期 Number 化——[`has_angle_unit_token`]）；% 树须真含
        // % 叶（LengthPercentage 族塌缩 px 长度——`min(0px)` 拒，minmax-number-
        // invalid 现存通过行）。
        "opacity" | "scale" => {
            (mt == Number && !has_angle_unit_token(&strip_trig_argument_spans(raw)))
                || (mt == LengthPercentage && calc_contains_percentage(expr))
        }
        // <angle> 槽容 number（角度产出弧度 Number 化与 bare-number-as-deg
        // 同口径）。
        "rotate" => matches!(mt, Angle | Number),
        // 未知槽族：文法门已过，放行（FIXME: 逐 parser 族扩面）。
        _ => true,
    }
}

/// 值内全部 `random-item(` 调用实参文法裁决（CSS Values 5
/// §funcdef-random-item 写面门）。调用未闭合（`random-item(auto,` 括号
/// 缺失）→ invalid——与 random() 提取器 break 放行不同，本门 spec 钉面
/// （random-item-invalid 语料）。字符串字面内 `random-item(` 文本由引号
/// 状态机跳过（`content: "random-item()"` 不触发）。
fn random_item_calls_valid(value: &str, lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let needle = b"random-item(";
    let mut i = 0;
    let mut in_string: Option<u8> = None;
    let mut escaped = false;
    while i < bytes.len() {
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        let b = bytes[i];
        if let Some(q) = in_string {
            if b == b'\\' {
                escaped = true;
            } else if b == q {
                in_string = None;
            }
            i += 1;
            continue;
        }
        if b == b'"' || b == b'\'' {
            in_string = Some(b);
            i += 1;
            continue;
        }
        if i + needle.len() > bytes.len() || bytes[i..i + needle.len()] != needle[..] {
            i += 1;
            continue;
        }
        // ident 边界：前随字母/数字/-/_ 的命中属更长 ident（`foorandom-item(`）。
        if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_') {
            i += 1;
            continue;
        }
        let start = i + needle.len();
        let mut depth = 1usize;
        let mut j = start;
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            // 括号不闭合 → invalid（语料 `random-item(auto,` 形）。
            return false;
        }
        if !values::random_item_args_valid(&value[start..j]) {
            return false;
        }
        i = j + 1;
    }
    true
}

/// 值内是否存在集外函数调用（粗扫 `ident(` 形态——字符串面 CSS 值无此歧义；
/// URL/字符串字面量内形态属未实现面，放行方向正确）。
fn value_has_unimplemented_function(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // ident 起始（字母/-/_）
        if !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
        }
        // ident 后随 `(`（允许空白）→ 函数调用
        let mut j = i;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'(' && !IMPLEMENTED_VALUE_FUNCTIONS.contains(&&lower[start..i]) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod t54_validate_decl_value_tests {
    use super::*;

    #[test]
    fn whitelist_properties_reject_invalid_math_forms() {
        // WPT invalid 面（acos-asin-atan-atan2-invalid.html）：无参/坏单位/悬空
        // 运算符形整体拒绝。
        assert!(!validate_decl_value("transform", "rotate(asin())"));
        assert!(!validate_decl_value("transform", "rotate(asin(1dag))"));
        assert!(!validate_decl_value("transform", "rotate(asin(1deg + )"));
        assert!(!validate_decl_value("rotate", "asin()"));
        assert!(!validate_decl_value("z-index", "round(1dag)"));
    }

    #[test]
    fn whitelist_properties_accept_valid_math_and_plain_forms() {
        assert!(validate_decl_value("transform", "rotate(45deg)"));
        assert!(validate_decl_value("rotate", "acos(1)"));
        assert!(validate_decl_value("scale", "2"));
        // g12：±∞ 数学形放行（§10.6，calc-infinity-nan-computed scale ≥1e6 驱动）。
        assert!(validate_decl_value("scale", "calc(infinity)"));
        assert!(validate_decl_value("scale", "calc(max(infinity, 10))"));
        assert!(validate_decl_value("scale", "calc(1 / sign(1em - 16px))"));
        assert!(validate_decl_value("translate", "10px 20px"));
        assert!(validate_decl_value("z-index", "sign(-3)"));
        assert!(validate_decl_value("margin-left", "round(23px, 10px)"));
        // r2320：transform 列表 rotate 臂 math 接线（WPT acos serialize 页
        // 曾因该臂无 math 落 "disallowed value"）；NaN 域外合法（§10.10），
        // asin/acos/atan 角度叶仍拒（invalid 面）。
        assert!(validate_decl_value("transform", "rotate(acos(1))"));
        assert!(validate_decl_value("transform", "rotate(acos(2))"));
        assert!(!validate_decl_value("transform", "rotate(atan(45deg))"));
        assert!(!validate_decl_value("transform", "rotate(asin(90px))"));
    }

    #[test]
    fn tentative_unimplemented_functions_pass_through() {
        // v5e -12 回归钉：progress()/random()/sibling-index() 属未实现面
        // ——store 原文（apply oracle 恒拒会误拒引擎 store+serialize 正常的合法值）。
        // WPT progress-serialize / random-serialize 以 `scale(progress(...))` 形态
        // 驱动 transform 槽。calc-mix 已实现化（见
        // [`Self::calc_mix_implemented_grammar_gates`]），不在本钉面。
        assert!(validate_decl_value("transform", "scale(progress(100px, 0px, 100px))"));
        assert!(validate_decl_value("transform", "scale(random(1px, 5px))"));
        assert!(validate_decl_value("transform", "scale(calc(sibling-index() * 1px))"));
    }

    #[test]
    fn calc_mix_implemented_grammar_gates() {
        // calc-mix 实现化（CSS Values 5 tentative，invalid/serialize 页驱动）：
        // 从 passthrough 集转入文法断言集——合法形放行、跨族/越界权重拒。
        assert!(validate_decl_value("scale", "calc-mix(1)"));
        assert!(validate_decl_value("scale", "calc-mix(1 50%, 3 50%)"));
        assert!(validate_decl_value("transform", "scale(calc-mix(1 50%, 3 50%))"));
        // number 位（scale）拒 length/angle 族与文法拒形（invalid 页驱动）。
        assert!(!validate_decl_value("scale", "calc-mix(1px 50%, 3px 50%)"));
        assert!(!validate_decl_value("scale", "calc-mix(1deg 50%, 3turn 50%)"));
        assert!(!validate_decl_value("scale", "calc-mix(1 150%, 3)"));
        assert!(!validate_decl_value("scale", "calc-mix(1 50, 3)"));
        assert!(!validate_decl_value("scale", "calc-mix(1% 50%, 3 50%)"));
        assert!(!validate_decl_value("scale", "calc-mix()"));
        // text-indent（length-percentage 位）：length 族放行、number/angle 族拒
        //（apply 链 text_indent_length_is_valid §10.5 型别门）。
        assert!(apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc-mix(10px 50%, 3em 50%)"
        ));
        assert!(!apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc-mix(1 50%, 3 50%)"
        ));
        assert!(!apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc-mix(1deg 50%, 3turn 50%)"
        ));
        assert!(!apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc-mix(1% 50%, 3 50%)"
        ));
        // 既有 calc 形零漂移：length-percentage 树放行、纯 number/angle 树拒
        //（`calc(5)` 此前经 `Calc(_) => true` 误放行，本刀收口）。
        assert!(apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc(20% + 10px)"
        ));
        assert!(!apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc(5)"
        ));
        assert!(!apply_property_value(
            &mut ComputedStyle::default(),
            "text-indent",
            "calc(45deg)"
        ));
    }

    #[test]
    fn calc_mix_invalid_length_slots_write_face_gates() {
        // calc-mix-invalid 语料 6F 收口（2026-11-12 g14b 勘根）：text-indent /
        // transform-origin-z 槽 number/angle/混族 calc-mix 此前白名单外放行
        // store 原文——写面文法门接入后整拒（runner shim
        // __zw_validate_decl_value 与 native bindings 同源裁决）。
        // text-indent：<length-percentage> 位（§10.5 型别一致门）。
        for v in [
            "calc-mix(1 50%, 3 50%)",
            "calc-mix(1deg 50%, 3turn 50%)",
            "calc-mix(1% 50%, 3 50%)",
        ] {
            assert!(!validate_decl_value("text-indent", v), "{v} 应拒");
        }
        // transform-origin-z：<length> 位（% 与含 % calc 拒——registry 未收 →
        // parser 型别门直裁）。
        for v in [
            "calc-mix(1 50%, 3 50%)",
            "calc-mix(1deg 50%, 3turn 50%)",
            "calc-mix(1% 50%, 3 50%)",
            "50%",
            "calc(50% + 10px)",
            "10",
            "auto",
            "10deg",
        ] {
            assert!(!validate_decl_value("transform-origin-z", v), "{v} 应拒");
        }
        // 合法形零漂移：length / length 族 calc-mix / length 族 calc 放行。
        assert!(validate_decl_value("transform-origin-z", "10px"));
        assert!(validate_decl_value("transform-origin-z", "calc(10px + 2em)"));
        assert!(validate_decl_value(
            "transform-origin-z",
            "calc-mix(10px 50%, 30px 50%)"
        ));
    }

    #[test]
    fn text_indent_keyword_forms_write_face() {
        // css-text-3 §text-indent `&&` 组合文法（css-text parsing 语料
        // text-indent-valid/invalid 驱动）：关键字任意序各至多一次、长度位必在。
        // 合法关键字形放行（store 原文——readback 重排序归一面不在本门）。
        for v in [
            "10px hanging",
            "20% each-line",
            "calc(2em + 3ex) hanging each-line",
            "hanging calc(50% + 60px)",
            "each-line 10px",
            "hanging 20% each-line",
        ] {
            assert!(validate_decl_value("text-indent", v), "{v} 应放行");
        }
        // invalid 页全形：裸关键字/重复关键字/双长度/unitless number/auto。
        for v in [
            "hanging",
            "each-line",
            "auto",
            "10",
            "10px hanging 20px",
            "hanging 20% hanging",
            "each-line each-line",
        ] {
            assert!(!validate_decl_value("text-indent", v), "{v} 应拒");
        }
        // 纯 length-percentage 形不经关键字臂、仍走 apply oracle（零漂移钉）。
        assert!(validate_decl_value("text-indent", "calc(50% + 60px)"));
        assert!(validate_decl_value("text-indent", "-30px"));
    }

    #[test]
    fn passthrough_faces_return_true() {
        // 白名单外属性放行（parser 假阴性兼容面零回退）。width 已入白名单
        //（g16 random() 文法门槽，见 [`Self::random_grammar_gate`]），钉例换
        // 同槽位未入表的 height。
        assert!(validate_decl_value("height", "not-a-width-grammar"));
        assert!(validate_decl_value("unknown-prop", "anything"));
        // shorthand 放行（apply 链无展开臂）。
        assert!(validate_decl_value("margin", "10px 20px"));
        // css-wide keyword 与空值恒真。
        for kw in ["inherit", "INHERIT", "initial", "unset", "revert", "revert-layer"] {
            assert!(validate_decl_value("transform", kw), "{kw} 放行");
        }
        assert!(validate_decl_value("transform", ""));
        assert!(validate_decl_value("transform", "   "));
    }

    #[test]
    fn time_family_rejects_malformed_math_and_wrong_units() {
        // WPT minmax-time-invalid.html 语法检查桶（52F 页全形代表集）：
        // 空参/悬空运算符/缺逗号/双逗号/坏单位。
        for v in [
            "min()",
            "min( )",
            "min(,)",
            "min(1mt)",
            "min(1s, )",
            "min(, 1s)",
            "min(1s + )",
            "min(1s - )",
            "min(1s * )",
            "min(1s / )",
            "min(1s 2s)",
            "min(1s, , 2s)",
            "max()",
            "max(1dag)",
            "max(1s, , 2s)",
        ] {
            assert!(!validate_decl_value("transition-delay", v), "{v} 应拒");
        }
        // 类型检查桶：unitless 0 与非 time 单位（含 0px/0%/0deg/0Hz/0dpi/0fr）
        // 混入 time 槽 spec invalid（css-values-4 §10.1 型检查）。
        for v in [
            "min(0)",
            "min(0%)",
            "min(0px)",
            "min(0deg)",
            "min(0Hz)",
            "min(0dpi)",
            "min(0fr)",
            "min(1s, 0)",
            "min(1s, 0px)",
            "max(0)",
            "max(1s, 0deg)",
        ] {
            assert!(!validate_decl_value("transition-delay", v), "{v} 应拒");
        }
    }

    #[test]
    fn time_family_accepts_plain_math_and_list_forms() {
        // 合法形（同页 test_valid 面 + transitions 语料写形）：
        assert!(validate_decl_value("transition-delay", "1s"));
        assert!(validate_decl_value("transition-delay", "200ms"));
        // 负 delay 合法（css-transitions-1 §transition-delay：<time> 可负）。
        assert!(validate_decl_value("transition-delay", "-0.5s"));
        // g10 家族：min/max/round 数学形（MTTime 型别推进）。
        assert!(validate_decl_value("transition-delay", "min(1s, 2s)"));
        assert!(validate_decl_value(
            "transition-delay",
            "max(1000ms, min(2000ms, 1.5s))"
        ));
        assert!(validate_decl_value("transition-delay", "round(10s, 6s)"));
        assert!(validate_decl_value("transition-delay", "calc(10s + 500ms)"));
        // `<time>#` 逗号列表（transitions 语料 events-004 写形）。
        assert!(validate_decl_value("transition-delay", "0.01s, 0.02s"));
        assert!(validate_decl_value("transition-delay", "min(1s, 2s), 3s"));
        // duration 非负 + animation 族同面。
        assert!(validate_decl_value("transition-duration", "1s"));
        assert!(validate_decl_value("transition-duration", "1s, 2s"));
        assert!(!validate_decl_value("transition-duration", "-1s"));
        assert!(!validate_decl_value("transition-duration", "1dag"));
        assert!(validate_decl_value("animation-delay", "min(1s, 2s)"));
        assert!(validate_decl_value("animation-duration", "0s, 0.5s"));
        assert!(!validate_decl_value("animation-duration", "min(0px)"));
        // css-animations-2 §propdef-animation-duration `auto` 关键字（csswg#6530
        // 滚动驱动；项列表化，shorthand `auto, 1s` 混合形合法）；大小写不敏感。
        assert!(validate_decl_value("animation-duration", "auto"));
        assert!(validate_decl_value("animation-duration", "auto, 1s"));
        assert!(validate_decl_value("animation-duration", "AUTO"));
        // delay 与 transition-duration 无 auto 关键字（css-animations-2 §propdef-
        // animation-delay 仅 <time>#；css-transitions-1 无 auto）。
        assert!(!validate_decl_value("animation-delay", "auto"));
        assert!(!validate_decl_value("transition-duration", "auto"));
    }

    #[test]
    fn time_family_unimplemented_and_var_escape() {
        // var()/未实现函数放行（与 transform 槽同一逃逸门）。
        assert!(validate_decl_value("transition-delay", "var(--d)"));
        assert!(validate_decl_value("transition-delay", "min(calc(1s + var(--x)), 2s)"));
        assert!(validate_decl_value("transition-duration", "random(1s, 2s)"));
    }

    #[test]
    fn random_grammar_gate() {
        // CSS Values 5 §random 文法门（random-invalid 39 形 / random-serialize
        // 71 接受形，全走 width 槽——invalid 页唯一属性）。valid 形放行 store
        // 原文（serialize 页零漂移锚），invalid 形整拒。
        // serialize 页接受形（width 槽）。
        for v in [
            "random(0px, 100px)",
            "random(fixed 0.5, 0px, 100px)",
            "random(auto, 0px, 100px)",
            "random(--foo, 0px, 100px)",
            "random(--foo element-scoped, 0px, 100px)",
            "random(element-scoped, 0px, 100px)",
            "random(property-index-scoped, 0px, 100px)",
            "random(element-scoped property-scoped, 0px, 100px)",
            "random(property-index-scoped element-scoped , 0px, 100px)",
            "random(--foo element-scoped property-index-scoped, 0px, 100px)",
            "random(0px, 100px, 3px)",
            "random(--foo, 10px, 20%)",
            "random(--foo, 100px, 0px)",
        ] {
            assert!(validate_decl_value("width", v), "{v} 应放行");
        }
        // invalid 页拒形（结构残缺/identification 错组/双 UA 域）。
        for v in [
            "random()",
            "random( )",
            "random(,)",
            "random(1px, )",
            "random(,1px)",
            "random(1px)",
            "random(1px,2px,)",
            "random(1px,2px,1px foo)",
            "random(foo, 1px, 2px)",
            "random(\"--foo\", 1px, 2px)",
            "random(--foo --bar, 1px, 2px)",
            "random(fixed -1, 1px, 2px)",
            "random(fixed 0.5px, 1px, 2px)",
            "random(element-scoped uawidth, 1px, 2px)",
            "random(property-scoped ua-width-1, 1px, 2px)",
            "random(property-index-scoped --foo property-scoped, 1px, 2px)",
        ] {
            assert!(!validate_decl_value("width", v), "{v} 应拒");
        }
        // 族随属性槽：width（length 槽）拒 angle 族（invalid 页 `10deg, 20deg` 行
        // ——文法 valid，槽位拒）；嵌套形放行（transform 复合槽无语料钉面）。
        assert!(!validate_decl_value("width", "random(10deg, 20deg)"));
        assert!(validate_decl_value("transform", "scale(random(1px, 5px))"));
        // 非 random width 值零漂移（oracle 通道既序）。
        assert!(validate_decl_value("width", "10px"));
        assert!(validate_decl_value("width", "auto"));
        assert!(validate_decl_value("width", "calc(50% - 10px)"));
        assert!(validate_decl_value("width", "min-content"));
    }

    #[test]
    fn sibling_grammar_gate() {
        // CSS Values 5 §tree-counting 写面门（calc-sibling-function-parsing 语料
        // 13 invalid / 23 valid 形逐钉）。valid 形放行（fold_decl_specified
        // left/z-index 分派 canonical 串化）；invalid 形整拒。
        // valid 面（槽族一致）。
        for (prop, v) in [
            ("left", "calc(1px * sibling-index())"),
            ("left", "calc(1em * sibling-index())"),
            ("left", "calc(1rem * sibling-count())"),
            ("left", "calc(1vh * sibling-index())"),
            ("left", "calc(1ch * sibling-count())"),
            ("left", "calc(1% * sibling-index())"),
            ("left", "calc(5% * sibling-count())"),
            ("rotate", "calc(10deg * sibling-index())"),
            ("rotate", "calc(sibling-index() * 2rad * pi)"),
            ("rotate", "calc(1turn * sibling-count())"),
            ("opacity", "calc(0.1 * sibling-index())"),
            ("scale", "calc(2 * sibling-count())"),
            ("z-index", "sibling-index()"),
            ("z-index", "sibling-count()"),
            ("z-index", "calc(2 * sibling-index())"),
            ("animation-duration", "calc(1s * sibling-index())"),
            ("animation-duration", "calc(100ms * sibling-count())"),
        ] {
            assert!(validate_decl_value(prop, v), "{prop}: {v} 应放行");
        }
        // invalid 面（槽族错配，parse 可达形）。
        for (prop, v) in [
            ("left", "calc(10 * sibling-index())"),
            ("rotate", "calc(1px * sibling-index())"),
            ("opacity", "calc(1px * sibling-count())"),
            ("animation-duration", "calc(1px * sibling-index())"),
            ("animation-duration", "calc(1deg * sibling-index())"),
            ("left", "sibling-index()"),
        ] {
            assert!(!validate_decl_value(prop, v), "{prop}: {v} 应拒");
        }
        // Time 叶被共享 calc 入口拒（math 形 parse 失败 → 门整拒——非时槽 time
        // 叶本就槽族 invalid；rotate/opacity 槽 parse 亦拒 time 原子兜底）。
        for (prop, v) in [
            ("rotate", "calc(1s * sibling-index())"),
            ("opacity", "calc(1ms * sibling-index())"),
        ] {
            assert!(!validate_decl_value(prop, v), "{prop}: {v} 应拒");
        }
        // 带参形（无参文法 invalid，math 形 parse 失败）→ 拒（parsing 页语料形）。
        assert!(!validate_decl_value("left", "calc(1px * sibling-index(100px))"));
        assert!(!validate_decl_value("left", "calc(1px * sibling-index(1))"));
        assert!(!validate_decl_value("z-index", "sibling-index(100px)"));
        assert!(!validate_decl_value("z-index", "sibling-count(1)"));
        // color() 实参槽嵌套形放行挂账（FIXME: color 通道型别门）。
        assert!(validate_decl_value(
            "color",
            "color(srgb calc(0.1 * sibling-index()) 0.5 0.5)"
        ));
        // 非 math 形（引号字符串字面含函数名文本）放行——parse-fail 拒面只收
        // calc/裸函数前缀（content 字符串值合法含任意文本）。
        assert!(validate_decl_value("content", "\"sibling-index()\""));
        // 非 sibling 值零漂移。
        assert!(validate_decl_value("left", "10px"));
        assert!(validate_decl_value("z-index", "5"));
        assert!(validate_decl_value("z-index", "auto"));
    }

    #[test]
    fn sibling_apply_eval_with_truth() {
        // CSS Values 5 §tree-counting 真值求值：tree-counting 页断言形态
        //（#test 序 2 / 计 5）。
        let mut style = ComputedStyle::default();
        assert!(apply_property_value_with_quirks(
            &mut style,
            "z-index",
            "calc(sibling-index())",
            false,
            false,
            Some((2, 5))
        ));
        assert_eq!(style.z_index, ZIndexValue::Integer(2));
        // 哨兵 (1,1)（探测口径）：合法性序数无关。
        let mut style = ComputedStyle::default();
        assert!(apply_property_value_with_quirks(
            &mut style,
            "z-index",
            "calc(2 * sibling-index())",
            false,
            false,
            Some((1, 1))
        ));
        assert_eq!(style.z_index, ZIndexValue::Integer(2));
        // None fail-closed（无 DOM 视图）：声明拒。
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value_with_quirks(
            &mut style,
            "z-index",
            "calc(sibling-index())",
            false,
            false,
            None
        ));
        // order math 臂（新接线）：真值 + clamp。
        let mut style = ComputedStyle::default();
        assert!(apply_property_value_with_quirks(
            &mut style,
            "order",
            "calc(sibling-count() * 2)",
            false,
            false,
            Some((2, 5))
        ));
        assert_eq!(style.order, 10);
        // 裸字面 order 零漂移。
        let mut style = ComputedStyle::default();
        assert!(apply_property_value_with_quirks(
            &mut style,
            "order",
            "-3",
            false,
            false,
            Some((2, 5))
        ));
        assert_eq!(style.order, -3);
    }

    #[test]
    fn math_grammar_slot_family_gate() {
        // CSS Values 4 §10.5 math 形通用文法+槽族型别门（g23：minmax/clamp/
        // round-mod-rem invalid 六页 155F 槽收口——border-left-width 48 形 /
        // opacity 42 形 / margin-left 10 形 / height 32 形 / letter-spacing
        // 23 形恰集；语料只覆盖这些槽，测试按恰集全量断言）。
        // border-left-width（<line-width> = <length [0,∞]>，% 不可）：语法 24
        // + 型别 24（number/%/time/frequency/resolution/flex 叶混入即无效）。
        for v in [
            "min()",
            "min( )",
            "min(,)",
            "min(1py)",
            "min(1px, )",
            "min(, 1px)",
            "min(1px + )",
            "min(1px - )",
            "min(1px * )",
            "min(1px / )",
            "min(1px 2px)",
            "min(1px, , 2px)",
            "max()",
            "max( )",
            "max(,)",
            "max(1py)",
            "max(1px, )",
            "max(, 1px)",
            "max(1px + )",
            "max(1px - )",
            "max(1px * )",
            "max(1px / )",
            "max(1px 2px)",
            "max(1px, , 2px)",
            "min(0)",
            "min(0%)",
            "min(0s)",
            "min(0Hz)",
            "min(0dpi)",
            "min(0fr)",
            "min(1px, 0)",
            "min(1px, 0%)",
            "min(1px, 0s)",
            "min(1px, 0Hz)",
            "min(1px, 0dpi)",
            "min(1px, 0fr)",
            "max(0)",
            "max(0%)",
            "max(0s)",
            "max(0Hz)",
            "max(0dpi)",
            "max(0fr)",
            "max(1px, 0)",
            "max(1px, 0%)",
            "max(1px, 0s)",
            "max(1px, 0Hz)",
            "max(1px, 0dpi)",
            "max(1px, 0fr)",
        ] {
            assert!(!validate_decl_value("border-left-width", v), "{v} 应拒");
        }
        // opacity（<number> 槽）：语法 22 + 型别 20（number 槽不容 %/px/s/
        // deg——deg 叶 parse 期 Number 化，文本 token 扫描补拒绝面）。
        for v in [
            "min()",
            "min( )",
            "min(,)",
            "min(1, )",
            "min(, 1)",
            "min(1 + )",
            "min(1 - )",
            "min(1 * )",
            "min(1 / )",
            "min(1 2)",
            "min(1, , 2)",
            "max()",
            "max( )",
            "max(,)",
            "max(1, )",
            "max(, 1)",
            "max(1 + )",
            "max(1 - )",
            "max(1 * )",
            "max(1 / )",
            "max(1 2)",
            "max(1, , 2)",
            "min(0Hz)",
            "min(0dpi)",
            "min(0fr)",
            "min(1, 1%)",
            "min(1, 0px)",
            "min(1, 0s)",
            "min(1, 0deg)",
            "min(1, 0Hz)",
            "min(1, 0dpi)",
            "min(1, 0fr)",
            "max(0Hz)",
            "max(0dpi)",
            "max(0fr)",
            "max(1, 1%)",
            "max(1, 0px)",
            "max(1, 0s)",
            "max(1, 0deg)",
            "max(1, 0Hz)",
            "max(1, 0dpi)",
            "max(1, 0fr)",
            "min(0deg)",
        ] {
            assert!(!validate_decl_value("opacity", v), "{v} 应拒");
        }
        // margin-left（<length-percentage> 槽，% 可、混族不可）：percentage 页
        // 6 形 + length-percent 页 4 形（min(1%, 0)/max(1%, 0) 双页重复去重）。
        for v in [
            "min(0deg)",
            "min(1%, 0)",
            "min(1%, 0deg)",
            "max(0deg)",
            "max(1%, 0)",
            "max(1%, 0deg)",
            "min(1px, 0)",
            "max(1px, 0)",
        ] {
            assert!(!validate_decl_value("margin-left", v), "{v} 应拒");
        }
        // height（<length-percentage> 槽）：round/mod/rem 型别行 bare+calc ×2
        //（round-mod-rem-invalid 32F 恰集——0px 首参/混族次参，×2 包裹展开）。
        for core in [
            "round(0px)",
            "round(0px, 0s)",
            "round(0px, 0deg)",
            "round(0px, 0Hz)",
            "round(0px, 0dpi)",
            "round(0px, 0fr)",
            "mod(0px, 0s)",
            "mod(0px, 0deg)",
            "mod(0px, 0Hz)",
            "mod(0px, 0dpi)",
            "mod(0px, 0fr)",
            "rem(0px, 0s)",
            "rem(0px, 0deg)",
            "rem(0px, 0Hz)",
            "rem(0px, 0dpi)",
            "rem(0px, 0fr)",
        ] {
            assert!(!validate_decl_value("height", core), "{core} 应拒");
            assert!(
                !validate_decl_value("height", &format!("calc({core})")),
                "calc({core}) 应拒"
            );
        }
        // letter-spacing（<length-percentage> 槽）：语法 12 + none 实参滥用 11
        //（clamp-length-invalid 23F 恰集）。
        for v in [
            "clamp()",
            "clamp( )",
            "clamp(,)",
            "clamp(1px, )",
            "clamp(, 1px)",
            "clamp(1px, 1px)",
            "clamp(1px, , 1px)",
            "clamp(, 1px, 1px)",
            "clamp(1px, 1px, )",
            "clamp(1px, 1px, 1px, )",
            "clamp(1px 1px 1px)",
            "clamp(0, 10rem, 100%)",
            "clamp(none, none, none)",
            "clamp(none, none + none, none)",
            "clamp(none, none + 1, none)",
            "clamp(-none, 1px, 1px)",
            "clamp(10px, none * none, 20px)",
            "clamp(10px, 2px * none, 20px)",
            "clamp(10px, 2px + none, 20px)",
            "clamp(10px + none, 10px / 2px + none, 20px / none)",
            "clamp(10px, -none, 20px)",
            "clamp(10px, 15px, abs(none) * 1px)",
            "clamp(10px, 15px, abs(none * 1px))",
        ] {
            assert!(!validate_decl_value("letter-spacing", v), "{v} 应拒");
        }
        // valid 形放行（新接线槽 + 既有槽零漂移）。
        for (prop, v) in [
            ("border-left-width", "min(10px, 20px)"),
            ("margin-left", "min(1%, 2%)"),
            ("margin-left", "min(10% + 30px, 5em + 5%)"),
            ("margin-top", "clamp(none , 30px, 33px)"),
            ("margin-top", "min(10px, 33px)"),
            ("width", "min(1em, 10%)"),
            // 除法单位代数幂抵消形（getComputedStyle-calc-mixed-units-003 四
            // P 钉面）——线性型别内层 Invalid 但含除法放行交 fold/computed 链。
            ("width", "calc(5px * 10lh / 1px)"),
            ("width", "calc(4px * 4em / 1px)"),
            ("width", "calc(400px / 4lh * 1px)"),
            ("width", "calc(100px * 1px / 1px / 1)"),
            // 双 calc 拐角语法（getComputedStyle-border-radius-002 五~八钉面）
            // 与未知函数实参（css-values-5 inherit()，inherit-function-parsing
            // 钉面）——parse-None 双豁免交 oracle 既有裁决（g22 零漂移）。
            ("border-top-left-radius", "calc(50%) calc(25%)"),
            ("left", "calc(inherit(--x) + 1px)"),
            ("text-indent", "calc(20% + 10px)"),
            ("opacity", "round(1.1,1)"),
            ("opacity", "min(1, 2)"),
            ("opacity", "calc(25% * 2)"),
            ("opacity", "min(50%, 60%)"),
            ("letter-spacing", "clamp(10px, 15px, 20px)"),
            ("letter-spacing", "calc(100%)"),
            ("height", "clamp(0px, 30px, 100px)"),
            ("height", "round(5px, 2px)"),
            ("rotate", "min(0deg)"),
            ("transform-origin-z", "calc(1px + 2px)"),
            ("transition-duration", "min(1s, 2s)"),
            ("width", "10px"),
            ("opacity", "0.5"),
            ("margin-left", "10%"),
        ] {
            assert!(validate_decl_value(prop, v), "{prop}: {v} 应放行");
        }
        // 槽族门收口面：LengthOnly 槽拒 %（transform-origin-z 既有臂同口径——
        // calc-mix-invalid 收口面保零漂移）；number 槽角度 token 扫描（scale）。
        assert!(!validate_decl_value("transform-origin-z", "calc(1px + 1%)"));
        // Invalid 无除法二分：无抵消路径的长度×长度积拒（003 反面——除法缺席
        // 即线性型别终判）；三角实参角是合法定义域（§trig-funcs，剥跨度后
        // token 扫描不误伤），三角长度实参仍拒。
        assert!(!validate_decl_value("width", "calc(1px * 1em)"));
        // parse 失败双豁免（oracle 既有裁决，g22 零漂移）：非单函数平衡跨度
        //（`calc(50%) calc(25%)` 双 calc 拐角语法——border-radius-002 五~八
        // 四 P 钉面）与未知函数实参（css-values-5 inherit()——
        // inherit-function-parsing 钉面）。
        assert!(validate_decl_value("border-top-left-radius", "calc(50%) calc(25%)"));
        assert!(validate_decl_value("border-top-right-radius", "calc(50%) calc(25%)"));
        assert!(validate_decl_value("left", "calc(inherit(--x) + 1px)"));
        assert!(validate_decl_value("opacity", "sin(30deg)"));
        assert!(validate_decl_value("opacity", "cos(66.66666666grad)"));
        assert!(validate_decl_value("opacity", "tan(0.125turn)"));
        assert!(validate_decl_value(
            "opacity",
            "calc(sin(30deg) + cos(60deg) + tan(45deg))"
        ));
        assert!(validate_decl_value("opacity", "sin(cos(1turn))"));
        assert!(!validate_decl_value("opacity", "sin(1px)"));
        assert!(!validate_decl_value("opacity", "calc(cos(infinity) * 1px)"));
        // number 槽 %/px 区分（LengthPercentage 族塌缩——px 长度拒、% 叶放行；
        // scale 为白名单槽，gate 放行后仍走 oracle 既有裁决面）。
        assert!(!validate_decl_value("opacity", "min(0px)"));
        assert!(!validate_decl_value("scale", "calc(1px * 2)"));
        assert!(!validate_decl_value("scale", "min(0grad)"));
        assert!(!validate_decl_value("scale", "round(1, 0.5turn)"));
        assert!(!validate_decl_value("transition-duration", "min(0px)"));
        // 豁免面：替换函数/未接线函数 parse 树不可达交既有门；custom property
        // 值任意文法（CSS Parsing 3）。
        assert!(validate_decl_value("width", "calc(var(--x) + 1px)"));
        assert!(validate_decl_value("opacity", "var(--o)"));
        assert!(validate_decl_value("margin-left", "calc(progress(50%) * 1px)"));
        assert!(validate_decl_value("width", "random(1px, 2px)"));
        assert!(validate_decl_value("--x", "min(1px, )"));
        // plain 值/非 math 形零漂移（gate 不触发）。
        assert!(validate_decl_value("border-left-width", "2px"));
        assert!(validate_decl_value("border-left-width", "thick"));
        assert!(validate_decl_value("height", "auto"));
    }
}

#[cfg(test)]
mod r4086_stretch_tests {
    use super::*;

    #[test]
    fn r4086_height_stretch_parses() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "height", "stretch"));
        assert_eq!(style.height, LengthValue::Stretch);
        let mut style2 = ComputedStyle::default();
        assert!(apply_property_value(&mut style2, "height", "-webkit-fill-available"));
        assert_eq!(style2.height, LengthValue::Stretch);
    }
}
