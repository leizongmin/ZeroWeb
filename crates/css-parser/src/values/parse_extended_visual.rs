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

/// 按顶层空白分割（paren-aware：使 `drop-shadow(2px 4px red)` 等含空白参数的
/// 函数保持一体）。用于 filter 多函数列表（`<filter-function>+`，空格分隔）拆分，
/// 及渐变双位置色标（`red 0% 50%` / `red calc(10% + 5px) 80%`）的位置 token 拆分。
pub(crate) fn split_top_level_whitespace(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            c if c.is_whitespace() && depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() {
                    parts.push(t);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() {
        parts.push(t);
    }
    parts
}

/// 解析 filter 多函数列表（CSS Filter Effects：`none | <filter-function>+`）。
/// `none` → 空 Vec；否则 paren-aware 顶层空白分割后逐个 parse_filter（任一失败 → None）。
/// 多函数按声明顺序返回；render 侧 `FilterPrimitive.filters: Vec<FilterKind>` 已支持顺序应用。
pub fn parse_filter_list(value: &str) -> Option<Vec<FilterValue>> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let parts = split_top_level_whitespace(v);
    if parts.is_empty() {
        return None;
    }
    let mut filters = Vec::with_capacity(parts.len());
    for p in &parts {
        filters.push(parse_filter(p)?);
    }
    Some(filters)
}
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

/// 解析 CSS background-image 多图层值（逗号分隔）。
///
/// 支持格式如 `"url(a.png), linear-gradient(red, blue)"`。
/// 如果只有单个值，返回长度为 1 的 Vec。
/// 如果全部解析失败，返回 None。
pub fn parse_background_image_layers(value: &str) -> Option<Vec<BackgroundImageValue>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    // 快速路径：如果没有逗号，使用单值解析
    if !value.contains(',') {
        let single = parse_background_image(value)?;
        return Some(vec![single]);
    }

    // 多图层：按逗号分隔（注意不要拆分渐变函数内的逗号）
    let layers = split_background_layers(value);
    let mut result = Vec::with_capacity(layers.len());
    for layer in &layers {
        let v = parse_background_image(layer.trim())?;
        result.push(v);
    }
    if result.is_empty() { None } else { Some(result) }
}

/// 按顶层逗号分隔 background-image 值（跳过函数内的逗号）。
fn split_background_layers(value: &str) -> Vec<&str> {
    let mut layers = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                layers.push(&value[start..i]);
                start = i + 1; // 逗号是 1 字节 ASCII
            }
            _ => {}
        }
    }
    layers.push(&value[start..]);
    layers
}

// ── CSS Mask 值类型 ──────────────────────────────────────────────

/// CSS mask-mode 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum MaskModeValue {
    /// alpha — 使用 mask 图像的 alpha 通道。
    Alpha,
    /// luminance — 使用 mask 图像的亮度值。
    Luminance,
    /// match-source — 默认值，根据 mask 图像类型自动选择。
    MatchSource,
}

/// 解析 CSS mask-mode 属性值。
pub fn parse_mask_mode(value: &str) -> Option<MaskModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "alpha" => Some(MaskModeValue::Alpha),
        "luminance" => Some(MaskModeValue::Luminance),
        "match-source" => Some(MaskModeValue::MatchSource),
        _ => None,
    }
}

/// 解析 CSS mask-image 属性值。
///
/// 格式与 background-image 相同：`none`、`url(...)`、`linear-gradient(...)` 等。
/// 复用 BackgroundImageValue 类型。
pub fn parse_mask_image(value: &str) -> Option<BackgroundImageValue> {
    parse_background_image(value)
}

/// 解析 CSS mask-image 多图层值（逗号分隔）。
///
/// 与 parse_background_image_layers 相同的分隔逻辑。
pub fn parse_mask_image_layers(value: &str) -> Option<Vec<BackgroundImageValue>> {
    parse_background_image_layers(value)
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
    /// 长度值（任意单位，px/em/rem/ex/vh/vw 等；em/rem 等相对单位在 style-system apply
    /// 时按元素 font-size/viewport 解析为 px）。R1417：此前仅 px（Length(f32)），致
    /// `background-position: <em/rem/ex>` 解析失败（parse_length 返回 Em/Rem 而非 Px，
    /// `if let LengthValue::Px` 不匹配 → None）。
    Length(LengthValue),
    /// 百分比值（如 50%）。
    Percent(f32),
    /// calc()/min()/max()/clamp() 数学函数（CSS Values 4）。延迟到 paint 期解析——
    /// % 相对 (container-image)（与 Percent 同语义），由 eval_calc(parent_length=
    /// container-image) 求值。R2313：driving background-position-calc-minmax-001。
    Calc(crate::values::CalcExpr),
    /// 两个值组合（水平 垂直）。
    TwoValue(Box<BackgroundPositionValue>, Box<BackgroundPositionValue>),
}

/// 解析 CSS background-position 属性值。
///
/// 支持单个关键字、长度/百分比，以及两个值的组合（水平 垂直）。
pub fn parse_background_position(value: &str) -> Option<BackgroundPositionValue> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();

    // 先检查是否为两个值组合（R2313：split_top_level_whitespace paren-aware，
    // 使 calc/min/max 含空白参数的函数保持一体，如 `min(0%, 100%) max(0%, 100%)`）。
    let parts = split_top_level_whitespace(&lower);
    if parts.len() == 2 {
        let first = parse_position_component(&parts[0])?;
        let second = parse_position_component(&parts[1])?;
        // CSS background-position 两值语法（CSS Backgrounds §3.6）：关键字顺序无关——
        // 水平专属（left/right）→ x，垂直专属（top/bottom）→ y，center 兼容两轴。
        // 故须交换当：第一值是垂直专属（top/bottom），或第二值是水平专属（left/right）。
        // R508 仅覆盖前者；R2048 补后者——"center left" 应为 x=left/y=center，否则
        // resolve 把 Left 当 y 解析致 background-position-145/146 图像错位。
        let (x, y) = if matches!(first, BackgroundPositionValue::Top | BackgroundPositionValue::Bottom)
            || matches!(second, BackgroundPositionValue::Left | BackgroundPositionValue::Right)
        {
            (second, first)
        } else {
            (first, second)
        };
        return Some(BackgroundPositionValue::TwoValue(Box::new(x), Box::new(y)));
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

    // R2313：calc()/min()/max()/clamp() 单值数学函数 → Calc（延迟到 paint 期按
    // (container-image) 解析 %）。整条是一个数学函数（2 值情形由上方 2-value 分支处理）。
    if ["calc(", "min(", "max(", "clamp("].iter().any(|p| lower.starts_with(p)) {
        return crate::values::parse_math_function(&lower).map(BackgroundPositionValue::Calc);
    }

    // 单个百分比
    if lower.ends_with('%') {
        let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
        return Some(BackgroundPositionValue::Percent(pct));
    }

    // 单个长度值（R1417：接受任意单位——px/em/rem/ex/vh/vw/ch 等；em/rem 等相对单位
    // 在 style-system apply 时解析为 px）。排除 auto/min-content/max-content/fit-content
    // （非 bg-position 合法长度）与百分比（已在上方处理）。
    if let Some(lv) = parse_length(&lower)
        && is_background_position_length(&lv)
    {
        return Some(BackgroundPositionValue::Length(lv));
    }

    None
}

/// 判断 LengthValue 是否为 background-position 合法的长度（px/em/rem/ex/vh/vw/vmin/vmax/ch）。
/// 排除 auto/min-content/max-content/fit-content/percentage/calc（非长度或已单独处理）。
fn is_background_position_length(lv: &LengthValue) -> bool {
    matches!(
        lv,
        LengthValue::Px(_)
            | LengthValue::Em(_)
            | LengthValue::Rem(_)
            | LengthValue::Vh(_)
            | LengthValue::Vw(_)
            | LengthValue::Vmin(_)
            | LengthValue::Vmax(_)
            | LengthValue::Ch(_)
    )
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
            // R2313：calc/min/max/clamp 单分量数学函数 → Calc。
            let t = s.trim();
            if ["calc(", "min(", "max(", "clamp("].iter().any(|p| t.starts_with(p)) {
                return crate::values::parse_math_function(t).map(BackgroundPositionValue::Calc);
            }
            if s.ends_with('%') {
                let pct: f32 = s.trim_end_matches('%').parse().ok()?;
                Some(BackgroundPositionValue::Percent(pct))
            } else if let Some(lv) = parse_length(s)
                && is_background_position_length(&lv)
            {
                Some(BackgroundPositionValue::Length(lv))
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

/// 解析 background-position 多层列表（CSS Backgrounds §3.6 `<position>#`，顶层逗号分隔）。
///
/// 单层内可含空格（如 `center top` = 单个 2 值 position），逗号才是图层分隔符。
/// R2311：单层 byte-identical（1 项 Vec）；多层改变存储。任一层失败 → None；空输入 → None。
/// position/size/repeat 值均不含括号，故顶层逗号分割用简单 `split(',')`（无需 paren-aware）。
pub fn parse_background_position_list(value: &str) -> Option<Vec<BackgroundPositionValue>> {
    let mut list = Vec::new();
    for part in value.split(',') {
        list.push(parse_background_position(part)?);
    }
    (!list.is_empty()).then_some(list)
}

/// 解析 background-repeat 多层列表（CSS Backgrounds §3.4 `<repeat-style>#`，顶层逗号分隔）。
/// 单层 byte-identical；多层改变存储。任一层失败 → None；空输入 → None。
pub fn parse_background_repeat_list(value: &str) -> Option<Vec<BackgroundRepeatValue>> {
    let mut list = Vec::new();
    for part in value.split(',') {
        list.push(parse_background_repeat(part)?);
    }
    (!list.is_empty()).then_some(list)
}

/// 解析 background-size 多层列表（CSS Backgrounds §3.5 `<bg-size>#`，顶层逗号分隔）。
/// 单层内可含空格（如 `50% auto` = 单个 2 值 size）。单层 byte-identical；多层改变存储。
/// 任一层失败 → None；空输入 → None。
pub fn parse_background_size_list(value: &str) -> Option<Vec<BackgroundSizeValue>> {
    let mut list = Vec::new();
    for part in value.split(',') {
        list.push(parse_background_size(part)?);
    }
    (!list.is_empty()).then_some(list)
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
