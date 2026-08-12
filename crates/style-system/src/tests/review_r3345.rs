//! R3345 deep-review 修复回归测试（zero-style-system）。
//!
//! 本轮 deep-review 发现并修复的数值属性 finite 性 bug 的常驻断言：
//!
//! **flex-grow / flex-shrink longhand 接受 Infinity**：apply.rs 的 longhand 路径
//! `value.parse::<f64>()` 仅检 `v >= 0.0`——`Infinity >= 0.0` 为真 → 存储无穷大值。
//! `flex_grow` 经 `style.flex_grow as f32` 喂入 layout-engine 的 Taffy flex 算法
//!（converter/mod.rs:323），无穷大 grow factor 可致 flex 分配计算异常/NaN 传播。
//! CSS Flexbox §7.3.1/§7.3.2 要求 flex-grow/shrink 为 `<number>`（有限非负）。
//! **不一致证据**：`flex` 简写（shorthand/mod.rs:1123）用 `is_finite()` 严格拒绝
//! Infinity/NaN，而 longhand 路径无此检查——同属性简写严、longhand 松。改 longhand
//! 路径加 `v.is_finite()` 前置。
//! // https://www.w3.org/TR/css-flexbox-1/#flex-grow-property

#![allow(clippy::float_cmp)]

use crate::property::ComputedStyle;
use crate::property::apply::apply_property_value;

// ── flex-grow / flex-shrink 须拒绝 Infinity / NaN ───────────────────────

#[test]
fn test_flex_grow_rejects_infinity_r3345() {
    let mut style = ComputedStyle::default();
    // Infinity 须被拒绝（v >= 0.0 对 inf 为真，但 is_finite 为假 → 修复前误存）。
    let applied = apply_property_value(&mut style, "flex-grow", "Infinity");
    // 修复前：applied=true 且 style.flex_grow=Infinity（bug）。修复后：applied=true 但
    // flex_grow 保持初值 0.0（非法值按未声明处理，不赋值）。
    assert!(
        style.flex_grow.is_finite(),
        "flex-grow: Infinity 不得存储为无穷大（实际 {}）",
        style.flex_grow
    );
    let _ = applied;
}

#[test]
fn test_flex_grow_rejects_scientific_overflow_r3345() {
    let mut style = ComputedStyle::default();
    // 1e999 → f64 溢出为 Infinity（Rust parse 接受）。
    apply_property_value(&mut style, "flex-grow", "1e999");
    assert!(
        style.flex_grow.is_finite(),
        "flex-grow: 1e999（溢出为 inf）不得存储为无穷大（实际 {}）",
        style.flex_grow
    );
}

#[test]
fn test_flex_shrink_rejects_infinity_r3345() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-shrink", "Infinity");
    assert!(
        style.flex_shrink.is_finite(),
        "flex-shrink: Infinity 不得存储为无穷大（实际 {}）",
        style.flex_shrink
    );
}

#[test]
fn test_flex_grow_rejects_nan_r3345() {
    let mut style = ComputedStyle::default();
    // NaN >= 0.0 为假，旧实现已不赋值，但仍加测守护（修复须保持 NaN 拒绝）。
    apply_property_value(&mut style, "flex-grow", "NaN");
    assert!(
        !style.flex_grow.is_nan(),
        "flex-grow: NaN 不得存储为 NaN（实际 {}）",
        style.flex_grow
    );
}

#[test]
fn test_flex_grow_normal_values_unchanged_r3345() {
    // 合法值不得被破坏。
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-grow", "2");
    assert_eq!(style.flex_grow, 2.0);
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-grow", "0.5");
    assert_eq!(style.flex_grow, 0.5);
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-grow", "0");
    assert_eq!(style.flex_grow, 0.0);
}

#[test]
fn test_flex_grow_negative_still_rejected_r3345() {
    // 负值仍须拒绝（CSS Flexbox §7.3.1）。
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-grow", "-1");
    assert_eq!(style.flex_grow, 0.0, "flex-grow 负值须拒绝，保持初值 0");
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-shrink", "-0.5");
    // flex-shrink 初值为 1.0（CSS 初始值）。
    assert_eq!(style.flex_shrink, 1.0, "flex-shrink 负值须拒绝，保持初值 1");
}

#[test]
fn test_flex_shrink_normal_values_unchanged_r3345() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-shrink", "3");
    assert_eq!(style.flex_shrink, 3.0);
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "flex-shrink", "0");
    assert_eq!(style.flex_shrink, 0.0);
}
