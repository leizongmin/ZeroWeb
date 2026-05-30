//! CSS 解析器综合测试。

use crate::tokenizer::{Token, Tokenizer};
use crate::parser::Parser;
use crate::ast::*;
use crate::selector;

// ═══════════════════════════════════════════════════════════════════════
// 1. Tokenizer 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_ident() {
    let tokens: Vec<_> = Tokenizer::new("div").collect();
    assert_eq!(tokens, vec![Token::Ident("div".to_string())]);
}

#[test]
fn test_tokenize_at_keyword() {
    let tokens: Vec<_> = Tokenizer::new("@media").collect();
    assert_eq!(tokens, vec![Token::AtKeyword("media".to_string())]);
}

#[test]
fn test_tokenize_hash() {
    let tokens: Vec<_> = Tokenizer::new("#main").collect();
    assert_eq!(tokens, vec![Token::Hash("main".to_string())]);
}

#[test]
fn test_tokenize_hash_color() {
    let tokens: Vec<_> = Tokenizer::new("#fff").collect();
    assert_eq!(tokens, vec![Token::Hash("fff".to_string())]);
}

#[test]
fn test_tokenize_string_double() {
    let tokens: Vec<_> = Tokenizer::new("\"hello world\"").collect();
    assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
}

#[test]
fn test_tokenize_string_single() {
    let tokens: Vec<_> = Tokenizer::new("'hello'").collect();
    assert_eq!(tokens, vec![Token::String("hello".to_string())]);
}

#[test]
fn test_tokenize_number() {
    let tokens: Vec<_> = Tokenizer::new("42").collect();
    assert!(matches!(tokens[0], Token::Number(n) if n == 42.0));
}

#[test]
fn test_tokenize_number_decimal() {
    let tokens: Vec<_> = Tokenizer::new("3.14").collect();
    assert!(matches!(tokens[0], Token::Number(n) if (n - 3.14).abs() < 0.001));
}

#[test]
fn test_tokenize_percentage() {
    let tokens: Vec<_> = Tokenizer::new("50%").collect();
    assert!(matches!(tokens[0], Token::Percentage(n) if n == 50.0));
}

#[test]
fn test_tokenize_dimension_px() {
    let tokens: Vec<_> = Tokenizer::new("10px").collect();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
}

#[test]
fn test_tokenize_dimension_em() {
    let tokens: Vec<_> = Tokenizer::new("1.5em").collect();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if (*n - 1.5).abs() < 0.001 && u == "em"));
}

#[test]
fn test_tokenize_function() {
    let tokens: Vec<_> = Tokenizer::new("rgb(").collect();
    assert_eq!(tokens, vec![Token::Function("rgb".to_string())]);
}

#[test]
fn test_tokenize_url() {
    let tokens: Vec<_> = Tokenizer::new("url(image.png)").collect();
    assert_eq!(tokens, vec![Token::Url("image.png".to_string())]);
}

#[test]
fn test_tokenize_colon() {
    let tokens: Vec<_> = Tokenizer::new(":").collect();
    assert_eq!(tokens, vec![Token::Colon]);
}

#[test]
fn test_tokenize_semicolon() {
    let tokens: Vec<_> = Tokenizer::new(";").collect();
    assert_eq!(tokens, vec![Token::Semicolon]);
}

#[test]
fn test_tokenize_comma() {
    let tokens: Vec<_> = Tokenizer::new(",").collect();
    assert_eq!(tokens, vec![Token::Comma]);
}

#[test]
fn test_tokenize_braces() {
    let tokens: Vec<_> = Tokenizer::new("{}").collect();
    assert_eq!(tokens, vec![Token::LBrace, Token::RBrace]);
}

#[test]
fn test_tokenize_brackets() {
    let tokens: Vec<_> = Tokenizer::new("[]").collect();
    assert_eq!(tokens, vec![Token::LBracket, Token::RBracket]);
}

#[test]
fn test_tokenize_parens() {
    let tokens: Vec<_> = Tokenizer::new("()").collect();
    assert_eq!(tokens, vec![Token::LParen, Token::RParen]);
}

#[test]
fn test_tokenize_whitespace() {
    let tokens: Vec<_> = Tokenizer::new("  \t\n").collect();
    assert_eq!(tokens, vec![Token::Whitespace]);
}

#[test]
fn test_tokenize_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* hello */").collect();
    assert_eq!(tokens, vec![Token::Comment(" hello ".to_string())]);
}

#[test]
fn test_tokenize_attribute_matchers() {
    let tokens: Vec<_> = Tokenizer::new("~=").collect();
    assert_eq!(tokens, vec![Token::IncludeMatch]);

    let tokens: Vec<_> = Tokenizer::new("|=").collect();
    assert_eq!(tokens, vec![Token::DashMatch]);

    let tokens: Vec<_> = Tokenizer::new("^=").collect();
    assert_eq!(tokens, vec![Token::PrefixMatch]);

    let tokens: Vec<_> = Tokenizer::new("$=").collect();
    assert_eq!(tokens, vec![Token::SuffixMatch]);

    let tokens: Vec<_> = Tokenizer::new("*=").collect();
    assert_eq!(tokens, vec![Token::SubstringMatch]);
}

#[test]
fn test_tokenize_negative_number() {
    let tokens: Vec<_> = Tokenizer::new("-10px").collect();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == -10.0 && u == "px"));
}

#[test]
fn test_tokenize_simple_rule() {
    let tokens: Vec<_> = Tokenizer::new("div { color: red; }").collect();
    assert!(tokens.len() >= 5);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Specificity 测试
// ═══════════════════════════════════════════════════════════════════════

fn tag_sel(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}

fn id_sel(id: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                },
                None,
            )],
        },
    }
}

fn class_sel(cls: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_specificity_simple_tag() {
    assert_eq!(selector::specificity(&tag_sel("div")), (0, 0, 1));
}

#[test]
fn test_specificity_simple_id() {
    assert_eq!(selector::specificity(&id_sel("main")), (1, 0, 0));
}

#[test]
fn test_specificity_simple_class() {
    assert_eq!(selector::specificity(&class_sel("active")), (0, 1, 0));
}

#[test]
fn test_specificity_attribute() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "type".to_string(),
                        matcher: AttributeMatcher::Exact("text".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert_eq!(selector::specificity(&sel), (0, 1, 0));
}

#[test]
fn test_specificity_combined() {
    // div#main.active → (1, 1, 1)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![
                        SubclassSelector::Id("main".to_string()),
                        SubclassSelector::Class("active".to_string()),
                    ],
                },
                None,
            )],
        },
    };
    assert_eq!(selector::specificity(&sel), (1, 1, 1));
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Parser 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_simple_rule() {
    let stylesheet = Parser::parse_stylesheet("div { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_multiple_rules() {
    let stylesheet = Parser::parse_stylesheet("div { color: red; } span { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 2);
}

#[test]
fn test_parse_at_media() {
    let stylesheet = Parser::parse_stylesheet("@media screen { div { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            assert!(at_rule.prelude.contains("screen"));
        }
        _ => panic!("Expected At rule"),
    }
}

#[test]
fn test_parse_at_import() {
    let stylesheet = Parser::parse_stylesheet("@import url(style.css);");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "import");
            assert!(matches!(at_rule.body, AtRuleBody::Statement));
        }
        _ => panic!("Expected At rule"),
    }
}

#[test]
fn test_parse_declaration() {
    let stylesheet = Parser::parse_stylesheet("div { color: red; font-size: 16px; }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(style_rule) => {
            assert!(!style_rule.declarations.is_empty());
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_parse_empty_stylesheet() {
    let stylesheet = Parser::parse_stylesheet("");
    assert_eq!(stylesheet.rules.len(), 0);
}

#[test]
fn test_parse_comment_only() {
    let stylesheet = Parser::parse_stylesheet("/* comment */");
    assert_eq!(stylesheet.rules.len(), 0);
}

#[test]
fn test_parse_at_layer() {
    let stylesheet = Parser::parse_stylesheet("@layer base { div { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "layer");
        }
        _ => panic!("Expected At rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Tokenizer 边界条件
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_zero() {
    let tokens: Vec<_> = Tokenizer::new("0").collect();
    assert!(matches!(tokens[0], Token::Number(0.0)));
}

#[test]
fn test_tokenize_escaped_ident() {
    let tokens: Vec<_> = Tokenizer::new("\\41 ").collect(); // \41 = 'A', needs space terminator
    // Escaped hex codepoint should produce a valid ident (could be "A" or "A ")
    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_multiple_rules() {
    let css = "div { color: red; } .class { font-size: 16px; }";
    let tokens: Vec<_> = Tokenizer::new(css).collect();
    assert!(tokens.len() > 10);
}

#[test]
fn test_tokenize_nested_parens() {
    let css = "rgba(255, 0, 0, 0.5)";
    let tokens: Vec<_> = Tokenizer::new(css).collect();
    assert!(tokens.len() >= 2); // At least Function + some content
}

#[test]
fn test_tokenize_rem_dimension() {
    let tokens: Vec<_> = Tokenizer::new("1.2rem").collect();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if (*n - 1.2).abs() < 0.001 && u == "rem"));
}

#[test]
fn test_tokenize_vh_dimension() {
    let tokens: Vec<_> = Tokenizer::new("100vh").collect();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 100.0 && u == "vh"));
}

#[test]
fn test_tokenize_unterminated_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* unterminated").collect();
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenize_unterminated_string() {
    let tokens: Vec<_> = Tokenizer::new("\"unterminated").collect();
    // Should still return a string (partial)
    assert!(matches!(&tokens[0], Token::String(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Tokenizer Delim 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_dot_as_delim() {
    let tokens: Vec<_> = Tokenizer::new(".").collect();
    assert_eq!(tokens, vec![Token::Delim('.')]);
}

#[test]
fn test_tokenize_bang_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("!").collect();
    assert_eq!(tokens, vec![Token::Delim('!')]);
}

#[test]
fn test_tokenize_greater_as_delim() {
    let tokens: Vec<_> = Tokenizer::new(">").collect();
    assert_eq!(tokens, vec![Token::Delim('>')]);
}

#[test]
fn test_tokenize_plus_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("+").collect();
    assert_eq!(tokens, vec![Token::Delim('+')]);
}

#[test]
fn test_tokenize_star_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("*").collect();
    assert_eq!(tokens, vec![Token::Delim('*')]);
}

#[test]
fn test_tokenize_tilde_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("~").collect();
    assert_eq!(tokens, vec![Token::Delim('~')]);
}

#[test]
fn test_tokenize_complex_selector() {
    // div.class#id:hover → Ident Delim('.') Ident Hash Colon Ident
    let tokens: Vec<_> = Tokenizer::new("div.class#id:hover").collect();
    assert!(tokens.len() >= 6);
    assert_eq!(tokens[0], Token::Ident("div".to_string()));
    assert_eq!(tokens[1], Token::Delim('.'));
    assert_eq!(tokens[2], Token::Ident("class".to_string()));
    assert_eq!(tokens[3], Token::Hash("id".to_string()));
    assert_eq!(tokens[4], Token::Colon);
    assert_eq!(tokens[5], Token::Ident("hover".to_string()));
}

#[test]
fn test_tokenize_dot_before_digit_still_number() {
    // ".5" → Number(0.5)
    let tokens: Vec<_> = Tokenizer::new(".5").collect();
    assert!(matches!(tokens[0], Token::Number(n) if (n - 0.5).abs() < 0.001));
}

#[test]
fn test_tokenize_child_combinator_in_context() {
    // div > p → Ident Whitespace Delim('>') Whitespace Ident
    let tokens: Vec<_> = Tokenizer::new("div > p").collect();
    assert!(tokens.len() >= 5);
    assert_eq!(tokens[0], Token::Ident("div".to_string()));
    assert_eq!(tokens[1], Token::Whitespace);
    assert_eq!(tokens[2], Token::Delim('>'));
    assert_eq!(tokens[3], Token::Whitespace);
    assert_eq!(tokens[4], Token::Ident("p".to_string()));
}

#[test]
fn test_tokenize_important() {
    // !important → Delim('!') Ident("important")
    let tokens: Vec<_> = Tokenizer::new("!important").collect();
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0], Token::Delim('!'));
    assert_eq!(tokens[1], Token::Ident("important".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Parser 选择器测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_class_selector() {
    let stylesheet = Parser::parse_stylesheet(".class { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.type_selector.is_none());
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Class(c) if c == "class"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_id_selector() {
    let stylesheet = Parser::parse_stylesheet("#main { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Id(id) if id == "main"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_tag_class() {
    let stylesheet = Parser::parse_stylesheet("div.active { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(matches!(
            &compound.type_selector,
            Some(TypeSelector::Tag(t)) if t == "div"
        ));
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Class(c) if c == "active"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_universal() {
    let stylesheet = Parser::parse_stylesheet("* { margin: 0; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(matches!(compound.type_selector, Some(TypeSelector::Universal)));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_descendant() {
    let stylesheet = Parser::parse_stylesheet("div p { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        // 第一个组合器应为 Descendant
        assert_eq!(parts[0].1, Some(Combinator::Descendant));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_child() {
    let stylesheet = Parser::parse_stylesheet("div > p { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].1, Some(Combinator::Child));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_next_sibling() {
    let stylesheet = Parser::parse_stylesheet("h1 + p { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].1, Some(Combinator::NextSibling));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_subsequent_sibling() {
    let stylesheet = Parser::parse_stylesheet("h1 ~ p { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].1, Some(Combinator::SubsequentSibling));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_attribute_exists() {
    let stylesheet = Parser::parse_stylesheet("[type] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Exists,
            }) if name == "type"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_attribute_exact() {
    let stylesheet = Parser::parse_stylesheet("[type=text] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Exact(val),
            }) if name == "type" && val == "text"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_attribute_includes() {
    let stylesheet = Parser::parse_stylesheet("[class~=active] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Includes(val),
            }) if name == "class" && val == "active"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_pseudo_class() {
    let stylesheet = Parser::parse_stylesheet("a:hover { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name))
                if name == "hover"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_pseudo_element() {
    let stylesheet = Parser::parse_stylesheet("p::before { content: \"\"; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoElement(PseudoElementSelector::Standard(name))
                if name == "before"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_selector_list() {
    let stylesheet = Parser::parse_stylesheet("div, span { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // 应该有 2 个选择器
        assert_eq!(sr.selectors.len(), 2);
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_important() {
    let stylesheet = Parser::parse_stylesheet("div { color: red !important; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let has_important = sr.declarations.iter().any(|d| d.important);
        assert!(has_important, "Expected !important declaration");
    } else {
        panic!("Expected Style rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 7. 值解析测试
// ═══════════════════════════════════════════════════════════════════════

use crate::values::*;

#[test]
fn test_parse_color_named() {
    let result = parse_color("red");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_parse_color_hex3() {
    let result = parse_color("#f00");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_parse_color_hex6() {
    let result = parse_color("#ff0000");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_parse_color_hex8() {
    let result = parse_color("#ff000080");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 128)));
}

#[test]
fn test_parse_color_rgb() {
    let result = parse_color("rgb(255, 0, 0)");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_parse_color_transparent() {
    let result = parse_color("transparent");
    assert_eq!(result, Some(ColorValue::Transparent));
}

#[test]
fn test_parse_color_current_color() {
    let result = parse_color("currentColor");
    assert_eq!(result, Some(ColorValue::CurrentColor));
}

#[test]
fn test_parse_length_px() {
    let result = parse_length("10px");
    assert_eq!(result, Some(LengthValue::Px(10.0)));
}

#[test]
fn test_parse_length_em() {
    let result = parse_length("1.5em");
    assert_eq!(result, Some(LengthValue::Em(1.5)));
}

#[test]
fn test_parse_length_rem() {
    let result = parse_length("2rem");
    assert_eq!(result, Some(LengthValue::Rem(2.0)));
}

#[test]
fn test_parse_display_values() {
    assert_eq!(parse_display("block"), Some(DisplayValue::Block));
    assert_eq!(parse_display("inline"), Some(DisplayValue::Inline));
    assert_eq!(parse_display("inline-block"), Some(DisplayValue::InlineBlock));
    assert_eq!(parse_display("flex"), Some(DisplayValue::Flex));
    assert_eq!(parse_display("inline-flex"), Some(DisplayValue::InlineFlex));
    assert_eq!(parse_display("grid"), Some(DisplayValue::Grid));
    assert_eq!(parse_display("inline-grid"), Some(DisplayValue::InlineGrid));
    assert_eq!(parse_display("none"), Some(DisplayValue::None));
    assert_eq!(parse_display("contents"), Some(DisplayValue::Contents));
    assert_eq!(parse_display("flow"), Some(DisplayValue::Flow));
    assert_eq!(parse_display("flow-root"), Some(DisplayValue::FlowRoot));
    assert_eq!(parse_display("list-item"), Some(DisplayValue::ListItem));
    assert_eq!(parse_display("unknown"), None);
}

#[test]
fn test_parse_position_values() {
    assert_eq!(parse_position("static"), Some(PositionValue::Static));
    assert_eq!(parse_position("relative"), Some(PositionValue::Relative));
    assert_eq!(parse_position("absolute"), Some(PositionValue::Absolute));
    assert_eq!(parse_position("fixed"), Some(PositionValue::Fixed));
    assert_eq!(parse_position("sticky"), Some(PositionValue::Sticky));
    assert_eq!(parse_position("unknown"), None);
}

#[test]
fn test_parse_var_simple() {
    let result = parse_var("var(--color)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--color");
    assert!(var.fallback.is_none());
}

#[test]
fn test_parse_var_fallback() {
    let result = parse_var("var(--color, red)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--color");
    assert_eq!(var.fallback, Some("red".to_string()));
}

#[test]
fn test_parse_var_invalid() {
    let result = parse_var("not-a-var");
    assert_eq!(result, None);
}
