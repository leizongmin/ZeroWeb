//! R1616：definite-height 容器不被 clear-float 撑高（float 应溢出）回归测试。
//!
//! floats-placement-006 谱系：容器 `height:100px width:100px`（definite）含
//! inline-block（50×100）+ float-right（50×100）+ float-left clear:both（30×50）。
//! float-left clear:both 被推到 y=100（清过 100 高的前置 float），其 border-box 底
//! y=150。`remeasure_text_with_float_exclusions` 此前按 `float_bottom=150` **无条件**
//! 扩容器到 150——违反 CSS §10.5/§10.6（显式高度容器不被 float 子覆盖，float 应溢出）。
//!
//! R1616 fix：仅 height:auto 容器才按 float/文本底扩展高度。definite-height 容器
//! 保持显式值（env `ZW_REMEASURE_FLOAT_DEFHEIGHT=0` 关闭回退旧行为）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::FloatValue;
use zero_style_system::StyleSystem;

/// 找到第一个含 float 子元素且指定宽度的块级容器（模拟 floats-placement-006 的
/// `.container`，width=100）。
fn find_float_container(root: &LayoutBox, width: f32) -> Option<&LayoutBox> {
    if (root.content_width - width).abs() < 0.5 && root.children.iter().any(|c| !matches!(c.float, FloatValue::None)) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_float_container(child, width) {
            return Some(f);
        }
    }
    None
}

/// R1616 load-bearing：definite-height 容器（height:100）含 cleared float（float-left
/// clear:both @y=100，底=150）时，容器高度保持 100（float 溢出），不被 remeasure 扩到 150。
///
/// 旧实现：remeasure_text_with_float_exclusions 无条件按 float_bottom 扩 → 容器 150。
#[test]
fn test_definite_height_container_not_expanded_by_cleared_float() {
    let html = r#"<html><body style="margin:0"><div style="width:100px;height:100px"><div style="display:inline-block;width:50px;height:100px"></div><div style="float:right;width:50px;height:100px"></div><div style="float:left;clear:both;width:30px;height:50px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container =
        find_float_container(&result.root, 100.0).expect("should find .container (w=100 with float children)");
    assert!(
        (container.content_height - 100.0).abs() < 0.5,
        "definite-height container must stay 100 (float overflows), got content_height={}",
        container.content_height
    );
}

/// 守卫不误伤 auto-height：height:auto 的 BFC 容器（overflow:hidden 建立 BFC）含 float
/// 子时，仍应按 float 底扩展高度（CSS §10.6.7 auto-height BFC 含 float）。
#[test]
fn test_auto_height_bfc_container_still_expanded_by_float() {
    // overflow:hidden 建立 BFC + height:auto：容器应扩展到含 float（底=100）。
    let html = r#"<html><body style="margin:0"><div style="overflow:hidden;width:100px"><div style="float:left;width:50px;height:100px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container =
        find_float_container(&result.root, 100.0).expect("should find BFC container (w=100 with float child)");
    assert!(
        (container.content_height - 100.0).abs() < 1.0,
        "auto-height BFC container should expand to contain float (≈100), got content_height={}",
        container.content_height
    );
}
