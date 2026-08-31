//! R3858 回归测试：abspos 最近 positioned 祖先为**非根**元素且隔有 static 中间层时，
//! inset 相对该 positioned 祖先的 padding-box 解析（CSS §10.1.2），非静态父。
//!
//! taffy 0.7 把 absolute 子 inset 相对其静态父解析；`.relative > div > span{position:
//! absolute; bottom:0}` 场景 span 被放到 static 中间层底部而非 relative 祖先底。
//! driving：inline-replaced-width-015（绿色覆盖 span 落 y=166.6 非 216.6，红 img 露出）。
//! kill-switch `ZW_ABSPOS_NESTED_CB=0` 关闭本 fix 时本测试应失败（load-bearing）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// static 中间层场景：bottom:0 应相对非父 positioned 祖先解析。
#[test]
fn r3858_abspos_bottom0_resolves_against_non_parent_positioned_ancestor() {
    let html = r#"<html><body>
<div id="outer" style="position:relative; width:300px; height:100px; background:red">
  <div id="mid" style="width:300px; height:50px; background:yellow">
    <span id="abs" style="position:absolute; bottom:0; left:0; width:300px; height:50px; background:green"></span>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer_id = doc.get_element_by_id("outer").expect("outer");
    let abs_id = doc.get_element_by_id("abs").expect("abs");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer = find(&result.root, outer_id).unwrap();
    let abs = find(&result.root, abs_id).unwrap();
    // CB = outer padding-box（0..100 容器内）；bottom:0 + h=50 → abs 顶 = outer 顶 + 50。
    let expected = outer.y + 50.0;
    assert!(
        (abs.y - expected).abs() < 0.5,
        "R3858: bottom:0 应相对 positioned 祖先（outer）底缘解析：abs.y 应 = {:.1}，实际 {:.1}\
         （若 ≈ mid.y 说明 taffy 仍按 static 中间层解析）",
        expected,
        abs.y
    );
}

/// 直接子场景守卫：abspos 直连 positioned 父（taffy 已正确）不受本 pass 影响。
#[test]
fn r3858_direct_child_of_positioned_parent_unchanged() {
    let html = r#"<html><body>
<div id="outer" style="position:relative; width:300px; height:100px">
  <span id="abs" style="position:absolute; top:0; left:0; width:50px; height:50px"></span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer_id = doc.get_element_by_id("outer").expect("outer");
    let abs_id = doc.get_element_by_id("abs").expect("abs");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer = find(&result.root, outer_id).unwrap();
    let abs = find(&result.root, abs_id).unwrap();
    assert!(
        (abs.x - outer.x).abs() < 0.5 && (abs.y - outer.y).abs() < 0.5,
        "直连 positioned 父的 top:0/left:0 应保持 taffy 原解（abs 与 outer 同源点），\
         got abs=({:.1},{:.1}) outer=({:.1},{:.1})",
        abs.x,
        abs.y,
        outer.x,
        outer.y
    );
}
