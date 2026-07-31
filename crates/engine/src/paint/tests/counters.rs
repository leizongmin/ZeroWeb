//! CSS 计数器渲染和 TransformPrimitive 单元测试。

use zero_css_parser::values::{CounterActionValue, LengthValue, TransformFunction, TransformValue};
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

use super::super::helpers::compute_transform_matrix;
use super::super::painter::Painter;

// ── update_counters 测试 ────────────────────────────────────────────

#[test]
fn test_counter_reset_sets_value() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_reset = vec![CounterActionValue {
        name: "section".to_string(),
        value: Some(0),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("section"), Some(0));
}

#[test]
fn test_counter_reset_default_is_zero() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_reset = vec![CounterActionValue {
        name: "item".to_string(),
        value: None, // 默认 0
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("item"), Some(0));
}

#[test]
fn test_counter_reset_to_custom_value() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_reset = vec![CounterActionValue {
        name: "chapter".to_string(),
        value: Some(5),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("chapter"), Some(5));
}

#[test]
fn test_counter_increment_adds_one() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_increment = vec![CounterActionValue {
        name: "item".to_string(),
        value: None, // 默认 +1
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("item"), Some(1));
}

#[test]
fn test_counter_increment_custom_value() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_increment = vec![CounterActionValue {
        name: "step".to_string(),
        value: Some(5),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("step"), Some(5));
}

#[test]
fn test_counter_increment_accumulates() {
    let mut painter = Painter::new();

    for _ in 0..3 {
        let mut style = ComputedStyle::default();
        style.counter_increment = vec![CounterActionValue {
            name: "item".to_string(),
            value: None,
        }];
        painter.update_counters(&style);
    }
    assert_eq!(painter.get_counter("item"), Some(3));
}

#[test]
fn test_counter_set_overwrites() {
    let mut painter = Painter::new();

    // 先递增到 3
    let mut style = ComputedStyle::default();
    style.counter_increment = vec![CounterActionValue {
        name: "item".to_string(),
        value: Some(3),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("item"), Some(3));

    // counter-set 覆盖为 10
    let mut style2 = ComputedStyle::default();
    style2.counter_set = vec![CounterActionValue {
        name: "item".to_string(),
        value: Some(10),
    }];
    painter.update_counters(&style2);
    assert_eq!(painter.get_counter("item"), Some(10));
}

#[test]
fn test_counter_reset_then_increment() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_reset = vec![CounterActionValue {
        name: "item".to_string(),
        value: Some(0),
    }];
    style.counter_increment = vec![CounterActionValue {
        name: "item".to_string(),
        value: None,
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("item"), Some(1));
}

#[test]
fn test_counter_reset_clears_previous() {
    let mut painter = Painter::new();

    let mut style = ComputedStyle::default();
    style.counter_increment = vec![CounterActionValue {
        name: "section".to_string(),
        value: Some(5),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("section"), Some(5));

    let mut style2 = ComputedStyle::default();
    style2.counter_reset = vec![CounterActionValue {
        name: "section".to_string(),
        value: Some(0),
    }];
    painter.update_counters(&style2);
    assert_eq!(painter.get_counter("section"), Some(0));
}

#[test]
fn test_multiple_counters_independent() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_increment = vec![
        CounterActionValue {
            name: "section".to_string(),
            value: None,
        },
        CounterActionValue {
            name: "figure".to_string(),
            value: Some(2),
        },
    ];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("section"), Some(1));
    assert_eq!(painter.get_counter("figure"), Some(2));
}

#[test]
fn test_counter_nonexistent_returns_none() {
    let painter = Painter::new();
    assert_eq!(painter.get_counter("nonexistent"), None);
}

#[test]
fn test_counter_negative_increment() {
    let mut painter = Painter::new();
    let mut style = ComputedStyle::default();
    style.counter_reset = vec![CounterActionValue {
        name: "count".to_string(),
        value: Some(10),
    }];
    style.counter_increment = vec![CounterActionValue {
        name: "count".to_string(),
        value: Some(-3),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("count"), Some(7));
}

#[test]
fn test_counter_processing_order_reset_set_increment() {
    let mut painter = Painter::new();

    // 先 setup: counter = 5
    let mut style = ComputedStyle::default();
    style.counter_increment = vec![CounterActionValue {
        name: "x".to_string(),
        value: Some(5),
    }];
    painter.update_counters(&style);
    assert_eq!(painter.get_counter("x"), Some(5));

    // 同一节点 reset + set + increment
    let mut style2 = ComputedStyle::default();
    style2.counter_reset = vec![CounterActionValue {
        name: "x".to_string(),
        value: Some(0),
    }];
    style2.counter_set = vec![CounterActionValue {
        name: "x".to_string(),
        value: Some(100),
    }];
    style2.counter_increment = vec![CounterActionValue {
        name: "x".to_string(),
        value: Some(1),
    }];
    painter.update_counters(&style2);
    // reset → 0, set → 100, increment → 101
    assert_eq!(painter.get_counter("x"), Some(101));
}

// ── TransformPrimitive 与 transform-origin 测试 ────────────────────

#[test]
fn test_transform_with_custom_origin_px() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::Scale(2.0, None)]);
    style.transform_origin_x = LengthValue::Px(0.0);
    style.transform_origin_y = LengthValue::Px(0.0);
    let rect = Rect::new(50.0, 50.0, 100.0, 100.0);

    let tp = compute_transform_matrix(&style, &rect).expect("should generate transform");
    assert!((tp.origin_x - 50.0).abs() < 0.1);
    assert!((tp.origin_y - 50.0).abs() < 0.1);
    assert!((tp.a - 2.0).abs() < 0.01);
    assert!((tp.d - 2.0).abs() < 0.01);
}

#[test]
fn test_transform_matrix_rotate_scale() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Scale(2.0, None),
        TransformFunction::Rotate(90.0),
    ]);
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tp = compute_transform_matrix(&style, &rect).expect("should generate transform");

    // Scale(2,2) * Rotate(90°) = [[0, -2], [2, 0]]
    assert!(tp.a.abs() < 0.01, "a should be ~0, got {}", tp.a);
    assert!((tp.b - 2.0).abs() < 0.01, "b should be ~2, got {}", tp.b);
    assert!((tp.c + 2.0).abs() < 0.01, "c should be ~-2, got {}", tp.c);
    assert!(tp.d.abs() < 0.01, "d should be ~0, got {}", tp.d);
}

#[test]
fn test_transform_matrix_skew() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::Skew(45.0, Some(30.0))]);
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tp = compute_transform_matrix(&style, &rect).expect("should generate transform");

    let tan30 = 30.0_f64.to_radians().tan() as f32;
    assert!((tp.a - 1.0).abs() < 0.01);
    assert!((tp.b - tan30).abs() < 0.01, "b should be tan(30°), got {}", tp.b);
    assert!((tp.c - 1.0).abs() < 0.01, "c should be tan(45°)=1, got {}", tp.c);
    assert!((tp.d - 1.0).abs() < 0.01);
}

/// R2294：translate(%) 相对元素 border-box 求值（centering pattern translate(-50%,-50%)）。
#[test]
fn test_transform_matrix_translate_percent() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::TranslateMixed(-50.0, true, -50.0, true)]);
    style.transform_origin_x = LengthValue::Px(0.0);
    style.transform_origin_y = LengthValue::Px(0.0);
    // 100×100 box：-50% 应解析为 -50px（width/height 的 50%）。
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tp = compute_transform_matrix(&style, &rect).expect("percent translate should yield a transform");
    assert!((tp.a - 1.0).abs() < 0.01);
    assert!((tp.d - 1.0).abs() < 0.01);
    assert!(
        (tp.tx + 50.0).abs() < 0.01,
        "tx should be -50 (50% of 100), got {}",
        tp.tx
    );
    assert!(
        (tp.ty + 50.0).abs() < 0.01,
        "ty should be -50 (50% of 100), got {}",
        tp.ty
    );

    // 非 1:1 box：tx 按 width、ty 按 height 独立解析。
    let rect2 = Rect::new(0.0, 0.0, 200.0, 50.0);
    let tp2 = compute_transform_matrix(&style, &rect2).expect("should generate transform");
    assert!(
        (tp2.tx + 100.0).abs() < 0.01,
        "tx should be -100 (50% of 200), got {}",
        tp2.tx
    );
    assert!(
        (tp2.ty + 25.0).abs() < 0.01,
        "ty should be -25 (50% of 50), got {}",
        tp2.ty
    );
}
