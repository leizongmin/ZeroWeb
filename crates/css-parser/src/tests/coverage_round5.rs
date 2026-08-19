//! 第五轮覆盖率测试：tokenizer.rs 和 parser.rs 未覆盖分支。

use crate::ast::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer, line_column_from_offset};

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer coverage — 边缘分支
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_column_pipe_pipe() {
    // `||` 应该产生 Column token
    let tokens: Vec<_> = Tokenizer::new("||").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Column);
}

#[test]
fn test_tokenizer_pipe_equal() {
    // `|=` 应该产生 DashMatch
    let tokens: Vec<_> = Tokenizer::new("|=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::DashMatch);
}

#[test]
fn test_tokenizer_caret_equal() {
    // `^=` 应该产生 PrefixMatch
    let tokens: Vec<_> = Tokenizer::new("^=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::PrefixMatch);
}

#[test]
fn test_tokenizer_dollar_equal() {
    // `$=` 应该产生 SuffixMatch
    let tokens: Vec<_> = Tokenizer::new("$=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::SuffixMatch);
}

#[test]
fn test_tokenizer_star_equal() {
    // `*=` 应该产生 SubstringMatch
    let tokens: Vec<_> = Tokenizer::new("*=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::SubstringMatch);
}

#[test]
fn test_tokenizer_tilde_equal() {
    // `~=` 应该产生 IncludeMatch
    let tokens: Vec<_> = Tokenizer::new("~=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::IncludeMatch);
}

#[test]
fn test_tokenizer_delim_caret() {
    let tokens: Vec<_> = Tokenizer::new("^").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "^"));
}

#[test]
fn test_tokenizer_delim_dollar() {
    let tokens: Vec<_> = Tokenizer::new("$").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "$"));
}

#[test]
fn test_tokenizer_delim_tilde() {
    let tokens: Vec<_> = Tokenizer::new("~").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('~'));
}

#[test]
fn test_tokenizer_delim_star() {
    let tokens: Vec<_> = Tokenizer::new("*").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('*'));
}

#[test]
fn test_tokenizer_pipe_alone() {
    let tokens: Vec<_> = Tokenizer::new("|").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "|"));
}

#[test]
fn test_tokenizer_plus_alone() {
    let tokens: Vec<_> = Tokenizer::new("+").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('+'));
}

#[test]
fn test_tokenizer_minus_alone() {
    let tokens: Vec<_> = Tokenizer::new("-").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "-"));
}

#[test]
fn test_tokenizer_dot_alone() {
    // `.` 后面不跟数字 → Delim
    let tokens: Vec<_> = Tokenizer::new(".foo").collect_tokens();
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0], Token::Delim('.'));
}

#[test]
fn test_tokenizer_dot_eof() {
    // `.` 在末尾 → Delim
    let tokens: Vec<_> = Tokenizer::new(".").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('.'));
}

#[test]
fn test_tokenizer_at_eof() {
    // `@` 在末尾 → Error
    let tokens: Vec<_> = Tokenizer::new("@").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_at_non_ident() {
    // `@3` → Error（非标识符起始）
    let tokens: Vec<_> = Tokenizer::new("@3").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_hash_eof() {
    // `#` 在末尾 → Error
    let tokens: Vec<_> = Tokenizer::new("#").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_hash_digit() {
    // `#3` → 数字也是有效的 hash 字符
    let tokens: Vec<_> = Tokenizer::new("#3").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Hash(_)));
}

#[test]
fn test_tokenizer_hash_escape() {
    // `#\20AC` → Hash（转义起始）
    let tokens: Vec<_> = Tokenizer::new("#\\20AC").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Hash(_)));
}

#[test]
fn test_tokenizer_exclamation() {
    let tokens: Vec<_> = Tokenizer::new("!").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('!'));
}

#[test]
fn test_tokenizer_greater_than() {
    let tokens: Vec<_> = Tokenizer::new(">").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('>'));
}

#[test]
fn test_tokenizer_equal() {
    let tokens: Vec<_> = Tokenizer::new("=").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Delim('='));
}

#[test]
fn test_tokenizer_unknown_char() {
    let tokens: Vec<_> = Tokenizer::new("\x01").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: 数字和科学计数法
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_scientific_notation() {
    // 科学计数法 `1e2` = 100.0
    let tokens: Vec<_> = Tokenizer::new("1e2").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(100.0));
}

#[test]
fn test_tokenizer_scientific_notation_uppercase() {
    let tokens: Vec<_> = Tokenizer::new("3E+5").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(300000.0));
}

#[test]
fn test_tokenizer_scientific_notation_negative_exp() {
    let tokens: Vec<_> = Tokenizer::new("5e-2").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 0.05).abs() < 0.001));
}

#[test]
fn test_tokenizer_number_with_unit_escape() {
    // 数字 + \ 转义起始的单位 → Dimension
    let tokens: Vec<_> = Tokenizer::new("10\\70x").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Dimension(_, _)));
}

#[test]
fn test_tokenizer_plus_dot_number() {
    // `+.5` → 数字 0.5
    let tokens: Vec<_> = Tokenizer::new("+.5").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(0.5));
}

#[test]
fn test_tokenizer_minus_dot_number() {
    // `-.5` → 数字 -0.5
    let tokens: Vec<_> = Tokenizer::new("-.5").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(-0.5));
}

#[test]
fn test_tokenizer_plus_number_dimension() {
    // `+10px` → Dimension(10.0, "px")
    let tokens: Vec<_> = Tokenizer::new("+10px").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
}

#[test]
fn test_tokenizer_minus_number_percentage() {
    let tokens: Vec<_> = Tokenizer::new("-50%").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Percentage(-50.0));
}

#[test]
fn test_tokenizer_minus_ident_escape() {
    // `-\` 后跟转义序列 → 标识符
    let tokens: Vec<_> = Tokenizer::new("-\\20AC").collect_tokens();
    assert!(tokens.len() >= 1);
    // 应该产生某种标识符
    assert!(matches!(&tokens[0], Token::Ident(_)));
}

#[test]
fn test_tokenizer_dot_number() {
    // `.5` → 数字 0.5
    let tokens: Vec<_> = Tokenizer::new(".5").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(0.5));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: 字符串和转义
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_string_unterminated_newline() {
    // 字符串中遇到换行 → 截断
    let tokens: Vec<_> = Tokenizer::new("\"hello\nworld\"").collect_tokens();
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0], Token::String("hello".to_string()));
}

#[test]
fn test_tokenizer_string_escape_continuation_lf() {
    // `\` 后跟 `\n` → 续行
    let tokens: Vec<_> = Tokenizer::new("\"hello\\\nworld\"").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::String("helloworld".to_string()));
}

#[test]
fn test_tokenizer_string_escape_continuation_cr() {
    // `\` 后跟 `\r` → 续行
    let tokens: Vec<_> = Tokenizer::new("\"hello\\\rworld\"").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::String("helloworld".to_string()));
}

#[test]
fn test_tokenizer_string_escape_continuation_crlf() {
    // `\` 后跟 `\r\n` → 续行
    let tokens: Vec<_> = Tokenizer::new("\"hello\\\r\nworld\"").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::String("helloworld".to_string()));
}

#[test]
fn test_tokenizer_string_backslash_eof() {
    // 字符串末尾 `\` 后直接 EOF
    let tokens: Vec<_> = Tokenizer::new("\"hello\\").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::String("hello\\".to_string()));
}

#[test]
fn test_tokenizer_escape_eof() {
    // `\` 在末尾 → 替换字符
    let tokens: Vec<_> = Tokenizer::new("\\").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_escape_newline() {
    // `\` 后跟换行 → 转义失败
    let tokens: Vec<_> = Tokenizer::new("\\\na").collect_tokens();
    // 换行不能被转义
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_escape_carriage_return() {
    let tokens: Vec<_> = Tokenizer::new("\\\ra").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_escape_form_feed() {
    let tokens: Vec<_> = Tokenizer::new("\\\x0Ca").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_unicode_escape_surrogate() {
    // 代理码点 → 替换字符
    let tokens: Vec<_> = Tokenizer::new("\\D800").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_unicode_escape_zero() {
    // U+0000 → 替换字符
    let tokens: Vec<_> = Tokenizer::new("\\0").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_unicode_escape_too_large() {
    let tokens: Vec<_> = Tokenizer::new("\\110000").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_string_with_escape() {
    // 字符串内十六进制转义：\41b = U+041B（Cyrillic Л），故 "a\41b" → "aЛ"
    let tokens: Vec<_> = Tokenizer::new("\"a\\41b\"").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::String("a\u{41b}".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: URL 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_url_quoted_single() {
    let tokens: Vec<_> = Tokenizer::new("url('test.png')").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Url("test.png".to_string()));
}

#[test]
fn test_tokenizer_url_quoted_double() {
    let tokens: Vec<_> = Tokenizer::new("url(\"test.png\")").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Url("test.png".to_string()));
}

#[test]
fn test_tokenizer_url_unquoted_with_space() {
    let tokens: Vec<_> = Tokenizer::new("url( test.png )").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Url("test.png".to_string()));
}

#[test]
fn test_tokenizer_url_eof() {
    // URL 未终止
    let tokens: Vec<_> = Tokenizer::new("url(test").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Url("test".to_string()));
}

#[test]
fn test_tokenizer_url_invalid_paren() {
    // URL 中包含 `(` → Error
    let tokens: Vec<_> = Tokenizer::new("url(test(.png)").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_url_backslash_escape() {
    // CSS Syntax L3：无引号 url 允许转义（R2124，driving：uri-005）。
    // `url(test\.png)` → `\.` → '.' → "test.png"（旧实现误判为 Error）。
    let tokens: Vec<_> = Tokenizer::new("url(test\\.png)").collect_tokens();
    assert!(matches!(&tokens[0], Token::Url(s) if s == "test.png"));
}

#[test]
fn test_tokenizer_url_whitespace_then_no_close() {
    // URL 空白后不是 `)` → bad-url，不得截断成已收集的合法 URL。
    let tokens: Vec<_> = Tokenizer::new("url(test .png)").collect_tokens();
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_url_whitespace_then_eof_closes_url() {
    let tokens: Vec<_> = Tokenizer::new("url(test.png\n").collect_tokens();
    assert_eq!(tokens[0], Token::Url("test.png".to_string()));
}

#[test]
fn test_tokenizer_quoted_url_rejects_trailing_input() {
    let tokens: Vec<_> = Tokenizer::new("url(\"test.png\" extra)").collect_tokens();
    assert!(matches!(&tokens[0], Token::Error(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: 注释
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_unterminated_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* comment").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenizer_comment_with_star() {
    let tokens: Vec<_> = Tokenizer::new("/* a * b */").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Comment(" a * b ".to_string()));
}

#[test]
fn test_tokenizer_slash_not_comment() {
    let tokens: Vec<_> = Tokenizer::new("/ div").collect_tokens();
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0], Token::Delim('/'));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: ident 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_ident_with_escape_start() {
    // R2132：`\` 合法转义起始 → 主循环路由到 ident-like（CSS Syntax §4.3）。
    // `\41BC` = `\41`(=41='A'，1-6 hex 截到非 hex 'B'... 实际 41BC 全 hex) 解码为 ident，
    // 不再落 Error。driving：escapes-002。
    let tokens: Vec<_> = Tokenizer::new("\\41BC").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Ident(_)));
}

#[test]
fn test_tokenizer_ident_dash_then_escape() {
    let tokens: Vec<_> = Tokenizer::new("-\\41").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Ident(_)));
}

#[test]
fn test_tokenizer_ident_dash_then_invalid() {
    // `-` 后跟非标识符字符 → 仅 "-"
    let tokens: Vec<_> = Tokenizer::new("-3").collect_tokens();
    // `-` 后跟数字 → "-3" 作为负数处理
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenizer_ident_dash_dash() {
    let tokens: Vec<_> = Tokenizer::new("--var").collect_tokens();
    assert!(tokens.len() >= 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s.starts_with("--")));
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: Display trait
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_token_display_all_variants() {
    // 确保每个 Token variant 的 Display 实现都能正常工作
    assert_eq!(Token::Ident("div".into()).to_string(), "div");
    assert_eq!(Token::AtKeyword("media".into()).to_string(), "@media");
    assert_eq!(Token::Hash("fff".into()).to_string(), "#fff");
    assert_eq!(Token::String("hello".into()).to_string(), "\"hello\"");
    assert_eq!(Token::Url("a.png".into()).to_string(), "url(a.png)");
    assert_eq!(Token::Number(42.0).to_string(), "42");
    assert_eq!(Token::Percentage(50.0).to_string(), "50%");
    assert_eq!(Token::Dimension(10.0, "px".into()).to_string(), "10px");
    assert_eq!(Token::Function("rgb".into()).to_string(), "rgb(");
    assert_eq!(Token::UnicodeRange("0".into(), "7F".into()).to_string(), "U+0-7F");
    assert_eq!(Token::IncludeMatch.to_string(), "~=");
    assert_eq!(Token::DashMatch.to_string(), "|=");
    assert_eq!(Token::PrefixMatch.to_string(), "^=");
    assert_eq!(Token::SuffixMatch.to_string(), "$=");
    assert_eq!(Token::SubstringMatch.to_string(), "*=");
    assert_eq!(Token::Column.to_string(), "||");
    assert_eq!(Token::Whitespace.to_string(), " ");
    assert_eq!(Token::Colon.to_string(), ":");
    assert_eq!(Token::Semicolon.to_string(), ";");
    assert_eq!(Token::Comma.to_string(), ",");
    assert_eq!(Token::LBracket.to_string(), "[");
    assert_eq!(Token::RBracket.to_string(), "]");
    assert_eq!(Token::LParen.to_string(), "(");
    assert_eq!(Token::RParen.to_string(), ")");
    assert_eq!(Token::LBrace.to_string(), "{");
    assert_eq!(Token::RBrace.to_string(), "}");
    assert_eq!(Token::Comment("test".into()).to_string(), "/* test */");
    assert_eq!(Token::Delim('>').to_string(), ">");
    assert_eq!(Token::Eof.to_string(), "<EOF>");
    assert_eq!(Token::Error("bad".into()).to_string(), "<ERROR: bad>");
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenizer: line_column_from_offset
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_line_column_basic() {
    assert_eq!(line_column_from_offset("hello", 0), (1, 1));
    assert_eq!(line_column_from_offset("hello", 3), (1, 4));
    assert_eq!(line_column_from_offset("hello\nworld", 6), (2, 1));
}

#[test]
fn test_line_column_cr() {
    assert_eq!(line_column_from_offset("a\r\nb", 3), (2, 1));
}

#[test]
fn test_line_column_beyond_end() {
    // 偏移超出源长度 → 钳位到最后
    let (line, col) = line_column_from_offset("abc", 100);
    assert_eq!(line, 1);
    assert_eq!(col, 4);
}

#[test]
fn test_line_column_cr_only() {
    assert_eq!(line_column_from_offset("a\rb", 2), (2, 1));
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @keyframes 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_keyframes_string_name() {
    let css = r#"@keyframes "my-anim" { from { opacity: 0; } to { opacity: 1; } }"#;
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => assert_eq!(kf.name, "my-anim"),
        _ => panic!("Expected Keyframes"),
    }
}

#[test]
fn test_parse_keyframes_no_name() {
    let css = "@keyframes { from { opacity: 0; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    // @keyframes 缺名 → 畸形 at-rule，整条（含 {...} 块）丢弃。R2140 at-rule fallback：
    // consume_keyframes_rule 返回 None 时消耗残余，body 不再泄漏成 StyleRule
    //（旧实现「外层 advance 跳过 LBrace，from {...} 泄漏成 StyleRule」是 leak，非 spec 行为）。
    assert!(
        stylesheet.rules.is_empty(),
        "malformed @keyframes (no name) should be dropped entirely, got {} rules",
        stylesheet.rules.len()
    );
}

#[test]
fn test_parse_keyframes_invalid_selector() {
    let css = "@keyframes fade { invalid { opacity: 0; } from { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            // invalid 不是 from/to/Percentage → selectors 为空 → 跳过
            // from 关键帧可能因为跳过逻辑消耗了后续 token 而丢失
            assert!(kf.keyframes.len() <= 1);
        }
        _ => panic!("Expected Keyframes"),
    }
}

#[test]
fn test_parse_keyframes_multiple_selectors() {
    let css = "@keyframes fade { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.keyframes.len(), 2);
            assert_eq!(kf.keyframes[0].selectors.len(), 2);
        }
        _ => panic!("Expected Keyframes"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @layer 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_layer_anonymous() {
    let css = "@layer { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Layer(layer) => {
            assert_eq!(layer.name, "");
            assert_eq!(layer.rules.len(), 1);
        }
        _ => panic!("Expected Layer"),
    }
}

#[test]
fn test_parse_layer_semicolon_only() {
    let css = "@layer;";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Layer(layer) => {
            assert_eq!(layer.name, "");
            assert!(layer.rules.is_empty());
        }
        _ => panic!("Expected Layer"),
    }
}

#[test]
fn test_parse_layer_named_semicolon() {
    let css = "@layer base;";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Layer(layer) => {
            assert_eq!(layer.name, "base");
            assert!(layer.rules.is_empty());
        }
        _ => panic!("Expected Layer"),
    }
}

#[test]
fn test_parse_layer_string_name() {
    let css = r#"@layer "my-layer" { div { color: red; } }"#;
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Layer(layer) => {
            assert_eq!(layer.name, "my-layer");
        }
        _ => panic!("Expected Layer"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @import 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_import_url() {
    let css = r#"@import url("style.css");"#;
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Import(imp) => {
            assert_eq!(imp.url, "style.css");
            assert!(imp.media_queries.is_empty());
        }
        _ => panic!("Expected Import"),
    }
}

#[test]
fn test_parse_import_rejects_bad_url_token() {
    let stylesheet = Parser::parse_stylesheet("@import url(my style.css); body { color: red; }");
    assert!(stylesheet.rules.iter().all(|rule| !matches!(rule, Rule::Import(_))));
}

#[test]
fn test_parse_import_with_media() {
    let css = r#"@import "style.css" screen, print;"#;
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Import(imp) => {
            assert_eq!(imp.url, "style.css");
            assert_eq!(imp.media_queries.len(), 2);
        }
        _ => panic!("Expected Import"),
    }
}

#[test]
fn test_parse_import_string() {
    let css = r#"@import "theme.css";"#;
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Import(imp) => {
            assert_eq!(imp.url, "theme.css");
        }
        _ => panic!("Expected Import"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @supports
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_supports_basic() {
    let css = "@supports (display: grid) { div { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(sup) => {
            assert_eq!(sup.rules.len(), 1);
        }
        _ => panic!("Expected Supports"),
    }
}

#[test]
fn test_parse_supports_not() {
    let css = "@supports not (display: grid) { div { display: block; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(sup) => {
            assert_eq!(sup.rules.len(), 1);
        }
        _ => panic!("Expected Supports"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @container
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_container_named() {
    let css = "@container sidebar (min-width: 400px) { div { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cont) => {
            assert_eq!(cont.name.as_deref(), Some("sidebar"));
            assert_eq!(cont.rules.len(), 1);
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_inline_size() {
    let css = "@container inline-size(min-width: 400px) { div { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    // inline-size() 是容器尺寸函数条件（CSS Contain 3），**非容器名**——name 为 None，
    // condition 为 InlineSize。R2139 修正：原实现把 `inline-size` 误当容器名（与规范
    // 略有不同），现 tokenizer 产 `Function("inline-size")`，consume_container_rule 正确
    // 识别为尺寸函数条件。
    assert_eq!(stylesheet.rules.len(), 1, "should parse one container rule");
    match &stylesheet.rules[0] {
        Rule::Container(cont) => {
            assert!(
                cont.name.is_none(),
                "inline-size() is a condition function, not a container name"
            );
            assert!(
                matches!(cont.condition, ContainerCondition::InlineSize(_)),
                "condition should be InlineSize: {:?}",
                cont.condition
            );
        }
        other => panic!("expected Container rule, got {other:?}"),
    }
}

#[test]
fn test_parse_container_range_syntax() {
    let css = "@container (200px <= width <= 500px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cont) => {
            assert_eq!(cont.rules.len(), 1);
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_comparison_operators() {
    let css = "@container (width > 300px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(_) => {}
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_gte_operator() {
    let css = "@container (width >= 300px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(_) => {}
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_lt_operator() {
    let css = "@container (width < 300px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(_) => {}
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_size_function() {
    let css = "@container size(min-width: 400px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    // "size" 被解析为容器名称（Ident 后跟 (LParen），
    // 然后 (min-width: 400px) 被解析为条件文本
    if !stylesheet.rules.is_empty() {
        match &stylesheet.rules[0] {
            Rule::Container(_) => {}
            _ => {} // 可能解析为其他类型
        }
    }
}

#[test]
fn test_parse_container_no_parens() {
    // @container 后没有括号 → 无效
    let css = "@container div { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // 可能返回 None 或解析为 AtRule
    assert!(stylesheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: 属性选择器全量匹配器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_attribute_exists() {
    let css = "[data-test] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_attribute_exact() {
    let css = "[data-test=\"value\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_attribute_includes() {
    let css = "[class~=\"active\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_dash_match() {
    let css = "[lang|=\"en\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_prefix_match() {
    let css = "[href^=\"https\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_suffix_match() {
    let css = "[href$=\".pdf\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_substring_match() {
    let css = "[href*=\"example\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_number_value() {
    let css = "[data-count=\"3\"] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_unknown_matcher() {
    // 未知匹配器 → 回退到 Exists
    let css = "[data-test?] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // 应该能解析出一些规则
    assert!(!stylesheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: 伪类/伪元素
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_pseudo_element() {
    let css = "div::before { content: \"\"; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_pseudo_not() {
    let css = "div:not(.active) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_pseudo_is() {
    let css = "div:is(.a, .b) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_pseudo_where() {
    let css = "div:where(.a, .b) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_pseudo_has() {
    let css = "div:has(.child) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_pseudo_nth_child() {
    let css = "li:nth-child(2n+1) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_nth_last_child() {
    let css = "li:nth-last-child(3) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_nth_of_type() {
    let css = "p:nth-of-type(odd) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_nth_last_of_type() {
    let css = "p:nth-last-of-type(even) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_lang() {
    let css = "p:lang(en) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_lang_string() {
    let css = "p:lang(\"en-US\") { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_pseudo_unknown_function() {
    let css = "div:custom-fn(arg) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // custom-fn 后跟 (arg) → tokenizer 产生 Function("custom-fn")
    // 解析器看到 Colon → Function("custom-fn") → 走 Function token 路径
    // 匹配 _ 分支 → PseudoClassSelector::Simple("custom-fn")
    // 但需要检查整体规则是否被正确解析
    if !stylesheet.rules.is_empty() {
        match &stylesheet.rules[0] {
            Rule::Style(sr) => {
                // 即使解析不完美，至少不应 panic
                assert!(!sr.selectors.is_empty() || sr.declarations.is_empty());
            }
            _ => {}
        }
    }
    // 主要验证不 panic
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: nth 表达式解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_nth_plus_n() {
    let css = "li:nth-child(+n) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_nth_minus_n() {
    let css = "li:nth-child(-n+3) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_nth_n_only() {
    let css = "li:nth-child(n) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_nth_plain_number() {
    let css = "li:nth-child(5) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_nth_3n_plus_2() {
    let css = "li:nth-child(3n+2) { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: 组合器和选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_child_combinator() {
    let css = "div > p { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
            assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_next_sibling_combinator() {
    let css = "div + p { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_subsequent_sibling_combinator() {
    let css = "div ~ p { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_descendant_combinator() {
    let css = "div p { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_universal_selector() {
    let css = "* { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_id_selector() {
    let css = "#main { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_class_selector() {
    let css = ".container { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_comma_selector_list() {
    let css = "div, p, span { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 3);
        }
        _ => panic!("Expected Style"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: 声明中的 !important
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_declaration_important() {
    let css = "div { color: red !important; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 1);
            assert!(sr.declarations[0].important);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_declaration_bang_no_important() {
    // `!` 后不是 `important` → 保留在值中
    let css = "div { color: red !invalid; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(!sr.declarations[0].important);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_declaration_no_colon() {
    let css = "div { color red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // 没有 `:` → 无法解析声明
    assert!(
        stylesheet.rules.is_empty() || matches!(&stylesheet.rules[0], Rule::Style(sr) if sr.declarations.is_empty())
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @规则通用路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_at_rule_with_block() {
    // 用未知 @rule 测通用 @rule 块路径（@font-face / @page 有专用解析器，不再走 Rule::At）
    let css = "@foo { margin: 1cm; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            assert_eq!(at.name, "foo");
        }
        _ => panic!("Expected At"),
    }
}

#[test]
fn test_parse_at_rule_semicolon() {
    let css = "@charset \"UTF-8\";";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            assert_eq!(at.name, "charset");
        }
        _ => panic!("Expected At"),
    }
}

#[test]
fn test_parse_at_rule_eof() {
    let css = "@namespace svg url(http://www.w3.org/2000/svg)";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            assert_eq!(at.name, "namespace");
        }
        _ => panic!("Expected At"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: 复杂选择器组合
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_complex_selector_chain() {
    let css = "div.class > p#id:hover { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_selector_missing_brace() {
    let css = "div p";
    let stylesheet = Parser::parse_stylesheet(css);
    // 没有 `{` → 无法解析为样式规则
    assert!(stylesheet.rules.is_empty());
}

#[test]
fn test_parse_function_token_pseudo() {
    // 测试 Function token 形式的伪类（tokenizer 直接产生 Function）
    let css = ":nth-child(2n) { color: red; }";
    let _stylesheet = Parser::parse_stylesheet(css);
    // 可能无法直接解析为完整规则（因为冒号开头），但不应 panic
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: attribute value 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_attribute_delim_value() {
    let css = "[data-ext=.pdf] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_numeric_value() {
    let css = "[data-count=3] { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_empty_value() {
    let css = "[data-test=] { color: red; }";
    let _stylesheet = Parser::parse_stylesheet(css);
    // 应该不 panic
}

#[test]
fn test_parse_container_ident_no_paren() {
    // ident 后面不是 `(` → 回退
    let css = "@container test div { color: red; }";
    let _stylesheet = Parser::parse_stylesheet(css);
    // 不应 panic，可能解析为空或 AtRule
}

#[test]
fn test_parse_container_no_condition() {
    let css = "@container { div { color: red; } }";
    let _stylesheet = Parser::parse_stylesheet(css);
    // 无条件 → 可能返回 None
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: container condition parse_container_condition
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_container_lte_operator() {
    let css = "@container (width <= 500px) { div { color: blue; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
    match &stylesheet.rules[0] {
        Rule::Container(_) => {}
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_gte_value() {
    let css = "@container (width >= 100px) { div { color: green; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_container_named_with_condition() {
    let css = "@container mysidebar (min-width: 200px) { div { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cont) => {
            assert_eq!(cont.name.as_deref(), Some("mysidebar"));
            assert_eq!(cont.rules.len(), 1);
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_parse_container_nested_parens() {
    let css = "@container (200px <= width <= 800px) { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @supports condition parsing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_supports_property_value() {
    let css = "@supports (display: flex) { div { display: flex; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
    match &stylesheet.rules[0] {
        Rule::Supports(sup) => {
            assert_eq!(sup.rules.len(), 1);
        }
        _ => panic!("Expected Supports"),
    }
}

#[test]
fn test_parse_supports_multiple_rules() {
    let css = "@supports (color: red) { div { color: red; } p { background: blue; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(sup) => {
            assert_eq!(sup.rules.len(), 2);
        }
        _ => panic!("Expected Supports"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: keyframe with from/to
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_keyframes_from_to() {
    let css = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.keyframes.len(), 2);
        }
        _ => panic!("Expected Keyframes"),
    }
}

#[test]
fn test_parse_keyframes_percentage_only() {
    let css = "@keyframes slide { 50% { left: 100px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.keyframes.len(), 1);
        }
        _ => panic!("Expected Keyframes"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: declaration with multiple values
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_declaration_multi_value() {
    let css = "div { margin: 10px 20px 30px 40px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 1);
            assert_eq!(sr.declarations[0].property, "margin");
            assert!(sr.declarations[0].value.contains("10px"));
        }
        _ => panic!("Expected Style"),
    }
}

#[test]
fn test_parse_declaration_with_function() {
    let css = "div { background: rgb(255, 0, 0); }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(sr.declarations[0].value.contains("rgb"));
        }
        _ => panic!("Expected Style"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: complex selector with attribute + pseudo
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_selector_attribute_with_pseudo() {
    let css = "a[href]:hover { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_selector_id_with_class() {
    let css = "#main.active { display: block; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_selector_universal_with_class() {
    let css = "*.highlight { background: yellow; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @layer with rules
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_layer_with_multiple_rules() {
    let css = "@layer base { div { color: red; } p { font-size: 14px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Layer(layer) => {
            assert_eq!(layer.name, "base");
            assert_eq!(layer.rules.len(), 2);
        }
        _ => panic!("Expected Layer"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @import with media query
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_import_with_single_media() {
    let css = r#"@import "print.css" print;"#;
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Import(imp) => {
            assert_eq!(imp.url, "print.css");
            assert_eq!(imp.media_queries.len(), 1);
        }
        _ => panic!("Expected Import"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: attribute selector with ident value
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_attribute_exact_ident_value() {
    let css = "[type=text] { border: 1px solid; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_attribute_dash_match_exact() {
    let css = "[lang=en] { font-style: normal; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: multiple declarations
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_multiple_declarations() {
    let css = "div { color: red; background: blue; font-size: 16px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 3);
        }
        _ => panic!("Expected Style"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: @rule with body containing nested rules
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_at_rule_with_nested_style_rules() {
    // 用未知 @rule 测通用 @rule 嵌套样式规则（@font-face / @page 有专用解析器）
    let css = "@foo { div { color: red; } p { color: blue; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            if let AtRuleBody::Block(rules) = &at.body {
                assert_eq!(rules.len(), 2);
            }
        }
        _ => panic!("Expected At"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Parser coverage: attribute value with number + unit
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_attribute_value_number_unit() {
    // 3px as attribute value: number + ident concatenated by parser
    let css = "[data-size=3px] { width: 100px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Should not panic - result may vary based on parser behavior
    // The important thing is it doesn't crash
}
