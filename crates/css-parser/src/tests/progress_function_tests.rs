//! progress() 三实参数学函数钉面（CSS Values 5 §progress，tentative）。
//!
//! parse 文法门（三实参顶层逗号 + `no-clamp` 错位/`from`/`to`/混型/未闭合
//! invalid 矩阵）、型别表（恒 <number>）、求值（clamp [0,1] / S==E 简并
//! 0/±∞ / NaN 毒化 / % 族符号基准重试）、specified 折叠串化（可求值塌缩
//! `calc(<number>)`、不可求值 canonical 保形含 no-clamp）。
//!
//! driving: wptfyi-css DC-2 g33（progress-serialize/-invalid/-computed 语料桶）。

use crate::values::{
    CalcContext, CalcMathType, TransformFunction, calc_expr_has_length, calc_expr_is_number, calc_expr_math_type,
    eval_calc, eval_calc_with_context, fold_decl_specified, fold_math_specified, parse_individual_rotate,
    parse_math_function, parse_transform,
};

#[test]
fn progress_parse_valid_matrix() {
    // 三实参同族（px）。
    assert!(parse_math_function("progress(100px, 0px, 100px)").is_some());
    // no-clamp 前置关键字。
    assert!(parse_math_function("progress(no-clamp 100px, 0px, 100px)").is_some());
    // % 与维度同 length-percentage 族可混（§progress 实参 consistent type）。
    assert!(parse_math_function("progress(50%, 0px, 100px)").is_some());
    // 嵌套函数实参。
    assert!(parse_math_function("progress(abs(10%), (10% - 10%), 100% / 10)").is_some());
    // calc 包裹。
    assert!(parse_math_function("calc(0.5 * progress(100px, 0px, 100px))").is_some());
    // 角度族。
    assert!(parse_math_function("progress(2rad, 1rad, 1rad)").is_some());
    // 纯 number 族。
    assert!(parse_math_function("progress(1, 0, 1)").is_some());
}

#[test]
fn progress_parse_invalid_matrix() {
    // 空参/单参/双参/尾逗号。
    for v in [
        "progress()",
        "progress( )",
        "progress(,)",
        "progress(1, )",
        "progress(1)",
        "progress(0, 1,)",
        "progress(0,, 0)",
        "progress(0,,, 10,, 200)",
    ] {
        assert!(parse_math_function(v).is_none(), "should reject {v}");
    }
    // from/to 非关键字与错位形。
    for v in [
        "progress(from,)",
        "progress(from 1, 0)",
        "progress(from 1, 0 1)",
        "progress(from 1 0)",
        "progress(0 from 10 to 200)",
    ] {
        assert!(parse_math_function(v).is_none(), "should reject {v}");
    }
    // no-clamp 错位（缺失实参/非首实参位/紧贴逗号）。
    for v in [
        "progress(no-clamp)",
        "progress(no-clamp 1)",
        "progress(no-clamp 1, 0)",
        "progress(no-clamp 1 0 1)",
        "progress(no-clamp 1, 0 1)",
        "progress(no-clamp, 1, 0 1)",
        "progress(1 no-clamp, 0, 1)",
        "progress(1, 0, 1 no-clamp)",
    ] {
        assert!(parse_math_function(v).is_none(), "should reject {v}");
    }
    // 混型（number×angle、number×length、angle×number）与未闭合。
    for v in [
        "progress(5, 0deg, 8deg)",
        "progress(5, 0%, 8deg)",
        "progress(10deg, 0, 10)",
        "progress(5%, 0, 8",
        "progress(5, 0%, sign(10px)",
    ] {
        assert!(parse_math_function(v).is_none(), "should reject {v}");
    }
    // 混型拒收经 calc 包裹同样无效（letter-spacing 语料 `calc(1px *
    // progress(10deg, 0, 10))` 钉面）。
    assert!(parse_math_function("calc(1px * progress(10deg, 0, 10))").is_none());
}

#[test]
fn progress_type_table_and_predicates() {
    let expr = parse_math_function("progress(100px, 0px, 100px)").unwrap();
    // 结果恒 <number>（§progress）。
    assert_eq!(calc_expr_math_type(&expr), CalcMathType::Number);
    assert!(calc_expr_is_number(&expr));
    // 实参维度函数内消解不外泄——树级 length 性只看节点外叶子。
    assert!(!calc_expr_has_length(&expr));
    let wrapped = parse_math_function("calc(progress(100px, 0px, 100px) * 1px)").unwrap();
    assert!(calc_expr_has_length(&wrapped));
    // 角度族 progress 同样产出 number。
    let expr = parse_math_function("progress(2rad, 1rad, 1rad)").unwrap();
    assert_eq!(calc_expr_math_type(&expr), CalcMathType::Number);
}

#[test]
fn progress_eval_clamp_and_degenerate() {
    // 常规插值：(100-0)/(100-0) = 1。
    let expr = parse_math_function("progress(100px, 0px, 100px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(1.0));
    // clamp [0,1]：V 越界截断。
    let expr = parse_math_function("progress(200px, 0px, 100px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(1.0));
    let expr = parse_math_function("progress(-100px, 0px, 100px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
    // no-clamp 越界保留。
    let expr = parse_math_function("progress(no-clamp 200px, 0px, 100px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(2.0));
    // S==E 简并：clamped 恒 0；no-clamp 按比较 0/−∞/+∞。
    let expr = parse_math_function("progress(2rad, 1rad, 1rad)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
    let expr = parse_math_function("progress(no-clamp 2rad, 1rad, 1rad)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(f64::INFINITY));
    let expr = parse_math_function("progress(no-clamp 0rad, 1rad, 1rad)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(f64::NEG_INFINITY));
    let expr = parse_math_function("progress(no-clamp 1rad, 1rad, 1rad)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
    // NaN 任一实参毒化（§10.12）。
    let expr = parse_math_function("progress(calc(NaN * 1px), 0px, 100px)").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
}

#[test]
fn progress_eval_partial_se_arm() {
    // V 不可求值（em 无字体基准）而 S/E 可求值：部分求值臂——S==E clamped
    // 简并恒 0（不依赖 V）；no-clamp 需 V 参与比较则 None。
    let expr = parse_math_function("progress(10em, 0px, 0px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
    let expr = parse_math_function("progress(no-clamp 10em, 0px, 0px)").unwrap();
    assert_eq!(eval_calc(&expr, None), None);
    // cqw 无容器基准同臂（progress-computed 语料 `progress(0cqw, 0px, 0px)`）。
    let expr = parse_math_function("progress(0cqw, 0px, 0px)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
    // S/E 含 NaN 毒化（§10.12 在部分求值臂同样生效）。
    let expr = parse_math_function("progress(10em, calc(NaN * 1px), 0px)").unwrap();
    assert!(matches!(eval_calc(&expr, None), Some(v) if v.is_nan()));
}

#[test]
fn progress_eval_nested() {
    // 嵌套 progress / sign 嵌套（progress-computed 语料钉面）：
    // V=progress(1,0,1)=1、S=progress(0px,0px,1px)=0、
    // E=progress(1deg,0deg,1deg)=1 → (1-0)/(1-0) = 1。
    let expr = parse_math_function("progress(progress(1, 0, 1), progress(0px, 0px, 1px), progress(1deg, 0deg, 1deg))")
        .unwrap();
    assert_eq!(eval_calc(&expr, None), Some(1.0));
    // V=sign(-10px)*10px=-10、S=(10px-10px)=0、E=10px*progress(...)=10 →
    // (-10-0)/(10-0) = -1 → clamp [0,1] → 0。
    let expr =
        parse_math_function("progress(sign(-10px) * 10px, (10px - 10px), 10px * progress(1deg, 0deg, 1deg))").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.0));
}

#[test]
fn progress_eval_percent_family_symbolic() {
    // % 族无基准符号解析（比值与基准无关）：(1-0)/(100-0) = 0.01。
    let expr = parse_math_function("progress(1%, (10% - 10%), 100%)").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(0.01));
    // 混族（% 与 px）无基准 fail-closed——% 需真实基准。
    let expr = parse_math_function("progress(50%, 0px, 100px)").unwrap();
    assert_eq!(eval_calc(&expr, None), None);
    // 有基准（width computed ctx parent_length=100）：0.5。
    let ctx = CalcContext {
        parent_length: Some(100.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(0.5));
    // em 族需字体基准；无 ctx None，有 ctx 按基准求值（10px 字号：
    // (5-1)/(11-1) = 0.4）。
    let expr = parse_math_function("progress(50em, 10em, 110em)").unwrap();
    assert_eq!(eval_calc(&expr, None), None);
    let ctx = CalcContext {
        font_size: Some(10.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(0.4));
}

#[test]
fn progress_rotate3d_transform_slots() {
    // transform rotate3d 槽数学函数文法位（progress-computed 语料钉面）：
    // 三 <number> 分量位经 parse_transform_number（裸 f64::parse 使 progress
    // 形整体无效）、<angle> 位经 parse_angle_or_math（eval 与 number 槽同走
    // default-font 基准，calc 包裹 progress 角度形求值 2deg）。
    // progress(21em, 1rem, 11em)：(21e−r)/(11e−r) 分子−分母 = 10e > 0，
    // 任意正字号基准下 ratio 恒 >1 → 默认 clamp [0,1] → 分量 1（css-values-5
    // "if omitted, the return value is clamped to the range [0, 1]"，Issue
    // 11825 澄清）。语料 rotate3d 案期望 `rotate3d(2, 2, 2, 2deg)` 与同页
    // `progress(1000em, 10em, 110em)` → '1' 自相矛盾，系 11825 之前陈旧期望，
    // 挂账不追；本钉面对齐规范 clamp 语义。
    let value = "rotate3d(progress(21em, 1rem, 11em), progress(21em, 1rem, 11em), progress(21em, 1rem, 11em), calc(progress(11em, 1rem, 11em) * 2deg))";
    assert!(parse_transform(value).is_some(), "rotate3d progress 全形应解析");
    // individual rotate 四元组臂同口径（rotate: <number>{3} <angle>）；
    // 分词器只认括号外逗号/空白——depth-0 盲拆把实参表撕成垃圾 token。
    let parsed = parse_individual_rotate(
        "progress(21em, 1rem, 11em) progress(21em, 1rem, 11em) progress(21em, 1rem, 11em) calc(progress(11em, 1rem, 11em) * 2deg)",
    );
    assert!(
        matches!(parsed, Some(TransformFunction::Rotate3d(x, y, z, a)) if x == 1.0 && y == 1.0 && z == 1.0 && (a - 2.0).abs() < 1e-9),
        "四元组臂实际 {:?}",
        parsed
    );
}

#[test]
fn progress_fold_specified_serialize() {
    // 可求值塌缩 `calc(<number>)`（opacity = <number> 槽）。
    assert_eq!(
        fold_decl_specified("opacity", "progress(100px, 0px, 100px)"),
        Some("calc(1)".to_string())
    );
    assert_eq!(
        fold_decl_specified("opacity", "progress(1%, (10% - 10%), 100%)"),
        Some("calc(0.01)".to_string())
    );
    // 外层 calc 内 progress 折叠。
    assert_eq!(
        fold_decl_specified("opacity", "calc(0.5 * progress(100px, 0px, 100px))"),
        Some("calc(0.5)".to_string())
    );
    assert_eq!(
        fold_decl_specified("opacity", "calc(0.5 * progress(-100px, 0px, 100px))"),
        Some("calc(0)".to_string())
    );
    // no-clamp 简并 ±∞ 发射。
    assert_eq!(
        fold_decl_specified("opacity", "progress(no-clamp 2rad, 1rad, 1rad)"),
        Some("calc(infinity)".to_string())
    );
    assert_eq!(
        fold_decl_specified("opacity", "progress(no-clamp 0rad, 1rad, 1rad)"),
        Some("calc(-infinity)".to_string())
    );
    // clamp 简并 0 发射。
    assert_eq!(
        fold_decl_specified("opacity", "progress(2rad, 1rad, 1rad)"),
        Some("calc(0)".to_string())
    );
    // 不可求值（em 无基准）canonical 保形（width = <length-percentage> 槽，
    // progress number 型树不过尺寸槽折叠门——原样保留）。
    assert_eq!(fold_decl_specified("width", "progress(10em, 0px, 10em)"), None);
    assert_eq!(fold_decl_specified("width", "progress(no-clamp 10em, 0px, 10em)"), None);
    // 尺寸槽外层算式折叠：50px * 1 → calc(50px)。
    assert_eq!(
        fold_decl_specified("width", "calc(50px * progress(100px, 0px, 100px))"),
        Some("calc(50px)".to_string())
    );
    // scale wrapper（transform 槽）。
    assert_eq!(
        fold_decl_specified("transform", "scale(calc(0.5 * progress(100px, 0px, 100px)))"),
        Some("scale(calc(0.5))".to_string())
    );
    // invalid 形不折叠（原样保留交写面拒收）。
    assert_eq!(fold_decl_specified("opacity", "progress(5, 0%, 8deg)"), None);
    assert_eq!(fold_decl_specified("opacity", "progress()"), None);
}
