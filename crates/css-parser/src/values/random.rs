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
        // keyword 数字节首节（`random(NaN, 100)` / `random(infinity, 100)`
        // 语料形——key 省略 + 数字节 keyword 臂）。
        None if infinity_nan_keyword_text(sections[0]).is_some() || canonical_section(sections[0]).is_some() => {
            RandomKey {
                omitted: true,
                ..RandomKey::none()
            }
        }
        None => return None,
    };
    let has_fixed = key.fixed.is_some();
    if key.omitted && sections.len() > 3 {
        return None; // key 省略 → min,max[,step] ≤3 节
    }
    let mut family: Option<RandomFamily> = None;
    let mut canon = Vec::new();
    let mut keywords = 0usize;
    for sec in sections.iter().skip(if key.omitted { 0 } else { 1 }) {
        // ∞/NaN keyword 节（css-values-4 §10.10；random-computed 语料 infinity
        // 组——keyword 无固有型别，一致型别由其余数字节锚定）。
        if let Some(kw) = infinity_nan_keyword_text(sec) {
            keywords += 1;
            canon.push(kw.to_string());
            continue;
        }
        let (text, fam) = canonical_section(sec)?;
        match family {
            None => family = Some(fam),
            Some(f) if f == fam => {}
            Some(_) => return None, // 跨族混（§10.5 一致门）
        }
        canon.push(text);
    }
    // 全 keyword 节（`random(infinity, infinity)` 语料形）无维度锚定——按
    // <number> 型别（keyword 型别灵活；该形仅 WPT scale 数槽钉面）。
    let family = match family {
        Some(f) => f,
        None if keywords > 0 => RandomFamily::Number,
        None => return None,
    };
    Some(RandomParts {
        key,
        sections: canon,
        family,
        has_fixed,
    })
}

/// ∞/NaN keyword 节识别（ASCII 大小写不敏感，CSS keyword 口径）。返回
/// canonical 文本（`infinity` / `-infinity` / `NaN`）。
fn infinity_nan_keyword_text(sec: &str) -> Option<&'static str> {
    let t = sec.trim();
    // css-values-4 §10.10：`infinity` / `-infinity` / `NaN` calc keyword。
    if t.eq_ignore_ascii_case("infinity") {
        Some("infinity")
    } else if t.eq_ignore_ascii_case("-infinity") {
        Some("-infinity")
    } else if t.eq_ignore_ascii_case("nan") {
        Some("NaN")
    } else {
        None
    }
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

// ── computed 期播种求值替换（CSS Values 5 §9.3 Evaluating Random Values +
// §9.4 Sharing (Or Not) Random Values；apply 入口消费）──

/// 随机求值上下文（random cache name 身份项，§9.4 四元组的 document/element
/// 分量）。spec 要求 document/element 标识与对应 JS 引用同 lifetime/等价
/// 语义——UA-defined 随机源无跨进程契约，调用方以进程内稳定派生（Document
/// 地址 / NodeId 哈希）满足。
/// <https://drafts.csswg.org/css-values-5/#random-sharing>
#[derive(Debug, Clone, Copy)]
pub struct RandomEvalContext {
    /// 文档标识（四元组第 4 项；全部调用参与 seed）。
    pub doc_seed: u64,
    /// 元素标识（element-scoped 调用参与 seed；None = 无元素视图 →
    /// element-scoped 调用 fail-closed——§9.4 元素标识缺失）。
    pub element_seed: Option<u64>,
}

impl RandomEvalContext {
    /// 支持性/合法性哨兵上下文：seed 无关判定（CSS.supports、cascade
    /// apply-on-dummy——值是否可求值与具体 seed 无关；哨兵保证全形可解析，
    /// element-scoped 也出值）。
    pub const SENTINEL: Self = Self {
        doc_seed: 0,
        element_seed: Some(0),
    };
}

/// `fixed <number [0,1]>` 语义的最高可表示 <1 值（spec：base 钳到 1 以下最高
/// 可表示值，保持 [0,1) 半开区间）。f64 prev(1.0) = 1 - 2⁻⁵³。
const FIXED_MAX_LESS_THAN_ONE: f64 = 0.9999999999999999;

/// 值内 random()/random-item() 调用播种求值替换（§9.3 公式 + §9.1.1 实参域
/// 规则；§9.4.1 播种）。字符串字面内的函数名不识别（canonicalize 同状态机）。
///
/// 返回：
/// - 无 random 调用 → 借用原文（零分配快路径）；
/// - 全调用可解析 → 替换文本（每调用替换为求值结果，NaN/±∞ 以 `calc(...)`
///   keyword 形保留——消费面 censor（NaN→0、∞ 钳）既有承担，css-values-4
///   §10.10/§10.12）；
/// - 调用含 % 叶 → 该调用替换为 fixed-echo 保留形 `random(fixed <R>, ...)`——
///   §9.3 unresolvable percentage：计算值保留 random()，key 换显式
///   `fixed <R>`（显式 fixed 原样保留；无 fixed key 的以播种 R 补显）；
/// - random-item() 含 var() 实参 / element-scoped 无元素视图 / 文法 invalid
///   → None（声明 fail-closed，与旧 parse 失败同观测）。
///
/// `property`：property-scoped/index-scoped key 展开（var() 代入值未经写面
/// canonical 化，展开在本函数内补；canonical 存文 key 已含 `ua-` ident，
/// 展开幂等）。序数：值内 random/random-item 调用 1 基合并序（写面
/// canonicalize 同口径，ua-<prop>-<index> 展开）；fixed key 内嵌套 random 先
/// 内层替换（实参递归一轮；嵌套深度 ≤ 输入长度，语料 ≤ 2）。
/// <https://drafts.csswg.org/css-values-5/#random-evaluating>
pub fn substitute_random_values<'a>(
    value: &'a str,
    property: &str,
    ctx: &RandomEvalContext,
) -> Option<std::borrow::Cow<'a, str>> {
    // 快路径门：无 random 词原样返回（覆盖 random( 与 random-item( 两函数；
    // 大小写混合覆盖 CSS 作者面，canonical 存文恒小写）。
    let lower = value.to_ascii_lowercase();
    if !lower.contains("random") {
        return Some(std::borrow::Cow::Borrowed(value));
    }
    let mut out = String::with_capacity(value.len());
    let mut ordinal = 0usize;
    let mut in_string: Option<char> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((pos, ch)) = chars.next() {
        // 字符串字面状态机：引号内函数名不识别（`content: "random(...)"`
        // 字符串值合法含函数名文本——canonicalize 同口径）。
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
        let Some((_, rel_open, is_item)) = next_random_call(rel) else {
            out.push(ch);
            continue;
        };
        let open_abs = pos + rel_open;
        let Some(close) = match_paren(&lower, open_abs) else {
            out.push(ch);
            continue;
        };
        ordinal += 1;
        let result = if is_item {
            eval_random_item_call(&value[open_abs + 1..close], property, ordinal, ctx)?
        } else {
            eval_random_call(&value[open_abs + 1..close], property, ordinal, ctx)?
        };
        out.push_str(&result);
        // 跳过已消费段。
        while chars.peek().is_some_and(|&(p, _)| p <= close) {
            chars.next();
        }
    }
    Some(std::borrow::Cow::Owned(out))
}

/// 单个 random() 调用求值（§9.3 公式；返回替换文本——数值随族带单位，
/// NaN/±∞ 以 `calc(...)` keyword 形保留；% 叶以 fixed-echo 保留形返回）。
fn eval_random_call(args: &str, property: &str, ordinal: usize, ctx: &RandomEvalContext) -> Option<String> {
    // fixed 实参内嵌 random（`fixed random(a, b)` 语料形）：先递归替换内层
    // 调用（fixed 直取数值不经 seed；写面 key 保原文文本，两边一致）；替换
    // 产物 fixed 字面钳 [0,1)（替换后不再过写面字面 range 门——§9.4 clamp）。
    let owned;
    let clamped;
    let args_eff: &str = if args.to_ascii_lowercase().contains("random(") {
        owned = substitute_random_values(args, property, ctx)?.into_owned();
        clamped = clamp_substituted_fixed(&owned);
        &clamped
    } else {
        args
    };
    let parts = parse_random_parts(args_eff)?;
    // 数字节求值（canonical 文本 calc() 包裹解析——单 token 数字/维度与
    // lite 折叠结果同路）。
    let mut vals = Vec::with_capacity(parts.sections.len());
    for sec in &parts.sections {
        // Time 族走 allow_time 解析（<time> 原子仅 parse_time_calc 接受，
        // parse_math_function 默认拒）；其余族共用入口。
        let wrapped = format!("calc({sec})");
        let expr = if parts.family == RandomFamily::Time {
            super::parse_time_calc(&wrapped)?
        } else {
            super::parse_math_function(&wrapped)?
        };
        if super::calc_mix::expr_has_percentage_leaf(&expr) {
            // §9.3 unresolvable percentage：计算值保留 random()，key 换显式
            // `fixed <R>`——显式 fixed 原文回显；播种形以 base 值补显
            //（element-scoped 无元素视图仍 fail-closed）。
            let f = match &parts.key.fixed {
                Some(fx) => canonical_fixed_arg(fx).unwrap_or_else(|| fx.clone()),
                None => {
                    if key_element_scoped(&parts.key) && ctx.element_seed.is_none() {
                        return None;
                    }
                    format!("{}", random_base_value(&parts.key, property, ordinal, ctx))
                }
            };
            return Some(format!("random(fixed {}, {})", f, parts.sections.join(", ")));
        }
        let v = super::eval_calc_with_default_font(&expr)?;
        // Angle 叶解析为弧度 Number（trig 口径）——回转 degree 域（serialize
        // 面单位 deg；NaN/±∞ 乘正常数不变号不丢毒）。
        vals.push(if parts.family == RandomFamily::Angle {
            v.to_degrees()
        } else {
            v
        });
    }
    let min = vals.first().copied()?;
    let max = vals.get(1).copied()?;
    let step = vals.get(2).copied();
    // random base value：fixed 直取（[0,1) 钳制，§9.4）；否则按 cache name
    // 四元组播种（§9.4.1 canonical key 文本即 name 的 (name, ua-ident) 分量；
    // element-scoped 无元素视图 fail-closed）。
    let base = match &parts.key.fixed {
        Some(fx) => fixed_base_value(fx)?,
        None => {
            if key_element_scoped(&parts.key) && ctx.element_seed.is_none() {
                return None;
            }
            random_base_value(&parts.key, property, ordinal, ctx)
        }
    };
    let v = eval_random_formula(base, min, max, step);
    Some(serialize_random_result(v, parts.family))
}

/// key 是否元素相关（canonical 文本含 `element-scoped`——显式声明或 auto/
/// 省略展开，[`canonical_key_text`] 同口径）：seed 混入 element 分量与
/// 无元素视图 fail-closed 判定共用。
fn key_element_scoped(key: &RandomKey) -> bool {
    key.element_scoped || key.is_auto || key.omitted
}

/// cache name → random base value ∈ [0,1)（UA-defined 方法——spec 只约束
/// 同 name 同值、异 name 独立）。sha256(doc ‖ [element] ‖ canonical key)，
/// 取前 8 字节高 53 位均匀映射 [0,1)。
fn random_base_value(key: &RandomKey, property: &str, ordinal: usize, ctx: &RandomEvalContext) -> f64 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(ctx.doc_seed.to_le_bytes());
    if key_element_scoped(key) {
        hasher.update(ctx.element_seed.unwrap_or(0).to_le_bytes());
    }
    hasher.update(canonical_key_text(key, property, ordinal).as_bytes());
    let digest = hasher.finalize();
    let n = u64::from_le_bytes(digest[..8].try_into().expect("sha256 digest ≥ 8 bytes"));
    // 高 53 位 × 2⁻⁵³：精确均匀 [0,1)（永不到 1）。
    (n >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// `fixed <expr>` 实参 → base ∈ [0,1)：单 token 数字或 lite 计算式，越界钳制
///（§9.4 fixed <number [0,1]>：1 语法合法但钳至最高可表示 <1 值）。
fn fixed_base_value(fx: &str) -> Option<f64> {
    let t = fx.trim();
    let v = if let Some(inner) = t.strip_prefix("calc(").and_then(|s| s.strip_suffix(')')) {
        eval_lite_expr(inner)?.0
    } else {
        t.parse::<f64>().ok()?
    };
    if v.is_nan() {
        return Some(0.0);
    }
    Some(v.clamp(0.0, FIXED_MAX_LESS_THAN_ONE))
}

/// §9.1.1 实参域规则 + §9.3 求值公式：
/// - 任一实参 NaN → 结果 NaN（NaN 传染，消费面 censor NaN→0）；
/// - min 无穷 → 结果无穷（同号）；
/// - max < min → max=min（§9.1 behaves as if equal to the minimum——钳制
///   非交换，`random(100, 10)` → 100 语料钉面）；
/// - (max-min) 无穷（min 有限）→ NaN；
/// - step 无穷 → 结果恰 min；step ≤ 0 或步长乘数区间无穷 → step 忽略；
/// - 无 step → `min + R·(max-min)`；有 step → [min, max] 上 `min + N·step`
///   均匀取格（epsilon = step/1000 max 修入；index = floor(R·(N+1))）。
fn eval_random_formula(r: f64, min: f64, max: f64, step: Option<f64>) -> f64 {
    if min.is_nan() || max.is_nan() || step.is_some_and(|s| s.is_nan()) {
        return f64::NAN;
    }
    if min.is_infinite() {
        return min;
    }
    let max = if max < min { min } else { max };
    let span = max - min;
    if span.is_infinite() {
        return f64::NAN;
    }
    match step {
        Some(s) if s > 0.0 => {
            if s.is_infinite() {
                // step +∞：格点集退化为 {min}——random integer < N+1 而
                // N = floor(span/∞) = 0 → index 0（语料 `random(10, 100,
                // infinity)` → '10' 钉面）。
                return min;
            }
            let n_exact = span / s;
            if !n_exact.is_finite() {
                // 步长乘数区间无穷 → step 忽略（§9.1.1）。
                return min + r * span;
            }
            // N = 最大整数使 min + N·step ≤ max；epsilon（step/1000，下溢取
            // 最小可表示正数）修入 max 近邻（§9.3 epsilon fixup）。
            let epsilon = if s / 1000.0 > 0.0 {
                s / 1000.0
            } else {
                f64::MIN_POSITIVE
            };
            let mut n = n_exact.floor();
            if max - (min + n * s) > epsilon {
                let next = min + (n + 1.0) * s;
                if (next - max).abs() <= epsilon {
                    n += 1.0;
                }
            }
            // step index：random integer < N+1（§9.3 generate a random integer
            // = round(down, R·limit, 1)）。
            let index = (r * (n + 1.0)).floor().clamp(0.0, n);
            let value = min + index * s;
            if index >= n && (max - value).abs() <= epsilon {
                max
            } else {
                value
            }
        }
        _ => min + r * span,
    }
}

/// 求值结果 → 替换文本：数值随族带单位（Length px / Number 无单位 / Angle
/// deg / Time s）；NaN/±∞ 以 `calc(...)` keyword 形保留——`parse::<f64>`
/// 快路径拒非有限值（transform_helpers finite 门），calc 包裹走 math 路径
/// 放行由消费面 censor（§10.10/§10.12）。
fn serialize_random_result(v: f64, family: RandomFamily) -> String {
    let unit = match family {
        RandomFamily::Length => "px",
        RandomFamily::Number => "",
        RandomFamily::Time => "s",
        RandomFamily::Angle => "deg",
    };
    if v.is_nan() {
        return "calc(NaN)".to_string();
    }
    if v.is_infinite() {
        return if v < 0.0 {
            "calc(-infinity)".to_string()
        } else {
            "calc(infinity)".to_string()
        };
    }
    format!("{v}{unit}")
}

/// random-item() item 列表切分：顶层逗号分割——`{}` 花括号包裹项（含逗号
/// item）与 `()` 函数/嵌套整体计深，字符串字面内逗号不识别（canonicalize
/// 同状态机）。返回原切片（花括号保留，调用方剥壳）。
fn split_item_list(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth_paren = 0usize;
    let mut depth_brace = 0usize;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    let mut start = 0usize;
    for (idx, ch) in args.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if let Some(q) = in_string {
            if ch == '\\' {
                escaped = true;
            } else if ch == q {
                in_string = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => in_string = Some(ch),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            ',' if depth_paren == 0 && depth_brace == 0 => {
                out.push(&args[start..idx]);
                start = idx + 1;
            }
            _ => {}
        }
    }
    out.push(&args[start..]);
    out
}

/// 嵌套 fixed 内层替换产物钳制（`fixed random(1, 2)` → `fixed 1.x` →
/// `fixed 0.9999999999999999`；`fixed random(-2, -1)` → `fixed 0`）：替换
/// 产物不再过写面字面 range 门（substitution 语义），求值期按 §9.4 clamp
/// 到 [0,1)。仅首节 fixed 单 token 数字形（语料形）；calc 包裹形（语料
/// 缺席）不动，交 fixed_base_value 求值钳。
fn clamp_substituted_fixed(args: &str) -> String {
    let trimmed = args.trim();
    let (first, rest) = match trimmed.find(',') {
        Some(i) => (&trimmed[..i], Some(&trimmed[i..])),
        None => (trimmed, None),
    };
    let mut it = first.splitn(2, char::is_whitespace);
    let kw = it.next().unwrap_or_default();
    let num = it.next().unwrap_or_default();
    if !kw.eq_ignore_ascii_case("fixed") || num.contains(char::is_whitespace) {
        return args.to_string();
    }
    let Some((v, None)) = number_token_parts(num) else {
        return args.to_string();
    };
    if (0.0..=1.0).contains(&v) {
        return args.to_string();
    }
    // 负域钳 0（fixed_base_value clamp 同口径）；上域钳最高可表示 <1。
    let clamped = if v < 0.0 { "fixed 0" } else { "fixed 0.9999999999999999" };
    match rest {
        Some(r) => format!("{clamped}{r}"),
        None => clamped.to_string(),
    }
}

/// 单个 random-item() 调用求值（§9.5 Random values from a list——播种同
/// §9.4，选中 item = `floor(R × N)` 钳 `[0, N-1]`；返回选中 item 文本整体
/// 替换调用，未选 item 从计算值消失）。首节按 `<random-key>` 文法贪心匹配
///（spec 同序——item 文本恰为 key keyword 形时归 key）；`var(` 实参
/// fail-closed（注册 custom prop 代入序挂账）。
fn eval_random_item_call(args: &str, property: &str, ordinal: usize, ctx: &RandomEvalContext) -> Option<String> {
    if args.to_ascii_lowercase().contains("var(") {
        return None;
    }
    let sections = split_item_list(args);
    // 首节 key 判定：花括号起始节非 key（包裹 item）；parse_key_section
    // 结构化成功 → key 形（省略 key 首节即 item，parse 失败落 item 形）。
    let mut items_start = 0usize;
    let key = match sections.first() {
        Some(first) if !first.trim().starts_with('{') => parse_key_section(first).inspect(|_| items_start = 1),
        _ => None,
    };
    let items = &sections[items_start..];
    // 文法门：≥1 item 且无空 item（`a,,b` 违反 <value>#；全空 invalid）。
    if items.is_empty() || items.iter().any(|s| s.trim().is_empty()) {
        return None;
    }
    let base = match &key {
        Some(k) if k.fixed.is_some() => {
            // key 节经 parse_key_section 文法门（fixed 单 token 数字/calc-lite
            // 形，无内嵌 random）——直取数值。
            fixed_base_value(k.fixed.as_deref().unwrap_or_default())?
        }
        Some(k) => {
            if key_element_scoped(k) && ctx.element_seed.is_none() {
                return None;
            }
            random_base_value(k, property, ordinal, ctx)
        }
        // key 省略 → auto 展开（canonical_key_text 同口径
        // element-scoped ua-<property>-<index>）；无元素视图 fail-closed。
        None => {
            ctx.element_seed?;
            random_base_value(
                &RandomKey {
                    omitted: true,
                    ..RandomKey::none()
                },
                property,
                ordinal,
                ctx,
            )
        }
    };
    let n = items.len();
    let index = (base * n as f64).floor().clamp(0.0, (n - 1) as f64) as usize;
    let item = items[index].trim();
    // `{}` 花括号包裹项剥壳（包裹仅为保顶层逗号，壳不属值）。
    let stripped = item
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(item);
    // 选中 item 内嵌 random：递归替换。FIXME: 内层调用 ordinal 独立起计，
    // 与外层序不互斥（嵌套 random-item 语料缺席，g21 不展开）。
    substitute_random_values(stripped, property, ctx).map(std::borrow::Cow::into_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 元素视图齐全的测试上下文（doc/element 恒定——断言可复现）。
    fn ctx() -> RandomEvalContext {
        RandomEvalContext {
            doc_seed: 7,
            element_seed: Some(42),
        }
    }

    fn sub(value: &str, property: &str) -> String {
        substitute_random_values(value, property, &ctx())
            .unwrap_or_else(|| "<None>".to_string().into())
            .into_owned()
    }

    #[test]
    fn no_random_call_borrows_original() {
        match substitute_random_values("10px solid red", "width", &ctx()) {
            Some(std::borrow::Cow::Borrowed(s)) => assert_eq!(s, "10px solid red"),
            other => panic!("expected borrowed, got {other:?}"),
        }
    }

    #[test]
    fn fixed_formula_exact_and_units() {
        assert_eq!(sub("random(fixed 0.5, 10, 100)", "width"), "55");
        assert_eq!(sub("random(fixed 0.5, 10px, 20px)", "width"), "15px");
        // 非二进制小数（0.1s）f64 连续公式带尾差——spec 数字串化最短往返，
        // 近似断言。
        let t: f64 = sub("random(fixed 0.5, 0.1s, 0.2s)", "transition-duration")
            .trim_end_matches('s')
            .parse()
            .expect("数值形");
        assert!((t - 0.15).abs() < 1e-12, "got {t}");
        // ms 归一 s（canonical_section 单位换算）。
        let t2: f64 = sub("random(fixed 0.5, 100ms, 200ms)", "transition-duration")
            .trim_end_matches('s')
            .parse()
            .expect("数值形");
        assert!((t2 - 0.15).abs() < 1e-12, "got {t2}");
        assert_eq!(sub("random(fixed 0.5, 0turn, 0.5turn)", "rotate"), "90deg");
    }

    #[test]
    fn step_grid_and_epsilon() {
        // 格点：span=10, s=3 → N=3；index = floor(0.5*4) = 2 → 6。
        assert_eq!(sub("random(fixed 0.5, 0, 10, 3)", "scale"), "6");
        // R 邻 1 取顶格：index = floor(0.999*4) = 3 → 9；max=10 距 9 超出
        // epsilon → 9（非修入 max）。
        assert_eq!(sub("random(fixed 0.999, 0, 10, 3)", "scale"), "9");
    }

    #[test]
    fn step_infinity_degenerates_to_min() {
        // 语料钉面 `random(10, 100, infinity)` → '10'。
        assert_eq!(sub("random(fixed 0.99, 10, 100, infinity)", "scale"), "10");
    }

    #[test]
    fn step_non_positive_ignored() {
        // step ≤ 0 忽略 → 连续公式。
        assert_eq!(sub("random(fixed 0.5, 10, 100, 0)", "scale"), "55");
        assert_eq!(sub("random(fixed 0.5, 10, 100, -3)", "scale"), "55");
    }

    #[test]
    fn reversed_args_clamp_to_min() {
        // max < min → max:=min（结果恰 min；钳制非交换）。
        assert_eq!(sub("random(fixed 0.5, 100, 10)", "scale"), "100");
    }

    #[test]
    fn nan_and_infinity_keywords() {
        // NaN 传染（消费面 censor）。
        assert_eq!(sub("random(fixed 0.5, NaN, 10)", "scale"), "calc(NaN)");
        // min 无穷 → 结果同号无穷。
        assert_eq!(sub("random(fixed 0.5, infinity, 10)", "scale"), "calc(infinity)");
        assert_eq!(sub("random(fixed 0.5, -infinity, 10)", "scale"), "calc(-infinity)");
        // (max-min) 无穷 → NaN。
        assert_eq!(sub("random(fixed 0.5, 10, infinity)", "scale"), "calc(NaN)");
        // keyword 首节（key 省略形 + keyword 数字节——语料 scale 槽
        // `random(NaN, 100)` / `random(infinity, 100)` 支持性钉面）。
        let a = sub("random(NaN, 100)", "scale");
        assert!(a != "<None>", "NaN 首节: {a}");
        let b = sub("random(infinity, 100)", "scale");
        assert!(b != "<None>", "infinity 首节: {b}");
    }

    #[test]
    fn nested_fixed_key_substitutes_cleanly() {
        // 语料钉面：`random(fixed random(1, 2), 10px, 100px)`（width/scale
        // supports 门 + computed 求值）——内层先替换，外层 fixed 数值求值。
        let out = sub("random(fixed random(1, 2), 10px, 100px)", "width");
        let v: f64 = out
            .trim_end_matches("px")
            .parse()
            .unwrap_or_else(|e| panic!("数值形: {e} — got {out}"));
        assert!((10.0..=100.0).contains(&v), "got {out}");
        // 负域 fixed 内嵌（translate 语料形）。
        let out2 = sub("random(fixed random(-2, -1), 10%, 100%)", "translate");
        assert!(out2.contains("random(fixed "), "translate % echo: {out2}");
    }

    #[test]
    fn percentage_leaf_fixed_echo() {
        // 无 key：播种 R 补显式 fixed。
        let out = sub("random(10%, 30%)", "width");
        let rest = out.strip_prefix("random(fixed ").expect("echo form");
        let (f, sections) = rest.split_once(", ").expect("fixed 分节");
        let r: f64 = f.parse().expect("R 数字形");
        assert!((0.0..1.0).contains(&r), "R 越界: {r}");
        assert_eq!(sections, "10%, 30%)");
        // 显式 fixed 原文回显。
        assert_eq!(
            sub("random(fixed 0.5, 10%, 30%)", "width"),
            "random(fixed 0.5, 10%, 30%)"
        );
    }

    #[test]
    fn element_scoped_fail_closed_without_view() {
        let noview = RandomEvalContext {
            doc_seed: 1,
            element_seed: None,
        };
        assert!(substitute_random_values("random(element-scoped, 10, 20)", "width", &noview).is_none());
        // key 省略 = auto = element-scoped（canonical 展开口径）→ 同需元素视图。
        assert!(substitute_random_values("random(10, 20)", "width", &noview).is_none());
        // property-scoped 与 fixed 不需要元素视图。
        assert!(substitute_random_values("random(property-scoped, 10, 20)", "width", &noview).is_some());
        assert!(substitute_random_values("random(fixed 0.5, 10, 20)", "width", &noview).is_some());
    }

    #[test]
    fn sentinel_resolves_element_scoped() {
        let out = substitute_random_values(
            "random(element-scoped, 10px, 20px)",
            "width",
            &RandomEvalContext::SENTINEL,
        )
        .expect("SENTINEL 出值");
        let v: f64 = out.trim_end_matches("px").parse().expect("数值形");
        assert!((10.0..20.0).contains(&v));
    }

    #[test]
    fn string_literal_call_not_substituted() {
        let out = sub(r#"random(1, 2) "random(3, 4)""#, "width");
        assert!(out.contains("\"random(3, 4)\""), "字符串字面保原文: {out}");
        assert!(!out.starts_with('"'));
    }

    #[test]
    fn nested_random_in_fixed_key() {
        // 内层先替换 → fixed 数值 → 外层公式。
        let out = sub("random(fixed random(0.2, 0.4), 0, 10)", "scale");
        let v: f64 = out.parse().expect("数值形");
        assert!((0.0..=10.0).contains(&v));
    }

    #[test]
    fn item_fixed_selection_matrix() {
        assert_eq!(sub("random-item(fixed 0.0, red, green, blue)", "color"), "red");
        assert_eq!(sub("random-item(fixed 0.5, red, green, blue)", "color"), "green");
        assert_eq!(sub("random-item(fixed 0.999999, red, green, blue)", "color"), "blue");
    }

    #[test]
    fn item_braced_comma_preserved() {
        // 花括号包裹项整体选中、剥壳保内逗号。
        assert_eq!(sub("random-item(fixed 0.5, red, {a, b}, blue)", "font-family"), "a, b");
    }

    #[test]
    fn item_omitted_key_seeded() {
        let a = sub("random-item(10px, 20px, 30px)", "width");
        assert!(["10px", "20px", "30px"].contains(&a.as_str()));
    }

    #[test]
    fn item_var_arg_fail_closed() {
        assert!(substitute_random_values("random-item(fixed 0.5, var(--x), red)", "color", &ctx()).is_none());
    }

    #[test]
    fn item_empty_and_gap_invalid() {
        assert!(substitute_random_values("random-item(fixed 0.5, red, , blue)", "color", &ctx()).is_none());
        assert!(substitute_random_values("random-item(fixed 0.5)", "color", &ctx()).is_none());
    }

    #[test]
    fn item_uppercase_and_keyword_key() {
        // 函数名大小写不敏感（作者面）；fixed 0.5 × 2 项 → floor(1.0) = 1。
        assert_eq!(sub("RANDOM-ITEM(fixed 0.5, red, green)", "color"), "green");
        assert_eq!(sub("Random-Item(fixed 0.25, red, green)", "color"), "red");
        // `normal` 非 `<random-key>` 文法成分——按 item 处理（spec 文法序
        // key 匹配失败落 item 形）。
        let out = sub("random-item(normal, red, green)", "color");
        assert!(["normal", "red", "green"].contains(&out.as_str()), "got {out}");
    }

    #[test]
    fn ordinal_separated_calls_differ() {
        // 同属性两调用不同序 → 独立 seed（fixed 形此处不验——非 fixed 形
        // 异序独立由 canonical_key_text index 保证，值域断言）。
        let out = sub("random(10, 20) random(10, 20)", "width");
        let two: Vec<f64> = out
            .split(' ')
            .map(|s| s.trim_end_matches("px").parse().unwrap())
            .collect();
        assert_eq!(two.len(), 2);
        assert!(two.iter().all(|v| (10.0..20.0).contains(v)));
    }
}
