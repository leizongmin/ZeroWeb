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
    // tree-counting 叶（DOM 桥求值域）指定值归一化串复用项原文——非零权重
    // 标量不可解析落归一化臂，50/50 组恒等输出（serialize 页
    // 「Parse-time unresolveable value」首行，读出文本与写入原文一致）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 * sibling-index() 50%, 3 50%)"),
        Some("calc-mix(1 * sibling-index() 50%, 3 50%)".into())
    );
}

// ---- tree-counting 叶（CSS Values 5 §tree-counting，tentative）——serialize/
// computed 语料「Parse-time unresolveable value/weight」桶 parse/eval/fold 钉面 ----

#[test]
fn test_tree_counting_parse_and_type_gates() {
    // 无实参合法形（calc 包裹与裸函数、calc-mix 项值/权重位）。
    assert!(parse_math_function("calc(sibling-index())").is_some());
    assert!(parse_math_function("calc(sibling-count() * 10px)").is_some());
    assert!(parse_math_function("sibling-index()").is_some());
    assert!(parse_math_function("sibling-count()").is_some());
    assert!(parse_calc_mix("calc-mix(1 * sibling-index() 50%, 3 50%)").is_some());
    assert!(parse_calc_mix("calc-mix(1 calc(50% * sibling-index()), 3 50%)").is_some());
    // 带实参形草案未定稿（无语料驱动案）——拒。
    assert_eq!(parse_math_function("sibling-index(2)"), None);
    assert_eq!(parse_math_function("sibling-count(odd)"), None);
    // 型别：<integer>（§tree-counting → number 族）——`1 * sibling-index()`
    // 乘法臂 number × number 合法；length 位（<length-percentage>）不含。
    assert!(calc_expr_is_number(
        &parse_math_function("calc(1 * sibling-index())").unwrap()
    ));
    assert!(!calc_expr_is_length_percentage(
        &parse_math_function("calc(sibling-index())").unwrap()
    ));
    // 求值不可达（DOM 兄弟序依赖，CalcContext 无 DOM 视图）。
    assert_eq!(
        eval_calc(&parse_math_function("calc(sibling-index())").unwrap(), None),
        None
    );
}

#[test]
fn test_tree_counting_eval_symbolic_zero() {
    // 符号零（0·v = 0）：0% 权重 × 不可求值项不阻塞整体——serialize 页
    // `calc-mix(1 * sibling-index() 0%, 3 100%)` 行注记「0% so calc-mix()
    // can still be resolved」（权重 100/100 → 3）。
    assert_eq!(
        eval_calc(
            &parse_calc_mix("calc-mix(1 * sibling-index() 0%, 3 100%)").unwrap(),
            None
        ),
        Some(3.0)
    );
    // 全零权重（含 sibling-index() 项）→ 0（computed 页 used 语料同形）。
    assert_eq!(
        eval_calc(
            &parse_calc_mix("calc-mix(1 0%, 3 0%, 5 * sibling-index() 0%)").unwrap(),
            None
        ),
        Some(0.0)
    );
    // 非零权重 × 不可求值项 → 整体不可解析（None）。
    assert_eq!(
        eval_calc(
            &parse_calc_mix("calc-mix(1 * sibling-index() 50%, 3 50%)").unwrap(),
            None
        ),
        None
    );
    // NaN 毒化保持：可求值 NaN 项 × 0 权重仍 NaN（IEEE 传播链不因符号零门丢失）。
    assert!(
        eval_calc(&parse_calc_mix("calc-mix(calc(NaN * 1) 0%, 3 100%)").unwrap(), None).is_some_and(|v| v.is_nan())
    );
}

#[test]
fn test_tree_counting_fold_specified_corpus() {
    // 全零塌缩：单位取首项维度叶（sibling-index() 叶无单位，or_else 溯源
    // `1% *` / `5px *` 的维度叶——serialize 页 unresolved/resolved 组）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 0%, 3 0%, 5 * sibling-index() 0%)"),
        Some("calc(0)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1px 0%, 3px 0%, 5px * sibling-index() 0%)"),
        Some("calc(0px)".into())
    );
    assert_eq!(
        fold_decl_specified("text-indent", "calc-mix(1% 0%, 3% 0%, 5% * sibling-index() 0%)"),
        Some("calc(0%)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1% * sibling-index() 0%, 3px 0%)"),
        Some("calc(0%)".into())
    );
    assert_eq!(
        fold_decl_specified("width", "calc-mix(1px * sibling-index() 0%, 3% 0%)"),
        Some("calc(0px)".into())
    );
    // 归一化串：缺失权重 fill=0 丢弃（5, 7），项原文保形（serialize 页
    // 「Parse-time unresolveable value」组）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 * sibling-index() 50%, 3 50%, 5, 7)"),
        Some("calc-mix(1 * sibling-index() 50%, 3 50%)".into())
    );
    // 符号零标量塌缩 → `calc(3)`（number 位）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 * sibling-index() 0%, 3 100%)"),
        Some("calc(3)".into())
    );
    // Parse-time unresolveable weight：calc() 形权重（含 sibling-index()）→
    // Expr 门原样保留（serialize 页「Parse-time unresolveable weight」组）。
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 calc(50% * sibling-index()), 3 50%)"),
        None
    );
    assert_eq!(
        fold_decl_specified("scale", "calc-mix(1 calc(50% * sibling-index()), 3 50%, 5, 7)"),
        None
    );
    // 嵌套和串：非零权重 Mix 叶不可项化 → 整体放弃、原文透传（serialize 页
    // 「All values use dimension resolving」sibling 行，期望 = 原文）。
    assert_eq!(
        fold_decl_specified(
            "text-indent",
            "calc(10px + calc-mix(10% * sibling-index() 50%, 30% 50%))"
        ),
        None
    );
}
