use super::parse_extended::{calc_expr_has_length, calc_expr_is_number};
use super::{eval_calc, parse_math_function, span_has_top_level_angle_unit};

/// `<number>` 文法位的数学函数求值（scale 独立属性/scaleX/matrix/cubic-bezier
/// 系参数共用）。纯数字快路径；数学式按 §10.5 型别门：
/// <length> 叶（含 %——parse_atom 经 parse_length 归 Length）拒绝；trig 实参外
/// 角度叶拒绝（结果 <angle> 非 <number>，`calc(45deg)` 类）；trig 实参内角度叶
/// 放行（§10.5 trig 实参 `<angle>|<number>`，产出 <number>——`scale: cos(0deg)`
/// =1，WPT sin-cos-tan-serialize 210 案驱动；旧 `contains_dimension_token`
/// 文本扫描把实参内 `0deg` 误判顶层维度而拒收）。
pub(super) fn parse_finite_number_math(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if let Ok(number) = trimmed.parse::<f64>() {
        return number.is_finite().then_some(number);
    }
    let expr = parse_math_function(trimmed)?;
    if calc_expr_has_length(&expr) || span_has_top_level_angle_unit(trimmed) || !calc_expr_is_number(&expr) {
        return None;
    }
    let value = eval_calc(&expr, None)?;
    // CSS Values 4 §10.10：NaN 结果合法非 invalid（WPT sin-cos-tan-serialize
    // `scale: calc(sin(infinity))` → calc(NaN) 54 案驱动）；仅 ±Infinity 拒绝
    //（与 parse_angle_or_math 角度槽口径一致）。
    (!value.is_infinite()).then_some(value)
}

pub(super) fn split_top_level_comma_args(input: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ',' if depth == 0 => {
                let part = input[start..idx].trim();
                if part.is_empty() {
                    return None;
                }
                args.push(part);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }
    let last = input[start..].trim();
    if last.is_empty() {
        return None;
    }
    args.push(last);
    Some(args)
}

pub(super) fn split_transform_args(args: &str) -> Option<Vec<&str>> {
    let comma_args = split_top_level_comma_args(args)?;
    if comma_args.len() > 1 {
        let mut parts = Vec::new();
        for arg in comma_args {
            parts.extend(split_top_level_whitespace_args(arg)?);
        }
        return Some(parts);
    }
    split_top_level_whitespace_args(args)
}

fn split_top_level_whitespace_args(input: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = None;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                start.get_or_insert(idx);
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ch if ch.is_whitespace() && depth == 0 => {
                if let Some(begin) = start.take() {
                    let part = input[begin..idx].trim();
                    if !part.is_empty() {
                        args.push(part);
                    }
                }
            }
            _ => {
                start.get_or_insert(idx);
            }
        }
    }

    if depth != 0 {
        return None;
    }
    if let Some(begin) = start {
        let part = input[begin..].trim();
        if !part.is_empty() {
            args.push(part);
        }
    }
    if args.is_empty() { None } else { Some(args) }
}
