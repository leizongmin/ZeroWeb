//! R5377 回归测试：abspos 约束方程过约束臂按 **CB** direction 选边（CSS2 §10.3.7）——
//! rtl CB 的 nothing-auto 组与双 auto 负 slack 组忽略 `left`、按 right 边锚定（ltr 忽略
//! `right` = 既有恒左锚）。旧口径 root/nested 两 pass 恒左锚，与 taffy 直挂（R5372 pin
//! 470）劈叉。方向来源是 CB 自身 direction：static 中间层（sdir）与 abs 元素自身
//！（edir）的 rtl 不改变锚边（driving：WPT abspos-non-replaced-width-margin-000 adir
//! 组 spec 496，ZW 恒左锚 70）。
//! 嵌套案 load-bearing：kill-switch `ZW_ABSPOS_NESTED_CB=0` 时应失败。

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

/// 嵌套案骨架（R5372 同谱系）：outer(positioned CB，650 宽，extra_outer 可带 rtl) >
/// mid（static 中间层，content 原点在 outer content 系 x=60，mid_style 可带 rtl）>
/// abs（abs_style 可带 rtl）。断言 abs 盒位（outer content 系）= mid 偏移 + abs.x。
fn assert_nested_x(abs_style: &str, mid_style: &str, extra_outer: &str, expected_x: f32, label: &str) {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; {extra_outer}">
  <div id="mid" style="margin-left:7px; border-left:29px solid transparent; padding-left:24px; padding-right:14px; border-right:3px solid transparent; margin-right:18px; {mid_style}">
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

const NOTHING_AUTO: &str =
    "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:153px; height:1px;";

/// 主案（bug 复现）：rtl CB + nothing-auto 过约束 → 忽略 left，盒位 = CB − right − mr
/// − w = 650−8−19−153 = 470（taffy 直挂 R5372 pin 同值）。旧恒左锚 20。
#[test]
fn r5377_nested_rtl_cb_nothing_auto_ignores_left() {
    assert_nested_x(NOTHING_AUTO, "", "direction:rtl;", 470.0, "rtl CB nothing-auto 右锚");
}

/// widewidth 负解：650−8−19−660 = −37（spec 允许负位）。
#[test]
fn r5377_nested_rtl_cb_nothing_auto_wide_negative() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:660px; height:1px;",
        "",
        "direction:rtl;",
        -37.0,
        "rtl CB widewidth 负解右锚",
    );
}

/// 双 auto margin 负 slack：归零后仍过约束 → rtl 右锚（mr=0）：650−8−0−660 = −18。
#[test]
fn r5377_nested_rtl_cb_both_auto_negative_anchors_right() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:auto; width:660px; height:1px;",
        "",
        "direction:rtl;",
        -18.0,
        "rtl CB 双 auto 负 slack 右锚",
    );
}

/// 守卫：双 auto 正 slack 方程精确解 → 维持居中左锚 246（R5376 口径，rtl 不选边）。
#[test]
fn r5377_nested_rtl_cb_both_auto_positive_keeps_left() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:auto; width:153px; height:1px;",
        "",
        "direction:rtl;",
        246.0,
        "rtl CB 双 auto 正 slack 左锚",
    );
}

/// 守卫：right:auto → 右缘吸收域不选边，左锚 20。
#[test]
fn r5377_nested_rtl_cb_right_auto_keeps_left() {
    assert_nested_x(
        "position:absolute; left:3px; right:auto; margin-left:17px; margin-right:19px; width:153px; height:1px;",
        "",
        "direction:rtl;",
        20.0,
        "rtl CB right:auto 左锚",
    );
}

/// 守卫：width:auto → inset-stretch 域不选边，左锚 20（RTL 臂以 width definite 为门）。
#[test]
fn r5377_nested_rtl_cb_width_auto_keeps_left() {
    assert_nested_x(
        "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; height:1px;",
        "",
        "direction:rtl;",
        20.0,
        "rtl CB width:auto 左锚",
    );
}

/// 方向来源 = CB：static 中间层（sdir）与 abs 元素自身（edir）的 rtl 不改变锚边 →
/// ltr CB 左锚 20。
#[test]
fn r5377_nested_non_cb_rtl_keeps_left_anchor() {
    assert_nested_x(
        &format!("{NOTHING_AUTO} direction:rtl;"),
        "direction:rtl;",
        "",
        20.0,
        "sdir/edir 不选边",
    );
}

/// ltr CB 对照：nothing-auto 过约束忽略 right（左锚 20），行为不变。
#[test]
fn r5377_nested_ltr_cb_nothing_auto_keeps_left() {
    assert_nested_x(NOTHING_AUTO, "", "", 20.0, "ltr CB 左锚不变");
}

/// root-CB 臂（根 positioned + rtl）：CB = html padding-box 800，盒位 = 800−8−19−243
/// = 530（ltr 恒左锚 20）。
#[test]
fn r5377_root_cb_rtl_nothing_auto_ignores_left() {
    let html = r#"<html style="position:relative; direction:rtl"><body style="margin:0">
<div id="abs" style="position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; width:243px; height:1px"></div>
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
        (abs.x - 530.0).abs() < 0.5,
        "root-CB rtl nothing-auto 盒位应 530，实际 {:.1}",
        abs.x
    );
}
