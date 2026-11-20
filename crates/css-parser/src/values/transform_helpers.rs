use super::parse_extended::calc_expr_is_number;
use super::{parse_math_function, span_has_top_level_angle_unit};

/// `<number>` 文法位的数学函数求值（scale 独立属性/scaleX/matrix/cubic-bezier
/// 系参数共用）。纯数字快路径；数学式按 §10.5 型别表结果型别门
///（[`calc_expr_is_number`]）：结果非 <number> 拒（`calc(10px)`、`calc(45deg)`、
/// `acos(1px)` 类）；`sign(<length>)` 结果恒 <number> 放行（勿用
/// `calc_expr_has_length` 叶扫描——会误杀实参内 length 叶的合法形，
/// calc-infinity-nan-computed `scale: calc(1 / sign(1em - 16px))` ≥1e6 断言
/// 驱动）；trig 实参内角度叶放行（§10.5 trig 实参 `<angle>|<number>`，
/// `scale: cos(0deg)`=1，sin-cos-tan-serialize 驱动）。
pub(super) fn parse_finite_number_math(value: &str, font_size_px: Option<f64>) -> Option<f64> {
    let trimmed = value.trim();
    if let Ok(number) = trimmed.parse::<f64>() {
        return number.is_finite().then_some(number);
    }
    let expr = parse_math_function(trimmed)?;
    if span_has_top_level_angle_unit(trimmed) || !calc_expr_is_number(&expr) {
        return None;
    }
    // % 叶不在此树级整拒：同型百分比比值树（progress 全 % 族）按 §10.5
    // 型表上下文无关消解为 <number>（[`eval_number_ratio_pct_free`]
    // 基准消去），percentage-without-context
    // `scale(progress(abs(5%), hypot(3%, 4%), 10%))`=0 语料钉面；擦除
    // 原子（sign/abs）内的裸 % 无解析上下文仍拒（mathfn_pins scale 槽
    // `calc(sign(50%) * 180)` should-not-set 钉面），angle/ time 槽的
    // 不可消解形由各自槽级 % 门（parse_angle_or_math 等）承担。
    // https://drafts.csswg.org/css-values-4/#percentage-value
    if super::types::calc_expr_has_pct_under_sign_abs_erasure(&expr) {
        return None;
    }
    let value = super::types::eval_calc_with_font(&expr, font_size_px)?;
    // CSS Values 4 §10.10/§10.6：NaN 与 ±∞ 求值结果均合法非 invalid，computed
    // 期 censor（NaN→0、∞→属性范围钳）由消费面承担。±∞ 放行由
    // calc-infinity-nan-computed `scale: calc(infinity)` / `scale: calc(1 /
    // sign(1em - 16px))` ≥1e6 断言驱动——旧 `!is_infinite` 拒收令 T54 写面
    // 误弃合法声明。消费面自带门的槽不受扰：animation-iteration-count 自门
    // `is_finite() && >= 0`（parse_animation_iteration_count:227）。
    Some(value)
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

pub(super) fn split_top_level_whitespace_args(input: &str) -> Option<Vec<&str>> {
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
