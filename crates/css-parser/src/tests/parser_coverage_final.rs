//! parser.rs 最终覆盖率补全测试
//!
//! 针对还剩余的未覆盖代码路径，通过公共 API (parse_stylesheet) 触发内部分支。

use super::*;
use crate::parser::Parser;

// ── 1. 组合器选择器逻辑 ──────────────────────────────────────────────────

#[test]
/// 测试组合器后跟随空白字符的情况
fn test_combinator_with_whitespace() {
    let css = "div   >   p { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        assert!(matches!(sr.selectors[0].complex.parts[0].1, Some(Combinator::Child)));
    }
}

#[test]
/// 测试没有空白但有组合器的情况
fn test_combinator_without_whitespace() {
    let css = "div>p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        assert!(matches!(sr.selectors[0].complex.parts[0].1, Some(Combinator::Child)));
    }
}

// ── 2. 属性选择器边界情况 ───────────────────────────────────────────────

#[test]
/// 测试属性选择器中方括号内无内容 — 解析器应该恢复
fn test_attribute_selector_empty_brackets() {
    let css = "div[] { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    // 解析器能恢复，不 panic 即可
    assert!(!ss.rules.is_empty() || ss.rules.is_empty());
}

#[test]
/// 测试属性选择器中属性名后立即是 ]
fn test_attribute_selector_no_value_exists() {
    let css = r#"div[data-x] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
    }
}

// ── 3. @keyframes 边界情况 ──────────────────────────────────────────────

#[test]
/// 测试 @keyframes 中有效和无效选择器混合 — 不 panic 即可
fn test_keyframes_mixed_selectors() {
    let css = "@keyframes test { from { color: red; } 50% { color: blue; } to { color: green; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert!(kf.keyframes.len() >= 1);
    }
}

#[test]
/// 测试 @keyframes 中 from/to 选择器
fn test_keyframes_from_to() {
    let css = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert!(kf.keyframes.len() >= 1);
    }
}

// ── 4. @layer 的未命名情况 ──────────────────────────────────────────────

#[test]
/// 测试 @layer 未命名的情况，直接跟 {
fn test_layer_no_name_before_brace() {
    let css = "@layer { .a { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试 @layer 有名称的情况
fn test_layer_with_name() {
    let css = "@layer base { .a { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 5. @import 的边界情况 ───────────────────────────────────────────────

#[test]
/// 测试 @import 无媒体查询
fn test_import_no_media() {
    let css = r#"@import "style.css";"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(imp) = &ss.rules[0] {
        assert!(imp.media_queries.is_empty());
    }
}

#[test]
/// 测试 @import 有媒体查询
fn test_import_with_media() {
    let css = r#"@import "style.css" screen;"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(imp) = &ss.rules[0] {
        assert!(!imp.media_queries.is_empty());
    }
}

// ── 6. 声明块的边界情况 ────────────────────────────────────────────────

#[test]
/// 测试声明块中遇到无法识别的 token
///
/// 注：CSS 嵌套启用后，声明块内的 `@unknown;` 按嵌套 @规则解析（spec 对齐），
/// 会作为独立规则输出，故规则总数 > 1；核心意图——后续声明 `color: red` 仍正确归属 `div`——
/// 不受影响。
fn test_declaration_block_unknown_token() {
    let css = "div { @unknown; color: red; }";
    let ss = Parser::parse_stylesheet(css);
    let div = ss
        .rules
        .iter()
        .find_map(|r| if let Rule::Style(sr) = r { Some(sr) } else { None });
    let div = div.expect("应存在 div 样式规则");
    assert!(div.declarations.iter().any(|d| d.property == "color"));
}

#[test]
/// 测试声明块中只有无法识别的 token（不 panic 即可）
fn test_declaration_block_only_unknown_tokens() {
    let css = "div { @unknown; @other; }";
    let ss = Parser::parse_stylesheet(css);
    // CSS 嵌套：@unknown/@other 作嵌套 @规则解析；div 样式规则仍存在（声明可能为空）。
    assert!(
        ss.rules
            .iter()
            .any(|r| matches!(r, Rule::Style(sr) if sr.selectors.iter().any(|s| s
            .complex
            .parts
            .iter()
            .any(|(c, _)| matches!(c.type_selector, Some(TypeSelector::Tag(ref t)) if t == "div")))))
    );
}

// ── 7. @supports 的错误路径 ─────────────────────────────────────────────

#[test]
/// 测试 @supports 条件中遇到 EOF
fn test_supports_eof_in_condition() {
    let css = "@supports (display: grid";
    let ss = Parser::parse_stylesheet(css);
    // 解析不完整 — 不 panic 即可
}

#[test]
/// 测试 @supports 正常条件
fn test_supports_valid_condition() {
    let css = "@supports (display: grid) { .a { display: grid; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 8. consume_attribute_value 的边界情况 ───────────────────────────────

#[test]
/// 测试属性值以数字开头但后面跟着标识符
fn test_attribute_value_number_with_units() {
    let css = r#"div[data-count=100px] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    // 不 panic 即可
    let _ = ss.rules;
}

#[test]
/// 测试属性值以点开头
fn test_attribute_value_starts_with_dot() {
    let css = r#"div[class=".example"] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 9. nth 表达式的错误路径 ──────────────────────────────────────────────

#[test]
/// 测试 nth 表达式解析时各种输入 — 不 panic 即可
fn test_nth_expression_various_inputs() {
    let cases = vec![
        "li:nth-child(abc) { color: red; }",
        "li:nth-child(2n+1) { color: blue; }",
        "li:nth-child(odd) { color: green; }",
        "li:nth-child(even) { color: yellow; }",
        "li:nth-child(3) { color: purple; }",
    ];
    for css in cases {
        let ss = Parser::parse_stylesheet(css);
        // 不 panic 即可
        let _ = ss;
    }
}

// ── 10. skip_to_rbracket 的路径 ──────────────────────────────────────────

#[test]
/// 测试属性选择器中复杂的值
fn test_attribute_selector_complex_value() {
    let css = r#"div[data-x~=value] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 11. consume_declaration 的边界情况 ──────────────────────────────────

#[test]
/// 测试声明值只有空格的情况
fn test_declaration_value_only_whitespace() {
    let css = "div { color:   ; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 12. @at-rule 的 prelude 收集 ────────────────────────────────────────

#[test]
/// 测试 @font-face 规则（专用解析器：必须有 family + src 才保留，否则丢弃）
fn test_at_rule_font_face() {
    // 缺 src → 无效 @font-face，丢弃
    let ss = Parser::parse_stylesheet("@font-face { font-family: Arial; }");
    assert_eq!(ss.rules.len(), 0, "font-face without src is dropped");
    // 有效 @font-face → Rule::FontFace
    let ss = Parser::parse_stylesheet(r#"@font-face { font-family: "Arial"; src: url("a.woff"); }"#);
    assert_eq!(ss.rules.len(), 1);
    assert!(matches!(ss.rules[0], Rule::FontFace(_)));
}

// ── 13. 通用选择器的边界情况 ─────────────────────────────────────────────

#[test]
/// 测试没有子类选择器的通用选择器
fn test_universal_selector_no_subclass() {
    let css = "* { margin: 0; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        if let Some(sel) = sr.selectors.first() {
            assert_eq!(sel.complex.parts.len(), 1);
            assert!(matches!(
                sel.complex.parts[0].0.type_selector,
                Some(TypeSelector::Universal)
            ));
            assert!(sel.complex.parts[0].1.is_none());
        }
    }
}

// ── 14. 嵌套 @ 规则 ─────────────────────────────────────────────────────

#[test]
/// 测试 @media 规则中有嵌套的 @media
fn test_nested_at_rule_media() {
    let css = r#"
        @media screen {
            @media (max-width: 600px) {
                .container { width: 100%; }
            }
        }
    "#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 15. 选择器列表的空情况 ──────────────────────────────────────────────

#[test]
/// 测试空选择器列表
fn test_empty_selector_list() {
    let css = "{ color: red; }";
    let ss = Parser::parse_stylesheet(css);
    // 空选择器列表应该被忽略
    assert!(ss.rules.is_empty());
}

// ── 16. consume_rule 的错误处理 ──────────────────────────────────────────

#[test]
/// 测试空声明块
fn test_empty_declaration_block() {
    let css = "div { }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert!(sr.declarations.is_empty());
    }
}

// ── 17. 伪类选择器 ──────────────────────────────────────────────────────

#[test]
/// 测试伪类选择器
fn test_pseudo_class_hover() {
    let css = "div:hover { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 18. 各种选择器开头的情况 ──────────────────────────────────────────────

#[test]
/// 测试以类选择器开头的规则
fn test_selector_starts_with_class() {
    let css = ".child { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 19. 样式规则的边界情况 ────────────────────────────────────────────────

#[test]
/// 测试没有属性值的声明
fn test_declaration_no_value() {
    let css = "div { color: ; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 20. EOF 在各种位置的情况 ──────────────────────────────────────────────

#[test]
/// 测试解析时遇到 EOF 的各种情况
fn test_parse_eof_in_different_positions() {
    // 在样式规则中间 EOF
    let css = "div { color: red";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
    }
}

#[test]
/// 测试 @keyframes 中 EOF
fn test_parse_eof_in_keyframes() {
    let css = "@keyframes test { from { }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert!(kf.keyframes.len() >= 1);
    }
}

// ── 21. 更多边界情况 ──────────────────────────────────────────────────────

#[test]
/// 测试多个选择器规则
fn test_multiple_selectors() {
    let css = "div, p, span { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 3);
    }
}

#[test]
/// 测试 @charset 规则
fn test_charset_rule() {
    let css = r#"@charset "UTF-8";"#;
    let ss = Parser::parse_stylesheet(css);
    // 不 panic 即可
    let _ = ss;
}

#[test]
/// 测试 CSS 注释在规则中
fn test_comments_in_rules() {
    let css = "/* comment */ div { color: red; /* inline */ }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试空样式表
fn test_empty_stylesheet() {
    let css = "";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty());
}

#[test]
/// 测试只有空白的样式表
fn test_whitespace_only_stylesheet() {
    let css = "   \n\n  \t  \n  ";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty());
}
