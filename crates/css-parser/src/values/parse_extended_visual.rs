//! CSS 视觉效果属性解析（object-fit、filter、appearance、混合模式、滚动条、文本换行、背景等）。

use super::*;

// ── CSS Object Fit 值类型 ──────────────────────────────────────────

/// CSS object-fit 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFitValue {
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

/// 解析 CSS object-fit 属性值。
pub fn parse_object_fit(value: &str) -> Option<ObjectFitValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fill" => Some(ObjectFitValue::Fill),
        "contain" => Some(ObjectFitValue::Contain),
        "cover" => Some(ObjectFitValue::Cover),
        "none" => Some(ObjectFitValue::None),
        "scale-down" => Some(ObjectFitValue::ScaleDown),
        _ => None,
    }
}

// ── CSS Filter 值类型 ──────────────────────────────────────────────

/// CSS filter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
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

/// 解析 CSS filter 属性值。
///
/// 支持格式如 `"none"`、`"blur(5px)"`、`"brightness(1.5)"` 等。
pub fn parse_filter(value: &str) -> Option<FilterValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(FilterValue::None);
    }

    // 解析单个 filter 函数
    if let Some(paren_pos) = value.find('(') {
        let func_name = value[..paren_pos].trim();
        if !value.ends_with(')') {
            return None;
        }
        let inner = value[paren_pos + 1..value.len() - 1].trim();

        match func_name.to_ascii_lowercase().as_str() {
            "blur" => {
                let px: f32 = parse_filter_length_px(inner)?;
                Some(FilterValue::Blur(px))
            }
            "brightness" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Brightness(n))
            }
            "contrast" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Contrast(n))
            }
            "grayscale" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Grayscale(n))
            }
            "hue-rotate" => {
                let deg: f32 = parse_filter_angle(inner)?;
                Some(FilterValue::HueRotate(deg))
            }
            "invert" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Invert(n))
            }
            "opacity" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Opacity(n))
            }
            "saturate" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Saturate(n))
            }
            "sepia" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Sepia(n))
            }
            "drop-shadow" => parse_drop_shadow(inner),
            _ => None,
        }
    } else {
        None
    }
}

/// 解析 filter 函数中的长度值（返回 px 数值）。
fn parse_filter_length_px(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("px") {
        s.trim_end_matches("px").trim().parse::<f32>().ok()
    } else {
        // 无单位值在 blur 中无效，但尝试解析为纯数值
        s.parse::<f32>().ok()
    }
}

/// 解析 filter 函数中的数值（0-1 范围，也接受百分比和大于 1 的值）。
fn parse_filter_number(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with('%') {
        let pct: f32 = s.trim_end_matches('%').parse().ok()?;
        Some(pct / 100.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// 解析 filter 函数中的角度值（返回度数）。
fn parse_filter_angle(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("deg") {
        s.trim_end_matches("deg").trim().parse::<f32>().ok()
    } else if s.ends_with("rad") {
        let rad: f32 = s.trim_end_matches("rad").trim().parse::<f32>().ok()?;
        Some(rad.to_degrees())
    } else if s.ends_with("turn") {
        let turn: f32 = s.trim_end_matches("turn").trim().parse::<f32>().ok()?;
        Some(turn * 360.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// 解析 drop-shadow 参数。
///
/// 格式：`x-offset y-offset blur-radius color` 或 `x-offset y-offset color`。
fn parse_drop_shadow(inner: &str) -> Option<FilterValue> {
    // 简化解析：按空格分割，识别颜色值
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let x: f32 = parts[0].parse().ok()?;
    let y: f32 = parts[1].parse().ok()?;
    // 尝试解析第三个参数为 blur 或 color
    let (blur, color) = if parts.len() >= 4 {
        let blur: f32 = parts[2].parse().ok()?;
        let color = parse_color(parts[3..].join(" ").as_str())?;
        (blur, color)
    } else {
        // 第三个参数是颜色
        let color = parse_color(parts[2..].join(" ").as_str())?;
        (0.0, color)
    };

    Some(FilterValue::DropShadow(x, y, blur, color))
}

// ── CSS Appearance 值类型 ──────────────────────────────────────────────

/// CSS appearance 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AppearanceValue {
    /// none。
    None,
    /// auto。
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

/// 解析 CSS appearance 属性值。
pub fn parse_appearance(value: &str) -> Option<AppearanceValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(AppearanceValue::None),
        "auto" => Some(AppearanceValue::Auto),
        "button" => Some(AppearanceValue::Button),
        "checkbox" => Some(AppearanceValue::Checkbox),
        "listbox" => Some(AppearanceValue::Listbox),
        "menulist" => Some(AppearanceValue::Menulist),
        "meter" => Some(AppearanceValue::Meter),
        "progress-bar" => Some(AppearanceValue::ProgressBar),
        "push-button" => Some(AppearanceValue::PushButton),
        "radio" => Some(AppearanceValue::Radio),
        "searchfield" => Some(AppearanceValue::Searchfield),
        "slider-horizontal" => Some(AppearanceValue::SliderHorizontal),
        "square-button" => Some(AppearanceValue::SquareButton),
        "textarea" => Some(AppearanceValue::Textarea),
        "textfield" => Some(AppearanceValue::Textfield),
        _ => None,
    }
}

/// CSS accent-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AccentColorValue {
    /// auto。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// 解析 CSS accent-color 属性值。
///
/// 支持格式：`auto` 或任意有效 CSS 颜色值。
pub fn parse_accent_color(value: &str) -> Option<AccentColorValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(AccentColorValue::Auto);
    }
    parse_color(v).map(AccentColorValue::Color)
}

/// CSS caret-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaretColorValue {
    /// auto。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// 解析 CSS caret-color 属性值。
///
/// 支持格式：`auto` 或任意有效 CSS 颜色值。
pub fn parse_caret_color(value: &str) -> Option<CaretColorValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(CaretColorValue::Auto);
    }
    parse_color(v).map(CaretColorValue::Color)
}

// ── CSS Mix Blend Mode 值类型 ──────────────────────────────────────────

/// CSS mix-blend-mode 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum MixBlendModeValue {
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

/// 解析 CSS mix-blend-mode 属性值。
pub fn parse_mix_blend_mode(value: &str) -> Option<MixBlendModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(MixBlendModeValue::Normal),
        "multiply" => Some(MixBlendModeValue::Multiply),
        "screen" => Some(MixBlendModeValue::Screen),
        "overlay" => Some(MixBlendModeValue::Overlay),
        "darken" => Some(MixBlendModeValue::Darken),
        "lighten" => Some(MixBlendModeValue::Lighten),
        "color-dodge" => Some(MixBlendModeValue::ColorDodge),
        "color-burn" => Some(MixBlendModeValue::ColorBurn),
        "hard-light" => Some(MixBlendModeValue::HardLight),
        "soft-light" => Some(MixBlendModeValue::SoftLight),
        "difference" => Some(MixBlendModeValue::Difference),
        "exclusion" => Some(MixBlendModeValue::Exclusion),
        "hue" => Some(MixBlendModeValue::Hue),
        "saturation" => Some(MixBlendModeValue::Saturation),
        "color" => Some(MixBlendModeValue::Color),
        "luminosity" => Some(MixBlendModeValue::Luminosity),
        _ => None,
    }
}

/// CSS scrollbar-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarWidthValue {
    /// auto（默认值）— 浏览器默认滚动条宽度。
    Auto,
    /// thin — 细滚动条。
    Thin,
    /// none — 隐藏滚动条。
    None,
}

/// 解析 CSS scrollbar-width 属性值。
pub fn parse_scrollbar_width(value: &str) -> Option<ScrollbarWidthValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(ScrollbarWidthValue::Auto),
        "thin" => Some(ScrollbarWidthValue::Thin),
        "none" => Some(ScrollbarWidthValue::None),
        _ => None,
    }
}

/// CSS scrollbar-gutter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarGutterValue {
    /// auto（默认值）— 仅在内容溢出时保留滚动条空间。
    Auto,
    /// stable — 始终保留滚动条空间。
    Stable,
    /// stable both-edges — 在两侧都保留滚动条空间。
    StableBothEdges,
}

/// 解析 CSS scrollbar-gutter 属性值。
///
/// 支持格式：`auto`、`stable`、`stable both-edges`。
pub fn parse_scrollbar_gutter(value: &str) -> Option<ScrollbarGutterValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(ScrollbarGutterValue::Auto),
        "stable" => Some(ScrollbarGutterValue::Stable),
        "stable both-edges" | "both-edges stable" => Some(ScrollbarGutterValue::StableBothEdges),
        _ => None,
    }
}

// ── CSS Text Wrap 值类型 ──────────────────────────────────────────────

/// CSS text-wrap 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextWrapValue {
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

/// 解析 CSS text-wrap 属性值。
pub fn parse_text_wrap(value: &str) -> Option<TextWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "wrap" => Some(TextWrapValue::Wrap),
        "nowrap" => Some(TextWrapValue::Nowrap),
        "balance" => Some(TextWrapValue::Balance),
        "pretty" => Some(TextWrapValue::Pretty),
        "stable" => Some(TextWrapValue::Stable),
        _ => None,
    }
}

// ── CSS Hyphens 值类型 ──────────────────────────────────────────────

/// CSS hyphens 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum HyphensValue {
    /// none（默认值）— 不使用连字符断词。
    None,
    /// manual — 手动断词（需使用软连字符）。
    Manual,
    /// auto — 自动断词。
    Auto,
}

/// 解析 CSS hyphens 属性值。
pub fn parse_hyphens(value: &str) -> Option<HyphensValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(HyphensValue::None),
        "manual" => Some(HyphensValue::Manual),
        "auto" => Some(HyphensValue::Auto),
        _ => None,
    }
}

// ── CSS Line Clamp 值类型 ──────────────────────────────────────────────

/// CSS line-clamp 属性值（-webkit-line-clamp）。
#[derive(Debug, Clone, PartialEq)]
pub enum LineClampValue {
    /// none（默认值）— 不限制行数。
    None,
    /// 限制为指定行数。
    Count(u32),
}

/// 解析 CSS line-clamp 属性值。
///
/// 支持格式如 `"none"`、`"3"`。
pub fn parse_line_clamp(value: &str) -> Option<LineClampValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(LineClampValue::None);
    }
    let n: u32 = value.parse().ok()?;
    if n > 0 { Some(LineClampValue::Count(n)) } else { None }
}

// ── CSS Background 值类型 ──────────────────────────────────────────────

/// CSS background-image 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundImageValue {
    /// none（默认值）— 无背景图片。
    None,
    /// url(<string>) — 指定背景图片 URL。
    Url(String),
    /// 渐变函数 — linear-gradient / radial-gradient / conic-gradient。
    Gradient(GradientValue),
}

/// 解析 CSS background-image 属性值。
///
/// 支持格式如 `"none"`、`"url(image.png)"`、`"linear-gradient(...)"` 等。
pub fn parse_background_image(value: &str) -> Option<BackgroundImageValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(BackgroundImageValue::None);
    }

    // 解析 url(...) 函数
    if value.starts_with("url(") && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = inner.trim();
        // 去除可选的引号
        let url = if (url.starts_with('"') && url.ends_with('"')) || (url.starts_with('\'') && url.ends_with('\'')) {
            url.get(1..url.len() - 1)?
        } else {
            url
        };
        if url.is_empty() {
            return None;
        }
        return Some(BackgroundImageValue::Url(url.to_string()));
    }

    // 尝试解析渐变函数
    if let Some(gradient) = parse_gradient(value) {
        return Some(BackgroundImageValue::Gradient(gradient));
    }

    None
}

/// CSS background-position 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundPositionValue {
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
    TwoValue(Box<BackgroundPositionValue>, Box<BackgroundPositionValue>),
}

/// 解析 CSS background-position 属性值。
///
/// 支持单个关键字、长度/百分比，以及两个值的组合（水平 垂直）。
pub fn parse_background_position(value: &str) -> Option<BackgroundPositionValue> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();

    // 先检查是否为两个值组合
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.len() == 2 {
        let first = parse_position_component(parts[0])?;
        let second = parse_position_component(parts[1])?;
        return Some(BackgroundPositionValue::TwoValue(Box::new(first), Box::new(second)));
    }

    // 单个关键字
    match lower.as_str() {
        "center" => return Some(BackgroundPositionValue::Center),
        "left" => return Some(BackgroundPositionValue::Left),
        "right" => return Some(BackgroundPositionValue::Right),
        "top" => return Some(BackgroundPositionValue::Top),
        "bottom" => return Some(BackgroundPositionValue::Bottom),
        _ => {}
    }

    // 单个百分比
    if lower.ends_with('%') {
        let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
        return Some(BackgroundPositionValue::Percent(pct));
    }

    // 单个长度值
    if let Some(LengthValue::Px(px)) = parse_length(&lower) {
        return Some(BackgroundPositionValue::Length(px as f32));
    }

    None
}

/// 解析 background-position 的单个分量。
fn parse_position_component(s: &str) -> Option<BackgroundPositionValue> {
    match s {
        "center" => Some(BackgroundPositionValue::Center),
        "left" => Some(BackgroundPositionValue::Left),
        "right" => Some(BackgroundPositionValue::Right),
        "top" => Some(BackgroundPositionValue::Top),
        "bottom" => Some(BackgroundPositionValue::Bottom),
        _ => {
            if s.ends_with('%') {
                let pct: f32 = s.trim_end_matches('%').parse().ok()?;
                Some(BackgroundPositionValue::Percent(pct))
            } else if let Some(LengthValue::Px(px)) = parse_length(s) {
                Some(BackgroundPositionValue::Length(px as f32))
            } else {
                None
            }
        }
    }
}

/// CSS background-repeat 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundRepeatValue {
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

/// 解析 CSS background-repeat 属性值。
pub fn parse_background_repeat(value: &str) -> Option<BackgroundRepeatValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "repeat" => Some(BackgroundRepeatValue::Repeat),
        "repeat-x" => Some(BackgroundRepeatValue::RepeatX),
        "repeat-y" => Some(BackgroundRepeatValue::RepeatY),
        "no-repeat" => Some(BackgroundRepeatValue::NoRepeat),
        "space" => Some(BackgroundRepeatValue::Space),
        "round" => Some(BackgroundRepeatValue::Round),
        _ => None,
    }
}

/// CSS background-size 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundSizeValue {
    /// auto（默认值）— 背景图片保持原始尺寸。
    Auto,
    /// cover — 缩放图片以完全覆盖容器。
    Cover,
    /// contain — 缩放图片以完整显示在容器内。
    Contain,
    /// 长度值（如 100px）。
    Length(f32),
    /// 百分比值（如 50%）。
    Percent(f32),
}

/// 解析 CSS background-size 属性值。
///
/// 支持关键字（auto、cover、contain）和带单位的长度/百分比值。
pub fn parse_background_size(value: &str) -> Option<BackgroundSizeValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(BackgroundSizeValue::Auto),
        "cover" => Some(BackgroundSizeValue::Cover),
        "contain" => Some(BackgroundSizeValue::Contain),
        _ => {
            if v.ends_with('%') {
                let pct: f32 = v.trim_end_matches('%').parse().ok()?;
                Some(BackgroundSizeValue::Percent(pct))
            } else if let Some(lv) = parse_length(&v) {
                match lv {
                    LengthValue::Px(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    LengthValue::Em(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    LengthValue::Rem(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

/// CSS background-attachment 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundAttachmentValue {
    /// scroll（默认值）— 背景随元素内容滚动。
    Scroll,
    /// fixed — 背景相对于视口固定。
    Fixed,
    /// local — 背景随元素本地内容滚动。
    Local,
}

/// 解析 CSS background-attachment 属性值。
pub fn parse_background_attachment(value: &str) -> Option<BackgroundAttachmentValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "scroll" => Some(BackgroundAttachmentValue::Scroll),
        "fixed" => Some(BackgroundAttachmentValue::Fixed),
        "local" => Some(BackgroundAttachmentValue::Local),
        _ => None,
    }
}

/// CSS background-clip 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundClipValue {
    /// border-box（默认值）— 背景绘制到边框区域外边界。
    BorderBox,
    /// padding-box — 背景绘制到内边距区域外边界。
    PaddingBox,
    /// content-box — 背景绘制到内容区域外边界。
    ContentBox,
    /// text — 背景绘制到文本区域内。
    Text,
}

/// 解析 CSS background-clip 属性值。
pub fn parse_background_clip(value: &str) -> Option<BackgroundClipValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "border-box" => Some(BackgroundClipValue::BorderBox),
        "padding-box" => Some(BackgroundClipValue::PaddingBox),
        "content-box" => Some(BackgroundClipValue::ContentBox),
        "text" => Some(BackgroundClipValue::Text),
        _ => None,
    }
}

/// CSS background-origin 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundOriginValue {
    /// padding-box（默认值）— 背景定位从内边距区域开始。
    PaddingBox,
    /// border-box — 背景定位从边框区域开始。
    BorderBox,
    /// content-box — 背景定位从内容区域开始。
    ContentBox,
}

/// 解析 CSS background-origin 属性值。
pub fn parse_background_origin(value: &str) -> Option<BackgroundOriginValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "padding-box" => Some(BackgroundOriginValue::PaddingBox),
        "border-box" => Some(BackgroundOriginValue::BorderBox),
        "content-box" => Some(BackgroundOriginValue::ContentBox),
        _ => None,
    }
}
