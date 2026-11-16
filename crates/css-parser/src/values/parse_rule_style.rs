//! css-gaps-1 rule-style 列表文法（<line-style-list> | <auto-line-style-list>）。
//!
//! css-gaps-1 把 rule 族 style 重定义为列表文法（css-multicol-3 的纯关键字
//! 为其单段退化形）：
//!
//! ```text
//! <line-style-list>       = [ <line-style-or-repeat> ]+
//! <auto-line-style-list>  = [ <line-style-or-repeat> ]* <auto-repeat-line-style> [ <line-style-or-repeat> ]*
//! <line-style-or-repeat>  = [ <line-style> | <repeat-line-style> ]
//! <repeat-line-style>     = repeat( [ <integer [1,∞]> ] , [ <line-style> ]# )
//! <auto-repeat-line-style> = repeat( auto , [ <line-style> ]# )
//! ```
//!
//! `<line-style>` 复用 [`ColumnRuleStyleValue`] 枚举（10 关键字）。纯关键字
//! = 单段列表（multicol 兼容面：现有标量声明串化不变）。
//! driving: css-gaps 语料 gap-decorations-style-{valid,computed,invalid}
//! （wptfyi-css gaps 刀 B）。
//! https://drafts.csswg.org/css-gaps-1/#column-row-rule-style

use super::parse_layout::ColumnRuleStyleValue;
use super::parse_rule_color::{RuleRepeatCount, parse_rule_repeat_count, split_top_level_commas};

/// 单段：`<line-style>` 或 repeat() 块。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleStyleSegment {
    /// 平样式段。
    Plain(ColumnRuleStyleValue),
    /// repeat 块：计数 + 样式循环体（≥1 值）。
    Repeat {
        /// 计数。
        count: RuleRepeatCount,
        /// 循环体样式（≥1）。
        styles: Vec<ColumnRuleStyleValue>,
    },
}

/// 解析单段（`<line-style> | repeat(<count>, <line-style>#)`）。repeat 内
/// 样式为逗号列表（空格分隔 `groove dotted ridge` 拒——invalid 页钉）。
fn parse_rule_style_segment(seg: &str) -> Option<RuleStyleSegment> {
    let seg = seg.trim();
    let lower = seg.to_ascii_lowercase();
    if lower.starts_with("repeat(") && lower.ends_with(')') {
        let inner = &seg[7..seg.len() - 1];
        let (head, styles_text) = inner.split_once(',')?;
        let count = parse_rule_repeat_count(head)?;
        let mut styles = Vec::new();
        for s in split_top_level_commas(styles_text)? {
            styles.push(parse_line_style(s)?);
        }
        if styles.is_empty() {
            return None;
        }
        return Some(RuleStyleSegment::Repeat { count, styles });
    }
    Some(RuleStyleSegment::Plain(parse_line_style(seg)?))
}

/// `<line-style>`：10 关键字（同 border-style 语义但独立枚举）。
pub fn parse_line_style(value: &str) -> Option<ColumnRuleStyleValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "none" => Some(ColumnRuleStyleValue::None),
        "hidden" => Some(ColumnRuleStyleValue::Hidden),
        "dotted" => Some(ColumnRuleStyleValue::Dotted),
        "dashed" => Some(ColumnRuleStyleValue::Dashed),
        "solid" => Some(ColumnRuleStyleValue::Solid),
        "double" => Some(ColumnRuleStyleValue::Double),
        "groove" => Some(ColumnRuleStyleValue::Groove),
        "ridge" => Some(ColumnRuleStyleValue::Ridge),
        "inset" => Some(ColumnRuleStyleValue::Inset),
        "outset" => Some(ColumnRuleStyleValue::Outset),
        _ => None,
    }
}

/// 解析 rule-style 列表值（`<line-style-list> | <auto-line-style-list>`）：
/// ≥1 段、auto repeat 至多一处（invalid 页双 auto 钉）。纯关键字 = 单段列表。
pub fn parse_rule_style_list(value: &str) -> Option<Vec<RuleStyleSegment>> {
    let mut segments = Vec::new();
    let mut autos = 0usize;
    for seg in split_top_level_commas(value)? {
        let s = parse_rule_style_segment(seg)?;
        if matches!(
            &s,
            RuleStyleSegment::Repeat {
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

/// 纯单段平样式提取（multicol 保形通道：paint/gCS/shorthand 取样式）。
pub fn rule_style_list_single_style(segments: &[RuleStyleSegment]) -> Option<&ColumnRuleStyleValue> {
    if segments.len() == 1 {
        if let RuleStyleSegment::Plain(s) = &segments[0] {
            return Some(s);
        }
    }
    None
}

/// 首样式提取（paint 通道：列表形取首段首样式作分隔线样式；恒有段，缺省
/// 回落 none 语义由调用方处理）。
pub fn rule_style_list_first_style(segments: &[RuleStyleSegment]) -> Option<&ColumnRuleStyleValue> {
    segments.first().and_then(|s| match s {
        RuleStyleSegment::Plain(s) => Some(s),
        RuleStyleSegment::Repeat { styles, .. } => styles.first(),
    })
}

/// computed 串化（css-gaps-1 §column-row-rule-style computed 形）：关键字
/// 逐段小写保形，repeat 结构保形、auto 保 `repeat(auto, …)`、calc 计数 eval
/// 折叠整数（`repeat(calc(5 + 3), ridge)` → `repeat(8, ridge)`，computed 页
/// 钉）。折叠失败段返 None（调用方回落原文）。
pub fn rule_style_list_to_css(segments: &[RuleStyleSegment]) -> Option<String> {
    let mut parts = Vec::with_capacity(segments.len());
    for seg in segments {
        match seg {
            RuleStyleSegment::Plain(s) => parts.push(line_style_str(s).to_string()),
            RuleStyleSegment::Repeat { count, styles } => {
                let head = count.to_css_head()?;
                let ss: Vec<String> = styles.iter().map(|s| line_style_str(s).to_string()).collect();
                parts.push(format!("repeat({}, {})", head, ss.join(", ")));
            }
        }
    }
    Some(parts.join(", "))
}

/// 样式关键字 canonical 小写形（gCS/串化共用）。
pub fn line_style_str(s: &ColumnRuleStyleValue) -> &'static str {
    match s {
        ColumnRuleStyleValue::None => "none",
        ColumnRuleStyleValue::Hidden => "hidden",
        ColumnRuleStyleValue::Dotted => "dotted",
        ColumnRuleStyleValue::Dashed => "dashed",
        ColumnRuleStyleValue::Solid => "solid",
        ColumnRuleStyleValue::Double => "double",
        ColumnRuleStyleValue::Groove => "groove",
        ColumnRuleStyleValue::Ridge => "ridge",
        ColumnRuleStyleValue::Inset => "inset",
        ColumnRuleStyleValue::Outset => "outset",
    }
}

/// 段数（有限 repeat 展开后平列表长度，插值对齐用；auto 段计其循环体长）。
pub fn rule_style_list_expanded_len(segments: &[RuleStyleSegment]) -> usize {
    segments
        .iter()
        .map(|s| match s {
            RuleStyleSegment::Plain(_) => 1,
            RuleStyleSegment::Repeat { styles, .. } => styles.len(),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_style_list_valid_forms() {
        // computed/valid 页全形代表。
        for v in [
            "none",
            "hidden",
            "dotted, dashed, solid",
            "repeat(10, double)",
            "repeat(3, groove), repeat(4, ridge)",
            "repeat(auto, solid)",
            "repeat(auto, dotted, solid, inset)",
            "repeat(4, none, ridge, solid), repeat(auto, hidden)",
            "inset, repeat(auto, solid, ridge), repeat(4, none, groove, hidden)",
            "repeat(calc(5 + 3), ridge)",
            "repeat(4, dotted, double, dashed), repeat(auto, solid), ridge",
            "ridge, repeat(auto, solid), ridge",
        ] {
            assert!(parse_rule_style_list(v).is_some(), "valid form rejected: {v}");
        }
    }

    #[test]
    fn test_rule_style_list_invalid_forms() {
        // invalid 页 6 形（三属性共享）。
        for v in [
            "auto",
            "dashed dotted solid",
            "repeat(auto, groove dotted ridge)",
            "repeat(0, dotted, solid, double)",
            "repeat(-1, dotted, dashed, double)",
            "repeat(auto, dotted), red, repeat(auto, ridge)",
            "",
        ] {
            assert!(parse_rule_style_list(v).is_none(), "invalid form accepted: {v}");
        }
    }

    #[test]
    fn test_rule_style_list_single_and_first() {
        let segs = parse_rule_style_list("solid").unwrap();
        assert!(matches!(
            rule_style_list_single_style(&segs),
            Some(ColumnRuleStyleValue::Solid)
        ));
        let segs = parse_rule_style_list("dotted, dashed").unwrap();
        assert!(rule_style_list_single_style(&segs).is_none());
        let segs = parse_rule_style_list("repeat(2, groove, ridge)").unwrap();
        assert!(matches!(
            rule_style_list_first_style(&segs),
            Some(ColumnRuleStyleValue::Groove)
        ));
    }

    #[test]
    fn test_rule_style_list_to_css_computed() {
        let segs = parse_rule_style_list("repeat(calc(5 + 3), ridge)").unwrap();
        assert_eq!(rule_style_list_to_css(&segs).unwrap(), "repeat(8, ridge)");
        let segs = parse_rule_style_list("repeat(4, none, ridge, solid), repeat(auto, hidden)").unwrap();
        assert_eq!(
            rule_style_list_to_css(&segs).unwrap(),
            "repeat(4, none, ridge, solid), repeat(auto, hidden)"
        );
        let segs = parse_rule_style_list("inset, repeat(auto, solid, ridge), repeat(4, none, groove, hidden)").unwrap();
        assert_eq!(
            rule_style_list_to_css(&segs).unwrap(),
            "inset, repeat(auto, solid, ridge), repeat(4, none, groove, hidden)"
        );
    }
}
