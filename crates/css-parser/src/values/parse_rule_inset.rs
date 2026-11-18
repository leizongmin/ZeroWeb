//! css-gaps-1 rule-inset 文法（§inset `<normal|<length-percentage>>{1,2}` 双侧）。
//!
//! css-gaps-1 把 gap decorations inset 重定义为槽模型：
//!
//! ```text
//! <rule-inset>          = <rule-inset-side> [ / <rule-inset-side> ]?
//! <rule-inset-side>     = <rule-inset-slot>{1,2}
//! <rule-inset-slot>     = overlap-join | <length-percentage>
//! ```
//!
//! 单侧 1 值复制双槽（start/end），`/` 后第二侧独立展开；缺省第二侧 = 复制
//! 第一侧。8 longhand（`{column,row}-rule-inset-{cap,junction}-{start,end}`，
//! 初值 0px 非继承）+ 15 简写。canonical 串化：四槽全等 → 单值；否则恒
//! `cap-start cap-end / junction-start junction-end`（语料 valid/computed 页钉，
//! 侧内等值不折叠——`10px / 20px` → `10px 10px / 20px 20px`）。
//! driving: css-gaps 语料 rule-inset-{valid,invalid,computed,shorthand,…} 20 页
//! + animation/ 插值四页（wptfyi-css gaps 刀 D）。
//!
//! https://drafts.csswg.org/css-gaps-1/#inset

use super::parse_basic_shape::len_to_css;
use super::parse_extended::calc_expr_has_anchor;
use super::types::{LengthValue, parse_length, parse_math_function};

/// inset 槽值（css-gaps-1 §inset `<rule-inset-slot>`）：`overlap-join` 关键字
/// 或 `<length-percentage>`。Length 保单位指定值（em/vmin/% /calc 原文），
/// computed 期解析 px（engine gCS 串化单源）。
#[derive(Debug, Clone, PartialEq)]
pub enum RuleInsetComponent {
    /// `overlap-join` —— 装饰线与交叉节点重叠（不避让）。
    OverlapJoin,
    /// 长度/百分比槽（负值合法——inset 允许外扩）。
    Length(LengthValue),
}

/// `{1,2}` 展开后的单侧规范化形（start, end）。
pub type RuleInsetSide = (RuleInsetComponent, RuleInsetComponent);

/// 单轴四槽规范化形（cap-start, cap-end, junction-start, junction-end）。
/// `rule-inset` 解析产物，简写展开按槽位投影落 longhand。
#[derive(Debug, Clone, PartialEq)]
pub struct RuleInsetQuad {
    /// cap 侧 start 槽（column = 上缘 / row = 左缘，规范语义与书写方向相关）。
    pub cap_start: RuleInsetComponent,
    /// cap 侧 end 槽。
    pub cap_end: RuleInsetComponent,
    /// junction 侧 start 槽。
    pub junction_start: RuleInsetComponent,
    /// junction 侧 end 槽。
    pub junction_end: RuleInsetComponent,
}

/// `<length-percentage>` 纯度门：parse_length 的关键字/内容形拒收（`auto`/
/// `thin`/`medium`/`thick`/`min-content`/`max-content` 关键字臂 + `fit-content(`
/// 函数臂不在 inset 文法——invalid 页 `auto` 钉）。
fn rule_inset_lp(value: &str) -> Option<LengthValue> {
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "auto" | "thin" | "medium" | "thick" | "min-content" | "max-content" | "stretch"
    ) || lower.starts_with("fit-content(")
    {
        return None;
    }
    parse_length(value)
}

/// 解析单槽（`overlap-join | <length-percentage>`）。math 形（calc/min/max/
/// clamp）先裁（含 anchor() 叶式样式解析面判无效——parse_length_or_math 同口径），
/// 其后单位长度门（负值合法）。
pub fn parse_rule_inset_component(value: &str) -> Option<RuleInsetComponent> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("overlap-join") {
        return Some(RuleInsetComponent::OverlapJoin);
    }
    if let Some(expr) = parse_math_function(value) {
        if calc_expr_has_anchor(&expr) {
            return None;
        }
        return Some(RuleInsetComponent::Length(LengthValue::Calc(Box::new(expr))));
    }
    rule_inset_lp(value).map(RuleInsetComponent::Length)
}

/// 解析单侧 `<rule-inset-slot>{1,2}`：1 值复制双槽。分词括号感知（calc 内
/// 空白不切——`calc(8em + 4ex)` 单槽三词）。
pub fn parse_rule_inset_side(side: &str) -> Option<RuleInsetSide> {
    let toks = split_top_level_whitespace(side)?;
    match toks.as_slice() {
        [a] => {
            let c = parse_rule_inset_component(a)?;
            Some((c.clone(), c))
        }
        [a, b] => Some((parse_rule_inset_component(a)?, parse_rule_inset_component(b)?)),
        _ => None,
    }
}

/// 括号感知 top-level 空白切分（深度 >0 段内空白保留为 token 内部字符）。
fn split_top_level_whitespace(s: &str) -> Option<Vec<&str>> {
    let mut toks = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                start.get_or_insert(i);
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                start.get_or_insert(i);
            }
            c if c.is_whitespace() && depth == 0 => {
                if let Some(st) = start.take() {
                    toks.push(&s[st..i]);
                }
            }
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if depth != 0 {
        return None;
    }
    if let Some(st) = start {
        toks.push(&s[st..]);
    }
    Some(toks)
}

/// 括号感知 top-level 分隔切分（calc 内 `/` 不切——`calc(8px / 2)` 单槽）。
/// split_top_level_commas 同款深度门（深度负数/失衡拒）。
fn split_top_level_slashes(s: &str) -> Option<Vec<&str>> {
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
            '/' if depth == 0 => {
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

/// 解析 `rule-inset` 全值（`<rule-inset-side> [/ <rule-inset-side>]?`）：
/// 单侧复制 junction = cap；双侧独立展开。>2 侧 / 空侧 / 每侧 >2 槽拒。
pub fn parse_rule_inset_quad(value: &str) -> Option<RuleInsetQuad> {
    let sides = split_top_level_slashes(value)?;
    if sides.is_empty() || sides.len() > 2 {
        return None;
    }
    let cap = parse_rule_inset_side(sides[0])?;
    let junction = if sides.len() == 2 {
        parse_rule_inset_side(sides[1])?
    } else {
        cap.clone()
    };
    Some(RuleInsetQuad {
        cap_start: cap.0,
        cap_end: cap.1,
        junction_start: junction.0,
        junction_end: junction.1,
    })
}

/// 单槽指定值 canonical 串化（简写展开落 longhand 文本用）：单位保形
///（`10vmin`/`5%`/`calc(8em + 4ex)` 原文形），裸零折 `0px`（valid 页钉）。
/// 关键字/内容形（parse 门已拒）防御性 None。
pub fn rule_inset_component_to_css(c: &RuleInsetComponent) -> Option<String> {
    match c {
        RuleInsetComponent::OverlapJoin => Some("overlap-join".to_string()),
        RuleInsetComponent::Length(
            LengthValue::Auto
            | LengthValue::MinContent
            | LengthValue::MaxContent
            | LengthValue::Stretch
            | LengthValue::FitContent(_),
        ) => None,
        RuleInsetComponent::Length(lv) => Some(len_to_css(lv)),
    }
}

/// 单侧指定值重组（start==end → 单值；否则 `start end`）。
pub fn rule_inset_side_to_css(side: &RuleInsetSide) -> Option<String> {
    let s = rule_inset_component_to_css(&side.0)?;
    let e = rule_inset_component_to_css(&side.1)?;
    Some(if s == e { s } else { format!("{s} {e}") })
}

/// 四槽指定值重组（css-gaps-1 canonical 形，valid 页钉）：四槽全等 → 单值；
/// 否则恒 `cap-start cap-end / junction-start junction-end`（侧内等值不折叠
/// ——`10px / 20px` → `10px 10px / 20px 20px`，全 keyword 塌缩
/// `overlap-join ×4` → `overlap-join` 同门覆盖）。
pub fn rule_inset_quad_to_css(q: &RuleInsetQuad) -> Option<String> {
    let cs = rule_inset_component_to_css(&q.cap_start)?;
    let ce = rule_inset_component_to_css(&q.cap_end)?;
    let js = rule_inset_component_to_css(&q.junction_start)?;
    let je = rule_inset_component_to_css(&q.junction_end)?;
    if cs == ce && ce == js && js == je {
        return Some(cs);
    }
    Some(format!("{cs} {ce} / {js} {je}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_inset_valid_forms() {
        // valid 页 12 形（rule-inset 槽）。
        for v in [
            "0",
            "-20px",
            "5%",
            "10vmin",
            "calc(8em + 4ex)",
            "overlap-join",
            "10px 20px",
            "10px / 20px",
            "overlap-join overlap-join / overlap-join overlap-join",
            "overlap-join / 10px",
            "10px 20px / 30px -10px",
            "10px 20px / -5px",
            "5% 10% / 15% 25%",
        ] {
            assert!(parse_rule_inset_quad(v).is_some(), "valid form rejected: {v}");
        }
    }

    #[test]
    fn test_rule_inset_invalid_forms() {
        // invalid 页 10 形（三简写共享文法）。
        for v in [
            "auto",
            "none",
            "10",
            "10px 20px 10px 20px 20px",
            "10px / 20px / 10px",
            "/ 10px 20px",
            "10px 10px 20px",
            "10% / 10px 10px 20px",
            "overlap-join garbage",
            "overlap-join 10px 20px",
            "",
        ] {
            assert!(parse_rule_inset_quad(v).is_none(), "invalid form accepted: {v}");
        }
    }

    #[test]
    fn test_rule_inset_side_expansion() {
        // {1,2} 展开：1 值复制双槽；/ 缺省第二侧复制第一侧。
        let q = parse_rule_inset_quad("10px").unwrap();
        assert_eq!(
            q,
            RuleInsetQuad {
                cap_start: RuleInsetComponent::Length(LengthValue::Px(10.0)),
                cap_end: RuleInsetComponent::Length(LengthValue::Px(10.0)),
                junction_start: RuleInsetComponent::Length(LengthValue::Px(10.0)),
                junction_end: RuleInsetComponent::Length(LengthValue::Px(10.0)),
            }
        );
        let q = parse_rule_inset_quad("10px 20px").unwrap();
        assert_eq!(q.cap_start, q.junction_start);
        assert_eq!(q.cap_end, RuleInsetComponent::Length(LengthValue::Px(20.0)));
    }

    #[test]
    fn test_rule_inset_quad_to_css() {
        // canonical 串化（valid/computed 页钉）：全等塌缩、四槽全形、
        // keyword 全塌缩、calc 保形、裸零折 0px。
        let css = |v: &str| rule_inset_quad_to_css(&parse_rule_inset_quad(v).unwrap()).unwrap();
        assert_eq!(css("0"), "0px");
        assert_eq!(css("-20px"), "-20px");
        assert_eq!(css("10vmin"), "10vmin");
        assert_eq!(css("calc(8em + 4ex)"), "calc(8em + 4ex)");
        assert_eq!(css("overlap-join"), "overlap-join");
        assert_eq!(css("10px 20px"), "10px 20px / 10px 20px");
        assert_eq!(css("10px / 20px"), "10px 10px / 20px 20px");
        assert_eq!(
            css("overlap-join overlap-join / overlap-join overlap-join"),
            "overlap-join"
        );
        assert_eq!(css("overlap-join / 10px"), "overlap-join overlap-join / 10px 10px");
        assert_eq!(css("10px 20px / 30px -10px"), "10px 20px / 30px -10px");
        assert_eq!(css("10px 20px / -5px"), "10px 20px / -5px -5px");
    }

    #[test]
    fn test_rule_inset_calc_not_slash_split() {
        // calc 内 `/` 不切侧（`calc(8px / 2)` = 单槽 calc）。
        let q = parse_rule_inset_quad("calc(8px / 2)").unwrap();
        assert!(matches!(q.cap_start, RuleInsetComponent::Length(LengthValue::Calc(_))));
        assert_eq!(q.junction_start, q.cap_start);
    }
}
