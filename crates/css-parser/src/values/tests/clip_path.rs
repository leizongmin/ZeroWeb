//! CSS clip-path 属性解析测试。

use super::super::*;

// ── none ──

#[test]
fn test_clip_path_none() {
    let v = parse_clip_path("none").unwrap();
    assert!(matches!(v, ClipPathValue::None));
}

#[test]
fn test_clip_path_none_case_insensitive() {
    let v = parse_clip_path("NONE").unwrap();
    assert!(matches!(v, ClipPathValue::None));
}

#[test]
fn test_clip_path_invalid() {
    assert!(parse_clip_path("invalid").is_none());
    // rect() 文法刀（css-shapes-2 空白分隔 4 值）后原「rect(0 0 0 0) 拒收」
    // 旧钉失效；3 值残形仍拒收
    assert!(parse_clip_path("rect(0 0 0)").is_none());
}

#[test]
fn test_clip_path_basic_shape_function_names_are_case_insensitive() {
    // https://www.w3.org/TR/css-syntax-3/#function-token-diagram
    // Function names in CSS values are ASCII case-insensitive.
    assert!(matches!(
        parse_clip_path("INSET(10px)"),
        Some(ClipPathValue::Inset { .. })
    ));
    assert!(matches!(
        parse_clip_path("Circle(50px at center)"),
        Some(ClipPathValue::Circle { .. })
    ));
    assert!(matches!(
        parse_clip_path("ELLIPSE(100px 50px)"),
        Some(ClipPathValue::Ellipse { .. })
    ));
    assert!(matches!(
        parse_clip_path("Polygon(evenodd, 0 0, 100% 0, 50% 100%)"),
        Some(ClipPathValue::Polygon {
            fill_rule: PolygonFillRule::EvenOdd,
            ..
        })
    ));
}

// ── inset() ──

#[test]
fn test_clip_path_inset_single_value() {
    let v = parse_clip_path("inset(10px)").unwrap();
    match v {
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            assert_eq!(top, LengthValue::Px(10.0));
            assert_eq!(right, LengthValue::Px(10.0));
            assert_eq!(bottom, LengthValue::Px(10.0));
            assert_eq!(left, LengthValue::Px(10.0));
            assert!(round.is_none());
        }
        _ => panic!("Expected Inset variant"),
    }
}

#[test]
fn test_clip_path_inset_four_values() {
    let v = parse_clip_path("inset(10px 20px 30px 40px)").unwrap();
    match v {
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            assert_eq!(top, LengthValue::Px(10.0));
            assert_eq!(right, LengthValue::Px(20.0));
            assert_eq!(bottom, LengthValue::Px(30.0));
            assert_eq!(left, LengthValue::Px(40.0));
            assert!(round.is_none());
        }
        _ => panic!("Expected Inset variant"),
    }
}

#[test]
fn test_clip_path_inset_two_values() {
    let v = parse_clip_path("inset(10px 20px)").unwrap();
    match v {
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            ..
        } => {
            assert_eq!(top, LengthValue::Px(10.0));
            assert_eq!(right, LengthValue::Px(20.0));
            assert_eq!(bottom, LengthValue::Px(10.0)); // 复制 top
            assert_eq!(left, LengthValue::Px(20.0)); // 复制 right
        }
        _ => panic!("Expected Inset variant"),
    }
}

#[test]
fn test_clip_path_inset_three_values() {
    let v = parse_clip_path("inset(10px 20px 30px)").unwrap();
    match v {
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            ..
        } => {
            assert_eq!(top, LengthValue::Px(10.0));
            assert_eq!(right, LengthValue::Px(20.0));
            assert_eq!(bottom, LengthValue::Px(30.0));
            assert_eq!(left, LengthValue::Px(20.0)); // 复制 right
        }
        _ => panic!("Expected Inset variant"),
    }
}

#[test]
fn test_clip_path_inset_percentage() {
    let v = parse_clip_path("inset(10% 20%)").unwrap();
    match v {
        ClipPathValue::Inset { top, right, .. } => {
            assert_eq!(top, LengthValue::Percentage(10.0));
            assert_eq!(right, LengthValue::Percentage(20.0));
        }
        _ => panic!("Expected Inset variant"),
    }
}

#[test]
fn test_clip_path_inset_empty() {
    assert!(parse_clip_path("inset()").is_none());
}

#[test]
fn test_clip_path_inset_rejects_extra_values_and_invalid_round() {
    assert!(parse_clip_path("inset(1px 2px 3px 4px 5px)").is_none());
    assert!(parse_clip_path("inset(10px round bogus)").is_none());
}

#[test]
fn test_clip_path_inset_rejects_invalid_length_grammar() {
    assert!(parse_clip_path("inset(-10px 20%)").is_some());
    for value in ["inset(thin)", "inset(auto)", "inset(infpx)", "inset(10px round thin)"] {
        assert!(parse_clip_path(value).is_none(), "{value} should be rejected");
    }
}

// ── circle() ──

#[test]
fn test_clip_path_circle_default() {
    let v = parse_clip_path("circle()").unwrap();
    match v {
        ClipPathValue::Circle { radius, position } => {
            assert!(matches!(radius, ClipPathRadius::ClosestSide));
            assert!(position.is_none());
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_with_radius() {
    let v = parse_clip_path("circle(50px)").unwrap();
    match v {
        ClipPathValue::Circle { radius, position } => {
            assert!(matches!(radius, ClipPathRadius::Length(LengthValue::Px(50.0))));
            assert!(position.is_none());
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_closest_side() {
    let v = parse_clip_path("circle(closest-side)").unwrap();
    match v {
        ClipPathValue::Circle { radius, .. } => {
            assert!(matches!(radius, ClipPathRadius::ClosestSide));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_farthest_side() {
    let v = parse_clip_path("circle(farthest-side)").unwrap();
    match v {
        ClipPathValue::Circle { radius, .. } => {
            assert!(matches!(radius, ClipPathRadius::FarthestSide));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_at_position() {
    let v = parse_clip_path("circle(50px at 100px 200px)").unwrap();
    match v {
        ClipPathValue::Circle { radius, position } => {
            assert!(matches!(radius, ClipPathRadius::Length(LengthValue::Px(50.0))));
            let (x, y) = position.unwrap();
            assert_eq!(x, LengthValue::Px(100.0));
            assert_eq!(y, LengthValue::Px(200.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_at_center() {
    let v = parse_clip_path("circle(at center)").unwrap();
    match v {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert_eq!(x, LengthValue::Percentage(50.0));
            assert_eq!(y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_circle_rejects_invalid_position() {
    assert!(parse_clip_path("circle(50px at bogus)").is_none());
    assert!(parse_clip_path("circle(50px at left top extra)").is_none());
}

#[test]
fn test_clip_path_circle_rejects_invalid_radius_grammar() {
    for value in [
        "circle(-1px)",
        "circle(thin)",
        "circle(auto)",
        "circle(min-content)",
        "circle(infpx)",
        "circle(NaNpx)",
        "circle(50px at thin 20px)",
    ] {
        assert!(parse_clip_path(value).is_none(), "{value} should be rejected");
    }
}

// ── ellipse() ──

#[test]
fn test_clip_path_ellipse_default() {
    let v = parse_clip_path("ellipse()").unwrap();
    match v {
        ClipPathValue::Ellipse { rx, ry, position } => {
            assert!(matches!(rx, ClipPathRadius::ClosestSide));
            assert!(matches!(ry, ClipPathRadius::ClosestSide));
            assert!(position.is_none());
        }
        _ => panic!("Expected Ellipse variant"),
    }
}

#[test]
fn test_clip_path_ellipse_with_radii() {
    let v = parse_clip_path("ellipse(100px 50px)").unwrap();
    match v {
        ClipPathValue::Ellipse { rx, ry, .. } => {
            assert!(matches!(rx, ClipPathRadius::Length(LengthValue::Px(100.0))));
            assert!(matches!(ry, ClipPathRadius::Length(LengthValue::Px(50.0))));
        }
        _ => panic!("Expected Ellipse variant"),
    }
}

#[test]
fn test_clip_path_ellipse_at_position() {
    let v = parse_clip_path("ellipse(100px 50px at 0 0)").unwrap();
    match v {
        ClipPathValue::Ellipse { position, .. } => {
            let (x, y) = position.unwrap();
            assert_eq!(x, LengthValue::Px(0.0));
            assert_eq!(y, LengthValue::Px(0.0));
        }
        _ => panic!("Expected Ellipse variant"),
    }
}

#[test]
fn test_clip_path_ellipse_keyword_radii() {
    let v = parse_clip_path("ellipse(closest-side farthest-side)").unwrap();
    match v {
        ClipPathValue::Ellipse { rx, ry, .. } => {
            assert!(matches!(rx, ClipPathRadius::ClosestSide));
            assert!(matches!(ry, ClipPathRadius::FarthestSide));
        }
        _ => panic!("Expected Ellipse variant"),
    }
}

#[test]
fn test_clip_path_ellipse_rejects_extra_radii_and_invalid_position() {
    assert!(parse_clip_path("ellipse(10px 20px 30px)").is_none());
    assert!(parse_clip_path("ellipse(10px 20px at bad)").is_none());
    assert!(parse_clip_path("ellipse(10px 20px at left top extra)").is_none());
}

#[test]
fn test_clip_path_ellipse_rejects_invalid_radius_grammar() {
    for value in [
        "ellipse(-1px 20px)",
        "ellipse(10px -2%)",
        "ellipse(thin 20px)",
        "ellipse(10px auto)",
        "ellipse(10px infpx)",
    ] {
        assert!(parse_clip_path(value).is_none(), "{value} should be rejected");
    }
}

// ── polygon() ──

#[test]
fn test_clip_path_polygon_basic() {
    let v = parse_clip_path("polygon(0 0, 100px 0, 100px 100px)").unwrap();
    match v {
        ClipPathValue::Polygon { fill_rule, points } => {
            assert!(matches!(fill_rule, PolygonFillRule::NonZero));
            assert_eq!(points.len(), 3);
            assert_eq!(points[0].0, LengthValue::Px(0.0));
            assert_eq!(points[0].1, LengthValue::Px(0.0));
        }
        _ => panic!("Expected Polygon variant"),
    }
}

#[test]
fn test_clip_path_polygon_evenodd() {
    let v = parse_clip_path("polygon(evenodd, 0 0, 100% 0, 100% 100%)").unwrap();
    match v {
        ClipPathValue::Polygon { fill_rule, points } => {
            assert!(matches!(fill_rule, PolygonFillRule::EvenOdd));
            assert_eq!(points.len(), 3);
        }
        _ => panic!("Expected Polygon variant"),
    }
}

#[test]
fn test_clip_path_polygon_accepts_math_function_coordinates() {
    // https://drafts.fxtf.org/css-masking-1/#funcdef-basic-shape-polygon
    // polygon() points are comma-separated at the top level; commas inside CSS math functions
    // remain part of the <length-percentage> coordinate.
    let v = parse_clip_path("polygon(0 0, min(10%, 20%) calc(100% - 1px), 100% 100%)").unwrap();
    match v {
        ClipPathValue::Polygon { points, .. } => {
            assert_eq!(points.len(), 3);
            assert!(matches!(points[1].0, LengthValue::Calc(_)));
            assert!(matches!(points[1].1, LengthValue::Calc(_)));
        }
        _ => panic!("Expected Polygon variant"),
    }
}

#[test]
fn test_clip_path_polygon_fill_rule_is_case_insensitive() {
    // https://drafts.fxtf.org/css-masking-1/#typedef-basic-shape
    // CSS-defined polygon() fill-rule keywords are ASCII case-insensitive.
    let evenodd = parse_clip_path("polygon(EvEnOdD, 0 0, 100% 0, 100% 100%)").unwrap();
    assert!(matches!(
        evenodd,
        ClipPathValue::Polygon {
            fill_rule: PolygonFillRule::EvenOdd,
            ..
        }
    ));

    let nonzero = parse_clip_path("polygon(NoNzErO, 0 0, 50% 100%)").unwrap();
    assert!(matches!(
        nonzero,
        ClipPathValue::Polygon {
            fill_rule: PolygonFillRule::NonZero,
            ..
        }
    ));
}

#[test]
fn test_clip_path_polygon_nonzero_explicit() {
    let v = parse_clip_path("polygon(nonzero, 0 0, 50% 100%)").unwrap();
    match v {
        ClipPathValue::Polygon { fill_rule, points } => {
            assert!(matches!(fill_rule, PolygonFillRule::NonZero));
            assert_eq!(points.len(), 2);
        }
        _ => panic!("Expected Polygon variant"),
    }
}

#[test]
fn test_clip_path_polygon_single_point() {
    let v = parse_clip_path("polygon(50% 50%)").unwrap();
    match v {
        ClipPathValue::Polygon { points, .. } => {
            assert_eq!(points.len(), 1);
            assert_eq!(points[0].0, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected Polygon variant"),
    }
}

#[test]
fn test_clip_path_polygon_empty() {
    assert!(parse_clip_path("polygon()").is_none());
}

#[test]
fn test_clip_path_polygon_invalid_points_are_not_dropped() {
    assert!(parse_clip_path("polygon(0 0, , 100% 100%)").is_none());
    assert!(parse_clip_path("polygon(0 0, 100%)").is_none());
    assert!(parse_clip_path("polygon(0 0, 100% 100% 50%)").is_none());
}

#[test]
fn test_clip_path_polygon_rejects_invalid_length_grammar() {
    assert!(parse_clip_path("polygon(-10px -5%, 100% 0, 50% 100%)").is_some());
    for value in [
        "polygon(thin 0, 100% 0, 50% 100%)",
        "polygon(0 infpx, 100% 0, 50% 100%)",
        "polygon(0 auto, 100% 0, 50% 100%)",
        "polygon(calc(1) 0, 100% 0, 50% 100%)",
        "polygon(calc(1 + 1px) 0, 100% 0, 50% 100%)",
        "polygon(calc(auto) 0, 100% 0, 50% 100%)",
    ] {
        assert!(parse_clip_path(value).is_none(), "{value} should be rejected");
    }
}

#[test]
fn test_clip_path_radius_rejects_resolved_negative_math() {
    assert!(parse_clip_path("circle(calc(-1px))").is_none());
}

#[test]
fn test_clip_path_polygon_many_points() {
    // 五角星
    let v = parse_clip_path(
        "polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%)",
    )
    .unwrap();
    match v {
        ClipPathValue::Polygon { points, .. } => {
            assert_eq!(points.len(), 10);
        }
        _ => panic!("Expected Polygon variant"),
    }
}

// ===== at <position> 轴别严格文法（css-values-4 §position；wptfyi-css
// circle/ellipse-003 invalid 125 形 oracle + valid y 首词对换钉面）=====

#[test]
fn test_clip_path_position_mixed_axis_rejected() {
    // [lp][x-edge] / [y-edge][lp] 混轴 2 词形拒收（invalid-position 语料）。
    assert!(parse_clip_path("circle(at 50% left)").is_none());
    assert!(parse_clip_path("circle(at 50px left)").is_none());
    assert!(parse_clip_path("circle(at 80% right)").is_none());
    assert!(parse_clip_path("circle(at top 50%)").is_none());
    assert!(parse_clip_path("circle(at bottom 70px)").is_none());
}

#[test]
fn test_clip_path_position_same_axis_keyword_pair_rejected() {
    assert!(parse_clip_path("circle(at left right)").is_none());
    assert!(parse_clip_path("circle(at left left)").is_none());
    assert!(parse_clip_path("circle(at top bottom)").is_none());
    assert!(parse_clip_path("circle(at bottom bottom)").is_none());
}

#[test]
fn test_clip_path_position_three_token_mixed_rejected() {
    // 3 词混搭形全禁（`center center 60%` / `left center 60%` /
    // `center 60% top` 语料族）。
    assert!(parse_clip_path("circle(at center center 60%)").is_none());
    assert!(parse_clip_path("circle(at left center 60px)").is_none());
    assert!(parse_clip_path("circle(at center 60% top)").is_none());
    assert!(parse_clip_path("circle(at top 50% bottom)").is_none());
}

#[test]
fn test_clip_path_position_four_token_invalid_pairs_rejected() {
    // 4 词形：center 不带偏移 / 同轴双对拒收。
    assert!(parse_clip_path("circle(at center 60% top 50%)").is_none());
    assert!(parse_clip_path("circle(at left 50% right 80%)").is_none());
    assert!(parse_clip_path("circle(at top 50% bottom 70%)").is_none());
}

#[test]
fn test_clip_path_position_y_first_keyword_pair_reorder() {
    // validPositions y 首词对换 x-first（`top left`/`top center`/
    // `center left` → (x,y) 对换存储）。
    match parse_clip_path("circle(at top left)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 0.0));
            assert!(matches!(y, LengthValue::Percentage(p) if p == 0.0));
        }
        _ => panic!("Expected Circle variant"),
    }
    match parse_clip_path("circle(at top center)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 50.0));
            assert!(matches!(y, LengthValue::Percentage(p) if p == 0.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_position_single_y_keyword() {
    // 1 值 y-edge 关键字（`top` → x 缺省 center）。
    match parse_clip_path("circle(at top)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 50.0));
            assert!(matches!(y, LengthValue::Percentage(p) if p == 0.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_position_two_token_axis_resolution() {
    // 2 词形关键字按轴折 %（`left 50%` → (0%, 50%)，chrome computed
    // oracle validPositions `0% 50%`）。
    match parse_clip_path("circle(at left 50%)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 0.0));
            assert!(matches!(y, LengthValue::Percentage(p) if p == 50.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_position_edge_offset_percentage_complement() {
    // 4 值 edge-offset 百分比补数折算存储（`right 30%` → 70%、
    // `bottom 70%` → 30%；chrome computed `30% 70%` 形）。
    match parse_clip_path("circle(at bottom 70% right 30%)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 70.0));
            assert!(matches!(y, LengthValue::Percentage(p) if p == 30.0));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_position_edge_offset_length_keeps_calc() {
    // 长度缘偏移保 Calc(100%−L)（computed `calc(100% - 70px)` 不可声明期
    // 折算——corpus `circle(at 50% calc(100% - 1120px))` 形）。
    match parse_clip_path("circle(at bottom 70px right 30%)").unwrap() {
        ClipPathValue::Circle { position, .. } => {
            let (x, y) = position.unwrap();
            assert!(matches!(x, LengthValue::Percentage(p) if p == 70.0));
            assert!(matches!(
                y,
                LengthValue::Calc(boxed)
                    if matches!(*boxed, CalcExpr::BinaryOp(_, CalcOp::Subtract, _))
            ));
        }
        _ => panic!("Expected Circle variant"),
    }
}

#[test]
fn test_clip_path_inset_round_radius_lists() {
    // css-shapes-1 §funcdef-inset round 参量全形：`<radius>{1,4}
    // [/ <radius>{1,4}]?`——写面门 parse 判定域（corpus inset-002/003 valid
    // 页 round 形原文回显钉面；invalid 空 round 参量/5+ 半径拒收）。
    for v in [
        "inset(10px round 10px)",
        "inset(10cm round 10cm 20cm)",
        "inset(10cm round 10cm 20cm 30cm 40cm)",
        "inset(10cm round 10cm / 10cm)",
        "inset(10cm round 10cm / 10cm 20cm 30cm 40cm)",
        "inset(10px round closest-side)",
    ] {
        assert!(parse_clip_path(v).is_some(), "should parse: {}", v);
    }
    for v in [
        "inset(10px round)",
        "inset(10px round 10px 20px 30px 40px 50px)",
        "inset(10px round 10px / 10px 20px 30px 40px 50px)",
        "inset(10px round -10px)",
    ] {
        assert!(parse_clip_path(v).is_none(), "should reject: {}", v);
    }
}

#[test]
fn test_clip_path_polygon_round_tail() {
    // css-shapes-1 §funcdef-basic-shape-polygon 尾缀 round 参量：radius 组
    // 同 border-radius 简写文法；负半径拒（corpus polygon-round-invalid 域）。
    assert!(parse_clip_path("polygon(0% 0%, 100% 0%, 50% 100% round 10px)").is_some());
    assert!(parse_clip_path("polygon(evenodd, 0% 0%, 100% 0%, 50% 100% round 10px 20px / 30px)").is_some());
    assert!(parse_clip_path("polygon(0% 0%, 100% 0%, 50% 100% round)").is_none());
    assert!(parse_clip_path("polygon(0% 0%, 100% 0%, 50% 100% round -10px)").is_none());
}

#[test]
fn test_clip_path_polygon_round_prefix() {
    // css-shapes-1 §funcdef-basic-shape-polygon 前缀 round 参量：radius 组
    // 在坐标列表前（fill-rule 同槽无逗号直接相邻亦受理）。前四案为写面门
    // 接入后 b1v5 vs b0 −4 回归复现钉（corpus polygon-function-round-valid
    // 页；裸回显时代 Pass，门 consult parse 后拒收翻 Fail，前缀臂修复）。
    assert!(parse_clip_path("polygon(round 1px, 1% 2%)").is_some());
    assert!(parse_clip_path("polygon(nonzero round 1px, 1px 2px, 3em 4em)").is_some());
    assert!(parse_clip_path("polygon(evenodd round 1px, 1px 2px, 3em 4em, 5pt 6%)").is_some());
    assert!(parse_clip_path("polygon(round calc(1em + 2px), 1px 2px, 3px 4px)").is_some());
    // corpus polygon-function-round-invalid 域九案全拒。
    assert!(parse_clip_path("polygon(round, 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(round 1, 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(round -1px, 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(round 45deg, 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(round 1px 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(1px 2px, round 1px)").is_none());
    assert!(parse_clip_path("polygon(round 1px round 2px, 1px 2px)").is_none());
    assert!(parse_clip_path("polygon(round 1px nonzero, 1px 2px)").is_none());
    // Chrome 语料契约：polygon round 半径拒 <percentage>（round-invalid 域
    // `polygon(round 1%, …)`；inset round 依 border-radius 文法仍收）。
    assert!(parse_clip_path("polygon(round 1%, 1px 2px)").is_none());
}

#[test]
fn test_clip_path_inset_calc_offset_acceptance() {
    // inset calc 化偏移受理钉（残面⑤ calc 代数插值刀落地）：顶层空白切分
    // paren-aware 后 calc 分量不分断，此前朴素切分拒收边界（rejection
    // boundary）撤除——语料 inset-function-valid/008/computed 四案 + 动画
    // t=1 对解锁。b1v6/b1v7 伪通过池回落链由 interp calc 代数插值闭合。
    assert!(parse_clip_path("inset(0px calc(100% - 20px) 2% 3em)").is_some());
    assert!(parse_clip_path("inset(calc(50% - 10px) 0px)").is_some());
    assert!(parse_clip_path("inset(10% calc(100% - 100px) 50% 0px round 20px)").is_some());
    // round 参量槽不在偏移切分域，无偏移 calc 形不受理 round 仍解析。
    assert!(parse_clip_path("inset(0px round calc(1em + 2px))").is_some());
    // at <position> calc 分量不分断（circle/ellipse 位置文法刀同域）。
    assert!(parse_clip_path("circle(50% at calc(50% + 10px) 20px)").is_some());
    assert!(parse_clip_path("ellipse(40px 60px at left calc(50% - 10px))").is_some());
}
