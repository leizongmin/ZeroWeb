// calc-mix()（CSS Values 5 §calc-mix，tentative）parse/eval/fold 三面钉——
// calc-mix-serialize/computed/invalid 三页语料驱动。文法门（空项/尾逗号/字面
// 权重范围/裸 number 权重/族一致性）、求值公式（clamp/fill/denominator）、
// 指定值折叠（number 塌缩/length 归一化/全零塌缩/Expr 权重原样）。
//
// <https://drafts.csswg.org/css-values-5/#calc-mix>

use super::super::*;

// ---- parse 文法门（invalid 页 Syntax checking 组 + serialize 页合法形）----

#[test]
fn test_calc_mix_parse_syntax_gates() {
    // 合法形（serialize 页 Basic/Missing/Expr 权重组）。
    assert!(parse_calc_mix("calc-mix(1 100%)").is_some());
    assert!(parse_calc_mix("calc-mix(1)").is_some());
    assert!(parse_calc_mix("calc-mix(1, 3)").is_some());
    assert!(parse_calc_mix("calc-mix(1 50%, 3 50%)").is_some());
    assert!(parse_calc_mix("calc-mix(1 calc(150%), 3 100%)").is_some());
    assert!(parse_calc_mix("calc-mix(10px 75%, 3em 75%)").is_some());
    // 外层 calc 嵌套与函数实参位（parse_factor 嵌套臂 + depth 守卫）。
    assert!(parse_math_function("calc(10px + calc-mix(1% 0%, 3px 0%))").is_some());
    assert!(parse_math_function("calc(sign(calc-mix(1 50%, 3 50%)) + 10)").is_some());
    assert!(parse_math_function("calc(max(calc-mix(1 50%, 3 50%), 1))").is_some());

    // 语法拒形（invalid 页 Syntax checking 组）。
    assert_eq!(parse_calc_mix("calc-mix()"), None);
    assert_eq!(parse_calc_mix("calc-mix( )"), None);
    assert_eq!(parse_calc_mix("calc-mix(,)"), None);
    assert_eq!(parse_calc_mix("calc-mix(1, )"), None);
    assert_eq!(parse_calc_mix("calc-mix(0,1,)"), None);
    assert_eq!(parse_calc_mix("calc-mix(0,, 0)"), None);
    // 裸 number 权重（无 % 叶）拒。
    assert_eq!(parse_calc_mix("calc-mix(1 50, 3)"), None);
    // 字面权重 <percentage [0,100]> 范围门（calc() 形不受此门，求值期 clamp）。
    assert_eq!(parse_calc_mix("calc-mix(1 150%, 3)"), None);
    assert_eq!(parse_calc_mix("calc-mix(1 -50%, 3)"), None);
}

// ---- parse 族一致性门（§10.5 同族）+ 槽位型别门 ----

#[test]
fn test_calc_mix_parse_type_gates() {
    // 混族（% + number）parse 期拒（invalid 页 Type checking 组）。
    assert_eq!(parse_calc_mix("calc-mix(1% 50%, 3 50%)"), None);
    // 同族合法形放行（length-percentage / angle 族）。
    assert!(parse_calc_mix("calc-mix(1px 50%, 3px 50%)").is_some());
    assert!(parse_calc_mix("calc-mix(1deg 50%, 3turn 50%)").is_some());
    // number 位（scale）：length/angle 族拒、number 族放行。
    assert!(parse_individual_scale("calc-mix(1 50%, 3 50%)").is_some());
    assert_eq!(parse_individual_scale("calc-mix(1px 50%, 3px 50%)"), None);
    assert_eq!(parse_individual_scale("calc-mix(1deg 50%, 3turn 50%)"), None);
    // 语法拒形在 number 位同拒。
    assert_eq!(parse_individual_scale("calc-mix(1 150%, 3)"), None);
    assert_eq!(parse_individual_scale("calc-mix()"), None);
}

// ---- eval 求值公式（clamp/fill/denominator）——serialize/computed 页期望值 ----

#[test]
fn test_calc_mix_eval_formula() {
    // Σ=100%：直接加权平均。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 50%, 3 50%)").unwrap(), None),
        Some(2.0)
    );
    assert_eq!(eval_calc(&parse_calc_mix("calc-mix(1 100%)").unwrap(), None), Some(1.0));
    // Σ<100%：分母 max(Σ,100) 稀释不归一（25% 组 → 1）。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 25%, 3 25%)").unwrap(), None),
        Some(1.0)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 25%, 3 25%, 5 50%)").unwrap(), None),
        Some(3.5)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 25%, 3 25%, 7 0%, 5 50%)").unwrap(), None),
        Some(3.5)
    );
    // Σ>100%：除以 Σ 归一（75% 组 → 2）。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 75%, 3 75%)").unwrap(), None),
        Some(2.0)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 50%, 3 50%, 5 50%)").unwrap(), None),
        Some(3.0)
    );
    // calc() 形权重求值期 clamp：150%→100、-50%→0。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 calc(150%), 3 100%)").unwrap(), None),
        Some(2.0)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 calc(-50%), 3 100%)").unwrap(), None),
        Some(3.0)
    );
    // 缺失权重均分 max(0, 100-Σ)。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 50%, 3)").unwrap(), None),
        Some(2.0)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1, 3, 5 50%)").unwrap(), None),
        Some(3.5)
    );
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 25%, 3 25%, 5, 7)").unwrap(), None),
        Some(4.0)
    );
    // Σ>100% 时缺失项 fill=0（剩余为负 clamp 到 0）。
    assert_eq!(
        eval_calc(&parse_calc_mix("calc-mix(1 75%, 3 75%, 5, 7)").unwrap(), None),
        Some(2.0)
    );
    // 全零权重 → 0。
    assert_eq!(eval_calc(&parse_calc_mix("calc-mix(1 0%)").unwrap(), None), Some(0.0));
    // NaN 毒化自然传播（§10.6，IEEE 传播链）。
    assert!(
        eval_calc(&parse_calc_mix("calc-mix(1 50%, calc(NaN * 1) 50%)").unwrap(), None).is_some_and(|v| v.is_nan())
    );
    // 外层函数实参位（serialize 页 Nested 组：sign(mix)+10 → 11）。
    assert_eq!(
        eval_calc(
            &parse_math_function("calc(sign(calc-mix(1 50%, 3 50%)) + 10)").unwrap(),
            None
        ),
        Some(11.0)
    );
}

// ---- 指定值折叠：number 族塌缩（scale 槽）----

#[test]
fn test_calc_mix_fold_number_collapse() {
    // serialize 页 Basic 组。
    assert_eq!(fold_decl_specified("scale", "calc-mix(1 100%)"), Some("calc(1)".into()));
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 50%, 3 50%)"),
        Some("calc(2)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 25%, 3 25%, 5 50%)"),
        Some("calc(3.5)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 25%, 3 25%, 7 0%, 5 50%)"),
        Some("calc(3.5)".into())
    );
    // Missing weight percentages 组。
    assert_eq!(fold_decl_specified("scale", "calc-mix(1)"), Some("calc(1)".into()));
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 50%, 3)"),
        Some("calc(2)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1, 3, 5 50%)"),
        Some("calc(3.5)".into())
    );
    // more than 100% 组。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 75%, 3 75%)"),
        Some("calc(2)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 50%, 3 50%, 5 50%)"),
        Some("calc(3)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 75%, 3 75%, 5, 7)"),
        Some("calc(2)".into())
    );
    // less than 100% 组（不归一，稀释求值）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 25%, 3 25%)"),
        Some("calc(1)".into())
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 25%, 3 25%, 5, 7)"),
        Some("calc(4)".into())
    );
    // 全零权重 → calc(0)。
    assert_eq!(fold_decl_specified("scale", "calc-mix(1 0%)"), Some("calc(0)".into()));
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 0%, 3 0%, 5 0%)"),
        Some("calc(0)".into())
    );
}

// ---- 指定值折叠：Expr 权重原样保留（Chrome/Blink 口径：指定值期不折叠）----

#[test]
fn test_calc_mix_fold_expr_weight_passthrough() {
    assert_eq!(fold_decl_specified("scale", "calc-mix(1 calc(150%), 3 100%)"), None);
    assert_eq!(fold_decl_specified("scale", "calc-mix(1 calc(-50%), 3 100%)"), None);
}

// ---- 指定值折叠：length 族归一化串（text-indent 槽，值项复用原文）----

#[test]
fn test_calc_mix_fold_length_normalized() {
    // more than 100%：全项除以 Σ、缺失项 fill>0 保留/fill=0 丢弃。
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(10px 75%, 3em 75%)"),
        Some("calc-mix(10px 50%, 3em 50%)".into())
    );
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(10px 50%, 3em 50%, 5em 50%, 7em 50%)"),
        Some("calc-mix(10px 25%, 3em 25%, 5em 25%, 7em 25%)".into())
    );
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(10px 75%, 3em 75%, 5em, 7em)"),
        Some("calc-mix(10px 50%, 3em 50%)".into())
    );
    // less than 100%：Σ<100 且无缺失 → 原权重保持。
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(10px 25%, 3em 25%)"),
        Some("calc-mix(10px 25%, 3em 25%)".into())
    );
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(10px 25%, 3em 25%, 5em, 7em)"),
        Some("calc-mix(10px 25%, 3em 25%, 5em 25%, 7em 25%)".into())
    );
    // 布局依赖项（em）原文保留单位原貌；number 族混入 length 槽不折叠
    //（dimension_suffix 门 → 写面型别门拒）。
    assert_eq!(fold_decl_specified("text-indent", "calc-mix(1 50%, 3 50%)"), None);
}

// ---- 指定值折叠：全零塌缩（首项单位）----

#[test]
fn test_calc_mix_fold_all_zero_collapse() {
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1px 0%)"),
        Some("calc(0px)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1px 0%, 3px 0%, 5px 0%)"),
        Some("calc(0px)".into())
    );
    // % 项与 %/px 混族：塌缩单位取首项（语料 FIXME 注记 Chrome 同款首项规则）。
    assert_eq!(fold_decl_specified("width", "calc-mix(1% 0%)"), Some("calc(0%)".into()));
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1% 0%, 3% 0%, 5% 0%)"),
        Some("calc(0%)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1% 0%, 3px 0%)"),
        Some("calc(0%)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1px 0%, 3% 0%)"),
        Some("calc(0px)".into())
    );
}

// ---- 嵌套和串钉（外层 calc 树递归项化 + 单位排序，serialize_math
// `fold_mix_nest_sum`——serialize 语料「外层 calc 嵌套」桶指定值 10 面）----

#[test]
fn test_calc_mix_fold_outer_calc_nested_canonical_sum() {
    // CSS Values 4 §10.9 指定值 canonical 和串：Mix 叶塌缩项化 + 同单位
    // 常量合并 + % 项前置其余保序（语料期望五形逐字钉面）。
    assert_eq!(
        fold_decl_specified("width", "calc(10px + calc-mix(1% 0%, 3px 0%))").as_deref(),
        Some("calc(0% + 10px)")
    );
    assert_eq!(
        fold_decl_specified("width", "calc(10% + calc-mix(1% 0%, 3px 0%))").as_deref(),
        Some("calc(10%)")
    );
    assert_eq!(
        fold_decl_specified("width", "calc(10px + calc-mix(1px 0%, 3% 0%))").as_deref(),
        Some("calc(10px)")
    );
    assert_eq!(
        fold_decl_specified("width", "calc(10% + calc-mix(1px 0%, 3% 0%))").as_deref(),
        Some("calc(10% + 0px)")
    );
    assert_eq!(
        fold_decl_specified("width", "calc(10px + calc-mix(10% 50%, 30% 50%))").as_deref(),
        Some("calc(20% + 10px)")
    );
}

#[test]
fn test_calc_mix_fold_outer_calc_nested_guards() {
    // 挂账保持：sibling-index() 形（DOM 桥求值域）不项化——原文透传。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 * sibling-index() 50%, 3 50%)"),
        None
    );
}
