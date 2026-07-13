//! R1398 回归测试：abspos 元素的 containing block 是最近 positioned 祖先的
//! **padding box**（CSS §10.1.4）。taffy 0.12 错误地把祖先 **border** 计入 abspos
//! 的 location（loc.x = inset + cb_border），导致 abspos 元素整体偏移祖先 border 宽度。
//! 驱动案 css-position/position-absolute-semi-replaced-stretch-button（3.17%→<1% FLIP）。
//!
//! 构造 `.cb{position:relative; border:3px; width; height} > .abs{position:absolute;
//! left:3px; top:3px}`：无 fix 时 abs.x=6（inset 3 + cb border 3），应 3（padding-box CB）。
//! kill-switch `ZW_ABSPOS_CB_BORDER=0` 关闭本 fix 时本测试应失败（load-bearing）。

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

#[test]
fn r1398_abspos_x_y_exclude_positioned_ancestor_border() {
    // positioned 祖先带 3px border；abspos 子 left/top=3px。
    let html = r#"<html><body>
<div id="cb" style="position:relative; border:3px solid black; width:200px; height:200px">
  <div id="abs" style="position:absolute; left:3px; top:3px; width:50px; height:50px"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").expect("abs");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    println!(
        "R1398 abs x={:.1} y={:.1} (期望 3,3 — padding-box CB，不含祖先 border)",
        abs.x, abs.y
    );
    // padding-box CB：left/top=3px 直接相对 padding box，不应叠加祖先 border(3)→6。
    assert!(
        (abs.x - 3.0).abs() < 0.5 && (abs.y - 3.0).abs() < 0.5,
        "R1398: abspos 应相对 padding-box CB 定位 (3,3)，got ({:.1},{:.1}) — \
         若 ~(6,6) 说明祖先 border 被错误计入 abspos loc（taffy 0.12 bug 未修）",
        abs.x,
        abs.y
    );
}

#[test]
fn r1398_abspos_zero_border_ancestor_unchanged() {
    // 守卫：border:0 的 positioned 祖先，abspos 定位不受本 fix 影响（border=0 无偏移可减）。
    // 确保本 fix 不破坏已正确的 border:0 案例。
    let html = r#"<html><body>
<div id="cb" style="position:relative; width:200px; height:200px">
  <div id="abs" style="position:absolute; left:5px; top:5px; width:50px; height:50px"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").expect("abs");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    assert!(
        (abs.x - 5.0).abs() < 0.5 && (abs.y - 5.0).abs() < 0.5,
        "R1398 守卫：border:0 祖先下 abspos 应 (5,5)，got ({:.1},{:.1})",
        abs.x,
        abs.y
    );
}
