//! R1781 回归守卫：abspos semi-replaced（`<input>`）全-inset + auto 尺寸应 stretch 填满 CB。
//!
//! 背景（WPT `position-absolute-semi-replaced-stretch-{button,input,other}`，csswg-drafts #6789）：
//! semi-replaced 元素（button/input/select/textarea）position:absolute + 双长度 inset + width/height:auto
//! 应如非替换元素一样 stretch 填满 CB 减 inset。R1659 给 `<input>` 加了 UA 固有 width（text~148px），
//! 但 author `width:auto` 经 cascade（Author > UserAgent origin）覆盖 → Auto → taffy 全-inset stretch。
//!
//! R1781 实证：`<button>`（无 UA width）+ 简单 input type 已 <1%（stretch + 简单 UA 盒近似原生控件）；
//! input 测试 23% diff 残余 = 复杂 input type（color/date/file/range）的原生控件外观（R1695 native-widget），
//! **非** stretch 几何缺口。本测试锁定 stretch 几何正确性，防未来 R1659 UA width 或 cascade 改动
//! 误破 stretch（被误判为可修 lever）。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

/// 递归找首个满足谓词的 LayoutBox。
fn find_first<'a>(
    box_node: &'a crate::types::LayoutBox,
    pred: &dyn Fn(&crate::types::LayoutBox) -> bool,
) -> Option<&'a crate::types::LayoutBox> {
    if pred(box_node) {
        return Some(box_node);
    }
    for c in &box_node.children {
        if let Some(b) = find_first(c, pred) {
            return Some(b);
        }
    }
    None
}

/// abspos `<input>` 全-inset + auto 尺寸应 stretch 填满 CB（csswg-drafts #6789）。
#[test]
fn test_abspos_semi_replaced_input_stretches_to_cb() {
    // .cb = position:relative, border:3px, 150×100（border-box），display:inline-block。
    // input.abs = position:absolute, top/right/bottom/left:3px, width/height:auto, box-sizing:border-box。
    // 期望：input stretch = CB padding-box(150×100) - inset(3+3) = 144×94。
    let html = r#"<html><body style="margin:0">
<div style="position:relative;border:3px solid black;height:100px;width:150px;display:inline-block;vertical-align:top">
<input type="text" style="margin:0;position:absolute;box-sizing:border-box;top:3px;right:3px;bottom:3px;left:3px;width:auto;height:auto" value="text">
</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let input_box = find_first(&result.root, &|b: &crate::types::LayoutBox| {
        b.node_id.and_then(|nid| doc.get(nid)).is_some_and(
            |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("input")),
        )
    });
    let inp = input_box.expect("input LayoutBox should exist");
    assert!(inp.is_absolute, "input should be position:absolute");
    // author width:auto 经 cascade 覆盖 R1659 UA width:148px → Auto → stretch。
    assert!(
        (inp.width - 144.0).abs() < 2.0,
        "input.abs 应 stretch 到 144px (CB 150 - inset 3+3)，实际 {}；若 ≈148 说明 R1659 UA width 未被 author auto 覆盖（cascade 回归）",
        inp.width
    );
    assert!(
        (inp.height - 94.0).abs() < 2.0,
        "input.abs 应 stretch 到 94px (CB 100 - inset 3+3)，实际 {}",
        inp.height
    );
}
