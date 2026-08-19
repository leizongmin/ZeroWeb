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
        let func_name = &value[..paren_pos];
        if func_name.is_empty() || func_name.chars().any(char::is_whitespace) {
            return None;
        }
        if !value.ends_with(')') {
            return None;
        }
        let inner = value[paren_pos + 1..value.len() - 1].trim();

        match func_name.to_ascii_lowercase().as_str() {
            "blur" => {
                let px: f32 = parse_filter_non_negative_length_px(inner)?;
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
    // CSS Filter Effects：`blur() = blur( <length>? )`——参数可选，缺省 = 0。
    if s.is_empty() {
        return Some(0.0);
    }
    if let Some(num_str) = s.strip_suffix("px") {
        let px = num_str.trim().parse::<f32>().ok()?;
        return px.is_finite().then_some(px);
    }
    // CSS Values §5.4：裸 0 是合法 `<length>`（unitless-zero）；其他无单位值对 `<length>` 无效。
    match s.parse::<f32>() {
        Ok(0.0) => Some(0.0),
        _ => None,
    }
}

fn parse_filter_non_negative_length_px(s: &str) -> Option<f32> {
    let px = parse_filter_length_px(s)?;
    if px.is_finite() && px >= 0.0 { Some(px) } else { None }
}

/// 解析 filter 函数中的数值（0-1 范围，也接受百分比和大于 1 的值）。
fn parse_filter_number(s: &str) -> Option<f32> {
    let s = s.trim();
    let n = if s.ends_with('%') {
        let pct: f32 = s.trim_end_matches('%').parse().ok()?;
        pct / 100.0
    } else {
        s.parse::<f32>().ok()?
    };
    if n.is_finite() && n >= 0.0 { Some(n) } else { None }
}

/// 解析 filter 函数中的角度值（返回度数）。单位大小写不敏感（CSS Values §），
/// 支持 deg/grad/turn/rad（grad 须先于 rad——"Xgrad" 后缀含 "rad"）。
fn parse_filter_angle(s: &str) -> Option<f32> {
    let lower = s.trim().to_ascii_lowercase();
    let degrees = if let Some(n) = lower.strip_suffix("deg") {
        n.trim().parse::<f32>().ok()?
    } else if let Some(n) = lower.strip_suffix("grad") {
        // 400grad = 360deg → 1grad = 0.9deg
        n.trim().parse::<f32>().ok()? * 0.9
    } else if let Some(n) = lower.strip_suffix("turn") {
        n.trim().parse::<f32>().ok()? * 360.0
    } else if let Some(n) = lower.strip_suffix("rad") {
        n.trim().parse::<f32>().ok()?.to_degrees()
    } else {
        lower.parse::<f32>().ok()?
    };
    degrees.is_finite().then_some(degrees)
}

/// 解析 drop-shadow 参数。
///
/// 格式：`x-offset y-offset blur-radius color` 或 `x-offset y-offset color`。
fn parse_drop_shadow(inner: &str) -> Option<FilterValue> {
    // R2485：CSS Filter Effects `drop-shadow( <length>{2,3} && <color>? )`。
    // 改前 bug：① 裸 `parts[0].parse::<f32>()` 对 `"2px"` 失败 → **所有 px drop-shadow 整条丢**
    //    （仅 unitless 数字能过——但非零 unitless 是非法 `<length>`）；② `split_whitespace` 拆散
    //    `rgb(0, 0, 0)` 含空格色 → 解析失败；③ `< 3` 值（即 2 长度无 color）被拒（应默认 currentcolor）。
    // 复用 box-shadow（R2477）模式：paren-aware split → 抽首个 color（任意位置）→ 剩余必须全为长度。
    let parts = split_top_level_whitespace(inner);
    if parts.len() < 2 {
        return None;
    }
    // 抽取首个可解析颜色（`<color>?` 可在任意位置，缺省 currentcolor）。
    let mut color = ColorValue::CurrentColor;
    let mut color_idx = None;
    for (i, p) in parts.iter().enumerate() {
        if let Some(c) = parse_color(p) {
            color = c;
            color_idx = Some(i);
            break;
        }
    }
    // 剩余 token（排除 color）必须全为长度，恰 2 或 3 个 → ox/oy(/blur)。
    let ci = color_idx.unwrap_or(usize::MAX);
    let lengths: Vec<&str> = parts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != ci)
        .map(|(_, s)| s.as_str())
        .collect();
    if !(lengths.len() == 2 || lengths.len() == 3) {
        return None;
    }
    let ox = parse_filter_length_px(lengths[0])?;
    let oy = parse_filter_length_px(lengths[1])?;
    let blur = if lengths.len() == 3 {
        parse_filter_non_negative_length_px(lengths[2])?
    } else {
        0.0
    };
    Some(FilterValue::DropShadow(ox, oy, blur, color))
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
    if (value.len() >= 4 && value[..4].eq_ignore_ascii_case("url(")) && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = parse_css_url_payload(inner)?;
        return Some(BackgroundImageValue::Url(url));
    }

    // 尝试解析渐变函数
    if let Some(gradient) = parse_gradient(value) {
        return Some(BackgroundImageValue::Gradient(gradient));
    }

    None
}

pub(crate) fn parse_css_url_payload(inner: &str) -> Option<String> {
    let url = inner.trim();
    if url.is_empty() {
        return None;
    }
    if url.starts_with('"') || url.starts_with('\'') {
        let quote = url.as_bytes()[0] as char;
        if !url.ends_with(quote) || url.len() < 2 {
            return None;
        }
        let value = url.get(1..url.len() - 1)?;
        return (!value.is_empty()).then(|| value.to_string());
    }
    if url.ends_with('"') || url.ends_with('\'') || contains_unescaped_url_delim(url) {
        return None;
    }
    Some(url.to_string())
}

fn contains_unescaped_url_delim(url: &str) -> bool {
    let mut escaped = false;
    for ch in url.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch.is_whitespace() || matches!(ch, '"' | '\'' | '(' | ')') {
            return true;
        }
    }
    false
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
    /// R2478：3/4 值语法「边缘 + 偏移」对（CSS Backgrounds §3.6）：偏移从命名边度量，
    /// 如 `right 25px`（距右边缘 25px）、`top 75%`。side ∈ {Left,Right,Top,Bottom}
    ///（center 不可带偏移）；offset ∈ {Length,Percent,Calc}（关键字/TwoValue 不可作 offset）。
    /// 解析为 TwoValue 的轴分量：EdgeOffset(Left/Right) → 水平轴、EdgeOffset(Top/Bottom) → 垂直轴。
    EdgeOffset {
        /// 度量偏移的参考边缘。
        side: BackgroundEdge,
        /// 偏移分量（length/percent/calc；right/bottom 边在 resolve 期翻转）。
        offset: Box<BackgroundPositionValue>,
    },
}

/// R2478：background-position / object-position 3/4 值语法的参考边缘（CSS Backgrounds §3.6）。
///
/// left/right = 水平轴；top/bottom = 垂直轴。带偏移时（`right 25px`），偏移从该边缘度量；
/// resolve 期 right/bottom 翻转（位置 = (container-image) - offset）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundEdge {
    /// 左边缘（偏移从左度量 = offset 本身）。
    Left,
    /// 右边缘（偏移从右度量 = (container-image) - offset）。
    Right,
    /// 上边缘（偏移从上度量 = offset 本身）。
    Top,
    /// 下边缘（偏移从下度量 = (container-image) - offset）。
    Bottom,
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
    // R2478：3/4 值语法（CSS Backgrounds §3.6）= 一个或两个「边缘+偏移」对（`left 50px center`、
    // `right 25px top 75%`）。改前 parts.len()∈{3,4} 落单值分支 → None → 声明丢。
    // R2478：3/4 值语法（CSS Backgrounds §3.6）= 一个或两个「边缘+偏移」对（`left 50px center`、
    // `right 25px top 75%`）。改前 parts.len()∈{3,4} 落单值分支 → None → 声明丢。
    if parts.len() == 3 || parts.len() == 4 {
        return parse_position_three_four(&parts);
    }
    if parts.len() == 2 {
        let first = parse_position_component(&parts[0])?;
        let second = parse_position_component(&parts[1])?;
        let first_horizontal = matches!(first, BackgroundPositionValue::Left | BackgroundPositionValue::Right);
        let first_vertical = matches!(first, BackgroundPositionValue::Top | BackgroundPositionValue::Bottom);
        let second_horizontal = matches!(second, BackgroundPositionValue::Left | BackgroundPositionValue::Right);
        let second_vertical = matches!(second, BackgroundPositionValue::Top | BackgroundPositionValue::Bottom);
        if (first_horizontal && second_horizontal) || (first_vertical && second_vertical) {
            return None;
        }
        // CSS background-position 两值语法（CSS Backgrounds §3.6）：关键字顺序无关——
        // 水平专属（left/right）→ x，垂直专属（top/bottom）→ y，center 兼容两轴。
        // 故须交换当：第一值是垂直专属（top/bottom），或第二值是水平专属（left/right）。
        // R508 仅覆盖前者；R2048 补后者——"center left" 应为 x=left/y=center，否则
        // resolve 把 Left 当 y 解析致 background-position-145/146 图像错位。
        let (x, y) = if first_vertical || second_horizontal {
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
        return pct.is_finite().then_some(BackgroundPositionValue::Percent(pct));
    }

    // 单个长度值（R1417：接受任意单位——px/em/rem/ex/vh/vw/ch 等；em/rem 等相对单位
    // 在 style-system apply 时解析为 px）。排除 auto/min-content/max-content/fit-content
    // （非 bg-position 合法长度）与百分比（已在上方处理）。
    if let Some(lv) = parse_length(&lower)
        && is_background_position_length(&lower, &lv)
    {
        return Some(BackgroundPositionValue::Length(lv));
    }

    None
}

/// 判断 LengthValue 是否为 background-position 合法的长度（px/em/rem/ex/vh/vw/vmin/vmax/ch）。
/// 排除 auto/min-content/max-content/fit-content/percentage/calc（非长度或已单独处理）。
fn is_background_position_length(raw: &str, lv: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match lv {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Ch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v) => v.is_finite(),
        _ => false,
    }
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
                pct.is_finite().then_some(BackgroundPositionValue::Percent(pct))
            } else if let Some(lv) = parse_length(s)
                && is_background_position_length(s, &lv)
            {
                Some(BackgroundPositionValue::Length(lv))
            } else {
                None
            }
        }
    }
}

/// R2478：解析 background-position 3/4 值语法（CSS Backgrounds §3.6）。
///
/// 文法：`([left|right] <lp>) && ([top|bottom] <lp>)`，3 值时缺一轴 → center。
/// 每个 `<lp>` 是从前导关键字命名的边缘度量的偏移（`right 25px` = 距右边缘 25px）。
///
/// 算法：左→右扫 token，若当前为边缘关键字（left/right/top/bottom，非 center）且下一 token
/// 可解析为 lp（length/percent/calc），则两者结成 EdgeOffset 对；否则当前 token 为裸关键字。
/// 按关键字的轴（left/right=水平，top/bottom=垂直，center=补缺轴）分配到 h/v；同轴重复 → None
///（非法）。driving：css-backgrounds/background-position-three-four-values（4 案）。
fn parse_position_three_four(parts: &[String]) -> Option<BackgroundPositionValue> {
    let mut h: Option<BackgroundPositionValue> = None; // 水平轴分量
    let mut v: Option<BackgroundPositionValue> = None; // 垂直轴分量

    let assign = |axis_slot: &mut Option<BackgroundPositionValue>, val: BackgroundPositionValue| -> Option<()> {
        if axis_slot.is_some() {
            None // 同轴已赋值 → 非法
        } else {
            *axis_slot = Some(val);
            Some(())
        }
    };

    let mut i = 0;
    while i < parts.len() {
        let tok = parts[i].as_str();
        if let Some(side) = parse_background_edge(tok) {
            // 边缘关键字 + 可能的偏移
            if i + 1 < parts.len()
                && let Some(offset) = parse_position_component(parts[i + 1].as_str())
                && is_offset_only(&offset)
            {
                // [side, offset] 对
                let edge = BackgroundPositionValue::EdgeOffset {
                    side,
                    offset: Box::new(offset),
                };
                match side {
                    BackgroundEdge::Left | BackgroundEdge::Right => assign(&mut h, edge)?,
                    BackgroundEdge::Top | BackgroundEdge::Bottom => assign(&mut v, edge)?,
                }
                i += 2;
                continue;
            }
            // 裸边缘关键字（无偏移）= 该轴的 0%/100%（left/top=0、right/bottom=100%）
            let kw = match side {
                BackgroundEdge::Left => BackgroundPositionValue::Left,
                BackgroundEdge::Right => BackgroundPositionValue::Right,
                BackgroundEdge::Top => BackgroundPositionValue::Top,
                BackgroundEdge::Bottom => BackgroundPositionValue::Bottom,
            };
            match side {
                BackgroundEdge::Left | BackgroundEdge::Right => assign(&mut h, kw)?,
                BackgroundEdge::Top | BackgroundEdge::Bottom => assign(&mut v, kw)?,
            }
            i += 1;
            continue;
        }
        if tok == "center" {
            // center 补缺轴（先 h 后 v）
            if h.is_none() {
                assign(&mut h, BackgroundPositionValue::Center)?;
            } else if v.is_none() {
                assign(&mut v, BackgroundPositionValue::Center)?;
            } else {
                return None; // 两轴已满
            }
            i += 1;
            continue;
        }
        // 3/4 值语境中裸 length/percent（无前导关键字）非法
        return None;
    }

    let h = h.unwrap_or(BackgroundPositionValue::Center);
    let v = v.unwrap_or(BackgroundPositionValue::Center);
    Some(BackgroundPositionValue::TwoValue(Box::new(h), Box::new(v)))
}

/// 解析 background-position 边缘关键字为 BackgroundEdge（left/right/top/bottom）。
/// center 不算边缘（不可带偏移）。
fn parse_background_edge(s: &str) -> Option<BackgroundEdge> {
    match s {
        "left" => Some(BackgroundEdge::Left),
        "right" => Some(BackgroundEdge::Right),
        "top" => Some(BackgroundEdge::Top),
        "bottom" => Some(BackgroundEdge::Bottom),
        _ => None,
    }
}

/// 判断分量是否可作 3/4 值语法的偏移（仅 length/percent/calc；关键字/TwoValue/EdgeOffset 不可）。
fn is_offset_only(v: &BackgroundPositionValue) -> bool {
    matches!(
        v,
        BackgroundPositionValue::Length(_) | BackgroundPositionValue::Percent(_) | BackgroundPositionValue::Calc(_)
    )
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
    /// 两值语法 `<w> <h>`（CSS Backgrounds §3.9），每维 auto/length/percent。
    /// driving：css-backgrounds background-size-013/025/041 等（`auto 100px`/`200px auto`）。
    TwoValue(BgSizeComponent, BgSizeComponent),
}

/// background-size 两值语法的单维分量。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BgSizeComponent {
    /// auto — 该维由另一维 + 固有比推导（无比则为定位区尺寸）。
    Auto,
    /// 长度（px）。
    Length(f32),
    /// 百分比（相对定位区该维）。
    Percent(f32),
}

/// 解析 CSS background-size 属性值。
///
/// 支持关键字（auto、cover、contain）、带单位的长度/百分比值，以及两值语法 `<w> <h>`。
pub fn parse_background_size(value: &str) -> Option<BackgroundSizeValue> {
    let v = value.trim().to_ascii_lowercase();
    // 两值语法（空格分隔恰好两 token，每 token = auto/length/percent）。
    // cover/contain 不允许组合，两 token 时若含 cover/contain → parse_bg_size_component 返 None。
    let tokens: Vec<&str> = v.split_whitespace().collect();
    if tokens.len() == 2 {
        let c1 = parse_bg_size_component(tokens[0])?;
        let c2 = parse_bg_size_component(tokens[1])?;
        return Some(BackgroundSizeValue::TwoValue(c1, c2));
    }
    match v.as_str() {
        "auto" => Some(BackgroundSizeValue::Auto),
        "cover" => Some(BackgroundSizeValue::Cover),
        "contain" => Some(BackgroundSizeValue::Contain),
        _ => {
            if v.ends_with('%') {
                let pct: f32 = v.trim_end_matches('%').parse().ok()?;
                if pct.is_finite() && pct >= 0.0 {
                    Some(BackgroundSizeValue::Percent(pct))
                } else {
                    None
                }
            } else if let Some(lv) = parse_length(&v) {
                bg_size_length_px(&v, &lv).map(BackgroundSizeValue::Length)
            } else {
                None
            }
        }
    }
}

/// 解析两值语法的单维分量：auto / <length> / <percentage>。
fn parse_bg_size_component(token: &str) -> Option<BgSizeComponent> {
    if token.eq_ignore_ascii_case("auto") {
        return Some(BgSizeComponent::Auto);
    }
    if token.ends_with('%') {
        let pct: f32 = token.trim_end_matches('%').parse().ok()?;
        return if pct.is_finite() && pct >= 0.0 {
            Some(BgSizeComponent::Percent(pct))
        } else {
            None
        };
    }
    let lv = parse_length(token)?;
    bg_size_length_px(token, &lv).map(BgSizeComponent::Length)
}

fn bg_size_length_px(raw: &str, value: &LengthValue) -> Option<f32> {
    if matches!(raw.trim().to_ascii_lowercase().as_str(), "thin" | "medium" | "thick") {
        return None;
    }
    match value {
        LengthValue::Px(n) | LengthValue::Em(n) | LengthValue::Rem(n) if n.is_finite() && *n >= 0.0 => Some(*n as f32),
        _ => None,
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
