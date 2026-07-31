//! R2309：CSS3/4 堆叠上下文触发器补全单测。
//!
//! 旧 `style_creates_stacking_context` 仅含 positioned+z-index / opacity<1 / isolation:isolate。
//! CSS Filter Effects / CSS Backdrop Filter / CSS Masking / CSS Will Change / CSS Compositing §3.5 /
//! CSS Containment §4 规定：非 none 的 `filter`/`backdrop-filter`/`clip-path`、非 auto 的
//! `will-change`、非 normal 的 `mix-blend-mode`、含 paint/layout 的 `contain` 亦建立堆叠上下文，
//! 使后代与祖先背景隔离。R2309 补这些触发器（`transform` 因产品 fixture 有用例，待独立 A/B 切片）。
//!
//! 关键：`filter`/`backdrop-filter`/`will-change` 经解析层 `none`/`auto` → 空 Vec（R2306/R2308），
//! 故 SC 判定用 `!is_empty()` 等价于 spec 的「值非 none」。

use super::LayoutEngine;
use zero_css_parser::values::ClipPathRadius;
use zero_style_system::{
    ClipPathComputedValue, ComputedStyle, ContainComputedValue, FilterComputedValue, MixBlendModeComputedValue,
    WillChangeValue,
};

#[test]
fn test_default_style_no_stacking_context() {
    // 全默认（非 positioned、opacity=1、isolation:auto、filter/will-change 空、mix-blend normal、
    // clip-path none、contain none）→ 不建 SC
    let s = ComputedStyle::default();
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));
    assert!(!LayoutEngine::style_creates_stacking_context(true, &s));
}

#[test]
fn test_r2309_property_triggers_create_stacking_context() {
    // will-change（非空）→ SC（CSS Will Change）
    let mut s = ComputedStyle::default();
    s.will_change = vec![WillChangeValue::ScrollPosition];
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // filter（非 none）→ SC（CSS Filter Effects）
    let mut s = ComputedStyle::default();
    s.filter = vec![FilterComputedValue::Blur(2.0)];
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // backdrop-filter（非 none）→ SC（CSS Backdrop Filter）
    let mut s = ComputedStyle::default();
    s.backdrop_filter = vec![FilterComputedValue::Blur(2.0)];
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // mix-blend-mode（非 normal）→ SC（CSS Compositing §3.5）
    let mut s = ComputedStyle::default();
    s.mix_blend_mode = MixBlendModeComputedValue::Multiply;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // clip-path（非 none）→ SC（CSS Masking）
    let mut s = ComputedStyle::default();
    s.clip_path = ClipPathComputedValue::Circle {
        radius: ClipPathRadius::ClosestSide,
        position: None,
    };
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: paint → SC（CSS Containment §4）
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Paint;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: layout → SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Layout;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: strict（含 paint+layout）→ SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Strict;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: content（含 paint+layout）→ SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Content;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));
}

#[test]
fn test_r2309_default_values_do_not_create_stacking_context() {
    // 默认值均不触发：空 Vec / Normal / None
    let s = ComputedStyle::default();
    assert!(s.will_change.is_empty());
    assert!(s.filter.is_empty());
    assert!(s.backdrop_filter.is_empty());
    assert!(matches!(s.mix_blend_mode, MixBlendModeComputedValue::Normal));
    assert!(matches!(s.clip_path, ClipPathComputedValue::None));
    assert!(!s.contain.has_paint() && !s.contain.has_layout());
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: size（不含 paint/layout）→ 不建 SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Size;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: style（不含 paint/layout）→ 不建 SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::Style;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));

    // contain: none → 不建 SC
    let mut s = ComputedStyle::default();
    s.contain = ContainComputedValue::None;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));
}
