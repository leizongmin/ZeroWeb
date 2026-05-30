//! CSS 媒体查询解析与评估。
//!
//! 支持解析常见的媒体查询条件并在给定视口和媒体类型下评估。
//!
//! ## 支持的媒体特性
//!
//! - `width`, `min-width`, `max-width`
//! - `height`, `min-height`, `max-height`
//! - `orientation` (portrait/landscape)
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
}

/// 媒体特性比较操作。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaFeatureOp {
    /// 精确匹配。
    Exact,
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
        if let Some(cond) = parse_single_condition(inner) {
            conditions.push(cond);
        }

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
        MediaCondition::Width(MediaFeatureOp::Exact, v) => (ctx.viewport_width - *v).abs() < 0.01,
        MediaCondition::MinWidth(v) => ctx.viewport_width >= *v,
        MediaCondition::MaxWidth(v) => ctx.viewport_width <= *v,
        MediaCondition::Height(MediaFeatureOp::Exact, v) => (ctx.viewport_height - *v).abs() < 0.01,
        MediaCondition::MinHeight(v) => ctx.viewport_height >= *v,
        MediaCondition::MaxHeight(v) => ctx.viewport_height <= *v,
        MediaCondition::Orientation(orient) => {
            let is_portrait = ctx.viewport_height > ctx.viewport_width;
            match orient {
                OrientationValue::Portrait => is_portrait,
                OrientationValue::Landscape => !is_portrait,
            }
        }
    }
}

/// 解析单个括号内的条件（如 `min-width: 600px`）。
fn parse_single_condition(s: &str) -> Option<MediaCondition> {
    let colon_pos = s.find(':')?;
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
            // 尝试解析为数值条件
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
        assert_eq!(
            q.conditions[0],
            MediaCondition::Width(MediaFeatureOp::Exact, 800.0)
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
