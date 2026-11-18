//! transition 族声明值 canonical 串化（css-transitions-1）。
//!
//! 循 [`super::serialize_gradient`] 每族一模块先例：文本级 per-component
//! canonical，语料钉面（css-transitions parsing valid 页）；identity 形
//! None passthrough 零漂移，invalid 形由写面 gate 拒收。

/// transition-timing-function 声明值 canonical：`step-start` → `steps(1,
/// start)`、`step-end` → `steps(1)`、`steps(N, end|jump-end)` → `steps(N)`
/// （缺省步进位省略——valid 语料钉面）；ease 系/linear()/cubic-bezier()/
/// 其余 steps 位（start/jump-start/jump-both/jump-none）identity passthrough。
/// 逗号列表 per-component 处置（`ease, step-start` → `ease, steps(1, start)`），
/// 全组件 identity 返 None 保原文。keyword/函数名大小写不敏感（canonical
/// 小写），数值 token 原文保形。CSS-wide keyword/var()/attr() None。
///
/// https://drafts.csswg.org/css-transitions-1/#transition-timing-function-property
pub fn canonicalize_timing_function_specified(value: &str) -> Option<String> {
    let v = value.trim();
    if is_reserved_for_canonical(v) {
        return None;
    }
    let mut changed = false;
    let mut out = Vec::new();
    for comp in split_top_level_commas(v) {
        match canonicalize_timing_component(comp) {
            Some(c) => {
                changed = true;
                out.push(c);
            }
            None => out.push(comp.trim().to_string()),
        }
    }
    if !changed {
        return None;
    }
    Some(out.join(", "))
}

/// transition-property 声明值 canonical：关键字 `all`/`none` 大小写不敏感
/// 归一小写（`ALL` → `all`，valid 语料钉面）；custom ident 原文保形
///（`INVALID`/`SYNTAX`/`SRC` 不动）。逗号列表 per-component，全 identity
/// 返 None。
///
/// https://drafts.csswg.org/css-transitions-1/#transition-property-property
pub fn canonicalize_transition_property_specified(value: &str) -> Option<String> {
    let v = value.trim();
    if is_reserved_for_canonical(v) {
        return None;
    }
    let mut changed = false;
    let mut out = Vec::new();
    for comp in split_top_level_commas(v) {
        let t = comp.trim();
        let canonical = if t.eq_ignore_ascii_case("all") {
            "all"
        } else if t.eq_ignore_ascii_case("none") {
            "none"
        } else {
            // 空段/custom ident 原样（invalid 面写面 gate 拒收，不重排）
            t
        };
        if canonical != t {
            changed = true;
        }
        out.push(canonical.to_string());
    }
    if !changed {
        return None;
    }
    Some(out.join(", "))
}

/// CSS-wide keyword 与 var()/attr() 形不进 canonical（原文通道）。
fn is_reserved_for_canonical(v: &str) -> bool {
    let low = v.to_ascii_lowercase();
    matches!(
        low.as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
    ) || low.starts_with("var(")
        || low.starts_with("attr(")
}

/// 单组件 timing canonical（无处置 None）。
fn canonicalize_timing_component(comp: &str) -> Option<String> {
    let t = comp.trim();
    let low = t.to_ascii_lowercase();
    if low == "step-start" {
        return Some("steps(1, start)".to_string());
    }
    if low == "step-end" {
        return Some("steps(1)".to_string());
    }
    if !low.starts_with("steps(") || !t.ends_with(')') {
        return None;
    }
    let inner = &t[6..t.len() - 1];
    let args = split_top_level_commas(inner);
    if args.len() != 2 {
        return None;
    }
    let kw = args[1].trim().to_ascii_lowercase();
    if kw == "end" || kw == "jump-end" {
        Some(format!("steps({})", args[0].trim()))
    } else {
        None
    }
}

/// 括号深度感知顶层逗号切分（组件/实参两用；引号不感知——timing/property
/// 值面无引号 token）。
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}
