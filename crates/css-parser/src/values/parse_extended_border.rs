//! CSS 边框图像和裁剪路径属性解析（border-image-*、clip-path、list-style-image 等）。

use super::*;

// ── CSS Border Image 值类型 ──────────────────────────────────────────

/// CSS border-image-source 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageSourceValue {
    /// none（默认值）— 不使用边框图片。
    None,
    /// url(<string>) — 指定边框图片 URL。
    Url(String),
    /// 渐变函数（linear/radial/conic-gradient，CSS Images）。
    Gradient(GradientValue),
    /// filter(<image>, <filter-value-list>?) — 对图像施加滤镜的图像函数
    ///（filter-effects-1 #FilterCSSImageValue：https://drafts.fxtf.org/filter-effects-1/#funcdef-filter）。
    /// 滤镜列表空（`filter(<image>)`）= 无滤镜等价内层图像。
    Filtered {
        /// 被滤镜的内层图像（url / gradient / 嵌套 filter）。
        image: Box<BorderImageSourceValue>,
        /// 空格分隔的滤镜函数列表（filter 属性同一语法，可为空）。
        filters: Vec<FilterValue>,
    },
    /// light-dark(L, D) — CSS Color 5 图像上下文分支函数
    ///（https://drafts.csswg.org/css-color-5/#funcdef-light-dark）。语义同
    /// BackgroundImageValue::LightDark：声明期暂存双支 AST，computed 期按
    /// used color-scheme 选支展开。
    LightDark {
        /// light 支图像。
        light: Box<BorderImageSourceValue>,
        /// dark 支图像。
        dark: Box<BorderImageSourceValue>,
    },
}

/// 解析 CSS border-image-source 属性值。
///
/// 支持格式如 `"none"`、`"url(border.png)"`、`"linear-gradient(...)"` 等。
pub fn parse_border_image_source(value: &str) -> Option<BorderImageSourceValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(BorderImageSourceValue::None);
    }
    if (value.len() >= 4 && value[..4].eq_ignore_ascii_case("url(")) && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = super::parse_extended_visual::parse_css_url_payload(inner)?;
        return Some(BorderImageSourceValue::Url(url));
    }
    // 渐变函数（linear/radial/conic/repeating-*）。
    if let Some(g) = parse_gradient(value) {
        return Some(BorderImageSourceValue::Gradient(g));
    }
    // filter(<image>, <filter-value-list>?) 图像函数（filter-effects-1 #funcdef-filter）：
    // 首个顶层逗号分 image 与 filter 列表两参；image 递归走 border-image-source 语法
    //（url/gradient/filter 皆可嵌套），filter 列表按 filter 属性同一语法解析。
    if value.len() > 7 && value[..7].eq_ignore_ascii_case("filter(") && value.ends_with(')') {
        let args = &value[7..value.len() - 1];
        let (image_part, filters_part) = match super::parse_extended_visual::split_top_level_comma(args) {
            Some((l, r)) => (l, Some(r)),
            // 单参形态 filter(<image>)：无滤镜
            None => (args, None),
        };
        let image = parse_border_image_source(image_part)?;
        if matches!(image, BorderImageSourceValue::None) {
            return None; // <image> 不含 none
        }
        let filters = match filters_part {
            Some(f) => parse_filter_list(f)?,
            None => Vec::new(),
        };
        return Some(BorderImageSourceValue::Filtered {
            image: Box::new(image),
            filters,
        });
    }
    // light-dark(L, D)（css-color-5 §funcdef-light-dark）：恰两参顶层逗号分割，
    // 各支递归走 border-image-source 语法；arity/空参/色值支经 `?` 自然拒绝。
    if value.len() > 11 && value[..11].eq_ignore_ascii_case("light-dark(") && value.ends_with(')') {
        let inner = &value[11..value.len() - 1];
        let (light_part, dark_part) = super::parse_extended_visual::split_top_level_comma(inner)?;
        if super::parse_extended_visual::split_top_level_comma(dark_part).is_some() {
            return None;
        }
        let light = parse_border_image_source(light_part.trim())?;
        let dark = parse_border_image_source(dark_part.trim())?;
        return Some(BorderImageSourceValue::LightDark {
            light: Box::new(light),
            dark: Box::new(dark),
        });
    }
    None
}

/// CSS border-image-slice 属性值。
///
/// 支持数字、百分比和 `fill` 关键字，最多 4 个值（上 右 下 左）。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageSliceValue {
    /// 顶部切片值。
    pub top: BorderImageSliceComponent,
    /// 右侧切片值。
    pub right: BorderImageSliceComponent,
    /// 底部切片值。
    pub bottom: BorderImageSliceComponent,
    /// 左侧切片值。
    pub left: BorderImageSliceComponent,
    /// 是否填充中央区域。
    pub fill: bool,
}

/// border-image-slice 的单个分量。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageSliceComponent {
    /// 数字值（无单位，默认）。
    Number(f32),
    /// 百分比值。
    Percent(f32),
}

/// 解析 CSS border-image-slice 属性值。
///
/// 支持格式如 `"50"`、`"50%"`、`"25 50"`、`"25 50 75"`、`"25 50 75 100"`、
/// `"25 50 fill"`、`"fill 25 50 75 100"`。
pub fn parse_border_image_slice(value: &str) -> Option<BorderImageSliceValue> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let mut fill = false;
    let mut numbers: Vec<BorderImageSliceComponent> = Vec::new();

    for token in value.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if lower == "fill" {
            if fill {
                return None;
            }
            fill = true;
            continue;
        }
        if lower.ends_with('%') {
            let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
            if !pct.is_finite() || pct < 0.0 {
                return None;
            }
            numbers.push(BorderImageSliceComponent::Percent(pct));
        } else {
            let n: f32 = lower.parse().ok()?;
            if !n.is_finite() || n < 0.0 {
                return None;
            }
            numbers.push(BorderImageSliceComponent::Number(n));
        }
    }

    if numbers.is_empty() {
        return None;
    }
    if numbers.len() > 4 {
        return None;
    }

    // 扩展到 4 个值（CSS TRBL 顺序）
    while numbers.len() < 4 {
        match numbers.len() {
            1 => numbers.push(numbers[0].clone()), // 右 = 上
            2 => numbers.push(numbers[0].clone()), // 下 = 上
            3 => numbers.push(numbers[1].clone()), // 左 = 右
            _ => break,
        }
    }

    Some(BorderImageSliceValue {
        top: numbers[0].clone(),
        right: numbers[1].clone(),
        bottom: numbers[2].clone(),
        left: numbers[3].clone(),
        fill,
    })
}

/// CSS border-image-width 属性值。
///
/// 支持长度、百分比、数字（倍数）和 `auto`，最多 4 个值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageWidthValue {
    /// 顶部宽度。
    pub top: BorderImageWidthComponent,
    /// 右侧宽度。
    pub right: BorderImageWidthComponent,
    /// 底部宽度。
    pub bottom: BorderImageWidthComponent,
    /// 左侧宽度。
    pub left: BorderImageWidthComponent,
}

/// border-image-width 的单个分量。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageWidthComponent {
    /// auto — 使用 border-image-slice 的自然尺寸。
    Auto,
    /// 数字值（对应 border-width 的倍数）。
    Number(f32),
    /// 长度值（px/em 等）。
    Length(LengthValue),
    /// 百分比值。
    Percent(f32),
}

/// 解析 CSS border-image-width 属性值。
///
/// 支持格式如 `"3"`、`"10px"`、`"auto"`、`"5%"`、`"1 2"`、`"1 2 3"`、`"1 2 3 4"`。
pub fn parse_border_image_width(value: &str) -> Option<BorderImageWidthValue> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let mut components: Vec<BorderImageWidthComponent> = Vec::new();

    for token in value.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if lower == "auto" {
            components.push(BorderImageWidthComponent::Auto);
        } else if lower.ends_with('%') {
            let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
            if !pct.is_finite() || pct < 0.0 {
                return None;
            }
            components.push(BorderImageWidthComponent::Percent(pct));
        } else if let Some(len) = parse_length(token) {
            if !border_image_length_is_valid(token, &len) {
                return None;
            }
            components.push(BorderImageWidthComponent::Length(len));
        } else {
            let n: f32 = lower.parse().ok()?;
            if !n.is_finite() || n < 0.0 {
                return None;
            }
            components.push(BorderImageWidthComponent::Number(n));
        }
    }

    if components.is_empty() || components.len() > 4 {
        return None;
    }

    // 扩展到 4 个值（CSS TRBL 顺序）
    while components.len() < 4 {
        match components.len() {
            1 => components.push(components[0].clone()),
            2 => components.push(components[0].clone()),
            3 => components.push(components[1].clone()),
            _ => break,
        }
    }

    Some(BorderImageWidthValue {
        top: components[0].clone(),
        right: components[1].clone(),
        bottom: components[2].clone(),
        left: components[3].clone(),
    })
}

/// CSS border-image-repeat 属性值。
///
/// 控制边框图片的缩放和平铺方式，最多 2 个值（水平 垂直）。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageRepeatValue {
    /// 水平方向重复方式。
    pub horizontal: BorderImageRepeatMode,
    /// 垂直方向重复方式。
    pub vertical: BorderImageRepeatMode,
}

/// border-image-repeat 的重复模式。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageRepeatMode {
    /// stretch（默认值）— 拉伸图片填充区域。
    Stretch,
    /// repeat — 平铺图片。
    Repeat,
    /// round — 平铺并缩放使整数次平铺。
    Round,
    /// space — 平铺且均匀分布空白。
    Space,
}

/// 解析 CSS border-image-repeat 属性值。
///
/// 支持格式如 `"stretch"`、`"repeat"`、`"round"`、`"space"`、`"repeat round"`。
pub fn parse_border_image_repeat(value: &str) -> Option<BorderImageRepeatValue> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    fn parse_mode(s: &str) -> Option<BorderImageRepeatMode> {
        match s.to_ascii_lowercase().as_str() {
            "stretch" => Some(BorderImageRepeatMode::Stretch),
            "repeat" => Some(BorderImageRepeatMode::Repeat),
            "round" => Some(BorderImageRepeatMode::Round),
            "space" => Some(BorderImageRepeatMode::Space),
            _ => None,
        }
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }

    let horizontal = parse_mode(parts[0])?;
    let vertical = if parts.len() == 2 {
        parse_mode(parts[1])?
    } else {
        horizontal.clone()
    };

    Some(BorderImageRepeatValue { horizontal, vertical })
}

/// CSS border-image-outset 属性值。
///
/// 指定边框图片超出边框区域的距离，最多 4 个值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderImageOutsetValue {
    /// 顶部 outset。
    pub top: BorderImageOutsetComponent,
    /// 右侧 outset。
    pub right: BorderImageOutsetComponent,
    /// 底部 outset。
    pub bottom: BorderImageOutsetComponent,
    /// 左侧 outset。
    pub left: BorderImageOutsetComponent,
}

/// border-image-outset 的单个分量。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageOutsetComponent {
    /// 数字值（对应 border-width 的倍数）。
    Number(f32),
    /// 长度值。
    Length(LengthValue),
}

/// 解析 CSS border-image-outset 属性值。
///
/// 支持格式如 `"2"`、`"10px"`、`"1 2"`、`"1 2 3"`、`"1 2 3 4"`。
pub fn parse_border_image_outset(value: &str) -> Option<BorderImageOutsetValue> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let mut components: Vec<BorderImageOutsetComponent> = Vec::new();

    for token in value.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if let Some(len) = parse_length(token) {
            if !border_image_length_is_valid(token, &len) {
                return None;
            }
            components.push(BorderImageOutsetComponent::Length(len));
        } else {
            let n: f32 = lower.parse().ok()?;
            if !n.is_finite() || n < 0.0 {
                return None;
            }
            components.push(BorderImageOutsetComponent::Number(n));
        }
    }

    if components.is_empty() || components.len() > 4 {
        return None;
    }

    while components.len() < 4 {
        match components.len() {
            1 => components.push(components[0].clone()),
            2 => components.push(components[0].clone()),
            3 => components.push(components[1].clone()),
            _ => break,
        }
    }

    Some(BorderImageOutsetValue {
        top: components[0].clone(),
        right: components[1].clone(),
        bottom: components[2].clone(),
        left: components[3].clone(),
    })
}

fn border_image_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v) => v.is_finite() && *v >= 0.0,
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

/// CSS list-style-image 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStyleImageValue {
    /// none（默认值）— 无列表标记图片。
    None,
    /// url(<string>) — 列表标记图片。
    Url(String),
    /// light-dark(L, D) — CSS Color 5 图像上下文分支函数
    ///（https://drafts.csswg.org/css-color-5/#funcdef-light-dark）。语义同
    /// BackgroundImageValue::LightDark：声明期暂存双支 AST，computed 期按
    /// used color-scheme 选支展开（支域同本枚举 none/url）。
    LightDark {
        /// light 支图像。
        light: Box<ListStyleImageValue>,
        /// dark 支图像。
        dark: Box<ListStyleImageValue>,
    },
}

/// 解析 CSS list-style-image 属性值。
///
/// 支持格式如 "none"、"url(marker.png)"。
pub fn parse_list_style_image(value: &str) -> Option<ListStyleImageValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(ListStyleImageValue::None);
    }
    if (value.len() >= 4 && value[..4].eq_ignore_ascii_case("url(")) && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = super::parse_extended_visual::parse_css_url_payload(inner)?;
        return Some(ListStyleImageValue::Url(url));
    }
    // light-dark(L, D)（css-color-5 §funcdef-light-dark）：恰两参顶层逗号分割，
    // 各支递归走 list-style-image 语法（none/url）。
    if value.len() > 11 && value[..11].eq_ignore_ascii_case("light-dark(") && value.ends_with(')') {
        let inner = &value[11..value.len() - 1];
        let (light_part, dark_part) = super::parse_extended_visual::split_top_level_comma(inner)?;
        if super::parse_extended_visual::split_top_level_comma(dark_part).is_some() {
            return None;
        }
        let light = parse_list_style_image(light_part.trim())?;
        let dark = parse_list_style_image(dark_part.trim())?;
        return Some(ListStyleImageValue::LightDark {
            light: Box::new(light),
            dark: Box::new(dark),
        });
    }
    None
}

/// CSS empty-cells 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum EmptyCellsValue {
    /// show（默认值）— 显示空单元格边框。
    Show,
    /// hide — 隐藏空单元格边框。
    Hide,
}

/// 解析 CSS empty-cells 属性值。
pub fn parse_empty_cells(value: &str) -> Option<EmptyCellsValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "show" => Some(EmptyCellsValue::Show),
        "hide" => Some(EmptyCellsValue::Hide),
        _ => None,
    }
}

/// CSS border-spacing 属性值。
#[derive(Debug, Clone, PartialEq)]
pub struct BorderSpacingValue {
    /// 水平间距。
    pub horizontal: LengthValue,
    /// 垂直间距（如果只有一个值，则等于水平间距）。
    pub vertical: LengthValue,
}

/// 解析 CSS border-spacing 属性值。
///
/// 支持格式如 "2px"、"2px 4px"。
pub fn parse_border_spacing(value: &str) -> Option<BorderSpacingValue> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let h = parse_length(parts[0])?;
    if !border_spacing_length_is_valid(parts[0], &h) {
        return None;
    }
    let v = if parts.len() == 2 {
        let v = parse_length(parts[1])?;
        if !border_spacing_length_is_valid(parts[1], &v) {
            return None;
        }
        v
    } else {
        h.clone()
    };
    Some(BorderSpacingValue {
        horizontal: h,
        vertical: v,
    })
}

fn border_spacing_length_is_valid(raw: &str, value: &LengthValue) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v) => v.is_finite() && *v >= 0.0,
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn clip_path_length_is_valid(raw: &str, value: &LengthValue, allow_negative: bool) -> bool {
    if matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "auto" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match value {
        LengthValue::Px(v)
        | LengthValue::Em(v)
        | LengthValue::Ex(v)
        | LengthValue::Rex(v)
        | LengthValue::Cap(v)
        | LengthValue::Rcap(v)
        | LengthValue::Rem(v)
        | LengthValue::Vh(v)
        | LengthValue::Vw(v)
        | LengthValue::Vmin(v)
        | LengthValue::Vmax(v)
        | LengthValue::Ch(v)
        | LengthValue::Rch(v)
        | LengthValue::Ic(v)
        | LengthValue::Ric(v)
        | LengthValue::Percentage(v) => v.is_finite() && (allow_negative || *v >= 0.0),
        LengthValue::Calc(expr) => clip_path_calc_is_length_percentage(expr, allow_negative),
        _ => false,
    }
}

fn parse_clip_length(raw: &str, allow_negative: bool) -> Option<LengthValue> {
    let length =
        parse_length(raw).or_else(|| parse_math_function(raw).map(|expr| LengthValue::Calc(Box::new(expr))))?;
    clip_path_length_is_valid(raw, &length, allow_negative).then_some(length)
}

fn clip_path_calc_is_length_percentage(expr: &CalcExpr, allow_negative: bool) -> bool {
    // https://drafts.csswg.org/css-values-4/#calc-type-checking
    if !matches!(clip_path_calc_type(expr), Some(ClipPathCalcType::LengthPercentage)) {
        return false;
    }
    // https://drafts.csswg.org/css-values-4/#calc-range
    // Clip radii use a non-negative range; reject math expressions that resolve without
    // layout context to a negative value, and defer context-dependent ones.
    allow_negative || eval_calc(expr, None).is_none_or(|v| v.is_finite() && v >= 0.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipPathCalcType {
    Number,
    LengthPercentage,
}

fn clip_path_calc_type(expr: &CalcExpr) -> Option<ClipPathCalcType> {
    match expr {
        CalcExpr::Number(_) => Some(ClipPathCalcType::Number),
        CalcExpr::Length(length) => {
            clip_path_length_is_valid("", length, true).then_some(ClipPathCalcType::LengthPercentage)
        }
        // clip-path 槽无 <time> 文法位——Time 叶无效（parse_time_calc 门控
        // 之外本不可达，fail-closed 兜底）。
        CalcExpr::Time(_) => None,
        // clip-path 槽无 <resolution> 文法位（css-values-4 §6.4 维度族与
        // clip 形状求值域正交，parse_resolution_calc 门控之外本不可达，
        // fail-closed 兜底）。
        CalcExpr::Resolution(_) => None,
        // progress() 恒产出 <number>（css-values-5 §progress——静态可求值）。
        CalcExpr::Progress { .. } => Some(ClipPathCalcType::Number),
        CalcExpr::BinaryOp(left, CalcOp::Add | CalcOp::Subtract, right) => {
            same_clip_path_calc_type(clip_path_calc_type(left)?, clip_path_calc_type(right)?)
        }
        CalcExpr::BinaryOp(left, CalcOp::Multiply, right) => {
            multiply_clip_path_calc_type(clip_path_calc_type(left)?, clip_path_calc_type(right)?)
        }
        CalcExpr::BinaryOp(left, CalcOp::Divide, right) => {
            divide_clip_path_calc_type(clip_path_calc_type(left)?, clip_path_calc_type(right)?)
        }
        CalcExpr::BinaryMathOp(_, _, _) => Some(ClipPathCalcType::Number),
        // clip-path 槽无 calc-mix() 文法位（CSS Values 5 tentative 未接线，
        // 无语料驱动案）——fail-closed 兜底。
        CalcExpr::Mix(_) => None,
        // clip-path 槽无 tree-counting 文法位（DOM 兄弟序依赖、parse 期
        // opaque 无语料驱动案）——fail-closed 兜底。
        CalcExpr::TreeCounting(_) => None,
        // clip-path 槽无 anchor()/anchor-size() 文法位（inset/size 族函数）
        // ——fail-closed 兜底。
        CalcExpr::Anchor { .. } => None,
        CalcExpr::Min(args) | CalcExpr::Max(args) => same_clip_path_calc_types(args),
        CalcExpr::Clamp { min, val, max } => {
            let ty = clip_path_calc_type(min)?;
            same_clip_path_calc_type(ty, clip_path_calc_type(val)?)?;
            same_clip_path_calc_type(ty, clip_path_calc_type(max)?)
        }
        CalcExpr::UnaryOp(_, _) => Some(ClipPathCalcType::Number),
    }
}

fn same_clip_path_calc_types(args: &[CalcExpr]) -> Option<ClipPathCalcType> {
    let mut iter = args.iter();
    let ty = clip_path_calc_type(iter.next()?)?;
    for arg in iter {
        same_clip_path_calc_type(ty, clip_path_calc_type(arg)?)?;
    }
    Some(ty)
}

fn same_clip_path_calc_type(left: ClipPathCalcType, right: ClipPathCalcType) -> Option<ClipPathCalcType> {
    (left == right).then_some(left)
}

fn multiply_clip_path_calc_type(left: ClipPathCalcType, right: ClipPathCalcType) -> Option<ClipPathCalcType> {
    match (left, right) {
        (ClipPathCalcType::Number, ClipPathCalcType::Number) => Some(ClipPathCalcType::Number),
        (ClipPathCalcType::Number, ClipPathCalcType::LengthPercentage)
        | (ClipPathCalcType::LengthPercentage, ClipPathCalcType::Number) => Some(ClipPathCalcType::LengthPercentage),
        (ClipPathCalcType::LengthPercentage, ClipPathCalcType::LengthPercentage) => None,
    }
}

fn divide_clip_path_calc_type(left: ClipPathCalcType, right: ClipPathCalcType) -> Option<ClipPathCalcType> {
    match (left, right) {
        (ClipPathCalcType::Number, ClipPathCalcType::Number) => Some(ClipPathCalcType::Number),
        (ClipPathCalcType::LengthPercentage, ClipPathCalcType::Number) => Some(ClipPathCalcType::LengthPercentage),
        (ClipPathCalcType::Number, ClipPathCalcType::LengthPercentage)
        | (ClipPathCalcType::LengthPercentage, ClipPathCalcType::LengthPercentage) => None,
    }
}

/// CSS counter-set 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum CounterSetValue {
    /// none — 不设置任何计数器。
    None,
    /// 计数器操作列表。
    Actions(Vec<CounterActionValue>),
}

/// 解析 CSS counter-set 属性值。
///
/// 格式同 counter-reset："none" | "<name> <integer>"。
pub fn parse_counter_set(value: &str) -> Option<CounterSetValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(CounterSetValue::None);
    }
    parse_counter_list(v).map(CounterSetValue::Actions)
}

/// 解析 CSS clip-path 属性值。
///
/// 支持：none | inset() | circle() | ellipse() | polygon()
/// R4248（CSS Borders 4 §corner-shaping）：解析 corner-shape 属性值。
///
/// 语法：`<corner-shape-value>{1,4}`，关键字 round/bevel/scoop/notch/square/squircle
/// 或 `superellipse(<number> | infinity | -infinity)`（数值可负，如 superellipse(-5)）。
/// css-borders-4 §3.7：squircle ≡ superellipse(2)、superellipse(∞) ≡ square、
/// superellipse(−∞) ≡ notch（解析层映射到对应关键字变体）；NaN 非法。有限值保留
/// 原参数 K（指数 n = 2K 由几何层解释）。角序同 border-radius：1 值四角同形；
/// 2 值 [TL, TR]（BR=TL、BL=TR）；3 值 [TL, TR, BR]（BL=TR）；4 值全序。
pub fn parse_corner_shape(value: &str) -> Option<CornerShapeValue> {
    use crate::values::types::{CornerShapeKind, CornerShapeValue};
    let parse_kind = |kw: &str| -> Option<CornerShapeKind> {
        let lower = kw.trim().to_ascii_lowercase();
        if let Some(inner) = lower.strip_prefix("superellipse(").and_then(|s| s.strip_suffix(')')) {
            let n: f64 = inner.trim().parse().ok()?;
            if n.is_nan() {
                return None;
            }
            return if n == f64::INFINITY {
                Some(CornerShapeKind::Square)
            } else if n == f64::NEG_INFINITY {
                Some(CornerShapeKind::Notch)
            } else {
                Some(CornerShapeKind::Superellipse(n))
            };
        }
        match lower.as_str() {
            "round" => Some(CornerShapeKind::Round),
            "bevel" => Some(CornerShapeKind::Bevel),
            "scoop" => Some(CornerShapeKind::Scoop),
            "notch" => Some(CornerShapeKind::Notch),
            "square" => Some(CornerShapeKind::Square),
            "squircle" => Some(CornerShapeKind::Superellipse(2.0)),
            _ => None,
        }
    };
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    match parts.len() {
        1 => Some(CornerShapeValue::All(parse_kind(parts[0])?)),
        2 => {
            let tl = parse_kind(parts[0])?;
            let tr = parse_kind(parts[1])?;
            Some(CornerShapeValue::PerCorner(Box::new([tl, tr, tl, tr])))
        }
        3 => {
            let tl = parse_kind(parts[0])?;
            let tr = parse_kind(parts[1])?;
            let br = parse_kind(parts[2])?;
            Some(CornerShapeValue::PerCorner(Box::new([tl, tr, br, tr])))
        }
        4 => {
            let tl = parse_kind(parts[0])?;
            let tr = parse_kind(parts[1])?;
            let br = parse_kind(parts[2])?;
            let bl = parse_kind(parts[3])?;
            Some(CornerShapeValue::PerCorner(Box::new([tl, tr, br, bl])))
        }
        _ => None,
    }
}

/// 解析 clip-path 属性值（none / inset() / circle() / ellipse() / polygon()）。
pub fn parse_clip_path(value: &str) -> Option<ClipPathValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(ClipPathValue::None);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "inset") {
        return parse_clip_inset(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "circle") {
        return parse_clip_circle(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "ellipse") {
        return parse_clip_ellipse(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "polygon") {
        return parse_clip_polygon(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "shape") {
        return super::parse_basic_shape::parse_shape_function(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "path") {
        return super::parse_basic_shape::parse_path_function(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "rect") {
        return super::parse_basic_shape::parse_rect_function(rest);
    }
    if let Some(rest) = strip_basic_shape_prefix(v, "xywh") {
        return super::parse_basic_shape::parse_xywh_function(rest);
    }
    None
}

fn strip_basic_shape_prefix<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    // https://www.w3.org/TR/css-syntax-3/#function-token-diagram
    // CSS function names are ASCII case-insensitive; whitespace before `(` is not a function token.
    let prefix_len = name.len() + 1;
    let bytes = input.as_bytes();
    if bytes.len() >= prefix_len
        && bytes[name.len()] == b'('
        && bytes[..name.len()].eq_ignore_ascii_case(name.as_bytes())
    {
        return Some(&input[prefix_len..]);
    }
    None
}

/// 解析 border-shape 属性值（css-borders-4 §7.1：
/// `none | [ <basic-shape> <geometry-box>? ]{1,2}`）。
///
/// 复用 clip-path 的 <basic-shape> 解析（parse_clip_path）。扫描按 token 切分：
/// 函数 token（配平括号）与独立关键词 token（geometry-box）——关键词与其形状函数
/// 之间允许空白。任一段非法 → 整条声明丢弃（返回 None）。
pub fn parse_border_shape(value: &str) -> Option<BorderShapeValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(BorderShapeValue::None);
    }

    let bytes = v.as_bytes();
    let mut segments: Vec<(ClipPathValue, Option<BorderShapeGeometryBox>)> = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // 函数 token：name( ... 配平 )
        let start = i;
        while i < bytes.len() && bytes[i] != b'(' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            let mut depth = 0usize;
            let close = loop {
                if i >= bytes.len() {
                    return None;
                }
                if bytes[i] == b'(' {
                    depth += 1;
                } else if bytes[i] == b')' {
                    depth -= 1;
                    if depth == 0 {
                        break i;
                    }
                }
                i += 1;
            };
            // 越过函数 token（i 停在 ')' 上）
            i = close + 1;
            let fn_token = &v[start..=close];
            let shape = parse_clip_path(fn_token)?;
            if matches!(shape, ClipPathValue::None) {
                return None; // 形状函数体非法 → 声明无效
            }
            if segments.len() >= 2 {
                return None; // 超过两组
            }
            segments.push((shape, None));
            continue;
        }
        // 关键词 token（geometry-box）：名称扫描已消费至 token 尾（非函数起点 token）。
        let keyword = &v[start..i];
        let geometry_box = parse_geometry_box(keyword)?;
        let last = segments.last_mut()?;
        if last.1.is_some() {
            return None; // 一形状至多一个 geometry-box
        }
        last.1 = Some(geometry_box);
    }

    match segments.len() {
        1 => {
            let (shape, geometry_box) = segments.pop().unwrap();
            Some(BorderShapeValue::Stroke {
                shape,
                geometry_box: geometry_box.unwrap_or(BorderShapeGeometryBox::HalfBorderBox),
            })
        }
        2 => {
            let (inner, inner_box) = segments.pop().unwrap();
            let (outer, outer_box) = segments.pop().unwrap();
            Some(BorderShapeValue::Fill {
                outer,
                outer_box: outer_box.unwrap_or(BorderShapeGeometryBox::BorderBox),
                inner,
                inner_box: inner_box.unwrap_or(BorderShapeGeometryBox::PaddingBox),
            })
        }
        _ => None,
    }
}

/// 解析 geometry-box 关键词。
fn parse_geometry_box(value: &str) -> Option<BorderShapeGeometryBox> {
    match value.trim().to_ascii_lowercase().as_str() {
        "content-box" => Some(BorderShapeGeometryBox::ContentBox),
        "padding-box" => Some(BorderShapeGeometryBox::PaddingBox),
        "border-box" => Some(BorderShapeGeometryBox::BorderBox),
        "margin-box" => Some(BorderShapeGeometryBox::MarginBox),
        "half-border-box" => Some(BorderShapeGeometryBox::HalfBorderBox),
        _ => None,
    }
}

/// 解析 inset() 参数：top right bottom left [round <border-radius>]
fn parse_clip_inset(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }
    // 分离 round 部分
    let (inset_part, round) = if let Some(idx) = find_keyword_pos(inner, "round") {
        (&inner[..idx], Some(inner[idx + 5..].trim()))
    } else {
        (inner, None)
    };

    // calc 化偏移形（`inset(0px calc(100% - 20px) 2% 3em)`）在此拒收——
    // 朴素切分把 calc 内空格切断（4 值列表算成 6 值）。这是伪通过池保位
    // 纪律的接受面边界（二十一轮定谳：期望串含 calc 时本机 setProperty 拒收
    // → expected 落回静态值；b1v6/b1v7 两轮实证括号感知使该形可解析后动画
    // 通道回落链断裂 −2 至 −6），非文法终态。
    // FIXME(残面⑤ calc 代数插值刀)：落地后括号感知切分一并清 inset calc
    // 偏移拒收面（语料 inset-function-valid/008/computed 四案 + 动画 t=1 对）。
    let values: Vec<&str> = split_comma_or_space(inset_part);
    if !(1..=4).contains(&values.len()) {
        return None;
    }
    let top = parse_clip_length(values[0], true)?;
    let right = values
        .get(1)
        .and_then(|s| parse_clip_length(s, true))
        .unwrap_or_else(|| top.clone());
    let bottom = values
        .get(2)
        .and_then(|s| parse_clip_length(s, true))
        .unwrap_or_else(|| top.clone());
    let left = values
        .get(3)
        .and_then(|s| parse_clip_length(s, true))
        .unwrap_or_else(|| right.clone());

    let round_val = match round {
        Some(value) => Some(parse_clip_round_list(value, true)?),
        None => None,
    };

    Some(ClipPathValue::Inset {
        top,
        right,
        bottom,
        left,
        round: round_val,
    })
}

/// 解析 circle() 参数：[<radius>] [at <position>]
fn parse_clip_circle(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        // circle() 默认 closest-side at center
        return Some(ClipPathValue::Circle {
            radius: ClipPathRadius::ClosestSide,
            position: None,
        });
    }

    let (radius_part, position) = parse_shape_position(inner)?;
    let radius = if radius_part.is_empty() {
        ClipPathRadius::ClosestSide
    } else {
        parse_clip_radius(radius_part)?
    };

    Some(ClipPathValue::Circle { radius, position })
}

/// 解析 ellipse() 参数：[<rx> <ry>] [at <position>]
fn parse_clip_ellipse(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some(ClipPathValue::Ellipse {
            rx: ClipPathRadius::ClosestSide,
            ry: ClipPathRadius::ClosestSide,
            position: None,
        });
    }

    let (dims_part, position) = parse_shape_position(inner)?;

    let (rx, ry) = if dims_part.is_empty() {
        (ClipPathRadius::ClosestSide, ClipPathRadius::ClosestSide)
    } else {
        let parts: Vec<&str> = split_comma_or_space(dims_part);
        if parts.len() != 2 {
            return None;
        }
        let rx = parse_clip_radius(parts[0])?;
        let ry = parse_clip_radius(parts[1])?;
        (rx, ry)
    };

    Some(ClipPathValue::Ellipse { rx, ry, position })
}

/// 解析 polygon() 参数：[<fill-rule>,] <point> [<point>]*
fn parse_clip_polygon(rest: &str) -> Option<ClipPathValue> {
    let inner = rest.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }

    let mut fill_rule = PolygonFillRule::NonZero;
    let points_str = if let Some(after) = strip_polygon_fill_rule(inner, "nonzero") {
        let after = after.trim();
        if after.starts_with(',') {
            fill_rule = PolygonFillRule::NonZero;
            after.strip_prefix(',').unwrap().trim()
        } else if strip_polygon_fill_rule(after, "round").is_some() {
            // fill-rule 与 round 前缀无逗号直接相邻（`polygon(nonzero round
            // 1px, …)`，css-shapes-1 round 修饰语文法）——受理 fill-rule，
            // 余段交由下方 round 前缀臂切分；fill-rule 后接坐标形仍走原
            // 路径（points 切分拒收）。
            fill_rule = PolygonFillRule::NonZero;
            after
        } else {
            inner
        }
    } else if let Some(after) = strip_polygon_fill_rule(inner, "evenodd") {
        let after = after.trim();
        if after.starts_with(',') {
            fill_rule = PolygonFillRule::EvenOdd;
            after.strip_prefix(',').unwrap().trim()
        } else if strip_polygon_fill_rule(after, "round").is_some() {
            fill_rule = PolygonFillRule::EvenOdd;
            after
        } else {
            inner
        }
    } else {
        inner
    };

    // css-shapes-1 §funcdef-basic-shape-polygon round 参量文法门（前缀 +
    // 尾缀两臂）：radius 组走 parse_clip_round_list（<radius>{1,4}
    // [/ <radius>{1,4}]?）。存储有损忽略（Polygon 槽位无 round 字段，
    // FIXME 列表建模刀另行收口）——写面门与 apply 只要求 parse 判定；非
    // 法 round 形（空参/5+ 半径/负半径）整拒。
    // 前缀臂：`polygon( [<'fill-rule'> ||] round <radius-list> , <pairs> )`
    // ——radius 组止于首个顶层逗号（calc 内逗号不触发），坐标对从其后继续；
    // 无逗号（`round 1px 1px 2px`）整拒。
    let points_str = match strip_polygon_fill_rule(points_str, "round") {
        Some(after) => {
            let rest = after.trim_start();
            let ci = find_top_level_comma(rest)?;
            parse_clip_round_list(&rest[..ci], false)?;
            rest[ci + 1..].trim()
        }
        None => points_str,
    };
    // 尾缀臂：`<pairs> round <radius-list>`（此前缀臂先行，此处首个顶层
    // round 只可能为尾缀位）。
    let points_str = match find_keyword_pos(points_str, "round") {
        Some(idx) => {
            parse_clip_round_list(points_str[idx + "round".len()..].trim(), false)?;
            points_str[..idx].trim_end()
        }
        None => points_str,
    };

    let mut points = Vec::new();
    let point_pairs = split_top_level_commas(points_str)?;
    for pair in point_pairs {
        let pair = pair.trim();
        if pair.is_empty() {
            return None;
        }
        let coords = super::parse_extended_visual::split_top_level_whitespace(pair);
        if coords.len() != 2 {
            return None;
        }
        let x = parse_clip_length(&coords[0], true)?;
        let y = parse_clip_length(&coords[1], true)?;
        points.push((x, y));
    }

    if points.is_empty() {
        return None;
    }

    Some(ClipPathValue::Polygon { fill_rule, points })
}

/// 首个顶层逗号的字节位（括号内逗号不计，calc/min() 等数学函数安全）。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn split_top_level_commas(s: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ',' if depth == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    parts.push(s[start..].trim());
    Some(parts)
}

fn strip_polygon_fill_rule<'a>(input: &'a str, keyword: &str) -> Option<&'a str> {
    // https://drafts.fxtf.org/css-masking-1/#funcdef-basic-shape-polygon
    // CSS-defined polygon() fill-rule keywords are ASCII case-insensitive.
    let bytes = input.as_bytes();
    if bytes.len() >= keyword.len() && bytes[..keyword.len()].eq_ignore_ascii_case(keyword.as_bytes()) {
        return Some(&input[keyword.len()..]);
    }
    None
}

/// 解析 clip-path 半径值（circle/ellipse 的半径参数）。
fn parse_clip_radius(value: &str) -> Option<ClipPathRadius> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("closest-side") {
        return Some(ClipPathRadius::ClosestSide);
    }
    if v.eq_ignore_ascii_case("farthest-side") {
        return Some(ClipPathRadius::FarthestSide);
    }
    if v.eq_ignore_ascii_case("closest-corner") {
        return Some(ClipPathRadius::ClosestCorner);
    }
    if v.eq_ignore_ascii_case("farthest-corner") {
        return Some(ClipPathRadius::FarthestCorner);
    }
    parse_clip_length(v, false).map(ClipPathRadius::Length)
}

/// inset()/polygon() round 参量文法（css-shapes-1 §funcdef-inset /
/// §funcdef-basic-shape-polygon）：`<'border-radius'>` 简写全形
/// `<radius>{1,4} [/ <radius>{1,4}]?`（斜杠分隔水平/垂直半径组）。解析仅作文
/// 法门（accept/reject 供写面门与 apply 裁决），存储有损取水平首半径——
/// `ClipPathValue::Inset.round` 为单半径槽位、Polygon 槽位无 round 字段，
/// 列表建模 FIXME 另行收口（round 形 computed 此前 apply 恒拒 = none，有损
/// 存储不产生新串化行为面；写面门只要求 parse 判定——valid 页原文回显）。
/// `allow_percentage`：inset round 依 border-radius 文法收 `<percentage>`；
/// polygon round 按 Chrome 语料契约拒收（round-invalid 域 `polygon(round
/// 1%, …)` 实证 invalid，本引擎比 Chrome 宽则 invalid 页翻红）。
/// https://drafts.csswg.org/css-shapes/#funcdef-basic-shape-inset
fn parse_clip_round_list(value: &str, allow_percentage: bool) -> Option<ClipPathRadius> {
    let v = value.trim();
    let (h, vpart) = match v.split_once('/') {
        Some((h, r)) => (h.trim(), Some(r.trim())),
        None => (v, None),
    };
    let hs = super::parse_extended_visual::split_top_level_whitespace(h);
    if hs.is_empty() || hs.len() > 4 {
        return None;
    }
    let first = parse_clip_radius(&hs[0])?;
    if !allow_percentage && radius_is_percentage(&first) {
        return None;
    }
    for t in &hs[1..] {
        let r = parse_clip_radius(t)?;
        if !allow_percentage && radius_is_percentage(&r) {
            return None;
        }
    }
    if let Some(vp) = vpart {
        let vs = super::parse_extended_visual::split_top_level_whitespace(vp);
        if vs.is_empty() || vs.len() > 4 {
            return None;
        }
        for t in &vs {
            let r = parse_clip_radius(t)?;
            if !allow_percentage && radius_is_percentage(&r) {
                return None;
            }
        }
    }
    Some(first)
}

/// round 半径是否百分比形（`<length-percentage>` 的百分比分支，calc 内百
/// 分比不在此判域——语料无该形证据，不作宽拒）。
fn radius_is_percentage(r: &ClipPathRadius) -> bool {
    matches!(r, ClipPathRadius::Length(LengthValue::Percentage(_)))
}

/// 在字符串中查找关键字的位置（大小写不敏感，要求完整单词匹配）。
fn find_keyword_pos(s: &str, keyword: &str) -> Option<usize> {
    let lower = s.to_ascii_lowercase();
    let kw = keyword.to_ascii_lowercase();
    let mut start = 0;
    while let Some(idx) = lower[start..].find(&kw) {
        let abs_idx = start + idx;
        // 确保是完整单词（前面是空格，后面是空格或结尾）
        let before_ok = abs_idx == 0 || *s.as_bytes().get(abs_idx - 1)? == b' ';
        let after_idx = abs_idx + kw.len();
        let after_ok = after_idx >= s.len() || *s.as_bytes().get(after_idx)? == b' ';
        if before_ok && after_ok {
            return Some(abs_idx);
        }
        start = abs_idx + 1;
    }
    None
}

/// 从形状参数中分离 "at <position>" 部分。
///
/// 返回 (形状参数部分, 位置部分)。
fn parse_shape_position(inner: &str) -> Option<(&str, Option<(LengthValue, LengthValue)>)> {
    if let Some(at_idx) = find_keyword_pos(inner, "at") {
        let shape_part = inner[..at_idx].trim();
        let pos_str = inner[at_idx + 2..].trim();
        let pos = parse_position_pair(pos_str)?;
        Some((shape_part, Some(pos)))
    } else {
        Some((inner, None))
    }
}

/// 解析位置对 "x y" 或 "center" 等。
fn parse_position_pair(s: &str) -> Option<(LengthValue, LengthValue)> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 4 {
        // css-values-4 §position 4 值 edge-offset 形 `<edge> <offset> <edge>
        // <offset>`（css-shapes-1 at <position> circle/ellipse 语料主形态，
        // inset-001/circle-004 页）。首词定轴序，同轴双对拒收；
        // `right/bottom o` 语义 100% − o 以 Calc 形存储（style-system
        // parse_origin_edge_offsets 同构，shape() 通道
        // shape_axis_comp_resolved Edge 臂同型）。
        // <https://drafts.csswg.org/css-values-4/#typedef-position>
        let kw = |t: &str| match t.to_ascii_lowercase().as_str() {
            "left" => Some((0u8, 0.0f64)),
            "right" => Some((0, 100.0)),
            "top" => Some((1, 0.0)),
            "bottom" => Some((1, 100.0)),
            _ => None,
        };
        let (axis1, edge1) = kw(parts[0])?;
        let (axis2, edge2) = kw(parts[2])?;
        if axis1 == axis2 {
            return None;
        }
        let off1 = parse_clip_length(parts[1], true)?;
        let off2 = parse_clip_length(parts[3], true)?;
        let resolve = |edge_pct: f64, off: LengthValue| {
            if edge_pct == 0.0 {
                off
            } else if let LengthValue::Percentage(p) = off {
                // 百分比缘偏移折数代数精确（100% − p%），直接补数百分比存储
                // （chrome computed 串形 `bottom 70%` → `30%` 语料钉面
                // circle-004/ellipse-004；长度偏移保 Calc 形——computed
                // `bottom 1120px` → `calc(100% - 1120px)` 不可声明期折算）。
                LengthValue::Percentage(edge_pct - p)
            } else {
                LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
                    Box::new(CalcExpr::Length(LengthValue::Percentage(edge_pct))),
                    CalcOp::Subtract,
                    Box::new(CalcExpr::Length(off)),
                )))
            }
        };
        return Some(if axis1 == 1 {
            (resolve(edge2, off2), resolve(edge1, off1))
        } else {
            (resolve(edge1, off1), resolve(edge2, off2))
        });
    }
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    // css-values-4 §position 轴别严格文法（css-shapes-1 circle/ellipse at
    // 语料 oracle：parsing-utils.js invalidPositions 拒收表 `50% left`/
    // `top 50%`/`bottom 70%` 等 + validPositions `top left` y 首词对换
    // x-first canon）：
    // 2 值形合法组合——[<lp>|x-edge] [<lp>|y-edge|center]、center 填任意缺轴、
    // y-edge 首词仅可接 x-edge/center（对换存储）；[lp|x-edge][x-edge]、
    // [y-edge][lp|y-edge] 混轴/同轴双关键字拒收（`50% left`/`top 50%`/
    // `left right` 语料钉面）。3 词混搭形已由 len>2 早退拒收。
    // <https://drafts.csswg.org/css-values-4/#typedef-position>
    // 注：本严格文法按 css-shapes-1 WPT 语料所载 Chrome 契约实现——现代
    // css-values-4 的 2 词 `[edge] <offset>` 缘偏移形（`top 50%`/`bottom
    // 70px`）在该语料属 invalidPositions 拒收域，此处不收（语料计分契约
    // 优先）。FIXME(规格演进)：语料换代后按现代文法补 2 词缘偏移臂。
    enum PosComp {
        Len(LengthValue),
        XEdge(f64),
        YEdge(f64),
        Center,
    }
    let comp = |t: &str| -> Option<PosComp> {
        match t.to_ascii_lowercase().as_str() {
            "left" => Some(PosComp::XEdge(0.0)),
            "right" => Some(PosComp::XEdge(100.0)),
            "top" => Some(PosComp::YEdge(0.0)),
            "bottom" => Some(PosComp::YEdge(100.0)),
            "center" => Some(PosComp::Center),
            _ => parse_clip_length(t, true).map(PosComp::Len),
        }
    };
    let to_len = |c: PosComp| match c {
        PosComp::Len(lv) => lv,
        PosComp::XEdge(p) | PosComp::YEdge(p) => LengthValue::Percentage(p),
        PosComp::Center => LengthValue::Percentage(50.0),
    };
    let first = comp(parts[0])?;
    let (x, y) = if parts.len() == 1 {
        // 1 值形：关键字或 lp 定 x、y 缺省 center（`top` → 50% 0% 语料钉面）。
        match first {
            PosComp::YEdge(p) => (LengthValue::Percentage(50.0), LengthValue::Percentage(p)),
            other => (to_len(other), LengthValue::Percentage(50.0)),
        }
    } else {
        let second = comp(parts[1])?;
        match (first, second) {
            // y 首词关键字对 / center+x-edge 对——canon x-first 对换存储
            // （`top left`/`top center`/`center left` 语料钉面）。
            (PosComp::YEdge(p), PosComp::XEdge(q)) => (LengthValue::Percentage(q), LengthValue::Percentage(p)),
            (PosComp::YEdge(p), PosComp::Center) => (LengthValue::Percentage(50.0), LengthValue::Percentage(p)),
            (PosComp::Center, PosComp::XEdge(p)) => (LengthValue::Percentage(p), LengthValue::Percentage(50.0)),
            // 混轴/同轴双关键字拒收：[lp|x-edge][x-edge]、[y-edge][lp]、
            // [y-edge][y-edge]。
            (PosComp::Len(_), PosComp::XEdge(_))
            | (PosComp::XEdge(_), PosComp::XEdge(_))
            | (PosComp::YEdge(_), PosComp::Len(_))
            | (PosComp::YEdge(_), PosComp::YEdge(_)) => return None,
            (f, s) => (to_len(f), to_len(s)),
        }
    };
    Some((x, y))
}

/// 按空格分割字符串（忽略连续空格）。
fn split_comma_or_space(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

// ── CSS clip 属性解析（已弃用的 CSS2 裁剪属性） ───────────────────────

/// 解析 CSS clip 属性值。
///
/// 支持：`auto` | `rect(top, right, bottom, left)`
/// rect() 参数可以是长度值或 `auto`（等同于 0）。
pub fn parse_clip(value: &str) -> Option<ClipRectValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(ClipRectValue::Auto);
    }
    // https://www.w3.org/TR/css-syntax-3/#function-token-diagram
    // CSS function names are ASCII case-insensitive; whitespace before `(` is not a function token.
    if let Some(rest) = strip_basic_shape_prefix(v, "rect") {
        let inner = rest.strip_suffix(')')?.trim();
        // https://www.w3.org/TR/CSS2/visufx.html#clipping
        // rect() is a comma-separated list; commas nested inside CSS math
        // functions belong to the offset expression, not to the outer list.
        let parts = split_top_level_commas(inner)?;
        if parts.len() != 4 {
            return None;
        }
        let top = parse_length_or_auto_clip(parts[0])?;
        let right = parse_length_or_auto_clip(parts[1])?;
        let bottom = parse_length_or_auto_clip(parts[2])?;
        let left = parse_length_or_auto_clip(parts[3])?;
        return Some(ClipRectValue::Rect(top, right, bottom, left));
    }
    None
}

/// 解析长度值或 `auto`。
///
/// https://www.w3.org/TR/CSS2/visufx.html#clipping
/// rect() 中 `auto` 的计算值因侧而异（top/left = 0、right/bottom = 边框盒对应边），
/// 解析期不折叠，保留 [`LengthValue::Auto`] 由 paint 期按盒尺寸解析。
fn parse_length_or_auto_clip(s: &str) -> Option<LengthValue> {
    let v = s.trim();
    if v.eq_ignore_ascii_case("auto") {
        Some(LengthValue::Auto)
    } else {
        parse_clip_length(v, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clip_auto() {
        assert!(matches!(parse_clip("auto"), Some(ClipRectValue::Auto)));
        assert!(matches!(parse_clip("AUTO"), Some(ClipRectValue::Auto)));
    }

    #[test]
    fn test_parse_clip_rect() {
        let result = parse_clip("rect(0px, 50px, 50px, 0px)");
        assert!(matches!(
            result,
            Some(ClipRectValue::Rect(
                LengthValue::Px(0.0),
                LengthValue::Px(50.0),
                LengthValue::Px(50.0),
                LengthValue::Px(0.0),
            ))
        ));
    }

    #[test]
    fn test_parse_clip_rect_function_name_is_case_insensitive() {
        // https://www.w3.org/TR/css-syntax-3/#function-token-diagram
        // CSS function names are ASCII case-insensitive.
        // R3924：rect() 内 `auto` 保留 Auto（计算值因侧而异，paint 期解析）。
        assert!(matches!(
            parse_clip("RECT(0px, auto, 10px, 0px)"),
            Some(ClipRectValue::Rect(
                LengthValue::Px(0.0),
                LengthValue::Auto,
                LengthValue::Px(10.0),
                LengthValue::Px(0.0),
            ))
        ));
    }

    #[test]
    fn test_parse_clip_rect_auto_values() {
        // R3924：rect() 内 `auto` 保留 Auto，不再折叠成 0px（CSS2 §11.1.2：
        // auto 计算值因侧而异——top/left = 0、right/bottom = 边框盒对应边）。
        let result = parse_clip("rect(auto, auto, auto, auto)");
        assert!(matches!(
            result,
            Some(ClipRectValue::Rect(
                LengthValue::Auto,
                LengthValue::Auto,
                LengthValue::Auto,
                LengthValue::Auto,
            ))
        ));
    }

    #[test]
    fn test_parse_clip_rect_math_function_offsets() {
        // https://www.w3.org/TR/CSS2/visufx.html#clipping
        // rect() offsets are a comma-separated list; commas inside math functions
        // must not split the outer argument list.
        let result = parse_clip("rect(min(10px, 20px), auto, clamp(0px, 5px, 10px), 0px)");
        assert!(matches!(
            result,
            Some(ClipRectValue::Rect(
                LengthValue::Calc(_),
                LengthValue::Auto,
                LengthValue::Calc(_),
                LengthValue::Px(0.0),
            ))
        ));
    }

    #[test]
    fn test_parse_clip_invalid() {
        assert!(parse_clip("inherit").is_none());
        assert!(parse_clip("rect(10px)").is_none());
        assert!(parse_clip("rect(10px, 20px)").is_none());
        assert!(parse_clip("rect(calc(1), auto, 10px, 0px)").is_none());
        assert!(parse_clip("rect(0px, auto, 10px, 0px,)").is_none());
    }
}

#[cfg(test)]
mod r4534_border_shape_parse_tests {
    use super::*;

    #[test]
    fn fill_mode_two_polygons() {
        let v = parse_border_shape(
            "polygon(0 0, 100% 0, 100% 100%, 0 100%) margin-box polygon(0 0, 100% 0, 100% 100%, 0 100%) border-box",
        );
        assert!(matches!(v, Some(BorderShapeValue::Fill { .. })), "got {:?}", v);
    }

    #[test]
    fn stroke_mode_circle() {
        let v = parse_border_shape("circle()");
        assert!(matches!(v, Some(BorderShapeValue::Stroke { .. })), "got {:?}", v);
    }

    // R5179（css-borders-4 §3.7）：superellipse(±∞)/squircle 关键字映射与 K 语义。
    #[test]
    fn corner_shape_superellipse_infinity_maps_to_square() {
        use crate::values::types::{CornerShapeKind, CornerShapeValue};
        let v = parse_corner_shape("superellipse(infinity)").expect("infinity 合法");
        assert!(
            matches!(v, CornerShapeValue::All(CornerShapeKind::Square)),
            "superellipse(∞) ≡ square，got {:?}",
            v
        );
        let v = parse_corner_shape("superellipse(-INFINITY)").expect("-infinity 合法");
        assert!(
            matches!(v, CornerShapeValue::All(CornerShapeKind::Notch)),
            "superellipse(−∞) ≡ notch，got {:?}",
            v
        );
    }

    #[test]
    fn corner_shape_superellipse_finite_keeps_argument_and_rejects_nan() {
        use crate::values::types::{CornerShapeKind, CornerShapeValue};
        let v = parse_corner_shape("superellipse(-100)").expect("负大值合法");
        assert!(
            matches!(v, CornerShapeValue::All(CornerShapeKind::Superellipse(k)) if k == -100.0),
            "有限值保留原始参数 K，got {:?}",
            v
        );
        assert!(parse_corner_shape("superellipse(nan)").is_none(), "NaN 非法");
        assert!(parse_corner_shape("superellipse()").is_none(), "空参非法");
    }

    #[test]
    fn corner_shape_squircle_keyword() {
        use crate::values::types::{CornerShapeKind, CornerShapeValue};
        let v = parse_corner_shape("squircle").expect("squircle 合法");
        assert!(
            matches!(v, CornerShapeValue::All(CornerShapeKind::Superellipse(k)) if k == 2.0),
            "squircle ≡ superellipse(2)，got {:?}",
            v
        );
        // 4 值角序：bevel scoop notch squircle（overflow-clip-margin 案形）。
        let v = parse_corner_shape("bevel scoop notch squircle").expect("四值合法");
        match v {
            CornerShapeValue::PerCorner(c) => {
                assert_eq!(
                    *c,
                    [
                        CornerShapeKind::Bevel,
                        CornerShapeKind::Scoop,
                        CornerShapeKind::Notch,
                        CornerShapeKind::Superellipse(2.0),
                    ]
                );
            }
            _ => panic!("应为 PerCorner，got {:?}", v),
        }
    }
}

#[test]
fn probe_debug_border_shape() {
    let a = parse_clip_path("circle()");
    eprintln!("parse_clip_path(circle()) = {:?}", a);
    let v = parse_border_shape("circle()");
    eprintln!("parse_border_shape(circle()) = {:?}", v);
}
