// ∞/NaN 数学求值管线钉（css-values-4 §10.6 Infinities and NaNs、§10.12 computed
// 期 censor）——calc-infinity-nan-computed 语料桶驱动：NaN 任一环节丢毒/写面误拒
// 都会在 computed 序列化面翻车（"NaNpx"/写面弃声明）。

use super::super::*;

/// min/max 任一实参 NaN → 结果 NaN（§10.6）——IEEE `f64::min`/`max` 忽略 NaN
/// 会丢毒（旧实现 `max(15px, NaN)` 误产 15px，computed censor 不触发）。
#[test]
fn test_min_max_nan_poison_propagates() {
    // NaN≠NaN：断言走 is_nan 判定，不可 assert_eq 直比。
    let expr = parse_max("max(15px, calc(NaN * 1px))").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
    let expr = parse_min("min(15px, calc(NaN * 1px))").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
    // 非 NaN 实参路径零漂移。
    let expr = parse_max("max(10px, 20px, 15px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(20.0));
}

/// parse_time 数学形：NaN/±∞ 求值结果合法（§10.10/§10.6），computed 期 censor
/// 由消费面承担——旧 `is_finite` 门令 T54 写面误弃合法声明。
#[test]
fn test_parse_time_math_accepts_nan_and_infinity() {
    assert!(parse_time("calc(NaN * 1s)").map(|v| v.is_nan()).unwrap_or(false));
    assert!(
        parse_time("calc(infinity * 1s)")
            .map(|v| v.is_infinite() && v > 0.0)
            .unwrap_or(false)
    );
    assert!(
        parse_time("calc(1s / sign(1em - 16px))")
            .map(|v| v.is_infinite())
            .unwrap_or(false)
    );
    // 既有 IACVT/混型拒收零漂移。
    assert_eq!(parse_time("min(1s, 0)"), None);
    assert_eq!(parse_time("NaNs"), None);
    assert_eq!(parse_time("infs"), None);
}

/// animation-duration 数学形槽位（g11 批挂账兑现）：合法数学形放行、
/// NaN/±∞ 放行（computed 期 censor）、裸 Number/Length 混型叶与 plain 负值/
/// `infinite` 关键字仍拒。
#[test]
fn test_parse_animation_duration_math_forms() {
    assert!(parse_animation_duration("calc(1s / 3)").is_some());
    assert!(parse_animation_duration("min(1s, 2s)").is_some());
    assert!(
        parse_animation_duration("calc(NaN * 1s)")
            .is_some_and(|v| matches!(v, AnimationDurationValue::Time(t, TimeUnit::S) if t.is_nan()))
    );
    // 裸 number 叶（§10.5 混型）与 plain 形旧钉零漂移。
    assert_eq!(parse_animation_duration("min(1s, 0)"), None);
    assert_eq!(parse_animation_duration("calc(1)"), None);
    assert_eq!(parse_animation_duration("-1s"), None);
    assert_eq!(parse_animation_duration("infinite"), None);
    assert_eq!(parse_animation_duration("1s 2s"), None);
}

/// <number> 文法位数学形 ±∞ 放行（scale 槽 ≥1e6 断言面）；NaN 既有放行；
/// <length>/<angle> 叶型别门零漂移；iteration-count 自门 `is_finite() && >= 0`
/// 不受扰。
#[test]
fn test_parse_finite_number_math_accepts_infinity() {
    assert!(parse_individual_scale("calc(infinity)").is_some());
    assert!(parse_individual_scale("calc(1 / sign(1em - 16px))").is_some());
    // NaN 既有放行（sin-cos-tan-serialize 桶）。
    assert!(parse_individual_scale("calc(sin(infinity))").is_some());
    // 型别门零漂移。
    assert_eq!(parse_individual_scale("calc(10px)"), None);
    assert_eq!(parse_individual_scale("calc(45deg)"), None);
    // iteration-count 槽 ∞ 仍拒（关键字除外）。
    assert!(matches!(
        parse_animation_iteration_count("infinite"),
        Some(AnimationIterationCountValue::Infinite)
    ));
    assert_eq!(parse_animation_iteration_count("calc(infinity)"), None);
}
