//! css-gaps-1 gap-rule 联合简写文法（<gap-rule-list>，rule/row-rule/
//! column-rule 简写共享）。
//!
//! ```text
//! <gap-rule>            = [ <line-width> || <line-style> || <line-color> ]
//! <gap-repeat-rule>     = repeat( [ <integer [1,∞]> ] , <gap-rule># )
//! <gap-auto-repeat-rule> = repeat( auto , <gap-rule># )
//! <gap-rule-or-repeat>  = <gap-rule> | <gap-repeat-rule>
//! <gap-rule-list>       = <gap-rule-or-repeat>#
//! <gap-auto-rule-list>  = <gap-rule-or-repeat>#? <gap-auto-repeat-rule> <gap-rule-or-repeat>#?
//! ```
//!
//! 段内三维度顺序无关（`||`），缺省分量不隐含——简写展开期按维度投影补
//! 初值（width→medium、style→none、color→currentcolor）。投影输出
//! specified 原文（token 原样，命名色 `hotpink` 不归一 rgb()——shorthand
//! 页钉）。
//! driving: css-gaps 语料 gap-decorations-rule-shorthand{,-computed,-valid,
//! -invalid}（wptfyi-css gaps 刀 B）。
//! https://drafts.csswg.org/css-gaps-1/#rule-shorthand-properties

use super::color::parse_color;
use super::parse_layout::{ColumnRuleStyleValue, ColumnRuleWidthValue};
use super::parse_rule_color::{RuleRepeatCount, parse_rule_repeat_count, split_top_level_commas};
use super::parse_rule_style::parse_line_style;
use super::parse_rule_width::parse_line_width_item;
use super::types::ColorValue;

/// 单个 gap-rule 段：三维度至多各一（`||` 语义，重复同维拒——invalid 页
/// `red 5px solid red` 段内双 color 钉）。`*_raw` 为该维度 token 的
/// specified 原文（投影输出保原文，parse 归一值仅作合法性裁决）。
#[derive(Debug, Clone, PartialEq)]
pub struct GapRuleComponent {
    /// 宽度分量（裁决值）。
    pub width: Option<ColumnRuleWidthValue>,
    /// 样式分量（裁决值）。
    pub style: Option<ColumnRuleStyleValue>,
    /// 颜色分量（裁决值）。
    pub color: Option<ColorValue>,
    /// 宽度 token 原文。
    pub width_raw: Option<String>,
    /// 样式 token 原文。
    pub style_raw: Option<String>,
    /// 颜色 token 原文。
    pub color_raw: Option<String>,
}

/// 联合列表一项：平段或 repeat 块。
#[derive(Debug, Clone, PartialEq)]
pub enum GapRulePart {
    /// 平段。
    Flat(GapRuleComponent),
    /// repeat 块：计数 + 段循环体（≥1 段）。
    Repeat {
        /// 计数。
        count: RuleRepeatCount,
        /// 循环体段（≥1）。
        components: Vec<GapRuleComponent>,
    },
}

/// 解析单个 gap-rule 段（空格分隔 token 流，三维度顺序无关、至多各一）。
fn parse_gap_rule_component(text: &str) -> Option<GapRuleComponent> {
    let mut comp = GapRuleComponent {
        width: None,
        style: None,
        color: None,
        width_raw: None,
        style_raw: None,
        color_raw: None,
    };
    let mut any = false;
    // 空格切分安全：三维度 token 均不含顶层空白（<line-width> 括号感知门
    // 同款；color 的 rgb(...) 函数形含空格——paren 感知逐 token 扫描）。
    for tok in super::parse_transform::split_paren_aware_tokens(text) {
        if let Some(w) = parse_line_width_item(&tok) {
            if comp.width.is_some() {
                return None;
            }
            comp.width = Some(w);
            comp.width_raw = Some(tok.clone());
            any = true;
        } else if let Some(s) = parse_line_style(&tok) {
            if comp.style.is_some() {
                return None;
            }
            comp.style = Some(s);
            comp.style_raw = Some(tok.to_ascii_lowercase());
            any = true;
        } else {
            let c = parse_color(&tok)?;
            if comp.color.is_some() {
                return None;
            }
            comp.color = Some(c);
            comp.color_raw = Some(tok.clone());
            any = true;
        }
    }
    if !any {
        return None;
    }
    Some(comp)
}

/// 解析 gap-rule 联合列表值：≥1 段、auto repeat 至多一处。
pub fn parse_gap_rule_list(value: &str) -> Option<Vec<GapRulePart>> {
    let mut parts = Vec::new();
    let mut autos = 0usize;
    for seg in split_top_level_commas(value)? {
        let seg_trim = seg.trim();
        let lower = seg_trim.to_ascii_lowercase();
        if lower.starts_with("repeat(") && lower.ends_with(')') {
            let inner = &seg_trim[7..seg_trim.len() - 1];
            let (head, body_text) = inner.split_once(',')?;
            let count = parse_rule_repeat_count(head)?;
            let mut components = Vec::new();
            for comp_text in split_top_level_commas(body_text)? {
                components.push(parse_gap_rule_component(comp_text)?);
            }
            if components.is_empty() {
                return None;
            }
            if matches!(count, RuleRepeatCount::Auto) {
                autos += 1;
                if autos > 1 {
                    return None;
                }
            }
            parts.push(GapRulePart::Repeat { count, components });
        } else {
            parts.push(GapRulePart::Flat(parse_gap_rule_component(seg_trim)?));
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts)
}

/// 维度投影：联合列表 → width 列表串（缺省分量补 `medium` 初值，
/// css-gaps-1 §rule-shorthand-properties 展开语义——简写展开期补初值使
/// 未指定 longhand 获得确定性值；`shorthand.html` 页 `dashed` →
/// width 'medium' 钉）。repeat 结构保形（计数/循环体长度逐维对齐）。
pub fn project_widths(parts: &[GapRulePart]) -> Option<String> {
    project_dimension(parts, &|c| c.width_raw.clone().unwrap_or_else(|| "medium".to_string()))
}

/// 维度投影：style 列表串（缺省补 `none`；style keyword canonical 小写）。
pub fn project_styles(parts: &[GapRulePart]) -> Option<String> {
    project_dimension(parts, &|c| c.style_raw.clone().unwrap_or_else(|| "none".to_string()))
}

/// 维度投影：color 列表串（缺省补 `currentcolor`）。
pub fn project_colors(parts: &[GapRulePart]) -> Option<String> {
    project_dimension(parts, &|c| {
        c.color_raw.clone().unwrap_or_else(|| "currentcolor".to_string())
    })
}

/// 通用投影：按段结构逐维展开（repeat 保形）。投影串不走计数折叠（简写
/// 展开期为 specified 形：calc 计数原文保留，computed 期由 longhand 串化
/// 折叠）。
fn project_dimension(parts: &[GapRulePart], f: &dyn Fn(&GapRuleComponent) -> String) -> Option<String> {
    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            GapRulePart::Flat(c) => out.push(f(c)),
            GapRulePart::Repeat { count, components } => {
                let head = match count {
                    RuleRepeatCount::Auto => "auto".to_string(),
                    RuleRepeatCount::N(n) => n.to_string(),
                    RuleRepeatCount::Calc(expr) => expr.clone(),
                };
                let items: Vec<String> = components.iter().map(f).collect();
                out.push(format!("repeat({}, {})", head, items.join(", ")));
            }
        }
    }
    Some(out.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_rule_list_valid() {
        // shorthand.html 页全形代表。
        for v in [
            "5px solid red",
            "double",
            "blue 10px",
            "repeat(auto, 5px solid green)",
            "repeat(auto, 5px solid yellow, 10px dotted blue)",
            "repeat(auto, blue 6px, 5px solid red)",
            "repeat(4, 15px dotted pink)",
            "repeat(1, 15px ridge yellow, 10px dotted blue, 15px double green)",
            "repeat(3, lime 16px, dashed purple, 10px dotted)",
            "thin, dashed, hotpink",
            "5px double salmon, repeat(4, 5px ridge red)",
            "repeat(2, dashed gray, 10px blue dotted, 20px double), 5px solid red, repeat(4, blue 6px, 5px solid white)",
            "repeat(4, thick hidden skyblue), repeat(3, 5px solid red, 10px dotted)",
            "repeat(auto, 10px solid red), medium dotted green, repeat(3, thick dashed blue, 15px double green)",
            "ridge red, repeat(auto, 5px solid green), 10px dotted blue",
            "10px dotted salmon, repeat(4, thin blue, hidden 5px purple), repeat(auto, 5px solid red, teal)",
            "5px solid currentcolor",
            "rgb(0, 0, 255) 10px solid",
        ] {
            assert!(parse_gap_rule_list(v).is_some(), "valid rejected: {v}");
        }
    }

    #[test]
    fn test_gap_rule_list_invalid() {
        // shorthand-invalid 页 6 形。
        for v in [
            "auto",
            "red 5px solid red",
            "repeat(auto, red 5px green ridge)",
            "repeat(auto, 5px solid red), 5px solid red, repeat(auto, 5px solid red)",
            "repeat(0, 5px red)",
            "repeat(-1, thin green red)",
            "",
        ] {
            assert!(parse_gap_rule_list(v).is_none(), "invalid accepted: {v}");
        }
    }

    #[test]
    fn test_projection_defaults() {
        let parts = parse_gap_rule_list("thin, dashed, hotpink").unwrap();
        assert_eq!(project_widths(&parts).unwrap(), "thin, medium, medium");
        assert_eq!(project_styles(&parts).unwrap(), "none, dashed, none");
        assert_eq!(project_colors(&parts).unwrap(), "currentcolor, currentcolor, hotpink");
    }

    #[test]
    fn test_projection_repeat_structure() {
        let parts = parse_gap_rule_list("repeat(auto, 5px solid green)").unwrap();
        assert_eq!(project_widths(&parts).unwrap(), "repeat(auto, 5px)");
        assert_eq!(project_styles(&parts).unwrap(), "repeat(auto, solid)");
        assert_eq!(project_colors(&parts).unwrap(), "repeat(auto, green)");
        let parts = parse_gap_rule_list("repeat(3, lime 16px, dashed purple, 10px dotted)").unwrap();
        assert_eq!(project_widths(&parts).unwrap(), "repeat(3, 16px, medium, 10px)");
        assert_eq!(project_styles(&parts).unwrap(), "repeat(3, none, dashed, dotted)");
        assert_eq!(project_colors(&parts).unwrap(), "repeat(3, lime, purple, currentcolor)");
    }

    #[test]
    fn test_projection_function_color_preserved() {
        // 函数色原文保留（`rgb(0, 0, 255)` 归一裁决、原文输出）。
        let parts = parse_gap_rule_list("rgb(0, 0, 255) 10px solid").unwrap();
        assert_eq!(project_colors(&parts).unwrap(), "rgb(0, 0, 255)");
        assert_eq!(project_widths(&parts).unwrap(), "10px");
        assert_eq!(project_styles(&parts).unwrap(), "solid");
    }
}
