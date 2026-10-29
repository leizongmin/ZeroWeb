//! R3345 deep-review 修复回归测试（zero-style-system）。
//!
//! 本轮 deep-review 发现并修复的数值属性 finite 性 bug 的常驻断言：
//!
//! **flex-shrink longhand 拒绝 Infinity/NaN；flex-grow 语义其后修正**：
//! apply.rs 的 longhand 路径 `value.parse::<f64>()` 仅检 `v >= 0.0`——
//! `Infinity >= 0.0` 为真 → 存储无穷大值。无穷 grow/shrink 因子经 layout-engine
//! 的 Taffy flex 算法（converter/mod.rs:323）可致 flex 分配计算异常/NaN 传播。
//! **4af441b8e 语义修正（css-values-4 `<number [0,∞]>` 上界含 ∞ +
//! css-flexbox-1 §resolve-flexible-lengths；WPT flex-grow-009）**：flex-grow
//! 因子放行 +∞（taffy grow 臂有 ∞ 语义，calc(infinity) 吸收全部 free space），
//! R3345 对 grow 的旧拒绝断言作废；shrink 值域同含 ∞ 但 taffy shrink 臂无
//! ∞ 语义（scaled factor ∞ 比例 NaN）——维持拒绝。
//! // https://www.w3.org/TR/css-flexbox-1/#flex-grow-property

#![allow(clippy::float_cmp)]

use crate::property::ComputedStyle;
use crate::property::apply::apply_property_value;

// ── flex-grow 接受 +∞（4af441b8e 语义修正）；flex-shrink 仍拒绝 ───────────

#[test]
fn test_flex_grow_accepts_infinity_semantics_update() {
    let mut style = ComputedStyle::default();
    // 4af441b8e（css-values-4 <number [0,∞]> 上界含 ∞ + css-flexbox-1
    // §resolve-flexible-lengths；WPT flex-grow-009 calc(infinity) 吸收全部
    // free space）：R3345 旧定性「flex-grow: Infinity 非法」证伪——grow 因子
    // 放行 +∞（taffy grow 臂有 ∞ 语义）。
    let applied = apply_property_value(&mut style, "flex-grow", "Infinity");
    assert!(applied, "flex-grow: Infinity 按 4af441b8e 语义应接受");
    assert!(
        style.flex_grow.is_infinite(),
        "flex-grow: Infinity 应存储为 +∞（实际 {}）",
        style.flex_grow
    );
}

#[test]
fn test_flex_grow_accepts_scientific_overflow_semantics_update() {
    let mut style = ComputedStyle::default();
    // 1e999 → f64 溢出为 +∞，同上按 <number [0,∞]> 接受（原 R3345 断言随
    // 4af441b8e 语义修正作废）。
    assert!(apply_property_value(&mut style, "flex-grow", "1e999"));
    assert!(
        style.flex_grow.is_infinite(),
        "flex-grow: 1e999（溢出为 inf）应存储为 +∞（实际 {}）",
        style.flex_grow
    );
}

#[test]
fn test_flex_shrink_rejects_infinity_r3345() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "flex-shrink", "Infinity"));
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
    assert!(!apply_property_value(&mut style, "flex-grow", "NaN"));
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
    assert!(!apply_property_value(&mut style, "flex-grow", "-1"));
    assert_eq!(style.flex_grow, 0.0, "flex-grow 负值须拒绝，保持初值 0");
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "flex-shrink", "-0.5"));
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
