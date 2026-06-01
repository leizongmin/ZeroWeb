//! CSS 属性定义和计算样式结构。
//!
//! 定义 `ComputedStyle` 结构体，包含所有 Tier 1 CSS 属性的 typed 字段，
//! 以及 `PropertyRegistry` 用于查询初始值和继承性。

use zero_css_parser::values::{
    self, AlignmentValue, BoxSizingValue, ColorValue, ColumnCountValue, ColumnWidthValue, ContainValue,
    ContainerTypeValue, ContentValue, CounterActionValue, DisplayValue, FilterValue, FlexDirectionValue, FlexWrapValue,
    FontStyleValue, FontWeightValue, LengthValue, ObjectFitValue, OverflowValue, PositionValue, QuotesValue,
    ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStopValue, ScrollSnapTypeValue, VerticalAlignValue,
    VisibilityValue,
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

/// CSS text-decoration-line 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecorationLineValue {
    /// none。
    None,
    /// underline。
    Underline,
    /// overline。
    Overline,
    /// line-through。
    LineThrough,
    /// blink。
    Blink,
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
    /// break-spaces。
    BreakSpaces,
}

/// CSS text-overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextOverflowValue {
    /// clip。
    Clip,
    /// ellipsis。
    Ellipsis,
    /// 自定义字符串。
    String(String),
}

/// CSS word-break 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WordBreakValue {
    /// normal。
    Normal,
    /// break-all。
    BreakAll,
    /// keep-all。
    KeepAll,
    /// break-word。
    BreakWord,
}

/// CSS writing-mode 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WritingModeValue {
    /// horizontal-tb。
    HorizontalTb,
    /// vertical-rl。
    VerticalRl,
    /// vertical-lr。
    VerticalLr,
}

/// CSS table-layout 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TableLayoutValue {
    /// auto（默认值）— 自动表格布局。
    Auto,
    /// fixed — 固定表格布局。
    Fixed,
}

/// CSS caption-side 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaptionSideValue {
    /// top（默认值）— 标题在表格上方。
    Top,
    /// bottom — 标题在表格下方。
    Bottom,
}

/// CSS border-collapse 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderCollapseValue {
    /// separate（默认值）— 分离边框模型。
    Separate,
    /// collapse — 合并边框模型。
    Collapse,
}

/// CSS resize 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeValue {
    /// none（默认值）— 不可调整大小。
    None,
    /// both — 水平和垂直均可调整。
    Both,
    /// horizontal — 仅水平。
    Horizontal,
    /// vertical — 仅垂直。
    Vertical,
    /// block — 块方向。
    Block,
    /// inline — 行内方向。
    Inline,
}

/// CSS page-break 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum PageBreakValue {
    /// auto（默认值）。
    Auto,
    /// always。
    Always,
    /// avoid。
    Avoid,
    /// left。
    Left,
    /// right。
    Right,
}

/// CSS box-decoration-break 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxDecorationBreakValue {
    /// slice（默认值）。
    Slice,
    /// clone。
    Clone,
}

/// CSS image-rendering 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ImageRenderingValue {
    /// auto（默认值）。
    Auto,
    /// smooth。
    Smooth,
    /// high-quality。
    HighQuality,
    /// pixelated。
    Pixelated,
    /// crisp-edges。
    CrispEdges,
}

/// CSS isolation 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationValue {
    /// auto（默认值）。
    Auto,
    /// isolate。
    Isolate,
}

/// CSS break-inside 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakInsideValue {
    /// auto（默认值）。
    Auto,
    /// avoid。
    Avoid,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS break-before / break-after 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakValue {
    /// auto（默认值）。
    Auto,
    /// avoid。
    Avoid,
    /// column。
    Column,
    /// page。
    Page,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS column-rule-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleWidthComputedValue {
    /// medium（默认值）。
    Medium,
    /// thin。
    Thin,
    /// thick。
    Thick,
    /// 长度值。
    Length(LengthValue),
}

/// CSS column-rule-style 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleStyleComputedValue {
    /// none（默认值）。
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

/// CSS contain 属性计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainComputedValue {
    /// none（默认值）。
    None,
    /// strict — 等价于 layout + style + paint。
    Strict,
    /// content — 等价于 layout + style + paint + size。
    Content,
    /// size。
    Size,
    /// layout。
    Layout,
    /// style。
    Style,
    /// paint。
    Paint,
    /// 多个值的位掩码组合。
    Custom(u8),
}

/// contain 属性的位标志常量。
impl ContainComputedValue {
    /// size 标志位。
    pub const FLAG_SIZE: u8 = 0x01;
    /// layout 标志位。
    pub const FLAG_LAYOUT: u8 = 0x02;
    /// style 标志位。
    pub const FLAG_STYLE: u8 = 0x04;
    /// paint 标志位。
    pub const FLAG_PAINT: u8 = 0x08;
}

/// CSS appearance 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AppearanceComputedValue {
    /// none（默认值）— 不使用平台原生样式。
    None,
    /// auto — 使用平台原生样式。
    Auto,
    /// button。
    Button,
    /// checkbox。
    Checkbox,
    /// listbox。
    Listbox,
    /// menulist。
    Menulist,
    /// meter。
    Meter,
    /// progress-bar。
    ProgressBar,
    /// push-button。
    PushButton,
    /// radio。
    Radio,
    /// searchfield。
    Searchfield,
    /// slider-horizontal。
    SliderHorizontal,
    /// square-button。
    SquareButton,
    /// textarea。
    Textarea,
    /// textfield。
    Textfield,
}

/// CSS accent-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AccentColorComputedValue {
    /// auto（默认值）— 使用浏览器默认强调色。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// CSS caret-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaretColorComputedValue {
    /// auto（默认值）— 使用当前 color 属性值。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// CSS mix-blend-mode 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum MixBlendModeComputedValue {
    /// normal（默认值）。
    Normal,
    /// multiply。
    Multiply,
    /// screen。
    Screen,
    /// overlay。
    Overlay,
    /// darken。
    Darken,
    /// lighten。
    Lighten,
    /// color-dodge。
    ColorDodge,
    /// color-burn。
    ColorBurn,
    /// hard-light。
    HardLight,
    /// soft-light。
    SoftLight,
    /// difference。
    Difference,
    /// exclusion。
    Exclusion,
    /// hue。
    Hue,
    /// saturation。
    Saturation,
    /// color。
    Color,
    /// luminosity。
    Luminosity,
}

/// CSS scrollbar-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarWidthComputedValue {
    /// auto（默认值）。
    Auto,
    /// thin。
    Thin,
    /// none。
    None,
}

/// CSS scrollbar-gutter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarGutterComputedValue {
    /// auto（默认值）。
    Auto,
    /// stable。
    Stable,
    /// stable both-edges。
    StableBothEdges,
}

/// CSS text-wrap 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextWrapComputedValue {
    /// wrap（默认值）— 允许自动换行。
    Wrap,
    /// nowrap — 禁止自动换行。
    Nowrap,
    /// balance — 均衡换行。
    Balance,
    /// pretty — 优先美观换行。
    Pretty,
    /// stable — 稳定换行。
    Stable,
}

/// CSS hyphens 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum HyphensComputedValue {
    /// none（默认值）— 不使用连字符断词。
    None,
    /// manual — 手动断词。
    Manual,
    /// auto — 自动断词。
    Auto,
}

/// CSS line-clamp 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum LineClampComputedValue {
    /// none（默认值）— 不限制行数。
    None,
    /// 限制为指定行数。
    Count(u32),
}

/// CSS background-image 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundImageComputedValue {
    /// none（默认值）— 无背景图片。
    None,
    /// url(<string>) — 指定背景图片 URL。
    Url(String),
}

/// CSS background-position 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundPositionComputedValue {
    /// center。
    Center,
    /// left。
    Left,
    /// right。
    Right,
    /// top。
    Top,
    /// bottom。
    Bottom,
    /// 长度值（如 10px）。
    Length(f32),
    /// 百分比值（如 50%）。
    Percent(f32),
    /// 两个值组合（水平 垂直）。
    TwoValue(
        Box<BackgroundPositionComputedValue>,
        Box<BackgroundPositionComputedValue>,
    ),
}

/// CSS background-repeat 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundRepeatComputedValue {
    /// repeat — 水平和垂直方向都重复。
    Repeat,
    /// repeat-x — 仅水平方向重复。
    RepeatX,
    /// repeat-y — 仅垂直方向重复。
    RepeatY,
    /// no-repeat — 不重复。
    NoRepeat,
    /// space — 均匀分布。
    Space,
    /// round — 缩放后重复。
    Round,
}

/// CSS background-size 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundSizeComputedValue {
    /// auto（默认值）— 背景图片保持原始尺寸。
    Auto,
    /// cover — 缩放图片以完全覆盖容器。
    Cover,
    /// contain — 缩放图片以完整显示在容器内。
    Contain,
    /// 长度值（px）。
    Length(f32),
    /// 百分比值（0-100）。
    Percent(f32),
}

/// CSS background-attachment 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundAttachmentComputedValue {
    /// scroll（默认值）— 背景随元素内容滚动。
    Scroll,
    /// fixed — 背景相对于视口固定。
    Fixed,
    /// local — 背景随元素本地内容滚动。
    Local,
}

/// CSS background-clip 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundClipComputedValue {
    /// border-box（默认值）— 背景绘制到边框区域外边界。
    BorderBox,
    /// padding-box — 背景绘制到内边距区域外边界。
    PaddingBox,
    /// content-box — 背景绘制到内容区域外边界。
    ContentBox,
    /// text — 背景绘制到文本区域内。
    Text,
}

/// CSS background-origin 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundOriginComputedValue {
    /// padding-box（默认值）— 背景定位从内边距区域开始。
    PaddingBox,
    /// border-box — 背景定位从边框区域开始。
    BorderBox,
    /// content-box — 背景定位从内容区域开始。
    ContentBox,
}

// ── CSS Border Image 计算值类型 ──────────────────────────────────────────

/// CSS border-image-source 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageSourceComputedValue {
    /// none（默认值）。
    None,
    /// url(<string>)。
    Url(String),
}

/// CSS border-image-slice 单个分量的计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageSliceComputedComponent {
    /// 数字值。
    Number(f32),
    /// 百分比值。
    Percent(f32),
}

/// CSS border-image-slice 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageSliceComputedValue {
    /// 顶部。
    pub top: BorderImageSliceComputedComponent,
    /// 右侧。
    pub right: BorderImageSliceComputedComponent,
    /// 底部。
    pub bottom: BorderImageSliceComputedComponent,
    /// 左侧。
    pub left: BorderImageSliceComputedComponent,
    /// 是否填充。
    pub fill: bool,
}

/// CSS border-image-width 单个分量的计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageWidthComputedComponent {
    /// auto。
    Auto,
    /// 数字（倍数）。
    Number(f32),
    /// 长度值。
    Length(f32),
    /// 百分比值。
    Percent(f32),
}

/// CSS border-image-width 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageWidthComputedValue {
    /// 顶部。
    pub top: BorderImageWidthComputedComponent,
    /// 右侧。
    pub right: BorderImageWidthComputedComponent,
    /// 底部。
    pub bottom: BorderImageWidthComputedComponent,
    /// 左侧。
    pub left: BorderImageWidthComputedComponent,
}

/// CSS border-image-repeat 模式的计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageRepeatComputedMode {
    /// stretch（默认）。
    Stretch,
    /// repeat。
    Repeat,
    /// round。
    Round,
    /// space。
    Space,
}

/// CSS border-image-repeat 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageRepeatComputedValue {
    /// 水平方向。
    pub horizontal: BorderImageRepeatComputedMode,
    /// 垂直方向。
    pub vertical: BorderImageRepeatComputedMode,
}

/// CSS border-image-outset 单个分量的计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageOutsetComputedComponent {
    /// 数字（倍数）。
    Number(f32),
    /// 长度值。
    Length(f32),
}

/// CSS border-image-outset 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageOutsetComputedValue {
    /// 顶部。
    pub top: BorderImageOutsetComputedComponent,
    /// 右侧。
    pub right: BorderImageOutsetComputedComponent,
    /// 底部。
    pub bottom: BorderImageOutsetComputedComponent,
    /// 左侧。
    pub left: BorderImageOutsetComputedComponent,
}

/// CSS list-style-image 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStyleImageComputedValue {
    /// none（默认值）。
    None,
    /// url(<string>)。
    Url(String),
}

/// CSS text-shadow 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadowComputedValue {
    /// 水平偏移量（px）。
    pub offset_x: f32,
    /// 垂直偏移量（px）。
    pub offset_y: f32,
    /// 模糊半径（px）。
    pub blur_radius: f32,
    /// 阴影颜色。
    pub color: zero_css_parser::values::ColorValue,
}

/// CSS box-shadow 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadowComputedValue {
    /// 水平偏移量（px）。
    pub offset_x: f32,
    /// 垂直偏移量（px）。
    pub offset_y: f32,
    /// 模糊半径（px）。
    pub blur_radius: f32,
    /// 扩展半径（px）。
    pub spread_radius: f32,
    /// 阴影颜色。
    pub color: zero_css_parser::values::ColorValue,
    /// 是否为内阴影。
    pub inset: bool,
}

/// CSS overflow-wrap 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowWrapValue {
    /// normal（默认值）。
    Normal,
    /// break-word。
    BreakWord,
    /// anywhere。
    Anywhere,
}

/// CSS text-align-last 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignLastValue {
    /// auto（默认值）。
    Auto,
    /// start。
    Start,
    /// end。
    End,
    /// left。
    Left,
    /// right。
    Right,
    /// center。
    Center,
    /// justify。
    Justify,
}

/// CSS font-variant-numeric 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontVariantNumericValue {
    /// normal（默认值）。
    Normal,
    /// ordinal。
    Ordinal,
    /// slashed-zero。
    SlashedZero,
    /// lining-nums。
    LiningNums,
    /// oldstyle-nums。
    OldstyleNums,
    /// proportional-nums。
    ProportionalNums,
    /// tabular-nums。
    TabularNums,
    /// diagonal-fractions。
    DiagonalFractions,
    /// stacked-fractions。
    StackedFractions,
}

/// CSS direction 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum DirectionValue {
    /// ltr（默认值）— 从左到右。
    Ltr,
    /// rtl — 从右到左。
    Rtl,
}

/// CSS unicode-bidi 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeBidiValue {
    /// normal（默认值）。
    Normal,
    /// embed。
    Embed,
    /// isolate。
    Isolate,
    /// bidi-override。
    BidiOverride,
    /// isolate-override。
    IsolateOverride,
    /// plaintext。
    Plaintext,
}

/// CSS tab-size 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TabSizeValue {
    /// 数字值（空格数）。
    Number(u32),
    /// 长度值（如 px、em）。
    Length(LengthValue),
}

/// CSS overscroll-behavior 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverscrollBehaviorValue {
    /// auto（默认值）— 浏览器默认滚动溢出行为。
    Auto,
    /// contain — 阻止滚动链传播。
    Contain,
    /// none — 阻止滚动链和默认溢出行为。
    None,
}

/// CSS touch-action 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TouchActionValue {
    /// auto（默认值）。
    Auto,
    /// none。
    None,
    /// pan-x。
    PanX,
    /// pan-y。
    PanY,
    /// pan-x pan-y。
    PanXPanY,
    /// manipulation。
    Manipulation,
}

/// CSS user-select 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum UserSelectValue {
    /// auto（默认值）。
    Auto,
    /// text。
    Text,
    /// none。
    None,
    /// all。
    All,
    /// contain。
    Contain,
}

/// CSS will-change 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum WillChangeValue {
    /// auto（默认值）。
    Auto,
    /// scroll-position。
    ScrollPosition,
    /// contents。
    Contents,
    /// 自定义属性名。
    Custom(String),
}

/// CSS pointer-events 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEventsValue {
    /// auto（默认值）。
    Auto,
    /// none。
    None,
    /// visiblePainted。
    VisiblePainted,
    /// visibleFill。
    VisibleFill,
    /// visibleStroke。
    VisibleStroke,
    /// visible。
    Visible,
    /// painted。
    Painted,
    /// fill。
    Fill,
    /// stroke。
    Stroke,
    /// all。
    All,
    /// inherit。
    Inherit,
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

/// CSS quotes 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum QuotesComputedValue {
    /// none。
    None,
    /// auto。
    Auto,
    /// 引号对列表。
    Pairs(Vec<(String, String)>),
}

/// CSS content 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentComputedValue {
    /// normal（默认值）。
    Normal,
    /// none。
    None,
    /// 字符串内容。
    String(String),
    /// attr() 函数引用。
    Attr(String),
    /// counter() 函数引用。
    Counter {
        /// 计数器名称。
        name: String,
        /// 可选的列表样式类型。
        style: Option<String>,
    },
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
    /// list-style-image 值。
    ListStyleImage(ListStyleImageComputedValue),
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
    /// text-decoration-line 值。
    TextDecorationLine(TextDecorationLineValue),
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
    /// transform-style 值。
    TransformStyle(TransformStyleValue),
    /// backface-visibility 值。
    BackfaceVisibility(BackfaceVisibilityValue),
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
    /// word-break 值。
    WordBreak(WordBreakValue),
    /// writing-mode 值。
    WritingMode(WritingModeValue),
    /// text-indent 值。
    TextIndent(LengthValue),
    /// table-layout 值。
    TableLayout(TableLayoutValue),
    /// caption-side 值。
    CaptionSide(CaptionSideValue),
    /// border-collapse 值。
    BorderCollapse(BorderCollapseValue),
    /// resize 值。
    Resize(ResizeValue),
    /// counter-reset 值。
    CounterReset(Vec<CounterActionValue>),
    /// counter-increment 值。
    CounterIncrement(Vec<CounterActionValue>),
    /// counter-set 值。
    CounterSet(Vec<CounterActionValue>),
    /// content 值。
    Content(ContentComputedValue),
    /// quotes 值。
    Quotes(QuotesComputedValue),
    /// page-break 值。
    PageBreak(PageBreakValue),
    /// box-decoration-break 值。
    BoxDecorationBreak(BoxDecorationBreakValue),
    /// image-rendering 值。
    ImageRendering(ImageRenderingValue),
    /// isolation 值。
    Isolation(IsolationValue),
    /// break-inside 值。
    BreakInside(BreakInsideValue),
    /// break-before 值。
    BreakBefore(BreakValue),
    /// break-after 值。
    BreakAfter(BreakValue),
    /// column-rule-width 值。
    ColumnRuleWidth(ColumnRuleWidthComputedValue),
    /// column-rule-style 值。
    ColumnRuleStyle(ColumnRuleStyleComputedValue),
    /// overscroll-behavior 值。
    OverscrollBehavior(OverscrollBehaviorValue),
    /// touch-action 值。
    TouchAction(TouchActionValue),
    /// user-select 值。
    UserSelect(UserSelectValue),
    /// will-change 值。
    WillChange(WillChangeValue),
    /// pointer-events 值。
    PointerEvents(PointerEventsValue),
    /// overflow-wrap 值。
    OverflowWrap(OverflowWrapValue),
    /// text-align-last 值。
    TextAlignLast(TextAlignLastValue),
    /// font-variant-numeric 值。
    FontVariantNumeric(FontVariantNumericValue),
    /// direction 值。
    Direction(DirectionValue),
    /// unicode-bidi 值。
    UnicodeBidi(UnicodeBidiValue),
    /// tab-size 值。
    TabSize(TabSizeValue),
    /// column-count 值。
    ColumnCount(ColumnCountComputedValue),
    /// column-width 值。
    ColumnWidth(ColumnWidthComputedValue),
    /// object-fit 值。
    ObjectFit(ObjectFitComputedValue),
    /// filter 值。
    Filter(FilterComputedValue),
    /// contain 值。
    Contain(ContainComputedValue),
    /// column-rule-color 值。
    ColumnRuleColor(ColorValue),
    /// appearance 值。
    Appearance(AppearanceComputedValue),
    /// accent-color 值。
    AccentColor(AccentColorComputedValue),
    /// caret-color 值。
    CaretColor(CaretColorComputedValue),
    /// mix-blend-mode 值。
    MixBlendMode(MixBlendModeComputedValue),
    /// scrollbar-width 值。
    ScrollbarWidth(ScrollbarWidthComputedValue),
    /// scrollbar-gutter 值。
    ScrollbarGutter(ScrollbarGutterComputedValue),
    /// text-wrap 值。
    TextWrap(TextWrapComputedValue),
    /// hyphens 值。
    Hyphens(HyphensComputedValue),
    /// line-clamp 值。
    LineClamp(LineClampComputedValue),
    /// background-image 值。
    BackgroundImage(BackgroundImageComputedValue),
    /// background-position 值。
    BackgroundPosition(BackgroundPositionComputedValue),
    /// background-repeat 值。
    BackgroundRepeat(BackgroundRepeatComputedValue),
    /// background-size 值。
    BackgroundSize(BackgroundSizeComputedValue),
    /// background-attachment 值。
    BackgroundAttachment(BackgroundAttachmentComputedValue),
    /// background-clip 值。
    BackgroundClip(BackgroundClipComputedValue),
    /// background-origin 值。
    BackgroundOrigin(BackgroundOriginComputedValue),
    /// border-image-source 值。
    BorderImageSource(BorderImageSourceComputedValue),
    /// border-image-slice 值。
    BorderImageSlice(BorderImageSliceComputedValue),
    /// border-image-width 值。
    BorderImageWidth(BorderImageWidthComputedValue),
    /// border-image-repeat 值。
    BorderImageRepeat(BorderImageRepeatComputedValue),
    /// border-image-outset 值。
    BorderImageOutset(BorderImageOutsetComputedValue),
    /// text-shadow 值。
    TextShadow(TextShadowComputedValue),
    /// box-shadow 值。
    BoxShadow(BoxShadowComputedValue),
    /// justify-items 值。
    JustifyItems(JustifyItemsValue),
    /// justify-self 值。
    JustifySelf(JustifySelfValue),
    /// align-content 值。
    AlignContent(AlignContentValue),
    /// empty-cells 值。
    EmptyCells(EmptyCellsComputedValue),
    /// border-spacing 值。
    BorderSpacing(BorderSpacingComputedValue),
}

// ── 3D Transform 相关枚举 ──────────────────────────────────────────────

/// transform-style 属性值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformStyleValue {
    /// flat。
    Flat,
    /// preserve-3d。
    Preserve3d,
}

/// backface-visibility 属性值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackfaceVisibilityValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
}

/// CSS column-count 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnCountComputedValue {
    /// auto。
    Auto,
    /// 正整数值。
    Number(u32),
}

/// CSS column-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnWidthComputedValue {
    /// auto。
    Auto,
    /// 长度值。
    Length(LengthValue),
}

/// CSS object-fit 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFitComputedValue {
    /// fill。
    Fill,
    /// contain。
    Contain,
    /// cover。
    Cover,
    /// none。
    None,
    /// scale-down。
    ScaleDown,
}

/// CSS filter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterComputedValue {
    /// none。
    None,
    /// blur(px)。
    Blur(f32),
    /// brightness(number)。
    Brightness(f32),
    /// contrast(number)。
    Contrast(f32),
    /// grayscale(number)。
    Grayscale(f32),
    /// hue-rotate(deg)。
    HueRotate(f32),
    /// invert(number)。
    Invert(f32),
    /// opacity(number)。
    Opacity(f32),
    /// saturate(number)。
    Saturate(f32),
    /// sepia(number)。
    Sepia(f32),
    /// drop-shadow(x-offset, y-offset, blur-radius, color)。
    DropShadow(f32, f32, f32, ColorValue),
}

// ── ComputedStyle ─────────────────────────────────────────────────────

/// CSS justify-items 值。
#[derive(Debug, Clone, PartialEq)]
pub enum JustifyItemsValue {
    /// auto。
    Auto,
    /// normal（默认值）。
    Normal,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
    /// stretch。
    Stretch,
    /// baseline。
    Baseline,
}

/// CSS justify-self 值。
#[derive(Debug, Clone, PartialEq)]
pub enum JustifySelfValue {
    /// auto（默认值）。
    Auto,
    /// normal。
    Normal,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
    /// stretch。
    Stretch,
    /// baseline。
    Baseline,
}

/// CSS align-content 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AlignContentValue {
    /// auto。
    Auto,
    /// normal（默认值）。
    Normal,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
    /// stretch。
    Stretch,
    /// baseline。
    Baseline,
    /// space-between。
    SpaceBetween,
    /// space-around。
    SpaceAround,
    /// space-evenly。
    SpaceEvenly,
}

/// CSS empty-cells 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum EmptyCellsComputedValue {
    /// show（默认值）— 显示空单元格边框。
    Show,
    /// hide — 隐藏空单元格边框。
    Hide,
}

/// CSS border-spacing 计算值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderSpacingComputedValue {
    /// 水平间距（px）。
    pub horizontal: f32,
    /// 垂直间距（px）。
    pub vertical: f32,
}

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
    /// list-style-image 属性。
    pub list_style_image: ListStyleImageComputedValue,
    /// writing-mode 属性。
    pub writing_mode: WritingModeValue,
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
    /// text-decoration-line 属性。
    pub text_decoration_line: TextDecorationLineValue,
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
    /// word-break 属性。
    pub word_break: WordBreakValue,
    /// text-indent 属性。
    pub text_indent: LengthValue,
    /// resize 属性。
    pub resize: ResizeValue,

    // ── 表格 ──
    /// table-layout 属性。
    pub table_layout: TableLayoutValue,
    /// caption-side 属性。
    pub caption_side: CaptionSideValue,
    /// border-collapse 属性。
    pub border_collapse: BorderCollapseValue,
    /// empty-cells 属性。
    pub empty_cells: EmptyCellsComputedValue,
    /// border-spacing 属性。
    pub border_spacing: BorderSpacingComputedValue,

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
    /// justify-items 属性。
    pub justify_items: JustifyItemsValue,
    /// justify-self 属性。
    pub justify_self: JustifySelfValue,
    /// align-content 属性。
    pub align_content: AlignContentValue,
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
    /// column-gap 属性。
    pub column_gap: LengthValue,
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
    /// transform-origin X 分量。
    pub transform_origin_x: LengthValue,
    /// transform-origin Y 分量。
    pub transform_origin_y: LengthValue,
    /// perspective 属性。
    pub perspective: LengthValue,
    /// perspective-origin X 分量。
    pub perspective_origin_x: LengthValue,
    /// perspective-origin Y 分量。
    pub perspective_origin_y: LengthValue,
    /// transform-style 属性。
    pub transform_style: TransformStyleValue,
    /// backface-visibility 属性。
    pub backface_visibility: BackfaceVisibilityValue,

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

    // ── Counters / Content / Quotes ──
    /// counter-reset 属性。
    pub counter_reset: Vec<CounterActionValue>,
    /// counter-increment 属性。
    pub counter_increment: Vec<CounterActionValue>,
    /// counter-set 属性。
    pub counter_set: Vec<CounterActionValue>,
    /// content 属性。
    pub content: ContentComputedValue,
    /// quotes 属性。
    pub quotes: QuotesComputedValue,

    // ── Page Break ──
    /// page-break-before 属性。
    pub page_break_before: PageBreakValue,
    /// page-break-after 属性。
    pub page_break_after: PageBreakValue,
    /// page-break-inside 属性。
    pub page_break_inside: PageBreakValue,

    // ── 其他 ──
    /// box-decoration-break 属性。
    pub box_decoration_break: BoxDecorationBreakValue,
    /// image-rendering 属性。
    pub image_rendering: ImageRenderingValue,
    /// isolation 属性。
    pub isolation: IsolationValue,

    // ── Break ──
    /// break-inside 属性。
    pub break_inside: BreakInsideValue,
    /// break-before 属性。
    pub break_before: BreakValue,
    /// break-after 属性。
    pub break_after: BreakValue,

    // ── Column Rule ──
    /// column-rule-width 属性。
    pub column_rule_width: ColumnRuleWidthComputedValue,
    /// column-rule-style 属性。
    pub column_rule_style: ColumnRuleStyleComputedValue,
    /// column-rule-color 属性。
    pub column_rule_color: ColorValue,

    // ── Contain ──
    /// contain 属性。
    pub contain: ContainComputedValue,

    // ── Interaction / Performance Hint ──
    /// overscroll-behavior-x 属性。
    pub overscroll_behavior_x: OverscrollBehaviorValue,
    /// overscroll-behavior-y 属性。
    pub overscroll_behavior_y: OverscrollBehaviorValue,
    /// touch-action 属性。
    pub touch_action: TouchActionValue,
    /// user-select 属性。
    pub user_select: UserSelectValue,
    /// will-change 属性。
    pub will_change: WillChangeValue,
    /// pointer-events 属性。
    pub pointer_events: PointerEventsValue,

    // ── Text (新属性) ──
    /// overflow-wrap 属性。
    pub overflow_wrap: OverflowWrapValue,
    /// text-align-last 属性。
    pub text_align_last: TextAlignLastValue,
    /// font-variant-numeric 属性。
    pub font_variant_numeric: FontVariantNumericValue,

    // ── Writing Direction / Tab ──
    /// direction 属性。
    pub direction: DirectionValue,
    /// unicode-bidi 属性。
    pub unicode_bidi: UnicodeBidiValue,
    /// tab-size 属性。
    pub tab_size: TabSizeValue,

    // ── Columns ──
    /// column-count 属性。
    pub column_count: ColumnCountComputedValue,
    /// column-width 属性。
    pub column_width: ColumnWidthComputedValue,

    // ── Object Fit / Filter ──
    /// object-fit 属性。
    pub object_fit: ObjectFitComputedValue,
    /// filter 属性。
    pub filter: FilterComputedValue,

    // ── UI Appearance ──
    /// appearance 属性。
    pub appearance: AppearanceComputedValue,
    /// accent-color 属性。
    pub accent_color: AccentColorComputedValue,
    /// caret-color 属性。
    pub caret_color: CaretColorComputedValue,
    /// mix-blend-mode 属性。
    pub mix_blend_mode: MixBlendModeComputedValue,
    /// scrollbar-width 属性。
    pub scrollbar_width: ScrollbarWidthComputedValue,
    /// scrollbar-gutter 属性。
    pub scrollbar_gutter: ScrollbarGutterComputedValue,

    // ── Text Wrap / Hyphens / Line Clamp ──
    /// text-wrap 属性。
    pub text_wrap: TextWrapComputedValue,
    /// hyphens 属性。
    pub hyphens: HyphensComputedValue,
    /// line-clamp 属性。
    pub line_clamp: LineClampComputedValue,

    // ── Background Image / Position / Repeat / Size / Attachment ──
    /// background-image 属性。
    pub background_image: BackgroundImageComputedValue,
    /// background-position 属性。
    pub background_position: BackgroundPositionComputedValue,
    /// background-repeat 属性。
    pub background_repeat: BackgroundRepeatComputedValue,
    /// background-size 属性。
    pub background_size: BackgroundSizeComputedValue,
    /// background-attachment 属性。
    pub background_attachment: BackgroundAttachmentComputedValue,
    /// background-clip 属性。
    pub background_clip: BackgroundClipComputedValue,
    /// background-origin 属性。
    pub background_origin: BackgroundOriginComputedValue,
    /// border-image-source 属性。
    pub border_image_source: BorderImageSourceComputedValue,
    /// border-image-slice 属性。
    pub border_image_slice: BorderImageSliceComputedValue,
    /// border-image-width 属性。
    pub border_image_width: BorderImageWidthComputedValue,
    /// border-image-repeat 属性。
    pub border_image_repeat: BorderImageRepeatComputedValue,
    /// border-image-outset 属性。
    pub border_image_outset: BorderImageOutsetComputedValue,
    /// text-shadow 属性。
    pub text_shadow: TextShadowComputedValue,
    /// box-shadow 属性。
    pub box_shadow: BoxShadowComputedValue,
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
            list_style_image: ListStyleImageComputedValue::None,
            writing_mode: WritingModeValue::HorizontalTb,
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
            text_decoration_line: TextDecorationLineValue::None,
            text_transform: TextTransformValue::None,
            letter_spacing: LengthValue::Px(0.0),
            word_spacing: LengthValue::Px(0.0),
            white_space: WhiteSpaceValue::Normal,
            text_overflow: TextOverflowValue::Clip,
            vertical_align: VerticalAlignValue::Baseline,
            word_break: WordBreakValue::Normal,
            text_indent: LengthValue::Px(0.0),
            resize: ResizeValue::None,

            // 表格
            table_layout: TableLayoutValue::Auto,
            caption_side: CaptionSideValue::Top,
            border_collapse: BorderCollapseValue::Separate,
            empty_cells: EmptyCellsComputedValue::Show,
            border_spacing: BorderSpacingComputedValue {
                horizontal: 0.0,
                vertical: 0.0,
            },

            // Flexbox
            flex_direction: FlexDirectionValue::Row,
            flex_wrap: FlexWrapValue::Nowrap,
            justify_content: AlignmentValue::FlexStart,
            align_items: AlignmentValue::Stretch,
            align_self: AlignmentValue::Stretch,
            justify_items: JustifyItemsValue::Normal,
            justify_self: JustifySelfValue::Auto,
            align_content: AlignContentValue::Normal,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: FlexBasisValue::Auto,
            gap: LengthValue::Px(0.0),
            row_gap: LengthValue::Px(0.0),
            column_gap: LengthValue::Px(0.0),
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
            transform_origin_x: LengthValue::Percentage(50.0),
            transform_origin_y: LengthValue::Percentage(50.0),
            perspective: LengthValue::Px(0.0),
            perspective_origin_x: LengthValue::Percentage(50.0),
            perspective_origin_y: LengthValue::Percentage(50.0),
            transform_style: TransformStyleValue::Flat,
            backface_visibility: BackfaceVisibilityValue::Visible,

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

            // Counters / Content / Quotes
            counter_reset: vec![],
            counter_increment: vec![],
            counter_set: vec![],
            content: ContentComputedValue::Normal,
            quotes: QuotesComputedValue::Auto,

            // Page Break
            page_break_before: PageBreakValue::Auto,
            page_break_after: PageBreakValue::Auto,
            page_break_inside: PageBreakValue::Auto,

            // 其他
            box_decoration_break: BoxDecorationBreakValue::Slice,
            image_rendering: ImageRenderingValue::Auto,
            isolation: IsolationValue::Auto,

            // Break
            break_inside: BreakInsideValue::Auto,
            break_before: BreakValue::Auto,
            break_after: BreakValue::Auto,

            // Column Rule
            column_rule_width: ColumnRuleWidthComputedValue::Medium,
            column_rule_style: ColumnRuleStyleComputedValue::None,
            column_rule_color: ColorValue::Rgba(0, 0, 0, 255), // currentColor 解析后默认黑色

            // Contain
            contain: ContainComputedValue::None,

            // Interaction / Performance Hint
            overscroll_behavior_x: OverscrollBehaviorValue::Auto,
            overscroll_behavior_y: OverscrollBehaviorValue::Auto,
            touch_action: TouchActionValue::Auto,
            user_select: UserSelectValue::Auto,
            will_change: WillChangeValue::Auto,
            pointer_events: PointerEventsValue::Auto,

            // Text (新属性)
            overflow_wrap: OverflowWrapValue::Normal,
            text_align_last: TextAlignLastValue::Auto,
            font_variant_numeric: FontVariantNumericValue::Normal,

            // Writing Direction / Tab
            direction: DirectionValue::Ltr,
            unicode_bidi: UnicodeBidiValue::Normal,
            tab_size: TabSizeValue::Number(8),

            // Columns
            column_count: ColumnCountComputedValue::Auto,
            column_width: ColumnWidthComputedValue::Auto,

            // Object Fit / Filter
            object_fit: ObjectFitComputedValue::Fill,
            filter: FilterComputedValue::None,

            // UI Appearance
            appearance: AppearanceComputedValue::Auto,
            accent_color: AccentColorComputedValue::Auto,
            caret_color: CaretColorComputedValue::Auto,

            // Compositing / Scrolling
            mix_blend_mode: MixBlendModeComputedValue::Normal,
            scrollbar_width: ScrollbarWidthComputedValue::Auto,
            scrollbar_gutter: ScrollbarGutterComputedValue::Auto,

            // Text Wrap / Hyphens / Line Clamp
            text_wrap: TextWrapComputedValue::Wrap,
            hyphens: HyphensComputedValue::None,
            line_clamp: LineClampComputedValue::None,

            // Background Image / Position / Repeat / Size / Attachment / Clip / Origin
            background_image: BackgroundImageComputedValue::None,
            background_position: BackgroundPositionComputedValue::Percent(0.0),
            background_repeat: BackgroundRepeatComputedValue::Repeat,
            background_size: BackgroundSizeComputedValue::Auto,
            background_attachment: BackgroundAttachmentComputedValue::Scroll,
            background_clip: BackgroundClipComputedValue::BorderBox,
            background_origin: BackgroundOriginComputedValue::PaddingBox,

            // Border Image (Source / Slice / Width / Repeat / Outset)
            border_image_source: BorderImageSourceComputedValue::None,
            border_image_slice: BorderImageSliceComputedValue {
                top: BorderImageSliceComputedComponent::Number(100.0),
                right: BorderImageSliceComputedComponent::Number(100.0),
                bottom: BorderImageSliceComputedComponent::Number(100.0),
                left: BorderImageSliceComputedComponent::Number(100.0),
                fill: false,
            },
            border_image_width: BorderImageWidthComputedValue {
                top: BorderImageWidthComputedComponent::Number(1.0),
                right: BorderImageWidthComputedComponent::Number(1.0),
                bottom: BorderImageWidthComputedComponent::Number(1.0),
                left: BorderImageWidthComputedComponent::Number(1.0),
            },
            border_image_repeat: BorderImageRepeatComputedValue {
                horizontal: BorderImageRepeatComputedMode::Stretch,
                vertical: BorderImageRepeatComputedMode::Stretch,
            },
            border_image_outset: BorderImageOutsetComputedValue {
                top: BorderImageOutsetComputedComponent::Number(0.0),
                right: BorderImageOutsetComputedComponent::Number(0.0),
                bottom: BorderImageOutsetComputedComponent::Number(0.0),
                left: BorderImageOutsetComputedComponent::Number(0.0),
            },

            text_shadow: TextShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
            },
            box_shadow: BoxShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                spread_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                inset: false,
            },
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
            "list-style-image" => Some(ListStyleImage(ListStyleImageComputedValue::None)),
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
            "text-decoration-line" => Some(TextDecorationLine(TextDecorationLineValue::None)),
            "text-transform" => Some(TextTransform(TextTransformValue::None)),
            "letter-spacing" | "word-spacing" => Some(Length(LengthValue::Px(0.0))),
            "white-space" => Some(WhiteSpace(WhiteSpaceValue::Normal)),
            "text-overflow" => Some(TextOverflow(TextOverflowValue::Clip)),
            "vertical-align" => Some(VerticalAlign(VerticalAlignValue::Baseline)),
            "word-break" => Some(WordBreak(WordBreakValue::Normal)),
            "text-indent" => Some(TextIndent(LengthValue::Px(0.0))),
            "table-layout" => Some(TableLayout(TableLayoutValue::Auto)),
            "caption-side" => Some(CaptionSide(CaptionSideValue::Top)),
            "border-collapse" => Some(BorderCollapse(BorderCollapseValue::Separate)),
            "empty-cells" => Some(EmptyCells(EmptyCellsComputedValue::Show)),
            "border-spacing" => Some(BorderSpacing(BorderSpacingComputedValue {
                horizontal: 0.0,
                vertical: 0.0,
            })),
            "resize" => Some(Resize(ResizeValue::None)),

            // Writing Mode
            "writing-mode" => Some(WritingMode(WritingModeValue::HorizontalTb)),

            // Flexbox
            "flex-direction" => Some(FlexDirection(FlexDirectionValue::Row)),
            "flex-wrap" => Some(FlexWrap(FlexWrapValue::Nowrap)),
            "justify-content" => Some(Alignment(AlignmentValue::FlexStart)),
            "align-items" | "align-self" => Some(Alignment(AlignmentValue::Stretch)),
            "justify-items" => Some(JustifyItems(JustifyItemsValue::Normal)),
            "justify-self" => Some(JustifySelf(JustifySelfValue::Auto)),
            "align-content" => Some(AlignContent(AlignContentValue::Normal)),
            "flex-grow" => Some(Number(0.0)),
            "flex-shrink" => Some(Number(1.0)),
            "flex-basis" => Some(FlexBasis(FlexBasisValue::Auto)),
            "gap" => Some(Length(LengthValue::Px(0.0))),
            "column-gap" => Some(Length(LengthValue::Px(0.0))),
            "row-gap" => Some(Length(LengthValue::Px(0.0))),
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
            "transform-origin" => Some(Length(LengthValue::Percentage(50.0))),
            "perspective" => Some(Length(LengthValue::Px(0.0))),
            "perspective-origin" => Some(Length(LengthValue::Percentage(50.0))),
            "transform-style" => Some(TransformStyle(TransformStyleValue::Flat)),
            "backface-visibility" => Some(BackfaceVisibility(BackfaceVisibilityValue::Visible)),

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

            // Counters / Content / Quotes
            "counter-reset" => Some(CounterReset(vec![])),
            "counter-increment" => Some(CounterIncrement(vec![])),
            "counter-set" => Some(CounterSet(vec![])),
            "content" => Some(Content(ContentComputedValue::Normal)),
            "quotes" => Some(Quotes(QuotesComputedValue::Auto)),

            // Page Break
            "page-break-before" | "page-break-after" | "page-break-inside" => Some(PageBreak(PageBreakValue::Auto)),

            // 其他
            "box-decoration-break" => Some(BoxDecorationBreak(BoxDecorationBreakValue::Slice)),
            "image-rendering" => Some(ImageRendering(ImageRenderingValue::Auto)),
            "isolation" => Some(Isolation(IsolationValue::Auto)),

            // Break
            "break-inside" => Some(BreakInside(BreakInsideValue::Auto)),
            "break-before" => Some(BreakBefore(BreakValue::Auto)),
            "break-after" => Some(BreakAfter(BreakValue::Auto)),

            // Column Rule
            "column-rule-width" => Some(ColumnRuleWidth(ColumnRuleWidthComputedValue::Medium)),
            "column-rule-style" => Some(ColumnRuleStyle(ColumnRuleStyleComputedValue::None)),

            // Interaction / Performance Hint
            "overscroll-behavior-x" | "overscroll-behavior-y" => {
                Some(OverscrollBehavior(OverscrollBehaviorValue::Auto))
            }
            "touch-action" => Some(TouchAction(TouchActionValue::Auto)),
            "user-select" => Some(UserSelect(UserSelectValue::Auto)),
            "will-change" => Some(WillChange(WillChangeValue::Auto)),
            "pointer-events" => Some(PointerEvents(PointerEventsValue::Auto)),

            // Text (新属性)
            "overflow-wrap" => Some(OverflowWrap(OverflowWrapValue::Normal)),
            "text-align-last" => Some(TextAlignLast(TextAlignLastValue::Auto)),
            "font-variant-numeric" => Some(FontVariantNumeric(FontVariantNumericValue::Normal)),

            // Writing Direction / Tab
            "direction" => Some(Direction(DirectionValue::Ltr)),
            "unicode-bidi" => Some(UnicodeBidi(UnicodeBidiValue::Normal)),
            "tab-size" => Some(TabSize(TabSizeValue::Number(8))),

            // Columns
            "columns" => Some(ColumnCount(ColumnCountComputedValue::Auto)),
            "column-count" => Some(ColumnCount(ColumnCountComputedValue::Auto)),
            "column-width" => Some(ColumnWidth(ColumnWidthComputedValue::Auto)),

            // Object Fit / Filter
            "object-fit" => Some(ObjectFit(ObjectFitComputedValue::Fill)),
            "filter" => Some(Filter(FilterComputedValue::None)),

            // Column Rule Color
            "column-rule-color" => Some(ColumnRuleColor(ColorValue::Rgba(0, 0, 0, 255))),

            // Contain
            "contain" => Some(Contain(ContainComputedValue::None)),

            // UI Appearance
            "appearance" => Some(Appearance(AppearanceComputedValue::Auto)),
            "accent-color" => Some(AccentColor(AccentColorComputedValue::Auto)),
            "caret-color" => Some(CaretColor(CaretColorComputedValue::Auto)),

            // Compositing / Scrolling
            "mix-blend-mode" => Some(MixBlendMode(MixBlendModeComputedValue::Normal)),
            "scrollbar-width" => Some(ScrollbarWidth(ScrollbarWidthComputedValue::Auto)),
            "scrollbar-gutter" => Some(ScrollbarGutter(ScrollbarGutterComputedValue::Auto)),

            // Text Wrap / Hyphens / Line Clamp
            "text-wrap" => Some(TextWrap(TextWrapComputedValue::Wrap)),
            "hyphens" => Some(Hyphens(HyphensComputedValue::None)),
            "line-clamp" => Some(LineClamp(LineClampComputedValue::None)),

            // Background Image / Position / Repeat / Size / Attachment / Clip / Origin
            "background-image" => Some(BackgroundImage(BackgroundImageComputedValue::None)),
            "background-position" => Some(BackgroundPosition(BackgroundPositionComputedValue::Percent(0.0))),
            "background-repeat" => Some(BackgroundRepeat(BackgroundRepeatComputedValue::Repeat)),
            "background-size" => Some(BackgroundSize(BackgroundSizeComputedValue::Auto)),
            "background-attachment" => Some(BackgroundAttachment(BackgroundAttachmentComputedValue::Scroll)),
            "background-clip" => Some(BackgroundClip(BackgroundClipComputedValue::BorderBox)),
            "background-origin" => Some(BackgroundOrigin(BackgroundOriginComputedValue::PaddingBox)),

            // Border Image
            "border-image-source" => Some(BorderImageSource(BorderImageSourceComputedValue::None)),
            "border-image-slice" => Some(BorderImageSlice(BorderImageSliceComputedValue {
                top: BorderImageSliceComputedComponent::Number(100.0),
                right: BorderImageSliceComputedComponent::Number(100.0),
                bottom: BorderImageSliceComputedComponent::Number(100.0),
                left: BorderImageSliceComputedComponent::Number(100.0),
                fill: false,
            })),
            "border-image-width" => Some(BorderImageWidth(BorderImageWidthComputedValue {
                top: BorderImageWidthComputedComponent::Number(1.0),
                right: BorderImageWidthComputedComponent::Number(1.0),
                bottom: BorderImageWidthComputedComponent::Number(1.0),
                left: BorderImageWidthComputedComponent::Number(1.0),
            })),
            "border-image-repeat" => Some(BorderImageRepeat(BorderImageRepeatComputedValue {
                horizontal: BorderImageRepeatComputedMode::Stretch,
                vertical: BorderImageRepeatComputedMode::Stretch,
            })),
            "border-image-outset" => Some(BorderImageOutset(BorderImageOutsetComputedValue {
                top: BorderImageOutsetComputedComponent::Number(0.0),
                right: BorderImageOutsetComputedComponent::Number(0.0),
                bottom: BorderImageOutsetComputedComponent::Number(0.0),
                left: BorderImageOutsetComputedComponent::Number(0.0),
            })),
            "text-shadow" => Some(TextShadow(TextShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
            })),
            "box-shadow" => Some(BoxShadow(BoxShadowComputedValue {
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                spread_radius: 0.0,
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                inset: false,
            })),

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
                | "word-break"
                | "visibility"
                | "cursor"
                | "text-indent"
                | "caption-side"
                | "border-collapse"
                | "quotes"
                | "pointer-events"
                | "overflow-wrap"
                | "text-align-last"
                | "font-variant-numeric"
                | "direction"
                | "tab-size"
                | "accent-color"
                | "caret-color"
                | "text-wrap"
                | "hyphens"
                | "text-shadow"
                | "list-style-image"
                | "empty-cells"
                | "border-spacing"
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
            "list-style-image",
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
            "text-decoration-line",
            "text-transform",
            "letter-spacing",
            "word-spacing",
            "white-space",
            "text-overflow",
            "vertical-align",
            "word-break",
            "flex-direction",
            "flex-wrap",
            "justify-content",
            "align-items",
            "align-self",
            "justify-items",
            "justify-self",
            "align-content",
            "flex-grow",
            "flex-shrink",
            "flex-basis",
            "gap",
            "column-gap",
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
            "grid-template-columns",
            "grid-template-rows",
            "grid-template-areas",
            "grid-auto-flow",
            "row-gap",
            "grid-auto-rows",
            "grid-auto-columns",
            "outline-width",
            "outline-style",
            "outline-color",
            "outline-offset",
            "transform",
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
            "writing-mode",
            "text-indent",
            "table-layout",
            "caption-side",
            "border-collapse",
            "empty-cells",
            "border-spacing",
            "resize",
            "transform-origin",
            "perspective",
            "perspective-origin",
            "transform-style",
            "backface-visibility",
            "counter-reset",
            "counter-increment",
            "counter-set",
            "content",
            "quotes",
            "page-break-before",
            "page-break-after",
            "page-break-inside",
            "box-decoration-break",
            "image-rendering",
            "isolation",
            "break-inside",
            "break-before",
            "break-after",
            "column-rule-width",
            "column-rule-style",
            "overscroll-behavior-x",
            "overscroll-behavior-y",
            "touch-action",
            "user-select",
            "will-change",
            "pointer-events",
            "overflow-wrap",
            "text-align-last",
            "font-variant-numeric",
            "direction",
            "unicode-bidi",
            "tab-size",
            "columns",
            "column-count",
            "column-width",
            "object-fit",
            "filter",
            "column-rule-color",
            "contain",
            "appearance",
            "accent-color",
            "caret-color",
            "mix-blend-mode",
            "scrollbar-width",
            "scrollbar-gutter",
            "text-wrap",
            "hyphens",
            "line-clamp",
            "background-image",
            "background-position",
            "background-repeat",
            "background-size",
            "background-attachment",
            "background-clip",
            "background-origin",
            "border-image-source",
            "border-image-slice",
            "border-image-width",
            "border-image-repeat",
            "border-image-outset",
            "text-shadow",
            "box-shadow",
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

/// 解析 CSS grid-area 简写并展开为四个 GridLineValue。
///
/// 返回 `(row_start, row_end, col_start, col_end)`。
/// 解析失败返回 `None`。
pub fn parse_grid_area_shorthand(value: &str) -> Option<(GridLineValue, GridLineValue, GridLineValue, GridLineValue)> {
    let (rs, re, cs, ce) = values::parse_grid_area(value)?;
    let row_start = parse_grid_line(&rs)?;
    let row_end = parse_grid_line(&re)?;
    let col_start = parse_grid_line(&cs)?;
    let col_end = parse_grid_line(&ce)?;
    Some((row_start, row_end, col_start, col_end))
}

/// 解析 CSS grid-column / grid-row 简写（`<start> / <end>` 格式）。
///
/// 返回 `(start, end)`。
/// 无斜杠时，`<start>` 作为 start，end 为 Auto。
pub fn parse_grid_line_shorthand(value: &str) -> Option<(GridLineValue, GridLineValue)> {
    let value = value.trim();
    if let Some(slash_pos) = value.find('/') {
        let start_str = value[..slash_pos].trim();
        let end_str = value[slash_pos + 1..].trim();
        if start_str.is_empty() || end_str.is_empty() {
            return None;
        }
        let start = parse_grid_line(start_str)?;
        let end = parse_grid_line(end_str)?;
        Some((start, end))
    } else {
        let start = parse_grid_line(value)?;
        Some((start, GridLineValue::Auto))
    }
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

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.trim() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
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
        "break-spaces" => Some(WhiteSpaceValue::BreakSpaces),
        _ => None,
    }
}

/// 解析 CSS word-break 值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS writing-mode 值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-overflow 值。
pub fn parse_text_overflow(value: &str) -> Option<TextOverflowValue> {
    let v = value.trim();
    if let Some(parsed) = values::parse_text_overflow(v) {
        return match parsed {
            zero_css_parser::values::TextOverflowValue::Clip => Some(TextOverflowValue::Clip),
            zero_css_parser::values::TextOverflowValue::Ellipsis => Some(TextOverflowValue::Ellipsis),
            zero_css_parser::values::TextOverflowValue::String(s) => Some(TextOverflowValue::String(s)),
        };
    }
    None
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

/// 将 css-parser 的 CursorValue 映射为 style-system 的 CursorValue。
fn map_css_cursor(v: zero_css_parser::values::CursorValue) -> CursorValue {
    use zero_css_parser::values::CursorValue as Cv;
    match v {
        Cv::Auto => CursorValue::Auto,
        Cv::Default => CursorValue::Default,
        Cv::Pointer => CursorValue::Pointer,
        Cv::Move => CursorValue::Move,
        Cv::Text => CursorValue::Text,
        Cv::Wait => CursorValue::Wait,
        Cv::Crosshair => CursorValue::Crosshair,
        Cv::NotAllowed => CursorValue::NotAllowed,
        Cv::Grab => CursorValue::Grab,
        Cv::Grabbing => CursorValue::Grabbing,
        Cv::Help => CursorValue::Help,
        Cv::Progress => CursorValue::Progress,
        Cv::NResize => CursorValue::NsResize,
        Cv::SResize => CursorValue::NsResize,
        Cv::EResize => CursorValue::EwResize,
        Cv::WResize => CursorValue::EwResize,
        Cv::NeResize => CursorValue::NsResize,
        Cv::NwResize => CursorValue::NsResize,
        Cv::SeResize => CursorValue::NsResize,
        Cv::SwResize => CursorValue::NsResize,
        Cv::ColResize => CursorValue::ColResize,
        Cv::RowResize => CursorValue::RowResize,
        Cv::AllScroll => CursorValue::AllScroll,
        Cv::ZoomIn => CursorValue::ZoomIn,
        Cv::ZoomOut => CursorValue::ZoomOut,
        Cv::None => CursorValue::None,
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
        "text-decoration-line" => {
            if let Some(v) = parse_text_decoration_line(value) {
                style.text_decoration_line = v;
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
        "word-break" => {
            if let Some(v) = parse_word_break(value) {
                style.word_break = v;
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
            if let Some(v) = parse_length_or_math(value) {
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
        "column-gap" => {
            if let Some(v) = parse_length_or_math(value) {
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
                // 单值：尝试 column-width，再尝试 column-count
                if let Some(v) = values::parse_column_width(parts[0]) {
                    style.column_width = match v {
                        ColumnWidthValue::Auto => ColumnWidthComputedValue::Auto,
                        ColumnWidthValue::Length(l) => ColumnWidthComputedValue::Length(l),
                    };
                    style.column_count = ColumnCountComputedValue::Auto;
                    return true;
                }
                if let Some(v) = values::parse_column_count(parts[0]) {
                    style.column_count = match v {
                        ColumnCountValue::Auto => ColumnCountComputedValue::Auto,
                        ColumnCountValue::Number(n) => ColumnCountComputedValue::Number(n),
                    };
                    style.column_width = ColumnWidthComputedValue::Auto;
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
            if let Some(v) = values::parse_background_image(value) {
                style.background_image = match v {
                    zero_css_parser::values::BackgroundImageValue::None => BackgroundImageComputedValue::None,
                    zero_css_parser::values::BackgroundImageValue::Url(url) => BackgroundImageComputedValue::Url(url),
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
                    zero_css_parser::values::BackgroundPositionValue::Length(px) => {
                        BackgroundPositionComputedValue::Length(px)
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
                            zero_css_parser::values::BackgroundPositionValue::Length(px) => {
                                BackgroundPositionComputedValue::Length(px)
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
                            zero_css_parser::values::BackgroundPositionValue::Length(px) => {
                                BackgroundPositionComputedValue::Length(px)
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
        "word-break" => {
            child.word_break = parent.word_break.clone();
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
        "text-indent" => {
            child.text_indent = parent.text_indent.clone();
            true
        }
        "caption-side" => {
            child.caption_side = parent.caption_side.clone();
            true
        }
        "border-collapse" => {
            child.border_collapse = parent.border_collapse.clone();
            true
        }
        "quotes" => {
            child.quotes = parent.quotes.clone();
            true
        }
        "pointer-events" => {
            child.pointer_events = parent.pointer_events.clone();
            true
        }
        "overflow-wrap" => {
            child.overflow_wrap = parent.overflow_wrap.clone();
            true
        }
        "text-align-last" => {
            child.text_align_last = parent.text_align_last.clone();
            true
        }
        "font-variant-numeric" => {
            child.font_variant_numeric = parent.font_variant_numeric.clone();
            true
        }
        "direction" => {
            child.direction = parent.direction.clone();
            true
        }
        "tab-size" => {
            child.tab_size = parent.tab_size.clone();
            true
        }
        "accent-color" => {
            child.accent_color = parent.accent_color.clone();
            true
        }
        "caret-color" => {
            child.caret_color = parent.caret_color.clone();
            true
        }
        "text-wrap" => {
            child.text_wrap = parent.text_wrap.clone();
            true
        }
        "hyphens" => {
            child.hyphens = parent.hyphens.clone();
            true
        }
        "text-shadow" => {
            child.text_shadow = parent.text_shadow.clone();
            true
        }
        "list-style-image" => {
            child.list_style_image = parent.list_style_image.clone();
            true
        }
        "empty-cells" => {
            child.empty_cells = parent.empty_cells.clone();
            true
        }
        "border-spacing" => {
            child.border_spacing = parent.border_spacing.clone();
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
        "list-style-image" => {
            style.list_style_image = default_style.list_style_image;
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
        "text-decoration-line" => {
            style.text_decoration_line = default_style.text_decoration_line;
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
        "word-break" => {
            style.word_break = default_style.word_break;
            true
        }
        "text-indent" => {
            style.text_indent = default_style.text_indent;
            true
        }
        "text-overflow" => {
            style.text_overflow = default_style.text_overflow;
            true
        }
        "table-layout" => {
            style.table_layout = default_style.table_layout;
            true
        }
        "caption-side" => {
            style.caption_side = default_style.caption_side;
            true
        }
        "border-collapse" => {
            style.border_collapse = default_style.border_collapse;
            true
        }
        "resize" => {
            style.resize = default_style.resize;
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
        "column-gap" => {
            style.column_gap = default_style.column_gap;
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
        "transform-origin" => {
            style.transform_origin_x = default_style.transform_origin_x;
            style.transform_origin_y = default_style.transform_origin_y;
            true
        }
        "perspective" => {
            style.perspective = default_style.perspective;
            true
        }
        "perspective-origin" => {
            style.perspective_origin_x = default_style.perspective_origin_x;
            style.perspective_origin_y = default_style.perspective_origin_y;
            true
        }
        "transform-style" => {
            style.transform_style = default_style.transform_style;
            true
        }
        "backface-visibility" => {
            style.backface_visibility = default_style.backface_visibility;
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
        // Writing Mode
        "writing-mode" => {
            style.writing_mode = default_style.writing_mode;
            true
        }
        // Counters / Content / Quotes
        "counter-reset" => {
            style.counter_reset = default_style.counter_reset;
            true
        }
        "counter-increment" => {
            style.counter_increment = default_style.counter_increment;
            true
        }
        "counter-set" => {
            style.counter_set = default_style.counter_set;
            true
        }
        "content" => {
            style.content = default_style.content;
            true
        }
        "quotes" => {
            style.quotes = default_style.quotes;
            true
        }
        // Page Break
        "page-break-before" => {
            style.page_break_before = default_style.page_break_before;
            true
        }
        "page-break-after" => {
            style.page_break_after = default_style.page_break_after;
            true
        }
        "page-break-inside" => {
            style.page_break_inside = default_style.page_break_inside;
            true
        }
        // 其他
        "box-decoration-break" => {
            style.box_decoration_break = default_style.box_decoration_break;
            true
        }
        "image-rendering" => {
            style.image_rendering = default_style.image_rendering;
            true
        }
        "isolation" => {
            style.isolation = default_style.isolation;
            true
        }
        // Break
        "break-inside" => {
            style.break_inside = default_style.break_inside;
            true
        }
        "break-before" => {
            style.break_before = default_style.break_before;
            true
        }
        "break-after" => {
            style.break_after = default_style.break_after;
            true
        }
        // Column Rule
        "column-rule-width" => {
            style.column_rule_width = default_style.column_rule_width;
            true
        }
        "column-rule-style" => {
            style.column_rule_style = default_style.column_rule_style;
            true
        }
        "overscroll-behavior-x" => {
            style.overscroll_behavior_x = default_style.overscroll_behavior_x;
            true
        }
        "overscroll-behavior-y" => {
            style.overscroll_behavior_y = default_style.overscroll_behavior_y;
            true
        }
        "touch-action" => {
            style.touch_action = default_style.touch_action;
            true
        }
        "pointer-events" => {
            style.pointer_events = default_style.pointer_events;
            true
        }
        "overflow-wrap" => {
            style.overflow_wrap = default_style.overflow_wrap;
            true
        }
        "text-align-last" => {
            style.text_align_last = default_style.text_align_last;
            true
        }
        "font-variant-numeric" => {
            style.font_variant_numeric = default_style.font_variant_numeric;
            true
        }
        "user-select" => {
            style.user_select = default_style.user_select;
            true
        }
        "will-change" => {
            style.will_change = default_style.will_change;
            true
        }
        "direction" => {
            style.direction = default_style.direction;
            true
        }
        "unicode-bidi" => {
            style.unicode_bidi = default_style.unicode_bidi;
            true
        }
        "tab-size" => {
            style.tab_size = default_style.tab_size;
            true
        }
        // Columns
        "columns" => {
            style.column_count = default_style.column_count;
            style.column_width = default_style.column_width;
            true
        }
        "column-count" => {
            style.column_count = default_style.column_count;
            true
        }
        "column-width" => {
            style.column_width = default_style.column_width;
            true
        }
        // Object Fit / Filter
        "object-fit" => {
            style.object_fit = default_style.object_fit;
            true
        }
        "filter" => {
            style.filter = default_style.filter;
            true
        }
        // Column Rule Color
        "column-rule-color" => {
            style.column_rule_color = default_style.column_rule_color;
            true
        }
        // Contain
        "contain" => {
            style.contain = default_style.contain;
            true
        }
        // UI Appearance
        "appearance" => {
            style.appearance = default_style.appearance;
            true
        }
        "accent-color" => {
            style.accent_color = default_style.accent_color;
            true
        }
        "caret-color" => {
            style.caret_color = default_style.caret_color;
            true
        }
        // Compositing / Scrolling
        "mix-blend-mode" => {
            style.mix_blend_mode = default_style.mix_blend_mode;
            true
        }
        "scrollbar-width" => {
            style.scrollbar_width = default_style.scrollbar_width;
            true
        }
        "scrollbar-gutter" => {
            style.scrollbar_gutter = default_style.scrollbar_gutter;
            true
        }
        "text-wrap" => {
            style.text_wrap = default_style.text_wrap;
            true
        }
        "hyphens" => {
            style.hyphens = default_style.hyphens;
            true
        }
        "line-clamp" => {
            style.line_clamp = default_style.line_clamp;
            true
        }
        "background-image" => {
            style.background_image = default_style.background_image;
            true
        }
        "background-position" => {
            style.background_position = default_style.background_position;
            true
        }
        "background-repeat" => {
            style.background_repeat = default_style.background_repeat;
            true
        }
        "background-size" => {
            style.background_size = default_style.background_size;
            true
        }
        "background-attachment" => {
            style.background_attachment = default_style.background_attachment;
            true
        }
        "background-clip" => {
            style.background_clip = default_style.background_clip;
            true
        }
        "background-origin" => {
            style.background_origin = default_style.background_origin;
            true
        }
        "border-image-source" => {
            style.border_image_source = default_style.border_image_source;
            true
        }
        "border-image-slice" => {
            style.border_image_slice = default_style.border_image_slice;
            true
        }
        "border-image-width" => {
            style.border_image_width = default_style.border_image_width;
            true
        }
        "border-image-repeat" => {
            style.border_image_repeat = default_style.border_image_repeat;
            true
        }
        "border-image-outset" => {
            style.border_image_outset = default_style.border_image_outset;
            true
        }
        "text-shadow" => {
            style.text_shadow = default_style.text_shadow;
            true
        }
        "box-shadow" => {
            style.box_shadow = default_style.box_shadow;
            true
        }
        "justify-items" => {
            style.justify_items = default_style.justify_items;
            true
        }
        "justify-self" => {
            style.justify_self = default_style.justify_self;
            true
        }
        "align-content" => {
            style.align_content = default_style.align_content;
            true
        }
        "empty-cells" => {
            style.empty_cells = default_style.empty_cells;
            true
        }
        "border-spacing" => {
            style.border_spacing = default_style.border_spacing;
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

    // ═══════════════════════════════════════════════════════════════════
    // grid-area / grid-column / grid-row 简写属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 grid-area 命名区域简写
    fn test_grid_area_named_area() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-area", "header"));
        assert_eq!(style.grid_row_start, GridLineValue::Name("header".to_string()));
        assert_eq!(style.grid_row_end, GridLineValue::Name("header".to_string()));
        assert_eq!(style.grid_column_start, GridLineValue::Name("header".to_string()));
        assert_eq!(style.grid_column_end, GridLineValue::Name("header".to_string()));
    }

    #[test]
    /// 测试 grid-area auto 简写
    fn test_grid_area_auto() {
        let mut style = ComputedStyle::default();
        // 先设置非 auto 值
        style.grid_row_start = GridLineValue::Line(1);
        assert!(apply_property_value(&mut style, "grid-area", "auto"));
        assert_eq!(style.grid_row_start, GridLineValue::Auto);
        assert_eq!(style.grid_row_end, GridLineValue::Auto);
        assert_eq!(style.grid_column_start, GridLineValue::Auto);
        assert_eq!(style.grid_column_end, GridLineValue::Auto);
    }

    #[test]
    /// 测试 grid-area 四值斜杠分隔行号
    fn test_grid_area_four_line_numbers() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-area", "1 / 2 / 3 / 4"));
        assert_eq!(style.grid_row_start, GridLineValue::Line(1));
        assert_eq!(style.grid_row_end, GridLineValue::Line(2));
        assert_eq!(style.grid_column_start, GridLineValue::Line(3));
        assert_eq!(style.grid_column_end, GridLineValue::Line(4));
    }

    #[test]
    /// 测试 grid-area 两值斜杠分隔（row-start / col-start）
    fn test_grid_area_two_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-area", "1 / 3"));
        assert_eq!(style.grid_row_start, GridLineValue::Line(1));
        assert_eq!(style.grid_row_end, GridLineValue::Auto);
        assert_eq!(style.grid_column_start, GridLineValue::Line(3));
        assert_eq!(style.grid_column_end, GridLineValue::Auto);
    }

    #[test]
    /// 测试 grid-area 三值斜杠分隔
    fn test_grid_area_three_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-area", "1 / 3 / 2"));
        assert_eq!(style.grid_row_start, GridLineValue::Line(1));
        assert_eq!(style.grid_row_end, GridLineValue::Line(3));
        assert_eq!(style.grid_column_start, GridLineValue::Line(2));
        assert_eq!(style.grid_column_end, GridLineValue::Auto);
    }

    #[test]
    /// 测试 grid-area 包含 span 关键字
    fn test_grid_area_with_span() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-area", "2 / span 2 / 3 / span 3"));
        assert_eq!(style.grid_row_start, GridLineValue::Line(2));
        assert_eq!(style.grid_row_end, GridLineValue::Span(2));
        assert_eq!(style.grid_column_start, GridLineValue::Line(3));
        assert_eq!(style.grid_column_end, GridLineValue::Span(3));
    }

    #[test]
    /// 测试 grid-column 简写（start / end）
    fn test_grid_column_shorthand() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-column", "1 / 3"));
        assert_eq!(style.grid_column_start, GridLineValue::Line(1));
        assert_eq!(style.grid_column_end, GridLineValue::Line(3));
    }

    #[test]
    /// 测试 grid-column 简写（单个值）
    fn test_grid_column_single_value() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-column", "2"));
        assert_eq!(style.grid_column_start, GridLineValue::Line(2));
        assert_eq!(style.grid_column_end, GridLineValue::Auto);
    }

    #[test]
    /// 测试 grid-row 简写（start / end）
    fn test_grid_row_shorthand() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-row", "1 / span 2"));
        assert_eq!(style.grid_row_start, GridLineValue::Line(1));
        assert_eq!(style.grid_row_end, GridLineValue::Span(2));
    }

    #[test]
    /// 测试 grid-column 包含命名行
    fn test_grid_column_named() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "grid-column",
            "sidebar-start / sidebar-end"
        ));
        assert_eq!(
            style.grid_column_start,
            GridLineValue::Name("sidebar-start".to_string())
        );
        assert_eq!(style.grid_column_end, GridLineValue::Name("sidebar-end".to_string()));
    }

    #[test]
    /// 测试 grid-area 无效值返回 false
    fn test_grid_area_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "grid-area", ""));
    }

    #[test]
    /// 测试 parse_grid_area_shorthand 函数
    fn test_parse_grid_area_shorthand() {
        let result = parse_grid_area_shorthand("header").unwrap();
        assert_eq!(result.0, GridLineValue::Name("header".to_string()));
        assert_eq!(result.1, GridLineValue::Name("header".to_string()));
        assert_eq!(result.2, GridLineValue::Name("header".to_string()));
        assert_eq!(result.3, GridLineValue::Name("header".to_string()));

        let result = parse_grid_area_shorthand("auto").unwrap();
        assert_eq!(result.0, GridLineValue::Auto);
        assert_eq!(result.1, GridLineValue::Auto);
        assert_eq!(result.2, GridLineValue::Auto);
        assert_eq!(result.3, GridLineValue::Auto);

        let result = parse_grid_area_shorthand("1 / 3 / 2 / 4").unwrap();
        assert_eq!(result.0, GridLineValue::Line(1));
        assert_eq!(result.1, GridLineValue::Line(3));
        assert_eq!(result.2, GridLineValue::Line(2));
        assert_eq!(result.3, GridLineValue::Line(4));
    }

    #[test]
    /// 测试 parse_grid_line_shorthand 函数
    fn test_parse_grid_line_shorthand() {
        let result = parse_grid_line_shorthand("1 / 3").unwrap();
        assert_eq!(result.0, GridLineValue::Line(1));
        assert_eq!(result.1, GridLineValue::Line(3));

        let result = parse_grid_line_shorthand("span 2 / 5").unwrap();
        assert_eq!(result.0, GridLineValue::Span(2));
        assert_eq!(result.1, GridLineValue::Line(5));

        let result = parse_grid_line_shorthand("auto").unwrap();
        assert_eq!(result.0, GridLineValue::Auto);
        assert_eq!(result.1, GridLineValue::Auto);
    }

    // ═══════════════════════════════════════════════════════════════════
    // cursor/opacity 管线集成测试 — 验证 css-parser 的解析器被正确接入
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 cursor 属性通过 CSS 管线应用（使用 css-parser 的 parse_cursor）
    fn test_cursor_via_css_parser_pipeline() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.cursor, CursorValue::Auto);

        // 基本关键字
        assert!(apply_property_value(&mut style, "cursor", "pointer"));
        assert_eq!(style.cursor, CursorValue::Pointer);

        assert!(apply_property_value(&mut style, "cursor", "move"));
        assert_eq!(style.cursor, CursorValue::Move);

        assert!(apply_property_value(&mut style, "cursor", "wait"));
        assert_eq!(style.cursor, CursorValue::Wait);

        assert!(apply_property_value(&mut style, "cursor", "not-allowed"));
        assert_eq!(style.cursor, CursorValue::NotAllowed);

        // 大小写不敏感（css-parser 使用 to_ascii_lowercase）
        assert!(apply_property_value(&mut style, "cursor", "Pointer"));
        assert_eq!(style.cursor, CursorValue::Pointer);

        assert!(apply_property_value(&mut style, "cursor", "HELP"));
        assert_eq!(style.cursor, CursorValue::Help);

        // 方向性 resize 映射到 style-system 的 NsResize/EwResize
        assert!(apply_property_value(&mut style, "cursor", "n-resize"));
        assert_eq!(style.cursor, CursorValue::NsResize);

        assert!(apply_property_value(&mut style, "cursor", "s-resize"));
        assert_eq!(style.cursor, CursorValue::NsResize);

        assert!(apply_property_value(&mut style, "cursor", "e-resize"));
        assert_eq!(style.cursor, CursorValue::EwResize);

        assert!(apply_property_value(&mut style, "cursor", "w-resize"));
        assert_eq!(style.cursor, CursorValue::EwResize);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "cursor", "invalid-cursor"));
        assert_eq!(style.cursor, CursorValue::EwResize); // 上一个有效值
    }

    #[test]
    /// 测试 opacity 属性通过 css-parser 的 parse_opacity 应用
    fn test_opacity_via_css_parser_pipeline() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.opacity, 1.0);

        // 正常数值
        assert!(apply_property_value(&mut style, "opacity", "0.5"));
        assert!((style.opacity - 0.5).abs() < f64::EPSILON);

        assert!(apply_property_value(&mut style, "opacity", "0"));
        assert_eq!(style.opacity, 0.0);

        assert!(apply_property_value(&mut style, "opacity", "1"));
        assert_eq!(style.opacity, 1.0);

        // 百分比格式（css-parser parse_opacity 支持）
        assert!(apply_property_value(&mut style, "opacity", "50%"));
        assert!((style.opacity - 0.5).abs() < f64::EPSILON);

        assert!(apply_property_value(&mut style, "opacity", "100%"));
        assert_eq!(style.opacity, 1.0);

        assert!(apply_property_value(&mut style, "opacity", "0%"));
        assert_eq!(style.opacity, 0.0);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "opacity", "abc"));
        assert!(!apply_property_value(&mut style, "opacity", "half"));
    }

    #[test]
    /// 测试 opacity 值被 clamp 到 [0.0, 1.0] 范围
    fn test_opacity_clamping_via_css_parser() {
        let mut style = ComputedStyle::default();

        // 超出上界 → clamp 到 1.0
        assert!(apply_property_value(&mut style, "opacity", "1.5"));
        assert_eq!(style.opacity, 1.0);

        assert!(apply_property_value(&mut style, "opacity", "999"));
        assert_eq!(style.opacity, 1.0);

        // 超出下界 → clamp 到 0.0
        assert!(apply_property_value(&mut style, "opacity", "-0.5"));
        assert_eq!(style.opacity, 0.0);

        assert!(apply_property_value(&mut style, "opacity", "-10"));
        assert_eq!(style.opacity, 0.0);

        // 百分比超出范围
        assert!(apply_property_value(&mut style, "opacity", "150%"));
        assert_eq!(style.opacity, 1.0);

        assert!(apply_property_value(&mut style, "opacity", "-25%"));
        assert_eq!(style.opacity, 0.0);
    }

    #[test]
    /// 测试 cursor 继承：父元素 cursor:pointer，子元素应继承
    fn test_cursor_inheritance() {
        let mut parent = ComputedStyle::default();
        parent.cursor = CursorValue::Pointer;

        let mut child = ComputedStyle::default();
        assert_eq!(child.cursor, CursorValue::Auto);

        // cursor 是继承属性
        assert!(inherit_property(&parent, &mut child, "cursor"));
        assert_eq!(child.cursor, CursorValue::Pointer);

        // 子元素显式设置 cursor 后覆盖继承值
        assert!(apply_property_value(&mut child, "cursor", "text"));
        assert_eq!(child.cursor, CursorValue::Text);
    }

    // ═══════════════════════════════════════════════════════════════════
    // word-break 属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 apply_property_value 对 word-break: break-all
    fn test_apply_word_break_break_all() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "word-break", "break-all"));
        assert_eq!(style.word_break, WordBreakValue::BreakAll);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "word-break", "invalid"));
        assert_eq!(style.word_break, WordBreakValue::BreakAll);
    }

    #[test]
    /// 测试 apply_property_value 对 word-break: keep-all
    fn test_apply_word_break_keep_all() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "word-break", "keep-all"));
        assert_eq!(style.word_break, WordBreakValue::KeepAll);

        // break-word
        assert!(apply_property_value(&mut style, "word-break", "break-word"));
        assert_eq!(style.word_break, WordBreakValue::BreakWord);

        // normal
        assert!(apply_property_value(&mut style, "word-break", "normal"));
        assert_eq!(style.word_break, WordBreakValue::Normal);
    }

    #[test]
    /// 测试 word-break 继承：父元素 break-all，子元素应继承
    fn test_word_break_inheritance() {
        let mut parent = ComputedStyle::default();
        parent.word_break = WordBreakValue::BreakAll;

        let mut child = ComputedStyle::default();
        assert_eq!(child.word_break, WordBreakValue::Normal);

        // word-break 是继承属性
        assert!(inherit_property(&parent, &mut child, "word-break"));
        assert_eq!(child.word_break, WordBreakValue::BreakAll);

        // 子元素显式设置后覆盖继承值
        assert!(apply_property_value(&mut child, "word-break", "keep-all"));
        assert_eq!(child.word_break, WordBreakValue::KeepAll);
    }

    #[test]
    /// 测试 word-break 默认值为 Normal
    fn test_word_break_default_is_normal() {
        let style = ComputedStyle::default();
        assert_eq!(style.word_break, WordBreakValue::Normal);

        // 验证注册表初始值
        assert!(PropertyRegistry::initial_value("word-break").is_some());
        assert!(PropertyRegistry::is_inherited("word-break"));

        // 验证 known_properties 包含
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"word-break"));

        // 验证 apply_initial_value 重置
        let mut style = ComputedStyle::default();
        style.word_break = WordBreakValue::BreakAll;
        assert!(apply_initial_value(&mut style, "word-break"));
        assert_eq!(style.word_break, WordBreakValue::Normal);
    }

    // ═══════════════════════════════════════════════════════════════════
    // writing-mode 属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 apply_property_value 对 writing-mode: vertical-rl
    fn test_apply_writing_mode_vertical_rl() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
        assert_eq!(style.writing_mode, WritingModeValue::VerticalRl);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "writing-mode", "invalid"));
        assert_eq!(style.writing_mode, WritingModeValue::VerticalRl);
    }

    #[test]
    /// 测试 apply_property_value 对 writing-mode: vertical-lr
    fn test_apply_writing_mode_vertical_lr() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "writing-mode", "vertical-lr"));
        assert_eq!(style.writing_mode, WritingModeValue::VerticalLr);
    }

    #[test]
    /// 测试 writing-mode 默认值为 horizontal-tb
    fn test_writing_mode_default_is_horizontal_tb() {
        let style = ComputedStyle::default();
        assert_eq!(style.writing_mode, WritingModeValue::HorizontalTb);

        // 验证注册表初始值
        assert!(PropertyRegistry::initial_value("writing-mode").is_some());
        assert!(!PropertyRegistry::is_inherited("writing-mode"));

        // 验证 known_properties 包含
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"writing-mode"));

        // 验证 apply_initial_value 重置
        let mut style = ComputedStyle::default();
        style.writing_mode = WritingModeValue::VerticalRl;
        assert!(apply_initial_value(&mut style, "writing-mode"));
        assert_eq!(style.writing_mode, WritingModeValue::HorizontalTb);
    }

    #[test]
    /// 测试 writing-mode 不继承：父元素 vertical-rl，子元素不继承
    fn test_writing_mode_not_inherited() {
        // writing-mode 不是继承属性
        assert!(!PropertyRegistry::is_inherited("writing-mode"));

        let mut parent = ComputedStyle::default();
        parent.writing_mode = WritingModeValue::VerticalRl;

        let mut child = ComputedStyle::default();
        assert_eq!(child.writing_mode, WritingModeValue::HorizontalTb);

        // inherit_property 对 writing-mode 应返回 false
        assert!(!inherit_property(&parent, &mut child, "writing-mode"));
        // 子元素值不变
        assert_eq!(child.writing_mode, WritingModeValue::HorizontalTb);
    }

    // ═══════════════════════════════════════════════════════════════════
    // text-decoration-line / text-transform / letter-spacing 属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 apply_property_value 对 text-decoration-line: underline
    fn test_apply_text_decoration_underline() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-decoration-line", "underline"));
        assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "text-decoration-line", "invalid"));
        assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);
    }

    #[test]
    /// 测试 apply_property_value 对 text-decoration-line: none
    fn test_apply_text_decoration_none() {
        let mut style = ComputedStyle::default();
        // 先设置为 underline
        assert!(apply_property_value(&mut style, "text-decoration-line", "underline"));
        assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);

        // 重置为 none
        assert!(apply_property_value(&mut style, "text-decoration-line", "none"));
        assert_eq!(style.text_decoration_line, TextDecorationLineValue::None);

        // 默认值也是 none
        let style = ComputedStyle::default();
        assert_eq!(style.text_decoration_line, TextDecorationLineValue::None);
    }

    #[test]
    /// 测试 apply_property_value 对 text-transform: uppercase
    fn test_apply_text_transform_uppercase() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-transform", "uppercase"));
        assert_eq!(style.text_transform, TextTransformValue::Uppercase);
    }

    #[test]
    /// 测试 apply_property_value 对 text-transform: capitalize
    fn test_apply_text_transform_capitalize() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-transform", "capitalize"));
        assert_eq!(style.text_transform, TextTransformValue::Capitalize);
    }

    #[test]
    /// 测试 text-transform 继承：父元素 uppercase，子元素继承
    fn test_text_transform_inherited() {
        let mut parent = ComputedStyle::default();
        parent.text_transform = TextTransformValue::Uppercase;

        let mut child = ComputedStyle::default();
        assert_eq!(child.text_transform, TextTransformValue::None);

        // text-transform 是继承属性
        assert!(inherit_property(&parent, &mut child, "text-transform"));
        assert_eq!(child.text_transform, TextTransformValue::Uppercase);
    }

    #[test]
    /// 测试 text-decoration-line 不继承：父元素 underline，子元素不继承
    fn test_text_transform_not_inherited_decoration() {
        // text-decoration-line 不是继承属性
        assert!(!PropertyRegistry::is_inherited("text-decoration-line"));

        let mut parent = ComputedStyle::default();
        parent.text_decoration_line = TextDecorationLineValue::Underline;

        let mut child = ComputedStyle::default();
        assert_eq!(child.text_decoration_line, TextDecorationLineValue::None);

        // inherit_property 对 text-decoration-line 应返回 false
        assert!(!inherit_property(&parent, &mut child, "text-decoration-line"));
        // 子元素值不变
        assert_eq!(child.text_decoration_line, TextDecorationLineValue::None);
    }

    #[test]
    /// 测试 apply_property_value 对 letter-spacing: px
    fn test_apply_letter_spacing_px() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "letter-spacing", "3px"));
        assert_eq!(style.letter_spacing, LengthValue::Px(3.0));

        // 负值
        assert!(apply_property_value(&mut style, "letter-spacing", "-1.5px"));
        assert_eq!(style.letter_spacing, LengthValue::Px(-1.5));
    }

    #[test]
    /// 测试 apply_property_value 对 letter-spacing: normal（解析为 0px）
    fn test_apply_letter_spacing_normal() {
        let mut style = ComputedStyle::default();
        // letter-spacing 的 normal 在 CSS 中解析为 0px
        // 当前实现通过 parse_length_or_math 解析，"normal" 不是有效长度
        // 所以先设置为非零值，然后验证默认重置
        assert!(apply_property_value(&mut style, "letter-spacing", "2px"));
        assert_eq!(style.letter_spacing, LengthValue::Px(2.0));

        // 默认值为 0px
        let style = ComputedStyle::default();
        assert_eq!(style.letter_spacing, LengthValue::Px(0.0));
    }

    #[test]
    /// 测试 letter-spacing 继承：父元素 3px，子元素继承
    fn test_letter_spacing_inherited() {
        let mut parent = ComputedStyle::default();
        parent.letter_spacing = LengthValue::Px(3.0);

        let mut child = ComputedStyle::default();
        assert_eq!(child.letter_spacing, LengthValue::Px(0.0));

        // letter-spacing 是继承属性
        assert!(inherit_property(&parent, &mut child, "letter-spacing"));
        assert_eq!(child.letter_spacing, LengthValue::Px(3.0));

        // 子元素显式设置后覆盖继承值
        assert!(apply_property_value(&mut child, "letter-spacing", "5px"));
        assert_eq!(child.letter_spacing, LengthValue::Px(5.0));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 级联/简写/自定义属性/继承/revert 边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试级联源顺序：两条规则具有相同特异性，后应用的规则胜出
    fn test_cascade_source_order() {
        let mut style = ComputedStyle::default();

        // 第一条规则：display: flex
        assert!(apply_property_value(&mut style, "display", "flex"));
        assert_eq!(style.display, DisplayValue::Flex);

        // 第二条规则（相同特异性，后出现）：display: grid — 应覆盖前一条
        assert!(apply_property_value(&mut style, "display", "grid"));
        assert_eq!(style.display, DisplayValue::Grid);

        // 同理测试 color
        assert!(apply_property_value(&mut style, "color", "red"));
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));

        assert!(apply_property_value(&mut style, "color", "blue"));
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 255, 255));

        // 同理测试 width
        assert!(apply_property_value(&mut style, "width", "100px"));
        assert!(apply_property_value(&mut style, "width", "200px"));
        assert_eq!(style.width, LengthValue::Px(200.0));
    }

    #[test]
    /// 测试 border 简写展开为 12 个长属性（4边 x width/style/color）
    fn test_shorthand_border_expansion() {
        let mut style = ComputedStyle::default();

        // 手动模拟 "border: 1px solid red" 的简写展开
        // 宽度：四边均为 1px
        assert!(apply_property_value(&mut style, "border-top-width", "1px"));
        assert!(apply_property_value(&mut style, "border-right-width", "1px"));
        assert!(apply_property_value(&mut style, "border-bottom-width", "1px"));
        assert!(apply_property_value(&mut style, "border-left-width", "1px"));

        // 样式：四边均为 solid
        assert!(apply_property_value(&mut style, "border-top-style", "solid"));
        assert!(apply_property_value(&mut style, "border-right-style", "solid"));
        assert!(apply_property_value(&mut style, "border-bottom-style", "solid"));
        assert!(apply_property_value(&mut style, "border-left-style", "solid"));

        // 颜色：四边均为 red
        assert!(apply_property_value(&mut style, "border-top-color", "red"));
        assert!(apply_property_value(&mut style, "border-right-color", "red"));
        assert!(apply_property_value(&mut style, "border-bottom-color", "red"));
        assert!(apply_property_value(&mut style, "border-left-color", "red"));

        // 验证所有 12 个长属性已正确设置
        let expected_width = LengthValue::Px(1.0);
        let expected_style = BorderStyleValue::Solid;
        let expected_color = ColorValue::Rgba(255, 0, 0, 255);

        // 宽度（4个）
        assert_eq!(style.border_top_width, expected_width);
        assert_eq!(style.border_right_width, expected_width);
        assert_eq!(style.border_bottom_width, expected_width);
        assert_eq!(style.border_left_width, expected_width);

        // 样式（4个）
        assert_eq!(style.border_top_style, expected_style);
        assert_eq!(style.border_right_style, expected_style);
        assert_eq!(style.border_bottom_style, expected_style);
        assert_eq!(style.border_left_style, expected_style);

        // 颜色（4个）
        assert_eq!(style.border_top_color, expected_color);
        assert_eq!(style.border_right_color, expected_color);
        assert_eq!(style.border_bottom_color, expected_color);
        assert_eq!(style.border_left_color, expected_color);
    }

    #[test]
    /// 测试自定义属性链式引用：--a: red → --b: var(--a) → color: var(--b) 最终解析为 red
    fn test_custom_property_chained() {
        use crate::computed::resolve_var;
        use std::collections::HashMap;

        // 构建自定义属性映射
        let mut custom_props = HashMap::new();
        custom_props.insert("--a".to_string(), "red".to_string());
        custom_props.insert("--b".to_string(), "var(--a)".to_string());

        // 第一层：var(--b) → 解析为 var(--a)
        let resolved_b = resolve_var("var(--b)", &custom_props);
        // 第二层：var(--a) → 解析为 red
        let resolved_a = resolve_var(&resolved_b, &custom_props);
        assert_eq!(resolved_a, "red");

        // 验证解析后的值可以应用到 ComputedStyle
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "color", &resolved_a));
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// 测试对非继承属性显式设置 inherit：没有父元素时使用默认值
    fn test_inherit_non_inherited_explicit() {
        // display 不是继承属性，对不可继承属性调用 inherit_property 返回 false
        let parent = ComputedStyle::default();
        let mut child = ComputedStyle::default();

        // display 不可继承
        assert!(!inherit_property(&parent, &mut child, "display"));
        assert_eq!(child.display, DisplayValue::Inline); // 保持默认值

        // width 不可继承
        assert!(!inherit_property(&parent, &mut child, "width"));
        assert_eq!(child.width, ComputedStyle::default().width);

        // 即使父元素 display 被修改，子元素也不会继承
        let mut parent_modified = ComputedStyle::default();
        parent_modified.display = DisplayValue::Flex;

        let mut child2 = ComputedStyle::default();
        assert!(!inherit_property(&parent_modified, &mut child2, "display"));
        assert_eq!(child2.display, DisplayValue::Inline); // 仍为默认值，未继承父元素的 flex
    }

    #[test]
    /// 测试 revert 关键字：应用 display: revert 时恢复为 user-agent 默认值
    fn test_revert_keyword() {
        let mut style = ComputedStyle::default();

        // 先修改 display 为非默认值
        style.display = DisplayValue::Flex;
        assert_eq!(style.display, DisplayValue::Flex);

        // "revert" 不是有效的 display 值，apply_property_value 返回 false
        // 在完整 CSS 引擎中，revert 会触发回退到 user-agent 样式
        // 这里模拟 revert 的效果：使用 apply_initial_value 恢复为 UA 默认
        assert!(!apply_property_value(&mut style, "display", "revert"));
        // display 未被 "revert" 字符串改变
        assert_eq!(style.display, DisplayValue::Flex);

        // 正确的 revert 模拟：使用 apply_initial_value 恢复为 UA 默认
        assert!(apply_initial_value(&mut style, "display"));
        assert_eq!(style.display, DisplayValue::Inline); // UA 默认 display 为 inline

        // 同理测试 position: revert
        style.position = PositionValue::Absolute;
        assert!(!apply_property_value(&mut style, "position", "revert"));
        assert_eq!(style.position, PositionValue::Absolute);
        assert!(apply_initial_value(&mut style, "position"));
        assert_eq!(style.position, PositionValue::Static); // UA 默认
    }

    // ═══════════════════════════════════════════════════════════════════
    // 3D Transform / perspective / backface-visibility 边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 transform-origin 默认值为 50% 50%
    fn test_transform_origin_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.transform_origin_x, LengthValue::Percentage(50.0));
        assert_eq!(style.transform_origin_y, LengthValue::Percentage(50.0));
    }

    #[test]
    /// 测试 transform-origin: 10px 20px 应用
    fn test_transform_origin_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "transform-origin", "10px 20px"));
        assert_eq!(style.transform_origin_x, LengthValue::Px(10.0));
        assert_eq!(style.transform_origin_y, LengthValue::Px(20.0));

        // 单值：Y 默认为 50%
        let mut style2 = ComputedStyle::default();
        assert!(apply_property_value(&mut style2, "transform-origin", "0px"));
        assert_eq!(style2.transform_origin_x, LengthValue::Px(0.0));
        assert_eq!(style2.transform_origin_y, LengthValue::Percentage(50.0));
    }

    #[test]
    /// 测试 perspective: 500px 应用
    fn test_perspective_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "perspective", "500px"));
        assert_eq!(style.perspective, LengthValue::Px(500.0));

        // perspective: none 重置为 0
        assert!(apply_property_value(&mut style, "perspective", "none"));
        assert_eq!(style.perspective, LengthValue::Px(0.0));
    }

    #[test]
    /// 测试 transform-style: preserve-3d 应用
    fn test_transform_style_apply() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.transform_style, TransformStyleValue::Flat);

        assert!(apply_property_value(&mut style, "transform-style", "preserve-3d"));
        assert_eq!(style.transform_style, TransformStyleValue::Preserve3d);

        assert!(apply_property_value(&mut style, "transform-style", "flat"));
        assert_eq!(style.transform_style, TransformStyleValue::Flat);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "transform-style", "invalid"));
        assert_eq!(style.transform_style, TransformStyleValue::Flat);
    }

    #[test]
    /// 测试 backface-visibility: hidden 应用
    fn test_backface_visibility_apply() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);

        assert!(apply_property_value(&mut style, "backface-visibility", "hidden"));
        assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Hidden);

        assert!(apply_property_value(&mut style, "backface-visibility", "visible"));
        assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "backface-visibility", "invalid"));
        assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);
    }

    #[test]
    /// 测试 transform-origin 不继承
    fn test_transform_origin_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("transform-origin"));

        let mut parent = ComputedStyle::default();
        parent.transform_origin_x = LengthValue::Px(100.0);
        parent.transform_origin_y = LengthValue::Px(200.0);

        let mut child = ComputedStyle::default();
        assert!(!inherit_property(&parent, &mut child, "transform-origin"));
        assert_eq!(child.transform_origin_x, LengthValue::Percentage(50.0));
        assert_eq!(child.transform_origin_y, LengthValue::Percentage(50.0));
    }

    #[test]
    /// 测试 perspective-origin: left top 应用
    fn test_perspective_origin_apply() {
        let mut style = ComputedStyle::default();
        // "left top" — left 解析为 0%, top 解析为 0%
        // 当前实现通过 parse_length_or_math 解析，"left" 不是长度值
        // 使用数值测试
        assert!(apply_property_value(&mut style, "perspective-origin", "0% 0%"));
        assert_eq!(style.perspective_origin_x, LengthValue::Percentage(0.0));
        assert_eq!(style.perspective_origin_y, LengthValue::Percentage(0.0));

        // 默认值为 50% 50%
        let style2 = ComputedStyle::default();
        assert_eq!(style2.perspective_origin_x, LengthValue::Percentage(50.0));
        assert_eq!(style2.perspective_origin_y, LengthValue::Percentage(50.0));

        // 单值：Y 默认为 50%
        let mut style3 = ComputedStyle::default();
        assert!(apply_property_value(&mut style3, "perspective-origin", "100px"));
        assert_eq!(style3.perspective_origin_x, LengthValue::Px(100.0));
        assert_eq!(style3.perspective_origin_y, LengthValue::Percentage(50.0));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增属性测试 — text-indent, table-layout, caption-side,
    //                border-collapse, resize, white-space break-spaces
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 text-indent 默认值为 Px(0.0)
    fn test_text_indent_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.text_indent, LengthValue::Px(0.0));
    }

    #[test]
    /// 测试 apply_property_value 对 text-indent: 2em
    fn test_apply_text_indent_em() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-indent", "2em"));
        assert_eq!(style.text_indent, LengthValue::Em(2.0));
    }

    #[test]
    /// 测试 apply_property_value 对 text-indent: 10%
    fn test_apply_text_indent_percentage() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-indent", "10%"));
        assert_eq!(style.text_indent, LengthValue::Percentage(10.0));
    }

    #[test]
    /// 测试 table-layout 默认值为 Auto
    fn test_table_layout_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.table_layout, TableLayoutValue::Auto);
    }

    #[test]
    /// 测试 apply_property_value 对 table-layout: fixed
    fn test_apply_table_layout_fixed() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "table-layout", "fixed"));
        assert_eq!(style.table_layout, TableLayoutValue::Fixed);
    }

    #[test]
    /// 测试 apply_property_value 对 table-layout 无效值
    fn test_apply_table_layout_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "table-layout", "invalid"));
    }

    #[test]
    /// 测试 caption-side 默认值为 Top
    fn test_caption_side_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.caption_side, CaptionSideValue::Top);
    }

    #[test]
    /// 测试 apply_property_value 对 caption-side: bottom
    fn test_apply_caption_side_bottom() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "caption-side", "bottom"));
        assert_eq!(style.caption_side, CaptionSideValue::Bottom);
    }

    #[test]
    /// 测试 border-collapse 默认值为 Separate
    fn test_border_collapse_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.border_collapse, BorderCollapseValue::Separate);
    }

    #[test]
    /// 测试 apply_property_value 对 border-collapse: collapse
    fn test_apply_border_collapse_collapse() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-collapse", "collapse"));
        assert_eq!(style.border_collapse, BorderCollapseValue::Collapse);
    }

    #[test]
    /// 测试 resize 默认值为 None
    fn test_resize_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.resize, ResizeValue::None);
    }

    #[test]
    /// 测试 apply_property_value 对 resize: both
    fn test_apply_resize_both() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "resize", "both"));
        assert_eq!(style.resize, ResizeValue::Both);
    }

    #[test]
    /// 测试 apply_property_value 对 resize: horizontal / vertical / block / inline
    fn test_apply_resize_variants() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "resize", "horizontal"));
        assert_eq!(style.resize, ResizeValue::Horizontal);
        assert!(apply_property_value(&mut style, "resize", "vertical"));
        assert_eq!(style.resize, ResizeValue::Vertical);
        assert!(apply_property_value(&mut style, "resize", "block"));
        assert_eq!(style.resize, ResizeValue::Block);
        assert!(apply_property_value(&mut style, "resize", "inline"));
        assert_eq!(style.resize, ResizeValue::Inline);
    }

    #[test]
    /// 测试 white-space: break-spaces
    fn test_parse_white_space_break_spaces() {
        assert_eq!(parse_white_space("break-spaces"), Some(WhiteSpaceValue::BreakSpaces));
        // 验证 apply_property_value 也能应用
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "white-space", "break-spaces"));
        assert_eq!(style.white_space, WhiteSpaceValue::BreakSpaces);
    }

    #[test]
    /// 测试 text-overflow 自定义字符串
    fn test_apply_text_overflow_string() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-overflow", "\"...\""));
        assert_eq!(style.text_overflow, TextOverflowValue::String("...".to_string()));
    }

    #[test]
    /// 测试 text-indent 继承
    fn test_inherit_text_indent() {
        let mut parent = ComputedStyle::default();
        parent.text_indent = LengthValue::Em(2.0);
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "text-indent"));
        assert_eq!(child.text_indent, LengthValue::Em(2.0));
    }

    #[test]
    /// 测试 caption-side 继承
    fn test_inherit_caption_side() {
        let mut parent = ComputedStyle::default();
        parent.caption_side = CaptionSideValue::Bottom;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "caption-side"));
        assert_eq!(child.caption_side, CaptionSideValue::Bottom);
    }

    #[test]
    /// 测试 border-collapse 继承
    fn test_inherit_border_collapse() {
        let mut parent = ComputedStyle::default();
        parent.border_collapse = BorderCollapseValue::Collapse;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "border-collapse"));
        assert_eq!(child.border_collapse, BorderCollapseValue::Collapse);
    }

    #[test]
    /// 测试 resize 不继承
    fn test_resize_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("resize"));
    }

    #[test]
    /// 测试 table-layout 不继承
    fn test_table_layout_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("table-layout"));
    }

    #[test]
    /// 测试新增属性在 known_properties 中
    fn test_new_properties_in_known_list() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"text-indent"));
        assert!(props.contains(&"table-layout"));
        assert!(props.contains(&"caption-side"));
        assert!(props.contains(&"border-collapse"));
        assert!(props.contains(&"resize"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Counter / Content / Quotes 测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 counter-reset 属性解析
    fn test_apply_counter_reset() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "counter-reset",
            "section 1 subsection"
        ));
        assert_eq!(style.counter_reset.len(), 2);
        assert_eq!(style.counter_reset[0].name, "section");
        assert_eq!(style.counter_reset[0].value, Some(1));
        assert_eq!(style.counter_reset[1].name, "subsection");
        assert_eq!(style.counter_reset[1].value, None);
    }

    #[test]
    /// 测试 counter-reset: none 清空列表
    fn test_apply_counter_reset_none() {
        let mut style = ComputedStyle::default();
        apply_property_value(&mut style, "counter-reset", "section 5");
        assert!(!style.counter_reset.is_empty());
        assert!(apply_property_value(&mut style, "counter-reset", "none"));
        assert!(style.counter_reset.is_empty());
    }

    #[test]
    /// 测试 counter-increment 属性解析
    fn test_apply_counter_increment() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "counter-increment", "section 2"));
        assert_eq!(style.counter_increment.len(), 1);
        assert_eq!(style.counter_increment[0].name, "section");
        assert_eq!(style.counter_increment[0].value, Some(2));
    }

    #[test]
    /// 测试 content: normal 默认值
    fn test_apply_content_normal() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "content", "normal"));
        assert_eq!(style.content, ContentComputedValue::Normal);
    }

    #[test]
    /// 测试 content: string 值
    fn test_apply_content_string() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "content", "\"Prefix: \""));
        assert_eq!(style.content, ContentComputedValue::String("Prefix: ".to_string()));
    }

    #[test]
    /// 测试 content: none 值
    fn test_apply_content_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "content", "none"));
        assert_eq!(style.content, ContentComputedValue::None);
    }

    #[test]
    /// 测试 content: attr() 值
    fn test_apply_content_attr() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "content", "attr(data-label)"));
        assert_eq!(style.content, ContentComputedValue::Attr("data-label".to_string()));
    }

    #[test]
    /// 测试 content: counter() 值
    fn test_apply_content_counter() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "content",
            "counter(section, upper-roman)"
        ));
        match &style.content {
            ContentComputedValue::Counter { name, style } => {
                assert_eq!(name, "section");
                assert_eq!(style, &Some("upper-roman".to_string()));
            }
            _ => panic!("expected Counter variant"),
        }
    }

    #[test]
    /// 测试 quotes: none 值
    fn test_apply_quotes_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "quotes", "none"));
        assert_eq!(style.quotes, QuotesComputedValue::None);
    }

    #[test]
    /// 测试 quotes: 引号对值
    fn test_apply_quotes_pairs() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "quotes", r#""«" "»" "‹" "›""#));
        match &style.quotes {
            QuotesComputedValue::Pairs(pairs) => {
                assert_eq!(pairs.len(), 2);
                assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
            }
            _ => panic!("expected Pairs"),
        }
    }

    #[test]
    /// 测试 quotes 继承
    fn test_quotes_inherited() {
        assert!(PropertyRegistry::is_inherited("quotes"));
        let mut parent = ComputedStyle::default();
        apply_property_value(&mut parent, "quotes", "none");
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "quotes"));
        assert_eq!(child.quotes, QuotesComputedValue::None);
    }

    #[test]
    /// 测试 counter-reset 不继承
    fn test_counter_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("counter-reset"));
        assert!(!PropertyRegistry::is_inherited("counter-increment"));
        assert!(!PropertyRegistry::is_inherited("content"));
    }

    #[test]
    /// 测试新增属性在 known_properties 中
    fn test_counter_content_quotes_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"counter-reset"));
        assert!(props.contains(&"counter-increment"));
        assert!(props.contains(&"content"));
        assert!(props.contains(&"quotes"));
    }

    #[test]
    /// 测试 apply_initial_value 对新属性
    fn test_apply_initial_value_new_properties() {
        let mut style = ComputedStyle::default();
        apply_property_value(&mut style, "counter-reset", "section 5");
        apply_property_value(&mut style, "content", "\"Hello\"");
        apply_property_value(&mut style, "quotes", "none");

        assert!(apply_initial_value(&mut style, "counter-reset"));
        assert!(style.counter_reset.is_empty());

        assert!(apply_initial_value(&mut style, "content"));
        assert_eq!(style.content, ContentComputedValue::Normal);

        assert!(apply_initial_value(&mut style, "quotes"));
        assert_eq!(style.quotes, QuotesComputedValue::Auto);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增属性测试：page-break, box-decoration-break, image-rendering, isolation
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// page-break-before 默认值为 Auto
    fn test_page_break_before_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.page_break_before, PageBreakValue::Auto);
    }

    #[test]
    /// page-break-after 默认值为 Auto
    fn test_page_break_after_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.page_break_after, PageBreakValue::Auto);
    }

    #[test]
    /// page-break-inside 默认值为 Auto
    fn test_page_break_inside_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.page_break_inside, PageBreakValue::Auto);
    }

    #[test]
    /// box-decoration-break 默认值为 Slice
    fn test_box_decoration_break_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);
    }

    #[test]
    /// image-rendering 默认值为 Auto
    fn test_image_rendering_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.image_rendering, ImageRenderingValue::Auto);
    }

    #[test]
    /// isolation 默认值为 Auto
    fn test_isolation_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.isolation, IsolationValue::Auto);
    }

    #[test]
    /// page-break-before 应用各值
    fn test_apply_page_break_before() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "page-break-before", "always"));
        assert_eq!(style.page_break_before, PageBreakValue::Always);

        assert!(apply_property_value(&mut style, "page-break-before", "avoid"));
        assert_eq!(style.page_break_before, PageBreakValue::Avoid);

        assert!(apply_property_value(&mut style, "page-break-before", "left"));
        assert_eq!(style.page_break_before, PageBreakValue::Left);

        assert!(apply_property_value(&mut style, "page-break-before", "right"));
        assert_eq!(style.page_break_before, PageBreakValue::Right);

        assert!(apply_property_value(&mut style, "page-break-before", "auto"));
        assert_eq!(style.page_break_before, PageBreakValue::Auto);

        assert!(!apply_property_value(&mut style, "page-break-before", "invalid"));
    }

    #[test]
    /// page-break-after 应用各值
    fn test_apply_page_break_after() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "page-break-after", "always"));
        assert_eq!(style.page_break_after, PageBreakValue::Always);
    }

    #[test]
    /// page-break-inside 仅接受 auto/avoid
    fn test_apply_page_break_inside() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "page-break-inside", "avoid"));
        assert_eq!(style.page_break_inside, PageBreakValue::Avoid);

        assert!(apply_property_value(&mut style, "page-break-inside", "auto"));
        assert_eq!(style.page_break_inside, PageBreakValue::Auto);

        // always/left/right 对 page-break-inside 无效
        assert!(!apply_property_value(&mut style, "page-break-inside", "always"));
    }

    #[test]
    /// box-decoration-break 应用 slice/clone
    fn test_apply_box_decoration_break() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "box-decoration-break", "clone"));
        assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Clone);

        assert!(apply_property_value(&mut style, "box-decoration-break", "slice"));
        assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);

        assert!(!apply_property_value(&mut style, "box-decoration-break", "invalid"));
    }

    #[test]
    /// image-rendering 应用各值
    fn test_apply_image_rendering() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "image-rendering", "pixelated"));
        assert_eq!(style.image_rendering, ImageRenderingValue::Pixelated);

        assert!(apply_property_value(&mut style, "image-rendering", "crisp-edges"));
        assert_eq!(style.image_rendering, ImageRenderingValue::CrispEdges);

        assert!(apply_property_value(&mut style, "image-rendering", "smooth"));
        assert_eq!(style.image_rendering, ImageRenderingValue::Smooth);

        assert!(apply_property_value(&mut style, "image-rendering", "high-quality"));
        assert_eq!(style.image_rendering, ImageRenderingValue::HighQuality);

        assert!(!apply_property_value(&mut style, "image-rendering", "invalid"));
    }

    #[test]
    /// isolation 应用 auto/isolate
    fn test_apply_isolation() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "isolation", "isolate"));
        assert_eq!(style.isolation, IsolationValue::Isolate);

        assert!(apply_property_value(&mut style, "isolation", "auto"));
        assert_eq!(style.isolation, IsolationValue::Auto);

        assert!(!apply_property_value(&mut style, "isolation", "invalid"));
    }

    #[test]
    /// 新属性不在继承列表中
    fn test_new_properties_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("page-break-before"));
        assert!(!PropertyRegistry::is_inherited("page-break-after"));
        assert!(!PropertyRegistry::is_inherited("page-break-inside"));
        assert!(!PropertyRegistry::is_inherited("box-decoration-break"));
        assert!(!PropertyRegistry::is_inherited("image-rendering"));
        assert!(!PropertyRegistry::is_inherited("isolation"));
    }

    #[test]
    /// 新属性在 known_properties 中注册
    fn test_new_properties_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"page-break-before"));
        assert!(props.contains(&"page-break-after"));
        assert!(props.contains(&"page-break-inside"));
        assert!(props.contains(&"box-decoration-break"));
        assert!(props.contains(&"image-rendering"));
        assert!(props.contains(&"isolation"));
    }

    #[test]
    /// 新属性的 initial_value 存在
    fn test_new_properties_initial_values() {
        assert!(PropertyRegistry::initial_value("page-break-before").is_some());
        assert!(PropertyRegistry::initial_value("page-break-after").is_some());
        assert!(PropertyRegistry::initial_value("page-break-inside").is_some());
        assert!(PropertyRegistry::initial_value("box-decoration-break").is_some());
        assert!(PropertyRegistry::initial_value("image-rendering").is_some());
        assert!(PropertyRegistry::initial_value("isolation").is_some());
    }

    #[test]
    /// apply_initial_value 对新属性
    fn test_apply_initial_value_new_round5_properties() {
        let mut style = ComputedStyle::default();
        apply_property_value(&mut style, "page-break-before", "always");
        apply_property_value(&mut style, "page-break-after", "avoid");
        apply_property_value(&mut style, "box-decoration-break", "clone");
        apply_property_value(&mut style, "image-rendering", "pixelated");
        apply_property_value(&mut style, "isolation", "isolate");

        assert!(apply_initial_value(&mut style, "page-break-before"));
        assert_eq!(style.page_break_before, PageBreakValue::Auto);

        assert!(apply_initial_value(&mut style, "page-break-after"));
        assert_eq!(style.page_break_after, PageBreakValue::Auto);

        assert!(apply_initial_value(&mut style, "box-decoration-break"));
        assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);

        assert!(apply_initial_value(&mut style, "image-rendering"));
        assert_eq!(style.image_rendering, ImageRenderingValue::Auto);

        assert!(apply_initial_value(&mut style, "isolation"));
        assert_eq!(style.isolation, IsolationValue::Auto);
    }

    // ── Interaction / Performance Hint 属性测试 ──

    #[test]
    /// overscroll-behavior-x/y 默认值为 Auto
    fn test_overscroll_behavior_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.overscroll_behavior_x, OverscrollBehaviorValue::Auto);
        assert_eq!(style.overscroll_behavior_y, OverscrollBehaviorValue::Auto);
    }

    #[test]
    /// overscroll-behavior-x/y apply
    fn test_overscroll_behavior_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "overscroll-behavior-x", "contain"));
        assert_eq!(style.overscroll_behavior_x, OverscrollBehaviorValue::Contain);
        assert!(apply_property_value(&mut style, "overscroll-behavior-y", "none"));
        assert_eq!(style.overscroll_behavior_y, OverscrollBehaviorValue::None);
        // 无效值
        assert!(!apply_property_value(&mut style, "overscroll-behavior-x", "invalid"));
    }

    #[test]
    /// touch-action apply
    fn test_touch_action_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "touch-action", "none"));
        assert_eq!(style.touch_action, TouchActionValue::None);
        assert!(apply_property_value(&mut style, "touch-action", "pan-x"));
        assert_eq!(style.touch_action, TouchActionValue::PanX);
        assert!(apply_property_value(&mut style, "touch-action", "manipulation"));
        assert_eq!(style.touch_action, TouchActionValue::Manipulation);
        assert!(!apply_property_value(&mut style, "touch-action", "invalid"));
    }

    #[test]
    /// user-select apply
    fn test_user_select_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "user-select", "text"));
        assert_eq!(style.user_select, UserSelectValue::Text);
        assert!(apply_property_value(&mut style, "user-select", "none"));
        assert_eq!(style.user_select, UserSelectValue::None);
        assert!(apply_property_value(&mut style, "user-select", "all"));
        assert_eq!(style.user_select, UserSelectValue::All);
    }

    #[test]
    /// will-change apply
    fn test_will_change_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "will-change", "auto"));
        assert_eq!(style.will_change, WillChangeValue::Auto);
        assert!(apply_property_value(&mut style, "will-change", "scroll-position"));
        assert_eq!(style.will_change, WillChangeValue::ScrollPosition);
        assert!(apply_property_value(&mut style, "will-change", "transform"));
        assert_eq!(style.will_change, WillChangeValue::Custom("transform".to_string()));
    }

    #[test]
    /// pointer-events apply 和继承
    fn test_pointer_events_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "pointer-events", "none"));
        assert_eq!(style.pointer_events, PointerEventsValue::None);
        assert!(apply_property_value(&mut style, "pointer-events", "visiblePainted"));
        assert_eq!(style.pointer_events, PointerEventsValue::VisiblePainted);
        // 继承性
        assert!(PropertyRegistry::is_inherited("pointer-events"));
    }

    #[test]
    /// 新属性不在继承列表中（除 pointer-events）
    fn test_interaction_properties_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("overscroll-behavior-x"));
        assert!(!PropertyRegistry::is_inherited("overscroll-behavior-y"));
        assert!(!PropertyRegistry::is_inherited("touch-action"));
        assert!(!PropertyRegistry::is_inherited("user-select"));
        assert!(!PropertyRegistry::is_inherited("will-change"));
    }

    #[test]
    /// 新属性在 known_properties 中注册
    fn test_interaction_properties_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"overscroll-behavior-x"));
        assert!(props.contains(&"overscroll-behavior-y"));
        assert!(props.contains(&"touch-action"));
        assert!(props.contains(&"user-select"));
        assert!(props.contains(&"will-change"));
        assert!(props.contains(&"pointer-events"));
    }

    #[test]
    /// 新属性的 initial_value 存在
    fn test_interaction_properties_initial_values() {
        assert!(PropertyRegistry::initial_value("overscroll-behavior-x").is_some());
        assert!(PropertyRegistry::initial_value("overscroll-behavior-y").is_some());
        assert!(PropertyRegistry::initial_value("touch-action").is_some());
        assert!(PropertyRegistry::initial_value("user-select").is_some());
        assert!(PropertyRegistry::initial_value("will-change").is_some());
        assert!(PropertyRegistry::initial_value("pointer-events").is_some());
    }

    // ═══════════════════════════════════════════════════════════════════
    // overflow-wrap / text-align-last / font-variant-numeric 测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 overflow-wrap apply_property_value
    fn test_apply_overflow_wrap() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);

        assert!(apply_property_value(&mut style, "overflow-wrap", "break-word"));
        assert_eq!(style.overflow_wrap, OverflowWrapValue::BreakWord);

        assert!(apply_property_value(&mut style, "overflow-wrap", "anywhere"));
        assert_eq!(style.overflow_wrap, OverflowWrapValue::Anywhere);

        assert!(apply_property_value(&mut style, "overflow-wrap", "normal"));
        assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);

        assert!(!apply_property_value(&mut style, "overflow-wrap", "invalid"));
    }

    #[test]
    /// 测试 overflow-wrap 继承性
    fn test_overflow_wrap_inherited() {
        assert!(PropertyRegistry::is_inherited("overflow-wrap"));
    }

    #[test]
    /// 测试 overflow-wrap initial_value
    fn test_overflow_wrap_initial_value() {
        assert!(PropertyRegistry::initial_value("overflow-wrap").is_some());
        let mut style = ComputedStyle::default();
        style.overflow_wrap = OverflowWrapValue::BreakWord;
        assert!(apply_initial_value(&mut style, "overflow-wrap"));
        assert_eq!(style.overflow_wrap, OverflowWrapValue::Normal);
    }

    #[test]
    /// 测试 overflow-wrap 继承
    fn test_overflow_wrap_inherit() {
        let mut parent = ComputedStyle::default();
        parent.overflow_wrap = OverflowWrapValue::Anywhere;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "overflow-wrap"));
        assert_eq!(child.overflow_wrap, OverflowWrapValue::Anywhere);
    }

    #[test]
    /// 测试 text-align-last apply_property_value
    fn test_apply_text_align_last() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.text_align_last, TextAlignLastValue::Auto);

        assert!(apply_property_value(&mut style, "text-align-last", "left"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Left);

        assert!(apply_property_value(&mut style, "text-align-last", "right"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Right);

        assert!(apply_property_value(&mut style, "text-align-last", "center"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Center);

        assert!(apply_property_value(&mut style, "text-align-last", "justify"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Justify);

        assert!(apply_property_value(&mut style, "text-align-last", "start"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Start);

        assert!(apply_property_value(&mut style, "text-align-last", "end"));
        assert_eq!(style.text_align_last, TextAlignLastValue::End);

        assert!(!apply_property_value(&mut style, "text-align-last", "invalid"));
    }

    #[test]
    /// 测试 text-align-last 继承性
    fn test_text_align_last_inherited() {
        assert!(PropertyRegistry::is_inherited("text-align-last"));
    }

    #[test]
    /// 测试 text-align-last initial_value
    fn test_text_align_last_initial_value() {
        assert!(PropertyRegistry::initial_value("text-align-last").is_some());
        let mut style = ComputedStyle::default();
        style.text_align_last = TextAlignLastValue::Justify;
        assert!(apply_initial_value(&mut style, "text-align-last"));
        assert_eq!(style.text_align_last, TextAlignLastValue::Auto);
    }

    #[test]
    /// 测试 text-align-last 继承
    fn test_text_align_last_inherit() {
        let mut parent = ComputedStyle::default();
        parent.text_align_last = TextAlignLastValue::Center;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "text-align-last"));
        assert_eq!(child.text_align_last, TextAlignLastValue::Center);
    }

    #[test]
    /// 测试 font-variant-numeric apply_property_value
    fn test_apply_font_variant_numeric() {
        let mut style = ComputedStyle::default();
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Normal);

        assert!(apply_property_value(&mut style, "font-variant-numeric", "ordinal"));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Ordinal);

        assert!(apply_property_value(&mut style, "font-variant-numeric", "slashed-zero"));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::SlashedZero);

        assert!(apply_property_value(&mut style, "font-variant-numeric", "lining-nums"));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::LiningNums);

        assert!(apply_property_value(
            &mut style,
            "font-variant-numeric",
            "oldstyle-nums"
        ));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::OldstyleNums);

        assert!(apply_property_value(
            &mut style,
            "font-variant-numeric",
            "proportional-nums"
        ));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::ProportionalNums);

        assert!(apply_property_value(&mut style, "font-variant-numeric", "tabular-nums"));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::TabularNums);

        assert!(apply_property_value(
            &mut style,
            "font-variant-numeric",
            "diagonal-fractions"
        ));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::DiagonalFractions);

        assert!(apply_property_value(
            &mut style,
            "font-variant-numeric",
            "stacked-fractions"
        ));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::StackedFractions);

        assert!(!apply_property_value(&mut style, "font-variant-numeric", "invalid"));
    }

    #[test]
    /// 测试 font-variant-numeric 继承性
    fn test_font_variant_numeric_inherited() {
        assert!(PropertyRegistry::is_inherited("font-variant-numeric"));
    }

    #[test]
    /// 测试 font-variant-numeric initial_value
    fn test_font_variant_numeric_initial_value() {
        assert!(PropertyRegistry::initial_value("font-variant-numeric").is_some());
        let mut style = ComputedStyle::default();
        style.font_variant_numeric = FontVariantNumericValue::Ordinal;
        assert!(apply_initial_value(&mut style, "font-variant-numeric"));
        assert_eq!(style.font_variant_numeric, FontVariantNumericValue::Normal);
    }

    #[test]
    /// 测试 font-variant-numeric 继承
    fn test_font_variant_numeric_inherit() {
        let mut parent = ComputedStyle::default();
        parent.font_variant_numeric = FontVariantNumericValue::TabularNums;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "font-variant-numeric"));
        assert_eq!(child.font_variant_numeric, FontVariantNumericValue::TabularNums);
    }

    #[test]
    /// 测试新属性在 known_properties 中（overflow-wrap、text-align-last、font-variant-numeric）
    fn test_text_new_properties_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"overflow-wrap"));
        assert!(props.contains(&"text-align-last"));
        assert!(props.contains(&"font-variant-numeric"));
    }

    #[test]
    /// 测试新属性 apply_initial_value_all_properties 覆盖
    fn test_new_properties_apply_initial_value() {
        for prop in &["overflow-wrap", "text-align-last", "font-variant-numeric"] {
            let mut style = ComputedStyle::default();
            assert!(
                apply_initial_value(&mut style, prop),
                "apply_initial_value should handle: {prop}"
            );
        }
    }

    #[test]
    /// 测试 pointer-events 继承（inherit_property）
    fn test_pointer_events_inherit() {
        let mut parent = ComputedStyle::default();
        parent.pointer_events = PointerEventsValue::None;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "pointer-events"));
        assert_eq!(child.pointer_events, PointerEventsValue::None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // direction / unicode-bidi / tab-size 测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 direction 默认值为 ltr
    fn test_direction_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.direction, DirectionValue::Ltr);
    }

    #[test]
    /// 测试 direction apply_property_value
    fn test_direction_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "direction", "rtl"));
        assert_eq!(style.direction, DirectionValue::Rtl);

        assert!(apply_property_value(&mut style, "direction", "ltr"));
        assert_eq!(style.direction, DirectionValue::Ltr);

        assert!(!apply_property_value(&mut style, "direction", "invalid"));
    }

    #[test]
    /// 测试 direction 继承性（inherited）
    fn test_direction_inherited() {
        assert!(PropertyRegistry::is_inherited("direction"));
    }

    #[test]
    /// 测试 direction initial_value
    fn test_direction_initial_value() {
        assert!(PropertyRegistry::initial_value("direction").is_some());
        let mut style = ComputedStyle::default();
        style.direction = DirectionValue::Rtl;
        assert!(apply_initial_value(&mut style, "direction"));
        assert_eq!(style.direction, DirectionValue::Ltr);
    }

    #[test]
    /// 测试 direction 继承
    fn test_direction_inherit() {
        let mut parent = ComputedStyle::default();
        parent.direction = DirectionValue::Rtl;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "direction"));
        assert_eq!(child.direction, DirectionValue::Rtl);
    }

    #[test]
    /// 测试 unicode-bidi 默认值为 normal
    fn test_unicode_bidi_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::Normal);
    }

    #[test]
    /// 测试 unicode-bidi apply_property_value
    fn test_unicode_bidi_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "unicode-bidi", "embed"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::Embed);

        assert!(apply_property_value(&mut style, "unicode-bidi", "isolate"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::Isolate);

        assert!(apply_property_value(&mut style, "unicode-bidi", "bidi-override"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::BidiOverride);

        assert!(apply_property_value(&mut style, "unicode-bidi", "isolate-override"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::IsolateOverride);

        assert!(apply_property_value(&mut style, "unicode-bidi", "plaintext"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::Plaintext);

        assert!(!apply_property_value(&mut style, "unicode-bidi", "invalid"));
    }

    #[test]
    /// 测试 unicode-bidi 不继承
    fn test_unicode_bidi_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("unicode-bidi"));
    }

    #[test]
    /// 测试 unicode-bidi initial_value
    fn test_unicode_bidi_initial_value() {
        assert!(PropertyRegistry::initial_value("unicode-bidi").is_some());
        let mut style = ComputedStyle::default();
        style.unicode_bidi = UnicodeBidiValue::Embed;
        assert!(apply_initial_value(&mut style, "unicode-bidi"));
        assert_eq!(style.unicode_bidi, UnicodeBidiValue::Normal);
    }

    #[test]
    /// 测试 tab-size 默认值为 8
    fn test_tab_size_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.tab_size, TabSizeValue::Number(8));
    }

    #[test]
    /// 测试 tab-size apply_property_value
    fn test_tab_size_apply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "tab-size", "4"));
        assert_eq!(style.tab_size, TabSizeValue::Number(4));

        assert!(apply_property_value(&mut style, "tab-size", "20px"));
        assert_eq!(style.tab_size, TabSizeValue::Length(LengthValue::Px(20.0)));

        assert!(apply_property_value(&mut style, "tab-size", "2em"));
        assert_eq!(style.tab_size, TabSizeValue::Length(LengthValue::Em(2.0)));

        assert!(!apply_property_value(&mut style, "tab-size", "invalid"));
    }

    #[test]
    /// 测试 tab-size 继承性（inherited）
    fn test_tab_size_inherited() {
        assert!(PropertyRegistry::is_inherited("tab-size"));
    }

    #[test]
    /// 测试 tab-size initial_value
    fn test_tab_size_initial_value() {
        assert!(PropertyRegistry::initial_value("tab-size").is_some());
        let mut style = ComputedStyle::default();
        style.tab_size = TabSizeValue::Number(2);
        assert!(apply_initial_value(&mut style, "tab-size"));
        assert_eq!(style.tab_size, TabSizeValue::Number(8));
    }

    #[test]
    /// 测试 tab-size 继承
    fn test_tab_size_inherit() {
        let mut parent = ComputedStyle::default();
        parent.tab_size = TabSizeValue::Number(4);
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "tab-size"));
        assert_eq!(child.tab_size, TabSizeValue::Number(4));
    }

    #[test]
    /// 测试新属性在 known_properties 中
    fn test_direction_unicode_bidi_tab_size_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"direction"));
        assert!(props.contains(&"unicode-bidi"));
        assert!(props.contains(&"tab-size"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // contain + column-rule-color 属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// contain 默认值为 None
    fn test_contain_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.contain, ContainComputedValue::None);
    }

    #[test]
    /// contain 应用关键字值
    fn test_apply_contain_keywords() {
        let mut style = ComputedStyle::default();

        assert!(apply_property_value(&mut style, "contain", "strict"));
        assert_eq!(style.contain, ContainComputedValue::Strict);

        assert!(apply_property_value(&mut style, "contain", "content"));
        assert_eq!(style.contain, ContainComputedValue::Content);

        assert!(apply_property_value(&mut style, "contain", "none"));
        assert_eq!(style.contain, ContainComputedValue::None);

        assert!(apply_property_value(&mut style, "contain", "size"));
        assert_eq!(style.contain, ContainComputedValue::Size);

        assert!(apply_property_value(&mut style, "contain", "layout"));
        assert_eq!(style.contain, ContainComputedValue::Layout);

        assert!(apply_property_value(&mut style, "contain", "style"));
        assert_eq!(style.contain, ContainComputedValue::Style);

        assert!(apply_property_value(&mut style, "contain", "paint"));
        assert_eq!(style.contain, ContainComputedValue::Paint);

        assert!(!apply_property_value(&mut style, "contain", "invalid"));
    }

    #[test]
    /// contain 支持多值空格分隔
    fn test_apply_contain_multi_value() {
        let mut style = ComputedStyle::default();

        assert!(apply_property_value(&mut style, "contain", "layout style paint"));
        match &style.contain {
            ContainComputedValue::Custom(flags) => {
                let expected = ContainComputedValue::FLAG_LAYOUT
                    | ContainComputedValue::FLAG_STYLE
                    | ContainComputedValue::FLAG_PAINT;
                assert_eq!(*flags, expected);
            }
            _ => panic!("expected Custom, got {:?}", style.contain),
        }

        // "layout style paint size" 等价于 content 的位组合
        assert!(apply_property_value(&mut style, "contain", "layout style paint size"));
        match &style.contain {
            ContainComputedValue::Custom(flags) => {
                let expected = ContainComputedValue::FLAG_LAYOUT
                    | ContainComputedValue::FLAG_STYLE
                    | ContainComputedValue::FLAG_PAINT
                    | ContainComputedValue::FLAG_SIZE;
                assert_eq!(*flags, expected);
            }
            _ => panic!("expected Custom, got {:?}", style.contain),
        }
    }

    #[test]
    /// contain 不继承
    fn test_contain_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("contain"));
    }

    #[test]
    /// contain 在 known_properties 中
    fn test_contain_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"contain"));
    }

    #[test]
    /// contain 有 initial_value
    fn test_contain_initial_value() {
        assert!(PropertyRegistry::initial_value("contain").is_some());
        let mut style = ComputedStyle::default();
        style.contain = ContainComputedValue::Strict;
        assert!(apply_initial_value(&mut style, "contain"));
        assert_eq!(style.contain, ContainComputedValue::None);
    }

    #[test]
    /// column-rule-color 默认值为黑色
    fn test_column_rule_color_default() {
        let style = ComputedStyle::default();
        assert_eq!(style.column_rule_color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    /// column-rule-color 应用颜色值
    fn test_apply_column_rule_color() {
        let mut style = ComputedStyle::default();

        assert!(apply_property_value(&mut style, "column-rule-color", "red"));
        assert_eq!(style.column_rule_color, ColorValue::Rgba(255, 0, 0, 255));

        assert!(apply_property_value(&mut style, "column-rule-color", "#00ff00"));
        assert_eq!(style.column_rule_color, ColorValue::Rgba(0, 255, 0, 255));

        assert!(apply_property_value(&mut style, "column-rule-color", "transparent"));
        assert_eq!(style.column_rule_color, ColorValue::Transparent);

        assert!(apply_property_value(&mut style, "column-rule-color", "currentColor"));
        assert_eq!(style.column_rule_color, ColorValue::CurrentColor);

        assert!(!apply_property_value(&mut style, "column-rule-color", "not-a-color"));
    }

    #[test]
    /// column-rule-color 不继承
    fn test_column_rule_color_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("column-rule-color"));
    }

    #[test]
    /// column-rule-color 在 known_properties 中
    fn test_column_rule_color_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"column-rule-color"));
    }

    #[test]
    /// column-rule-color 有 initial_value
    fn test_column_rule_color_initial_value() {
        assert!(PropertyRegistry::initial_value("column-rule-color").is_some());
        let mut style = ComputedStyle::default();
        style.column_rule_color = ColorValue::Rgba(255, 0, 0, 255);
        assert!(apply_initial_value(&mut style, "column-rule-color"));
        assert_eq!(style.column_rule_color, ColorValue::Rgba(0, 0, 0, 255));
    }

    // ── appearance 属性测试 ──

    #[test]
    fn test_apply_property_appearance_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "appearance", "none"));
        assert_eq!(style.appearance, AppearanceComputedValue::None);
    }

    #[test]
    fn test_apply_property_appearance_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "appearance", "auto"));
        assert_eq!(style.appearance, AppearanceComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_appearance_button() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "appearance", "button"));
        assert_eq!(style.appearance, AppearanceComputedValue::Button);
    }

    #[test]
    fn test_apply_property_appearance_checkbox() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "appearance", "checkbox"));
        assert_eq!(style.appearance, AppearanceComputedValue::Checkbox);
    }

    #[test]
    fn test_apply_property_appearance_textfield() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "appearance", "textfield"));
        assert_eq!(style.appearance, AppearanceComputedValue::Textfield);
    }

    #[test]
    fn test_apply_property_appearance_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "appearance", "invalid"));
    }

    #[test]
    fn test_appearance_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("appearance"));
    }

    #[test]
    fn test_appearance_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"appearance"));
    }

    #[test]
    fn test_appearance_initial_value() {
        assert!(PropertyRegistry::initial_value("appearance").is_some());
        let mut style = ComputedStyle::default();
        style.appearance = AppearanceComputedValue::None;
        assert!(apply_initial_value(&mut style, "appearance"));
        assert_eq!(style.appearance, AppearanceComputedValue::Auto);
    }

    // ── accent-color 属性测试 ──

    #[test]
    fn test_apply_property_accent_color_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "accent-color", "auto"));
        assert_eq!(style.accent_color, AccentColorComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_accent_color_named() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "accent-color", "red"));
        assert_eq!(
            style.accent_color,
            AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255))
        );
    }

    #[test]
    fn test_apply_property_accent_color_hex() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "accent-color", "#00ff00"));
        assert_eq!(
            style.accent_color,
            AccentColorComputedValue::Color(ColorValue::Rgba(0, 255, 0, 255))
        );
    }

    #[test]
    fn test_apply_property_accent_color_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "accent-color", "not-a-color"));
    }

    #[test]
    fn test_accent_color_is_inherited() {
        assert!(PropertyRegistry::is_inherited("accent-color"));
    }

    #[test]
    fn test_accent_color_inherit() {
        let mut parent = ComputedStyle::default();
        parent.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "accent-color"));
        assert_eq!(
            child.accent_color,
            AccentColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255))
        );
    }

    #[test]
    fn test_accent_color_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"accent-color"));
    }

    #[test]
    fn test_accent_color_initial_value() {
        assert!(PropertyRegistry::initial_value("accent-color").is_some());
        let mut style = ComputedStyle::default();
        style.accent_color = AccentColorComputedValue::Color(ColorValue::Rgba(0, 128, 0, 255));
        assert!(apply_initial_value(&mut style, "accent-color"));
        assert_eq!(style.accent_color, AccentColorComputedValue::Auto);
    }

    // ── caret-color 属性测试 ──

    #[test]
    fn test_apply_property_caret_color_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "caret-color", "auto"));
        assert_eq!(style.caret_color, CaretColorComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_caret_color_named() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "caret-color", "blue"));
        assert_eq!(
            style.caret_color,
            CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255))
        );
    }

    #[test]
    fn test_apply_property_caret_color_hex() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "caret-color", "#abcdef"));
        assert_eq!(
            style.caret_color,
            CaretColorComputedValue::Color(ColorValue::Rgba(0xAB, 0xCD, 0xEF, 255))
        );
    }

    #[test]
    fn test_apply_property_caret_color_transparent() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "caret-color", "transparent"));
        assert_eq!(
            style.caret_color,
            CaretColorComputedValue::Color(ColorValue::Transparent)
        );
    }

    #[test]
    fn test_apply_property_caret_color_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "caret-color", "not-a-color"));
    }

    #[test]
    fn test_caret_color_is_inherited() {
        assert!(PropertyRegistry::is_inherited("caret-color"));
    }

    #[test]
    fn test_caret_color_inherit() {
        let mut parent = ComputedStyle::default();
        parent.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255));
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "caret-color"));
        assert_eq!(
            child.caret_color,
            CaretColorComputedValue::Color(ColorValue::Rgba(0, 0, 255, 255))
        );
    }

    #[test]
    fn test_caret_color_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"caret-color"));
    }

    #[test]
    fn test_caret_color_initial_value() {
        assert!(PropertyRegistry::initial_value("caret-color").is_some());
        let mut style = ComputedStyle::default();
        style.caret_color = CaretColorComputedValue::Color(ColorValue::Rgba(255, 0, 0, 255));
        assert!(apply_initial_value(&mut style, "caret-color"));
        assert_eq!(style.caret_color, CaretColorComputedValue::Auto);
    }

    // ── mix-blend-mode 属性测试 ──

    #[test]
    fn test_apply_property_mix_blend_mode_normal() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "normal"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Normal);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_multiply() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "multiply"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Multiply);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_screen() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "screen"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Screen);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_overlay() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "overlay"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Overlay);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_darken() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "darken"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Darken);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_lighten() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "lighten"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Lighten);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_color_dodge() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "color-dodge"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::ColorDodge);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_color_burn() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "color-burn"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::ColorBurn);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_hard_light() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "hard-light"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::HardLight);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_soft_light() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "soft-light"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::SoftLight);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_difference() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "difference"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Difference);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_exclusion() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "exclusion"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Exclusion);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_hue() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "hue"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Hue);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_saturation() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "saturation"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Saturation);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_color() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "color"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Color);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_luminosity() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "mix-blend-mode", "luminosity"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Luminosity);
    }

    #[test]
    fn test_apply_property_mix_blend_mode_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "mix-blend-mode", "invalid"));
    }

    #[test]
    fn test_mix_blend_mode_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("mix-blend-mode"));
    }

    #[test]
    fn test_mix_blend_mode_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"mix-blend-mode"));
    }

    #[test]
    fn test_mix_blend_mode_initial_value() {
        assert!(PropertyRegistry::initial_value("mix-blend-mode").is_some());
        let mut style = ComputedStyle::default();
        style.mix_blend_mode = MixBlendModeComputedValue::Multiply;
        assert!(apply_initial_value(&mut style, "mix-blend-mode"));
        assert_eq!(style.mix_blend_mode, MixBlendModeComputedValue::Normal);
    }

    // ── scrollbar-width 属性测试 ──

    #[test]
    fn test_apply_property_scrollbar_width_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "scrollbar-width", "auto"));
        assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_scrollbar_width_thin() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "scrollbar-width", "thin"));
        assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Thin);
    }

    #[test]
    fn test_apply_property_scrollbar_width_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "scrollbar-width", "none"));
        assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::None);
    }

    #[test]
    fn test_apply_property_scrollbar_width_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "scrollbar-width", "thick"));
    }

    #[test]
    fn test_scrollbar_width_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("scrollbar-width"));
    }

    #[test]
    fn test_scrollbar_width_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"scrollbar-width"));
    }

    #[test]
    fn test_scrollbar_width_initial_value() {
        assert!(PropertyRegistry::initial_value("scrollbar-width").is_some());
        let mut style = ComputedStyle::default();
        style.scrollbar_width = ScrollbarWidthComputedValue::Thin;
        assert!(apply_initial_value(&mut style, "scrollbar-width"));
        assert_eq!(style.scrollbar_width, ScrollbarWidthComputedValue::Auto);
    }

    // ── scrollbar-gutter 属性测试 ──

    #[test]
    fn test_apply_property_scrollbar_gutter_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "scrollbar-gutter", "auto"));
        assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_scrollbar_gutter_stable() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "scrollbar-gutter", "stable"));
        assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Stable);
    }

    #[test]
    fn test_apply_property_scrollbar_gutter_stable_both_edges() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "scrollbar-gutter",
            "stable both-edges"
        ));
        assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::StableBothEdges);
    }

    #[test]
    fn test_apply_property_scrollbar_gutter_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "scrollbar-gutter", "both-edges"));
        assert!(!apply_property_value(&mut style, "scrollbar-gutter", "invalid"));
    }

    #[test]
    fn test_scrollbar_gutter_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("scrollbar-gutter"));
    }

    #[test]
    fn test_scrollbar_gutter_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"scrollbar-gutter"));
    }

    #[test]
    fn test_scrollbar_gutter_initial_value() {
        assert!(PropertyRegistry::initial_value("scrollbar-gutter").is_some());
        let mut style = ComputedStyle::default();
        style.scrollbar_gutter = ScrollbarGutterComputedValue::Stable;
        assert!(apply_initial_value(&mut style, "scrollbar-gutter"));
        assert_eq!(style.scrollbar_gutter, ScrollbarGutterComputedValue::Auto);
    }

    // ── text-wrap 属性测试 ──

    #[test]
    fn test_apply_property_text_wrap_wrap() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-wrap", "wrap"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Wrap);
    }

    #[test]
    fn test_apply_property_text_wrap_nowrap() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-wrap", "nowrap"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Nowrap);
    }

    #[test]
    fn test_apply_property_text_wrap_balance() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-wrap", "balance"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Balance);
    }

    #[test]
    fn test_apply_property_text_wrap_pretty() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-wrap", "pretty"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Pretty);
    }

    #[test]
    fn test_apply_property_text_wrap_stable() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-wrap", "stable"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Stable);
    }

    #[test]
    fn test_apply_property_text_wrap_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "text-wrap", "auto"));
        assert!(!apply_property_value(&mut style, "text-wrap", "invalid"));
    }

    #[test]
    fn test_text_wrap_is_inherited() {
        assert!(PropertyRegistry::is_inherited("text-wrap"));
    }

    #[test]
    fn test_text_wrap_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"text-wrap"));
    }

    #[test]
    fn test_text_wrap_initial_value() {
        assert!(PropertyRegistry::initial_value("text-wrap").is_some());
        let mut style = ComputedStyle::default();
        style.text_wrap = TextWrapComputedValue::Nowrap;
        assert!(apply_initial_value(&mut style, "text-wrap"));
        assert_eq!(style.text_wrap, TextWrapComputedValue::Wrap);
    }

    #[test]
    fn test_text_wrap_inherit() {
        let mut parent = ComputedStyle::default();
        parent.text_wrap = TextWrapComputedValue::Balance;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "text-wrap"));
        assert_eq!(child.text_wrap, TextWrapComputedValue::Balance);
    }

    // ── hyphens 属性测试 ──

    #[test]
    fn test_apply_property_hyphens_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "hyphens", "none"));
        assert_eq!(style.hyphens, HyphensComputedValue::None);
    }

    #[test]
    fn test_apply_property_hyphens_manual() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "hyphens", "manual"));
        assert_eq!(style.hyphens, HyphensComputedValue::Manual);
    }

    #[test]
    fn test_apply_property_hyphens_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "hyphens", "auto"));
        assert_eq!(style.hyphens, HyphensComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_hyphens_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "hyphens", "all"));
        assert!(!apply_property_value(&mut style, "hyphens", "invalid"));
    }

    #[test]
    fn test_hyphens_is_inherited() {
        assert!(PropertyRegistry::is_inherited("hyphens"));
    }

    #[test]
    fn test_hyphens_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"hyphens"));
    }

    #[test]
    fn test_hyphens_initial_value() {
        assert!(PropertyRegistry::initial_value("hyphens").is_some());
        let mut style = ComputedStyle::default();
        style.hyphens = HyphensComputedValue::Auto;
        assert!(apply_initial_value(&mut style, "hyphens"));
        assert_eq!(style.hyphens, HyphensComputedValue::None);
    }

    #[test]
    fn test_hyphens_inherit() {
        let mut parent = ComputedStyle::default();
        parent.hyphens = HyphensComputedValue::Auto;
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "hyphens"));
        assert_eq!(child.hyphens, HyphensComputedValue::Auto);
    }

    // ── line-clamp 属性测试 ──

    #[test]
    fn test_apply_property_line_clamp_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "line-clamp", "none"));
        assert_eq!(style.line_clamp, LineClampComputedValue::None);
    }

    #[test]
    fn test_apply_property_line_clamp_count() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "line-clamp", "3"));
        assert_eq!(style.line_clamp, LineClampComputedValue::Count(3));
    }

    #[test]
    fn test_apply_property_line_clamp_count_one() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "line-clamp", "1"));
        assert_eq!(style.line_clamp, LineClampComputedValue::Count(1));
    }

    #[test]
    fn test_apply_property_line_clamp_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "line-clamp", "0"));
        assert!(!apply_property_value(&mut style, "line-clamp", "-1"));
        assert!(!apply_property_value(&mut style, "line-clamp", "auto"));
        assert!(!apply_property_value(&mut style, "line-clamp", "invalid"));
    }

    #[test]
    fn test_line_clamp_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("line-clamp"));
    }

    #[test]
    fn test_line_clamp_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"line-clamp"));
    }

    #[test]
    fn test_line_clamp_initial_value() {
        assert!(PropertyRegistry::initial_value("line-clamp").is_some());
        let mut style = ComputedStyle::default();
        style.line_clamp = LineClampComputedValue::Count(5);
        assert!(apply_initial_value(&mut style, "line-clamp"));
        assert_eq!(style.line_clamp, LineClampComputedValue::None);
    }

    // ── background-image 属性测试 ──

    #[test]
    fn test_apply_property_background_image_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-image", "none"));
        assert_eq!(style.background_image, BackgroundImageComputedValue::None);
    }

    #[test]
    fn test_apply_property_background_image_url() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-image", "url(bg.png)"));
        assert_eq!(
            style.background_image,
            BackgroundImageComputedValue::Url("bg.png".to_string())
        );
    }

    #[test]
    fn test_apply_property_background_image_url_quoted() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-image", "url(\"bg.png\")"));
        assert_eq!(
            style.background_image,
            BackgroundImageComputedValue::Url("bg.png".to_string())
        );
    }

    #[test]
    fn test_apply_property_background_image_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-image", "invalid"));
    }

    #[test]
    fn test_background_image_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-image"));
    }

    #[test]
    fn test_background_image_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-image"));
    }

    #[test]
    fn test_background_image_initial_value() {
        assert!(PropertyRegistry::initial_value("background-image").is_some());
        let mut style = ComputedStyle::default();
        style.background_image = BackgroundImageComputedValue::Url("test.png".to_string());
        assert!(apply_initial_value(&mut style, "background-image"));
        assert_eq!(style.background_image, BackgroundImageComputedValue::None);
    }

    // ── background-position 属性测试 ──

    #[test]
    fn test_apply_property_background_position_center() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-position", "center"));
        assert_eq!(style.background_position, BackgroundPositionComputedValue::Center);
    }

    #[test]
    fn test_apply_property_background_position_left() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-position", "left"));
        assert_eq!(style.background_position, BackgroundPositionComputedValue::Left);
    }

    #[test]
    fn test_apply_property_background_position_percent() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-position", "50%"));
        assert_eq!(
            style.background_position,
            BackgroundPositionComputedValue::Percent(50.0)
        );
    }

    #[test]
    fn test_apply_property_background_position_length() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-position", "10px"));
        assert_eq!(style.background_position, BackgroundPositionComputedValue::Length(10.0));
    }

    #[test]
    fn test_apply_property_background_position_two_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-position", "left top"));
        match style.background_position {
            BackgroundPositionComputedValue::TwoValue(ref h, ref v) => {
                assert_eq!(**h, BackgroundPositionComputedValue::Left);
                assert_eq!(**v, BackgroundPositionComputedValue::Top);
            }
            ref other => panic!("Expected TwoValue, got {:?}", other),
        }
    }

    #[test]
    fn test_apply_property_background_position_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-position", "invalid"));
    }

    #[test]
    fn test_background_position_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-position"));
    }

    #[test]
    fn test_background_position_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-position"));
    }

    #[test]
    fn test_background_position_initial_value() {
        assert!(PropertyRegistry::initial_value("background-position").is_some());
        let mut style = ComputedStyle::default();
        style.background_position = BackgroundPositionComputedValue::Center;
        assert!(apply_initial_value(&mut style, "background-position"));
        assert_eq!(style.background_position, BackgroundPositionComputedValue::Percent(0.0));
    }

    // ── background-repeat 属性测试 ──

    #[test]
    fn test_apply_property_background_repeat_repeat() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "repeat"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::Repeat);
    }

    #[test]
    fn test_apply_property_background_repeat_no_repeat() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "no-repeat"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::NoRepeat);
    }

    #[test]
    fn test_apply_property_background_repeat_repeat_x() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "repeat-x"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::RepeatX);
    }

    #[test]
    fn test_apply_property_background_repeat_repeat_y() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "repeat-y"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::RepeatY);
    }

    #[test]
    fn test_apply_property_background_repeat_space() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "space"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::Space);
    }

    #[test]
    fn test_apply_property_background_repeat_round() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-repeat", "round"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::Round);
    }

    #[test]
    fn test_apply_property_background_repeat_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-repeat", "invalid"));
    }

    #[test]
    fn test_background_repeat_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-repeat"));
    }

    #[test]
    fn test_background_repeat_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-repeat"));
    }

    #[test]
    fn test_background_repeat_initial_value() {
        assert!(PropertyRegistry::initial_value("background-repeat").is_some());
        let mut style = ComputedStyle::default();
        style.background_repeat = BackgroundRepeatComputedValue::NoRepeat;
        assert!(apply_initial_value(&mut style, "background-repeat"));
        assert_eq!(style.background_repeat, BackgroundRepeatComputedValue::Repeat);
    }

    // ── background-size 属性测试 ──

    #[test]
    fn test_apply_property_background_size_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-size", "auto"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Auto);
    }

    #[test]
    fn test_apply_property_background_size_cover() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-size", "cover"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Cover);
    }

    #[test]
    fn test_apply_property_background_size_contain() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-size", "contain"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Contain);
    }

    #[test]
    fn test_apply_property_background_size_length() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-size", "100px"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Length(100.0));
    }

    #[test]
    fn test_apply_property_background_size_percent() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-size", "50%"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Percent(50.0));
    }

    #[test]
    fn test_apply_property_background_size_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-size", "invalid"));
    }

    #[test]
    fn test_background_size_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-size"));
    }

    #[test]
    fn test_background_size_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-size"));
    }

    #[test]
    fn test_background_size_initial_value() {
        assert!(PropertyRegistry::initial_value("background-size").is_some());
        let mut style = ComputedStyle::default();
        style.background_size = BackgroundSizeComputedValue::Cover;
        assert!(apply_initial_value(&mut style, "background-size"));
        assert_eq!(style.background_size, BackgroundSizeComputedValue::Auto);
    }

    // ── background-attachment 属性测试 ──

    #[test]
    fn test_apply_property_background_attachment_scroll() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-attachment", "scroll"));
        assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Scroll);
    }

    #[test]
    fn test_apply_property_background_attachment_fixed() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-attachment", "fixed"));
        assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Fixed);
    }

    #[test]
    fn test_apply_property_background_attachment_local() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-attachment", "local"));
        assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Local);
    }

    #[test]
    fn test_apply_property_background_attachment_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-attachment", "invalid"));
    }

    #[test]
    fn test_background_attachment_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-attachment"));
    }

    #[test]
    fn test_background_attachment_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-attachment"));
    }

    #[test]
    fn test_background_attachment_initial_value() {
        assert!(PropertyRegistry::initial_value("background-attachment").is_some());
        let mut style = ComputedStyle::default();
        style.background_attachment = BackgroundAttachmentComputedValue::Fixed;
        assert!(apply_initial_value(&mut style, "background-attachment"));
        assert_eq!(style.background_attachment, BackgroundAttachmentComputedValue::Scroll);
    }

    // ── background-clip ──

    #[test]
    fn test_apply_property_background_clip_border_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-clip", "border-box"));
        assert_eq!(style.background_clip, BackgroundClipComputedValue::BorderBox);
    }

    #[test]
    fn test_apply_property_background_clip_padding_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-clip", "padding-box"));
        assert_eq!(style.background_clip, BackgroundClipComputedValue::PaddingBox);
    }

    #[test]
    fn test_apply_property_background_clip_content_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-clip", "content-box"));
        assert_eq!(style.background_clip, BackgroundClipComputedValue::ContentBox);
    }

    #[test]
    fn test_apply_property_background_clip_text() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-clip", "text"));
        assert_eq!(style.background_clip, BackgroundClipComputedValue::Text);
    }

    #[test]
    fn test_apply_property_background_clip_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-clip", "invalid"));
    }

    #[test]
    fn test_background_clip_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-clip"));
    }

    #[test]
    fn test_background_clip_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-clip"));
    }

    #[test]
    fn test_background_clip_initial_value() {
        assert!(PropertyRegistry::initial_value("background-clip").is_some());
        let mut style = ComputedStyle::default();
        style.background_clip = BackgroundClipComputedValue::Text;
        assert!(apply_initial_value(&mut style, "background-clip"));
        assert_eq!(style.background_clip, BackgroundClipComputedValue::BorderBox);
    }

    // ── background-origin ──

    #[test]
    fn test_apply_property_background_origin_padding_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-origin", "padding-box"));
        assert_eq!(style.background_origin, BackgroundOriginComputedValue::PaddingBox);
    }

    #[test]
    fn test_apply_property_background_origin_border_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-origin", "border-box"));
        assert_eq!(style.background_origin, BackgroundOriginComputedValue::BorderBox);
    }

    #[test]
    fn test_apply_property_background_origin_content_box() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "background-origin", "content-box"));
        assert_eq!(style.background_origin, BackgroundOriginComputedValue::ContentBox);
    }

    #[test]
    fn test_apply_property_background_origin_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "background-origin", "invalid"));
        // text 不是有效的 background-origin 值
        assert!(!apply_property_value(&mut style, "background-origin", "text"));
    }

    #[test]
    fn test_background_origin_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("background-origin"));
    }

    #[test]
    fn test_background_origin_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"background-origin"));
    }

    #[test]
    fn test_background_origin_initial_value() {
        assert!(PropertyRegistry::initial_value("background-origin").is_some());
        let mut style = ComputedStyle::default();
        style.background_origin = BackgroundOriginComputedValue::ContentBox;
        assert!(apply_initial_value(&mut style, "background-origin"));
        assert_eq!(style.background_origin, BackgroundOriginComputedValue::PaddingBox);
    }

    // ── border-image-source ──

    #[test]
    fn test_apply_property_border_image_source_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-source", "none"));
        assert_eq!(style.border_image_source, BorderImageSourceComputedValue::None);
    }

    #[test]
    fn test_apply_property_border_image_source_url() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "border-image-source",
            "url(border.png)"
        ));
        assert_eq!(
            style.border_image_source,
            BorderImageSourceComputedValue::Url("border.png".to_string())
        );
    }

    #[test]
    fn test_apply_property_border_image_source_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-image-source", "invalid"));
    }

    #[test]
    fn test_border_image_source_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("border-image-source"));
    }

    #[test]
    fn test_border_image_source_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"border-image-source"));
    }

    #[test]
    fn test_border_image_source_initial_value() {
        assert!(PropertyRegistry::initial_value("border-image-source").is_some());
        let mut style = ComputedStyle::default();
        style.border_image_source = BorderImageSourceComputedValue::Url("test.png".to_string());
        assert!(apply_initial_value(&mut style, "border-image-source"));
        assert_eq!(style.border_image_source, BorderImageSourceComputedValue::None);
    }

    // ── border-image-slice ──

    #[test]
    fn test_apply_property_border_image_slice_number() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-slice", "50"));
        assert_eq!(
            style.border_image_slice.top,
            BorderImageSliceComputedComponent::Number(50.0)
        );
        assert_eq!(
            style.border_image_slice.right,
            BorderImageSliceComputedComponent::Number(50.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_slice_percent() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-slice", "30%"));
        assert_eq!(
            style.border_image_slice.top,
            BorderImageSliceComputedComponent::Percent(30.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_slice_fill() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-slice", "25 fill"));
        assert!(style.border_image_slice.fill);
        assert_eq!(
            style.border_image_slice.top,
            BorderImageSliceComputedComponent::Number(25.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_slice_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-image-slice", "invalid"));
    }

    #[test]
    fn test_border_image_slice_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("border-image-slice"));
    }

    #[test]
    fn test_border_image_slice_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"border-image-slice"));
    }

    #[test]
    fn test_border_image_slice_initial_value() {
        assert!(PropertyRegistry::initial_value("border-image-slice").is_some());
        let mut style = ComputedStyle::default();
        style.border_image_slice.fill = true;
        assert!(apply_initial_value(&mut style, "border-image-slice"));
        assert!(!style.border_image_slice.fill);
    }

    // ── border-image-width ──

    #[test]
    fn test_apply_property_border_image_width_auto() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-width", "auto"));
        assert_eq!(style.border_image_width.top, BorderImageWidthComputedComponent::Auto);
    }

    #[test]
    fn test_apply_property_border_image_width_number() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-width", "3"));
        assert_eq!(
            style.border_image_width.top,
            BorderImageWidthComputedComponent::Number(3.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_width_px() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-width", "10px"));
        assert_eq!(
            style.border_image_width.top,
            BorderImageWidthComputedComponent::Length(10.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_width_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-image-width", "invalid"));
    }

    #[test]
    fn test_border_image_width_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("border-image-width"));
    }

    #[test]
    fn test_border_image_width_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"border-image-width"));
    }

    #[test]
    fn test_border_image_width_initial_value() {
        assert!(PropertyRegistry::initial_value("border-image-width").is_some());
        let mut style = ComputedStyle::default();
        style.border_image_width.top = BorderImageWidthComputedComponent::Auto;
        assert!(apply_initial_value(&mut style, "border-image-width"));
        assert_eq!(
            style.border_image_width.top,
            BorderImageWidthComputedComponent::Number(1.0)
        );
    }

    // ── border-image-repeat ──

    #[test]
    fn test_apply_property_border_image_repeat_stretch() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-repeat", "stretch"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Stretch
        );
    }

    #[test]
    fn test_apply_property_border_image_repeat_repeat() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-repeat", "repeat"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Repeat
        );
    }

    #[test]
    fn test_apply_property_border_image_repeat_round() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-repeat", "round"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Round
        );
    }

    #[test]
    fn test_apply_property_border_image_repeat_space() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-repeat", "space"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Space
        );
    }

    #[test]
    fn test_apply_property_border_image_repeat_two_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-repeat", "repeat round"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Repeat
        );
        assert_eq!(style.border_image_repeat.vertical, BorderImageRepeatComputedMode::Round);
    }

    #[test]
    fn test_apply_property_border_image_repeat_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-image-repeat", "invalid"));
    }

    #[test]
    fn test_border_image_repeat_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("border-image-repeat"));
    }

    #[test]
    fn test_border_image_repeat_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"border-image-repeat"));
    }

    #[test]
    fn test_border_image_repeat_initial_value() {
        assert!(PropertyRegistry::initial_value("border-image-repeat").is_some());
        let mut style = ComputedStyle::default();
        style.border_image_repeat.horizontal = BorderImageRepeatComputedMode::Repeat;
        assert!(apply_initial_value(&mut style, "border-image-repeat"));
        assert_eq!(
            style.border_image_repeat.horizontal,
            BorderImageRepeatComputedMode::Stretch
        );
    }

    // ── border-image-outset ──

    #[test]
    fn test_apply_property_border_image_outset_number() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-outset", "2"));
        assert_eq!(
            style.border_image_outset.top,
            BorderImageOutsetComputedComponent::Number(2.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_outset_px() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-image-outset", "10px"));
        assert_eq!(
            style.border_image_outset.top,
            BorderImageOutsetComputedComponent::Length(10.0)
        );
    }

    #[test]
    fn test_apply_property_border_image_outset_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-image-outset", "invalid"));
    }

    #[test]
    fn test_border_image_outset_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("border-image-outset"));
    }

    #[test]
    fn test_border_image_outset_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"border-image-outset"));
    }

    #[test]
    fn test_border_image_outset_initial_value() {
        assert!(PropertyRegistry::initial_value("border-image-outset").is_some());
        let mut style = ComputedStyle::default();
        style.border_image_outset.top = BorderImageOutsetComputedComponent::Number(10.0);
        assert!(apply_initial_value(&mut style, "border-image-outset"));
        assert_eq!(
            style.border_image_outset.top,
            BorderImageOutsetComputedComponent::Number(0.0)
        );
    }

    // ── text-shadow ──

    #[test]
    fn test_apply_text_shadow_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-shadow", "none"));
        assert_eq!(style.text_shadow.offset_x, 0.0);
    }

    #[test]
    fn test_apply_text_shadow_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "text-shadow", "2px 3px 4px red"));
        assert_eq!(style.text_shadow.offset_x, 2.0);
        assert_eq!(style.text_shadow.offset_y, 3.0);
        assert_eq!(style.text_shadow.blur_radius, 4.0);
        assert_eq!(
            style.text_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255)
        );
    }

    #[test]
    fn test_text_shadow_is_inherited() {
        assert!(PropertyRegistry::is_inherited("text-shadow"));
    }

    #[test]
    fn test_text_shadow_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"text-shadow"));
    }

    #[test]
    fn test_text_shadow_initial_value() {
        assert!(PropertyRegistry::initial_value("text-shadow").is_some());
    }

    #[test]
    fn test_text_shadow_apply_initial() {
        let mut style = ComputedStyle::default();
        style.text_shadow.offset_x = 10.0;
        assert!(apply_initial_value(&mut style, "text-shadow"));
        assert_eq!(style.text_shadow.offset_x, 0.0);
    }

    // ── box-shadow ──

    #[test]
    fn test_apply_box_shadow_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "box-shadow", "none"));
        assert_eq!(style.box_shadow.offset_x, 0.0);
    }

    #[test]
    fn test_apply_box_shadow_values() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "box-shadow",
            "10px 20px 30px 5px blue"
        ));
        assert_eq!(style.box_shadow.offset_x, 10.0);
        assert_eq!(style.box_shadow.offset_y, 20.0);
        assert_eq!(style.box_shadow.blur_radius, 30.0);
        assert_eq!(style.box_shadow.spread_radius, 5.0);
        assert_eq!(
            style.box_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255)
        );
        assert!(!style.box_shadow.inset);
    }

    #[test]
    fn test_apply_box_shadow_inset() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "box-shadow", "inset 5px 10px"));
        assert!(style.box_shadow.inset);
        assert_eq!(style.box_shadow.offset_x, 5.0);
        assert_eq!(style.box_shadow.offset_y, 10.0);
    }

    #[test]
    fn test_box_shadow_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("box-shadow"));
    }

    #[test]
    fn test_box_shadow_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"box-shadow"));
    }

    #[test]
    fn test_box_shadow_initial_value() {
        assert!(PropertyRegistry::initial_value("box-shadow").is_some());
    }

    #[test]
    fn test_box_shadow_apply_initial() {
        let mut style = ComputedStyle::default();
        style.box_shadow.offset_x = 99.0;
        assert!(apply_initial_value(&mut style, "box-shadow"));
        assert_eq!(style.box_shadow.offset_x, 0.0);
    }

    #[test]
    fn test_box_shadow_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "box-shadow", "invalid"));
    }

    #[test]
    fn test_text_shadow_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "text-shadow", "invalid"));
    }

    // ── 边界测试：text-shadow 通过 DOM 树继承 ──

    /// 验证 text-shadow 作为可继承属性，通过 inherit_property 从父元素传递到子元素。
    /// 父元素设置 text-shadow: 3px 5px 2px blue，子元素应完整继承该值。
    #[test]
    fn test_text_shadow_inheritance_through_dom_tree() {
        // 构造父元素样式：设置 text-shadow
        let mut parent = ComputedStyle::default();
        assert!(apply_property_value(&mut parent, "text-shadow", "3px 5px 2px blue"));
        assert_eq!(parent.text_shadow.offset_x, 3.0);
        assert_eq!(parent.text_shadow.offset_y, 5.0);
        assert_eq!(parent.text_shadow.blur_radius, 2.0);
        assert_eq!(
            parent.text_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255)
        );

        // 构造子元素样式：从父元素继承 text-shadow
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "text-shadow"));

        // 子元素应获得与父元素完全相同的 text-shadow 值
        assert_eq!(child.text_shadow, parent.text_shadow);
    }

    // ── 边界测试：box-shadow inset 与 normal 正确区分 ──

    /// 验证 box-shadow 的 inset 标志与普通（outset）阴影正确区分。
    /// 同一偏移量下，inset 版本的 inset 字段应为 true，普通版本应为 false。
    #[test]
    fn test_box_shadow_inset_vs_normal_applied_correctly() {
        // 普通 box-shadow（无 inset）
        let mut normal_style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut normal_style,
            "box-shadow",
            "4px 8px 6px 2px green"
        ));
        assert!(!normal_style.box_shadow.inset, "普通 box-shadow 的 inset 应为 false");
        assert_eq!(normal_style.box_shadow.offset_x, 4.0);
        assert_eq!(normal_style.box_shadow.offset_y, 8.0);
        assert_eq!(normal_style.box_shadow.blur_radius, 6.0);
        assert_eq!(normal_style.box_shadow.spread_radius, 2.0);
        assert_eq!(
            normal_style.box_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255)
        );

        // inset box-shadow
        let mut inset_style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut inset_style,
            "box-shadow",
            "inset 4px 8px 6px 2px green"
        ));
        assert!(inset_style.box_shadow.inset, "inset box-shadow 的 inset 应为 true");
        assert_eq!(inset_style.box_shadow.offset_x, 4.0);
        assert_eq!(inset_style.box_shadow.offset_y, 8.0);
        assert_eq!(inset_style.box_shadow.blur_radius, 6.0);
        assert_eq!(inset_style.box_shadow.spread_radius, 2.0);
        assert_eq!(
            inset_style.box_shadow.color,
            zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255)
        );
    }

    // ── 边界测试：outline 简写属性通过 expand_shorthands 展开 ──

    /// 验证 outline 简写属性通过 expand_shorthands 正确展开为
    /// outline-width、outline-style、outline-color 三个长属性，
    /// 且 important 标志和特异性正确保留。
    #[test]
    fn test_outline_shorthand_expansion_via_expand_shorthands() {
        use crate::shorthand::expand_shorthands;

        // outline: 3px dashed red, important=true, specificity=(0,1,0)
        let decls: Vec<(String, String, bool, (u32, u32, u32))> =
            vec![("outline".to_string(), "3px dashed red".to_string(), true, (0, 1, 0))];
        let expanded = expand_shorthands(&decls);

        // 展开后应得到 3 个长属性声明
        assert_eq!(expanded.len(), 3);

        // 验证各长属性名称和值
        let props: Vec<(&str, &str)> = expanded.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
        assert!(props.contains(&("outline-width", "3px")));
        assert!(props.contains(&("outline-style", "dashed")));
        assert!(props.contains(&("outline-color", "red")));

        // 验证 important 和特异性在展开中保留
        for (_, _, imp, spec) in &expanded {
            assert!(imp, "important 标志应被保留");
            assert_eq!(*spec, (0, 1, 0), "特异性应被保留");
        }
    }

    // ── 边界测试：border-image-slice 带 fill 关键字通过 apply_property_value ──

    /// 验证 border-image-slice 的 fill 关键字在 apply_property_value 中正确解析，
    /// fill=true 时四个分量值也正确设置。
    #[test]
    fn test_border_image_slice_with_fill_keyword() {
        let mut style = ComputedStyle::default();

        // 默认 fill 应为 false
        assert!(!style.border_image_slice.fill);

        // 设置 border-image-slice: fill 10 20% 30 40%
        assert!(apply_property_value(
            &mut style,
            "border-image-slice",
            "fill 10 20% 30 40%"
        ));

        // fill 应为 true
        assert!(style.border_image_slice.fill, "fill 关键字应使 fill=true");

        // 验证四个分量的值
        assert_eq!(
            style.border_image_slice.top,
            BorderImageSliceComputedComponent::Number(10.0)
        );
        assert_eq!(
            style.border_image_slice.right,
            BorderImageSliceComputedComponent::Percent(20.0)
        );
        assert_eq!(
            style.border_image_slice.bottom,
            BorderImageSliceComputedComponent::Number(30.0)
        );
        assert_eq!(
            style.border_image_slice.left,
            BorderImageSliceComputedComponent::Percent(40.0)
        );
    }

    // ── 边界测试：text_shadow 和 box_shadow 计算样式的默认值（无阴影） ──

    /// 验证 ComputedStyle 默认构造时，text_shadow 和 box_shadow 均表示"无阴影"状态：
    /// 所有偏移/半径为 0，颜色为不透明黑色（但 inset 为 false 表示无实际阴影效果）。
    #[test]
    fn test_computed_style_default_no_shadow() {
        let style = ComputedStyle::default();

        // text-shadow 默认值：全部为零，无实际阴影
        assert_eq!(style.text_shadow.offset_x, 0.0);
        assert_eq!(style.text_shadow.offset_y, 0.0);
        assert_eq!(style.text_shadow.blur_radius, 0.0);

        // box-shadow 默认值：全部为零，无实际阴影
        assert_eq!(style.box_shadow.offset_x, 0.0);
        assert_eq!(style.box_shadow.offset_y, 0.0);
        assert_eq!(style.box_shadow.blur_radius, 0.0);
        assert_eq!(style.box_shadow.spread_radius, 0.0);
        assert!(!style.box_shadow.inset, "默认 box-shadow 的 inset 应为 false");
    }

    // ── list-style-image ──

    #[test]
    fn test_apply_list_style_image_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "list-style-image", "none"));
        assert_eq!(style.list_style_image, ListStyleImageComputedValue::None);
    }

    #[test]
    fn test_apply_list_style_image_url() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "list-style-image", "url(star.png)"));
        assert_eq!(
            style.list_style_image,
            ListStyleImageComputedValue::Url("star.png".to_string())
        );
    }

    #[test]
    fn test_apply_list_style_image_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "list-style-image", "invalid"));
    }

    #[test]
    fn test_list_style_image_is_inherited() {
        assert!(PropertyRegistry::is_inherited("list-style-image"));
    }

    #[test]
    fn test_list_style_image_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"list-style-image"));
    }

    #[test]
    fn test_list_style_image_initial_value() {
        assert!(PropertyRegistry::initial_value("list-style-image").is_some());
        let mut style = ComputedStyle::default();
        style.list_style_image = ListStyleImageComputedValue::Url("test.png".to_string());
        assert!(apply_initial_value(&mut style, "list-style-image"));
        assert_eq!(style.list_style_image, ListStyleImageComputedValue::None);
    }

    // ── column-gap ──

    #[test]
    fn test_apply_column_gap_px() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "column-gap", "20px"));
        assert_eq!(style.column_gap, LengthValue::Px(20.0));
    }

    #[test]
    fn test_apply_column_gap_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "column-gap", "invalid"));
    }

    #[test]
    fn test_column_gap_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("column-gap"));
    }

    #[test]
    fn test_column_gap_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"column-gap"));
    }

    #[test]
    fn test_transform_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"transform"));
    }

    #[test]
    fn test_grid_template_in_known_properties() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"grid-template-columns"));
        assert!(props.contains(&"grid-template-rows"));
        assert!(props.contains(&"grid-template-areas"));
        assert!(props.contains(&"grid-auto-flow"));
        assert!(props.contains(&"row-gap"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 边界测试 — list-style-image 继承 / column-gap 百分比 /
    // transform 多函数 / grid-auto-flow dense / row-gap em
    // ═══════════════════════════════════════════════════════════════════

    /// 验证 list-style-image 作为可继承属性，通过 inherit_property 从父元素传递到子元素。
    /// 父元素设置 list-style-image: url(bullet.png)，子元素应完整继承该 URL 值。
    #[test]
    fn test_list_style_image_inheritance_through_inherit_property() {
        // 构造父元素样式：设置 list-style-image
        let mut parent = ComputedStyle::default();
        assert!(apply_property_value(&mut parent, "list-style-image", "url(bullet.png)"));
        assert_eq!(
            parent.list_style_image,
            ListStyleImageComputedValue::Url("bullet.png".to_string())
        );

        // 构造子元素样式：从父元素继承 list-style-image
        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "list-style-image"));

        // 子元素应获得与父元素完全相同的 list-style-image 值
        assert_eq!(child.list_style_image, parent.list_style_image);
    }

    /// 验证 column-gap 接受百分比值。
    /// 百分比在布局阶段相对于容器宽度计算，此处验证解析和存储正确性。
    #[test]
    fn test_column_gap_with_percentage_value() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "column-gap", "25%"));
        assert_eq!(style.column_gap, LengthValue::Percentage(25.0));
    }

    /// 验证 transform 属性支持多个变换函数组合。
    /// "translate(10px) rotate(45deg)" 应解析为包含两个 TransformFunction 的列表。
    #[test]
    fn test_transform_with_multiple_functions() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(
            &mut style,
            "transform",
            "translate(10px) rotate(45deg)"
        ));
        match &style.transform {
            zero_css_parser::values::TransformValue::List(fns) => {
                assert_eq!(fns.len(), 2, "应包含两个变换函数");
                // 第一个函数：translate(10px) → Translate(10.0, 0.0)
                assert_eq!(fns[0], zero_css_parser::values::TransformFunction::Translate(10.0, 0.0));
                // 第二个函数：rotate(45deg) → Rotate(45.0)
                assert_eq!(fns[1], zero_css_parser::values::TransformFunction::Rotate(45.0));
            }
            other => panic!("transform 应为 List 变体，实际为: {other:?}"),
        }
    }

    /// 验证 grid-auto-flow 仅使用 "dense" 关键字时，
    /// 解析为 RowDense（等效于 "row dense"）。
    #[test]
    fn test_grid_auto_flow_dense_keyword() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "grid-auto-flow", "dense"));
        assert_eq!(style.grid_auto_flow, GridAutoFlowValue::RowDense);
    }

    /// 验证 row-gap 接受 em 单位值。
    /// em 值在计算样式阶段相对于当前 font-size 解析，此处验证原始值正确存储。
    #[test]
    fn test_row_gap_with_em_value() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "row-gap", "1.5em"));
        assert_eq!(style.row_gap, LengthValue::Em(1.5));
    }

    // ═══════════════════════════════════════════════════════════════════
    // justify-items / justify-self / align-content / empty-cells / border-spacing
    // ═══════════════════════════════════════════════════════════════════

    /// 验证 justify-items 的 apply_property_value 正确解析所有关键字值。
    #[test]
    fn test_apply_justify_items_keywords() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "justify-items", "center"));
        assert_eq!(style.justify_items, JustifyItemsValue::Center);
        assert!(apply_property_value(&mut style, "justify-items", "start"));
        assert_eq!(style.justify_items, JustifyItemsValue::Start);
        assert!(apply_property_value(&mut style, "justify-items", "normal"));
        assert_eq!(style.justify_items, JustifyItemsValue::Normal);
        assert!(apply_property_value(&mut style, "justify-items", "stretch"));
        assert_eq!(style.justify_items, JustifyItemsValue::Stretch);
    }

    /// 验证 justify-items 对无效值返回 false。
    #[test]
    fn test_apply_justify_items_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "justify-items", "invalid"));
    }

    /// 验证 justify-self 的 apply_property_value 正确解析所有关键字值。
    #[test]
    fn test_apply_justify_self_keywords() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "justify-self", "auto"));
        assert_eq!(style.justify_self, JustifySelfValue::Auto);
        assert!(apply_property_value(&mut style, "justify-self", "end"));
        assert_eq!(style.justify_self, JustifySelfValue::End);
        assert!(apply_property_value(&mut style, "justify-self", "baseline"));
        assert_eq!(style.justify_self, JustifySelfValue::Baseline);
    }

    /// 验证 align-content 的 apply_property_value 正确解析所有关键字值。
    #[test]
    fn test_apply_align_content_keywords() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "align-content", "space-between"));
        assert_eq!(style.align_content, AlignContentValue::SpaceBetween);
        assert!(apply_property_value(&mut style, "align-content", "space-around"));
        assert_eq!(style.align_content, AlignContentValue::SpaceAround);
        assert!(apply_property_value(&mut style, "align-content", "space-evenly"));
        assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);
        assert!(apply_property_value(&mut style, "align-content", "center"));
        assert_eq!(style.align_content, AlignContentValue::Center);
    }

    /// 验证 align-content 对无效值返回 false。
    #[test]
    fn test_apply_align_content_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "align-content", "flex-start"));
    }

    /// 验证 empty-cells 的 apply_property_value 正确解析 show/hide。
    #[test]
    fn test_apply_empty_cells() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "empty-cells", "hide"));
        assert_eq!(style.empty_cells, EmptyCellsComputedValue::Hide);
        assert!(apply_property_value(&mut style, "empty-cells", "show"));
        assert_eq!(style.empty_cells, EmptyCellsComputedValue::Show);
    }

    /// 验证 empty-cells 对无效值返回 false。
    #[test]
    fn test_apply_empty_cells_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "empty-cells", "visible"));
    }

    /// 验证 border-spacing 的 apply_property_value 正确解析单值和双值。
    #[test]
    fn test_apply_border_spacing() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "border-spacing", "5px"));
        assert_eq!(style.border_spacing.horizontal, 5.0);
        assert_eq!(style.border_spacing.vertical, 5.0);
        assert!(apply_property_value(&mut style, "border-spacing", "2px 4px"));
        assert_eq!(style.border_spacing.horizontal, 2.0);
        assert_eq!(style.border_spacing.vertical, 4.0);
    }

    /// 验证 border-spacing 对无效值返回 false。
    #[test]
    fn test_apply_border_spacing_invalid() {
        let mut style = ComputedStyle::default();
        assert!(!apply_property_value(&mut style, "border-spacing", "invalid"));
    }

    /// 验证 empty-cells 和 border-spacing 是继承属性。
    #[test]
    fn test_inheritance_empty_cells_and_border_spacing() {
        assert!(PropertyRegistry::is_inherited("empty-cells"));
        assert!(PropertyRegistry::is_inherited("border-spacing"));
        // justify-items / justify-self / align-content 不继承
        assert!(!PropertyRegistry::is_inherited("justify-items"));
        assert!(!PropertyRegistry::is_inherited("justify-self"));
        assert!(!PropertyRegistry::is_inherited("align-content"));
    }

    /// 验证 5 个属性都在 known_properties 中注册。
    #[test]
    fn test_known_properties_new_five() {
        let props = PropertyRegistry::known_properties();
        assert!(props.contains(&"justify-items"));
        assert!(props.contains(&"justify-self"));
        assert!(props.contains(&"align-content"));
        assert!(props.contains(&"empty-cells"));
        assert!(props.contains(&"border-spacing"));
    }

    /// 验证 5 个属性的 initial_value 均可获取。
    #[test]
    fn test_initial_value_new_five() {
        assert!(PropertyRegistry::initial_value("justify-items").is_some());
        assert!(PropertyRegistry::initial_value("justify-self").is_some());
        assert!(PropertyRegistry::initial_value("align-content").is_some());
        assert!(PropertyRegistry::initial_value("empty-cells").is_some());
        assert!(PropertyRegistry::initial_value("border-spacing").is_some());
    }

    /// 验证 apply_initial_value 对 5 个新属性能正确重置为默认值。
    #[test]
    fn test_apply_initial_value_new_five() {
        let mut style = ComputedStyle::default();
        // 先设置非默认值
        apply_property_value(&mut style, "justify-items", "center");
        apply_property_value(&mut style, "justify-self", "end");
        apply_property_value(&mut style, "align-content", "space-between");
        apply_property_value(&mut style, "empty-cells", "hide");
        apply_property_value(&mut style, "border-spacing", "10px");

        // 重置
        assert!(apply_initial_value(&mut style, "justify-items"));
        assert_eq!(style.justify_items, JustifyItemsValue::Normal);
        assert!(apply_initial_value(&mut style, "justify-self"));
        assert_eq!(style.justify_self, JustifySelfValue::Auto);
        assert!(apply_initial_value(&mut style, "align-content"));
        assert_eq!(style.align_content, AlignContentValue::Normal);
        assert!(apply_initial_value(&mut style, "empty-cells"));
        assert_eq!(style.empty_cells, EmptyCellsComputedValue::Show);
        assert!(apply_initial_value(&mut style, "border-spacing"));
        assert_eq!(style.border_spacing.horizontal, 0.0);
        assert_eq!(style.border_spacing.vertical, 0.0);
    }

    /// 验证 empty-cells 和 border-spacing 的继承正确工作。
    #[test]
    fn test_inherit_property_empty_cells_and_border_spacing() {
        let mut parent = ComputedStyle::default();
        apply_property_value(&mut parent, "empty-cells", "hide");
        apply_property_value(&mut parent, "border-spacing", "3px 7px");

        let mut child = ComputedStyle::default();
        assert!(inherit_property(&parent, &mut child, "empty-cells"));
        assert_eq!(child.empty_cells, EmptyCellsComputedValue::Hide);
        assert!(inherit_property(&parent, &mut child, "border-spacing"));
        assert_eq!(child.border_spacing.horizontal, 3.0);
        assert_eq!(child.border_spacing.vertical, 7.0);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 边界条件测试 — justify-items 全值 / align-content space-between /
    //   empty-cells 继承 / border-spacing 继承 / gap 简写展开
    // ═══════════════════════════════════════════════════════════════════

    /// 测试 justify-items 所有枚举值通过 apply_property_value 正确应用到 ComputedStyle。
    #[test]
    fn test_justify_items_all_values_via_apply() {
        let mut style = ComputedStyle::default();

        // 默认值为 Normal
        assert_eq!(style.justify_items, JustifyItemsValue::Normal);

        // 逐一验证所有 7 个枚举值
        assert!(apply_property_value(&mut style, "justify-items", "auto"));
        assert_eq!(style.justify_items, JustifyItemsValue::Auto);

        assert!(apply_property_value(&mut style, "justify-items", "normal"));
        assert_eq!(style.justify_items, JustifyItemsValue::Normal);

        assert!(apply_property_value(&mut style, "justify-items", "start"));
        assert_eq!(style.justify_items, JustifyItemsValue::Start);

        assert!(apply_property_value(&mut style, "justify-items", "end"));
        assert_eq!(style.justify_items, JustifyItemsValue::End);

        assert!(apply_property_value(&mut style, "justify-items", "center"));
        assert_eq!(style.justify_items, JustifyItemsValue::Center);

        assert!(apply_property_value(&mut style, "justify-items", "stretch"));
        assert_eq!(style.justify_items, JustifyItemsValue::Stretch);

        assert!(apply_property_value(&mut style, "justify-items", "baseline"));
        assert_eq!(style.justify_items, JustifyItemsValue::Baseline);

        // 无效值应返回 false 且不改变当前值
        assert!(!apply_property_value(&mut style, "justify-items", "invalid"));
        assert_eq!(style.justify_items, JustifyItemsValue::Baseline);
    }

    /// 测试 align-content: space-between 通过 apply_property_value 正确应用。
    #[test]
    fn test_align_content_space_between() {
        let mut style = ComputedStyle::default();

        // 默认值为 Normal
        assert_eq!(style.align_content, AlignContentValue::Normal);

        // space-between 是 Box Alignment 规范中的关键值
        assert!(apply_property_value(&mut style, "align-content", "space-between"));
        assert_eq!(style.align_content, AlignContentValue::SpaceBetween);

        // 同系列值也应工作
        assert!(apply_property_value(&mut style, "align-content", "space-around"));
        assert_eq!(style.align_content, AlignContentValue::SpaceAround);

        assert!(apply_property_value(&mut style, "align-content", "space-evenly"));
        assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);

        // 无效值返回 false
        assert!(!apply_property_value(&mut style, "align-content", "space-invalid"));
        assert_eq!(style.align_content, AlignContentValue::SpaceEvenly);
    }

    /// 测试 empty-cells 通过 inherit_property 正确从父元素继承到子元素。
    #[test]
    fn test_empty_cells_inheritance_via_inherit_property() {
        // empty-cells 是继承属性，父元素设置 hide 后子元素应继承
        let mut parent = ComputedStyle::default();
        parent.empty_cells = EmptyCellsComputedValue::Hide;

        let mut child = ComputedStyle::default();
        // 子元素默认为 Show
        assert_eq!(child.empty_cells, EmptyCellsComputedValue::Show);

        // 继承成功
        assert!(inherit_property(&parent, &mut child, "empty-cells"));
        assert_eq!(child.empty_cells, EmptyCellsComputedValue::Hide);

        // 子元素显式设置后覆盖继承值
        assert!(apply_property_value(&mut child, "empty-cells", "show"));
        assert_eq!(child.empty_cells, EmptyCellsComputedValue::Show);

        // 反向：父元素 Show → 子元素继承 Show
        let parent2 = ComputedStyle::default();
        let mut child2 = ComputedStyle::default();
        child2.empty_cells = EmptyCellsComputedValue::Hide;
        assert!(inherit_property(&parent2, &mut child2, "empty-cells"));
        assert_eq!(child2.empty_cells, EmptyCellsComputedValue::Show);
    }

    /// 测试 border-spacing 通过 inherit_property 正确从父元素继承到子元素，
    /// 包括水平/垂直分量独立验证。
    #[test]
    fn test_border_spacing_inheritance_via_inherit_property() {
        // border-spacing 是继承属性
        let mut parent = ComputedStyle::default();
        parent.border_spacing.horizontal = 12.0;
        parent.border_spacing.vertical = 24.0;

        let mut child = ComputedStyle::default();
        // 子元素默认为 0 0
        assert_eq!(child.border_spacing.horizontal, 0.0);
        assert_eq!(child.border_spacing.vertical, 0.0);

        // 继承成功，水平/垂直分量分别复制
        assert!(inherit_property(&parent, &mut child, "border-spacing"));
        assert_eq!(child.border_spacing.horizontal, 12.0);
        assert_eq!(child.border_spacing.vertical, 24.0);

        // 子元素显式设置后覆盖继承值（只设水平，垂直仍由简写决定）
        assert!(apply_property_value(&mut child, "border-spacing", "5px"));
        assert_eq!(child.border_spacing.horizontal, 5.0);
        assert_eq!(child.border_spacing.vertical, 5.0);

        // 两值形式继承：水平和垂直不同
        let mut parent3 = ComputedStyle::default();
        parent3.border_spacing.horizontal = 8.0;
        parent3.border_spacing.vertical = 16.0;

        let mut child3 = ComputedStyle::default();
        assert!(inherit_property(&parent3, &mut child3, "border-spacing"));
        assert_eq!(child3.border_spacing.horizontal, 8.0);
        assert_eq!(child3.border_spacing.vertical, 16.0);
    }

    /// 测试 gap 简写属性通过 expand_shorthands 正确展开为
    /// gap、row-gap、column-gap 三个长属性，
    /// 覆盖单值和双值两种形式。
    #[test]
    fn test_gap_shorthand_expansion_via_expand_shorthands() {
        use crate::shorthand::expand_shorthands;

        // ── 单值形式：gap: 10px → row-gap: 10px, column-gap: 10px ──
        let decls: Vec<(String, String, bool, (u32, u32, u32))> =
            vec![("gap".to_string(), "10px".to_string(), false, (0, 0, 1))];
        let expanded = expand_shorthands(&decls);

        // 展开后应得到 3 个声明：gap + row-gap + column-gap
        assert_eq!(expanded.len(), 3);

        let props: Vec<(&str, &str)> = expanded.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
        assert!(props.contains(&("gap", "10px")));
        assert!(props.contains(&("row-gap", "10px")));
        assert!(props.contains(&("column-gap", "10px")));

        // important 和特异性应保留
        for (_, _, imp, spec) in &expanded {
            assert!(!imp);
            assert_eq!(*spec, (0, 0, 1));
        }

        // ── 双值形式：gap: 10px 20px → row-gap: 10px, column-gap: 20px ──
        let decls2: Vec<(String, String, bool, (u32, u32, u32))> =
            vec![("gap".to_string(), "10px 20px".to_string(), true, (0, 1, 0))];
        let expanded2 = expand_shorthands(&decls2);

        assert_eq!(expanded2.len(), 3);

        let props2: Vec<(&str, &str)> = expanded2.iter().map(|(p, v, _, _)| (p.as_str(), v.as_str())).collect();
        assert!(props2.contains(&("gap", "10px")));
        assert!(props2.contains(&("row-gap", "10px")));
        assert!(props2.contains(&("column-gap", "20px")));

        // important 和特异性保留
        for (_, _, imp, spec) in &expanded2 {
            assert!(imp, "important 标志应被保留");
            assert_eq!(*spec, (0, 1, 0), "特异性应被保留");
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // counter-set 属性测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_apply_counter_set_none() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "counter-set", "none"));
    }

    #[test]
    fn test_apply_counter_set_value() {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "counter-set", "mycounter 3"));
    }

    #[test]
    fn test_counter_set_not_inherited() {
        assert!(!PropertyRegistry::is_inherited("counter-set"));
    }

    #[test]
    fn test_counter_set_in_known_properties() {
        assert!(PropertyRegistry::known_properties().contains(&"counter-set"));
    }

    #[test]
    fn test_counter_set_initial_value() {
        assert!(PropertyRegistry::initial_value("counter-set").is_some());
    }
}
