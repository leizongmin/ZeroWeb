//! R2302：`isolation: isolate` 建立堆叠上下文（CSS Compositing §3）单测。
//!
//! 旧 impl 仅 parse/store `isolation` 字段，`creates_stacking_context` 判定不含
//! `Isolate` → isolation 无视觉效果（mix-blend-mode 不被隔离）。R2302 把判定抽出为
//! `LayoutEngine::style_creates_stacking_context` 并补 Isolate 触发器。

use super::LayoutEngine;
use zero_style_system::{ComputedStyle, IsolationValue, ZIndexValue};

#[test]
fn test_isolation_isolate_triggers_stacking_context() {
    // 默认（isolation:auto、非 positioned、opacity=1）→ 不建 SC
    let s = ComputedStyle::default();
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));
    assert!(!LayoutEngine::style_creates_stacking_context(true, &s));

    // isolation: isolate → 建 SC（即使非 positioned）
    let mut s = ComputedStyle::default();
    s.isolation = IsolationValue::Isolate;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // isolation: auto → 不建 SC
    s.isolation = IsolationValue::Auto;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));
}

#[test]
fn test_other_stacking_context_triggers_preserved() {
    // positioned + 显式整数 z-index → SC（z-index:auto 非 SC）
    let mut s = ComputedStyle::default();
    s.z_index = ZIndexValue::Integer(5);
    assert!(LayoutEngine::style_creates_stacking_context(true, &s));
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s)); // 非 positioned 不建

    // opacity < 1 → SC
    let mut s = ComputedStyle::default();
    s.opacity = 0.5;
    assert!(LayoutEngine::style_creates_stacking_context(false, &s));

    // opacity = 1 → 不建
    let mut s = ComputedStyle::default();
    s.opacity = 1.0;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &s));
}
