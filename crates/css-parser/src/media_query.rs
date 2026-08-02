//! CSS 媒体查询解析与评估。
//!
//! 支持解析常见的媒体查询条件并在给定视口和媒体类型下评估。
//!
//! ## 支持的媒体特性
//!
//! - `width`, `min-width`, `max-width`
//! - `height`, `min-height`, `max-height`
//! - `orientation` (portrait/landscape)
//! - Level 4 范围语法：`width > 600px`、`width >= 600px`、`width < 1000px`、`width <= 1000px`、`width = 800px`（`=` ≡ `:`）
//! - Level 4 组合范围：`600px <= width <= 1000px`
//! - 布尔特性：`hover`、`color`
//! - 媒体类型：`screen`, `print`, `all`

/// 媒体查询条件。
///
/// 表示 `@media` 规则中括号内的查询条件。
#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
    /// 媒体类型限制（如 `screen`、`print`）。
    /// `None` 表示不限制（等价于 `all`）。
    pub media_type: Option<MediaType>,
    /// 是否取反（`not`）。
    pub negated: bool,
    /// 媒体特性条件列表（用 `and` 连接）。
    pub conditions: Vec<MediaCondition>,
}

/// 媒体类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// screen。
    Screen,
    /// print。
    Print,
    /// all。
    All,
}

/// 单个媒体特性条件。
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCondition {
    /// width 特性。
    Width(MediaFeatureOp, f64),
    /// min-width 特性。
    MinWidth(f64),
    /// max-width 特性。
    MaxWidth(f64),
    /// height 特性。
    Height(MediaFeatureOp, f64),
    /// min-height 特性。
    MinHeight(f64),
    /// max-height 特性。
    MaxHeight(f64),
    /// orientation 特性。
    Orientation(OrientationValue),
    /// hover 布尔特性（是否有悬停能力）。
    Hover,
    /// color 布尔特性（是否支持彩色）。
    Color,
    /// 用户颜色方案偏好。
    PrefersColorScheme(PrefersColorSchemeValue),
    /// 用户动画偏好。
    PrefersReducedMotion(ReducedMotionValue),
    /// 指针设备类型。
    Pointer(PointerValue),
    /// 分辨率（dpi）。
    Resolution(MediaFeatureOp, f64),
}

/// 媒体特性比较操作。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaFeatureOp {
    /// 精确匹配（`=` 或 `:`）。
    Exact,
    /// 大于（`>`）。
    GreaterThan,
    /// 大于等于（`>=`）。
    GreaterEqual,
    /// 小于（`<`）。
    LessThan,
    /// 小于等于（`<=`）。
    LessEqual,
}

/// 方向值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrientationValue {
    /// 竖屏（width < height）。
    Portrait,
    /// 横屏（width >= height）。
    Landscape,
}

/// 用户颜色方案偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefersColorSchemeValue {
    /// 深色模式。
    Dark,
    /// 浅色模式。
    Light,
}

/// 用户动画偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducedMotionValue {
    /// 减少动画。
    Reduce,
    /// 无偏好。
    NoPreference,
}

/// 指针设备类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerValue {
    /// 无指针设备。
    None,
    /// 粗指针（如触摸屏）。
    Coarse,
    /// 精细指针（如鼠标）。
    Fine,
}

/// 媒体查询评估上下文。
#[derive(Debug, Clone)]
pub struct MediaContext {
    /// 视口宽度（px）。
    pub viewport_width: f64,
    /// 视口高度（px）。
    pub viewport_height: f64,
    /// 当前媒体类型。
    pub media_type: MediaType,
    /// 用户颜色方案偏好。
    pub prefers_color_scheme: PrefersColorSchemeValue,
    /// 用户动画偏好。
    pub prefers_reduced_motion: ReducedMotionValue,
    /// 指针设备类型。
    pub pointer_type: PointerValue,
    /// 分辨率（dpi），默认 96.0。
    pub resolution_dpi: f64,
}

impl MediaContext {
    /// 创建新的媒体上下文。
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            media_type: MediaType::Screen,
            prefers_color_scheme: PrefersColorSchemeValue::Light,
            prefers_reduced_motion: ReducedMotionValue::NoPreference,
            pointer_type: PointerValue::Coarse,
            resolution_dpi: 96.0,
        }
    }

    /// 创建指定媒体类型的上下文。
    pub fn with_type(width: f64, height: f64, media_type: MediaType) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            media_type,
            prefers_color_scheme: PrefersColorSchemeValue::Light,
            prefers_reduced_motion: ReducedMotionValue::NoPreference,
            pointer_type: PointerValue::Coarse,
            resolution_dpi: 96.0,
        }
    }
}

/// 解析 @media 规则的 prelude 字符串为 MediaQuery 列表。
///
/// 逗号分隔的查询表示 OR 关系——只要其中任意一个匹配，整体即为真。
///
/// 支持的格式示例：
/// - `"(min-width: 600px)"`
/// - `"screen and (max-width: 768px)"`
/// - `"print"`
/// - `"(orientation: landscape)"`
/// - `"screen and (min-width: 600px) and (max-width: 1024px)"`
/// - `"(width > 600px)"` — Level 4 范围语法
/// - `"(600px <= width <= 1000px)"` — Level 4 组合范围
/// - `"(hover)"` — 布尔特性
/// - `"only screen and ..."` — only 前缀（兼容旧浏览器）
/// - `"screen, print"` — 逗号分隔 OR 查询
pub fn parse_media_query(input: &str) -> Option<Vec<MediaQuery>> {
    let input = input.trim();
    if input.is_empty() {
        // CSS Media Queries §3：媒体查询列表省略时隐含 `all`（`@media { ... }` ≡
        // `@media all { ... }`，匹配一切）。返回无类型限制 + 无条件的查询（evaluate = true）。
        // 旧实现返回 None 导致 `@media{...}`（prelude 为空，含 `@media` 后无空格的
        // whitespace-optional 形式）规则不应用。driving: WPT at-media-whitespace-optional-001。
        return Some(vec![MediaQuery {
            media_type: None,
            negated: false,
            conditions: Vec::new(),
        }]);
    }

    // 按逗号分割为多个查询
    let parts = split_media_queries(input);
    let mut queries = Vec::new();

    for part in parts {
        if let Some(q) = parse_single_media_query(part.trim()) {
            queries.push(q);
        }
    }

    if queries.is_empty() { None } else { Some(queries) }
}

/// 按顶层逗号分割媒体查询字符串（不分割括号内的逗号）。
fn split_media_queries(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, b) in input.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push(&input[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&input[start..]);
    parts
}

/// 解析单个媒体查询（不含逗号分隔）。
fn parse_single_media_query(input: &str) -> Option<MediaQuery> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut remaining = input;
    let mut negated = false;
    let mut media_type = None;

    // 处理 "not" 前缀（CSS 关键字不区分大小写）
    {
        let lower = remaining.to_ascii_lowercase();
        if lower.starts_with("not ") {
            negated = true;
            remaining = remaining[4..].trim_start();
        }
    }

    // 处理 "only" 前缀（兼容旧浏览器，忽略不影响行为）
    {
        let lower = remaining.to_ascii_lowercase();
        if lower.starts_with("only ") {
            remaining = remaining[5..].trim_start();
        }
    }

    // 尝试提取媒体类型（CSS 关键字不区分大小写）
    let after_prefix = remaining; // not/only 剥离后、media-type 提取前的剩余（R2426 未知 type 检测）
    let lower_remaining = remaining.to_ascii_lowercase();
    if lower_remaining.starts_with("screen") {
        let after = remaining[6..].trim_start();
        if after.is_empty() || after.to_ascii_lowercase().starts_with("and") {
            media_type = Some(MediaType::Screen);
            let and_stripped = after
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_alphabetic())
                .map(|(i, _)| i)
                .last()
                .map(|i| i + 1)
                .unwrap_or(3);
            remaining = after.get(and_stripped..).unwrap_or(after).trim_start();
        }
    } else if lower_remaining.starts_with("print") {
        let after = remaining[5..].trim_start();
        if after.is_empty() || after.to_ascii_lowercase().starts_with("and") {
            media_type = Some(MediaType::Print);
            remaining = after
                .strip_prefix("and")
                .or_else(|| after.strip_prefix("And"))
                .or_else(|| after.strip_prefix("AND"))
                .unwrap_or(after)
                .trim_start();
        }
    } else if lower_remaining.starts_with("all") {
        let after = remaining[3..].trim_start();
        if after.is_empty() || after.to_ascii_lowercase().starts_with("and") {
            media_type = Some(MediaType::All);
            remaining = after
                .strip_prefix("and")
                .or_else(|| after.strip_prefix("And"))
                .or_else(|| after.strip_prefix("AND"))
                .unwrap_or(after)
                .trim_start();
        }
    }

    // R2426：未知 media type（裸标识符非 screen/print/all 且非括号条件开头）→ 不匹配
    // （MQ4 §3.6: "Unknown media types evaluate to false"）。此前 media_type=None 丢失
    // 「曾出现未知 type」信息，evaluate_media_query 把 None 当 "all"→匹配，致 `@media nonsense`
    // / `@import "x.css" (..), nonsense` 中未知 type 误判匹配。返回 None（解析失败）让调用方
    // 按 no-match 处理（与既有 `@media screen` / `(condition)` 行为不变——那些 media_type 非 None
    // 或 after_prefix 以 `(` 开头）。
    if media_type.is_none() && !after_prefix.is_empty() && !after_prefix.starts_with('(') {
        return None;
    }

    // 解析括号内的条件
    let mut conditions = Vec::new();
    while remaining.starts_with('(') {
        // 找到匹配的右括号
        let end = find_matching_paren(remaining)?;
        let inner = remaining[1..end].trim();
        // 组合范围可能产生两个条件，所以用 extend
        conditions.extend(parse_conditions_from_inner(inner));

        remaining = remaining[end + 1..].trim_start();

        // 跳过 "and" 连接词
        if remaining.starts_with("and") {
            remaining = remaining[3..].trim_start();
        }
    }

    Some(MediaQuery {
        media_type,
        negated,
        conditions,
    })
}

/// 评估媒体查询在给定上下文中是否为真。
pub fn evaluate_media_query(query: &MediaQuery, ctx: &MediaContext) -> bool {
    let mut result = true;

    // 检查媒体类型
    if let Some(ref mt) = query.media_type {
        result = match mt {
            MediaType::All => true,
            MediaType::Screen => ctx.media_type == MediaType::Screen,
            MediaType::Print => ctx.media_type == MediaType::Print,
        };
    }

    // 检查所有条件
    if result {
        for cond in &query.conditions {
            if !evaluate_condition(cond, ctx) {
                result = false;
                break;
            }
        }
    }

    // 应用取反
    if query.negated {
        result = !result;
    }

    result
}

/// 评估单个媒体条件。
fn evaluate_condition(cond: &MediaCondition, ctx: &MediaContext) -> bool {
    match cond {
        MediaCondition::Width(op, v) => match op {
            MediaFeatureOp::Exact => (ctx.viewport_width - *v).abs() < 0.01,
            MediaFeatureOp::GreaterThan => ctx.viewport_width > *v,
            MediaFeatureOp::GreaterEqual => ctx.viewport_width >= *v,
            MediaFeatureOp::LessThan => ctx.viewport_width < *v,
            MediaFeatureOp::LessEqual => ctx.viewport_width <= *v,
        },
        MediaCondition::MinWidth(v) => ctx.viewport_width >= *v,
        MediaCondition::MaxWidth(v) => ctx.viewport_width <= *v,
        MediaCondition::Height(op, v) => match op {
            MediaFeatureOp::Exact => (ctx.viewport_height - *v).abs() < 0.01,
            MediaFeatureOp::GreaterThan => ctx.viewport_height > *v,
            MediaFeatureOp::GreaterEqual => ctx.viewport_height >= *v,
            MediaFeatureOp::LessThan => ctx.viewport_height < *v,
            MediaFeatureOp::LessEqual => ctx.viewport_height <= *v,
        },
        MediaCondition::MinHeight(v) => ctx.viewport_height >= *v,
        MediaCondition::MaxHeight(v) => ctx.viewport_height <= *v,
        MediaCondition::Orientation(orient) => {
            let is_portrait = ctx.viewport_height > ctx.viewport_width;
            match orient {
                OrientationValue::Portrait => is_portrait,
                OrientationValue::Landscape => !is_portrait,
            }
        }
        MediaCondition::Hover => true,
        MediaCondition::Color => true,
        MediaCondition::PrefersColorScheme(val) => ctx.prefers_color_scheme == *val,
        MediaCondition::PrefersReducedMotion(val) => ctx.prefers_reduced_motion == *val,
        MediaCondition::Pointer(val) => ctx.pointer_type == *val,
        MediaCondition::Resolution(op, v) => match op {
            MediaFeatureOp::Exact => (ctx.resolution_dpi - *v).abs() < 0.01,
            MediaFeatureOp::GreaterThan => ctx.resolution_dpi > *v,
            MediaFeatureOp::GreaterEqual => ctx.resolution_dpi >= *v,
            MediaFeatureOp::LessThan => ctx.resolution_dpi < *v,
            MediaFeatureOp::LessEqual => ctx.resolution_dpi <= *v,
        },
    }
}

/// 解析括号内的条件字符串，返回零个或多个条件。
///
/// 支持四种格式：
/// - 布尔特性：`hover`、`color`（无值，仅检查是否支持）
/// - 传统冒号语法：`min-width: 600px`、`orientation: landscape`
/// - Level 4 范围语法：`width > 600px`、`width >= 600px`、`width < 1000px`、`width <= 1000px`
/// - Level 4 组合范围：`600px <= width <= 1000px`（展开为两个条件）
fn parse_conditions_from_inner(s: &str) -> Vec<MediaCondition> {
    let s = s.trim();
    let lower = s.to_ascii_lowercase();

    // 1) 布尔特性：纯特性名，无冒号也无运算符
    if !lower.contains(':') && !contains_range_op(&lower) {
        match lower.as_str() {
            "hover" => return vec![MediaCondition::Hover],
            "color" => return vec![MediaCondition::Color],
            "prefers-reduced-motion" => {
                // 布尔特性：当值为 reduce 时为真
                return vec![MediaCondition::PrefersReducedMotion(ReducedMotionValue::Reduce)];
            }
            "pointer" => {
                // 布尔特性：当指针设备不为 none 时为真
                return vec![MediaCondition::Pointer(PointerValue::Coarse)];
            }
            // prefers-color-scheme 无布尔语义，需要明确值
            _ => return vec![],
        }
    }

    // 2) 传统冒号语法
    if let Some(colon_pos) = s.find(':') {
        if let Some(cond) = parse_colon_syntax(s, colon_pos) {
            return vec![cond];
        }
        return vec![];
    }

    // 3) 范围语法（含组合范围）
    parse_range_syntax_vec(s)
}

/// 解析传统冒号语法（如 `min-width: 600px`）。
fn parse_colon_syntax(s: &str, colon_pos: usize) -> Option<MediaCondition> {
    let feature = s[..colon_pos].trim().to_ascii_lowercase();
    let value = s[colon_pos + 1..].trim();

    match feature.as_str() {
        "orientation" => {
            let lower = value.to_ascii_lowercase();
            match lower.as_str() {
                "portrait" => Some(MediaCondition::Orientation(OrientationValue::Portrait)),
                "landscape" => Some(MediaCondition::Orientation(OrientationValue::Landscape)),
                _ => None,
            }
        }
        "prefers-color-scheme" => {
            let lower = value.to_ascii_lowercase();
            match lower.as_str() {
                "dark" => Some(MediaCondition::PrefersColorScheme(PrefersColorSchemeValue::Dark)),
                "light" => Some(MediaCondition::PrefersColorScheme(PrefersColorSchemeValue::Light)),
                _ => None,
            }
        }
        "prefers-reduced-motion" => {
            let lower = value.to_ascii_lowercase();
            match lower.as_str() {
                "reduce" => Some(MediaCondition::PrefersReducedMotion(ReducedMotionValue::Reduce)),
                "no-preference" => Some(MediaCondition::PrefersReducedMotion(ReducedMotionValue::NoPreference)),
                _ => None,
            }
        }
        "pointer" => {
            let lower = value.to_ascii_lowercase();
            match lower.as_str() {
                "none" => Some(MediaCondition::Pointer(PointerValue::None)),
                "coarse" => Some(MediaCondition::Pointer(PointerValue::Coarse)),
                "fine" => Some(MediaCondition::Pointer(PointerValue::Fine)),
                _ => None,
            }
        }
        "resolution" => {
            let num = parse_dpi_value(value)?;
            Some(MediaCondition::Resolution(MediaFeatureOp::Exact, num))
        }
        "min-resolution" => {
            let num = parse_dpi_value(value)?;
            Some(MediaCondition::Resolution(MediaFeatureOp::GreaterEqual, num))
        }
        "max-resolution" => {
            let num = parse_dpi_value(value)?;
            Some(MediaCondition::Resolution(MediaFeatureOp::LessEqual, num))
        }
        _ => {
            let num = parse_px_value(value)?;
            match feature.as_str() {
                "width" => Some(MediaCondition::Width(MediaFeatureOp::Exact, num)),
                "min-width" => Some(MediaCondition::MinWidth(num)),
                "max-width" => Some(MediaCondition::MaxWidth(num)),
                "height" => Some(MediaCondition::Height(MediaFeatureOp::Exact, num)),
                "min-height" => Some(MediaCondition::MinHeight(num)),
                "max-height" => Some(MediaCondition::MaxHeight(num)),
                _ => None,
            }
        }
    }
}

/// 检查字符串是否包含范围运算符（`<`, `<=`, `>`, `>=`, `=`）。
///
/// 含 `=`（CSS MQ L4 §7.1 `<mf-comparison>` 包含 `=` 精确相等）——
/// `(width = 800px)` 须路由到范围语法分支而非布尔特性分支。
fn contains_range_op(s: &str) -> bool {
    s.as_bytes().iter().any(|&b| b == b'<' || b == b'>' || b == b'=')
}

/// 解析 Level 4 范围语法，返回条件向量。
///
/// 两种形式：
/// - 简单范围：`width > 600px` → 一个条件
/// - 组合范围：`600px <= width <= 1000px` → 两个条件
fn parse_range_syntax_vec(s: &str) -> Vec<MediaCondition> {
    // 先尝试组合范围：`value op feature op value`
    if let Some(conds) = try_parse_combined_range(s) {
        return conds;
    }

    // 简单范围：`feature op value`
    if let Some(cond) = parse_simple_range(s) {
        return vec![cond];
    }

    vec![]
}

/// 尝试解析组合范围（如 `600px <= width <= 1000px`）。
///
/// 组合范围会展开为两个独立条件。
/// 例如 `600px <= width <= 1000px` 展开为：
/// - `Width(GreaterEqual, 600.0)`  （即 width >= 600px）
/// - `Width(LessEqual, 1000.0)`    （即 width <= 1000px）
fn try_parse_combined_range(s: &str) -> Option<Vec<MediaCondition>> {
    let (left_val, after_left) = parse_leading_value(s)?;
    let (left_op, after_op1) = parse_op(after_left)?;
    let (feature, after_feature) = parse_feature_name(after_op1)?;
    let (right_op, after_op2) = parse_op(after_feature)?;
    let right_val = parse_px_value(after_op2)?;

    // 组合范围 `600px <= width <= 1000px` 中：
    // 左侧 `600px <= width` 等价于 `width >= 600px`，需翻转运算符
    let left_cond = make_feature_condition(feature, flip_op(left_op), left_val)?;
    let right_cond = make_feature_condition(feature, right_op, right_val)?;

    Some(vec![left_cond, right_cond])
}

/// 翻转比较运算符方向。
///
/// 组合范围 `600px <= width` 等价于 `width >= 600px`，
/// 所以左侧的 `<=` 需要翻转为 `>=`。
fn flip_op(op: MediaFeatureOp) -> MediaFeatureOp {
    match op {
        MediaFeatureOp::LessThan => MediaFeatureOp::GreaterThan,
        MediaFeatureOp::LessEqual => MediaFeatureOp::GreaterEqual,
        MediaFeatureOp::GreaterThan => MediaFeatureOp::LessThan,
        MediaFeatureOp::GreaterEqual => MediaFeatureOp::LessEqual,
        MediaFeatureOp::Exact => MediaFeatureOp::Exact,
    }
}

/// 从字符串开头解析数值（如 `600px`），返回数值和剩余字符串。
fn parse_leading_value(s: &str) -> Option<(f64, &str)> {
    let s = s.trim();
    let end = s.as_bytes().iter().position(|&b| !b.is_ascii_digit() && b != b'.')?;
    let num_str = &s[..end];
    let num = num_str.parse::<f64>().ok()?;
    let rest = s[end..].trim_start();
    let rest = rest.strip_prefix("px").unwrap_or(rest).trim_start();
    Some((num, rest))
}

/// 从字符串开头解析比较运算符，返回运算符和剩余字符串。
fn parse_op(s: &str) -> Option<(MediaFeatureOp, &str)> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix(">=") {
        Some((MediaFeatureOp::GreaterEqual, rest.trim_start()))
    } else if let Some(rest) = s.strip_prefix("<=") {
        Some((MediaFeatureOp::LessEqual, rest.trim_start()))
    } else if let Some(rest) = s.strip_prefix('>') {
        Some((MediaFeatureOp::GreaterThan, rest.trim_start()))
    } else if let Some(rest) = s.strip_prefix('<') {
        Some((MediaFeatureOp::LessThan, rest.trim_start()))
    } else if let Some(rest) = s.strip_prefix('=') {
        // CSS MQ L4 §7.1 `<mf-comparison>` 包含 `=`：`(width = 800px)` ≡ `(width: 800px)`。
        // Exact 同冒号形式生成的 op，evaluate 路径已存在。
        Some((MediaFeatureOp::Exact, rest.trim_start()))
    } else {
        None
    }
}

/// 从字符串开头解析特性名（如 `width`、`height`），返回特性名和剩余字符串。
fn parse_feature_name(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    let end = s
        .as_bytes()
        .iter()
        .position(|&b| !b.is_ascii_alphanumeric() && b != b'-')?;
    let name = &s[..end];
    match name {
        "width" | "height" => Some((name, s[end..].trim_start())),
        _ => None,
    }
}

/// 根据特性名和运算符创建对应的 MediaCondition。
fn make_feature_condition(feature: &str, op: MediaFeatureOp, value: f64) -> Option<MediaCondition> {
    match feature {
        "width" => Some(MediaCondition::Width(op, value)),
        "height" => Some(MediaCondition::Height(op, value)),
        _ => None,
    }
}

/// 解析简单范围语法（如 `width > 600px`）。
fn parse_simple_range(s: &str) -> Option<MediaCondition> {
    let s = s.trim();
    let lower = s.to_ascii_lowercase();

    let op_pos = find_range_op_pos(&lower)?;

    let feature = s[..op_pos].trim().to_ascii_lowercase();
    let (op, value_str) = parse_op(&lower[op_pos..])?;
    let num = parse_px_value(value_str)?;

    make_feature_condition(&feature, op, num)
}

/// 找到字符串中第一个范围运算符的位置。
///
/// `<`/`>` 优先于 `=` 命中（`width >= 600px` 命中 `>` 而非 `=`，确保 `parse_op` 见到完整 `>=`）。
fn find_range_op_pos(s: &str) -> Option<usize> {
    s.as_bytes().iter().position(|&b| b == b'<' || b == b'>' || b == b'=')
}

/// 从 CSS 值字符串解析像素数值。
///
/// 支持 `"600px"`、`"600"`、`"50.5px"` 等格式。
fn parse_px_value(s: &str) -> Option<f64> {
    let s = s.trim();
    let s = s.strip_suffix("px").unwrap_or(s);
    s.parse::<f64>().ok()
}

/// 从 CSS 值字符串解析分辨率（dpi）数值。
///
/// 支持 CSS Values 4 §7.3 `<resolution>` 全部单位（大小写不敏感，CSS Syntax §4.3）：
/// - `dpi`（dots per inch，换算 1）；
/// - `dpcm`（dots per cm，1 inch = 2.54 cm → 换算 2.54）；
/// - `dppx`（dots per px unit，1 px = 1/96 inch → 换算 96）；
/// - `x`（`dppx` 别名，换算 96）。
///
/// 裸数字按 dpi 解析（向后兼容既有 `96` / `150.5` 行为）。
/// 支持 `"96dpi"`、`"2dppx"`、`"10dpcm"`、`"1x"`、`"96"` 等格式。
fn parse_dpi_value(s: &str) -> Option<f64> {
    let lower = s.trim().to_ascii_lowercase();
    // 先匹配最长单位（`dppx` 优先于 `dpi`），避免误剥前缀重叠的单位。
    let (num_str, factor) = if let Some(n) = lower.strip_suffix("dppx") {
        (n, 96.0)
    } else if let Some(n) = lower.strip_suffix("dpcm") {
        (n, 2.54)
    } else if let Some(n) = lower.strip_suffix("dpi") {
        (n, 1.0)
    } else if let Some(n) = lower.strip_suffix("x") {
        (n, 96.0)
    } else {
        (s.trim(), 1.0)
    };
    num_str.trim().parse::<f64>().ok().map(|v| v * factor)
}

/// 找到第一个 `(` 对应的 `)` 的位置。
fn find_matching_paren(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.first()? != &b'(' {
        return None;
    }
    let mut depth = 1;
    for (i, &b) in bytes.iter().enumerate().skip(1) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助：从 parse_media_query 结果中取第一个查询。
    fn first_query(input: &str) -> MediaQuery {
        parse_media_query(input).unwrap().into_iter().next().unwrap()
    }

    // ── 解析测试 ──

    #[test]
    fn test_parse_simple_min_width() {
        let q = first_query("(min-width: 600px)");
        assert_eq!(q.media_type, None);
        assert!(!q.negated);
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
    }

    #[test]
    fn test_parse_max_width() {
        let q = first_query("(max-width: 768px)");
        assert_eq!(q.conditions[0], MediaCondition::MaxWidth(768.0));
    }

    #[test]
    fn test_parse_screen_and() {
        let q = first_query("screen and (min-width: 600px)");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 1);
    }

    #[test]
    fn test_parse_print() {
        let q = first_query("print");
        assert_eq!(q.media_type, Some(MediaType::Print));
        assert!(q.conditions.is_empty());
    }

    #[test]
    fn test_parse_multiple_conditions() {
        let q = first_query("screen and (min-width: 600px) and (max-width: 1024px)");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 2);
        assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
        assert_eq!(q.conditions[1], MediaCondition::MaxWidth(1024.0));
    }

    #[test]
    fn test_parse_orientation() {
        let q = first_query("(orientation: landscape)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Orientation(OrientationValue::Landscape)
        );
    }

    #[test]
    fn test_parse_height() {
        let q = first_query("(min-height: 400px)");
        assert_eq!(q.conditions[0], MediaCondition::MinHeight(400.0));
    }

    #[test]
    fn test_parse_not() {
        let q = first_query("not screen");
        assert!(q.negated);
        assert_eq!(q.media_type, Some(MediaType::Screen));
    }

    #[test]
    fn test_parse_empty() {
        // CSS Media Queries §3：空媒体查询列表 ≡ `all`（匹配一切），非 None。
        // driving: WPT at-media-whitespace-optional-001 `@media{...}`。
        let queries = parse_media_query("").expect("空媒体查询应解析为隐含 all");
        let q = &queries[0];
        assert!(q.media_type.is_none(), "media_type 不限制 ≡ all");
        assert!(q.conditions.is_empty());
        assert!(!q.negated);
        // 评估：无类型限制 + 无条件 → 匹配（screen ctx 下也为 true）
        let ctx = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
        assert!(evaluate_media_query(q, &ctx), "空查询应匹配（≡ all）");
    }

    #[test]
    fn test_parse_just_parentheses() {
        let q = first_query("(width: 800px)");
        assert_eq!(q.conditions[0], MediaCondition::Width(MediaFeatureOp::Exact, 800.0));
    }

    #[test]
    fn test_media_l4_equality_operator() {
        // (width = 800px) — CSS MQ L4 §7.1 <mf-comparison> `=`，≡ (width: 800px)
        let q = first_query("(width = 800px)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::Width(MediaFeatureOp::Exact, 800.0));

        // 评估：800 通过（精确相等），799/801 不通过
        let ctx_pass = MediaContext::new(800.0, 600.0);
        let ctx_fail = MediaContext::new(799.0, 600.0);
        assert!(evaluate_media_query(&q, &ctx_pass));
        assert!(!evaluate_media_query(&q, &ctx_fail));

        // height 同理
        let qh = first_query("(height = 600px)");
        assert_eq!(qh.conditions[0], MediaCondition::Height(MediaFeatureOp::Exact, 600.0));
    }

    // ── Level 4 范围语法测试 ──

    #[test]
    fn test_media_range_greater_than() {
        // (width > 600px) — 大于，不含 600
        let q = first_query("(width > 600px)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::GreaterThan, 600.0)
        );

        // 评估：601 通过，600 不通过（严格大于）
        let ctx_pass = MediaContext::new(601.0, 400.0);
        let ctx_fail = MediaContext::new(600.0, 400.0);
        assert!(evaluate_media_query(&q, &ctx_pass));
        assert!(!evaluate_media_query(&q, &ctx_fail));
    }

    #[test]
    fn test_media_range_less_than() {
        // (width < 1000px) — 小于，不含 1000
        let q = first_query("(width < 1000px)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::Width(MediaFeatureOp::LessThan, 1000.0));

        // 评估：999 通过，1000 不通过（严格小于）
        let ctx_pass = MediaContext::new(999.0, 400.0);
        let ctx_fail = MediaContext::new(1000.0, 400.0);
        assert!(evaluate_media_query(&q, &ctx_pass));
        assert!(!evaluate_media_query(&q, &ctx_fail));
    }

    #[test]
    fn test_media_range_combined() {
        // (600px <= width <= 1000px) — 组合范围
        let q = first_query("(600px <= width <= 1000px)");
        assert_eq!(q.conditions.len(), 2);
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::GreaterEqual, 600.0)
        );
        assert_eq!(
            q.conditions[1],
            MediaCondition::Width(MediaFeatureOp::LessEqual, 1000.0)
        );

        // 评估：800 在范围内通过
        assert!(evaluate_media_query(&q, &MediaContext::new(800.0, 600.0)));
        // 600 恰好下界通过（>=）
        assert!(evaluate_media_query(&q, &MediaContext::new(600.0, 400.0)));
        // 1000 恰好上界通过（<=）
        assert!(evaluate_media_query(&q, &MediaContext::new(1000.0, 400.0)));
        // 599 太小不通过
        assert!(!evaluate_media_query(&q, &MediaContext::new(599.0, 400.0)));
        // 1001 太大不通过
        assert!(!evaluate_media_query(&q, &MediaContext::new(1001.0, 400.0)));
    }

    #[test]
    fn test_media_boolean_feature() {
        // (hover) — 布尔特性
        let q = first_query("(hover)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::Hover);

        // 布尔特性始终为真（假设支持）
        assert!(evaluate_media_query(&q, &MediaContext::new(1024.0, 768.0)));

        // (color) — 布尔特性
        let q_color = first_query("(color)");
        assert_eq!(q_color.conditions.len(), 1);
        assert_eq!(q_color.conditions[0], MediaCondition::Color);
        assert!(evaluate_media_query(&q_color, &MediaContext::new(1024.0, 768.0)));
    }

    #[test]
    fn test_media_range_greater_equal() {
        // (width >= 600px) — 大于等于
        let q = first_query("(width >= 600px)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::GreaterEqual, 600.0)
        );

        assert!(evaluate_media_query(&q, &MediaContext::new(600.0, 400.0)));
        assert!(evaluate_media_query(&q, &MediaContext::new(601.0, 400.0)));
        assert!(!evaluate_media_query(&q, &MediaContext::new(599.0, 400.0)));
    }

    #[test]
    fn test_media_range_less_equal() {
        // (width <= 1000px) — 小于等于
        let q = first_query("(width <= 1000px)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::LessEqual, 1000.0)
        );

        assert!(evaluate_media_query(&q, &MediaContext::new(1000.0, 400.0)));
        assert!(evaluate_media_query(&q, &MediaContext::new(999.0, 400.0)));
        assert!(!evaluate_media_query(&q, &MediaContext::new(1001.0, 400.0)));
    }

    #[test]
    fn test_media_range_height() {
        // (height > 400px)
        let q = first_query("(height > 400px)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Height(MediaFeatureOp::GreaterThan, 400.0)
        );

        assert!(evaluate_media_query(&q, &MediaContext::new(800.0, 401.0)));
        assert!(!evaluate_media_query(&q, &MediaContext::new(800.0, 400.0)));
    }

    #[test]
    fn test_media_range_combined_with_type() {
        // screen and (600px <= width <= 1000px)
        let q = first_query("screen and (600px <= width <= 1000px)");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 2);
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::GreaterEqual, 600.0)
        );
        assert_eq!(
            q.conditions[1],
            MediaCondition::Width(MediaFeatureOp::LessEqual, 1000.0)
        );
    }

    // ── 评估测试 ──

    #[test]
    fn test_eval_min_width_pass() {
        let q = first_query("(min-width: 600px)");
        let ctx = MediaContext::new(1024.0, 768.0);
        assert!(evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_min_width_fail() {
        let q = first_query("(min-width: 600px)");
        let ctx = MediaContext::new(400.0, 300.0);
        assert!(!evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_max_width_pass() {
        let q = first_query("(max-width: 768px)");
        let ctx = MediaContext::new(600.0, 400.0);
        assert!(evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_max_width_fail() {
        let q = first_query("(max-width: 768px)");
        let ctx = MediaContext::new(1024.0, 768.0);
        assert!(!evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_screen_type() {
        let q = first_query("screen and (min-width: 600px)");
        let ctx_screen = MediaContext::with_type(1024.0, 768.0, MediaType::Screen);
        let ctx_print = MediaContext::with_type(1024.0, 768.0, MediaType::Print);
        assert!(evaluate_media_query(&q, &ctx_screen));
        assert!(!evaluate_media_query(&q, &ctx_print));
    }

    #[test]
    fn test_eval_print_type() {
        let q = first_query("print");
        let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
        let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
        assert!(evaluate_media_query(&q, &ctx_print));
        assert!(!evaluate_media_query(&q, &ctx_screen));
    }

    #[test]
    fn test_eval_orientation_portrait() {
        let q = first_query("(orientation: portrait)");
        let portrait = MediaContext::new(400.0, 800.0);
        let landscape = MediaContext::new(800.0, 400.0);
        assert!(evaluate_media_query(&q, &portrait));
        assert!(!evaluate_media_query(&q, &landscape));
    }

    #[test]
    fn test_eval_orientation_landscape() {
        let q = first_query("(orientation: landscape)");
        let landscape = MediaContext::new(1024.0, 768.0);
        let portrait = MediaContext::new(400.0, 800.0);
        assert!(evaluate_media_query(&q, &landscape));
        assert!(!evaluate_media_query(&q, &portrait));
    }

    #[test]
    fn test_eval_multiple_conditions_and() {
        let q = first_query("(min-width: 600px) and (max-width: 1024px)");
        // 在范围内
        assert!(evaluate_media_query(&q, &MediaContext::new(800.0, 600.0)));
        // 太小
        assert!(!evaluate_media_query(&q, &MediaContext::new(400.0, 300.0)));
        // 太大
        assert!(!evaluate_media_query(&q, &MediaContext::new(1200.0, 800.0)));
    }

    #[test]
    fn test_eval_negated() {
        let q = first_query("not screen");
        let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
        let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
        assert!(!evaluate_media_query(&q, &ctx_screen));
        assert!(evaluate_media_query(&q, &ctx_print));
    }

    #[test]
    fn test_eval_no_condition_always_true() {
        let q = MediaQuery {
            media_type: None,
            negated: false,
            conditions: vec![],
        };
        let ctx = MediaContext::new(100.0, 100.0);
        assert!(evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_boundary_exact() {
        let q = first_query("(min-width: 600px)");
        // 恰好 600px 应该通过
        assert!(evaluate_media_query(&q, &MediaContext::new(600.0, 400.0)));
        // 599.99 不通过
        assert!(!evaluate_media_query(&q, &MediaContext::new(599.99, 400.0)));
    }

    // ── 辅助函数测试 ──

    #[test]
    fn test_parse_px_value() {
        assert_eq!(parse_px_value("600px"), Some(600.0));
        assert_eq!(parse_px_value("600"), Some(600.0));
        assert_eq!(parse_px_value("50.5px"), Some(50.5));
        assert_eq!(parse_px_value("invalid"), None);
    }

    #[test]
    fn test_find_matching_paren() {
        assert_eq!(find_matching_paren("(test)"), Some(5));
        assert_eq!(find_matching_paren("(test) and (more)"), Some(5));
        assert_eq!(find_matching_paren("no paren"), None);
    }

    // ── "only" 前缀测试 ──

    #[test]
    fn test_parse_only_screen() {
        // "only screen" 应等价于 "screen"
        let q = first_query("only screen");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert!(!q.negated);
        assert!(q.conditions.is_empty());
    }

    #[test]
    fn test_parse_only_screen_with_conditions() {
        let q = first_query("only screen and (min-width: 600px)");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
    }

    // ── 逗号分隔 OR 查询测试 ──

    #[test]
    fn test_comma_separated_or_queries() {
        let queries = parse_media_query("screen, print").unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].media_type, Some(MediaType::Screen));
        assert_eq!(queries[1].media_type, Some(MediaType::Print));
    }

    #[test]
    fn test_comma_separated_or_evaluation() {
        // screen, print — screen 上下文应匹配（第一个）
        let queries = parse_media_query("screen, print").unwrap();
        let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
        let screen_matches = queries.iter().any(|q| evaluate_media_query(q, &ctx_screen));
        assert!(screen_matches);

        // print 上下文也应匹配（第二个）
        let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
        let print_matches = queries.iter().any(|q| evaluate_media_query(q, &ctx_print));
        assert!(print_matches);
    }

    #[test]
    fn test_comma_separated_with_conditions() {
        let queries = parse_media_query("screen and (min-width: 600px), print").unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].media_type, Some(MediaType::Screen));
        assert_eq!(queries[0].conditions.len(), 1);
        assert_eq!(queries[1].media_type, Some(MediaType::Print));
    }

    // ── prefers-color-scheme 测试 ──

    #[test]
    fn test_parse_prefers_color_scheme_dark() {
        let q = first_query("(prefers-color-scheme: dark)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::PrefersColorScheme(PrefersColorSchemeValue::Dark)
        );
    }

    #[test]
    fn test_parse_prefers_color_scheme_light() {
        let q = first_query("(prefers-color-scheme: light)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::PrefersColorScheme(PrefersColorSchemeValue::Light)
        );
    }

    #[test]
    fn test_eval_prefers_color_scheme() {
        let q = first_query("(prefers-color-scheme: dark)");

        // 深色模式上下文应匹配
        let mut ctx_dark = MediaContext::new(1024.0, 768.0);
        ctx_dark.prefers_color_scheme = PrefersColorSchemeValue::Dark;
        assert!(evaluate_media_query(&q, &ctx_dark));

        // 浅色模式上下文不应匹配
        let ctx_light = MediaContext::new(1024.0, 768.0);
        assert!(!evaluate_media_query(&q, &ctx_light));
    }

    #[test]
    fn test_prefers_color_scheme_not_boolean() {
        // "(prefers-color-scheme)" 无值，不产生有效条件
        let queries = parse_media_query("(prefers-color-scheme)").unwrap();
        assert_eq!(queries[0].conditions.len(), 0);
    }

    // ── prefers-reduced-motion 测试 ──

    #[test]
    fn test_parse_prefers_reduced_motion_reduce() {
        let q = first_query("(prefers-reduced-motion: reduce)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::PrefersReducedMotion(ReducedMotionValue::Reduce)
        );
    }

    #[test]
    fn test_parse_prefers_reduced_motion_no_preference() {
        let q = first_query("(prefers-reduced-motion: no-preference)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::PrefersReducedMotion(ReducedMotionValue::NoPreference)
        );
    }

    #[test]
    fn test_eval_prefers_reduced_motion() {
        let q = first_query("(prefers-reduced-motion: reduce)");

        let mut ctx_reduce = MediaContext::new(1024.0, 768.0);
        ctx_reduce.prefers_reduced_motion = ReducedMotionValue::Reduce;
        assert!(evaluate_media_query(&q, &ctx_reduce));

        let ctx_no_pref = MediaContext::new(1024.0, 768.0);
        assert!(!evaluate_media_query(&q, &ctx_no_pref));
    }

    #[test]
    fn test_prefers_reduced_motion_boolean() {
        // "(prefers-reduced-motion)" 布尔形式 — 当 reduce 时为真
        let q = first_query("(prefers-reduced-motion)");
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(
            q.conditions[0],
            MediaCondition::PrefersReducedMotion(ReducedMotionValue::Reduce)
        );

        let mut ctx_reduce = MediaContext::new(1024.0, 768.0);
        ctx_reduce.prefers_reduced_motion = ReducedMotionValue::Reduce;
        assert!(evaluate_media_query(&q, &ctx_reduce));

        let mut ctx_no_pref = MediaContext::new(1024.0, 768.0);
        ctx_no_pref.prefers_reduced_motion = ReducedMotionValue::NoPreference;
        assert!(!evaluate_media_query(&q, &ctx_no_pref));
    }

    // ── pointer 测试 ──

    #[test]
    fn test_parse_pointer_none() {
        let q = first_query("(pointer: none)");
        assert_eq!(q.conditions[0], MediaCondition::Pointer(PointerValue::None));
    }

    #[test]
    fn test_parse_pointer_coarse() {
        let q = first_query("(pointer: coarse)");
        assert_eq!(q.conditions[0], MediaCondition::Pointer(PointerValue::Coarse));
    }

    #[test]
    fn test_parse_pointer_fine() {
        let q = first_query("(pointer: fine)");
        assert_eq!(q.conditions[0], MediaCondition::Pointer(PointerValue::Fine));
    }

    #[test]
    fn test_eval_pointer() {
        let q = first_query("(pointer: fine)");

        let mut ctx_fine = MediaContext::new(1024.0, 768.0);
        ctx_fine.pointer_type = PointerValue::Fine;
        assert!(evaluate_media_query(&q, &ctx_fine));

        let mut ctx_coarse = MediaContext::new(1024.0, 768.0);
        ctx_coarse.pointer_type = PointerValue::Coarse;
        assert!(!evaluate_media_query(&q, &ctx_coarse));
    }

    #[test]
    fn test_pointer_boolean() {
        // "(pointer)" 布尔形式 — 当指针不为 none 时为真
        let q = first_query("(pointer)");
        assert_eq!(q.conditions.len(), 1);

        let mut ctx_coarse = MediaContext::new(1024.0, 768.0);
        ctx_coarse.pointer_type = PointerValue::Coarse;
        assert!(evaluate_media_query(&q, &ctx_coarse));

        let mut ctx_fine = MediaContext::new(1024.0, 768.0);
        ctx_fine.pointer_type = PointerValue::Fine;
        // 布尔形式解析为 Pointer(Coarse)，fine 上下文不匹配
        assert!(!evaluate_media_query(&q, &ctx_fine));

        let mut ctx_none = MediaContext::new(1024.0, 768.0);
        ctx_none.pointer_type = PointerValue::None;
        assert!(!evaluate_media_query(&q, &ctx_none));
    }

    // ── resolution 测试 ──

    #[test]
    fn test_parse_resolution_exact() {
        let q = first_query("(resolution: 150dpi)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Resolution(MediaFeatureOp::Exact, 150.0)
        );
    }

    #[test]
    fn test_parse_min_resolution() {
        let q = first_query("(min-resolution: 96dpi)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Resolution(MediaFeatureOp::GreaterEqual, 96.0)
        );
    }

    #[test]
    fn test_parse_max_resolution() {
        let q = first_query("(max-resolution: 300dpi)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Resolution(MediaFeatureOp::LessEqual, 300.0)
        );
    }

    #[test]
    fn test_eval_resolution_exact() {
        let q = first_query("(resolution: 96dpi)");

        let mut ctx_96 = MediaContext::new(1024.0, 768.0);
        ctx_96.resolution_dpi = 96.0;
        assert!(evaluate_media_query(&q, &ctx_96));

        let mut ctx_150 = MediaContext::new(1024.0, 768.0);
        ctx_150.resolution_dpi = 150.0;
        assert!(!evaluate_media_query(&q, &ctx_150));
    }

    #[test]
    fn test_eval_min_resolution() {
        let q = first_query("(min-resolution: 150dpi)");

        let mut ctx_200 = MediaContext::new(1024.0, 768.0);
        ctx_200.resolution_dpi = 200.0;
        assert!(evaluate_media_query(&q, &ctx_200));

        let mut ctx_96 = MediaContext::new(1024.0, 768.0);
        ctx_96.resolution_dpi = 96.0;
        assert!(!evaluate_media_query(&q, &ctx_96));
    }

    #[test]
    fn test_eval_max_resolution() {
        let q = first_query("(max-resolution: 150dpi)");

        let mut ctx_96 = MediaContext::new(1024.0, 768.0);
        ctx_96.resolution_dpi = 96.0;
        assert!(evaluate_media_query(&q, &ctx_96));

        let mut ctx_200 = MediaContext::new(1024.0, 768.0);
        ctx_200.resolution_dpi = 200.0;
        assert!(!evaluate_media_query(&q, &ctx_200));
    }

    // ── MediaContext 默认值测试 ──

    #[test]
    fn test_media_context_defaults() {
        let ctx = MediaContext::new(800.0, 600.0);
        assert_eq!(ctx.media_type, MediaType::Screen);
        assert_eq!(ctx.prefers_color_scheme, PrefersColorSchemeValue::Light);
        assert_eq!(ctx.prefers_reduced_motion, ReducedMotionValue::NoPreference);
        assert_eq!(ctx.pointer_type, PointerValue::Coarse);
        assert!((ctx.resolution_dpi - 96.0).abs() < 0.01);
    }

    #[test]
    fn test_media_context_with_type_defaults() {
        let ctx = MediaContext::with_type(800.0, 600.0, MediaType::Print);
        assert_eq!(ctx.media_type, MediaType::Print);
        assert_eq!(ctx.prefers_color_scheme, PrefersColorSchemeValue::Light);
        assert_eq!(ctx.prefers_reduced_motion, ReducedMotionValue::NoPreference);
        assert_eq!(ctx.pointer_type, PointerValue::Coarse);
        assert!((ctx.resolution_dpi - 96.0).abs() < 0.01);
    }

    // ── parse_dpi_value 辅助函数测试 ──

    #[test]
    fn test_parse_dpi_value() {
        assert_eq!(parse_dpi_value("96dpi"), Some(96.0));
        assert_eq!(parse_dpi_value("150dpi"), Some(150.0));
        assert_eq!(parse_dpi_value("96"), Some(96.0));
        assert_eq!(parse_dpi_value("invalid"), None);
    }

    // ── CSS Values 4 §7.3 <resolution> 全单位（dpi/dpcm/dppx/x）转换测试 ──

    #[test]
    fn test_parse_dpi_value_dppx() {
        // 1 dppx = 96 dpi；2 dppx = 192 dpi
        assert_eq!(parse_dpi_value("1dppx"), Some(96.0));
        assert_eq!(parse_dpi_value("2dppx"), Some(192.0));
        assert_eq!(parse_dpi_value("1.5dppx"), Some(144.0));
    }

    #[test]
    fn test_parse_dpi_value_dpcm() {
        // 1 dpcm = 2.54 dpi；10 dpcm = 25.4 dpi
        assert_eq!(parse_dpi_value("1dpcm"), Some(2.54));
        assert_eq!(parse_dpi_value("10dpcm"), Some(25.4));
    }

    #[test]
    fn test_parse_dpi_value_x_alias() {
        // x 是 dppx 的别名（CSS Values 4 §7.3）：1 x = 96 dpi
        assert_eq!(parse_dpi_value("1x"), Some(96.0));
        assert_eq!(parse_dpi_value("2x"), Some(192.0));
    }

    #[test]
    fn test_parse_dpi_value_case_insensitive() {
        // CSS 单位大小写不敏感（CSS Syntax §4.3）
        assert_eq!(parse_dpi_value("96DPI"), Some(96.0));
        assert_eq!(parse_dpi_value("2DPPX"), Some(192.0));
        assert_eq!(parse_dpi_value("1X"), Some(96.0));
    }

    #[test]
    fn test_parse_resolution_dppx_full_query() {
        // 端到端：@media (resolution: 2dppx) 应转换为 Resolution(Exact, 192 dpi)
        let q = first_query("(resolution: 2dppx)");
        assert_eq!(
            q.conditions[0],
            MediaCondition::Resolution(MediaFeatureOp::Exact, 192.0)
        );
    }

    #[test]
    fn test_eval_resolution_dppx() {
        // 2dppx = 192 dpi：在 192dpi 设备上匹配，在 96dpi 设备上不匹配
        let q = first_query("(resolution: 2dppx)");

        let mut ctx_192 = MediaContext::new(1024.0, 768.0);
        ctx_192.resolution_dpi = 192.0;
        assert!(evaluate_media_query(&q, &ctx_192));

        let mut ctx_96 = MediaContext::new(1024.0, 768.0);
        ctx_96.resolution_dpi = 96.0;
        assert!(!evaluate_media_query(&q, &ctx_96));
    }

    // ── 组合测试：新特性与媒体类型混合 ──

    #[test]
    fn test_screen_with_prefers_color_scheme() {
        let q = first_query("screen and (prefers-color-scheme: dark)");
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 1);

        let mut ctx = MediaContext::new(1024.0, 768.0);
        ctx.prefers_color_scheme = PrefersColorSchemeValue::Dark;
        assert!(evaluate_media_query(&q, &ctx));

        let ctx_light = MediaContext::new(1024.0, 768.0);
        assert!(!evaluate_media_query(&q, &ctx_light));
    }
}
