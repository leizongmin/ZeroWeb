//! R5376 回归测试：abspos 约束方程 margin 侧（CSS2 §10.3.7）——left/width/right 全
//! definite 时 auto margin-left 按方程求解（单 auto 解 ml、双 auto 均分、负 slack →
//! ltr ml=0）。旧口径 auto margin 当 0 → 落过约束左锚。
//! driving：WPT abspos-non-replaced-width-margin-000「only ml auto」组（53 应 496）。
//! 骨架与 r5372 同谱系（嵌套 = static 中间层隔在 abspos 与 positioned 祖先之间）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<LayoutBox> {
    let mut stack = vec![root.clone()];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter().cloned());
    }
    None
}

/// 嵌套案骨架：outer(positioned CB，650 宽) > mid（static 中间层，content 原点在
/// outer content 系 x=60）> abs。断言 abs 盒位（outer content 系）= mid 偏移 + abs.x。
fn assert_nested_x(abs_style: &str, expected_x: f32, label: &str) {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px">
  <div id="mid" style="margin-left:7px; border-left:29px solid transparent; padding-left:24px; padding-right:14px; border-right:3px solid transparent; margin-right:18px;">
    <div id="abs" style="{abs_style}"></div>
  </div>
</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer_id = doc.get_element_by_id("outer").unwrap();
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer = find(&result.root, outer_id).unwrap();
    let abs = find(&result.root, abs_id).unwrap();
    let mid = &outer.children[0];
    let x = mid.x + mid.border_left + mid.padding_left + abs.x;
    assert!(
        (x - expected_x).abs() < 0.5,
        "{label}: 盒位 x 应 {expected_x:.1}，实际 {x:.1}（abs.x={:.1}）",
        abs.x
    );
}

/// 单 auto：ml = 650−3−8−19−153 = 467 → 盒位 x = 3+467 = 470（与 right 锚方程一致）。
/// 旧口径 auto 当 0 → x=3。
#[test]
fn r5376_nested_single_auto_margin_left_solves_equation() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:19px; width:153px; height:1px;",
        470.0,
        "单 auto ml 解方程",
    );
}

/// 双 auto：slack = 650−3−8−153 = 486 → ml = mr = 243 → x = 3+243 = 246（居中）。
#[test]
fn r5376_nested_both_auto_margins_split_slack() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:auto; width:153px; height:1px;",
        246.0,
        "双 auto 均分",
    );
}

/// 单 auto 负解：ml = 650−3−8−19−660 = −40 → x = 3−40 = −37（spec 允许负 margin 解）。
#[test]
fn r5376_nested_single_auto_margin_negative_solution() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:19px; width:660px; height:1px;",
        -37.0,
        "单 auto 负解",
    );
}

/// 守卫：width:auto 行不入臂（shrink-to-fit 未实现域）——回退旧口径 ml=0，x=3。
#[test]
fn r5376_width_auto_row_keeps_legacy_anchor() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:19px; height:1px;",
        3.0,
        "width:auto 回退",
    );
}

/// 守卫：definite margin 行走既有口径（R5372 已账），方程臂不触——ml:17 → x=20。
#[test]
fn r5376_definite_margin_row_unchanged() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px;",
        20.0,
        "definite margin 旧口径",
    );
}

/// root-CB 臂（根 positioned）：cb = html padding-box 800，abs 直挂 body 下（nearest
/// positioned 祖先 = 根）。ml = 800−3−8−19−153 = 617 → x = 620。
#[test]
fn r5376_root_cb_single_auto_margin_left_solves_equation() {
    let html = r#"<html style="position:relative"><body style="margin:0">
<div id="abs" style="position:absolute; left:3px; right:8px; margin-left:auto; margin-right:19px; width:153px; height:1px"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    assert!(
        (abs.x - 620.0).abs() < 0.5,
        "root-CB 单 auto ml 盒位应 620，实际 {:.1}",
        abs.x
    );
}
