//! CSS 媒体查询解析与评估。
//!
//! 支持解析常见的媒体查询条件并在给定视口和媒体类型下评估。
//!
//! ## 支持的媒体特性
//!
//! - `width`, `min-width`, `max-width`
//! - `height`, `min-height`, `max-height`
//! - `orientation` (portrait/landscape)
//! - Level 4 范围语法：`width > 600px`、`width >= 600px`、`width < 1000px`、`width <= 1000px`
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

/// 媒体查询评估上下文。
#[derive(Debug, Clone)]
pub struct MediaContext {
    /// 视口宽度（px）。
    pub viewport_width: f64,
    /// 视口高度（px）。
    pub viewport_height: f64,
    /// 当前媒体类型。
    pub media_type: MediaType,
}

impl MediaContext {
    /// 创建新的媒体上下文。
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            media_type: MediaType::Screen,
        }
    }

    /// 创建指定媒体类型的上下文。
    pub fn with_type(width: f64, height: f64, media_type: MediaType) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            media_type,
        }
    }
}

/// 解析 @media 规则的 prelude 字符串为 MediaQuery。
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
pub fn parse_media_query(input: &str) -> Option<MediaQuery> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut remaining = input;
    let mut negated = false;
    let mut media_type = None;

    // 处理 "not" 前缀
    if remaining.starts_with("not ") || remaining.starts_with("NOT ") {
        negated = true;
        remaining = remaining[4..].trim_start();
    }

    // 尝试提取媒体类型
    if remaining.starts_with("screen") {
        let after = remaining[6..].trim_start();
        if after.is_empty() || after.starts_with("and") {
            media_type = Some(MediaType::Screen);
            remaining = after.strip_prefix("and").unwrap_or(after).trim_start();
        }
    } else if remaining.starts_with("print") {
        let after = remaining[5..].trim_start();
        if after.is_empty() || after.starts_with("and") {
            media_type = Some(MediaType::Print);
            remaining = after.strip_prefix("and").unwrap_or(after).trim_start();
        }
    } else if remaining.starts_with("all") {
        let after = remaining[3..].trim_start();
        if after.is_empty() || after.starts_with("and") {
            media_type = Some(MediaType::All);
            remaining = after.strip_prefix("and").unwrap_or(after).trim_start();
        }
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

/// 检查字符串是否包含范围运算符（`<`, `<=`, `>`, `>=`）。
fn contains_range_op(s: &str) -> bool {
    s.as_bytes().iter().any(|&b| b == b'<' || b == b'>')
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
fn find_range_op_pos(s: &str) -> Option<usize> {
    s.as_bytes().iter().position(|&b| b == b'<' || b == b'>')
}

/// 从 CSS 值字符串解析像素数值。
///
/// 支持 `"600px"`、`"600"`、`"50.5px"` 等格式。
fn parse_px_value(s: &str) -> Option<f64> {
    let s = s.trim();
    let s = s.strip_suffix("px").unwrap_or(s);
    s.parse::<f64>().ok()
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

    // ── 解析测试 ──

    #[test]
    fn test_parse_simple_min_width() {
        let q = parse_media_query("(min-width: 600px)").unwrap();
        assert_eq!(q.media_type, None);
        assert!(!q.negated);
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
    }

    #[test]
    fn test_parse_max_width() {
        let q = parse_media_query("(max-width: 768px)").unwrap();
        assert_eq!(q.conditions[0], MediaCondition::MaxWidth(768.0));
    }

    #[test]
    fn test_parse_screen_and() {
        let q = parse_media_query("screen and (min-width: 600px)").unwrap();
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 1);
    }

    #[test]
    fn test_parse_print() {
        let q = parse_media_query("print").unwrap();
        assert_eq!(q.media_type, Some(MediaType::Print));
        assert!(q.conditions.is_empty());
    }

    #[test]
    fn test_parse_multiple_conditions() {
        let q = parse_media_query("screen and (min-width: 600px) and (max-width: 1024px)").unwrap();
        assert_eq!(q.media_type, Some(MediaType::Screen));
        assert_eq!(q.conditions.len(), 2);
        assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
        assert_eq!(q.conditions[1], MediaCondition::MaxWidth(1024.0));
    }

    #[test]
    fn test_parse_orientation() {
        let q = parse_media_query("(orientation: landscape)").unwrap();
        assert_eq!(
            q.conditions[0],
            MediaCondition::Orientation(OrientationValue::Landscape)
        );
    }

    #[test]
    fn test_parse_height() {
        let q = parse_media_query("(min-height: 400px)").unwrap();
        assert_eq!(q.conditions[0], MediaCondition::MinHeight(400.0));
    }

    #[test]
    fn test_parse_not() {
        let q = parse_media_query("not screen").unwrap();
        assert!(q.negated);
        assert_eq!(q.media_type, Some(MediaType::Screen));
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_media_query("").is_none());
    }

    #[test]
    fn test_parse_just_parentheses() {
        let q = parse_media_query("(width: 800px)").unwrap();
        assert_eq!(q.conditions[0], MediaCondition::Width(MediaFeatureOp::Exact, 800.0));
    }

    // ── Level 4 范围语法测试 ──

    #[test]
    fn test_media_range_greater_than() {
        // (width > 600px) — 大于，不含 600
        let q = parse_media_query("(width > 600px)").unwrap();
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
        let q = parse_media_query("(width < 1000px)").unwrap();
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
        let q = parse_media_query("(600px <= width <= 1000px)").unwrap();
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
        let q = parse_media_query("(hover)").unwrap();
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.conditions[0], MediaCondition::Hover);

        // 布尔特性始终为真（假设支持）
        assert!(evaluate_media_query(&q, &MediaContext::new(1024.0, 768.0)));

        // (color) — 布尔特性
        let q_color = parse_media_query("(color)").unwrap();
        assert_eq!(q_color.conditions.len(), 1);
        assert_eq!(q_color.conditions[0], MediaCondition::Color);
        assert!(evaluate_media_query(&q_color, &MediaContext::new(1024.0, 768.0)));
    }

    #[test]
    fn test_media_range_greater_equal() {
        // (width >= 600px) — 大于等于
        let q = parse_media_query("(width >= 600px)").unwrap();
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
        let q = parse_media_query("(width <= 1000px)").unwrap();
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
        let q = parse_media_query("(height > 400px)").unwrap();
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
        let q = parse_media_query("screen and (600px <= width <= 1000px)").unwrap();
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
        let q = parse_media_query("(min-width: 600px)").unwrap();
        let ctx = MediaContext::new(1024.0, 768.0);
        assert!(evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_min_width_fail() {
        let q = parse_media_query("(min-width: 600px)").unwrap();
        let ctx = MediaContext::new(400.0, 300.0);
        assert!(!evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_max_width_pass() {
        let q = parse_media_query("(max-width: 768px)").unwrap();
        let ctx = MediaContext::new(600.0, 400.0);
        assert!(evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_max_width_fail() {
        let q = parse_media_query("(max-width: 768px)").unwrap();
        let ctx = MediaContext::new(1024.0, 768.0);
        assert!(!evaluate_media_query(&q, &ctx));
    }

    #[test]
    fn test_eval_screen_type() {
        let q = parse_media_query("screen and (min-width: 600px)").unwrap();
        let ctx_screen = MediaContext::with_type(1024.0, 768.0, MediaType::Screen);
        let ctx_print = MediaContext::with_type(1024.0, 768.0, MediaType::Print);
        assert!(evaluate_media_query(&q, &ctx_screen));
        assert!(!evaluate_media_query(&q, &ctx_print));
    }

    #[test]
    fn test_eval_print_type() {
        let q = parse_media_query("print").unwrap();
        let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
        let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
        assert!(evaluate_media_query(&q, &ctx_print));
        assert!(!evaluate_media_query(&q, &ctx_screen));
    }

    #[test]
    fn test_eval_orientation_portrait() {
        let q = parse_media_query("(orientation: portrait)").unwrap();
        let portrait = MediaContext::new(400.0, 800.0);
        let landscape = MediaContext::new(800.0, 400.0);
        assert!(evaluate_media_query(&q, &portrait));
        assert!(!evaluate_media_query(&q, &landscape));
    }

    #[test]
    fn test_eval_orientation_landscape() {
        let q = parse_media_query("(orientation: landscape)").unwrap();
        let landscape = MediaContext::new(1024.0, 768.0);
        let portrait = MediaContext::new(400.0, 800.0);
        assert!(evaluate_media_query(&q, &landscape));
        assert!(!evaluate_media_query(&q, &portrait));
    }

    #[test]
    fn test_eval_multiple_conditions_and() {
        let q = parse_media_query("(min-width: 600px) and (max-width: 1024px)").unwrap();
        // 在范围内
        assert!(evaluate_media_query(&q, &MediaContext::new(800.0, 600.0)));
        // 太小
        assert!(!evaluate_media_query(&q, &MediaContext::new(400.0, 300.0)));
        // 太大
        assert!(!evaluate_media_query(&q, &MediaContext::new(1200.0, 800.0)));
    }

    #[test]
    fn test_eval_negated() {
        let q = parse_media_query("not screen").unwrap();
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
        let q = parse_media_query("(min-width: 600px)").unwrap();
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
}
