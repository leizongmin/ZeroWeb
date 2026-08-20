// Auto-generated test file — split from property.rs
use super::super::*;

#[test]
fn test_default_computed_style() {
    let style = ComputedStyle::default();
    assert_eq!(style.display, DisplayValue::Inline);
    assert_eq!(style.position, PositionValue::Static);
    assert_eq!(style.font_size, LengthValue::Px(16.0));
    assert_eq!(style.opacity, 1.0);
    assert_eq!(style.flex_direction, FlexDirectionValue::Row);
    assert_eq!(style.overflow_x, OverflowValue::Visible);
}

/// R2637：守护 box 维度初始值不漂移回 Px(0.0)。
///
/// `PropertyRegistry::initial_value` 与 `ComputedStyle::default()` 必须一致——
/// 生产路径 `apply_initial_value` 读 default（Auto），width/height/min-width/min-height
/// 的 Px(0.0) 是历史漂移：min-* 的 0 会短路 taffy 内容下限使 flex/grid item 可缩至 0
/// （default_impl.rs R428-R437 已注释）。本测试锁定四值为 Auto。
#[test]
fn test_box_dimension_initial_values_are_auto() {
    for prop in ["width", "height", "min-width", "min-height"] {
        assert_eq!(
            PropertyRegistry::initial_value(prop),
            Some(PropertyValue::Length(LengthValue::Auto)),
            "{prop} initial value drifted away from Auto"
        );
        // 同时断言 default 与 registry 一致（同一来源真相）。
        let default = ComputedStyle::default();
        let default_val = match prop {
            "width" => &default.width,
            "height" => &default.height,
            "min-width" => &default.min_width,
            "min-height" => &default.min_height,
            _ => unreachable!(),
        };
        assert_eq!(default_val, &LengthValue::Auto, "{prop} default drifted away from Auto");
    }
}

/// R2637（第 11 vein 全扫）：column-gap 初始值 = normal（Auto），非 0。
///
/// `ComputedStyle::default()`（R1040）用 `Auto` 保留 normal 语义（multicol 解析 1em、
/// flex/grid 解析 0）。registry 旧返回 `Px(0.0)` 与 default 漂移，且对 multicol 是
/// spec-wrong（应为 1em 非 0）。本测试锁定 column-gap == registry == default == Auto。
#[test]
fn test_column_gap_initial_value_is_auto() {
    assert_eq!(
        PropertyRegistry::initial_value("column-gap"),
        Some(PropertyValue::Length(LengthValue::Auto)),
        "column-gap initial value drifted away from Auto (normal)"
    );
    assert_eq!(
        ComputedStyle::default().column_gap,
        LengthValue::Auto,
        "column-gap default drifted away from Auto (normal)"
    );
    // gap / row-gap 初始值在 default 中为 Px(0.0)（normal 解析 0），registry 须一致。
    assert_eq!(
        PropertyRegistry::initial_value("gap"),
        Some(PropertyValue::Length(LengthValue::Px(0.0)))
    );
    assert_eq!(
        PropertyRegistry::initial_value("row-gap"),
        Some(PropertyValue::Length(LengthValue::Px(0.0)))
    );
}

#[test]
fn test_property_registry_initial_values() {
    assert!(PropertyRegistry::initial_value("display").is_some());
    assert!(PropertyRegistry::initial_value("color").is_some());
    assert!(PropertyRegistry::initial_value("font-size").is_some());
    assert!(PropertyRegistry::initial_value("unknown-prop").is_none());
}

#[test]
fn test_property_registry_inheritance() {
    // 正确的继承属性
    assert!(PropertyRegistry::is_inherited("color"));
    assert!(PropertyRegistry::is_inherited("font-size"));
    assert!(PropertyRegistry::is_inherited("visibility"));
    assert!(PropertyRegistry::is_inherited("cursor"));
    assert!(PropertyRegistry::is_inherited("line-height"));
    assert!(PropertyRegistry::is_inherited("white-space"));
    assert!(PropertyRegistry::is_inherited("text-align"));
    // CSS Text Decoration 3 §3.1/§3.2：text-emphasis-style/color/position 均继承（R2597）
    assert!(PropertyRegistry::is_inherited("text-emphasis-color"));
    assert!(PropertyRegistry::is_inherited("text-emphasis-style"));
    assert!(PropertyRegistry::is_inherited("text-emphasis-position"));
    // 不应继承的属性
    assert!(!PropertyRegistry::is_inherited("display"));
    assert!(!PropertyRegistry::is_inherited("margin-top"));
    assert!(!PropertyRegistry::is_inherited("width"));
    assert!(!PropertyRegistry::is_inherited("opacity"));
    assert!(!PropertyRegistry::is_inherited("text-decoration"));
    assert!(!PropertyRegistry::is_inherited("text-overflow"));
}

#[test]
fn test_parse_border_style() {
    assert_eq!(parse_border_style("solid"), Some(BorderStyleValue::Solid));
    assert_eq!(parse_border_style("none"), Some(BorderStyleValue::None));
    assert_eq!(parse_border_style("dashed"), Some(BorderStyleValue::Dashed));
    assert_eq!(parse_border_style("invalid"), None);
}

#[test]
fn test_parse_line_height() {
    assert_eq!(parse_line_height("normal"), Some(LineHeightValue::Normal));
    assert_eq!(parse_line_height("1.5"), Some(LineHeightValue::Number(1.5)));
    assert_eq!(
        parse_line_height("24px"),
        Some(LineHeightValue::Length(LengthValue::Px(24.0)))
    );
}

#[test]
fn test_parse_text_align() {
    assert_eq!(parse_text_align("center"), Some(TextAlignValue::Center));
    assert_eq!(parse_text_align("justify"), Some(TextAlignValue::Justify));
    assert_eq!(parse_text_align("invalid"), None);
}

#[test]
fn test_parse_text_decoration() {
    assert_eq!(parse_text_decoration("underline"), Some(TextDecorationValue::Underline));
    assert_eq!(parse_text_decoration("none"), Some(TextDecorationValue::None));
}

#[test]
fn test_parse_text_transform() {
    assert_eq!(parse_text_transform("uppercase"), Some(TextTransformValue::Uppercase));
    assert_eq!(parse_text_transform("capitalize"), Some(TextTransformValue::Capitalize));
}

#[test]
fn test_parse_white_space() {
    assert_eq!(parse_white_space("nowrap"), Some(WhiteSpaceValue::Nowrap));
    assert_eq!(parse_white_space("pre-wrap"), Some(WhiteSpaceValue::PreWrap));
}

#[test]
fn test_parse_flex_basis() {
    assert_eq!(parse_flex_basis("auto"), Some(FlexBasisValue::Auto));
    assert_eq!(parse_flex_basis("content"), Some(FlexBasisValue::Content));
    assert_eq!(
        parse_flex_basis("100px"),
        Some(FlexBasisValue::Length(LengthValue::Px(100.0)))
    );
    assert_eq!(parse_flex_basis("-1px"), None);
    assert_eq!(parse_flex_basis("-50%"), None);
    assert_eq!(parse_flex_basis("thin"), None);
}

#[test]
fn test_parse_z_index() {
    assert_eq!(parse_z_index("auto"), Some(ZIndexValue::Auto));
    assert_eq!(parse_z_index("10"), Some(ZIndexValue::Integer(10)));
    assert_eq!(parse_z_index("-1"), Some(ZIndexValue::Integer(-1)));
}

#[test]
fn test_parse_font_family() {
    let families = parse_font_family("Arial, sans-serif");
    assert_eq!(families, vec!["Arial", "sans-serif"]);

    let families = parse_font_family("\"Times New Roman\", serif");
    assert_eq!(families, vec!["\"Times New Roman\"", "serif"]);
}

#[test]
fn test_apply_property_value() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "display", "flex"));
    assert_eq!(style.display, DisplayValue::Flex);

    assert!(apply_property_value(&mut style, "color", "red"));
    assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));

    assert!(apply_property_value(&mut style, "opacity", "0.5"));
    assert_eq!(style.opacity, 0.5);

    assert!(!apply_property_value(&mut style, "display", "invalid"));
}

#[test]
fn test_apply_property_value_border() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-top-width", "2px"));
    assert_eq!(style.border_top_width, LengthValue::Px(2.0));

    assert!(apply_property_value(&mut style, "border-top-style", "solid"));
    assert_eq!(style.border_top_style, BorderStyleValue::Solid);

    assert!(apply_property_value(&mut style, "border-top-color", "#ff0000"));
    assert_eq!(style.border_top_color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_inherit_property() {
    let mut parent = ComputedStyle::default();
    parent.color = ColorValue::Rgba(255, 0, 0, 255);
    parent.font_size = LengthValue::Px(20.0);

    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "color"));
    assert_eq!(child.color, ColorValue::Rgba(255, 0, 0, 255));

    assert!(inherit_property(&parent, &mut child, "font-size"));
    assert_eq!(child.font_size, LengthValue::Px(20.0));

    // transform 仍不在 inherit 表（display/float/position 等 R754 已支持显式 inherit）
    assert!(!inherit_property(&parent, &mut child, "transform"));
}

#[test]
fn test_apply_initial_value() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.opacity = 0.5;

    assert!(apply_initial_value(&mut style, "display"));
    assert_eq!(style.display, DisplayValue::Inline);

    assert!(apply_initial_value(&mut style, "opacity"));
    assert_eq!(style.opacity, 1.0);
}

#[test]
/// 测试 apply_initial_value 覆盖所有已知属性
fn test_apply_initial_value_all_properties() {
    for prop in PropertyRegistry::known_properties() {
        let mut style = ComputedStyle::default();
        // 先修改一个属性值
        apply_property_value(&mut style, prop, "999px");
        // 重置为初始值应成功
        assert!(
            apply_initial_value(&mut style, prop),
            "apply_initial_value should handle: {prop}"
        );
    }
    // 未知属性应返回 false
    assert!(!apply_initial_value(&mut ComputedStyle::default(), "unknown-prop"));
}

#[test]
fn test_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"display"));
    assert!(props.contains(&"color"));
    assert!(props.contains(&"flex-direction"));
    assert!(props.len() >= 50);
}

#[test]
fn test_parse_text_overflow() {
    assert_eq!(parse_text_overflow("ellipsis"), Some(TextOverflowValue::Ellipsis));
    assert_eq!(parse_text_overflow("clip"), Some(TextOverflowValue::Clip));
}

// ═══════════════════════════════════════════════════════════════════
// 扩展测试 — 提升 property.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 apply_property_value 对 display: flex
fn test_apply_property_display_flex() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "display", "flex"));
    assert_eq!(style.display, DisplayValue::Flex);
}

#[test]
/// 测试 apply_property_value 对 display: grid
fn test_apply_property_display_grid() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "display", "grid"));
    assert_eq!(style.display, DisplayValue::Grid);
}

#[test]
/// 测试 apply_property_value 对 position: absolute
fn test_apply_property_position_absolute() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "position", "absolute"));
    assert_eq!(style.position, PositionValue::Absolute);
}

#[test]
/// 测试 apply_property_value 对 font-size: em 单位
fn test_apply_property_font_size_em() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "font-size", "large"));
    assert_eq!(style.font_size, LengthValue::Px(18.0));
    assert!(apply_property_value(&mut style, "font-size", "1.5em"));
    assert_eq!(style.font_size, LengthValue::Em(1.5));
    assert!(apply_property_value(&mut style, "font-size", "125%"));
    assert_eq!(style.font_size, LengthValue::Percentage(125.0));
    let previous = style.font_size.clone();
    for value in [
        "auto",
        "thin",
        "-1px",
        "-5%",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "font-size", value));
        assert_eq!(style.font_size, previous, "{} should not overwrite", value);
    }
}

#[test]
/// 测试 apply_property_value 对 color: 十六进制
fn test_apply_property_color_hex() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "color", "#ff0000"));
    assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// R2523：text-emphasis-color（CSS Text Decoration 3 §3.3）apply + 默认 currentColor。
fn test_apply_property_text_emphasis_color() {
    // 默认 = currentColor
    let style = ComputedStyle::default();
    assert_eq!(style.text_emphasis_color, ColorValue::CurrentColor);
    // 显式色覆盖
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-emphasis-color", "green"));
    assert_eq!(style.text_emphasis_color, ColorValue::Rgba(0, 128, 0, 255));
    // 非法值 → false，字段不变
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "text-emphasis-color", "notacolor"));
    assert_eq!(style.text_emphasis_color, ColorValue::CurrentColor);
}

#[test]
/// 测试 apply_property_value 对 opacity
fn test_apply_property_opacity() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "opacity", "0.3"));
    assert!((style.opacity - 0.3).abs() < f64::EPSILON);

    // 超出范围应被 clamp
    assert!(apply_property_value(&mut style, "opacity", "2.0"));
    assert_eq!(style.opacity, 1.0);

    assert!(apply_property_value(&mut style, "opacity", "-0.5"));
    assert_eq!(style.opacity, 0.0);
}

#[test]
/// 测试 apply_property_value 对 flex-direction: column
fn test_apply_property_flex_direction_column() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "flex-direction", "column"));
    assert_eq!(style.flex_direction, FlexDirectionValue::Column);
}

#[test]
/// 测试 apply_property_value 对 z-index 整数
fn test_apply_property_z_index_integer() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "z-index", "100"));
    assert_eq!(style.z_index, ZIndexValue::Integer(100));

    assert!(apply_property_value(&mut style, "z-index", "auto"));
    assert_eq!(style.z_index, ZIndexValue::Auto);

    assert!(apply_property_value(&mut style, "z-index", "-5"));
    assert_eq!(style.z_index, ZIndexValue::Integer(-5));
}

#[test]
/// 测试 apply_property_value 对 text-align: center
fn test_apply_property_text_align_center() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-align", "center"));
    assert_eq!(style.text_align, TextAlignValue::Center);
}

#[test]
/// 测试 apply_property_value 对 line-height: 无单位数值
fn test_apply_property_line_height_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "line-height", "1.6"));
    assert_eq!(style.line_height, LineHeightValue::Number(1.6));
    let previous = style.line_height.clone();
    assert!(!apply_property_value(&mut style, "line-height", "-1"));
    assert_eq!(style.line_height, previous);
    assert!(!apply_property_value(&mut style, "line-height", "-2px"));
    assert_eq!(style.line_height, previous);
    assert!(!apply_property_value(&mut style, "line-height", "thin"));
    assert_eq!(style.line_height, previous);
}

#[test]
/// 测试 apply_property_value 对 border-style 各边
fn test_apply_property_border_style() {
    let mut style = ComputedStyle::default();

    assert!(apply_property_value(&mut style, "border-top-style", "dashed"));
    assert_eq!(style.border_top_style, BorderStyleValue::Dashed);

    assert!(apply_property_value(&mut style, "border-right-style", "dotted"));
    assert_eq!(style.border_right_style, BorderStyleValue::Dotted);

    assert!(apply_property_value(&mut style, "border-bottom-style", "solid"));
    assert_eq!(style.border_bottom_style, BorderStyleValue::Solid);

    assert!(apply_property_value(&mut style, "border-left-style", "double"));
    assert_eq!(style.border_left_style, BorderStyleValue::Double);

    // 无效值应返回 false
    assert!(!apply_property_value(&mut style, "border-top-style", "invalid"));
}

#[test]
/// 测试 apply_property_value 对 gap
fn test_apply_property_gap() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "gap", "10px"));
    assert_eq!(style.gap, LengthValue::Px(10.0));
    assert!(apply_property_value(&mut style, "gap", "normal"));
    assert_eq!(style.gap, LengthValue::Px(0.0));
    assert!(apply_property_value(&mut style, "gap", "25%"));
    assert_eq!(style.gap, LengthValue::Percentage(25.0));
    let previous = style.gap.clone();
    for value in [
        "auto",
        "thin",
        "-1px",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "gap", value));
        assert_eq!(style.gap, previous, "{} should not overwrite", value);
    }
}

#[test]
/// 测试 apply_property_value 应用多种不同属性
fn test_apply_property_multiple_different_properties() {
    let mut style = ComputedStyle::default();

    // 盒模型
    assert!(apply_property_value(&mut style, "width", "200px"));
    assert_eq!(style.width, LengthValue::Px(200.0));

    assert!(apply_property_value(&mut style, "height", "100px"));
    assert_eq!(style.height, LengthValue::Px(100.0));

    assert!(apply_property_value(&mut style, "min-width", "50px"));
    assert_eq!(style.min_width, LengthValue::Px(50.0));

    assert!(apply_property_value(&mut style, "max-width", "none"));
    assert_eq!(style.max_width, LengthValue::Px(f64::INFINITY));

    assert!(apply_property_value(&mut style, "max-height", "500px"));
    assert_eq!(style.max_height, LengthValue::Px(500.0));

    // margin 各边
    assert!(apply_property_value(&mut style, "margin-top", "10px"));
    assert!(apply_property_value(&mut style, "margin-right", "20px"));
    assert!(apply_property_value(&mut style, "margin-bottom", "10px"));
    assert!(apply_property_value(&mut style, "margin-left", "20px"));
    assert_eq!(style.margin_top, LengthValue::Px(10.0));
    assert_eq!(style.margin_right, LengthValue::Px(20.0));
    assert!(apply_property_value(&mut style, "margin-top", "auto"));
    assert_eq!(style.margin_top, LengthValue::Auto);
    assert!(apply_property_value(&mut style, "margin-right", "-5px"));
    assert_eq!(style.margin_right, LengthValue::Px(-5.0));
    assert!(apply_property_value(&mut style, "margin-bottom", "25%"));
    assert_eq!(style.margin_bottom, LengthValue::Percentage(25.0));
    let previous_left_margin = style.margin_left.clone();
    for value in ["thin", "min-content", "fit-content(10px)", "infpx", "NaNpx"] {
        assert!(!apply_property_value(&mut style, "margin-left", value));
        assert_eq!(
            style.margin_left, previous_left_margin,
            "{} should not overwrite",
            value
        );
    }

    // padding 各边
    assert!(apply_property_value(&mut style, "padding-top", "5px"));
    assert!(apply_property_value(&mut style, "padding-right", "10px"));
    assert!(apply_property_value(&mut style, "padding-bottom", "5px"));
    assert!(apply_property_value(&mut style, "padding-left", "10px"));
    assert_eq!(style.padding_top, LengthValue::Px(5.0));
    assert_eq!(style.padding_left, LengthValue::Px(10.0));

    // box-sizing
    assert!(apply_property_value(&mut style, "box-sizing", "border-box"));
    assert_eq!(style.box_sizing, BoxSizingValue::BorderBox);

    // 边框颜色各边
    assert!(apply_property_value(&mut style, "border-top-color", "red"));
    assert!(apply_property_value(&mut style, "border-right-color", "#00ff00"));
    assert!(apply_property_value(&mut style, "border-bottom-color", "blue"));
    assert!(apply_property_value(&mut style, "border-left-color", "transparent"));
    assert_eq!(style.border_top_color, ColorValue::Rgba(255, 0, 0, 255));
    assert_eq!(style.border_left_color, ColorValue::Transparent);

    // 边框宽度各边
    assert!(apply_property_value(&mut style, "border-top-width", "1px"));
    assert!(apply_property_value(&mut style, "border-right-width", "2px"));
    assert!(apply_property_value(&mut style, "border-bottom-width", "3px"));
    assert!(apply_property_value(&mut style, "border-left-width", "4px"));
    assert_eq!(style.border_top_width, LengthValue::Px(1.0));
    assert_eq!(style.border_left_width, LengthValue::Px(4.0));
    assert!(apply_property_value(&mut style, "border-top-width", "thin"));
    assert_eq!(style.border_top_width, LengthValue::Px(1.0));
    let previous_left_border = style.border_left_width.clone();
    for value in [
        "10%",
        "auto",
        "-1px",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "border-left-width", value));
        assert_eq!(
            style.border_left_width, previous_left_border,
            "{} should not overwrite",
            value
        );
    }

    // 圆角各角
    assert!(apply_property_value(&mut style, "border-top-left-radius", "8px"));
    assert!(apply_property_value(&mut style, "border-top-right-radius", "4px"));
    assert!(apply_property_value(&mut style, "border-bottom-right-radius", "8px"));
    assert!(apply_property_value(&mut style, "border-bottom-left-radius", "4px"));
    assert_eq!(style.border_top_left_radius, LengthValue::Px(8.0));
    assert_eq!(style.border_bottom_left_radius, LengthValue::Px(4.0));

    // background-color
    assert!(apply_property_value(&mut style, "background-color", "#0000ff"));
    assert_eq!(style.background_color, ColorValue::Rgba(0, 0, 255, 255));

    // visibility
    assert!(apply_property_value(&mut style, "visibility", "hidden"));
    assert_eq!(style.visibility, VisibilityValue::Hidden);

    // font-weight
    assert!(apply_property_value(&mut style, "font-weight", "bold"));
    assert_eq!(style.font_weight, FontWeightValue::Bold);

    // font-style
    assert!(apply_property_value(&mut style, "font-style", "italic"));
    assert_eq!(style.font_style, FontStyleValue::Italic);

    // line-height
    assert!(apply_property_value(&mut style, "line-height", "24px"));
    assert_eq!(style.line_height, LineHeightValue::Length(LengthValue::Px(24.0)));

    // text-decoration
    assert!(apply_property_value(&mut style, "text-decoration", "underline"));
    assert_eq!(style.text_decoration, TextDecorationValue::Underline);

    // text-transform
    assert!(apply_property_value(&mut style, "text-transform", "uppercase"));
    assert_eq!(style.text_transform, TextTransformValue::Uppercase);

    // letter-spacing, word-spacing
    assert!(apply_property_value(&mut style, "letter-spacing", "2px"));
    assert_eq!(style.letter_spacing, LengthValue::Px(2.0));
    assert!(apply_property_value(&mut style, "word-spacing", "4px"));
    assert_eq!(style.word_spacing, LengthValue::Px(4.0));

    // white-space
    assert!(apply_property_value(&mut style, "white-space", "nowrap"));
    assert_eq!(style.white_space, WhiteSpaceValue::Nowrap);

    // text-overflow
    assert!(apply_property_value(&mut style, "text-overflow", "ellipsis"));
    assert_eq!(style.text_overflow, TextOverflowValue::Ellipsis);

    // flex-wrap
    assert!(apply_property_value(&mut style, "flex-wrap", "wrap"));
    assert_eq!(style.flex_wrap, FlexWrapValue::Wrap);

    // justify-content
    assert!(apply_property_value(&mut style, "justify-content", "center"));
    assert_eq!(style.justify_content, AlignmentValue::Center);

    // align-items
    assert!(apply_property_value(&mut style, "align-items", "flex-end"));
    assert_eq!(style.align_items, AlignmentValue::FlexEnd);

    // align-self
    assert!(apply_property_value(&mut style, "align-self", "baseline"));
    assert_eq!(style.align_self, AlignmentValue::Baseline);

    // flex-grow, flex-shrink
    assert!(apply_property_value(&mut style, "flex-grow", "2.0"));
    assert_eq!(style.flex_grow, 2.0);
    assert!(apply_property_value(&mut style, "flex-shrink", "0.5"));
    assert_eq!(style.flex_shrink, 0.5);

    // flex-basis
    assert!(apply_property_value(&mut style, "flex-basis", "auto"));
    assert_eq!(style.flex_basis, FlexBasisValue::Auto);
    assert!(apply_property_value(&mut style, "flex-basis", "200px"));
    assert_eq!(style.flex_basis, FlexBasisValue::Length(LengthValue::Px(200.0)));
    let previous = style.flex_basis.clone();
    assert!(!apply_property_value(&mut style, "flex-basis", "-1px"));
    assert_eq!(style.flex_basis, previous);
    assert!(!apply_property_value(&mut style, "flex-basis", "thin"));
    assert_eq!(style.flex_basis, previous);

    // order
    assert!(apply_property_value(&mut style, "order", "3"));
    assert_eq!(style.order, 3);

    // 定位 top/right/bottom/left
    assert!(apply_property_value(&mut style, "top", "10px"));
    assert!(apply_property_value(&mut style, "right", "20px"));
    assert!(apply_property_value(&mut style, "bottom", "30px"));
    assert!(apply_property_value(&mut style, "left", "40px"));
    assert_eq!(style.top, LengthValue::Px(10.0));
    assert_eq!(style.left, LengthValue::Px(40.0));
    assert!(apply_property_value(&mut style, "top", "auto"));
    assert_eq!(style.top, LengthValue::Auto);
    assert!(apply_property_value(&mut style, "right", "-5px"));
    assert_eq!(style.right, LengthValue::Px(-5.0));
    assert!(apply_property_value(&mut style, "bottom", "25%"));
    assert_eq!(style.bottom, LengthValue::Percentage(25.0));
    let previous_left = style.left.clone();
    for value in ["thin", "min-content", "fit-content(10px)", "infpx", "NaNpx"] {
        assert!(!apply_property_value(&mut style, "left", value));
        assert_eq!(style.left, previous_left, "{} should not overwrite", value);
    }

    // overflow
    assert!(apply_property_value(&mut style, "overflow-x", "hidden"));
    assert!(apply_property_value(&mut style, "overflow-y", "scroll"));
    assert_eq!(style.overflow_x, OverflowValue::Hidden);
    assert_eq!(style.overflow_y, OverflowValue::Scroll);

    // 未知属性应返回 false
    assert!(!apply_property_value(&mut style, "unknown-prop", "value"));

    // 无效值应返回 false
    assert!(!apply_property_value(&mut style, "display", "invalid-display"));
}

#[test]
/// 测试 is_inherited 的全面列表
fn test_property_is_inherited_various() {
    // 继承属性（按 CSS 规范）
    assert!(PropertyRegistry::is_inherited("color"));
    assert!(PropertyRegistry::is_inherited("font-family"));
    assert!(PropertyRegistry::is_inherited("font-size"));
    assert!(PropertyRegistry::is_inherited("font-weight"));
    assert!(PropertyRegistry::is_inherited("font-style"));
    assert!(PropertyRegistry::is_inherited("line-height"));
    assert!(PropertyRegistry::is_inherited("text-align"));
    assert!(PropertyRegistry::is_inherited("text-transform"));
    assert!(PropertyRegistry::is_inherited("letter-spacing"));
    assert!(PropertyRegistry::is_inherited("word-spacing"));
    assert!(PropertyRegistry::is_inherited("white-space"));
    assert!(PropertyRegistry::is_inherited("visibility"));
    assert!(PropertyRegistry::is_inherited("cursor"));
    // 不继承的属性（按 CSS 规范）
    assert!(!PropertyRegistry::is_inherited("text-decoration"));
    assert!(!PropertyRegistry::is_inherited("text-overflow"));
    assert!(!PropertyRegistry::is_inherited("opacity"));

    // 非继承属性
    assert!(!PropertyRegistry::is_inherited("display"));
    assert!(!PropertyRegistry::is_inherited("position"));
    assert!(!PropertyRegistry::is_inherited("width"));
    assert!(!PropertyRegistry::is_inherited("height"));
    assert!(!PropertyRegistry::is_inherited("margin-top"));
    assert!(!PropertyRegistry::is_inherited("padding-top"));
    assert!(!PropertyRegistry::is_inherited("box-sizing"));
    assert!(!PropertyRegistry::is_inherited("border-top-width"));
    assert!(!PropertyRegistry::is_inherited("background-color"));
    assert!(!PropertyRegistry::is_inherited("flex-direction"));
    assert!(!PropertyRegistry::is_inherited("flex-wrap"));
    assert!(!PropertyRegistry::is_inherited("justify-content"));
    assert!(!PropertyRegistry::is_inherited("align-items"));
    assert!(!PropertyRegistry::is_inherited("gap"));
    assert!(!PropertyRegistry::is_inherited("z-index"));
    assert!(!PropertyRegistry::is_inherited("overflow-x"));
    assert!(!PropertyRegistry::is_inherited("order"));
    assert!(!PropertyRegistry::is_inherited("top"));
    assert!(!PropertyRegistry::is_inherited("unknown-prop"));
}

#[test]
/// 测试 parse_font_family 带引号
fn test_parse_font_family_with_quotes() {
    let families = parse_font_family("'Helvetica Neue', Arial, sans-serif");
    assert_eq!(families, vec!["\"Helvetica Neue\"", "Arial", "sans-serif"]);

    // 双引号
    let families = parse_font_family("\"Times New Roman\", serif");
    assert_eq!(families, vec!["\"Times New Roman\"", "serif"]);

    // 空字符串和空白处理
    let families = parse_font_family("  Arial  ,  sans-serif  ");
    assert_eq!(families, vec!["Arial", "sans-serif"]);
}

#[test]
/// 测试 parse_line_height 长度值
fn test_parse_line_height_length() {
    assert_eq!(
        parse_line_height("24px"),
        Some(LineHeightValue::Length(LengthValue::Px(24.0)))
    );
    assert_eq!(
        parse_line_height("2em"),
        Some(LineHeightValue::Length(LengthValue::Em(2.0)))
    );
    assert_eq!(
        parse_line_height("1.5rem"),
        Some(LineHeightValue::Length(LengthValue::Rem(1.5)))
    );
    assert_eq!(parse_line_height("normal"), Some(LineHeightValue::Normal));
    assert_eq!(parse_line_height("1.5"), Some(LineHeightValue::Number(1.5)));
    assert_eq!(parse_line_height("invalid"), None);
    assert_eq!(parse_line_height("-1"), None);
    assert_eq!(parse_line_height("-2px"), None);
    assert_eq!(parse_line_height("-50%"), None);
    assert_eq!(parse_line_height("thin"), None);
    assert_eq!(parse_line_height("inf"), None);
    assert_eq!(parse_line_height("NaN"), None);
}

// ── Grid 属性测试 ──

#[test]
fn test_apply_property_grid_template_columns() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "grid-template-columns",
        "100px 1fr auto"
    ));
    assert_eq!(style.grid_template_columns, Some("100px 1fr auto".to_string()));
}

#[test]
fn test_apply_property_grid_template_rows() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-template-rows", "50px 1fr"));
    assert_eq!(style.grid_template_rows, Some("50px 1fr".to_string()));
}

#[test]
fn test_apply_property_grid_auto_flow() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-auto-flow", "column"));
    assert_eq!(style.grid_auto_flow, GridAutoFlowValue::Column);

    assert!(apply_property_value(&mut style, "grid-auto-flow", "row dense"));
    assert_eq!(style.grid_auto_flow, GridAutoFlowValue::RowDense);

    assert!(apply_property_value(&mut style, "grid-auto-flow", "column dense"));
    assert_eq!(style.grid_auto_flow, GridAutoFlowValue::ColumnDense);

    // 无效值应返回 false
    assert!(!apply_property_value(&mut style, "grid-auto-flow", "invalid"));
}

#[test]
fn test_apply_property_row_gap() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "row-gap", "20px"));
    assert_eq!(style.row_gap, LengthValue::Px(20.0));
    let previous = style.row_gap.clone();
    assert!(!apply_property_value(&mut style, "row-gap", "-1px"));
    assert_eq!(style.row_gap, previous);
    assert!(!apply_property_value(&mut style, "row-gap", "thin"));
    assert_eq!(style.row_gap, previous);
    assert!(apply_property_value(&mut style, "row-gap", "normal"));
    assert_eq!(style.row_gap, LengthValue::Px(0.0));
}

#[test]
fn test_parse_grid_auto_flow() {
    assert_eq!(parse_grid_auto_flow("row"), Some(GridAutoFlowValue::Row));
    assert_eq!(parse_grid_auto_flow("column"), Some(GridAutoFlowValue::Column));
    assert_eq!(parse_grid_auto_flow("dense"), Some(GridAutoFlowValue::RowDense));
    assert_eq!(parse_grid_auto_flow("row dense"), Some(GridAutoFlowValue::RowDense));
    assert_eq!(
        parse_grid_auto_flow("column dense"),
        Some(GridAutoFlowValue::ColumnDense)
    );
    assert_eq!(parse_grid_auto_flow("invalid"), None);
}

#[test]
fn test_computed_style_default_grid() {
    let style = ComputedStyle::default();
    assert_eq!(style.grid_template_columns, None);
    assert_eq!(style.grid_template_rows, None);
    assert_eq!(style.grid_auto_flow, GridAutoFlowValue::Row);
    assert_eq!(style.row_gap, LengthValue::Px(0.0));
    assert_eq!(style.grid_column_start, GridLineValue::Auto);
    assert_eq!(style.grid_column_end, GridLineValue::Auto);
    assert_eq!(style.grid_row_start, GridLineValue::Auto);
    assert_eq!(style.grid_row_end, GridLineValue::Auto);
}

// ── Grid line 值测试 ──

#[test]
fn test_parse_grid_line() {
    assert_eq!(parse_grid_line("auto"), Some(GridLineValue::Auto));
    assert_eq!(parse_grid_line("1"), Some(GridLineValue::Line(1)));
    assert_eq!(parse_grid_line("-1"), Some(GridLineValue::Line(-1)));
    assert_eq!(parse_grid_line("5"), Some(GridLineValue::Line(5)));
    assert_eq!(parse_grid_line("span 2"), Some(GridLineValue::Span(2)));
    assert_eq!(parse_grid_line("span 3"), Some(GridLineValue::Span(3)));
    assert_eq!(parse_grid_line("0"), None); // 0 is invalid
    assert_eq!(
        parse_grid_line("invalid"),
        Some(GridLineValue::Name("invalid".to_string()))
    );
    assert_eq!(
        parse_grid_line("header"),
        Some(GridLineValue::Name("header".to_string()))
    );
}

#[test]
fn test_apply_grid_column_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-column-start", "1"));
    assert_eq!(style.grid_column_start, GridLineValue::Line(1));

    assert!(apply_property_value(&mut style, "grid-column-start", "-1"));
    assert_eq!(style.grid_column_start, GridLineValue::Line(-1));

    assert!(apply_property_value(&mut style, "grid-column-start", "span 2"));
    assert_eq!(style.grid_column_start, GridLineValue::Span(2));

    assert!(apply_property_value(&mut style, "grid-column-start", "auto"));
    assert_eq!(style.grid_column_start, GridLineValue::Auto);
}

#[test]
fn test_apply_grid_row_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-row-start", "2"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(2));

    assert!(apply_property_value(&mut style, "grid-row-end", "3"));
    assert_eq!(style.grid_row_end, GridLineValue::Line(3));
}

// ── Transition 属性测试 ──

#[test]
fn test_computed_style_default_transition() {
    let style = ComputedStyle::default();
    assert!(style.transition_property.is_empty());
    assert!(style.transition_duration.is_empty());
    assert!(style.transition_timing_function.is_empty());
    assert!(style.transition_delay.is_empty());
}

#[test]
fn test_apply_transition_property() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transition-property", "opacity"));
    assert_eq!(style.transition_property, vec!["opacity"]);

    assert!(apply_property_value(
        &mut style,
        "transition-property",
        "opacity, transform"
    ));
    assert_eq!(style.transition_property, vec!["opacity", "transform"]);

    assert!(apply_property_value(&mut style, "transition-property", "all"));
    assert_eq!(style.transition_property, vec!["all"]);
}

#[test]
fn test_apply_transition_duration() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transition-duration", "0.3s"));
    assert_eq!(style.transition_duration, vec![0.3]);

    assert!(apply_property_value(&mut style, "transition-duration", "0.3s, 0.5s"));
    assert_eq!(style.transition_duration, vec![0.3, 0.5]);

    assert!(apply_property_value(&mut style, "transition-duration", "200ms"));
    assert_eq!(style.transition_duration, vec![0.2]);

    assert!(!apply_property_value(&mut style, "transition-duration", "-1s"));
    assert_eq!(style.transition_duration, vec![0.2]);

    assert!(!apply_property_value(&mut style, "transition-duration", "infs"));
    assert_eq!(style.transition_duration, vec![0.2]);

    assert!(apply_property_value(&mut style, "transition-delay", "0.1s"));
    assert_eq!(style.transition_delay, vec![0.1]);
    assert!(!apply_property_value(&mut style, "transition-delay", "NaNs"));
    assert_eq!(style.transition_delay, vec![0.1]);
}

#[test]
fn test_apply_transition_timing_function() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transition-timing-function", "ease"));
    assert_eq!(style.transition_timing_function.len(), 1);
    assert_eq!(
        style.transition_timing_function[0],
        zero_css_parser::values::TimingFunctionValue::Ease
    );

    assert!(apply_property_value(
        &mut style,
        "transition-timing-function",
        "cubic-bezier(0.25, 0.1, 0.25, 1.0)"
    ));
    assert_eq!(style.transition_timing_function.len(), 1);

    assert!(apply_property_value(
        &mut style,
        "transition-timing-function",
        "ease, linear"
    ));
    assert_eq!(style.transition_timing_function.len(), 2);
}

#[test]
fn test_apply_transition_delay() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transition-delay", "0.1s"));
    assert_eq!(style.transition_delay, vec![0.1]);

    assert!(apply_property_value(&mut style, "transition-delay", "0.1s, 0.2s"));
    assert_eq!(style.transition_delay, vec![0.1, 0.2]);

    assert!(apply_property_value(&mut style, "transition-delay", "50ms"));
    assert_eq!(style.transition_delay, vec![0.05]);

    assert!(apply_property_value(&mut style, "transition-delay", "-1s"));
    assert_eq!(style.transition_delay, vec![-1.0]);

    assert!(!apply_property_value(&mut style, "transition-delay", "0.1s, bogus"));
    assert_eq!(style.transition_delay, vec![-1.0]);
}

#[test]
fn test_transition_property_registry() {
    assert!(PropertyRegistry::initial_value("transition-property").is_some());
    assert!(PropertyRegistry::initial_value("transition-duration").is_some());
    assert!(PropertyRegistry::initial_value("transition-delay").is_some());
    // transition-timing-function 没有 PropertyValue 变体，但仍应被已知属性接受
    assert!(!PropertyRegistry::is_inherited("transition-property"));
    assert!(!PropertyRegistry::is_inherited("transition-duration"));
}

#[test]
fn test_transition_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"transition-property"));
    assert!(props.contains(&"transition-duration"));
    assert!(props.contains(&"transition-timing-function"));
    assert!(props.contains(&"transition-delay"));
}

#[test]
fn test_parse_comma_separated_timing_functions() {
    let result = parse_comma_separated_timing_functions("ease, linear").unwrap();
    assert_eq!(result.len(), 2);

    let result = parse_comma_separated_timing_functions("cubic-bezier(0.25, 0.1, 0.25, 1.0)").unwrap();
    assert_eq!(result.len(), 1);

    let result = parse_comma_separated_timing_functions("ease, cubic-bezier(0.25, 0.1, 0.25, 1.0), steps(4)").unwrap();
    assert_eq!(result.len(), 3);

    assert!(parse_comma_separated_timing_functions("ease, bogus").is_none());
    assert!(parse_comma_separated_timing_functions("ease,").is_none());
}

// ── float/clear 属性测试 ──

#[test]
fn test_apply_property_float() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.float, zero_css_parser::values::FloatValue::None);

    assert!(apply_property_value(&mut style, "float", "left"));
    assert_eq!(style.float, zero_css_parser::values::FloatValue::Left);

    assert!(apply_property_value(&mut style, "float", "right"));
    assert_eq!(style.float, zero_css_parser::values::FloatValue::Right);

    assert!(apply_property_value(&mut style, "float", "none"));
    assert_eq!(style.float, zero_css_parser::values::FloatValue::None);

    assert!(!apply_property_value(&mut style, "float", "center"));
}

#[test]
fn test_apply_property_clear() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.clear, zero_css_parser::values::ClearValue::None);

    assert!(apply_property_value(&mut style, "clear", "both"));
    assert_eq!(style.clear, zero_css_parser::values::ClearValue::Both);

    assert!(apply_property_value(&mut style, "clear", "left"));
    assert_eq!(style.clear, zero_css_parser::values::ClearValue::Left);

    assert!(apply_property_value(&mut style, "clear", "right"));
    assert_eq!(style.clear, zero_css_parser::values::ClearValue::Right);

    assert!(apply_property_value(&mut style, "clear", "none"));
    assert_eq!(style.clear, zero_css_parser::values::ClearValue::None);

    assert!(!apply_property_value(&mut style, "clear", "all"));
}

#[test]
fn test_float_clear_property_registry() {
    assert!(PropertyRegistry::initial_value("float").is_some());
    assert!(PropertyRegistry::initial_value("clear").is_some());
    assert!(!PropertyRegistry::is_inherited("float"));
    assert!(!PropertyRegistry::is_inherited("clear"));
}

#[test]
fn test_float_clear_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"float"));
    assert!(props.contains(&"clear"));
}

// ── 逻辑属性测试 ──

#[test]
fn test_margin_block_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-block-start", "10px"));
    assert_eq!(style.margin_top, LengthValue::Px(10.0));
}

#[test]
fn test_margin_block_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-block-end", "20px"));
    assert_eq!(style.margin_bottom, LengthValue::Px(20.0));
}

#[test]
fn test_margin_inline_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-inline-start", "5px"));
    assert_eq!(style.margin_left, LengthValue::Px(5.0));
}

#[test]
fn test_margin_inline_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-inline-end", "15px"));
    assert_eq!(style.margin_right, LengthValue::Px(15.0));
}

#[test]
fn test_padding_block_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "padding-block-start", "8px"));
    assert_eq!(style.padding_top, LengthValue::Px(8.0));
}

#[test]
fn test_padding_block_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "padding-block-end", "12px"));
    assert_eq!(style.padding_bottom, LengthValue::Px(12.0));
}

#[test]
fn test_padding_inline_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "padding-inline-start", "3px"));
    assert_eq!(style.padding_left, LengthValue::Px(3.0));
}

#[test]
fn test_padding_inline_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "padding-inline-end", "7px"));
    assert_eq!(style.padding_right, LengthValue::Px(7.0));
}

#[test]
fn test_inset_block_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "inset-block-start", "100px"));
    assert_eq!(style.top, LengthValue::Px(100.0));
}

#[test]
fn test_inset_block_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "inset-block-end", "200px"));
    assert_eq!(style.bottom, LengthValue::Px(200.0));
}

#[test]
fn test_inset_inline_start() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "inset-inline-start", "50px"));
    assert_eq!(style.left, LengthValue::Px(50.0));
}

#[test]
fn test_inset_inline_end() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "inset-inline-end", "75px"));
    assert_eq!(style.right, LengthValue::Px(75.0));
}

// ── border 逻辑属性（CSS Logical Properties §3，writing-mode-aware）──

#[test]
fn test_border_logical_inline_start_horizontal_tb() {
    // horizontal-tb（ltr）：inline-start = left
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "horizontal-tb"));
    assert!(apply_property_value(&mut style, "border-inline-start-width", "5px"));
    assert!(apply_property_value(&mut style, "border-inline-start-style", "solid"));
    assert_eq!(style.border_left_width, LengthValue::Px(5.0));
    assert_eq!(style.border_left_style, BorderStyleValue::Solid);
}

#[test]
fn test_border_logical_block_end_horizontal_tb() {
    // horizontal-tb：block-end = bottom
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-block-end-width", "3px"));
    assert_eq!(style.border_bottom_width, LengthValue::Px(3.0));
}

#[test]
fn test_border_logical_inline_start_vertical_rl() {
    // vertical-rl：inline-start = top（inline 轴垂直，ltr 方向 start=top）
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
    assert!(apply_property_value(&mut style, "border-inline-start-width", "5px"));
    assert_eq!(style.border_top_width, LengthValue::Px(5.0));
}

#[test]
fn test_border_logical_block_start_vertical_rl() {
    // vertical-rl：block-start = right（block 轴水平，rl 方向 start=right）
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
    assert!(apply_property_value(&mut style, "border-block-start-width", "5px"));
    assert_eq!(style.border_right_width, LengthValue::Px(5.0));
    let previous = style.border_right_width.clone();
    for value in [
        "10%",
        "auto",
        "-1px",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "border-block-start-width", value));
        assert_eq!(style.border_right_width, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_border_logical_block_start_vertical_lr() {
    // vertical-lr：block-start = left（block 轴水平，lr 方向 start=left）
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-lr"));
    assert!(apply_property_value(&mut style, "border-block-start-width", "5px"));
    assert_eq!(style.border_left_width, LengthValue::Px(5.0));
}

#[test]
fn test_border_logical_shorthand_inline_start_color() {
    // 简写经 shorthand 展开为 logical longhand，再 writing-mode 映射：
    // horizontal-tb + border-inline-start: 2px solid green → border-left-*
    let mut style = ComputedStyle::default();
    // 直接验证 longhand color 路径
    assert!(apply_property_value(&mut style, "border-inline-start-color", "green"));
    assert_eq!(style.border_left_color, ColorValue::Rgba(0, 128, 0, 255));
}

// ── margin/padding/inset 逻辑属性 writing-mode-aware（R1049）──
// horizontal-tb 字节同 R143 静态；vertical-rl/lr 映射到正确物理边。

#[test]
fn test_margin_logical_wm_aware_horizontal_tb() {
    // horizontal-tb：与 R143 静态字节一致
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "horizontal-tb"));
    assert!(apply_property_value(&mut style, "margin-block-start", "10px"));
    assert!(apply_property_value(&mut style, "margin-inline-end", "20px"));
    assert_eq!(style.margin_top, LengthValue::Px(10.0));
    assert_eq!(style.margin_right, LengthValue::Px(20.0));
}

#[test]
fn test_margin_logical_wm_aware_vertical_rl() {
    // vertical-rl：block-start=right, inline-end=bottom
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
    assert!(apply_property_value(&mut style, "margin-block-start", "10px"));
    assert!(apply_property_value(&mut style, "margin-inline-end", "20px"));
    assert_eq!(style.margin_right, LengthValue::Px(10.0));
    assert_eq!(style.margin_bottom, LengthValue::Px(20.0));
    let previous = style.margin_right.clone();
    for value in ["thin", "min-content", "fit-content(10px)", "infpx", "NaNpx"] {
        assert!(!apply_property_value(&mut style, "margin-block-start", value));
        assert_eq!(style.margin_right, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_padding_logical_wm_aware_vertical_lr() {
    // vertical-lr：block-start=left, inline-start=top
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-lr"));
    assert!(apply_property_value(&mut style, "padding-block-start", "8px"));
    assert!(apply_property_value(&mut style, "padding-inline-start", "5px"));
    assert_eq!(style.padding_left, LengthValue::Px(8.0));
    assert_eq!(style.padding_top, LengthValue::Px(5.0));
}

#[test]
fn test_inset_logical_wm_aware_vertical_rl() {
    // vertical-rl：block-end=left
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
    assert!(apply_property_value(&mut style, "inset-block-end", "30px"));
    assert_eq!(style.left, LengthValue::Px(30.0));
    let previous = style.left.clone();
    for value in ["thin", "min-content", "fit-content(10px)", "infpx", "NaNpx"] {
        assert!(!apply_property_value(&mut style, "inset-block-end", value));
        assert_eq!(style.left, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_logical_properties_with_percentage() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-block-start", "10%"));
    assert_eq!(style.margin_top, LengthValue::Percentage(10.0));

    assert!(apply_property_value(&mut style, "padding-inline-end", "5%"));
    assert_eq!(style.padding_right, LengthValue::Percentage(5.0));
}

#[test]
fn test_logical_properties_with_auto() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "margin-block-start", "auto"));
    assert_eq!(style.margin_top, LengthValue::Auto);
}

// ── Animation 属性测试 ──

#[test]
fn test_computed_style_default_animation() {
    let style = ComputedStyle::default();
    assert!(style.animation_name.is_empty());
    assert!(style.animation_duration.is_empty());
    assert!(style.animation_timing_function.is_empty());
    assert!(style.animation_delay.is_empty());
    assert!(style.animation_iteration_count.is_empty());
    assert!(style.animation_direction.is_empty());
    assert!(style.animation_fill_mode.is_empty());
    assert!(style.animation_play_state.is_empty());
}

#[test]
fn test_apply_animation_name() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-name", "fadeIn"));
    assert_eq!(style.animation_name, vec!["fadeIn"]);

    assert!(apply_property_value(&mut style, "animation-name", "fadeIn, slideIn"));
    assert_eq!(style.animation_name, vec!["fadeIn", "slideIn"]);
}

#[test]
fn test_apply_animation_duration() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-duration", "0.5s"));
    assert_eq!(style.animation_duration, vec![0.5]);

    assert!(apply_property_value(&mut style, "animation-duration", "0.3s, 0.6s"));
    assert_eq!(style.animation_duration, vec![0.3, 0.6]);

    assert!(apply_property_value(&mut style, "animation-duration", "200ms"));
    assert_eq!(style.animation_duration, vec![0.2]);

    assert!(!apply_property_value(&mut style, "animation-duration", "-1s"));
    assert_eq!(style.animation_duration, vec![0.2]);

    assert!(!apply_property_value(&mut style, "animation-duration", "infs"));
    assert_eq!(style.animation_duration, vec![0.2]);

    assert!(apply_property_value(&mut style, "animation-delay", "0.1s"));
    assert_eq!(style.animation_delay, vec![0.1]);
    assert!(!apply_property_value(&mut style, "animation-delay", "NaNs"));
    assert_eq!(style.animation_delay, vec![0.1]);
}

#[test]
fn test_apply_animation_timing_function() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-timing-function", "ease-in"));
    assert_eq!(style.animation_timing_function.len(), 1);

    assert!(apply_property_value(
        &mut style,
        "animation-timing-function",
        "cubic-bezier(0.0, 0.0, 1.0, 1.0)"
    ));
    assert_eq!(style.animation_timing_function.len(), 1);
}

#[test]
fn test_apply_animation_delay() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-delay", "0.2s"));
    assert_eq!(style.animation_delay, vec![0.2]);

    assert!(apply_property_value(&mut style, "animation-delay", "-1s"));
    assert_eq!(style.animation_delay, vec![-1.0]);

    assert!(!apply_property_value(&mut style, "animation-delay", "0.2s, bogus"));
    assert_eq!(style.animation_delay, vec![-1.0]);
}

#[test]
fn test_apply_animation_iteration_count() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-iteration-count", "3"));
    assert_eq!(style.animation_iteration_count, vec![Some(3.0)]);

    assert!(apply_property_value(
        &mut style,
        "animation-iteration-count",
        "infinite"
    ));
    assert_eq!(style.animation_iteration_count, vec![None]);

    assert!(apply_property_value(
        &mut style,
        "animation-iteration-count",
        "2, infinite"
    ));
    assert_eq!(style.animation_iteration_count, vec![Some(2.0), None]);

    assert!(apply_property_value(&mut style, "animation-iteration-count", "0"));
    assert_eq!(style.animation_iteration_count, vec![Some(0.0)]);

    assert!(!apply_property_value(&mut style, "animation-iteration-count", "-1"));
    assert_eq!(style.animation_iteration_count, vec![Some(0.0)]);

    assert!(!apply_property_value(&mut style, "animation-iteration-count", "inf"));
    assert_eq!(style.animation_iteration_count, vec![Some(0.0)]);
    assert!(!apply_property_value(&mut style, "animation-iteration-count", "NaN"));
    assert_eq!(style.animation_iteration_count, vec![Some(0.0)]);
}

#[test]
fn test_apply_animation_direction() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-direction", "alternate"));
    assert_eq!(style.animation_direction.len(), 1);

    assert!(apply_property_value(
        &mut style,
        "animation-direction",
        "normal, reverse"
    ));
    assert_eq!(style.animation_direction.len(), 2);

    assert!(!apply_property_value(
        &mut style,
        "animation-direction",
        "normal, bogus"
    ));
    assert_eq!(style.animation_direction.len(), 2);
}

#[test]
fn test_apply_animation_fill_mode() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-fill-mode", "forwards"));
    assert_eq!(style.animation_fill_mode.len(), 1);

    assert!(apply_property_value(&mut style, "animation-fill-mode", "both"));
    assert_eq!(style.animation_fill_mode.len(), 1);

    assert!(!apply_property_value(&mut style, "animation-fill-mode", "both, bogus"));
    assert_eq!(style.animation_fill_mode.len(), 1);
}

#[test]
fn test_apply_animation_play_state() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "animation-play-state", "paused"));
    assert_eq!(style.animation_play_state.len(), 1);

    assert!(apply_property_value(
        &mut style,
        "animation-play-state",
        "running, paused"
    ));
    assert_eq!(style.animation_play_state.len(), 2);

    assert!(!apply_property_value(
        &mut style,
        "animation-play-state",
        "running, bogus"
    ));
    assert_eq!(style.animation_play_state.len(), 2);
}

#[test]
fn test_animation_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"animation-name"));
    assert!(props.contains(&"animation-duration"));
    assert!(props.contains(&"animation-timing-function"));
    assert!(props.contains(&"animation-delay"));
    assert!(props.contains(&"animation-iteration-count"));
    assert!(props.contains(&"animation-direction"));
    assert!(props.contains(&"animation-fill-mode"));
    assert!(props.contains(&"animation-play-state"));
}

// ── grid-auto-rows/columns 属性测试 ──

#[test]
fn test_apply_property_grid_auto_rows() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-auto-rows", "100px"));
    assert_eq!(style.grid_auto_rows, Some("100px".to_string()));

    assert!(apply_property_value(&mut style, "grid-auto-rows", "minmax(100px, 1fr)"));
    assert_eq!(style.grid_auto_rows, Some("minmax(100px, 1fr)".to_string()));

    // default is None
    let style = ComputedStyle::default();
    assert_eq!(style.grid_auto_rows, None);
}

#[test]
fn test_apply_property_grid_auto_columns() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-auto-columns", "1fr auto"));
    assert_eq!(style.grid_auto_columns, Some("1fr auto".to_string()));

    // default is None
    let style = ComputedStyle::default();
    assert_eq!(style.grid_auto_columns, None);
}

#[test]
fn test_grid_auto_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"grid-auto-rows"));
    assert!(props.contains(&"grid-auto-columns"));
}

// ── Outline 属性测试 ──

#[test]
fn test_computed_style_default_outline() {
    let style = ComputedStyle::default();
    // outline-width 初始 = medium(3px)（CSS UI，与 border-width 同）；outline-style:none 抑制绘制。
    assert_eq!(style.outline_width, LengthValue::Px(3.0));
    assert_eq!(style.outline_style, OutlineStyleValue::None);
    // outline-color 初始 = currentColor（invert 无浏览器支持回落 currentColor，CSSWG #9199）。
    assert_eq!(style.outline_color, ColorValue::CurrentColor);
    assert_eq!(style.outline_offset, LengthValue::Px(0.0));
}

#[test]
fn test_apply_outline_width() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "outline-width", "2px"));
    assert_eq!(style.outline_width, LengthValue::Px(2.0));

    assert!(apply_property_value(&mut style, "outline-width", "thin"));
    assert_eq!(style.outline_width, LengthValue::Px(1.0));

    assert!(apply_property_value(&mut style, "outline-width", "0.5em"));
    assert_eq!(style.outline_width, LengthValue::Em(0.5));
    let previous = style.outline_width.clone();

    assert!(!apply_property_value(&mut style, "outline-width", "invalid"));
    for value in [
        "10%",
        "auto",
        "-1px",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "outline-width", value));
        assert_eq!(style.outline_width, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_apply_outline_style() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "outline-style", "solid"));
    assert_eq!(style.outline_style, OutlineStyleValue::Solid);

    assert!(apply_property_value(&mut style, "outline-style", "dashed"));
    assert_eq!(style.outline_style, OutlineStyleValue::Dashed);

    assert!(apply_property_value(&mut style, "outline-style", "dotted"));
    assert_eq!(style.outline_style, OutlineStyleValue::Dotted);

    assert!(apply_property_value(&mut style, "outline-style", "double"));
    assert_eq!(style.outline_style, OutlineStyleValue::Double);

    assert!(apply_property_value(&mut style, "outline-style", "none"));
    assert_eq!(style.outline_style, OutlineStyleValue::None);

    assert!(!apply_property_value(&mut style, "outline-style", "invalid"));
}

#[test]
fn test_apply_outline_color() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "outline-color", "red"));
    assert_eq!(style.outline_color, ColorValue::Rgba(255, 0, 0, 255));

    assert!(apply_property_value(&mut style, "outline-color", "#00ff00"));
    assert_eq!(style.outline_color, ColorValue::Rgba(0, 255, 0, 255));

    assert!(apply_property_value(&mut style, "outline-color", "transparent"));
    assert_eq!(style.outline_color, ColorValue::Transparent);
}

#[test]
fn test_apply_outline_offset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "outline-offset", "4px"));
    assert_eq!(style.outline_offset, LengthValue::Px(4.0));

    assert!(apply_property_value(&mut style, "outline-offset", "-2px"));
    assert_eq!(style.outline_offset, LengthValue::Px(-2.0));

    assert!(!apply_property_value(&mut style, "outline-offset", "invalid"));
    let previous = style.outline_offset.clone();
    for value in [
        "10%",
        "auto",
        "thin",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "outline-offset", value));
        assert_eq!(style.outline_offset, previous, "{} should not overwrite", value);
        assert!(!style.outline_offset_inset);
    }

    // CSS-UI-4 §4.4: `outline-offset: inset` 关键字 ≡ 负 outline-width 的偏移
    // （outline 绘制在 border-box 内侧）。driving: outline-offset-inset-001/003/004。
    assert!(!style.outline_offset_inset, "inset flag defaults to false");
    assert!(apply_property_value(&mut style, "outline-offset", "inset"));
    assert!(style.outline_offset_inset, "inset keyword sets the flag");
    // 重新赋一个长度应清除 inset 标记（长度与 inset 互斥）
    assert!(apply_property_value(&mut style, "outline-offset", "5px"));
    assert!(!style.outline_offset_inset, "length clears the inset flag");
    assert_eq!(style.outline_offset, LengthValue::Px(5.0));
}

#[test]
fn test_outline_property_registry() {
    assert!(PropertyRegistry::initial_value("outline-width").is_some());
    assert!(PropertyRegistry::initial_value("outline-style").is_some());
    assert!(PropertyRegistry::initial_value("outline-color").is_some());
    assert!(PropertyRegistry::initial_value("outline-offset").is_some());
    assert!(!PropertyRegistry::is_inherited("outline-width"));
    assert!(!PropertyRegistry::is_inherited("outline-style"));
}

#[test]
fn test_outline_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"outline-width"));
    assert!(props.contains(&"outline-style"));
    assert!(props.contains(&"outline-color"));
    assert!(props.contains(&"outline-offset"));
}

#[test]
fn test_parse_outline_style() {
    assert_eq!(parse_outline_style("solid"), Some(OutlineStyleValue::Solid));
    assert_eq!(parse_outline_style("none"), Some(OutlineStyleValue::None));
    assert_eq!(parse_outline_style("dashed"), Some(OutlineStyleValue::Dashed));
    assert_eq!(parse_outline_style("dotted"), Some(OutlineStyleValue::Dotted));
    assert_eq!(parse_outline_style("double"), Some(OutlineStyleValue::Double));
    assert_eq!(parse_outline_style("groove"), Some(OutlineStyleValue::Groove));
    assert_eq!(parse_outline_style("ridge"), Some(OutlineStyleValue::Ridge));
    assert_eq!(parse_outline_style("inset"), Some(OutlineStyleValue::Inset));
    assert_eq!(parse_outline_style("outset"), Some(OutlineStyleValue::Outset));
    // R2379：CSS UI 4 outline-style:auto（UA-defined，ZW 按 solid 渲染）。修复前 None → 声明被丢。
    assert_eq!(parse_outline_style("auto"), Some(OutlineStyleValue::Auto));
    assert_eq!(parse_outline_style("AUTO"), Some(OutlineStyleValue::Auto)); // 大小写不敏感
    assert_eq!(parse_outline_style("invalid"), None);
}

// ── Cursor 属性测试 ──

#[test]
fn test_parse_cursor_values() {
    assert_eq!(parse_cursor("auto"), Some(CursorValue::Auto));
    assert_eq!(parse_cursor("pointer"), Some(CursorValue::Pointer));
    assert_eq!(parse_cursor("move"), Some(CursorValue::Move));
    assert_eq!(parse_cursor("text"), Some(CursorValue::Text));
    assert_eq!(parse_cursor("wait"), Some(CursorValue::Wait));
    assert_eq!(parse_cursor("crosshair"), Some(CursorValue::Crosshair));
    assert_eq!(parse_cursor("help"), Some(CursorValue::Help));
    assert_eq!(parse_cursor("not-allowed"), Some(CursorValue::NotAllowed));
    assert_eq!(parse_cursor("grab"), Some(CursorValue::Grab));
    assert_eq!(parse_cursor("grabbing"), Some(CursorValue::Grabbing));
    assert_eq!(parse_cursor("col-resize"), Some(CursorValue::ColResize));
    assert_eq!(parse_cursor("row-resize"), Some(CursorValue::RowResize));
    assert_eq!(parse_cursor("ns-resize"), Some(CursorValue::NsResize));
    assert_eq!(parse_cursor("ew-resize"), Some(CursorValue::EwResize));
    assert_eq!(parse_cursor("none"), Some(CursorValue::None));
    assert_eq!(parse_cursor("progress"), Some(CursorValue::Progress));
    assert_eq!(parse_cursor("cell"), Some(CursorValue::Cell));
    assert_eq!(parse_cursor("copy"), Some(CursorValue::Copy));
    assert_eq!(parse_cursor("alias"), Some(CursorValue::Alias));
    assert_eq!(parse_cursor("all-scroll"), Some(CursorValue::AllScroll));
    assert_eq!(parse_cursor("zoom-in"), Some(CursorValue::ZoomIn));
    assert_eq!(parse_cursor("zoom-out"), Some(CursorValue::ZoomOut));
    assert_eq!(parse_cursor("default"), Some(CursorValue::Default));
    assert_eq!(parse_cursor("invalid"), None);
}

#[test]
fn test_apply_property_cursor() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.cursor, CursorValue::Auto);

    assert!(apply_property_value(&mut style, "cursor", "pointer"));
    assert_eq!(style.cursor, CursorValue::Pointer);

    assert!(apply_property_value(&mut style, "cursor", "not-allowed"));
    assert_eq!(style.cursor, CursorValue::NotAllowed);

    assert!(apply_property_value(&mut style, "cursor", "grab"));
    assert_eq!(style.cursor, CursorValue::Grab);

    assert!(!apply_property_value(&mut style, "cursor", "invalid"));
}

#[test]
fn test_cursor_default_value() {
    let style = ComputedStyle::default();
    assert_eq!(style.cursor, CursorValue::Auto);
}

#[test]
fn test_cursor_property_registry() {
    assert!(PropertyRegistry::initial_value("cursor").is_some());
    // cursor 按 CSS 规范是继承属性
    assert!(PropertyRegistry::is_inherited("cursor"));
}

#[test]
fn test_cursor_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"cursor"));
}

// ── initial_value 完整性测试 ──

#[test]
/// 交叉验证：known_properties() 中的每个属性在 initial_value() 中都应返回 Some。
fn test_initial_value_completeness() {
    let mut missing = Vec::new();
    for prop in PropertyRegistry::known_properties() {
        if PropertyRegistry::initial_value(prop).is_none() {
            missing.push(*prop);
        }
    }
    assert!(
        missing.is_empty(),
        "initial_value() returns None for known properties: {missing:?}"
    );
}

#[test]
/// 验证 initial_value 的返回值与 ComputedStyle::default() 一致（抽查）。
fn test_initial_value_matches_default() {
    use PropertyValue::*;

    // transition-timing-function 的初始值为空列表
    assert_eq!(
        PropertyRegistry::initial_value("transition-timing-function"),
        Some(TimingFunctionList(vec![]))
    );

    // animation-name 的初始值为空列表
    assert_eq!(
        PropertyRegistry::initial_value("animation-name"),
        Some(StringList(vec![]))
    );

    // grid-auto-flow 的初始值为 Row
    assert_eq!(
        PropertyRegistry::initial_value("grid-auto-flow"),
        Some(GridAutoFlow(GridAutoFlowValue::Row))
    );

    // grid-column-start 的初始值为 Auto
    assert_eq!(
        PropertyRegistry::initial_value("grid-column-start"),
        Some(GridLine(GridLineValue::Auto))
    );

    // transform 的初始值为 None
    assert_eq!(
        PropertyRegistry::initial_value("transform"),
        Some(Transform(zero_css_parser::values::TransformValue::None))
    );

    // grid-template-columns 的初始值为 None
    assert_eq!(
        PropertyRegistry::initial_value("grid-template-columns"),
        Some(OptionalString(None))
    );

    // grid-auto-rows 的初始值为 None
    assert_eq!(
        PropertyRegistry::initial_value("grid-auto-rows"),
        Some(OptionalString(None))
    );
}

// ═══════════════════════════════════════════════════════════════════
// Scroll Snap 和 Container Query 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_scroll_snap_type_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.scroll_snap_type.strictness, ScrollSnapStrictness::None);
    assert_eq!(style.scroll_snap_type.axis, ScrollSnapAxis::Both);
}

#[test]
fn test_scroll_snap_type_variants() {
    let mut style = ComputedStyle::default();

    assert!(apply_property_value(&mut style, "scroll-snap-type", "mandatory y"));
    assert_eq!(style.scroll_snap_type.strictness, ScrollSnapStrictness::Mandatory);
    assert_eq!(style.scroll_snap_type.axis, ScrollSnapAxis::Y);

    assert!(apply_property_value(&mut style, "scroll-snap-type", "proximity x"));
    assert_eq!(style.scroll_snap_type.strictness, ScrollSnapStrictness::Proximity);
    assert_eq!(style.scroll_snap_type.axis, ScrollSnapAxis::X);

    assert!(apply_property_value(&mut style, "scroll-snap-type", "none"));
    assert_eq!(style.scroll_snap_type.strictness, ScrollSnapStrictness::None);

    assert!(!apply_property_value(&mut style, "scroll-snap-type", "invalid"));
}

#[test]
fn test_scroll_snap_align_default_and_variants() {
    let style = ComputedStyle::default();
    assert_eq!(style.scroll_snap_align, ScrollSnapAlign::None);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scroll-snap-align", "start"));
    assert_eq!(style.scroll_snap_align, ScrollSnapAlign::Start);

    assert!(apply_property_value(&mut style, "scroll-snap-align", "end"));
    assert_eq!(style.scroll_snap_align, ScrollSnapAlign::End);

    assert!(apply_property_value(&mut style, "scroll-snap-align", "center"));
    assert_eq!(style.scroll_snap_align, ScrollSnapAlign::Center);

    assert!(!apply_property_value(&mut style, "scroll-snap-align", "invalid"));
}

#[test]
fn test_scroll_snap_stop_default_and_variants() {
    let style = ComputedStyle::default();
    assert_eq!(style.scroll_snap_stop, ScrollSnapStop::Normal);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scroll-snap-stop", "always"));
    assert_eq!(style.scroll_snap_stop, ScrollSnapStop::Always);

    assert!(apply_property_value(&mut style, "scroll-snap-stop", "normal"));
    assert_eq!(style.scroll_snap_stop, ScrollSnapStop::Normal);

    assert!(!apply_property_value(&mut style, "scroll-snap-stop", "invalid"));
}

#[test]
fn test_scroll_margin_defaults() {
    let style = ComputedStyle::default();
    assert_eq!(style.scroll_margin_top, 0.0);
    assert_eq!(style.scroll_margin_right, 0.0);
    assert_eq!(style.scroll_margin_bottom, 0.0);
    assert_eq!(style.scroll_margin_left, 0.0);
}

#[test]
fn test_scroll_margin_applied() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scroll-margin-top", "10px"));
    assert_eq!(style.scroll_margin_top, 10.0);

    assert!(apply_property_value(&mut style, "scroll-margin-right", "20px"));
    assert_eq!(style.scroll_margin_right, 20.0);

    assert!(apply_property_value(&mut style, "scroll-margin-bottom", "5px"));
    assert_eq!(style.scroll_margin_bottom, 5.0);

    assert!(apply_property_value(&mut style, "scroll-margin-left", "15px"));
    assert_eq!(style.scroll_margin_left, 15.0);

    assert!(apply_property_value(&mut style, "scroll-margin-top", "-4px"));
    assert_eq!(style.scroll_margin_top, -4.0);

    assert!(apply_property_value(&mut style, "scroll-margin-right", "1em"));
    assert_eq!(style.scroll_margin_right, 16.0);

    let previous = style.scroll_margin_right;
    for value in [
        "10%",
        "auto",
        "thin",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "scroll-margin-right", value));
        assert_eq!(style.scroll_margin_right, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_scroll_padding_defaults() {
    let style = ComputedStyle::default();
    assert_eq!(style.scroll_padding_top, ScrollPadding::Auto);
    assert_eq!(style.scroll_padding_right, ScrollPadding::Auto);
    assert_eq!(style.scroll_padding_bottom, ScrollPadding::Auto);
    assert_eq!(style.scroll_padding_left, ScrollPadding::Auto);
}

#[test]
fn test_scroll_padding_applied() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scroll-padding-top", "10px"));
    assert_eq!(style.scroll_padding_top, ScrollPadding::Length(10.0));

    assert!(apply_property_value(&mut style, "scroll-padding-top", "25%"));
    assert_eq!(style.scroll_padding_top, ScrollPadding::Length(0.0));

    assert!(apply_property_value(&mut style, "scroll-padding-top", "1em"));
    assert_eq!(style.scroll_padding_top, ScrollPadding::Length(16.0));

    assert!(apply_property_value(&mut style, "scroll-padding-right", "auto"));
    assert_eq!(style.scroll_padding_right, ScrollPadding::Auto);

    assert!(apply_property_value(&mut style, "scroll-padding-bottom", "5px"));
    assert_eq!(style.scroll_padding_bottom, ScrollPadding::Length(5.0));

    assert!(apply_property_value(&mut style, "scroll-padding-left", "0px"));
    assert_eq!(style.scroll_padding_left, ScrollPadding::Length(0.0));

    let previous = style.scroll_padding_bottom.clone();
    for value in [
        "-1px",
        "-5%",
        "thin",
        "min-content",
        "max-content",
        "fit-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "scroll-padding-bottom", value));
        assert_eq!(style.scroll_padding_bottom, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_container_type_default_and_variants() {
    let style = ComputedStyle::default();
    assert_eq!(style.container_type, ContainerType::Normal);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "container-type", "size"));
    assert_eq!(style.container_type, ContainerType::Size);

    assert!(apply_property_value(&mut style, "container-type", "inline-size"));
    assert_eq!(style.container_type, ContainerType::InlineSize);

    assert!(apply_property_value(&mut style, "container-type", "normal"));
    assert_eq!(style.container_type, ContainerType::Normal);

    assert!(!apply_property_value(&mut style, "container-type", "invalid"));
}

#[test]
fn test_container_name_default_and_applied() {
    let style = ComputedStyle::default();
    assert_eq!(style.container_name, None);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "container-name", "sidebar"));
    assert_eq!(style.container_name, Some("sidebar".to_string()));

    assert!(apply_property_value(&mut style, "container-name", "none"));
    assert_eq!(style.container_name, None);

    assert!(apply_property_value(&mut style, "container-name", "my-container"));
    assert_eq!(style.container_name, Some("my-container".to_string()));
}

#[test]
fn test_computed_style_new_fields_present() {
    let style = ComputedStyle::default();
    // 验证所有新字段都存在且可访问
    let _ = &style.scroll_snap_type;
    let _ = &style.scroll_snap_align;
    let _ = &style.scroll_snap_stop;
    let _ = &style.scroll_margin_top;
    let _ = &style.scroll_margin_right;
    let _ = &style.scroll_margin_bottom;
    let _ = &style.scroll_margin_left;
    let _ = &style.scroll_padding_top;
    let _ = &style.scroll_padding_right;
    let _ = &style.scroll_padding_bottom;
    let _ = &style.scroll_padding_left;
    let _ = &style.container_type;
    let _ = &style.container_name;
}

#[test]
fn test_scroll_snap_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("scroll-snap-type"));
    assert!(!PropertyRegistry::is_inherited("scroll-snap-align"));
    assert!(!PropertyRegistry::is_inherited("scroll-snap-stop"));
    assert!(!PropertyRegistry::is_inherited("scroll-margin-top"));
    assert!(!PropertyRegistry::is_inherited("scroll-padding-top"));
}

#[test]
fn test_container_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("container-type"));
    assert!(!PropertyRegistry::is_inherited("container-name"));
}

#[test]
fn test_scroll_and_container_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"scroll-snap-type"));
    assert!(props.contains(&"scroll-snap-align"));
    assert!(props.contains(&"scroll-snap-stop"));
    assert!(props.contains(&"scroll-margin-top"));
    assert!(props.contains(&"scroll-margin-right"));
    assert!(props.contains(&"scroll-margin-bottom"));
    assert!(props.contains(&"scroll-margin-left"));
    assert!(props.contains(&"scroll-padding-top"));
    assert!(props.contains(&"scroll-padding-right"));
    assert!(props.contains(&"scroll-padding-bottom"));
    assert!(props.contains(&"scroll-padding-left"));
    assert!(props.contains(&"container-type"));
    assert!(props.contains(&"container-name"));
}

#[test]
fn test_scroll_and_container_initial_values() {
    assert!(PropertyRegistry::initial_value("scroll-snap-type").is_some());
    assert!(PropertyRegistry::initial_value("scroll-snap-align").is_some());
    assert!(PropertyRegistry::initial_value("scroll-snap-stop").is_some());
    assert!(PropertyRegistry::initial_value("scroll-margin-top").is_some());
    assert!(PropertyRegistry::initial_value("scroll-padding-top").is_some());
    assert!(PropertyRegistry::initial_value("container-type").is_some());
    assert!(PropertyRegistry::initial_value("container-name").is_some());
}

#[test]
fn test_apply_initial_value_scroll_and_container() {
    let mut style = ComputedStyle::default();
    // 修改 scroll-snap-type
    apply_property_value(&mut style, "scroll-snap-type", "mandatory y");
    assert!(apply_initial_value(&mut style, "scroll-snap-type"));
    assert_eq!(style.scroll_snap_type.strictness, ScrollSnapStrictness::None);

    // 修改 container-type
    apply_property_value(&mut style, "container-type", "size");
    assert!(apply_initial_value(&mut style, "container-type"));
    assert_eq!(style.container_type, ContainerType::Normal);

    // 修改 container-name
    apply_property_value(&mut style, "container-name", "test");
    assert!(apply_initial_value(&mut style, "container-name"));
    assert_eq!(style.container_name, None);

    // 修改 scroll-margin
    apply_property_value(&mut style, "scroll-margin-top", "10px");
    assert!(apply_initial_value(&mut style, "scroll-margin-top"));
    assert_eq!(style.scroll_margin_top, 0.0);

    // 修改 scroll-padding
    apply_property_value(&mut style, "scroll-padding-top", "10px");
    assert!(apply_initial_value(&mut style, "scroll-padding-top"));
    assert_eq!(style.scroll_padding_top, ScrollPadding::Auto);
}

// ── list-style 属性测试 ──
