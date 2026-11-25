//! `@function` prelude 定义文法（css-mixins-1 §2.1，wptfyi-css k114 抽出）。
//!
//! 涵盖 `<function-parameter-list>` token 级状态机、`<parameter-type>`/returns
//! `<css-type>` 分量文法（15 名 `<css-type>` 白名单＋乘数＋type() 联合）、
//! default `<declaration-value>` 文法与保守类型核验。本模块为 `parser` 的子
//! 模块，故可访问 `Parser` 的私有方法（Rust 隐私按模块树）；`pub(super)` 供
//! `at_rules.rs`（父）调用。driving: wptfyi-css at-function-parsing 48 键
//! ＋defaults 求值域 glued/space 冒号形参数段误析根修。
//! <https://drafts.csswg.org/css-mixins-1/#function-rule>

use super::Parser;
use crate::ast::FunctionParam;
use crate::tokenizer::Token;
use std::fmt::Write as _;

impl<'a> Parser<'a> {
    /// `<function-parameter-list>` token 级状态机（wptfyi-css k114 严格门）：
    /// 参数以顶层逗号分隔；起手/连续逗号空段 invalid（`--foo(,)`/`(,,)`），
    /// ≥1 参数后的尾逗号容忍（k109 `trailing_comma_drops_empty_final_arg_param`
    /// 钉承接，§3 Using 尾逗号形同源）；非 dashed-ident 起手／参数后残留
    /// 垃圾 token invalid。返回 None＝文法非法（调用方走链尾恢复）。
    pub(super) fn parse_function_param_list(&mut self) -> Option<Vec<FunctionParam>> {
        let mut params: Vec<FunctionParam> = Vec::new();
        let mut param_since_comma = false;
        loop {
            self.skip_whitespace();
            match self.peek().clone() {
                Token::RParen => {
                    self.advance();
                    return Some(params);
                }
                Token::Eof => return None,
                Token::Comma => {
                    if !param_since_comma {
                        // 起手空段（`--foo(,)`）或连续逗号空段（`(,,)`）。
                        return None;
                    }
                    self.advance();
                    param_since_comma = false;
                    self.skip_whitespace();
                    if matches!(self.peek(), Token::RParen) {
                        // 尾逗号容忍：`--foo(--x,)`。
                        self.advance();
                        return Some(params);
                    }
                }
                Token::Ident(_) => {
                    let param = self.parse_function_param()?;
                    params.push(param);
                    param_since_comma = true;
                    self.skip_whitespace();
                    match self.peek() {
                        Token::Comma | Token::RParen => {}
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
    }

    /// 单个 `<function-parameter>`：`<parameter-name>`＋可选类型分量＋可选
    /// `: <declaration-value>` default（冒号 glued/space 形——k109 字符串
    /// 路径误析根修）。类型文法非法 / default 空 / default `!` 形 / default
    /// 确定错配 → None。
    pub(super) fn parse_function_param(&mut self) -> Option<FunctionParam> {
        let Token::Ident(name) = self.peek().clone() else {
            return None;
        };
        if !name.starts_with("--") {
            return None;
        }
        self.advance();
        self.skip_whitespace();

        // 类型分量 token 收集到深度 0 的 Colon/Comma/RParen/Eof
        //（type() 复合形内深度感知——内文 `|`/`(` 不误判段界）。
        let mut ty_tokens: Vec<Token> = Vec::new();
        let mut depth = 0i32;
        let sep = loop {
            match self.peek().clone() {
                Token::Function(_) | Token::LParen => {
                    depth += 1;
                    ty_tokens.push(self.peek().clone());
                }
                Token::RParen => {
                    if depth == 0 {
                        break Token::RParen;
                    }
                    depth -= 1;
                    ty_tokens.push(self.peek().clone());
                }
                Token::Colon | Token::Comma if depth == 0 => break self.peek().clone(),
                Token::Eof => return None,
                tok => ty_tokens.push(tok),
            }
            self.advance();
        };

        // 类型位：空＝无类型（`--x`/`--x : 10px`）；非空＝过文法门。
        let ty_spec = {
            let trimmed = trim_ws_tokens(&ty_tokens);
            if trimmed.is_empty() {
                None
            } else {
                Some(parse_bare_type_spec(trimmed)?)
            }
        };
        let ty_text = tokens_text(trim_ws_tokens(&ty_tokens));
        let ty = (!ty_text.is_empty()).then_some(ty_text);

        let default = if matches!(sep, Token::Colon) {
            self.advance(); // :
            // default 值 token 收集到深度 0 的 Comma/RParen/Eof（深度感知——
            // random()/calc() 内逗号属 default 值）。
            let mut dv: Vec<Token> = Vec::new();
            let mut ddepth = 0i32;
            loop {
                match self.peek().clone() {
                    Token::Function(_) | Token::LParen => {
                        ddepth += 1;
                        dv.push(self.peek().clone());
                    }
                    Token::RParen => {
                        if ddepth == 0 {
                            break;
                        }
                        ddepth -= 1;
                        dv.push(self.peek().clone());
                    }
                    Token::Comma if ddepth == 0 => break,
                    Token::Eof => return None,
                    tok => dv.push(tok),
                }
                self.advance();
            }
            let trimmed = trim_ws_tokens(&dv);
            if trimmed.is_empty() {
                // `--x:` / `--x: `——default 缺值 invalid。
                return None;
            }
            // `!important`/裸 `!` 拒（§2.1.1：default 为 <declaration-value>，
            // bang 形非法；顶层深度 0 判——字符串/函数内 `!` 不误伤）。
            if trimmed.iter().any(|t| matches!(t, Token::Delim('!'))) {
                return None;
            }
            // 类型核验（ty 在场时）：确定错配 → 规则 invalid。
            if let Some(spec) = &ty_spec {
                if !default_matches_spec(trimmed, spec) {
                    return None;
                }
            }
            Some(tokens_text(trimmed))
        } else {
            None
        };

        Some(FunctionParam { name, ty, default })
    }
}

// ── @function prelude 文法（wptfyi-css k114）──────────────────────────
//
// css-mixins-1 §2.1.1 `<function-parameter>`／`<parameter-type>`／returns
// `<css-type>` 的 token 级严格门。driving: wptfyi-css at-function-parsing
// 48 键（valid 形 cssRules 读面 1 规则 / invalid 形 0 规则）＋defaults 求值
// 域 glued/space 冒号形（`--x: PASS`/`--a : attr(...)`）参数段误析根修。
// <https://drafts.csswg.org/css-mixins-1/#function-rule>

/// 已知 `<css-type>` 名单（§2.1.1 白名单——`<dino>` 等未知名 → 分量 invalid）。
fn known_css_type_kind(name: &str) -> Option<KnownKind> {
    Some(match name {
        "angle" => KnownKind::Angle,
        "color" => KnownKind::Color,
        "custom-ident" => KnownKind::CustomIdent,
        "image" => KnownKind::Image,
        "integer" => KnownKind::Integer,
        "length" => KnownKind::Length,
        "length-percentage" => KnownKind::LengthPercentage,
        "number" => KnownKind::Number,
        "percentage" => KnownKind::Percentage,
        "resolution" => KnownKind::Resolution,
        "string" => KnownKind::String,
        "time" => KnownKind::Time,
        "url" => KnownKind::Url,
        "transform-function" => KnownKind::TransformFunction,
        "transform-list" => KnownKind::TransformList,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum KnownKind {
    Angle,
    Color,
    CustomIdent,
    Image,
    Integer,
    Length,
    LengthPercentage,
    Number,
    Percentage,
    Resolution,
    String,
    Time,
    Url,
    TransformFunction,
    TransformList,
}

/// 语法乘数（`+` 空分隔列表 / `#` 逗号分隔列表；`<transform-list>` 禁乘数）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Mult {
    None,
    Plus,
    Hash,
}

/// 联合分量：已知类型、裸 ident（语法关键字，如 `auto`/`none`——大小写敏感）
/// 或 `*` 万能型。
#[derive(Clone, Debug, PartialEq)]
pub(super) enum TypeBranch {
    Known(KnownKind),
    IdentName(String),
    Universal,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TypeComp {
    branch: TypeBranch,
    mult: Mult,
}

/// 单分量解析产物：普通分量或 type() 复合形（联合分量表）。
enum CompParsed {
    Comp(TypeComp),
    TypeFnForm(Vec<TypeComp>),
}

/// 参数/返回类型规格：裸单分量（`<known>`/ident）或 type() 复合形。裸联合与
/// 裸 `*` 非法（仅 type() 内合法）。
#[derive(Clone, Debug, PartialEq)]
pub(super) enum TypeSpec {
    Single(TypeComp),
    TypeFn(Vec<TypeComp>),
}

/// 去首尾 Whitespace token。
pub(super) fn trim_ws_tokens(tokens: &[Token]) -> &[Token] {
    let mut start = 0usize;
    let mut end = tokens.len();
    while start < end && matches!(tokens[start], Token::Whitespace) {
        start += 1;
    }
    while end > start && matches!(tokens[end - 1], Token::Whitespace) {
        end -= 1;
    }
    &tokens[start..end]
}

/// token 文本重铸（Whitespace Display＝单空格保形——与既有段文本路径恒同，
/// registry 消费方 default/ty 原文逐字节不变）。
pub(super) fn tokens_text(tokens: &[Token]) -> String {
    let mut out = String::new();
    for t in tokens {
        let _ = write!(out, "{}", t);
    }
    out
}

/// 解析单个语法分量（`<known>`/裸 ident/`*`/type() 复合形）＋可选乘数。
/// 返回 `(解析产物, 消耗 token 数)`；文法非法 → None。
///
/// `<transform-list>` 禁乘数（§2.1.1：transform-list 本身即列表）——裸形与
/// type() 内形同门。
fn parse_type_component(tokens: &[Token]) -> Option<(CompParsed, usize)> {
    let mut i;
    let parsed = match tokens.first()? {
        Token::Delim('<') => {
            let Token::Ident(name) = tokens.get(1)? else {
                return None;
            };
            let kind = known_css_type_kind(name)?;
            if !matches!(tokens.get(2)?, Token::Delim('>')) {
                return None;
            }
            i = 3;
            CompParsed::Comp(TypeComp {
                branch: TypeBranch::Known(kind),
                mult: Mult::None,
            })
        }
        Token::Delim('*') => {
            i = 1;
            CompParsed::Comp(TypeComp {
                branch: TypeBranch::Universal,
                mult: Mult::None,
            })
        }
        Token::Ident(s) => {
            i = 1;
            CompParsed::Comp(TypeComp {
                branch: TypeBranch::IdentName(s.clone()),
                mult: Mult::None,
            })
        }
        Token::Function(f) if f.eq_ignore_ascii_case("type") => {
            // type() 复合形：Function token 已含 `(`，扫描到配对 RParen。
            let j = scan_type_paren_close(tokens)?;
            let comps = parse_union_components(&tokens[1..j])?;
            i = j + 1;
            CompParsed::TypeFnForm(comps)
        }
        _ => return None,
    };
    // 可选乘数（仅普通分量位；type() 复合形后乘数语料零驱动不放宽）。
    let comp = match parsed {
        CompParsed::TypeFnForm(comps) => {
            return Some((CompParsed::TypeFnForm(comps), i));
        }
        CompParsed::Comp(c) => c,
    };
    let mult = match tokens.get(i) {
        Some(Token::Delim('+')) => {
            i += 1;
            Mult::Plus
        }
        Some(Token::Delim('#')) => {
            i += 1;
            Mult::Hash
        }
        // 裸 `#`（后随非 ident 字符）tokenizer 产 `Error("Unexpected '#'")` 形
        // （css-syntax §4.3.1 应为 delim-token；tokenizer 现行 Error 承之且
        // tokenizer_coverage2 钉锁此形）——乘数位认作 `#`，不放宽全局面。
        Some(Token::Error(e)) if e == "Unexpected '#'" => {
            i += 1;
            Mult::Hash
        }
        _ => Mult::None,
    };
    if comp.branch == TypeBranch::Known(KnownKind::TransformList) && mult != Mult::None {
        return None;
    }
    Some((
        CompParsed::Comp(TypeComp {
            branch: comp.branch,
            mult,
        }),
        i,
    ))
}

/// type() 内文顶层 `|` 切联合分量（空内文/文法非法/transform-list 乘数 → None）。
fn parse_union_components(inner: &[Token]) -> Option<Vec<TypeComp>> {
    let inner = trim_ws_tokens(inner);
    if inner.is_empty() {
        return None;
    }
    let mut comps: Vec<TypeComp> = Vec::new();
    let mut part: Vec<Token> = Vec::new();
    let mut d = 0i32;
    for t in inner {
        match t {
            Token::Function(_) | Token::LParen => {
                d += 1;
                part.push(t.clone());
            }
            Token::RParen => {
                d -= 1;
                part.push(t.clone());
            }
            Token::Delim('|') if d == 0 => {
                comps.push(expect_plain_comp(&part)?);
                part.clear();
            }
            _ => part.push(t.clone()),
        }
    }
    comps.push(expect_plain_comp(&part)?);
    if comps
        .iter()
        .any(|c| c.branch == TypeBranch::Known(KnownKind::TransformList) && c.mult != Mult::None)
    {
        return None;
    }
    Some(comps)
}

/// 联合内分量：仅普通分量形（`<known>`/ident/`*`＋乘数），消耗须竭。
fn expect_plain_comp(part: &[Token]) -> Option<TypeComp> {
    let (parsed, consumed) = parse_type_component(trim_ws_tokens(part))?;
    if consumed != trim_ws_tokens(part).len() {
        return None;
    }
    match parsed {
        CompParsed::Comp(c) => Some(c),
        CompParsed::TypeFnForm(_) => None, // 嵌套 type() 非法
    }
}

/// Function token 形起手（`type(`）扫描到配对 RParen，返回 RParen 位置。
fn scan_type_paren_close(tokens: &[Token]) -> Option<usize> {
    let mut depth = 1i32;
    let mut j = 1usize;
    loop {
        match tokens.get(j)? {
            Token::Function(_) | Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            Token::Eof => return None,
            _ => {}
        }
        j += 1;
    }
}

/// 裸类型规格（参数类型位/returns 位）：单分量或 type() 复合形；裸联合/裸
/// `*` 非法；分量后残留 token（`--x <length> | auto`、`<length>!`）→ None。
pub(super) fn parse_bare_type_spec(tokens: &[Token]) -> Option<TypeSpec> {
    let (parsed, consumed) = parse_type_component(tokens)?;
    if consumed != tokens.len() {
        return None;
    }
    match parsed {
        CompParsed::TypeFnForm(comps) => Some(TypeSpec::TypeFn(comps)),
        CompParsed::Comp(c) => match c.branch {
            TypeBranch::Universal => None,
            _ => Some(TypeSpec::Single(c)),
        },
    }
}

/// 长度类单位（CSS Values §<length>）。
fn is_length_unit(u: &str) -> bool {
    matches!(
        u,
        "em" | "ex"
            | "cap"
            | "ch"
            | "ic"
            | "rem"
            | "lh"
            | "rlh"
            | "vw"
            | "vh"
            | "vi"
            | "vb"
            | "vmin"
            | "vmax"
            | "cm"
            | "mm"
            | "q"
            | "in"
            | "pt"
            | "pc"
            | "px"
    )
}

/// default 值 token 对类型规格的保守核验：仅拒绝"确定错配"（dimension 单位
/// 类不符/裸数对非数型/ident 联合不命中/列表分量逐核）；Function token
///（var()/env()/random()/calc()/attr() 等不定型）保守接受——random 语料钉
///（`--x <number>: random(...)` 必须存活）。
fn default_matches_spec(default: &[Token], spec: &TypeSpec) -> bool {
    let comps = match spec {
        TypeSpec::TypeFn(list) => list,
        TypeSpec::Single(c) => std::slice::from_ref(c),
    };
    // 万能型分量（`*`，含联合内）恒真。
    if comps.iter().any(|c| c.branch == TypeBranch::Universal) {
        return true;
    }
    // 列表乘数：任一分量带乘数 → 按该乘数切分 default；多分量联合乘数形
    // 语料零驱动，取首个非 None 乘数。
    let mult = comps
        .iter()
        .map(|c| c.mult)
        .find(|m| *m != Mult::None)
        .unwrap_or(Mult::None);
    let components: Vec<&[Token]> = match mult {
        Mult::None => split_default_ws(default),
        Mult::Plus => split_default_ws(default),
        Mult::Hash => split_default_commas(default),
    };
    components
        .into_iter()
        .all(|c| comps.iter().any(|b| component_matches_branch(c, &b.branch)))
}

/// default 顶层空白切分（深度感知——函数/括号内空白不切）。
fn split_default_ws(tokens: &[Token]) -> Vec<&[Token]> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, t) in tokens.iter().enumerate() {
        match t {
            Token::Function(_) | Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            Token::Whitespace if depth == 0 => {
                if i > start {
                    out.push(&tokens[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < tokens.len() {
        out.push(&tokens[start..]);
    }
    out.into_iter().filter(|s| !s.is_empty()).collect()
}

/// default 顶层逗号切分（`#` 列表；深度感知——`#` 列表分量含函数形逗号不切）。
fn split_default_commas(tokens: &[Token]) -> Vec<&[Token]> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, t) in tokens.iter().enumerate() {
        match t {
            Token::Function(_) | Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            Token::Comma if depth == 0 => {
                out.push(&tokens[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < tokens.len() {
        out.push(&tokens[start..]);
    }
    out.into_iter().filter(|s| !trim_ws_tokens(s).is_empty()).collect()
}

/// 单分量对 default 组件的定谳匹配。
fn component_matches_branch(comp: &[Token], branch: &TypeBranch) -> bool {
    let comp = trim_ws_tokens(comp);
    // Function 形（var()/env()/random()/calc()/attr() 等不定型）保守接受，
    // 内文不下钻——组件是含括号体的整段 token 切片（css-mixins-1 §2.1.1
    // default <declaration-value>；random 语料钉 `--x <number>: random(...)`
    // 必须存活）。
    if matches!(comp.first(), Some(Token::Function(_))) {
        return true;
    }
    match branch {
        // 万能型：任意组件恒真。
        TypeBranch::Universal => true,
        // 语法关键字：Ident 逐字命中（大小写敏感——语法组件 ident 不折叠）。
        TypeBranch::IdentName(name) => {
            matches!(comp, [Token::Ident(s)] if s == name)
        }
        TypeBranch::Known(kind) => match kind {
            KnownKind::Number | KnownKind::Integer => matches!(comp, [Token::Number(_, _)]),
            KnownKind::Length => match comp {
                [Token::Dimension(_, u)] => is_length_unit(u),
                [Token::Number(v, _)] => *v == 0.0,
                _ => false,
            },
            KnownKind::LengthPercentage => match comp {
                [Token::Dimension(_, u)] => is_length_unit(u),
                [Token::Percentage(_)] | [Token::Number(0.0, _)] => true,
                _ => false,
            },
            KnownKind::Percentage => matches!(comp, [Token::Percentage(_)]),
            KnownKind::Angle => match comp {
                [Token::Dimension(_, u)] => {
                    matches!(u.to_ascii_lowercase().as_str(), "deg" | "grad" | "rad" | "turn")
                }
                [Token::Number(v, _)] => *v == 0.0,
                _ => false,
            },
            KnownKind::Time => match comp {
                [Token::Dimension(_, u)] => {
                    matches!(u.to_ascii_lowercase().as_str(), "s" | "ms")
                }
                _ => false,
            },
            KnownKind::Resolution => match comp {
                [Token::Dimension(_, u)] => {
                    matches!(u.to_ascii_lowercase().as_str(), "dpi" | "dpcm" | "dppx")
                }
                _ => false,
            },
            KnownKind::Color => {
                matches!(comp, [Token::Hash(_)] | [Token::Ident(_)] | [Token::Url(_)])
            }
            KnownKind::String => matches!(comp, [Token::String(_)]),
            KnownKind::Url => matches!(
                comp,
                [Token::Url(_)] | [Token::UrlWithModifiers(_, _)] | [Token::String(_)]
            ),
            KnownKind::Image => matches!(
                comp,
                [Token::Url(_)] | [Token::UrlWithModifiers(_, _)] | [Token::String(_)] | [Token::Hash(_)]
            ),
            KnownKind::CustomIdent => matches!(comp, [Token::Ident(_)]),
            // 乘数禁用于 transform-list（文法门已拒）；分量级 transform 函数
            // 走上方 Function 形早退。
            KnownKind::TransformFunction | KnownKind::TransformList => false,
        },
    }
}
