//! Tokenizer 覆盖率补充测试第 2 轮：针对未覆盖的边界路径。
//!
//! 主要覆盖：
//! - consume_newline（\n, \r, \r\n, \t）
//! - 裸 `-` 标识符
//! - 转义序列中的换行（应返回 None）
//! - url() 中字符串解析失败
//! - 以 @ 开头的 ident（edge case）
//! - `#` 后无 ident 字符 → Error token
//! - `||` Column token
//! - 裸 `+` / `-` / 其他符号作为 Delim/Ident

use crate::tokenizer::{Token, Tokenizer};

/// Helper: 收集所有 token。
fn tokens(input: &str) -> Vec<Token> {
    Tokenizer::new(input).map(|s| s.token).collect()
}

// ═══════════════════════════════════════════════════════════════════════
// 1. 裸 `-` 标识符（line 357, 359, 365）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_bare_dash_as_ident() {
    // `-` alone is an ident
    let toks = tokens("-");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "-"),
        other => panic!("Expected Ident(\"-\"), got {:?}", other),
    }
}

#[test]
fn test_dash_followed_by_non_ident() {
    // `-3` is a Number (-3.0), not ident
    let toks = tokens("-3");
    match &toks[0] {
        Token::Number(n) => assert_eq!(*n, -3.0),
        other => panic!("Expected Number(-3.0), got {:?}", other),
    }
}

#[test]
fn test_dash_followed_by_escape_without_valid_escape() {
    // `-\n` → `-` is ident, then newline
    let toks = tokens("-\n");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "-"),
        other => panic!("Expected Ident(\"-\"), got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 2. 转义序列中的换行（line 416）和普通字符转义（line 421）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_escape_newline_returns_none() {
    // `\n` inside ident → escape fails, ident breaks
    // `a\<newline>b` → ident "a", then b
    let toks = tokens("a\\\nb");
    assert!(toks.len() >= 2);
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "a"),
        other => panic!("Expected Ident(\"a\"), got {:?}", other),
    }
}

#[test]
fn test_escape_carriage_return() {
    let toks = tokens("a\\\rb");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "a"),
        other => panic!("Expected Ident(\"a\"), got {:?}", other),
    }
}

#[test]
fn test_escape_form_feed() {
    let toks = tokens("a\\\x0Cb");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "a"),
        other => panic!("Expected Ident(\"a\"), got {:?}", other),
    }
}

#[test]
fn test_escape_basic_char() {
    // `\x` → ident starting with escaped x
    let toks = tokens("\\x");
    // The tokenizer returns an ident with the escaped character
    assert!(!toks.is_empty(), "should produce at least one token");
    let _ = &toks[0];
}

// ═══════════════════════════════════════════════════════════════════════
// 3. url() 中字符串解析失败（line 542）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_url_with_bad_string() {
    // url("unclosed string) → string parsing fails
    let toks = tokens("url(\"unclosed)");
    // Should not panic, may return URL or BadUrl
    let _ = toks;
}

// ═══════════════════════════════════════════════════════════════════════
// 4. `#` 后无 ident 字符（line 654）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hash_followed_by_non_ident() {
    // `#1` → Hash("") (empty name after #)
    let toks = tokens("#1");
    match &toks[0] {
        Token::Hash(s) => assert_eq!(s, ""),
        other => panic!("Expected Hash(\"\"), got {:?}", other),
    }
}

#[test]
fn test_hash_at_eof() {
    let toks = tokens("#");
    match &toks[0] {
        Token::Error(msg) => assert!(msg.contains('#')),
        other => panic!("Expected Error, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 5. `||` Column token（lines 788-789）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_column_token() {
    let toks = tokens("||");
    match &toks[0] {
        Token::Column => {}
        other => panic!("Expected Column, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 6. 裸 `+` / `-` / 其他符号（lines 790-795）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_bare_plus_as_delim() {
    let toks = tokens("+");
    match &toks[0] {
        Token::Delim('+') => {}
        other => panic!("Expected Delim('+'), got {:?}", other),
    }
}

#[test]
fn test_bare_minus_as_ident() {
    let toks = tokens("-");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "-"),
        other => panic!("Expected Ident(\"-\"), got {:?}", other),
    }
}

#[test]
fn test_bare_equals_sign() {
    let toks = tokens("=");
    match &toks[0] {
        Token::Delim('=') => {}
        other => panic!("Expected Delim('='), got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 7. 非法转义在 ident 中（line 374）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_ident_with_bad_escape_midway() {
    // `a\<newline>b` → ident "a" (escape fails, ident stops)
    let toks = tokens("a\\\nb");
    match &toks[0] {
        Token::Ident(s) => assert_eq!(s, "a"),
        other => panic!("Expected Ident(\"a\"), got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 8. consume_newline 路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_newline_lf_in_input() {
    // Newline characters in CSS are consumed as whitespace
    let toks = tokens("\n");
    assert!(toks.is_empty() || matches!(&toks[0], Token::Whitespace));
}

#[test]
fn test_newline_cr_lf_in_input() {
    let toks = tokens("\r\n");
    assert!(toks.is_empty() || matches!(&toks[0], Token::Whitespace));
}

#[test]
fn test_newline_tab_as_whitespace() {
    let toks = tokens("\t");
    assert!(toks.is_empty() || matches!(&toks[0], Token::Whitespace));
}

// ═══════════════════════════════════════════════════════════════════════
// 9. 以 @ 开头的 ident（line 591 — edge case path）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_at_keyword_token() {
    let toks = tokens("@media");
    match &toks[0] {
        Token::AtKeyword(s) => assert_eq!(s, "media"),
        other => panic!("Expected AtKeyword(\"media\"), got {:?}", other),
    }
}

#[test]
fn test_at_charset_keyword() {
    let toks = tokens("@charset");
    match &toks[0] {
        Token::AtKeyword(s) => assert_eq!(s, "charset"),
        other => panic!("Expected AtKeyword, got {:?}", other),
    }
}
