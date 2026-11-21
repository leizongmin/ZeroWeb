//! calc-size() 指定值 canonical 串化（刀 CALCSIZE-CANON）。
//!
//! css-values-5 §8 `calc-size(<calc-size-basis>, <calc-size-sum>)` 写面
//! store 期 canonical 形（WPT css-values calc-size-parsing serialize 桶 6F
//! 钉面，Chromium 口径）：
//! - basis 绝对长度 px 归一（`2in` → `192px`）；basis 关键字保形；嵌套
//!   calc-size 仅 basis 位（§8 文法）递归串化；
//! - sum 项序 % 前置（`50px + 30%` → `30% + 50px`，与 [`super::serialize_math`]
//!   §10.6 sort 同口径）；积项规范形——数字因子前置（`size * 2` →
//!   `2 * size`）、除法收倒数乘法（`size / 2` → `0.5 * size`）；和内积项
//!   加括号（`30px + size / 2` → `30px + (0.5 * size)`）。
//! - 非单一顶层 calc-size 形 / 0 或 2+ 顶层逗号 / sum 混合 +/- 序 /
//!   多因子积 / 除零均返 `None` 保留原文零漂移（[`super::serialize_math`]
//!   `fold_decl_specified` 通道契约：None = 原文透传）。
//!
//! 写面门（style-system `calc_size_decl_gate`）以同一 `parse_math_function`
//! 代裁裁决 canonical 形（`size` 叶替换后语法等价），store 回写文本再写恒
//! 通过——重设/replay 门不丢声明。
//!
//! <https://drafts.csswg.org/css-values-5/#calc-size>

/// calc-size() 前缀（含开括号，长度 10）。
const PREFIX: &str = "calc-size(";

/// 刀 CALCSIZE-CANON 电池 A/B 烙印（b0 剥 face 时随父版消失；零引用，
/// `#[used]` 保证 rodata 携带，勿 deref）——函数符号会被 release 内联抹除，
/// 电池判别只认代码字面 ASCII（CALCSIZE grammar-gate 先例）。
#[used]
pub(crate) static CALCSIZE_CANON_FACE_MARKER: &str = "CALCSIZE canon-serialize";

/// calc-size() 指定值 canonical 串化入口。非 calc-size 形返 `None`。
pub fn canonicalize_calc_size_specified(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    let inner = single_calc_size_span(v)?;
    let (basis, sum) = split_top_level_comma(inner)?;
    let basis_c = canonicalize_basis(basis.trim())?;
    let sum_c = canonicalize_sum(sum.trim())?;
    // 函数名保作者形（store 零漂移；WPT serialize 桶全小写语料不受影响）
    Some(format!("{}({basis_c}, {sum_c})", &v[..PREFIX.len() - 1]))
}

/// 整值是否单一顶层 `calc-size(...)` 跨度（大小写不敏感前缀 + 括号配平 +
/// 闭括号后无残余）。返回括号内 inner 文本切片（原文形）。小写副本只作
/// 探测，切片锚回原文（`to_ascii_lowercase` 保字节长度对齐）。
fn single_calc_size_span(v: &str) -> Option<&str> {
    let lower = v.to_ascii_lowercase();
    if !lower.starts_with(PREFIX) {
        return None;
    }
    let bytes = lower.as_bytes();
    let mut depth = 1usize;
    let mut end = None;
    // 前缀第 10 字节即 `(` 已计入 depth，扫描自其后一字节起
    for j in PREFIX.len()..bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    // 闭括号后残余 → 多跨度/尾 token 拒（门同口径）
                    if !v[j + 1..].trim().is_empty() {
                        return None;
                    }
                    end = Some(j);
                    break;
                }
            }
            _ => {}
        }
    }
    Some(&v[PREFIX.len()..end?])
}

/// 括号配平下的唯一顶层逗号切分（0/2+ 顶层逗号拒——门双参文法同口径）。
fn split_top_level_comma(inner: &str) -> Option<(&str, &str)> {
    let bytes = inner.as_bytes();
    let mut depth = 0i32;
    let mut idx: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            b',' if depth == 0 => {
                if idx.is_some() {
                    return None;
                }
                idx = Some(i);
            }
            _ => {}
        }
    }
    let i = idx?;
    Some((&inner[..i], &inner[i + 1..]))
}

/// basis canonical：关键字保形（大小写归一不做——store 保作者形零漂移；
/// 写面门按 per-属性域裁决，此处不重裁域）；嵌套 calc-size 递归；绝对
/// 长度 px 归一；其余 length-percentage token（%/相对单位）保形。
fn canonicalize_basis(b: &str) -> Option<String> {
    let t = b.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    // <calc-size-basis> 关键字域（css-values-5 §8；`size` 非 basis 关键字）
    if matches!(
        lower.as_str(),
        "auto" | "any" | "min-content" | "max-content" | "fit-content" | "stretch" | "content"
    ) {
        return Some(t.to_string());
    }
    // 嵌套 calc-size 仅 basis 位（§8）——递归 canonical
    if lower.starts_with("calc-size(") {
        return canonicalize_calc_size_specified(t);
    }
    // length-percentage 叶：绝对单位 px 归一，其余保形
    match absolute_dim_to_px_text(t) {
        Some(px) => Some(px),
        None => Some(t.to_string()),
    }
}

/// 绝对长度原子 px 归一（css-values-4 §8.4 绝对单位 96 DPI 表，与
/// `parse_basic::parse_length` 同系数）；`px` 形/相对单位/%/非长度 token
/// 返 `None` 由调用方原文保形。
fn absolute_dim_to_px_text(token: &str) -> Option<String> {
    let t = token.trim();
    let (num_part, unit) = split_number_unit(t)?;
    let n: f64 = num_part.parse().ok()?;
    let px = match unit.to_ascii_lowercase().as_str() {
        "in" => n * 96.0,
        "pt" => n * 96.0 / 72.0,
        "pc" => n * 96.0 / 6.0,
        "cm" => n * 96.0 / 2.54,
        "mm" => n * 96.0 / 25.4,
        "q" => n * 96.0 / 101.6,
        _ => return None,
    };
    Some(format!("{}px", super::serialize_math::canonical_number_text(px)))
}

/// 数字 + 单位切分：数字段（含符号/小数/指数）自头起，其余为单位段。
fn split_number_unit(t: &str) -> Option<(&str, &str)> {
    let b = t.as_bytes();
    let mut i = 0usize;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut saw_digit = false;
    let mut saw_dot = false;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_digit() {
            saw_digit = true;
            i += 1;
        } else if c == b'.' && !saw_dot {
            saw_dot = true;
            i += 1;
        } else if (c == b'e' || c == b'E') && saw_digit {
            // 指数段（`2e2px` 罕见形；保守只在后随数字/符号时纳入）
            if i + 1 < b.len() && (b[i + 1].is_ascii_digit() || b[i + 1] == b'+' || b[i + 1] == b'-') {
                i += 2;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    if !saw_digit || i == 0 || i >= b.len() {
        return None;
    }
    Some((&t[..i], &t[i..]))
}

/// sum canonical：顶层 `+` 拆项（含顶层 `-` 的混合符号序不重排返 `None`
/// 保形）；逐项积规范形/原子 px 归一；% 前置稳定排序（仅全 `+` 序重排，
/// 符号语义不变）；多顶和内积项加括号。
fn canonicalize_sum(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let terms = split_top_level_plus(t)?;
    let mut canonical: Vec<(bool, String)> = Vec::with_capacity(terms.len());
    for term in &terms {
        let term = term.trim();
        if term.is_empty() {
            return None;
        }
        if has_top_level_minus(term) {
            // 混合 +/- 序：canonical 序不重排（符号迁移语义面未钉）——整值
            // 保形零漂移
            return None;
        }
        let is_pct = is_top_level_percentage(term);
        let rendered = canonicalize_term_text(term)?;
        canonical.push((is_pct, rendered));
    }
    // % 前置稳定排序（§10.6 部分 sort；非 % 类保原相对序——
    // serialize_math「% 项前置」同口径）
    canonical.sort_by_key(|(is_pct, _)| !*is_pct);
    let parts: Vec<String> = canonical.into_iter().map(|(_, s)| s).collect();
    Some(join_sum_terms(&parts))
}

/// 顶层 `+` 拆项（括号/函数内 `+` 不拆；空段拒）。
fn split_top_level_plus(s: &str) -> Option<Vec<&str>> {
    let bytes = s.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            b'+' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    parts.push(&s[start..]);
    Some(parts)
}

/// 项内顶层 `-` 探测（前随空白判定——负号数字字面 `-.5px`/`2e-2` 不误伤；
/// calc-size sum 文法操作数间必须空白分隔，故前随空白即顶层二元减）。
fn has_top_level_minus(term: &str) -> bool {
    let bytes = term.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'-' if depth == 0 && i > 0 && bytes[i - 1].is_ascii_whitespace() => return true,
            _ => {}
        }
    }
    false
}

/// 项是否顶层裸百分比 token（`N%`——数字段 + 尾 `%`，无内层运算）。
fn is_top_level_percentage(term: &str) -> bool {
    match term.trim().strip_suffix('%') {
        Some(num_part) => num_part.parse::<f64>().is_ok(),
        None => false,
    }
}

/// 单项 canonical 文本：积项规范形（数字因子前置/除法收倒数乘法）或原子
/// （绝对单位 px 归一；`size` 关键字/%/相对单位/嵌套括号原子保形）。
fn canonicalize_term_text(term: &str) -> Option<String> {
    let t = term.trim();
    // 整括号包裹项先剥外括号（canonical 重入幂等——上一轮 join 已加的和内
    // 积括号不得再叠一层）；仅当内层恰为单积/除形才剥，其余括号组保形
    if let Some(inner) = fully_wrapped_parens(t) {
        if top_level_muldiv_ops(inner.trim()).len() == 1 {
            return canonicalize_term_text(inner.trim());
        }
    }
    let ops = top_level_muldiv_ops(t);
    if ops.len() > 1 {
        // 多因子积：canonical 形未钉——整值保形拒
        return None;
    }
    if ops.len() == 1 {
        let (i, op) = ops[0];
        // 顶层积/除运算符（括号内不拆；css-values-4 §9.6 乘除不要求空白）
        return canonicalize_product(t[..i].trim(), op, t[i + 1..].trim());
    }
    // 原子：绝对单位 px 归一，其余保形
    match absolute_dim_to_px_text(t) {
        Some(px) => Some(px),
        None => Some(t.to_string()),
    }
}

/// `t` 是否被单一外层括号对完整包裹（首字符 `(` 的深度在末字符处才归零）。
fn fully_wrapped_parens(t: &str) -> Option<&str> {
    let b = t.as_bytes();
    if b.first() != Some(&b'(') || b.last() != Some(&b')') {
        return None;
    }
    let mut depth = 0i32;
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 && i != b.len() - 1 {
                    // 中途归零 → 括号非整包（如 `(a)(b)`）——不剥
                    return None;
                }
            }
            _ => {}
        }
    }
    Some(&t[1..b.len() - 1])
}

/// 顶层 `*`/`/` 运算符清点（括号内不计；位置 + 运算符序对列表）。
fn top_level_muldiv_ops(t: &str) -> Vec<(usize, char)> {
    let bytes = t.as_bytes();
    let mut depth = 0i32;
    let mut ops = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'*' | b'/' if depth == 0 => ops.push((i, b as char)),
            _ => {}
        }
    }
    ops
}

/// 积项规范形（§10.6 canonical：数字因子前置；除法收倒数乘法）：
/// - `X * n` / `n * X` → `{n} * {X}`（X 先经原子 px 归一）；
/// - `X / n` → `{1/n} * {X}`（n=0 拒）；
/// - `n * X` 保形；双侧非裸数字/`size` 关键字以外复合因子返 `None` 保形。
fn canonicalize_product(left: &str, op: char, right: &str) -> Option<String> {
    let l = left.trim();
    let r = right.trim();
    let r_num = r.parse::<f64>().ok();
    let l_num = l.parse::<f64>().ok();
    let factor_atom = |atom: &str| -> Option<String> {
        // 因子原子：`size` 关键字保形；绝对单位 px 归一；其余（%/相对单位）
        // 保形
        if atom.eq_ignore_ascii_case("size") {
            return Some(atom.to_string());
        }
        match absolute_dim_to_px_text(atom) {
            Some(px) => Some(px),
            None => Some(atom.to_string()),
        }
    };
    match (l_num, r_num, op) {
        // n * X：保形（已 canonical 因子序）
        (Some(n), None, '*') => {
            let atom = factor_atom(r)?;
            Some(format!(
                "{} * {}",
                super::serialize_math::canonical_number_text(n),
                atom
            ))
        }
        // X * n：数字前置
        (None, Some(n), '*') => {
            let atom = factor_atom(l)?;
            Some(format!(
                "{} * {}",
                super::serialize_math::canonical_number_text(n),
                atom
            ))
        }
        // X / n：倒数乘法（n=0 拒——门 sum 文法除零非法同口径）
        (None, Some(n), '/') if n != 0.0 => {
            let atom = factor_atom(l)?;
            Some(format!(
                "{} * {}",
                super::serialize_math::canonical_number_text(1.0 / n),
                atom
            ))
        }
        _ => None,
    }
}

/// 和串装配：单项裸形（积项不加括号）；多顶和内积项加括号（§10.6 和内
/// 积序列化约定，`30px + (0.5 * size)` 语料钉面）；原子保形。
fn join_sum_terms(parts: &[String]) -> String {
    if parts.len() == 1 {
        return parts[0].clone();
    }
    parts
        .iter()
        .map(|p| {
            if p.contains('*') || p.contains('/') {
                format!("({p})")
            } else {
                p.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod tests {
    use super::canonicalize_calc_size_specified as canon;

    #[test]
    fn calc_size_canon_six_serialize_forms() {
        // calc-size-parsing serialize 桶 6F 钉面（WPT Chromium 口径）
        assert_eq!(
            canon("calc-size(min-content, size * 2)").as_deref(),
            Some("calc-size(min-content, 2 * size)")
        );
        assert_eq!(
            canon("calc-size(max-content, size / 2)").as_deref(),
            Some("calc-size(max-content, 0.5 * size)")
        );
        assert_eq!(
            canon("calc-size(fit-content, 30px + size / 2)").as_deref(),
            Some("calc-size(fit-content, 30px + (0.5 * size))")
        );
        assert_eq!(
            canon("calc-size(fit-content, 50% + size / 2)").as_deref(),
            Some("calc-size(fit-content, 50% + (0.5 * size))")
        );
        assert_eq!(
            canon("calc-size(any, 50px + 30%)").as_deref(),
            Some("calc-size(any, 30% + 50px)")
        );
        assert_eq!(
            canon("calc-size(calc-size(2in, 30px), 25em)").as_deref(),
            Some("calc-size(calc-size(192px, 30px), 25em)")
        );
    }

    #[test]
    fn calc_size_canon_idempotent_and_pass_through() {
        // canonical 入再串化恒等
        for v in [
            "calc-size(min-content, 2 * size)",
            "calc-size(any, 30% + 50px)",
            "calc-size(fit-content, 30px + (0.5 * size))",
            "calc-size(auto, size)",
            "calc-size(192px, 30px)",
        ] {
            assert_eq!(canon(v).as_deref(), Some(v), "idempotent: {v}");
        }
        // 非 calc-size 形 / 非文法形 None 保形（fold 通道零漂移契约）
        assert_eq!(canon("calc(50px + 30%)"), None);
        assert_eq!(canon("calc-size(min-content, size * 2) extra"), None);
        assert_eq!(canon("calc-size(min-content)"), None);
        assert_eq!(canon("calc-size(min-content, 1px, 2px)"), None);
        assert_eq!(canon("calc-size(min-content, 30px - size / 2)"), None);
        assert_eq!(canon("calc-size(min-content, size / 0)"), None);
        assert_eq!(canon("calc-size(min-content, size * 2 * 3)"), None);
        assert_eq!(canon("calc-size(auto)"), None);
        assert_eq!(canon(""), None);
    }

    #[test]
    fn calc_size_canon_relative_and_pct_basis_keep_shape() {
        // 相对单位/% basis 保形；大小写关键字保形
        assert_eq!(canon("calc-size(10em, size)").as_deref(), Some("calc-size(10em, size)"));
        assert_eq!(canon("calc-size(50%, size)").as_deref(), Some("calc-size(50%, size)"));
        assert_eq!(
            canon("CALC-SIZE(MAX-CONTENT, SIZE * 2)").as_deref(),
            Some("CALC-SIZE(MAX-CONTENT, 2 * SIZE)")
        );
    }
}
