//! 背景与边框图像相关 CSS 计算值类型。
//!
//! 涵盖 `background-image`/`-position`/`-repeat`/`-size`/`-attachment`/`-clip`/`-origin`、
//! `mask-mode`、`border-image-*` 与 `list-style-image` 的计算值定义。R2534 从
//! `property/types.rs` 抽出为子模块，以满足单文件 ≤2000 行约束（CLAUDE.md §5 / rally
//! run-rules）；均为纯数据类型（`#[derive]`，无 inherent `impl`），机械迁移，原
//! `property::types::*` 公共路径与 types.rs 内部引用通过 `types.rs` 的 `pub use image::*`
//! 保持不变。

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
    /// calc()/min()/max()/clamp() 数学函数（延迟到 paint 期解析，% 相对 (container-image)）。
    /// R2313。
    Calc(zero_css_parser::values::CalcExpr),
    /// 两个值组合（水平 垂直）。
    TwoValue(
        Box<BackgroundPositionComputedValue>,
        Box<BackgroundPositionComputedValue>,
    ),
    /// R2478：3/4 值语法「边缘+偏移」对（CSS Backgrounds §3.6）。偏移从命名边度量，
    /// resolve 期 right/bottom 翻转（位置 = (container-image) - offset）。side 复用
    /// css-parser BackgroundEdge（left/right=水平，top/bottom=垂直）。
    EdgeOffset(
        zero_css_parser::values::BackgroundEdge,
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
