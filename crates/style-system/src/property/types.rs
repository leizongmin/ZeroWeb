//! CSS 属性定义和计算样式结构。
//!
//! 定义 `ComputedStyle` 结构体，包含所有 Tier 1 CSS 属性的 typed 字段，
//! 以及 `PropertyRegistry` 用于查询初始值和继承性。

pub use zero_css_parser::values::{
    self, AlignmentValue, BoxSizingValue, ClipPathRadius, ColorValue, ColumnCountValue, ColumnWidthValue, ContainValue,
    ContainerTypeValue, ContentValue, CounterActionValue, DisplayValue, FilterValue, FlexDirectionValue, FlexWrapValue,
    FontStyleValue, FontWeightValue, LengthValue, ObjectFitValue, OverflowValue, PolygonFillRule, PositionValue,
    QuotesValue, ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStopValue, ScrollSnapTypeValue,
    TextEmphasisPositionValue, TextEmphasisStyleValue, VerticalAlignValue, VisibilityValue,
};

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

/// CSS text-decoration-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecorationStyleValue {
    /// solid（默认）。
    Solid,
    /// double。
    Double,
    /// dotted。
    Dotted,
    /// dashed。
    Dashed,
    /// wavy。
    Wavy,
}

/// CSS text-transform 值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TextTransformValue {
    /// 把 `text` 按本 transform 值转换。
    ///
    /// 与 CSS Text 3 §3.1 一致：`None` 原样；`Uppercase`/`Lowercase` 走 Rust
    /// `char` 大小写折叠（覆盖 Latin/扩展 Latin 等基本多文种面）；`Capitalize`
    /// 把每个「词」首字母（前一字符为非字母数字边界后的首个字母）转 titlecase，
    /// 其余字符原样保留。
    ///
    /// 放在 style-system（非 engine/paint/helpers.rs）以便 layout-engine 在
    /// `collect_inline_items` 期也能调用——text-transform 须在**行断前**应用，
    /// 使 layout 用转换后文本宽度行断（R1012 Phase A IFC 统一首切）。
    pub fn apply(&self, text: &str) -> String {
        match self {
            TextTransformValue::None => text.to_string(),
            TextTransformValue::Uppercase => text.to_uppercase(),
            TextTransformValue::Lowercase => text.to_lowercase(),
            TextTransformValue::Capitalize => {
                let mut result = String::with_capacity(text.len());
                let mut prev_is_boundary = true;
                for ch in text.chars() {
                    if prev_is_boundary && ch.is_alphabetic() {
                        for c in ch.to_uppercase() {
                            result.push(c);
                        }
                    } else {
                        result.push(ch);
                    }
                    prev_is_boundary = !ch.is_alphanumeric();
                }
                result
            }
        }
    }
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

/// CSS line-break 值（CSS Text 3 §5.3）。
///
/// `Anywhere` 在每个排版字符处创建换行机会（覆盖 GL/JW/ZJW 禁则），
/// 近似 `word-break: break-all` 但更强。其余值（strict/loose/normal/auto）
/// 涉及 CJK 标点/小假名等规则，当前解析但按默认（normal）行为处理。
#[derive(Debug, Clone, PartialEq)]
pub enum LineBreakValue {
    /// auto（依 locale，等价 normal）。
    Auto,
    /// loose。
    Loose,
    /// normal（默认）。
    Normal,
    /// strict。
    Strict,
    /// anywhere。
    Anywhere,
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
    /// 渐变函数 — linear-gradient / radial-gradient / conic-gradient。
    Gradient(zero_css_parser::values::GradientValue),
}

/// CSS mask-mode 计算值。
#[derive(Debug, Clone, PartialEq)]
pub enum MaskModeComputedValue {
    /// alpha — 使用 mask 图像的 alpha 通道。
    Alpha,
    /// luminance — 使用 mask 图像的亮度值。
    Luminance,
    /// match-source — 默认值。
    MatchSource,
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
    /// text-decoration-color 值。
    TextDecorationColor(ColorValue),
    /// text-decoration-style 值。
    TextDecorationStyle(TextDecorationStyleValue),
    /// text-emphasis-style 值（CSS Text Decoration 3 §3.1）。
    TextEmphasisStyle(TextEmphasisStyleValue),
    /// text-emphasis-position 值（§3.2）。
    TextEmphasisPosition(TextEmphasisPositionValue),
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
    /// column-fill 值。
    ColumnFill(ColumnFillComputedValue),
    /// column-span 值。
    ColumnSpan(ColumnSpanComputedValue),
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
    /// clip-path 值。
    ClipPath(ClipPathComputedValue),
    /// clip 属性值（已弃用的 CSS2 裁剪属性）。
    Clip(ClipRectComputedValue),
    /// mask-image 值。
    MaskImage(Vec<BackgroundImageComputedValue>),
    /// mask-mode 值。
    MaskMode(MaskModeComputedValue),
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

/// CSS column-fill 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnFillComputedValue {
    /// balance — 均衡分配内容到各列（默认值）。
    Balance,
    /// auto — 按顺序填充列。
    Auto,
}

/// CSS column-span 属性值。
///
/// §6.1：`column-span: all` 使元素脱离列流，跨越 multicol 容器全宽
/// （成为「spanner」），将其上下内容分成独立平衡的列区域。默认 `none`
/// （留在列流中）。ZW 仅支持 `all`/`none`（`column-span` 在 css-multicol-1
/// 只有这两值）。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnSpanComputedValue {
    /// none — 元素留在正常列流中（默认）。
    None,
    /// all — 元素跨越所有列（spanner）。
    All,
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

/// CSS clip-path 属性计算值。
///
/// 直接复用 css-parser 的 ClipPathValue 类型，
/// 因为计算阶段不需要额外转换。
pub type ClipPathComputedValue = zero_css_parser::values::ClipPathValue;

/// CSS clip 属性计算值。
///
/// 直接复用 css-parser 的 ClipRectValue 类型。
pub type ClipRectComputedValue = zero_css_parser::values::ClipRectValue;

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

// ComputedStyle 结构体已拆分到 computed_style.rs
pub use super::computed_style::ComputedStyle;
