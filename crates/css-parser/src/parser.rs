//! CSS 语法解析器。
//!
//! 将 token 流（来自 [`Tokenizer`](crate::Tokenizer)）转换为 CSS AST。

use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer};

/// CSS 解析器。
///
/// 消费 `Tokenizer` 产生的 token 流，构建 CSS AST。
pub struct Parser<'a> {
    /// Token 流。
    tokens: &'a mut Vec<Token>,
    /// 当前位置。
    pos: usize,
}

impl<'a> Parser<'a> {
    /// 创建新的解析器。
    pub fn new(tokens: &'a mut Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 从 CSS 文本解析完整的样式表。
    pub fn parse_stylesheet(input: &str) -> Stylesheet {
        let tokenizer = Tokenizer::new(input);
        let mut tokens: Vec<Token> = tokenizer.collect();
        // 添加 EOF
        tokens.push(Token::Eof);

        let mut parser = Parser::new(&mut tokens);
        let mut rules = Vec::new();

        while !parser.is_eof() {
            parser.skip_whitespace();
            if parser.is_eof() {
                break;
            }

            if let Some(rule) = parser.consume_rule() {
                rules.push(rule);
            } else {
                parser.advance();
            }
        }

        Stylesheet { rules }
    }

    // ── 内部方法 ────────────────────────────────────────────────

    /// 查看当前 token。
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    /// 前进到下一个 token。
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// 是否到达末尾。
    fn is_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// 跳过空白。
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Token::Whitespace | Token::Comment(_)) {
            self.advance();
        }
    }

    /// 消耗一个规则。
    fn consume_rule(&mut self) -> Option<Rule> {
        match self.peek().clone() {
            Token::AtKeyword(name) => {
                self.advance();
                Some(Rule::At(self.consume_at_rule(name)))
            }
            _ => {
                // 尝试解析样式规则：选择器 + { 声明块 }
                let selectors = self.consume_selector_list()?;
                self.skip_whitespace();

                if !matches!(self.peek(), Token::LBrace) {
                    return None;
                }
                self.advance(); // {

                let declarations = self.consume_declaration_block();

                self.skip_whitespace();
                if matches!(self.peek(), Token::RBrace) {
                    self.advance();
                }

                Some(Rule::Style(StyleRule {
                    selectors,
                    declarations,
                }))
            }
        }
    }

    /// 消耗选择器列表。
    fn consume_selector_list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            if matches!(self.peek(), Token::LBrace | Token::Eof) {
                break;
            }

            if let Some(sel) = self.consume_selector() {
                selectors.push(sel);
            }

            self.skip_whitespace();

            if matches!(self.peek(), Token::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        if selectors.is_empty() {
            None
        } else {
            Some(selectors)
        }
    }

    /// 消耗单个复杂选择器。
    fn consume_selector(&mut self) -> Option<Selector> {
        let mut parts = Vec::new();

        loop {
            self.skip_whitespace();

            let compound = self.consume_compound_selector()?;
            let had_whitespace_before = self.pos > 0
                && matches!(
                    self.tokens.get(self.pos - 1),
                    Some(Token::Whitespace)
                );

            self.skip_whitespace();

            // 检查组合器
            let combinator = match self.peek() {
                Token::Ident(s) if s == ">" => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::Child)
                }
                Token::Ident(s) if s == "+" => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::NextSibling)
                }
                Token::Ident(s) if s == "~" => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::SubsequentSibling)
                }
                Token::LBrace | Token::Comma | Token::RBrace | Token::Eof => None,
                _ => {
                    // 后代组合器（空白分隔）
                    if had_whitespace_before {
                        Some(Combinator::Descendant)
                    } else {
                        None
                    }
                }
            };

            parts.push((compound, combinator));

            if combinator.is_none() {
                break;
            }

            // 检查是否继续
            self.skip_whitespace();
            if matches!(
                self.peek(),
                Token::LBrace | Token::Comma | Token::RBrace | Token::Eof
            ) {
                break;
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(Selector {
                complex: ComplexSelector { parts },
            })
        }
    }

    /// 消耗复合选择器。
    fn consume_compound_selector(&mut self) -> Option<CompoundSelector> {
        let mut type_selector = None;
        let mut subclass_selectors = Vec::new();

        // 类型选择器
        match self.peek().clone() {
            Token::Ident(tag) if tag != ">" && tag != "+" && tag != "~" && tag != "|" => {
                type_selector = Some(TypeSelector::Tag(tag));
                self.advance();
            }
            Token::Ident(s) if s == "*" => {
                type_selector = Some(TypeSelector::Universal);
                self.advance();
            }
            _ => {}
        }

        // 子类选择器
        loop {
            match self.peek().clone() {
                Token::Hash(id) => {
                    subclass_selectors.push(SubclassSelector::Id(id));
                    self.advance();
                }
                Token::Ident(cls) if cls.starts_with('.') => {
                    // 不应该发生 — class 选择器以 . 开头
                    // . 由 tokenizer 处理为单独的字符
                    break;
                }
                _ => break,
            }
        }

        // 如果没有类型选择器也没有子类选择器，返回 None
        if type_selector.is_none() && subclass_selectors.is_empty() {
            None
        } else {
            Some(CompoundSelector {
                type_selector,
                subclass_selectors,
            })
        }
    }

    /// 消耗声明块。
    fn consume_declaration_block(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            self.skip_whitespace();

            if matches!(self.peek(), Token::RBrace | Token::Eof) {
                break;
            }

            if let Some(decl) = self.consume_declaration() {
                declarations.push(decl);
            }

            // 消耗分号
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
        }

        declarations
    }

    /// 消耗单个声明。
    fn consume_declaration(&mut self) -> Option<Declaration> {
        let property = match self.peek().clone() {
            Token::Ident(name) => name,
            _ => return None,
        };
        self.advance();

        self.skip_whitespace();

        // 期望冒号
        if !matches!(self.peek(), Token::Colon) {
            return None;
        }
        self.advance();

        self.skip_whitespace();

        // 收集值（直到分号或花括号）
        let mut value_parts = String::new();
        let mut important = false;

        loop {
            match self.peek() {
                Token::Semicolon | Token::RBrace | Token::Eof => break,
                Token::Whitespace => {
                    value_parts.push(' ');
                    self.advance();
                }
                Token::Ident(s) if s == "important" => {
                    // 简化处理：检查 !important
                    if value_parts.ends_with('!') {
                        value_parts.pop(); // 移除 '!'
                        important = true;
                    } else {
                        value_parts.push_str(s);
                    }
                    self.advance();
                }
                _ => {
                    let display = format!("{}", self.peek());
                    value_parts.push_str(&display);
                    self.advance();
                }
            }
        }

        let value = value_parts.trim().to_string();

        Some(Declaration {
            property,
            value,
            important,
        })
    }

    /// 消耗 @规则。
    fn consume_at_rule(&mut self, name: String) -> AtRule {
        self.skip_whitespace();

        // 收集前导部分
        let mut prelude = String::new();
        loop {
            match self.peek() {
                Token::Semicolon => {
                    self.advance();
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Statement,
                    };
                }
                Token::LBrace => {
                    self.advance();
                    let mut rules = Vec::new();
                    loop {
                        self.skip_whitespace();
                        if matches!(self.peek(), Token::RBrace | Token::Eof) {
                            if matches!(self.peek(), Token::RBrace) {
                                self.advance();
                            }
                            break;
                        }
                        if let Some(rule) = self.consume_rule() {
                            rules.push(rule);
                        } else {
                            self.advance();
                        }
                    }
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Block(rules),
                    };
                }
                Token::Eof => {
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Statement,
                    };
                }
                Token::Whitespace => {
                    prelude.push(' ');
                    self.advance();
                }
                _ => {
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
            }
        }
    }
}
