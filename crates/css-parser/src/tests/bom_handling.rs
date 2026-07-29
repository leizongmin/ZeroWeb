//! CSS Syntax §3.3 输入预处理：stylesheet 首 U+FEFF (BOM) 须忽略。
//!
//! 背景：external CSS 经 `net::charset::decode_with` 已剥 UTF-8/UTF-16 BOM，但 inline
//! `<style>` 文本（html5ever 不剥离文档中段的 FEFF）与直接 `parse_stylesheet`/`load_html`
//! 调用仍可能带首 BOM。修复前 FEFF（`!is_ascii()`）被 `is_ident_start` 当成标识符首字符，
//! 污染紧跟其后的首个选择器标签名（`"\u{FEFF}body"` ≠ `"body"`），致该规则选择器失配。
//! 修复：`Tokenizer::new` 按 §3.3 剥离首个 U+FEFF（中段 BOM 作 ZERO WIDTH NO-BREAK SPACE
//! 是合法 ident 字符，保留）。

use super::*;

/// 提取首个复杂选择器的标签名（type selector），便于断言。
fn first_tag(rule: &Rule) -> Option<&str> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style
        .selectors
        .first()
        .and_then(|s| s.complex.parts.first())
        .and_then(|(compound, _)| match &compound.type_selector {
            Some(TypeSelector::Tag(t)) => Some(t.as_str()),
            _ => None,
        })
}

/// 首个 U+FEFF (BOM) 被忽略，首个选择器标签名仍是 `body`。
#[test]
fn test_leading_bom_does_not_pollute_first_selector() {
    let css = "\u{FEFF}body { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "首 BOM 不应吞掉规则");
    assert_eq!(
        first_tag(&stylesheet.rules[0]),
        Some("body"),
        "首 BOM 不应污染首个选择器标签名"
    );
}

/// 首个 BOM + 多规则：全部保留，标签名正确。
#[test]
fn test_leading_bom_multiple_rules() {
    let css = "\u{FEFF}body { color: red; }\np { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2, "两条规则都应保留");
    assert_eq!(first_tag(&stylesheet.rules[0]), Some("body"));
    assert_eq!(first_tag(&stylesheet.rules[1]), Some("p"));
}

/// tokenizer 层：首 BOM 剥离后首个 token 是 Ident("body")（非含 BOM 的 ident）。
#[test]
fn test_leading_bom_stripped_at_token_level() {
    let tokens = Tokenizer::new("\u{FEFF}body").collect_tokens();
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "body")),
        "首 BOM 剥离后应得到干净的 Ident(\"body\")"
    );
}

/// 无 BOM 输入行为不变（回归守护）。
#[test]
fn test_no_bom_input_unchanged() {
    let css = "body { color: red; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert_eq!(first_tag(&stylesheet.rules[0]), Some("body"));
}
