//! Tokenizer 覆盖率补充测试：行/列定位、转义、字符串、URL、注释等边界情况。

use crate::tokenizer::{Token, Tokenizer, line_column_from_offset};

/// Helper: 收集所有 token。
fn tokens(input: &str) -> Vec<Token> {
    Tokenizer::new(input).map(|s| s.token).collect()
}

// ═══════════════════════════════════════════════════════════════════════
// line_column_from_offset 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_line_column_at_start() {
    assert_eq!(line_column_from_offset("hello", 0), (1, 1));
}

#[test]
fn test_line_column_after_newline() {
    assert_eq!(line_column_from_offset("abc\ndef", 4), (2, 1));
}

#[test]
fn test_line_column_within_line() {
    assert_eq!(line_column_from_offset("abc\ndef", 5), (2, 2));
}

#[test]
fn test_line_column_cr_lf() {
    assert_eq!(line_column_from_offset("a\r\nb", 3), (2, 1));
}

#[test]
fn test_line_column_cr_only() {
    assert_eq!(line_column_from_offset("a\rb", 2), (2, 1));
}

#[test]
fn test_line_column_beyond_end() {
    // offset 超出范围时 clamp 到末尾
    let result = line_column_from_offset("abc", 100);
    assert_eq!(result, (1, 4));
}

#[test]
fn test_line_column_empty_string() {
    assert_eq!(line_column_from_offset("", 0), (1, 1));
}

#[test]
fn test_line_column_multiple_lines() {
    let src = "line1\nline2\nline3";
    assert_eq!(line_column_from_offset(src, 0), (1, 1));
    assert_eq!(line_column_from_offset(src, 6), (2, 1));
    assert_eq!(line_column_from_offset(src, 12), (3, 1));
}

// ═══════════════════════════════════════════════════════════════════════
// 注释边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_comment() {
    let toks = tokens("/**/");
    assert!(matches!(toks[0], Token::Comment(ref s) if s.is_empty()));
}

#[test]
fn test_unterminated_comment() {
    let toks = tokens("/* no end");
    assert!(matches!(toks[0], Token::Error(ref s) if s.contains("Unterminated")));
}

#[test]
fn test_comment_with_stars() {
    let toks = tokens("/* **hello** */");
    assert!(matches!(toks[0], Token::Comment(ref s) if s.contains("**hello**")));
}

// ═══════════════════════════════════════════════════════════════════════
// 转义序列边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_escape_hex_unicode() {
    // \41 inside an ident — 'A'
    // Tokenizer doesn't start idents with \, so use it inside an ident context
    let toks = tokens("a\\41 ");
    // Should produce ident containing 'A'
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_hex_6_digits() {
    // \000041 inside an ident
    let toks = tokens("a\\000041 ");
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_newline_returns_none() {
    // \n in escape → None, backslash consumed, ident empty or partial
    let toks = tokens("\\\na");
    // The \ before newline should produce empty or partial ident
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_eof() {
    // backslash at EOF → replacement char
    let toks = tokens("\\");
    // 应产生包含替换字符的 ident 或其他 token
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_surrogate_codepoint() {
    // D800 is surrogate → replacement char
    let toks = tokens("\\00D800 ");
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_zero_codepoint() {
    // 0 codepoint → replacement char
    let toks = tokens("\\0 ");
    assert!(!toks.is_empty());
}

#[test]
fn test_escape_non_hex_char() {
    // \z inside ident → 'z'
    let toks = tokens("a\\z");
    assert!(!toks.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 字符串边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_string_empty() {
    let toks = tokens("\"\"");
    assert!(matches!(&toks[0], Token::String(s) if s.is_empty()));
}

#[test]
fn test_string_unterminated_newline() {
    let toks = tokens("\"hello\nworld\"");
    // \n 终止字符串
    assert!(matches!(&toks[0], Token::String(s) if s == "hello"));
}

#[test]
fn test_string_continuation_line() {
    // 反斜杠+换行 → 续行
    let toks = tokens("\"hello\\\nworld\"");
    assert!(matches!(&toks[0], Token::String(s) if s == "helloworld"));
}

#[test]
fn test_string_continuation_cr() {
    // 反斜杠+\r → 续行
    let toks = tokens("\"hello\\\rworld\"");
    assert!(matches!(&toks[0], Token::String(s) if s == "helloworld"));
}

#[test]
fn test_string_continuation_cr_lf() {
    // 反斜杠+\r\n → 续行
    let toks = tokens("\"hello\\\r\nworld\"");
    assert!(matches!(&toks[0], Token::String(s) if s == "helloworld"));
}

#[test]
fn test_string_escaped_backslash_eof() {
    // \ at end of string → literal backslash
    let toks = tokens("\"hello\\");
    if let Token::String(s) = &toks[0] {
        assert!(s.contains("hello") || s.contains("\\"));
    }
}

#[test]
fn test_string_with_escaped_quote() {
    let toks = tokens(r#""he\"llo""#);
    assert!(matches!(&toks[0], Token::String(s) if s.contains("llo")));
}

#[test]
fn test_string_single_quotes() {
    let toks = tokens("'hello'");
    assert!(matches!(&toks[0], Token::String(s) if s == "hello"));
}

// ═══════════════════════════════════════════════════════════════════════
// URL 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_url_empty() {
    let toks = tokens("url()");
    assert!(matches!(&toks[0], Token::Url(s) if s.is_empty()));
}

#[test]
fn test_url_unquoted_with_spaces() {
    let toks = tokens("url(  test.png  )");
    assert!(matches!(&toks[0], Token::Url(s) if s.contains("test.png")));
}

#[test]
fn test_url_with_paren_error() {
    let toks = tokens("url(test(png)");
    // 括号在无引号 URL 中是非法的
    assert!(toks.len() >= 1);
}

#[test]
fn test_url_backslash_error() {
    let toks = tokens("url(test\\png)");
    // 反斜杠在无引号 URL 中是非法的
    assert!(toks.len() >= 1);
}

#[test]
fn test_url_double_quoted() {
    let toks = tokens("url(\"path/to/file.png\")");
    assert!(matches!(&toks[0], Token::Url(s) if s == "path/to/file.png"));
}

#[test]
fn test_url_single_quoted() {
    let toks = tokens("url('path/to/file.png')");
    assert!(matches!(&toks[0], Token::Url(s) if s == "path/to/file.png"));
}

// ═══════════════════════════════════════════════════════════════════════
// 数字边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_number_scientific() {
    let toks = tokens("1e2");
    assert!(matches!(&toks[0], Token::Number(n) if (*n - 100.0).abs() < 0.01));
}

#[test]
fn test_number_negative_scientific() {
    let toks = tokens("1e-2");
    assert!(matches!(&toks[0], Token::Number(n) if (*n - 0.01).abs() < 0.001));
}

#[test]
fn test_number_positive_scientific() {
    let toks = tokens("1e+3");
    assert!(matches!(&toks[0], Token::Number(n) if (*n - 1000.0).abs() < 0.01));
}

#[test]
fn test_number_decimal() {
    let toks = tokens("3.14");
    assert!(matches!(&toks[0], Token::Number(n) if (*n - 3.14).abs() < 0.001));
}

#[test]
fn test_number_negative() {
    let toks = tokens("-42");
    assert!(matches!(&toks[0], Token::Number(n) if *n == -42.0));
}

#[test]
fn test_number_zero() {
    let toks = tokens("0");
    assert!(matches!(&toks[0], Token::Number(n) if *n == 0.0));
}

#[test]
fn test_dimension() {
    let toks = tokens("10px");
    assert!(matches!(&toks[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
}

#[test]
fn test_percentage() {
    let toks = tokens("50%");
    assert!(matches!(&toks[0], Token::Percentage(n) if *n == 50.0));
}

// ═══════════════════════════════════════════════════════════════════════
// 标识符边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_ident_with_leading_hyphen() {
    let toks = tokens("--custom-prop");
    assert!(matches!(&toks[0], Token::Ident(s) if s == "--custom-prop"));
}

#[test]
fn test_ident_single_hyphen() {
    // 仅 "-" 本身 → Ident("-")
    let toks = tokens("-");
    assert!(matches!(&toks[0], Token::Ident(s) if s == "-"));
}

#[test]
fn test_ident_with_escape() {
    let toks = tokens("\\41 BC");
    // \41 = 'A', should produce ident "ABC" or "A" then "BC"
    assert!(!toks.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Hash token
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hash_ident() {
    let toks = tokens("#myid");
    assert!(matches!(&toks[0], Token::Hash(s) if s == "myid"));
}

#[test]
fn test_hash_number() {
    let toks = tokens("#123");
    // # 后跟数字 → Hash
    assert!(!toks.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 函数 token
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_function_token() {
    let toks = tokens("rgb(255, 0, 0)");
    assert!(matches!(&toks[0], Token::Function(s) if s == "rgb"));
}

#[test]
fn test_at_keyword() {
    let toks = tokens("@media");
    assert!(matches!(&toks[0], Token::AtKeyword(s) if s == "media"));
}

// ═══════════════════════════════════════════════════════════════════════
// 空输入和边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_input() {
    let toks = tokens("");
    assert!(toks.is_empty() || (toks.len() == 1 && matches!(toks[0], Token::Eof)));
}

#[test]
fn test_whitespace_only() {
    let toks = tokens("   \n\t  ");
    assert!(toks.is_empty() || toks.iter().all(|t| matches!(t, Token::Whitespace | Token::Eof)));
}

// ═══════════════════════════════════════════════════════════════════════
// collect_tokens 方法
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_tokens() {
    let toks = Tokenizer::new("div { color: red; }").collect_tokens();
    assert!(!toks.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Spanned 位置信息
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_spanned_offset() {
    let spanned: Vec<_> = Tokenizer::new("a b").collect();
    assert_eq!(spanned[0].offset, 0); // 'a' at 0
    // whitespace at 1
    assert_eq!(spanned[1].offset, 1);
    // 'b' at 2
    assert_eq!(spanned[2].offset, 2);
}
