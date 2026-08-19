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
        } else if lower.ends_with("px") || lower.ends_with("em") || lower.ends_with("rem") {
            let len = parse_length(token)?;
            if length_is_negative(&len) {
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
        if lower.ends_with("px") || lower.ends_with("em") || lower.ends_with("rem") {
            let len = parse_length(token)?;
            if length_is_negative(&len) {
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

/// CSS list-style-image 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStyleImageValue {
    /// none（默认值）— 无列表标记图片。
    None,
    /// url(<string>) — 列表标记图片。
    Url(String),
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

fn length_is_negative(value: &LengthValue) -> bool {
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
        | LengthValue::Percentage(v) => *v < 0.0,
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
        LengthValue::Calc(_) => true,
        _ => false,
    }
}

fn parse_clip_length(raw: &str, allow_negative: bool) -> Option<LengthValue> {
    let length = parse_length(raw)?;
    clip_path_length_is_valid(raw, &length, allow_negative).then_some(length)
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
pub fn parse_clip_path(value: &str) -> Option<ClipPathValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(ClipPathValue::None);
    }
    if let Some(rest) = v.strip_prefix("inset(") {
        return parse_clip_inset(rest);
    }
    if let Some(rest) = v.strip_prefix("circle(") {
        return parse_clip_circle(rest);
    }
    if let Some(rest) = v.strip_prefix("ellipse(") {
        return parse_clip_ellipse(rest);
    }
    if let Some(rest) = v.strip_prefix("polygon(") {
        return parse_clip_polygon(rest);
    }
    None
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
        Some(value) => Some(parse_clip_radius_single(value)?),
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
    let points_str = if inner.starts_with("nonzero") || inner.starts_with("NonZero") || inner.starts_with("NONZERO") {
        let after = inner[7..].trim();
        if after.starts_with(',') {
            fill_rule = PolygonFillRule::NonZero;
            after.strip_prefix(',').unwrap().trim()
        } else {
            inner
        }
    } else if inner.starts_with("evenodd") || inner.starts_with("EvenOdd") || inner.starts_with("EVENODD") {
        let after = inner[7..].trim();
        if after.starts_with(',') {
            fill_rule = PolygonFillRule::EvenOdd;
            after.strip_prefix(',').unwrap().trim()
        } else {
            inner
        }
    } else {
        inner
    };

    let mut points = Vec::new();
    for pair in points_str.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            return None;
        }
        let coords: Vec<&str> = pair.split_whitespace().collect();
        if coords.len() != 2 {
            return None;
        }
        let x = parse_clip_length(coords[0], true)?;
        let y = parse_clip_length(coords[1], true)?;
        points.push((x, y));
    }

    if points.is_empty() {
        return None;
    }

    Some(ClipPathValue::Polygon { fill_rule, points })
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
    parse_clip_length(v, false).map(ClipPathRadius::Length)
}

/// 解析单个圆角半径（用于 inset 的 round 参数）。
fn parse_clip_radius_single(value: &str) -> Option<ClipPathRadius> {
    parse_clip_radius(value)
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
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let x = if parts[0].eq_ignore_ascii_case("center") {
        LengthValue::Percentage(50.0)
    } else if parts[0].eq_ignore_ascii_case("left") {
        LengthValue::Percentage(0.0)
    } else if parts[0].eq_ignore_ascii_case("right") {
        LengthValue::Percentage(100.0)
    } else {
        parse_clip_length(parts[0], true)?
    };
    let y = if parts.len() < 2 || parts[1].eq_ignore_ascii_case("center") {
        LengthValue::Percentage(50.0)
    } else if parts[1].eq_ignore_ascii_case("top") {
        LengthValue::Percentage(0.0)
    } else if parts[1].eq_ignore_ascii_case("bottom") {
        LengthValue::Percentage(100.0)
    } else {
        parse_clip_length(parts[1], true)?
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
    // rect(top, right, bottom, left)
    if let Some(rest) = v.strip_prefix("rect(") {
        let inner = rest.strip_suffix(')')?.trim();
        // rect() 内部参数用逗号分隔
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 4 {
            return None;
        }
        let top = parse_length_or_auto_clip(parts[0].trim())?;
        let right = parse_length_or_auto_clip(parts[1].trim())?;
        let bottom = parse_length_or_auto_clip(parts[2].trim())?;
        let left = parse_length_or_auto_clip(parts[3].trim())?;
        return Some(ClipRectValue::Rect(top, right, bottom, left));
    }
    None
}

/// 解析长度值或 `auto`（视为 0px）。
fn parse_length_or_auto_clip(s: &str) -> Option<LengthValue> {
    let v = s.trim();
    if v.eq_ignore_ascii_case("auto") {
        Some(LengthValue::Px(0.0))
    } else {
        parse_length(v)
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
    fn test_parse_clip_rect_auto_values() {
        // auto inside rect() is treated as 0px
        let result = parse_clip("rect(auto, auto, auto, auto)");
        assert!(matches!(
            result,
            Some(ClipRectValue::Rect(
                LengthValue::Px(0.0),
                LengthValue::Px(0.0),
                LengthValue::Px(0.0),
                LengthValue::Px(0.0),
            ))
        ));
    }

    #[test]
    fn test_parse_clip_invalid() {
        assert!(parse_clip("inherit").is_none());
        assert!(parse_clip("rect(10px)").is_none());
        assert!(parse_clip("rect(10px, 20px)").is_none());
    }
}
