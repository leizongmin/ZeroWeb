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

#[test]
fn test_r5176_perspective_stacking_context_transformable_gate() {
    // R5176（css-transforms-2 §perspective-property）：perspective ≠ none 建 SC。
    // none 恒为 Px(0.0) 哨兵（R5175 store clamp 后显式值 ≥1px）。块级元素显式
    // Block（真实引擎 UA 层 div→block；ComputedStyle::default() 是 Inline 初始值）。
    let mut s = ComputedStyle::default();
    s.display = zero_css_parser::values::DisplayValue::Block;
    s.perspective = zero_css_parser::values::LengthValue::Px(100.0);
    assert!(
        LayoutEngine::style_creates_stacking_context(false, &s),
        "perspective ≠ none on transformable element → SC"
    );

    // inline span 上的 perspective 不适用（非 transformable）→ 不建 SC
    //（perspective-untransformable-no-stacking-context：z:-1 abspos 子须画在
    // span 背景之后，即归祖先 SC step 2）。R4070 同款排除 Inline/Contents/None。
    let mut inline = ComputedStyle::default();
    inline.perspective = zero_css_parser::values::LengthValue::Px(100.0);
    inline.display = zero_css_parser::values::DisplayValue::Inline;
    assert!(!LayoutEngine::style_creates_stacking_context(false, &inline));

    // perspective: none（Px(0.0) 哨兵）→ 不建 SC
    let none = ComputedStyle::default();
    assert!(!LayoutEngine::style_creates_stacking_context(false, &none));
}
