//! R1277：float-lift（②）+ 显式高度守卫（④）回归测试。
//!
//! floats-006 谱系：非 BFC 容器 `#div1{height:200px}` 含 inline `<span>` + float 子。
//! ②（`ZW_FLOAT_LIFT_INLINE` default-on）：inline 级子不推进 flow_bottom → float 上提到
//! 容器顶部（CSS §9.5.1）。④（`ZW_FLOAT_RESPECT_HEIGHT` default-on）：显式高度容器不被
//! 「非 BFC content_bottom 收缩」塌缩（CSS §10.5 used height = 显式值）。
//!
//! R1273 曾推断 ② 应用后 div1 塌缩到 100 来自 `exclude_floats_from_non_bfc_auto_height`；
//! R1277 经 DIV1_TRACE 二分证伪——塌缩源是 `adjust_float_positions_with_context` 内部的
//! 非 BFC 收缩路径（float_positioning.rs），④ 守卫修复之。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::FloatValue;
use zero_style_system::StyleSystem;

/// 找到第一个含 float 子元素的、指定宽度的块级容器（模拟 floats-006 的 `#div1`）。
fn find_float_container(root: &LayoutBox, width: f32) -> Option<&LayoutBox> {
    if (root.width - width).abs() < 0.5 && root.children.iter().any(|c| !matches!(c.float, FloatValue::None)) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_float_container(child, width) {
            return Some(f);
        }
    }
    None
}

/// ②+④：非 BFC 显式高度容器（height:200）含 inline span + 2 floats 时，float 上提到
/// 顶部且容器高度不被塌缩（CSS §10.5 / §9.5.1）。
///
/// 旧实现（②+④ 缺失）：float 被 inline span 推到 rel_y=100（底部）+ 容器被收缩到 100。
#[test]
fn test_float_lift_keeps_explicit_height_container() {
    let html = r#"<html><body style="margin:0"><div style="height:200px;width:300px"><span style="font:100px/1 Ahem">X</span><div style="float:left;width:100px;height:100px"></div><div style="float:left;width:100px;height:100px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div1 = find_float_container(&result.root, 300.0).expect("should find #div1 (w=300 with float children)");
    // ④：显式 height:200 不被非 BFC content_bottom 收缩塌缩（旧实现塌缩到 100）。
    assert!(
        (div1.height - 200.0).abs() < 0.5,
        "explicit-height container must not collapse; expected 200, got {}",
        div1.height
    );

    // ②：float 上提到容器顶部（rel_y = float.y - div1.content_y ≈ 0，旧实现 ≈ 100）。
    let floats: Vec<&LayoutBox> = div1
        .children
        .iter()
        .filter(|c| !matches!(c.float, FloatValue::None))
        .collect();
    assert_eq!(floats.len(), 2, "should have 2 float children");
    for f in &floats {
        let rel_y = f.y - div1.content_y;
        assert!(
            rel_y.abs() < 1.0,
            "float should be lifted to container top (rel_y≈0), got rel_y={}",
            rel_y
        );
    }
}

/// ④ 守卫的 load-bearing 性：auto-height 容器（height:auto）仍应被非 BFC 收缩路径
/// 处理（float 不贡献高度）。即守卫只跳过 definite-height，不误伤 auto-height。
#[test]
fn test_explicit_height_guard_does_not_affect_auto_height() {
    // height:auto 的容器：float 在内部，容器应收缩（float 不贡献高度，CSS §10.5.1）。
    let html = r#"<html><body style="margin:0"><div style="width:300px"><div style="float:left;width:100px;height:100px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container = find_float_container(&result.root, 300.0).expect("should find container");
    // auto-height 非 BFC 容器：float 不贡献高度 → 容器收缩到 ~0（无 in-flow 子元素）。
    // （exclude_floats_from_non_bfc_auto_height + float_positioning 收缩共同作用。）
    assert!(
        container.height < 5.0,
        "auto-height non-BFC container should collapse (float contributes no height); got {}",
        container.height
    );
}
