//! CSS Values 5 [`attr()`](https://drafts.csswg.org/css-values-5/#attr-notations)
//! 属性引用函数文法与文本级替换（WPT css-values attr-all-types/
//! attr-argument-grammar/attr-css-wide-keywords 语料驱动）。
//!
//! 文法（§attr-notations，`raw-string` 型与单位简写为语料实收形）：
//! ```text
//! attr() = attr( <attr-name> <attr-type>? , <declaration-value>? )
//! <attr-type> = <attr-unit> | raw-string | type( <syntax> )
//! ```
//!
//! 职责分层：本模块裁**文法**（写面门 [`attr_calls_valid`]）+ **文本级
//! 替换**（[`substitute_attr_values`]，读取器注入——求值方按元素属性表
//! 提供 `attr-name` 原文）；型别匹配（[`AttrSyntax`] 分量 vs 属性值文本）
//! 在替换器内完成，`<length>`/`<number>` 数学形折叠走既有
//! `fold_math_specified` 管线（语料 `calc(3 + 3)`→`calc(6)`、
//! `calc(1px + 3px)`→`calc(4px)` 钉面）。
//!
//! 语料钉面（attr-all-types）：
//! - 缺省型（无 type 选择子）＝字符串：`attr(data-foo)` 解析为带引号
//!   CSS 字符串（`abc` → `"abc"`）；属性缺失 → 解析失败（fallback 或
//!   IACVT——`font-family: attr(non-existent)` 语料 IACVT 面钉）。
//! - 单位简写（`px`/`%`/`em`…）：属性值须为 `<number>`，结果 number+
//!   unit 串接（`3`+`px` → `3px`）；值非数字 → 型不合 → fallback。
//!   单位合法性（`xx` 未知单位）不在本层裁——下游 apply 型合承担。
//! - `type(<T>)` 语法匹配：整值按 `<T>` 分量匹配（union `|` 任一命中、
//!   `*` 恒真、裸 ident 分量关键字精确匹配）；`<custom-ident>` 拒 CSS
//!   wide 关键字（`initial` → fallback 语料钉）；`<string>` 拒未引号文。
//! - fallback：逗号后整段 declaration-value 原文透传（可为空——
//!   `attr(data-foo,)` 语料形）；CSS wide 关键字 fallback（`inherit`/
//!   `initial`/`unset`/`revert`）原样透传，css-wide 处理由下游 compute
//!   路径承担（attr-css-wide-keywords 语料钉）。
//! - 文法 invalid（`attr(!)`/`attr(())`/`type(invalid |)` 尾管/`&`
//!   token/三实参）→ 声明拒（非 fallback）。
//! - 未闭合调用按 css-syntax-3 consume-a-function EOF 恢复：跨度消费至
//!   值尾后仍走文法裁决——`type(<invalid>` 拒（语料 invalid 面钉）、
//!   `type(<color>` / `type(invalid | blue)` / `type(<length> |
//!   <percentage>)` 宽容形放行（Chrome 观测口径，语料 valid 面钉）。
//! - var()/env() 在实参内：文法宽容（延迟形）——compute 期 var 代入后
//!   由 [`substitute_attr_values`] 重裁（attr-argument-grammar 语料
//!   `attr(var(--x))` 先 var 后 attr 钉面）。

use std::borrow::Cow;

/// attr() 型别选择子（§attr-notations `<attr-type>`；缺省 ＝ 字符串）。
#[derive(Debug, Clone, PartialEq)]
pub enum AttrTypeSelector {
    /// 缺省/`raw-string`：属性原文按字符串解析（带引号 CSS 字符串输出）。
    DefaultString,
    /// `<attr-unit>` 单位简写：属性值须为 `<number>`，结果 `number+unit`。
    Unit(String),
    /// `type( <syntax> )` 语法匹配（裸关键字型别名 string/number/length…
    /// 统一归一为 [`AttrSyntax`] 分量后走同一匹配器）。
    Syntax(AttrSyntax),
}

/// `type()` 语法（§attr-syntax `*`/`<T>`/union/裸 ident 分量；`+` 乘数只
/// 收 `<string>+` 语料形）。
#[derive(Debug, Clone, PartialEq)]
pub enum AttrSyntax {
    /// `*`——任意值恒真（原文透传）。
    Any,
    /// `<length>`。
    Length,
    /// `<percentage>`。
    Percentage,
    /// `<number>`。
    Number,
    /// `<integer>`。
    Integer,
    /// `<angle>`。
    Angle,
    /// `<time>`。
    Time,
    /// `<frequency>`。
    Frequency,
    /// `<string>`（CSS 引号字符串）。
    String,
    /// `<color>`。
    Color,
    /// `<custom-ident>`（拒 CSS wide 关键字）。
    CustomIdent,
    /// `<url>`。
    Url,
    /// 裸 ident 分量（`lighter`/`bold`/`blue`/语料 `invalid` 等——关键字
    /// 精确匹配，ASCII 大小写不敏感）。
    Ident(String),
    /// union（`|` 任一分量命中）。
    Union(Vec<AttrSyntax>),
    /// 乘数（`<string>+`——空白分隔重复；语料 `type(<string>+)` 形）。
    OneOrMore(Box<AttrSyntax>),
}

/// 单个 attr() 调用规格。
#[derive(Debug, Clone, PartialEq)]
pub struct AttrCall {
    /// 属性名（`<attr-name>` ident 原文——大小写保真，读取方按 HTML 属性
    /// ASCII 大小写不敏感语义对齐）。
    pub name: String,
    /// 型别选择子（缺省 [`AttrTypeSelector::DefaultString`]）。
    pub type_sel: AttrTypeSelector,
    /// fallback 文本（`None` ＝ 无逗号节；`Some("")` ＝ 空 fallback 语料形
    /// `attr(data-foo,)`）。
    pub fallback: Option<String>,
    /// 实参含 var()/env() 替换函数（延迟形——compute 期代入后重裁）。
    pub has_substitution: bool,
}

/// 值内全部 attr() 调用文法裁决（CSS Values 5 §attr-notations 写面门——
/// math 门同位接线，native/shim 三写臂共用）。字符串字面内 `attr(` 不
/// 识别（random/canonicalize 同状态机）。
///
/// - 任一调用文法 invalid（空名/`!`/`&` 非 ident token/尾管 union/未知
///   `<T>`/类型 token 双节/三实参）→ false（声明拒——语料 `attr(!)`/
///   `attr(())`/`attr(data-foo &, 3px)`/`type(invalid |)` 面）；
/// - 全调用文法 valid（含 var()/env() 延迟形）→ true。
pub fn attr_calls_valid(value: &str) -> bool {
    if !has_attr_word(value) {
        return true;
    }
    scan_attr_calls(value).into_iter().all(|r| r.is_ok())
}

/// 值含 attr 词调用（ASCII 大小写不敏感词级粗扫——精确识别在扫描器内；
/// 词假阳无害，扫描器按词边界+括号配对甄别）。
pub fn has_attr_word(value: &str) -> bool {
    let b = value.as_bytes();
    b.windows(5).any(|w| w.eq_ignore_ascii_case(b"attr("))
}

/// 值含真 attr() 调用（词边界 + 字符串字面状态机甄别——`dataattr(` 词假
/// 阳与 `"attr(...)"` 字面均不认；级联门/规则 prescan 用，避免假阳绕过
/// apply-on-dummy 裁决）。
pub fn has_attr_call(value: &str) -> bool {
    if !has_attr_word(value) {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    let mut in_string: Option<char> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((_pos, ch)) = chars.next() {
        if let Some(q) = in_string {
            if ch == '\\' {
                // 转义对——后继字符按普通字符消费（`'\''` 不误闭）。
                if let Some(&(_, esc)) = chars.peek() {
                    let _ = esc;
                    chars.next();
                }
            } else if ch == q {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            continue;
        }
        if attr_word_boundary(value, _pos) && lower[_pos..].starts_with("attr(") {
            return true;
        }
    }
    false
}

/// attr() 调用扫描结果：`Ok`＝文法 valid 调用；`Err`＝文法 invalid。
type AttrScanResult = Result<AttrCall, ()>;

/// 扫描值内全部 attr() 调用（字符串字面状态机 + 词边界 + 括号配平；未闭
/// 合调用按「消费至值尾」EOF 恢复取跨度——恢复跨度仍走文法裁决）。
fn scan_attr_calls(value: &str) -> Vec<AttrScanResult> {
    let mut out = Vec::new();
    let lower = value.to_ascii_lowercase();
    let mut in_string: Option<char> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((pos, ch)) = chars.next() {
        if let Some(q) = in_string {
            if ch == '\\' {
                if let Some(&(_, esc)) = chars.peek() {
                    let _ = esc;
                    chars.next();
                }
            } else if ch == q {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            continue;
        }
        if !attr_word_boundary(value, pos) || !lower[pos..].starts_with("attr(") {
            continue;
        }
        let open_abs = pos + 4;
        // 配平取调用跨度；未闭合消费至值尾（css-syntax-3 EOF 恢复）。
        let (inner_end, span_end) = match match_paren(&lower, open_abs) {
            Some(c) => (c, c + 1),
            None => (value.len(), value.len()),
        };
        out.push(parse_attr_call(&value[open_abs + 1..inner_end]));
        while chars.peek().is_some_and(|&(p, _)| p < span_end) {
            chars.next();
        }
    }
    out
}

/// attr( 词边界（前邻非 ident 续字符——`dataattr(` 不识别）。
fn attr_word_boundary(value: &str, pos: usize) -> bool {
    pos == 0
        || match value[..pos].chars().next_back() {
            None => true,
            Some(c) => !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c >= '\u{80}'),
        }
}

/// 括号配平（字节扫描——UTF-8 续字节 ≥ 0x80 不撞 ASCII 括号）。返回配对
/// `)` 字节位。random/canonicalize 扫描器同型私有副本（values/random.rs
/// 同实现——共享文件 pub(crate) 化会卷入并行流在途改动，特此解耦）。
fn match_paren(s: &str, open: usize) -> Option<usize> {
    let b = s.as_bytes();
    if b.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth = 0usize;
    for (i, &byte) in b.iter().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// 单个 attr() 实参文法裁决（`<attr-name> <attr-type>? , <declaration-value>?`）。
pub(crate) fn parse_attr_call(args: &str) -> AttrScanResult {
    // 实参按顶层逗号切分（括号配平——fallback 可含函数逗号）。
    let mut parts: Vec<&str> = Vec::new();
    let mut depth = 0usize;
    let mut cur_start = 0usize;
    let bytes = args.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    // EOF 恢复跨度内深度负（`attr(a)extra)` 形）——文法拒。
                    return Err(());
                }
                depth -= 1;
            }
            b',' if depth == 0 => {
                parts.push(&args[cur_start..i]);
                cur_start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&args[cur_start..]);
    if parts.len() > 2 {
        return Err(());
    }
    // 节 1：<attr-name> <attr-type>?
    let head = strip_comments_ws(parts[0]);
    let toks = split_top_tokens(&head);
    if toks.is_empty() {
        return Err(());
    }
    let name = toks[0].clone();
    let deferred_name = tok_contains_substitution(&name);
    if !deferred_name && !is_attr_name(&name) {
        return Err(());
    }
    let mut has_substitution = deferred_name;
    let type_sel = match toks.len() {
        1 => AttrTypeSelector::DefaultString,
        2 => {
            has_substitution |= tok_contains_substitution(&toks[1]);
            if has_substitution {
                // 延迟形：var 代入后重裁——型别选择子占位。
                AttrTypeSelector::DefaultString
            } else {
                parse_type_selector(&toks[1])?
            }
        }
        _ => return Err(()),
    };
    let fallback = match parts.get(1) {
        None => None,
        Some(f) => {
            let f = strip_comments_ws(f);
            if tok_contains_substitution(&f) {
                has_substitution = true;
            }
            Some(f)
        }
    };
    Ok(AttrCall {
        name,
        type_sel,
        fallback,
        has_substitution,
    })
}

/// `<attr-name>`：单个 ident token（含 `-` 前缀、非 ASCII ident 字符）。
fn is_attr_name(tok: &str) -> bool {
    let mut chars = tok.chars();
    match chars.next() {
        None => return false,
        Some(first) => {
            if !(first.is_alphabetic() || first == '-' || first == '_' || first >= '\u{80}') {
                return false;
            }
        }
    }
    tok.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c >= '\u{80}')
}

/// token 内 var()/env() 检测（写面延迟形——compute 期代入后重裁）。
fn tok_contains_substitution(tok: &str) -> bool {
    let lower = tok.to_ascii_lowercase();
    lower.contains("var(") || lower.contains("env(")
}

/// `<attr-type>`：`raw-string` | `<attr-unit>` | `type( <syntax> )` | 裸
/// 关键字型别名（string/color/url/integer/number/length/angle/time/frequency/
/// percentage → 同名 syntax）。
fn parse_type_selector(tok: &str) -> Result<AttrTypeSelector, ()> {
    let lower = tok.to_ascii_lowercase();
    if lower == "raw-string" {
        return Ok(AttrTypeSelector::DefaultString);
    }
    if let Some(inner) = lower.strip_prefix("type(") {
        // 闭合形 `type(...)` / EOF 恢复形 `type(...`（末 `)` 可缺——Chrome
        // 未闭合宽容口径，语料 `type(<color>)` 截断形钉）。
        let inner = if inner.ends_with(')') {
            &tok[5..tok.len() - 1]
        } else {
            &tok[5..]
        };
        return Ok(AttrTypeSelector::Syntax(parse_attr_syntax(inner)?));
    }
    // 裸关键字型别名。
    for (kw, syn) in [
        ("string", AttrSyntax::String),
        ("color", AttrSyntax::Color),
        ("url", AttrSyntax::Url),
        ("integer", AttrSyntax::Integer),
        ("number", AttrSyntax::Number),
        ("length", AttrSyntax::Length),
        ("angle", AttrSyntax::Angle),
        ("time", AttrSyntax::Time),
        ("frequency", AttrSyntax::Frequency),
        ("percentage", AttrSyntax::Percentage),
    ] {
        if lower == kw {
            return Ok(AttrTypeSelector::Syntax(syn));
        }
    }
    // `<attr-unit>`：CSS 单位 token（字母串或 `%`）——单位合法性（未知单位
    // 走下游 apply 型合，非本层文法拒——语料 `xx` 形）。
    if lower == "%" || (!lower.is_empty() && lower.bytes().all(|b| b.is_ascii_alphabetic())) {
        return Ok(AttrTypeSelector::Unit(tok.to_string()));
    }
    Err(())
}

/// `type()` 语法解析（§attr-syntax）：`*` | `<T>` | 裸 ident | union（`|`
/// 分隔，任一）。尾管/空分量/未知 `<...>` 形拒（语料 `type(invalid |)`/
/// `type(<invalid>)` 面）。
fn parse_attr_syntax(inner: &str) -> Result<AttrSyntax, ()> {
    let inner = strip_comments_ws(inner);
    if inner.is_empty() {
        return Err(());
    }
    let lower = inner.to_ascii_lowercase();
    if lower == "*" {
        return Ok(AttrSyntax::Any);
    }
    // union 切分（`|` 顶层分隔——`<T>` 形内无 `|`，无需配平）。
    let alts: Vec<&str> = inner.split('|').map(str::trim).collect();
    let mut parsed = Vec::with_capacity(alts.len());
    for alt in &alts {
        if alt.is_empty() {
            return Err(());
        }
        parsed.push(parse_syntax_component(alt)?);
    }
    if parsed.len() == 1 {
        Ok(parsed.pop().unwrap())
    } else {
        Ok(AttrSyntax::Union(parsed))
    }
}

/// 单个语法分量：`<T>` 尖括号型 / 裸 ident / `+` 乘数。EOF 恢复形容忍
/// 未闭合 `<T`（`type(<color>` 截断——Chrome 宽容口径钉面）。
fn parse_syntax_component(alt: &str) -> Result<AttrSyntax, ()> {
    let alt = alt.trim();
    let lower = alt.to_ascii_lowercase();
    if let Some(mul) = lower.strip_suffix('+') {
        // 乘数须紧贴分量（`type(<length>+)` 合法 / `type(<length> +)` 文法拒
        // ——css-properties-values-api §syntax-component 乘数无空白分隔，
        // 语料 `type(<length> +)` invalid 面钉）。
        if mul.ends_with(char::is_whitespace) {
            return Err(());
        }
        let base = parse_syntax_component(&alt[..mul.len()])?;
        return Ok(AttrSyntax::OneOrMore(Box::new(base)));
    }
    if let Some(inner) = lower.strip_prefix('<') {
        let inner = inner.strip_suffix('>').unwrap_or(inner);
        let syn = match inner {
            "length" => AttrSyntax::Length,
            "percentage" => AttrSyntax::Percentage,
            "number" => AttrSyntax::Number,
            "integer" => AttrSyntax::Integer,
            "angle" => AttrSyntax::Angle,
            "time" => AttrSyntax::Time,
            "frequency" => AttrSyntax::Frequency,
            "string" => AttrSyntax::String,
            "color" => AttrSyntax::Color,
            "custom-ident" => AttrSyntax::CustomIdent,
            "url" => AttrSyntax::Url,
            "image" | "transform-function" | "filter-function" | "flex" | "ratio" | "position" | "bg-size" => {
                AttrSyntax::Any
            }
            // 未知型（`<invalid>` 语料形）→ 文法拒（写面拒——语料
            // `type(<invalid>)` invalid 面钉）。
            _ => return Err(()),
        };
        return Ok(syn);
    }
    // 裸 ident 分量（关键字/`invalid`/`blue` 等任意 ident——匹配期精确
    // 对照；文法面只裁 ident 形）。
    if is_attr_name(alt) {
        return Ok(AttrSyntax::Ident(alt.to_ascii_lowercase()));
    }
    Err(())
}

/// 属性值按语法匹配（§attr-syntax 分量 vs 值文本整值匹配）。返回替换文本
///（`<length>`/`<number>` 数学形折叠——语料 `calc(3 + 3)`→`calc(6)`、
/// `calc(1px + 3px)`→`calc(4px)` 钉面；其余命中分量原文透传）。
fn match_syntax(syn: &AttrSyntax, raw: &str) -> Option<String> {
    let v = raw.trim();
    match syn {
        AttrSyntax::Any => Some(v.to_string()),
        AttrSyntax::Length => {
            if super::parse_length(v).is_some() {
                Some(v.to_string())
            } else {
                fold_math_value(v, super::serialize_math::MathFoldKind::Length)
            }
        }
        AttrSyntax::Percentage => parse_pct_only(v),
        AttrSyntax::Number => {
            if v.parse::<f64>().is_ok() {
                Some(v.to_string())
            } else {
                fold_math_value(v, super::serialize_math::MathFoldKind::Number)
            }
        }
        AttrSyntax::Integer => v.parse::<i64>().ok().map(|_| v.to_string()),
        AttrSyntax::Angle => parse_dim_unit(v, &["deg", "grad", "rad", "turn"])
            .or_else(|| fold_math_value(v, super::serialize_math::MathFoldKind::Angle)),
        AttrSyntax::Time => {
            parse_dim_unit(v, &["s", "ms"]).or_else(|| fold_math_value(v, super::serialize_math::MathFoldKind::Time))
        }
        AttrSyntax::Frequency => parse_dim_unit(v, &["hz", "khz"]),
        AttrSyntax::String => is_css_string(v).then(|| v.to_string()),
        AttrSyntax::Color => super::parse_color(v).is_some().then(|| v.to_string()),
        AttrSyntax::CustomIdent => {
            let lower = v.to_ascii_lowercase();
            let css_wide = matches!(lower.as_str(), "inherit" | "initial" | "unset" | "revert")
                || lower.starts_with("revert-layer");
            (!css_wide && is_attr_name(v)).then(|| v.to_string())
        }
        AttrSyntax::Url => v.to_ascii_lowercase().starts_with("url(").then(|| v.to_string()),
        AttrSyntax::Ident(kw) => (v.to_ascii_lowercase() == *kw).then(|| v.to_string()),
        AttrSyntax::Union(alts) => alts.iter().find_map(|a| match_syntax(a, v)),
        AttrSyntax::OneOrMore(base) => {
            // 空白分隔重复（语料 `<string>+` 形）——逐段匹配，原文拼接。
            let segs: Vec<&str> = v.split_ascii_whitespace().collect();
            if segs.is_empty() {
                return None;
            }
            let mut out = String::new();
            for (idx, seg) in segs.iter().enumerate() {
                match_syntax(base, seg)?;
                if idx > 0 {
                    out.push(' ');
                }
                out.push_str(seg);
            }
            Some(out)
        }
    }
}

/// 数学形折叠为文本（`fold_math_specified` 既有管线——g23 canonical
/// 保持串化口径）；不可折叠 fail-closed → None。
fn fold_math_value(v: &str, kind: super::serialize_math::MathFoldKind) -> Option<String> {
    super::serialize_math::fold_math_specified(v, kind)
}

/// 裸 `%` 值匹配（`10%`）。
fn parse_pct_only(v: &str) -> Option<String> {
    let t = v.trim();
    if t.ends_with('%') && t[..t.len() - 1].trim().parse::<f64>().is_ok() {
        return Some(t.to_string());
    }
    None
}

/// 维度单位族匹配（angle/time/frequency 单位集；数学形 fail-closed——
/// 语料 time 页 fallback 面承担）。
fn parse_dim_unit(v: &str, units: &[&str]) -> Option<String> {
    let t = v.trim();
    let lower = t.to_ascii_lowercase();
    for u in units {
        if let Some(num) = lower.strip_suffix(u) {
            if !num.is_empty() && num.parse::<f64>().is_ok() {
                return Some(t.to_string());
            }
        }
    }
    None
}

/// CSS 引号字符串判定（首尾同引号；字符级校验交下游 content/color 通道）。
fn is_css_string(v: &str) -> bool {
    let t = v.trim();
    let b = t.as_bytes();
    b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'') && b[b.len() - 1] == b[0]
}

/// 单 attr() 调用解析（属性值读取器注入）：`Some`＝替换文本；`None`＝解析
/// 失败（属性缺失/型不合且无 fallback——IACVT）。延迟形（含 var/env）不
/// 在本函数消费（调用方保证 var 已代入；残留替换函数 → None）。
pub fn eval_attr_call(call: &AttrCall, reader: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    if call.has_substitution {
        return None;
    }
    let raw = reader(&call.name);
    let resolved = match raw {
        None => None,
        Some(raw) => match &call.type_sel {
            // 缺省字符串型：原文带引号（content `abc` → `"abc"` 语料钉面）。
            AttrTypeSelector::DefaultString => Some(format!("\"{}\"", raw.replace('\\', "\\\\").replace('"', "\\\""))),
            AttrTypeSelector::Unit(u) => raw
                .trim()
                .parse::<f64>()
                .ok()
                .map(|n| format!("{}{}", format_number_6dp(n), u)),
            AttrTypeSelector::Syntax(syn) => match_syntax(syn, &raw),
        },
    };
    resolved.or_else(|| fallback_value(&call.type_sel, call.fallback.as_deref()?))
}

/// fallback 按声明型别解析（css-values-5 §attr-notations：fallback 与属性
/// 值同型解析）。型别特例：
/// - 缺省字符串 / 裸关键字别名（`type(invalid), 10px`）→ 原文透传；
/// - 单位型（`attr(non-existent px, 3px)`）→ 裸数字归一 number+unit、
///   量纲 token 原文、其余拒；
/// - 维度型（length/percentage/angle/time/frequency）拒 numberish（裸数字
///   /number-calc 非维度 → `None`（IACVT）——语料 `type(<length>), 30`/
///   `type(<length>), calc(10 + 20)`/`type(<angle>), calc(10 + 20)` invalid
///   面钉）；其余原文透传——合法形本就同型（`type(<percentage>), 10px`），
///   不合形（`type(<angle>), italic`、嵌套 attr）交 cascade 按属性值文法
///   裁决（解析失败与 IACVT 同为 unset 语义；语料 `type(<custom-ident>),
///   3px` 钉面）。
fn fallback_value(type_sel: &AttrTypeSelector, fb: &str) -> Option<String> {
    let fbt = fb.trim();
    if fbt.is_empty() {
        // 空 fallback 语料形 `attr(data-foo,)`——替换空串维持既有行为。
        return Some(String::new());
    }
    match type_sel {
        AttrTypeSelector::DefaultString => Some(fb.to_string()),
        AttrTypeSelector::Syntax(AttrSyntax::Ident(_)) => Some(fb.to_string()),
        AttrTypeSelector::Unit(u) => {
            if let Ok(n) = fbt.parse::<f64>() {
                Some(format!("{}{}", format_number_6dp(n), u))
            } else if is_dimension_token(fbt) {
                Some(fb.to_string())
            } else {
                None
            }
        }
        AttrTypeSelector::Syntax(syn) => {
            if syntax_all_dimension(syn) && is_numberish_token(fbt) {
                None
            } else {
                Some(fb.to_string())
            }
        }
    }
}

/// 维度型判定（fallback numberish 拒收面）：Union/乘数递归——Union 须全
/// 分量为维度才拒（numberish fallback 命中任一非维度分量即合法，如
/// `type(<length> | <number>), 30`）。
fn syntax_all_dimension(syn: &AttrSyntax) -> bool {
    match syn {
        AttrSyntax::Length | AttrSyntax::Percentage | AttrSyntax::Angle | AttrSyntax::Time | AttrSyntax::Frequency => {
            true
        }
        AttrSyntax::Union(list) => !list.is_empty() && list.iter().all(syntax_all_dimension),
        AttrSyntax::OneOrMore(base) => syntax_all_dimension(base),
        _ => false,
    }
}

/// numberish token 判定：裸 f64 或可折叠为纯 number 的 math 函数
///（`30` / `calc(10 + 20)`——语料 fallback 拒收双形）。
fn is_numberish_token(v: &str) -> bool {
    v.parse::<f64>().is_ok() || fold_math_value(v, super::serialize_math::MathFoldKind::Number).is_some()
}

/// 量纲 token 形判定（`<number>` 后接字母单位或 `%`——`3px`/`50%`；单位
/// 合法性不在本层裁——语料未知单位 `xx` fallback `3px` 原文钉面）。
fn is_dimension_token(v: &str) -> bool {
    let t = v.trim();
    let mut split = t.char_indices();
    let mut num_end = 0usize;
    for (idx, ch) in split.by_ref() {
        match ch {
            '0'..='9' | '.' | '+' | '-' | 'e' | 'E' => num_end = idx + ch.len_utf8(),
            _ => break,
        }
    }
    let (num, unit) = t.split_at(num_end);
    num.parse::<f64>().is_ok() && !unit.is_empty() && (unit == "%" || unit.bytes().all(|b| b.is_ascii_alphabetic()))
}

/// CSSOM serialize-a-number 口径私有副本（serialize_math 同实现——最多
/// 6 位小数、去尾零、`-0` 归 `0`；单位简写拼接专用）。
fn format_number_6dp(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "infinity".into()
        } else {
            "-infinity".into()
        };
    }
    let mut s = format!("{:.6}", value);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}

/// 值内全部 attr() 调用替换（§attr-notations compute 期文本级替换——random
/// 替换器同构）。字符串字面内函数名不识别；调用未闭合按 EOF 恢复跨度。
///
/// 返回：
/// - 无 attr 调用 → 借用原文（零分配快路径）；
/// - 全调用可解析 → 替换文本；
/// - 文法 invalid / 延迟形残 attr / 任一调用 IACVT（属性缺失或型不合且
///   无 fallback）→ `None`（上层按声明丢弃语义处置——IACVT）。
pub fn substitute_attr_values<'a>(value: &'a str, reader: &dyn Fn(&str) -> Option<String>) -> Option<Cow<'a, str>> {
    substitute_attr_values_checked(value, reader).map(|(v, _)| v)
}

/// 替换产物 url 判定（css-values-5 §attr security 的污染源甄别）：替换片
/// 段携带可取 URL——`url(` token 或带 scheme 的裸/引号 url 字符串——为真。
/// 渐变/颜色等本地构造图源为假（attr(type(<image>)) + gradient 语料钉面）。
fn replaced_fragment_is_url(value: &str) -> bool {
    let t = value.trim();
    let lower = t.to_ascii_lowercase();
    if lower.contains("url(") {
        return true;
    }
    // 剥一层匹配引号后看 scheme（image-set(attr(data-foo)) 字符串形态）。
    let b = t.as_bytes();
    let bare = if b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'') && b[b.len() - 1] == b[0] {
        &t[1..t.len() - 1]
    } else {
        t
    };
    let bare = bare.trim();
    bare.starts_with("http://")
        || bare.starts_with("https://")
        || bare.starts_with("data:")
        || bare.starts_with("blob:")
        || bare.starts_with("ftp://")
        || bare.starts_with("file://")
        || bare.starts_with("//")
}

/// [`substitute_attr_values`] 带污染标记变体：第二返回值＝任一替换片段
/// 携带 url（css-values-5 §attr security——取 URL 属性遇污染替换 → 上层
/// 判声明 invalid）。<https://drafts.csswg.org/css-values-5/#attr-security>
pub fn substitute_attr_values_checked<'a>(
    value: &'a str,
    reader: &dyn Fn(&str) -> Option<String>,
) -> Option<(Cow<'a, str>, bool)> {
    if !has_attr_word(value) {
        return Some((Cow::Borrowed(value), false));
    }
    let lower = value.to_ascii_lowercase();
    let mut out = String::with_capacity(value.len());
    let mut last = 0usize;
    let mut replaced = false;
    let mut tainted = false;
    let mut in_string: Option<char> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((pos, ch)) = chars.next() {
        if let Some(q) = in_string {
            if ch == '\\' {
                if let Some(&(_, esc)) = chars.peek() {
                    let _ = esc;
                    chars.next();
                }
            } else if ch == q {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            continue;
        }
        if !attr_word_boundary(value, pos) || !lower[pos..].starts_with("attr(") {
            continue;
        }
        let open_abs = pos + 4;
        let (inner_end, span_end) = match match_paren(&lower, open_abs) {
            Some(c) => (c, c + 1),
            None => (value.len(), value.len()),
        };
        // 单遍：文法裁决 + 求值（任一 invalid / IACVT → 整值 None）。
        let call = parse_attr_call(&value[open_abs + 1..inner_end]).ok()?;
        let repl = eval_attr_call(&call, reader)?;
        out.push_str(&value[last..pos]);
        out.push_str(&repl);
        last = span_end;
        replaced = true;
        tainted |= replaced_fragment_is_url(&repl);
        while chars.peek().is_some_and(|&(p, _)| p < span_end) {
            chars.next();
        }
    }
    if !replaced {
        // 词假阳（字面内 attr( 等）——值不变。
        return Some((Cow::Borrowed(value), false));
    }
    out.push_str(&value[last..]);
    Some((Cow::Owned(out), tainted))
}

/// 取 URL 属性清单（css-values-5 §attr security：替换产物携带 url 时声明
/// invalid 的属性面——`background`/`background-image`/`border-image`/
/// `list-style-image`/`cursor`）。
pub fn attr_target_fetches_url(prop: &str) -> bool {
    matches!(
        prop.to_ascii_lowercase().as_str(),
        "background" | "background-image" | "border-image" | "list-style-image" | "cursor"
    )
}

/// 顶层 token 切分（空白分隔、括号平衡感知——`type(<length> | bold)` 单
/// token；注释已剥）。
fn split_top_tokens(head: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut cur = String::new();
    for c in head.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push('(');
            }
            ')' => {
                depth = depth.saturating_sub(1);
                cur.push(')');
            }
            ' ' | '\t' | '\n' | '\r' | '\x0c' if depth == 0 => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// 注释剥除 + 空白归一（css-syntax-3 §4.4 注释等价空白——`attr(data-foo
/// /*t*/, 3px)` 形容忍；未闭合注释消费至尾）。
fn strip_comments_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '/' && s[i..].starts_with("/*") {
            match s[i + 2..].find("*/") {
                Some(end) => {
                    let skip_until = i + 2 + end + 2;
                    while chars.peek().is_some_and(|&(p, _)| p < skip_until) {
                        chars.next();
                    }
                    out.push(' ');
                    continue;
                }
                None => break,
            }
        }
        if c.is_ascii_whitespace() {
            out.push(' ');
            continue;
        }
        out.push(c);
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reader_missing(_name: &str) -> Option<String> {
        None
    }

    #[test]
    fn attr_grammar_matrix() {
        // 文法 valid 形（写面放行）。
        for v in [
            "attr(data-foo)",
            "attr(data-foo,)",
            "attr(data-foo, 3)",
            "attr(data-foo px)",
            "attr(data-foo %)",
            "attr(data-foo xx, 3px)",
            "attr(data-foo type(<length>))",
            "attr(data-foo type(<length>), red)",
            "attr(data-foo type(<length>), 42px)",
            "attr(data-foo type(<number> | lighter | bold), bold)",
            "attr(data-foo type(<color>), green)",
            "attr(data-foo type(<string>+))",
            "attr(data-foo type(*))",
            "attr(data-foo type(invalid | blue)",
            "attr(data-foo type(<length> | <percentage>)",
            "attr(data-foo raw-string)",
            "attr(data-foo number)",
            "attr(data-foo string)",
            "attr(non-existent, serif)",
            "attr(non-existent, inherit)",
            "attr(var(--x))",
            "attr(var(--x),)",
            "attr(var(--x), 10)",
            "attr(data-foo, calc(10s + 20s))",
            "attr(data-foo type(<color>)",
        ] {
            assert!(attr_calls_valid(v), "{v} 应 valid");
        }
        // 文法 invalid 形（写面拒）。
        for v in [
            "attr(!)",
            "attr(())",
            "attr()",
            "attr(data-foo &, 3px)",
            "attr(data-foo type(invalid |), 10px)",
            "attr(data-foo type(<invalid>)",
            "attr(a, b, c)",
            "attr(data-foo px type(<length>))",
        ] {
            assert!(!attr_calls_valid(v), "{v} 应 invalid");
        }
    }

    #[test]
    fn attr_string_default_resolution() {
        // 缺省字符串型：带引号（content 语料钉面）。
        let r = |_n: &str| -> Option<String> { Some("abc".into()) };
        assert_eq!(substitute_attr_values("attr(data-foo)", &r).unwrap(), "\"abc\"");
        // 属性值含 attr( 文本原样引用（语料 `attr(` 值形——字面内不重扫）。
        let r2 = |_n: &str| -> Option<String> { Some("attr(".into()) };
        assert_eq!(substitute_attr_values("attr(data-foo)", &r2).unwrap(), "\"attr(\"");
        // 缺属性无 fallback → IACVT None。
        assert!(substitute_attr_values("attr(data-foo)", &reader_missing).is_none());
        // 缺属性有 fallback → fallback 文本。
        assert_eq!(
            substitute_attr_values("attr(non-existent, serif)", &reader_missing).unwrap(),
            "serif"
        );
        // CSS wide 关键字 fallback 原样透传。
        assert_eq!(
            substitute_attr_values("attr(non-existent, inherit)", &reader_missing).unwrap(),
            "inherit"
        );
        // 字符串字面内 attr( 不识别。
        let rs = |_n: &str| -> Option<String> { Some("x".into()) };
        assert_eq!(
            substitute_attr_values("content: \"attr(data-foo)\"", &rs).unwrap(),
            "content: \"attr(data-foo)\""
        );
    }

    #[test]
    fn attr_unit_shorthand_resolution() {
        let r = |_n: &str| -> Option<String> { Some("10".into()) };
        assert_eq!(substitute_attr_values("attr(data-foo px)", &r).unwrap(), "10px");
        assert_eq!(substitute_attr_values("attr(data-foo %)", &r).unwrap(), "10%");
        // 小数归一（serialize-a-number 口径）。
        let r2 = |_n: &str| -> Option<String> { Some("3.0".into()) };
        assert_eq!(substitute_attr_values("attr(data-foo px)", &r2).unwrap(), "3px");
        // 值非数字 → 型不合 → fallback。
        let rp = |_n: &str| -> Option<String> { Some("abc".into()) };
        assert_eq!(substitute_attr_values("attr(data-foo px, 3px)", &rp).unwrap(), "3px");
    }

    #[test]
    fn attr_syntax_math_fold() {
        // type(<number>) 数学形折叠（语料 calc(3 + 3) 钉面）。
        let r = |_n: &str| -> Option<String> { Some("calc(3 + 3)".into()) };
        let got = substitute_attr_values("attr(data-foo type(<number>))", &r).unwrap();
        assert!(got == "calc(6)" || got == "6", "{got}");
        // type(<length>) 数学形折叠（语料 calc(1px + 3px) 钉面）。
        let r2 = |_n: &str| -> Option<String> { Some("calc(1px + 3px)".into()) };
        let got2 = substitute_attr_values("attr(data-foo type(<length>))", &r2).unwrap();
        assert!(got2 == "calc(4px)" || got2 == "4px", "{got2}");
        // 裸长度透传。
        let r3 = |_n: &str| -> Option<String> { Some("10px".into()) };
        assert_eq!(
            substitute_attr_values("attr(data-foo type(<length>))", &r3).unwrap(),
            "10px"
        );
    }

    #[test]
    fn attr_fallback_and_union() {
        // 型不合 → fallback（语料 type(<color>), red 值 '10' 形）。
        let r = |_n: &str| -> Option<String> { Some("10".into()) };
        assert_eq!(
            substitute_attr_values("attr(data-foo type(<color>), red)", &r).unwrap(),
            "red"
        );
        // union ident 分量命中。
        let rb = |_n: &str| -> Option<String> { Some("blue".into()) };
        assert_eq!(
            substitute_attr_values("attr(data-foo type(invalid | blue))", &rb).unwrap(),
            "blue"
        );
        // custom-ident 拒 css-wide → fallback。
        let ri = |_n: &str| -> Option<String> { Some("initial".into()) };
        assert_eq!(
            substitute_attr_values("attr(data-foo type(<custom-ident>), anim-fallback)", &ri).unwrap(),
            "anim-fallback"
        );
        // 延迟形（var）→ None（compute 期 var 代入后重裁）。
        assert!(substitute_attr_values("attr(var(--x))", &r).is_none());
        // 混合替换：前后文保留。
        let rp = |_n: &str| -> Option<String> { Some("5".into()) };
        assert_eq!(
            substitute_attr_values("0 0 attr(data-foo px) 0", &rp).unwrap(),
            "0 0 5px 0"
        );
    }

    #[test]
    fn attr_multiplier_and_fallback_typing() {
        // 乘数须紧贴分量：`<length> +`（空白分隔）文法拒，`<length>+` 合法。
        assert!(!attr_calls_valid("attr(data-foo type(<length> +), 3px)"));
        assert!(attr_calls_valid("attr(data-foo type(<length>+), 3px)"));

        // fallback 型校验（css-values-5 §attr-notations）：number-calc/裸数字
        // 不合 <length> → IACVT；time-calc 折叠命中；ident/缺省型原文透传。
        let r = |n: &str| -> Option<String> {
            match n {
                "abc" => Some("abc".into()),
                "t" => Some("10m".into()),
                _ => None,
            }
        };
        assert!(substitute_attr_values("attr(abc type(<length>), calc(10 + 20))", &r).is_none());
        assert!(substitute_attr_values("attr(abc type(<length>), 30)", &r).is_none());
        assert!(substitute_attr_values("attr(t type(<time>), calc(10s + 20s))", &r).is_some());
        assert_eq!(
            substitute_attr_values("attr(abc type(invalid), 10px)", &r).unwrap(),
            "10px"
        );
        // 单位型 fallback：量纲 token 原文、裸数字归一 number+unit。
        assert_eq!(substitute_attr_values("attr(non-existent px, 3px)", &r).unwrap(), "3px");
        assert_eq!(substitute_attr_values("attr(non-existent px, 3)", &r).unwrap(), "3px");
        // 型不合且 fallback 亦不合 → IACVT（`attr(data-foo px)` 值 '10px' 非
        // 裸数字、无 fallback——语料 invalid 面钉）。
        assert!(substitute_attr_values("attr(abc px)", &r).is_none());

        // 维度型 fallback 原文透传面（语料钉）：percentage-for-10px（42）/
        // angle-for-italic（50）/ custom-ident-for-3px（cycle 11）不合型不拒——
        // 交 cascade 按属性值文法裁决。
        assert_eq!(
            substitute_attr_values("attr(abc type(<percentage>), 10px)", &r).unwrap(),
            "10px"
        );
        assert_eq!(
            substitute_attr_values("attr(abc type(<angle>), italic)", &r).unwrap(),
            "italic"
        );
        assert_eq!(
            substitute_attr_values("attr(t type(<custom-ident>), 3px)", &r).unwrap(),
            "3px"
        );
        // angle 维度型拒 number-calc（语料 156 钉面）。
        assert!(substitute_attr_values("attr(t type(<angle>), calc(10 + 20))", &r).is_none());
    }
}
