//! css-gaps-1 rule-color 列表文法（<line-color-list> | <auto-line-color-list>）。
//!
//! css-gaps-1 把 rule 族 color 重定义为列表文法（css-multicol-3 的纯值为其
//! 单段退化形）：
//!
//! ```text
//! <line-color-list>       = [ <line-color-or-repeat> ]+
//! <auto-line-color-list>  = [ <line-color-or-repeat> ]* <auto-repeat-line-color> [ <line-color-or-repeat> ]*
//! <line-color-or-repeat>  = [ <color> | <repeat-line-color> ]
//! <repeat-line-color>     = repeat( [ <integer [1,∞]> ] , [ <color> ]# )
//! <auto-repeat-line-color> = repeat( auto , [ <color> ]# )
//! ```
//!
//! 纯 `<color>` = 单段列表（multicol 兼容面：现有纯色声明串化不变）。
//! driving: css-gaps 语料 gap-decorations-color-{valid,computed,invalid} +
//! rule-color 插值五页（wptfyi-css gaps 刀 A）。
//! https://drafts.csswg.org/css-gaps-1/#column-row-rule-color

use super::color::{parse_color, parse_color_with_scheme};
use super::types::ColorValue;

/// repeat() 计数：`repeat([<integer [1,∞]>] | auto, <item>#)`。calc 整数形
///（`repeat(calc(5 + 3), salmon)`，computed 页钉）保留表达式，computed 串化
/// 期折叠（[`rule_color_list_to_css`]）。rule-color/rule-width/rule-style
/// 三列表档共享（css-gaps-1 计数文法同型）。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleRepeatCount {
    /// auto —— 逐槽自动重复（插值期结构配对）。
    Auto,
    /// 非负整数循环次数（≥1；`repeat(0,…)`/`repeat(-1,…)` parse 拒）。
    N(u32),
    /// calc 整数形（原始实参文本，computed 期 eval 折叠整数）。
    Calc(String),
}

impl RuleRepeatCount {
    /// computed 串化头部：auto 保 `auto`、整数直接串化、calc eval 折叠整数
    ///（`repeat(calc(5 + 3),…)` → `repeat(8, …)`，computed 页钉）。折叠失败
    /// 返 None（调用方回落原文）。
    pub fn to_css_head(&self) -> Option<String> {
        match self {
            RuleRepeatCount::Auto => Some("auto".to_string()),
            RuleRepeatCount::N(n) => Some(n.to_string()),
            RuleRepeatCount::Calc(expr) => {
                // calc 整数折叠：parse_calc → 无 ctx eval（纯 number 树）→
                // 四舍五入整（`repeat(calc(5 + 3),…)` → 8）。
                let v = super::types::parse_calc(expr)?;
                Some(format!("{}", super::types::eval_calc(&v, None)?.round() as i64))
            }
        }
    }
}

/// 解析 repeat() 头部计数（`auto` | `<integer [1,∞]>` | `calc(…)`）。
/// 0/负/非整数拒（invalid 页 `repeat(0,…)`/`repeat(-1,…)` 钉）。
pub fn parse_rule_repeat_count(head: &str) -> Option<RuleRepeatCount> {
    let head = head.trim();
    if head.eq_ignore_ascii_case("auto") {
        return Some(RuleRepeatCount::Auto);
    }
    if head.to_ascii_lowercase().starts_with("calc(") {
        return Some(RuleRepeatCount::Calc(head.to_string()));
    }
    // <integer [1,∞]>：可选 + 号整数。
    let t = head.trim_start_matches('+');
    let n: i64 = t.parse().ok()?;
    if n < 1 {
        return None;
    }
    Some(RuleRepeatCount::N(n as u32))
}

/// 单段：<color> 或 repeat() 块。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleColorSegment {
    /// 平色段。
    Plain(ColorValue),
    /// repeat 块：计数 + 颜色循环体（≥1 色）。
    Repeat {
        /// 计数。
        count: RuleRepeatCount,
        /// 循环体颜色（≥1）。
        colors: Vec<ColorValue>,
    },
}

/// 括号感知 top-level 逗号切分（parse_extended_border 同款；深度负数/失衡拒）。
/// rule-color/rule-width/rule-style 三列表档共享（values:: 导出）。
pub fn split_top_level_commas(s: &str) -> Option<Vec<&str>> {
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

fn parse_color_ctx(v: &str, dark: Option<bool>) -> Option<ColorValue> {
    match dark {
        Some(d) => parse_color_with_scheme(v, d),
        None => parse_color(v),
    }
}

/// 解析单段（`<color> | repeat(<count>, <color>#)`）。repeat 内颜色为逗号
/// 列表（空格分隔 `red blue green` 拒——invalid 页钉）；计数解析走
/// [`RuleRepeatCount::parse_head`]（整数 ≥1、auto、calc 形）。
fn parse_rule_color_segment(seg: &str, dark: Option<bool>) -> Option<RuleColorSegment> {
    let seg = seg.trim();
    let lower = seg.to_ascii_lowercase();
    if lower.starts_with("repeat(") && lower.ends_with(')') {
        let inner = &seg[7..seg.len() - 1];
        let (head, colors_text) = inner.split_once(',')?;
        let count = parse_rule_repeat_count(head)?;
        let mut colors = Vec::new();
        for c in split_top_level_commas(colors_text)? {
            colors.push(parse_color_ctx(c, dark)?);
        }
        if colors.is_empty() {
            return None;
        }
        return Some(RuleColorSegment::Repeat { count, colors });
    }
    Some(RuleColorSegment::Plain(parse_color_ctx(seg, dark)?))
}

/// 解析 rule-color 列表值（`<line-color-list> | <auto-line-color-list>`）：
/// ≥1 段、auto repeat 至多一处（invalid 页双 auto 钉）。纯色 = 单段列表。
pub fn parse_rule_color_list(value: &str) -> Option<Vec<RuleColorSegment>> {
    parse_rule_color_list_impl(value, None)
}

/// 同 [`parse_rule_color_list`]，颜色段带 system-color dark scheme 上下文
///（column-rule-color 既有臂 parse_color_with_scheme 同口径）。
pub fn parse_rule_color_list_with_scheme(value: &str, dark: bool) -> Option<Vec<RuleColorSegment>> {
    parse_rule_color_list_impl(value, Some(dark))
}

fn parse_rule_color_list_impl(value: &str, dark: Option<bool>) -> Option<Vec<RuleColorSegment>> {
    let mut segments = Vec::new();
    let mut autos = 0usize;
    for seg in split_top_level_commas(value)? {
        let s = parse_rule_color_segment(seg, dark)?;
        if matches!(
            &s,
            RuleColorSegment::Repeat {
                count: RuleRepeatCount::Auto,
                ..
            }
        ) {
            autos += 1;
            if autos > 1 {
                return None;
            }
        }
        segments.push(s);
    }
    if segments.is_empty() {
        return None;
    }
    Some(segments)
}

/// 纯单段平色提取（multicol 保形通道：gCS/paint/shorthand 取色）。
pub fn rule_color_list_single_color(segments: &[RuleColorSegment]) -> Option<&ColorValue> {
    if segments.len() == 1 {
        if let RuleColorSegment::Plain(c) = &segments[0] {
            return Some(c);
        }
    }
    None
}

/// 首色提取（paint 通道：列表形取首段首色作避让装饰色；恒有段故对默认
/// 单段值恒 Some）。
pub fn rule_color_list_first_color(segments: &[RuleColorSegment]) -> Option<&ColorValue> {
    segments.first().and_then(|s| match s {
        RuleColorSegment::Plain(c) => Some(c),
        RuleColorSegment::Repeat { colors, .. } => colors.first(),
    })
}

/// computed 串化（css-gaps-1 §column-row-rule-color computed 形）：颜色逐段
/// resolve（currentcolor→元素色、color-mix→解析形），repeat 结构保形、auto
/// 保 `repeat(auto, …)`、calc 计数 eval 折叠整数（`repeat(calc(5 + 3),…)` →
/// `repeat(8, …)`，computed 页钉）。折叠失败段返 None（调用方回落原文）。
pub fn rule_color_list_to_css(
    segments: &[RuleColorSegment],
    resolve: &dyn Fn(&ColorValue) -> String,
) -> Option<String> {
    let mut parts = Vec::with_capacity(segments.len());
    for seg in segments {
        match seg {
            RuleColorSegment::Plain(c) => parts.push(resolve(c)),
            RuleColorSegment::Repeat { count, colors } => {
                let head = count.to_css_head()?;
                let cs: Vec<String> = colors.iter().map(resolve).collect();
                parts.push(format!("repeat({}, {})", head, cs.join(", ")));
            }
        }
    }
    Some(parts.join(", "))
}

/// 段数（有限 repeat 展开后平列表长度，插值对齐用；auto 段计其循环体长）。
pub fn rule_color_list_expanded_len(segments: &[RuleColorSegment]) -> usize {
    segments
        .iter()
        .map(|s| match s {
            RuleColorSegment::Plain(_) => 1,
            RuleColorSegment::Repeat { colors, .. } => colors.len(),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_color_list_valid_forms() {
        // valid 页 12 形代表（三属性共享文法）。
        for v in [
            "repeat(4, blue)",
            "repeat(3, red, green, blue)",
            "repeat(1, red, green, blue, yellow)",
            "red",
            "repeat(4, cyan)",
            "green, blue",
            "red, blue, green, yellow, pink",
            "repeat(3, blue), repeat(4, red)",
            "red, repeat(3, blue), blue, repeat(4, red, green, cyan)",
            "repeat(4, red, green, cyan), repeat(5, yellow), blue",
            "repeat(auto, red)",
            "repeat(auto, red, green, blue)",
            "repeat(auto, red, green), red",
            "repeat(4, blue, red, green), repeat(auto, red)",
            "blue, repeat(auto, red, green), repeat(4, blue, red, green)",
            "repeat(calc(5 + 3), salmon)",
        ] {
            assert!(parse_rule_color_list(v).is_some(), "valid form rejected: {v}");
        }
    }

    #[test]
    fn test_rule_color_list_invalid_forms() {
        // invalid 页 6 形（三属性共享）。
        for v in [
            "auto",
            "red blue green",
            "repeat(auto, red blue green)",
            "repeat(0, red, blue, green)",
            "repeat(-1, red, blue, green)",
            "repeat(auto, red), red, repeat(auto, blue)",
            "",
        ] {
            assert!(parse_rule_color_list(v).is_none(), "invalid form accepted: {v}");
        }
    }

    #[test]
    fn test_rule_color_list_pure_color_compat() {
        // multicol 纯色 = 单段平色（parse_color 命名色归一 Rgba）。
        let segs = parse_rule_color_list("red").unwrap();
        assert!(matches!(
            rule_color_list_single_color(&segs),
            Some(ColorValue::Rgba(255, 0, 0, 255))
        ));
        // 列表形不退化。
        let segs = parse_rule_color_list("red, blue").unwrap();
        assert!(rule_color_list_single_color(&segs).is_none());
    }

    #[test]
    fn test_rule_color_list_to_css_computed() {
        // computed 页钉：calc 计数折叠整数、currentcolor resolve、结构保形。
        let resolve = |c: &ColorValue| -> String {
            match c {
                ColorValue::Rgba(250, 128, 114, 255) => "rgb(250, 128, 114)".to_string(),
                ColorValue::CurrentColor => "rgb(0, 255, 0)".to_string(),
                _ => format!("{c:?}"),
            }
        };
        let segs = parse_rule_color_list("repeat(calc(5 + 3), salmon)").unwrap();
        assert_eq!(
            rule_color_list_to_css(&segs, &resolve).unwrap(),
            "repeat(8, rgb(250, 128, 114))"
        );
        let segs = parse_rule_color_list("currentcolor").unwrap();
        assert_eq!(rule_color_list_to_css(&segs, &resolve).unwrap(), "rgb(0, 255, 0)");
        let segs = parse_rule_color_list("repeat(auto, currentcolor)").unwrap();
        assert_eq!(
            rule_color_list_to_css(&segs, &resolve).unwrap(),
            "repeat(auto, rgb(0, 255, 0))"
        );
    }
}
