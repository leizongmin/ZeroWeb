//! CSS 属性值类型。
//!
//! 定义 CSS 属性值的类型化表示，以及解析函数。

/// CSS 长度值。
#[derive(Debug, Clone, PartialEq)]
pub enum LengthValue {
    /// 绝对长度（px）。
    Px(f64),
    /// em 单位。
    Em(f64),
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
    /// 百分比值（0-100）。
    Percentage(f64),
    /// auto 关键字。
    Auto,
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

/// 解析 CSS calc() 表达式。
///
/// 支持格式如 `"calc(100% - 20px)"`、`"calc(50% + 10px)"`、`"calc(2 * 10px)"`。
/// 目前仅处理单次二元运算（左 操作数 右）。
pub fn parse_calc(value: &str) -> Option<CalcExpr> {
    let value = value.trim();

    // 检查 calc(...) 包装
    if !value.starts_with("calc(") || !value.ends_with(')') {
        return None;
    }

    let inner = value.get(5..value.len() - 1)?.trim();

    // 尝试查找运算符：+、-、*、/
    // 按优先级搜索 *、/ 优先于 +、-
    // 简单实现：搜索第一个运算符
    if let Some((op_pos, op)) = find_calc_operator(inner) {
        let left_str = inner[..op_pos].trim();
        let right_str = inner[op_pos + 1..].trim();

        let left = parse_calc_operand(left_str)?;
        let right = parse_calc_operand(right_str)?;

        Some(CalcExpr::BinaryOp(Box::new(left), op, Box::new(right)))
    } else {
        // 单个操作数
        parse_calc_operand(inner)
    }
}

/// 在 calc 表达式内部查找运算符位置。
///
/// 返回第一个匹配的运算符及其位置。
fn find_calc_operator(s: &str) -> Option<(usize, CalcOp)> {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'+' => return Some((i, CalcOp::Add)),
            b'-' => {
                // 跳过负号（前面紧跟着运算符或位于开头）
                if i == 0 {
                    continue;
                }
                // 前一个非空白字符必须是数字、%或字母(单位)
                let prev_non_ws = s[..i].chars().rev().find(|c| !c.is_ascii_whitespace());
                if let Some(prev) = prev_non_ws
                    && (prev.is_ascii_digit() || prev == '%' || prev.is_ascii_alphabetic())
                {
                    return Some((i, CalcOp::Subtract));
                }
            }
            b'*' => return Some((i, CalcOp::Multiply)),
            b'/' => return Some((i, CalcOp::Divide)),
            _ => {}
        }
    }
    None
}

/// 解析 calc 操作数（数值或长度值）。
fn parse_calc_operand(s: &str) -> Option<CalcExpr> {
    let s = s.trim();

    // 尝试解析为纯数字（无单位）
    if let Ok(num) = s.parse::<f64>() {
        return Some(CalcExpr::Number(num));
    }

    // 尝试解析为长度值
    if let Some(length) = parse_length(s) {
        return Some(CalcExpr::Length(length));
    }

    None
}

/// 计算 CSS calc() 表达式的像素值。
///
/// `parent_length` 用于解析百分比值（如 `100%` = `parent_length`）。
/// 返回计算结果（像素）。
pub fn eval_calc(expr: &CalcExpr, parent_length: Option<f64>) -> Option<f64> {
    match expr {
        CalcExpr::Number(n) => Some(*n),
        CalcExpr::Length(lv) => match lv {
            LengthValue::Px(v) => Some(*v),
            LengthValue::Percentage(pct) => parent_length.map(|pl| pct / 100.0 * pl),
            _ => None, // 其他单位需要额外上下文，暂不支持
        },
        CalcExpr::BinaryOp(left, op, right) => {
            let lv = eval_calc(left, parent_length)?;
            let rv = eval_calc(right, parent_length)?;
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
    }
}

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()` 和 `hsl()`/`hsla()` 函数。
pub fn parse_color(value: &str) -> Option<ColorValue> {
    let value = value.trim();

    // 特殊关键字
    if value.eq_ignore_ascii_case("transparent") {
        return Some(ColorValue::Transparent);
    }
    if value.eq_ignore_ascii_case("currentColor") || value == "currentcolor" {
        return Some(ColorValue::CurrentColor);
    }

    // 十六进制颜色
    if value.starts_with('#') {
        return parse_hex_color(value);
    }

    // rgb() / rgba() 函数
    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        return parse_rgb_function(value);
    }

    // hsl() / hsla() 函数
    if value.starts_with("hsl(") || value.starts_with("hsla(") {
        return parse_hsl_function(value);
    }

    // 命名颜色
    parse_named_color(value)
}

/// 解析十六进制颜色。
fn parse_hex_color(value: &str) -> Option<ColorValue> {
    let hex = &value[1..]; // 去掉 #
    match hex.len() {
        3 => {
            // #RGB → RRGGBB
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        4 => {
            // #RGBA → RRGGBBAA
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let c3 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            let a = hex_char_to_byte(c3, c3);
            Some(ColorValue::Rgba(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// 将两个十六进制字符合并为一个字节（重复单字符，如 'f' → 0xFF）。
fn hex_char_to_byte(c1: char, c2: char) -> u8 {
    let s = format!("{}{}", c1, c2);
    u8::from_str_radix(&s, 16).unwrap_or(0)
}

/// 解析 rgb() / rgba() 函数。
fn parse_rgb_function(value: &str) -> Option<ColorValue> {
    // 提取括号内的内容
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let r = parse_color_component(parts[0].trim())?;
    let g = parse_color_component(parts[1].trim())?;
    let b = parse_color_component(parts[2].trim())?;
    let a = if parts.len() > 3 {
        parse_alpha_component(parts[3].trim())?
    } else {
        255u8
    };

    Some(ColorValue::Rgba(r, g, b, a))
}

/// 解析颜色分量（0-255 或 0%-100%）。
fn parse_color_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 alpha 分量（0-1 或 0%-100%）。
fn parse_alpha_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some((v * 255.0).round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 hsl() / hsla() 函数。
fn parse_hsl_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let h: f64 = parts[0].trim().trim_end_matches("deg").parse().ok()?;
    let s: f64 = parts[1].trim().trim_end_matches('%').parse().ok()?;
    let l: f64 = parts[2].trim().trim_end_matches('%').parse().ok()?;
    let a = if parts.len() > 3 {
        parts[3].trim().parse().ok()?
    } else {
        1.0
    };

    Some(ColorValue::Hsla(h, s, l, a))
}

/// 解析命名颜色。
///
/// 支持至少 16 种基本 CSS 颜色。
fn parse_named_color(value: &str) -> Option<ColorValue> {
    // 基本 CSS 颜色映射
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "black" => Some(ColorValue::Rgba(0, 0, 0, 255)),
        "white" => Some(ColorValue::Rgba(255, 255, 255, 255)),
        "red" => Some(ColorValue::Rgba(255, 0, 0, 255)),
        "green" => Some(ColorValue::Rgba(0, 128, 0, 255)),
        "blue" => Some(ColorValue::Rgba(0, 0, 255, 255)),
        "yellow" => Some(ColorValue::Rgba(255, 255, 0, 255)),
        "cyan" | "aqua" => Some(ColorValue::Rgba(0, 255, 255, 255)),
        "magenta" | "fuchsia" => Some(ColorValue::Rgba(255, 0, 255, 255)),
        "silver" => Some(ColorValue::Rgba(192, 192, 192, 255)),
        "gray" | "grey" => Some(ColorValue::Rgba(128, 128, 128, 255)),
        "maroon" => Some(ColorValue::Rgba(128, 0, 0, 255)),
        "olive" => Some(ColorValue::Rgba(128, 128, 0, 255)),
        "lime" => Some(ColorValue::Rgba(0, 255, 0, 255)),
        "teal" => Some(ColorValue::Rgba(0, 128, 128, 255)),
        "navy" => Some(ColorValue::Rgba(0, 0, 128, 255)),
        "purple" => Some(ColorValue::Rgba(128, 0, 128, 255)),
        "orange" => Some(ColorValue::Rgba(255, 165, 0, 255)),
        _ => Some(ColorValue::Named(value.to_string())),
    }
}

/// 解析 CSS 长度值。
///
/// 支持格式如 `"10px"`、`"1.5em"`、`"2rem"`、`"100vh"`、`"50%"`、`"auto"` 等。
pub fn parse_length(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 处理 auto 关键字
    if value.eq_ignore_ascii_case("auto") {
        return Some(LengthValue::Auto);
    }

    // 找到数字部分的结束位置
    let num_end = value
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(value.len());

    let num_str = &value[..num_end];
    let unit = &value[num_end..];

    let num: f64 = num_str.parse().ok()?;

    match unit {
        "px" => Some(LengthValue::Px(num)),
        "em" => Some(LengthValue::Em(num)),
        "rem" => Some(LengthValue::Rem(num)),
        "vh" => Some(LengthValue::Vh(num)),
        "vw" => Some(LengthValue::Vw(num)),
        "vmin" => Some(LengthValue::Vmin(num)),
        "vmax" => Some(LengthValue::Vmax(num)),
        "ch" => Some(LengthValue::Ch(num)),
        "%" => Some(LengthValue::Percentage(num)),
        _ => None,
    }
}

/// 解析 CSS display 属性值。
pub fn parse_display(value: &str) -> Option<DisplayValue> {
    match value.trim() {
        "block" => Some(DisplayValue::Block),
        "inline" => Some(DisplayValue::Inline),
        "inline-block" => Some(DisplayValue::InlineBlock),
        "flex" => Some(DisplayValue::Flex),
        "inline-flex" => Some(DisplayValue::InlineFlex),
        "grid" => Some(DisplayValue::Grid),
        "inline-grid" => Some(DisplayValue::InlineGrid),
        "none" => Some(DisplayValue::None),
        "contents" => Some(DisplayValue::Contents),
        "flow" => Some(DisplayValue::Flow),
        "flow-root" => Some(DisplayValue::FlowRoot),
        "list-item" => Some(DisplayValue::ListItem),
        _ => None,
    }
}

/// 解析 CSS position 属性值。
pub fn parse_position(value: &str) -> Option<PositionValue> {
    match value.trim() {
        "static" => Some(PositionValue::Static),
        "relative" => Some(PositionValue::Relative),
        "absolute" => Some(PositionValue::Absolute),
        "fixed" => Some(PositionValue::Fixed),
        "sticky" => Some(PositionValue::Sticky),
        _ => None,
    }
}

/// 解析 CSS overflow 属性值。
pub fn parse_overflow(value: &str) -> Option<OverflowValue> {
    match value.trim() {
        "visible" => Some(OverflowValue::Visible),
        "hidden" => Some(OverflowValue::Hidden),
        "scroll" => Some(OverflowValue::Scroll),
        "auto" => Some(OverflowValue::Auto),
        "clip" => Some(OverflowValue::Clip),
        _ => None,
    }
}

/// 解析 CSS flex-direction 属性值。
pub fn parse_flex_direction(value: &str) -> Option<FlexDirectionValue> {
    match value.trim() {
        "row" => Some(FlexDirectionValue::Row),
        "row-reverse" => Some(FlexDirectionValue::RowReverse),
        "column" => Some(FlexDirectionValue::Column),
        "column-reverse" => Some(FlexDirectionValue::ColumnReverse),
        _ => None,
    }
}

/// 解析 CSS flex-wrap 属性值。
pub fn parse_flex_wrap(value: &str) -> Option<FlexWrapValue> {
    match value.trim() {
        "nowrap" => Some(FlexWrapValue::Nowrap),
        "wrap" => Some(FlexWrapValue::Wrap),
        "wrap-reverse" => Some(FlexWrapValue::WrapReverse),
        _ => None,
    }
}

/// 解析 CSS justify-content / align-items 属性值。
pub fn parse_alignment(value: &str) -> Option<AlignmentValue> {
    match value.trim() {
        "flex-start" => Some(AlignmentValue::FlexStart),
        "flex-end" => Some(AlignmentValue::FlexEnd),
        "center" => Some(AlignmentValue::Center),
        "space-between" => Some(AlignmentValue::SpaceBetween),
        "space-around" => Some(AlignmentValue::SpaceAround),
        "space-evenly" => Some(AlignmentValue::SpaceEvenly),
        "stretch" => Some(AlignmentValue::Stretch),
        "start" => Some(AlignmentValue::Start),
        "end" => Some(AlignmentValue::End),
        "baseline" => Some(AlignmentValue::Baseline),
        _ => None,
    }
}

/// 解析 CSS box-sizing 属性值。
pub fn parse_box_sizing(value: &str) -> Option<BoxSizingValue> {
    match value.trim() {
        "content-box" => Some(BoxSizingValue::ContentBox),
        "border-box" => Some(BoxSizingValue::BorderBox),
        _ => None,
    }
}

/// 解析 CSS visibility 属性值。
pub fn parse_visibility(value: &str) -> Option<VisibilityValue> {
    match value.trim() {
        "visible" => Some(VisibilityValue::Visible),
        "hidden" => Some(VisibilityValue::Hidden),
        "collapse" => Some(VisibilityValue::Collapse),
        _ => None,
    }
}

/// 解析 CSS font-weight 属性值。
pub fn parse_font_weight(value: &str) -> Option<FontWeightValue> {
    match value.trim() {
        "bold" => Some(FontWeightValue::Bold),
        "normal" => Some(FontWeightValue::Normal),
        "bolder" => Some(FontWeightValue::Bolder),
        "lighter" => Some(FontWeightValue::Lighter),
        s => {
            let w: u16 = s.parse().ok()?;
            if (100..=900).contains(&w) {
                Some(FontWeightValue::Absolute(w))
            } else {
                None
            }
        }
    }
}

/// 解析 CSS font-style 属性值。
pub fn parse_font_style(value: &str) -> Option<FontStyleValue> {
    let value = value.trim();
    if value == "normal" {
        Some(FontStyleValue::Normal)
    } else if value == "italic" {
        Some(FontStyleValue::Italic)
    } else if value.starts_with("oblique") {
        let angle_str = value.strip_prefix("oblique")?.trim();
        if angle_str.is_empty() {
            Some(FontStyleValue::Oblique(None))
        } else {
            // 处理 "(angle)" 或 "(angledeg)" 形式
            let angle_str = angle_str
                .strip_prefix('(')
                .unwrap_or(angle_str)
                .strip_suffix(')')
                .unwrap_or(angle_str);
            let angle: f64 = angle_str.trim_end_matches("deg").trim().parse().ok()?;
            Some(FontStyleValue::Oblique(Some(angle)))
        }
    } else {
        None
    }
}

/// 解析 CSS var() 函数引用。
///
/// 支持格式如 `var(--name)` 和 `var(--name, fallback)`。
pub fn parse_var(value: &str) -> Option<VarReference> {
    let value = value.trim();

    // 检查是否以 var( 开头
    if !value.starts_with("var(") || !value.ends_with(')') {
        return None;
    }

    // 提取括号内的内容
    let inner = value.get(4..value.len() - 1)?.trim();

    // 找到逗号（如果有）
    if let Some(comma_pos) = inner.find(',') {
        let name = inner[..comma_pos].trim().to_string();
        let fallback = inner[comma_pos + 1..].trim().to_string();
        Some(VarReference {
            name,
            fallback: Some(fallback),
        })
    } else {
        Some(VarReference {
            name: inner.to_string(),
            fallback: None,
        })
    }
}

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
    match value.trim() {
        "normal" => Some(AnimationDirectionValue::Normal),
        "reverse" => Some(AnimationDirectionValue::Reverse),
        "alternate" => Some(AnimationDirectionValue::Alternate),
        "alternate-reverse" => Some(AnimationDirectionValue::AlternateReverse),
        _ => None,
    }
}

/// 解析 CSS animation-fill-mode 值。
pub fn parse_animation_fill_mode(value: &str) -> Option<AnimationFillModeValue> {
    match value.trim() {
        "none" => Some(AnimationFillModeValue::None),
        "forwards" => Some(AnimationFillModeValue::Forwards),
        "backwards" => Some(AnimationFillModeValue::Backwards),
        "both" => Some(AnimationFillModeValue::Both),
        _ => None,
    }
}

/// 解析 CSS animation-play-state 值。
pub fn parse_animation_play_state(value: &str) -> Option<AnimationPlayStateValue> {
    match value.trim() {
        "running" => Some(AnimationPlayStateValue::Running),
        "paused" => Some(AnimationPlayStateValue::Paused),
        _ => None,
    }
}

/// 解析 CSS transition-timing-function 值。
pub fn parse_timing_function(value: &str) -> Option<TimingFunctionValue> {
    let value = value.trim();

    match value {
        "ease" => Some(TimingFunctionValue::Ease),
        "linear" => Some(TimingFunctionValue::Linear),
        "ease-in" => Some(TimingFunctionValue::EaseIn),
        "ease-out" => Some(TimingFunctionValue::EaseOut),
        "ease-in-out" => Some(TimingFunctionValue::EaseInOut),
        "step-start" => Some(TimingFunctionValue::StepStart),
        "step-end" => Some(TimingFunctionValue::StepEnd),
        _ if value.starts_with("cubic-bezier(") => {
            let inner = extract_parens_content(value, "cubic-bezier")?;
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
            let inner = extract_parens_content(value, "steps")?;
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
    match s.trim() {
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
/// 返回秒为单位的 f64 值。
pub fn parse_time(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.ends_with("ms") {
        let ms: f64 = value.trim_end_matches("ms").trim().parse().ok()?;
        Some(ms / 1000.0)
    } else if value.ends_with('s') {
        let s: f64 = value.trim_end_matches('s').trim().parse().ok()?;
        Some(s)
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

        // 读取函数名
        let name_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
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
        if let Some(func) = parse_transform_function(name, args_str) {
            functions.push(func);
        } else {
            return None;
        }
    }

    if functions.is_empty() {
        None
    } else {
        Some(TransformValue::List(functions))
    }
}

/// 解析单个变换函数。
fn parse_transform_function(name: &str, args: &str) -> Option<TransformFunction> {
    match name {
        "translate" => {
            let vals = parse_transform_args(args)?;
            let tx = vals.first().copied()?;
            let ty = vals.get(1).copied().unwrap_or(0.0);
            Some(TransformFunction::Translate(tx, ty))
        }
        "translateX" => {
            let vals = parse_transform_args(args)?;
            let tx = vals.first().copied()?;
            Some(TransformFunction::TranslateX(tx))
        }
        "translateY" => {
            let vals = parse_transform_args(args)?;
            let ty = vals.first().copied()?;
            Some(TransformFunction::TranslateY(ty))
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
        "scaleX" => {
            let vals = parse_transform_args(args)?;
            let sx = vals.first().copied()?;
            Some(TransformFunction::ScaleX(sx))
        }
        "scaleY" => {
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
        if let Some(val) = parse_css_number(part) {
            result.push(val);
        } else {
            return None;
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// 解析 CSS 数值（可能带 px/deg/rad/turn 等单位）。
///
/// 返回原始数值（px 直接返回数值，deg 转为度数）。
fn parse_css_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with("deg") {
        s.trim_end_matches("deg").trim().parse::<f64>().ok()
    } else if s.ends_with("rad") {
        let rad: f64 = s.trim_end_matches("rad").trim().parse().ok()?;
        Some(rad.to_degrees())
    } else if s.ends_with("turn") {
        let turn: f64 = s.trim_end_matches("turn").trim().parse().ok()?;
        Some(turn * 360.0)
    } else if s.ends_with("px") || s.ends_with("em") || s.ends_with("rem") {
        // 对于 translate，返回数值部分
        let num_end = s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')?;
        s[..num_end].parse::<f64>().ok()
    } else {
        s.parse::<f64>().ok()
    }
}

/// 解析角度值（返回度数）。
fn parse_angle(s: &str) -> Option<f64> {
    parse_css_number(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_timing_function ──

    #[test]
    fn test_parse_timing_function_keywords() {
        assert_eq!(
            parse_timing_function("ease"),
            Some(TimingFunctionValue::Ease)
        );
        assert_eq!(
            parse_timing_function("linear"),
            Some(TimingFunctionValue::Linear)
        );
        assert_eq!(
            parse_timing_function("ease-in"),
            Some(TimingFunctionValue::EaseIn)
        );
        assert_eq!(
            parse_timing_function("ease-out"),
            Some(TimingFunctionValue::EaseOut)
        );
        assert_eq!(
            parse_timing_function("ease-in-out"),
            Some(TimingFunctionValue::EaseInOut)
        );
        assert_eq!(
            parse_timing_function("step-start"),
            Some(TimingFunctionValue::StepStart)
        );
        assert_eq!(
            parse_timing_function("step-end"),
            Some(TimingFunctionValue::StepEnd)
        );
    }

    #[test]
    fn test_parse_timing_function_cubic_bezier() {
        let result = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)");
        assert_eq!(
            result,
            Some(TimingFunctionValue::CubicBezier(0.25, 0.1, 0.25, 1.0))
        );
    }

    #[test]
    fn test_parse_timing_function_steps() {
        assert_eq!(
            parse_timing_function("steps(4)"),
            Some(TimingFunctionValue::Steps(4, None))
        );
        assert_eq!(
            parse_timing_function("steps(4, end)"),
            Some(TimingFunctionValue::Steps(4, Some(StepPosition::End)))
        );
        assert_eq!(
            parse_timing_function("steps(4, start)"),
            Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
        );
        assert_eq!(
            parse_timing_function("steps(2, jump-both)"),
            Some(TimingFunctionValue::Steps(2, Some(StepPosition::Both)))
        );
    }

    #[test]
    fn test_parse_timing_function_invalid() {
        assert_eq!(parse_timing_function("invalid"), None);
    }

    // ── parse_time ──

    #[test]
    fn test_parse_time_seconds() {
        assert_eq!(parse_time("0.3s"), Some(0.3));
        assert_eq!(parse_time("1s"), Some(1.0));
        assert_eq!(parse_time("2.5s"), Some(2.5));
    }

    #[test]
    fn test_parse_time_milliseconds() {
        assert_eq!(parse_time("200ms"), Some(0.2));
        assert_eq!(parse_time("1000ms"), Some(1.0));
        assert_eq!(parse_time("50ms"), Some(0.05));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(parse_time("10"), None);
        assert_eq!(parse_time("abc"), None);
    }

    #[test]
    fn test_parse_time_zero() {
        assert_eq!(parse_time("0s"), Some(0.0));
        assert_eq!(parse_time("0ms"), Some(0.0));
    }

    // ── parse_calc ──

    #[test]
    fn test_parse_calc_percentage_minus_px() {
        let expr = parse_calc("calc(100% - 20px)");
        let expr = expr.expect("should parse calc(100% - 20px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(100.0)));
                assert_eq!(*op, CalcOp::Subtract);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(20.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_percentage_plus_px() {
        let expr = parse_calc("calc(50% + 10px)");
        let expr = expr.expect("should parse calc(50% + 10px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(50.0)));
                assert_eq!(*op, CalcOp::Add);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_multiply() {
        let expr = parse_calc("calc(2 * 10px)");
        let expr = expr.expect("should parse calc(2 * 10px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Number(2.0));
                assert_eq!(*op, CalcOp::Multiply);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_divide() {
        let expr = parse_calc("calc(100px / 2)");
        let expr = expr.expect("should parse calc(100px / 2)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Px(100.0)));
                assert_eq!(*op, CalcOp::Divide);
                assert_eq!(**right, CalcExpr::Number(2.0));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_eval_calc_percentage_minus_px() {
        let expr = parse_calc("calc(100% - 20px)").unwrap();
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(180.0));
    }

    #[test]
    fn test_eval_calc_percentage_plus_px() {
        let expr = parse_calc("calc(50% + 10px)").unwrap();
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(110.0));
    }

    #[test]
    fn test_eval_calc_multiply() {
        let expr = parse_calc("calc(2 * 10px)").unwrap();
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(20.0));
    }

    #[test]
    fn test_eval_calc_divide() {
        let expr = parse_calc("calc(100px / 2)").unwrap();
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(50.0));
    }

    #[test]
    fn test_parse_calc_invalid() {
        assert_eq!(parse_calc("calc()"), None);
        assert_eq!(parse_calc("calc("), None);
        assert_eq!(parse_calc("not-a-calc"), None);
        assert_eq!(parse_calc(""), None);
    }

    #[test]
    fn test_eval_calc_percentage_without_parent() {
        let expr = parse_calc("calc(50% + 10px)").unwrap();
        // 百分比没有 parent_length，应返回 None
        assert_eq!(eval_calc(&expr, None), None);
    }
}
