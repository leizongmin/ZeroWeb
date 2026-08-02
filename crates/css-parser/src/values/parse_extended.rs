//! CSS 扩展属性解析（文本、表格、交互、计数器、内容、引用、包含、列）。
//!
//! 视觉效果相关属性见 [`parse_extended_visual`]，边框图像和裁剪路径见 [`parse_extended_border`]。

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

/// CSS margin-trim 值（css-box-4 §margin-trim）。
///
/// 表示为四向边 flag（block-start / block-end / inline-start / inline-end），统一
/// 支持单值（`block` / `inline` / `both` / `block-start` / `block-end` /
/// `inline-start` / `inline-end`）与空格分隔组合（如 `block-start inline-start`）。
/// `<inset()>` 形式未实现。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarginTrimValue {
    /// 裁剪块首边距（首子 margin-block-start）。
    pub block_start: bool,
    /// 裁剪块末边距（末子 margin-block-end）。
    pub block_end: bool,
    /// 裁剪行内首边距（首子 margin-inline-start）。
    pub inline_start: bool,
    /// 裁剪行内末边距（末子 margin-inline-end）。
    pub inline_end: bool,
}

impl MarginTrimValue {
    /// 全 false（`none`，默认）。
    pub const NONE: Self = Self {
        block_start: false,
        block_end: false,
        inline_start: false,
        inline_end: false,
    };
}

/// 解析 CSS margin-trim 属性值（css-box-4）。
///
/// 支持：`none`、`block`、`inline`、`both`、`block-start`、`block-end`、
/// `inline-start`、`inline-end`，以及空格分隔的组合（如 `block-start inline-start`）。
/// `none` 仅单独合法（与其他 token 混用 → 非法）；未识别 token → None（整条声明非法）。
pub fn parse_margin_trim(value: &str) -> Option<MarginTrimValue> {
    let tokens: Vec<&str> = value.split_ascii_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("none") {
        return Some(MarginTrimValue::NONE);
    }
    let mut v = MarginTrimValue::NONE;
    for tok in tokens {
        match tok.to_ascii_lowercase().as_str() {
            "block" => {
                v.block_start = true;
                v.block_end = true;
            }
            "inline" => {
                v.inline_start = true;
                v.inline_end = true;
            }
            "both" => {
                v.block_start = true;
                v.block_end = true;
                v.inline_start = true;
                v.inline_end = true;
            }
            "block-start" => v.block_start = true,
            "block-end" => v.block_end = true,
            "inline-start" => v.inline_start = true,
            "inline-end" => v.inline_end = true,
            // none 与其他 token 混用 / 未识别 token → 非法。
            _ => return None,
        }
    }
    Some(v)
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

/// 解析单个 will-change 标识符（非 auto/scroll-position/contents 的 custom-ident）。
fn parse_will_change_ident(token: &str) -> Option<WillChangeValue> {
    let t = token.trim().to_ascii_lowercase();
    match t.as_str() {
        "scroll-position" => Some(WillChangeValue::ScrollPosition),
        "contents" => Some(WillChangeValue::Contents),
        "" | "auto" => None, // auto 仅作为整体值合法，不能混入 ident 列表
        _ => {
            if t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                Some(WillChangeValue::Custom(t))
            } else {
                None
            }
        }
    }
}

/// 解析 will-change 多 ident 列表（CSS Will Change：`auto | scroll-position | contents | <custom-ident>+`）。
/// `auto` → 空 Vec（默认值）；否则按空白分割（spec 为 `<custom-ident>+` 空格分隔，亦容忍逗号）
/// 逐个解析 ident，任一失败 → None。空 Vec = auto。
pub fn parse_will_change_list(value: &str) -> Option<Vec<WillChangeValue>> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(Vec::new());
    }
    // 按空白和逗号分割（容忍 `will-change: transform, opacity` 的逗号写法）。
    let tokens: Vec<&str> = v
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.is_empty() {
        return None;
    }
    let mut list = Vec::with_capacity(tokens.len());
    for t in &tokens {
        list.push(parse_will_change_ident(t)?);
    }
    Some(list)
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
    ///
    /// R2473：i64（原 i32）—— CSS 计数器值在规范上无界，CJK counter 测试用 10^16 量级
    /// counter-reset（simp/trad-chinese 等），i32 会静默溢出丢弃声明。i64 覆盖到 ~9.2×10^18。
    pub value: Option<i64>,
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
        Some(parts[1].parse::<i64>().ok()?)
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
        let value = if tokens.peek().is_some_and(|t| t.parse::<i64>().is_ok()) {
            tokens.next().and_then(|t| t.parse::<i64>().ok())
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
    /// `url(...)` 图片引用（generated content image，如 `content: url(icon.png)`）。
    /// R1988：伪元素 content:url() 渲染为替换图片。
    Url(String),
    /// 多 item 混合内容序列（如 `content: "Chapter " counter(c) ": "`）。
    /// CSS Content §content-property：content 值可是多个 component value 串联，
    /// 字符串与 counter() 交替（counter() 真实用法）。仅 string + counter() item；
    /// 含 url()/attr() 的多 item 暂不支持（defer，回退 None 同旧行为）。
    List(Vec<ContentListItem>),
}

/// content 混合序列的单个 item（CSS Content §content-property 多 component value）。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentListItem {
    /// 字符串字面量。
    Str(String),
    /// counter(name[, style]) 引用。
    Counter {
        /// 计数器名称。
        name: String,
        /// 可选的列表样式类型。
        style: Option<String>,
    },
}

/// 若 input 恰为单个完整函数调用 `fn_open ... )`（fn_open 含末尾 '('，如 "counter("），
/// 且匹配闭括号在 input 末尾（无后续 token = 单 item），返回括号内 inner（已 trim）。
///
/// 替代 `input.ends_with(')')` 的宽松匹配——后者会误匹配多 item 序列（首 fn( + 末 )）
/// 或未平衡输入（如 `counter(c counter(d)`）。单 item 分支须确保只捕获真单 item，
/// 多 item / 畸形交由 parse_content_list 或返回 None。
fn extract_single_function_inner<'a>(input: &'a str, fn_open: &str) -> Option<&'a str> {
    if input.len() < fn_open.len() || !input[..fn_open.len()].eq_ignore_ascii_case(fn_open) {
        return None;
    }
    let bytes = input.as_bytes();
    let mut depth = 1i32; // fn_open 末尾的 '(' 已开一层
    let mut i = fn_open.len();
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    // 须平衡（depth==0）且匹配 ) 在末尾（i==len，无后续 token）才是单 item。
    if depth != 0 || i != input.len() {
        return None;
    }
    Some(input[fn_open.len()..input.len() - 1].trim())
}

/// 解析 CSS content 属性值。
///
/// 支持格式：`normal`、`none`、字符串、`attr(name)`、`counter(name)` 或 `counter(name, style)`、
/// `url(...)`（R1988 generated content image，伪元素 content:url() 渲染为替换图片）、
/// 多 item 混合序列（`"Chapter " counter(c)`，CSS Content §content-property）。
pub fn parse_content(input: &str) -> Option<ContentValue> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("normal") {
        return Some(ContentValue::Normal);
    }
    if input.eq_ignore_ascii_case("none") {
        return Some(ContentValue::None);
    }
    // 多 item 混合内容优先（如 `"Chapter " counter(c) ": "`）。须在单 item 分支之前：
    // 单 counter() 分支用 ends_with(')') 会误匹配多 item 输入（首 counter( + 末 )），
    // 故先让 parse_content_list 拦截 ≥2 item 序列；单 item 它返回 None，自然落回下方分支。
    if let Some(v) = parse_content_list(input) {
        return Some(v);
    }
    // 字符串：引号包裹
    if (input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')) {
        if input.len() < 2 {
            return None;
        }
        return Some(ContentValue::String(input[1..input.len() - 1].to_string()));
    }
    // attr(name) — CSS Values §4：函数名大小写不敏感（ATTR ≡ attr）；属性名内容保持原样。
    if let Some(inner) = extract_single_function_inner(input, "attr(") {
        if inner.is_empty() {
            return None;
        }
        return Some(ContentValue::Attr(inner.to_string()));
    }
    // counter(name) 或 counter(name, style)
    // counter(name[, style]) — CSS Values §4：函数名大小写不敏感（COUNTER ≡ counter）；计数器名/样式保持原样。
    if let Some(inner) = extract_single_function_inner(input, "counter(") {
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
    // url(...) — generated content image（R1988）。支持引号包裹的 url："url('x.png')" / 'url("x.png")'。
    if let Some(inner) = extract_single_function_inner(input, "url(") {
        if inner.is_empty() {
            return None;
        }
        let url = inner.trim_matches('"').trim_matches('\'').trim();
        if url.is_empty() {
            return None;
        }
        return Some(ContentValue::Url(url.to_string()));
    }
    None
}

/// 解析 content 多 item 混合序列。返回 List 当且仅当 ≥2 个合法 string/counter item；
/// 否则 None（单 item 由 parse_content 上方分支处理，畸形/含不支持函数亦 None）。
fn parse_content_list(input: &str) -> Option<ContentValue> {
    let bytes = input.as_bytes();
    let mut items: Vec<ContentListItem> = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        // 跳过空白
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        if bytes[pos] == b'"' || bytes[pos] == b'\'' {
            // 引号字符串（转义解码与单 item String 分支一致：取原始子串）
            let quote = bytes[pos];
            pos += 1;
            let start = pos;
            while pos < bytes.len() && bytes[pos] != quote {
                pos += 1;
            }
            if pos >= bytes.len() {
                return None; // 未闭合引号
            }
            items.push(ContentListItem::Str(input[start..pos].to_string()));
            pos += 1; // 消费闭合引号
        } else {
            // 函数 token：读 ident 后须紧跟 '('
            let id_start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
                pos += 1;
            }
            let ident = &input[id_start..pos];
            if ident.is_empty() || pos >= bytes.len() || bytes[pos] != b'(' {
                return None; // 非法 token
            }
            // 消费平衡括号
            let inner_start = pos + 1;
            let mut depth = 1;
            pos += 1;
            while pos < bytes.len() && depth > 0 {
                match bytes[pos] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    _ => {}
                }
                pos += 1;
            }
            if depth != 0 {
                return None; // 未闭合括号
            }
            let inner = input[inner_start..pos - 1].trim();
            if ident.eq_ignore_ascii_case("counter") {
                let (name, style) = parse_counter_call_args(inner)?;
                items.push(ContentListItem::Counter { name, style });
            } else {
                // attr/url/counters 等在多 item 序列暂不支持 → None（defer，同旧行为）
                return None;
            }
        }
    }
    if items.len() >= 2 {
        Some(ContentValue::List(items))
    } else {
        None
    }
}

/// 解析 counter() 函数参数 inner（`name` 或 `name, style`）。
fn parse_counter_call_args(inner: &str) -> Option<(String, Option<String>)> {
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
    Some((name, style))
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
    /// strict — 等价于 size layout paint style（CSS Containment §2）。
    Strict,
    /// content — 等价于 layout paint style（不含 size）。
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
/// - `"strict"` — 等价于 `size layout paint style`。
/// - `"content"` — 等价于 `layout paint style`（不含 size）。
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
