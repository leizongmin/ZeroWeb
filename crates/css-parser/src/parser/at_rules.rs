//! CSS 解析器 @规则消费方法（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 涵盖 `@font-face` / `@page` / `@property` / `@counter-style` / `@keyframes` / `@layer`
//! / `@import` / `@supports` / `@container` 等 at-rule 的解析。`impl Parser` 拆分跨文件：
//! 本模块为 `parser` 的子模块，故可访问 `Parser` 的私有字段/方法（Rust 隐私按模块树）；
//! `pub(super)` 等价原「parser 模块私有」语义，供 `mod.rs`（父）调用。

use super::Parser;
use super::helpers::*;
use crate::ast::*;
use crate::tokenizer::Token;
use crate::values::types::FontStyleValue;

impl<'a> Parser<'a> {
    /// 消耗 @规则。
    pub(super) fn consume_at_rule(&mut self, name: String) -> AtRule {
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
    pub(super) fn consume_font_face_rule(&mut self) -> Option<FontFaceRule> {
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
        let mut style: Option<FontStyleValue> = None;
        let mut stretch: Option<f32> = None;
        let mut feature_settings = crate::values::FontFeatureSettingsValue::Normal;
        for decl in &declarations {
            if decl.property.eq_ignore_ascii_case("font-family") {
                family = strip_css_quotes(decl.value.trim());
            } else if decl.property.eq_ignore_ascii_case("src") {
                for url in extract_urls_from_src(&decl.value) {
                    sources.push(url);
                }
            } else if decl.property.eq_ignore_ascii_case("font-weight") {
                weight = Self::parse_font_face_weight(&decl.value);
            } else if decl.property.eq_ignore_ascii_case("font-style") {
                style = Self::parse_font_face_style(&decl.value);
            } else if decl.property.eq_ignore_ascii_case("font-stretch") {
                stretch = crate::values::parse_font_stretch(&decl.value);
            } else if decl.property.eq_ignore_ascii_case("font-feature-settings")
                && let Some(parsed) = crate::values::parse_font_feature_settings(&decl.value)
            {
                feature_settings = parsed;
            }
        }

        if family.is_empty() || sources.is_empty() {
            return None;
        }

        Some(FontFaceRule {
            family,
            sources,
            weight,
            style,
            stretch,
            feature_settings,
        })
    }

    /// 解析 `@font-face` 的 `font-weight` 描述符为绝对权重（R2417 font-weight matching）。
    ///
    /// `normal`→400、`bold`→700、数字（100-900）原值；`lighter`/`bolder`（相对，@font-face
    /// 描述符无父上下文）或无法识别 → `None`（调用方视为 normal/400，不构粗体键）。
    pub(super) fn parse_font_face_weight(value: &str) -> Option<u16> {
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

    /// 解析 `@font-face` 的 `font-style` 描述符（R2493 font-style matching）。
    ///
    /// `normal`→Normal、`italic`→Italic、`oblique`（含可选角度，匹配视为 italic）→Oblique(None)；
    /// 无法识别 → `None`（调用方视为 normal/upright，不构 italic 键）。
    pub(super) fn parse_font_face_style(value: &str) -> Option<FontStyleValue> {
        let v = value.trim();
        if v.eq_ignore_ascii_case("normal") {
            return Some(FontStyleValue::Normal);
        }
        if v.eq_ignore_ascii_case("italic") {
            return Some(FontStyleValue::Italic);
        }
        // `oblique` 或 `oblique <angle>`：匹配视为 italic，角度当前忽略（无须精确）。
        if v.eq_ignore_ascii_case("oblique") || v.to_ascii_lowercase().starts_with("oblique") {
            return Some(FontStyleValue::Oblique(None));
        }
        None
    }

    /// 消耗 @page 规则（CSS Paged Media）。
    ///
    /// 格式：`@page { size: A4; margin: 2cm; }`（prelude 可为命名页 `:first` / `name`，
    /// 当前忽略——仅消费 body 声明块提取 `size` 描述符）。
    /// body 是声明块（同 @font-face），用 `consume_declaration_block` 解析，
    /// 提取 `size` 描述符并经 `resolve_page_size_px` 解析为像素 `(width, height)`。
    pub(super) fn consume_page_rule(&mut self) -> Option<PageRule> {
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
    pub(super) fn consume_property_rule(&mut self) -> Option<PropertyRule> {
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
    pub(super) fn consume_counter_style_rule(&mut self) -> Option<CounterStyleRule> {
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
    pub(super) fn consume_keyframes_rule(&mut self) -> Option<KeyframesRule> {
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
    pub(super) fn consume_layer_rule(&mut self) -> Option<LayerRule> {
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
    pub(super) fn consume_import_rule(&mut self) -> Option<ImportRule> {
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
    pub(super) fn consume_supports_rule(&mut self) -> Option<SupportsRule> {
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
    pub(super) fn consume_container_rule(&mut self) -> Option<ContainerRule> {
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
    pub(super) fn collect_paren_content(&mut self) -> Option<String> {
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
