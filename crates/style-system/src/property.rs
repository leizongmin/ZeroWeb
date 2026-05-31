//! CSS 属性定义和计算样式结构。
//!
//! 定义 `ComputedStyle` 结构体，包含所有 Tier 1 CSS 属性的 typed 字段，
//! 以及 `PropertyRegistry` 用于查询初始值和继承性。

use zero_css_parser::values::{
    self, AlignmentValue, BoxSizingValue, ColorValue, ContainerTypeValue, DisplayValue, FlexDirectionValue,
    FlexWrapValue, FontStyleValue, FontWeightValue, LengthValue, OverflowValue, PositionValue, ScrollSnapAlignValue,
    ScrollSnapAxis, ScrollSnapStopValue, ScrollSnapTypeValue, VerticalAlignValue, VisibilityValue,
};

/// 尝试解析 CSS 长度值，支持简单值和数学函数（calc/min/max/clamp）。
///
/// 先尝试简单解析（parse_length），失败时尝试数学函数（parse_math_function）。
/// 数学函数在属性应用阶段存储为 `LengthValue::Calc`，后续由 `resolve_computed_style` 求值。
fn parse_length_or_math(value: &str) -> Option<LengthValue> {
    if let Some(v) = values::parse_length(value) {
        return Some(v);
    }
    // 尝试解析 calc/min/max/clamp 数学表达式
    values::parse_math_function(value).map(|expr| LengthValue::Calc(Box::new(expr)))
}

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

/// CSS outline-style 值。
/// 与 BorderStyleValue 相同但不含 Hidden。
#[derive(Debug, Clone, PartialEq)]
pub enum OutlineStyleValue {
    /// none。
    None,
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

/// CSS cursor 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CursorValue {
    /// auto。
    Auto,
    /// default。
    Default,
    /// pointer。
    Pointer,
    /// move。
    Move,
    /// text。
    Text,
    /// wait。
    Wait,
    /// crosshair。
    Crosshair,
    /// help。
    Help,
    /// not-allowed。
    NotAllowed,
    /// grab。
    Grab,
    /// grabbing。
    Grabbing,
    /// col-resize。
    ColResize,
    /// row-resize。
    RowResize,
    /// ns-resize。
    NsResize,
    /// ew-resize。
    EwResize,
    /// none。
    None,
    /// progress。
    Progress,
    /// cell。
    Cell,
    /// copy。
    Copy,
    /// alias。
    Alias,
    /// all-scroll。
    AllScroll,
    /// zoom-in。
    ZoomIn,
    /// zoom-out。
    ZoomOut,
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

/// CSS grid line 值（用于 grid-column-start/end、grid-row-start/end）。
#[derive(Debug, Clone, PartialEq)]
pub enum GridLineValue {
    /// auto。
    Auto,
    /// 行号（1-based，负数为从末尾计数）。
    Line(i16),
    /// span N（跨越 N 条轨道）。
    Span(u16),
    /// 命名区域（grid-area: header / grid-row-start: sidebar）。
    Name(String),
}

/// CSS scroll-snap-type 计算值。
///
/// 包含吸附严格度和轴方向。
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollSnapType {
    /// 吸附严格度。
    pub strictness: ScrollSnapStrictness,
    /// 吸附轴。
    pub axis: ScrollSnapAxis,
}

/// scroll-snap-type 严格度。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapStrictness {
    /// none — 不吸附。
    None,
    /// mandatory — 必须吸附。
    Mandatory,
    /// proximity — 接近时吸附。
    Proximity,
}

/// CSS scroll-snap-align 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAlign {
    /// none。
    None,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
}

/// CSS scroll-snap-stop 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapStop {
    /// normal。
    Normal,
    /// always。
    Always,
}

/// CSS scroll-padding 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollPadding {
    /// auto。
    Auto,
    /// 长度值（px）。
    Length(f32),
}

/// CSS container-type 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    /// normal。
    Normal,
    /// size。
    Size,
    /// inline-size。
    InlineSize,
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
    /// float 值。
    Float(zero_css_parser::values::FloatValue),
    /// clear 值。
    Clear(zero_css_parser::values::ClearValue),
    /// list-style-type 值。
    ListStyleType(zero_css_parser::values::ListStyleTypeValue),
    /// list-style-position 值。
    ListStylePosition(zero_css_parser::values::ListStylePositionValue),
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
    /// outline-style 值。
    OutlineStyle(OutlineStyleValue),
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
    /// cursor 值。
    Cursor(CursorValue),
    /// 数值（opacity, flex-grow, flex-shrink）。
    Number(f64),
    /// 整数（order）。
    Integer(i32),
    /// 字符串列表（font-family）。
    StringList(Vec<String>),
    /// 时间函数列表（transition-timing-function、animation-timing-function）。
    TimingFunctionList(Vec<zero_css_parser::values::TimingFunctionValue>),
    /// 动画方向列表（animation-direction）。
    AnimationDirectionList(Vec<zero_css_parser::values::AnimationDirectionValue>),
    /// 动画填充模式列表（animation-fill-mode）。
    AnimationFillModeList(Vec<zero_css_parser::values::AnimationFillModeValue>),
    /// 动画播放状态列表（animation-play-state）。
    AnimationPlayStateList(Vec<zero_css_parser::values::AnimationPlayStateValue>),
    /// 可选浮点数列表（animation-iteration-count，None 表示 infinite）。
    OptionalNumberList(Vec<Option<f64>>),
    /// grid-auto-flow 值。
    GridAutoFlow(GridAutoFlowValue),
    /// grid line 值（grid-column-start/end、grid-row-start/end）。
    GridLine(GridLineValue),
    /// transform 值。
    Transform(zero_css_parser::values::TransformValue),
    /// 可选字符串（grid-template-columns/rows、grid-auto-rows/columns）。
    OptionalString(Option<String>),
    /// scroll-snap-type 值。
    ScrollSnapType(ScrollSnapType),
    /// scroll-snap-align 值。
    ScrollSnapAlign(ScrollSnapAlign),
    /// scroll-snap-stop 值。
    ScrollSnapStop(ScrollSnapStop),
    /// scroll-padding 值。
    ScrollPadding(ScrollPadding),
    /// container-type 值。
    ContainerType(ContainerType),
    /// container-name 值。
    ContainerName(Option<String>),
    /// vertical-align 值。
    VerticalAlign(VerticalAlignValue),
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
    /// float 属性。
    pub float: zero_css_parser::values::FloatValue,
    /// clear 属性。
    pub clear: zero_css_parser::values::ClearValue,
    /// list-style-type 属性。
    pub list_style_type: zero_css_parser::values::ListStyleTypeValue,
    /// list-style-position 属性。
    pub list_style_position: zero_css_parser::values::ListStylePositionValue,
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

    // ── Outline ──
    /// outline-width 属性。
    pub outline_width: LengthValue,
    /// outline-style 属性。
    pub outline_style: OutlineStyleValue,
    /// outline-color 属性。
    pub outline_color: ColorValue,
    /// outline-offset 属性。
    pub outline_offset: LengthValue,

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
    /// vertical-align 属性。
    pub vertical_align: VerticalAlignValue,

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
    /// grid-column-start 属性。
    pub grid_column_start: GridLineValue,
    /// grid-column-end 属性。
    pub grid_column_end: GridLineValue,
    /// grid-row-start 属性。
    pub grid_row_start: GridLineValue,
    /// grid-row-end 属性。
    pub grid_row_end: GridLineValue,
    /// grid-auto-rows 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_auto_rows: Option<String>,
    /// grid-auto-columns 属性。
    /// 存储 CSS 原始值字符串，在布局转换时解析。
    pub grid_auto_columns: Option<String>,
    /// grid-template-areas 属性。
    /// 存储 CSS 原始值字符串（如 '"header header" "sidebar main"'），
    /// 在布局转换时解析为区域映射。
    pub grid_template_areas: Option<String>,

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

    // ── Aspect Ratio ──
    /// aspect-ratio 属性（width / height 比值），None 表示 auto。
    pub aspect_ratio: Option<f32>,

    // ── Cursor ──
    /// cursor 属性。
    pub cursor: CursorValue,

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

    // ── Animations ──
    /// animation-name 属性（逗号分隔的动画名列表）。
    pub animation_name: Vec<String>,
    /// animation-duration 属性（逗号分隔的秒数列表）。
    pub animation_duration: Vec<f64>,
    /// animation-timing-function 属性（逗号分隔的时间函数列表）。
    pub animation_timing_function: Vec<zero_css_parser::values::TimingFunctionValue>,
    /// animation-delay 属性（逗号分隔的秒数列表）。
    pub animation_delay: Vec<f64>,
    /// animation-iteration-count 属性（逗号分隔的迭代次数列表）。
    /// None 表示 infinite。
    pub animation_iteration_count: Vec<Option<f64>>,
    /// animation-direction 属性（逗号分隔的方向列表）。
    pub animation_direction: Vec<zero_css_parser::values::AnimationDirectionValue>,
    /// animation-fill-mode 属性（逗号分隔的填充模式列表）。
    pub animation_fill_mode: Vec<zero_css_parser::values::AnimationFillModeValue>,
    /// animation-play-state 属性（逗号分隔的播放状态列表）。
    pub animation_play_state: Vec<zero_css_parser::values::AnimationPlayStateValue>,

    // ── Scroll Snap ──
    /// scroll-snap-type 属性。
    pub scroll_snap_type: ScrollSnapType,
    /// scroll-snap-align 属性。
    pub scroll_snap_align: ScrollSnapAlign,
    /// scroll-snap-stop 属性。
    pub scroll_snap_stop: ScrollSnapStop,
    /// scroll-margin-top 属性（px）。
    pub scroll_margin_top: f32,
    /// scroll-margin-right 属性（px）。
    pub scroll_margin_right: f32,
    /// scroll-margin-bottom 属性（px）。
    pub scroll_margin_bottom: f32,
    /// scroll-margin-left 属性（px）。
    pub scroll_margin_left: f32,
    /// scroll-padding-top 属性。
    pub scroll_padding_top: ScrollPadding,
    /// scroll-padding-right 属性。
    pub scroll_padding_right: ScrollPadding,
    /// scroll-padding-bottom 属性。
    pub scroll_padding_bottom: ScrollPadding,
    /// scroll-padding-left 属性。
    pub scroll_padding_left: ScrollPadding,

    // ── Container Query ──
    /// container-type 属性。
    pub container_type: ContainerType,
    /// container-name 属性。
    pub container_name: Option<String>,
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
            float: zero_css_parser::values::FloatValue::None,
            clear: zero_css_parser::values::ClearValue::None,
            list_style_type: zero_css_parser::values::ListStyleTypeValue::Disc,
            list_style_position: zero_css_parser::values::ListStylePositionValue::Outside,
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

            // Outline
            outline_width: LengthValue::Px(0.0),
            outline_style: OutlineStyleValue::None,
            outline_color: initial_color.clone(),
            outline_offset: LengthValue::Px(0.0),

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
            vertical_align: VerticalAlignValue::Baseline,

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
            grid_column_start: GridLineValue::Auto,
            grid_column_end: GridLineValue::Auto,
            grid_row_start: GridLineValue::Auto,
            grid_row_end: GridLineValue::Auto,
            grid_auto_rows: None,
            grid_auto_columns: None,
            grid_template_areas: None,

            // 定位
            top: zero.clone(),
            right: zero.clone(),
            bottom: zero.clone(),
            left: zero,
            z_index: ZIndexValue::Auto,

            // Overflow
            overflow_x: OverflowValue::Visible,
            overflow_y: OverflowValue::Visible,

            // Aspect Ratio
            aspect_ratio: None,

            // Cursor
            cursor: CursorValue::Auto,

            // Transforms
            transform: zero_css_parser::values::TransformValue::None,

            // Transitions
            transition_property: vec![],
            transition_duration: vec![],
            transition_timing_function: vec![],
            transition_delay: vec![],

            // Animations
            animation_name: vec![],
            animation_duration: vec![],
            animation_timing_function: vec![],
            animation_delay: vec![],
            animation_iteration_count: vec![],
            animation_direction: vec![],
            animation_fill_mode: vec![],
            animation_play_state: vec![],

            // Scroll Snap
            scroll_snap_type: ScrollSnapType {
                strictness: ScrollSnapStrictness::None,
                axis: ScrollSnapAxis::Both,
            },
            scroll_snap_align: ScrollSnapAlign::None,
            scroll_snap_stop: ScrollSnapStop::Normal,
            scroll_margin_top: 0.0,
            scroll_margin_right: 0.0,
            scroll_margin_bottom: 0.0,
            scroll_margin_left: 0.0,
            scroll_padding_top: ScrollPadding::Auto,
            scroll_padding_right: ScrollPadding::Auto,
            scroll_padding_bottom: ScrollPadding::Auto,
            scroll_padding_left: ScrollPadding::Auto,

            // Container Query
            container_type: ContainerType::Normal,
            container_name: None,
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
            "float" => Some(Float(zero_css_parser::values::FloatValue::None)),
            "clear" => Some(Clear(zero_css_parser::values::ClearValue::None)),
            "list-style-type" => Some(ListStyleType(zero_css_parser::values::ListStyleTypeValue::Disc)),
            "list-style-position" => Some(ListStylePosition(
                zero_css_parser::values::ListStylePositionValue::Outside,
            )),
            "width" | "height" => Some(Length(LengthValue::Px(0.0))),
            "min-width" | "min-height" => Some(Length(LengthValue::Px(0.0))),
            "max-width" | "max-height" => Some(Length(LengthValue::Px(f64::INFINITY))),
            "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => Some(Length(LengthValue::Px(0.0))),
            "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => Some(Length(LengthValue::Px(0.0))),
            "box-sizing" => Some(BoxSizing(BoxSizingValue::ContentBox)),

            // 边框
            "border-top-width" | "border-right-width" | "border-bottom-width" | "border-left-width" => {
                Some(Length(LengthValue::Px(0.0)))
            }
            "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
                Some(Color(ColorValue::Rgba(0, 0, 0, 255)))
            }
            "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
                Some(BorderStyle(BorderStyleValue::None))
            }
            "border-top-left-radius"
            | "border-top-right-radius"
            | "border-bottom-right-radius"
            | "border-bottom-left-radius" => Some(Length(LengthValue::Px(0.0))),

            // Outline
            "outline-width" | "outline-offset" => Some(Length(LengthValue::Px(0.0))),
            "outline-style" => Some(OutlineStyle(OutlineStyleValue::None)),
            "outline-color" => Some(Color(ColorValue::Rgba(0, 0, 0, 255))),

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
            "vertical-align" => Some(VerticalAlign(VerticalAlignValue::Baseline)),

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

            // Aspect Ratio
            "aspect-ratio" => Some(Number(f64::NAN)), // NaN 表示 auto

            // Cursor
            "cursor" => Some(Cursor(CursorValue::Auto)),

            // Transitions
            "transition-property" => Some(StringList(vec![])),
            "transition-duration" | "transition-delay" => Some(Number(0.0)),
            "transition-timing-function" => Some(TimingFunctionList(vec![])),

            // Animations
            "animation-name" => Some(StringList(vec![])),
            "animation-duration" | "animation-delay" => Some(Number(0.0)),
            "animation-timing-function" => Some(TimingFunctionList(vec![])),
            "animation-iteration-count" => Some(OptionalNumberList(vec![])),
            "animation-direction" => Some(AnimationDirectionList(vec![])),
            "animation-fill-mode" => Some(AnimationFillModeList(vec![])),
            "animation-play-state" => Some(AnimationPlayStateList(vec![])),

            // Grid
            "grid-template-columns" | "grid-template-rows" => Some(OptionalString(None)),
            "grid-auto-flow" => Some(GridAutoFlow(GridAutoFlowValue::Row)),
            "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
                Some(GridLine(GridLineValue::Auto))
            }
            "grid-auto-rows" | "grid-auto-columns" => Some(OptionalString(None)),
            "grid-template-areas" => Some(OptionalString(None)),

            // Transform
            "transform" => Some(Transform(zero_css_parser::values::TransformValue::None)),

            // Scroll Snap
            "scroll-snap-type" => {
                let default_sst = crate::property::ScrollSnapType {
                    strictness: crate::property::ScrollSnapStrictness::None,
                    axis: zero_css_parser::values::ScrollSnapAxis::Both,
                };
                Some(ScrollSnapType(default_sst))
            }
            "scroll-snap-align" => Some(ScrollSnapAlign(crate::property::ScrollSnapAlign::None)),
            "scroll-snap-stop" => Some(ScrollSnapStop(crate::property::ScrollSnapStop::Normal)),
            "scroll-margin-top" | "scroll-margin-right" | "scroll-margin-bottom" | "scroll-margin-left" => {
                Some(Number(0.0))
            }
            "scroll-padding-top" | "scroll-padding-right" | "scroll-padding-bottom" | "scroll-padding-left" => {
                Some(ScrollPadding(crate::property::ScrollPadding::Auto))
            }

            // Container Query
            "container-type" => Some(ContainerType(crate::property::ContainerType::Normal)),
            "container-name" => Some(ContainerName(None)),

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
                | "text-transform"
                | "letter-spacing"
                | "word-spacing"
                | "white-space"
                | "visibility"
                | "cursor"
        )
    }

    /// 获取所有已知属性名的列表。
    pub fn known_properties() -> &'static [&'static str] {
        &[
            "display",
            "position",
            "float",
            "clear",
            "list-style-type",
            "list-style-position",
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
            "vertical-align",
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
            "aspect-ratio",
            "cursor",
            "transition-property",
            "transition-duration",
            "transition-timing-function",
            "transition-delay",
            "animation-name",
            "animation-duration",
            "animation-timing-function",
            "animation-delay",
            "animation-iteration-count",
            "animation-direction",
            "animation-fill-mode",
            "animation-play-state",
            "grid-column-start",
            "grid-column-end",
            "grid-row-start",
            "grid-row-end",
            "grid-auto-rows",
            "grid-auto-columns",
            "outline-width",
            "outline-style",
            "outline-color",
            "outline-offset",
            "scroll-snap-type",
            "scroll-snap-align",
            "scroll-snap-stop",
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
            "container-type",
            "container-name",
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

/// 解析 CSS outline-style 值。
pub fn parse_outline_style(value: &str) -> Option<OutlineStyleValue> {
    match value.trim() {
        "none" => Some(OutlineStyleValue::None),
        "dotted" => Some(OutlineStyleValue::Dotted),
        "dashed" => Some(OutlineStyleValue::Dashed),
        "solid" => Some(OutlineStyleValue::Solid),
        "double" => Some(OutlineStyleValue::Double),
        "groove" => Some(OutlineStyleValue::Groove),
        "ridge" => Some(OutlineStyleValue::Ridge),
        "inset" => Some(OutlineStyleValue::Inset),
        "outset" => Some(OutlineStyleValue::Outset),
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

/// 解析 CSS grid line 值（用于 grid-column/row-start/end）。
///
/// 支持格式：`auto`、`1`（行号）、`-1`（从末尾）、`span 2`。
pub fn parse_grid_line(value: &str) -> Option<GridLineValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridLineValue::Auto);
    }
    if let Some(span_str) = value.strip_prefix("span ") {
        let span: u16 = span_str.trim().parse().ok()?;
        return Some(GridLineValue::Span(span));
    }
    if let Some(span_str) = value.strip_prefix("span") {
        let span: u16 = span_str.trim().parse().ok()?;
        return Some(GridLineValue::Span(span));
    }
    if let Ok(line) = value.parse::<i16>() {
        if line == 0 {
            return None; // 0 是非法的 grid line 值
        }
        return Some(GridLineValue::Line(line));
    }
    // 非数字值视为命名区域（如 "header"、"sidebar"）
    // 合法的命名区域标识符：非空，不含 / 和数字开头
    if !value.is_empty() && !value.starts_with(|c: char| c.is_ascii_digit()) && !value.contains('/') {
        return Some(GridLineValue::Name(value.to_string()));
    }
    None
}

/// 解析逗号分隔的 transition-timing-function 列表。
///
/// 需要处理 cubic-bezier() 和 steps() 内部的逗号。
fn parse_comma_separated_timing_functions(value: &str) -> Vec<zero_css_parser::values::TimingFunctionValue> {
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

/// 解析 CSS cursor 值。
pub fn parse_cursor(value: &str) -> Option<CursorValue> {
    match value.trim() {
        "auto" => Some(CursorValue::Auto),
        "default" => Some(CursorValue::Default),
        "pointer" => Some(CursorValue::Pointer),
        "move" => Some(CursorValue::Move),
        "text" => Some(CursorValue::Text),
        "wait" => Some(CursorValue::Wait),
        "crosshair" => Some(CursorValue::Crosshair),
        "help" => Some(CursorValue::Help),
        "not-allowed" => Some(CursorValue::NotAllowed),
        "grab" => Some(CursorValue::Grab),
        "grabbing" => Some(CursorValue::Grabbing),
        "col-resize" => Some(CursorValue::ColResize),
        "row-resize" => Some(CursorValue::RowResize),
        "ns-resize" => Some(CursorValue::NsResize),
        "ew-resize" => Some(CursorValue::EwResize),
        "none" => Some(CursorValue::None),
        "progress" => Some(CursorValue::Progress),
        "cell" => Some(CursorValue::Cell),
        "copy" => Some(CursorValue::Copy),
        "alias" => Some(CursorValue::Alias),
        "all-scroll" => Some(CursorValue::AllScroll),
        "zoom-in" => Some(CursorValue::ZoomIn),
        "zoom-out" => Some(CursorValue::ZoomOut),
        _ => None,
    }
}

/// 解析 CSS scroll-snap-type 值。
///
/// 格式：none | [ mandatory | proximity ] [ x | y | both ]?
pub fn parse_scroll_snap_type_computed(value: &str) -> Option<ScrollSnapType> {
    let parsed = values::parse_scroll_snap_type(value)?;
    let strictness = match parsed.0 {
        ScrollSnapTypeValue::None => ScrollSnapStrictness::None,
        ScrollSnapTypeValue::Mandatory => ScrollSnapStrictness::Mandatory,
        ScrollSnapTypeValue::Proximity => ScrollSnapStrictness::Proximity,
    };
    let axis = parsed.1.unwrap_or(ScrollSnapAxis::Both);
    Some(ScrollSnapType { strictness, axis })
}

/// 解析 CSS scroll-snap-align 值。
pub fn parse_scroll_snap_align_computed(value: &str) -> Option<ScrollSnapAlign> {
    match values::parse_scroll_snap_align(value)? {
        ScrollSnapAlignValue::None => Some(ScrollSnapAlign::None),
        ScrollSnapAlignValue::Start => Some(ScrollSnapAlign::Start),
        ScrollSnapAlignValue::End => Some(ScrollSnapAlign::End),
        ScrollSnapAlignValue::Center => Some(ScrollSnapAlign::Center),
    }
}

/// 解析 CSS scroll-snap-stop 值。
pub fn parse_scroll_snap_stop_computed(value: &str) -> Option<ScrollSnapStop> {
    match values::parse_scroll_snap_stop(value)? {
        ScrollSnapStopValue::Normal => Some(ScrollSnapStop::Normal),
        ScrollSnapStopValue::Always => Some(ScrollSnapStop::Always),
    }
}

/// 解析 CSS scroll-padding 值。
pub fn parse_scroll_padding(value: &str) -> Option<ScrollPadding> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(ScrollPadding::Auto);
    }
    values::parse_length(v).map(|l| {
        let px = match l {
            LengthValue::Px(n) => n as f32,
            other => resolve_length_to_px(other),
        };
        ScrollPadding::Length(px)
    })
}

/// 将 LengthValue 转换为 f32 px（简单近似，非相对单位返回 0.0）。
fn resolve_length_to_px(l: LengthValue) -> f32 {
    match l {
        LengthValue::Px(n) => n as f32,
        _ => 0.0,
    }
}

/// 解析 CSS container-type 值。
pub fn parse_container_type_computed(value: &str) -> Option<ContainerType> {
    match values::parse_container_type(value)? {
        ContainerTypeValue::Normal => Some(ContainerType::Normal),
        ContainerTypeValue::Size => Some(ContainerType::Size),
        ContainerTypeValue::InlineSize => Some(ContainerType::InlineSize),
    }
}

/// 解析 font-family 值。
///
/// 简单实现：按逗号分割，去除引号和空格。
pub fn parse_font_family(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
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
        "width" => {
            if let Some(v) = parse_length_or_math(value) {
                style.width = v;
                return true;
            }
        }
        "height" => {
            if let Some(v) = parse_length_or_math(value) {
                style.height = v;
                return true;
            }
        }
        "min-width" => {
            if let Some(v) = parse_length_or_math(value) {
                style.min_width = v;
                return true;
            }
        }
        "min-height" => {
            if let Some(v) = parse_length_or_math(value) {
                style.min_height = v;
                return true;
            }
        }
        "max-width" => {
            if value == "none" {
                style.max_width = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_or_math(value) {
                style.max_width = v;
                return true;
            }
        }
        "max-height" => {
            if value == "none" {
                style.max_height = LengthValue::Px(f64::INFINITY);
                return true;
            }
            if let Some(v) = parse_length_or_math(value) {
                style.max_height = v;
                return true;
            }
        }
        "margin-top" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_top = v;
                return true;
            }
        }
        "margin-right" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_right = v;
                return true;
            }
        }
        "margin-bottom" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_bottom = v;
                return true;
            }
        }
        "margin-left" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_left = v;
                return true;
            }
        }
        "padding-top" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_top = v;
                return true;
            }
        }
        "padding-right" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_right = v;
                return true;
            }
        }
        "padding-bottom" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_bottom = v;
                return true;
            }
        }
        "padding-left" => {
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_length_or_math(value) {
                style.border_top_width = v;
                return true;
            }
        }
        "border-right-width" => {
            if let Some(v) = parse_length_or_math(value) {
                style.border_right_width = v;
                return true;
            }
        }
        "border-bottom-width" => {
            if let Some(v) = parse_length_or_math(value) {
                style.border_bottom_width = v;
                return true;
            }
        }
        "border-left-width" => {
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_length_or_math(value) {
                style.border_top_left_radius = v;
                return true;
            }
        }
        "border-top-right-radius" => {
            if let Some(v) = parse_length_or_math(value) {
                style.border_top_right_radius = v;
                return true;
            }
        }
        "border-bottom-right-radius" => {
            if let Some(v) = parse_length_or_math(value) {
                style.border_bottom_right_radius = v;
                return true;
            }
        }
        "border-bottom-left-radius" => {
            if let Some(v) = parse_length_or_math(value) {
                style.border_bottom_left_radius = v;
                return true;
            }
        }
        // ── Outline 属性 ──
        "outline-width" => {
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = values::parse_color(value) {
                style.outline_color = v;
                return true;
            }
        }
        "outline-offset" => {
            if let Some(v) = parse_length_or_math(value) {
                style.outline_offset = v;
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
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_length_or_math(value) {
                style.letter_spacing = v;
                return true;
            }
        }
        "word-spacing" => {
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_length_or_math(value) {
                style.top = v;
                return true;
            }
        }
        "right" => {
            if let Some(v) = parse_length_or_math(value) {
                style.right = v;
                return true;
            }
        }
        "bottom" => {
            if let Some(v) = parse_length_or_math(value) {
                style.bottom = v;
                return true;
            }
        }
        "left" => {
            if let Some(v) = parse_length_or_math(value) {
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
            if let Some(v) = parse_cursor(value) {
                style.cursor = v;
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
        "row-gap" => {
            if let Some(v) = parse_length_or_math(value) {
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
            style.transition_property = value.split(',').map(|s| s.trim().to_string()).collect();
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
        // margin-block-start → margin-top (horizontal writing-mode 映射)
        "margin-block-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_top = v;
                return true;
            }
        }
        "margin-block-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_bottom = v;
                return true;
            }
        }
        "margin-inline-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_left = v;
                return true;
            }
        }
        "margin-inline-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.margin_right = v;
                return true;
            }
        }
        "padding-block-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_top = v;
                return true;
            }
        }
        "padding-block-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_bottom = v;
                return true;
            }
        }
        "padding-inline-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_left = v;
                return true;
            }
        }
        "padding-inline-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.padding_right = v;
                return true;
            }
        }
        "inset-block-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.top = v;
                return true;
            }
        }
        "inset-block-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.bottom = v;
                return true;
            }
        }
        "inset-inline-start" => {
            if let Some(v) = parse_length_or_math(value) {
                style.left = v;
                return true;
            }
        }
        "inset-inline-end" => {
            if let Some(v) = parse_length_or_math(value) {
                style.right = v;
                return true;
            }
        }

        // ── Animation 属性 ──
        "animation-name" => {
            style.animation_name = value.split(',').map(|s| s.trim().to_string()).collect();
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
        _ => {}
    }
    false
}

/// 从父元素样式继承指定属性到子元素样式。
///
/// 返回 true 表示成功继承。
pub fn inherit_property(parent: &ComputedStyle, child: &mut ComputedStyle, property: &str) -> bool {
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
        "visibility" => {
            child.visibility = parent.visibility.clone();
            true
        }
        "cursor" => {
            child.cursor = parent.cursor.clone();
            true
        }
        _ => false,
    }
}

/// 将初始值设置到 ComputedStyle 的对应字段。
///
/// 返回 true 表示成功设置。
pub fn apply_initial_value(style: &mut ComputedStyle, property: &str) -> bool {
    let default_style = ComputedStyle::default();
    match property {
        // 盒模型
        "display" => {
            style.display = default_style.display;
            true
        }
        "position" => {
            style.position = default_style.position;
            true
        }
        "float" => {
            style.float = default_style.float;
            true
        }
        "clear" => {
            style.clear = default_style.clear;
            true
        }
        "list-style-type" => {
            style.list_style_type = default_style.list_style_type;
            true
        }
        "list-style-position" => {
            style.list_style_position = default_style.list_style_position;
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
        "min-width" => {
            style.min_width = default_style.min_width;
            true
        }
        "min-height" => {
            style.min_height = default_style.min_height;
            true
        }
        "max-width" => {
            style.max_width = default_style.max_width;
            true
        }
        "max-height" => {
            style.max_height = default_style.max_height;
            true
        }
        "margin-top" => {
            style.margin_top = default_style.margin_top;
            true
        }
        "margin-right" => {
            style.margin_right = default_style.margin_right;
            true
        }
        "margin-bottom" => {
            style.margin_bottom = default_style.margin_bottom;
            true
        }
        "margin-left" => {
            style.margin_left = default_style.margin_left;
            true
        }
        "padding-top" => {
            style.padding_top = default_style.padding_top;
            true
        }
        "padding-right" => {
            style.padding_right = default_style.padding_right;
            true
        }
        "padding-bottom" => {
            style.padding_bottom = default_style.padding_bottom;
            true
        }
        "padding-left" => {
            style.padding_left = default_style.padding_left;
            true
        }
        "box-sizing" => {
            style.box_sizing = default_style.box_sizing;
            true
        }
        // 边框
        "border-top-width" => {
            style.border_top_width = default_style.border_top_width;
            true
        }
        "border-right-width" => {
            style.border_right_width = default_style.border_right_width;
            true
        }
        "border-bottom-width" => {
            style.border_bottom_width = default_style.border_bottom_width;
            true
        }
        "border-left-width" => {
            style.border_left_width = default_style.border_left_width;
            true
        }
        "border-top-color" => {
            style.border_top_color = default_style.border_top_color;
            true
        }
        "border-right-color" => {
            style.border_right_color = default_style.border_right_color;
            true
        }
        "border-bottom-color" => {
            style.border_bottom_color = default_style.border_bottom_color;
            true
        }
        "border-left-color" => {
            style.border_left_color = default_style.border_left_color;
            true
        }
        "border-top-style" => {
            style.border_top_style = default_style.border_top_style;
            true
        }
        "border-right-style" => {
            style.border_right_style = default_style.border_right_style;
            true
        }
        "border-bottom-style" => {
            style.border_bottom_style = default_style.border_bottom_style;
            true
        }
        "border-left-style" => {
            style.border_left_style = default_style.border_left_style;
            true
        }
        "border-top-left-radius" => {
            style.border_top_left_radius = default_style.border_top_left_radius;
            true
        }
        "border-top-right-radius" => {
            style.border_top_right_radius = default_style.border_top_right_radius;
            true
        }
        "border-bottom-right-radius" => {
            style.border_bottom_right_radius = default_style.border_bottom_right_radius;
            true
        }
        "border-bottom-left-radius" => {
            style.border_bottom_left_radius = default_style.border_bottom_left_radius;
            true
        }
        // Outline
        "outline-width" => {
            style.outline_width = default_style.outline_width;
            true
        }
        "outline-style" => {
            style.outline_style = default_style.outline_style;
            true
        }
        "outline-color" => {
            style.outline_color = default_style.outline_color;
            true
        }
        "outline-offset" => {
            style.outline_offset = default_style.outline_offset;
            true
        }
        // 颜色和背景
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
        "visibility" => {
            style.visibility = default_style.visibility;
            true
        }
        // 字体
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
        "line-height" => {
            style.line_height = default_style.line_height;
            true
        }
        // 文本
        "text-align" => {
            style.text_align = default_style.text_align;
            true
        }
        "text-decoration" => {
            style.text_decoration = default_style.text_decoration;
            true
        }
        "text-transform" => {
            style.text_transform = default_style.text_transform;
            true
        }
        "letter-spacing" => {
            style.letter_spacing = default_style.letter_spacing;
            true
        }
        "word-spacing" => {
            style.word_spacing = default_style.word_spacing;
            true
        }
        "white-space" => {
            style.white_space = default_style.white_space;
            true
        }
        "text-overflow" => {
            style.text_overflow = default_style.text_overflow;
            true
        }
        "vertical-align" => {
            style.vertical_align = default_style.vertical_align;
            true
        }
        // Flexbox
        "flex-direction" => {
            style.flex_direction = default_style.flex_direction;
            true
        }
        "flex-wrap" => {
            style.flex_wrap = default_style.flex_wrap;
            true
        }
        "justify-content" => {
            style.justify_content = default_style.justify_content;
            true
        }
        "align-items" => {
            style.align_items = default_style.align_items;
            true
        }
        "align-self" => {
            style.align_self = default_style.align_self;
            true
        }
        "flex-grow" => {
            style.flex_grow = default_style.flex_grow;
            true
        }
        "flex-shrink" => {
            style.flex_shrink = default_style.flex_shrink;
            true
        }
        "flex-basis" => {
            style.flex_basis = default_style.flex_basis;
            true
        }
        "gap" => {
            style.gap = default_style.gap;
            true
        }
        "row-gap" => {
            style.row_gap = default_style.row_gap;
            true
        }
        "order" => {
            style.order = default_style.order;
            true
        }
        // Grid
        "grid-template-columns" => {
            style.grid_template_columns = default_style.grid_template_columns;
            true
        }
        "grid-template-rows" => {
            style.grid_template_rows = default_style.grid_template_rows;
            true
        }
        "grid-auto-flow" => {
            style.grid_auto_flow = default_style.grid_auto_flow;
            true
        }
        "grid-column-start" => {
            style.grid_column_start = default_style.grid_column_start;
            true
        }
        "grid-column-end" => {
            style.grid_column_end = default_style.grid_column_end;
            true
        }
        "grid-row-start" => {
            style.grid_row_start = default_style.grid_row_start;
            true
        }
        "grid-row-end" => {
            style.grid_row_end = default_style.grid_row_end;
            true
        }
        "grid-auto-rows" => {
            style.grid_auto_rows = default_style.grid_auto_rows;
            true
        }
        "grid-auto-columns" => {
            style.grid_auto_columns = default_style.grid_auto_columns;
            true
        }
        "grid-template-areas" => {
            style.grid_template_areas = default_style.grid_template_areas;
            true
        }
        // 定位
        "top" => {
            style.top = default_style.top;
            true
        }
        "right" => {
            style.right = default_style.right;
            true
        }
        "bottom" => {
            style.bottom = default_style.bottom;
            true
        }
        "left" => {
            style.left = default_style.left;
            true
        }
        "z-index" => {
            style.z_index = default_style.z_index;
            true
        }
        // Overflow
        "overflow-x" => {
            style.overflow_x = default_style.overflow_x;
            true
        }
        "overflow-y" => {
            style.overflow_y = default_style.overflow_y;
            true
        }
        // Aspect Ratio
        "aspect-ratio" => {
            style.aspect_ratio = default_style.aspect_ratio;
            true
        }
        // Cursor
        "cursor" => {
            style.cursor = default_style.cursor;
            true
        }
        // Transform
        "transform" => {
            style.transform = default_style.transform;
            true
        }
        // Transitions
        "transition-property" => {
            style.transition_property = default_style.transition_property;
            true
        }
        "transition-duration" => {
            style.transition_duration = default_style.transition_duration;
            true
        }
        "transition-timing-function" => {
            style.transition_timing_function = default_style.transition_timing_function;
            true
        }
        "transition-delay" => {
            style.transition_delay = default_style.transition_delay;
            true
        }
        // Animations
        "animation-name" => {
            style.animation_name = default_style.animation_name;
            true
        }
        "animation-duration" => {
            style.animation_duration = default_style.animation_duration;
            true
        }
        "animation-timing-function" => {
            style.animation_timing_function = default_style.animation_timing_function;
            true
        }
        "animation-delay" => {
            style.animation_delay = default_style.animation_delay;
            true
        }
        "animation-iteration-count" => {
            style.animation_iteration_count = default_style.animation_iteration_count;
            true
        }
        "animation-direction" => {
            style.animation_direction = default_style.animation_direction;
            true
        }
        "animation-fill-mode" => {
            style.animation_fill_mode = default_style.animation_fill_mode;
            true
        }
        "animation-play-state" => {
            style.animation_play_state = default_style.animation_play_state;
            true
        }
        // Scroll Snap
        "scroll-snap-type" => {
            style.scroll_snap_type = default_style.scroll_snap_type;
            true
        }
        "scroll-snap-align" => {
            style.scroll_snap_align = default_style.scroll_snap_align;
            true
        }
        "scroll-snap-stop" => {
            style.scroll_snap_stop = default_style.scroll_snap_stop;
            true
        }
        "scroll-margin-top" => {
            style.scroll_margin_top = default_style.scroll_margin_top;
            true
        }
        "scroll-margin-right" => {
            style.scroll_margin_right = default_style.scroll_margin_right;
            true
        }
        "scroll-margin-bottom" => {
            style.scroll_margin_bottom = default_style.scroll_margin_bottom;
            true
        }
        "scroll-margin-left" => {
            style.scroll_margin_left = default_style.scroll_margin_left;
            true
        }
        "scroll-padding-top" => {
            style.scroll_padding_top = default_style.scroll_padding_top;
            true
        }
        "scroll-padding-right" => {
            style.scroll_padding_right = default_style.scroll_padding_right;
            true
        }
        "scroll-padding-bottom" => {
            style.scroll_padding_bottom = default_style.scroll_padding_bottom;
            true
        }
        "scroll-padding-left" => {
            style.scroll_padding_left = default_style.scroll_padding_left;
            true
        }
        // Container Query
        "container-type" => {
            style.container_type = default_style.container_type;
            true
        }
        "container-name" => {
            style.container_name = default_style.container_name;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
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
        // 正确的继承属性
        assert!(PropertyRegistry::is_inherited("color"));
        assert!(PropertyRegistry::is_inherited("font-size"));
        assert!(PropertyRegistry::is_inherited("visibility"));
        assert!(PropertyRegistry::is_inherited("cursor"));
        assert!(PropertyRegistry::is_inherited("line-height"));
        assert!(PropertyRegistry::is_inherited("white-space"));
        assert!(PropertyRegistry::is_inherited("text-align"));
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
    }

    #[test]
    fn test_apply_animation_fill_mode() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "animation-fill-mode", "forwards"));
        assert_eq!(style.animation_fill_mode.len(), 1);

        assert!(apply_property_value(&mut style, "animation-fill-mode", "both"));
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
        assert_eq!(style.outline_width, LengthValue::Px(0.0));
        assert_eq!(style.outline_style, OutlineStyleValue::None);
        assert_eq!(style.outline_color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.outline_offset, LengthValue::Px(0.0));
    }

    #[test]
    fn test_apply_outline_width() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "outline-width", "2px"));
        assert_eq!(style.outline_width, LengthValue::Px(2.0));

        assert!(apply_property_value(&mut style, "outline-width", "0.5em"));
        assert_eq!(style.outline_width, LengthValue::Em(0.5));

        assert!(!apply_property_value(&mut style, "outline-width", "invalid"));
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

        assert!(apply_property_value(&mut style, "scroll-padding-right", "auto"));
        assert_eq!(style.scroll_padding_right, ScrollPadding::Auto);

        assert!(apply_property_value(&mut style, "scroll-padding-bottom", "5px"));
        assert_eq!(style.scroll_padding_bottom, ScrollPadding::Length(5.0));

        assert!(apply_property_value(&mut style, "scroll-padding-left", "0px"));
        assert_eq!(style.scroll_padding_left, ScrollPadding::Length(0.0));
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

    #[test]
    fn test_apply_property_list_style_type() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.list_style_type, zero_css_parser::values::ListStyleTypeValue::Disc);

        assert!(apply_property_value(&mut style, "list-style-type", "circle"));
        assert_eq!(
            style.list_style_type,
            zero_css_parser::values::ListStyleTypeValue::Circle
        );

        assert!(apply_property_value(&mut style, "list-style-type", "decimal"));
        assert_eq!(
            style.list_style_type,
            zero_css_parser::values::ListStyleTypeValue::Decimal
        );

        assert!(apply_property_value(&mut style, "list-style-type", "none"));
        assert_eq!(style.list_style_type, zero_css_parser::values::ListStyleTypeValue::None);

        assert!(!apply_property_value(&mut style, "list-style-type", "invalid"));
    }

    #[test]
    fn test_apply_property_list_style_position() {
        let mut style = ComputedStyle::default();
        assert_eq!(
            style.list_style_position,
            zero_css_parser::values::ListStylePositionValue::Outside
        );

        assert!(apply_property_value(&mut style, "list-style-position", "inside"));
        assert_eq!(
            style.list_style_position,
            zero_css_parser::values::ListStylePositionValue::Inside
        );

        assert!(!apply_property_value(&mut style, "list-style-position", "invalid"));
    }

    #[test]
    fn test_list_style_property_registry() {
        assert!(PropertyRegistry::initial_value("list-style-type").is_some());
        assert!(PropertyRegistry::initial_value("list-style-position").is_some());
        assert!(!PropertyRegistry::is_inherited("list-style-type"));
        assert!(!PropertyRegistry::is_inherited("list-style-position"));
    }

    #[test]
    fn test_list_style_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"list-style-type"));
        assert!(props.contains(&"list-style-position"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增 property 边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// PropertyRegistry 已注册的属性数量
    fn test_property_registry_count() {
        let props = PropertyRegistry::known_properties();
        // 确保至少有 80 个已知属性
        assert!(
            props.len() >= 80,
            "known_properties should have at least 80 entries, got {}",
            props.len()
        );
    }

    #[test]
    /// inherit 关键字在 apply_property_value 中不被当作 display 值
    fn test_inherit_keyword_not_valid_display() {
        let mut style = ComputedStyle::default();
        // "inherit" 不是一个有效的 display 值
        assert!(!apply_property_value(&mut style, "display", "inherit"));
        // display 不应该改变
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// initial 关键字在 apply_property_value 中不被当作 display 值
    fn test_initial_keyword_not_valid_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        // "initial" 不是一个有效的 display 值
        assert!(!apply_property_value(&mut style, "display", "initial"));
        assert_eq!(style.display, DisplayValue::Flex);
    }

    #[test]
    /// unset 关键字在 apply_property_value 中不被当作 position 值
    fn test_unset_keyword_not_valid_position() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "position", "unset"));
        assert_eq!(style.position, PositionValue::Static);
    }

    #[test]
    /// revert 关键字在 apply_property_value 中不被当作 position 值
    fn test_revert_keyword_not_valid_position() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "position", "revert"));
        assert_eq!(style.position, PositionValue::Static);
    }

    #[test]
    /// ComputedStyle::default 所有继承属性初始值正确性
    fn test_default_inherited_properties_initial_values() {
        let style = ComputedStyle::default();
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.font_family, Vec::<String>::new());
        assert_eq!(style.font_size, LengthValue::Px(16.0));
        assert_eq!(style.font_weight, FontWeightValue::Normal);
        assert_eq!(style.font_style, FontStyleValue::Normal);
        assert_eq!(style.line_height, LineHeightValue::Normal);
        assert_eq!(style.text_align, TextAlignValue::Start);
        assert_eq!(style.text_transform, TextTransformValue::None);
        assert_eq!(style.letter_spacing, LengthValue::Px(0.0));
        assert_eq!(style.word_spacing, LengthValue::Px(0.0));
        assert_eq!(style.white_space, WhiteSpaceValue::Normal);
        assert_eq!(style.visibility, VisibilityValue::Visible);
        assert_eq!(style.cursor, CursorValue::Auto);
    }

    #[test]
    /// apply_property_value 对 opacity 的 clamp 行为
    fn test_opacity_clamp_edge_values() {
        let mut style = ComputedStyle::default();

        // 正常值
        assert!(apply_property_value(&mut style, "opacity", "0.0"));
        assert_eq!(style.opacity, 0.0);

        assert!(apply_property_value(&mut style, "opacity", "1.0"));
        assert_eq!(style.opacity, 1.0);

        // 超出范围 clamp
        assert!(apply_property_value(&mut style, "opacity", "1.5"));
        assert_eq!(style.opacity, 1.0);

        assert!(apply_property_value(&mut style, "opacity", "-0.1"));
        assert_eq!(style.opacity, 0.0);

        // 无效值
        assert!(!apply_property_value(&mut style, "opacity", "abc"));
    }

    #[test]
    /// parse_border_style 所有变体
    fn test_parse_border_style_all_variants() {
        assert_eq!(parse_border_style("none"), Some(BorderStyleValue::None));
        assert_eq!(parse_border_style("hidden"), Some(BorderStyleValue::Hidden));
        assert_eq!(parse_border_style("dotted"), Some(BorderStyleValue::Dotted));
        assert_eq!(parse_border_style("dashed"), Some(BorderStyleValue::Dashed));
        assert_eq!(parse_border_style("solid"), Some(BorderStyleValue::Solid));
        assert_eq!(parse_border_style("double"), Some(BorderStyleValue::Double));
        assert_eq!(parse_border_style("groove"), Some(BorderStyleValue::Groove));
        assert_eq!(parse_border_style("ridge"), Some(BorderStyleValue::Ridge));
        assert_eq!(parse_border_style("inset"), Some(BorderStyleValue::Inset));
        assert_eq!(parse_border_style("outset"), Some(BorderStyleValue::Outset));
        assert_eq!(parse_border_style("unknown"), None);
    }

    #[test]
    /// parse_text_align 所有变体
    fn test_parse_text_align_all_variants() {
        assert_eq!(parse_text_align("left"), Some(TextAlignValue::Left));
        assert_eq!(parse_text_align("right"), Some(TextAlignValue::Right));
        assert_eq!(parse_text_align("center"), Some(TextAlignValue::Center));
        assert_eq!(parse_text_align("justify"), Some(TextAlignValue::Justify));
        assert_eq!(parse_text_align("start"), Some(TextAlignValue::Start));
        assert_eq!(parse_text_align("end"), Some(TextAlignValue::End));
        assert_eq!(parse_text_align("invalid"), None);
    }

    #[test]
    /// parse_text_decoration 所有变体
    fn test_parse_text_decoration_all_variants() {
        assert_eq!(parse_text_decoration("none"), Some(TextDecorationValue::None));
        assert_eq!(parse_text_decoration("underline"), Some(TextDecorationValue::Underline));
        assert_eq!(parse_text_decoration("overline"), Some(TextDecorationValue::Overline));
        assert_eq!(
            parse_text_decoration("line-through"),
            Some(TextDecorationValue::LineThrough)
        );
        assert_eq!(parse_text_decoration("blink"), None);
    }

    #[test]
    /// parse_white_space 所有变体
    fn test_parse_white_space_all_variants() {
        assert_eq!(parse_white_space("normal"), Some(WhiteSpaceValue::Normal));
        assert_eq!(parse_white_space("pre"), Some(WhiteSpaceValue::Pre));
        assert_eq!(parse_white_space("nowrap"), Some(WhiteSpaceValue::Nowrap));
        assert_eq!(parse_white_space("pre-wrap"), Some(WhiteSpaceValue::PreWrap));
        assert_eq!(parse_white_space("pre-line"), Some(WhiteSpaceValue::PreLine));
        assert_eq!(parse_white_space("invalid"), None);
    }

    #[test]
    /// parse_text_transform 所有变体
    fn test_parse_text_transform_all_variants() {
        assert_eq!(parse_text_transform("none"), Some(TextTransformValue::None));
        assert_eq!(parse_text_transform("uppercase"), Some(TextTransformValue::Uppercase));
        assert_eq!(parse_text_transform("lowercase"), Some(TextTransformValue::Lowercase));
        assert_eq!(parse_text_transform("capitalize"), Some(TextTransformValue::Capitalize));
        assert_eq!(parse_text_transform("invalid"), None);
    }

    #[test]
    /// parse_text_overflow 所有变体
    fn test_parse_text_overflow_all_variants() {
        assert_eq!(parse_text_overflow("clip"), Some(TextOverflowValue::Clip));
        assert_eq!(parse_text_overflow("ellipsis"), Some(TextOverflowValue::Ellipsis));
        assert_eq!(parse_text_overflow("invalid"), None);
    }

    #[test]
    /// parse_grid_line: span 不带空格
    fn test_parse_grid_line_span_no_space() {
        assert_eq!(parse_grid_line("span2"), Some(GridLineValue::Span(2)));
        assert_eq!(parse_grid_line("span3"), Some(GridLineValue::Span(3)));
    }

    #[test]
    /// parse_grid_line: 命名区域
    fn test_parse_grid_line_named_area() {
        assert_eq!(
            parse_grid_line("header"),
            Some(GridLineValue::Name("header".to_string()))
        );
        assert_eq!(
            parse_grid_line("sidebar"),
            Some(GridLineValue::Name("sidebar".to_string()))
        );
    }

    #[test]
    /// parse_grid_line: 0 是非法值
    fn test_parse_grid_line_zero_invalid() {
        assert_eq!(parse_grid_line("0"), None);
    }

    #[test]
    /// parse_flex_basis 所有变体
    fn test_parse_flex_basis_all_variants() {
        assert_eq!(parse_flex_basis("auto"), Some(FlexBasisValue::Auto));
        assert_eq!(parse_flex_basis("content"), Some(FlexBasisValue::Content));
        assert_eq!(
            parse_flex_basis("50%"),
            Some(FlexBasisValue::Length(LengthValue::Percentage(50.0)))
        );
        assert_eq!(parse_flex_basis("invalid-basis"), None);
    }

    #[test]
    /// parse_z_index 正负整数和 auto
    fn test_parse_z_index_variants() {
        assert_eq!(parse_z_index("auto"), Some(ZIndexValue::Auto));
        assert_eq!(parse_z_index("0"), Some(ZIndexValue::Integer(0)));
        assert_eq!(parse_z_index("9999"), Some(ZIndexValue::Integer(9999)));
        assert_eq!(parse_z_index("-999"), Some(ZIndexValue::Integer(-999)));
        assert_eq!(parse_z_index("abc"), None);
    }

    #[test]
    /// apply_property_value 对无效 display 值返回 false
    fn test_apply_property_invalid_display() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "display", "invalid"));
        assert!(!apply_property_value(&mut style, "display", ""));
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// apply_property_value 对 max-width: none 设置无穷大
    fn test_apply_property_max_width_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "max-width", "none"));
        assert_eq!(style.max_width, LengthValue::Px(f64::INFINITY));
    }

    #[test]
    /// apply_property_value 对 max-height: none 设置无穷大
    fn test_apply_property_max_height_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "max-height", "none"));
        assert_eq!(style.max_height, LengthValue::Px(f64::INFINITY));
    }

    #[test]
    /// apply_property_value 对 transform: none
    fn test_apply_property_transform_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "transform", "none"));
        assert_eq!(style.transform, zero_css_parser::values::TransformValue::None);
    }

    #[test]
    /// apply_property_value 对 aspect-ratio: auto 设置为 None
    fn test_apply_property_aspect_ratio_auto() {
        let mut style = ComputedStyle::default();
        style.aspect_ratio = Some(1.5);
        assert!(apply_property_value(&mut style, "aspect-ratio", "auto"));
        assert_eq!(style.aspect_ratio, None);
    }

    #[test]
    /// apply_property_value 对 aspect-ratio: 16/9
    fn test_apply_property_aspect_ratio_slash() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "aspect-ratio", "16 / 9"));
        let ratio = style.aspect_ratio.expect("should have ratio");
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    /// apply_property_value 对 aspect-ratio: 数值
    fn test_apply_property_aspect_ratio_number() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "aspect-ratio", "2"));
        assert_eq!(style.aspect_ratio, Some(2.0));
    }

    #[test]
    /// apply_property_value 对 aspect-ratio: 除零返回 false
    fn test_apply_property_aspect_ratio_divide_by_zero() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "aspect-ratio", "1 / 0"));
    }

    #[test]
    /// apply_property_value 对 vertical-align
    fn test_apply_property_vertical_align() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "vertical-align", "middle"));
        assert_eq!(style.vertical_align, VerticalAlignValue::Middle);

        assert!(apply_property_value(&mut style, "vertical-align", "top"));
        assert_eq!(style.vertical_align, VerticalAlignValue::Top);

        assert!(apply_property_value(&mut style, "vertical-align", "baseline"));
        assert_eq!(style.vertical_align, VerticalAlignValue::Baseline);
    }

    #[test]
    /// apply_property_value 对 grid-template-areas
    fn test_apply_property_grid_template_areas() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "grid-template-areas",
            "\"header header\" \"sidebar main\""
        ));
        assert_eq!(
            style.grid_template_areas,
            Some("\"header header\" \"sidebar main\"".to_string())
        );
    }

    #[test]
    /// apply_property_value 对未知属性返回 false
    fn test_apply_property_unknown() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "foobar", "baz"));
    }

    #[test]
    /// parse_font_family 空字符串过滤
    fn test_parse_font_family_empty_segments() {
        let families = parse_font_family(", , Arial, , sans-serif, ");
        assert_eq!(families, vec!["Arial", "sans-serif"]);
    }

    #[test]
    /// parse_font_family 单个字体
    fn test_parse_font_family_single() {
        let families = parse_font_family("monospace");
        assert_eq!(families, vec!["monospace"]);
    }

    #[test]
    /// parse_line_height 无单位零
    fn test_parse_line_height_zero() {
        assert_eq!(parse_line_height("0"), Some(LineHeightValue::Number(0.0)));
    }

    #[test]
    /// parse_grid_auto_flow 大小写不敏感
    fn test_parse_grid_auto_flow_case_insensitive() {
        assert_eq!(parse_grid_auto_flow("Row"), Some(GridAutoFlowValue::Row));
        assert_eq!(parse_grid_auto_flow("COLUMN"), Some(GridAutoFlowValue::Column));
        assert_eq!(parse_grid_auto_flow("Row Dense"), Some(GridAutoFlowValue::RowDense));
    }

    #[test]
    /// inherit_property 对不可继承属性返回 false
    fn test_inherit_property_returns_false_for_non_inheritable() {
        let parent = ComputedStyle::default();
        let mut child = ComputedStyle::default();
        assert!(!inherit_property(&parent, &mut child, "display"));
        assert!(!inherit_property(&parent, &mut child, "width"));
        assert!(!inherit_property(&parent, &mut child, "unknown-prop"));
    }

    #[test]
    /// apply_initial_value 对未知属性返回 false
    fn test_apply_initial_value_unknown() {
        let mut style = ComputedStyle::default();
        assert!(!apply_initial_value(&mut style, "unknown-prop"));
    }
}
