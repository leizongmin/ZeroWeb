//! 渐变色标 calc 槽型别门与 canonical 序（g47）。
//!
//! driving: wpt css-images conic-gradient-calc-angle-percentage-{valid,invalid}。
//! 色标位置槽型别（css-images-4 §color-stop）：linear/radial 为
//! `<length-percentage>`（不收角度原子），conic 为 `<angle-percentage>`（不收
//! 纯数字项）——数学函数槽在 parse 前按族拒绝。conic 色标 calc canonical：
//! 顶层二元 `+` 百分比项在角度项前（`calc(0deg + 100%)` → `calc(100% + 0deg)`）。

use super::serialize_math::span_has_top_level_angle_unit;

/// 数学函数前缀（calc/min/max/clamp——色标位置与 hint 共用的 math 判定面）。
const MATH_PREFIXES: [&str; 4] = ["calc(", "min(", "max(", "clamp("];

/// span 是否以数学函数开头。
pub(crate) fn is_math_span(s: &str) -> bool {
    MATH_PREFIXES.iter().any(|p| s.starts_with(p))
}

/// math span 顶层（括号深度 0）`+`/`-` 段中是否出现纯数字操作数。
///
/// conic 色标 `<angle-percentage>` 槽用：角度叶在 [`super::types`]
/// parse_atom 已 Number 化（弧度），型别信息不可从 CalcExpr 恢复，须按
/// 原文 provenance 判——`calc(50% + 0)` 拒、`calc(50% * 2)` 放行（乘法
/// 纯数字系数合法）。
pub(crate) fn span_has_top_level_number_operand(span: &str) -> bool {
    // 剥外层 math 函数壳（`calc(` 至平衡尾括号），顶层 +/- 在内参深度 0。
    let Some(open) = span.find('(') else {
        return false;
    };
    let inner = span[open + 1..].strip_suffix(')').unwrap_or(&span[open + 1..]);
    let bytes = inner.as_bytes();
    let mut depth = 0i32;
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            // 深度 0 空白后紧跟二元 +/- → 运算符边界（css-values-4 calc 语法
            // +/- 两侧须空白；`*`/`/` 系数与积式不入段——`50% * 2` 非纯数字段）。
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                    let seg = inner[seg_start..i].trim();
                    if !seg.is_empty() && seg.parse::<f64>().is_ok() {
                        return true;
                    }
                    seg_start = j + 1;
                    i = j;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let seg = inner[seg_start..].trim();
    !seg.is_empty() && seg.parse::<f64>().is_ok()
}

/// 色标参数内 calc span canonical：顶层二元 `+` 百分比项在角度项前。
///
/// 语料钉（conic-gradient-calc-angle-percentage-valid）：
/// `calc(0deg + 100%)` → `calc(100% + 0deg)`、`calc(90deg + 50%)` →
/// `calc(50% + 90deg)`、`calc(90deg + 0%)` → `calc(0% + 90deg)`。
/// 仅交换「前项含角单位 + 后项纯百分比」的二元 `+`（`-` 不可交换；
/// 其余形原文保形）。参数可为 `red calc(...)` 色标或裸 calc（hint）。
pub(crate) fn canonicalize_calc_stop_arg(arg: &str) -> String {
    let lower = arg.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        // 顶层 calc( 头（名前不得为 ident 字符）。
        let boundary_ok =
            i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_');
        if depth == 0 && boundary_ok && lower[i..].starts_with("calc(") {
            // 平衡括号取 span。
            let mut d = 1i32;
            let mut j = i + 5;
            while j < bytes.len() && d > 0 {
                match bytes[j] {
                    b'(' => d += 1,
                    b')' => d -= 1,
                    _ => {}
                }
                j += 1;
            }
            if d != 0 {
                return arg.to_string(); // 不平衡由上层 parse 拒，此处保形
            }
            let inner = &arg[i + 5..j - 1];
            if let Some(swapped) = swap_pct_before_angle(inner) {
                let mut out = String::with_capacity(arg.len());
                out.push_str(&arg[..i]);
                out.push_str("calc(");
                out.push_str(&swapped);
                out.push(')');
                out.push_str(&arg[j..]);
                return out;
            }
            return arg.to_string();
        }
        i += 1;
    }
    arg.to_string()
}

/// calc 内参顶层二元 `+` 的百分比/角度换位；不匹配返 None。
fn swap_pct_before_angle(inner: &str) -> Option<String> {
    let bytes = inner.as_bytes();
    let mut depth = 0i32;
    let mut tokens: Vec<&str> = Vec::new();
    let mut seg_start = 0usize;
    for (idx, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                if seg_start < idx {
                    tokens.push(&inner[seg_start..idx]);
                }
                seg_start = idx + 1;
            }
            _ => {}
        }
    }
    if seg_start < inner.len() {
        tokens.push(&inner[seg_start..]);
    }
    // 恰为「operand + operand」三元组（二元加法），其余形不重排。
    if tokens.len() != 3 || tokens[1] != "+" {
        return None;
    }
    let first_is_angle = span_has_top_level_angle_unit(tokens[0]);
    let second_is_pct = tokens[2]
        .strip_suffix('%')
        .is_some_and(|n| !n.trim().is_empty() && n.parse::<f64>().is_ok());
    if first_is_angle && second_is_pct {
        Some(format!("{} + {}", tokens[2], tokens[0]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_calc_stop_arg, span_has_top_level_number_operand};

    /// 语料钉：conic 色标 calc canonical 序（valid 页 5 案期望串）。
    #[test]
    fn test_canonicalize_calc_stop_pct_before_angle() {
        assert_eq!(canonicalize_calc_stop_arg("calc(0deg + 100%)"), "calc(100% + 0deg)");
        assert_eq!(canonicalize_calc_stop_arg("calc(90deg + 50%)"), "calc(50% + 90deg)");
        assert_eq!(canonicalize_calc_stop_arg("calc(90deg + 0%)"), "calc(0% + 90deg)");
        // 色标前缀形（指定面 body 文本）。
        assert_eq!(
            canonicalize_calc_stop_arg("red calc(90deg + 50%)"),
            "red calc(50% + 90deg)"
        );
        // 已 canonical 恒等。
        assert_eq!(canonicalize_calc_stop_arg("calc(50% + 90deg)"), "calc(50% + 90deg)");
        // `-` 不可交换保形；非 %/角度二元保形。
        assert_eq!(canonicalize_calc_stop_arg("calc(90deg - 50%)"), "calc(90deg - 50%)");
        assert_eq!(canonicalize_calc_stop_arg("calc(10px + 50%)"), "calc(10px + 50%)");
    }

    /// 顶层纯数字操作数探测（conic `<angle-percentage>` 槽拒面）。
    #[test]
    fn test_span_has_top_level_number_operand() {
        assert!(span_has_top_level_number_operand("calc(50% + 0)"));
        assert!(span_has_top_level_number_operand("calc(0)"));
        assert!(span_has_top_level_number_operand("calc(min(50%, 60%) + 3)"));
        // 乘法纯数字系数合法不命中。
        assert!(!span_has_top_level_number_operand("calc(50% * 2)"));
        assert!(!span_has_top_level_number_operand("calc(50% + 30deg)"));
        // 嵌套函数内数字不外泄。
        assert!(!span_has_top_level_number_operand("calc(min(1, 2) + 3deg)"));
    }

    /// parse 级端到端：invalid 3 案拒收（槽型别门）+ valid canonical 串（语料
    /// conic-gradient-calc-angle-percentage-{valid,invalid} 全 8 案）。
    #[test]
    fn test_gradient_stop_calc_type_gates_corpus_pins() {
        use crate::values::{canonicalize_background_image_specified, parse_gradient};
        for v in [
            "linear-gradient(red calc(50% + 30deg), blue)",
            "radial-gradient(red calc(50% + 30deg), blue)",
            "conic-gradient(red calc(50% + 0), blue)",
        ] {
            assert!(parse_gradient(v).is_none(), "{v} 应拒收");
        }
        assert_eq!(
            canonicalize_background_image_specified("conic-gradient(red, calc(0deg + 100%), blue)").as_deref(),
            Some("conic-gradient(red, calc(100% + 0deg), blue)")
        );
        assert_eq!(
            canonicalize_background_image_specified("conic-gradient(red calc(0deg + 100%), blue)").as_deref(),
            Some("conic-gradient(red calc(100% + 0deg), blue)")
        );
        assert_eq!(
            canonicalize_background_image_specified("conic-gradient(red calc(90deg + 50%), blue)").as_deref(),
            Some("conic-gradient(red calc(50% + 90deg), blue)")
        );
        assert_eq!(
            canonicalize_background_image_specified("conic-gradient(red calc(90deg + 0%), blue)").as_deref(),
            Some("conic-gradient(red calc(0% + 90deg), blue)")
        );
        assert_eq!(
            canonicalize_background_image_specified("repeating-conic-gradient(red calc(90deg + 50%), blue)").as_deref(),
            Some("repeating-conic-gradient(red calc(50% + 90deg), blue)")
        );
    }
}
