//! CSS 扩展属性解析（文本、表格、交互、计数器、背景、边框图像等）。

use super::*;

// ── CSS text-overflow / table / caption / border-collapse / resize 值类型 ──

/// CSS text-overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextOverflowValue {
    /// clip（默认值）— 裁剪溢出内容。
    Clip,
    /// ellipsis — 显示省略号。
    Ellipsis,
    /// 自定义字符串。
    String(String),
}

/// 解析 CSS text-overflow 属性值。
///
/// 支持 `clip`、`ellipsis` 和自定义字符串（带引号）。
pub fn parse_text_overflow(value: &str) -> Option<TextOverflowValue> {
    let v = value.trim();
    match v {
        "clip" => Some(TextOverflowValue::Clip),
        "ellipsis" => Some(TextOverflowValue::Ellipsis),
        s => {
            // 支持引号包裹的自定义字符串，如 `"…"` 或 `'...'`
            if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
                || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
            {
                let inner = &s[1..s.len() - 1];
                if inner.is_empty() {
                    return None;
                }
                Some(TextOverflowValue::String(inner.to_string()))
            } else {
                None
            }
        }
    }
}

/// CSS table-layout 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TableLayoutValue {
    /// auto（默认值）— 自动表格布局。
    Auto,
    /// fixed — 固定表格布局。
    Fixed,
}

/// 解析 CSS table-layout 属性值。
pub fn parse_table_layout(value: &str) -> Option<TableLayoutValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TableLayoutValue::Auto),
        "fixed" => Some(TableLayoutValue::Fixed),
        _ => None,
    }
}

/// CSS caption-side 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaptionSideValue {
    /// top（默认值）— 标题在表格上方。
    Top,
    /// bottom — 标题在表格下方。
    Bottom,
}

/// 解析 CSS caption-side 属性值。
pub fn parse_caption_side(value: &str) -> Option<CaptionSideValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "top" => Some(CaptionSideValue::Top),
        "bottom" => Some(CaptionSideValue::Bottom),
        _ => None,
    }
}

/// CSS border-collapse 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderCollapseValue {
    /// separate（默认值）— 分离边框模型。
    Separate,
    /// collapse — 合并边框模型。
    Collapse,
}

/// 解析 CSS border-collapse 属性值。
pub fn parse_border_collapse(value: &str) -> Option<BorderCollapseValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "separate" => Some(BorderCollapseValue::Separate),
        "collapse" => Some(BorderCollapseValue::Collapse),
        _ => None,
    }
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

/// 解析 CSS resize 属性值。
pub fn parse_resize(value: &str) -> Option<ResizeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ResizeValue::None),
        "both" => Some(ResizeValue::Both),
        "horizontal" => Some(ResizeValue::Horizontal),
        "vertical" => Some(ResizeValue::Vertical),
        "block" => Some(ResizeValue::Block),
        "inline" => Some(ResizeValue::Inline),
        _ => None,
    }
}

// ── CSS Interaction / Performance Hint 值类型 ──────────────────────────

/// CSS overscroll-behavior 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverscrollBehaviorValue {
    /// auto（默认值）— 浏览器默认滚动溢出行为。
    Auto,
    /// contain — 阻止滚动链传播到祖先元素。
    Contain,
    /// none — 阻止滚动链和默认溢出行为。
    None,
}

/// 解析 CSS overscroll-behavior 属性值。
pub fn parse_overscroll_behavior(value: &str) -> Option<OverscrollBehaviorValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(OverscrollBehaviorValue::Auto),
        "contain" => Some(OverscrollBehaviorValue::Contain),
        "none" => Some(OverscrollBehaviorValue::None),
        _ => None,
    }
}

/// CSS touch-action 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TouchActionValue {
    /// auto（默认值）— 浏览器处理所有触摸操作。
    Auto,
    /// none — 禁用所有触摸操作。
    None,
    /// pan-x — 仅允许水平平移。
    PanX,
    /// pan-y — 仅允许垂直平移。
    PanY,
    /// pan-x pan-y — 允许水平和垂直平移。
    PanXPanY,
    /// manipulation — 仅允许平移和缩放（禁用双击缩放）。
    Manipulation,
}

/// 解析 CSS touch-action 属性值。
pub fn parse_touch_action(value: &str) -> Option<TouchActionValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(TouchActionValue::Auto),
        "none" => Some(TouchActionValue::None),
        "pan-x" => Some(TouchActionValue::PanX),
        "pan-y" => Some(TouchActionValue::PanY),
        "pan-x pan-y" | "pan-y pan-x" => Some(TouchActionValue::PanXPanY),
        "manipulation" => Some(TouchActionValue::Manipulation),
        _ => None,
    }
}

/// CSS user-select 值。
#[derive(Debug, Clone, PartialEq)]
pub enum UserSelectValue {
    /// auto（默认值）— 由浏览器决定。
    Auto,
    /// text — 可选择文本。
    Text,
    /// none — 禁止选择。
    None,
    /// all — 点击即全选。
    All,
    /// contain — 选择限制在元素内。
    Contain,
}

/// 解析 CSS user-select 属性值。
pub fn parse_user_select(value: &str) -> Option<UserSelectValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(UserSelectValue::Auto),
        "text" => Some(UserSelectValue::Text),
        "none" => Some(UserSelectValue::None),
        "all" => Some(UserSelectValue::All),
        "contain" => Some(UserSelectValue::Contain),
        _ => None,
    }
}

/// CSS will-change 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WillChangeValue {
    /// auto（默认值）— 无特别提示。
    Auto,
    /// scroll-position — 预期滚动位置会变化。
    ScrollPosition,
    /// contents — 预期内容会变化。
    Contents,
    /// 自定义属性名（如 transform、opacity）。
    Custom(String),
}

/// 解析 CSS will-change 属性值。
pub fn parse_will_change(value: &str) -> Option<WillChangeValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(WillChangeValue::Auto),
        "scroll-position" => Some(WillChangeValue::ScrollPosition),
        "contents" => Some(WillChangeValue::Contents),
        _ => {
            // 接受任意标识符（如 transform、opacity、top、left）
            if v.is_empty() {
                return None;
            }
            // 简单验证：只包含字母、数字、连字符
            if v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                Some(WillChangeValue::Custom(v))
            } else {
                None
            }
        }
    }
}

/// CSS pointer-events 值。
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEventsValue {
    /// auto（默认值）— 元素是指针事件的目标。
    Auto,
    /// none — 元素不是指针事件的目标。
    None,
    /// visiblePainted — SVG：可见且填充/描边区域。
    VisiblePainted,
    /// visibleFill — SVG：可见且填充区域。
    VisibleFill,
    /// visibleStroke — SVG：可见且描边区域。
    VisibleStroke,
    /// visible — SVG：可见区域。
    Visible,
    /// painted — SVG：填充/描边区域（不论可见性）。
    Painted,
    /// fill — SVG：填充区域。
    Fill,
    /// stroke — SVG：描边区域。
    Stroke,
    /// all — SVG：所有区域。
    All,
    /// inherit — 显式继承。
    Inherit,
}

/// 解析 CSS pointer-events 属性值。
pub fn parse_pointer_events(value: &str) -> Option<PointerEventsValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(PointerEventsValue::Auto),
        "none" => Some(PointerEventsValue::None),
        "visiblepainted" => Some(PointerEventsValue::VisiblePainted),
        "visiblefill" => Some(PointerEventsValue::VisibleFill),
        "visiblestroke" => Some(PointerEventsValue::VisibleStroke),
        "visible" => Some(PointerEventsValue::Visible),
        "painted" => Some(PointerEventsValue::Painted),
        "fill" => Some(PointerEventsValue::Fill),
        "stroke" => Some(PointerEventsValue::Stroke),
        "all" => Some(PointerEventsValue::All),
        "inherit" => Some(PointerEventsValue::Inherit),
        _ => None,
    }
}

// ── CSS Counter 值类型 ──────────────────────────────────────────────

/// CSS counter-increment / counter-reset 单个计数器操作值。
#[derive(Debug, Clone, PartialEq)]
pub struct CounterActionValue {
    /// 计数器名称。
    pub name: String,
    /// 增量或重置值，None 表示默认（increment=1, reset=0）。
    pub value: Option<i32>,
}

/// 解析单个计数器操作值。
///
/// 格式：`"counter-name"` 或 `"counter-name 5"`。
pub fn parse_counter_action(input: &str) -> Option<CounterActionValue> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let parts: Vec<&str> = input.split_whitespace().collect();
    let name = parts.first()?.to_string();
    // 计数器名称不能是 none
    if name.eq_ignore_ascii_case("none") {
        return None;
    }
    let value = if parts.len() > 1 {
        Some(parts[1].parse::<i32>().ok()?)
    } else {
        None
    };
    Some(CounterActionValue { name, value })
}

/// 解析计数器操作列表。
///
/// 格式：`"section 1 subsection"` → `[CounterActionValue { name: "section", value: Some(1) }, CounterActionValue { name: "subsection", value: None }]`。
/// 特殊值 `"none"` 返回空列表。
pub fn parse_counter_list(input: &str) -> Option<Vec<CounterActionValue>> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("none") {
        return Some(vec![]);
    }
    let mut result = Vec::new();
    let mut tokens = input.split_whitespace().peekable();
    while let Some(name) = tokens.next() {
        if name.eq_ignore_ascii_case("none") {
            return None;
        }
        // 检查下一个 token 是否为整数
        let value = if tokens.peek().is_some_and(|t| t.parse::<i32>().is_ok()) {
            tokens.next().and_then(|t| t.parse::<i32>().ok())
        } else {
            None
        };
        result.push(CounterActionValue {
            name: name.to_string(),
            value,
        });
    }
    if result.is_empty() {
        return None;
    }
    Some(result)
}

// ── CSS Content 值类型 ──────────────────────────────────────────────

/// CSS content 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentValue {
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

/// 解析 CSS content 属性值。
///
/// 支持格式：`normal`、`none`、字符串、`attr(name)`、`counter(name)` 或 `counter(name, style)`。
pub fn parse_content(input: &str) -> Option<ContentValue> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("normal") {
        return Some(ContentValue::Normal);
    }
    if input.eq_ignore_ascii_case("none") {
        return Some(ContentValue::None);
    }
    // 字符串：引号包裹
    if (input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')) {
        if input.len() < 2 {
            return None;
        }
        return Some(ContentValue::String(input[1..input.len() - 1].to_string()));
    }
    // attr(name)
    if input.starts_with("attr(") && input.ends_with(')') {
        let inner = input[5..input.len() - 1].trim();
        if inner.is_empty() {
            return None;
        }
        return Some(ContentValue::Attr(inner.to_string()));
    }
    // counter(name) 或 counter(name, style)
    if input.starts_with("counter(") && input.ends_with(')') {
        let inner = input[8..input.len() - 1].trim();
        if inner.is_empty() {
            return None;
        }
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        let name = parts.first()?.to_string();
        let style = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        return Some(ContentValue::Counter { name, style });
    }
    None
}

// ── CSS Quotes 值类型 ──────────────────────────────────────────────

/// CSS quotes 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum QuotesValue {
    /// none — 不使用引号。
    None,
    /// auto — 使用基于内容语言的引号。
    Auto,
    /// 引号对列表，每对为 (open, close)。
    Pairs(Vec<(String, String)>),
}

/// 解析 CSS quotes 属性值。
///
/// 支持格式：
/// - `none`
/// - `auto`
/// - 引号对列表：`"«" "»" "‹" "›"`（开引号和闭引号交替出现）
pub fn parse_quotes(input: &str) -> Option<QuotesValue> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("none") {
        return Some(QuotesValue::None);
    }
    if input.eq_ignore_ascii_case("auto") {
        return Some(QuotesValue::Auto);
    }
    // 解析引号对：交替出现的引号字符串
    let mut pairs = Vec::new();
    let mut chars = input.chars().peekable();
    loop {
        // 跳过空白
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        // 读取开引号
        let open = parse_quoted_string_chars(&mut chars)?;
        // 跳过空白
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        // 读取闭引号
        let close = parse_quoted_string_chars(&mut chars)?;
        pairs.push((open, close));
    }
    if pairs.is_empty() {
        return None;
    }
    Some(QuotesValue::Pairs(pairs))
}

/// 从字符流中解析引号包裹的字符串内容。
fn parse_quoted_string_chars(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    let quote = chars.peek()?;
    if *quote != '"' && *quote != '\'' {
        return None;
    }
    let q = chars.next()?; // 消费开头引号
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == q {
            return Some(result);
        }
        if c == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(escaped);
            }
        } else {
            result.push(c);
        }
    }
    None
}

// ── CSS Contain 值类型 ──────────────────────────────────────────────

/// CSS contain 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainValue {
    /// none（默认值）。
    None,
    /// strict — 等价于 layout style paint。
    Strict,
    /// content — 等价于 layout style paint size。
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
impl ContainValue {
    /// size 标志位。
    pub const FLAG_SIZE: u8 = 0x01;
    /// layout 标志位。
    pub const FLAG_LAYOUT: u8 = 0x02;
    /// style 标志位。
    pub const FLAG_STYLE: u8 = 0x04;
    /// paint 标志位。
    pub const FLAG_PAINT: u8 = 0x08;
}

/// 解析 CSS contain 属性值。
///
/// 支持格式：
/// - `"none"` — 无包含。
/// - `"strict"` — 等价于 `layout style paint`。
/// - `"content"` — 等价于 `layout style paint size`。
/// - 单个关键字：`"size"`、`"layout"`、`"style"`、`"paint"`。
/// - 多个空格分隔的关键字：`"layout style paint"`。
pub fn parse_contain(value: &str) -> Option<ContainValue> {
    let value = value.trim().to_ascii_lowercase();

    match value.as_str() {
        "none" => Some(ContainValue::None),
        "strict" => Some(ContainValue::Strict),
        "content" => Some(ContainValue::Content),
        "size" => Some(ContainValue::Size),
        "layout" => Some(ContainValue::Layout),
        "style" => Some(ContainValue::Style),
        "paint" => Some(ContainValue::Paint),
        _ => {
            // 解析空格分隔的关键字列表
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let mut flags: u8 = 0;
            for part in parts {
                match part {
                    "size" => flags |= ContainValue::FLAG_SIZE,
                    "layout" => flags |= ContainValue::FLAG_LAYOUT,
                    "style" => flags |= ContainValue::FLAG_STYLE,
                    "paint" => flags |= ContainValue::FLAG_PAINT,
                    _ => return None,
                }
            }

            if flags == 0 {
                None
            } else {
                Some(ContainValue::Custom(flags))
            }
        }
    }
}

// ── CSS Column 值类型 ──────────────────────────────────────────────

/// CSS column-count 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnCountValue {
    /// auto。
    Auto,
    /// 正整数值。
    Number(u32),
}

/// 解析 CSS column-count 属性值。
///
/// 支持格式如 `"auto"`、`"3"`。
pub fn parse_column_count(value: &str) -> Option<ColumnCountValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ColumnCountValue::Auto);
    }
    let n: u32 = value.parse().ok()?;
    if n > 0 { Some(ColumnCountValue::Number(n)) } else { None }
}

/// CSS column-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnWidthValue {
    /// auto。
    Auto,
    /// 长度值。
    Length(LengthValue),
}

/// 解析 CSS column-width 属性值。
///
/// 支持格式如 `"auto"`、`"200px"`、`"10em"`。
pub fn parse_column_width(value: &str) -> Option<ColumnWidthValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ColumnWidthValue::Auto);
    }
    parse_length(value).map(ColumnWidthValue::Length)
}

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
        let rad: f32 = s.trim_end_matches("rad").trim().parse().ok()?;
        Some(rad.to_degrees())
    } else if s.ends_with("turn") {
        let turn: f32 = s.trim_end_matches("turn").trim().parse().ok()?;
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

// ── CSS Border Image 值类型 ──────────────────────────────────────────

/// CSS border-image-source 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderImageSourceValue {
    /// none（默认值）— 不使用边框图片。
    None,
    /// url(<string>) — 指定边框图片 URL。
    Url(String),
}

/// 解析 CSS border-image-source 属性值。
///
/// 支持格式如 `"none"`、`"url(border.png)"`。
pub fn parse_border_image_source(value: &str) -> Option<BorderImageSourceValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(BorderImageSourceValue::None);
    }
    if value.starts_with("url(") && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = inner.trim();
        let url = if (url.starts_with('"') && url.ends_with('"')) || (url.starts_with('\'') && url.ends_with('\'')) {
            url.get(1..url.len() - 1)?
        } else {
            url
        };
        if url.is_empty() {
            return None;
        }
        return Some(BorderImageSourceValue::Url(url.to_string()));
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
            fill = true;
            continue;
        }
        if lower.ends_with('%') {
            let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
            if pct < 0.0 {
                return None;
            }
            numbers.push(BorderImageSliceComponent::Percent(pct));
        } else {
            let n: f32 = lower.parse().ok()?;
            if n < 0.0 {
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
            if pct < 0.0 {
                return None;
            }
            components.push(BorderImageWidthComponent::Percent(pct));
        } else if lower.ends_with("px") || lower.ends_with("em") || lower.ends_with("rem") {
            let len = parse_length(token)?;
            components.push(BorderImageWidthComponent::Length(len));
        } else {
            let n: f32 = lower.parse().ok()?;
            if n < 0.0 {
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
            components.push(BorderImageOutsetComponent::Length(len));
        } else {
            let n: f32 = lower.parse().ok()?;
            if n < 0.0 {
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
    if value.starts_with("url(") && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = inner.trim();
        let url = if (url.starts_with('"') && url.ends_with('"')) || (url.starts_with('\'') && url.ends_with('\'')) {
            url.get(1..url.len() - 1)?
        } else {
            url
        };
        if url.is_empty() {
            return None;
        }
        return Some(ListStyleImageValue::Url(url.to_string()));
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
    let v = if parts.len() == 2 {
        parse_length(parts[1])?
    } else {
        h.clone()
    };
    Some(BorderSpacingValue {
        horizontal: h,
        vertical: v,
    })
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
