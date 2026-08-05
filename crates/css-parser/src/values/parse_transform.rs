//! CSS Transition、Animation、Transform、Gradient 解析。

use super::*;

// ── CSS Transition 值类型 ──────────────────────────────────────────────

/// CSS transition-timing-function / animation-timing-function 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunctionValue {
    /// ease。
    Ease,
    /// linear。
    Linear,
    /// ease-in。
    EaseIn,
    /// ease-out。
    EaseOut,
    /// ease-in-out。
    EaseInOut,
    /// cubic-bezier(x1, y1, x2, y2)。
    CubicBezier(f64, f64, f64, f64),
    /// step-start。
    StepStart,
    /// step-end。
    StepEnd,
    /// steps(n, position)。
    Steps(i32, Option<StepPosition>),
}

/// steps() 的位置参数。
#[derive(Debug, Clone, PartialEq)]
pub enum StepPosition {
    /// jump-start / start。
    Start,
    /// jump-end / end（默认）。
    End,
    /// jump-both。
    Both,
    /// jump-none。
    None,
}

/// CSS animation-direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDirectionValue {
    /// normal。
    Normal,
    /// reverse。
    Reverse,
    /// alternate。
    Alternate,
    /// alternate-reverse。
    AlternateReverse,
}

/// CSS animation-fill-mode 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationFillModeValue {
    /// none。
    None,
    /// forwards。
    Forwards,
    /// backwards。
    Backwards,
    /// both。
    Both,
}

/// CSS animation-play-state 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPlayStateValue {
    /// running。
    Running,
    /// paused。
    Paused,
}

/// 解析 CSS animation-direction 值。
pub fn parse_animation_direction(value: &str) -> Option<AnimationDirectionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(AnimationDirectionValue::Normal),
        "reverse" => Some(AnimationDirectionValue::Reverse),
        "alternate" => Some(AnimationDirectionValue::Alternate),
        "alternate-reverse" => Some(AnimationDirectionValue::AlternateReverse),
        _ => None,
    }
}

/// 解析 CSS animation-fill-mode 值。
pub fn parse_animation_fill_mode(value: &str) -> Option<AnimationFillModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(AnimationFillModeValue::None),
        "forwards" => Some(AnimationFillModeValue::Forwards),
        "backwards" => Some(AnimationFillModeValue::Backwards),
        "both" => Some(AnimationFillModeValue::Both),
        _ => None,
    }
}

/// 解析 CSS animation-play-state 值。
pub fn parse_animation_play_state(value: &str) -> Option<AnimationPlayStateValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "running" => Some(AnimationPlayStateValue::Running),
        "paused" => Some(AnimationPlayStateValue::Paused),
        _ => None,
    }
}

// ── CSS Animation Name 值类型 ─────────────────────────────────────

/// CSS animation-name 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationNameValue {
    /// none — 无动画。
    None,
    /// 自定义动画名称。
    Custom(String),
}

/// 解析 CSS animation-name 属性值。
///
/// 支持格式如 `"none"`、`"fadeIn"`、`"slide-in"`。
pub fn parse_animation_name(value: &str) -> Option<AnimationNameValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(AnimationNameValue::None);
    }
    // 动画名称必须是有效 CSS 标识符（字母/下划线/连字符开头，不含空格）
    if v.is_empty() || v.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    if v.contains(|c: char| c.is_whitespace()) {
        return None;
    }
    Some(AnimationNameValue::Custom(v.to_string()))
}

// ── CSS Animation Duration 值类型 ──────────────────────────────────

/// CSS animation-duration 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDurationValue {
    /// 时间值（秒或毫秒）。
    Time(f64, TimeUnit),
}

/// 时间单位。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeUnit {
    /// 秒。
    S,
    /// 毫秒。
    Ms,
}

/// 解析 CSS animation-duration 属性值。
///
/// 支持格式如 `"1s"`、`"500ms"`、`"0.5s"`。
pub fn parse_animation_duration(value: &str) -> Option<AnimationDurationValue> {
    let v = value.trim().to_ascii_lowercase();
    if v.ends_with("ms") {
        let n: f64 = v.trim_end_matches("ms").trim().parse().ok()?;
        if n >= 0.0 {
            return Some(AnimationDurationValue::Time(n, TimeUnit::Ms));
        }
    } else if v.ends_with('s') {
        let n: f64 = v.trim_end_matches('s').trim().parse().ok()?;
        if n >= 0.0 {
            return Some(AnimationDurationValue::Time(n, TimeUnit::S));
        }
    }
    None
}

// ── CSS Animation Iteration Count 值类型 ────────────────────────────

/// CSS animation-iteration-count 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationIterationCountValue {
    /// infinite — 无限循环。
    Infinite,
    /// 有限次数。
    Number(f64),
}

/// 解析 CSS animation-iteration-count 属性值。
///
/// 支持格式如 `"infinite"`、`"3"`、`"2.5"`。
pub fn parse_animation_iteration_count(value: &str) -> Option<AnimationIterationCountValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("infinite") {
        return Some(AnimationIterationCountValue::Infinite);
    }
    let n: f64 = v.parse().ok()?;
    if n > 0.0 {
        Some(AnimationIterationCountValue::Number(n))
    } else {
        None
    }
}

/// 解析 CSS transition-timing-function 值。
pub fn parse_timing_function(value: &str) -> Option<TimingFunctionValue> {
    // CSS 关键字与函数名大小写不敏感（CSS Syntax §）。整体小写——数值/逗号/括号不受影响，
    // cubic-bezier()/steps() 的数字参数解析保持精确。
    let value = value.trim().to_ascii_lowercase();

    match value.as_str() {
        "ease" => Some(TimingFunctionValue::Ease),
        "linear" => Some(TimingFunctionValue::Linear),
        "ease-in" => Some(TimingFunctionValue::EaseIn),
        "ease-out" => Some(TimingFunctionValue::EaseOut),
        "ease-in-out" => Some(TimingFunctionValue::EaseInOut),
        "step-start" => Some(TimingFunctionValue::StepStart),
        "step-end" => Some(TimingFunctionValue::StepEnd),
        _ if value.starts_with("cubic-bezier(") => {
            let inner = extract_parens_content(&value, "cubic-bezier")?;
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                return None;
            }
            let x1 = parts[0].parse::<f64>().ok()?;
            let y1 = parts[1].parse::<f64>().ok()?;
            let x2 = parts[2].parse::<f64>().ok()?;
            let y2 = parts[3].parse::<f64>().ok()?;
            Some(TimingFunctionValue::CubicBezier(x1, y1, x2, y2))
        }
        _ if value.starts_with("steps(") => {
            let inner = extract_parens_content(&value, "steps")?;
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            let n: i32 = parts.first()?.parse().ok()?;
            let position = if parts.len() > 1 {
                Some(parse_step_position(parts[1])?)
            } else {
                None
            };
            Some(TimingFunctionValue::Steps(n, position))
        }
        _ => None,
    }
}

/// 解析 steps() 位置参数。
fn parse_step_position(s: &str) -> Option<StepPosition> {
    match s.trim().to_ascii_lowercase().as_str() {
        "jump-start" | "start" => Some(StepPosition::Start),
        "jump-end" | "end" => Some(StepPosition::End),
        "jump-both" | "both" => Some(StepPosition::Both),
        "jump-none" | "none" => Some(StepPosition::None),
        _ => None,
    }
}

/// 提取函数括号内的内容。
fn extract_parens_content<'a>(value: &'a str, func_name: &str) -> Option<&'a str> {
    let prefix = format!("{}(", func_name);
    if !value.starts_with(&prefix) || !value.ends_with(')') {
        return None;
    }
    Some(&value[func_name.len() + 1..value.len() - 1])
}

/// 解析 CSS 时间值（如 `"0.3s"`、`"200ms"`）。
///
/// 返回秒为单位的 f64 值。单位大小写不敏感（CSS Values §：500MS ≡ 500ms）。
pub fn parse_time(value: &str) -> Option<f64> {
    let lower = value.trim().to_ascii_lowercase();
    if let Some(n) = lower.strip_suffix("ms") {
        n.trim().parse::<f64>().ok().map(|ms| ms / 1000.0)
    } else if let Some(n) = lower.strip_suffix('s') {
        n.trim().parse::<f64>().ok()
    } else {
        None
    }
}

// ── CSS Transform 值类型 ──────────────────────────────────────────────

/// CSS transform 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TransformValue {
    /// none。
    None,
    /// 变换函数列表。
    List(Vec<TransformFunction>),
}

/// CSS 单个变换函数。
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    /// translate(tx, ty)。
    Translate(f64, f64),
    /// translate(tx, ty) 至少一个分量为百分比（CSS Transforms L1：`%` 相对元素 border-box）。
    /// `(tx, tx_is_pct, ty, ty_is_pct)`。仅当任一分量为 `%` 时使用（纯 px 仍走 [`Translate`](TransformFunction::Translate)，
    /// 零回归）。R2294：修复 `translate(50%)` 旧解析 `%` 失败→整 transform 丢弃。
    TranslateMixed(f64, bool, f64, bool),
    /// translateX(tx) tx 为百分比。
    TranslateXMixed(f64, bool),
    /// translateY(ty) ty 为百分比。
    TranslateYMixed(f64, bool),
    /// translateX(tx)。
    TranslateX(f64),
    /// translateY(ty)。
    TranslateY(f64),
    /// rotate(angle) — 角度（度数）。
    Rotate(f64),
    /// scale(sx, sy)。
    Scale(f64, Option<f64>),
    /// scaleX(sx)。
    ScaleX(f64),
    /// scaleY(sy)。
    ScaleY(f64),
    /// skew(ax, ay) — 角度（度数）。
    Skew(f64, Option<f64>),
    /// rotateX(angle) — 绕 X 轴旋转（度数）。
    RotateX(f64),
    /// rotateY(angle) — 绕 Y 轴旋转（度数）。
    RotateY(f64),
    /// rotateZ(angle) — 绕 Z 轴旋转（度数）。
    RotateZ(f64),
    /// translate3d(tx, ty, tz) — 三维平移。
    Translate3d(f64, f64, f64),
    /// scale3d(sx, sy, sz) — 三维缩放。
    Scale3d(f64, f64, f64),
    /// rotate3d(x, y, z, angle) — 绕任意轴旋转。
    Rotate3d(f64, f64, f64, f64),
    /// perspective(length) — 透视距离。
    Perspective(f64),
    /// matrix(a, b, c, d, e, f) — 二维矩阵变换。
    Matrix(f64, f64, f64, f64, f64, f64),
}

/// 解析 CSS transform 属性值。
///
/// 支持格式如 `"translate(10px, 20px) rotate(45deg) scale(2)"`。
pub fn parse_transform(value: &str) -> Option<TransformValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(TransformValue::None);
    }

    let mut functions = Vec::new();
    let mut pos = 0;
    let bytes = value.as_bytes();

    while pos < bytes.len() {
        // 跳过空白
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        // 读取函数名（允许字母和数字，如 translate3d、scale3d、rotate3d）
        let name_start = pos;
        while pos < bytes.len() && (bytes[pos].is_ascii_alphabetic() || bytes[pos].is_ascii_digit()) {
            pos += 1;
        }
        let name = &value[name_start..pos];

        // 跳过空白和 '('
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() || bytes[pos] != b'(' {
            return None;
        }
        pos += 1; // skip '('

        // 找到匹配的 ')'
        let args_start = pos;
        let mut depth = 1;
        while pos < bytes.len() && depth > 0 {
            if bytes[pos] == b'(' {
                depth += 1;
            } else if bytes[pos] == b')' {
                depth -= 1;
            }
            pos += 1;
        }
        let args_str = value[args_start..pos - 1].trim();

        // 解析函数
        let func = parse_transform_function(name, args_str)?;
        functions.push(func);
    }

    if functions.is_empty() {
        None
    } else {
        Some(TransformValue::List(functions))
    }
}

/// 解析单个变换函数。
fn parse_transform_function(name: &str, args: &str) -> Option<TransformFunction> {
    // CSS 关键字大小写不敏感（CSS Syntax §3.1）：`translatex`/`Translate`/`MATRIX` 等同 canonical-case。
    // R2295：修复 parse_transform 不 lowercase name → 非 canonical-case 函数名被丢（WPT transform 测试常用小写）。
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "translate" => {
            let parts = split_transform_value_args(args)?;
            let (tx, txp) = parse_len_or_pct(parts.first()?)?;
            let (ty, typ) = match parts.get(1) {
                Some(p) => parse_len_or_pct(p)?,
                None => (0.0, false),
            };
            // 任一分量为 % → Mixed 变体（paint 期对 border-box 求值）；纯 px 走既有 Translate（零回归）。
            Some(if txp || typ {
                TransformFunction::TranslateMixed(tx, txp, ty, typ)
            } else {
                TransformFunction::Translate(tx, ty)
            })
        }
        "translatex" => {
            let parts = split_transform_value_args(args)?;
            let (tx, txp) = parse_len_or_pct(parts.first()?)?;
            Some(if txp {
                TransformFunction::TranslateXMixed(tx, true)
            } else {
                TransformFunction::TranslateX(tx)
            })
        }
        "translatey" => {
            let parts = split_transform_value_args(args)?;
            let (ty, typ) = parse_len_or_pct(parts.first()?)?;
            Some(if typ {
                TransformFunction::TranslateYMixed(ty, true)
            } else {
                TransformFunction::TranslateY(ty)
            })
        }
        "rotate" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::Rotate(angle))
        }
        "scale" => {
            let vals = parse_transform_args(args)?;
            let sx = vals.first().copied()?;
            let sy = vals.get(1).copied();
            Some(TransformFunction::Scale(sx, sy))
        }
        "scalex" => {
            let vals = parse_transform_args(args)?;
            let sx = vals.first().copied()?;
            Some(TransformFunction::ScaleX(sx))
        }
        "scaley" => {
            let vals = parse_transform_args(args)?;
            let sy = vals.first().copied()?;
            Some(TransformFunction::ScaleY(sy))
        }
        "skew" => {
            let vals = parse_transform_args(args)?;
            let ax = vals.first().copied()?;
            let ay = vals.get(1).copied();
            Some(TransformFunction::Skew(ax, ay))
        }
        "rotatex" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateX(angle))
        }
        "rotatey" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateY(angle))
        }
        "rotatez" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateZ(angle))
        }
        "translate3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return None;
            }
            let tx = parse_css_number(parts[0])?;
            let ty = parse_css_number(parts[1])?;
            let tz = parse_css_number(parts[2])?;
            Some(TransformFunction::Translate3d(tx, ty, tz))
        }
        "scale3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return None;
            }
            let sx = parse_css_number(parts[0])?;
            let sy = parse_css_number(parts[1])?;
            let sz = parse_css_number(parts[2])?;
            Some(TransformFunction::Scale3d(sx, sy, sz))
        }
        "rotate3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                return None;
            }
            let x = parse_css_number(parts[0])?;
            let y = parse_css_number(parts[1])?;
            let z = parse_css_number(parts[2])?;
            let angle = parse_angle(parts[3])?;
            Some(TransformFunction::Rotate3d(x, y, z, angle))
        }
        "perspective" => {
            let val = parse_css_number(args)?;
            if val <= 0.0 {
                return None;
            }
            Some(TransformFunction::Perspective(val))
        }
        "matrix" => {
            let vals = parse_transform_args(args)?;
            if vals.len() != 6 {
                return None;
            }
            Some(TransformFunction::Matrix(
                vals[0], vals[1], vals[2], vals[3], vals[4], vals[5],
            ))
        }
        _ => None,
    }
}

/// 解析变换参数列表（逗号或空格分隔的数值）。
fn parse_transform_args(args: &str) -> Option<Vec<f64>> {
    let mut result = Vec::new();
    for part in args.split(|c: char| c == ',' || c.is_whitespace()) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // 尝试解析为带单位的角度或长度
        let val = parse_css_number(part)?;
        result.push(val);
    }
    if result.is_empty() { None } else { Some(result) }
}

/// 按逗号/空白拆分 translate 参数为原始 token（保留 `%` 后缀供 [`parse_len_or_pct`] 判定）。
fn split_transform_value_args(args: &str) -> Option<Vec<&str>> {
    let parts: Vec<&str> = args
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() { None } else { Some(parts) }
}

/// 解析 translate 分量：返回 `(数值, 是否百分比)`。`%` 保留原数（paint 期对 border-box 求值），
/// 其余（px/em/rem/裸数）走 [`parse_css_number`]。driving: R2294 transform `translate(%)`。
fn parse_len_or_pct(s: &str) -> Option<(f64, bool)> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('%') {
        Some((num.trim().parse::<f64>().ok()?, true))
    } else {
        Some((parse_css_number(s)?, false))
    }
}

/// 解析 CSS 数值（可能带 px/deg/grad/rad/turn 等单位）。
///
/// 返回原始数值（px 直接返回数值，角度转为度数）。CSS 单位大小写不敏感（CSS Values §）。
/// 注意：grad 须在 rad 之前判定（"Xgrad" 后缀含 "rad"）。
fn parse_css_number(s: &str) -> Option<f64> {
    let lower = s.trim().to_ascii_lowercase();
    if let Some(n) = lower.strip_suffix("deg") {
        n.trim().parse::<f64>().ok()
    } else if let Some(n) = lower.strip_suffix("grad") {
        // 400grad = 360deg → 1grad = 0.9deg
        n.trim().parse::<f64>().ok().map(|g| g * 0.9)
    } else if let Some(n) = lower.strip_suffix("turn") {
        n.trim().parse::<f64>().ok().map(|t| t * 360.0)
    } else if let Some(n) = lower.strip_suffix("rad") {
        n.trim().parse::<f64>().ok().map(|r| r.to_degrees())
    } else if lower.ends_with("px") || lower.ends_with("em") || lower.ends_with("rem") {
        // 对于 translate，返回数值部分
        let num_end = lower.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')?;
        lower[..num_end].parse::<f64>().ok()
    } else {
        lower.parse::<f64>().ok()
    }
}

/// 解析角度值（返回度数）。
fn parse_angle(s: &str) -> Option<f64> {
    parse_css_number(s)
}

// ── CSS Gradient 值类型 ──────────────────────────────────────────────

/// CSS 渐变方向。
#[derive(Debug, Clone, PartialEq)]
pub enum GradientDirection {
    /// 角度（度数）。
    Angle(f64),
    /// to top。
    ToTop,
    /// to bottom。
    ToBottom,
    /// to left。
    ToLeft,
    /// to right。
    ToRight,
    /// to top left / to left top。
    ToTopLeft,
    /// to top right / to right top。
    ToTopRight,
    /// to bottom left / to left bottom。
    ToBottomLeft,
    /// to bottom right / to right bottom。
    ToBottomRight,
}

/// CSS 渐变色标。
#[derive(Debug, Clone, PartialEq)]
pub struct GradientColorStop {
    /// 颜色值。
    pub color: ColorValue,
    /// 位置提示（百分比或长度），如 `50%`、`10px`。
    pub position: Option<LengthValue>,
}

/// CSS Color 4 渐变颜色插值色彩空间（`gradient in <colorspace>`，R2289）。
///
/// 与 `render-foundation::primitive::GradientColorSpace` 一一对应；engine 负责映射。
/// wide-gamut（display-p3/xyz/rec2020/...）与未知空间在解析期归一为 Srgb（无色彩管理管线，
/// 优雅回退）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorInterpolationSpace {
    /// gamma 编码 sRGB（默认）。
    #[default]
    Srgb,
    /// 线性光 sRGB。
    SrgbLinear,
    /// CIE Lab。
    Lab,
    /// OKLab。
    Oklab,
    /// CIE LCH（极坐标）。
    Lch,
    /// OKLCH（极坐标）。
    Oklch,
}

/// 极坐标色彩空间（LCH/OKLCH）的色相插值法（CSS Color 4 §13.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorHueMethod {
    /// `shorter hue`（默认，短弧）。
    #[default]
    Shorter,
    /// `longer hue`。
    Longer,
    /// `increasing hue`。
    Increasing,
    /// `decreasing hue`。
    Decreasing,
}

/// CSS Color 4 渐变颜色插值配置：色彩空间 + （极坐标时）色相插值法。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorInterpolation {
    /// 插值色彩空间。
    pub space: ColorInterpolationSpace,
    /// 色相插值法（仅 Lch/Oklch 有意义）。
    pub hue: ColorHueMethod,
}

/// CSS linear-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// 渐变方向，默认为 to bottom。
    pub direction: GradientDirection,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-linear-gradient。
    pub repeating: bool,
    /// 颜色插值配置（CSS Color 4 `in <colorspace>`）。默认 Srgb = 既有行为。
    pub interpolation: ColorInterpolation,
}

/// CSS radial-gradient 形状。
#[derive(Debug, Clone, PartialEq)]
pub enum RadialShape {
    /// circle。
    Circle,
    /// ellipse。
    Ellipse,
}

/// CSS radial-gradient 尺寸。
#[derive(Debug, Clone, PartialEq)]
pub enum RadialSize {
    /// closest-side。
    ClosestSide,
    /// farthest-side。
    FarthestSide,
    /// closest-corner。
    ClosestCorner,
    /// farthest-corner（默认）。
    FarthestCorner,
    /// 明确的半径值。
    Length(LengthValue),
}

/// CSS radial-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// 形状，默认为 ellipse。
    pub shape: RadialShape,
    /// 尺寸，默认为 farthest-corner。
    pub size: RadialSize,
    /// 中心位置 X，默认为 center (50%)。
    pub position_x: LengthValue,
    /// 中心位置 Y，默认为 center (50%)。
    pub position_y: LengthValue,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-radial-gradient。
    pub repeating: bool,
    /// 颜色插值配置（CSS Color 4 `in <colorspace>`）。
    pub interpolation: ColorInterpolation,
}

/// CSS conic-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    /// 起始角度（度数），默认为 0。
    pub from_angle: f64,
    /// 中心位置 X，默认为 center (50%)。
    pub position_x: LengthValue,
    /// 中心位置 Y，默认为 center (50%)。
    pub position_y: LengthValue,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-conic-gradient。
    pub repeating: bool,
    /// 颜色插值配置（CSS Color 4 `in <colorspace>`）。
    pub interpolation: ColorInterpolation,
}

/// CSS 渐变值（所有渐变类型的统一表示）。
#[derive(Debug, Clone, PartialEq)]
pub enum GradientValue {
    /// linear-gradient() / repeating-linear-gradient()。
    Linear(LinearGradient),
    /// radial-gradient() / repeating-radial-gradient()。
    Radial(RadialGradient),
    /// conic-gradient() / repeating-conic-gradient()。
    Conic(ConicGradient),
}

/// 解析 CSS 渐变值。
///
/// 支持格式：
/// - `linear-gradient(direction, color-stop1, color-stop2, ...)`
/// - `radial-gradient(shape size at position, color-stop1, ...)`
/// - `conic-gradient(from angle at position, color-stop1, ...)`
/// - 以及对应的 repeating- 变体。
pub fn parse_gradient(value: &str) -> Option<GradientValue> {
    let value = value.trim();

    let (func_name, inner) = split_function_call(value)?;

    match func_name.to_ascii_lowercase().as_str() {
        "linear-gradient" => parse_linear_gradient_inner(inner, false),
        "repeating-linear-gradient" => parse_linear_gradient_inner(inner, true),
        "radial-gradient" => parse_radial_gradient_inner(inner, false),
        "repeating-radial-gradient" => parse_radial_gradient_inner(inner, true),
        "conic-gradient" => parse_conic_gradient_inner(inner, false),
        "repeating-conic-gradient" => parse_conic_gradient_inner(inner, true),
        _ => None,
    }
}

/// 将函数调用拆分为 (函数名, 括号内内容)。
fn split_function_call(value: &str) -> Option<(String, &str)> {
    let paren_pos = value.find('(')?;
    let name = &value[..paren_pos];
    if !value.ends_with(')') {
        return None;
    }
    let inner = &value[paren_pos + 1..value.len() - 1];
    Some((name.to_string(), inner))
}

/// 从渐变首参中剥离并解析 CSS Color 4 `in <colorspace> [<hue-method>]` 颜色插值提示。
///
/// 返回 `(剩余方向/形状部分, 解析得到的插值配置)`：
/// - `"in oklab"` → `(None, Some(oklab))`
/// - `"to right in srgb"` → `(Some("to right"), Some(srgb))`
/// - `"circle at center in oklch longer hue"` → `(Some("circle at center"), Some(oklch, longer))`
/// - `"in display-p3"` → `(None, Some(srgb))`（wide-gamut 无色彩管理，优雅回退 srgb）
/// - 无 `in` 提示 → `(Some(orig), None)`
///
/// `Some(ColorInterpolation)` = 检测到 `in` 提示并解析（未知空间/hue 归一为 Srgb/Shorter
/// 优雅回退，保留 R2288 不丢弃 gradient 的行为）。driving: css-images oklab/lch/oklch/
/// srgb-linear gradient（R2289 render-math）。
fn strip_interpolation_hint(arg: &str) -> (Option<&str>, Option<ColorInterpolation>) {
    let arg = arg.trim();
    let bytes = arg.as_bytes();

    // 前缀 "in "（大小写不敏感）：整参为提示，无方向部分。
    if bytes.len() >= 3
        && (bytes[0] == b'i' || bytes[0] == b'I')
        && (bytes[1] == b'n' || bytes[1] == b'N')
        && bytes[2] == b' '
    {
        let interp = parse_color_interpolation(arg[3..].trim());
        return (None, interp);
    }

    // 子串 " in "（大小写不敏感；空格 ASCII 单字节，切片边界安全）。
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i] == b' '
            && (bytes[i + 1] == b'i' || bytes[i + 1] == b'I')
            && (bytes[i + 2] == b'n' || bytes[i + 2] == b'N')
            && bytes[i + 3] == b' '
        {
            let dir = arg[..i].trim();
            let interp = parse_color_interpolation(arg[i + 4..].trim());
            return (if dir.is_empty() { None } else { Some(dir) }, interp);
        }
        i += 1;
    }

    (Some(arg), None)
}

/// 解析 CSS Color 4 `<color-interpolation-method>` = `<colorspace> [<hue-method>]?`。
///
/// 已知色彩空间：srgb / srgb-linear / lab / oklab / lch / oklch。极坐标空间（lch/oklch）
/// 可选 hue 插值法（shorter/longer/increasing/decreasing hue）。wide-gamut（display-p3/
/// a98-rgb/rec2020/prophoto-rgb/xyz[-d50/-d65]）与未知空间 → Srgb 优雅回退（无色彩管理）。
fn parse_color_interpolation(s: &str) -> Option<ColorInterpolation> {
    let lower = s.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    let first = *parts.first()?;
    let space = match first {
        "srgb" => ColorInterpolationSpace::Srgb,
        "srgb-linear" => ColorInterpolationSpace::SrgbLinear,
        "lab" => ColorInterpolationSpace::Lab,
        "oklab" => ColorInterpolationSpace::Oklab,
        "lch" => ColorInterpolationSpace::Lch,
        "oklch" => ColorInterpolationSpace::Oklch,
        // wide-gamut / xyz：无色彩管理管线，优雅回退 srgb（gamma 插值）。
        "display-p3" | "a98-rgb" | "rec2020" | "prophoto-rgb" | "xyz" | "xyz-d50" | "xyz-d65" => {
            ColorInterpolationSpace::Srgb
        }
        // 未知空间：按 `in` 提示存在但无法识别处理 → Srgb 回退（不丢弃 gradient）。
        _ => ColorInterpolationSpace::Srgb,
    };
    let mut hue = ColorHueMethod::default();
    if matches!(space, ColorInterpolationSpace::Lch | ColorInterpolationSpace::Oklch) {
        // 寻找 `<method> hue` 两词序列。
        let mut it = parts.iter().skip(1);
        while let Some(w) = it.next() {
            let method = match *w {
                "shorter" => Some(ColorHueMethod::Shorter),
                "longer" => Some(ColorHueMethod::Longer),
                "increasing" => Some(ColorHueMethod::Increasing),
                "decreasing" => Some(ColorHueMethod::Decreasing),
                _ => None,
            };
            if let Some(m) = method
                && matches!(it.next(), Some(next) if *next == "hue")
            {
                hue = m;
                break;
            }
        }
    }
    Some(ColorInterpolation { space, hue })
}

/// 解析 linear-gradient 内部参数。
fn parse_linear_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut direction = GradientDirection::ToBottom;
    let mut stop_start = 0;
    let mut interpolation = ColorInterpolation::default();

    // 检查第一个参数是否为方向（可能附带 `in <colorspace>` 插值提示）
    let first = args[0].trim();
    let (dir_part, interp) = strip_interpolation_hint(first);
    if let Some(i) = interp {
        // 首参含 `in <colorspace>`：剥离后解析方向（无方向则用默认）
        interpolation = i;
        if let Some(dp) = dir_part
            && let Some(dir) = parse_linear_direction(dp)
        {
            direction = dir;
        }
        stop_start = 1;
    } else if let Some(dir) = parse_linear_direction(first) {
        direction = dir;
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..], false)?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Linear(LinearGradient {
        direction,
        stops,
        repeating,
        interpolation,
    }))
}

/// 解析 linear-gradient 方向参数。
fn parse_linear_direction(s: &str) -> Option<GradientDirection> {
    let s = s.trim();
    // 角度
    if let Some(angle) = parse_angle(s) {
        return Some(GradientDirection::Angle(angle));
    }
    // to 关键字方向
    match s.to_ascii_lowercase().as_str() {
        "to top" => Some(GradientDirection::ToTop),
        "to bottom" => Some(GradientDirection::ToBottom),
        "to left" => Some(GradientDirection::ToLeft),
        "to right" => Some(GradientDirection::ToRight),
        "to top left" | "to left top" => Some(GradientDirection::ToTopLeft),
        "to top right" | "to right top" => Some(GradientDirection::ToTopRight),
        "to bottom left" | "to left bottom" => Some(GradientDirection::ToBottomLeft),
        "to bottom right" | "to right bottom" => Some(GradientDirection::ToBottomRight),
        _ => None,
    }
}

/// 解析 radial-gradient 内部参数。
fn parse_radial_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut shape = RadialShape::Ellipse;
    let mut size = RadialSize::FarthestCorner;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);
    let mut stop_start = 0;
    let mut interpolation = ColorInterpolation::default();

    // 第一个参数可能包含 shape/size/position（可能附带 `in <colorspace>` 插值提示）
    let first = args[0].trim();
    let (shape_part, interp) = strip_interpolation_hint(first);
    let shape_src = shape_part.unwrap_or("");
    let shape_lower = shape_src.to_ascii_lowercase();

    if !shape_src.is_empty()
        && (shape_lower.starts_with("circle")
            || shape_lower.starts_with("ellipse")
            || shape_lower.starts_with("closest")
            || shape_lower.starts_with("farthest")
            || shape_lower.starts_with("at ")
            || shape_lower.contains(" at "))
    {
        // 解析 shape + size + at position
        if let Some((s, sz, px, py)) = parse_radial_shape_and_position(shape_src) {
            shape = s;
            size = sz;
            pos_x = px;
            pos_y = py;
        }
        if let Some(i) = interp {
            interpolation = i;
        }
        stop_start = 1;
    } else if let Some(i) = interp {
        // 首参仅为 `in <colorspace>` 提示（无 shape），跳过用默认 shape
        interpolation = i;
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..], false)?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Radial(RadialGradient {
        shape,
        size,
        position_x: pos_x,
        position_y: pos_y,
        stops,
        repeating,
        interpolation,
    }))
}

/// 解析 radial-gradient 的 shape、size 和 at position。
fn parse_radial_shape_and_position(s: &str) -> Option<(RadialShape, RadialSize, LengthValue, LengthValue)> {
    let mut shape = RadialShape::Ellipse;
    let mut size = RadialSize::FarthestCorner;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);

    let lower = s.to_ascii_lowercase();

    // 解析 "at x y" 位置：position-首位（`at x y`，无 shape 前缀）或 `shape/size at x y`。
    let at_pos = if lower.starts_with("at ") {
        Some(0usize)
    } else {
        lower.find(" at ").map(|p| p + 1)
    };
    if let Some(ap) = at_pos {
        // ap 指向 "at" 起始（首位=0；` at ` 命中时=find+1 指向其首空格后的 a）。
        let pos_str = &s[ap + 2..]; // 跳过 "at"
        if let Some((px, py)) = parse_position_pair(pos_str.trim_start()) {
            pos_x = px;
            pos_y = py;
        }
        // 解析 at 之前的部分为 shape/size（首位时为空 → 默认 ellipse farthest-corner）
        let shape_str = s[..ap].trim();
        parse_radial_shape_size(shape_str, &mut shape, &mut size);
    } else {
        parse_radial_shape_size(s, &mut shape, &mut size);
    }

    Some((shape, size, pos_x, pos_y))
}

/// 解析 radial shape 和 size 关键字。
fn parse_radial_shape_size(s: &str, shape: &mut RadialShape, size: &mut RadialSize) {
    let lower = s.trim().to_ascii_lowercase();

    // 检查长度值（如 "50px 100px" 或 "circle 50px"）
    let parts: Vec<&str> = lower.split_whitespace().collect();
    for part in parts {
        match part {
            "circle" => *shape = RadialShape::Circle,
            "ellipse" => *shape = RadialShape::Ellipse,
            "closest-side" => *size = RadialSize::ClosestSide,
            "farthest-side" => *size = RadialSize::FarthestSide,
            "closest-corner" => *size = RadialSize::ClosestCorner,
            "farthest-corner" => *size = RadialSize::FarthestCorner,
            _ => {
                // 尝试解析为长度值
                if let Some(lv) = parse_length(part) {
                    *size = RadialSize::Length(lv);
                }
            }
        }
    }
}

/// 解析位置对（如 "center center"、"50% 50%"、"left top"）。
fn parse_position_pair(s: &str) -> Option<(LengthValue, LengthValue)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();

    match parts.len() {
        1 => {
            let p = parse_position_keyword(parts[0]);
            Some((p.clone(), p))
        }
        2 => {
            let px = parse_position_keyword(parts[0]);
            let py = parse_position_keyword(parts[1]);
            Some((px, py))
        }
        _ => None,
    }
}

/// 解析位置关键字为 LengthValue。
fn parse_position_keyword(s: &str) -> LengthValue {
    match s.to_ascii_lowercase().as_str() {
        "center" => LengthValue::Percentage(50.0),
        "left" => LengthValue::Percentage(0.0),
        "right" => LengthValue::Percentage(100.0),
        "top" => LengthValue::Percentage(0.0),
        "bottom" => LengthValue::Percentage(100.0),
        other => parse_length(other).unwrap_or(LengthValue::Percentage(50.0)),
    }
}

/// 解析 conic-gradient 内部参数。
fn parse_conic_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut from_angle = 0.0;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);
    let mut stop_start = 0;
    let mut interpolation = ColorInterpolation::default();

    let first = args[0].trim();
    let (config_part, interp) = strip_interpolation_hint(first);
    let config_src = config_part.unwrap_or("");
    let config_lower = config_src.to_ascii_lowercase();

    if !config_src.is_empty()
        && (config_lower.starts_with("from ") || config_lower.starts_with("at ") || config_lower.contains(" at "))
    {
        if let Some((angle, px, py)) = parse_conic_config(config_src) {
            from_angle = angle;
            pos_x = px;
            pos_y = py;
        }
        if let Some(i) = interp {
            interpolation = i;
        }
        stop_start = 1;
    } else if let Some(i) = interp {
        // 首参仅为 `in <colorspace>` 提示（无 from/at），跳过用默认配置
        interpolation = i;
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..], true)?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Conic(ConicGradient {
        from_angle,
        position_x: pos_x,
        position_y: pos_y,
        stops,
        repeating,
        interpolation,
    }))
}

/// 解析 conic-gradient 的 from angle 和 at position 配置。
fn parse_conic_config(s: &str) -> Option<(f64, LengthValue, LengthValue)> {
    let mut angle = 0.0;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);

    let lower = s.to_ascii_lowercase();

    // 解析 "from <angle>"
    if let Some(from_pos) = lower.find("from ") {
        let after_from = &s[from_pos + 5..];
        // 找到 from 和 at 之间的部分作为角度
        let at_pos = after_from.to_ascii_lowercase().find(" at ").unwrap_or(after_from.len());
        let angle_str = after_from[..at_pos].trim();
        if !angle_str.is_empty() {
            angle = parse_angle(angle_str).unwrap_or(0.0);
        }
    }

    // 解析 "at <position>"（支持 "from X at Y" 和直接 "at Y"）
    let at_keyword = if lower.starts_with("at ") {
        Some(0)
    } else {
        lower.find(" at ")
    };
    if let Some(at_pos) = at_keyword {
        let pos_str = &s[at_pos + 3..];
        // 在第一个逗号处截断，避免渐变色标干扰位置解析
        let pos_str = pos_str.split(',').next().unwrap_or(pos_str).trim();
        if let Some((px, py)) = parse_position_pair(pos_str) {
            pos_x = px;
            pos_y = py;
        }
    }

    Some((angle, pos_x, pos_y))
}

/// 将渐变参数按顶层逗号分割（不分割括号内的逗号）。
fn split_gradient_args(inner: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let bytes = inner.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                args.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < inner.len() {
        args.push(&inner[start..]);
    }

    Some(args)
}

/// 解析色标列表。
fn parse_color_stops(args: &[&str], is_conic: bool) -> Option<Vec<GradientColorStop>> {
    let mut stops = Vec::new();
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }
        // CSS Images 4 §4.3.8：色标间的裸 <length-percentage> 是 color interpolation hint
        //（插值提示），指定相邻两色标的中点位置，本身不是色标。修复前裸 %/长度落
        // parse_color 失败 → None 经 `?` 传播 → 整个渐变被拒（背景回退，无渐变渲染）。
        // 现正确识别并消费 hint（hint 必须有前导色标，首位裸长度/% 仍非法）。
        // 渲染侧暂用线性插值——hint 中点偏移为可选 follow-up（需 GradientColorStop 加
        // hint 字段 + 渲染器改动）；本切片仅 parse-compliance，与 R2204 CDO/CDC 同族。
        if !stops.is_empty() && is_color_interpolation_hint(arg) {
            continue;
        }
        stops.extend(parse_color_stop(arg, is_conic)?);
    }
    Some(stops)
}

/// 判断 arg 是否为 CSS Images 4 §4.3.8 color interpolation hint（色标间裸 <length-percentage>）：
/// 能解析为色标位置（长度/%/calc/min/max/clamp）但不能解析为颜色（裸值，无 color 前缀）。
/// driving: `linear-gradient(red, 30%, blue)` 中间的 `30%` / `20px` / `calc(25%)`。
fn is_color_interpolation_hint(arg: &str) -> bool {
    parse_stop_position(arg.trim()).is_some() && parse_color(arg.trim()).is_none()
}

/// 解析单个色标（如 `red`、`red 50%`、`#00ff00 10px`、`red calc(1px / 0)`）。
///
/// 返回 1 个或 2 个色标：CSS Images 4 双位置（`red 0% 50%`）展开为两个同色色标
/// `red@0%` + `red@50%`（硬过渡）。多于 2 个位置非法 → None。
///
/// `is_conic` 为真时（conic-gradient），色标位置额外接受 `<angle>`（deg/grad/rad/turn）：
/// CSS Images 4 §4.3.3 规定 conic 色标位置为 `<angle-percentage>`，% 相对 360deg。
/// 角度归一为 Percentage(rad/2π×100)（180deg→50%、360deg→100%），与 conic 渲染的 t∈[0,1) 一致。
/// driving: R2318 css-images multiple-position-color-stop-conic（green 0% 180deg）。
fn parse_color_stop(s: &str, is_conic: bool) -> Option<Vec<GradientColorStop>> {
    let s = s.trim();

    // "color [position] [position]"：在括号深度 0 处切分 color 与位置部分
    //（位置可含空格如 `calc(1px / 0)`，故 split_color_stop_position 已 paren-aware）。
    if let Some((color_str, pos_str)) = split_color_stop_position(s)
        && let Some(color) = parse_color(color_str)
    {
        // 位置部分按顶层空白拆分为 1 或 2 个位置 token（双位置 `0% 50%`；calc 内空格保持一体）。
        let pos_tokens = super::parse_extended_visual::split_top_level_whitespace(pos_str);
        match pos_tokens.len() {
            1 => {
                let position = parse_stop_position_maybe_angle(&pos_tokens[0], is_conic)?;
                return Some(vec![GradientColorStop {
                    color,
                    position: Some(position),
                }]);
            }
            2 => {
                // CSS Images 4 双位置：同色两色标
                let p1 = parse_stop_position_maybe_angle(&pos_tokens[0], is_conic)?;
                let p2 = parse_stop_position_maybe_angle(&pos_tokens[1], is_conic)?;
                return Some(vec![
                    GradientColorStop {
                        color: color.clone(),
                        position: Some(p1),
                    },
                    GradientColorStop {
                        color,
                        position: Some(p2),
                    },
                ]);
            }
            // 0 个位置 token（pos_str 仅空白，不应发生，split_color_stop_position 已守）或 >2（非法）
            _ => return None,
        }
    }

    // 仅颜色
    let color = parse_color(s)?;
    Some(vec![GradientColorStop { color, position: None }])
}

/// 解析色标位置：先试长度/%/calc（[`parse_stop_position`]）；conic 额外接受裸 `<angle>`。
fn parse_stop_position_maybe_angle(s: &str, is_conic: bool) -> Option<LengthValue> {
    if let Some(lv) = parse_stop_position(s) {
        return Some(lv);
    }
    if is_conic {
        if let Some(rad) = parse_angle_to_radians(s.trim()) {
            let pct = rad / (2.0 * std::f64::consts::PI) * 100.0;
            return Some(LengthValue::Percentage(pct));
        }
    }
    None
}

/// 在括号深度 0 处切分 `<color> <position-part>`，返回 `(color, position-part)`。
///
/// 切**首个**顶层空格：颜色部分是首个 token（可能含空格如 `rgb(0 0 0)`，
/// 其空格在 depth 1），位置部分为其后全部（单位置 `50%`、含空格的 `calc(1px / 0)`、
/// 或双位置 `0% 50%`，CSS Images 4）。切首个而非末个空格是双位置的前提
///（末个空格会把 `red 0% 50%` 切成 `("red 0%", "50%")` 致颜色解析失败）。
fn split_color_stop_position(s: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ' ' if depth == 0 => {
                let color = s[..i].trim();
                let pos = s[i..].trim();
                if color.is_empty() || pos.is_empty() {
                    return None;
                }
                return Some((color, pos));
            }
            _ => {}
        }
    }
    None
}

/// 解析渐变色标位置：长度或 calc/min/max/clamp 数学函数（→ `LengthValue::Calc` 延迟求值）。
/// driving: css-images gradient-infinity（`calc(1px / 0)` / `calc(Infinity * 1px)` 色标位置）。
fn parse_stop_position(s: &str) -> Option<LengthValue> {
    if let Some(lv) = parse_length(s) {
        return Some(lv);
    }
    let t = s.trim();
    let is_math = ["calc(", "min(", "max(", "clamp("].iter().any(|p| t.starts_with(p));
    if is_math {
        return parse_math_function(t).map(|e| LengthValue::Calc(Box::new(e)));
    }
    None
}

/// 解析 CSS grid-area 简写属性值。
///
/// 支持格式：
/// - 单值：`"header"` → 四个值均为 `"header"`
/// - `"auto"` → 四个值均为 `"auto"`
/// - 四值斜杠分隔：`"1 / 2 / 3 / 4"` → `("1", "2", "3", "4")`
/// - 两值斜杠分隔：`"1 / 3"` → `("1", "auto", "3", "auto")`
/// - 三值斜杠分隔：`"1 / 2 / 3"` → `("1", "2", "3", "auto")`
///
/// 返回 `(row_start, row_end, col_start, col_end)` 原始字符串元组，
/// 由 style-system 调用 `parse_grid_line` 转换为 `GridLineValue`。
pub fn parse_grid_area(input: &str) -> Option<(String, String, String, String)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // 包含斜杠 → 按斜杠分割
    if input.contains('/') {
        let parts: Vec<&str> = input.split('/').map(|s| s.trim()).collect();
        match parts.len() {
            1 => {
                // 单值（斜杠后为空，不合法）
                let v = parts[0].to_string();
                if v.is_empty() {
                    return None;
                }
                Some((v.clone(), v.clone(), v.clone(), v))
            }
            2 => {
                // row-start / col-start
                let rs = parts[0].to_string();
                let cs = parts[1].to_string();
                if rs.is_empty() || cs.is_empty() {
                    return None;
                }
                Some((rs, "auto".to_string(), cs, "auto".to_string()))
            }
            3 => {
                // row-start / row-end / col-start
                let rs = parts[0].to_string();
                let re = parts[1].to_string();
                let cs = parts[2].to_string();
                if rs.is_empty() || re.is_empty() || cs.is_empty() {
                    return None;
                }
                Some((rs, re, cs, "auto".to_string()))
            }
            4 => {
                // row-start / row-end / col-start / col-end
                let rs = parts[0].to_string();
                let re = parts[1].to_string();
                let cs = parts[2].to_string();
                let ce = parts[3].to_string();
                if rs.is_empty() || re.is_empty() || cs.is_empty() || ce.is_empty() {
                    return None;
                }
                Some((rs, re, cs, ce))
            }
            _ => None,
        }
    } else {
        // 单值，所有四个都设为同一值
        let v = input.to_string();
        Some((v.clone(), v.clone(), v.clone(), v))
    }
}

/// CSS text-shadow 值。
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadowValue {
    /// 水平偏移量。
    pub offset_x: LengthValue,
    /// 垂直偏移量。
    pub offset_y: LengthValue,
    /// 模糊半径。
    pub blur_radius: LengthValue,
    /// 阴影颜色。
    pub color: ColorValue,
}

/// 解析 CSS text-shadow 值。
///
/// 格式：`"none"` | `"<color>? && <offset-x> <offset-y> [<blur-radius>] && <color>?"`。
pub fn parse_text_shadow(value: &str) -> Option<TextShadowValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(TextShadowValue {
            offset_x: LengthValue::Px(0.0),
            offset_y: LengthValue::Px(0.0),
            blur_radius: LengthValue::Px(0.0),
            color: ColorValue::Rgba(0, 0, 0, 255),
        });
    }
    // R2477：CSS Text Decoration §3 `<shadow> = <length>{2,3} && <color>?` —— `&&` 组合子
    // 表示长度与颜色可任意顺序（`red 2px 2px`、`2px red 2px`、`2px 2px red` 均合法）。
    // 旧实现按固定下标 parts[0..1]=长度、parts[2/3]=模糊/颜色，致颜色在前/中 → parse_length
    // 失败整条丢，或 `2px 2px red 4px` 中 red 占模糊槽（unwrap_or 0）、4px 被静默丢。
    // 现用括号感知拆 token，先抽首个可解析为颜色的 token 作 color，剩余必须全为长度并
    // 按序映射 ox/oy/blur（第二/第三个颜色会因 parse_length 失败被拒，符合 spec 至多一色）。
    // driving: css-pseudo/marker-text-shadow `#0f0 1px 2px 3px`、selectors/focus-within
    // `text-shadow: black 0px 0px 0px`（颜色在前）。与 R2476 inset-anywhere 同族。
    let owned = split_paren_aware_tokens(v);
    let mut color = ColorValue::CurrentColor;
    let mut color_found = false;
    let lengths: Vec<&str> = owned
        .iter()
        .filter_map(|s| {
            if !color_found && let Some(c) = parse_color(s) {
                color = c;
                color_found = true;
                return None;
            }
            Some(s.as_str())
        })
        .collect();
    if !(2..=3).contains(&lengths.len()) {
        return None;
    }
    let ox = parse_length(lengths[0])?;
    let oy = parse_length(lengths[1])?;
    let blur = if lengths.len() == 3 {
        parse_length(lengths[2])?
    } else {
        LengthValue::Px(0.0)
    };
    Some(TextShadowValue {
        offset_x: ox,
        offset_y: oy,
        blur_radius: blur,
        color,
    })
}

/// 解析 text-shadow 多阴影列表（CSS Text Decoration §3：`none | <shadow>#`）。
/// `none` → 空 Vec；否则顶层逗号分割（paren-aware，`rgb()`/`rgba()` 内部逗号保持一体）
/// 后逐个 parse_text_shadow，任一失败 → None。语义镜像 parse_box_shadow_list。
pub fn parse_text_shadow_list(value: &str) -> Option<Vec<TextShadowValue>> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let parts = split_top_level_commas(v);
    if parts.is_empty() {
        return None;
    }
    let mut shadows = Vec::with_capacity(parts.len());
    for p in &parts {
        shadows.push(parse_text_shadow(p)?);
    }
    Some(shadows)
}

/// CSS box-shadow 单个阴影。
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadowValue {
    /// 水平偏移量。
    pub offset_x: LengthValue,
    /// 垂直偏移量。
    pub offset_y: LengthValue,
    /// 模糊半径。
    pub blur_radius: LengthValue,
    /// 扩展半径。
    pub spread_radius: LengthValue,
    /// 阴影颜色。
    pub color: ColorValue,
    /// 是否为内阴影。
    pub inset: bool,
}

/// 按空白分割 CSS 值，但不在括号内分割——保留 `rgba(0, 0, 0, 0.08)`、
/// `hsla(...)`、`var(...)` 等含内部空白的函数为单个 token。
///
/// 通用工具：box-shadow / text-shadow（parse_transform.rs）与 border / outline /
/// column-rule 简写（style-system）共享。此前各处用 `split_whitespace()` 会把
/// `rgba(0, 0, 0, 0.08)` 拆成碎片，导致颜色解析失败并回退为默认实心黑（alpha=255），
/// 使 welcome.html 等用标准带空格 rgba 的页面渲染出大面积实心黑阴影/边框
/// （DC-13 welcome.html 51.59% 差距主因）。
pub fn split_paren_aware_tokens(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            c if c.is_whitespace() => {
                if depth == 0 {
                    if !cur.is_empty() {
                        tokens.push(std::mem::take(&mut cur));
                    }
                } else {
                    cur.push(c);
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// 解析 CSS box-shadow 值。
///
/// 格式：`"none"` | `"[inset] <offset-x> <offset-y> [<blur>] [<spread>] [<color>]"`。
pub fn parse_box_shadow(value: &str) -> Option<BoxShadowValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(BoxShadowValue {
            offset_x: LengthValue::Px(0.0),
            offset_y: LengthValue::Px(0.0),
            blur_radius: LengthValue::Px(0.0),
            spread_radius: LengthValue::Px(0.0),
            color: ColorValue::Rgba(0, 0, 0, 255),
            inset: false,
        });
    }
    // R2476：`inset` 关键字可在值任意位置（前/中/后），CSS Backgrounds §7.1。旧实现仅
    // starts_with("inset") 致 `black 10px 10px 0px 0px inset`（inset 在末尾）漏识别 →
    // inset=false 渲为 outset。扫全部 token 提取 inset 并从 parts 移除。
    // R2477：颜色同样可在任意位置（同 `<length>{2,4} && <color>?` 的 `&&`），如
    // `rgba(0,255,0,1) 10px 10px`（css-backgrounds/box-shadow-005）。旧 find_map 已能找
    // 任意位颜色，但颜色占 parts[0..1] 时 parse_length 失败整条丢、占 parts[2/3] 时被
    // 当 blur/spread（unwrap_or 0）致真实长度被静默丢。现先抽 inset + 首个颜色，剩余必须
    // 全为长度并按序映射 ox/oy/blur/spread（spec：至多一色一 inset）。
    let owned = split_paren_aware_tokens(v);
    let mut inset = false;
    let mut color = ColorValue::CurrentColor;
    let mut color_found = false;
    let lengths: Vec<&str> = owned
        .iter()
        .filter_map(|s| {
            if s.eq_ignore_ascii_case("inset") {
                inset = true;
                return None;
            }
            if !color_found && let Some(c) = parse_color(s) {
                color = c;
                color_found = true;
                return None;
            }
            Some(s.as_str())
        })
        .collect();
    if !(2..=4).contains(&lengths.len()) {
        return None;
    }
    let ox = parse_length(lengths[0])?;
    let oy = parse_length(lengths[1])?;
    let blur = if lengths.len() >= 3 {
        parse_length(lengths[2])?
    } else {
        LengthValue::Px(0.0)
    };
    let spread = if lengths.len() >= 4 {
        parse_length(lengths[3])?
    } else {
        LengthValue::Px(0.0)
    };
    Some(BoxShadowValue {
        offset_x: ox,
        offset_y: oy,
        blur_radius: blur,
        spread_radius: spread,
        color,
        inset,
    })
}

/// 按顶层逗号分割（paren-aware：使 `rgb(0, 0, 0)` / `hsl(...)` 等含逗号函数保持一体）。
/// 用于 box-shadow / text-shadow 多值列表拆分。空白修剪；空段丢弃。
fn split_top_level_commas(s: &str) -> Vec<String> {
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
            ',' if depth == 0 => {
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

/// 解析 box-shadow 多阴影列表（CSS Backgrounds §7.2：<shadow>#）。
/// `none` → 空 Vec；否则顶层逗号分割后逐个 parse_box_shadow（任一失败 → None）。
pub fn parse_box_shadow_list(value: &str) -> Option<Vec<BoxShadowValue>> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let parts = split_top_level_commas(v);
    if parts.is_empty() {
        return None;
    }
    let mut shadows = Vec::with_capacity(parts.len());
    for p in &parts {
        shadows.push(parse_box_shadow(p)?);
    }
    Some(shadows)
}
