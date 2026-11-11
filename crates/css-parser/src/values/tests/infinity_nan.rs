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

/// clamp 任一实参 NaN → 结果 NaN（§10.6）——IEEE `min`/`max` 链式折叠忽略
/// NaN 会洗毒（旧实现 `clamp(NaN, 0, 10)` 误产 0，computed 期 censor §10.12
/// 不触发）；min>max 退化（回退 MIN）不 panic 语义保留（R3344）。
#[test]
fn test_clamp_nan_poison_propagates() {
    let expr = parse_clamp("clamp(NaN, 0, 10)").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
    let expr = parse_clamp("clamp(0, 10, NaN)").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
    // min>max 退化不 panic。
    let expr = parse_clamp("clamp(100px, 50px, 10px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(100.0));
}

/// round 族 B=±∞（A 有限）策略表（css-values-4 §round-func；round-function/
/// signed-zero 语料钉面）：nearest/to-zero → ±0 签号随 A；up → 正 A 给 +∞、
/// 余 ±0；down → 负 A 给 −∞、余 ±0；B 符号无关；A=±∞ 有限步 → A 原样、
/// 双 ∞ → NaN。IEEE 直算 x*B 在 0×∞ 产 NaN 丢语义——clamp NaN 毒化门落地后
/// 曾暴露为 signed-zero -5 回归（旧 clamp min/max 链洗毒掩盖）。
#[test]
fn test_round_infinite_step_strategy_table() {
    let ev = |s: &str| eval_calc(&parse_math_function(s).unwrap(), None);
    // nearest/to-zero：±0 签号随 A（signed-zero 语料 round(nearest, ±0/±1, ∞) 行）。
    let plus_zero = |v: Option<f64>| matches!(v, Some(v) if v == 0.0 && !v.is_sign_negative());
    let minus_zero = |v: Option<f64>| matches!(v, Some(v) if v == 0.0 && v.is_sign_negative());
    assert!(plus_zero(ev("round(5, infinity)")));
    assert!(plus_zero(ev("round(5, -infinity)")));
    assert!(minus_zero(ev("round(-5, infinity)")));
    assert!(plus_zero(ev("round(nearest, 0, infinity)")));
    assert!(minus_zero(ev("round(nearest, -0, infinity)")));
    assert!(plus_zero(ev("round(nearest, 1, infinity)")));
    assert!(minus_zero(ev("round(nearest, -1, infinity)")));
    assert!(plus_zero(ev("round(to-zero, 5, infinity)")));
    assert!(minus_zero(ev("round(to-zero, -5, -infinity)")));
    // up：正 A → +∞；±0/负 A → ±0 签号随 A。
    assert_eq!(ev("round(up, 1, infinity)"), Some(f64::INFINITY));
    assert!(plus_zero(ev("round(up, 0, infinity)")));
    assert!(minus_zero(ev("round(up, -0, infinity)")));
    assert!(minus_zero(ev("round(up, -1, infinity)")));
    // down：负 A → −∞；±0/正 A → ±0 签号随 A。
    assert_eq!(ev("round(down, -1, infinity)"), Some(f64::NEG_INFINITY));
    assert!(minus_zero(ev("round(down, -0, infinity)")));
    assert!(plus_zero(ev("round(down, 0, infinity)")));
    assert!(plus_zero(ev("round(down, 1, infinity)")));
    // A=±∞ 边界：有限步 → A 原样；双 ∞ → NaN。
    assert_eq!(ev("round(infinity, 5)"), Some(f64::INFINITY));
    assert_eq!(ev("round(-infinity, 5)"), Some(f64::NEG_INFINITY));
    assert!(matches!(ev("round(infinity, infinity)"), Some(v) if v.is_nan()));
    // signed-zero clamp 端到端形（回归驱动案）：clamp(-1, 1/sign(round), 1)。
    let clamp_form = |inner: &str| {
        eval_calc(
            &parse_math_function(&format!("clamp(-1, 1 / sign({inner}), 1)")).unwrap(),
            None,
        )
    };
    // A=+0/正有限 → 1；A=−0/−1 → −1（1/±0 → ±∞、1/±1 → ±1）。
    assert_eq!(clamp_form("round(nearest,  0, infinity)"), Some(1.0));
    assert_eq!(clamp_form("round(nearest,  1, infinity)"), Some(1.0));
    assert_eq!(clamp_form("round(up,  0, infinity)"), Some(1.0));
    assert_eq!(clamp_form("round(down,  0, infinity)"), Some(1.0));
    assert_eq!(clamp_form("round(down,  1, infinity)"), Some(1.0));
    assert_eq!(clamp_form("round(nearest, -1, infinity)"), Some(-1.0));
    assert_eq!(clamp_form("round(nearest, -0, infinity)"), Some(-1.0));
}

/// specified 常量折叠 canonical 形（§10.11/§10.6，calc-infinity-nan-serialize
/// 五页语料驱动）：非有限 × 维度 → `calc(NaN|±infinity * 1<unit>)`；物理单位
/// parse 期归一 px、ms 归一 s；min/max NaN 塌缩树首维度单位；clamp NaN 单位
/// 分歧保结构逐实参规范化；% 维度仅非有限折叠（有限 % 式布局依赖不折叠）；
/// plain 指定值原样保留。
#[test]
fn test_fold_specified_infinity_nan_canonical_forms() {
    use super::super::serialize_math::{MathFoldKind, fold_decl_specified};
    let fold_w = |v| fold_decl_specified("width", v);
    // <length> 槽：NaN/±∞ canonical、物理单位归一 px、% 维度、finite 形。
    assert_eq!(fold_w("calc(1px * NaN)").as_deref(), Some("calc(NaN * 1px)"));
    assert_eq!(fold_w("calc(1in * NaN)").as_deref(), Some("calc(NaN * 1px)"));
    assert_eq!(fold_w("calc(1% * NaN)").as_deref(), Some("calc(NaN * 1%)"));
    assert_eq!(fold_w("calc(1px * infinity)").as_deref(), Some("calc(infinity * 1px)"));
    assert_eq!(
        fold_w("calc(1px * -infinity)").as_deref(),
        Some("calc(-infinity * 1px)")
    );
    assert_eq!(fold_w("calc(1px * 1/infinity)").as_deref(), Some("calc(0px)"));
    assert_eq!(
        fold_w("calc(1px * clamp(-Infinity, 0, infinity))").as_deref(),
        Some("calc(0px)")
    );
    assert_eq!(
        fold_w("calc(1px * clamp(-inFinity, infinity, 10))").as_deref(),
        Some("calc(10px)")
    );
    assert_eq!(
        fold_w("calc(1 * clamp(-INFINITY*20px, 0px, infiniTY*10px))").as_deref(),
        Some("calc(0px)")
    );
    // min/max NaN 塌缩树首维度单位（pt/cm→px；px,em 混型取 px）。
    assert_eq!(
        fold_w("calc(1 * max(INFinity*3px, 0px))").as_deref(),
        Some("calc(infinity * 1px)")
    );
    assert_eq!(
        fold_w("calc(1 * min(nan*3px, 0px))").as_deref(),
        Some("calc(NaN * 1px)")
    );
    assert_eq!(
        fold_w("calc(1 * min(NaN * 1pt, NaN * 1cm))").as_deref(),
        Some("calc(NaN * 1px)")
    );
    assert_eq!(
        fold_w("calc(1 * min(NaN * 2px, NaN * 4em))").as_deref(),
        Some("calc(NaN * 1px)")
    );
    // clamp NaN 单位分歧保结构逐实参规范化；重折叠幂等（写面 + 读回双链）。
    let structured = "calc(1 * clamp(NaN * 2em, NaN * 4px, NaN * 8pt))";
    let folded = fold_w(structured).unwrap();
    assert_eq!(folded, "clamp(NaN * 1em, NaN * 1px, NaN * 1px)");
    assert_eq!(fold_w(&folded).as_deref(), Some(folded.as_str()));
    // plain/布局依赖值不折叠（specified 原样保留）。
    assert_eq!(fold_w("10px"), None);
    assert_eq!(fold_w("calc(100% + 50px)"), None);
    assert_eq!(fold_w("calc(1px + 1em)"), None);
    // <time> 槽：ms 归一 s、plain 时长不折叠。
    let fold_t = |v| fold_decl_specified("animation-duration", v);
    assert_eq!(fold_t("calc(1s * NaN)").as_deref(), Some("calc(NaN * 1s)"));
    assert_eq!(fold_t("calc(1ms * NaN)").as_deref(), Some("calc(NaN * 1s)"));
    assert_eq!(fold_t("calc(1s * 1/infinity)").as_deref(), Some("calc(0s)"));
    assert_eq!(fold_t("calc(1 * min(nan*3s, 0s))").as_deref(), Some("calc(NaN * 1s)"));
    assert_eq!(fold_t("1s"), None);
    assert_eq!(fold_t("2.5s"), None);
    // <number> 槽：裸 canonical 形、clamp NaN 塌缩（单位一致臂）。
    let fold_n = |v| fold_decl_specified("opacity", v);
    assert_eq!(fold_n("calc(1 * max(nAn*2, 0))").as_deref(), Some("calc(NaN)"));
    assert_eq!(fold_n("calc(1 * clamp(0, 10, NaN))").as_deref(), Some("calc(NaN)"));
    assert_eq!(fold_n("5"), None);
    // <angle> 槽（transform wrapper）：±∞ canonical（§10.10 无穷合法非 invalid）。
    assert_eq!(
        fold_decl_specified("transform", "rotate(calc(1deg * NaN))").as_deref(),
        Some("rotate(calc(NaN * 1deg))")
    );
    assert_eq!(
        fold_decl_specified("transform", "rotate(calc(1deg * infinity))").as_deref(),
        Some("rotate(calc(infinity * 1deg))")
    );
    assert_eq!(
        fold_decl_specified("transform", "rotate(calc(1deg * -infinity))").as_deref(),
        Some("rotate(calc(-infinity * 1deg))")
    );
    assert_eq!(
        fold_decl_specified("transform", "rotate(calc(1deg * 1/infinity))").as_deref(),
        Some("rotate(calc(0deg))")
    );
    // MathFoldKind 直连面（ Angle 槽 ±∞）零漂移抽检。
    use super::super::serialize_math::fold_math_specified;
    assert_eq!(
        fold_math_specified("calc(1turn * NaN)", MathFoldKind::Angle).as_deref(),
        Some("calc(NaN * 1deg)")
    );
}

/// min/max 全实参参与：任一实参不可解析（无 ctx 的 %）→ 整体不可解析——
/// 旧 filter_map 静默丢 % 实参（`min(20px, 10%)` 误塌缩 `calc(20px)`），
/// minmax-length-percent-serialize 指定值语料回归钉（g14 轮 -1 捕获）。
#[test]
fn test_min_max_partial_unresolvable_is_none() {
    // 混 % 形：无 ctx 不可解析 → fold None（指定值原样保留）。
    assert_eq!(eval_calc(&parse_min("min(20px, 10%)").unwrap(), None), None);
    assert_eq!(eval_calc(&parse_min("min(10%, 20px)").unwrap(), None), None);
    assert_eq!(eval_calc(&parse_max("max(20px, 10%)").unwrap(), None), None);
    assert_eq!(fold_decl_specified("width", "min(20px, 10%)"), None);
    // 全可解析形零漂移（px-only 塌缩与 NaN 毒化不变）。
    assert_eq!(eval_calc(&parse_min("min(20px, 10px)").unwrap(), None), Some(10.0));
    assert!(matches!(
        eval_calc(&parse_min("min(15px, calc(NaN * 1px))").unwrap(), None),
        Some(v) if v.is_nan()
    ));
}
