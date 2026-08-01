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

/// 未编译的样式规则（CSS 嵌套中间结构）。
///
/// 解析阶段保留嵌套子规则为树形（父级选择器此时未知），由
/// `Parser::compile_parsed_style_rule` 自顶向下线程父级后展平为扁平 `Vec<Rule>`。
struct ParsedStyleRule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
    nested: Vec<ParsedStyleRule>,
    /// 嵌套 @规则（@media/@supports/@layer 等）：body 以 token 形式保留，编译时相对
    /// 父级重父化（合成 `&` 规则包裹 body 后编译）。
    nested_at: Vec<ParsedAtRule>,
}

/// 未编译的嵌套 @规则（CSS 嵌套中间结构）。
///
/// body 以原始 token 序列保留——编译时用 `consume_style_block_with_nesting` 在子 Parser
/// 上重解析为 `(decls, nested)`，合成 `ParsedStyleRule{ selectors:[&], decls, nested }` 相对
/// 父级编译（body 内裸声明 → 隐式 `& { ... }` → 父级；body 内嵌套规则 → 相对父级），
/// 再包裹回对应 @规则变体。
enum ParsedAtRule {
    /// @media / 通用块 @规则（@container 等下游非 media 的通用 AtRule 被忽略 = 无回归）。
    GenericBlock {
        name: String,
        prelude: String,
        body: Vec<Token>,
    },
    /// @supports：已解析条件 + body tokens。
    Supports {
        condition: SupportsCondition,
        body: Vec<Token>,
    },
    /// @layer：层名（可为空=匿名层）+ body tokens。
    Layer { name: String, body: Vec<Token> },
    /// 语句式 @规则（无块，如嵌套 @import）：原样输出。
    Statement { name: String, prelude: String },
}

/// 构造表示 `&`（嵌套选择器）的单化合物选择器，用于嵌套 @规则 body 的合成包裹。
fn amp_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Nesting],
                },
                None,
            )],
        },
    }
}

/// 构造表示 `:scope` 的选择器（顶层 `&` 的去糖目标；文档样式表中 `:scope` ≡ `:root`）。
fn scope_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "scope".to_string(),
                    ))],
                },
                None,
            )],
        },
    }
}

/// 复杂选择器是否含 `&`（化合物级 `SubclassSelector::Nesting`，或 :is/:not/:where/:has
/// 参数内嵌套）。
fn complex_contains_amp(complex: &ComplexSelector) -> bool {
    for (compound, _) in &complex.parts {
        if compound
            .subclass_selectors
            .iter()
            .any(|s| matches!(s, SubclassSelector::Nesting))
        {
            return true;
        }
        for sub in &compound.subclass_selectors {
            if let SubclassSelector::PseudoClass(pc) = sub {
                let list: Option<&Vec<Selector>> = match pc {
                    PseudoClassSelector::Is(l)
                    | PseudoClassSelector::Not(l)
                    | PseudoClassSelector::Where(l)
                    | PseudoClassSelector::Has(l) => Some(l),
                    _ => None,
                };
                if let Some(list) = list {
                    if list.iter().any(|s| complex_contains_amp(&s.complex)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 选择器列表中是否存在「`&` 出现在 `:has()` 参数内」——unsupported 相对选择器 + `&`
/// 嵌套（如 `:has(> &)`），其去糖需专门处理相对选择器隐式主题，v1 跳过（零回归）。
fn selectors_have_amp_inside_has(selectors: &[Selector]) -> bool {
    selectors.iter().any(|s| complex_has_amp_inside_has(&s.complex))
}

/// 选择器列表中是否含伪元素（`::before` 等）——用于 :is/:where/:not/:has 参数校验。
/// CSS Selectors L4：这些 functional pseudo-class 参数不得含伪元素，含则整函数
///（连同所在选择器）非法（contextually invalid，非 forgiving-skip）。
/// driving: contextually-invalid-selectors-002 `:is(*, ::before)`。
fn selectors_contain_pseudo_element(selectors: &[Selector]) -> bool {
    selectors.iter().any(|s| {
        s.complex.parts.iter().any(|(c, _)| {
            c.subclass_selectors
                .iter()
                .any(|sub| matches!(sub, SubclassSelector::PseudoElement(_)))
        })
    })
}

/// 递归检测复杂选择器内 `:has()` 参数是否含 `&`。
fn complex_has_amp_inside_has(complex: &ComplexSelector) -> bool {
    for (compound, _) in &complex.parts {
        for sub in &compound.subclass_selectors {
            if let SubclassSelector::PseudoClass(PseudoClassSelector::Has(list)) = sub {
                if list.iter().any(|s| complex_contains_amp(&s.complex)) {
                    return true;
                }
            }
        }
    }
    false
}

/// 将选择器列表相对父级列表去糖（CSS 嵌套 compile 算法）。
///
/// - 含 `&`：顶层（parent=None）替换为 `:scope`；嵌套替换为各父级化合物（交叉积）。
/// - 不含 `&` 且 parent=Some：隐式嵌套——`父级 后代 本选择器`（交叉积）。
/// - 不含 `&` 且 parent=None：顶层独立，原样返回。
fn desugar_selectors(selectors: &[Selector], parent: Option<&[Selector]>) -> Vec<Selector> {
    let mut out = Vec::new();
    for sel in selectors {
        let has_amp = complex_contains_amp(&sel.complex);
        if has_amp {
            let parents: Vec<Selector> = match parent {
                Some(p) => p.to_vec(),
                None => vec![scope_selector()],
            };
            for p in &parents {
                if let Some(d) = substitute_amp(&sel.complex, &p.complex) {
                    out.push(Selector { complex: d });
                }
            }
        } else if let Some(parents) = parent {
            for p in parents {
                out.push(prepend_descendant(&p.complex, &sel.complex));
            }
        } else {
            out.push(sel.clone());
        }
    }
    out
}

/// 隐式嵌套：`parent 后代 nested`。父级末化合物的组合器改为 Descendant（链接嵌套首化合物）。
fn prepend_descendant(parent: &ComplexSelector, nested: &ComplexSelector) -> Selector {
    let mut parts = parent.parts.clone();
    if let Some(last) = parts.last_mut() {
        last.1 = Some(Combinator::Descendant);
    } else {
        return Selector {
            complex: nested.clone(),
        };
    }
    parts.extend(nested.parts.iter().cloned());
    Selector {
        complex: ComplexSelector { parts },
    }
}

/// 把 `nested` 中每个 `&`（Nesting 化合物）替换为 `parent` 的化合物链，并递归处理
/// :is/:not/:where/:has 参数内的 `&`。组合器簿记：
/// - 父级内部组合器保留；
/// - `&` 化合物的**尾随**组合器转移到父级末化合物；
/// - `&` 化合物的**前导**组合器（在前一嵌套化合物上）自然指向父级首化合物，无需改。
fn substitute_amp(nested: &ComplexSelector, parent: &ComplexSelector) -> Option<ComplexSelector> {
    let mut new_parts: Vec<(CompoundSelector, Option<Combinator>)> = Vec::new();
    let last_parent_idx = parent.parts.len().saturating_sub(1);
    for (compound, comb) in &nested.parts {
        let has_nesting = compound
            .subclass_selectors
            .iter()
            .any(|s| matches!(s, SubclassSelector::Nesting));
        if has_nesting {
            for (pi, (p_compound, p_comb)) in parent.parts.iter().enumerate() {
                let merged = if pi == last_parent_idx {
                    merge_compound(p_compound, compound)
                } else {
                    p_compound.clone()
                };
                let use_comb = if pi == last_parent_idx { *comb } else { *p_comb };
                new_parts.push((merged, use_comb));
            }
        } else {
            // 无化合物级 `&`：但仍可能在 :is/:not/:where/:has 内 → 递归替换为 parent。
            let mut new_compound = compound.clone();
            for sub in &mut new_compound.subclass_selectors {
                if let SubclassSelector::PseudoClass(pc) = sub {
                    let list: Option<&mut Vec<Selector>> = match pc {
                        PseudoClassSelector::Is(l)
                        | PseudoClassSelector::Not(l)
                        | PseudoClassSelector::Where(l)
                        | PseudoClassSelector::Has(l) => Some(l),
                        _ => None,
                    };
                    if let Some(list) = list {
                        let mut new_list = Vec::new();
                        for inner in list.iter() {
                            if let Some(d) = substitute_amp(&inner.complex, parent) {
                                new_list.push(Selector { complex: d });
                            }
                        }
                        *list = new_list;
                    }
                }
            }
            new_parts.push((new_compound, *comb));
        }
    }
    Some(ComplexSelector { parts: new_parts })
}

/// 合并父级末化合物与嵌套化合物的非 `&` 部分（用于 `&.cls` / `div&` / `&:is()`）。
///
/// 类型选择器优先取嵌套（`div&` 的 div），嵌套无则取父级；子类 = 父级子类 + 嵌套非-Nesting
/// 子类。两类型相异（如 `span&` 父级 `div`）的矛盾情形取嵌套类型（近似，罕见边角 defer）。
fn merge_compound(parent: &CompoundSelector, nested: &CompoundSelector) -> CompoundSelector {
    let type_selector = nested.type_selector.clone().or_else(|| parent.type_selector.clone());
    let mut subclass = parent.subclass_selectors.clone();
    for s in &nested.subclass_selectors {
        if !matches!(s, SubclassSelector::Nesting) {
            subclass.push(s.clone());
        }
    }
    CompoundSelector {
        type_selector,
        subclass_selectors: subclass,
    }
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

            // 记录 consume_one_rule 前位置：若其返回空且未通过错误恢复前进，则强制
            // advance 一个保证进度；若已前进（如畸形规则恢复消耗了整段），不重复 advance
            // 以免跳过下一条合法规则的首 token（driving: matching-brackets-003）。
            let pos_before_rule = parser.pos();
            let produced = parser.consume_one_rule(None);
            if produced.is_empty() && parser.pos() == pos_before_rule {
                parser.advance();
            }
            rules.extend(produced);
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

    /// 当前位置（供调用方判断 `consume_one_rule` 等是否已通过错误恢复前进）。
    fn pos(&self) -> usize {
        self.pos
    }

    /// 跳过空白。
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Token::Whitespace | Token::Comment(_)) {
            self.advance();
        }
    }

    /// 消耗一个规则，返回**编译并展平后**的规则列表（CSS 嵌套展开为多条顶层等价规则）。
    ///
    /// `parent = None` 用于顶层 / @规则块体内（规则独立，无嵌套父级；但选择器中的 `&`
    /// 仍按 `:scope` 去糖）。`parent = Some(...)` 用于嵌套样式规则的声明块内（隐式/显式
    /// 嵌套相对父级去糖）。当前嵌套样式规则的解析由 `consume_style_block_with_nesting`
    /// 构造 `ParsedStyleRule` 树后统一编译，本函数的样式分支仅处理 parent=None 的入口。
    fn consume_one_rule(&mut self, parent: Option<&[Selector]>) -> Vec<Rule> {
        match self.peek().clone() {
            Token::AtKeyword(name) => {
                self.advance(); // @
                // 专用 at-rule 解析器。注意：这些解析器在条件/名解析失败时可能**中途**
                // 返回 None（未消费 body），故不能 early-return——须在 None 时消耗 at-rule
                // 残余（prelude + `{...}` 块），否则 body 会泄漏成顶层规则（CSS Syntax L3
                // consume_an_at_rule：at-rule 须消费全部 extent）。driving: matching-brackets-003
                // 前置 + @keyframes-no-name/missing-lbrace 不泄漏。
                let rule_opt = if name.eq_ignore_ascii_case("keyframes") {
                    self.consume_keyframes_rule().map(Rule::Keyframes)
                } else if name.eq_ignore_ascii_case("layer") {
                    self.consume_layer_rule().map(Rule::Layer)
                } else if name.eq_ignore_ascii_case("import") {
                    self.consume_import_rule().map(Rule::Import)
                } else if name.eq_ignore_ascii_case("supports") {
                    self.consume_supports_rule().map(Rule::Supports)
                } else if name.eq_ignore_ascii_case("container") {
                    self.consume_container_rule().map(Rule::Container)
                } else if name.eq_ignore_ascii_case("font-face") {
                    self.consume_font_face_rule().map(Rule::FontFace)
                } else if name.eq_ignore_ascii_case("page") {
                    self.consume_page_rule().map(Rule::Page)
                } else if name.eq_ignore_ascii_case("property") {
                    self.consume_property_rule().map(Rule::Property)
                } else if name.eq_ignore_ascii_case("counter-style") {
                    self.consume_counter_style_rule().map(Rule::CounterStyle)
                } else {
                    // 通用 at-rule：consume_at_rule 内部循环到 `;`/`{block}`，总消费全部 extent，
                    // 不触发 fallback。
                    return vec![Rule::At(self.consume_at_rule(name))];
                };
                match rule_opt {
                    Some(r) => vec![r],
                    None => {
                        // 专用 at-rule 中途失败 → 消耗残余到 `;`/`{...}` 块/EOF，避免 body 泄漏。
                        self.skip_malformed_qualified_rule();
                        vec![]
                    }
                }
            }
            _ => {
                // 样式规则：选择器 + { 声明块（可能含嵌套）}。parse_style_rule_structure
                // 解析选择器与块结构；选择器非法或块缺失时返回 None，由调用方做畸形恢复。
                match self.parse_style_rule_structure(parent.is_some()) {
                    Some(parsed) => Self::compile_parsed_style_rule(parsed, parent),
                    None => {
                        self.skip_malformed_qualified_rule();
                        vec![]
                    }
                }
            }
        }
    }

    /// CSS 嵌套 kill-switch（default-on）。`ZW_CSS_NESTING=0` 关闭 → 样式声明块不检测
    /// 嵌套规则（落 skip_malformed_declaration 丢弃，等价 R2260 前行为），保证零回归回退。
    fn nesting_enabled() -> bool {
        std::env::var("ZW_CSS_NESTING").as_deref() != Ok("0")
    }

    /// 解析样式规则结构：选择器列表 + `{` + 声明块（支持嵌套）+ `}`。
    ///
    /// `nesting=true` 时选择器按嵌套上下文解析（前导组合器注入 `&`）。返回未编译的
    /// `ParsedStyleRule` 树（嵌套子规则保留为树形），由 `compile_parsed_style_rule`
    /// 自顶向下线程父级选择器后展平。选择器非法或块缺失时返回 None（不消耗 `{`）。
    fn parse_style_rule_structure(&mut self, nesting: bool) -> Option<ParsedStyleRule> {
        let selectors = self.consume_selector_list(nesting)?;
        self.skip_whitespace();
        if !matches!(self.peek(), Token::LBrace) {
            // 畸形 qualified rule：选择器后非 `{`（driving: matching-brackets-003）。
            // 由调用方做 skip_malformed_qualified_rule 恢复。
            return None;
        }
        self.advance(); // {
        let (declarations, nested, nested_at) = self.consume_style_block_with_nesting();
        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
        }
        Some(ParsedStyleRule {
            selectors,
            declarations,
            nested,
            nested_at,
        })
    }

    /// 消耗规则列表直到 `}` / EOF（不消耗 `}`），用于 @media/@layer/@supports/@container
    /// 块体。内部用 `consume_one_rule(None)` 逐条解析并展平，含进度保证（避免死循环）。
    fn consume_rules_until_rbrace(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if matches!(self.peek(), Token::RBrace | Token::Eof) {
                break;
            }
            let pos_before = self.pos();
            let produced = self.consume_one_rule(None);
            if produced.is_empty() && self.pos() == pos_before {
                self.advance(); // 进度保证（畸形规则未前进时）
            }
            rules.extend(produced);
        }
        rules
    }

    /// 向前扫描（不消耗）判断当前位置是否起始一条嵌套规则（而非声明）。
    ///
    /// 判据：当前为 `@keyword`（嵌套 @规则），或在顶层 `}` / `;` / EOF 之前出现顶层 `{`
    ///（声明值不会含顶层 `{`，故此判据可靠）。`(`/`[`/Function 计入嵌套深度——值内的
    /// `(...)`/`[...]` 块（如 `calc()`、属性值）中的 token 不误判。
    fn peek_starts_nested_rule(&self) -> bool {
        // 嵌套 @规则（`.a { @media print { ... } }`）由 `consume_nested_at_rule` 解析，
        // 编译时 body 相对父级重父化（`.a { @media { & {...} } }` → `@media { .a {...} }`）。
        if matches!(self.peek(), Token::AtKeyword(_)) {
            return true;
        }
        let mut depth: i32 = 0;
        let mut i = self.pos;
        while let Some(tok) = self.tokens.get(i) {
            match tok {
                Token::Eof => return false,
                Token::Semicolon | Token::RBrace if depth == 0 => return false,
                Token::LBrace if depth == 0 => return true,
                Token::LParen | Token::LBracket | Token::Function(_) => depth += 1,
                Token::RParen | Token::RBracket => depth = (depth - 1).max(0),
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// 消耗样式声明块（支持 CSS 嵌套）。
    ///
    /// 返回 `(直接声明, 嵌套样式规则树, 嵌套 @规则)`。kill-switch 关闭时退化为纯声明（等价
    /// `consume_declaration_block`，嵌套规则丢弃），保证零回归回退。不消耗闭合 `}`
    ///（由 `parse_style_rule_structure` 消耗）。
    fn consume_style_block_with_nesting(&mut self) -> (Vec<Declaration>, Vec<ParsedStyleRule>, Vec<ParsedAtRule>) {
        if !Self::nesting_enabled() {
            return (self.consume_declaration_block(), vec![], vec![]);
        }
        let mut declarations = Vec::new();
        let mut nested: Vec<ParsedStyleRule> = Vec::new();
        let mut nested_at: Vec<ParsedAtRule> = Vec::new();

        loop {
            self.skip_whitespace();
            if matches!(self.peek(), Token::RBrace | Token::Eof) {
                break;
            }
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
                continue;
            }

            if matches!(self.peek(), Token::AtKeyword(_)) {
                // 嵌套 @规则：consume_nested_at_rule 消费全部 extent（prelude + `{...}`）。
                let pos_before = self.pos();
                match self.consume_nested_at_rule() {
                    Some(at) => nested_at.push(at),
                    None => {
                        if self.pos() == pos_before {
                            self.skip_malformed_qualified_rule();
                        }
                    }
                }
                continue;
            }

            if self.peek_starts_nested_rule() {
                let pos_before = self.pos();
                match self.parse_style_rule_structure(true) {
                    Some(r) => nested.push(r),
                    None => {
                        // 选择器非法：畸形 qualified rule 恢复（消耗到 `;`/`{...}`/`}`）。
                        if self.pos() == pos_before {
                            self.skip_malformed_qualified_rule();
                        }
                    }
                }
                continue;
            }

            if let Some(decl) = self.consume_declaration() {
                declarations.push(decl);
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                }
            } else {
                self.skip_malformed_declaration();
            }
        }

        (declarations, nested, nested_at)
    }

    /// 消耗嵌套 @规则（CSS 嵌套）：`@name prelude { body }` 或 `@name prelude;`。
    ///
    /// 全消费其 extent：prelude 跟踪 `()`/`[]`/Function 嵌套（块内 `;`/`}` 不终止），块体
    /// 以原始 token 序列保留（`{` 与匹配 `}` 之间的 token，不含两端括号）。条件式 @规则
    ///（@supports）解析条件就地存储；@media/@layer/通用按名 + prelude 字符串 + body 存储。
    fn consume_nested_at_rule(&mut self) -> Option<ParsedAtRule> {
        let name = match self.peek().clone() {
            Token::AtKeyword(n) => {
                self.advance();
                n
            }
            _ => return None,
        };

        let mut prelude = String::new();
        let mut group_depth: i32 = 0;
        loop {
            match self.peek() {
                Token::Eof => return None,
                Token::Semicolon | Token::RBrace if group_depth == 0 => {
                    if matches!(self.peek(), Token::Semicolon) {
                        self.advance();
                    }
                    return Some(ParsedAtRule::Statement {
                        name,
                        prelude: prelude.trim().to_string(),
                    });
                }
                Token::LBrace if group_depth == 0 => {
                    self.advance(); // {
                    let body_start = self.pos;
                    self.skip_simple_block(); // 消耗到匹配 `}`（含）
                    // body = `{` 之后到 `}` 之前（不含两端）
                    let body_end = self.pos.saturating_sub(1);
                    let body: Vec<Token> = if body_start <= body_end {
                        self.tokens[body_start..body_end].to_vec()
                    } else {
                        vec![]
                    };
                    return Some(Self::build_parsed_at_rule(&name, prelude.trim(), body));
                }
                Token::Function(_) | Token::LParen | Token::LBracket => {
                    group_depth += 1;
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
                Token::RParen | Token::RBracket => {
                    group_depth = (group_depth - 1).max(0);
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
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

    /// 按名称把 (name, prelude, body tokens) 装配为对应 `ParsedAtRule` 变体。
    fn build_parsed_at_rule(name: &str, prelude: &str, body: Vec<Token>) -> ParsedAtRule {
        if name.eq_ignore_ascii_case("supports") {
            if let Some(cond) = crate::supports_condition::parse_supports_condition(prelude) {
                return ParsedAtRule::Supports { condition: cond, body };
            }
        } else if name.eq_ignore_ascii_case("layer") {
            return ParsedAtRule::Layer {
                name: prelude.to_string(),
                body,
            };
        }
        // @media（下游评估 media query）/ @container / 通用 / @supports 条件解析失败 →
        // GenericBlock（下游仅处理 name=media，其余被忽略 = 无回归）。
        ParsedAtRule::GenericBlock {
            name: name.to_ascii_lowercase(),
            prelude: prelude.to_string(),
            body,
        }
    }

    /// 自顶向下编译 `ParsedStyleRule` 树为扁平 `Vec<Rule>`，线程父级选择器。
    ///
    /// - 自身选择器经 `desugar_selectors` 相对 `parent` 去糖（顶层无 `&` → 原样；
    ///   顶层含 `&` → `:scope`；嵌套含 `&` → 替换父级；嵌套无 `&` → 隐式后代前缀）。
    /// - 去糖后的自身选择器成为子规则的父级（递归）。
    /// - **unsupported 守卫**：`&` 出现在 `:has()` 参数内（相对选择器 + `&`，如 `:has(> &)`）
    ///   的嵌套规则被跳过——该组合的相对选择器去糖需专门处理（R2260 defer），跳过 = 丢弃 =
    ///   R2260 前行为，零回归。
    fn compile_parsed_style_rule(rule: ParsedStyleRule, parent: Option<&[Selector]>) -> Vec<Rule> {
        let own = desugar_selectors(&rule.selectors, parent);
        let mut out = Vec::new();
        if !own.is_empty() {
            out.push(Rule::Style(StyleRule {
                selectors: own.clone(),
                declarations: rule.declarations,
            }));
        }
        for child in rule.nested {
            if selectors_have_amp_inside_has(&child.selectors) {
                continue; // unsupported 相对选择器 + & 嵌套：跳过（零回归）
            }
            out.extend(Self::compile_parsed_style_rule(child, Some(&own)));
        }
        for at in rule.nested_at {
            // 嵌套 @规则 body 相对父级 own 重父化：body 在子 Parser 上重解析为声明+嵌套树，
            // 合成 `&` 规则包裹后相对 own 编译（裸声明 → 隐式 & → own；嵌套规则 → 相对 own），
            // 再包裹回对应 @规则变体。
            if let Some(r) = Self::compile_nested_at_rule(at, Some(&own)) {
                out.push(r);
            }
        }
        out
    }

    /// 编译嵌套 @规则：body tokens 重解析 + 合成 `&` 包裹 + 相对父级编译 + 包裹回 @变体。
    fn compile_nested_at_rule(at: ParsedAtRule, parent: Option<&[Selector]>) -> Option<Rule> {
        match at {
            ParsedAtRule::Statement { name, prelude } => Some(Rule::At(AtRule {
                name,
                prelude,
                body: AtRuleBody::Statement,
            })),
            ParsedAtRule::GenericBlock { name, prelude, body } => {
                let rules = Self::compile_at_body(body, parent);
                Some(Rule::At(AtRule {
                    name,
                    prelude,
                    body: AtRuleBody::Block(rules),
                }))
            }
            ParsedAtRule::Supports { condition, body } => {
                let rules = Self::compile_at_body(body, parent);
                Some(Rule::Supports(SupportsRule { condition, rules }))
            }
            ParsedAtRule::Layer { name, body } => {
                let rules = Self::compile_at_body(body, parent);
                Some(Rule::Layer(LayerRule { name, rules }))
            }
        }
    }

    /// 在子 Parser 上重解析 @规则 body tokens 为声明+嵌套树，合成 `&` 规则相对 `parent`
    /// 编译，返回扁平 body 规则列表。body 内裸声明经合成 `&` → `parent`（隐式 `& { ... }`），
    /// body 内嵌套规则相对 `parent` 去糖——与父样式规则 body 语义一致。
    fn compile_at_body(body_tokens: Vec<Token>, parent: Option<&[Selector]>) -> Vec<Rule> {
        if body_tokens.is_empty() {
            return vec![];
        }
        let mut toks = body_tokens;
        toks.push(Token::Eof);
        let mut sub = Parser::new(&mut toks);
        let (declarations, nested, nested_at) = sub.consume_style_block_with_nesting();
        let synth = ParsedStyleRule {
            selectors: vec![amp_selector()],
            declarations,
            nested,
            nested_at,
        };
        Self::compile_parsed_style_rule(synth, parent)
    }

    /// 消耗选择器列表。
    ///
    /// `nesting=true` 时（CSS 嵌套上下文：规则位于父样式规则的声明块内），前导组合器
    /// （`> .c` / `+ .c` / `~ .c`）注入嵌套选择器 `&` 作为隐式主题（→ `& > .c`），
    /// 而非通用选择器 `*`。`&` 本身始终由 `consume_compound_selector` 解析为
    /// `SubclassSelector::Nesting`（与 nesting 标志无关）。
    fn consume_selector_list(&mut self, nesting: bool) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            if matches!(self.peek(), Token::LBrace | Token::Eof) {
                break;
            }

            // CSS Selectors L3：选择器列表中任一非法选择器（如未知伪类）invalidates 整个列表
            // → 整条规则丢弃（driving: selectors-parsing-001 `p:invalidPseudoClass, p.test1`）。
            // 旧实现跳过 None 选择器继续，导致同组其他合法选择器（如 p.test1）泄漏应用。
            let sel = self.consume_selector(nesting)?;
            selectors.push(sel);

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
    ///
    /// `nesting=true` 时前导组合器注入 `&`（嵌套选择器）而非 `*`（通用选择器）——
    /// 见 `consume_selector_list` 文档。
    fn consume_selector(&mut self, nesting: bool) -> Option<Selector> {
        let mut parts = Vec::new();

        loop {
            self.skip_whitespace();

            // 检查是否到达选择器列表的结束位置
            if matches!(self.peek(), Token::LBrace | Token::Comma | Token::RBrace | Token::Eof) {
                break;
            }

            // 处理前导组合器（如 :has(> .child) / 嵌套 `> .c`），隐式添加主题化合物：
            // 嵌套上下文 → `&`（SubclassSelector::Nesting），否则 → `*`（Universal）。
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
                // 隐式主题：嵌套上下文用 `&`（SubclassSelector::Nesting），否则通用选择器 `*`。
                // `&` 经 compile 算法替换为父级化合物；`:has(> .c)` 等函数参数仍用 `*`。
                let implicit = if nesting {
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Nesting],
                    }
                } else {
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    }
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

    /// 已知**简单**伪类名（非函数形式）。CSS Selectors L4 + 常见浏览器伪类。
    /// vendor 前缀（`-` 开头，如 `-webkit-`/`-moz-`）视为已知（容忍）。
    /// 未知伪类使选择器非法（CSS Selectors L3 §13：未知伪类 invalidates 选择器 → 整个选择器
    /// 列表非法 → 规则丢弃）。driving: selectors-parsing-001 `p:invalidPseudoClass, p.test1 {...}`
    /// —— `:invalidPseudoClass` 未知故整条规则（含 `p.test1`）应丢弃，`p.test1` 不应用。
    fn is_known_simple_pseudo_class(name: &str) -> bool {
        if name.starts_with('-') {
            return true; // vendor 前缀容忍
        }
        matches!(
            name,
            "link"
                | "visited"
                | "hover"
                | "active"
                | "focus"
                | "focus-visible"
                | "focus-within"
                | "any-link"
                | "local-link"
                | "target"
                | "target-within"
                | "scope"
                | "root"
                | "empty"
                | "blank"
                | "first-child"
                | "last-child"
                | "only-child"
                | "first-of-type"
                | "last-of-type"
                | "only-of-type"
                | "enabled"
                | "disabled"
                | "checked"
                | "indeterminate"
                | "default"
                | "required"
                | "optional"
                | "valid"
                | "invalid"
                | "in-range"
                | "out-of-range"
                | "placeholder-shown"
                | "read-only"
                | "read-write"
                | "user-invalid"
                | "defined"
                | "host"
                | "fullscreen"
                | "modal"
                | "picture-in-picture"
                | "playing"
                | "paused"
                | "muted"
                | "seeking"
                | "buffering"
                | "stalled"
                | "volume-locked"
                | "autoplay"
                | "locked"
                | "current"
                | "past"
                | "future"
                | "spatial-navigation"
                | "inert"
                | "has-slotted"
                | "open"
                | "closed"
                | "top-layer"
        )
    }

    /// 已知函数伪类名（`name(...)` 形式，未被上方 match 显式分发处理的）。
    fn is_known_function_pseudo_class(name: &str) -> bool {
        if name.starts_with('-') {
            return true;
        }
        matches!(
            name,
            "dir" | "matches" | "any" | "host-context" | "nth-col" | "nth-last-col"
        )
    }

    /// 已知伪元素名。vendor 前缀 + `view-transition-*` 视为已知。
    fn is_known_pseudo_element(name: &str) -> bool {
        if name.starts_with('-') || name.starts_with("view-transition-") {
            return true;
        }
        matches!(
            name,
            "first-line"
                | "first-letter"
                | "before"
                | "after"
                | "selection"
                | "placeholder"
                | "backdrop"
                | "marker"
                | "cue"
                | "grammar-error"
                | "spelling-error"
                | "file-selector-button"
                | "details-content"
                | "search-text"
                | "target-text"
                | "view-transition"
                | "highlight"
                | "shadow-tree"
                | "content"
        )
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
                            // 未知伪元素使选择器非法（CSS Selectors L3），返回 None
                            if !Self::is_known_pseudo_element(&name) {
                                return None;
                            }
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
                                "not" => self.parse_pseudo_class_function_list("not")?,
                                "is" => self.parse_pseudo_class_function_list("is")?,
                                "where" => self.parse_pseudo_class_function_list("where")?,
                                "has" => self.parse_pseudo_class_function_list("has")?,
                                "nth-child" => self.parse_nth_pattern("nth-child")?,
                                "nth-last-child" => self.parse_nth_pattern("nth-last-child")?,
                                "nth-of-type" => self.parse_nth_pattern("nth-of-type")?,
                                "nth-last-of-type" => self.parse_nth_last_of_type_pattern()?,
                                "lang" => self.parse_lang(),
                                "dir" => self.parse_dir(),
                                _ => {
                                    // 未知函数伪类（如 `:foo(...)`）使选择器非法：整块消耗参数
                                    //（含 Function token 的 `(`）后再判非法，避免参数碎片化泄漏。
                                    if !Self::is_known_function_pseudo_class(&name) {
                                        self.consume_balanced_function_args();
                                        return None;
                                    }
                                    PseudoClassSelector::Simple(name)
                                }
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
                                    // 未知简单伪类（如 `:invalidPseudoClass`）使选择器非法
                                    if !Self::is_known_simple_pseudo_class(&name) {
                                        return None;
                                    }
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
                            "not" => self.parse_pseudo_class_function_list("not")?,
                            "is" => self.parse_pseudo_class_function_list("is")?,
                            "where" => self.parse_pseudo_class_function_list("where")?,
                            "has" => self.parse_pseudo_class_function_list("has")?,
                            "nth-child" => self.parse_nth_pattern("nth-child")?,
                            "nth-last-child" => self.parse_nth_pattern("nth-last-child")?,
                            "nth-of-type" => self.parse_nth_pattern("nth-of-type")?,
                            "nth-last-of-type" => self.parse_nth_last_of_type_pattern()?,
                            "lang" => self.parse_lang(),
                            "dir" => self.parse_dir(),
                            _ => {
                                // 未知函数伪类（Function token 形式）使选择器非法：整块消耗参数
                                //（Function token 的 `(` 已含）后再判非法，避免参数碎片化泄漏。
                                if !Self::is_known_function_pseudo_class(&name) {
                                    self.consume_balanced_function_args();
                                    return None;
                                }
                                PseudoClassSelector::Simple(name)
                            }
                        };
                        subclass_selectors.push(SubclassSelector::PseudoClass(pseudo));
                    }
                }
                // CSS 嵌套选择器 `&`（可出现在复合选择器任意位置：`&`、`&.cls`、`div&`、`&:is()`）。
                // 解析为 SubclassSelector::Nesting 标记，由 compile 算法（compile_style_rule）
                // 替换为父级化合物；非嵌套上下文（顶层）的 `&` 由 desugar 解析为 `:scope`。
                Token::Delim('&') => {
                    subclass_selectors.push(SubclassSelector::Nesting);
                    self.advance();
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
    fn parse_pseudo_class_function_list(&mut self, name: &str) -> Option<PseudoClassSelector> {
        // Selectors L4：:is()/:where()/:has() 取 forgiving selector list（无效选择器跳过而非吞整列表）；
        // :not() 与 nth `of S` 取普通列表（无效即停）。
        let forgiving = matches!(name, "is" | "where" | "has");
        let selectors = self.consume_selector_list_for_function(forgiving);

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        // CSS Selectors L4：:is/:where/:not/:has 参数不得含伪元素——含则整函数（连同所在
        // 选择器）非法。伪元素在这些 functional pseudo-class 中是 contextually invalid，使整个
        // 函数失效（非 forgiving-skip——forgiving 仅跳过未知/非法选择器，伪元素是更强的约束）。
        // driving: contextually-invalid-selectors-002 `:is(*, ::before)`（应不匹配、零特异性）。
        if selectors_contain_pseudo_element(&selectors) {
            return None;
        }

        let pseudo = match name {
            "not" => PseudoClassSelector::Not(selectors),
            "is" => PseudoClassSelector::Is(selectors),
            "where" => PseudoClassSelector::Where(selectors),
            "has" => PseudoClassSelector::Has(selectors),
            _ => PseudoClassSelector::Simple(name.to_string()),
        };
        Some(pseudo)
    }

    /// 为函数伪类内部消耗选择器列表。
    ///
    /// `forgiving=true`（:is/:where/:has）时，遇无效选择器跳过其残余到下一个逗号或 `)`，
    /// 继续解析后续选择器（spec 的 forgiving selector list）；`false`（:not、nth `of S`）时，
    /// 遇无效即停（普通选择器列表）。
    fn consume_selector_list_for_function(&mut self, forgiving: bool) -> Vec<Selector> {
        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();

            if matches!(self.peek(), Token::RParen | Token::Eof) {
                break;
            }

            let arg_start = self.pos;
            match self.consume_selector(false) {
                Some(sel) => selectors.push(sel),
                None => {
                    if !forgiving {
                        break;
                    }
                    // forgiving：跳过无效选择器残余到逗号或 `)`（不消耗目标 token）。
                    // CSS Nesting：若被跳过的参数含 `&`（如 `:is(.a, !&)`、`:is(.a, :unknown(&))`），
                    // 注入一个 bare-`&` 标记选择器——使外层嵌套按**显式嵌套**去糖（& 替换父级），
                    // 而非隐式后代前缀。这样 `:is(.a, !&)` → `:is(.a, <父级>)` 直接匹配 .a，
                    // 而非 `<父级> :is(.a)`（父级不存在则永不匹配）。driving: nest-containing-forgiving。
                    // 扫描从 arg_start（consume_selector 之前）到 skip 后——覆盖 consume_compound
                    // 已整块消耗的未知函数参数内的 `&`（如 `:unknown(div,&)`）。
                    self.skip_to_comma_or_rparen();
                    let scan_end = self.pos;
                    let had_amp = self
                        .tokens
                        .get(arg_start..scan_end)
                        .is_some_and(|toks| toks.iter().any(|t| matches!(t, Token::Delim('&'))));
                    if had_amp {
                        selectors.push(amp_selector());
                    }
                    // 若停在 `)`/EOF（无后续逗号），结束
                    if !matches!(self.peek(), Token::Comma) {
                        break;
                    }
                }
            }

            // 成功选择器后，或 forgiving 恢复停在逗号后：消耗逗号继续
            self.skip_whitespace();
            if matches!(self.peek(), Token::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        selectors
    }

    /// 跳过 token 直到遇到 `,`、`)` 或 EOF（不消耗目标 token）。用于 forgiving 选择器列表恢复。
    fn skip_to_comma_or_rparen(&mut self) {
        while !matches!(self.peek(), Token::Comma | Token::RParen | Token::Eof) {
            self.advance();
        }
    }

    /// 消耗「`(` 已消费」（含 Function token 的 `name(`）的函数参数到匹配的 `)`。
    /// 用于未知函数伪类失败时**整块**跳过参数——避免参数内的逗号/`&` 碎片化（旧实现 `return None`
    /// 后由 `skip_to_comma_or_rparen` 接手，遇参数内首个 `,`/`)` 即停，泄漏 `:is(.a, :unknown(b,c))`
    /// 的内层 `c` 为 :is 成员；且无法扫描到 `:unknown(div,&)` 内的 `&`）。driving: nest-containing-forgiving。
    fn consume_balanced_function_args(&mut self) {
        let mut depth: i32 = 1; // `(` 已消费
        while depth > 0 {
            match self.peek() {
                Token::Eof => return,
                Token::LParen | Token::LBracket | Token::Function(_) => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen | Token::RBracket => {
                    depth -= 1;
                    self.advance();
                }
                _ => self.advance(),
            }
        }
    }

    /// 解析 nth 函数模式（:nth-child、:nth-last-child、:nth-of-type）。
    ///
    /// 调用前已消耗 `(`。`:nth-child`/`:nth-last-child` 支持 Selectors L4 的 `of S`
    /// 选择器参数；`:nth-of-type`/`:nth-last-of-type` 不支持 `of S`（出现 `of` → 非法）。
    /// 非法 An+B 或 `of` 后空选择器列表 → 返回 None（选择器非法 → 整条规则丢弃）。
    fn parse_nth_pattern(&mut self, name: &str) -> Option<PseudoClassSelector> {
        match name {
            "nth-child" | "nth-last-child" => {
                let (pattern, of_selectors) = self.parse_nth_with_optional_of()?;
                Some(if of_selectors.is_empty() {
                    if name == "nth-child" {
                        PseudoClassSelector::NthChild(pattern)
                    } else {
                        PseudoClassSelector::NthLastChild(pattern)
                    }
                } else if name == "nth-child" {
                    PseudoClassSelector::NthChildOf(pattern, of_selectors)
                } else {
                    PseudoClassSelector::NthLastChildOf(pattern, of_selectors)
                })
            }
            _ => Some(PseudoClassSelector::NthOfType(self.parse_nth_expression()?)),
        }
    }

    /// 解析 nth 表达式，并可选地解析 L4 `of S` 选择器列表（消耗到 `)`）。
    ///
    /// An+B 用 [`consume_an_plus_b`] 严格校验；`of` 后选择器列表为空 → None（非法）。
    fn parse_nth_with_optional_of(&mut self) -> Option<(NthPattern, Vec<Selector>)> {
        let pattern = self.consume_an_plus_b()?;

        // 检测可选 `of S`（consume_an_plus_b 已停在 `)`/`of`/EOF，可能停在 of 前的空白上）
        let saved = self.pos;
        self.skip_whitespace();
        let of_selectors = if matches!(self.peek(), Token::Ident(s) if s.eq_ignore_ascii_case("of")) {
            self.advance(); // 消耗 of
            let sels = self.consume_selector_list_for_function(false);
            if sels.is_empty() {
                return None;
            }
            sels
        } else {
            self.pos = saved; // 回退空白
            Vec::new()
        };

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }
        Some((pattern, of_selectors))
    }

    /// 解析 nth-last-of-type 函数模式（消耗到 `)`）。
    fn parse_nth_last_of_type_pattern(&mut self) -> Option<PseudoClassSelector> {
        Some(PseudoClassSelector::NthLastOfType(self.parse_nth_expression()?))
    }

    /// 解析 nth 表达式（:nth-of-type / :nth-last-of-type 路径）。
    /// of-type 系不支持 `of S`：An+B 后出现 `of` → None（非法）。
    fn parse_nth_expression(&mut self) -> Option<NthPattern> {
        let pattern = self.consume_an_plus_b()?;
        let saved = self.pos;
        self.skip_whitespace();
        if matches!(self.peek(), Token::Ident(s) if s.eq_ignore_ascii_case("of")) {
            return None; // of-type 不支持 of
        }
        self.pos = saved;
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }
        Some(pattern)
    }

    /// 从 token 流解析 An+B（CSS Values §7.1.1）。
    ///
    /// 消耗 An+B token 与其后空白，停在 `)` / `of` 关键字 / EOF（of 不消耗，且回退其前空白
    /// 以便上层 of 检测统一）。非法形式（如 `1 n`、`even .x`、`of`、`n-1of`、`+`、空）→ None。
    fn consume_an_plus_b(&mut self) -> Option<NthPattern> {
        self.skip_whitespace();
        // (a, 粘合在 token 内的 B)。粘合形式见 parse_n_ident / parse_dim_unit（tokenizer 把
        // `n-1` 粘进 ident/dimension 单位，因 `-` 是 name-char）。
        let (a, glued_b) = match self.peek().clone() {
            Token::Ident(s) => {
                let l = s.to_ascii_lowercase();
                if l == "odd" {
                    self.advance();
                    return self.finish_an_plus_b(2, 1);
                }
                if l == "even" {
                    self.advance();
                    return self.finish_an_plus_b(2, 0);
                }
                let (a, b) = Self::parse_n_ident(&l)?;
                self.advance();
                (a, b)
            }
            Token::Dimension(v, ref unit) => {
                let (a, b) = Self::parse_dim_unit(v, unit)?;
                self.advance();
                (a, b)
            }
            Token::Delim('+') => {
                // `+n` / `+n-1` 形式（`+` 后跟 ident n...；`+2n` 会被 tokenizer 合成 Dimension，不落此）
                self.advance();
                self.skip_whitespace();
                match self.peek().clone() {
                    Token::Ident(s) => {
                        let (a, b) = Self::parse_n_ident(&s.to_ascii_lowercase())?;
                        self.advance();
                        (a, b)
                    }
                    _ => return None,
                }
            }
            Token::Number(v) => {
                // 纯整数 B（无 n）。`An` 形式由 tokenizer 合成 Dimension，不会落此。
                self.advance();
                return self.finish_an_plus_b(0, v as i32);
            }
            _ => return None,
        };

        // n-form：粘合 B 已在 token 内 → 直接收尾；否则消耗可选的独立 B token
        match glued_b {
            Some(b) => self.finish_an_plus_b(a, b),
            None => self.consume_nth_b(a),
        }
    }

    /// 解析 n-form ident（无数值系数）：`n` / `-n` / `+n` / `n-1` / `-n-1` / `n+2`...
    /// 返回 (a, Option<b>)。b 仅当 tokenizer 把 `-<int>`/`+<int>` 粘进 ident 时存在。
    /// 输入须为小写。非法（如 `n-1of`、`even`、`of`）→ None。
    fn parse_n_ident(s: &str) -> Option<(i32, Option<i32>)> {
        let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
            (1, r)
        } else if let Some(r) = s.strip_prefix('-') {
            (-1, r)
        } else {
            (1, s)
        };
        let rest = rest.strip_prefix('n')?;
        if rest.is_empty() {
            return Some((sign, None));
        }
        // 剩余须为 signed-integer（如 "-1"、"+3"），否则非法（如 "-1of"）
        let b: i32 = rest.parse().ok()?;
        Some((sign, Some(b)))
    }

    /// 解析 n-dimension 的单位：value 是系数 a，unit 是 `n` 或 `n<signed-int>`
    ///（tokenizer 把 `2n-1` 粘成 Dimension(2,"n-1")）。返回 (a, Option<b>)。非法 → None。
    fn parse_dim_unit(value: f64, unit: &str) -> Option<(i32, Option<i32>)> {
        let l = unit.to_ascii_lowercase();
        if l == "n" {
            return Some((value as i32, None));
        }
        let rest = l.strip_prefix('n')?;
        let b: i32 = rest.parse().ok()?;
        Some((value as i32, Some(b)))
    }

    /// An+B 的 B 部分（n-form 之后）。消耗可选 `<signed-integer>` 或 `['+'|'-'] <signless-integer>`。
    fn consume_nth_b(&mut self, a: i32) -> Option<NthPattern> {
        let saved = self.pos;
        self.skip_whitespace();
        match self.peek().clone() {
            Token::RParen | Token::Eof => Some(NthPattern { a, b: 0 }),
            Token::Ident(s) if s.eq_ignore_ascii_case("of") => {
                self.pos = saved; // 回退空白，让上层 of 检测定位
                Some(NthPattern { a, b: 0 })
            }
            Token::Number(v) => {
                let b = v as i32;
                self.advance();
                self.finish_an_plus_b(a, b)
            }
            Token::Delim('+') | Token::Delim('-') | Token::Ident(_) if Self::is_nth_sign(self.peek()) => {
                // `['+'|'-'] <signless-integer>`：`+` 为 Delim，孤立 `-` 为 Ident
                //（tokenizer 把不跟数字/标识符的 `-` 作 Ident("-")）。
                let neg =
                    matches!(self.peek(), Token::Delim('-')) || matches!(self.peek(), Token::Ident(s) if s == "-");
                self.advance();
                self.skip_whitespace();
                match self.peek() {
                    Token::Number(v) => {
                        let b = if neg { -(*v as i32) } else { *v as i32 };
                        self.advance();
                        self.finish_an_plus_b(a, b)
                    }
                    _ => None,
                }
            }
            _ => None, // 残余非法 token（如 `2n.x` 的 `.`、`n + of` 的 `of`）
        }
    }

    /// An+B 中的 `+`/`-` 符号判定：Delim('+')/Delim('-') 或孤立 Ident("+")/Ident("-")。
    fn is_nth_sign(tok: &Token) -> bool {
        matches!(tok, Token::Delim('+') | Token::Delim('-')) || matches!(tok, Token::Ident(s) if s == "+" || s == "-")
    }

    /// An+B 结束校验：An+B 完成后须紧跟 `)` / `of` / EOF（空白可 intervening）。
    /// 成功时若停在 `of`，回退其前空白以便上层 of 检测统一；停在 `)`/EOF 则保持。
    fn finish_an_plus_b(&mut self, a: i32, b: i32) -> Option<NthPattern> {
        let saved = self.pos;
        self.skip_whitespace();
        match self.peek() {
            Token::RParen | Token::Eof => Some(NthPattern { a, b }),
            Token::Ident(s) if s.eq_ignore_ascii_case("of") => {
                self.pos = saved; // 回退空白
                Some(NthPattern { a, b })
            }
            _ => None, // 残余非法 token（如 `even .x` 的 `.`、`1 n` 的 `n`、`2px)` 的 px 已在 Dimension 排除）
        }
    }

    /// 解析 `:lang()` 函数（CSS Selectors L4 §14）。
    ///
    /// 调用前已消耗 `(`。取逗号分隔的语言范围列表，每项为 ident、string 或 BCP 47 通配
    /// （`*`、`*-CA`），以 `)` 结束。修复前仅读单项后只在紧跟 `)` 时消耗它，`:lang(en, fr)`
    /// 的 `,` 致 `)` 未消耗 → 残余 token 吞规则（R2204/R2208 同族 bug）。
    fn parse_lang(&mut self) -> PseudoClassSelector {
        let mut ranges = Vec::new();
        loop {
            self.skip_whitespace();
            let range = self.parse_lang_range();
            if !range.is_empty() {
                ranges.push(range);
            }
            self.skip_whitespace();
            match self.peek() {
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::RParen => {
                    self.advance();
                    break;
                }
                _ => break,
            }
        }
        PseudoClassSelector::Lang(ranges)
    }

    /// 读取单个 `:lang()` 语言范围（直到 `,` 或 `)`）。范围为 BCP 47 子标签序列（`-` 分隔），
    /// 子标签可为 `*`（通配）或 ident 字符；quoted string 取整体。`*-CA` 经 tokenizer 为
    /// `Delim('*')` + `Ident("-CA")`，逐 token 累加得 `"*-CA"`。
    fn parse_lang_range(&mut self) -> String {
        let mut s = String::new();
        loop {
            match self.peek().clone() {
                Token::String(t) => {
                    self.advance();
                    return t;
                }
                Token::Delim(c) if c == '*' || c == '-' => {
                    self.advance();
                    s.push(c);
                }
                Token::Ident(t) => {
                    self.advance();
                    s.push_str(&t);
                }
                _ => break,
            }
        }
        s
    }

    /// 解析 `:dir(ltr|rtl)` 参数。调用前已消耗 `(`，本方法消耗参数标识与 `)`。
    /// 参数归一化为小写（HTML `dir` 属性值 ASCII 大小写不敏感）。
    fn parse_dir(&mut self) -> PseudoClassSelector {
        self.skip_whitespace();

        let dir = match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                s.to_ascii_lowercase()
            }
            _ => String::new(),
        };

        self.skip_whitespace();

        // 消耗右括号
        if matches!(self.peek(), Token::RParen) {
            self.advance();
        }

        PseudoClassSelector::Dir(dir)
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
                    case: AttrCaseModifier::Default,
                });
            }
            Token::Delim('=') => {
                // [attr=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::Exact(val)
            }
            Token::IncludeMatch => {
                // [attr~=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::Includes(val)
            }
            Token::DashMatch => {
                // [attr|=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::DashMatch(val)
            }
            Token::PrefixMatch => {
                // [attr^=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::Prefix(val)
            }
            Token::SuffixMatch => {
                // [attr$=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::Suffix(val)
            }
            Token::SubstringMatch => {
                // [attr*=val]
                self.advance();
                self.skip_whitespace();
                let val = self.consume_attribute_value();
                AttributeMatcher::Substring(val)
            }
            _ => {
                // 未知匹配器，跳到 ]
                self.skip_to_rbracket();
                return Some(AttributeSelector {
                    name,
                    matcher: AttributeMatcher::Exists,
                    case: AttrCaseModifier::Default,
                });
            }
        };

        // Selectors Level 4：取值后可选空白 + `i`/`s` 大小写修饰符 + 可选空白再 `]`。
        // 修复前各 matcher arm 自行「紧跟 ] 才消耗 ]」，遇 `i`/`s` 时 ] 不消耗 → 残余
        // `i` `]` 破坏选择器解析、整条规则被丢（driving: attribute_case_flag）。现统一在
        // 取值后消耗修饰符与 ]。`i` → Insensitive、`s` → Sensitive、缺省 → Default。
        self.skip_whitespace();
        let case = self.consume_attr_case_flag();
        self.skip_whitespace();
        if matches!(self.peek(), Token::RBracket) {
            self.advance();
        }
        Some(AttributeSelector { name, matcher, case })
    }

    /// Selectors Level 4：消耗属性选择器值后可选的大小写修饰符（`i`/`s`）。
    /// `i`/`I` → [`AttrCaseModifier::Insensitive`]，`s`/`S` → [`AttrCaseModifier::Sensitive`]，
    /// 无修饰符 → [`AttrCaseModifier::Default`]。
    fn consume_attr_case_flag(&mut self) -> AttrCaseModifier {
        if let Token::Ident(s) = self.peek().clone()
            && (s == "i" || s == "I" || s == "s" || s == "S")
        {
            self.advance();
            match s.as_str() {
                "i" | "I" => AttrCaseModifier::Insensitive,
                _ => AttrCaseModifier::Sensitive,
            }
        } else {
            AttrCaseModifier::Default
        }
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

    /// 消耗一个平衡块：当前 token 须是开块符（`{` / `(` / `[`），消耗到匹配闭块符，
    /// 期间所有开块符 `+1`、闭块符 `-1`（统一计数，足以跳过畸形区域；token 级别字符串/
    /// 转义已由 tokenizer 处理）。depth 回到 0 即返回。
    fn skip_simple_block(&mut self) {
        let mut depth: i32 = 0;
        loop {
            match self.peek() {
                Token::Eof => return,
                Token::LBrace | Token::LParen | Token::LBracket => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace | Token::RParen | Token::RBracket => {
                    depth -= 1;
                    self.advance();
                    if depth <= 0 {
                        return;
                    }
                }
                _ => self.advance(),
            }
        }
    }

    /// 畸形 qualified rule 错误恢复（CSS 2.1 §4.2「Malformed statements」/ CSS Syntax L3
    /// consume_a_qualified_rule）：消耗 prelude 残余直到非嵌套的 `{`（消耗其 `{...}` 块）
    /// 或 `;`/EOF。prelude 内的 `()`/`[]` 块**整块跳过**（块内 `{`/`}` 不提前终止）。
    ///
    /// driving: matching-brackets-003 `p ( { border...; } p { background...; } ) p { color:red }`
    ///——`p (` 选择器后非 `{`，`(...)` 块内含 `{...}`/`p {...}`，须把整个 `(...)` 作 prelude
    /// 一部分整块消耗，再到真 `{`（规则 3 的块），整条作为一条畸形 qualified rule 丢弃
    ///（prelude `p(...)p` 非法），故规则 3 不应用，`<p>` 保持规则 1 的 green。
    fn skip_malformed_qualified_rule(&mut self) {
        loop {
            match self.peek() {
                Token::Eof => return,
                Token::Semicolon => {
                    self.advance();
                    return;
                }
                Token::LBrace => {
                    // 顶层 `{` = 规则块开始，消耗整个 {...} 块，结束恢复
                    self.skip_simple_block();
                    return;
                }
                Token::LParen | Token::LBracket => {
                    // prelude 内的 ()/[] 块：整块消耗（内部 { } 受保护）
                    self.skip_simple_block();
                }
                Token::RBrace => {
                    // 顶层多余 `}`：不消耗，返回让 parse_stylesheet 在下一轮识别
                    return;
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

        // 收集前导部分。跟踪 `()`/`[]`（含 Function token 隐含的 `(`）嵌套：depth>0 时
        // 块内的 `;`/`{`/`}` 不作终止符（CSS Syntax L3 consume_a_qualified_rule / consume_an_at_rule
        // 「observing nesting」）。`}` 在 depth==0 时是外层块结束，不消耗（让调用方识别）——
        // 旧实现把 `}` 收进 prelude 会吞掉外层块的闭合，driving: matching-brackets-001
        // `@foo ] } ) ...`（@media 内畸形 @foo，`}` 应闭 @media 而非进 @foo prelude）。
        let mut prelude = String::new();
        let mut group_depth: i32 = 0;
        loop {
            match self.peek() {
                Token::Semicolon if group_depth == 0 => {
                    self.advance();
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Statement,
                    };
                }
                Token::LBrace if group_depth == 0 => {
                    self.advance();
                    let rules = self.consume_rules_until_rbrace();
                    self.skip_whitespace();
                    if matches!(self.peek(), Token::RBrace) {
                        self.advance();
                    }
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Block(rules),
                    };
                }
                Token::RBrace if group_depth == 0 => {
                    // 外层块结束：不消耗 `}`，返回（让外层 consume_declaration_block / @media
                    // body 循环识别）。prelude 为已收集部分（可能为畸形 at-rule 语句）。
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Statement,
                    };
                }
                Token::Eof => {
                    return AtRule {
                        name,
                        prelude: prelude.trim().to_string(),
                        body: AtRuleBody::Statement,
                    };
                }
                Token::Function(_) | Token::LParen | Token::LBracket => {
                    group_depth += 1;
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
                Token::RParen | Token::RBracket => {
                    group_depth = (group_depth - 1).max(0);
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
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
        let mut weight: Option<u16> = None;
        for decl in &declarations {
            if decl.property.eq_ignore_ascii_case("font-family") {
                family = strip_css_quotes(decl.value.trim());
            } else if decl.property.eq_ignore_ascii_case("src") {
                for url in extract_urls_from_src(&decl.value) {
                    sources.push(url);
                }
            } else if decl.property.eq_ignore_ascii_case("font-weight") {
                weight = Self::parse_font_face_weight(&decl.value);
            }
        }

        if family.is_empty() || sources.is_empty() {
            return None;
        }

        Some(FontFaceRule {
            family,
            sources,
            weight,
        })
    }

    /// 解析 `@font-face` 的 `font-weight` 描述符为绝对权重（R2417 font-weight matching）。
    ///
    /// `normal`→400、`bold`→700、数字（100-900）原值；`lighter`/`bolder`（相对，@font-face
    /// 描述符无父上下文）或无法识别 → `None`（调用方视为 normal/400，不构粗体键）。
    fn parse_font_face_weight(value: &str) -> Option<u16> {
        let v = value.trim();
        if v.eq_ignore_ascii_case("normal") {
            return Some(400);
        }
        if v.eq_ignore_ascii_case("bold") {
            return Some(700);
        }
        let n: u16 = v.parse().ok()?;
        (100..=900).contains(&n).then_some(n)
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

    /// 消耗 `@property` 规则（CSS Properties and Values API Level 1）。
    ///
    /// 格式：`@property --foo { syntax: "<color>"; inherits: false; initial-value: #c0ffee; }`
    /// prelude 必须是自定义属性名（以 `--` 起始的标识符）；body 是描述符声明块
    /// （`syntax` / `inherits` / `initial-value`），用 `consume_declaration_block` 解析。
    /// 名称非法（非 `--` 起始）或块缺失 → 返回 None（上层做畸形恢复，消费残余 extent）。
    fn consume_property_rule(&mut self) -> Option<PropertyRule> {
        self.skip_whitespace();
        // prelude：自定义属性名（`--foo` 整体为单个 Ident token）。
        let name = match self.peek().clone() {
            Token::Ident(s) if s.starts_with("--") => {
                self.advance();
                s
            }
            _ => return None,
        };

        self.skip_whitespace();
        if !matches!(self.peek(), Token::LBrace) {
            // 非 `{`（如 `@property --foo;` 语句形式）：返回 None，上层消费残余。
            return None;
        }
        self.advance(); // {

        let declarations = self.consume_declaration_block();

        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
        }

        let mut syntax = String::new();
        let mut inherits = false;
        let mut inherits_seen = false;
        let mut initial_value: Option<String> = None;
        for decl in &declarations {
            if decl.property.eq_ignore_ascii_case("syntax") && syntax.is_empty() {
                syntax = decl.value.trim().to_string();
            } else if decl.property.eq_ignore_ascii_case("inherits") && !inherits_seen {
                // 取首个 inherits 描述符（@property 单一语义）。
                inherits_seen = true;
                inherits = decl.value.trim().eq_ignore_ascii_case("true");
            } else if decl.property.eq_ignore_ascii_case("initial-value") && initial_value.is_none() {
                initial_value = Some(decl.value.trim().to_string());
            }
        }

        Some(PropertyRule {
            name,
            syntax,
            inherits,
            initial_value,
        })
    }

    /// 消耗 `@counter-style` 规则（CSS Counter Styles 3 §3）。driving: R2392。
    ///
    /// 格式：`@counter-style <name> { system: cyclic; symbols: "a" "b"; suffix: ") "; }`
    /// prelude = 计数器名（单个 Ident）；body = 描述符声明块。解析 `system`/`symbols`/
    /// `prefix`/`suffix`/`fallback` 为类型化字段。无名 / 无 `{` → None（上层畸形恢复）。
    /// 非法 system/symbols 不足 → None（at-rule 无效，整体丢弃）。
    fn consume_counter_style_rule(&mut self) -> Option<CounterStyleRule> {
        self.skip_whitespace();
        // prelude：计数器名（单个 Ident，CSS Counter Styles 3 §3.1：`<custom-ident>`）。
        let name = match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                s
            }
            _ => return None,
        };

        self.skip_whitespace();
        if !matches!(self.peek(), Token::LBrace) {
            // 非 `{`（语句形式）：返回 None，上层消费残余。
            return None;
        }
        self.advance(); // {

        let declarations = self.consume_declaration_block();

        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
        }

        // 收集描述符（后声明覆盖前者——CSS @rule 描述符语义）。
        let mut system: Option<String> = None;
        let mut symbols_raw: Option<String> = None;
        let mut additive_raw: Option<String> = None;
        let mut prefix: Option<String> = None;
        let mut suffix: Option<String> = None;
        let mut fallback: Option<String> = None;
        let mut range_raw: Option<String> = None;
        for decl in &declarations {
            match decl.property.to_ascii_lowercase().as_str() {
                "system" if system.is_none() => system = Some(decl.value.trim().to_string()),
                "symbols" if symbols_raw.is_none() => symbols_raw = Some(decl.value.trim().to_string()),
                "additive-symbols" if additive_raw.is_none() => additive_raw = Some(decl.value.trim().to_string()),
                "prefix" if prefix.is_none() => prefix = Some(strip_css_quotes(decl.value.trim())),
                "suffix" if suffix.is_none() => suffix = Some(strip_css_quotes(decl.value.trim())),
                "fallback" if fallback.is_none() => fallback = Some(decl.value.trim().to_string()),
                "range" if range_raw.is_none() => range_raw = Some(decl.value.trim().to_string()),
                _ => {}
            }
        }

        // 解析 system（缺省 symbolic；CSS Counter Styles 3 §3.1.4）。
        let system = parse_counter_system(system.as_deref())?;
        // 解析 symbols（逐个去引号/按空白切分）。
        let symbols: Vec<String> = symbols_raw.as_deref().map(split_counter_symbols).unwrap_or_default();
        // 解析 additive-symbols（`<integer> && <symbol>` 对，按 weight 降序）。
        let additive_symbols: Vec<(i32, String)> = additive_raw
            .as_deref()
            .map(parse_counter_additive_symbols)
            .unwrap_or_default();
        // 解析 range（`[lower upper]` 对；`infinite`→i32 边界）。
        let range: Option<Vec<(i32, i32)>> = range_raw.as_deref().and_then(parse_counter_range);

        // 合法性（CSS §3.1.4）：extends 无需 symbols（继承）；非 additive 系统须 ≥1 symbol；
        // additive 须有 ≥1 additive-symbols 对（否则整体无效丢弃，走 fallback）。
        let needs_symbols = !matches!(system, CounterSystem::Extends(_));
        if needs_symbols && symbols.is_empty() && additive_symbols.is_empty() {
            return None;
        }

        Some(CounterStyleRule {
            name,
            system,
            symbols,
            additive_symbols,
            prefix: prefix.unwrap_or_default(),
            // suffix 缺省 = ". "（period + space）；显式描述符（含空串）已覆盖。
            suffix: suffix.unwrap_or_else(|| ". ".to_string()),
            fallback: fallback.unwrap_or_else(|| "decimal".to_string()),
            range,
        })
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
                let rules = self.consume_rules_until_rbrace();
                self.skip_whitespace();
                if matches!(self.peek(), Token::RBrace) {
                    self.advance();
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

        // 可选的媒体查询部分：收集直到分号。跟踪 `()`/`[]`/`{}`/Function 嵌套——
        // depth>0 时块内的 `;`/`,` 不作分隔/终止符（CSS Syntax「observing nesting」）；
        // `}` 在 depth==0 时是外层块（@media）结束，不消耗（让外层识别）——否则 `}` 被收进
        // media query 会吞掉外层闭合 + trailing 规则泄漏（driving: at-rule-013 #eob-complex
        // `@import "..." [; #eob-complex { background: red; } ] ...`）。
        let mut media_queries = Vec::new();
        let mut current_query = String::new();
        let mut group_depth: i32 = 0;

        loop {
            match self.peek() {
                Token::Semicolon if group_depth == 0 => {
                    self.advance();
                    break;
                }
                Token::RBrace if group_depth == 0 => {
                    // 外层块结束（@media 闭合）：不消耗 `}`，结束 @import（无 `;`）
                    break;
                }
                Token::Eof => {
                    break;
                }
                Token::Comma if group_depth == 0 => {
                    // 逗号分隔多个媒体查询
                    let trimmed = current_query.trim().to_string();
                    if !trimmed.is_empty() {
                        media_queries.push(trimmed);
                    }
                    current_query.clear();
                    self.advance();
                    self.skip_whitespace();
                }
                Token::Function(_) | Token::LParen | Token::LBracket | Token::LBrace => {
                    group_depth += 1;
                    current_query.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
                Token::RParen | Token::RBracket | Token::RBrace => {
                    group_depth = (group_depth - 1).max(0);
                    current_query.push_str(&format!("{}", self.peek()));
                    self.advance();
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

        // 收集 prelude 文本（直到**括号外**的 `{`）。条件内可能含嵌套 `(`/`[`/`{` 或函数
        // `selector(...)`/`func(...)`（Function token 自带开括号），故须按括号深度区分条件
        // 内容与规则体起始。开括号类（LParen / Function / LBracket / 嵌套 LBrace）+1，
        // 闭括号类（RParen / RBracket / RBrace）-1，顶层 LBrace（depth==0）= 规则体起始。
        // driving: WPT css-supports-033/034 `not ({ ... })`；selector() regression。
        let mut prelude = String::new();
        let mut depth = 0i32;
        loop {
            match self.peek() {
                Token::LBrace if depth == 0 => break,
                // `@supports` 是块 at-rule（须有 `{...}`）。prelude 收集期间遇顶层 `;`
                //（depth==0，无块）= 畸形语句 → 返回 None（不消耗 `;`），由 consume_rule
                // 的 skip_malformed_qualified_rule 消耗 `;` 后继续下一条规则。否则 prelude
                // 会越过 `;` 吞掉紧跟其后的合法规则。driving: WPT at-supports-024 `@supports;`。
                Token::Semicolon if depth == 0 => return None,
                Token::LParen | Token::Function(_) | Token::LBracket | Token::LBrace => {
                    depth += 1;
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
                Token::RParen | Token::RBracket | Token::RBrace => {
                    depth = depth.saturating_sub(1);
                    prelude.push_str(&format!("{}", self.peek()));
                    self.advance();
                }
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
        let rules = self.consume_rules_until_rbrace();
        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
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
        let rules = self.consume_rules_until_rbrace();
        self.skip_whitespace();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
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
                // Function token（如 `size(`）已隐含一个 `(`——其匹配 `)` 须纳入深度计数，
                // 否则 `(size(min-width: 400px))` 的内层 `)`（size 闭合）会把 depth 提前归零，
                // 仅收集到 `size(min-width: 400px`（无闭合），致 parse_container_condition 失败
                //（driving: test_container_with_size_function `(size(...))` 形式）。
                Token::Function(_) => {
                    depth += 1;
                    text.push_str(&format!("{}", self.peek()));
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

/// 解析 `@counter-style` 的 `system` 描述符为类型化算法（CSS Counter Styles 3 §3.1.4）。
/// driving: R2392。`None` = 非法 system（at-rule 无效）。
fn parse_counter_system(value: Option<&str>) -> Option<CounterSystem> {
    let v = value.unwrap_or("symbolic").trim(); // 缺省 symbolic
    let lower = v.to_ascii_lowercase();
    let mut parts = lower.split_whitespace();
    let head = parts.next()?;
    let system = match head {
        "cyclic" => CounterSystem::Cyclic,
        "fixed" => {
            // `fixed <integer>?`：首符号值（缺省 1）。
            let first = parts.next().and_then(|s| s.parse::<i32>().ok());
            CounterSystem::Fixed(first)
        }
        "symbolic" => CounterSystem::Symbolic,
        "alphabetic" => CounterSystem::Alphabetic,
        "numeric" => CounterSystem::Numeric,
        "additive" => CounterSystem::Additive,
        "extends" => {
            // `extends <counter-style-name>`：继承名（原始大小写，取未 lower 的下一段）。
            let ext = v.split_whitespace().nth(1)?.to_string();
            if ext.is_empty() {
                return None;
            }
            CounterSystem::Extends(ext)
        }
        _ => return None,
    };
    Some(system)
}

/// 切分 `@counter-style` 的 `symbols` 描述符值为独立符号列表（CSS Counter Styles 3 §3.1.5）。
/// 符号可为带引号串（`"a"` / `'◆'`）或裸标识/字形（`◆`），按空白分隔；逐个去引号。
/// driving: R2392。
fn split_counter_symbols(value: &str) -> Vec<String> {
    value.split_whitespace().map(strip_css_quotes).collect()
}

/// 解析 `additive-symbols` 描述符（CSS Counter Styles 3 §3.1.8）。
///
/// 格式：逗号分隔的 `<integer> && <symbol>` 对，如 `6 \2685, 5 \2684, ...` 或
/// `3 "a", 2 "b"`。每对中整数与符号（引号串/裸字形）顺序可互换。结果按 weight 降序排序
/// （贪心分解算法所需）。任一对缺整数或符号 → 该对跳过；全无效返回空 Vec（上层据空判非法）。
/// driving: R2394 slice 2。
fn parse_counter_additive_symbols(value: &str) -> Vec<(i32, String)> {
    let mut pairs: Vec<(i32, String)> = Vec::new();
    for part in value.split(',') {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }
        // 整数与符号二元组：整数可能在首或次位置。
        let (weight, symbol) = match (tokens[0].parse::<i32>().ok(), tokens[1].parse::<i32>().ok()) {
            (Some(w), None) => (w, strip_css_quotes(tokens[1])),
            (None, Some(w)) => (w, strip_css_quotes(tokens[0])),
            _ => continue, // 两端都非整数 / 都为整数 → 非法对，跳过。
        };
        pairs.push((weight, symbol));
    }
    // 降序排序（贪心分解从最大 weight 起）。稳定排序保留同 weight 声明顺序。
    pairs.sort_by_key(|b| std::cmp::Reverse(b.0));
    pairs
}

/// 解析 `range` 描述符（CSS Counter Styles 3 §3.1.2）。
///
/// 格式：逗号分隔的 `[lower upper]` 对，每对两值，`infinite` → i32::{MIN,MAX}。
/// 如 `1 5`、`1 5, 10 20`、`infinite -1`。仅当所有对解析成功时返回 Some；任一畸形返回 None
/// （缺省 range 由系统默认决定，slice 2 不应用）。`auto` 关键字返回 None（走系统默认）。
/// driving: R2394 slice 2。
fn parse_counter_range(value: &str) -> Option<Vec<(i32, i32)>> {
    let lower = value.to_ascii_lowercase();
    if lower.split_whitespace().eq(["auto"]) {
        return None;
    }
    let mut ranges = Vec::new();
    for part in lower.split(',') {
        let mut iter = part.split_whitespace();
        // lower 为 infinite → -∞（i32::MIN）；upper 为 infinite → +∞（i32::MAX）。
        let lo = parse_range_bound(iter.next()?, false)?;
        let hi = parse_range_bound(iter.next()?, true)?;
        ranges.push((lo, hi));
    }
    if ranges.is_empty() { None } else { Some(ranges) }
}

/// 解析单个 range 边界：`infinite` → 极值（lower→MIN，upper→MAX），否则十进制整数。
fn parse_range_bound(tok: &str, is_upper: bool) -> Option<i32> {
    match tok {
        "infinite" => Some(if is_upper { i32::MAX } else { i32::MIN }),
        _ => tok.parse::<i32>().ok(),
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
