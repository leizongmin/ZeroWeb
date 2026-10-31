//! R2234：CSS Logical `float`/`clear` 的 `inline-start`/`inline-end` 按方向解析为物理值。
use super::*;
use zero_css_parser::values::{ClearValue, FloatValue};

#[test]
fn test_resolve_float_physical_ltr() {
    // LTR：inline-start→left，inline-end→right；物理值原样。
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineStart, &DirectionValue::Ltr),
        FloatValue::Left
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineEnd, &DirectionValue::Ltr),
        FloatValue::Right
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::Left, &DirectionValue::Ltr),
        FloatValue::Left
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::Right, &DirectionValue::Ltr),
        FloatValue::Right
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::None, &DirectionValue::Ltr),
        FloatValue::None
    );
}

#[test]
fn test_resolve_float_physical_rtl() {
    // RTL：inline-start→right，inline-end→left；物理值原样。
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineStart, &DirectionValue::Rtl),
        FloatValue::Right
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::InlineEnd, &DirectionValue::Rtl),
        FloatValue::Left
    );
    // 物理值不受 direction 影响。
    assert_eq!(
        resolve_float_physical(&FloatValue::Left, &DirectionValue::Rtl),
        FloatValue::Left
    );
    assert_eq!(
        resolve_float_physical(&FloatValue::Right, &DirectionValue::Rtl),
        FloatValue::Right
    );
}

#[test]
fn test_resolve_clear_physical_ltr_rtl() {
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineStart, &DirectionValue::Ltr),
        ClearValue::Left
    );
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineEnd, &DirectionValue::Ltr),
        ClearValue::Right
    );
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineStart, &DirectionValue::Rtl),
        ClearValue::Right
    );
    assert_eq!(
        resolve_clear_physical(&ClearValue::InlineEnd, &DirectionValue::Rtl),
        ClearValue::Left
    );
    // 物理值 / Both / None 不受影响。
    assert_eq!(
        resolve_clear_physical(&ClearValue::Both, &DirectionValue::Rtl),
        ClearValue::Both
    );
    assert_eq!(
        resolve_clear_physical(&ClearValue::None, &DirectionValue::Rtl),
        ClearValue::None
    );
}

/// R5356：CSS Logical §float-clear — float/clear 逻辑值的物理映射取**包含块**
///（最近块容器祖先）的 direction，与元素自身 direction 无关。块容器父直接命中；
/// `display:inline` / `display:contents` 祖先不建包含块、跳过上溯；链上无带样式
/// 块容器祖先 → None（调用方回退元素自身 direction）。driving: WPT
/// logical-values-float-clear-reftest（float-direction ≠ container-direction 组）。
#[test]
fn test_r5356_containing_block_direction_walk() {
    use zero_style_system::{DirectionValue, DisplayValue};

    let mut doc = zero_dom::Document::new();
    let container = doc.create_element("div");
    let inline_wrap = doc.create_element("span");
    let contents_wrap = doc.create_element("div");
    let float_el = doc.create_element("div");
    doc.append_child(doc.root(), container).unwrap();
    doc.append_child(container, inline_wrap).unwrap();
    doc.append_child(inline_wrap, contents_wrap).unwrap();
    doc.append_child(contents_wrap, float_el).unwrap();

    let mut styles = std::collections::HashMap::new();
    // 容器：块容器 + rtl —— float 元素自身 ltr 不参与。
    let mut c = ComputedStyle::default();
    c.display = DisplayValue::Block;
    c.direction = DirectionValue::Rtl;
    styles.insert(container, c);
    // inline 包装（自身 ltr）：非块容器，须跳过。
    let mut i = ComputedStyle::default();
    i.display = DisplayValue::Inline;
    i.direction = DirectionValue::Ltr;
    styles.insert(inline_wrap, i);
    // contents 包装（自身 rtl）：盒不生成，须跳过。
    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Contents;
    w.direction = DirectionValue::Rtl;
    styles.insert(contents_wrap, w);

    let d = super::super::containing_block_direction(&doc, &styles, float_el);
    assert_eq!(
        d,
        Some(DirectionValue::Rtl),
        "must resolve to the nearest block container ancestor"
    );

    // 无带样式块容器祖先 → None（元素自身不回填进 Option）。
    let orphan = doc.create_element("div");
    doc.append_child(doc.root(), orphan).unwrap();
    assert_eq!(
        super::super::containing_block_direction(&doc, &std::collections::HashMap::new(), orphan),
        None
    );

    // 物理值解析直接消费包含块 direction：容器 rtl → inline-start = right。
    assert_eq!(
        super::super::resolve_float_physical(&FloatValue::InlineStart, d.as_ref().unwrap()),
        FloatValue::Right
    );
}
