//! R2234：CSS Logical `float`/`clear` 的 `inline-start`/`inline-end` 按方向解析为物理值。
use super::*;
use zero_css_parser::values::{ClearValue, FloatValue};

#[test]
fn test_resolve_float_physical_ltr() {
    // LTR：inline-start→left，inline-end→right；物理值原样。
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineStart, false),
        FloatValue::Left
    );
    assert_eq!(resolve_float_physical(&FloatValue::InlineEnd, false), FloatValue::Right);
    assert_eq!(resolve_float_physical(&FloatValue::Left, false), FloatValue::Left);
    assert_eq!(resolve_float_physical(&FloatValue::Right, false), FloatValue::Right);
    assert_eq!(resolve_float_physical(&FloatValue::None, false), FloatValue::None);
}

#[test]
fn test_resolve_float_physical_rtl() {
    // RTL：inline-start→right，inline-end→left；物理值原样。
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineStart, true),
        FloatValue::Right
    );
    assert_eq!(resolve_float_physical(&FloatValue::InlineEnd, true), FloatValue::Left);
    // 物理值不受 direction 影响。
    assert_eq!(resolve_float_physical(&FloatValue::Left, true), FloatValue::Left);
    assert_eq!(resolve_float_physical(&FloatValue::Right, true), FloatValue::Right);
}

#[test]
fn test_resolve_clear_physical_ltr_rtl() {
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineStart, false),
        ClearValue::Left
    );
    assert_eq!(resolve_clear_physical(&ClearValue::InlineEnd, false), ClearValue::Right);
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineStart, true),
        ClearValue::Right
    );
    assert_eq!(resolve_clear_physical(&ClearValue::InlineEnd, true), ClearValue::Left);
    // 物理值 / Both / None 不受影响。
    assert_eq!(resolve_clear_physical(&ClearValue::Both, true), ClearValue::Both);
    assert_eq!(resolve_clear_physical(&ClearValue::None, true), ClearValue::None);
}
