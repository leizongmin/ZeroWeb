//! CSS Syntax §3.3 输入预处理：所有 U+0000 (NULL) 须替换为 U+FFFD REPLACEMENT CHARACTER。
//!
//! 背景：转义 NULL（`\0`）已在 consume_escape 正确替换为 FFFD，但**原始** NULL（未转义）
//! 修复前落默认 `_ => Token::Error` 分支。顶层 Error token 会被 consume_rule 当选择器解析
//! 失败 → skip_malformed_qualified_rule 可能吞掉相邻规则（与 pre-R2204 的 CDO bug 同源）。
//! §3.3 预处理规定所有 NULL→FFFD；FFFD（`!is_ascii()`）是合法 ident 字符，会并入相邻
//! 标识符（与 chromium 一致）。这是 NULL-byte robustness + spec 合规。

use super::*;

/// 原始 NULL 被替换为 FFFD，并入相邻标识符（非 Error token）。
#[test]
fn test_raw_null_replaced_with_replacement_char() {
    let tokens = Tokenizer::new("di\0v").collect_tokens();
    // 期望单个 Ident("di\u{FFFD}v")，NULL 不应产生独立 Token::Error。
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Ident(s) if s == "di\u{FFFD}v")),
        "原始 NULL 应被 FFFD 替换并并入相邻 ident，实际 tokens: {:?}",
        tokens
    );
    assert!(
        !tokens.iter().any(|t| matches!(t, Token::Error(_))),
        "NULL 不应产生 Token::Error，实际含 Error: {:?}",
        tokens
    );
}

/// 多个 NULL 全部替换（不止首字符）。
#[test]
fn test_multiple_nulls_all_replaced() {
    let tokens = Tokenizer::new("a\0b\0c").collect_tokens();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Ident(s) if s == "a\u{FFFD}b\u{FFFD}c")),
        "所有 NULL 应替换为 FFFD，实际: {:?}",
        tokens
    );
}

/// NULL 不应吞掉相邻规则（顶层 Error 不会触发 skip_malformed_qualified_rule 吞块）。
#[test]
fn test_null_does_not_swallow_adjacent_rule() {
    // NULL 紧贴在选择器前：FFFD 并入选择器使该规则选择器失配（spec 行为），但规则本身
    // 不应丢失，且后续规则必须保留。
    let css = "p { color: red; }\n/* */\ndiv { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2, "无 NULL 基线：2 条规则");
    // 含 NULL：NULL 在两规则间不应使规则总数少于 2（FFFD 并入相邻 token，不吞块）。
    let css_null = "p { color: red; }\n\0\ndiv { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css_null);
    assert!(
        stylesheet.rules.len() >= 2,
        "NULL 在规则间不应吞掉规则，实际规则数: {}",
        stylesheet.rules.len()
    );
}

/// 无 NULL 输入行为不变（回归守护）。
#[test]
fn test_no_null_input_unchanged() {
    let tokens = Tokenizer::new("div { color: red }").collect_tokens();
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "div")));
    assert!(!tokens.iter().any(|t| matches!(t, Token::Error(_))));
}
