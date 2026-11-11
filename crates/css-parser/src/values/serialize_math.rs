//! 数学函数 specified 值常量折叠序列化（CSS Values 4 §10.11）。
//!
//! WPT css-values serialize 桶（sin-cos-tan-serialize / acos-asin-atan-atan2-serialize）
//! 断言 `el.style.transform = 'rotate(acos(1))'` 读回 `rotate(calc(0deg))`——
//! specified 值序列化须把全常量数学函数折叠为 `calc(<结果>)` 规范文本。
//! 折叠原语在此（css-parser 面）；调用方（inline style 读回/写面归一）按需接线。

use super::parse_extended::{calc_expr_has_angle_result, calc_expr_has_length, calc_expr_is_number};
use super::types::{eval_calc, parse_math_function};

/// 折叠结果的文法型别（决定 NaN/单位序列化形态）。
///
/// - [`MathFoldKind::Number`]：`<number>` 文法位（scale/z-index 等）——
///   NaN 序列化 `calc(NaN)`。
/// - [`MathFoldKind::Angle`]：`<angle>` 文法位（rotate 等）——结果弧度转度，
///   NaN 序列化 `calc(NaN * 1deg)`（Chrome/Blink 口径，WPT serialize 页期望）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathFoldKind {
    /// `<number>` 文法位。
    Number,
    /// `<angle>` 文法位。
    Angle,
}

/// CSS Values 4 §10.11：数学函数 specified 值常量折叠。
///
/// 输入值为纯数学函数（calc/min/max/clamp/裸 trig 等，[`parse_math_function`]
/// 面）时求值并产出规范文本；非数学函数、含长度叶（型别不符）、`<number>`
/// 位非 number 式、角度位 ±Infinity 结果——均返 `None`（调用方保留原文）。
/// NaN 结果合法（§10.10），照常折叠。
///
/// <https://drafts.csswg.org/css-values-4/#calc-serialize>
pub fn fold_math_specified(value: &str, kind: MathFoldKind) -> Option<String> {
    let expr = parse_math_function(value)?;
    // <angle>/<number> 文法位不容 <length> 叶混入（§10.5 型别一致，与
    // parse_angle_or_math 同口径）。
    if calc_expr_has_length(&expr) {
        return None;
    }
    if matches!(kind, MathFoldKind::Number) && !calc_expr_is_number(&expr) {
        return None;
    }
    let value = eval_calc(&expr, None)?;
    Some(match kind {
        MathFoldKind::Number => format_calc_number(value),
        MathFoldKind::Angle => {
            // 角度产出函数（asin/acos/atan/atan2）结果为弧度 → 度；纯 number
            // 式按度直读（与 parse_angle_or_math 单位口径一致）。
            let deg = if calc_expr_has_angle_result(&expr) {
                value.to_degrees()
            } else {
                value
            };
            format_calc_angle(deg)?
        }
    })
}

/// CSSOM serialize-a-number 口径：最多 6 位小数、去尾零、`-0` 归 `0`；
/// NaN/Infinity 字面（Blink `calc(sin(infinity))` → `calc(NaN)` 驱动）。
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

/// `<number>` 折叠形态：`calc(<n>)`。
fn format_calc_number(value: f64) -> String {
    format!("calc({})", format_number_6dp(value))
}

/// `<angle>` 折叠形态：`calc(<n>deg)`；NaN → `calc(NaN * 1deg)`；±Infinity
/// 拒折（返 None——`<angle>` 位不容无穷，保守口径）。
fn format_calc_angle(deg: f64) -> Option<String> {
    if deg.is_infinite() {
        return None;
    }
    if deg.is_nan() {
        return Some("calc(NaN * 1deg)".into());
    }
    Some(format!("calc({}deg)", format_number_6dp(deg)))
}

/// 检测 asin/acos/atan（单参反三角）实参跨度内是否出现角度单位叶。
///
/// CSS Values 4 §10.5：asin/acos/atan 实参仅 `<number>`（结果才是 `<angle>`）。
/// CalcParser 把角度叶 parse 期 Number 化（`45deg` → 弧度 Number），型别信息
/// 丢失，须在字符串面拒绝——WPT acos-asin-atan-atan2-invalid
/// `rotate(atan(45deg ))` 驱动。`atan2(`/sin/cos/tan 实参容角度，不受此限。
pub(crate) fn inverse_trig_arg_has_angle_unit(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（排除 `xasin(` 类嵌套 ident 段；
        // `atan2(` 因 '(' 不紧随 'atan' 天然不命中）。
        let hit = ["asin(", "acos(", "atan("].iter().find(|name| {
            lower[i..].starts_with(*name)
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
        });
        let Some(name) = hit else {
            i += 1;
            continue;
        };
        // 平衡括号取实参跨度（嵌套子式整段入扫——`acos(asin(1deg) + 2)` 内层
        // 角度叶同样命中外层拒绝）。
        let open = i + name.len();
        let mut depth = 1usize;
        let mut j = open;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            // 括号不平衡——上层 parse 整体拒绝，此处不误判放行。
            return false;
        }
        if span_has_angle_unit(&lower[open..j - 1]) {
            return true;
        }
        i = j;
    }
    false
}

/// 跨度内是否出现 `数字 + 角度单位` token：单位前须紧邻数字/点，后须非
/// ident 字符——`45deg`/`3.5grad`/`1turn` 命中；`gradius`/`45radix` 不误报。
fn span_has_angle_unit(span: &str) -> bool {
    const UNITS: [&str; 4] = ["deg", "grad", "rad", "turn"];
    let bytes = span.as_bytes();
    for unit in UNITS {
        let mut from = 0;
        while let Some(pos) = span[from..].find(unit) {
            let start = from + pos;
            let end = start + unit.len();
            let prev_is_num = start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.');
            let next_is_ident =
                end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-' || bytes[end] == b'_');
            if prev_is_num && !next_is_ident {
                return true;
            }
            from = end;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_fold_sin_cos_tan_serialize_table() {
        // WPT sin-cos-tan-serialize.html 期望表（specified 层，type: number）。
        use MathFoldKind::Number;
        assert_eq!(fold_math_specified("cos(0)", Number).as_deref(), Some("calc(1)"));
        assert_eq!(fold_math_specified("sin(0)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(fold_math_specified("tan(0)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(fold_math_specified("calc(cos(0))", Number).as_deref(), Some("calc(1)"));
        assert_eq!(
            fold_math_specified("calc(sin(0) + 0.5)", Number).as_deref(),
            Some("calc(0.5)")
        );
        assert_eq!(
            fold_math_specified("calc(sin(0) + cos(0) + tan(0))", Number).as_deref(),
            Some("calc(1)")
        );
        assert_eq!(fold_math_specified("sin(30deg)", Number).as_deref(), Some("calc(0.5)"));
        assert_eq!(fold_math_specified("cos(60deg)", Number).as_deref(), Some("calc(0.5)"));
        assert_eq!(fold_math_specified("tan(45deg)", Number).as_deref(), Some("calc(1)"));
        assert_eq!(fold_math_specified("cos(180deg)", Number).as_deref(), Some("calc(-1)"));
        assert_eq!(fold_math_specified("sin(270deg)", Number).as_deref(), Some("calc(-1)"));
        assert_eq!(fold_math_specified("sin(-180deg)", Number).as_deref(), Some("calc(0)"));
        assert_eq!(
            fold_math_specified("calc(sin(infinity))", Number).as_deref(),
            Some("calc(NaN)")
        );
        assert_eq!(
            fold_math_specified("calc(cos(NaN))", Number).as_deref(),
            Some("calc(NaN)")
        );
    }

    #[test]
    fn angle_fold_acos_serialize_table() {
        // WPT acos-asin-atan-atan2-serialize.html 期望表（transform: rotate(...)）。
        use MathFoldKind::Angle;
        assert_eq!(fold_math_specified("acos(1)", Angle).as_deref(), Some("calc(0deg)"));
        assert_eq!(fold_math_specified("acos(-1)", Angle).as_deref(), Some("calc(180deg)"));
        assert_eq!(fold_math_specified("acos(0)", Angle).as_deref(), Some("calc(90deg)"));
        assert_eq!(fold_math_specified("acos(0.5)", Angle).as_deref(), Some("calc(60deg)"));
        assert_eq!(fold_math_specified("asin(0.5)", Angle).as_deref(), Some("calc(30deg)"));
        assert_eq!(fold_math_specified("asin(1)", Angle).as_deref(), Some("calc(90deg)"));
        assert_eq!(fold_math_specified("asin(-1)", Angle).as_deref(), Some("calc(-90deg)"));
        assert_eq!(fold_math_specified("atan(1)", Angle).as_deref(), Some("calc(45deg)"));
        assert_eq!(fold_math_specified("atan(0)", Angle).as_deref(), Some("calc(0deg)"));
        assert_eq!(
            fold_math_specified("atan(infinity)", Angle).as_deref(),
            Some("calc(90deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(37.320508075, 10)", Angle).as_deref(),
            Some("calc(75deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(infinity, infinity)", Angle).as_deref(),
            Some("calc(45deg)")
        );
        assert_eq!(
            fold_math_specified("atan2(-infinity, -infinity)", Angle).as_deref(),
            Some("calc(-135deg)")
        );
        // NaN 域外（§10.10 合法）：`calc(NaN * 1deg)`。
        assert_eq!(
            fold_math_specified("acos(1.5)", Angle).as_deref(),
            Some("calc(NaN * 1deg)")
        );
        assert_eq!(
            fold_math_specified("asin(-1.5)", Angle).as_deref(),
            Some("calc(NaN * 1deg)")
        );
        // pi 常量组合。
        assert_eq!(
            fold_math_specified("asin(pi - pi + 1)", Angle).as_deref(),
            Some("calc(90deg)")
        );
    }

    #[test]
    fn fold_rejects_non_math_and_type_mismatch() {
        use MathFoldKind::{Angle, Number};
        // 非数学函数原样保留（None）。
        assert_eq!(fold_math_specified("45deg", Number), None);
        assert_eq!(fold_math_specified("rotate(45deg)", Angle), None);
        assert_eq!(fold_math_specified("none", Number), None);
        // <length> 叶混入：型别不符拒折。
        assert_eq!(fold_math_specified("calc(1px + 2px)", Number), None);
        assert_eq!(fold_math_specified("asin(90px)", Angle), None);
        // <number> 位不容角度产出式（结果为 <angle>）。
        assert_eq!(fold_math_specified("acos(1)", Number), None);
        // 角度位 ±Infinity 拒折（保守口径）。
        assert_eq!(fold_math_specified("calc(infinity)", Angle), None);
    }

    #[test]
    fn inverse_trig_angle_unit_gate_scopes() {
        assert!(inverse_trig_arg_has_angle_unit("atan(45deg )"));
        assert!(inverse_trig_arg_has_angle_unit("asin(30deg + 1rad)"));
        assert!(inverse_trig_arg_has_angle_unit("calc(acos(45deg))"));
        assert!(inverse_trig_arg_has_angle_unit("ACOS(1turn)"));
        // sin/cos/tan 与 atan2 实参容角度——不命中。
        assert!(!inverse_trig_arg_has_angle_unit("sin(30deg)"));
        assert!(!inverse_trig_arg_has_angle_unit("atan2(30deg, 1rad)"));
        assert!(!inverse_trig_arg_has_angle_unit("tan(1turn)"));
        // ident 边界不误报：gradius/45radix/嵌套 ident 段。
        assert!(!inverse_trig_arg_has_angle_unit("asin(gradius)"));
        assert!(!inverse_trig_arg_has_angle_unit("asin(45radix)"));
        // atan2( 不命中 atan( 名扫描。
        assert!(!inverse_trig_arg_has_angle_unit("atan2(1dag, 2deg)"));
        // 括号不平衡不误判。
        assert!(!inverse_trig_arg_has_angle_unit("asin(45deg"));
    }

    #[test]
    fn format_number_6dp_rounding() {
        assert_eq!(format_number_6dp(1.0), "1");
        assert_eq!(format_number_6dp(0.5), "0.5");
        assert_eq!(format_number_6dp(0.49999999999999994), "0.5");
        assert_eq!(format_number_6dp(100.0), "100");
        assert_eq!(format_number_6dp(-0.0), "0");
        assert_eq!(format_number_6dp(-1.0), "-1");
        assert_eq!(format_number_6dp(f64::NAN), "NaN");
        assert_eq!(format_number_6dp(f64::INFINITY), "infinity");
        assert_eq!(format_number_6dp(f64::NEG_INFINITY), "-infinity");
    }
}
