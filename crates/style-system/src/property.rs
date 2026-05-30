//! CSS 属性定义和计算样式结构。
//!
//! 定义 `ComputedStyle` 结构体，包含所有 Tier 1 CSS 属性的 typed 字段，
//! 以及 `PropertyRegistry` 用于查询初始值和继承性。

use zero_css_parser::values::{
    self,
    AlignmentValue, BoxSizingValue, ColorValue, DisplayValue, FlexDirectionValue, FlexWrapValue,
    FontStyleValue, FontWeightValue, LengthValue, OverflowValue, PositionValue, VisibilityValue,
};

// ── 额外枚举类型 ─────────────────────────────────────────────────────

/// CSS border-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyleValue {
    /// none。
    None,
    /// hidden。
    Hidden,
    /// dotted。
    Dotted,
    /// dashed。
    Dashed,
    /// solid。
    Solid,
    /// double。
    Double,
    /// groove。
    Groove,
    /// ridge。
    Ridge,
    /// inset。
    Inset,
    /// outset。
    Outset,
}

/// CSS line-height 值。
#[derive(Debug, Clone, PartialEq)]
pub enum LineHeightValue {
    /// normal。
    Normal,
    /// 无单位数值。
    Number(f64),
    /// 长度值。
    Length(LengthValue),
}

/// CSS text-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignValue {
    /// left。
    Left,
    /// right。
    Right,
    /// center。
    Center,
    /// justify。
    Justify,
    /// start。
    Start,
    /// end。
    End,
}

/// CSS text-decoration 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecorationValue {
    /// none。
    None,
    /// underline。
    Underline,
    /// overline。
    Overline,
    /// line-through。
    LineThrough,
}

/// CSS text-transform 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextTransformValue {
    /// none。
    None,
    /// uppercase。
    Uppercase,
    /// lowercase。
    Lowercase,
    /// capitalize。
    Capitalize,
}

/// CSS white-space 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WhiteSpaceValue {
    /// normal。
    Normal,
    /// pre。
    Pre,
    /// nowrap。
    Nowrap,
    /// pre-wrap。
    PreWrap,
    /// pre-line。
    PreLine,
}

/// CSS text-overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextOverflowValue {
    /// clip。
    Clip,
    /// ellipsis。
    Ellipsis,
}

/// CSS flex-basis 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FlexBasisValue {
    /// auto。
    Auto,
    /// content。
    Content,
    /// 长度值。
    Length(LengthValue),
}

/// CSS z-index 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ZIndexValue {
    /// auto。
    Auto,
    /// 整数值。
    Integer(i32),
}

/// CSS grid-auto-flow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum GridAutoFlowValue {
    /// row（默认）。
    Row,
    /// column。
    Column,
    /// dense。
    RowDense,
    /// column dense。
    ColumnDense,
}

/// 属性值枚举，用于 PropertyRegistry 返回的初始值。
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// 长度值。
    Length(LengthValue),
    /// 颜色值。
    Color(ColorValue),
    /// display 值。
    Display(DisplayValue),
    /// position 值。
    Position(PositionValue),
    /// overflow 值。
    Overflow(OverflowValue),
    /// flex-direction 值。
    FlexDirection(FlexDirectionValue),
    /// flex-wrap 值。
    FlexWrap(FlexWrapValue),
    /// 对齐值。
    Alignment(AlignmentValue),
    /// box-sizing 值。
    BoxSizing(BoxSizingValue),
    /// visibility 值。
    Visibility(VisibilityValue),
    /// font-weight 值。
    FontWeight(FontWeightValue),
    /// font-style 值。
    FontStyle(FontStyleValue),
    /// border-style 值。
    BorderStyle(BorderStyleValue),
    /// line-height 值。
    LineHeight(LineHeightValue),
    /// text-align 值。
    TextAlign(TextAlignValue),
    /// text-decoration 值。
    TextDecoration(TextDecorationValue),
    /// text-transform 值。
    TextTransform(TextTransformValue),
    /// white-space 值。
    WhiteSpace(WhiteSpaceValue),
    /// text-overflow 值。
    TextOverflow(TextOverflowValue),
    /// flex-basis 值。
    FlexBasis(FlexBasisValue),
    /// z-index 值。
    ZIndex(ZIndexValue),
    /// 数值（opacity, flex-grow, flex-shrink）。
    Number(f64),
    /// 整数（order）。
    Integer(i32),
    /// 字符串列表（font-family）。
    StringList(Vec<String>),
}

// ── ComputedStyle ─────────────────────────────────────────────────────

/// 计算样式结构体，包含所有 Tier 1 CSS 属性。
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    // ── 盒模型 ──
    /// display 属性。
    pub display: DisplayValue,
    /// position 属性。
    pub position: PositionValue,
    /// width 属性。
    pub width: LengthValue,
    /// height 属性。
    pub height: LengthValue,
    /// min-width 属性。
    pub min_width: LengthValue,
    /// min-height 属性。
    pub min_height: LengthValue,
    /// max-width 属性。
    pub max_width: LengthValue,
    /// max-height 属性。
    pub max_height: LengthValue,
    /// margin-top 属性。
    pub margin_top: LengthValue,
    /// margin-right 属性。
    pub margin_right: LengthValue,
    /// margin-bottom 属性。
    pub margin_bottom: LengthValue,
    /// margin-left 属性。
    pub margin_left: LengthValue,
    /// padding-top 属性。
    pub padding_top: LengthValue,
    /// padding-right 属性。
    pub padding_right: LengthValue,
    /// padding-bottom 属性。
    pub padding_bottom: LengthValue,
    /// padding-left 属性。
    pub padding_left: LengthValue,
    /// box-sizing 属性。
    pub box_sizing: BoxSizingValue,

    // ── 边框 ──
    /// border-top-width 属性。
    pub border_top_width: LengthValue,
    /// border-right-width 属性。
    pub border_right_width: LengthValue,
    /// border-bottom-width 属性。
    pub border_bottom_width: LengthValue,
    /// border-left-width 属性。
    pub border_left_width: LengthValue,
    /// border-top-color 属性。
    pub border_top_color: ColorValue,
    /// border-right-color 属性。
    pub border_right_color: ColorValue,
    /// border-bottom-color 属性。
    pub border_bottom_color: ColorValue,
    /// border-left-color 属性。
    pub border_left_color: ColorValue,
    /// border-top-style 属性。
    pub border_top_style: BorderStyleValue,
    /// border-right-style 属性。
    pub border_right_style: BorderStyleValue,
    /// border-bottom-style 属性。
    pub border_bottom_style: BorderStyleValue,
    /// border-left-style 属性。
    pub border_left_style: BorderStyleValue,
    /// border-top-left-radius 属性。
    pub border_top_left_radius: LengthValue,
    /// border-top-right-radius 属性。
    pub border_top_right_radius: LengthValue,
    /// border-bottom-right-radius 属性。
    pub border_bottom_right_radius: LengthValue,
    /// border-bottom-left-radius 属性。
    pub border_bottom_left_radius: LengthValue,

    // ── 颜色和背景 ──
    /// color 属性（前景色）。
    pub color: ColorValue,
    /// background-color 属性。
    pub background_color: ColorValue,
    /// opacity 属性。
    pub opacity: f64,
    /// visibility 属性。
    pub visibility: VisibilityValue,

    // ── 字体 ──
    /// font-family 属性。
    pub font_family: Vec<String>,
    /// font-size 属性。
    pub font_size: LengthValue,
    /// font-weight 属性。
    pub font_weight: FontWeightValue,
    /// font-style 属性。
    pub font_style: FontStyleValue,
    /// line-height 属性。
    pub line_height: LineHeightValue,

    // ── 文本 ──
    /// text-align 属性。
    pub text_align: TextAlignValue,
    /// text-decoration 属性。
    pub text_decoration: TextDecorationValue,
    /// text-transform 属性。
    pub text_transform: TextTransformValue,
    /// letter-spacing 属性。
    pub letter_spacing: LengthValue,
    /// word-spacing 属性。
    pub word_spacing: LengthValue,
    /// white-space 属性。
    pub white_space: WhiteSpaceValue,
    /// text-overflow 属性。
    pub text_overflow: TextOverflowValue,

    // ── Flexbox ──
    /// flex-direction 属性。
    pub flex_direction: FlexDirectionValue,
    /// flex-wrap 属性。
    pub flex_wrap: FlexWrapValue,
    /// justify-content 属性。
    pub justify_content: AlignmentValue,
    /// align-items 属性。
    pub align_items: AlignmentValue,
    /// align-self 属性。
    pub align_self: AlignmentValue,
    /// flex-grow 属性。
    pub flex_grow: f64,
    /// flex-shrink 属性。
    pub flex_shrink: f64,
    /// flex-basis 属性。
    pub flex_basis: FlexBasisValue,
    /// gap 属性。
    pub gap: LengthValue,
    /// row-gap 属性。
    pub row_gap: LengthValue,
    /// order 属性。
    pub order: i32,

    // ── Grid ──
    /// grid-template-columns 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_template_columns: Option<String>,
    /// grid-template-rows 属性。
    pub grid_template_rows: Option<String>,
    /// grid-auto-flow 属性。
    pub grid_auto_flow: GridAutoFlowValue,

    // ── 定位 ──
    /// top 属性。
    pub top: LengthValue,
    /// right 属性。
    pub right: LengthValue,
    /// bottom 属性。
    pub bottom: LengthValue,
    /// left 属性。
    pub left: LengthValue,
    /// z-index 属性。
    pub z_index: ZIndexValue,

    // ── Overflow ──
    /// overflow-x 属性。
    pub overflow_x: OverflowValue,
    /// overflow-y 属性。
    pub overflow_y: OverflowValue,

    // ── Transforms ──
    /// transform 属性。
    pub transform: zero_css_parser::values::TransformValue,

    // ── Transitions ──
    /// transition-property 属性（逗号分隔的属性名列表）。
    pub transition_property: Vec<String>,
    /// transition-duration 属性（逗号分隔的秒数列表）。
    pub transition_duration: Vec<f64>,
    /// transition-timing-function 属性（逗号分隔的时间函数列表）。
    pub transition_timing_function: Vec<zero_css_parser::values::TimingFunctionValue>,
    /// transition-delay 属性（逗号分隔的秒数列表）。
    pub transition_delay: Vec<f64>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        let zero = LengthValue::Px(0.0);
        let auto_length = LengthValue::Auto;
        let initial_color = ColorValue::Rgba(0, 0, 0, 255); // black
        let transparent = ColorValue::Transparent;

        Self {
            // 盒模型
            display: DisplayValue::Inline,
            position: PositionValue::Static,
            width: auto_length.clone(),
            height: auto_length.clone(),
            min_width: LengthValue::Px(0.0),
            min_height: LengthValue::Px(0.0),
            max_width: LengthValue::Px(f64::INFINITY),
            max_height: LengthValue::Px(f64::INFINITY),
            margin_top: LengthValue::Px(0.0),
            margin_right: LengthValue::Px(0.0),
            margin_bottom: LengthValue::Px(0.0),
            margin_left: LengthValue::Px(0.0),
            padding_top: LengthValue::Px(0.0),
            padding_right: LengthValue::Px(0.0),
            padding_bottom: LengthValue::Px(0.0),
            padding_left: LengthValue::Px(0.0),
            box_sizing: BoxSizingValue::ContentBox,

            // 边框
            border_top_width: LengthValue::Px(0.0),
            border_right_width: LengthValue::Px(0.0),
            border_bottom_width: LengthValue::Px(0.0),
            border_left_width: LengthValue::Px(0.0),
            border_top_color: initial_color.clone(),
            border_right_color: initial_color.clone(),
            border_bottom_color: initial_color.clone(),
            border_left_color: initial_color.clone(),
            border_top_style: BorderStyleValue::None,
            border_right_style: BorderStyleValue::None,
            border_bottom_style: BorderStyleValue::None,
            border_left_style: BorderStyleValue::None,
            border_top_left_radius: LengthValue::Px(0.0),
            border_top_right_radius: LengthValue::Px(0.0),
            border_bottom_right_radius: LengthValue::Px(0.0),
            border_bottom_left_radius: LengthValue::Px(0.0),

            // 颜色和背景
            color: initial_color.clone(),
            background_color: transparent,
            opacity: 1.0,
            visibility: VisibilityValue::Visible,

            // 字体
            font_family: vec![],
            font_size: LengthValue::Px(16.0),
            font_weight: FontWeightValue::Normal,
            font_style: FontStyleValue::Normal,
            line_height: LineHeightValue::Normal,

            // 文本
            text_align: TextAlignValue::Start,
            text_decoration: TextDecorationValue::None,
            text_transform: TextTransformValue::None,
            letter_spacing: LengthValue::Px(0.0),
            word_spacing: LengthValue::Px(0.0),
            white_space: WhiteSpaceValue::Normal,
            text_overflow: TextOverflowValue::Clip,

            // Flexbox
            flex_direction: FlexDirectionValue::Row,
            flex_wrap: FlexWrapValue::Nowrap,
            justify_content: AlignmentValue::FlexStart,
            align_items: AlignmentValue::Stretch,
            align_self: AlignmentValue::Stretch,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: FlexBasisValue::Auto,
            gap: LengthValue::Px(0.0),
            row_gap: LengthValue::Px(0.0),
            order: 0,

            // Grid
            grid_template_columns: None,
            grid_template_rows: None,
            grid_auto_flow: GridAutoFlowValue::Row,

            // 定位
            top: zero.clone(),
            right: zero.clone(),
            bottom: zero.clone(),
            left: zero,
            z_index: ZIndexValue::Auto,

            // Overflow
            overflow_x: OverflowValue::Visible,
            overflow_y: OverflowValue::Visible,

            // Transforms
            transform: zero_css_parser::values::TransformValue::None,

            // Transitions
            transition_property: vec![],
            transition_duration: vec![],
            transition_timing_function: vec![],
            transition_delay: vec![],
        }
    }
}

// ── PropertyRegistry ──────────────────────────────────────────────────

/// 属性注册表，提供初始值和继承性查询。
///
/// 使用 match 语句实现属性名到初始值和继承性的映射。
pub struct PropertyRegistry;

impl PropertyRegistry {
    /// 获取指定属性的初始值。
    ///
    /// 返回 `None` 表示未知属性。
    pub fn initial_value(property: &str) -> Option<PropertyValue> {
        use PropertyValue::*;
        match property {
            // 盒模型
            "display" => Some(Display(DisplayValue::Inline)),
            "position" => Some(Position(PositionValue::Static)),
            "width" | "height" => Some(Length(LengthValue::Px(0.0))),
            "min-width" | "min-height" => Some(Length(LengthValue::Px(0.0))),
            "max-width" | "max-height" => Some(Length(LengthValue::Px(f64::INFINITY))),
            "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
                Some(Length(LengthValue::Px(0.0)))
            }
            "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
                Some(Length(LengthValue::Px(0.0)))
            }
            "box-sizing" => Some(BoxSizing(BoxSizingValue::ContentBox)),

            // 边框
            "border-top-width"
            | "border-right-width"
            | "border-bottom-width"
            | "border-left-width" => Some(Length(LengthValue::Px(0.0))),
            "border-top-color"
            | "border-right-color"
            | "border-bottom-color"
            | "border-left-color" => Some(Color(ColorValue::Rgba(0, 0, 0, 255))),
            "border-top-style"
            | "border-right-style"
            | "border-bottom-style"
            | "border-left-style" => Some(BorderStyle(BorderStyleValue::None)),
            "border-top-left-radius"
            | "border-top-right-radius"
            | "border-bottom-right-radius"
            | "border-bottom-left-radius" => Some(Length(LengthValue::Px(0.0))),

            // 颜色和背景
            "color" => Some(Color(ColorValue::Rgba(0, 0, 0, 255))),
            "background-color" => Some(Color(ColorValue::Transparent)),
            "opacity" => Some(Number(1.0)),
            "visibility" => Some(Visibility(VisibilityValue::Visible)),

            // 字体
            "font-family" => Some(StringList(vec![])),
            "font-size" => Some(Length(LengthValue::Px(16.0))),
            "font-weight" => Some(FontWeight(FontWeightValue::Normal)),
            "font-style" => Some(FontStyle(FontStyleValue::Normal)),
            "line-height" => Some(LineHeight(LineHeightValue::Normal)),

            // 文本
            "text-align" => Some(TextAlign(TextAlignValue::Start)),
            "text-decoration" => Some(TextDecoration(TextDecorationValue::None)),
            "text-transform" => Some(TextTransform(TextTransformValue::None)),
            "letter-spacing" | "word-spacing" => Some(Length(LengthValue::Px(0.0))),
            "white-space" => Some(WhiteSpace(WhiteSpaceValue::Normal)),
            "text-overflow" => Some(TextOverflow(TextOverflowValue::Clip)),

            // Flexbox
            "flex-direction" => Some(FlexDirection(FlexDirectionValue::Row)),
            "flex-wrap" => Some(FlexWrap(FlexWrapValue::Nowrap)),
            "justify-content" => Some(Alignment(AlignmentValue::FlexStart)),
            "align-items" | "align-self" => Some(Alignment(AlignmentValue::Stretch)),
            "flex-grow" => Some(Number(0.0)),
            "flex-shrink" => Some(Number(1.0)),
            "flex-basis" => Some(FlexBasis(FlexBasisValue::Auto)),
            "gap" => Some(Length(LengthValue::Px(0.0))),
            "order" => Some(Integer(0)),

            // 定位
            "top" | "right" | "bottom" | "left" => Some(Length(LengthValue::Px(0.0))),
            "z-index" => Some(ZIndex(ZIndexValue::Auto)),

            // Overflow
            "overflow-x" | "overflow-y" => Some(Overflow(OverflowValue::Visible)),

            // Transitions
            "transition-property" => Some(StringList(vec![])),
            "transition-duration" | "transition-delay" => Some(Number(0.0)),

            _ => None,
        }
    }

    /// 查询指定属性是否为继承属性。
    ///
    /// 继承属性在没有显式值时会从父元素继承。
    pub fn is_inherited(property: &str) -> bool {
        matches!(
            property,
            "color"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "text-align"
                | "text-decoration"
                | "text-transform"
                | "letter-spacing"
                | "word-spacing"
                | "white-space"
                | "text-overflow"
                | "visibility"
                | "opacity"
        )
    }

    /// 获取所有已知属性名的列表。
    pub fn known_properties() -> &'static [&'static str] {
        &[
            "display",
            "position",
            "width",
            "height",
            "min-width",
            "min-height",
            "max-width",
            "max-height",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
            "box-sizing",
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
            "color",
            "background-color",
            "opacity",
            "visibility",
            "font-family",
            "font-size",
            "font-weight",
            "font-style",
            "line-height",
            "text-align",
            "text-decoration",
            "text-transform",
            "letter-spacing",
            "word-spacing",
            "white-space",
            "text-overflow",
            "flex-direction",
            "flex-wrap",
            "justify-content",
            "align-items",
            "align-self",
            "flex-grow",
            "flex-shrink",
            "flex-basis",
            "gap",
            "order",
            "top",
            "right",
            "bottom",
            "left",
            "z-index",
            "overflow-x",
            "overflow-y",
            "transition-property",
            "transition-duration",
            "transition-timing-function",
            "transition-delay",
        ]
    }
}

/// 解析 CSS border-style 值。
pub fn parse_border_style(value: &str) -> Option<BorderStyleValue> {
    match value.trim() {
        "none" => Some(BorderStyleValue::None),
        "hidden" => Some(BorderStyleValue::Hidden),
        "dotted" => Some(BorderStyleValue::Dotted),
        "dashed" => Some(BorderStyleValue::Dashed),
        "solid" => Some(BorderStyleValue::Solid),
        "double" => Some(BorderStyleValue::Double),
        "groove" => Some(BorderStyleValue::Groove),
        "ridge" => Some(BorderStyleValue::Ridge),
        "inset" => Some(BorderStyleValue::Inset),
        "outset" => Some(BorderStyleValue::Outset),
        _ => None,
    }
}

/// 解析 CSS grid-auto-flow 值。
pub fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlowValue> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "row" => Some(GridAutoFlowValue::Row),
        "column" => Some(GridAutoFlowValue::Column),
        "dense" | "row dense" => Some(GridAutoFlowValue::RowDense),
        "column dense" => Some(GridAutoFlowValue::ColumnDense),
        _ => None,
    }
}

/// 解析逗号分隔的 transition-timing-function 列表。
///
/// 需要处理 cubic-bezier() 和 steps() 内部的逗号。
fn parse_comma_separated_timing_functions(
    value: &str,
) -> Vec<zero_css_parser::values::TimingFunctionValue> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let part = value[start..i].trim();
                if let Some(tf) = values::parse_timing_function(part) {
                    result.push(tf);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // 处理最后一个
    let last = value[start..].trim();
    if let Some(tf) = values::parse_timing_function(last) {
        result.push(tf);
    }

    result
}

/// 解析 CSS line-height 值。
pub fn parse_line_height(value: &str) -> Option<LineHeightValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some(LineHeightValue::Normal);
    }
    // 尝试解析为无单位数值
    if let Ok(num) = value.parse::<f64>() {
        // 如果值不含单位后缀，视为无单位数值
        if !value.contains("px")
            && !value.contains("em")
            && !value.contains("rem")
            && !value.contains("%")
            && !value.contains("vh")
            && !value.contains("vw")
        {
            return Some(LineHeightValue::Number(num));
        }
    }
    // 尝试解析为长度
    if let Some(length) = values::parse_length(value) {
        return Some(LineHeightValue::Length(length));
    }
    None
}

/// 解析 CSS text-align 值。
pub fn parse_text_align(value: &str) -> Option<TextAlignValue> {
    match value.trim() {
        "left" => Some(TextAlignValue::Left),
        "right" => Some(TextAlignValue::Right),
        "center" => Some(TextAlignValue::Center),
        "justify" => Some(TextAlignValue::Justify),
        "start" => Some(TextAlignValue::Start),
        "end" => Some(TextAlignValue::End),
        _ => None,
    }
}

/// 解析 CSS text-decoration 值。
pub fn parse_text_decoration(value: &str) -> Option<TextDecorationValue> {
    match value.trim() {
        "none" => Some(TextDecorationValue::None),
        "underline" => Some(TextDecorationValue::Underline),
        "overline" => Some(TextDecorationValue::Overline),
        "line-through" => Some(TextDecorationValue::LineThrough),
        _ => None,
    }
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.trim() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS white-space 值。
pub fn parse_white_space(value: &str) -> Option<WhiteSpaceValue> {
    match value.trim() {
        "normal" => Some(WhiteSpaceValue::Normal),
        "pre" => Some(WhiteSpaceValue::Pre),
        "nowrap" => Some(WhiteSpaceValue::Nowrap),
        "pre-wrap" => Some(WhiteSpaceValue::PreWrap),
        "pre-line" => Some(WhiteSpaceValue::PreLine),
        _ => None,
    }
}

/// 解析 CSS text-overflow 值。
pub fn parse_text_overflow(value: &str) -> Option<TextOverflowValue> {
    match value.trim() {
        "clip" => Some(TextOverflowValue::Clip),
        "ellipsis" => Some(TextOverflowValue::Ellipsis),
        _ => None,
    }
}

/// 解析 CSS flex-basis 值。
pub fn parse_flex_basis(value: &str) -> Option<FlexBasisValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(FlexBasisValue::Auto);
    }
    if value.eq_ignore_ascii_case("content") {
        return Some(FlexBasisValue::Content);
    }
    if let Some(length) = values::parse_length(value) {
        return Some(FlexBasisValue::Length(length));
    }
    None
}

/// 解析 CSS z-index 值。
pub fn parse_z_index(value: &str) -> Option<ZIndexValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ZIndexValue::Auto);
    }
    let int: i32 = value.parse().ok()?;
    Some(ZIndexValue::Integer(int))
}

/// 解析 font-family 值。
///
/// 简单实现：按逗号分割，去除引号和空格。
pub fn parse_font_family(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| {
            s.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// 将属性字符串值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
pub fn apply_property_value(style: &mut ComputedStyle, property: &str, value: &str) -> bool {
    let value = value.trim();
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
        "width" => {
            if let Some(v) = values::parse_length(value) {
                style.width = v;
                return true;
            }
        }
        "height" => {
            if let Some(v) = values::parse_length(value) {
                style.height = v;
                return true;
            }
        }
        "min-width" => {
            if let Some(v) = values::parse_length(value) {
                style.min_width = v;
                return true;
            }
        }
        "min-height" => {
            if let Some(v) = values::parse_length(value) {
                style.min_height = v;
                return true;
            }
        }
        "max-width" => {
            if value == "none" {
                style.max_width = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = values::parse_length(value) {
                style.max_width = v;
                return true;
            }
        }
        "max-height" => {
            if value == "none" {
                style.max_height = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = values::parse_length(value) {
                style.max_height = v;
                return true;
            }
        }
        "margin-top" => {
            if let Some(v) = values::parse_length(value) {
                style.margin_top = v;
                return true;
            }
        }
        "margin-right" => {
            if let Some(v) = values::parse_length(value) {
                style.margin_right = v;
                return true;
            }
        }
        "margin-bottom" => {
            if let Some(v) = values::parse_length(value) {
                style.margin_bottom = v;
                return true;
            }
        }
        "margin-left" => {
            if let Some(v) = values::parse_length(value) {
                style.margin_left = v;
                return true;
            }
        }
        "padding-top" => {
            if let Some(v) = values::parse_length(value) {
                style.padding_top = v;
                return true;
            }
        }
        "padding-right" => {
            if let Some(v) = values::parse_length(value) {
                style.padding_right = v;
                return true;
            }
        }
        "padding-bottom" => {
            if let Some(v) = values::parse_length(value) {
                style.padding_bottom = v;
                return true;
            }
        }
        "padding-left" => {
            if let Some(v) = values::parse_length(value) {
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
            if let Some(v) = values::parse_length(value) {
                style.border_top_width = v;
                return true;
            }
        }
        "border-right-width" => {
            if let Some(v) = values::parse_length(value) {
                style.border_right_width = v;
                return true;
            }
        }
        "border-bottom-width" => {
            if let Some(v) = values::parse_length(value) {
                style.border_bottom_width = v;
                return true;
            }
        }
        "border-left-width" => {
            if let Some(v) = values::parse_length(value) {
                style.border_left_width = v;
                return true;
            }
        }
        "border-top-color" => {
            if let Some(v) = values::parse_color(value) {
                style.border_top_color = v;
                return true;
            }
        }
        "border-right-color" => {
            if let Some(v) = values::parse_color(value) {
                style.border_right_color = v;
                return true;
            }
        }
        "border-bottom-color" => {
            if let Some(v) = values::parse_color(value) {
                style.border_bottom_color = v;
                return true;
            }
        }
        "border-left-color" => {
            if let Some(v) = values::parse_color(value) {
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
            if let Some(v) = values::parse_length(value) {
                style.border_top_left_radius = v;
                return true;
            }
        }
        "border-top-right-radius" => {
            if let Some(v) = values::parse_length(value) {
                style.border_top_right_radius = v;
                return true;
            }
        }
        "border-bottom-right-radius" => {
            if let Some(v) = values::parse_length(value) {
                style.border_bottom_right_radius = v;
                return true;
            }
        }
        "border-bottom-left-radius" => {
            if let Some(v) = values::parse_length(value) {
                style.border_bottom_left_radius = v;
                return true;
            }
        }
        "color" => {
            if let Some(v) = values::parse_color(value) {
                style.color = v;
                return true;
            }
        }
        "background-color" => {
            if let Some(v) = values::parse_color(value) {
                style.background_color = v;
                return true;
            }
        }
        "opacity" => {
            if let Ok(v) = value.parse::<f64>() {
                style.opacity = v.clamp(0.0, 1.0);
                return true;
            }
        }
        "visibility" => {
            if let Some(v) = values::parse_visibility(value) {
                style.visibility = v;
                return true;
            }
        }
        "font-family" => {
            style.font_family = parse_font_family(value);
            return true;
        }
        "font-size" => {
            if let Some(v) = values::parse_length(value) {
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
        "text-transform" => {
            if let Some(v) = parse_text_transform(value) {
                style.text_transform = v;
                return true;
            }
        }
        "letter-spacing" => {
            if let Some(v) = values::parse_length(value) {
                style.letter_spacing = v;
                return true;
            }
        }
        "word-spacing" => {
            if let Some(v) = values::parse_length(value) {
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
        "text-overflow" => {
            if let Some(v) = parse_text_overflow(value) {
                style.text_overflow = v;
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
        "flex-grow" => {
            if let Ok(v) = value.parse::<f64>() {
                style.flex_grow = v;
                return true;
            }
        }
        "flex-shrink" => {
            if let Ok(v) = value.parse::<f64>() {
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
            if let Some(v) = values::parse_length(value) {
                style.gap = v;
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
            if let Some(v) = values::parse_length(value) {
                style.top = v;
                return true;
            }
        }
        "right" => {
            if let Some(v) = values::parse_length(value) {
                style.right = v;
                return true;
            }
        }
        "bottom" => {
            if let Some(v) = values::parse_length(value) {
                style.bottom = v;
                return true;
            }
        }
        "left" => {
            if let Some(v) = values::parse_length(value) {
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
        "row-gap" => {
            if let Some(v) = values::parse_length(value) {
                style.row_gap = v;
                return true;
            }
        }
        // ── Transforms ──
        "transform" => {
            if let Some(v) = values::parse_transform(value) {
                style.transform = v;
                return true;
            }
        }
        // ── Transitions ──
        "transition-property" => {
            style.transition_property =
                value.split(',').map(|s| s.trim().to_string()).collect();
            return true;
        }
        "transition-duration" => {
            let durations = value
                .split(',')
                .filter_map(|s| values::parse_time(s.trim()))
                .collect();
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
            let delays = value
                .split(',')
                .filter_map(|s| values::parse_time(s.trim()))
                .collect();
            style.transition_delay = delays;
            return true;
        }
        _ => {}
    }
    false
}

/// 从父元素样式继承指定属性到子元素样式。
///
/// 返回 true 表示成功继承。
pub fn inherit_property(
    parent: &ComputedStyle,
    child: &mut ComputedStyle,
    property: &str,
) -> bool {
    match property {
        "color" => {
            child.color = parent.color.clone();
            true
        }
        "font-family" => {
            child.font_family = parent.font_family.clone();
            true
        }
        "font-size" => {
            child.font_size = parent.font_size.clone();
            true
        }
        "font-weight" => {
            child.font_weight = parent.font_weight.clone();
            true
        }
        "font-style" => {
            child.font_style = parent.font_style.clone();
            true
        }
        "line-height" => {
            child.line_height = parent.line_height.clone();
            true
        }
        "text-align" => {
            child.text_align = parent.text_align.clone();
            true
        }
        "text-decoration" => {
            child.text_decoration = parent.text_decoration.clone();
            true
        }
        "text-transform" => {
            child.text_transform = parent.text_transform.clone();
            true
        }
        "letter-spacing" => {
            child.letter_spacing = parent.letter_spacing.clone();
            true
        }
        "word-spacing" => {
            child.word_spacing = parent.word_spacing.clone();
            true
        }
        "white-space" => {
            child.white_space = parent.white_space.clone();
            true
        }
        "text-overflow" => {
            child.text_overflow = parent.text_overflow.clone();
            true
        }
        "visibility" => {
            child.visibility = parent.visibility.clone();
            true
        }
        "opacity" => {
            child.opacity = parent.opacity;
            true
        }
        _ => false,
    }
}

/// 将初始值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
pub fn apply_initial_value(style: &mut ComputedStyle, property: &str) -> bool {
    // 先构建默认样式，然后继承对应字段
    let default_style = ComputedStyle::default();
    match property {
        "display" => {
            style.display = default_style.display;
            true
        }
        "position" => {
            style.position = default_style.position;
            true
        }
        "width" => {
            style.width = default_style.width;
            true
        }
        "height" => {
            style.height = default_style.height;
            true
        }
        "color" => {
            style.color = default_style.color;
            true
        }
        "background-color" => {
            style.background_color = default_style.background_color;
            true
        }
        "opacity" => {
            style.opacity = default_style.opacity;
            true
        }
        "font-family" => {
            style.font_family = default_style.font_family;
            true
        }
        "font-size" => {
            style.font_size = default_style.font_size;
            true
        }
        "font-weight" => {
            style.font_weight = default_style.font_weight;
            true
        }
        "font-style" => {
            style.font_style = default_style.font_style;
            true
        }
        "visibility" => {
            style.visibility = default_style.visibility;
            true
        }
        "overflow-x" => {
            style.overflow_x = default_style.overflow_x;
            true
        }
        "overflow-y" => {
            style.overflow_y = default_style.overflow_y;
            true
        }
        // 对于其他所有已知属性也提供初始值回退
        _ => {
            if PropertyRegistry::initial_value(property).is_some() {
                // 未知但已注册的属性：重新构建 default 并设值
                // 简化处理：直接用 apply_property_value 应用初始值字符串
                false
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_property_registry_initial_values() {
        assert!(PropertyRegistry::initial_value("display").is_some());
        assert!(PropertyRegistry::initial_value("color").is_some());
        assert!(PropertyRegistry::initial_value("font-size").is_some());
        assert!(PropertyRegistry::initial_value("unknown-prop").is_none());
    }

    #[test]
    fn test_property_registry_inheritance() {
        assert!(PropertyRegistry::is_inherited("color"));
        assert!(PropertyRegistry::is_inherited("font-size"));
        assert!(PropertyRegistry::is_inherited("visibility"));
        assert!(!PropertyRegistry::is_inherited("display"));
        assert!(!PropertyRegistry::is_inherited("margin-top"));
        assert!(!PropertyRegistry::is_inherited("width"));
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
        assert_eq!(parse_line_height("24px"), Some(LineHeightValue::Length(LengthValue::Px(24.0))));
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
        assert_eq!(parse_flex_basis("100px"), Some(FlexBasisValue::Length(LengthValue::Px(100.0))));
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
        assert_eq!(families, vec!["Times New Roman", "serif"]);
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

        assert!(!inherit_property(&parent, &mut child, "display"));
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
        assert!(apply_property_value(&mut style, "font-size", "1.5em"));
        assert_eq!(style.font_size, LengthValue::Em(1.5));
    }

    #[test]
    /// 测试 apply_property_value 对 color: 十六进制
    fn test_apply_property_color_hex() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "color", "#ff0000"));
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
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
        // 继承属性
        assert!(PropertyRegistry::is_inherited("color"));
        assert!(PropertyRegistry::is_inherited("font-family"));
        assert!(PropertyRegistry::is_inherited("font-size"));
        assert!(PropertyRegistry::is_inherited("font-weight"));
        assert!(PropertyRegistry::is_inherited("font-style"));
        assert!(PropertyRegistry::is_inherited("line-height"));
        assert!(PropertyRegistry::is_inherited("text-align"));
        assert!(PropertyRegistry::is_inherited("text-decoration"));
        assert!(PropertyRegistry::is_inherited("text-transform"));
        assert!(PropertyRegistry::is_inherited("letter-spacing"));
        assert!(PropertyRegistry::is_inherited("word-spacing"));
        assert!(PropertyRegistry::is_inherited("white-space"));
        assert!(PropertyRegistry::is_inherited("text-overflow"));
        assert!(PropertyRegistry::is_inherited("visibility"));
        assert!(PropertyRegistry::is_inherited("opacity"));

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
        assert_eq!(families, vec!["Helvetica Neue", "Arial", "sans-serif"]);

        // 双引号
        let families = parse_font_family("\"Times New Roman\", serif");
        assert_eq!(families, vec!["Times New Roman", "serif"]);

        // 空字符串和空白处理
        let families = parse_font_family("  Arial  ,  sans-serif  ");
        assert_eq!(families, vec!["Arial", "sans-serif"]);
    }

    #[test]
    /// 测试 parse_line_height 长度值
    fn test_parse_line_height_length() {
        assert_eq!(parse_line_height("24px"), Some(LineHeightValue::Length(LengthValue::Px(24.0))));
        assert_eq!(parse_line_height("2em"), Some(LineHeightValue::Length(LengthValue::Em(2.0))));
        assert_eq!(parse_line_height("1.5rem"), Some(LineHeightValue::Length(LengthValue::Rem(1.5))));
        assert_eq!(parse_line_height("normal"), Some(LineHeightValue::Normal));
        assert_eq!(parse_line_height("1.5"), Some(LineHeightValue::Number(1.5)));
        assert_eq!(parse_line_height("invalid"), None);
    }

    // ── Grid 属性测试 ──

    #[test]
    fn test_apply_property_grid_template_columns() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-template-columns", "100px 1fr auto"));
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
    }

    #[test]
    fn test_parse_grid_auto_flow() {
        assert_eq!(parse_grid_auto_flow("row"), Some(GridAutoFlowValue::Row));
        assert_eq!(parse_grid_auto_flow("column"), Some(GridAutoFlowValue::Column));
        assert_eq!(parse_grid_auto_flow("dense"), Some(GridAutoFlowValue::RowDense));
        assert_eq!(parse_grid_auto_flow("row dense"), Some(GridAutoFlowValue::RowDense));
        assert_eq!(parse_grid_auto_flow("column dense"), Some(GridAutoFlowValue::ColumnDense));
        assert_eq!(parse_grid_auto_flow("invalid"), None);
    }

    #[test]
    fn test_computed_style_default_grid() {
        let style = ComputedStyle::default();
        assert_eq!(style.grid_template_columns, None);
        assert_eq!(style.grid_template_rows, None);
        assert_eq!(style.grid_auto_flow, GridAutoFlowValue::Row);
        assert_eq!(style.row_gap, LengthValue::Px(0.0));
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

        assert!(apply_property_value(&mut style, "transition-property", "opacity, transform"));
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
        let result = parse_comma_separated_timing_functions("ease, linear");
        assert_eq!(result.len(), 2);

        let result = parse_comma_separated_timing_functions("cubic-bezier(0.25, 0.1, 0.25, 1.0)");
        assert_eq!(result.len(), 1);

        let result = parse_comma_separated_timing_functions("ease, cubic-bezier(0.25, 0.1, 0.25, 1.0), steps(4)");
        assert_eq!(result.len(), 3);
    }
}
