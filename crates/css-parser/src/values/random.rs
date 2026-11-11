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
//! 本模块只裁文法（写面门消费）；求值（sha256 种子 + element/property 上下文）
//! 挂账 engine 面——写面 valid 形放行 store 原文，`CSS.supports` 维持拒面。
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

/// 数字 token 解析：`[+-]?<digits>[.<digits>]?([eE]<digits>)?<unit>?`——按
/// CSS number 词法最长匹配数值前缀，余段为单位。返回族；未知单位/无整数位/
/// 尾随点返 None。
fn parse_number_token(tok: &str) -> Option<RandomFamily> {
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
    let unit = &tok[i..];
    if unit.is_empty() {
        return Some(RandomFamily::Number);
    }
    let fam = match unit.to_ascii_lowercase().as_str() {
        "px" | "em" | "rem" | "ex" | "ch" | "rex" | "rch" | "ic" | "ric" | "cap" | "rcap" | "lh" | "rlh" | "vh"
        | "vw" | "vmin" | "vmax" | "svh" | "lvh" | "dvh" | "svw" | "lvw" | "dvw" | "q" | "cm" | "mm" | "in" | "pt"
        | "pc" | "%" => RandomFamily::Length,
        "s" | "ms" => RandomFamily::Time,
        "deg" | "rad" | "grad" | "turn" => RandomFamily::Angle,
        _ => return None,
    };
    Some(fam)
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

/// identification 节（首节非单数字形）token 裁决。成分（语料反推：
/// serialize 页 71 接受形 / invalid 页 39 拒形）：
/// - `fixed <number [0,1)>` 独占节（恰两 token）；
/// - `auto` 单 token 独占节（WebKit shorthand，serialize 页 `random(auto, ...)` 行）；
/// - `--<dashed-ident>`（至多一、必须在任何 scoping 之前）× element 域
///   `element-scoped`（至多一）× UA 域 `property-scoped|property-index-scoped|
///   ua-<ident>`（至多一）的组合——element × UA 域任意序可共存（serialize 页
///   `element-scoped property-scoped` / `property-index-scoped element-scoped`
///   行），双 UA 域拒（invalid 页 `property-scoped ua-width-1` /
///   `property-index-scoped --foo property-scoped` 形）。
///
/// 返回 (has_fixed, 是否含 identification)。
fn parse_identification_tokens(tokens: &[&str]) -> Option<(bool, bool)> {
    match tokens.first().copied()? {
        "fixed" => {
            if tokens.len() != 2 {
                return None; // `fixed` 独占节：恰两 token（`fixed 0.5 element-shared` 拒）
            }
            let fam = parse_number_token(tokens[1])?;
            if fam != RandomFamily::Number {
                return None; // `fixed 0.5px` 非 `<number>`
            }
            let n: f64 = tokens[1].parse().ok()?;
            if !(0.0..1.0).contains(&n) {
                return None; // `fixed -1` / `fixed 1` 出 [0,1) 范围
            }
            Some((true, true))
        }
        "auto" if tokens.len() == 1 => Some((false, true)),
        _ => {
            let mut ident_seen = false;
            let mut element = 0usize;
            let mut ua_domain = 0usize;
            for &tok in tokens {
                if tok.starts_with("--") {
                    // ident 至多一且必须最前（invalid 页
                    // `element-scoped --foo --bar` / serialize 页
                    // `--foo element-scoped property-scoped` 口径）。
                    if ident_seen || element > 0 || ua_domain > 0 {
                        return None;
                    }
                    ident_seen = true;
                } else if is_element_scoped(tok) {
                    element += 1;
                } else if is_ua_domain_scoping(tok) {
                    ua_domain += 1;
                } else {
                    return None; // 裸 ident/`uawidth`/字符串节等未知 token
                }
            }
            if element > 1 || ua_domain > 1 {
                return None; // 同关键字重复 / 双 UA 域
            }
            Some((false, true))
        }
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
/// invalid 形整拒（random-invalid 语料 39 形），valid 形放行 store 原文
/// （random-serialize 页 29 接受面零漂移）。
pub fn parse_random_args(args: &str) -> Option<RandomSpec> {
    let sections = split_sections(args);
    if !(2..=4).contains(&sections.len()) {
        return None;
    }
    // 首节形态：单 token 数字 → 无 identification；其余 → identification 节。
    let first_toks: Vec<&str> = sections[0].split_whitespace().collect();
    let ident_first = match first_toks.as_slice() {
        [t] => parse_number_token(t).is_none(),
        _ => true,
    };
    let mut has_fixed = false;
    if ident_first {
        let (fixed, _) = parse_identification_tokens(&first_toks)?;
        has_fixed = fixed;
    }
    if sections.len() > 3 && !ident_first {
        return None; // 无 identification → min,max[,step] ≤3 节
    }
    // 数字节：identification 形自第 2 节起；纯数字形首节即数字节（族已并入
    // 下循环——重新解析首节保统一路径）。
    let mut family: Option<RandomFamily> = None;
    for sec in sections.iter().skip(if ident_first { 1 } else { 0 }) {
        let mut parts = sec.split_whitespace();
        let tok = parts.next()?;
        if parts.next().is_some() {
            return None; // `1px foo` 双 token 节
        }
        let fam = parse_number_token(tok)?;
        match family {
            None => family = Some(fam),
            Some(f) if f == fam => {}
            Some(_) => return None, // 跨族混（§10.5 一致门）
        }
    }
    Some(RandomSpec {
        family: family?,
        has_fixed,
    })
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
