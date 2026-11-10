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
    // Angle 槽单位叶 provenance 用原文串（求值 f64 无单位信息，先行绑定）。
    let has_unit = matches!(kind, MathFoldKind::Angle) && span_has_top_level_angle_unit(&value.to_ascii_lowercase());
    let value = eval_calc(&expr, None)?;
    Some(match kind {
        MathFoldKind::Number => format_calc_number(value),
        MathFoldKind::Angle => {
            // 求值口径全弧度 → 度的两条 provenance：①角度产出函数
            //（asin/acos/atan/atan2）；②trig 实参外角度单位叶——CalcParser
            // parse 期把 `180deg` 类平角度叶 Number 化为弧度（型别标记丢失），
            // 缺 ② 时 `rotate(calc(180deg))` 往返恒等断言会把 180deg 折成
            // `calc(3.141593deg)`（WPT serialize 页双侧同形关键）；②须屏蔽
            // trig 实参——`tan(45deg)` 实参角度叶产出 <number>，按度直读会翻倍
            // 换算（§10.5）。平裸 number 式按度直读（与 parse_angle_or_math
            // 单位口径一致）。
            let deg = if calc_expr_has_angle_result(&expr) || has_unit {
                value.to_degrees()
            } else {
                value
            };
            format_calc_angle(deg)?
        }
    })
}

/// 声明值 specified 折叠入口（按属性分派；None = 保留原文）。
///
/// 消费方：inline style 写面（native CSD canonicalize 链）与读出视图
///（mutation log replay `style_from_mutations_lw`）。白名单仅本轮有 WPT
/// 驱动案的属性：transform（rotate 族 wrapper 折叠）、rotate/scale（裸数学
/// 函数）。其余属性原样保留。
pub fn fold_decl_specified(prop: &str, value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    match prop {
        "transform" => fold_transform_math_specified(v),
        "rotate" => fold_math_specified(v, MathFoldKind::Angle),
        "scale" => fold_math_specified(v, MathFoldKind::Number),
        _ => None,
    }
}

/// transform 列表 wrapper 折叠：rotate/rotatex/rotatey/rotatez/rotate3d 的角度
/// 实参为数学函数时以 [`fold_math_specified`]（Angle）折叠替换，其余函数与原文
/// 大小写保留；无折叠发生返 `None`。skew 族未接线（FIXME：无驱动案，WPT
/// serialize 页仅 rotate 族）。
pub fn fold_transform_math_specified(value: &str) -> Option<String> {
    const ANGLE_FNS: [&str; 5] = ["rotate(", "rotatex(", "rotatey(", "rotatez(", "rotate3d("];
    let lower = value.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（排除嵌套 ident 段误命中）。
        let hit = ANGLE_FNS.iter().find(|name| {
            lower[i..].starts_with(*name)
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
        });
        let Some(name) = hit else {
            i += 1;
            continue;
        };
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
            // 括号不平衡——非法面整体保留原文。
            return None;
        }
        // rotate3d(x y z angle)（空白/逗号分隔）：仅末 token（角度实参）折叠替换；
        // 其余单角度形整体折叠。
        let span = &lower[open..j - 1];
        let folded = if *name == "rotate3d(" {
            let trimmed = span.trim_end();
            let tok_rel = trimmed.rfind([' ', ',']).map(|p| p + 1).unwrap_or(0);
            let tok = trimmed[tok_rel..].trim_start();
            let tok_start = open + tok_rel + (trimmed[tok_rel..].len() - tok.len());
            fold_math_specified(tok.trim(), MathFoldKind::Angle).map(|f| (tok_start, j - 1, f))
        } else {
            fold_math_specified(span.trim(), MathFoldKind::Angle).map(|f| (open, j - 1, f))
        };
        if let Some(f) = folded {
            replacements.push(f);
        }
        i = j;
    }
    if replacements.is_empty() {
        return None;
    }
    // 仅替换折叠跨度，原文其余部分（含大小写）逐字节保留。
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    for (start, end, text) in replacements {
        out.push_str(&value[last..start]);
        out.push_str(&text);
        last = end;
    }
    out.push_str(&value[last..]);
    Some(out)
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
        // 平衡括号取实参跨度；只查**直系顶层叶**（嵌套括号组屏蔽）——
        // `asin(sin(100grad))` 直系实参为函数 `sin(...)`，内层 sin 容角度（§10.5
        // trig 实参 <angle>|<number>，结果 <number> 喂 asin 合法），整段扫会误拒
        // （WPT acos-asin-atan-atan2-computed used 面 6 案驱动）。内层反三角由
        // 下方线性扫描继续深入逐个 gate——`acos(asin(1deg) + 2)` 内层直系叶
        // 照样命中拒绝。
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
        if direct_span_has_angle_unit(&lower[open..j - 1]) {
            return true;
        }
        // 不跳过整段：从函数名后继续扫，嵌套反三角 occurrence 各自 gate。
        i += name.len();
    }
    false
}

/// 直系顶层叶角度单位检测：屏蔽嵌套括号组后的平铺段逐段查
/// [`span_has_angle_unit`]——`asin(1deg + sin(1rad))` 平铺段 '1deg + sin' 命中；
/// `asin(sin(1deg))` 平铺段仅 'sin'（组内内容跳过，由外层线性扫描对内层
/// occurrence 重复 gate）。
fn direct_span_has_angle_unit(span: &str) -> bool {
    let bytes = span.as_bytes();
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            // 平铺段为组前文（组内内容由外层线性扫描对内层 occurrence gate）。
            if span_has_angle_unit(&span[seg_start..i]) {
                return true;
            }
            let mut depth = 1usize;
            i += 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            seg_start = i;
        } else {
            i += 1;
        }
    }
    span_has_angle_unit(&span[seg_start..])
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

/// 跨度内 trig 实参**外**是否出现角度单位叶。
///
/// CSS Values 4 §10.5：sin/cos/tan 实参容 `<angle>`（产出 `<number>`）、
/// asin/acos/atan 实参仅 `<number>`、atan2 同型双实参——角度叶出现在 trig
/// 实参内不代表结果是 `<angle>`（`tan(45deg)` 产出 number，按度直读即错）。
/// 因此顶层判定须屏蔽 trig 调用实参跨度；非 trig 括号组照常扫入
///（`calc((45deg + 1))` 组内角度叶仍命中）。与 [`inverse_trig_arg_has_angle_unit`]
/// 同构：函数名前平铺段先行判定，命中即返回。
pub(crate) fn span_has_top_level_angle_unit(span: &str) -> bool {
    const TRIG: [&str; 7] = ["sin(", "cos(", "tan(", "asin(", "acos(", "atan(", "atan2("];
    let lower = span.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut seg_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        // 函数名 + '('；名前不得为 ident 字符（`xsin(` 不命中；`asin(`/`atan2(`
        // 起始字符不同，不命中 `sin(`/`tan(` 检测）。
        let hit = TRIG.iter().find(|name| {
            lower[i..].starts_with(*name)
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-' || bytes[i - 1] == b'_'))
        });
        let Some(name) = hit else {
            i += 1;
            continue;
        };
        if span_has_angle_unit(&lower[seg_start..i]) {
            return true;
        }
        // 平衡括号取实参跨度整体屏蔽；不平衡时上层 parse 拒绝，此处不误判。
        let open = i + name.len();
        let mut depth = 1usize;
        let mut j = open + 1;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            return false;
        }
        seg_start = j;
        i = j;
    }
    span_has_angle_unit(&lower[seg_start..])
}

/// 跨度内是否出现产出 `<number>` 的三角函数调用（sin/cos/tan）。
///
/// `<angle>` 槽型别门用：式子含 sin/cos/tan 而无 `<angle>` 产出 provenance
///（反三角/atan2 或 [`span_has_top_level_angle_unit`]）时按度直读违反槽型别
///——`rotate(tan(45deg ))` invalid 案驱动（WPT sin-cos-tan-invalid）。
pub(crate) fn span_has_number_result_trig(span: &str) -> bool {
    const NUMBER_TRIG: [&str; 3] = ["sin(", "cos(", "tan("];
    let lower = span.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for name in NUMBER_TRIG {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(name) {
            let start = from + pos;
            let prev_ok = start == 0
                || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'-' || bytes[start - 1] == b'_');
            if prev_ok {
                return true;
            }
            from = start + name.len();
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
        // 嵌套非反三角内部角度不命中直系叶（WPT computed used 面 valid 形）。
        assert!(!inverse_trig_arg_has_angle_unit("asin(sin(100grad))"));
        assert!(!inverse_trig_arg_has_angle_unit("atan(tan(30deg + 0.261799rad))"));
        assert!(!inverse_trig_arg_has_angle_unit("asin(sin(0.25turn))"));
        // 嵌套反三角的直系叶照样命中（线性扫描深入逐个 gate）。
        assert!(inverse_trig_arg_has_angle_unit("acos(asin(1deg) + 2)"));
        assert!(inverse_trig_arg_has_angle_unit("asin(atan(45deg))"));
    }

    #[test]
    fn span_top_level_angle_unit_masks_trig_args() {
        // 顶层角度叶命中。
        assert!(span_has_top_level_angle_unit("calc(180deg)"));
        assert!(span_has_top_level_angle_unit("calc(45deg + sin(30deg))"));
        assert!(span_has_top_level_angle_unit("calc((45deg + 1))"));
        assert!(span_has_top_level_angle_unit("90grad"));
        assert!(span_has_top_level_angle_unit("CALC(180DEG)"));
        // trig 实参内角度叶屏蔽（§10.5 trig 实参容角度、产出 <number>）。
        assert!(!span_has_top_level_angle_unit("tan(45deg)"));
        assert!(!span_has_top_level_angle_unit("TAN(45DEG)"));
        assert!(!span_has_top_level_angle_unit("sin(cos(45deg))"));
        assert!(!span_has_top_level_angle_unit("calc(sin(30deg) + tan(45deg))"));
        assert!(!span_has_top_level_angle_unit("atan2(30deg, 1rad)"));
        // ident 边界：xsin( 非 trig 调用，内部照扫。
        assert!(span_has_top_level_angle_unit("xsin(45deg)"));
        // 括号不平衡不误判。
        assert!(!span_has_top_level_angle_unit("tan(45deg"));
    }

    #[test]
    fn span_number_result_trig_detection() {
        assert!(span_has_number_result_trig("tan(45deg )"));
        assert!(span_has_number_result_trig("calc(sin(0))"));
        assert!(span_has_number_result_trig("cos(0deg)"));
        // asin/acos/atan/atan2 产出 <angle>，非 number-result trig；
        // "asin(" 不命中 "sin(" 子串（ident 边界）。
        assert!(!span_has_number_result_trig("asin(0.5)"));
        assert!(!span_has_number_result_trig("acos(1)"));
        assert!(!span_has_number_result_trig("atan(1)"));
        assert!(!span_has_number_result_trig("atan2(1, 1)"));
        assert!(!span_has_number_result_trig("xsin(0)"));
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

    #[test]
    fn fold_decl_dispatch_by_property() {
        // WPT serialize 页驱动形态：transform: rotate(...)、scale: cos(0)。
        assert_eq!(
            fold_decl_specified("transform", "rotate(acos(1))").as_deref(),
            Some("rotate(calc(0deg))")
        );
        // 平角度叶往返恒等（WPT serialize 页对期望值双侧同形断言关键——
        // parse 期 Number 化丢单位标记，含单位叶须按弧度→度还原）。
        assert_eq!(
            fold_decl_specified("transform", "rotate(calc(180deg))").as_deref(),
            Some("rotate(calc(180deg))")
        );
        assert_eq!(
            fold_decl_specified("transform", "rotate(calc(90deg))").as_deref(),
            Some("rotate(calc(90deg))")
        );
        assert_eq!(
            fold_decl_specified("rotate", "calc(180deg)").as_deref(),
            Some("calc(180deg)")
        );
        assert_eq!(fold_decl_specified("scale", "cos(0)").as_deref(), Some("calc(1)"));
        assert_eq!(
            fold_decl_specified("rotate", "acos(-1)").as_deref(),
            Some("calc(180deg)")
        );
        // 白名单外属性 / 非数学值 / 空值保留原文。
        assert_eq!(fold_decl_specified("width", "calc(1px + 2px)"), None);
        assert_eq!(fold_decl_specified("transform", "rotate(45deg) scale(2)"), None);
        assert_eq!(fold_decl_specified("scale", "2"), None);
        assert_eq!(fold_decl_specified("scale", ""), None);
    }

    #[test]
    fn fold_transform_wrapper_replacements() {
        // 混合列表：仅折叠 rotate 族跨度，其余原样（大小写保留）。
        assert_eq!(
            fold_transform_math_specified("rotate(acos(1)) scale(2)"),
            Some("rotate(calc(0deg)) scale(2)".into())
        );
        assert_eq!(
            fold_transform_math_specified("Rotate(ASIN(0.5))"),
            Some("Rotate(calc(30deg))".into())
        );
        assert_eq!(
            fold_transform_math_specified("rotate3d(1, 0, 0, acos(1))"),
            Some("rotate3d(1, 0, 0, calc(0deg))".into())
        );
        assert_eq!(
            fold_transform_math_specified("rotate(acos(1)) rotate(acos(0))"),
            Some("rotate(calc(0deg)) rotate(calc(90deg))".into())
        );
        // 无折叠点 / 括号不平衡 → None（保留原文）。
        assert_eq!(fold_transform_math_specified("scale(2) translateX(10px)"), None);
        assert_eq!(fold_transform_math_specified("rotate(acos(1)"), None);
    }
}
