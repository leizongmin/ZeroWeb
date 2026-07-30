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
    // CSS Conditional §7：同一括号层内 `and` 与 `or` 不可混用——条件须为
    // `in-parens (and in-parens)*` 或 `in-parens (or in-parens)*`。混用 → 非法 → None
    //（整条 @supports 块不应用）。本函数在每个层级（含 parse_primary 对括号内的递归调用）
    // 入口校验，故嵌套混用亦被拒。driving: WPT css-supports-013 `(A) and (B) or (C)`。
    let (has_and, has_or) = top_level_ops(input);
    if has_and && has_or {
        return None;
    }
    // CSS Conditional §7：`not` 是条件前导形式（`not <in-parens>`），不可与顶层 and/or
    // 共现——`not X and/or Y` / `X and/or not Y` 非法（and/or 操作数须为 in-parens）。
    // driving: WPT css-supports-019/029/030。
    if top_level_not(input) && (has_and || has_or) {
        return None;
    }

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

/// 检测字符串**顶层**（括号外）是否含 ` and ` / ` or ` 运算符。
fn top_level_ops(input: &str) -> (bool, bool) {
    let lower = input.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut has_and = false;
    let mut has_or = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                if i + 5 <= bytes.len() && &lower[i..i + 5] == " and " {
                    has_and = true;
                    i += 5;
                    continue;
                }
                if i + 4 <= bytes.len() && &lower[i..i + 4] == " or " {
                    has_or = true;
                    i += 4;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (has_and, has_or)
}

/// 检测字符串**顶层**（括号外）是否含 `not` 关键字（词边界：前为串首/空格，后为空格）。
fn top_level_not(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            _ => {
                if depth == 0 {
                    let prev_boundary = i == 0 || bytes[i - 1] == b' ';
                    if prev_boundary && lower[i..].starts_with("not") {
                        let after = i + 3;
                        if after == bytes.len() || bytes[after] == b' ' {
                            return true;
                        }
                    }
                }
                i += 1;
            }
        }
    }
    false
}

/// 若 `input` 恰为**单个匹配括号对**包裹（首个 `(` 的匹配 `)` 在末尾），返回其内部；
/// 否则 None。用于 `parse_primary` 递归剥层（`((X))` → `X`）。
fn strip_one_paren_pair(input: &str) -> Option<&str> {
    let input = input.trim();
    if !input.starts_with('(') {
        return None;
    }
    let bytes = input.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return if i == bytes.len() - 1 { Some(&input[1..i]) } else { None };
                }
            }
            _ => {}
        }
    }
    None
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
///
/// `<supports-feature>` = `( <declaration> )` —— 属性值测试**必须在括号内**；
/// 裸 `property: value`（无括号）非法 → None（driving: WPT css-supports-002
/// `@supports color: green` 须不应用）。
fn parse_primary(input: &str) -> Option<SupportsCondition> {
    let input = input.trim();

    // selector() 函数
    if let Some(rest) = input.strip_prefix("selector(")
        && let Some(inner) = rest.strip_suffix(')')
    {
        return Some(SupportsCondition::Selector(inner.trim().to_string()));
    }

    // 括号包裹：单个匹配括号对
    if let Some(inner) = strip_one_paren_pair(input) {
        let inner = inner.trim();
        // 内含 and/or/not 或自身仍为括号组（嵌套 `((X))`）→ 递归为条件（复用校验逻辑）
        let (has_and, has_or) = top_level_ops(inner);
        if has_and || has_or || top_level_not(inner) || strip_one_paren_pair(inner).is_some() {
            return parse_or_expression(inner);
        }
        // 底层 feature：property: value
        if let Some(colon_pos) = inner.find(':') {
            let property = inner[..colon_pos].trim().to_string();
            let value = inner[colon_pos + 1..].trim().to_string();
            return Some(SupportsCondition::Property(property, value));
        }
        // 非合法 condition/feature 的括号内容 → general-enclosed（恒求值 false）。
        // driving: WPT css-supports-032/033/034/040 `(@page)` / `()`。
        return Some(SupportsCondition::GeneralEnclosed(inner.to_string()));
    }

    // 裸形式（无括号）→ feature 须在括号内，非法
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

    #[test]
    fn test_parse_mixed_and_or_invalid() {
        // CSS Conditional §7：顶层 and/or 不可混用（须全 and 或全 or，否则非法 → None，
        // 整条 @supports 块不应用）。driving: WPT css-supports-013
        // `(A) and (B) or (C)`。
        assert_eq!(
            parse_supports_condition("(color: green) and (color: green) or (color: green)"),
            None
        );
        assert_eq!(
            parse_supports_condition("(color: green) or (color: green) and (color: green)"),
            None
        );
    }

    #[test]
    fn test_parse_nested_mixed_in_parens_invalid() {
        // 括号内顶层 and/or 混用同样非法（递归层级独立校验）。
        assert_eq!(
            parse_supports_condition("((color: green) and (color: green) or (color: green))"),
            None
        );
    }

    #[test]
    fn test_parse_parenthesized_mix_is_valid() {
        // `(A and B) or C` 合法：顶层仅 or；and 在括号内（非顶层）。
        let cond = parse_supports_condition("(color: red) and (color: blue) or (color: green)");
        // 注意：上一行的 `(A) and (B) or (C)` 是非法（顶层混用）；这里改用合法形式
        let cond = parse_supports_condition("((color: red) and (color: blue)) or (color: green)");
        assert!(cond.is_some(), "括号包裹的 and 链与 or 混用合法");
    }

    #[test]
    fn test_parse_not_mixed_with_and_or_invalid() {
        // CSS Conditional §7：`not` 是条件**前导**形式（`not <in-parens>`），不可与顶层
        // and/or 共现——`not X and/or Y` 非法（and/or 操作数须为 in-parens，非 `not X`）。
        // driving: WPT css-supports-019/029/030。
        assert_eq!(
            parse_supports_condition("not (color: rainbow) and not (color: iridescent)"),
            None
        );
        assert_eq!(parse_supports_condition("not (color: rainbow) or (color: green)"), None);
        assert_eq!(
            parse_supports_condition("(color: green) and not (color: rainbow)"),
            None
        );
        // 括号包裹的 `not X or Y` 内层非法 → 外层整体非法（030）。
        assert_eq!(
            parse_supports_condition("(not (color: rainbow) or (color: green))"),
            None
        );
    }

    #[test]
    fn test_parse_not_alone_valid() {
        // 回归：`not (X)` 单独合法。
        assert!(parse_supports_condition("not (color: green)").is_some());
    }

    #[test]
    fn test_parse_bare_property_invalid() {
        // feature 须在括号内：裸 `property: value`（无括号）非法 → None。
        // driving: WPT css-supports-002 `@supports color: green`。
        assert_eq!(parse_supports_condition("color: green"), None);
    }

    #[test]
    fn test_parse_double_not_invalid() {
        // `not` 取单个 in-parens；`not not (X)` 中外层 not 的操作数 `not (X)` 非 in-parens
        // → 非法 → None。driving: WPT css-supports-017。
        assert_eq!(parse_supports_condition("not not (color: green)"), None);
    }

    #[test]
    fn test_parse_general_enclosed_is_false_not_inverts() {
        // CSS Conditional §7：`(<any-value>)` 非合法 condition/feature 时为 general-enclosed，
        // 恒求值为 false；故 `not (@page)` / `not ()` 求值为 true（块应用）。
        // driving: WPT css-supports-032/033/034/040。parse 须返回 Some（非 None）。
        assert!(
            parse_supports_condition("(@page)").is_some(),
            "`(@page)` 应解析为 general-enclosed"
        );
        assert!(
            parse_supports_condition("()").is_some(),
            "`()` 应解析为 general-enclosed"
        );
        assert!(
            parse_supports_condition("not (@page)").is_some(),
            "`not (@page)` 应解析（求值 true）"
        );
        assert!(
            parse_supports_condition("not ()").is_some(),
            "`not ()` 应解析（求值 true）"
        );
    }

    #[test]
    fn test_parse_general_enclosed_regression_property_still_works() {
        // 回归：合法 feature 仍解析为 Property，不误判为 general-enclosed。
        let cond = parse_supports_condition("(color: green)");
        assert!(matches!(cond, Some(SupportsCondition::Property(_, _))));
    }

    #[test]
    fn test_parse_nested_extra_parens_valid() {
        // 任意子表达式可被额外一层括号包裹：`((X))` 合法（CSS Conditional §7），
        // 且应解析为正确的 Property（不应把 "(" 并入属性名）。
        // driving: WPT css-supports-003。
        let cond = parse_supports_condition("((color: green))");
        match cond {
            Some(SupportsCondition::Property(p, v)) => {
                assert_eq!(p, "color", "属性名不应含括号");
                assert_eq!(v, "green");
            }
            other => panic!("`((color: green))` 应解析为 Property(color, green)，实际: {:?}", other),
        }
        assert!(parse_supports_condition("(((display: grid)))").is_some());
    }
}
