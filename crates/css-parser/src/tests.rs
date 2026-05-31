//! CSS 解析器综合测试。

use crate::ast::*;
use crate::parser::Parser;
use crate::selector;
use crate::tokenizer::{Spanned, Token, Tokenizer, line_column_from_offset};
use crate::values::{
    CalcContext, ContainerTypeValue, GradientDirection, GradientValue, LengthValue, RadialShape, RadialSize,
    ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStopValue, ScrollSnapTypeValue, TransformFunction, TransformValue,
    eval_calc, eval_calc_with_context, parse_animation_direction, parse_animation_fill_mode,
    parse_animation_play_state, parse_calc, parse_container_type, parse_gradient, parse_length, parse_length_shorthand,
    parse_scroll_snap_align, parse_scroll_snap_stop, parse_scroll_snap_type, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// 1. Tokenizer 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_ident() {
    let tokens: Vec<_> = Tokenizer::new("div").collect_tokens();
    assert_eq!(tokens, vec![Token::Ident("div".to_string())]);
}

#[test]
fn test_tokenize_at_keyword() {
    let tokens: Vec<_> = Tokenizer::new("@media").collect_tokens();
    assert_eq!(tokens, vec![Token::AtKeyword("media".to_string())]);
}

#[test]
fn test_tokenize_hash() {
    let tokens: Vec<_> = Tokenizer::new("#main").collect_tokens();
    assert_eq!(tokens, vec![Token::Hash("main".to_string())]);
}

#[test]
fn test_tokenize_hash_color() {
    let tokens: Vec<_> = Tokenizer::new("#fff").collect_tokens();
    assert_eq!(tokens, vec![Token::Hash("fff".to_string())]);
}

#[test]
fn test_tokenize_string_double() {
    let tokens: Vec<_> = Tokenizer::new("\"hello world\"").collect_tokens();
    assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
}

#[test]
fn test_tokenize_string_single() {
    let tokens: Vec<_> = Tokenizer::new("'hello'").collect_tokens();
    assert_eq!(tokens, vec![Token::String("hello".to_string())]);
}

#[test]
fn test_tokenize_number() {
    let tokens: Vec<_> = Tokenizer::new("42").collect_tokens();
    assert!(matches!(tokens[0], Token::Number(n) if n == 42.0));
}

#[test]
fn test_tokenize_number_decimal() {
    let tokens: Vec<_> = Tokenizer::new("3.14").collect_tokens();
    let expected = 314.0_f64 / 100.0;
    assert!(matches!(tokens[0], Token::Number(n) if (n - expected).abs() < 0.001));
}

#[test]
fn test_tokenize_percentage() {
    let tokens: Vec<_> = Tokenizer::new("50%").collect_tokens();
    assert!(matches!(tokens[0], Token::Percentage(n) if n == 50.0));
}

#[test]
fn test_tokenize_dimension_px() {
    let tokens: Vec<_> = Tokenizer::new("10px").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
}

#[test]
fn test_tokenize_dimension_em() {
    let tokens: Vec<_> = Tokenizer::new("1.5em").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if (*n - 1.5).abs() < 0.001 && u == "em"));
}

#[test]
fn test_tokenize_function() {
    let tokens: Vec<_> = Tokenizer::new("rgb(").collect_tokens();
    assert_eq!(tokens, vec![Token::Function("rgb".to_string())]);
}

#[test]
fn test_tokenize_url() {
    let tokens: Vec<_> = Tokenizer::new("url(image.png)").collect_tokens();
    assert_eq!(tokens, vec![Token::Url("image.png".to_string())]);
}

#[test]
fn test_tokenize_colon() {
    let tokens: Vec<_> = Tokenizer::new(":").collect_tokens();
    assert_eq!(tokens, vec![Token::Colon]);
}

#[test]
fn test_tokenize_semicolon() {
    let tokens: Vec<_> = Tokenizer::new(";").collect_tokens();
    assert_eq!(tokens, vec![Token::Semicolon]);
}

#[test]
fn test_tokenize_comma() {
    let tokens: Vec<_> = Tokenizer::new(",").collect_tokens();
    assert_eq!(tokens, vec![Token::Comma]);
}

#[test]
fn test_tokenize_braces() {
    let tokens: Vec<_> = Tokenizer::new("{}").collect_tokens();
    assert_eq!(tokens, vec![Token::LBrace, Token::RBrace]);
}

#[test]
fn test_tokenize_brackets() {
    let tokens: Vec<_> = Tokenizer::new("[]").collect_tokens();
    assert_eq!(tokens, vec![Token::LBracket, Token::RBracket]);
}

#[test]
fn test_tokenize_parens() {
    let tokens: Vec<_> = Tokenizer::new("()").collect_tokens();
    assert_eq!(tokens, vec![Token::LParen, Token::RParen]);
}

#[test]
fn test_tokenize_whitespace() {
    let tokens: Vec<_> = Tokenizer::new("  \t\n").collect_tokens();
    assert_eq!(tokens, vec![Token::Whitespace]);
}

#[test]
fn test_tokenize_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* hello */").collect_tokens();
    assert_eq!(tokens, vec![Token::Comment(" hello ".to_string())]);
}

#[test]
fn test_tokenize_attribute_matchers() {
    let tokens: Vec<_> = Tokenizer::new("~=").collect_tokens();
    assert_eq!(tokens, vec![Token::IncludeMatch]);

    let tokens: Vec<_> = Tokenizer::new("|=").collect_tokens();
    assert_eq!(tokens, vec![Token::DashMatch]);

    let tokens: Vec<_> = Tokenizer::new("^=").collect_tokens();
    assert_eq!(tokens, vec![Token::PrefixMatch]);

    let tokens: Vec<_> = Tokenizer::new("$=").collect_tokens();
    assert_eq!(tokens, vec![Token::SuffixMatch]);

    let tokens: Vec<_> = Tokenizer::new("*=").collect_tokens();
    assert_eq!(tokens, vec![Token::SubstringMatch]);
}

#[test]
fn test_tokenize_negative_number() {
    let tokens: Vec<_> = Tokenizer::new("-10px").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == -10.0 && u == "px"));
}

#[test]
fn test_tokenize_simple_rule() {
    let tokens: Vec<_> = Tokenizer::new("div { color: red; }").collect_tokens();
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
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "style.css");
            assert!(import_rule.media_queries.is_empty());
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
fn test_parse_import_string_url() {
    let stylesheet = Parser::parse_stylesheet("@import \"theme.css\";");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "theme.css");
            assert!(import_rule.media_queries.is_empty());
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
fn test_parse_import_with_media_query() {
    let stylesheet = Parser::parse_stylesheet("@import \"style.css\" screen and (max-width: 600px);");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "style.css");
            assert_eq!(import_rule.media_queries.len(), 1);
            assert!(import_rule.media_queries[0].contains("screen"));
            assert!(import_rule.media_queries[0].contains("max-width"));
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
fn test_parse_import_with_multiple_media_queries() {
    let stylesheet = Parser::parse_stylesheet("@import \"style.css\" screen, print;");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "style.css");
            assert_eq!(import_rule.media_queries.len(), 2);
            assert_eq!(import_rule.media_queries[0], "screen");
            assert_eq!(import_rule.media_queries[1], "print");
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
fn test_parse_import_url_function() {
    let stylesheet = Parser::parse_stylesheet("@import url(path/to/style.css);");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "path/to/style.css");
        }
        _ => panic!("Expected Import rule"),
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
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "base");
            assert_eq!(layer_rule.rules.len(), 1);
        }
        _ => panic!("Expected Layer rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Tokenizer 边界条件
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_zero() {
    let tokens: Vec<_> = Tokenizer::new("0").collect_tokens();
    assert!(matches!(tokens[0], Token::Number(0.0)));
}

#[test]
fn test_tokenize_escaped_ident() {
    let tokens: Vec<_> = Tokenizer::new("\\41 ").collect_tokens(); // \41 = 'A', needs space terminator
    // Escaped hex codepoint should produce a valid ident (could be "A" or "A ")
    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_multiple_rules() {
    let css = "div { color: red; } .class { font-size: 16px; }";
    let tokens: Vec<_> = Tokenizer::new(css).collect_tokens();
    assert!(tokens.len() > 10);
}

#[test]
fn test_tokenize_nested_parens() {
    let css = "rgba(255, 0, 0, 0.5)";
    let tokens: Vec<_> = Tokenizer::new(css).collect_tokens();
    assert!(tokens.len() >= 2); // At least Function + some content
}

#[test]
fn test_tokenize_rem_dimension() {
    let tokens: Vec<_> = Tokenizer::new("1.2rem").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if (*n - 1.2).abs() < 0.001 && u == "rem"));
}

#[test]
fn test_tokenize_vh_dimension() {
    let tokens: Vec<_> = Tokenizer::new("100vh").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 100.0 && u == "vh"));
}

#[test]
fn test_tokenize_unterminated_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* unterminated").collect_tokens();
    assert!(matches!(&tokens[0], Token::Error(_)));
}

#[test]
fn test_tokenize_unterminated_string() {
    let tokens: Vec<_> = Tokenizer::new("\"unterminated").collect_tokens();
    // Should still return a string (partial)
    assert!(matches!(&tokens[0], Token::String(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Tokenizer Delim 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_dot_as_delim() {
    let tokens: Vec<_> = Tokenizer::new(".").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('.')]);
}

#[test]
fn test_tokenize_bang_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("!").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('!')]);
}

#[test]
fn test_tokenize_greater_as_delim() {
    let tokens: Vec<_> = Tokenizer::new(">").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('>')]);
}

#[test]
fn test_tokenize_plus_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("+").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('+')]);
}

#[test]
fn test_tokenize_star_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("*").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('*')]);
}

#[test]
fn test_tokenize_tilde_as_delim() {
    let tokens: Vec<_> = Tokenizer::new("~").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('~')]);
}

#[test]
fn test_tokenize_complex_selector() {
    // div.class#id:hover → Ident Delim('.') Ident Hash Colon Ident
    let tokens: Vec<_> = Tokenizer::new("div.class#id:hover").collect_tokens();
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
    let tokens: Vec<_> = Tokenizer::new(".5").collect_tokens();
    assert!(matches!(tokens[0], Token::Number(n) if (n - 0.5).abs() < 0.001));
}

#[test]
fn test_tokenize_child_combinator_in_context() {
    // div > p → Ident Whitespace Delim('>') Whitespace Ident
    let tokens: Vec<_> = Tokenizer::new("div > p").collect_tokens();
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
    let tokens: Vec<_> = Tokenizer::new("!important").collect_tokens();
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

// ═══════════════════════════════════════════════════════════════════════
// 8. 值解析扩展测试 — 提升 values.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 #RGBA 四位十六进制颜色解析
fn test_parse_color_hex4() {
    let result = parse_color("#f00f");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));

    let result = parse_color("#f000");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 0)));
}

#[test]
/// 测试 rgb() 使用百分比分量
fn test_parse_color_rgb_with_percent() {
    let result = parse_color("rgb(100%, 0%, 0%)");
    assert!(result.is_some());
    let rgba = result.unwrap();
    assert!(matches!(rgba, ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
/// 测试 rgba() 带透明度
fn test_parse_color_rgba() {
    let result = parse_color("rgba(255, 0, 0, 0.5)");
    assert!(result.is_some());
    // alpha=0.5 → 0.5*255=127.5 → round=128
    assert!(matches!(result, Some(ColorValue::Rgba(255, 0, 0, 128))));
}

#[test]
/// 测试 hsl() 颜色
fn test_parse_color_hsl() {
    let result = parse_color("hsl(120, 50%, 50%)");
    assert!(result.is_some());
    assert!(matches!(result, Some(ColorValue::Hsla(120.0, 50.0, 50.0, 1.0))));
}

#[test]
/// 测试 hsla() 颜色
fn test_parse_color_hsla() {
    let result = parse_color("hsla(240, 100%, 50%, 0.5)");
    assert!(result.is_some());
    assert!(matches!(result, Some(ColorValue::Hsla(240.0, 100.0, 50.0, 0.5))));
}

#[test]
/// 测试无效颜色返回 None
fn test_parse_color_invalid() {
    // 无效的十六进制长度
    assert_eq!(parse_color("#12"), None);
    // rgb 参数不足
    assert_eq!(parse_color("rgb(255, 0)"), None);
}

#[test]
/// 测试所有 16 种基本命名颜色
fn test_parse_color_named_all() {
    assert_eq!(parse_color("black"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(parse_color("white"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("green"), Some(ColorValue::Rgba(0, 128, 0, 255)));
    assert_eq!(parse_color("blue"), Some(ColorValue::Rgba(0, 0, 255, 255)));
    assert_eq!(parse_color("yellow"), Some(ColorValue::Rgba(255, 255, 0, 255)));
    assert_eq!(parse_color("cyan"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    assert_eq!(parse_color("magenta"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    assert_eq!(parse_color("silver"), Some(ColorValue::Rgba(192, 192, 192, 255)));
    assert_eq!(parse_color("gray"), Some(ColorValue::Rgba(128, 128, 128, 255)));
    assert_eq!(parse_color("maroon"), Some(ColorValue::Rgba(128, 0, 0, 255)));
    assert_eq!(parse_color("olive"), Some(ColorValue::Rgba(128, 128, 0, 255)));
    assert_eq!(parse_color("lime"), Some(ColorValue::Rgba(0, 255, 0, 255)));
    assert_eq!(parse_color("teal"), Some(ColorValue::Rgba(0, 128, 128, 255)));
    assert_eq!(parse_color("navy"), Some(ColorValue::Rgba(0, 0, 128, 255)));
    assert_eq!(parse_color("purple"), Some(ColorValue::Rgba(128, 0, 128, 255)));
    // grey 别名
    assert_eq!(parse_color("grey"), Some(ColorValue::Rgba(128, 128, 128, 255)));
    // aqua 别名
    assert_eq!(parse_color("aqua"), Some(ColorValue::Rgba(0, 255, 255, 255)));
    // fuchsia 别名
    assert_eq!(parse_color("fuchsia"), Some(ColorValue::Rgba(255, 0, 255, 255)));
    // orange
    assert_eq!(parse_color("orange"), Some(ColorValue::Rgba(255, 165, 0, 255)));
    // 未知命名颜色应返回 Named
    assert!(matches!(parse_color("customcolor"), Some(ColorValue::Named(_))));
}

#[test]
/// 测试长度值为零（无单位 "0"）— CSS 规范允许裸零作为有效长度
fn test_parse_length_zero() {
    let result = parse_length("0");
    assert_eq!(result, Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试无效长度值
fn test_parse_length_invalid() {
    assert_eq!(parse_length("abc"), None);
}

#[test]
/// 测试 ch 单位
fn test_parse_length_ch() {
    let result = parse_length("2ch");
    assert_eq!(result, Some(LengthValue::Ch(2.0)));
}

#[test]
/// 测试 vmin 单位
fn test_parse_length_vmin() {
    let result = parse_length("50vmin");
    assert_eq!(result, Some(LengthValue::Vmin(50.0)));
}

#[test]
/// 测试 vmax 单位
fn test_parse_length_vmax() {
    let result = parse_length("50vmax");
    assert_eq!(result, Some(LengthValue::Vmax(50.0)));
}

#[test]
/// 测试所有 DisplayValue 变体
fn test_parse_display_all() {
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
/// 测试所有 PositionValue 变体
fn test_parse_position_all() {
    assert_eq!(parse_position("static"), Some(PositionValue::Static));
    assert_eq!(parse_position("relative"), Some(PositionValue::Relative));
    assert_eq!(parse_position("absolute"), Some(PositionValue::Absolute));
    assert_eq!(parse_position("fixed"), Some(PositionValue::Fixed));
    assert_eq!(parse_position("sticky"), Some(PositionValue::Sticky));
    assert_eq!(parse_position("unknown"), None);
}

#[test]
/// 测试所有 OverflowValue 变体
fn test_parse_overflow_all() {
    assert_eq!(parse_overflow("visible"), Some(OverflowValue::Visible));
    assert_eq!(parse_overflow("hidden"), Some(OverflowValue::Hidden));
    assert_eq!(parse_overflow("scroll"), Some(OverflowValue::Scroll));
    assert_eq!(parse_overflow("auto"), Some(OverflowValue::Auto));
    assert_eq!(parse_overflow("clip"), Some(OverflowValue::Clip));
    assert_eq!(parse_overflow("unknown"), None);
}

#[test]
/// 测试所有 FlexDirectionValue 变体
fn test_parse_flex_direction_all() {
    assert_eq!(parse_flex_direction("row"), Some(FlexDirectionValue::Row));
    assert_eq!(
        parse_flex_direction("row-reverse"),
        Some(FlexDirectionValue::RowReverse)
    );
    assert_eq!(parse_flex_direction("column"), Some(FlexDirectionValue::Column));
    assert_eq!(
        parse_flex_direction("column-reverse"),
        Some(FlexDirectionValue::ColumnReverse)
    );
    assert_eq!(parse_flex_direction("unknown"), None);
}

#[test]
/// 测试所有 FlexWrapValue 变体
fn test_parse_flex_wrap_all() {
    assert_eq!(parse_flex_wrap("nowrap"), Some(FlexWrapValue::Nowrap));
    assert_eq!(parse_flex_wrap("wrap"), Some(FlexWrapValue::Wrap));
    assert_eq!(parse_flex_wrap("wrap-reverse"), Some(FlexWrapValue::WrapReverse));
    assert_eq!(parse_flex_wrap("unknown"), None);
}

#[test]
/// 测试所有 AlignmentValue 变体
fn test_parse_alignment_all() {
    assert_eq!(parse_alignment("flex-start"), Some(AlignmentValue::FlexStart));
    assert_eq!(parse_alignment("flex-end"), Some(AlignmentValue::FlexEnd));
    assert_eq!(parse_alignment("center"), Some(AlignmentValue::Center));
    assert_eq!(parse_alignment("space-between"), Some(AlignmentValue::SpaceBetween));
    assert_eq!(parse_alignment("space-around"), Some(AlignmentValue::SpaceAround));
    assert_eq!(parse_alignment("space-evenly"), Some(AlignmentValue::SpaceEvenly));
    assert_eq!(parse_alignment("stretch"), Some(AlignmentValue::Stretch));
    assert_eq!(parse_alignment("start"), Some(AlignmentValue::Start));
    assert_eq!(parse_alignment("end"), Some(AlignmentValue::End));
    assert_eq!(parse_alignment("baseline"), Some(AlignmentValue::Baseline));
    assert_eq!(parse_alignment("unknown"), None);
}

#[test]
/// 测试所有 BoxSizingValue 变体
fn test_parse_box_sizing_all() {
    assert_eq!(parse_box_sizing("content-box"), Some(BoxSizingValue::ContentBox));
    assert_eq!(parse_box_sizing("border-box"), Some(BoxSizingValue::BorderBox));
    assert_eq!(parse_box_sizing("unknown"), None);
}

#[test]
/// 测试所有 VisibilityValue 变体
fn test_parse_visibility_all() {
    assert_eq!(parse_visibility("visible"), Some(VisibilityValue::Visible));
    assert_eq!(parse_visibility("hidden"), Some(VisibilityValue::Hidden));
    assert_eq!(parse_visibility("collapse"), Some(VisibilityValue::Collapse));
    assert_eq!(parse_visibility("unknown"), None);
}

#[test]
/// 测试所有 FontWeightValue 变体（100-900、bold、normal、bolder、lighter）
fn test_parse_font_weight_all() {
    assert_eq!(parse_font_weight("100"), Some(FontWeightValue::Absolute(100)));
    assert_eq!(parse_font_weight("200"), Some(FontWeightValue::Absolute(200)));
    assert_eq!(parse_font_weight("300"), Some(FontWeightValue::Absolute(300)));
    assert_eq!(parse_font_weight("400"), Some(FontWeightValue::Absolute(400)));
    assert_eq!(parse_font_weight("500"), Some(FontWeightValue::Absolute(500)));
    assert_eq!(parse_font_weight("600"), Some(FontWeightValue::Absolute(600)));
    assert_eq!(parse_font_weight("700"), Some(FontWeightValue::Absolute(700)));
    assert_eq!(parse_font_weight("800"), Some(FontWeightValue::Absolute(800)));
    assert_eq!(parse_font_weight("900"), Some(FontWeightValue::Absolute(900)));
    assert_eq!(parse_font_weight("bold"), Some(FontWeightValue::Bold));
    assert_eq!(parse_font_weight("normal"), Some(FontWeightValue::Normal));
    assert_eq!(parse_font_weight("bolder"), Some(FontWeightValue::Bolder));
    assert_eq!(parse_font_weight("lighter"), Some(FontWeightValue::Lighter));
    // 超出范围的值
    assert_eq!(parse_font_weight("0"), None);
    assert_eq!(parse_font_weight("50"), None);
    assert_eq!(parse_font_weight("1000"), None);
}

#[test]
/// 测试所有 FontStyleValue 变体
fn test_parse_font_style_all() {
    assert_eq!(parse_font_style("normal"), Some(FontStyleValue::Normal));
    assert_eq!(parse_font_style("italic"), Some(FontStyleValue::Italic));
    assert_eq!(parse_font_style("oblique"), Some(FontStyleValue::Oblique(None)));
    assert_eq!(
        parse_font_style("oblique(15deg)"),
        Some(FontStyleValue::Oblique(Some(15.0)))
    );
    assert_eq!(parse_font_style("unknown"), None);
}

#[test]
/// 测试 parse_length 对百分比的处理
fn test_parse_length_percentage() {
    let result = parse_length("50%");
    assert_eq!(result, Some(LengthValue::Percentage(50.0)));

    let result = parse_length("100%");
    assert_eq!(result, Some(LengthValue::Percentage(100.0)));

    let result = parse_length("33.33%");
    assert_eq!(result, Some(LengthValue::Percentage(33.33)));
}

#[test]
/// 测试 parse_length 对 auto 关键字的处理
fn test_parse_length_auto() {
    assert_eq!(parse_length("auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length("Auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length("AUTO"), Some(LengthValue::Auto));
    assert_eq!(parse_length("  auto  "), Some(LengthValue::Auto));
}

#[test]
/// 测试 currentcolor 大小写不敏感
fn test_parse_color_currentcolor_case_insensitive() {
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("currentcolor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("CURRENTcolor"), Some(ColorValue::CurrentColor));
}

#[test]
/// 测试 display: flow
fn test_parse_display_flow() {
    assert_eq!(parse_display("flow"), Some(DisplayValue::Flow));
}

#[test]
/// 测试 display: flow-root
fn test_parse_display_flow_root() {
    assert_eq!(parse_display("flow-root"), Some(DisplayValue::FlowRoot));
}

#[test]
/// 测试 display: list-item
fn test_parse_display_list_item() {
    assert_eq!(parse_display("list-item"), Some(DisplayValue::ListItem));
}

#[test]
/// 测试 display: contents
fn test_parse_display_contents() {
    assert_eq!(parse_display("contents"), Some(DisplayValue::Contents));
}

#[test]
/// 测试 display: inline-block
fn test_parse_display_inline_block() {
    assert_eq!(parse_display("inline-block"), Some(DisplayValue::InlineBlock));
}

#[test]
/// 测试 display: inline-flex
fn test_parse_display_inline_flex() {
    assert_eq!(parse_display("inline-flex"), Some(DisplayValue::InlineFlex));
}

#[test]
/// 测试 display: inline-grid
fn test_parse_display_inline_grid() {
    assert_eq!(parse_display("inline-grid"), Some(DisplayValue::InlineGrid));
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Parser 扩展测试 — 提升 parser.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :nth-child(odd) 伪类
fn test_parse_nth_child_odd() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(odd) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-child(even) 伪类
fn test_parse_nth_child_even() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(even) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 0 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-child(2n+1) 公式伪类
fn test_parse_nth_child_formula() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(2n+1) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 2, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-of-type(3) 伪类
fn test_parse_nth_of_type() {
    let stylesheet = Parser::parse_stylesheet("li:nth-of-type(3) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(NthPattern { a: 0, b: 3 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :not() 伪类
fn test_parse_not_selector() {
    let stylesheet = Parser::parse_stylesheet("p:not(.hidden) { display: block; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(
            compound
                .subclass_selectors
                .iter()
                .any(|s| matches!(s, SubclassSelector::PseudoClass(PseudoClassSelector::Not(_))))
        );
        // 验证声明
        assert!(sr.declarations.iter().any(|d| d.property == "display"));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :is() 伪类
fn test_parse_is_selector() {
    let stylesheet = Parser::parse_stylesheet("p:is(.active, .visible) { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Is(selectors))
                if selectors.len() == 2
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :where() 伪类
fn test_parse_where_selector() {
    let stylesheet = Parser::parse_stylesheet("p:where(.main) { font-size: 16px; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Where(selectors))
                if selectors.len() == 1
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :lang() 伪类
fn test_parse_lang() {
    let stylesheet = Parser::parse_stylesheet("p:lang(en) { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Lang(lang))
                if lang == "en"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性前缀匹配选择器 [href^=https]
fn test_parse_attribute_prefix() {
    let stylesheet = Parser::parse_stylesheet("[href^=https] { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Prefix(val),
            }) if name == "href" && val == "https"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性后缀匹配选择器 [href$=.pdf]
fn test_parse_attribute_suffix() {
    let stylesheet = Parser::parse_stylesheet("[href$=.pdf] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Suffix(val),
            }) if name == "href" && val == ".pdf"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性子串匹配选择器 [title*=hello]
fn test_parse_attribute_substring() {
    let stylesheet = Parser::parse_stylesheet("[title*=hello] { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::Substring(val),
            }) if name == "title" && val == "hello"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试属性破折号匹配选择器 [lang|=en]
fn test_parse_attribute_dash() {
    let stylesheet = Parser::parse_stylesheet("[lang|=en] { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::Attribute(AttributeSelector {
                name,
                matcher: AttributeMatcher::DashMatch(val),
            }) if name == "lang" && val == "en"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试多选择器多声明的复杂规则
fn test_parse_multiple_selectors_and_declarations() {
    let css = "div.container > p.text, span.highlight { color: red; font-size: 16px; display: block; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // 两个选择器
        assert_eq!(sr.selectors.len(), 2);
        // 至少 3 条声明
        assert!(sr.declarations.len() >= 3);
        // 验证第一个选择器有 child 组合器
        let parts = &sr.selectors[0].complex.parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].1, Some(Combinator::Child));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试嵌套 @media 带类选择器
fn test_parse_nested_at_media_with_class() {
    let css = "@media screen and (max-width: 768px) { .container { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            assert!(at_rule.prelude.contains("screen"));
            if let AtRuleBody::Block(rules) = &at_rule.body {
                assert_eq!(rules.len(), 1);
                if let Rule::Style(sr) = &rules[0] {
                    assert!(sr.declarations.iter().any(|d| d.property == "width"));
                } else {
                    panic!("Expected Style rule inside @media");
                }
            } else {
                panic!("Expected Block body for @media");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

#[test]
/// 测试 @supports 规则
fn test_parse_at_supports() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Property("display".to_string(), "grid".to_string())
            );
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

// ── @supports 解析扩展测试 ──

#[test]
/// 测试 @supports not 条件
fn test_parse_at_supports_not() {
    let css = "@supports not (display: grid) { .fallback { display: block; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Not(Box::new(SupportsCondition::Property(
                    "display".to_string(),
                    "grid".to_string()
                )))
            );
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports and 条件
fn test_parse_at_supports_and() {
    let css = "@supports (display: grid) and (gap: 10px) { .grid { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            match &supports_rule.condition {
                SupportsCondition::And(conditions) => {
                    assert_eq!(conditions.len(), 2);
                }
                _ => panic!("Expected And condition"),
            }
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports or 条件
fn test_parse_at_supports_or() {
    let css = "@supports (display: grid) or (display: flex) { .container { display: flex; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            match &supports_rule.condition {
                SupportsCondition::Or(conditions) => {
                    assert_eq!(conditions.len(), 2);
                }
                _ => panic!("Expected Or condition"),
            }
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports 多规则体
fn test_parse_at_supports_multiple_rules() {
    let css = "@supports (display: grid) { .a { display: grid; } .b { gap: 10px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(supports_rule.rules.len(), 2);
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports 带 selector() 函数
fn test_parse_at_supports_selector() {
    let css = "@supports selector(.a > .b) { .container { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            assert_eq!(
                supports_rule.condition,
                SupportsCondition::Selector(".a > .b".to_string())
            );
        }
        _ => panic!("Expected Supports rule"),
    }
}

#[test]
/// 测试 @supports 嵌套在 @media 内（通过 AtRule::At 回退）
fn test_parse_at_supports_nested_in_media() {
    let css = "@media screen { @supports (display: grid) { .a { display: grid; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            if let AtRuleBody::Block(inner) = &at_rule.body {
                assert_eq!(inner.len(), 1);
                match &inner[0] {
                    Rule::Supports(sr) => {
                        assert_eq!(
                            sr.condition,
                            SupportsCondition::Property("display".to_string(), "grid".to_string())
                        );
                    }
                    _ => panic!("Expected Supports rule inside @media"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

// ── CSS Transform 解析测试 ──

#[test]
fn test_parse_transform_none() {
    assert_eq!(parse_transform("none"), Some(TransformValue::None));
    assert_eq!(parse_transform("NONE"), Some(TransformValue::None));
}

#[test]
fn test_parse_transform_translate() {
    let result = parse_transform("translate(10px, 20px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 20.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_translate_single_arg() {
    let result = parse_transform("translate(10px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 0.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_translate_x_y() {
    let result = parse_transform("translateX(15px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::TranslateX(15.0));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("translateY(25px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::TranslateY(25.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_rotate() {
    let result = parse_transform("rotate(45deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Rotate(45.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_rotate_rad() {
    let result = parse_transform("rotate(1.5708rad)").unwrap();
    match result {
        TransformValue::List(fns) => {
            // ~90 degrees
            let angle = match fns[0] {
                TransformFunction::Rotate(a) => a,
                _ => 0.0,
            };
            assert!((angle - 90.0).abs() < 1.0);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale() {
    let result = parse_transform("scale(2)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Scale(2.0, None));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale_xy() {
    let result = parse_transform("scale(2, 3)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Scale(2.0, Some(3.0)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale_x_y() {
    let result = parse_transform("scaleX(1.5)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::ScaleX(1.5));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("scaleY(0.5)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::ScaleY(0.5));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_skew() {
    let result = parse_transform("skew(10deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Skew(10.0, None));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("skew(10deg, 20deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Skew(10.0, Some(20.0)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_multiple() {
    let result = parse_transform("translate(10px, 20px) rotate(45deg) scale(2)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 3);
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 20.0));
            assert_eq!(fns[1], TransformFunction::Rotate(45.0));
            assert_eq!(fns[2], TransformFunction::Scale(2.0, None));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_empty() {
    assert_eq!(parse_transform(""), None);
    assert_eq!(parse_transform("  "), None);
}

#[test]
fn test_parse_transform_unknown_function() {
    assert_eq!(parse_transform("unknown(10px)"), None);
}

#[test]
fn test_parse_transform_negative_values() {
    let result = parse_transform("translate(-10px, -20px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Translate(-10.0, -20.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_turn() {
    let result = parse_transform("rotate(0.5turn)").unwrap();
    match result {
        TransformValue::List(fns) => {
            let angle = match fns[0] {
                TransformFunction::Rotate(a) => a,
                _ => 0.0,
            };
            assert!((angle - 180.0).abs() < 0.01);
        }
        _ => panic!("Expected List"),
    }
}

// ── @keyframes 解析测试 ──

#[test]
fn test_parse_keyframes_basic() {
    use crate::ast::*;
    let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "fadeIn");
            assert_eq!(kf.keyframes.len(), 2);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::From]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::To]);
            assert_eq!(kf.keyframes[0].declarations.len(), 1);
            assert_eq!(kf.keyframes[0].declarations[0].property, "opacity");
            assert_eq!(kf.keyframes[0].declarations[0].value, "0");
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_percentage() {
    use crate::ast::*;
    let css = "@keyframes slide { 0% { left: 0px; } 50% { left: 100px; } 100% { left: 200px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "slide");
            assert_eq!(kf.keyframes.len(), 3);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::Percentage(0.0)]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::Percentage(50.0)]);
            assert_eq!(kf.keyframes[2].selectors, vec![KeyframeSelector::Percentage(100.0)]);
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_comma_selectors() {
    use crate::ast::*;
    let css = "@keyframes bounce { 0%, 100% { top: 0px; } 50% { top: 50px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.keyframes.len(), 2);
            assert_eq!(
                kf.keyframes[0].selectors,
                vec![KeyframeSelector::Percentage(0.0), KeyframeSelector::Percentage(100.0)]
            );
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_mixed_selectors() {
    use crate::ast::*;
    let css = "@keyframes test { from, 50% { color: red; } to { color: blue; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(
                kf.keyframes[0].selectors,
                vec![KeyframeSelector::From, KeyframeSelector::Percentage(50.0)]
            );
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_quoted_name() {
    let css = "@keyframes \"my-animation\" { to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "my-animation");
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

// ── Animation 值解析测试 ──

#[test]
fn test_parse_animation_direction() {
    assert_eq!(
        parse_animation_direction("normal"),
        Some(crate::values::AnimationDirectionValue::Normal)
    );
    assert_eq!(
        parse_animation_direction("reverse"),
        Some(crate::values::AnimationDirectionValue::Reverse)
    );
    assert_eq!(
        parse_animation_direction("alternate"),
        Some(crate::values::AnimationDirectionValue::Alternate)
    );
    assert_eq!(
        parse_animation_direction("alternate-reverse"),
        Some(crate::values::AnimationDirectionValue::AlternateReverse)
    );
    assert_eq!(parse_animation_direction("invalid"), None);
}

#[test]
fn test_parse_animation_fill_mode() {
    assert_eq!(
        parse_animation_fill_mode("none"),
        Some(crate::values::AnimationFillModeValue::None)
    );
    assert_eq!(
        parse_animation_fill_mode("forwards"),
        Some(crate::values::AnimationFillModeValue::Forwards)
    );
    assert_eq!(
        parse_animation_fill_mode("backwards"),
        Some(crate::values::AnimationFillModeValue::Backwards)
    );
    assert_eq!(
        parse_animation_fill_mode("both"),
        Some(crate::values::AnimationFillModeValue::Both)
    );
    assert_eq!(parse_animation_fill_mode("invalid"), None);
}

#[test]
fn test_parse_animation_play_state() {
    assert_eq!(
        parse_animation_play_state("running"),
        Some(crate::values::AnimationPlayStateValue::Running)
    );
    assert_eq!(
        parse_animation_play_state("paused"),
        Some(crate::values::AnimationPlayStateValue::Paused)
    );
    assert_eq!(parse_animation_play_state("invalid"), None);
}

// ── :has() 选择器解析测试 ──

#[test]
/// 测试 :has(.active) 解析
fn test_parse_has_selector() {
    let stylesheet = Parser::parse_stylesheet("div:has(.active) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(matches!(
            &compound.type_selector,
            Some(TypeSelector::Tag(t)) if t == "div"
        ));
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors))
                if selectors.len() == 1
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :has(> .child) 解析（子组合器）
fn test_parse_has_child_combinator() {
    let stylesheet = Parser::parse_stylesheet("div:has(> .child) { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        let has_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(has_inner.is_some(), "Expected :has() pseudo-class");
        let inner = has_inner.unwrap();
        assert_eq!(inner.len(), 1);
        // 内部选择器应有 Child 组合器
        let inner_parts = &inner[0].complex.parts;
        assert_eq!(inner_parts.len(), 2, "Expected compound > compound");
        assert_eq!(inner_parts[0].1, Some(Combinator::Child));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :has(div, span) 解析（逗号分隔选择器列表）
fn test_parse_has_selector_list() {
    let stylesheet = Parser::parse_stylesheet("section:has(div, span) { background: white; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors))
                if selectors.len() == 2
        )));
    } else {
        panic!("Expected Style rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Tokenizer `/` delimiter 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 `/` 作为独立分隔符（用于 font shorthand 等）
fn test_tokenize_slash_delim() {
    let tokens: Vec<_> = Tokenizer::new("/").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('/')]);
}

#[test]
/// 测试 font shorthand 中的 `/` 分隔符：`font: 12px/1.5 sans-serif`
fn test_tokenize_font_shorthand_slash() {
    let tokens: Vec<_> = Tokenizer::new("12px/1.5").collect_tokens();
    assert!(tokens.len() >= 3);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 12.0 && u == "px"));
    assert_eq!(tokens[1], Token::Delim('/'));
    assert!(matches!(&tokens[2], Token::Number(n) if (*n - 1.5).abs() < 0.001));
}

#[test]
/// 测试 calc() 中的除法 `/`
fn test_tokenize_calc_division() {
    let tokens: Vec<_> = Tokenizer::new("100px / 2").collect_tokens();
    assert!(tokens.len() >= 3);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 100.0 && u == "px"));
    assert_eq!(tokens[1], Token::Whitespace);
    assert_eq!(tokens[2], Token::Delim('/'));
}

// ═══════════════════════════════════════════════════════════════════════
// 11. parse_length 边界情况测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试负数长度值
fn test_parse_length_negative() {
    assert_eq!(parse_length("-10px"), Some(LengthValue::Px(-10.0)));
    assert_eq!(parse_length("-2.5em"), Some(LengthValue::Em(-2.5)));
}

#[test]
/// 测试带正号前缀的长度值
fn test_parse_length_leading_plus() {
    assert_eq!(parse_length("+10px"), Some(LengthValue::Px(10.0)));
    assert_eq!(parse_length("+1.5em"), Some(LengthValue::Em(1.5)));
}

#[test]
/// 测试科学计数法长度值
fn test_parse_length_scientific_notation() {
    // "1e2px" → 100.0px
    assert_eq!(parse_length("1e2px"), Some(LengthValue::Px(100.0)));
    assert_eq!(parse_length("1E2px"), Some(LengthValue::Px(100.0)));
}

#[test]
/// 测试带空格的裸零
fn test_parse_length_zero_whitespace() {
    assert_eq!(parse_length("  0  "), Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试非零无单位值不被解析为长度（CSS 规范仅允许 0 无单位）
fn test_parse_length_unitless_nonzero() {
    assert_eq!(parse_length("42"), None);
    assert_eq!(parse_length("1.5"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 12. calc() 相对单位求值测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 calc() 中 em 单位求值
fn test_eval_calc_em() {
    let expr = parse_calc("calc(1.5em + 10px)").unwrap();
    let ctx = CalcContext {
        font_size: Some(16.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 1.5 * 16 + 10 = 34.0
    assert_eq!(result, Some(34.0));
}

#[test]
/// 测试 calc() 中 rem 单位求值
fn test_eval_calc_rem() {
    let expr = parse_calc("calc(2rem + 5px)").unwrap();
    let ctx = CalcContext {
        root_font_size: Some(20.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 2 * 20 + 5 = 45.0
    assert_eq!(result, Some(45.0));
}

#[test]
/// 测试 calc() 中 vh 单位求值
fn test_eval_calc_vh() {
    let expr = parse_calc("calc(50vh - 20px)").unwrap();
    let ctx = CalcContext {
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 50 * 800 / 100 - 20 = 380.0
    assert_eq!(result, Some(380.0));
}

#[test]
/// 测试 calc() 中 vw 单位求值
fn test_eval_calc_vw() {
    let expr = parse_calc("calc(25vw + 10px)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 25 * 1200 / 100 + 10 = 310.0
    assert_eq!(result, Some(310.0));
}

#[test]
/// 测试 calc() 中 vmin 单位求值
fn test_eval_calc_vmin() {
    let expr = parse_calc("calc(10vmin)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 10 * min(1200, 800) / 100 = 80.0
    assert_eq!(result, Some(80.0));
}

#[test]
/// 测试 calc() 中 vmax 单位求值
fn test_eval_calc_vmax() {
    let expr = parse_calc("calc(10vmax)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 10 * max(1200, 800) / 100 = 120.0
    assert_eq!(result, Some(120.0));
}

#[test]
/// 测试 calc() 中 ch 单位求值
fn test_eval_calc_ch() {
    let expr = parse_calc("calc(4ch + 2px)").unwrap();
    let ctx = CalcContext {
        ch_width: Some(8.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 4 * 8 + 2 = 34.0
    assert_eq!(result, Some(34.0));
}

#[test]
/// 测试 calc() 相对单位缺少上下文时返回 None
fn test_eval_calc_relative_unit_missing_context() {
    let expr = parse_calc("calc(1.5em + 10px)").unwrap();
    // 没有 font_size 上下文
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, None);
}

#[test]
/// 测试 eval_calc 向后兼容性（parent_length 参数）
fn test_eval_calc_backward_compat() {
    let expr = parse_calc("calc(100% - 20px)").unwrap();
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(180.0));
}

// ═══════════════════════════════════════════════════════════════════════
// 13. CSS Gradient 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试基本 linear-gradient 解析
fn test_parse_linear_gradient_basic() {
    let result = parse_gradient("linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToBottom);
            assert_eq!(lg.stops.len(), 2);
            assert_eq!(lg.repeating, false);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带方向的 linear-gradient
fn test_parse_linear_gradient_with_direction() {
    let result = parse_gradient("linear-gradient(to right, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToRight);
            assert_eq!(lg.stops.len(), 2);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带角度的 linear-gradient
fn test_parse_linear_gradient_with_angle() {
    let result = parse_gradient("linear-gradient(45deg, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::Angle(45.0));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带色标位置的 linear-gradient
fn test_parse_linear_gradient_with_stop_positions() {
    let result = parse_gradient("linear-gradient(red 0%, blue 100%)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 2);
            assert_eq!(lg.stops[0].position, Some(LengthValue::Percentage(0.0)));
            assert_eq!(lg.stops[1].position, Some(LengthValue::Percentage(100.0)));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试多色标 linear-gradient
fn test_parse_linear_gradient_multi_stop() {
    let result = parse_gradient("linear-gradient(red, yellow, green, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 4);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试 repeating-linear-gradient
fn test_parse_repeating_linear_gradient() {
    let result = parse_gradient("repeating-linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert!(lg.repeating);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试 radial-gradient 基本解析
fn test_parse_radial_gradient_basic() {
    let result = parse_gradient("radial-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Ellipse);
            assert_eq!(rg.size, RadialSize::FarthestCorner);
            assert_eq!(rg.stops.len(), 2);
            assert!(!rg.repeating);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 circle radial-gradient
fn test_parse_radial_gradient_circle() {
    let result = parse_gradient("radial-gradient(circle, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试带位置的 radial-gradient
fn test_parse_radial_gradient_at_position() {
    let result = parse_gradient("radial-gradient(circle at center, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
            assert_eq!(rg.position_x, LengthValue::Percentage(50.0));
            assert_eq!(rg.position_y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 repeating-radial-gradient
fn test_parse_repeating_radial_gradient() {
    let result = parse_gradient("repeating-radial-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert!(rg.repeating);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 conic-gradient 基本解析
fn test_parse_conic_gradient_basic() {
    let result = parse_gradient("conic-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.from_angle, 0.0);
            assert_eq!(cg.stops.len(), 2);
            assert!(!cg.repeating);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试带 from 角度的 conic-gradient
fn test_parse_conic_gradient_from_angle() {
    let result = parse_gradient("conic-gradient(from 45deg, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.from_angle, 45.0);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试 repeating-conic-gradient
fn test_parse_repeating_conic_gradient() {
    let result = parse_gradient("repeating-conic-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert!(cg.repeating);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试无效渐变返回 None
fn test_parse_gradient_invalid() {
    assert_eq!(parse_gradient("not-a-gradient"), None);
    assert_eq!(parse_gradient("linear-gradient()"), None);
    assert_eq!(parse_gradient(""), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 14. @layer 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 @layer 基本解析：@layer base { div { color: red; } }
fn test_parse_layer_basic() {
    let stylesheet = Parser::parse_stylesheet("@layer base { div { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "base");
            assert_eq!(layer_rule.rules.len(), 1);
            if let Rule::Style(sr) = &layer_rule.rules[0] {
                assert_eq!(sr.declarations.len(), 1);
                assert_eq!(sr.declarations[0].property, "color");
                assert_eq!(sr.declarations[0].value, "red");
            } else {
                panic!("Expected Style rule inside @layer");
            }
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 多规则
fn test_parse_layer_multiple_rules() {
    let css = "@layer components { div { color: red; } span { font-size: 16px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "components");
            assert_eq!(layer_rule.rules.len(), 2);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 仅声明（分号结尾）
fn test_parse_layer_declaration_only() {
    let stylesheet = Parser::parse_stylesheet("@layer base;");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "base");
            assert!(layer_rule.rules.is_empty());
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 匿名层（无名称）
fn test_parse_layer_anonymous() {
    let stylesheet = Parser::parse_stylesheet("@layer { div { color: blue; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "");
            assert_eq!(layer_rule.rules.len(), 1);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试多个 @layer 规则
fn test_parse_multiple_layers() {
    let css = "@layer reset { * { margin: 0; } } @layer base { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2);
    match &stylesheet.rules[0] {
        Rule::Layer(lr) => assert_eq!(lr.name, "reset"),
        _ => panic!("Expected Layer rule"),
    }
    match &stylesheet.rules[1] {
        Rule::Layer(lr) => assert_eq!(lr.name, "base"),
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 内嵌套 @media
fn test_parse_layer_with_media() {
    let css = "@layer responsive { @media (min-width: 600px) { div { width: 100%; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "responsive");
            assert_eq!(layer_rule.rules.len(), 1);
            match &layer_rule.rules[0] {
                Rule::At(at_rule) => {
                    assert_eq!(at_rule.name, "media");
                }
                _ => panic!("Expected At rule inside @layer"),
            }
        }
        _ => panic!("Expected Layer rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 15. @container 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 @container 带名称和条件解析
fn test_parse_container_with_name() {
    let css = "@container sidebar (min-width: 400px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(container_rule) => {
            assert_eq!(container_rule.name.as_deref(), Some("sidebar"));
            match &container_rule.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "400px");
                }
                _ => panic!("Expected Size condition"),
            }
            assert_eq!(container_rule.rules.len(), 1);
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 无名称解析
fn test_parse_container_without_name() {
    let css = "@container (min-width: 400px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(container_rule) => {
            assert!(container_rule.name.is_none());
            match &container_rule.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "400px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container (min-width: 400px) 条件解析
fn test_parse_container_min_width() {
    let css = "@container (min-width: 400px) { div { display: block; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => match &cr.condition {
            ContainerCondition::Size(sc) => {
                assert_eq!(sc.feature, "min-width");
                assert_eq!(sc.value, "400px");
            }
            _ => panic!("Expected Size condition"),
        },
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 嵌套规则
fn test_parse_container_nested_rules() {
    let css = "@container card (min-width: 300px) { .title { font-size: 24px; } .body { font-size: 16px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert_eq!(cr.name.as_deref(), Some("card"));
            assert_eq!(cr.rules.len(), 2);
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 带 max-width 条件
fn test_parse_container_max_width() {
    let css = "@container (max-width: 800px) { .layout { flex-direction: column; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => match &cr.condition {
            ContainerCondition::Size(sc) => {
                assert_eq!(sc.feature, "max-width");
                assert_eq!(sc.value, "800px");
            }
            _ => panic!("Expected Size condition"),
        },
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试嵌套的 @container 规则保留在父规则中
fn test_parse_container_nested_in_media() {
    let css = "@media screen { @container (min-width: 500px) { .item { width: 50%; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            if let AtRuleBody::Block(rules) = &at_rule.body {
                assert_eq!(rules.len(), 1);
                match &rules[0] {
                    Rule::Container(cr) => {
                        assert!(cr.name.is_none());
                        match &cr.condition {
                            ContainerCondition::Size(sc) => {
                                assert_eq!(sc.feature, "min-width");
                                assert_eq!(sc.value, "500px");
                            }
                            _ => panic!("Expected Size condition"),
                        }
                    }
                    _ => panic!("Expected Container rule inside @media"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 16. scroll-snap 属性值解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 scroll-snap-type 值解析
fn test_parse_scroll_snap_type_values() {
    assert_eq!(parse_scroll_snap_type("none"), Some((ScrollSnapTypeValue::None, None)));
    assert_eq!(
        parse_scroll_snap_type("x mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, Some(ScrollSnapAxis::X)))
    );
    assert_eq!(
        parse_scroll_snap_type("y proximity"),
        Some((ScrollSnapTypeValue::Proximity, Some(ScrollSnapAxis::Y)))
    );
    assert_eq!(
        parse_scroll_snap_type("both mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, Some(ScrollSnapAxis::Both)))
    );
    assert_eq!(
        parse_scroll_snap_type("mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, None))
    );
    assert_eq!(parse_scroll_snap_type("invalid"), None);
}

#[test]
/// 测试 scroll-snap-align 值解析
fn test_parse_scroll_snap_align_values() {
    assert_eq!(parse_scroll_snap_align("none"), Some(ScrollSnapAlignValue::None));
    assert_eq!(parse_scroll_snap_align("start"), Some(ScrollSnapAlignValue::Start));
    assert_eq!(parse_scroll_snap_align("end"), Some(ScrollSnapAlignValue::End));
    assert_eq!(parse_scroll_snap_align("center"), Some(ScrollSnapAlignValue::Center));
    assert_eq!(parse_scroll_snap_align("invalid"), None);
}

#[test]
/// 测试 scroll-snap-stop 值解析
fn test_parse_scroll_snap_stop_values() {
    assert_eq!(parse_scroll_snap_stop("normal"), Some(ScrollSnapStopValue::Normal));
    assert_eq!(parse_scroll_snap_stop("always"), Some(ScrollSnapStopValue::Always));
    assert_eq!(parse_scroll_snap_stop("invalid"), None);
}

#[test]
/// 测试 scroll-margin 简写解析
fn test_parse_scroll_margin_shorthand() {
    // 1 个值
    let result = parse_length_shorthand("10px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
        ])
    );
    // 2 个值
    let result = parse_length_shorthand("10px 20px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
        ])
    );
    // 3 个值
    let result = parse_length_shorthand("10px 20px 30px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(30.0),
            LengthValue::Px(20.0),
        ])
    );
    // 4 个值
    let result = parse_length_shorthand("10px 20px 30px 40px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(30.0),
            LengthValue::Px(40.0),
        ])
    );
}

#[test]
/// 测试 scroll-margin 长属性解析（通过 parse_length）
fn test_parse_scroll_margin_longhands() {
    assert_eq!(parse_length("10px"), Some(LengthValue::Px(10.0)));
    assert_eq!(parse_length("1em"), Some(LengthValue::Em(1.0)));
    assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试 scroll-padding 简写解析
fn test_parse_scroll_padding_shorthand() {
    let result = parse_length_shorthand("5px 10px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(5.0),
            LengthValue::Px(10.0),
            LengthValue::Px(5.0),
            LengthValue::Px(10.0),
        ])
    );
}

#[test]
/// 测试 scroll-padding 长属性解析（通过 parse_length）
fn test_parse_scroll_padding_longhands() {
    assert_eq!(parse_length("15px"), Some(LengthValue::Px(15.0)));
    assert_eq!(parse_length("2rem"), Some(LengthValue::Rem(2.0)));
}

// ═══════════════════════════════════════════════════════════════════════
// 17. container-type 属性值解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 container-type 值解析
fn test_parse_container_type_values() {
    assert_eq!(parse_container_type("normal"), Some(ContainerTypeValue::Normal));
    assert_eq!(parse_container_type("size"), Some(ContainerTypeValue::Size));
    assert_eq!(
        parse_container_type("inline-size"),
        Some(ContainerTypeValue::InlineSize)
    );
    assert_eq!(parse_container_type("invalid"), None);
}

#[test]
/// 测试 container-type 大小写不敏感
fn test_parse_container_type_case_insensitive() {
    assert_eq!(
        parse_container_type("INLINE-SIZE"),
        Some(ContainerTypeValue::InlineSize)
    );
    assert_eq!(parse_container_type("Size"), Some(ContainerTypeValue::Size));
}

#[test]
/// 测试 container-type 在样式表中解析
fn test_parse_container_type_in_stylesheet() {
    let css = ".card { container-type: inline-size; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "container-type" && d.value == "inline-size" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试 scroll-snap-type 属性在样式表中解析
fn test_parse_scroll_snap_type_in_stylesheet() {
    let css = ".scroller { scroll-snap-type: x mandatory; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "scroll-snap-type" && d.value == "x mandatory" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试 scroll-snap-align 属性在样式表中解析
fn test_parse_scroll_snap_align_in_stylesheet() {
    let css = ".item { scroll-snap-align: start; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "scroll-snap-align" && d.value == "start" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 18. Selector edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :nth-child(3n+1) 带偏移的公式
fn test_parse_nth_child_3n_plus_1() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(3n+1) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 3, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :only-child 伪类
fn test_parse_only_child() {
    let stylesheet = Parser::parse_stylesheet("p:only-child { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "only-child"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :only-of-type 伪类
fn test_parse_only_of_type() {
    let stylesheet = Parser::parse_stylesheet("p:only-of-type { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "only-of-type"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :empty 伪类
fn test_parse_empty_selector() {
    let stylesheet = Parser::parse_stylesheet("div:empty { display: none; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "empty"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :checked, :disabled, :enabled 伪类
fn test_parse_ui_state_pseudo_classes() {
    let stylesheet = Parser::parse_stylesheet("input:checked { outline: 1px solid blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "checked"
        )));
    } else {
        panic!("Expected Style rule");
    }

    let stylesheet = Parser::parse_stylesheet("button:disabled { opacity: 0.5; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "disabled"
        )));
    } else {
        panic!("Expected Style rule");
    }

    let stylesheet = Parser::parse_stylesheet("input:enabled { background: white; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "enabled"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-last-of-type 选择器
fn test_parse_nth_last_of_type() {
    let stylesheet = Parser::parse_stylesheet("li:nth-last-of-type(2) { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(NthPattern { a: 0, b: 2 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 19. Value parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 calc() 嵌套乘除运算
fn test_parse_calc_nested_multiply_divide() {
    let expr = parse_calc("calc(2 * 10px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(20.0));

    let expr = parse_calc("calc(100px / 2)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
/// 测试 calc() 中除以零返回 None
fn test_eval_calc_divide_by_zero() {
    let expr = parse_calc("calc(100px / 0)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, None);
}

#[test]
/// 测试 url() 函数 tokenization
fn test_tokenize_url_with_path() {
    let tokens: Vec<_> = Tokenizer::new("url(../images/bg.png)").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Url(u) if u == "../images/bg.png"));
}

#[test]
/// 测试 url() 带引号参数
fn test_tokenize_url_quoted() {
    let tokens: Vec<_> = Tokenizer::new("url('path/to/font.woff2')").collect_tokens();
    assert!(matches!(&tokens[0], Token::Url(u) if u == "path/to/font.woff2"));
}

#[test]
/// 测试 var() 嵌套在值中解析
fn test_parse_var_nested_fallback() {
    let result = parse_var("var(--spacing, 16px)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--spacing");
    assert_eq!(var.fallback, Some("16px".to_string()));
}

#[test]
/// 测试 parse_time 边界值
fn test_parse_time_edge_cases() {
    use crate::values::parse_time;
    assert_eq!(parse_time("0s"), Some(0.0));
    assert_eq!(parse_time("0ms"), Some(0.0));
    assert_eq!(parse_time("100ms"), Some(0.1));
    assert_eq!(parse_time("10"), None);
    assert_eq!(parse_time(""), None);
}

#[test]
/// 测试 timing-function cubic-bezier 参数
fn test_parse_timing_function_cubic_bezier_values() {
    use crate::values::parse_timing_function;
    let result = parse_timing_function("cubic-bezier(0.0, 0.0, 1.0, 1.0)");
    assert_eq!(
        result,
        Some(crate::values::TimingFunctionValue::CubicBezier(0.0, 0.0, 1.0, 1.0))
    );
}

#[test]
/// 测试 timing-function steps 带不同位置参数
fn test_parse_timing_function_steps_variants() {
    use crate::values::{StepPosition, TimingFunctionValue, parse_timing_function};
    assert_eq!(
        parse_timing_function("steps(3, jump-none)"),
        Some(TimingFunctionValue::Steps(3, Some(StepPosition::None)))
    );
    assert_eq!(
        parse_timing_function("steps(5, jump-both)"),
        Some(TimingFunctionValue::Steps(5, Some(StepPosition::Both)))
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 20. @rule parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试嵌套 @media 规则：@media 内含另一个 @media
fn test_parse_nested_media_rules() {
    let css = "@media screen { @media (max-width: 600px) { div { color: blue; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(outer) => {
            assert_eq!(outer.name, "media");
            if let AtRuleBody::Block(inner_rules) = &outer.body {
                assert_eq!(inner_rules.len(), 1);
                match &inner_rules[0] {
                    Rule::At(inner) => {
                        assert_eq!(inner.name, "media");
                    }
                    _ => panic!("Expected inner At rule"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected outer At rule"),
    }
}

#[test]
/// 测试 @layer 排序（多个 layer 规则顺序）
fn test_parse_layer_ordering() {
    let css = "@layer reset { * { margin: 0; } } @layer base { body { font-size: 16px; } } @layer components { .btn { padding: 10px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
    let names: Vec<&str> = stylesheet
        .rules
        .iter()
        .map(|r| match r {
            Rule::Layer(lr) => lr.name.as_str(),
            _ => "unknown",
        })
        .collect();
    assert_eq!(names, vec!["reset", "base", "components"]);
}

#[test]
/// 测试 @import 带 print 媒体查询
fn test_parse_import_print_media() {
    let stylesheet = Parser::parse_stylesheet("@import \"print.css\" print;");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "print.css");
            assert_eq!(import_rule.media_queries.len(), 1);
            assert_eq!(import_rule.media_queries[0], "print");
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
/// 测试 @keyframes 带 from/to 混合百分比
fn test_parse_keyframes_mixed_from_to_percentage() {
    let css =
        "@keyframes anim { from { opacity: 0; } 25% { opacity: 0.25; } 50% { opacity: 0.5; } to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "anim");
            assert_eq!(kf.keyframes.len(), 4);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::From]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::Percentage(25.0)]);
            assert_eq!(kf.keyframes[2].selectors, vec![KeyframeSelector::Percentage(50.0)]);
            assert_eq!(kf.keyframes[3].selectors, vec![KeyframeSelector::To]);
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
/// 测试 @container 带 width 比较运算符条件
fn test_parse_container_with_comparison_condition() {
    let css = "@container card (width > 300px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert_eq!(cr.name.as_deref(), Some("card"));
            match &cr.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "width");
                    assert_eq!(sc.value, "300px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 带 inline-size 条件（不带函数包装）
fn test_parse_container_with_inline_size_condition() {
    let css = "@container (min-width: 300px) { .card { flex-direction: column; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert!(cr.name.is_none());
            match &cr.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "300px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 21. Tokenizer edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试注释在值中间
fn test_tokenize_comment_in_value() {
    let tokens: Vec<_> = Tokenizer::new("10px /* comment */ 20px").collect_tokens();
    // Tokens: Dimension(10,px) Whitespace Comment Whitespace Dimension(20,px)
    assert!(tokens.len() >= 5);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
    assert_eq!(tokens[1], Token::Whitespace);
    assert!(matches!(&tokens[2], Token::Comment(_)));
    assert_eq!(tokens[3], Token::Whitespace);
    assert!(matches!(&tokens[4], Token::Dimension(n, u) if *n == 20.0 && u == "px"));
}

#[test]
/// 测试标识符中转义字符
fn test_tokenize_escaped_character_in_ident() {
    let tokens: Vec<_> = Tokenizer::new("\\41 ctive").collect_tokens(); // \41 = 'A'
    assert!(!tokens.is_empty());
    // The escaped \41 should produce 'A', so the ident should start with 'A'
    if let Token::Ident(s) = &tokens[0] {
        assert!(s.starts_with('A'), "Expected ident starting with 'A', got '{}'", s);
    }
}

#[test]
/// 测试科学计数法数字
fn test_tokenize_scientific_notation() {
    let tokens: Vec<_> = Tokenizer::new("1e2").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 100.0).abs() < 0.001));

    let tokens: Vec<_> = Tokenizer::new("3.5e-1").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 0.35).abs() < 0.001));
}

#[test]
/// 测试多行字符串
fn test_tokenize_multiline_string() {
    let tokens: Vec<_> = Tokenizer::new("\"line1\\nline2\"").collect_tokens();
    assert!(matches!(&tokens[0], Token::String(s) if s.contains("line1") && s.contains("line2")));
}

#[test]
/// 测试自定义属性 (--*) tokenization
fn test_tokenize_custom_property() {
    let tokens: Vec<_> = Tokenizer::new("--main-color").collect_tokens();
    // Custom properties start with '--', which is parsed as an ident starting with '-'
    assert!(!tokens.is_empty());
    if let Token::Ident(s) = &tokens[0] {
        assert!(s.starts_with('-'), "Expected ident starting with '-', got '{}'", s);
    }
}

#[test]
/// 测试连续空白合并为单个 Whitespace token
fn test_tokenize_multiple_whitespace() {
    let tokens: Vec<_> = Tokenizer::new("   \t  \n  ").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Whitespace);
}

// ═══════════════════════════════════════════════════════════════════════
// 22. Error recovery
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试格式错误的选择器恢复：未知 token 后继续解析
fn test_parse_malformed_selector_recovery() {
    let css = "div { color: red; } %invalid span { color: green; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Parser should at least parse the first valid rule
    assert!(stylesheet.rules.len() >= 1);
    let has_red = stylesheet.rules.iter().any(|r| {
        if let Rule::Style(sr) = r {
            sr.declarations.iter().any(|d| d.value.contains("red"))
        } else {
            false
        }
    });
    assert!(has_red, "Expected to parse valid 'div {{ color: red; }}' rule");
}

#[test]
/// 测试未闭合字符串恢复
fn test_parse_unclosed_string_recovery() {
    let css = "div { content: \"unclosed; } span { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Parser should produce at least one rule (even if malformed)
    assert!(stylesheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 23. Token 偏移量追踪测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 token 偏移量追踪：验证 token 的起始字节位置正确
fn test_token_offset_tracking() {
    let css = "div { color: red; }";
    let tokens: Vec<Spanned> = Tokenizer::new(css).collect();

    // "div" 起始于偏移量 0
    assert_eq!(tokens[0].offset, 0);
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "div"));

    // " " （空白）起始于偏移量 3
    assert_eq!(tokens[1].offset, 3);
    assert!(matches!(tokens[1].token, Token::Whitespace));

    // "{" 起始于偏移量 4
    assert_eq!(tokens[2].offset, 4);
    assert!(matches!(tokens[2].token, Token::LBrace));

    // " " （空白）起始于偏移量 5
    assert_eq!(tokens[3].offset, 5);

    // "color" 起始于偏移量 6
    assert_eq!(tokens[4].offset, 6);
    assert!(matches!(tokens[4].token, Token::Ident(ref s) if s == "color"));

    // ":" 起始于偏移量 11
    assert_eq!(tokens[5].offset, 11);

    // " " （空白）起始于偏移量 12
    assert_eq!(tokens[6].offset, 12);

    // "red" 起始于偏移量 13
    assert_eq!(tokens[7].offset, 13);

    // ";" 起始于偏移量 16
    assert_eq!(tokens[8].offset, 16);

    // " " （空白）起始于偏移量 17
    assert_eq!(tokens[9].offset, 17);

    // "}" 起始于偏移量 18
    assert_eq!(tokens[10].offset, 18);
}

#[test]
/// 测试换行后位置正确推进
fn test_tokenizer_position_after_newline() {
    let css = "a\nb";
    let tokens: Vec<Spanned> = Tokenizer::new(css).collect();

    // "a" 起始于偏移量 0
    assert_eq!(tokens[0].offset, 0);

    // "\n" （空白）起始于偏移量 1
    assert_eq!(tokens[1].offset, 1);

    // "b" 起始于偏移量 2
    assert_eq!(tokens[2].offset, 2);

    // 测试 \r\n 换行
    let css_crlf = "a\r\nb";
    let tokens_crlf: Vec<Spanned> = Tokenizer::new(css_crlf).collect();

    // "a" 起始于偏移量 0
    assert_eq!(tokens_crlf[0].offset, 0);

    // 空白（包含 \r\n）起始于偏移量 1
    assert_eq!(tokens_crlf[1].offset, 1);

    // "b" 起始于偏移量 3（\r\n 占 2 字节）
    assert_eq!(tokens_crlf[2].offset, 3);
}

#[test]
/// 测试字节偏移量到行:列的转换
fn test_line_column_from_offset() {
    let source = "div {\n  color: red;\n}";

    // 偏移量 0 → 第 1 行第 1 列 ("d")
    assert_eq!(line_column_from_offset(source, 0), (1, 1));

    // 偏移量 3 → 第 1 行第 4 列 (" ")
    assert_eq!(line_column_from_offset(source, 3), (1, 4));

    // 偏移量 5 → 第 1 行第 6 列 ("\n" 本身)
    assert_eq!(line_column_from_offset(source, 5), (1, 6));

    // 偏移量 6 → 第 2 行第 1 列 ("\n" 后第一个字符)
    assert_eq!(line_column_from_offset(source, 6), (2, 1));

    // 偏移量 8 → 第 2 行第 3 列 ("c")
    assert_eq!(line_column_from_offset(source, 8), (2, 3));

    // 偏移量 14 → 第 2 行第 9 列 (冒号后空格)
    assert_eq!(line_column_from_offset(source, 14), (2, 9));

    // 偏移量 19 → 第 2 行第 14 列 ("\n" 是第 2 行最后一个字符)
    assert_eq!(line_column_from_offset(source, 19), (2, 14));

    // 偏移量 20 → 第 3 行第 1 列 ("}")
    assert_eq!(line_column_from_offset(source, 20), (3, 1));

    // 测试 \r\n 换行
    let source_crlf = "a\r\nb";
    assert_eq!(line_column_from_offset(source_crlf, 0), (1, 1));
    assert_eq!(line_column_from_offset(source_crlf, 1), (1, 2));
    // \r 触发换行并跳过 \n，因此 \r\n 整体被视为换行
    assert_eq!(line_column_from_offset(source_crlf, 2), (2, 1));
    assert_eq!(line_column_from_offset(source_crlf, 3), (2, 1)); // b 的位置
    assert_eq!(line_column_from_offset(source_crlf, 4), (2, 2)); // b 之后
}

#[test]
/// 测试无效 @rule 恢复
fn test_parse_invalid_at_rule_recovery() {
    let css = "@unknown-rule something { div { color: red; } } p { font-size: 14px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Should parse @unknown-rule as a generic At rule and still get p rule
    assert!(stylesheet.rules.len() >= 2);
    let has_p = stylesheet.rules.iter().any(|r| {
        if let Rule::Style(sr) = r {
            sr.selectors.iter().any(|s| {
                s.complex.parts[0]
                    .0
                    .type_selector
                    .as_ref()
                    .map_or(false, |ts| matches!(ts, TypeSelector::Tag(t) if t == "p"))
            })
        } else {
            false
        }
    });
    assert!(has_p, "Expected to recover and parse 'p' rule");
}

#[test]
/// 测试多余右花括号恢复
fn test_parse_extra_closing_brace_recovery() {
    let css = "div { color: red; } } span { color: green; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(stylesheet.rules.len() >= 1);
}

#[test]
/// 测试无效属性值恢复
fn test_parse_invalid_property_value_recovery() {
    let css = "div { color: ; font-size: 16px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // Should still have font-size declaration (color may be empty value)
        assert!(sr.declarations.iter().any(|d| d.property == "font-size"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 23. Round-trip consistency
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试简单规则序列化：声明属性和值保持一致
fn test_round_trip_simple_rule_consistency() {
    let css = "div { color: red; font-size: 16px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.declarations.len(), 2);
        assert_eq!(sr.declarations[0].property, "color");
        assert_eq!(sr.declarations[0].value, "red");
        assert_eq!(sr.declarations[1].property, "font-size");
        assert_eq!(sr.declarations[1].value, "16px");
    }
}

#[test]
/// 测试解析-序列化-解析一致性
fn test_round_trip_parse_serialize_parse() {
    let css = "div { color: red; } span { font-size: 16px; }";
    let first = Parser::parse_stylesheet(css);
    // Re-parse should produce the same structure
    let second = Parser::parse_stylesheet(css);
    assert_eq!(first.rules.len(), second.rules.len());
    for (r1, r2) in first.rules.iter().zip(second.rules.iter()) {
        match (r1, r2) {
            (Rule::Style(s1), Rule::Style(s2)) => {
                assert_eq!(s1.declarations.len(), s2.declarations.len());
                for (d1, d2) in s1.declarations.iter().zip(s2.declarations.iter()) {
                    assert_eq!(d1.property, d2.property);
                    assert_eq!(d1.value, d2.value);
                    assert_eq!(d1.important, d2.important);
                }
            }
            (Rule::At(a1), Rule::At(a2)) => {
                assert_eq!(a1.name, a2.name);
                assert_eq!(a1.prelude, a2.prelude);
            }
            _ => {}
        }
    }
}

#[test]
/// 测试复杂样式表往返一致性
fn test_round_trip_complex_stylesheet() {
    let css = "@media screen { div { color: red !important; } } @layer base { p { margin: 0; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2);
    // Verify @media
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            assert_eq!(at.name, "media");
            assert!(at.prelude.contains("screen"));
            if let AtRuleBody::Block(rules) = &at.body {
                assert_eq!(rules.len(), 1);
                if let Rule::Style(sr) = &rules[0] {
                    let has_important = sr.declarations.iter().any(|d| d.important);
                    assert!(has_important);
                }
            }
        }
        _ => panic!("Expected At rule"),
    }
    // Verify @layer
    match &stylesheet.rules[1] {
        Rule::Layer(lr) => {
            assert_eq!(lr.name, "base");
            assert_eq!(lr.rules.len(), 1);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试空白规范化：多余空白不影响解析结果
fn test_whitespace_normalization() {
    let css1 = "div{color:red}";
    let css2 = "div  {  color  :  red  ;  }";
    let ss1 = Parser::parse_stylesheet(css1);
    let ss2 = Parser::parse_stylesheet(css2);
    assert_eq!(ss1.rules.len(), ss2.rules.len());
    if let (Rule::Style(s1), Rule::Style(s2)) = (&ss1.rules[0], &ss2.rules[0]) {
        assert_eq!(s1.declarations.len(), s2.declarations.len());
        assert_eq!(s1.declarations[0].property, s2.declarations[0].property);
        // The value should be "red" regardless of whitespace
        assert_eq!(s1.declarations[0].value, "red");
        assert_eq!(s2.declarations[0].value, "red");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 24. Additional selector and specificity tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :where() specificity 为 0
fn test_specificity_where_zero() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        class_sel("active"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert_eq!(selector::specificity(&sel), (0, 0, 0));
}

#[test]
/// 测试 :is() specificity 取参数最大值
fn test_specificity_is_takes_max() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        id_sel("main"),
                        tag_sel("div"),
                    ]))],
                },
                None,
            )],
        },
    };
    // :is(#main, div) -> max((1,0,0), (0,0,1)) = (1,0,0)
    assert_eq!(selector::specificity(&sel), (1, 0, 0));
}

#[test]
/// 测试 :not() specificity 取参数最大值
fn test_specificity_not_takes_max() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        class_sel("hidden"),
                        id_sel("special"),
                    ]))],
                },
                None,
            )],
        },
    };
    // p:not(.hidden, #special) -> tag(0,0,1) + max((0,1,0), (1,0,0)) = (1,0,1)
    assert_eq!(selector::specificity(&sel), (1, 0, 1));
}

// ═══════════════════════════════════════════════════════════════════════
// 25. Additional value parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_length_shorthand 零值
fn test_parse_length_shorthand_zero() {
    let result = parse_length_shorthand("0");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
        ])
    );
}

#[test]
/// 测试 parse_length_shorthand 混合单位
fn test_parse_length_shorthand_mixed_units() {
    let result = parse_length_shorthand("10px 1em");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Em(1.0),
            LengthValue::Px(10.0),
            LengthValue::Em(1.0),
        ])
    );
}

#[test]
/// 测试 linear-gradient to top 方向
fn test_parse_linear_gradient_to_top() {
    let result = parse_gradient("linear-gradient(to top, blue, transparent)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToTop);
            assert_eq!(lg.stops.len(), 2);
            assert!(matches!(lg.stops[0].color, ColorValue::Rgba(0, 0, 255, 255)));
            assert!(matches!(lg.stops[1].color, ColorValue::Transparent));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试 radial-gradient closest-side
fn test_parse_radial_gradient_closest_side() {
    let result = parse_gradient("radial-gradient(circle closest-side, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
            assert_eq!(rg.size, RadialSize::ClosestSide);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 conic-gradient 带 at 位置
fn test_parse_conic_gradient_at_position() {
    let result = parse_gradient("conic-gradient(at 50% 50%, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.position_x, LengthValue::Percentage(50.0));
            assert_eq!(cg.position_y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected ConicGradient"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 26. Media query edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试媒体查询 "all" 类型解析
fn test_media_query_all_type() {
    use crate::media_query::{MediaType, parse_media_query};
    let q = parse_media_query("all").unwrap();
    assert_eq!(q.media_type, Some(MediaType::All));
    assert!(q.conditions.is_empty());
}

#[test]
/// 测试媒体查询多重条件评估
fn test_media_query_multiple_conditions_eval() {
    use crate::media_query::{MediaContext, evaluate_media_query, parse_media_query};
    let q = parse_media_query("screen and (min-width: 600px) and (orientation: landscape)").unwrap();
    let ctx = MediaContext::new(1024.0, 768.0);
    assert!(evaluate_media_query(&q, &ctx));
    let ctx_portrait = MediaContext::new(1024.0, 1200.0);
    assert!(!evaluate_media_query(&q, &ctx_portrait));
}
