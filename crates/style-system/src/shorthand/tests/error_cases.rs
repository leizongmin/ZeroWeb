use super::super::*;

// ═══════════════════════════════════════════════════════════════════
// 错误路径和边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 无效的 margin 值（超过 4 个）应返回空 vec
fn test_invalid_margin_too_many_values() {
    let result = expand_one("margin", "1px 2px 3px 4px 5px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// 无效的 padding 值（超过 4 个）应返回空 vec
fn test_invalid_padding_too_many_values() {
    let result = expand_one("padding", "1px 2px 3px 4px 5px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// 无效的 border-radius 值（超过 4 个）应返回空 vec
fn test_invalid_border_radius_too_many_values() {
    let result = expand_one("border-radius", "1px 2px 3px 4px 5px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// 无效的 inset 值（超过 4 个）应返回空 vec
fn test_invalid_inset_too_many_values() {
    let result = expand_one("inset", "1px 2px 3px 4px 5px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// margin 仅包含空格应展开为默认值
fn test_margin_empty_whitespace() {
    let result = expand_one("margin", "   ", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// gap 无效值（超过 2 个）应返回空 vec
fn test_gap_invalid_too_many_values() {
    let result = expand_one("gap", "10px 20px 30px", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// gap 空值应返回空 vec
fn test_gap_empty_value() {
    let result = expand_one("gap", "", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// gap 仅空格应返回空 vec
fn test_gap_whitespace_only() {
    let result = expand_one("gap", "   ", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// overflow 单个非关键字值应展开为两个相同的值
fn test_overflow_unknown_keyword() {
    let result = expand_one("overflow", "invalid", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "overflow-x");
    assert_eq!(result[0].1, "invalid");
    assert_eq!(result[1].0, "overflow-y");
    assert_eq!(result[1].1, "invalid");
}

#[test]
/// overscroll-behavior 无效值应展开为两个相同的值
fn test_overscroll_behavior_invalid_value() {
    let result = expand_one("overscroll-behavior", "invalid", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "overscroll-behavior-x");
    assert_eq!(result[0].1, "invalid");
    assert_eq!(result[1].0, "overscroll-behavior-y");
    assert_eq!(result[1].1, "invalid");
}

#[test]
/// grid-column 无效格式（如多个 /）应正确处理第一个 /
fn test_grid_column_multiple_slashes() {
    let result = expand_one("grid-column", "1 / 2 / 3", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "grid-column-start");
    assert_eq!(result[0].1, "1");
    assert_eq!(result[1].0, "grid-column-end");
    assert_eq!(result[1].1, "2 / 3");
}

#[test]
/// grid-area 无效格式（超过 4 个值）应返回空 vec
fn test_grid_area_too_many_values() {
    let result = expand_one("grid-area", "1 / 2 / 3 / 4 / 5", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// place-items 超过两个值应返回空 vec
fn test_place_items_too_many_values() {
    let result = expand_one("place-items", "start end center", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// place-content 超过两个值应返回空 vec
fn test_place_content_too_many_values() {
    let result = expand_one("place-content", "start end center", false, (0, 0, 1));
    assert!(result.is_empty());
}

#[test]
/// place-self 超过两个值应返回空 vec
fn test_place_self_too_many_values() {
    let result = expand_one("place-self", "start end center", false, (0, 0, 1));
    assert!(result.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// 文本装饰简写测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// text-decoration 仅 line 值（R2592：第 4 longhand text-decoration-thickness 重置 auto）
fn test_text_decoration_line_only() {
    let result = expand_one("text-decoration", "underline", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "underline");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "currentcolor");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration 仅 style 值
fn test_text_decoration_style_only() {
    let result = expand_one("text-decoration", "dashed", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "dashed");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "currentcolor");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration 仅 color 值
fn test_text_decoration_color_only() {
    let result = expand_one("text-decoration", "blue", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "blue");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration line + style
fn test_text_decoration_line_and_style() {
    let result = expand_one("text-decoration", "underline dotted", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "underline");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "dotted");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "currentcolor");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration line + color
fn test_text_decoration_line_and_color() {
    let result = expand_one("text-decoration", "underline blue", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "underline");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "blue");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration style + color
fn test_text_decoration_style_and_color() {
    let result = expand_one("text-decoration", "dashed red", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "dashed");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "red");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration "none" 值
fn test_text_decoration_none() {
    let result = expand_one("text-decoration", "none", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "currentcolor");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration 顺序独立 - color, line, style
fn test_text_decoration_order_color_line_style() {
    let result = expand_one("text-decoration", "blue underline wavy", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "underline");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "wavy");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "blue");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

#[test]
/// text-decoration blink
fn test_text_decoration_blink() {
    let result = expand_one("text-decoration", "blink", false, (0, 0, 1));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, "text-decoration-line");
    assert_eq!(result[0].1, "blink");
    assert_eq!(result[1].0, "text-decoration-style");
    assert_eq!(result[1].1, "solid");
    assert_eq!(result[2].0, "text-decoration-color");
    assert_eq!(result[2].1, "currentcolor");
    assert_eq!(result[3].0, "text-decoration-thickness");
    assert_eq!(result[3].1, "auto");
}

// ═══════════════════════════════════════════════════════════════════
// 列相关简写测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// columns auto 值
fn test_columns_auto() {
    let result = expand_one("columns", "auto", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "auto");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "auto");
}

#[test]
/// columns 单个长度值带单位
fn test_columns_length_units() {
    let result = expand_one("columns", "2em", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "auto");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "2em");
}

#[test]
/// columns 双值第二个是 auto
fn test_columns_second_auto() {
    let result = expand_one("columns", "3 auto", false, (0, 0, 1));
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "column-count");
    assert_eq!(result[0].1, "3");
    assert_eq!(result[1].0, "column-width");
    assert_eq!(result[1].1, "auto");
}

#[test]
/// column-rule 仅 width
fn test_column_rule_width_only() {
    let result = expand_one("column-rule", "2px", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "column-rule-width");
    assert_eq!(result[0].1, "2px");
    assert_eq!(result[1].0, "column-rule-style");
    assert_eq!(result[1].1, "none");
    assert_eq!(result[2].0, "column-rule-color");
    assert_eq!(result[2].1, "currentcolor");
}

#[test]
/// column-rule 仅 style
fn test_column_rule_style_only() {
    let result = expand_one("column-rule", "dotted", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "column-rule-width");
    assert_eq!(result[0].1, "medium");
    assert_eq!(result[1].0, "column-rule-style");
    assert_eq!(result[1].1, "dotted");
    assert_eq!(result[2].0, "column-rule-color");
    assert_eq!(result[2].1, "currentcolor");
}

#[test]
/// column-rule 仅 color
fn test_column_rule_color_only() {
    let result = expand_one("column-rule", "red", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "column-rule-width");
    assert_eq!(result[0].1, "medium");
    assert_eq!(result[1].0, "column-rule-style");
    assert_eq!(result[1].1, "none");
    assert_eq!(result[2].0, "column-rule-color");
    assert_eq!(result[2].1, "red");
}

// ═══════════════════════════════════════════════════════════════════
// border 复合情况测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// border 仅 style
fn test_border_style_only() {
    let result = expand_one("border", "solid", false, (0, 0, 1));
    assert_eq!(result.len(), 12);
    assert_eq!(result[0].1, "medium"); // top-width
    assert_eq!(result[1].1, "solid"); // top-style
    assert_eq!(result[2].1, "currentcolor"); // top-color
    assert_eq!(result[3].1, "medium"); // right-width
    assert_eq!(result[4].1, "solid"); // right-style
    assert_eq!(result[5].1, "currentcolor"); // right-color
    assert_eq!(result[6].1, "medium"); // bottom-width
    assert_eq!(result[7].1, "solid"); // bottom-style
    assert_eq!(result[8].1, "currentcolor"); // bottom-color
    assert_eq!(result[9].1, "medium"); // left-width
    assert_eq!(result[10].1, "solid"); // left-style
    assert_eq!(result[11].1, "currentcolor"); // left-color
}

#[test]
/// border 仅 width
fn test_border_width_only() {
    let result = expand_one("border", "2px", false, (0, 0, 1));
    assert_eq!(result.len(), 12);
    assert_eq!(result[0].1, "2px"); // top-width
    assert_eq!(result[1].1, "none"); // top-style
    assert_eq!(result[2].1, "currentcolor"); // top-color
}

#[test]
/// border 仅 color
fn test_border_color_only() {
    let result = expand_one("border", "blue", false, (0, 0, 1));
    assert_eq!(result.len(), 12);
    assert_eq!(result[0].1, "medium"); // top-width
    assert_eq!(result[1].1, "none"); // top-style
    assert_eq!(result[2].1, "blue"); // top-color
    assert_eq!(result[3].1, "medium"); // right-width
    assert_eq!(result[4].1, "none"); // right-style
    assert_eq!(result[5].1, "blue"); // right-color
}

#[test]
/// border width + color
fn test_border_width_and_color() {
    let result = expand_one("border", "2px blue", false, (0, 0, 1));
    assert_eq!(result.len(), 12);
    assert_eq!(result[0].1, "2px"); // top-width
    assert_eq!(result[1].1, "none"); // top-style
    assert_eq!(result[2].1, "blue"); // top-color
}

#[test]
/// border style + color
fn test_border_style_and_color() {
    let result = expand_one("border", "solid red", false, (0, 0, 1));
    assert_eq!(result.len(), 12);
    assert_eq!(result[0].1, "medium"); // top-width
    assert_eq!(result[1].1, "solid"); // top-style
    assert_eq!(result[2].1, "red"); // top-color
}

// ═══════════════════════════════════════════════════════════════════
// 复杂背景简写测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// background 渐变函数
fn test_background_gradient_linear() {
    let result = expand_one("background", "linear-gradient(to right, red, blue)", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "transparent");
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "linear-gradient(to right, red, blue)");
}

#[test]
/// background 渐变函数带前缀
fn test_background_gradient_radial() {
    let result = expand_one("background", "radial-gradient(circle, #fff, #000)", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "radial-gradient(circle, #fff, #000)");
}

#[test]
/// background 渐变函数带 repeating
fn test_background_gradient_repeating() {
    let result = expand_one(
        "background",
        "repeating-linear-gradient(90deg, red, blue 10px)",
        false,
        (0, 0, 1),
    );
    assert_eq!(result.len(), 8);
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "repeating-linear-gradient(90deg, red, blue 10px)");
}

#[test]
/// background 颜色值加 url — 展开为所有子属性
fn test_background_color_and_url() {
    let result = expand_one("background", "red url(img.png)", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "red");
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "url(img.png)");
}

#[test]
/// background 仅 url
fn test_background_url_only() {
    let result = expand_one("background", "url('data:image/png;base64,...')", false, (0, 0, 1));
    assert_eq!(result.len(), 8);
    assert_eq!(result[0].0, "background-color");
    assert_eq!(result[0].1, "transparent");
    assert_eq!(result[1].0, "background-image");
    assert_eq!(result[1].1, "url('data:image/png;base64,...')");
}

// ═══════════════════════════════════════════════════════════════════
// font 简写边界测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// font 仅 size
fn test_font_size_only() {
    let result = expand_one("font", "16px", false, (0, 0, 1));
    assert_eq!(result.len(), 5);
    assert_eq!(result[0].0, "font-style");
    assert_eq!(result[0].1, "normal");
    assert_eq!(result[1].0, "font-weight");
    assert_eq!(result[1].1, "normal");
    assert_eq!(result[2].0, "font-size");
    assert_eq!(result[2].1, "16px");
    assert_eq!(result[3].0, "line-height");
    assert_eq!(result[3].1, "normal");
    assert_eq!(result[4].0, "font-family");
    assert_eq!(result[4].1, "");
}

#[test]
/// font 仅 weight
fn test_font_weight_only() {
    // CSS 规范：font 简写必须包含 font-size 和 font-family。
    // "bold" 缺少 font-size，声明无效。
    let result = expand_one("font", "bold", false, (0, 0, 1));
    assert!(result.is_empty(), "font: bold should be invalid");
}

#[test]
/// font 仅 style
fn test_font_style_only() {
    // CSS 规范：font 简写必须包含 font-size 和 font-family。
    // "italic" 缺少 font-size，声明无效。
    let result = expand_one("font", "italic", false, (0, 0, 1));
    assert!(result.is_empty(), "font: italic should be invalid");
}

#[test]
/// font 仅 family - 注意：当前实现不支持只有 family 的情况
fn test_font_family_only() {
    // CSS 规范：font 简写必须包含 font-size 和 font-family。
    // "Arial" 缺少 font-size，声明无效。
    let result = expand_one("font", "Arial", false, (0, 0, 1));
    assert!(result.is_empty(), "font: Arial should be invalid");
}

#[test]
/// font 简写中负 line-height 应使整个声明无效
fn test_font_negative_line_height_invalid() {
    // CSS Fonts §3.7：负 line-height 值非法，整个 font 声明无效。
    let result = expand_one("font", "4em/-2em serif", false, (0, 0, 1));
    assert!(
        result.is_empty(),
        "font: 4em/-2em serif should be invalid (negative line-height)"
    );
}

#[test]
/// font 简写中负 line-height（px 单位）应使整个声明无效
fn test_font_negative_line_height_px_invalid() {
    let result = expand_one("font", "16px/-10px sans-serif", false, (0, 0, 1));
    assert!(
        result.is_empty(),
        "font with negative line-height in px should be invalid"
    );
}

#[test]
/// font size/line-height 分隔符带空格
fn test_font_size_line_spacing_with_spaces() {
    let result = expand_one("font", "16px / 1.5", false, (0, 0, 1));
    // 验证至少展开了 font-size
    let has_size = result.iter().any(|(p, _, _, _)| p == "font-size");
    assert!(has_size, "应包含 font-size");
}

#[test]
/// font 所有属性按不常见顺序
fn test_font_unusual_order() {
    let result = expand_one("font", "oblique 700 20px/1.2 'Courier New'", false, (0, 0, 1));
    // 验证至少展开了关键属性
    let has_size = result.iter().any(|(p, _, _, _)| p == "font-size");
    let has_weight = result.iter().any(|(p, _, _, _)| p == "font-weight");
    assert!(has_size, "应包含 font-size");
    assert!(has_weight, "应包含 font-weight");
}

// ═══════════════════════════════════════════════════════════════════
// @supports font 简写求值（driving: WPT css-supports-024）
// ═══════════════════════════════════════════════════════════════════

#[test]
/// @supports 求值 font 简写：合法值（含 font-size + font-family）应判为 supported。
fn test_font_shorthand_supported_valid() {
    // driving: WPT css-supports-024 `(font: 16px serif)` 须判为 supported（块应用 → green）。
    assert!(font_shorthand_supported("16px serif"), "font: 16px serif 应 supported");
    assert!(font_shorthand_supported("italic bold 14px/1.5 Arial, sans-serif"));
    assert!(font_shorthand_supported("caption"), "系统字体关键字应 supported");
}

#[test]
/// @supports 求值 font 简写：非法值（缺 font-size / 负 line-height）应判为 unsupported。
fn test_font_shorthand_supported_invalid() {
    assert!(!font_shorthand_supported("bold"), "缺 font-size 应 unsupported");
    assert!(!font_shorthand_supported("italic"), "缺 font-size 应 unsupported");
    assert!(!font_shorthand_supported("Arial"), "仅 family 缺 size 应 unsupported");
    assert!(
        !font_shorthand_supported("16px/-2em serif"),
        "负 line-height 应 unsupported"
    );
}

// ═══════════════════════════════════════════════════════════════════
// border-image 复杂情况测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// border-image 多个 repeat 关键字
fn test_border_image_multiple_repeat() {
    let result = expand_one(
        "border-image",
        "url(img.png) 30 / 2 / 4 round stretch",
        false,
        (0, 0, 1),
    );
    // 验证展开了子属性
    let has_repeat = result.iter().any(|(p, _, _, _)| p == "border-image-repeat");
    assert!(has_repeat, "应包含 border-image-repeat");
}

#[test]
/// border-image 复杂 slice 值
fn test_border_image_complex_slice() {
    let result = expand_one(
        "border-image",
        "url(img.png) fill 30 40 / 2 3 / 4 5 round",
        false,
        (0, 0, 1),
    );
    let slice_values: Vec<_> = result
        .iter()
        .filter(|(p, _, _, _)| p == "border-image-slice")
        .map(|(_, v, _, _)| v)
        .collect();
    assert_eq!(slice_values.len(), 1);
    assert_eq!(slice_values[0], "fill 30 40");
}

#[test]
/// border-image url 中带空格
fn test_border_image_url_with_spaces() {
    let result = expand_one("border-image", "url('image with spaces.png') 30", false, (0, 0, 1));
    let has_source = result
        .iter()
        .any(|(p, v, _, _)| p == "border-image-source" && v == "url('image with spaces.png')");
    let has_slice = result.iter().any(|(p, v, _, _)| p == "border-image-slice" && v == "30");
    assert!(has_source && has_slice);
}

// ═══════════════════════════════════════════════════════════════════
// list-style 边界测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// list-type 为 none 时 position 保持 outside（R2487：现展开 3 longhand，image=none）
fn test_list_style_type_none_position_outside() {
    let result = expand_one("list-style", "none outside", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "list-style-type");
    assert_eq!(result[0].1, "none");
    assert_eq!(result[1].0, "list-style-position");
    assert_eq!(result[1].1, "outside");
    assert_eq!(result[2].0, "list-style-image");
    assert_eq!(result[2].1, "none");
}

#[test]
/// url() 作 list-style-image（R2487：现展开 image longhand，type 退回默认 disc）
fn test_list_style_type_url() {
    let result = expand_one("list-style", "url(bullet.png)", false, (0, 0, 1));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "list-style-type");
    assert_eq!(result[0].1, "disc"); // 默认
    assert_eq!(result[1].0, "list-style-position");
    assert_eq!(result[1].1, "outside"); // 默认
    assert_eq!(result[2].0, "list-style-image");
    assert_eq!(result[2].1, "url(bullet.png)");
}

// ═══════════════════════════════════════════════════════════════════
// 重要标记和特异性的传播测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 所有简写都应正确传播 important 标记
fn test_shorthand_important_propagation() {
    let properties = [
        "margin",
        "padding",
        "border",
        "border-width",
        "border-style",
        "border-color",
        "border-radius",
        "font",
        "background",
        "outline",
        "text-decoration",
        "list-style",
        "columns",
        "gap",
        "overflow",
        "overscroll-behavior",
        "transition",
        "animation",
        "grid-column",
        "grid-row",
        "grid-area",
        "place-items",
        "place-content",
        "place-self",
        "grid-template",
        "inset",
        "flex",
        "margin-block",
        "margin-inline",
        "padding-block",
        "padding-inline",
        "inset-block",
        "inset-inline",
        "border-image",
        "column-rule",
    ];

    for property in properties.iter() {
        let result = expand_one(property, "1px", true, (1, 2, 3));
        for (_, _, imp, spec) in &result {
            assert!(*imp, "Property {} should propagate important marker", property);
            assert_eq!(*spec, (1, 2, 3), "Property {} should preserve specificity", property);
        }
    }
}

#[test]
/// 简写展开时保留所有属性相同的 specificity
fn test_shorthand_specificity_consistency() {
    let result = expand_one("margin", "10px 20px", false, (5, 0, 1));
    for (_, _, _, spec) in &result {
        assert_eq!(*spec, (5, 0, 1));
    }
}

// ═══════════════════════════════════════════════════════════════════
// 非简写属性应原样返回
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 非简写属性应原样返回
fn test_non_shorthand_properties_passthrough() {
    let properties = [
        "color",
        "width",
        "height",
        "display",
        "position",
        "top",
        "left",
        "right",
        "bottom",
        "float",
        "clear",
        "z-index",
        "opacity",
        "visibility",
        "cursor",
        "white-space",
        "word-wrap",
        "text-align",
        "line-height",
        "vertical-align",
        "direction",
        "unicode-bidi",
        "letter-spacing",
        "word-spacing",
        "text-indent",
        "text-transform",
        "box-sizing",
        "content",
        "quotes",
        "counter-reset",
        "counter-increment",
        "page-break-before",
        "page-break-after",
        "page-break-inside",
        "orphans",
        "widows",
    ];

    for property in properties.iter() {
        let result = expand_one(property, "value", false, (0, 0, 1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, *property);
        assert_eq!(result[0].1, "value");
    }
}
