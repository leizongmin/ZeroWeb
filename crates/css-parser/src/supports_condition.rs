//! CSS @supports 条件解析。
//!
//! 解析 `@supports` 规则中的条件表达式。

use crate::ast::SupportsCondition;

/// 解析 @supports 条件文本。
///
/// 支持的格式：
/// - `(property: value)` — 属性值测试
/// - `selector(selector)` — 选择器测试
/// - `not <cond>` — 逻辑非
/// - `<cond1> and <cond2>` — 逻辑与
/// - `<cond1> or <cond2>` — 逻辑或
pub fn parse_supports_condition(input: &str) -> Option<SupportsCondition> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    parse_or_expression(input)
}

/// 解析 or 表达式（最低优先级）。
fn parse_or_expression(input: &str) -> Option<SupportsCondition> {
    let parts = split_top_level(input, " or ");
    if parts.len() > 1 {
        let conditions: Vec<SupportsCondition> = parts
            .into_iter()
            .filter_map(|p| parse_and_expression(p.trim()))
            .collect();
        if conditions.is_empty() {
            return None;
        }
        if conditions.len() == 1 {
            return conditions.into_iter().next();
        }
        return Some(SupportsCondition::Or(conditions));
    }
    parse_and_expression(input)
}

/// 解析 and 表达式。
fn parse_and_expression(input: &str) -> Option<SupportsCondition> {
    let parts = split_top_level(input, " and ");
    if parts.len() > 1 {
        let conditions: Vec<SupportsCondition> = parts
            .into_iter()
            .filter_map(|p| parse_not_expression(p.trim()))
            .collect();
        if conditions.is_empty() {
            return None;
        }
        if conditions.len() == 1 {
            return conditions.into_iter().next();
        }
        return Some(SupportsCondition::And(conditions));
    }
    parse_not_expression(input)
}

/// 解析 not 表达式。
fn parse_not_expression(input: &str) -> Option<SupportsCondition> {
    let input = input.trim();
    if let Some(rest) = input.strip_prefix("not ") {
        let cond = parse_primary(rest.trim())?;
        Some(SupportsCondition::Not(Box::new(cond)))
    } else {
        parse_primary(input)
    }
}

/// 解析基本条件（括号表达式或 selector()）。
fn parse_primary(input: &str) -> Option<SupportsCondition> {
    let input = input.trim();

    // selector() 函数
    if let Some(rest) = input.strip_prefix("selector(")
        && let Some(inner) = rest.strip_suffix(')')
    {
        return Some(SupportsCondition::Selector(inner.trim().to_string()));
    }

    // 括号表达式
    if input.starts_with('(') && input.ends_with(')') {
        let inner = &input[1..input.len() - 1];
        let inner = inner.trim();

        // 检查内部是否包含嵌套的 and/or/not
        if contains_top_level_keyword(inner) {
            return parse_or_expression(inner);
        }

        // 属性值测试：(property: value)
        if let Some(colon_pos) = inner.find(':') {
            let property = inner[..colon_pos].trim().to_string();
            let value = inner[colon_pos + 1..].trim().to_string();
            return Some(SupportsCondition::Property(property, value));
        }
    }

    None
}

/// 在顶层（不在括号内）按关键字分割字符串。
fn split_top_level<'a>(input: &'a str, keyword: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let keyword_len = keyword.len();
    let lower = input.to_ascii_lowercase();

    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            depth += 1;
        } else if bytes[i] == b')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && i + keyword_len <= bytes.len() && &lower[i..i + keyword_len] == keyword {
            parts.push(&input[start..i]);
            start = i + keyword_len;
            i += keyword_len;
            continue;
        }
        i += 1;
    }
    parts.push(&input[start..]);
    parts
}

/// 检查字符串是否在顶层包含 and/or/not 关键字。
fn contains_top_level_keyword(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let mut depth = 0i32;
    for b in lower.as_bytes() {
        if *b == b'(' {
            depth += 1;
        } else if *b == b')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 {
            if lower.contains(" and ") || lower.contains(" or ") {
                return true;
            }
            if lower.starts_with("not ") {
                return true;
            }
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_property_condition() {
        let cond = parse_supports_condition("(display: grid)").unwrap();
        assert_eq!(
            cond,
            SupportsCondition::Property("display".to_string(), "grid".to_string())
        );
    }

    #[test]
    fn test_parse_selector_condition() {
        let cond = parse_supports_condition("selector(.a > .b)").unwrap();
        assert_eq!(cond, SupportsCondition::Selector(".a > .b".to_string()));
    }

    #[test]
    fn test_parse_not_condition() {
        let cond = parse_supports_condition("not (display: grid)").unwrap();
        match cond {
            SupportsCondition::Not(inner) => {
                assert_eq!(
                    *inner,
                    SupportsCondition::Property("display".to_string(), "grid".to_string())
                );
            }
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_parse_and_condition() {
        let cond = parse_supports_condition("(display: grid) and (gap: 10px)").unwrap();
        match cond {
            SupportsCondition::And(conditions) => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_or_condition() {
        let cond = parse_supports_condition("(display: grid) or (display: flex)").unwrap();
        match cond {
            SupportsCondition::Or(conditions) => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected Or"),
        }
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_supports_condition(""), None);
    }
}
