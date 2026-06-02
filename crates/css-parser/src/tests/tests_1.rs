use super::*;

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
    // 未知命名颜色应返回 None（非标准名称无法解析）
    assert_eq!(parse_color("customcolor"), None);
}

#[test]
/// 测试扩展命名颜色（coral、darkred、tomato、crimson 等 CSS 标准颜色）
fn test_parse_color_extended_named() {
    // coral
    assert_eq!(parse_color("coral"), Some(ColorValue::Rgba(255, 127, 80, 255)));
    // darkred
    assert_eq!(parse_color("darkred"), Some(ColorValue::Rgba(139, 0, 0, 255)));
    // tomato — 之前会返回 Named(String)，现在正确返回 Rgba
    assert_eq!(parse_color("tomato"), Some(ColorValue::Rgba(255, 99, 71, 255)));
    // crimson — 验证大小写不敏感
    assert_eq!(parse_color("Crimson"), Some(ColorValue::Rgba(220, 20, 60, 255)));
    assert_eq!(parse_color("CRIMSON"), Some(ColorValue::Rgba(220, 20, 60, 255)));
    // 更多扩展颜色抽样
    assert_eq!(
        parse_color("cornflowerblue"),
        Some(ColorValue::Rgba(100, 149, 237, 255))
    );
    assert_eq!(parse_color("dodgerblue"), Some(ColorValue::Rgba(30, 144, 255, 255)));
    assert_eq!(parse_color("steelblue"), Some(ColorValue::Rgba(70, 130, 180, 255)));
    assert_eq!(parse_color("chartreuse"), Some(ColorValue::Rgba(127, 255, 0, 255)));
    // darkgray 和 darkgrey 别名
    assert_eq!(parse_color("darkgray"), Some(ColorValue::Rgba(169, 169, 169, 255)));
    assert_eq!(parse_color("darkgrey"), Some(ColorValue::Rgba(169, 169, 169, 255)));
    // transparent 和 currentcolor
    assert_eq!(parse_color("transparent"), Some(ColorValue::Transparent));
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("TRANSPARENT"), Some(ColorValue::Transparent));
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
/// 测试 fit-content() CSS 函数解析
fn test_parse_fit_content() {
    // fit-content(200px)
    let result = parse_length("fit-content(200px)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Px(200.0)));

    // fit-content(50%)
    let result = parse_length("fit-content(50%)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Percentage(50.0)));

    // fit-content(0)
    let result = parse_length("fit-content(0)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Px(0.0)));

    // fit-content() 空参数应返回 None
    assert_eq!(parse_length("fit-content()"), None);

    // fit-content(10em)
    let result = parse_length("fit-content(10em)");
    assert!(matches!(result, Some(LengthValue::FitContent(inner)) if *inner == LengthValue::Em(10.0)));

    // 大小写不敏感
    let result = parse_length("FIT-CONTENT(100px)");
    assert!(result.is_none()); // starts_with 是大小写敏感的，当前实现要求小写
}

#[test]
/// 测试 min-content/max-content 关键字解析
fn test_parse_min_max_content() {
    // min-content
    assert_eq!(parse_length("min-content"), Some(LengthValue::MinContent));
    assert_eq!(parse_length("MIN-CONTENT"), Some(LengthValue::MinContent));
    assert_eq!(parse_length("Min-Content"), Some(LengthValue::MinContent));

    // max-content
    assert_eq!(parse_length("max-content"), Some(LengthValue::MaxContent));
    assert_eq!(parse_length("MAX-CONTENT"), Some(LengthValue::MaxContent));
    assert_eq!(parse_length("Max-Content"), Some(LengthValue::MaxContent));

    // 不是关键字
    assert_eq!(parse_length("content"), None);
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

// ═══════════════════════════════════════════════════════════════════════
// 33. CSS 类型值解析测试（覆盖 types.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 LengthValue 构造函数
fn test_length_value_constructors() {
    let test_cases = vec![
        (LengthValue::Px(10.0), "10px"),
        (LengthValue::Em(2.5), "2.5em"),
        (LengthValue::Rem(1.0), "1rem"),
        (LengthValue::Vh(100.0), "100vh"),
        (LengthValue::Vw(50.0), "50vw"),
        (LengthValue::Vmin(20.0), "20vmin"),
        (LengthValue::Vmax(80.0), "80vmax"),
        (LengthValue::Ch(16.0), "16ch"),
        (LengthValue::Percentage(50.0), "50%"),
        (LengthValue::Auto, "auto"),
        (LengthValue::MinContent, "min-content"),
        (LengthValue::MaxContent, "max-content"),
        (
            LengthValue::FitContent(Box::new(LengthValue::Px(100.0))),
            "fit-content(100px)",
        ),
    ];

    for (length_value, _expected_str) in test_cases {
        // 这里只是测试构造函数，不测试解析
        let _ = length_value;
    }
}

#[test]
/// 测试 LengthValue 的相等性比较
fn test_length_value_equality() {
    let test_cases = vec![
        (LengthValue::Px(10.0), LengthValue::Px(10.0), true),
        (LengthValue::Px(10.0), LengthValue::Px(20.0), false),
        (LengthValue::Em(1.0), LengthValue::Em(1.0), true),
        (LengthValue::Em(1.0), LengthValue::Px(1.0), false),
        (LengthValue::Auto, LengthValue::Auto, true),
        (LengthValue::MinContent, LengthValue::MinContent, true),
        (LengthValue::MaxContent, LengthValue::MaxContent, true),
        (LengthValue::Percentage(50.0), LengthValue::Percentage(50.0), true),
        (LengthValue::Percentage(50.0), LengthValue::Percentage(100.0), false),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 ColorValue 变体
fn test_color_value_variants() {
    let test_cases = vec![
        (ColorValue::Rgba(255, 0, 0, 255), "rgba(255, 0, 0, 255)"),
        (ColorValue::Rgba(0, 0, 255, 128), "rgba(0, 0, 255, 128)"),
        (ColorValue::Hsla(0.0, 100.0, 50.0, 1.0), "hsla(0, 100%, 50%, 1)"),
        (ColorValue::Hsla(120.0, 100.0, 50.0, 0.5), "hsla(120, 100%, 50%, 0.5)"),
        (ColorValue::Named("red".to_string()), "red"),
        (ColorValue::Named("blue".to_string()), "blue"),
        (ColorValue::Transparent, "transparent"),
        (ColorValue::CurrentColor, "currentColor"),
    ];

    for (color_value, _) in test_cases {
        // 测试 Debug 格式化
        let _ = format!("{:?}", color_value);

        // 测试 Clone
        let cloned = color_value.clone();
        assert_eq!(color_value, cloned);
    }
}

#[test]
/// 测试 DisplayValue 枚举
fn test_display_value_equality() {
    let test_cases = vec![
        (DisplayValue::Block, DisplayValue::Block, true),
        (DisplayValue::Inline, DisplayValue::Inline, true),
        (DisplayValue::InlineBlock, DisplayValue::InlineBlock, true),
        (DisplayValue::Flex, DisplayValue::Flex, true),
        (DisplayValue::InlineFlex, DisplayValue::InlineFlex, true),
        (DisplayValue::Grid, DisplayValue::Grid, true),
        (DisplayValue::InlineGrid, DisplayValue::InlineGrid, true),
        (DisplayValue::None, DisplayValue::None, true),
        (DisplayValue::Contents, DisplayValue::Contents, true),
        (DisplayValue::Flow, DisplayValue::Flow, true),
        (DisplayValue::FlowRoot, DisplayValue::FlowRoot, true),
        (DisplayValue::ListItem, DisplayValue::ListItem, true),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 FloatValue 和 ClearValue 枚举
fn test_float_and_clear_values() {
    let float_test_cases = vec![
        (FloatValue::None, "none"),
        (FloatValue::Left, "left"),
        (FloatValue::Right, "right"),
        (FloatValue::InlineStart, "inline-start"),
        (FloatValue::InlineEnd, "inline-end"),
    ];

    let clear_test_cases = vec![
        (ClearValue::None, "none"),
        (ClearValue::Left, "left"),
        (ClearValue::Right, "right"),
        (ClearValue::Both, "both"),
        (ClearValue::InlineStart, "inline-start"),
        (ClearValue::InlineEnd, "inline-end"),
    ];

    for (float_value, _) in float_test_cases {
        let _ = format!("{:?}", float_value);
        let _ = float_value.clone();
    }

    for (clear_value, _) in clear_test_cases {
        let _ = format!("{:?}", clear_value);
        let _ = clear_value.clone();
    }
}

#[test]
/// 测试 PositionValue 枚举
fn test_position_value() {
    let test_cases = vec![
        (PositionValue::Static, "static"),
        (PositionValue::Relative, "relative"),
        (PositionValue::Absolute, "absolute"),
        (PositionValue::Fixed, "fixed"),
        (PositionValue::Sticky, "sticky"),
    ];

    for (position_value, _) in test_cases {
        let _ = format!("{:?}", position_value);
        let _ = position_value.clone();
    }
}

#[test]
/// 测试 OverflowValue 枚举
fn test_overflow_value() {
    let test_cases = vec![
        (OverflowValue::Visible, "visible"),
        (OverflowValue::Hidden, "hidden"),
        (OverflowValue::Scroll, "scroll"),
        (OverflowValue::Auto, "auto"),
        (OverflowValue::Clip, "clip"),
    ];

    for (overflow_value, _) in test_cases {
        let _ = format!("{:?}", overflow_value);
        let _ = overflow_value.clone();
    }
}

#[test]
/// 测试 ListStyleTypeValue 枚举
fn test_list_style_type_value() {
    let test_cases = vec![
        (ListStyleTypeValue::Disc, "disc"),
        (ListStyleTypeValue::Circle, "circle"),
        (ListStyleTypeValue::Square, "square"),
        (ListStyleTypeValue::Decimal, "decimal"),
        (ListStyleTypeValue::DecimalLeadingZero, "decimal-leading-zero"),
        (ListStyleTypeValue::LowerRoman, "lower-roman"),
        (ListStyleTypeValue::UpperRoman, "upper-roman"),
        (ListStyleTypeValue::LowerAlpha, "lower-alpha"),
        (ListStyleTypeValue::UpperAlpha, "upper-alpha"),
        (ListStyleTypeValue::None, "none"),
    ];

    for (list_style_type, _) in test_cases {
        let _ = format!("{:?}", list_style_type);
        let _ = list_style_type.clone();
    }
}

#[test]
/// 测试 ListStylePositionValue 枚举
fn test_list_style_position_value() {
    let test_cases = vec![
        (ListStylePositionValue::Outside, "outside"),
        (ListStylePositionValue::Inside, "inside"),
    ];

    for (position_value, _) in test_cases {
        let _ = format!("{:?}", position_value);
        let _ = position_value.clone();
    }
}

#[test]
/// 测试 FlexDirectionValue 枚举
fn test_flex_direction_value() {
    let test_cases = vec![
        (FlexDirectionValue::Row, "row"),
        (FlexDirectionValue::RowReverse, "row-reverse"),
        (FlexDirectionValue::Column, "column"),
        (FlexDirectionValue::ColumnReverse, "column-reverse"),
    ];

    for (flex_direction, _) in test_cases {
        let _ = format!("{:?}", flex_direction);
        let _ = flex_direction.clone();
    }
}

#[test]
/// 测试 FlexWrapValue 枚举
fn test_flex_wrap_value() {
    let test_cases = vec![(FlexWrapValue::Nowrap, "nowrap"), (FlexWrapValue::Wrap, "wrap")];

    for (flex_wrap, _) in test_cases {
        let _ = format!("{:?}", flex_wrap);
        let _ = flex_wrap.clone();
    }
}

#[test]
/// 测试所有 CSS 类型值的 Clone 实现
fn test_all_css_values_clone() {
    // 这里测试各种类型值的 Clone 是否正常工作
    let _ = LengthValue::Px(10.0).clone();
    let _ = ColorValue::Rgba(255, 0, 0, 255).clone();
    let _ = DisplayValue::Block.clone();
    let _ = FloatValue::None.clone();
    let _ = ClearValue::None.clone();
    let _ = PositionValue::Static.clone();
    let _ = OverflowValue::Visible.clone();
    let _ = ListStyleTypeValue::Disc.clone();
    let _ = ListStylePositionValue::Outside.clone();
    let _ = FlexDirectionValue::Row.clone();
    let _ = FlexWrapValue::Nowrap.clone();

    // 测试嵌套类型的 Clone
    let _ = LengthValue::FitContent(Box::new(LengthValue::Px(100.0))).clone();
}

#[test]
/// 测试 CSS 类型值的 Debug 格式化
fn test_all_css_values_debug() {
    // 这里测试各种类型值的 Debug 格式化是否正常工作
    let _ = format!("{:?}", LengthValue::Px(10.0));
    let _ = format!("{:?}", ColorValue::Rgba(255, 0, 0, 255));
    let _ = format!("{:?}", DisplayValue::Block);
    let _ = format!("{:?}", FloatValue::None);
    let _ = format!("{:?}", ClearValue::None);
    let _ = format!("{:?}", PositionValue::Static);
    let _ = format!("{:?}", OverflowValue::Visible);
    let _ = format!("{:?}", ListStyleTypeValue::Disc);
    let _ = format!("{:?}", ListStylePositionValue::Outside);
    let _ = format!("{:?}", FlexDirectionValue::Row);
    let _ = format!("{:?}", FlexWrapValue::Nowrap);

    // 测试嵌套类型的 Debug 格式化
    let _ = format!("{:?}", LengthValue::FitContent(Box::new(LengthValue::Px(100.0))));
}

// ═══════════════════════════════════════════════════════════════════════
// 36. Transform/Timing 边界测试（覆盖 parse_transform.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_animation_direction 的各种格式
fn test_parse_animation_direction_formats() {
    let test_cases = vec![
        ("normal", AnimationDirectionValue::Normal),
        ("reverse", AnimationDirectionValue::Reverse),
        ("alternate", AnimationDirectionValue::Alternate),
        ("alternate-reverse", AnimationDirectionValue::AlternateReverse),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_direction(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_direction 无效输入
fn test_parse_animation_direction_invalid() {
    let test_cases = vec![
        "",
        " ",
        "invalid",
        "alternate-reverse-extra",
        "normal extra",
        "123",
        "normal123",
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_direction(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_fill_mode 的各种格式
fn test_parse_animation_fill_mode_formats() {
    let test_cases = vec![
        ("none", AnimationFillModeValue::None),
        ("forwards", AnimationFillModeValue::Forwards),
        ("backwards", AnimationFillModeValue::Backwards),
        ("both", AnimationFillModeValue::Both),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_fill_mode(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_fill_mode 无效输入
fn test_parse_animation_fill_mode_invalid() {
    let test_cases = vec!["", " ", "invalid", "forwards extra", "none123", "123"];

    for input in test_cases {
        let result = crate::values::parse_animation_fill_mode(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_play_state 的各种格式
fn test_parse_animation_play_state_formats() {
    let test_cases = vec![
        ("running", AnimationPlayStateValue::Running),
        ("paused", AnimationPlayStateValue::Paused),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_play_state(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_play_state 无效输入
fn test_parse_animation_play_state_invalid() {
    let test_cases = vec!["", " ", "invalid", "running extra", "paused123", "123"];

    for input in test_cases {
        let result = crate::values::parse_animation_play_state(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_name 的各种格式
fn test_parse_animation_name_formats() {
    let test_cases = vec![
        ("none", AnimationNameValue::None),
        ("fadeIn", AnimationNameValue::Custom("fadeIn".to_string())),
        ("slide-in", AnimationNameValue::Custom("slide-in".to_string())),
        ("test123", AnimationNameValue::Custom("test123".to_string())),
        ("_valid", AnimationNameValue::Custom("_valid".to_string())),
        ("-valid", AnimationNameValue::Custom("-valid".to_string())),
        ("valid_name", AnimationNameValue::Custom("valid_name".to_string())),
        ("NONE", AnimationNameValue::None),
        ("fadeIn", AnimationNameValue::Custom("fadeIn".to_string())),
        ("SLIDE-IN", AnimationNameValue::Custom("SLIDE-IN".to_string())),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_name(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_name 无效输入
fn test_parse_animation_name_invalid() {
    let test_cases = vec![
        "",             // 空字符串
        " ",            // 只有空格
        "123invalid",   // 以数字开头
        "invalid name", // 包含空格
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_name(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_duration 的各种格式
fn test_parse_animation_duration_formats() {
    let test_cases = vec![
        ("1s", AnimationDurationValue::Time(1.0, TimeUnit::S)),
        ("0.5s", AnimationDurationValue::Time(0.5, TimeUnit::S)),
        ("2s", AnimationDurationValue::Time(2.0, TimeUnit::S)),
        ("500ms", AnimationDurationValue::Time(500.0, TimeUnit::Ms)),
        ("100ms", AnimationDurationValue::Time(100.0, TimeUnit::Ms)),
        ("0ms", AnimationDurationValue::Time(0.0, TimeUnit::Ms)),
        ("1.5s", AnimationDurationValue::Time(1.5, TimeUnit::S)),
        ("1500ms", AnimationDurationValue::Time(1500.0, TimeUnit::Ms)),
        ("1S", AnimationDurationValue::Time(1.0, TimeUnit::S)),
        ("0.5S", AnimationDurationValue::Time(0.5, TimeUnit::S)),
        ("500MS", AnimationDurationValue::Time(500.0, TimeUnit::Ms)),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_duration(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_duration 无效输入
fn test_parse_animation_duration_invalid() {
    let test_cases = vec![
        "",    // 空字符串
        " ",   // 只有空格
        "1",   // 没有单位
        "s",   // 只有单位
        "ms",  // 只有单位
        "1x",  // 无效单位
        "1xs", // 无效单位
        "1sm", // 无效单位
        "abc", // 无效格式
        "-1s", // 负值
        "0s",  // 零值（应该有效）
        "0ms", // 零值（应该有效）
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_duration(input);
        if input != "0s" && input != "0ms" {
            // 0 应该有效
            assert_eq!(result, None, "Should fail to parse: {}", input);
        }
    }
}

#[test]
/// 测试 parse_animation_iteration_count 的各种格式
fn test_parse_animation_iteration_count_formats() {
    let test_cases = vec![
        ("infinite", AnimationIterationCountValue::Infinite),
        ("1", AnimationIterationCountValue::Number(1.0)),
        ("2", AnimationIterationCountValue::Number(2.0)),
        ("0.5", AnimationIterationCountValue::Number(0.5)),
        ("2.5", AnimationIterationCountValue::Number(2.5)),
        ("3.0", AnimationIterationCountValue::Number(3.0)),
        ("INFINITE", AnimationIterationCountValue::Infinite),
        ("1", AnimationIterationCountValue::Number(1.0)),
        ("0.5", AnimationIterationCountValue::Number(0.5)),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_iteration_count(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_iteration_count 无效输入
fn test_parse_animation_iteration_count_invalid() {
    let test_cases = vec![
        "",               // 空字符串
        " ",              // 只有空格
        "0",              // 零值
        "-1",             // 负值
        "-0.5",           // 负值
        "infinite extra", // 额外字符
        "1 extra",        // 额外字符
        "abc",            // 无效格式
        "1.2.3",          // 多个小数点
        "1x",             // 非数字字符
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_iteration_count(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 TimingFunctionValue 枚举的各种情况
fn test_timing_function_value_variants() {
    let test_cases = vec![
        (TimingFunctionValue::Ease, "ease"),
        (TimingFunctionValue::Linear, "linear"),
        (TimingFunctionValue::EaseIn, "ease-in"),
        (TimingFunctionValue::EaseOut, "ease-out"),
        (TimingFunctionValue::EaseInOut, "ease-in-out"),
        (
            TimingFunctionValue::CubicBezier(0.25, 0.1, 0.25, 1.0),
            "cubic-bezier(0.25, 0.1, 0.25, 1.0)",
        ),
        (TimingFunctionValue::StepStart, "step-start"),
        (TimingFunctionValue::StepEnd, "step-end"),
        (
            TimingFunctionValue::Steps(5, Some(StepPosition::Start)),
            "steps(5, start)",
        ),
        (TimingFunctionValue::Steps(3, Some(StepPosition::End)), "steps(3, end)"),
        (
            TimingFunctionValue::Steps(10, Some(StepPosition::Both)),
            "steps(10, both)",
        ),
        (
            TimingFunctionValue::Steps(2, Some(StepPosition::None)),
            "steps(2, none)",
        ),
        (TimingFunctionValue::Steps(4, None), "steps(4)"),
    ];

    for (timing_value, _) in test_cases {
        // 测试 Clone
        let cloned = timing_value.clone();
        assert_eq!(timing_value, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", timing_value);
    }
}

#[test]
/// 测试 StepPosition 枚举
fn test_step_position_variants() {
    let test_cases = vec![
        (StepPosition::Start, "start"),
        (StepPosition::End, "end"),
        (StepPosition::Both, "both"),
        (StepPosition::None, "none"),
    ];

    for (step_position, _) in test_cases {
        // 测试 Clone
        let cloned = step_position.clone();
        assert_eq!(step_position, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", step_position);
    }
}
