//! CSSOM anchor 族声明值文法校验与序列化（css-anchor-position-1，wptfyi-css 组 F 批）。
//!
//! 与 [`super::grid_cssom`]/[`super::position_area`] 同架构（单实现双通道）：
//! - **CSSOM declared 面**（[`canonicalize_decl`]）：shim 写通道经
//!   `__zw_anchor_normalize_decl` 回调、`CSS.supports` 经
//!   [`crate::js_dom_bridge`] `decl_supported` 分派——非法值丢弃声明 / 判 false，
//!   合法值回写 canonical 串。
//! - **computed 面**（[`computed_value`]）：gCS plain 臂经 `__zw_anchor_computed`
//!   回调读 declared 值折 computed（族内属性 computed = as-specified 恒等折叠，
//!   css-wide 语义由 shim 侧处理，本函数只供空声明初始值）。
//! - **简写拆分**（[`split_position_try`]/[`join_position_try`]）：`position-try`
//!   简写 ↔ `position-try-order` + `position-try-fallbacks` 长手对。
//! - **anchor()/anchor-size() 函数校验**（[`anchor_funcs_valid`]）：通用 inset/
//!   size/margin 属性域值含 anchor 调用时的属性域 + 参数文法拒面（保形放行，
//!   不重写值——valid 语料 round-trip 以原文 echo 钉）。
//!
//! 序列化矩阵全部以语料为 oracle（css-anchor-position/parsing 三页 + position-anchor-
//! basics + anchor-parse-invalid/valid 等页），非 spec 字面推演：Blink 简写解析对
//! `most-height, flip-start`（order 独占首段后跟逗号段）判非法、fallback 段内
//! ident/try-tactic 与 position-area 不得同段、`none` 只能整体单独。
//!
//! https://drafts.csswg.org/css-anchor-position-1/

/// anchor 族 CSSOM 文法面属性集（七属性；position-area 另有专属模块
/// [`super::position_area`] 不在本集）。
pub(crate) const ANCHOR_FAMILY_PROPS: [&str; 7] = [
    "position-anchor",
    "position-visibility",
    "position-try-order",
    "position-try-fallbacks",
    "position-try",
    "anchor-name",
    "anchor-scope",
];

pub(crate) fn is_anchor_family_prop(prop: &str) -> bool {
    ANCHOR_FAMILY_PROPS.contains(&prop)
}

/// `<dashed-ident>` 判定：`--` 前缀（转义形保原文——wire 面 tokenizer 解码先例
/// 由 serialize_css_ident 承担，本面按字面 `--` 起头接受）。
fn is_dashed_ident(tok: &str) -> bool {
    tok.starts_with("--")
}

/// try-tactic 关键字集（css-anchor-position-1 §try-tactic：flip-block/flip-inline/
/// flip-start 原三键 + flip-x/flip-y 别名——语料 valid 段全键钉面）。
fn is_try_tactic(tok: &str) -> bool {
    matches!(tok, "flip-block" | "flip-inline" | "flip-start" | "flip-x" | "flip-y")
}

/// position-try-order 关键字集。
fn is_try_order_kw(tok: &str) -> bool {
    matches!(
        tok,
        "normal" | "most-width" | "most-height" | "most-block-size" | "most-inline-size"
    )
}

/// 顶层逗号分段（嵌套函数/引号内逗号不分段；括号不平衡返 `None`——调用方判拒）。
fn split_top_commas(v: &str) -> Option<Vec<&str>> {
    let mut segs = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    let bytes = v.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            b',' if depth == 0 => {
                segs.push(&v[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        return None;
    }
    segs.push(&v[start..]);
    Some(segs)
}

/// CSS-wide keyword 归小写 echo（与 position_area 同款门）。
fn css_wide(lower: &str) -> Option<String> {
    if ["inherit", "initial", "unset", "revert", "revert-layer"].contains(&lower) {
        return Some(lower.to_string());
    }
    None
}

/// 值是否含 var() 引用（css-variables-1 parse 期恒有效保形——全族同款门）。
fn has_var(v: &str) -> bool {
    v.contains("var(")
}

/// anchor 族 declared 面：canonical 串；非法值返 `None`（调用方丢弃声明）。
///
/// 空串 = 移除声明（CSSOM 空值语义）放行 `Some("")`；CSS-wide keyword ASCII ci
/// 归小写 echo；var() 值 parse 期恒有效原文保形（position_area/grid_cssom 同款）。
/// ident() 函数形（css-values-5 `ident()`）parse 期接受保形——求值面挂账批。
pub(crate) fn canonicalize_decl(prop: &str, value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return Some(String::new());
    }
    let lower = v.to_ascii_lowercase();
    if let Some(kw) = css_wide(&lower) {
        return Some(kw);
    }
    if has_var(v) {
        return Some(v.to_string());
    }
    match prop {
        "position-anchor" => canonicalize_anchor_kw(&lower),
        "anchor-name" | "anchor-scope" => canonicalize_dashed_list(prop, v, &lower),
        "position-visibility" => canonicalize_visibility(&lower),
        "position-try-order" => {
            if is_try_order_kw(&lower) {
                Some(lower)
            } else {
                None
            }
        }
        "position-try-fallbacks" => canonicalize_fallbacks(v, &lower),
        "position-try" => canonicalize_shorthand(v),
        _ => None,
    }
}

/// position-anchor 文法：`normal | none | auto | <dashed-ident>`（单 token，
/// position-anchor-basics 语料钉面：foo-bar/多值/维度拒，normal/none/auto 收）。
fn canonicalize_anchor_kw(lower: &str) -> Option<String> {
    if lower.split_whitespace().count() != 1 {
        return None;
    }
    if matches!(lower, "normal" | "none" | "auto") || is_dashed_ident(lower) {
        return Some(lower.to_string());
    }
    None
}

/// anchor-name / anchor-scope 文法：`none`（scope 另有 `all`）或 `<dashed-ident>#`
/// 逗号列表（保输入序——anchor-scope-parsing `--bar, --foo` 保序钉面）；关键字
/// 不与 dashed 段混合、非 dashed 段拒（anchor-scope-parsing invalid 矩阵）。
fn canonicalize_dashed_list(prop: &str, v: &str, lower: &str) -> Option<String> {
    if prop == "anchor-scope" && matches!(lower, "none" | "all") {
        return Some(lower.to_string());
    }
    if prop == "anchor-name" && lower == "none" {
        return Some(lower.to_string());
    }
    // ident() 函数形（css-values-5 §ident）：parse 期接受保形（求值挂账批——
    // anchor-ident-function 语料 computed 面消费）。混合列表（`--tl, ident(...)`)
    // 同保形（语料 valid 案形状）。
    if lower.contains("ident(") {
        return Some(v.to_string());
    }
    let segs = split_top_commas(v)?;
    let mut out = Vec::new();
    for seg in segs {
        // 每段恰一 token（`--a none` 整段以 `--` 起头但为双 token——按 token 判，
        // 不按段首缀）。
        let mut toks = seg.split_whitespace();
        let one = toks.next();
        if toks.next().is_some() {
            return None;
        }
        match one {
            Some(tok) if is_dashed_ident(tok) => out.push(tok.to_ascii_lowercase()),
            _ => return None,
        }
    }
    Some(out.join(", "))
}

/// position-visibility 文法：`always` 单独，或 `[anchors-valid && anchors-visible &&
/// no-overflow]` 三键可选子集（至少一、无重复、canonical 固定序——语料 valid 矩阵
/// 全排列 + invalid 重复/always 混合钉面）。
fn canonicalize_visibility(lower: &str) -> Option<String> {
    let toks: Vec<&str> = lower.split_whitespace().collect();
    if toks.len() == 1 && toks[0] == "always" {
        return Some(toks[0].to_string());
    }
    let mut has_valid = false;
    let mut has_visible = false;
    let mut has_overflow = false;
    for tok in &toks {
        match *tok {
            "anchors-valid" if !has_valid => has_valid = true,
            "anchors-visible" if !has_visible => has_visible = true,
            "no-overflow" if !has_overflow => has_overflow = true,
            _ => return None,
        }
    }
    if !(has_valid || has_visible || has_overflow) {
        return None;
    }
    let mut parts = Vec::new();
    if has_valid {
        parts.push("anchors-valid");
    }
    if has_visible {
        parts.push("anchors-visible");
    }
    if has_overflow {
        parts.push("no-overflow");
    }
    Some(parts.join(" "))
}

/// position-try-fallbacks 文法：`none` 单独，或 `<fallback-item>#` 逗号列表。
///
/// item 三形（语料矩阵）：
/// - `<dashed-ident> [<try-tactic>]*`：ident 与 tactic 任意混排同段，canonical =
///   ident 最前 + tactic 输入序（段内 tactic 无重复——`flip-inline flip-inline` 拒）；
/// - 纯 `<try-tactic>*`（至少一，无重复）；
/// - `<position-area>`：整段委托 [`super::position_area::canonicalize_decl`]
///   （`top left`→`left top`、`start start`→`start` 折叠复用同源实现），独占段
///   （与 ident/tactic 同段拒——`--foo left`/`left flip-start` invalid 钉面）。
///
/// `none` 只能整体单独（`none, X` / `X, none` 拒）；段间保输入序。
fn canonicalize_fallbacks(v: &str, _lower: &str) -> Option<String> {
    if v.trim().eq_ignore_ascii_case("none") {
        return Some("none".to_string());
    }
    let segs = split_top_commas(v)?;
    let mut out = Vec::new();
    for seg in segs {
        out.push(canonicalize_fallback_seg(seg)?);
    }
    if out.is_empty() {
        return None;
    }
    Some(out.join(", "))
}

/// 单段 fallback 文法（输入原文段；返 canonical 段串）。
fn canonicalize_fallback_seg(seg: &str) -> Option<String> {
    let s = seg.trim();
    if s.is_empty() {
        return None;
    }
    let s_lower = s.to_ascii_lowercase();
    let toks: Vec<&str> = s_lower.split_whitespace().collect();
    let mut ident: Option<&str> = None;
    let mut tactics: Vec<&str> = Vec::new();
    let mut tactics_at_ident = 0usize;
    let mut area_toks: Vec<&str> = Vec::new();
    for tok in &toks {
        if is_dashed_ident(tok) {
            if ident.is_some() {
                return None; // 双 ident 拒（`--bar flip-block --foo` 形）
            }
            ident = Some(tok);
            tactics_at_ident = tactics.len();
        } else if is_try_tactic(tok) {
            if tactics.contains(tok) {
                return None; // 段内 tactic 重复拒（`flip-block flip-block` 等）
            }
            tactics.push(tok);
        } else if *tok != "none" && super::position_area::is_area_kw(tok) {
            // `none` 非 area 段 keyword（fallbacks 里 `none` 只能整体单独——
            // 顶层早臂已收，段内出现即拒）。
            area_toks.push(tok);
        } else {
            return None; // 非法 token（foo/-foo/123/order 关键字混段等）
        }
    }
    if let Some(name) = ident {
        // 语料 T I T 拒面（`flip-inline --bar flip-block`）：ident 夹在 tactic
        // 中间不得混排；`IT*`（--bar flip-block）/`T*I`（flip-start --foo）两种
        // 邻接形均合法。
        if tactics_at_ident > 0 && tactics.len() > tactics_at_ident {
            return None;
        }
        let _ = name;
    }
    // position-area token 与 ident/tactic 不得同段（语料 invalid 矩阵）。
    if !area_toks.is_empty() {
        if ident.is_some() || !tactics.is_empty() {
            return None;
        }
        return super::position_area::canonicalize_decl(&area_toks.join(" ")).filter(|r| !r.is_empty());
    }
    if ident.is_none() && tactics.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if let Some(name) = ident {
        parts.push(name.to_string());
    }
    parts.extend(tactics.iter().map(|t| t.to_string()));
    Some(parts.join(" "))
}

/// position-try 简写 canonical：拆分后按 join 规则合成（`normal none`→`none`、
/// `normal --foo`→`--foo`、`most-width none` 保显式——语料钉面）。
fn canonicalize_shorthand(v: &str) -> Option<String> {
    let (order, fb) = split_position_try(v)?;
    Some(join_position_try(&order, &fb))
}

/// `position-try` 简写拆分：`(order, fallbacks)` canonical 对。
///
/// order 关键字若出现必须位于**首段开头**（段内剩余 = fallbacks 首段）且**不得
/// 独占首段后还跟逗号段**（`most-height, flip-start` invalid——Blink 简写解析
/// order 与 fallbacks 之间无逗号分隔位）；fallbacks 段文法见
/// [`canonicalize_fallbacks`]。order 缺省返 "normal"；fallbacks 缺省返 "none"。
pub(crate) fn split_position_try(v: &str) -> Option<(String, String)> {
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if let Some(kw) = css_wide(&lower) {
        // 简写 css-wide 语义：长手对同 keyword（CSS Cascading §7.1-§7.4）。
        return Some((kw.clone(), kw));
    }
    if has_var(v) {
        // var() 形 parse 期恒有效：order 缺省 + fallbacks 原文保形。
        return Some(("normal".to_string(), v.to_string()));
    }
    let segs = split_top_commas(v)?;
    let first = segs[0].trim();
    let first_toks: Vec<&str> = first.split_whitespace().collect();
    let order: String;
    let fb_first: String;
    let has_fb_rest: bool;
    match first_toks.first() {
        Some(&head) if is_try_order_kw(&head.to_ascii_lowercase()) => {
            if first_toks.len() == 1 && segs.len() > 1 {
                // order 独占首段 + 后续逗号段：invalid（语料 most-height, flip-start）。
                return None;
            }
            order = head.to_ascii_lowercase();
            fb_first = first_toks[1..].join(" ");
            has_fb_rest = segs.len() > 1;
        }
        Some(_) => {
            order = "normal".to_string();
            fb_first = first.to_string();
            has_fb_rest = segs.len() > 1;
        }
        None => return None,
    }
    let fb = if !has_fb_rest && fb_first.is_empty() {
        // 仅 order 单 token：fallbacks 缺省 none。
        "none".to_string()
    } else if !has_fb_rest && fb_first == "none" {
        "none".to_string()
    } else {
        let mut joined = fb_first.clone();
        for seg in &segs[1..] {
            joined.push(',');
            joined.push_str(seg);
        }
        let canon = canonicalize_fallbacks(joined.trim(), "")?;
        if canon == "none" {
            return None;
        }
        canon
    };
    Some((order, fb))
}

/// `position-try` 简写合成：长手对 → 简写 canonical 串（order=normal 省略、
/// 双初始折叠 `none`）。
pub(crate) fn join_position_try(order: &str, fallbacks: &str) -> String {
    let o = order.trim();
    let f = fallbacks.trim();
    let o_norm = o == "normal";
    let f_none = f == "none";
    match (o_norm, f_none) {
        (true, true) => "none".to_string(),
        (true, false) => f.to_string(),
        (false, true) => format!("{o} none"),
        (false, false) => format!("{o} {f}"),
    }
}

/// anchor()/anchor-size() 函数值校验（通用 inset/size/margin 属性域拒面）。
///
/// 值含 anchor 调用时逐调用校验（平面扫描，嵌套 calc/min 树内调用同面）：属性域
/// （anchor() → inset 族；anchor-size() → width/inset/margin 族——anchor-size-parse-
/// valid 语料全集）+ 参数文法（段数/name dashed/side·size 关键字集/fallback 形）。
/// **只拒不重写**：合法值原文放行（valid 语料 round-trip 原文 echo 钉面，
/// `min(50%, 100%)` 保形）。括号不平衡（`anchor(--foo top, auto`）拒。shim 写面在
/// var() 保形检查**之后**调用（含 var 值 parse 期不校验——替换后校验语义挂账）。
pub(crate) fn anchor_funcs_valid(prop: &str, value: &str) -> bool {
    let v = value.trim();
    if v.is_empty() {
        return true;
    }
    let lower = v.to_ascii_lowercase();
    // 前置短路：无 anchor 调用字面的值零成本放行（通用写面高频路径）。
    // `anchor-size(` 不含 `anchor(` 子串（anchor 与 `(` 间有 `-`），双形并查。
    if !lower.contains("anchor(") && !lower.contains("anchor-size(") {
        return true;
    }
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let is_size = if lower[i..].starts_with("anchor-size(") {
            true
        } else if lower[i..].starts_with("anchor(") {
            false
        } else {
            i += 1;
            continue;
        };
        // 词边界：前字符是 ident 字节（`xanchor(`/`-anchor(` 属更长 ident）不触发。
        if i > 0 && is_ident_byte(bytes[i - 1]) {
            i += 1;
            continue;
        }
        let open = i + if is_size { "anchor-size(".len() } else { "anchor(".len() } - 1;
        // 空参形：`anchor-size()` 隐式形（语料 anchor-size-parse-valid implicit
        // 段钉面）——`(` 为末字节时无 `)` 可寻，balanced_seg 会误判不平衡。
        let seg = if open + 1 == v.len() {
            Some("")
        } else {
            balanced_seg(v, open)
        };
        match seg {
            Some(seg) => {
                if !anchor_call_args_valid(seg, is_size, prop) {
                    return false;
                }
            }
            None => return false,
        }
        // 逐字符推进：嵌套调用（fallback/calc 树内）同样平面校验。
        i += 1;
    }
    true
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// 取 `(` 起的平衡段（返括号内内容，不含括号本身）；不平衡返 `None`。
fn balanced_seg(v: &str, open: usize) -> Option<&str> {
    let bytes = v.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut i = open;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                in_str = None;
            }
        } else {
            match b {
                b'"' | b'\'' => in_str = Some(b),
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&v[open + 1..i]);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// 单个 anchor()/anchor-size() 调用参数校验（嵌套调用在此放行——由平面主扫描
/// 独立校验其参数）。
fn anchor_call_args_valid(seg: &str, is_size: bool, prop: &str) -> bool {
    // 属性域（按函数类型；语料 anchor-parse-valid insetProperties +
    // anchor-size-parse-valid size/inset/margin 全集）。
    if is_size {
        if !anchor_size_domain(prop) {
            return false;
        }
    } else if !anchor_inset_domain(prop) {
        return false;
    }
    let Some(segs) = split_top_commas(seg) else {
        return false;
    };
    if segs.len() > 2 {
        return false;
    }
    // 段1：`[<dashed-ident> || <side|size|percentage|math>]`——`||` 两侧序均可
    //（语料 flip 钉面：`anchor(<side> <name>)` / `anchor-size(<size> <name>)`
    // 同法合法，canonical 重写为 named 形——本面只拒不重写，重写挂账序列化批）；
    // 空参仅 anchor-size() 隐式形合法（anchor-size-parse-valid implicit 段；
    // `anchor()` 空参语料未收、按拒）。anchor-size 隐式单参收 length/percentage
    //（`anchor-size(10px)` 语料钉面）；anchor() 单参收 side/%/math。
    let first = segs[0].trim();
    let toks = split_top_tokens(first);
    // dashed 名后槽（2-token 形第二槽）：anchor() 收 side/%/math；anchor-size()
    // 仅 size 关键字（语料 `anchor-size(--foo 10em)`/`--foo 50%` 拒面——length
    // 非 size 槽位值）。
    let pair_second_ok = |t: &str| {
        if is_size {
            is_anchor_size_kw(t)
        } else {
            is_anchor_side_kw(t) || is_percentage(t) || is_math_func(t)
        }
    };
    // 单 token 形：dashed name-only，或隐式值形——anchor-size 隐式额外收
    // length/percentage/math（`anchor-size(10px)` 语料钉面）。
    let single_ok = |t: &str| {
        is_dashed_ident(t) || pair_second_ok(t) || (is_size && (is_length_or_percentage(t) || is_math_func(t)))
    };
    match toks.len() {
        0 => return is_size && segs.len() == 1, // `anchor-size()` / `anchor(, 10px)` 空段
        1 => {
            if !single_ok(toks[0]) {
                return false;
            }
        }
        2 => {
            let (a, b) = (toks[0], toks[1]);
            if !(is_dashed_ident(a) && pair_second_ok(b)) && !(pair_second_ok(a) && is_dashed_ident(b)) {
                return false;
            }
        }
        _ => return false, // math 形 `2 * 20%` 等多 token 拒
    }
    // 段2（fallback）：单 top token——length/percentage/math 形/嵌套 anchor 调用
    // （嵌套调用的参数校验由平面主扫描承担——嵌套名非 dashed 在其自身调用点拒）。
    if segs.len() == 2 {
        let fb = segs[1].trim();
        let ftoks = split_top_tokens(fb);
        if ftoks.len() != 1 {
            return false;
        }
        let tl = ftoks[0].to_ascii_lowercase();
        let is_nested_anchor = tl.starts_with("anchor(") || tl.starts_with("anchor-size(");
        if !is_nested_anchor && !(is_length_or_percentage(&tl) || is_math_func(&tl)) {
            return false;
        }
    }
    true
}

/// 顶层 token 分词（空白切分，括号平衡函数形聚合单 token；不平衡括号段聚合为
/// 单 token——调用方按关键字/函数形判定拒绝，`anchor((2 * 20%))` 形）。
fn split_top_tokens(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                if start.is_none() {
                    start = Some(idx);
                }
            }
            ')' => {
                depth -= 1;
            }
            c if c.is_whitespace() && depth <= 0 => {
                if let Some(st) = start.take() {
                    out.push(&s[st..idx]);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(idx);
                }
            }
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out.retain(|t| !t.is_empty());
    out
}

fn is_anchor_side_kw(t: &str) -> bool {
    matches!(
        t.to_ascii_lowercase().as_str(),
        "inside"
            | "outside"
            | "left"
            | "right"
            | "top"
            | "bottom"
            | "start"
            | "end"
            | "self-start"
            | "self-end"
            | "center"
    )
}

fn is_anchor_size_kw(t: &str) -> bool {
    matches!(
        t.to_ascii_lowercase().as_str(),
        "width" | "height" | "block" | "inline" | "self-block" | "self-inline"
    )
}

fn is_percentage(t: &str) -> bool {
    let Some(body) = t.strip_suffix('%') else {
        return false;
    };
    parse_number_body(body).is_some()
}

/// `<length-percentage>` token 判定：数字体 + 长度单位白名单 / percentage /
/// unitless zero（`anchor(--foo left, 0)` 语料钉面；非零裸数字仍拒——
/// `anchor(--foo top, 1)`/`100s` invalid 钉面）。
fn is_length_or_percentage(t: &str) -> bool {
    if parse_number_body(t) == Some(0.0) {
        return true;
    }
    if is_percentage(t) {
        return true;
    }
    const LEN_UNITS: [&str; 45] = [
        "px", "pt", "pc", "in", "cm", "mm", "q", "em", "rem", "ex", "ch", "cap", "ic", "lh", "rlh", "vw", "vh", "vmin",
        "vmax", "vi", "vb", "svw", "svh", "svmin", "svmax", "svi", "svb", "lvw", "lvh", "lvmin", "lvmax", "lvi", "lvb",
        "dvw", "dvh", "dvmin", "dvmax", "dvi", "dvb", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ];
    let tl = t.to_ascii_lowercase();
    for u in LEN_UNITS {
        if let Some(body) = tl.strip_suffix(u)
            && parse_number_body(body).is_some()
        {
            return true;
        }
    }
    false
}

/// 数字体判定（`[+-]?(\d+(\.\d+)?|\.\d+)`——`-foo` 形拒）。
fn parse_number_body(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let mut chars = t.chars().peekable();
    if matches!(chars.peek(), Some('+') | Some('-')) {
        chars.next();
    }
    let mut seen_digit = false;
    let mut seen_dot = false;
    for c in chars {
        match c {
            '0'..='9' => seen_digit = true,
            '.' if !seen_dot => seen_dot = true,
            _ => return None,
        }
    }
    if seen_digit { t.parse::<f64>().ok() } else { None }
}

/// 数学函数形判定（平衡闭合由 balanced_seg 前提保证——此处只认函数头）。
fn is_math_func(t: &str) -> bool {
    let tl = t.to_ascii_lowercase();
    tl.starts_with("calc(") || tl.starts_with("min(") || tl.starts_with("max(") || tl.starts_with("clamp(")
}

/// anchor() 属性域：inset 族（物理四边 + 简写 + 逻辑六边——anchor-parse-valid
/// insetProperties + inset/inset-block/inset-inline 简写同族纳入）。
fn anchor_inset_domain(prop: &str) -> bool {
    matches!(
        prop,
        "left"
            | "right"
            | "top"
            | "bottom"
            | "inset"
            | "inset-block"
            | "inset-inline"
            | "inset-block-start"
            | "inset-block-end"
            | "inset-inline-start"
            | "inset-inline-end"
    )
}

/// anchor-size() 属性域：width/height 全族 + inset 族 + margin 族
///（anchor-size-parse-valid 语料三集并）。
fn anchor_size_domain(prop: &str) -> bool {
    matches!(
        prop,
        "width"
            | "height"
            | "min-width"
            | "max-width"
            | "min-height"
            | "max-height"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
            | "inline-size"
            | "min-inline-size"
            | "max-inline-size"
    ) || anchor_inset_domain(prop)
        || matches!(
            prop,
            "margin"
                | "margin-left"
                | "margin-right"
                | "margin-top"
                | "margin-bottom"
                | "margin-block"
                | "margin-inline"
                | "margin-block-start"
                | "margin-block-end"
                | "margin-inline-start"
                | "margin-inline-end"
        )
}

/// anchor 族 computed 面：族内属性 computed = as-specified（css-anchor-position-1
/// 各属性 Computed value 行）——declared canonical 串恒等 echo；空声明返初始值。
/// css-wide 语义（initial/unset/revert-layer → 初始值；inherit → 父链）由 shim
/// plain 臂处理，本函数只供空声明初始值与恒等折叠。
pub(crate) fn computed_value(prop: &str, decl: &str) -> String {
    match prop {
        // position-visibility 初始 anchors-visible（computed 语料 initial 钉面）。
        "position-visibility" if decl.is_empty() => "anchors-visible".to_string(),
        "position-anchor" if decl.is_empty() => "normal".to_string(),
        "position-try-order" if decl.is_empty() => "normal".to_string(),
        "position-try-fallbacks" | "anchor-name" | "anchor-scope" if decl.is_empty() => "none".to_string(),
        _ => decl.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_anchor() {
        for v in ["normal", "none", "auto", "--foo"] {
            assert_eq!(canonicalize_decl("position-anchor", v).as_deref(), Some(v), "{v}");
        }
        for v in ["foo-bar", "--foo --bar", "--foo, --bar", "100px", "100%"] {
            assert_eq!(canonicalize_decl("position-anchor", v), None, "invalid {v}");
        }
        assert_eq!(canonicalize_decl("position-anchor", ""), Some(String::new()));
        assert_eq!(
            canonicalize_decl("position-anchor", "INHERIT").as_deref(),
            Some("inherit")
        );
        assert_eq!(
            canonicalize_decl("position-anchor", "var(--a)").as_deref(),
            Some("var(--a)")
        );
    }

    #[test]
    fn test_anchor_name_scope() {
        for v in ["none", "--a", "--a, --b", "--a, --b, --c", "--bar, --foo"] {
            assert_eq!(canonicalize_decl("anchor-scope", v).as_deref(), Some(v), "{v}");
        }
        assert_eq!(canonicalize_decl("anchor-scope", "all").as_deref(), Some("all"));
        assert_eq!(canonicalize_decl("anchor-name", "none").as_deref(), Some("none"));
        for v in ["--a none", "none --a", "none all", "--a --b", "a, b, c"] {
            assert_eq!(canonicalize_decl("anchor-scope", v), None, "invalid {v}");
        }
        assert_eq!(canonicalize_decl("anchor-name", "--a none"), None);
    }

    #[test]
    fn test_position_visibility() {
        for v in [
            "always",
            "anchors-valid",
            "anchors-visible",
            "no-overflow",
            "anchors-valid anchors-visible",
            "anchors-valid no-overflow",
            "anchors-visible no-overflow",
        ] {
            assert_eq!(canonicalize_decl("position-visibility", v).as_deref(), Some(v), "{v}");
        }
        // canonical 固定序重排（语料六全排列）。
        assert_eq!(
            canonicalize_decl("position-visibility", "anchors-visible anchors-valid").as_deref(),
            Some("anchors-valid anchors-visible")
        );
        assert_eq!(
            canonicalize_decl("position-visibility", "no-overflow anchors-valid").as_deref(),
            Some("anchors-valid no-overflow")
        );
        assert_eq!(
            canonicalize_decl("position-visibility", "no-overflow anchors-visible anchors-valid").as_deref(),
            Some("anchors-valid anchors-visible no-overflow")
        );
        for v in [
            "foobar",
            "always foobar",
            "always anchors-valid",
            "always anchors-visible no-overflow",
            "no-overflow foobar",
            "anchors-valid no-overflow anchors-valid anchors-visible",
        ] {
            assert_eq!(canonicalize_decl("position-visibility", v), None, "invalid {v}");
        }
    }

    #[test]
    fn test_try_order_and_fallbacks() {
        for v in [
            "normal",
            "most-width",
            "most-height",
            "most-block-size",
            "most-inline-size",
        ] {
            assert_eq!(canonicalize_decl("position-try-order", v).as_deref(), Some(v), "{v}");
        }
        for v in [
            "normal most-inline-size",
            "most-block-size most-inline-size",
            "most-block-size, most-inline-size",
        ] {
            assert_eq!(canonicalize_decl("position-try-order", v), None, "invalid {v}");
        }
        // fallbacks valid 矩阵。
        for (input, expect) in [
            ("none", "none"),
            ("flip-block", "flip-block"),
            ("flip-block ", "flip-block"),
            ("flip-start, flip-block", "flip-start, flip-block"),
            (
                "flip-start flip-inline, flip-block",
                "flip-start flip-inline, flip-block",
            ),
            ("flip-start, flip-start", "flip-start, flip-start"),
            ("flip-start flip-inline flip-block", "flip-start flip-inline flip-block"),
            ("flip-block, --foo", "flip-block, --foo"),
            ("--bar, flip-block flip-start", "--bar, flip-block flip-start"),
            ("--foo, --bar, --baz", "--foo, --bar, --baz"),
            ("--bar flip-block", "--bar flip-block"),
            ("--bar flip-inline flip-block", "--bar flip-inline flip-block"),
            ("flip-inline --foo", "--foo flip-inline"),
            ("flip-inline flip-start --foo", "--foo flip-inline flip-start"),
            ("left top", "left top"),
            ("top left", "left top"),
            ("start start", "start"),
            ("left, right", "left, right"),
            ("--foo, left", "--foo, left"),
            ("--foo, left, --bar", "--foo, left, --bar"),
            ("--foo, flip-start, left", "--foo, flip-start, left"),
            ("--foo flip-start, left", "--foo flip-start, left"),
            ("left, --bar flip-start", "left, --bar flip-start"),
            ("flip-x", "flip-x"),
            ("flip-start flip-y flip-x", "flip-start flip-y flip-x"),
            ("flip-y --foo", "--foo flip-y"),
            ("flip-y flip-x", "flip-y flip-x"),
            (
                "flip-start flip-inline flip-y flip-block flip-x",
                "flip-start flip-inline flip-y flip-block flip-x",
            ),
        ] {
            assert_eq!(
                canonicalize_decl("position-try-fallbacks", input).as_deref(),
                Some(expect),
                "valid {input}"
            );
        }
        // fallbacks invalid 矩阵。
        for v in [
            "none, flip-start",
            "flip-block flip-block",
            "flip-block flip-inline flip-inline",
            "flip-block, flip-inline flip-inline",
            "--bar flip-block --foo",
            "--foo --bar",
            "flip-inline --bar flip-block",
            "-foo",
            "foo",
            "flip-start 123",
            "--foo 123",
            "--foo left",
            "flip-start left",
            "left --foo ",
            "left flip-start",
            "--foo, none",
            "none, flip-x",
            "flip-y flip-y",
        ] {
            assert_eq!(canonicalize_decl("position-try-fallbacks", v), None, "invalid {v}");
        }
    }

    #[test]
    fn test_position_try_shorthand() {
        // valid 矩阵（含 canonical 折叠）。
        for (input, expect) in [
            ("flip-inline", "flip-inline"),
            ("flip-y", "flip-y"),
            ("most-height none", "most-height none"),
            ("--bar, --baz", "--bar, --baz"),
            (
                "most-inline-size --baz, flip-inline",
                "most-inline-size --baz, flip-inline",
            ),
            (
                "most-block-size flip-inline flip-block, --bar, --baz",
                "most-block-size flip-inline flip-block, --bar, --baz",
            ),
            (
                "most-width flip-y flip-x, --foo, --bar",
                "most-width flip-y flip-x, --foo, --bar",
            ),
            ("normal none", "none"),
            ("most-width none", "most-width none"),
            ("normal --foo", "--foo"),
            ("none", "none"),
        ] {
            assert_eq!(
                canonicalize_decl("position-try", input).as_deref(),
                Some(expect),
                "valid {input}"
            );
        }
        for v in [
            "normal --foo, most-width --bar",
            "none normal",
            "flip-block most-height",
            "most-height, flip-start",
        ] {
            assert_eq!(canonicalize_decl("position-try", v), None, "invalid {v}");
        }
        // 拆分对。
        for (input, order, fb) in [
            ("flip-inline", "normal", "flip-inline"),
            ("flip-y", "normal", "flip-y"),
            ("most-width none", "most-width", "none"),
            ("--foo, --bar", "normal", "--foo, --bar"),
            (
                "most-inline-size --foo, flip-inline",
                "most-inline-size",
                "--foo, flip-inline",
            ),
            (
                "most-inline-size flip-inline flip-block, --foo, --bar",
                "most-inline-size",
                "flip-inline flip-block, --foo, --bar",
            ),
            (
                "most-width flip-y flip-x, --foo, --bar",
                "most-width",
                "flip-y flip-x, --foo, --bar",
            ),
            ("none", "normal", "none"),
            ("normal --foo", "normal", "--foo"),
        ] {
            let (o, f) = split_position_try(input).unwrap();
            assert_eq!((o.as_str(), f.as_str()), (order, fb), "split {input}");
        }
        // 合成对（拆分↔合成为逆）。
        assert_eq!(join_position_try("normal", "flip-inline"), "flip-inline");
        assert_eq!(join_position_try("normal", "none"), "none");
        assert_eq!(join_position_try("most-width", "none"), "most-width none");
        assert_eq!(
            join_position_try("most-inline-size", "--foo, flip-inline"),
            "most-inline-size --foo, flip-inline"
        );
        // 幂等：canonical 简写重拆重合不变。
        for v in [
            "flip-inline",
            "none",
            "most-width none",
            "most-width flip-y flip-x, --foo, --bar",
        ] {
            let canon = canonicalize_decl("position-try", v).unwrap();
            let (o, f) = split_position_try(&canon).unwrap();
            assert_eq!(join_position_try(&o, &f), canon, "idempotent {v}");
        }
    }

    #[test]
    fn test_anchor_func_invalid() {
        // 属性域拒。
        for prop in ["margin-top", "height", "font-size"] {
            assert!(!anchor_funcs_valid(prop, "anchor(--foo top)"), "{prop} domain");
        }
        // 参数畸形。
        for v in [
            "anchor(--foo, top)",
            "anchor(--foo top,)",
            "anchor(--foo top bottom)",
            "anchor(--foo top, 10px 20%)",
            "anchor(--foo top, 10px, 20%)",
            "anchor(2 * 20%)",
            "anchor((2 * 20%))",
            "anchor(foo top)",
            "anchor(top foo)",
            "anchor(--foo height)",
            "anchor(--foo 10em)",
            "anchor(--foo 100s)",
            "anchor(--foo top, 1)",
            "anchor(--foo top, 100s)",
            "anchor(--foo top, bottom)",
            "anchor(--foo top, anchor(bar top))",
            "anchor(--foo top, anchor-size(bar height))",
            "anchor(--foo top, auto",
            "calc(anchor(foo top) + 10px + 10%)",
            "calc(10px + 100 * anchor(--foo top, anchor(bar bottom)))",
            "min(anchor(--foo top), anchor(--bar bottom), anchor-size(baz height))",
            "calc(anchor(--foo top, 1) + 1px)",
        ] {
            assert!(!anchor_funcs_valid("top", v), "invalid {v}");
        }
        // anchor-size 参数畸形（width 域）。
        for v in [
            "anchor-size(--foo, width)",
            "anchor-size(--foo width,)",
            "anchor-size(--foo width height)",
            "anchor-size(--foo width, 10px 20%)",
            "anchor-size(--foo width, 10px, 20%)",
            "anchor-size(--foo,)",
            "anchor-size(, 10px)",
            "anchor-size(foo width)",
            "anchor-size(--foo top)",
            "anchor-size(--foo 10em)",
            "anchor-size(--foo 100s)",
            "anchor-size(--foo 50%)",
            "anchor-size(--foo width, 1)",
            "anchor-size(--foo width, 100s)",
            "anchor-size(--foo width, height)",
            "anchor-size(--foo width, anchor-size(bar width))",
            "anchor-size(--foo width, anchor(--bar top))",
            "calc(anchor-size(foo width) + 10px + 10%)",
            "calc(10px + 100 * anchor-size(--foo width, anchor-size(bar bottom)))",
            "min(anchor-size(--foo width), anchor-size(--bar height), anchor(--baz top))",
        ] {
            assert!(!anchor_funcs_valid("width", v), "size invalid {v}");
        }
        // anchor-size 属性域：size/inset/margin 三族内均合法（语料 valid 循环
        // size+inset+margin 全集；top 为 inset 族成员），域外（font-size）拒。
        assert!(anchor_funcs_valid("top", "anchor-size(--foo width)"));
        assert!(!anchor_funcs_valid("font-size", "anchor-size(--foo width)"));
    }

    #[test]
    fn test_anchor_func_valid_passthrough() {
        // valid 面保形放行（round-trip 原文 echo 语料钉面）。
        for v in [
            "anchor(50%)",
            "anchor(calc(50%))",
            "anchor(min(50%, 100%))",
            "anchor(--foo left)",
            "anchor(--bar left, anchor(--baz right))",
            "anchor(--foo top, 1px)",
            "anchor(--foo top, 50%)",
            "anchor(--foo top, calc(50% + 1px))",
            "calc(anchor(--foo left) + 10px)",
            "anchor(inside)",
            "anchor(self-start)",
            "anchor(--foo center)",
        ] {
            assert!(anchor_funcs_valid("top", v), "valid {v}");
            assert!(anchor_funcs_valid("left", v), "valid left {v}");
        }
        for v in [
            "anchor-size(--foo width)",
            "anchor-size(width)",
            "anchor-size()",
            "anchor-size(10px)",
            "anchor-size(width --foo)",
            "anchor-size(--foo width, 10px)",
            "anchor-size(--foo width, 0)",
            "anchor-size(--foo width, calc(50%))",
            "anchor-size(--foo width, anchor-size(--baz height))",
            "calc(anchor-size(--foo) + 10px)",
        ] {
            assert!(anchor_funcs_valid("width", v), "size valid {v}");
            assert!(anchor_funcs_valid("margin-top", v), "size margin valid {v}");
            assert!(anchor_funcs_valid("top", v), "size inset valid {v}");
        }
        // anchor() flip 序（语料 `anchor(<side> <name>)` 钉面）+ unitless zero
        // fallback。
        for v in ["anchor(top --foo)", "anchor(50% --foo)", "anchor(--foo left, 0)"] {
            assert!(anchor_funcs_valid("top", v), "flip/zero valid {v}");
        }
        // anchor() 空参拒（语料未收空参形；anchor-size() 隐式形专属）。
        assert!(!anchor_funcs_valid("top", "anchor()"));
        // 非 anchor 值零介入（通用 inset 值照常走后续链）。
        assert!(anchor_funcs_valid("top", "10px"));
        assert!(anchor_funcs_valid("top", "auto"));
        // var() 混合形由 shim 侧先行保形（本函数不触达），此处防御放行。
        assert!(anchor_funcs_valid("top", "var(--x)"));
    }

    #[test]
    fn test_computed_identity() {
        assert_eq!(computed_value("position-anchor", "--foo"), "--foo");
        assert_eq!(computed_value("position-anchor", ""), "normal");
        assert_eq!(computed_value("position-visibility", ""), "anchors-visible");
        assert_eq!(computed_value("position-try-order", ""), "normal");
        assert_eq!(computed_value("position-try-fallbacks", ""), "none");
        assert_eq!(computed_value("anchor-name", ""), "none");
        assert_eq!(computed_value("anchor-scope", "--a"), "--a");
    }
}
