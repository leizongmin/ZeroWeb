//! CSS clip-path 样式管线测试。

use super::super::*;

// ── apply_property_value 测试 ──

#[test]
fn test_clip_path_apply_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "none"));
    assert!(matches!(style.clip_path, ClipPathComputedValue::None));
}

#[test]
fn test_clip_path_apply_inset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "inset(10px 20px)"));
    match &style.clip_path {
        ClipPathComputedValue::Inset { top, right, .. } => {
            assert_eq!(*top, LengthValue::Px(10.0));
            assert_eq!(*right, LengthValue::Px(20.0));
        }
        _ => panic!("Expected Inset"),
    }
}

#[test]
fn test_clip_path_apply_basic_shape_function_names_are_case_insensitive() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "INSET(10px)"));
    assert!(matches!(style.clip_path, ClipPathComputedValue::Inset { .. }));

    assert!(apply_property_value(&mut style, "clip-path", "Circle(50px at center)"));
    assert!(matches!(style.clip_path, ClipPathComputedValue::Circle { .. }));
}

#[test]
fn test_clip_rect_apply_function_name_is_case_insensitive() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip", "RECT(0px, auto, 10px, 0px)"));
    assert!(matches!(style.clip, ClipRectComputedValue::Rect(_, _, _, _)));
}

#[test]
fn test_clip_rect_apply_math_function_offsets() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip",
        "rect(min(10px, 20px), auto, clamp(0px, 5px, 10px), 0px)"
    ));
    assert!(matches!(
        style.clip,
        ClipRectComputedValue::Rect(
            LengthValue::Calc(_),
            LengthValue::Px(0.0),
            LengthValue::Calc(_),
            LengthValue::Px(0.0)
        )
    ));

    let previous = style.clip.clone();
    assert!(!apply_property_value(
        &mut style,
        "clip",
        "rect(calc(1), auto, 10px, 0px)"
    ));
    assert_eq!(style.clip, previous);
}

#[test]
fn test_clip_path_apply_circle() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "circle(50px)"));
    match &style.clip_path {
        ClipPathComputedValue::Circle { radius, .. } => {
            assert!(matches!(radius, ClipPathRadius::Length(LengthValue::Px(50.0))));
        }
        _ => panic!("Expected Circle"),
    }
}

#[test]
fn test_clip_path_apply_ellipse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "ellipse(100px 50px)"));
    match &style.clip_path {
        ClipPathComputedValue::Ellipse { rx, ry, .. } => {
            assert!(matches!(rx, ClipPathRadius::Length(LengthValue::Px(100.0))));
            assert!(matches!(ry, ClipPathRadius::Length(LengthValue::Px(50.0))));
        }
        _ => panic!("Expected Ellipse"),
    }
}

#[test]
fn test_clip_path_apply_polygon() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip-path",
        "polygon(0 0, 100px 0, 50px 100px)"
    ));
    match &style.clip_path {
        ClipPathComputedValue::Polygon { points, .. } => {
            assert_eq!(points.len(), 3);
        }
        _ => panic!("Expected Polygon"),
    }
}

#[test]
fn test_clip_path_apply_polygon_math_function_coordinates() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip-path",
        "polygon(0 0, min(10%, 20%) calc(100% - 1px), 100% 100%)"
    ));
    match &style.clip_path {
        ClipPathComputedValue::Polygon { points, .. } => {
            assert_eq!(points.len(), 3);
            assert!(matches!(points[1].0, LengthValue::Calc(_)));
            assert!(matches!(points[1].1, LengthValue::Calc(_)));
        }
        _ => panic!("Expected Polygon"),
    }
}

#[test]
fn test_clip_path_apply_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "clip-path", "invalid"));
    // 应保持默认值 None
    assert!(matches!(style.clip_path, ClipPathComputedValue::None));

    assert!(apply_property_value(&mut style, "clip-path", "circle(50px)"));
    let previous = style.clip_path.clone();
    for value in [
        "circle(-1px)",
        "circle(thin)",
        "circle(infpx)",
        "ellipse(10px auto)",
        "inset(thin)",
        "polygon(0 infpx, 100% 0, 50% 100%)",
    ] {
        assert!(!apply_property_value(&mut style, "clip-path", value));
        assert_eq!(style.clip_path, previous, "{value} should not overwrite");
    }
}

// ── 初始值测试 ──

#[test]
fn test_clip_path_initial_value() {
    assert!(PropertyRegistry::initial_value("clip-path").is_some());
}

#[test]
fn test_clip_path_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"clip-path"));
}

#[test]
fn test_clip_path_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("clip-path"));
}

#[test]
fn test_clip_path_default_is_none() {
    let style = ComputedStyle::default();
    assert!(matches!(style.clip_path, ClipPathComputedValue::None));
}

// ── apply_initial_value 测试 ──

#[test]
fn test_clip_path_apply_initial() {
    let mut style = ComputedStyle::default();
    // 先设为非默认值
    apply_property_value(&mut style, "clip-path", "circle(50px)");
    // 应用初始值应重置为 none
    assert!(apply_initial_value(&mut style, "clip-path"));
    assert!(matches!(style.clip_path, ClipPathComputedValue::None));
}

// ── 继承测试 ──

#[test]
fn test_clip_path_not_inherited_from_parent() {
    let mut parent = ComputedStyle::default();
    apply_property_value(&mut parent, "clip-path", "inset(10px)");
    // clip-path 不继承，子元素应为默认值
    let child_default = ComputedStyle::default();
    assert!(matches!(child_default.clip_path, ClipPathComputedValue::None));
}

// ── 管线集成测试（解析→样式） ──

#[test]
fn test_clip_path_pipeline_inset_roundtrip() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip-path",
        "inset(5px 10px 15px 20px)"
    ));
    match &style.clip_path {
        ClipPathComputedValue::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            assert_eq!(*top, LengthValue::Px(5.0));
            assert_eq!(*right, LengthValue::Px(10.0));
            assert_eq!(*bottom, LengthValue::Px(15.0));
            assert_eq!(*left, LengthValue::Px(20.0));
            assert!(round.is_none());
        }
        _ => panic!("Expected Inset"),
    }
}

#[test]
fn test_clip_path_pipeline_circle_at_position() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "clip-path", "circle(30% at 50% 50%)"));
    match &style.clip_path {
        ClipPathComputedValue::Circle { radius, position } => {
            assert!(matches!(radius, ClipPathRadius::Length(LengthValue::Percentage(30.0))));
            let (x, y) = position.as_ref().unwrap();
            assert_eq!(*x, LengthValue::Percentage(50.0));
            assert_eq!(*y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected Circle"),
    }
}

#[test]
fn test_clip_path_pipeline_polygon_evenodd() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip-path",
        "polygon(evenodd, 0 0, 100% 0, 50% 100%)"
    ));
    match &style.clip_path {
        ClipPathComputedValue::Polygon { fill_rule, points } => {
            assert!(matches!(fill_rule, PolygonFillRule::EvenOdd));
            assert_eq!(points.len(), 3);
        }
        _ => panic!("Expected Polygon"),
    }
}

#[test]
fn test_clip_path_pipeline_polygon_fill_rule_is_case_insensitive() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "clip-path",
        "polygon(EvEnOdD, 0 0, 100% 0, 50% 100%)"
    ));
    assert!(matches!(
        style.clip_path,
        ClipPathComputedValue::Polygon {
            fill_rule: PolygonFillRule::EvenOdd,
            ..
        }
    ));
}
