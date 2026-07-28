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
        let mut tokens: Vec<Token> = tokenizer.map(|s| s.token).collect();
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
                // 对 @keyframes 使用专用解析器
                if name.eq_ignore_ascii_case("keyframes") {
                    return self.consume_keyframes_rule().map(Rule::Keyframes);
                }
                // 对 @layer 使用专用解析器
                if name.eq_ignore_ascii_case("layer") {
                    return self.consume_layer_rule().map(Rule::Layer);
                }
                // 对 @import 使用专用解析器
                if name.eq_ignore_ascii_case("import") {
                    return self.consume_import_rule().map(Rule::Import);
                }
                // 对 @supports 使用专用解析器
                if name.eq_ignore_ascii_case("supports") {
                    return self.consume_supports_rule().map(Rule::Supports);
                }
                // 对 @container 使用专用解析器
                if name.eq_ignore_ascii_case("container") {
                    return self.consume_container_rule().map(Rule::Container);
                }
                // 对 @font-face 使用专用解析器（body 是声明块，非嵌套规则）
                if name.eq_ignore_ascii_case("font-face") {
                    return self.consume_font_face_rule().map(Rule::FontFace);
                }
                // 对 @page 使用专用解析器（body 是声明块：size/margin 描述符）
                if name.eq_ignore_ascii_case("page") {
                    return self.consume_page_rule().map(Rule::Page);
                }
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

        if selectors.is_empty() { None } else { Some(selectors) }
    }

    /// 消耗单个复杂选择器。
    fn consume_selector(&mut self) -> Option<Selector> {
        let mut parts = Vec::new();

        loop {
            self.skip_whitespace();

            // 检查是否到达选择器列表的结束位置
            if matches!(self.peek(), Token::LBrace | Token::Comma | Token::RBrace | Token::Eof) {
                break;
            }

            // 处理前导组合器（如 :has(> .child)），隐式添加通用选择器
            let leading_combinator = match self.peek() {
                Token::Delim('>') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::Child)
                }
                Token::Delim('+') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::NextSibling)
                }
                Token::Delim('~') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::SubsequentSibling)
                }
                _ => None,
            };

            if leading_combinator.is_some() {
                // 隐式主题（:has() 元素自身）作为通用选择器
                let implicit = CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                };
                parts.push((implicit, leading_combinator));
                continue;
            }

            let compound = self.consume_compound_selector()?;

            // 保存当前位置，检查 skip_whitespace 是否跳过了空白
            let pos_before_ws = self.pos;
            self.skip_whitespace();
            let had_whitespace = self.pos > pos_before_ws;

            // 检查组合器
            let combinator = match self.peek() {
                // 使用 Delim 处理组合器
                Token::Delim('>') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::Child)
                }
                Token::Delim('+') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::NextSibling)
                }
                Token::Delim('~') => {
                    self.advance();
                    self.skip_whitespace();
                    Some(Combinator::SubsequentSibling)
                }
                Token::LBrace | Token::Comma | Token::RBrace | Token::Eof => None,
                _ => {
                    // 后代组合器（空白分隔）
                    if had_whitespace {
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
            if matches!(self.peek(), Token::LBrace | Token::Comma | Token::RBrace | Token::Eof) {
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
            Token::Ident(tag) => {
                type_selector = Some(TypeSelector::Tag(tag));
                self.advance();
            }
            Token::Delim('*') => {
                type_selector = Some(TypeSelector::Universal);
                self.advance();
            }
            _ => {}
        }

        // 子类选择器循环
        loop {
            match self.peek().clone() {
                // ID 选择器
                Token::Hash(id) => {
                    subclass_selectors.push(SubclassSelector::Id(id));
                    self.advance();
                }
                // 类选择器
                Token::Delim('.') => {
                    self.advance();
                    if let Token::Ident(cls) = self.peek().clone() {
                        subclass_selectors.push(SubclassSelector::Class(cls));
                        self.advance();
                    }
                }
                // 属性选择器
                Token::LBracket => {
                    self.advance();
                    if let Some(attr_sel) = self.consume_attribute_selector() {
                        subclass_selectors.push(SubclassSelector::Attribute(attr_sel));
                    }
                }
                // 伪类 / 伪元素选择器
                Token::Colon => {
                    self.advance();
                    if matches!(self.peek(), Token::Colon) {
                        // 伪元素（::before, ::after）
                        self.advance();
                        if let Token::Ident(name) = self.peek().clone() {
                            // CSS 伪元素名 ASCII 大小写不敏感（CSS Syntax §5），归一化为小写
                            // 供下游 matcher 按小写名匹配（WPT case-sensitive-003 `::FiRst-LiNe`）。
                            let name = name.to_ascii_lowercase();
                            subclass_selectors
                                .push(SubclassSelector::PseudoElement(PseudoElementSelector::Standard(name)));
                            self.advance();
                        }
                    } else if let Token::Ident(name) = self.peek().clone() {
                        // 简单伪类或函数伪类（Ident + LParen 形式）。伪类名 ASCII 大小写不敏感
                        // （CSS Syntax §5），归一化为小写——既为下方 `match name.as_str()` 分发，
                        // 也为 PseudoClassSelector::Simple 存储供 matcher 匹配（WPT case-sensitive-003）。
                        let name = name.to_ascii_lowercase();
                        self.advance();
                        if matches!(self.peek(), Token::LParen) {
                            self.advance(); // (
                            // 解析函数伪类参数
                            let pseudo = match name.as_str() {
                                "not" => self.parse_pseudo_class_function_list("not"),
                                "is" => self.parse_pseudo_class_function_list("is"),
                                "where" => self.parse_pseudo_class_function_list("where"),
                                "has" => self.parse_pseudo_class_function_list("has"),
                                "nth-child" => self.parse_nth_pattern("nth-child"),
                                "nth-last-child" => self.parse_nth_pattern("nth-last-child"),
                                "nth-of-type" => self.parse_nth_pattern("nth-of-type"),
                                "nth-last-of-type" => self.parse_nth_last_of_type_pattern(),
                                "lang" => self.parse_lang(),
                                _ => PseudoClassSelector::Simple(name),
                            };
                            subclass_selectors.push(SubclassSelector::PseudoClass(pseudo));
                        } else {
                            // CSS2 允许单冒号伪元素语法：:before/:after/:first-letter/:first-line
                            // 等价于 CSS3 的 ::before 等（选择器规范 §7）。归为伪元素而非伪类。
                            match name.as_str() {
                                "before" | "after" | "first-letter" | "first-line" => {
                                    subclass_selectors
                                        .push(SubclassSelector::PseudoElement(PseudoElementSelector::Standard(name)));
                                }
                                _ => {
                                    subclass_selectors
                                        .push(SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)));
                                }
                            }
                        }
                    } else if let Token::Function(name) = self.peek().clone() {
                        // 函数伪类（Function token 形式，tokenizer 直接产生 Function）。
                        // 函数名 ASCII 大小写不敏感（CSS Syntax §5），归一化为小写（同上）。
                        let name = name.to_ascii_lowercase();
                        self.advance(); // 消耗 Function token（已包含 '('）
                        let pseudo = match name.as_str() {
                            "not" => self.parse_pseudo_class_function_list("not"),
                            "is" => self.parse_pseudo_class_function_list("is"),
                            "where" => self.parse_pseudo_class_function_list("where"),
                            "has" => self.parse_pseudo_class_function_list("has"),
                            "nth-child" => self.parse_nth_pattern("nth-child"),
                            "nth-last-child" => self.parse_nth_pattern("nth-last-child"),
                            "nth-of-type" => self.parse_nth_pattern("nth-of-type"),
                            "nth-last-of-type" => self.parse_nth_last_of_type_pattern(),
                            "lang" => self.parse_lang(),
                            _ => PseudoClassSelector::Simple(name),
                        };
                        subclass_selectors.push(SubclassSelector::PseudoClass(pseudo));
                    }
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

    /// 解析函数伪类选择器列表（:not()、:is()、:where()）。
    ///
    /// 调用前已消耗 `(`。
    fn parse_pseudo_class_function_list(&mut self, _name: &str) -> PseudoClassSelector {
        let selectors = self.consume_selector_list_for_function();

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        match _name {
            "not" => PseudoClassSelector::Not(selectors),
            "is" => PseudoClassSelector::Is(selectors),
            "where" => PseudoClassSelector::Where(selectors),
            "has" => PseudoClassSelector::Has(selectors),
            _ => PseudoClassSelector::Simple(_name.to_string()),
        }
    }

    /// 为函数伪类内部消耗选择器列表。
    fn consume_selector_list_for_function(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            if matches!(self.peek(), Token::RParen | Token::Eof) {
                break;
            }

            if let Some(sel) = self.consume_selector() {
                selectors.push(sel);
            } else {
                break;
            }

            self.skip_whitespace();

            if matches!(self.peek(), Token::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        selectors
    }

    /// 解析 nth 函数模式（:nth-child、:nth-last-child、:nth-of-type）。
    ///
    /// 调用前已消耗 `(`。
    fn parse_nth_pattern(&mut self, name: &str) -> PseudoClassSelector {
        let pattern = self.parse_nth_expression();

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        match name {
            "nth-child" => PseudoClassSelector::NthChild(pattern),
            "nth-last-child" => PseudoClassSelector::NthLastChild(pattern),
            "nth-of-type" => PseudoClassSelector::NthOfType(pattern),
            _ => PseudoClassSelector::Simple(name.to_string()),
        }
    }

    /// 解析 nth-last-of-type 函数模式。
    ///
    /// 调用前已消耗 `(`。
    fn parse_nth_last_of_type_pattern(&mut self) -> PseudoClassSelector {
        let pattern = self.parse_nth_expression();

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        PseudoClassSelector::NthLastOfType(pattern)
    }

    /// 解析 nth 表达式（如 `2n+1`、`odd`、`even`、`3`）。
    fn parse_nth_expression(&mut self) -> NthPattern {
        self.skip_whitespace();

        // 收集 nth 表达式的文本
        let mut expr = String::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            match self.peek() {
                Token::Whitespace => {
                    if !expr.is_empty() && !expr.ends_with(' ') {
                        expr.push(' ');
                    }
                    self.advance();
                }
                _ => {
                    expr.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
            }
        }

        let expr = expr.trim();

        // 解析特殊关键字
        match expr {
            "odd" => return NthPattern { a: 2, b: 1 },
            "even" => return NthPattern { a: 2, b: 0 },
            _ => {}
        }

        // 解析 an+b 模式
        Self::parse_nth_expression_str(expr)
    }

    /// 从字符串解析 nth 表达式。
    fn parse_nth_expression_str(expr: &str) -> NthPattern {
        let expr = expr.replace(' ', "");
        let expr_lower = expr.to_lowercase();

        // 尝试匹配 an+b 或 an-b 模式
        if let Some(n_pos) = expr_lower.find('n') {
            let a_part = &expr_lower[..n_pos];
            let b_part = &expr_lower[n_pos + 1..];

            let a: i32 = if a_part.is_empty() || a_part == "+" {
                1
            } else if a_part == "-" {
                -1
            } else {
                a_part.parse().unwrap_or(0)
            };

            let b: i32 = if b_part.is_empty() {
                0
            } else {
                b_part.parse().unwrap_or(0)
            };

            return NthPattern { a, b };
        }

        // 纯数字
        let b: i32 = expr_lower.parse().unwrap_or(0);
        NthPattern { a: 0, b }
    }

    /// 解析 :lang() 函数。
    ///
    /// 调用前已消耗 `(`。
    fn parse_lang(&mut self) -> PseudoClassSelector {
        self.skip_whitespace();

        let lang = match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                s
            }
            Token::String(s) => {
                self.advance();
                s
            }
            _ => String::new(),
        };

        self.skip_whitespace();

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        PseudoClassSelector::Lang(lang)
    }

    /// 消耗属性选择器。
    ///
    /// 调用前已消耗 `[`。
    fn consume_attribute_selector(&mut self) -> Option<AttributeSelector> {
        self.skip_whitespace();

        // 属性名
        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                // 跳到 ] 并返回 None
                self.skip_to_rbracket();
                return None;
            }
        };

        self.skip_whitespace();

        // 检查匹配器
        let matcher = match self.peek() {
            Token::RBracket => {
                // [attr] — 属性存在
                self.advance();
                return Some(AttributeSelector {
                    name,
                    matcher: AttributeMatcher::Exists,
                });
            }
            Token::Delim('=') => {
                // [attr=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::Exact(val)
            }
            Token::IncludeMatch => {
                // [attr~=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::Includes(val)
            }
            Token::DashMatch => {
                // [attr|=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::DashMatch(val)
            }
            Token::PrefixMatch => {
                // [attr^=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::Prefix(val)
            }
            Token::SuffixMatch => {
                // [attr$=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::Suffix(val)
            }
            Token::SubstringMatch => {
                // [attr*=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBracket) {
                    self.advance();
                }
                AttributeMatcher::Substring(val)
            }
            _ => {
                // 未知匹配器，跳到 ]
                self.skip_to_rbracket();
                return Some(AttributeSelector {
                    name,
                    matcher: AttributeMatcher::Exists,
                });
            }
        };

        Some(AttributeSelector { name, matcher })
    }

    /// 消耗属性选择器的值。
    fn consume_attribute_value(&mut self) -> String {
        match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                s
            }
            Token::String(s) => {
                self.advance();
                s
            }
            Token::Delim(c) => {
                // 处理以非标识符字符开头的值（如 .pdf）
                let mut val = c.to_string();
                self.advance();
                // 继续收集后续的标识符部分
                loop {
                    match self.peek() {
                        Token::Ident(s) => {
                            val.push_str(s);
                            self.advance();
                        }
                        Token::Delim('.') => {
                            val.push('.');
                            self.advance();
                        }
                        Token::Number(n) => {
                            val.push_str(&n.to_string());
                            self.advance();
                        }
                        _ => break,
                    }
                }
                val
            }
            Token::Number(n) => {
                self.advance();
                // 可能后面跟着标识符（如数字+单位）
                let mut val = n.to_string();
                if let Token::Ident(unit) = self.peek().clone() {
                    val.push_str(&unit);
                    self.advance();
                }
                val
            }
            _ => String::new(),
        }
    }

    /// 跳到右方括号。
    fn skip_to_rbracket(&mut self) {
        while !matches!(self.peek(), Token::RBracket | Token::Eof) {
            self.advance();
        }
        if matches!(self.peek(), Token::RBracket) {
            self.advance();
        }
    }

    /// 消耗畸形声明：读到 `;` / `}` / EOF，期间尊重 `()` / `[]` / `{}` 嵌套块
    ///（嵌套块内的 `;` / `}` 不作终止符）。CSS 2.1 §4.2「Malformed declarations」错误恢复。
    ///
    /// 调用方（`consume_declaration_block`）保证进入时当前 token **非** `;` / `}` / EOF
    ///（均已在上游分别处理），故首 token 必被本函数消耗，保证解析进度、避免死循环。
    /// 典型场景（driving: core-syntax-001）：`test { :nested; color: yellow; } : junk;`
    ///——`test` 后非 `:` 而是 `{`，整个 `{...}` 块 + trailing `: junk` 须作为一条畸形声明
    /// 整体跳过（块内 `;`/`}` 受嵌套保护不提前终止），否则块内 `color/background` 会泄漏
    /// 进外层规则、且外层块会在嵌套 `}` 处提前关闭。
    fn skip_malformed_declaration(&mut self) {
        let mut depth: i32 = 0;
        loop {
            match self.peek() {
                Token::Eof => return,
                Token::Semicolon | Token::RBrace if depth == 0 => return,
                Token::LParen | Token::LBracket | Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen | Token::RBracket | Token::RBrace => {
                    depth = (depth - 1).max(0);
                    self.advance();
                }
                _ => self.advance(),
            }
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

            // 空声明 / 多余分号：直接跳过（避免传入 consume_declaration 返回 None 后
            // skip_malformed_declaration 在 `;` 处零进度导致死循环）。
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
                continue;
            }

            if let Some(decl) = self.consume_declaration() {
                declarations.push(decl);
                // consume_declaration 消耗到分号或 RBrace 前停止
                // 如果当前是分号则消耗它，否则不推进（避免吞噬 RBrace）
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                }
            } else {
                // 畸形声明错误恢复（CSS 2.1 §4.2）：当前 token 非 `;`/`}`/EOF，读到下一条
                // 声明边界，尊重嵌套块。旧实现仅 `advance()` 一个 token，会把 `{...}` 嵌套块
                // 内容泄漏进外层规则 + 提前关闭外层块（driving: core-syntax-001）。
                self.skip_malformed_declaration();
            }
        }

        declarations
    }

    /// 消耗单个声明。
    fn consume_declaration(&mut self) -> Option<Declaration> {
        let property = match self.peek().clone() {
            // CSS 属性名 ASCII 大小写不敏感（CSS Syntax §5「All CSS keywords are
            // case-insensitive」，规范形式为小写），下游 apply.rs / shorthand 按小写名
            // dispatch，故此处归一化为小写，否则 `bACkGRounD` 不匹配 `"background"` 致
            // 声明被丢弃（WPT case-sensitive-000/001）。
            // 但 CSS 自定义属性（`--foo`）大小写敏感（CSS Variables §2），须保留原值——
            // tokenizer 把 `--foo` 整体消费为单个 Ident（'-' 是 ident-start 字符），故
            // starts_with("--") 可区分。
            Token::Ident(name) => {
                if name.starts_with("--") {
                    name
                } else {
                    name.to_ascii_lowercase()
                }
            }
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
        // 跟踪 () / [] 嵌套：CSS Syntax L3 中，值内的简单块（函数、calc 括号等）
        // 作为单个 component value 消费，其内部的 ; / } 不应终止声明。
        // 关键场景：`font-family: test(foo, Ahem` 中未匹配的 `(` 会使后续的
        // `;` 和 `}` 都属于该 `(` 块，从而吞掉后续规则直到匹配的 `)`。
        let mut group_depth: i32 = 0;

        // 延迟空白：仅当后续有非空白 token 时才写入空格（首尾空白 token 不入值）。
        // 这样无需末尾 trim，**保留**值内由转义产生的空白（如 `red\9` → `red\t`，
        // `red\t` ≠ 关键字 `red`，apply 拒绝→cascade R2126 丢弃，下个合法声明胜出）。
        // 原 `value_parts.trim()` 会把转义产生的空白一并剥掉（driving：escapes-014/015/016）。
        let mut pending_ws = false;
        macro_rules! flush_ws {
            () => {
                if pending_ws {
                    if !value_parts.is_empty() {
                        value_parts.push(' ');
                    }
                    pending_ws = false;
                }
            };
        }

        loop {
            match self.peek() {
                Token::Semicolon | Token::RBrace if group_depth == 0 => break,
                // Eof 是输入结束，无论嵌套深度都必须终止。否则未闭合的 ()/[] 会让
                // Eof 落入下方 `_ =>` arm：advance() 越界后 peek() 永远返回 Eof，
                // 死循环无限 `format!`+`push_str` → OOM（曾反复整垮 tmux session）。
                Token::Eof => break,
                // Function token = `ident(`，隐含一个未闭合的 `(`，需要匹配的 `)`。
                Token::Function(_) | Token::LParen | Token::LBracket => {
                    group_depth += 1;
                    flush_ws!();
                    let display = format!("{}", self.peek());
                    value_parts.push_str(&display);
                    self.advance();
                }
                Token::RParen | Token::RBracket => {
                    group_depth = (group_depth - 1).max(0);
                    flush_ws!();
                    let display = format!("{}", self.peek());
                    value_parts.push_str(&display);
                    self.advance();
                }
                Token::Whitespace => {
                    pending_ws = true;
                    self.advance();
                }
                Token::Delim('!') if group_depth == 0 => {
                    self.advance();
                    self.skip_whitespace();
                    if let Token::Ident(s) = self.peek().clone()
                        && s.eq_ignore_ascii_case("important")
                    {
                        self.advance(); // 消耗 "important"
                        // CSS Syntax：`!important` 必须紧跟 `;` / `}` / EOF 才有效。若其后
                        // 还有非空白 token（如 `background: red !important fail`），整个声明
                        // 非法（driving: core-syntax-006 `background: red ! important fail`）。
                        // 旧实现此处直接 break，把 `red !important` 当有效声明，trailing `fail`
                        // 成独立坏声明——应改为把 `!important` 回填进值，使值整体无效→cascade
                        // 丢弃（与 chromium 一致：trailing token 后 !important 不生效）。
                        self.skip_whitespace();
                        if matches!(self.peek(), Token::Semicolon | Token::RBrace | Token::Eof) {
                            important = true;
                            break;
                        }
                        // 有 trailing token → !important 无效，回填值使声明整体非法
                        flush_ws!();
                        value_parts.push_str("!important");
                    } else {
                        flush_ws!();
                        value_parts.push('!');
                    }
                }
                _ => {
                    flush_ws!();
                    let display = format!("{}", self.peek());
                    value_parts.push_str(&display);
                    self.advance();
                }
            }
        }

        let value = value_parts;

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

    /// 消耗 @font-face 规则。
    ///
    /// 格式：`@font-face { font-family: "X"; src: url("X.woff") format("woff"); }`
    /// body 是声明块（非嵌套样式规则），用 `consume_declaration_block` 解析，
    /// 提取 `font-family`（族名，去引号）与 `src`（所有 url()，按出现顺序）。
    fn consume_font_face_rule(&mut self) -> Option<FontFaceRule> {
        self.skip_whitespace();
        // @font-face 无 prelude，必须直接是 `{`
        if !matches!(self.peek(), Token::LBrace) {
            // 非 `{`：跳到 `;` 或 `}`，返回 None 让上层丢弃
            while !matches!(self.peek(), Token::Semicolon | Token::Eof) {
                self.advance();
            }
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
            return None;
        }
        self.advance(); // {

        let declarations = self.consume_declaration_block();

        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
        }

        let mut family = String::new();
        let mut sources: Vec<String> = Vec::new();
        for decl in &declarations {
            if decl.property.eq_ignore_ascii_case("font-family") {
                family = strip_css_quotes(decl.value.trim());
            } else if decl.property.eq_ignore_ascii_case("src") {
                for url in extract_urls_from_src(&decl.value) {
                    sources.push(url);
                }
            }
        }

        if family.is_empty() || sources.is_empty() {
            return None;
        }

        Some(FontFaceRule { family, sources })
    }

    /// 消耗 @page 规则（CSS Paged Media）。
    ///
    /// 格式：`@page { size: A4; margin: 2cm; }`（prelude 可为命名页 `:first` / `name`，
    /// 当前忽略——仅消费 body 声明块提取 `size` 描述符）。
    /// body 是声明块（同 @font-face），用 `consume_declaration_block` 解析，
    /// 提取 `size` 描述符并经 `resolve_page_size_px` 解析为像素 `(width, height)`。
    fn consume_page_rule(&mut self) -> Option<PageRule> {
        self.skip_whitespace();
        // 跳过 prelude（命名页 / `:first` 等）直到 `{` / `;` / EOF。
        while !matches!(self.peek(), Token::LBrace | Token::Semicolon | Token::Eof) {
            self.advance();
        }
        if !matches!(self.peek(), Token::LBrace) {
            // 非 `{`（如 `@page :first;` 语句形式）：吞 `;`，无 body → 无 size。
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
            return Some(PageRule {
                size: None,
                margin: None,
            });
        }
        self.advance(); // {

        let declarations = self.consume_declaration_block();

        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
        }

        // 提取首个有效 `size` 描述符（CSS 规范：后声明优先，但 @page size 单一语义，
        // 取首个有效即可）。
        let mut size: Option<(f32, f32)> = None;
        let mut margin: Option<(f32, f32, f32, f32)> = None;
        for decl in &declarations {
            if decl.property.eq_ignore_ascii_case("size") && size.is_none() {
                if let Some(resolved) = resolve_page_size_px(&decl.value) {
                    size = Some(resolved);
                }
            } else if decl.property.eq_ignore_ascii_case("margin") && margin.is_none() {
                if let Some(resolved) = resolve_page_margin_px(&decl.value) {
                    margin = Some(resolved);
                }
            }
        }

        Some(PageRule { size, margin })
    }

    /// 消耗 @keyframes 规则。
    ///
    /// 格式：`@keyframes name { from { ... } 50% { ... } to { ... } }`
    fn consume_keyframes_rule(&mut self) -> Option<KeyframesRule> {
        use crate::ast::*;

        self.skip_whitespace();

        // 读取动画名称
        let name = match self.peek().clone() {
            Token::Ident(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            Token::String(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            _ => return None,
        };

        self.skip_whitespace();

        // 期望 {
        if !matches!(self.peek(), Token::LBrace) {
            return None;
        }
        self.advance();

        // 解析关键帧块列表
        let mut keyframes = Vec::new();

        loop {
            self.skip_whitespace();
            if matches!(self.peek(), Token::RBrace | Token::Eof) {
                if matches!(self.peek(), Token::RBrace) {
                    self.advance();
                }
                break;
            }

            // 读取关键帧选择器（百分比、from、to），逗号分隔
            let mut selectors = Vec::new();
            loop {
                self.skip_whitespace();
                match self.peek().clone() {
                    Token::Ident(ref s) if s.eq_ignore_ascii_case("from") => {
                        selectors.push(KeyframeSelector::From);
                        self.advance();
                    }
                    Token::Ident(ref s) if s.eq_ignore_ascii_case("to") => {
                        selectors.push(KeyframeSelector::To);
                        self.advance();
                    }
                    Token::Percentage(pct) => {
                        selectors.push(KeyframeSelector::Percentage(pct));
                        self.advance();
                    }
                    _ => {
                        // 无法识别的选择器，跳过这个关键帧块
                        break;
                    }
                }

                self.skip_whitespace();
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }

            if selectors.is_empty() {
                // 跳过无效内容直到 } 或下一个可识别的选择器
                self.advance();
                continue;
            }

            self.skip_whitespace();

            // 期望 {
            if !matches!(self.peek(), Token::LBrace) {
                continue;
            }
            self.advance();

            // 解析声明块
            let declarations = self.consume_declaration_block();

            self.skip_whitespace();
            if matches!(self.peek(), Token::RBrace) {
                self.advance();
            }

            keyframes.push(KeyframeBlock {
                selectors,
                declarations,
            });
        }

        Some(KeyframesRule { name, keyframes })
    }

    /// 消耗 @layer 规则。
    ///
    /// 格式：`@layer <name> { <rules> }` 或 `@layer <name>;`
    fn consume_layer_rule(&mut self) -> Option<LayerRule> {
        use crate::ast::*;

        self.skip_whitespace();

        // 读取层名称（可选）
        let name = match self.peek().clone() {
            Token::Ident(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            Token::String(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            // 匿名层：@layer { ... }
            Token::LBrace => String::new(),
            // @layer; — 声明-only（无名称无规则体）
            Token::Semicolon => {
                self.advance();
                return Some(LayerRule {
                    name: String::new(),
                    rules: vec![],
                });
            }
            _ => return None,
        };

        self.skip_whitespace();

        // 期望 { 或 ;
        match self.peek() {
            Token::Semicolon => {
                // @layer <name>; — 仅声明层名
                self.advance();
                Some(LayerRule { name, rules: vec![] })
            }
            Token::LBrace => {
                self.advance();

                // 解析层内规则列表
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

                Some(LayerRule { name, rules })
            }
            _ => None,
        }
    }

    /// 消耗 @import 规则。
    ///
    /// 格式：`@import url("path");` 或 `@import "path";`
    /// 可选媒体查询：`@import "style.css" screen and (max-width: 600px);`
    fn consume_import_rule(&mut self) -> Option<ImportRule> {
        self.skip_whitespace();

        // 读取 URL：可以是 url(...) 或字符串字面量
        let url = match self.peek().clone() {
            Token::Url(u) => {
                let result = u;
                self.advance();
                result
            }
            Token::String(s) => {
                let result = s;
                self.advance();
                result
            }
            _ => return None,
        };

        self.skip_whitespace();

        // 可选的媒体查询部分：收集直到分号
        let mut media_queries = Vec::new();
        let mut current_query = String::new();

        loop {
            match self.peek() {
                Token::Semicolon => {
                    self.advance();
                    break;
                }
                Token::Eof => {
                    break;
                }
                Token::Comma => {
                    // 逗号分隔多个媒体查询
                    let trimmed = current_query.trim().to_string();
                    if !trimmed.is_empty() {
                        media_queries.push(trimmed);
                    }
                    current_query.clear();
                    self.advance();
                    self.skip_whitespace();
                }
                Token::Whitespace => {
                    if !current_query.is_empty() && !current_query.ends_with(' ') {
                        current_query.push(' ');
                    }
                    self.advance();
                }
                _ => {
                    current_query.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
            }
        }

        let trimmed = current_query.trim().to_string();
        if !trimmed.is_empty() {
            media_queries.push(trimmed);
        }

        Some(ImportRule { url, media_queries })
    }

    /// 消耗 @supports 规则。
    ///
    /// 格式：`@supports (<条件>) { <规则> }`
    fn consume_supports_rule(&mut self) -> Option<SupportsRule> {
        self.skip_whitespace();

        // 收集 prelude 文本（直到 {）
        let mut prelude = String::new();
        loop {
            match self.peek() {
                Token::LBrace => break,
                Token::Eof => return None,
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

        let condition = crate::supports_condition::parse_supports_condition(prelude.trim())?;

        self.skip_whitespace();

        // 期望 {
        if !matches!(self.peek(), Token::LBrace) {
            return None;
        }
        self.advance();

        // 解析规则列表
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

        Some(SupportsRule { condition, rules })
    }

    /// 消耗 @container 规则。
    ///
    /// 格式：`@container <name>? (<条件>) { <规则> }`
    fn consume_container_rule(&mut self) -> Option<ContainerRule> {
        self.skip_whitespace();

        // 读取可选的容器名称（标识符，在括号之前）
        // 名称是第一个标识符（如果后面跟着 '('），条件则以 '(' 开头
        let name = if let Token::Ident(s) = self.peek().clone() {
            // 保存位置，前进查看下一个非空白 token
            let pos_before = self.pos;
            self.advance();
            self.skip_whitespace();
            if matches!(self.peek(), Token::LParen) {
                // ident 后面跟着 '(' → ident 是容器名称
                Some(s)
            } else {
                // ident 后面不是 '(' → 回退，这不是名称
                self.pos = pos_before;
                None
            }
        } else {
            None
        };

        self.skip_whitespace();

        // 收集条件文本。两种形式：
        //   (1) `(...)` 普通条件；
        //   (2) `size(...)` / `inline-size(...)` 尺寸函数条件（CSS Contain 3）——tokenizer
        //       把 `size(` 产成 `Function("size")`（ident 紧跟 `(`），Function token 已含 `(`，
        //       须单独处理，否则 `size` 被跳过、条件丢失 `size(` 包装致 parse_container_condition
        //       解析为裸 width 条件（driving: `@container size(width > 300px)`）。
        let condition = if matches!(self.peek(), Token::LParen) {
            self.advance(); // (
            let cond_text = self.collect_paren_content()?;
            parse_container_condition(cond_text.trim())?
        } else if let Token::Function(func) = self.peek().clone() {
            if !func.eq_ignore_ascii_case("size") && !func.eq_ignore_ascii_case("inline-size") {
                return None;
            }
            let func = func.to_ascii_lowercase();
            self.advance(); // Function token（已含 `(`）
            let inner = self.collect_paren_content()?;
            parse_container_condition(&format!("{func}({inner})"))?
        } else {
            return None;
        };

        self.skip_whitespace();

        // 期望 {
        if !matches!(self.peek(), Token::LBrace) {
            return None;
        }
        self.advance();

        // 解析规则列表
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

        Some(ContainerRule { name, condition, rules })
    }

    /// 收集已消耗 `(` 后的括号内容文本，直到匹配 `)`（嵌套 `()` 保留）。
    /// 供 `consume_container_rule` 条件收集复用。EOF（未闭合）返回 None。
    fn collect_paren_content(&mut self) -> Option<String> {
        let mut text = String::new();
        let mut depth = 1;
        loop {
            match self.peek() {
                Token::LParen => {
                    depth += 1;
                    text.push('(');
                    self.advance();
                }
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        self.advance(); // )
                        break;
                    }
                    text.push(')');
                    self.advance();
                }
                Token::Whitespace => {
                    text.push(' ');
                    self.advance();
                }
                Token::Eof => return None,
                _ => {
                    text.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
            }
        }
        Some(text)
    }
}

/// 解析容器条件文本。
///
/// 支持格式如 `min-width: 400px`、`width > 300px`、`max-width: 800px`。
fn parse_container_condition(text: &str) -> Option<ContainerCondition> {
    let text = text.trim();

    // 检查 size() 或 inline-size() 包装
    if let Some(inner) = text.strip_prefix("size(").and_then(|s| s.strip_suffix(')')) {
        return Some(ContainerCondition::Size(parse_size_condition(inner.trim())?));
    }
    if let Some(inner) = text.strip_prefix("inline-size(").and_then(|s| s.strip_suffix(')')) {
        return Some(ContainerCondition::InlineSize(parse_size_condition(inner.trim())?));
    }

    // 默认为 Size 条件（裸条件如 `min-width: 400px`）
    Some(ContainerCondition::Size(parse_size_condition(text)?))
}

/// 解析尺寸条件。
///
/// 支持格式如 `min-width: 400px`、`width > 300px`、`200px <= width <= 500px`。
fn parse_size_condition(text: &str) -> Option<ContainerSizeCondition> {
    let text = text.trim();

    // 尝试范围语法：`200px <= width <= 500px`
    // 找到 "<=" ... "<=" 模式
    let mut first_le = None;
    let mut second_le = None;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'=' {
            if first_le.is_none() {
                first_le = Some(i);
            } else {
                second_le = Some(i);
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    if let (Some(pos1), Some(pos2)) = (first_le, second_le) {
        let min_val = text[..pos1].trim().to_string();
        let feature = text[pos1 + 2..pos2].trim().to_string();
        let max_val = text[pos2 + 2..].trim().to_string();
        if !min_val.is_empty() && !feature.is_empty() && !max_val.is_empty() {
            return Some(ContainerSizeCondition {
                feature,
                value: String::new(),
                operator: None,
                range_min: Some(min_val),
                range_max: Some(max_val),
            });
        }
    }

    // 尝试冒号分隔格式：`min-width: 400px`
    if let Some(colon_pos) = text.find(':') {
        let feature = text[..colon_pos].trim().to_string();
        let value = text[colon_pos + 1..].trim().to_string();
        if feature.is_empty() || value.is_empty() {
            return None;
        }
        return Some(ContainerSizeCondition {
            feature,
            value,
            operator: None,
            range_min: None,
            range_max: None,
        });
    }

    // 尝试比较运算符格式：`width > 300px`、`width >= 300px`、`width < 300px`、`width <= 300px`
    for op in [">=", "<=", ">", "<"] {
        if let Some(op_pos) = text.find(op) {
            let feature = text[..op_pos].trim().to_string();
            let value = text[op_pos + op.len()..].trim().to_string();
            if feature.is_empty() || value.is_empty() {
                return None;
            }
            return Some(ContainerSizeCondition {
                feature,
                value,
                operator: Some(op.to_string()),
                range_min: None,
                range_max: None,
            });
        }
    }

    None
}

/// 去掉 CSS 字符串值两端的引号（单引号或双引号）。
fn strip_css_quotes(s: &str) -> String {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
        || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// 从 `src` 描述符值中提取所有 `url(...)` 内的 URL（按出现顺序，去引号）。
///
/// 支持 `url("X.woff")`、`url(X.woff)`、`url('X.woff')`，忽略 `format(...)` 等其他部分。
fn extract_urls_from_src(src: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let lower = src.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel_idx) = lower[search_from..].find("url(") {
        let open_paren = search_from + rel_idx + 3; // 指向 '('
        // 找匹配的 ')'
        let after = &src[open_paren + 1..];
        let close_rel = match after.find(')') {
            Some(r) => r,
            None => break,
        };
        let inner = after[..close_rel].trim();
        let url = strip_css_quotes(inner);
        if !url.is_empty() {
            urls.push(url);
        }
        search_from = open_paren + 1 + close_rel + 1;
        if search_from >= src.len() {
            break;
        }
    }
    urls
}

/// 解析 @page `size` 描述符为像素 `(width, height)`（@96dpi）。
///
/// 支持：
/// - 命名尺寸：`a3` / `a4` / `a5` / `b5` / `letter` / `legal` / `ledger`（portrait 朝向）
/// - 朝向修饰：`<name> portrait` / `<name> landscape`（或单独 `portrait` / `landscape`，默认 A4）
/// - 显式长度：`<length>`（正方页）或 `<length> <length>`（宽 高）
///
/// 其他值（`auto` / 未知关键字 / 相对单位）→ `None`（调用方回退默认 A4）。
pub fn resolve_page_size_px(size: &str) -> Option<(f32, f32)> {
    use crate::values::{LengthValue, parse_length};

    /// @96dpi 命名页尺寸 `(width, height)`，portrait 朝向（w ≤ h）。
    fn named(name: &str) -> Option<(f32, f32)> {
        const PX_PER_MM: f32 = 96.0 / 25.4;
        const PX_PER_IN: f32 = 96.0;
        match name {
            "a5" => Some((148.0 * PX_PER_MM, 210.0 * PX_PER_MM)),
            "a4" => Some((210.0 * PX_PER_MM, 297.0 * PX_PER_MM)),
            "a3" => Some((297.0 * PX_PER_MM, 420.0 * PX_PER_MM)),
            "b5" => Some((176.0 * PX_PER_MM, 250.0 * PX_PER_MM)),
            "letter" => Some((8.5 * PX_PER_IN, 11.0 * PX_PER_IN)),
            "legal" => Some((8.5 * PX_PER_IN, 14.0 * PX_PER_IN)),
            "ledger" => Some((11.0 * PX_PER_IN, 17.0 * PX_PER_IN)),
            _ => None,
        }
    }

    let lower = size.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    match parts.as_slice() {
        [one] => {
            if let Some(b) = named(one) {
                return Some(b);
            }
            if *one == "portrait" {
                return named("a4");
            }
            if *one == "landscape" {
                return named("a4").map(|(w, h)| (h, w));
            }
            // 单长度 → 正方页
            match parse_length(one) {
                Some(LengthValue::Px(p)) => Some((p as f32, p as f32)),
                _ => None,
            }
        }
        [a, b] => {
            let base = named(a).or_else(|| named(b));
            let orient_is_landscape = *a == "landscape" || *b == "landscape";
            if let Some((w, h)) = base {
                // named 返回 portrait（w ≤ h）；landscape 交换两轴。
                return Some(if orient_is_landscape { (h, w) } else { (w, h) });
            }
            // `<length> <length>`（宽 高）
            match (parse_length(a), parse_length(b)) {
                (Some(LengthValue::Px(w)), Some(LengthValue::Px(h))) => Some((w as f32, h as f32)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// 解析 @page `margin` 描述符为像素 `(top, right, bottom, left)`。
///
/// 同 CSS `margin` 1-4 值简写：1 值四边同；2 值 `(top bottom, right left)`；
/// 3 值 `(top, right left, bottom)`；4 值 `(top, right, bottom, left)`。仅绝对长度
/// （px/in/cm/mm/pt/pc），相对单位 / 未知 / 空串 → `None`。
pub fn resolve_page_margin_px(margin: &str) -> Option<(f32, f32, f32, f32)> {
    use crate::values::{LengthValue, parse_length};
    let to_px = |s: &str| match parse_length(s) {
        Some(LengthValue::Px(p)) => Some(p as f32),
        _ => None,
    };
    let parts: Vec<&str> = margin.split_whitespace().collect();
    match parts.as_slice() {
        [a] => to_px(a).map(|v| (v, v, v, v)),
        [a, b] => Some((to_px(a)?, to_px(b)?, to_px(a)?, to_px(b)?)),
        [a, b, c] => Some((to_px(a)?, to_px(b)?, to_px(c)?, to_px(b)?)),
        [a, b, c, d] => Some((to_px(a)?, to_px(b)?, to_px(c)?, to_px(d)?)),
        _ => None,
    }
}
