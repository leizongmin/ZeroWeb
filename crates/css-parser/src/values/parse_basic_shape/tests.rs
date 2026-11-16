//! basic-shape 文法族单测——钉面全部取自上游语料（2026-11-16 盘点）：
//! css-shapes/shape-functions/{shape,rect}-function-{valid,invalid}.html、
//! css-masking/parsing/clip-path-shape-parsing.html、
//! css-masking/parsing/clip-path-{valid,invalid}.html、
//! css-masking/clip-path/clip-path-xywh-00{1,2,3}.html 等语料形。
//! 声明级钉面：computed 绝对化（关键字→%、em→px、path 相对段→绝对）归
//! engine 层（Stage B），此处只钉 parse + 声明级 canonical 串化。

use super::*;

fn shape(v: &str) -> Option<String> {
    super::super::parse_extended_border::parse_clip_path(v)
        .as_ref()
        .and_then(clip_path_value_to_css)
}

// ── clip-path-shape-parsing.html 声明级钉面 ─────────────────────────────

#[test]
fn shape_parsing_basic_and_fill_rule() {
    assert_eq!(
        shape("shape(from 0px 0px, line to 10px 10px)").unwrap(),
        "shape(from 0px 0px, line to 10px 10px)"
    );
    assert_eq!(
        shape("shape(evenodd from 0px 0px, line to 10px 10px)").unwrap(),
        "shape(evenodd from 0px 0px, line to 10px 10px)"
    );
    // nonzero 缺省串化省略
    assert_eq!(
        shape("shape(nonzero from 0px 0px, line to 10px 10px)").unwrap(),
        "shape(from 0px 0px, line to 10px 10px)"
    );
    // 段内空白归一
    assert_eq!(
        shape("shape(  from 0px    0px, line  to 10px  10px     )").unwrap(),
        "shape(from 0px 0px, line to 10px 10px)"
    );
    // 大小写归一 + fill-rule 保形 + 裸数字补 px
    assert_eq!(
        shape("shape(EvenOdd from 0Px, CLOSE)").unwrap(),
        "shape(evenodd from 0px 0px, close)"
    );
    // 单位保形
    assert_eq!(
        shape("shape(from 1ch 50px, line to 10rem 10vh)").unwrap(),
        "shape(from 1ch 50px, line to 10rem 10vh)"
    );
    assert_eq!(
        shape("shape(from 1ch -50px, line to -10% 12px)").unwrap(),
        "shape(from 1ch -50px, line to -10% 12px)"
    );
}

#[test]
fn shape_parsing_command_inventory() {
    assert_eq!(
        shape("shape(from 10px 10px, move by 10px 5px, line by 20px 40%, close)").unwrap(),
        "shape(from 10px 10px, move by 10px 5px, line by 20px 40%, close)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, hline by 10px, vline to 5rem)").unwrap(),
        "shape(from 10px 10px, hline by 10px, vline to 5rem)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, vline by 5%, hline to 1vw)").unwrap(),
        "shape(from 10px 10px, vline by 5%, hline to 1vw)"
    );
    // pt 解析期归一 px（types.rs parse_length 共享行为；单位保形 pt/pc
    // 需 LengthValue 增变体——FIXME Stage B 决策）
    assert_eq!(
        shape("shape(from 10px 10px, smooth to 50px 1pt)").unwrap(),
        "shape(from 10px 10px, smooth to 50px 1.333333px)"
    );
    assert_eq!(
        shape("shape(evenodd from 0px 0px, close)").unwrap(),
        "shape(evenodd from 0px 0px, close)"
    );
}

#[test]
fn shape_parsing_curve_control_points() {
    assert_eq!(
        shape("shape(from 10px 10px, curve to 50px 20px with 10rem 1%)").unwrap(),
        "shape(from 10px 10px, curve to 50px 20px with 10rem 1%)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, curve to 50px 20px with 10rem 1px / 20vh 1ch)").unwrap(),
        "shape(from 10px 10px, curve to 50px 20px with 10rem 1px / 20vh 1ch)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, curve by 50px 20px with 10rem 1px / 20vh 1ch)").unwrap(),
        "shape(from 10px 10px, curve by 50px 20px with 10rem 1px / 20vh 1ch)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, smooth to 50px 20px with 10rem 1%)").unwrap(),
        "shape(from 10px 10px, smooth to 50px 20px with 10rem 1%)"
    );
}

#[test]
fn shape_parsing_arc_defaults_omitted() {
    assert_eq!(
        shape("shape(from 10px 10px, arc to 50px 1pt of 10px 10px)").unwrap(),
        "shape(from 10px 10px, arc to 50px 1.333333px of 10px 10px)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, arc to 50px 1pt of 10%)").unwrap(),
        "shape(from 10px 10px, arc to 50px 1.333333px of 10%)"
    );
    // small + rotate 0 缺省省略
    assert_eq!(
        shape("shape(from 10px 10px, arc to 50px 1pt of 10px 10px small rotate 0deg)").unwrap(),
        "shape(from 10px 10px, arc to 50px 1.333333px of 10px 10px)"
    );
    assert_eq!(
        shape("shape(from 10px 10px, arc to 50px 1pt of 10px small rotate 0deg)").unwrap(),
        "shape(from 10px 10px, arc to 50px 1.333333px of 10px)"
    );
    // 显式修饰 canonical 序保形
    assert_eq!(
        shape("shape(from 10% 1rem, arc to 50px 1pt of 20% cw large rotate 25deg)").unwrap(),
        "shape(from 10% 1rem, arc to 50px 1.333333px of 20% cw large rotate 25deg)"
    );
}

#[test]
fn shape_parsing_arc_wrong_order_invalid() {
    // 语料「arc with wrong order」组：端点后置 / 修饰词前移皆拒
    assert!(shape("shape(from 10% 1rem, arc of 20% 1pt to 50px cw large rotate 25deg)").is_none());
    assert!(shape("shape(from 10% 1rem, arc cw to 50px of 20% 1pt large rotate 25deg)").is_none());
    assert!(shape("shape(from 10% 1rem, arc rotate 25deg to 50px 1pt of 20% cw large )").is_none());
}

#[test]
fn shape_parsing_invalid_matrix() {
    for v in [
        // nonsense
        "shape(evenodd from 0px 0px, close path)",
        "shape(nonzero, from 0px 0px, line to 10px 10px)",
        "shape(from 10px 10px, curve to 50px 20px via 10rem)",
        "shape(from 10px 10px, curve to 50px 20px via 10rem 1% 12px)",
        "shape(from 10px 10px, hline byy 10px, vline to 5rem)",
        "shape(from 10px 10px, vline by 5% hline by 1vw",
        "shape(from 10px 10px, smooth to 50px 20px via 10rem)",
        "shape(from 10px 10px, smooth to 50px 20px via 10rem 2px 2pt)",
        "shape()",
        "shape(from)",
        "shape(from 0px)",
        "shape(from 0px 20px,)",
        "shape(close)",
        "shape(nonzero, close)",
        "shape(from 0px 10px)",
        "shape(allkindsofnonsense)",
        "shape(arc)",
    ] {
        assert!(shape(v).is_none(), "should reject: {v}");
    }
}

// ── shape-function-invalid.html 声明级钉面 ──────────────────────────────

#[test]
fn shape_invalid_grammar_breaks() {
    for v in [
        // 命令逗号缺失 / 悬挂逗号 / from 内逗号
        "shape(from 20px 40px line to 20px 30px)",
        "shape(from 20px 40px line to 20px 30px,)",
        "shape(from 20px, 40px, line to 20px, 30px)",
        // hline/vline 轴×关键字正交
        "shape(from 20px 40px, move to 20px 30px, hline to top)",
        "shape(from 20px 40px, move to 20px 30px, hline to bottom)",
        "shape(from 20px 40px, move to 20px 30px, hline to y-start)",
        "shape(from 20px 40px, move to 20px 30px, hline to y-end)",
        "shape(from 20px 40px, move to 20px 30px, vline to left)",
        "shape(from 20px 40px, move to 20px 30px, vline to right)",
        "shape(from 20px 40px, move to 20px 30px, vline to x-start)",
        "shape(from 20px 40px, move to 20px 30px, vline to x-end)",
        // 组合残段
        "shape(from 20px 40px, move to 20px 30px, hline to top 20px)",
        "shape(from 20px 40px, move to 20px 30px, hline to right, vline to bottom left, close)",
        // by 形拒关键字
        "shape(from 20px 40px, move to 20px 30px, hline by left)",
        "shape(from 20px 40px, move to 20px 30px, hline by right)",
        "shape(from 20px 40px, move to 20px 30px, vline by top)",
        "shape(from 20px 40px, move to 20px 30px, vline by bottom)",
        // 控制点必须 with 引入（using/via 拒）
        "shape(from 20px 40px, curve to 20px 20px, using 10px 30px)",
        "shape(from 20px 40px, curve to 20px 20px using 10px 30px, 12px 32px)",
        "shape(from 20px 40px, curve by 20px 20px, using 10px 30px)",
        "shape(from 20px 40px, curve by 20px 20px using 10px 30px, 12px 32px)",
        "shape(from 20px 40px, curve by 20px 20px via 10px 30px)",
        "shape(from 20px 40px, curve by 20px 20px via 10px 30px 12px 32px)",
        "shape(from 20px 40px, smooth by 20px 20px via 10px 30px / 12px 32px)",
        "shape(from 20px 40px, smooth by 20px 20px using 10px 30px 12px 32px)",
        // 双控制点必须 / 分隔
        "shape(from 20px 40px, curve to 20px 20px with 10px 30px 12px 32px)",
        "shape(from 20px 40px, curve by 20px 20px with 10px 30px 12px 32px)",
        "shape(from 20px 40px, curve by 20px 20px with 10px 30px /)",
        // <position> 只许 to 形（by 拒关键字）
        "shape(from 20px 40px, curve by top left using 10px 30px)",
        "shape(from 20px 40px, curve by 20px 30px using top right)",
        "shape(from 20px 40px, curve to 20px 30px using top right / 20px from end)",
        "shape(from 20px 40px, arc by top right of 10% 20%)",
        "shape(from 20px 40px, arc by 10px 20px of top right small cw)",
        // from 参考系仅控制点可带
        "shape(from 20px 40px, curve by 20px 30px from start, using 23px 32px)",
        // arc 半径/修饰残形
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% 12deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% rotate 12deg rotate 13deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw large 12deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% small large)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw ccw)",
    ] {
        assert!(shape(v).is_none(), "should reject: {v}");
    }
}

// ── shape-function-valid.html 声明级钉面 ────────────────────────────────

#[test]
fn shape_valid_basic_and_bare_numbers() {
    assert_eq!(
        shape("shape(from 20px 40px, line to 20px 30px)").unwrap(),
        "shape(from 20px 40px, line to 20px 30px)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, line to 20px 30px )").unwrap(),
        "shape(from 20px 40px, line to 20px 30px)"
    );
    assert_eq!(
        shape("shape(from 0 0, line to 100% 100%)").unwrap(),
        "shape(from 0px 0px, line to 100% 100%)"
    );
}

#[test]
fn shape_valid_position_keywords_canonical() {
    // 2 词轴关键字串化 x-then-y（top left → left top 等）
    assert_eq!(
        shape("shape(from 20px 40px, curve to top left with 10px 30px / 12px 32px)").unwrap(),
        "shape(from 20px 40px, curve to left top with 10px 30px / 12px 32px)"
    );
    assert_eq!(
        shape("shape(from 20% 40%, curve to center with center right / bottom center)").unwrap(),
        "shape(from 20% 40%, curve to center center with right center / center bottom)"
    );
    assert_eq!(
        shape("shape(from bottom right, curve to left with center right / bottom center)").unwrap(),
        "shape(from right bottom, curve to left center with right center / center bottom)"
    );
    assert_eq!(
        shape("shape(from top left, smooth to top right)").unwrap(),
        "shape(from left top, smooth to right top)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, arc to top right of 10%)").unwrap(),
        "shape(from 20px 40px, arc to right top of 10%)"
    );
    // 1 词关键字展 2 分量（from center → center center；to left → left center）
    assert_eq!(
        shape("shape(from center, smooth by 20px 20px with 12px 32px from origin)").unwrap(),
        "shape(from center center, smooth by 20px 20px with 12px 32px from origin)"
    );
}

#[test]
fn shape_valid_control_point_refs_to_by_defaults() {
    // to 形：origin 缺省省略，start/end 显式保留
    assert_eq!(
        shape("shape(from 20px 40px, curve to right center with 10px 30px from start / 12px 32px from end)").unwrap(),
        "shape(from 20px 40px, curve to right center with 10px 30px from start / 12px 32px from end)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve to right center with 10px 30px from start/12px 32px from end)").unwrap(),
        "shape(from 20px 40px, curve to right center with 10px 30px from start / 12px 32px from end)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve to right center with 10px 30px from  end / 12px 32px from  origin)")
            .unwrap(),
        "shape(from 20px 40px, curve to right center with 10px 30px from end / 12px 32px)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve to right center with 10px 30px from origin / 12px 32px from start)")
            .unwrap(),
        "shape(from 20px 40px, curve to right center with 10px 30px / 12px 32px from start)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve to bottom left with 10px 30px from end)").unwrap(),
        "shape(from 20px 40px, curve to left bottom with 10px 30px from end)"
    );
    // by 形：start 缺省省略，end/origin 显式保留
    assert_eq!(
        shape("shape(from 20px 40px, curve by 20px 20px with 10px 30px from origin)").unwrap(),
        "shape(from 20px 40px, curve by 20px 20px with 10px 30px from origin)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve by 20% 20em with 10px 30px from start / 12px 32px from end)").unwrap(),
        "shape(from 20px 40px, curve by 20% 20em with 10px 30px / 12px 32px from end)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, curve by 20% 20em with 10.3% 30px from origin / 12pt 5.4% from start)").unwrap(),
        "shape(from 20px 40px, curve by 20% 20em with 10.3% 30px from origin / 16px 5.4%)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, smooth by 20pt 20px with 12px 32px from start)").unwrap(),
        "shape(from 20px 40px, smooth by 26.666667px 20px with 12px 32px)"
    );
    assert_eq!(
        shape("shape(from center 40px, smooth by 20% 20% with 12px 32px from end)").unwrap(),
        "shape(from center 40px, smooth by 20% 20% with 12px 32px from end)"
    );
}

#[test]
fn shape_valid_arc_matrix() {
    let a = |input: &str, expected: &str| {
        assert_eq!(shape(input).unwrap(), expected, "input: {input}");
    };
    a(
        "shape(from 20px 40px, arc by 20px 20px of 0)",
        "shape(from 20px 40px, arc by 20px 20px of 0px)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 0)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 0px)",
    );
    a(
        "shape(from 20px 40px, arc by 20% 20pt of 10% rotate 0deg)",
        "shape(from 20px 40px, arc by 20% 26.666667px of 10%)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20%)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20%)",
    );
    // 显式 cw / large 保形（非缺省修饰）
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of -10% -20% large)",
        "shape(from 20px 40px, arc by 20px 20px of -10% -20% large)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% rotate 1deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% rotate 1deg)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw rotate 12deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw rotate 12deg)",
    );
    // rad 单位声明级保形
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10px 20px cw rotate 0.52rad)",
        "shape(from 20px 40px, arc by 20px 20px of 10px 20px cw rotate 0.52rad)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large rotate 12deg)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large rotate 12deg)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw large)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw large)",
    );
    // 修饰词输入任意序 → canonical 序 cw → large → rotate
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large cw)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% cw large)",
    );
    a(
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% rotate 12deg large)",
        "shape(from 20px 40px, arc by 20px 20px of 10% 20% large rotate 12deg)",
    );
}

#[test]
fn shape_valid_axis_keyword_inventory() {
    // hline/vline 轴关键字声明级保形（computed 析 % 归 engine 层）
    assert_eq!(
        shape("shape(from 20px 40px, move to 20px 30px, hline to left, hline to center, hline to right)").unwrap(),
        "shape(from 20px 40px, move to 20px 30px, hline to left, hline to center, hline to right)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, move to 20px 30px, hline to x-start, vline to y-start)").unwrap(),
        "shape(from 20px 40px, move to 20px 30px, hline to x-start, vline to y-start)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, move to 20px 30px, vline to top, vline to center, vline to bottom)").unwrap(),
        "shape(from 20px 40px, move to 20px 30px, vline to top, vline to center, vline to bottom)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, move to 20px 30px, vline to y-start, vline to y-end)").unwrap(),
        "shape(from 20px 40px, move to 20px 30px, vline to y-start, vline to y-end)"
    );
    assert_eq!(
        shape("shape(from 20px 40px, smooth to center 20%)").unwrap(),
        "shape(from 20px 40px, smooth to center 20%)"
    );
}

// ── edge-offset 位置（css-values-4 §position；14 轮 parse_origin_xy 同型） ─

#[test]
fn shape_edge_offset_positions() {
    // 4 词形：x-then-y 对序保形（声明级无 Calc 折算）
    assert_eq!(
        shape("shape(from right 30% top 10px, line to 0 0)").unwrap(),
        "shape(from right 30% top 10px, line to 0px 0px)"
    );
    // 2 词 edge-offset 形：缺轴补 center
    assert_eq!(
        shape("shape(from right 30%, line to 0 0)").unwrap(),
        "shape(from right 30% center, line to 0px 0px)"
    );
    // 残形拒收：center 作 edge / 同轴双 edge / 3 词
    assert!(shape("shape(from center 30% top 10px, line to 0 0)").is_none());
    assert!(shape("shape(from right 30% right 10px, line to 0 0)").is_none());
    assert!(shape("shape(from right 30% top, line to 0 0)").is_none());
}

#[test]
fn shape_axis_comp_resolved_folds_right_bottom() {
    // computed 折算：right/bottom 缘 → calc(100% − o)；left/top 缘保原值
    let p = parse_shape_position(&["right", "30%", "top", "10px"]).unwrap();
    assert_eq!(
        shape_axis_comp_resolved(&p.x),
        LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
            Box::new(CalcExpr::Length(LengthValue::Percentage(100.0))),
            CalcOp::Subtract,
            Box::new(CalcExpr::Length(LengthValue::Percentage(30.0))),
        )))
    );
    assert_eq!(shape_axis_comp_resolved(&p.y), LengthValue::Px(10.0));
    // 关键字析 %
    let p = parse_shape_position(&["center"]).unwrap();
    assert_eq!(shape_axis_comp_resolved(&p.x), LengthValue::Percentage(50.0));
    assert_eq!(shape_axis_comp_resolved(&p.y), LengthValue::Percentage(50.0));
    let p = parse_shape_position(&["bottom", "right"]).unwrap();
    assert_eq!(shape_axis_comp_resolved(&p.x), LengthValue::Percentage(100.0));
    assert_eq!(shape_axis_comp_resolved(&p.y), LengthValue::Percentage(100.0));
    // Edge(Top, o) 保原值
    let p = parse_shape_position(&["top", "10px"]).unwrap();
    assert_eq!(shape_axis_comp_resolved(&p.y), LengthValue::Px(10.0));
}

// ── calc 混合形（插值 expected 形；Stage B engine 混合单位臂同形） ────────

#[test]
fn shape_calc_forms_roundtrip() {
    assert_eq!(
        shape("shape(from calc(-4.5% - 6.5px) 2px, line to 1px 1px)").unwrap(),
        "shape(from calc(-4.5% - 6.5px) 2px, line to 1px 1px)"
    );
    assert_eq!(
        shape("shape(from 5px calc(10% + 5px), line to 1px 1px)").unwrap(),
        "shape(from 5px calc(10% + 5px), line to 1px 1px)"
    );
}

// ── path()（clip-path-valid/invalid.html 钉面） ─────────────────────────

#[test]
fn path_valid_forms() {
    assert_eq!(shape("path(\"m 20 0 h -100\")").unwrap(), "path(\"m 20 0 h -100\")");
    assert_eq!(
        shape("path(evenodd, \"M 20 20 h 60 v 60 h -60 Z M 30 30 h 40 v 40 h -40 Z\")").unwrap(),
        "path(evenodd, \"M 20 20 h 60 v 60 h -60 Z M 30 30 h 40 v 40 h -40 Z\")"
    );
    // nonzero 省略；段内逗号（数字间）合法
    assert_eq!(
        shape("path(nonzero, \"M20,20h60 v60 h-60z M30,30 h40 v40 h-40z\")").unwrap(),
        "path(\"M20,20h60 v60 h-60z M30,30 h40 v40 h-40z\")"
    );
    assert_eq!(
        shape("path(\"M 20 20 h 60 v 60 h -60 Z M 30 30 h 40 v 40 h -40 Z\")").unwrap(),
        "path(\"M 20 20 h 60 v 60 h -60 Z M 30 30 h 40 v 40 h -40 Z\")"
    );
    // 同命令多段（隐式重复）+ arc 7 参 + 数字归一
    assert_eq!(
        shape("path('m 0 0 10 10 20 20')").unwrap(),
        "path(\"m 0 0 10 10 20 20\")"
    );
    assert_eq!(
        shape("path('M 0 0 A 5 5 0 0 1 10 10')").unwrap(),
        "path(\"M 0 0 A 5 5 0 0 1 10 10\")"
    );
    // data 逐字回显（trivial serialization；段级数字归一归 engine computed）
    assert_eq!(
        shape("path('M 10.0 10 L 20.00 20')").unwrap(),
        "path(\"M 10.0 10 L 20.00 20\")"
    );
}

#[test]
fn path_invalid_forms() {
    for v in [
        "path(\" \")",
        "path(evenodd, \"\")",
        "path(abc, \"m 20 0 h -100 z\")",
        "path(nonzero)",
        "path(\"m 20 0 h -100\", nonzero)",
        // 逗号后随命令字母非法 + 非数字尾字符
        "path(nonzero, 'M0 0, L100 0, L0 100, L 0 0;')",
        "path('X 0 0')",
        "path('L 10')",
        "path(10 10)",
    ] {
        assert!(shape(v).is_none(), "should reject: {v}");
    }
}

// ── rect()（rect-function-{valid,invalid}.html 钉面） ───────────────────

#[test]
fn rect_valid_forms() {
    let a = |input: &str, expected: &str| {
        assert_eq!(shape(input).unwrap(), expected, "input: {input}");
    };
    a("rect(0 0 0 0)", "rect(0px 0px 0px 0px)");
    a("rect(0px 0px 0px 0px)", "rect(0px 0px 0px 0px)");
    a("rect(10px auto 20px 30px)", "rect(10px auto 20px 30px)");
    a("rect(auto auto auto auto)", "rect(auto auto auto auto)");
    a("rect(10% 20% 15% 12%)", "rect(10% 20% 15% 12%)");
    a("rect(10% 95% 97% 12%)", "rect(10% 95% 97% 12%)");
    a("rect(-10% -20% -15% -12%)", "rect(-10% -20% -15% -12%)");
    a(
        "rect(10px 12% 20px 30px round 100%)",
        "rect(10px 12% 20px 30px round 100%)",
    );
    // 裸 0 补 px + 2 值镜像折叠
    a(
        "rect(10px 12% 20px 30px round 0 1px)",
        "rect(10px 12% 20px 30px round 0px 1px)",
    );
    a(
        "rect(10px 12% 20px 30px round 0px 1px 2%)",
        "rect(10px 12% 20px 30px round 0px 1px 2%)",
    );
    a(
        "rect(10px 12% 20px 30px round 0px 1px 2% 3em)",
        "rect(10px 12% 20px 30px round 0px 1px 2% 3em)",
    );
    // / 垂直组保形（声明级单位保形；em 绝对化归 engine）
    a(
        "rect(10px 12% 20px 30px round 20% / 0px 1px 2% 3em)",
        "rect(10px 12% 20px 30px round 20% / 0px 1px 2% 3em)",
    );
}

#[test]
fn rect_invalid_forms() {
    for v in [
        "rect()",
        "rect(0px)",
        "rect(0px 1px)",
        "rect(0px 1px 2px)",
        // 逗号形非法（css-shapes-2 空白分隔文法）
        "rect(0px, 1px, 2px, 3px)",
        "rect(0, 1, 2, 3)",
        "rect(1% 2% 3% 4% 5%)",
        "rect(round 0)",
        "rect(10px auto 20px 30px round)",
        "rect(10px auto 20px 30px round 123)",
        "rect(10px auto 20px 30px round 1% 2% 3% 4% 5%)",
        "rect(10px auto 20px 30px round / 1px)",
        // 半径非负
        "rect(10px auto 20px 30px round -20px)",
        "rect(10px auto 20px 30px round -40%)",
    ] {
        assert!(shape(v).is_none(), "should reject: {v}");
    }
}

// ── xywh()（clip-path-xywh-00x / clip-path-interpolation-xywh-rect 形） ──

#[test]
fn xywh_valid_forms() {
    let a = |input: &str, expected: &str| {
        assert_eq!(shape(input).unwrap(), expected, "input: {input}");
    };
    a("xywh(12.5% 25% 37.5% 50%)", "xywh(12.5% 25% 37.5% 50%)");
    a("xywh(50px 50px 150px 100px)", "xywh(50px 50px 150px 100px)");
    a(
        "xywh(0px 10% 100px 40% round 20px)",
        "xywh(0px 10% 100px 40% round 20px)",
    );
    a(
        "xywh(50px 50px 150px 100px round 20px / 20px)",
        "xywh(50px 50px 150px 100px round 20px / 20px)",
    );
    // slash 形非法（round 关键字文法）
    assert!(shape("xywh(0 0 10px 20px / 5px)").is_none());
    assert!(shape("xywh(0 0 10px)").is_none());
    assert!(shape("xywh()").is_none());
}

// ── 派发与互斥（大小写不敏感） ──────────────────────────────────────────

#[test]
fn dispatch_reaches_new_variants_case_insensitive() {
    assert!(matches!(
        super::super::parse_extended_border::parse_clip_path("SHAPE(from 0 0, line to 1px 1px)"),
        Some(ClipPathValue::Shape(_))
    ));
    assert!(matches!(
        super::super::parse_extended_border::parse_clip_path("Path('M 0 0')"),
        Some(ClipPathValue::Path(_))
    ));
    assert!(matches!(
        super::super::parse_extended_border::parse_clip_path("XYWH(0 0 1px 1px)"),
        Some(ClipPathValue::Xywh(_))
    ));
    assert!(matches!(
        super::super::parse_extended_border::parse_clip_path("Rect(0px 1px 2px 3px)"),
        Some(ClipPathValue::Rect(_))
    ));
}

// ── 紧贴斜杠 token 拆分（控制点形；split_tokens_with_slash） ────────────

#[test]
fn glued_slash_tokens_split() {
    let toks = split_tokens_with_slash("10px 30px from start/12px 32px");
    assert_eq!(toks, vec!["10px", "30px", "from", "start", "/", "12px", "32px"]);
    // calc token 括号保护
    let toks = split_tokens_with_slash("calc(100% / 3) 5px");
    assert_eq!(toks, vec!["calc(100% / 3)", "5px"]);
}

// ── 串化双通道修钉（声明级物理单位守卫 + computed 级关键字折算） ──────────
// 语料钉面：shape-function-valid 4 pt 形（b0 verbatim 回显过、canonical
// 绝对化翻红的守卫回归面）+ shape-function-computed 关键字 9 形。

#[test]
fn physical_length_unit_detection() {
    for v in [
        "shape(from 20px 40px, curve to 20em 20pt with 10vw 30vh)",
        "shape(from 0 0, line to 1.5cm 2px)",
        "shape(from 0 0, arc to 2px 2px of 1in)",
        "rect(1q 2px 3px 4px)",
        "shape(from 0 0, arc by 1px 1px of 2px rotate 1pc)",
        "xywh(calc(2in + 3px) 0 1px 1px)",
    ] {
        assert!(has_physical_length_unit(v), "should detect: {v}");
    }
    for v in [
        "shape(from 20px 40px, curve to 20em 20% with 10vw 30vh)",
        "shape(from center, hline to left)",
        "shape(from 0 0, arc by 1px 1px of 2px rotate 3.14159265rad)",
        "polygon(50% 0%, 0% 100%)",
        "circle(10px at center)",
        "x-end 2px",
        "shape(from 0 0, arc by 1px 1px of 2px rotate 0.5turn)",
    ] {
        assert!(!has_physical_length_unit(v), "should not detect: {v}");
    }
}

#[test]
fn shape_declaration_fold_guard_skips_physical_units() {
    use crate::values::serialize_math::fold_decl_specified;
    // pt 形：守卫跳过 canonical（parse 期折 px 无法保原单位形），回落原文。
    assert!(fold_decl_specified("clip-path", "shape(from 20px 40px, curve to 20em 20pt with 10vw 30vh)").is_none());
    // 非物理单位形：canonical 折叠照常（裸数字补 px 语料钉面）。
    assert_eq!(
        fold_decl_specified("clip-path", "shape(from 0 0, line to 10px 20px)").as_deref(),
        Some("shape(from 0px 0px, line to 10px 20px)")
    );
}

fn shape_computed(v: &str) -> Option<String> {
    match super::super::parse_extended_border::parse_clip_path(v)? {
        ClipPathValue::Shape(s) => Some(shape_to_css_computed(&s)),
        _ => None,
    }
}

#[test]
fn shape_computed_serialization_resolves_keywords() {
    // from 位置关键字折 %（1 词补 2 分量）。
    assert_eq!(
        shape_computed("shape(from center, line to 10px 10px)").unwrap(),
        "shape(from 50% 50%, line to 10px 10px)"
    );
    assert_eq!(
        shape_computed("shape(from center right, line to 0 0)").unwrap(),
        "shape(from 100% 50%, line to 0px 0px)"
    );
    // hline/vline 轴关键字折 %。
    assert_eq!(
        shape_computed("shape(from 20px 40px, move to 20px 30px, hline to left, hline to center, hline to right)")
            .unwrap(),
        "shape(from 20px 40px, move to 20px 30px, hline to 0%, hline to 50%, hline to 100%)"
    );
    assert_eq!(
        shape_computed("shape(from 20px 40px, vline to y-start, vline to y-end)").unwrap(),
        "shape(from 20px 40px, vline to 0%, vline to 100%)"
    );
    // curve 端点/控制点关键字折 %（`top right` x-then-y 序归一）。
    assert_eq!(
        shape_computed("shape(from 20px 40px, curve to top right with 10px 30px / 12px 32px)").unwrap(),
        "shape(from 20px 40px, curve to 100% 0% with 10px 30px / 12px 32px)"
    );
    assert_eq!(
        shape_computed("shape(from center, curve to center bottom with top right / bottom right)").unwrap(),
        "shape(from 50% 50%, curve to 50% 100% with 100% 0% / 100% 100%)"
    );
    // edge-offset right/bottom 折 calc（14 轮 offset-position 同型）。
    assert_eq!(
        shape_computed("shape(from 0 0, curve to right 30% top 0% with 0 0 / 0 0)").unwrap(),
        "shape(from 0px 0px, curve to calc(100% - 30%) 0% with 0px 0px / 0px 0px)"
    );
    // rotate 归 deg（声明级保 rad 不变）。
    assert_eq!(
        shape_computed("shape(from 0 0, arc by 20px 20px of 10% 20% cw rotate 3.14159265rad)").unwrap(),
        "shape(from 0px 0px, arc by 20px 20px of 10% 20% cw rotate 180deg)"
    );
    assert_eq!(
        shape("shape(from 0 0, arc by 20px 20px of 10% 20% cw rotate 3.14159265rad)").unwrap(),
        "shape(from 0px 0px, arc by 20px 20px of 10% 20% cw rotate 3.141593rad)"
    );
    // 声明级通道关键字保形（回归钉：computed 化不外溢声明级）。
    assert_eq!(
        shape("shape(from center, hline to left)").unwrap(),
        "shape(from center center, hline to left)"
    );
}

// ── circle/ellipse at <position> 4 值 edge-offset（shape-outside 门线刀刀 B） ──

#[test]
fn circle_at_position_four_value_edge_offsets() {
    // css-values-4 §position 4 值 edge-offset 形（css-shapes-1 circle/ellipse
    // at <position> 语料主形态）：top/left 偏移直读、right/bottom 偏移按
    // 100%−o Calc 折算存储、同轴双对拒收。语料钉面 shape-outside/circle-004。
    // <https://drafts.csswg.org/css-values-4/#typedef-position>
    let circle_at = |v: &str| match super::super::parse_extended_border::parse_clip_path(v) {
        Some(ClipPathValue::Circle { position, .. }) => position,
        _ => None,
    };
    // top/left 直读：x=left 偏移 30px、y=top 偏移 10px。
    assert_eq!(
        circle_at("circle(50px at top 10px left 30px)"),
        Some((LengthValue::Px(30.0), LengthValue::Px(10.0)))
    );
    // right/bottom → Calc(100% − o)。
    let assert_edge_calc = |v: &str, off_px: f64, label: &str| {
        let got = circle_at(v).unwrap().0;
        let ok = matches!(
            &got,
            LengthValue::Calc(c)
                if matches!(
                    &**c,
                    CalcExpr::BinaryOp(l, CalcOp::Subtract, r)
                        if matches!(&**l, CalcExpr::Length(LengthValue::Percentage(100.0)))
                            && matches!(&**r, CalcExpr::Length(LengthValue::Px(p)) if *p == off_px)
                )
        );
        assert!(ok, "{label}: expected calc(100% - {off_px}px), got {got:?}");
    };
    assert_edge_calc("circle(50px at right 30px bottom 20px)", 30.0, "x");
    // y 分量同臂（bottom 20px → calc(100% - 20px)）。
    let (_, y) = circle_at("circle(50px at right 30px bottom 20px)").unwrap();
    assert!(matches!(
        y,
        LengthValue::Calc(ref c)
            if matches!(**c, CalcExpr::BinaryOp(_, CalcOp::Subtract,
                ref r) if matches!(**r, CalcExpr::Length(LengthValue::Px(p)) if p == 20.0))
    ));
    // 同轴双对拒收（css-values-4 <position> 文法：每轴至多一对）。
    assert_eq!(circle_at("circle(50px at left 10px right 20px)"), None);
    // FIXME: `<offset> <edge>` 乱序形（css-values-4 文法合法）现臂拒收，
    // 语料现值均为 edge 首词形，后置。
}
