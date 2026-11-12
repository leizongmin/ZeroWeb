//! CSS Values 5 [`random()`](https://drafts.csswg.org/css-values-5/#random)
//! 随机值文法（tentative 草案面，WPT css-values random-invalid/serialize 语料
//! 驱动）。
//!
//! 文法（草案 ED §random-function，组合/`auto`/`property-scoped` 形为 WebKit
//! 语料口径、草案未收）：
//! ```text
//! random() = random( <random-identification>? , <min-value>?, <max-value>, <step-value>? )
//! <random-identification> = --<dashed-ident> <random-scoping>* | <random-scoping>+
//!                         | fixed <number [0,1)> | auto
//! <random-scoping> = element-scoped | property-scoped | property-index-scoped
//!                  | ua-<ident>      （element 域 × UA 域各至多一，任意序组合；
//!                                      双 UA 域/同关键字重复拒）
//! ```
//!
//! 本模块裁文法（写面门消费）+ §9.4.1 `<random-key>` parse 期 canonical 化
//!（[`canonicalize_random_keys`]，写面 fold 入口消费）；求值（sha256 种子 +
//! element/property 上下文 + computed 数值化）挂账 engine 面，`CSS.supports`
//! 维持拒面。
//!
//! 语料钉面：
//! - 首节为数字（无 identification）→ 实参 2..=3 节（min,max[,step]）；带
//!   identification → 2..=4 节。
//! - min>max 合法（serialize 页 `random(--foo, 100px, 0px)` 行——求值期交换，
//!   文法门不裁值序）。
//! - identification 组合（serialize 页 71 接受形 / invalid 页 39 拒形反推）：
//!   element 域 `element-scoped` 与 UA 域 `property-scoped|property-index-scoped|
//!   ua-<ident>` 各至多一、任意序可共存（`element-scoped property-scoped` /
//!   `property-index-scoped element-scoped` 行），双 UA 域拒（`property-scoped
//!   ua-width-1` 形）；`--<dashed-ident>` 至多一且必须在 scoping 之前；`auto`
//!   为 WebKit shorthand 独占节（`random(auto, ...)` 行）；同关键字重复/未知
//!   关键字/裸 ident 拒（random-invalid 语料形）。
//! - 数字节同族一致（number/length-percentage/angle/time）；`%` 归 length 族
//!   （serialize 页 `random(--foo, 10px, 20%)` 行）。族随属性槽校验在写面门
//!   （[`random_family_allowed`]），此处只归类。

/// random() 实参数字族（§10.5 型别一致门）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomFamily {
    /// `<length>` | `<percentage>`（% 归 length 族）。
    Length,
    /// 无单位 `<number>`。
    Number,
    /// `<time>`（s/ms）。
    Time,
    /// `<angle>`（deg/rad/grad/turn）。
    Angle,
}

/// element 域 scoping 关键字（ASCII 大小写不敏感——CSS 关键字口径）。
fn is_element_scoped(tok: &str) -> bool {
    tok.eq_ignore_ascii_case("element-scoped")
}

/// UA 域 scoping 成分（property-scoped ≙ `ua-<prop>`、property-index-scoped
/// ≙ `ua-<prop>-<index>` 与显式 `ua-<ident>` 同属 UA 域，域内至多一个）。
fn is_ua_domain_scoping(tok: &str) -> bool {
    tok.eq_ignore_ascii_case("property-scoped")
        || tok.eq_ignore_ascii_case("property-index-scoped")
        || (tok.starts_with("ua-") && tok.len() > 3)
}

use super::serialize_math::format_dimension_inner;

/// 单位 → 数字族（`0<unit>` 归类；未知单位 None）。
fn family_of_unit(unit: &str) -> Option<RandomFamily> {
    Some(match unit.to_ascii_lowercase().as_str() {
        "px" | "em" | "rem" | "ex" | "ch" | "rex" | "rch" | "ic" | "ric" | "cap" | "rcap" | "lh" | "rlh" | "vh"
        | "vw" | "vmin" | "vmax" | "svh" | "lvh" | "dvh" | "svw" | "lvw" | "dvw" | "q" | "cm" | "mm" | "in" | "pt"
        | "pc" | "%" => RandomFamily::Length,
        "s" | "ms" => RandomFamily::Time,
        "deg" | "rad" | "grad" | "turn" => RandomFamily::Angle,
        _ => return None,
    })
}

/// 数字 token 拆值与单位：`[+-]?<digits>[.<digits>]?([eE]<digits>)?<unit>?`——
/// 按 CSS number 词法最长匹配数值前缀，余段为单位。返回 (数值, 单位
/// None=纯 number)；未知单位/无整数位/尾随点返 None。
fn number_token_parts(tok: &str) -> Option<(f64, Option<&str>)> {
    let b = tok.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == int_start {
        return None; // 无整数位（`.5`/纯符号/单位起手）
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return None; // `1.` 尾随点
        }
    }
    // 科学记数位：`e`/`E` 后有 `[+-]?<digits>` 才按指数消耗（`1ex` 的 e 后
    // 无数字 → 落单位解析路径，`1e5px` 的 5 被指数消耗、余 `px`）。
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    let value: f64 = tok[..i].parse().ok()?;
    let unit = &tok[i..];
    Some((value, if unit.is_empty() { None } else { Some(unit) }))
}

/// 顶层逗号分节（calc-mix 实参同口径——括号深度越零切分）。
fn split_sections(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in args.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(&args[start..idx]);
                start = idx + 1;
            }
            _ => {}
        }
    }
    out.push(&args[start..]);
    out
}

/// UA 域成分（§9.4 `<random-cache-key>` 的 UA 域归一结构）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum UaDomain {
    /// 无 UA 域成分。
    None,
    /// `property-scoped`（canonical 化替换为 `ua-<property>`）。
    PropertyScoped,
    /// `property-index-scoped`（替换为 `ua-<property>-<index>`）。
    PropertyIndexScoped,
    /// 显式 `ua-<ident>`（原样保留——含 `ua-height-1`/`ua-\{` 转义形）。
    Ident(String),
}

/// identification 节结构（canonical 化消费；文法门只用 has_fixed/family）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct RandomKey {
    /// `--<dashed-ident>` 原文本（含 `--\{` 转义形）。
    dashed: Option<String>,
    /// 显式 `element-scoped`。
    element_scoped: bool,
    /// UA 域成分。
    ua: UaDomain,
    /// `fixed <expr>` 实参原文（`calc(2 / 4)` 形含包裹）。
    fixed: Option<String>,
    /// `auto` 关键字独占节（WebKit shorthand）。
    is_auto: bool,
    /// key 节整个省略（首节即数字）。
    omitted: bool,
}

impl RandomKey {
    fn none() -> Self {
        Self {
            dashed: None,
            element_scoped: false,
            ua: UaDomain::None,
            fixed: None,
            is_auto: false,
            omitted: false,
        }
    }
}

/// identification 节解析（首节非单数字形）。成分（语料反推：serialize 页 71
/// 接受形 / invalid 页 39 拒形）：
/// - `fixed <expr>` 独占节——`<expr>` 为无单位 `<number [0,1)>` 单 token 或
///   lite 计算式（`calc(2 / 4)` 形，serialize 页 canonical `calc(0.5)` 钉）；
/// - `auto` 单 token 独占节；
/// - `--<dashed-ident>`（至多一、必须在任何 scoping 之前）× element 域
///   `element-scoped`（至多一）× UA 域 `property-scoped|property-index-scoped|
///   ua-<ident>`（至多一）的组合——element × UA 域任意序可共存，双 UA 域拒。
fn parse_key_section(section: &str) -> Option<RandomKey> {
    let trimmed = section.trim();
    let first_word = trimmed.split_whitespace().next()?;
    if first_word.eq_ignore_ascii_case("fixed") {
        let expr = trimmed[first_word.len()..].trim();
        if expr.is_empty() {
            return None;
        }
        // 实参求值：单 token 直拆；多 token 限 `calc(` 包裹 lite 计算式。
        let (val, unit) = if expr.contains(char::is_whitespace) {
            let lower = expr.to_ascii_lowercase();
            if !(lower.starts_with("calc(") && expr.ends_with(')')) {
                return None; // `fixed 0.5 auto` / 裸多 token 式拒（invalid 页形）
            }
            let (v, u) = eval_lite_expr(&expr[5..expr.len() - 1])?;
            (v, u)
        } else {
            let (v, u) = number_token_parts(expr)?;
            (v, u.map(str::to_string))
        };
        if unit.is_some() || !(0.0..1.0).contains(&val) {
            return None; // `fixed 0.5px` 非 <number> / `fixed -1` 出 [0,1)
        }
        return Some(RandomKey {
            fixed: Some(expr.to_string()),
            ..RandomKey::none()
        });
    }
    let toks: Vec<&str> = trimmed.split_whitespace().collect();
    if toks.len() == 1 && toks[0].eq_ignore_ascii_case("auto") {
        return Some(RandomKey {
            is_auto: true,
            ..RandomKey::none()
        });
    }
    let mut key = RandomKey::none();
    let mut ua: Option<UaDomain> = None;
    let mut element = 0usize;
    for tok in toks {
        if tok.starts_with("--") {
            // ident 至多一且必须最前（invalid 页
            // `element-scoped --foo --bar` 拒形口径）。
            if key.dashed.is_some() || element > 0 || ua.is_some() {
                return None;
            }
            key.dashed = Some(tok.to_string());
        } else if is_element_scoped(tok) {
            element += 1;
        } else if tok.eq_ignore_ascii_case("property-scoped") {
            if ua.is_some() {
                return None; // 双 UA 域
            }
            ua = Some(UaDomain::PropertyScoped);
        } else if tok.eq_ignore_ascii_case("property-index-scoped") {
            if ua.is_some() {
                return None;
            }
            ua = Some(UaDomain::PropertyIndexScoped);
        } else if is_ua_domain_scoping(tok) {
            // 显式 `ua-<ident>`（含转义形）。
            if ua.is_some() {
                return None;
            }
            ua = Some(UaDomain::Ident(tok.to_string()));
        } else {
            return None; // 裸 ident/`uawidth`/数字节混入等未知 token
        }
    }
    if element > 1 {
        return None; // `element-scoped element-scoped` 同关键字重复
    }
    key.element_scoped = element > 0;
    key.ua = ua.unwrap_or(UaDomain::None);
    Some(key)
}

/// lite 计算式求值（`<calc-sum>` 语料形：括号/乘除链/同单位和差）。产出
/// (值, 单位)；语法超纲（双操作数相邻、维度×维度、number/维度混除、除零、
/// 跨单位和差）返 None——写面门拒、canonical 保原文。
fn eval_lite_expr(expr: &str) -> Option<(f64, Option<String>)> {
    let toks = lite_tokens(expr)?;
    let mut pos = 0usize;
    let v = lite_sum(&toks, &mut pos)?;
    if pos != toks.len() {
        return None; // 尾随残 token（`1px foo` 形）
    }
    Some(v)
}

/// lite tokenizer：空白与括号边切分（`(1px + 2px) * 2` → `["(", "1px", "+",
/// "2px", ")", "*", "2"]`）。
fn lite_tokens(s: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '(' | ')' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(ch.to_string());
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() { None } else { Some(out) }
}

type LiteVal = (f64, Option<String>);

fn lite_sum(toks: &[String], pos: &mut usize) -> Option<LiteVal> {
    let mut acc = lite_product(toks, pos)?;
    while let Some(op) = toks.get(*pos).filter(|t| t.as_str() == "+" || t.as_str() == "-") {
        *pos += 1;
        let rhs = lite_product(toks, pos)?;
        let (av, au) = acc;
        let (bv, bu) = rhs;
        if au != bu {
            return None; // 跨单位和差超纲（`1px + 1em`）
        }
        acc = (if op == "+" { av + bv } else { av - bv }, au);
    }
    Some(acc)
}

fn lite_product(toks: &[String], pos: &mut usize) -> Option<LiteVal> {
    let mut acc = lite_unit(toks, pos)?;
    while let Some(op) = toks.get(*pos).filter(|t| t.as_str() == "*" || t.as_str() == "/") {
        *pos += 1;
        let rhs = lite_unit(toks, pos)?;
        let (av, au) = acc;
        let (bv, bu) = rhs;
        acc = if op == "*" {
            match (&au, &bu) {
                // 维度×维度超纲；单位随有维度侧透传（`10 * 100px` → 1000px）。
                (Some(_), Some(_)) => return None,
                (u, None) => (av * bv, u.clone()),
                (None, u) => (av * bv, u.clone()),
            }
        } else {
            match (&au, &bu) {
                (_, None) if bv == 0.0 => return None,
                (_, None) => (av / bv, au.clone()),
                (Some(_), Some(_)) if au == bu && bv != 0.0 => (av / bv, None),
                _ => return None, // number/维度混除、维度互除跨单位
            }
        };
    }
    Some(acc)
}

fn lite_unit(toks: &[String], pos: &mut usize) -> Option<LiteVal> {
    let tok = toks.get(*pos)?;
    *pos += 1;
    if tok == "(" {
        let v = lite_sum(toks, pos)?;
        let close = toks.get(*pos)?;
        if close != ")" {
            return None;
        }
        *pos += 1;
        return Some(v);
    }
    let (v, unit) = number_token_parts(tok)?;
    Some((v, unit.map(str::to_string)))
}

/// 实参节 canonical 化：单 token 形单位归一（ms→s、turn/grad/rad→deg——
/// §10.6 canonical 单位，余单位原样保往返）；多 token 形 lite 求值（`10 *
/// 100px` → `1000px`、`calc(2 / 4)` → `calc(0.5)` 保包裹）。求值失败返 None。
/// 产出 (canonical 文本, 族)。
fn canonical_section(section: &str) -> Option<(String, RandomFamily)> {
    let trimmed = section.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.contains(char::is_whitespace) {
        let (v, unit) = number_token_parts(trimmed)?;
        let fam = match unit {
            None => RandomFamily::Number,
            Some(u) => family_of_unit(u)?,
        };
        let text = match unit.map(str::to_ascii_lowercase) {
            // 单位归一仅换算面（ms/turn/grad/rad）；canonical 单位（s/deg）
            // 与物理/字体/百分比单位原样保往返（`50s`/`1em`/`200rem` 语料行）。
            Some(ref u) if u == "ms" => format_dimension_inner(v / 1000.0, Some("s")),
            Some(ref u) if u == "turn" => format_dimension_inner(v * 360.0, Some("deg")),
            Some(ref u) if u == "grad" => format_dimension_inner(v * 0.9, Some("deg")),
            Some(ref u) if u == "rad" => format_dimension_inner(v.to_degrees(), Some("deg")),
            _ => trimmed.to_string(),
        };
        return Some((text, fam));
    }
    // 多 token：calc 包裹剥离（保留包裹形态）。
    let lower = trimmed.to_ascii_lowercase();
    let wrapped = lower.starts_with("calc(") && trimmed.ends_with(')');
    let inner = if wrapped {
        &trimmed[5..trimmed.len() - 1]
    } else {
        trimmed
    };
    let (v, unit) = eval_lite_expr(inner)?;
    let fam = match &unit {
        None => RandomFamily::Number,
        Some(u) => family_of_unit(u)?,
    };
    let body = format_dimension_inner(v, unit.as_deref());
    Some((if wrapped { format!("calc({body})") } else { body }, fam))
}

/// `fixed <expr>` 实参 canonical 化：`calc(` 包裹形 lite 求值规范
///（`calc(2 / 4)` → `calc(0.5)` 语料钉）；单 token 形不重写（`fixed 0.5`
/// 原样往返）。
fn canonical_fixed_arg(expr: &str) -> Option<String> {
    let trimmed = expr.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("calc(") && trimmed.ends_with(')')) {
        return None;
    }
    let (v, unit) = eval_lite_expr(&trimmed[5..trimmed.len() - 1])?;
    if unit.is_some() {
        return None;
    }
    Some(format!("calc({})", format_dimension_inner(v, None)))
}

/// canonical key 文本（§9.4.1 Simplification）：序归一 `[--ident]
/// element-scoped ua-ident`——
/// - auto/省略 key 展开 `element-scoped ua-<property>-<index>`；
/// - `property-scoped` → `ua-<property>`、`property-index-scoped` →
///   `ua-<property>-<index>`（index = 声明值内 random 函数 1 基序，canonical
///   重排前文本序——§9.4.1 INDEX 口径）；仅显式 element-scoped 不加 ua-ident；
/// - 显式 `--ident`/`ua-<ident>` 原样保留；
/// - `fixed <expr>` 独占（实参按 [`canonical_fixed_arg`]）。
fn canonical_key_text(key: &RandomKey, property: &str, index: usize) -> String {
    if let Some(fx) = &key.fixed {
        let arg = canonical_fixed_arg(fx).unwrap_or_else(|| fx.clone());
        return format!("fixed {arg}");
    }
    let mut parts: Vec<String> = Vec::new();
    if let Some(d) = &key.dashed {
        parts.push(d.clone());
    }
    let auto_expanded = key.is_auto || key.omitted;
    if key.element_scoped || auto_expanded {
        parts.push("element-scoped".to_string());
    }
    let ua = match &key.ua {
        UaDomain::None => auto_expanded.then(|| format!("ua-{property}-{index}")),
        UaDomain::PropertyScoped => Some(format!("ua-{property}")),
        UaDomain::PropertyIndexScoped => Some(format!("ua-{property}-{index}")),
        UaDomain::Ident(s) => Some(s.clone()),
    };
    if let Some(u) = ua {
        parts.push(u);
    }
    parts.join(" ")
}

/// random() 调用解析结果（canonical 化消费；文法门经
/// [`parse_random_args`] 取 family/has_fixed 投影）。
struct RandomParts {
    key: RandomKey,
    /// 数字节 canonical 文本（key 省略形自首节起，否则自第 2 节起）。
    sections: Vec<String>,
    family: RandomFamily,
    has_fixed: bool,
}

/// 解析 random() 实参原文（括号内、未 trim）：identification 节结构化 +
/// 数字节 canonical 化 + 族一致门。文法 invalid 返 None。
fn parse_random_parts(args: &str) -> Option<RandomParts> {
    let sections = split_sections(args);
    if !(2..=4).contains(&sections.len()) {
        return None;
    }
    // 首节形态：identification 节解析成功 → key 形；失败而按数字节
    // canonical 化成功（单 token 数字或 calc-sum 多 token，`10 * 100px`
    // 语料形）→ key 省略形；两者皆否 → invalid。
    let key = match parse_key_section(sections[0]) {
        Some(k) => k,
        None if canonical_section(sections[0]).is_some() => RandomKey {
            omitted: true,
            ..RandomKey::none()
        },
        None => return None,
    };
    let omitted = key.omitted;
    let has_fixed = key.fixed.is_some();
    if key.omitted && sections.len() > 3 {
        return None; // key 省略 → min,max[,step] ≤3 节
    }
    let mut family: Option<RandomFamily> = None;
    let mut canon = Vec::new();
    for sec in sections.iter().skip(if key.omitted { 0 } else { 1 }) {
        let (text, fam) = canonical_section(sec)?;
        match family {
            None => family = Some(fam),
            Some(f) if f == fam => {}
            Some(_) => return None, // 跨族混（§10.5 一致门）
        }
        canon.push(text);
    }
    Some(RandomParts {
        key,
        sections: canon,
        family: family?,
        has_fixed,
    })
}

/// random() 文法解析结果（文法门消费；求值期扩展字段挂账）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSpec {
    /// 数字实参族（全部数字节一致）。
    pub family: RandomFamily,
    /// `fixed <number>` 形（求值期直取固定值跳 PRNG）。
    pub has_fixed: bool,
}

/// 解析 random() 实参原文（括号内、未 trim）。文法 valid 返 [`RandomSpec`]，
/// invalid 返 None。
///
/// 消费面：style-system 写面门（`validate_decl_value` 的 random 臂）——
/// invalid 形整拒（random-invalid 语料 39 形），valid 形放行走
/// [`canonicalize_random_keys`] canonical 串化。
pub fn parse_random_args(args: &str) -> Option<RandomSpec> {
    let parts = parse_random_parts(args)?;
    Some(RandomSpec {
        family: parts.family,
        has_fixed: parts.has_fixed,
    })
}

/// 值内下一 random 函数词边界命中（`random(` / `random-item(`——前随
/// ident 字母/数字/`-`/`_`/非 ASCII 的命中属更长 ident，如 `foorandom(`）。
/// 返回 (起始字节偏移, 实参开括号偏移, 是否 random-item)。输入为小写化
/// 切片（调用方以偏移基点换算绝对位）。
fn next_random_call(lower: &str) -> Option<(usize, usize, bool)> {
    let mut search = 0usize;
    while search < lower.len() {
        let off = lower[search..].find("random")?;
        let start = search + off;
        let prev_ok = start == 0
            || match lower[..start].chars().next_back() {
                None => true,
                Some(c) => !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c >= '\u{80}'),
            };
        let rest = &lower[start + 6..];
        let (is_item, call_len) = if rest.starts_with('(') {
            (false, "random(".len())
        } else if rest.starts_with("-item(") {
            (true, "random-item(".len())
        } else {
            search = start + 6;
            continue;
        };
        if prev_ok {
            return Some((start, start + call_len - 1, is_item));
        }
        search = start + call_len;
    }
    None
}

/// 括号配平（字节扫描——UTF-8 续字节 ≥ 0x80 不会撞 ASCII 括号）。返回配对
/// `)` 字节位。
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

/// CSS Values 5 §9.4.1 [Simplification of
/// `<random-key>`](https://drafts.csswg.org/css-values-5/#random-key-simplification)
/// ——parse 期 canonical 化（写面 fold 入口消费，custom property 值由调用方
/// 守卫跳过——§9.4 substitution 语义 random 在未注册自定义属性内不识别）：
///
/// - auto/省略 key 展开、property-scoped/index-scoped 换 `ua-` ident、序归一
///   与实参节单位归一/lite 折叠（详见 [`canonical_key_text`]/
///   [`canonical_section`]）；
/// - index = 值内全部 random 函数（random() + random-item()）1 基序；
/// - random-item() 为 arbitrary substitution 函数——指定值原样（item-serialize
///   语料钉面），仅计入 index 序；
/// - invalid 调用不改写（写面门整拒；复合上下文保原文）。
///
/// 值无 random( 或全部调用已 canonical → None（调用方保留原文）。
pub fn canonicalize_random_keys(value: &str, property: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let mut out = String::with_capacity(value.len());
    let mut changed = false;
    let mut ordinal = 0usize;
    let mut in_string: Option<char> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((pos, ch)) = chars.next() {
        // 字符串字面状态机：引号内函数名不识别（`content: "random(...)"`
        // 字符串值合法含函数名文本——t54 非 math 形放行同口径）。
        if let Some(q) = in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some(&(_, esc)) = chars.peek() {
                    out.push(esc);
                    chars.next();
                }
            } else if ch == q {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            out.push(ch);
            continue;
        }
        // 词边界 + 函数名匹配（lower 与 value 同字节布局——to_ascii_lowercase
        // 字节级 ASCII，切片偏移通用）。
        let rel = &lower[pos..];
        let boundary_ok = pos == 0
            || match value[..pos].chars().next_back() {
                None => true,
                Some(c) => !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c >= '\u{80}'),
            };
        if !boundary_ok || !rel.starts_with("random") {
            out.push(ch);
            continue;
        }
        let Some((rel_start, rel_open, is_item)) = next_random_call(rel) else {
            out.push(ch);
            continue;
        };
        let call_start = pos + rel_start;
        let open_abs = pos + rel_open;
        let Some(close) = match_paren(&lower, open_abs) else {
            out.push(ch);
            continue;
        };
        ordinal += 1;
        if is_item {
            // 指定值原样（arbitrary substitution 序列化不 canonical 化）。
            out.push_str(&value[call_start..=close]);
        } else if let Some(parts) = parse_random_parts(&value[open_abs + 1..close]) {
            let mut rebuilt = String::from("random(");
            rebuilt.push_str(&canonical_key_text(&parts.key, property, ordinal));
            for sec in &parts.sections {
                rebuilt.push_str(", ");
                rebuilt.push_str(sec);
            }
            rebuilt.push(')');
            let orig = &value[call_start..=close];
            if rebuilt != *orig {
                changed = true;
                out.push_str(&rebuilt);
            } else {
                out.push_str(orig);
            }
        } else {
            // invalid 形不改写（写面门整拒；复合上下文保原文）。
            out.push_str(&value[call_start..=close]);
        }
        // 跳过已消费段。
        while chars.peek().is_some_and(|&(p, _)| p <= close) {
            chars.next();
        }
    }
    changed.then_some(out)
}

/// 族随属性槽校验（写面门消费）：写面白名单属性按槽位文法族收口；未列槽
/// （complex 列表/无语料钉面）放行。
pub fn random_family_allowed(prop: &str, family: RandomFamily) -> bool {
    match prop {
        "width" | "margin-left" | "text-indent" | "transform-origin-z" => family == RandomFamily::Length,
        "scale" | "z-index" => family == RandomFamily::Number,
        "transition-delay" | "transition-duration" | "animation-delay" | "animation-duration" => {
            family == RandomFamily::Time
        }
        "rotate" => family == RandomFamily::Angle,
        // transform/translate 等 complex 槽：random 嵌入无语料钉面，放行
        _ => true,
    }
}
