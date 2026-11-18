//! css-gaps-1 rule-width 列表文法（<line-width-list> | <auto-line-width-list>）。
//!
//! css-gaps-1 把 rule 族 width 重定义为列表文法（css-multicol-3 的纯值为其
//! 单段退化形），段模型与 [`super::parse_rule_color`] 同构：
//!
//! ```text
//! <line-width-list>       = [ <line-width-or-repeat> ]+
//! <auto-line-width-list>  = [ <line-width-or-repeat> ]* <auto-repeat-line-width> [ <line-width-or-repeat> ]*
//! <line-width-or-repeat>  = [ <line-width> | <repeat-line-width> ]
//! <repeat-line-width>     = repeat( [ <integer [1,∞]> ] , [ <line-width> ]# )
//! <auto-repeat-line-width> = repeat( auto , [ <line-width> ]# )
//! <line-width>            = <length [0,∞]> | thin | medium | thick
//! ```
//!
//! 纯 `<line-width>` = 单段列表（multicol 兼容面：现有标量声明串化不变）。
//! driving: css-gaps 语料 gap-decorations-width-{valid,computed,invalid} +
//! rule-width 插值页（wptfyi-css gaps 刀 B）。
//! https://drafts.csswg.org/css-gaps-1/#column-row-rule-width

use super::parse_layout::{ColumnRuleWidthValue, parse_column_rule_width};
use super::parse_rule_color::{RuleRepeatCount, parse_rule_repeat_count, split_top_level_commas};
use super::types::{LengthValue, is_math_function_value, parse_math_function};

/// 单段：<line-width> 或 repeat() 块。条目类型复用 multicol 既有
/// [`ColumnRuleWidthValue`]（thin/medium/thick 保 keyword，负长在
/// [`parse_column_rule_width`] 拒）。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleWidthSegment {
    /// 平宽段。
    Plain(ColumnRuleWidthValue),
    /// repeat 块：计数 + 宽度循环体（≥1 项）。
    Repeat {
        /// 计数。
        count: RuleRepeatCount,
        /// 循环体宽度（≥1）。
        widths: Vec<ColumnRuleWidthValue>,
    },
}

/// 顶层（括号外）空白判定：`<line-width>` 顶层为单 token 文法，空格分隔
/// 复合形（`thin medium thick`——invalid 页钉）拒；数学函数形实参内空格
///（`calc(10px + 0.5em)`，computed 页钉）合法。
fn has_top_level_whitespace(s: &str) -> bool {
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// 解析单条目（`<line-width>`）。顶层单 token 门（括号感知）；`auto` 非本
/// 槽关键字拒；percentage 拒（<length [0,∞]>，invalid 页 `30%` 钉）；数学
/// 函数形（`calc(10px + 0.5em)`/`min(5px, 10px)`，computed 页钉）保 Calc
/// 树，computed 期折叠（负值 clamp 0）。
pub fn parse_line_width_item(seg: &str) -> Option<ColumnRuleWidthValue> {
    let seg = seg.trim();
    if seg.is_empty() || has_top_level_whitespace(seg) {
        return None;
    }
    if seg.eq_ignore_ascii_case("auto") {
        return None;
    }
    if let Some(w) = parse_column_rule_width(seg) {
        if matches!(&w, ColumnRuleWidthValue::Length(LengthValue::Percentage(_))) {
            return None;
        }
        return Some(w);
    }
    // CSS Values 4 §11.3：任意 calc() 合法槽位接受裸数学函数；槽族型别门
    //（LengthPercentage 且无 %）由 apply 写面 math 预门按属性裁决。
    if is_math_function_value(seg) {
        let expr = parse_math_function(seg)?;
        return Some(ColumnRuleWidthValue::Length(LengthValue::Calc(Box::new(expr))));
    }
    None
}

/// 解析单段（`<line-width> | repeat(<count>, <line-width>#)`）。repeat 内
/// 宽度为逗号列表（空格分隔拒——单条目 token 门）；计数整数 ≥1。
fn parse_rule_width_segment(seg: &str) -> Option<RuleWidthSegment> {
    let seg = seg.trim();
    let lower = seg.to_ascii_lowercase();
    if lower.starts_with("repeat(") && lower.ends_with(')') {
        let inner = &seg[7..seg.len() - 1];
        let (head, widths_text) = inner.split_once(',')?;
        let count = parse_rule_repeat_count(head.trim())?;
        let mut widths = Vec::new();
        for w in split_top_level_commas(widths_text)? {
            widths.push(parse_line_width_item(w)?);
        }
        if widths.is_empty() {
            return None;
        }
        return Some(RuleWidthSegment::Repeat { count, widths });
    }
    Some(RuleWidthSegment::Plain(parse_line_width_item(seg)?))
}

/// 解析 rule-width 列表值（`<line-width-list> | <auto-line-width-list>`）：
/// ≥1 段、auto repeat 至多一处（invalid 页双 auto 钉）。纯宽 = 单段列表。
pub fn parse_rule_width_list(value: &str) -> Option<Vec<RuleWidthSegment>> {
    let mut segments = Vec::new();
    let mut autos = 0usize;
    for seg in split_top_level_commas(value)? {
        let s = parse_rule_width_segment(seg)?;
        if matches!(
            &s,
            RuleWidthSegment::Repeat {
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

/// 纯单段平宽提取（multicol 保形通道：gCS/paint/shorthand 取宽）。
pub fn rule_width_list_single_width(segments: &[RuleWidthSegment]) -> Option<&ColumnRuleWidthValue> {
    if segments.len() == 1 {
        if let RuleWidthSegment::Plain(w) = &segments[0] {
            return Some(w);
        }
    }
    None
}

/// 首宽提取（paint 通道：列表形取首段首宽作避让装饰宽；恒有段故对默认
/// 单段值恒 Some）。
pub fn rule_width_list_first_width(segments: &[RuleWidthSegment]) -> Option<&ColumnRuleWidthValue> {
    segments.first().and_then(|s| match s {
        RuleWidthSegment::Plain(w) => Some(w),
        RuleWidthSegment::Repeat { widths, .. } => widths.first(),
    })
}

/// computed 串化（css-gaps-1 §column-row-rule-width computed 形）：宽度逐段
/// resolve（keyword→used px、长度→px、calc→折叠形），repeat 结构保形、auto
/// 保 `repeat(auto, …)`、calc 计数 eval 折叠整数（`repeat(calc(5 + 3),…)` →
/// `repeat(8, …)`，computed 页钉）。折叠失败段返 None（调用方回落原文）。
pub fn rule_width_list_to_css(
    segments: &[RuleWidthSegment],
    resolve: &dyn Fn(&ColumnRuleWidthValue) -> String,
) -> Option<String> {
    let mut parts = Vec::with_capacity(segments.len());
    for seg in segments {
        match seg {
            RuleWidthSegment::Plain(w) => parts.push(resolve(w)),
            RuleWidthSegment::Repeat { count, widths } => {
                let head = match count {
                    RuleRepeatCount::Auto => "auto".to_string(),
                    RuleRepeatCount::N(n) => n.to_string(),
                    RuleRepeatCount::Calc(expr) => {
                        // calc 整数折叠：parse_calc → 无 ctx eval（纯 number
                        // 树）→ 四舍五入 u32（`repeat(calc(5 + 3),…)` → 8）。
                        let v = super::types::parse_calc(expr)?;
                        format!("{}", super::types::eval_calc(&v, None)?.round() as i64)
                    }
                };
                let ws: Vec<String> = widths.iter().map(resolve).collect();
                parts.push(format!("repeat({}, {})", head, ws.join(", ")));
            }
        }
    }
    Some(parts.join(", "))
}

/// 段数（有限 repeat 展开后平列表长度，插值对齐用；auto 段计其循环体长）。
pub fn rule_width_list_expanded_len(segments: &[RuleWidthSegment]) -> usize {
    segments
        .iter()
        .map(|s| match s {
            RuleWidthSegment::Plain(_) => 1,
            RuleWidthSegment::Repeat { widths, .. } => widths.len(),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_width_list_valid_forms() {
        // valid 页 16 形代表（三属性共享文法）。
        for v in [
            "repeat(4, thin)",
            "repeat(3, thin, medium, thick)",
            "repeat(1, 10px, thin, medium, thick)",
            "10px",
            "repeat(4, medium)",
            "10px, 15px",
            "10px, 15px, 20px, 25px, 30px",
            "repeat(3, 10px), repeat(4, 15px)",
            "10px, repeat(3, 20px), 30px, repeat(4, thin, medium, thick)",
            "repeat(4, 10px, 20px, 30px), repeat(5, 40px), 50px",
            "repeat(auto, thin)",
            "repeat(auto, 10px, 20px, 30px)",
            "repeat(auto, 10px, 20px), 30px",
            "repeat(4, 10px, 20px, 30px), repeat(auto, 40px)",
            "10px, repeat(auto, 20px, 30px), repeat(4, thin, 50px, 60px)",
            "calc(10px + 0.5em)",
        ] {
            assert!(parse_rule_width_list(v).is_some(), "valid form rejected: {v}");
        }
    }

    #[test]
    fn test_rule_width_list_invalid_forms() {
        // invalid 页 9 形（三属性共享）。
        for v in [
            "auto",
            "thin medium thick",
            "repeat(auto, thin 10px thick)",
            "repeat(0, 10px, 20px, 30px)",
            "repeat(-1, 10px, thin, medium)",
            "repeat(auto, 10px), thin, repeat(auto, thick)",
            "10 20 30",
            "repeat(2, -20px)",
            "30%",
            "",
        ] {
            assert!(parse_rule_width_list(v).is_none(), "invalid form accepted: {v}");
        }
    }

    #[test]
    fn test_rule_width_list_pure_width_compat() {
        // multicol 纯宽 = 单段平宽（keyword 保形）。
        let segs = parse_rule_width_list("thin").unwrap();
        assert!(matches!(
            rule_width_list_single_width(&segs),
            Some(ColumnRuleWidthValue::Thin)
        ));
        // 列表形不退化。
        let segs = parse_rule_width_list("10px, 15px").unwrap();
        assert!(rule_width_list_single_width(&segs).is_none());
    }

    #[test]
    fn test_rule_width_list_to_css_computed() {
        // computed 页钉：calc 计数折叠整数、repeat/auto 结构保形。
        let resolve = |w: &ColumnRuleWidthValue| -> String {
            match w {
                ColumnRuleWidthValue::Thin => "1px".to_string(),
                ColumnRuleWidthValue::Medium => "3px".to_string(),
                ColumnRuleWidthValue::Thick => "5px".to_string(),
                ColumnRuleWidthValue::Length(_) => "30px".to_string(),
            }
        };
        let segs = parse_rule_width_list("repeat(calc(5 + 3), 10px)").unwrap();
        assert_eq!(rule_width_list_to_css(&segs, &resolve).unwrap(), "repeat(8, 30px)");
        let segs = parse_rule_width_list("repeat(auto, thin)").unwrap();
        assert_eq!(rule_width_list_to_css(&segs, &resolve).unwrap(), "repeat(auto, 1px)");
    }
}
