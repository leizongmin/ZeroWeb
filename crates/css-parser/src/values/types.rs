//! CSS 属性值类型定义。
//!
//! 定义 CSS 属性值的类型化表示：长度、颜色、显示、布局等枚举类型，
//! 以及 calc() 表达式的 AST（CalcExpr、CalcContext、CalcParser）。

/// CSS 长度值。
#[derive(Debug, Clone, PartialEq)]
pub enum LengthValue {
    /// 绝对长度（px）。
    Px(f64),
    /// em 单位。
    Em(f64),
    /// ex 单位（当前字体的 x-height）。
    Ex(f64),
    // https://drafts.csswg.org/css-values-4/#font-relative-lengths
    /// rex 单位（根元素字体的 x-height）。
    Rex(f64),
    /// cap 单位（当前字体的大写字母高度）。
    Cap(f64),
    /// rcap 单位（根元素字体的大写字母高度）。
    Rcap(f64),
    /// rem 单位。
    Rem(f64),
    /// vh 单位。
    Vh(f64),
    /// vw 单位。
    Vw(f64),
    /// vmin 单位。
    Vmin(f64),
    /// vmax 单位。
    Vmax(f64),
    /// ch 单位。
    Ch(f64),
    /// rch 单位（根元素字体中 U+0030 "0" 字形的 advance）。
    Rch(f64),
    /// ic 单位（当前字体中 U+6C34 "水" 字形的 advance）。
    Ic(f64),
    /// ric 单位（根元素字体中 U+6C34 "水" 字形的 advance）。
    Ric(f64),
    /// 百分比值（0-100）。
    Percentage(f64),
    /// auto 关键字。
    Auto,
    /// min-content 关键字 — 最小内容宽度。
    MinContent,
    /// max-content 关键字 — 最大内容宽度。
    MaxContent,
    /// 数学表达式（calc/min/max/clamp），在样式解析阶段无法直接求值，
    /// 需要在 [`resolve_computed_style`](crate::resolve_computed_style) 阶段用完整上下文求值。
    Calc(Box<CalcExpr>),
    /// fit-content() 函数，将尺寸限制为内容最大宽度不超过给定值。
    /// 参数可以是长度或百分比。
    FitContent(Box<LengthValue>),
}

/// CSS 颜色值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorValue {
    /// RGB 颜色。
    Rgba(u8, u8, u8, u8),
    /// HSL 颜色。
    Hsla(f64, f64, f64, f64),
    /// 命名颜色。
    Named(String),
    /// transparent。
    Transparent,
    /// currentColor。
    CurrentColor,
    /// CSS Color 5 `color-mix()` —— **未解析**（currentColor 在 paint 时按元素色解析，
    /// 支持 inherit 透传：`background-color: inherit` 把 Mix 原样传给子元素，currentColor
    /// 在子元素按其自身 color 重解析）。仅 `in srgb` 色彩空间（gamma-encoded 线性插值）。
    Mix(Box<ColorMixSpec>),
    /// CSS Color 5 相对色（RCS）非 identity —— **未解析**（currentColor origin 在 paint 时
    /// 按元素色解析，支持 inherit 透传，同 Mix）。rgb/rgba/hsl/hsla 与 lab/lch/oklab/oklch 输出空间，
    /// 通道为关键字引用或数字字面量（无 calc）。identity（channels 恰为自然关键字）在 parse 阶段
    /// 已短路为 origin。driving: css-color relative-currentcolor-rgb-02（g r b swap）/ hsl-02（120 s l 覆盖）。
    RelativeColor(Box<RelativeColorSpec>),
}

/// `color-mix()` 规范（CSS Color 4）。
#[derive(Debug, Clone, PartialEq)]
pub struct ColorMixSpec {
    /// 第一个颜色分量。
    pub c1: ColorMixComponent,
    /// 第二个颜色分量。
    pub c2: ColorMixComponent,
    /// 插值色彩空间（sRGB gamma-encoded 线性 / LCH 极坐标 + 色相短弧）。
    pub space: ColorMixSpace,
    /// 极坐标空间（LCH/OKLCH）的色相插值法（CSS Color 4 §12.3），默认 Shorter。R2381。
    pub hue: crate::values::parse_transform::ColorHueMethod,
}

/// `color-mix()` 插值色彩空间。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMixSpace {
    /// `in srgb` —— gamma-encoded sRGB 线性插值（premultiplied alpha）。
    Srgb,
    /// `in srgb-linear` —— 线性光 sRGB 笛卡尔插值（L/a/b→R/G/B 三通道独立线性）。R2377。
    SrgbLinear,
    /// `in lch` —— CIE LCH 极坐标插值（L/C 线性、h 色相短弧）。driving: color-mix-percents-01/02。
    Lch,
    /// `in lab` —— CIE Lab 笛卡尔插值（L/a/b 三通道独立线性）。R2376。
    Lab,
    /// `in oklab` —— OKLab 笛卡尔插值（L/a/b 三通道独立线性）。R2376。
    OkLab,
    /// `in oklch` —— OKLCH 极坐标插值（L/C 线性、h 色相短弧）。R2376。
    OkLch,
    /// `in xyz`/`in xyz-d65` —— CIE XYZ-D65 笛卡尔插值（X/Y/Z 三通道独立线性）。R2378。
    Xyz,
}

/// `color-mix()` 的单个分量（颜色 + 可选百分比）。
#[derive(Debug, Clone, PartialEq)]
pub struct ColorMixComponent {
    /// 颜色（可为 currentColor 等，运行时解析）。
    pub color: ColorValue,
    /// 百分比 [0, 100]，None 表示省略（按 spec 默认：双省略=50/50，单省略=100-另一）。
    pub percentage: Option<f64>,
}

/// RCS（CSS Color 5 相对色）非 identity 规范：`<func>(from <origin> <ch1> <ch2> <ch3> [/ <alpha>])`。
///
/// currentColor origin 保留未解析（paint 时按元素色解析，支持 inherit 透传）。rgb/rgba/hsl/hsla +
/// lab/lch/oklab/oklch/color() 输出空间。driving: css-color relative-currentcolor-rgb-02 / hsl-02。
#[derive(Debug, Clone, PartialEq)]
pub struct RelativeColorSpec {
    /// 输出函数（决定通道语义与单位）。
    pub func: RelativeColorFunc,
    /// origin 颜色（可为 currentColor，运行时解析）。
    pub origin: ColorValue,
    /// 3 个输出通道（rgb: r/g/b；hsl: h/s/l；lab/oklab: l/a/b；lch/oklch: l/c/h；color: r/g/b 或 x/y/z）。
    pub channels: [RcsChannel; 3],
    /// alpha（省略 = 用 origin alpha）。
    pub alpha: RcsAlpha,
    /// color() 的预定义色彩空间名（仅 func==Color 时 Some，如 "display-p3"/"xyz-d50"）；其余函数 None。
    pub space: Option<String>,
}

/// RCS 输出函数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeColorFunc {
    /// rgb()/rgba() —— 通道为 0-255。
    Rgb,
    /// hsl()/hsla() —— h 为度 [0,360)，s/l 为 [0,100]。
    Hsl,
    /// lab() —— L∈[0,100]，a/b 为 a*/b*（常见 ±125）。
    Lab,
    /// lch() —— L∈[0,100]，C（常见 0-150），h 为度。
    Lch,
    /// oklab() —— L∈[0,1]，a/b（常见 ±0.4）。
    Oklab,
    /// oklch() —— L∈[0,1]，C（常见 0-0.4），h 为度。
    Oklch,
    /// color() —— 预定义色彩空间（space 字段），通道为 0-1（rect 空间 r/g/b；xyz 空间 x/y/z）。
    Color,
}

/// RCS 单个输出通道。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RcsChannel {
    /// 引用 origin 的某通道（按函数通道序：rgb 0=r/1=g/2=b；hsl 0=h/1=s/2=l）。
    /// 支持置换（如 `rgb(from X g r b)` 的首通道引用 origin green=index 1）。
    Ref(u8),
    /// 数字字面量覆盖（rgb: 0-255；hsl h: 度；s/l: 0-100）。
    Num(f64),
    /// `none`（缺失分量 → 0）。
    None,
}

/// RCS alpha 分量。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RcsAlpha {
    /// 用 origin alpha（省略时的默认；currentColor origin → 元素色 alpha）。
    Origin,
    /// 数字字面量（0-1 或 0-100%，paint 时归一到 0-255）。
    Num(f64),
    /// `none` → alpha 0。
    None,
}

/// CSS display 值。
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayValue {
    /// block。
    Block,
    /// inline。
    Inline,
    /// inline-block。
    InlineBlock,
    /// flex。
    Flex,
    /// inline-flex。
    InlineFlex,
    /// grid。
    Grid,
    /// inline-grid。
    InlineGrid,
    /// none。
    None,
    /// contents。
    Contents,
    /// flow。
    Flow,
    /// flow-root。
    FlowRoot,
    /// list-item。
    ListItem,
    /// table。
    Table,
    /// inline-table。
    InlineTable,
    /// table-row。
    TableRow,
    /// table-cell。
    TableCell,
    /// table-caption。
    TableCaption,
    /// table-column。
    TableColumn,
    /// table-column-group。
    TableColumnGroup,
    /// table-row-group。
    TableRowGroup,
    /// table-header-group。
    TableHeaderGroup,
    /// table-footer-group。
    TableFooterGroup,
}

/// CSS float 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FloatValue {
    /// none（默认值）。
    None,
    /// left。
    Left,
    /// right。
    Right,
    /// inline-start。
    InlineStart,
    /// inline-end。
    InlineEnd,
}

/// CSS clear 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ClearValue {
    /// none（默认值）。
    None,
    /// left。
    Left,
    /// right。
    Right,
    /// both。
    Both,
    /// inline-start。
    InlineStart,
    /// inline-end。
    InlineEnd,
}

/// CSS position 值。
#[derive(Debug, Clone, PartialEq)]
pub enum PositionValue {
    /// static。
    Static,
    /// relative。
    Relative,
    /// absolute。
    Absolute,
    /// fixed。
    Fixed,
    /// sticky。
    Sticky,
}

/// CSS clip 属性值（已弃用但仍广泛使用）。
///
/// `clip: auto | rect(top, right, bottom, left)`
/// 仅对绝对定位元素生效。坐标相对于元素的边框盒。
#[derive(Debug, Clone, PartialEq)]
pub enum ClipRectValue {
    /// auto — 不裁剪。
    Auto,
    /// rect(top, right, bottom, left) — 矩形裁剪区域。
    Rect(LengthValue, LengthValue, LengthValue, LengthValue),
}

/// CSS overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
    /// scroll。
    Scroll,
    /// auto。
    Auto,
    /// clip。
    Clip,
}

/// CSS overflow-clip-margin 的视觉盒基准（CSS Overflow 3 §3）。
///
/// 决定 `overflow:clip` 元素的裁剪边从哪个盒起算，再用 [`OverflowClipMarginValue::length`]
/// 向外（正值）/向内（负值）扩展。仅对 `overflow:clip` 生效——`hidden`/`scroll`/`auto`
/// 始终按 padding-box 裁剪，忽略 overflow-clip-margin。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowClipMarginBox {
    /// content-box — 内容盒（仅 content，不含 padding/border）。
    ContentBox,
    /// padding-box — 内边距盒（content + padding）。**规范初值**（与 ZW 既有 overflow
    /// 裁剪到 padding-box 一致，故 length=0 + PaddingBox 时零行为变更）。
    PaddingBox,
    /// border-box — 边框盒（content + padding + border）。
    BorderBox,
}

/// CSS overflow-clip-margin 值（CSS Overflow 3 §3）。
///
/// 文法 `<visual-box> || <length>`：视觉盒基准 + 扩展长度。两者皆可缺省
///（缺省 box = PaddingBox，缺省 length = 0）。length 保留为 [`LengthValue`] 以支持
/// em/% 在 compute 期按 font-size 解析。
#[derive(Debug, Clone, PartialEq)]
pub struct OverflowClipMarginValue {
    /// 视觉盒基准。
    pub box_kind: OverflowClipMarginBox,
    /// 扩展长度（解析期保留单位，compute 期 resolve 为 px）。
    pub length: LengthValue,
}

/// CSS list-style-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStyleTypeValue {
    /// disc（默认值）。
    Disc,
    /// circle。
    Circle,
    /// square。
    Square,
    /// decimal。
    Decimal,
    /// decimal-leading-zero。
    DecimalLeadingZero,
    /// lower-roman。
    LowerRoman,
    /// upper-roman。
    UpperRoman,
    /// lower-alpha / lower-latin。
    LowerAlpha,
    /// upper-alpha / upper-latin。
    UpperAlpha,
    /// lower-greek（α-ω，CSS Counter Styles 3 §6 预定义）。R2445。
    LowerGreek,
    /// persian（۰-۹ 数字，CSS Counter Styles 3 §6 预定义）。R2445。
    Persian,
    /// armenian（传统亚美尼亚数字，additive；CSS Counter Styles 3 §6.1 预定义，≡ upper-armenian）。R2447。
    Armenian,
    /// lower-armenian（小写亚美尼亚数字，同 armenian 算法 + to_lowercase；CSS Counter Styles 3 §6.1）。R2448。
    LowerArmenian,
    /// georgian（传统格鲁吉亚数字，additive；CSS Counter Styles 3 §6.1 预定义，range 1-19999）。R2449。
    Georgian,
    /// hebrew（传统希伯来数字，additive；CSS Counter Styles 3 §6.1 预定义，range 1-10999）。R2450。
    Hebrew,
    /// arabic-indic（阿拉伯-印度数字 ٠-٩，numeric；CSS Counter Styles 3 §6.1 预定义）。R2451。
    ArabicIndic,
    /// R2471：CSS Counter Styles 3 §6.1 预定义 numeric system（十进制位数字替换，同 arabic-indic
    /// 算法，仅 digit 字形块不同）。
    Devanagari,
    /// bengali 数字 ০-৯（U+09E6+，numeric）。R2471。
    Bengali,
    /// gujarati 数字 ૦-૯（U+0AE6+，numeric）。R2471。
    Gujarati,
    /// gurmukhi 数字 ੦-੯（U+0A66+，numeric）。R2471。
    Gurmukhi,
    /// kannada 数字 ೦-೯（U+0CE6+，numeric）。R2471。
    Kannada,
    /// malayalam 数字 ൦-൯（U+0D66+，numeric）。R2471。
    Malayalam,
    /// tamil 数字 ௦-௯（U+0BE6+，numeric）。R2471。
    Tamil,
    /// telugu 数字 ౦-౯（U+0C66+，numeric）。R2471。
    Telugu,
    /// lao 数字 ໐-໙（U+0ED0+，numeric）。R2471。
    Lao,
    /// khmer 数字 ០-៩（U+17E0+，numeric）。R2471。
    Khmer,
    /// myanmar 数字 ၀-၉（U+1040+，numeric）。R2471。
    Myanmar,
    /// cjk-decimal（CJK ideographic digits 〇一二...九，非连续 → lookup table；CSS Counter
    /// Styles 3 §6.1 预定义 numeric system，digit 0=U+3007 / 1-9=U+4E00,U+4E8C...）。R2472。
    CjkDecimal,
    /// none。
    None,
    /// 自定义计数器样式名（`@counter-style` 定义；CSS Counter Styles 3）。driving: R2392。
    /// 非 builtin 的 `<custom-ident>` → Custom(name)；渲染时查 CounterStyleRegistry，
    /// 未命中走 fallback（默认 decimal）。
    Custom(String),
    /// `<string>` 标记类型（CSS Lists 3）：引号字符串作为固定标记文本（每个 li 同值，
    /// 非计数器）。如 `list-style-type: "▶"`。driving: R2574。
    String(String),
}

/// CSS list-style-position 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStylePositionValue {
    /// outside（默认值）。
    Outside,
    /// inside。
    Inside,
}

/// CSS flex-direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FlexDirectionValue {
    /// row。
    Row,
    /// row-reverse。
    RowReverse,
    /// column。
    Column,
    /// column-reverse。
    ColumnReverse,
}

/// CSS flex-wrap 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FlexWrapValue {
    /// nowrap。
    Nowrap,
    /// wrap。
    Wrap,
    /// wrap-reverse。
    WrapReverse,
}

/// CSS justify-content / align-items 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentValue {
    /// auto（align-self 初始值，继承容器 align-items）。
    Auto,
    /// normal（CSS Box Align 3：justify-content/align-items/align-self 初始值；
    /// converter 中 align-items/self→stretch、justify-content→flex-start）。R2383。
    Normal,
    /// flex-start。
    FlexStart,
    /// flex-end。
    FlexEnd,
    /// center。
    Center,
    /// space-between。
    SpaceBetween,
    /// space-around。
    SpaceAround,
    /// space-evenly。
    SpaceEvenly,
    /// stretch。
    Stretch,
    /// start。
    Start,
    /// end。
    End,
    /// baseline。
    Baseline,
}

/// CSS box-sizing 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxSizingValue {
    /// content-box。
    ContentBox,
    /// border-box。
    BorderBox,
}

/// CSS visibility 值。
#[derive(Debug, Clone, PartialEq)]
pub enum VisibilityValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
    /// collapse。
    Collapse,
}

/// CSS content-visibility 值（CSS Containment Module Level 2）。
///
/// - `Visible`（初始值）：元素正常渲染，不影响内容。
/// - `Hidden`：元素自身盒（背景/边框）仍绘制，但其整个子树（子元素 + 直属文本）
///   被跳过——不参与绘制，且不贡献元素尺寸（等价 `contain: size layout paint` +
///   内容跳过）。静态渲染等价于「内容不渲染」。
/// - `Auto`：动态跳过（需 IntersectionObserver 类视口观测）。静态全在屏内容
///   等价 `Visible`，故此处按 `Visible` 处理（无观测基础设施时 spec 允许退化为不跳过）。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentVisibilityValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
    /// auto（静态等价 visible）。
    Auto,
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
/// 控制 CJK 文本的换行严格度。`Anywhere` 在每个排版字符处创建换行机会
/// （覆盖 GL/JW/ZJW 禁则），近似 `word-break: break-all` 但更强。其余值
/// （strict/loose/normal/auto）涉及 CJK 标点/小假名等规则，当前解析但按
/// 默认（normal）行为处理。
#[derive(Debug, Clone, PartialEq)]
pub enum LineBreakValue {
    /// auto（依 locale，等价 normal）。
    Auto,
    /// loose（松散，用于短行如报纸）。
    Loose,
    /// normal（默认）。
    Normal,
    /// strict（严格）。
    Strict,
    /// anywhere（任意字符可换行，覆盖禁则）。
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

/// CSS text-decoration-thickness 值（CSS Text Decoration 4 §2.3）。
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecorationThicknessValue {
    /// auto（默认）：由字体度量决定厚度。
    Auto,
    /// from-font：显式从字体度量取 underline 厚度（ZW 无字体度量，回退 auto）。
    FromFont,
    /// 明确长度或百分比。R1402/R3565：used value 阶段按字体上下文解析。
    Length(LengthValue),
}

/// CSS text-underline-offset 值（CSS Text Decoration 4 §2.5）。
///
/// 下划线相对默认位置的额外下沉量（正值=远离文字下沉，负值=上抬）。
/// `auto`（默认）= 不加额外偏移，保留既有 baseline+font_size×0.15 近似位置。
/// 仅 underline 受影响（overline/line-through 不受）。继承属性。em/rem/% 在 paint
/// 期按元素 font_size 解析（driver test text-underline-offset-002 用 px，
/// text-underline-offset-percentage 用 %）。
#[derive(Debug, Clone, PartialEq)]
pub enum TextUnderlineOffsetValue {
    /// auto（默认）：无额外偏移。
    Auto,
    /// 明确长度（px/em/rem/ch/% 等原始 LengthValue，paint 期 resolve）。
    Length(LengthValue),
}

/// CSS text-decoration-inset 值（CSS Text Decoration 4 §2.4）。R1607。
///
/// 装饰线在 inline 轴的内缩量：`start` 控制 inline-start 端，`end` 控制 inline-end 端。
/// 负值表示向外延伸（延长装饰线）。em/rem 在 paint 期按元素 font_size 解析为 px
/// （与 text-decoration-thickness 相反，inset 的 driver test 用 em，故不在解析期 resolve）。
#[derive(Debug, Clone, PartialEq)]
pub struct TextDecorationInsetValue {
    /// inline-start 端内缩（正值=向内缩进，负值=向外延伸）。
    pub start: LengthValue,
    /// inline-end 端内缩。
    pub end: LengthValue,
}

/// CSS text-emphasis-style 值（CSS Text Decoration 3 §3.1）。
/// 解析后存为标记字符（如 filled dot → '•' U+2022），None = 无标记。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEmphasisStyleValue {
    /// none（默认）。
    None,
    /// 解析后的标记字符（关键字组合或 `<string>` 首字符）。
    Char(char),
}

/// CSS text-emphasis-position 值（CSS Text Decoration 3 §3.2）。
/// 水平书写模式：over = 文本上方，under = 下方；left/right 仅垂直模式有视觉差异。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEmphasisPositionValue {
    /// over right（默认）。
    OverRight,
    /// over left。
    OverLeft,
    /// under right。
    UnderRight,
    /// under left。
    UnderLeft,
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

/// CSS font-weight 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontWeightValue {
    /// 绝对权重（100-900）。
    Absolute(u16),
    /// bold。
    Bold,
    /// normal。
    Normal,
    /// bolder。
    Bolder,
    /// lighter。
    Lighter,
}

/// CSS font-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyleValue {
    /// normal。
    Normal,
    /// italic。
    Italic,
    /// oblique。
    Oblique(Option<f64>),
}

/// CSS 自定义属性引用（`var()` 函数）。
#[derive(Debug, Clone, PartialEq)]
pub struct VarReference {
    /// 自定义属性名（如 `--main-color`）。
    pub name: String,
    /// 回退值。
    pub fallback: Option<String>,
}

/// CSS calc() 表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    /// 数值常量。
    Number(f64),
    /// 长度值（带单位）。
    Length(LengthValue),
    /// 二元运算：left op right。
    BinaryOp(Box<CalcExpr>, CalcOp, Box<CalcExpr>),
    /// min() 函数：取所有参数中的最小值。
    Min(Vec<CalcExpr>),
    /// max() 函数：取所有参数中的最大值。
    Max(Vec<CalcExpr>),
    /// clamp(min, val, max) 函数：将 val 限制在 [min, max] 范围内。
    Clamp {
        /// 最小值。
        min: Box<CalcExpr>,
        /// 首选值。
        val: Box<CalcExpr>,
        /// 最大值。
        max: Box<CalcExpr>,
    },
    /// CSS Values L4 单参数数学函数：abs/sign/sqrt/exp/log（driving: R2279 CSS Values L4 数学函数）。
    UnaryOp(UnaryMathOp, Box<CalcExpr>),
    /// CSS Values L4 双参数数学函数：pow/hypot/round/mod/rem（driving: R2280 CSS Values L4 数学函数）。
    BinaryMathOp(BinaryMathOp, Box<CalcExpr>, Box<CalcExpr>),
}

/// CSS Values L4 单参数数学函数运算符（number → number）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryMathOp {
    /// abs(x) —— 绝对值。
    Abs,
    /// sign(x) —— 符号（-1/0/1）。
    Sign,
    /// sqrt(x) —— 平方根。
    Sqrt,
    /// exp(x) —— 自然指数 eˣ。
    Exp,
    /// log(x) —— 自然对数 ln(x)。
    Log,
    /// sin(x) —— 正弦（x 为弧度 number 或 <angle>，由 parse_angle_to_radians 归一）。
    Sin,
    /// cos(x) —— 余弦。
    Cos,
    /// tan(x) —— 正切。
    Tan,
    /// asin(x) —— 反正弦（[-1,1] → 弧度 [-π/2,π/2]；|x|>1 无效）。
    Asin,
    /// acos(x) —— 反余弦（[-1,1] → 弧度 [0,π]；|x|>1 无效）。
    Acos,
    /// atan(x) —— 反正切（任意 → 弧度 (-π/2,π/2)）。
    Atan,
}

/// CSS Values L4 双参数数学函数运算符（number, number → number）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryMathOp {
    /// pow(a, b) —— 幂 a^b。
    Pow,
    /// hypot(a, b) —— √(a²+b²)。
    Hypot,
    /// round(a, b) —— a 四舍五入到 b 的最近整数倍（nearest 策略，半值远离零）。
    Round,
    /// mod(a, b) —— 取模（结果符号同 b，floor 除法）。
    Mod,
    /// rem(a, b) —— 余数（结果符号同 a，trunc 除法）。
    Rem,
    /// atan2(y, x) —— 二参数反正切（弧度 (-π,π]）。
    Atan2,
}

/// CSS calc() 运算符。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcOp {
    /// 加法。
    Add,
    /// 减法。
    Subtract,
    /// 乘法。
    Multiply,
    /// 除法。
    Divide,
}

/// CSS clip-path 基本形状值。
///
/// 支持 inset()、circle()、ellipse()、polygon() 四种基本形状，
/// 以及 none 关键字。不支持 url() 引用 SVG clipPath 元素。
#[derive(Debug, Clone, PartialEq)]
pub enum ClipPathValue {
    /// none — 不裁剪（默认值）。
    None,
    /// inset() — 矩形裁剪，四个内缩距离。
    Inset {
        /// 上内缩。
        top: LengthValue,
        /// 右内缩。
        right: LengthValue,
        /// 下内缩。
        bottom: LengthValue,
        /// 左内缩。
        left: LengthValue,
        /// 可选圆角半径（border-radius 语法）。
        round: Option<ClipPathRadius>,
    },
    /// circle() — 圆形裁剪。
    Circle {
        /// 圆的半径。
        radius: ClipPathRadius,
        /// 圆心位置（at 关键字）。
        position: Option<(LengthValue, LengthValue)>,
    },
    /// ellipse() — 椭圆形裁剪。
    Ellipse {
        /// 水平半径。
        rx: ClipPathRadius,
        /// 垂直半径。
        ry: ClipPathRadius,
        /// 圆心位置（at 关键字）。
        position: Option<(LengthValue, LengthValue)>,
    },
    /// polygon() — 多边形裁剪。
    Polygon {
        /// 填充规则（nonzero 或 evenodd）。
        fill_rule: PolygonFillRule,
        /// 顶点坐标列表。
        points: Vec<(LengthValue, LengthValue)>,
    },
}

/// clip-path 半径值。
///
/// 可以是具体长度、百分比或 closest-side/farthest-side 关键字。
#[derive(Debug, Clone, PartialEq)]
pub enum ClipPathRadius {
    /// 具体长度值。
    Length(LengthValue),
    /// closest-side — 最近边。
    ClosestSide,
    /// farthest-side — 最远边。
    FarthestSide,
}

/// polygon() 填充规则。
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PolygonFillRule {
    /// nonzero（默认）。
    #[default]
    NonZero,
    /// evenodd。
    EvenOdd,
}

/// CSS calc() 表达式求值上下文。
///
/// 提供相对单位转换为像素值所需的参考尺寸。
#[derive(Debug, Clone, Default)]
pub struct CalcContext {
    /// 父元素长度，用于百分比计算。
    pub parent_length: Option<f64>,
    /// 当前字体大小（px），用于 em 单位转换。
    pub font_size: Option<f64>,
    /// 当前字体 x-height（px），用于 ex 单位转换。
    pub x_height: Option<f64>,
    /// 根元素字体大小（px），用于 rem 单位转换。
    pub root_font_size: Option<f64>,
    /// 根元素字体 x-height（px），用于 rex 单位转换。
    pub root_x_height: Option<f64>,
    /// 当前字体 cap-height（px），用于 cap 单位转换。
    pub cap_height: Option<f64>,
    /// 根元素字体 cap-height（px），用于 rcap 单位转换。
    pub root_cap_height: Option<f64>,
    /// 根元素字体 "0" 字形宽度（px），用于 rch 单位转换。
    pub root_ch_width: Option<f64>,
    /// 当前字体 "水" 字形宽度（px），用于 ic 单位转换。
    pub ic_width: Option<f64>,
    /// 根元素字体 "水" 字形宽度（px），用于 ric 单位转换。
    pub root_ic_width: Option<f64>,
    /// 视口高度（px），用于 vh/vmin/vmax 单位转换。
    pub viewport_height: Option<f64>,
    /// 视口宽度（px），用于 vw/vmin/vmax 单位转换。
    pub viewport_width: Option<f64>,
    /// "0" 字形宽度（px），用于 ch 单位转换。
    pub ch_width: Option<f64>,
}

/// calc() 表达式解析器内部状态。
struct CalcParser<'a> {
    /// 待解析的输入切片。
    input: &'a str,
    /// 当前位置（字节偏移）。
    pos: usize,
    /// 当前递归深度。
    depth: u32,
}

/// 最大递归深度限制。
const MAX_CALC_DEPTH: u32 = 10;

impl<'a> CalcParser<'a> {
    /// 跳过前导空白。
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// 查看当前剩余输入。
    fn peek_rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// 尝试消费指定前缀。
    fn try_consume(&mut self, prefix: &str) -> bool {
        let rest = self.peek_rest();
        if rest.starts_with(prefix) {
            self.pos += prefix.len();
            true
        } else {
            false
        }
    }

    /// 解析顶层表达式（处理 + - 运算符，优先级较低）。
    fn parse_expr(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            let rest = self.peek_rest();
            if rest.starts_with(')') || rest.is_empty() {
                break;
            }
            if rest.starts_with('+') {
                self.pos += 1;
                let right = self.parse_term()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Add, Box::new(right));
            } else if rest.starts_with('-') {
                // 区分减号和负号：减号前面有操作数
                self.pos += 1;
                let right = self.parse_term()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Subtract, Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析高优先级项（处理 * / 运算符）。
    fn parse_term(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_factor()?;

        loop {
            self.skip_whitespace();
            let rest = self.peek_rest();
            if rest.starts_with('*') {
                self.pos += 1;
                let right = self.parse_factor()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Multiply, Box::new(right));
            } else if rest.starts_with('/') {
                self.pos += 1;
                let right = self.parse_factor()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Divide, Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析原子因子：数字、长度值、嵌套 calc() 或括号表达式。
    fn parse_factor(&mut self) -> Option<CalcExpr> {
        self.skip_whitespace();

        // 处理负号前缀
        let neg = if self.peek_rest().starts_with('-') {
            // 判断是否为负号（而非减号）：后面紧跟数字、小数点或标识符首字符
            // （支持 -5、-.5、-infinity/-pi/-e/-nan 常量、-calc(...) 等；二元减号由 parse_expr 处理）。
            let after = self.peek_rest()[1..].trim_start();
            if after.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c.is_ascii_alphabetic()) {
                self.pos += 1;
                true
            } else {
                false
            }
        } else {
            false
        };

        self.skip_whitespace();

        let mut expr = if self.try_consume("calc(") {
            // 嵌套 calc() 表达式
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            inner
        } else if self.try_consume("min(") {
            // min(v1, v2, ...) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let args = self.parse_comma_list()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Min(args)
        } else if self.try_consume("max(") {
            // max(v1, v2, ...) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let args = self.parse_comma_list()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Max(args)
        } else if self.try_consume("clamp(") {
            // clamp(min, val, max) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let min = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(",") {
                return None;
            }
            let val = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(",") {
                return None;
            }
            let max = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Clamp {
                min: Box::new(min),
                val: Box::new(val),
                max: Box::new(max),
            }
        } else if let Some(b) = self.try_parse_binary_math() {
            // CSS Values L4 双参数数学函数 pow/hypot/round/mod/rem。
            b
        } else if let Some(u) = self.try_parse_unary_math() {
            // CSS Values L4 单参数数学函数 abs/sign/sqrt/exp/log。
            u
        } else if self.try_consume("(") {
            // 括号表达式
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            inner
        } else {
            // 解析原子操作数：数值或长度值
            self.parse_atom()?
        };

        if neg {
            expr = CalcExpr::BinaryOp(Box::new(CalcExpr::Number(0.0)), CalcOp::Subtract, Box::new(expr));
        }

        Some(expr)
    }

    /// 尝试解析 CSS Values L4 单参数数学函数 abs/sign/sqrt/exp/log + trig sin/cos/tan/asin/acos/atan。
    /// 命中则消费前缀 + 解析内层 expr + 消费 `)`，返回 UnaryOp；否则不消费、返回 None。
    fn try_parse_unary_math(&mut self) -> Option<CalcExpr> {
        if self.depth >= MAX_CALC_DEPTH {
            return None;
        }
        let op = if self.try_consume("abs(") {
            UnaryMathOp::Abs
        } else if self.try_consume("sign(") {
            UnaryMathOp::Sign
        } else if self.try_consume("sqrt(") {
            UnaryMathOp::Sqrt
        } else if self.try_consume("exp(") {
            UnaryMathOp::Exp
        } else if self.try_consume("log(") {
            UnaryMathOp::Log
        } else if self.try_consume("sin(") {
            UnaryMathOp::Sin
        } else if self.try_consume("cos(") {
            UnaryMathOp::Cos
        } else if self.try_consume("tan(") {
            UnaryMathOp::Tan
        } else if self.try_consume("asin(") {
            UnaryMathOp::Asin
        } else if self.try_consume("acos(") {
            UnaryMathOp::Acos
        } else if self.try_consume("atan(") {
            UnaryMathOp::Atan
        } else {
            return None; // 未命中：try_consume 仅在匹配时消费，此处无消费
        };
        self.depth += 1;
        let inner = self.parse_expr()?;
        self.skip_whitespace();
        if !self.try_consume(")") {
            return None;
        }
        self.depth -= 1;
        Some(CalcExpr::UnaryOp(op, Box::new(inner)))
    }

    /// 尝试解析 CSS Values L4 双参数数学函数 pow/hypot/round/mod/rem/atan2。
    /// 命中则消费前缀 + 解析 2 个逗号分隔参数 + 消费 `)`；否则不消费、返回 None。
    /// round 的 rounding-strategy 关键字形式（round(nearest, A, B)）defer —— 仅支持 round(A, B)。
    fn try_parse_binary_math(&mut self) -> Option<CalcExpr> {
        if self.depth >= MAX_CALC_DEPTH {
            return None;
        }
        let op = if self.try_consume("pow(") {
            BinaryMathOp::Pow
        } else if self.try_consume("hypot(") {
            BinaryMathOp::Hypot
        } else if self.try_consume("round(") {
            BinaryMathOp::Round
        } else if self.try_consume("mod(") {
            BinaryMathOp::Mod
        } else if self.try_consume("rem(") {
            BinaryMathOp::Rem
        } else if self.try_consume("atan2(") {
            BinaryMathOp::Atan2
        } else {
            return None;
        };
        self.depth += 1;
        let args = self.parse_comma_list()?;
        if args.len() != 2 {
            return None;
        }
        self.skip_whitespace();
        if !self.try_consume(")") {
            return None;
        }
        self.depth -= 1;
        let mut iter = args.into_iter();
        Some(CalcExpr::BinaryMathOp(
            op,
            Box::new(iter.next()?),
            Box::new(iter.next()?),
        ))
    }

    /// 解析逗号分隔的表达式列表（用于 min/max 函数）。
    fn parse_comma_list(&mut self) -> Option<Vec<CalcExpr>> {
        let mut args = Vec::new();
        args.push(self.parse_expr()?);
        loop {
            self.skip_whitespace();
            if !self.try_consume(",") {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Some(args)
    }

    /// 解析原子操作数（数值或带单位的长度值）。
    fn parse_atom(&mut self) -> Option<CalcExpr> {
        self.skip_whitespace();
        let rest = self.peek_rest();

        // 从当前位置读取到下一个运算符、空白、右括号或逗号
        let end = rest
            .bytes()
            .position(|b| b == b'+' || b == b'-' || b == b'*' || b == b'/' || b == b')' || b == b',')
            .unwrap_or(rest.len());

        if end == 0 {
            return None;
        }

        let token = rest[..end].trim();
        if token.is_empty() {
            return None;
        }

        self.pos += rest[..end].len();

        // 尝试解析为纯数字
        if let Ok(num) = token.parse::<f64>() {
            return Some(CalcExpr::Number(num));
        }

        // CSS Values L4 常量：pi/e/infinity/NaN（大小写不敏感）。
        match token.to_ascii_lowercase().as_str() {
            "pi" => return Some(CalcExpr::Number(std::f64::consts::PI)),
            "e" => return Some(CalcExpr::Number(std::f64::consts::E)),
            "infinity" => return Some(CalcExpr::Number(f64::INFINITY)),
            "nan" => return Some(CalcExpr::Number(f64::NAN)),
            _ => {}
        }

        // CSS Values L4 <angle> 单位（deg/grad/turn/rad）→ 弧度 Number（供 trig 函数参数）。
        if let Some(rad) = parse_angle_to_radians(token) {
            return Some(CalcExpr::Number(rad));
        }

        // 尝试解析为长度值
        if let Some(length) = parse_length(token) {
            return Some(CalcExpr::Length(length));
        }

        None
    }
}

/// 解析 CSS calc() 表达式。
///
/// 支持格式如 `"calc(100% - 20px)"`、`"calc(50% + 10px)"`、`"calc(2 * 10px)"`。
/// 支持嵌套 calc 表达式如 `"calc(calc(100% - 20px) / 2)"`。
/// 运算符优先级：`*` `/` 高于 `+` `-`。
pub fn parse_calc(value: &str) -> Option<CalcExpr> {
    let value = value.trim();

    // 检查 calc(...) 包装
    if !value.starts_with("calc(") || !value.ends_with(')') {
        return None;
    }

    let inner = value.get(5..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }

    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };

    let expr = parser.parse_expr()?;

    // 确保整个输入已被消费
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }

    Some(expr)
}

/// 解析 CSS 数学函数（calc/min/max/clamp）。
///
/// 根据前缀自动识别并解析对应的数学函数。
/// 返回统一的 [`CalcExpr`] 表达式树。
pub fn parse_math_function(value: &str) -> Option<CalcExpr> {
    // CSS Values §4：函数名大小写不敏感（CALC ≡ calc、MIN ≡ min、MAX ≡ max、CLAMP ≡ clamp）。
    // 归一化小写后分发；内容（数字 + 长度）亦大小写不敏感（R2346），故整体小写委托安全。
    // 修复前 starts_with 仅认小写 → 大写/混合大小写函数名落 None（部分调用方先 lowercase
    // 掩盖、部分直传 raw 暴露，如 parse_transform.rs / parse_extended_visual.rs 不一致）。
    let value = value.trim().to_ascii_lowercase();

    if value.starts_with("calc(") && value.ends_with(')') {
        parse_calc(&value)
    } else if value.starts_with("min(") && value.ends_with(')') {
        parse_min(&value)
    } else if value.starts_with("max(") && value.ends_with(')') {
        parse_max(&value)
    } else if value.starts_with("clamp(") && value.ends_with(')') {
        parse_clamp(&value)
    } else {
        None
    }
}

/// 解析 CSS min() 函数。
///
/// 格式：`min(v1, v2, ...)` — 取所有参数中的最小值。
pub fn parse_min(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("min(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(4..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let args = parser.parse_comma_list()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Min(args))
}

/// 解析 CSS max() 函数。
///
/// 格式：`max(v1, v2, ...)` — 取所有参数中的最大值。
pub fn parse_max(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("max(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(4..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let args = parser.parse_comma_list()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Max(args))
}

/// 解析 CSS clamp() 函数。
///
/// 格式：`clamp(min, val, max)` — 将 val 限制在 [min, max] 范围。
pub fn parse_clamp(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("clamp(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(6..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let min = parser.parse_expr()?;
    parser.skip_whitespace();
    if !parser.try_consume(",") {
        return None;
    }
    let val = parser.parse_expr()?;
    parser.skip_whitespace();
    if !parser.try_consume(",") {
        return None;
    }
    let max = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Clamp {
        min: Box::new(min),
        val: Box::new(val),
        max: Box::new(max),
    })
}

/// 计算 CSS calc() 表达式的像素值。
///
/// `parent_length` 用于解析百分比值（如 `100%` = `parent_length`）。
/// 返回计算结果（像素）。
pub fn eval_calc(expr: &CalcExpr, parent_length: Option<f64>) -> Option<f64> {
    let ctx = CalcContext {
        parent_length,
        ..Default::default()
    };
    eval_calc_with_context(expr, &ctx)
}

/// 使用完整上下文计算 CSS calc() 表达式的像素值。
///
/// 支持所有单位：px、百分比、em、rem、vh、vw、vmin、vmax、ch。
/// 相对单位需要对应的上下文字段已设置，否则返回 `None`。
pub fn eval_calc_with_context(expr: &CalcExpr, ctx: &CalcContext) -> Option<f64> {
    match expr {
        CalcExpr::Number(n) => Some(*n),
        CalcExpr::Length(lv) => resolve_length_to_px(lv, ctx),
        CalcExpr::BinaryOp(left, op, right) => {
            let lv = eval_calc_with_context(left, ctx)?;
            let rv = eval_calc_with_context(right, ctx)?;
            match op {
                CalcOp::Add => Some(lv + rv),
                CalcOp::Subtract => Some(lv - rv),
                CalcOp::Multiply => Some(lv * rv),
                CalcOp::Divide => {
                    if rv == 0.0 {
                        None
                    } else {
                        Some(lv / rv)
                    }
                }
            }
        }
        CalcExpr::Min(args) => {
            let vals: Vec<f64> = args.iter().filter_map(|a| eval_calc_with_context(a, ctx)).collect();
            if vals.is_empty() {
                None
            } else {
                Some(vals.into_iter().reduce(f64::min).unwrap())
            }
        }
        CalcExpr::Max(args) => {
            let vals: Vec<f64> = args.iter().filter_map(|a| eval_calc_with_context(a, ctx)).collect();
            if vals.is_empty() {
                None
            } else {
                Some(vals.into_iter().reduce(f64::max).unwrap())
            }
        }
        CalcExpr::Clamp { min, val, max } => {
            let min_v = eval_calc_with_context(min, ctx)?;
            let val_v = eval_calc_with_context(val, ctx)?;
            let max_v = eval_calc_with_context(max, ctx)?;
            // CSS Values §11：clamp 语义 = max(MIN, min(VAL, MAX))。
            // 注意：不可用 `f64::clamp`——std 在 `min > max` 或含 NaN 时 **panic**
            //（R3344 deep-review：`calc(clamp(100px,50px,10px))` MIN>MAX 致渲染进程
            // panic，攻击者 CSS 可触发）。`min`/`max` 显式实现 min>max 退化（回退到
            // MIN）且对 NaN 安全（NaN 比较恒 false，传播为 NaN 而非 panic）。
            // https://www.w3.org/TR/css-values-3/#calc-range
            Some(val_v.min(max_v).max(min_v))
        }
        // CSS Values L4 单参数数学函数（number → number）。sqrt(负)/log(≤0) → None（IACVT 无效）。
        CalcExpr::UnaryOp(op, inner) => {
            let v = eval_calc_with_context(inner, ctx)?;
            match op {
                UnaryMathOp::Abs => Some(v.abs()),
                // CSS Values L4 sign：>0→1、<0 或 -0→-1、+0→0（f64::signum(0.0)=1.0 不符 spec）。
                UnaryMathOp::Sign => Some(if v > 0.0 {
                    1.0
                } else if v.is_sign_negative() {
                    -1.0
                } else {
                    0.0
                }),
                UnaryMathOp::Sqrt => {
                    if v >= 0.0 {
                        Some(v.sqrt())
                    } else {
                        None
                    }
                }
                UnaryMathOp::Exp => Some(v.exp()),
                UnaryMathOp::Log => {
                    if v > 0.0 {
                        Some(v.ln())
                    } else {
                        None
                    }
                }
                // trig：sin/cos/tan 输入弧度（parse_angle_to_radians 已把 <angle> 归一）。
                UnaryMathOp::Sin => Some(v.sin()),
                UnaryMathOp::Cos => Some(v.cos()),
                UnaryMathOp::Tan => Some(v.tan()),
                // 反三角：asin/acos 对 |v|>1 产生 NaN → None（CSS 无效）；atan 恒有效。
                UnaryMathOp::Asin => {
                    let r = v.asin();
                    if r.is_nan() { None } else { Some(r) }
                }
                UnaryMathOp::Acos => {
                    let r = v.acos();
                    if r.is_nan() { None } else { Some(r) }
                }
                UnaryMathOp::Atan => Some(v.atan()),
            }
        }
        // CSS Values L4 双参数数学函数（number, number → number）。
        CalcExpr::BinaryMathOp(op, a, b) => {
            let av = eval_calc_with_context(a, ctx)?;
            let bv = eval_calc_with_context(b, ctx)?;
            match op {
                BinaryMathOp::Pow => {
                    let r = av.powf(bv);
                    if r.is_nan() { None } else { Some(r) }
                }
                BinaryMathOp::Hypot => Some(av.hypot(bv)),
                // round(x, 0) = x；否则 (x/y).round()*y（nearest，半值远离零）。
                BinaryMathOp::Round => Some(if bv == 0.0 { av } else { (av / bv).round() * bv }),
                // mod(x, 0) 无效 → None；否则 x - y*floor(x/y)（符号同 y）。
                BinaryMathOp::Mod => {
                    if bv == 0.0 {
                        None
                    } else {
                        Some(av - bv * (av / bv).floor())
                    }
                }
                // rem(x, 0) 无效 → None；否则 x % y（符号同 x）。
                BinaryMathOp::Rem => {
                    if bv == 0.0 {
                        None
                    } else {
                        Some(av % bv)
                    }
                }
                // atan2(y, x) —— 二参数反正切（弧度 (-π,π]），恒有效。
                BinaryMathOp::Atan2 => Some(av.atan2(bv)),
            }
        }
    }
}

/// 将长度值解析为像素值。
///
/// 使用 [`CalcContext`] 中提供的参考尺寸转换相对单位。
fn resolve_length_to_px(lv: &LengthValue, ctx: &CalcContext) -> Option<f64> {
    match lv {
        LengthValue::Px(v) => Some(*v),
        LengthValue::Percentage(pct) => ctx.parent_length.map(|pl| pct / 100.0 * pl),
        LengthValue::Em(v) => ctx.font_size.map(|fs| v * fs),
        LengthValue::Ex(v) => ctx.x_height.map(|xh| v * xh),
        LengthValue::Rex(v) => ctx.root_x_height.map(|xh| v * xh),
        LengthValue::Cap(v) => ctx.cap_height.map(|height| v * height),
        LengthValue::Rcap(v) => ctx.root_cap_height.map(|height| v * height),
        LengthValue::Rem(v) => ctx.root_font_size.map(|rfs| v * rfs),
        LengthValue::Vh(v) => ctx.viewport_height.map(|vh| v * vh / 100.0),
        LengthValue::Vw(v) => ctx.viewport_width.map(|vw| v * vw / 100.0),
        LengthValue::Vmin(v) => match (ctx.viewport_width, ctx.viewport_height) {
            (Some(vw), Some(vh)) => Some(v * vw.min(vh) / 100.0),
            _ => None,
        },
        LengthValue::Vmax(v) => match (ctx.viewport_width, ctx.viewport_height) {
            (Some(vw), Some(vh)) => Some(v * vw.max(vh) / 100.0),
            _ => None,
        },
        LengthValue::Ch(v) => ctx.ch_width.map(|cw| v * cw),
        LengthValue::Rch(v) => ctx.root_ch_width.map(|cw| v * cw),
        LengthValue::Ic(v) => ctx.ic_width.map(|width| v * width),
        LengthValue::Ric(v) => ctx.root_ic_width.map(|width| v * width),
        LengthValue::Auto => None,
        LengthValue::Calc(expr) => eval_calc_with_context(expr, ctx),
        LengthValue::FitContent(inner) => resolve_length_to_px(inner, ctx),
        // min-content/max-content 需要内容信息才能计算，此处返回 None
        LengthValue::MinContent | LengthValue::MaxContent => None,
    }
}

/// 解析 CSS `<angle>` 为弧度数值（CSS Values L4，供 calc trig 函数参数）。
///
/// 支持单位：`deg`（°×π/180）、`grad`（×π/200）、`turn`（×2π）、`rad`（×1）。
/// 裸数字非 angle（返回 None）；大小写不敏感。driving: CSS Values L4 sin/cos/tan(<angle>)。
pub fn parse_angle_to_radians(token: &str) -> Option<f64> {
    let lower = token.to_ascii_lowercase();
    let (num_str, factor) = [
        ("deg", std::f64::consts::PI / 180.0),
        ("grad", std::f64::consts::PI / 200.0),
        ("turn", 2.0 * std::f64::consts::PI),
        ("rad", 1.0),
    ]
    .into_iter()
    .find_map(|(suffix, f)| lower.strip_suffix(suffix).map(|n| (n, f)))?;
    let v: f64 = num_str.trim().parse().ok()?;
    Some(v * factor)
}

/// 解析 CSS 长度值。
///
/// 支持格式如 `"10px"`、`"1.5em"`、`"2rem"`、`"100vh"`、`"50%"`、`"auto"`、
/// `"fit-content(200px)"` 等。
pub fn parse_length(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 处理 auto 关键字
    if value.eq_ignore_ascii_case("auto") {
        return Some(LengthValue::Auto);
    }

    // 处理 border-width 关键字（CSS 2.1 §8.5.1）：thin=1px / medium=3px / thick=5px。
    // border 简写缺省宽度时展开为 medium；死代码 parse_basic.rs 有此处理但 types.rs（活）
    // 遗漏（R544 死代码陷阱同谱系）→ `border: solid` 解析为 width:0（无边框 + 不阻断 margin 折叠）。
    if value.eq_ignore_ascii_case("thin") {
        return Some(LengthValue::Px(1.0));
    }
    if value.eq_ignore_ascii_case("medium") {
        return Some(LengthValue::Px(3.0));
    }
    if value.eq_ignore_ascii_case("thick") {
        return Some(LengthValue::Px(5.0));
    }

    // 处理 min-content/max-content 关键字
    if value.eq_ignore_ascii_case("min-content") {
        return Some(LengthValue::MinContent);
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some(LengthValue::MaxContent);
    }

    // R1018：bare `fit-content` 关键字（无参数，CSS css-sizing-3 legacy form）。
    // 语义 ≡ fit-content(max-content)，触发 shrink-to-fit（与 max-content 同 gate 触发）。
    // 复用 MaxContent 变体（layout trigger 等价；clamp-to-available 由 taffy block 布局自然处理）。
    // 注意：fit-content(arg) 函数形式有独立 FitContent(Box<LengthValue>) 变体（下方）。
    if value.eq_ignore_ascii_case("fit-content") {
        return Some(LengthValue::MaxContent);
    }

    // 处理 fit-content() 函数
    if value.starts_with("fit-content(") && value.ends_with(')') {
        let inner = &value["fit-content(".len()..value.len() - 1];
        let inner = inner.trim();
        // fit-content() 不接受空参数
        if inner.is_empty() {
            return None;
        }
        let arg = parse_length(inner)?;
        return Some(LengthValue::FitContent(Box::new(arg)));
    }

    // 从字符串末尾扫描，找到单位部分的起始位置。
    // 单位部分由字母组成（可能以 '%' 结尾）；数字部分在单位之前。
    // 这样可以正确处理科学计数法（如 "1e2px"），因为 'e' 在数字部分内。
    let unit_start = find_unit_start(value);

    let num_str = &value[..unit_start];
    let unit = &value[unit_start..];

    let num: f64 = num_str.parse().ok()?;

    // CSS Values §4：长度单位大小写不敏感（1PX ≡ 1px、1Q ≡ 1q、12.5EX ≡ 12.5ex）。
    // 归一化为小写后匹配（修复前仅认小写 + "Q" 大写 → 常规 `1q` 等失败）。
    let unit = unit.to_ascii_lowercase();

    match unit.as_str() {
        "px" => Some(LengthValue::Px(num)),
        "em" => Some(LengthValue::Em(num)),
        "ex" => Some(LengthValue::Ex(num)),
        "rex" if std::env::var("ZW_ROOT_FONT_UNITS").as_deref() != Ok("0") => Some(LengthValue::Rex(num)),
        "cap" if std::env::var("ZW_ROOT_CAP_UNITS").as_deref() != Ok("0") => Some(LengthValue::Cap(num)),
        "rcap" if std::env::var("ZW_ROOT_CAP_UNITS").as_deref() != Ok("0") => Some(LengthValue::Rcap(num)),
        "rem" => Some(LengthValue::Rem(num)),
        "vh" => Some(LengthValue::Vh(num)),
        "vw" => Some(LengthValue::Vw(num)),
        "vmin" => Some(LengthValue::Vmin(num)),
        "vmax" => Some(LengthValue::Vmax(num)),
        // CSS Values 4 视口单位（small/large/dynamic viewport）：ZW 视口模型为桌面单视口，
        // svh/lvh/dvh 在桌面等价于 vh（移动端动态 UI 高度差异未建模）。映射到既有变体，
        // 修复前这些合法单位返回 None 致声明被丢弃。
        "svh" | "lvh" | "dvh" => Some(LengthValue::Vh(num)),
        "svw" | "lvw" | "dvw" => Some(LengthValue::Vw(num)),
        "svmin" | "lvmin" | "dvmin" => Some(LengthValue::Vmin(num)),
        "svmax" | "lvmax" | "dvmax" => Some(LengthValue::Vmax(num)),
        // CSS Values 4 视口 inline/block 单位：`vi`=1% 视口 inline 轴、`vb`=1% 视口 block 轴
        //（含 small/large/dynamic 变体）。ZW 仅水平书写模式布局（vertical-mode R1043 待授权），
        // 水平 tb 下 inline=水平、block=垂直，故 vi≡vw、vb≡vh（同 R2359 sv*/lv*/dv* 模式）。
        "vi" | "svi" | "lvi" | "dvi" => Some(LengthValue::Vw(num)),
        "vb" | "svb" | "lvb" | "dvb" => Some(LengthValue::Vh(num)),
        "ch" => Some(LengthValue::Ch(num)),
        "rch" if std::env::var("ZW_ROOT_FONT_UNITS").as_deref() != Ok("0") => Some(LengthValue::Rch(num)),
        "ic" if std::env::var("ZW_ROOT_IC_UNITS").as_deref() != Ok("0") => Some(LengthValue::Ic(num)),
        "ric" if std::env::var("ZW_ROOT_IC_UNITS").as_deref() != Ok("0") => Some(LengthValue::Ric(num)),
        "%" => Some(LengthValue::Percentage(num)),
        // CSS 绝对长度单位 → 转换为 px（96 DPI）
        "in" => Some(LengthValue::Px(num * 96.0)),
        "pt" => Some(LengthValue::Px(num * 96.0 / 72.0)),
        "pc" => Some(LengthValue::Px(num * 96.0 / 6.0)),
        "cm" => Some(LengthValue::Px(num * 96.0 / 2.54)),
        "mm" => Some(LengthValue::Px(num * 96.0 / 25.4)),
        "q" => Some(LengthValue::Px(num * 96.0 / 101.6)), // 1q = 1/4mm（大小写不敏感）
        // Per CSS spec, a bare zero without units is a valid length (0px).
        "" if num == 0.0 => Some(LengthValue::Px(0.0)),
        _ => None,
    }
}

/// Quirks mode 长度解析。
///
/// 先尝试标准 `parse_length`，如果失败，则尝试将裸数字解析为 px 值。
/// 这是浏览器在 quirks mode 下对长度属性值的宽容行为（如 `width: 100` 等同于 `width: 100px`）。
pub fn parse_length_quirks(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 先尝试标准解析
    if let Some(v) = parse_length(value) {
        return Some(v);
    }

    // Quirks: 裸数字视为 px
    if let Ok(num) = value.parse::<f64>() {
        return Some(LengthValue::Px(num));
    }

    None
}

/// 从字符串末尾找到单位部分的起始索引。
///
/// 从右向左扫描：跳过 '%'（如果有），然后跳过连续的字母字符，
/// 剩下的就是数字部分的结束位置。
fn find_unit_start(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();

    // 跳过末尾的 '%'
    if i > 0 && bytes[i - 1] == b'%' {
        i -= 1;
        return i;
    }

    // 从末尾向前跳过连续的 ASCII 字母（单位名）
    while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }

    i
}
