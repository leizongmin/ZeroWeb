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
    assert!(parse_clip_path("rect(0 0 0 0)").is_none());
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
