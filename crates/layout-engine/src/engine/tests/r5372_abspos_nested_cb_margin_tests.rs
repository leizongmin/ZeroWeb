//! R5372 回归测试：嵌套 CB 重解析（static 中间层隔在 abspos 与 positioned 祖先之间）
//! 的长度 inset 定位方程计入对应 margin（CSS2 §10.3.7/§10.6.4）。
//!
//! taffy 直挂（parent 即 CB）margin 解析正确；R3858 嵌套路径的锚定写点丢 margin 项
//!（ltr 长度锚丢 +ml、right 锚丢 −mr）。taffy 直挂三案为守卫基线。
//! driving：WPT abspos-non-replaced-width-margin-000（ltr 锚 53 应 70）。
//! 嵌套案 load-bearing：kill-switch `ZW_ABSPOS_NESTED_CB=0` 时应失败。

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

/// 布局并取两个 id 元素盒。harness `compute_styles(&doc, &[])` 不吃文档 `<style>` 块，
/// 样式全 inline。
fn layout_two(html: &str, id_a: &str, id_b: &str) -> (LayoutBox, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let node_a = doc.get_element_by_id(id_a).expect("元素存在");
    let node_b = doc.get_element_by_id(id_b).expect("元素存在");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (
        find(&result.root, node_a).unwrap().clone(),
        find(&result.root, node_b).unwrap().clone(),
    )
}

/// 嵌套案公共骨架：outer(positioned CB，650×300，无 border/padding → padding-box =
/// content-box) > mid（static 中间层，content 原点在 outer content 系 x=60）> abs。
const MID: &str = "margin-left:7px; border-left:29px solid transparent; padding-left:24px; \
padding-right:14px; border-right:3px solid transparent; margin-right:18px;";

fn nested_html(abs_style: &str, extra_outer: &str) -> String {
    format!(
        r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; {extra_outer}">
  <div id="mid" style="{MID}">
    <div id="abs" style="{abs_style}"></div>
  </div>
</div>
</body></html>"#
    )
}

/// 嵌套案断言：abs 盒位（outer content 系）= mid border/padding 偏移 + abs.x/y。
/// 全 auto inset 的轴不入断言（静态位是另一域）。
fn assert_nested_pos(html: &str, expected_x: Option<f32>, expected_y: Option<f32>, label: &str) {
    let (outer, abs) = layout_two(html, "outer", "abs");
    let mid = &outer.children[0];
    if let Some(expected_x) = expected_x {
        let x = mid.x + mid.border_left + mid.padding_left + abs.x;
        assert!(
            (x - expected_x).abs() < 0.5,
            "{label}: 盒位 x 应 {expected_x:.1}，实际 {x:.1}（abs.x={:.1}）",
            abs.x
        );
    }
    if let Some(expected_y) = expected_y {
        let y = mid.y + mid.border_top + mid.padding_top + abs.y;
        assert!(
            (y - expected_y).abs() < 0.5,
            "{label}: 盒位 y 应 {expected_y:.1}，实际 {y:.1}（abs.y={:.1}）",
            abs.y
        );
    }
}

/// ltr 长度锚：left:3 + ml:17 → 盒位 x 20（§10.3.7 盒 margin 缘 = CB 左 + left，边盒位
/// 再 + ml）。旧写点丢 ml → 3。
#[test]
fn r5372_nested_ltr_left_anchor_includes_margin_left() {
    let html = nested_html(
        "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px;",
        "",
    );
    assert_nested_pos(&html, Some(20.0), None, "ltr 长度锚 +ml");
}

/// right 锚（left auto）：盒位 x = CB 宽 − right − mr − w = 650−8−19−153 = 470
///（§10.3.7 解 left 方程后 + ml 抵消）。旧写点丢 mr → 489。
#[test]
fn r5372_nested_right_anchor_includes_margin_right() {
    let html = nested_html(
        "position:absolute; left:auto; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px;",
        "",
    );
    assert_nested_pos(&html, Some(470.0), None, "right 锚 −mr");
}

/// rtl 下 right 锚方程不变（left auto 时 §10.3.7 与方向无关）。
#[test]
fn r5372_nested_rtl_right_anchor_same_equation() {
    let html = nested_html(
        "position:absolute; left:auto; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px;",
        "direction:rtl;",
    );
    assert_nested_pos(&html, Some(470.0), None, "rtl right 锚 −mr");
}

/// top 长度锚：top:11 + mt:13 → 盒位 y 24（§10.6.4 同方程）。旧写点丢 mt → 11。
#[test]
fn r5372_nested_top_anchor_includes_margin_top() {
    let html = nested_html("position:absolute; top:11px; margin-top:13px; height:2px;", "");
    assert_nested_pos(&html, None, Some(24.0), "top 锚 +mt");
}

/// bottom 锚（top auto）：盒位 y = CB 高 − bottom − mb − h = 300−3−13−2 = 282
///（§10.6.4 rule 2）。旧写点丢 mb → 295。
#[test]
fn r5372_nested_bottom_anchor_includes_margin_bottom() {
    let html = nested_html(
        "position:absolute; top:auto; bottom:3px; margin-bottom:13px; height:2px;",
        "",
    );
    assert_nested_pos(&html, None, Some(282.0), "bottom 锚 −mb");
}

/// 守卫基线（taffy 直挂，parent 即 CB）：① ltr 双锚过约束 ignore mr → 盒位 = left+ml
/// = 20；③ left/right 全 auto → static pos + ml = 17。
#[test]
fn r5372_direct_hang_taffy_baseline() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:5px">
  <div id="a1" style="position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px"></div>
  <div id="a3" style="position:absolute; margin-left:17px; margin-right:19px; width:153px; height:1px"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a1 = find(&result.root, doc.get_element_by_id("a1").unwrap()).unwrap();
    let a3 = find(&result.root, doc.get_element_by_id("a3").unwrap()).unwrap();
    assert!((a1.x - 20.0).abs() < 0.5, "直挂 ltr 双锚盒位应 20，实际 {:.1}", a1.x);
    assert!(
        (a3.x - 17.0).abs() < 0.5,
        "直挂全 auto 盒位应 static+ml=17，实际 {:.1}",
        a3.x
    );
}

/// 守卫基线（taffy 直挂）：rtl 双锚过约束 ignore left → 盒右缘 = CB 右 − right − mr
/// = 623，盒位 = 623−153 = 470。
#[test]
fn r5372_direct_hang_rtl_double_anchor_ignores_left() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:5px; direction:rtl">
  <div id="a2" style="position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a2 = find(&result.root, doc.get_element_by_id("a2").unwrap()).unwrap();
    assert!((a2.x - 470.0).abs() < 0.5, "直挂 rtl 双锚盒位应 470，实际 {:.1}", a2.x);
}
