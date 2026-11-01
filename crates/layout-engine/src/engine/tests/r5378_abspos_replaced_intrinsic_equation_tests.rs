//! R5378 回归测试：abspos 约束方程对 **replaced width:auto（内在宽）** 的覆盖
//！（CSS2 §10.3.8：replaced 的 used width = 内在宽，taffy 已实测到 LayoutBox.width
//! ——方程基准 definite，R5376 方程解 margin 臂与 R5377 rtl 过约束选边臂同覆盖）。
//! 旧口径两 helper 对 `width:auto` 一律 None → replaced 的 nothing-auto 组落 ltr 左锚
//!（WPT abspos-replaced-width-margin-000 adir 组 spec −11 形态，ZW 恒 70）、only-ml-auto
//! 组 margin 当 0（53 应 −11 形态）。
//! replaced 元素用 inline `<svg width height>`：属性 → 内在尺寸，CSS width 保持 auto。
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

/// 嵌套案骨架（R5377 同谱系）：outer(positioned CB，650 宽，extra_outer 可带 rtl) >
/// mid（static 中间层，content 原点在 outer content 系 x=60）> replaced svg（abs_style
/// 全参数化，attr 尺寸 svg_w×10）。断言盒位（outer content 系）= mid 偏移 + abs.x。
fn assert_replaced_x(abs_style: &str, extra_outer: &str, svg_w: f32, expected_x: f32, label: &str) {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; {extra_outer}">
  <div id="mid" style="margin-left:7px; border-left:29px solid transparent; padding-left:24px;">
    <svg id="abs" width="{svg_w}" height="10" style="{abs_style}"></svg>
  </div>
</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    assert!(abs.is_replaced, "{label}: svg 应为 replaced 盒（is_replaced）");
    assert_eq!(
        abs.width, svg_w,
        "{label}: svg 内在宽应作 used width（border-box {svg_w}）"
    );
    let x = 7.0 + 29.0 + 24.0 + abs.x;
    assert!(
        (x - expected_x).abs() < 0.5,
        "{label}: 盒位 x 应 {expected_x:.1}，实际 {x:.1}（abs.x={:.1}）",
        abs.x
    );
}

const NOTHING_AUTO_REPLACED: &str =
    "position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; height:1px;";

/// 主案（bug 复现，WPT adir 组形态）：rtl CB + nothing-auto + width:auto（内在 200）
/// 过约束 → 忽略 left 右锚：x_rel = 650−8−19−200 = 423（盒位在 outer content 系同值，
/// mid 偏移 60 与 abs.x=363 相抵）。旧口径 width:auto 不入臂 → 恒左锚 3+17 = 20。
#[test]
fn r5378_replaced_intrinsic_rtl_nothing_auto_right_anchor() {
    assert_replaced_x(
        NOTHING_AUTO_REPLACED,
        "direction:rtl;",
        200.0,
        423.0,
        "replaced rtl nothing-auto 右锚",
    );
}

/// ltr 对照：同款过约束 ltr 忽略 right = 左锚既有口径 x_rel = 3+17 = 20 → 盒位 20。
#[test]
fn r5378_replaced_intrinsic_ltr_nothing_auto_left_anchor() {
    assert_replaced_x(NOTHING_AUTO_REPLACED, "", 200.0, 20.0, "replaced ltr nothing-auto 左锚");
}

/// R5376 臂覆盖（WPT ml 组形态）：only ml auto + width:auto（内在 200）→ 方程解
/// ml = 650−3−8−19−200 = 420 → x_rel = 3+420 = 423。旧口径 ml 当 0 → 过约束左锚 3。
#[test]
fn r5378_replaced_intrinsic_single_ml_auto_solved() {
    assert_replaced_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:19px; height:1px;",
        "",
        200.0,
        423.0,
        "replaced only-ml-auto 方程解",
    );
}

/// 双 auto margin 正 slack（WPT ml+mr 组）：均分 slack/2 → ml = 439/2 → x_rel =
/// 3+219.5 = 222.5（本扩展前 ml 当 0 落左锚 3，即 margin:0 auto 居中对 replaced 生效）。
#[test]
fn r5378_replaced_intrinsic_both_auto_positive_slack_centered() {
    assert_replaced_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:auto; height:1px;",
        "",
        200.0,
        222.5,
        "replaced 双 auto 正 slack 均分居中",
    );
}

/// 双 auto margin 负 slack：归零后仍过约束 → rtl 右锚（mr=0）：x_rel = 650−8−0−660 =
/// −18（R5377 双 auto 臂 replaced 覆盖，负位 spec 允许）。
#[test]
fn r5378_replaced_intrinsic_both_auto_negative_slack_right_anchor() {
    assert_replaced_x(
        "position:absolute; left:3px; right:8px; margin-left:auto; margin-right:auto; height:1px;",
        "direction:rtl;",
        660.0,
        -18.0,
        "replaced 双 auto 负 slack 右锚",
    );
}

/// 守卫：非 replaced（div）width:auto 维持 inset-stretch 域旧口径——rtl 不选边、
/// ml 不解方程，恒左锚 3+17 = 20 → 盒位 80（R5376/R5377 面不变）。
#[test]
fn r5378_nonreplaced_width_auto_unchanged() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; direction:rtl">
  <div id="mid" style="margin-left:7px; border-left:29px solid transparent; padding-left:24px;">
    <div id="abs" style="position:absolute; left:3px; right:8px; margin-left:17px; margin-right:19px; height:1px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    let x = 7.0 + 29.0 + 24.0 + abs.x;
    assert!(
        (x - 20.0).abs() < 0.5,
        "非 replaced width:auto 守卫：盒位应 20，实际 {x:.1}"
    );
}
