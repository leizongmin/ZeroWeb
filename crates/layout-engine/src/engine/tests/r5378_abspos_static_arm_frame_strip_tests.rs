//! R5378 回归测试：abspos/fixed 静态位臂的父 border 残剥（CSS2 §10.3.7 静态位）。
//! taffy 对 abs location 一律携带父 border（block `area_offset` = border；静态位
//! `static_position.x = resolved_content_box_inset.left` = border+padding），extract
//!（R787）只剥 padding，旧 `fix_abspos_cb_border` 仅 positioned 父剥残差 border →
//! static 中间层下 all-auto inset 组（abspos pass 不覆写 x/y、taffy 直通）终绘多加
//! 一份父 border。修复 = fix_abspos_cb_border 去 positioned 门（R1398 遗留的「深层
//! abspos 不在处理范围」挂账）。驱动：WPT abspos-non-replaced-width-margin-000
//! all-auto 组 +29 = 静态中间层 border-left。
//! 嵌套案 load-bearing：kill-switch `ZW_ABSPOS_CB_BORDER=0` 时应失败。

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

fn layout(html: &str) -> (LayoutBox, LayoutBox, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer_id = doc.get_element_by_id("outer").unwrap();
    let mid_id = doc.get_element_by_id("mid").unwrap();
    let abs_id = doc.get_element_by_id("abs").unwrap();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer = find(&result.root, outer_id).unwrap();
    let mid = find(&result.root, mid_id).unwrap();
    let abs = find(&result.root, abs_id).unwrap();
    (outer, mid, abs)
}

/// 嵌套案骨架：outer(positioned CB，650 宽，outer_extra 可带 border) > mid（static
/// 中间层，mid_style 全参数化）> abs（all-auto inset，ml:17 w:153）。断言 abs 盒位
///（outer content 系）= mid.x + mid.frame + abs.x（content 相对约定，绘制层叠父
/// content origin）。
fn assert_static_x(mid_style: &str, outer_extra: &str, expected_x: f32, label: &str) {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; {outer_extra}">
  <div id="mid" style="{mid_style}">
    <div id="abs" style="position:absolute; margin-left:17px; width:153px; height:1px;"></div>
  </div>
</div>
</body></html>"#
    );
    let (_outer, mid, abs) = layout(&html);
    let x = mid.x + mid.border_left + mid.padding_left + abs.x;
    assert!(
        (x - expected_x).abs() < 0.5,
        "{label}: 盒位 x 应 {expected_x:.1}，实际 {x:.1}（abs.x={:.1}）",
        abs.x
    );
}

/// 主案（残差②复现）：完整 frame 中间层（ml:7 bl:29 pl:24）→ 静态位 = content 原点
/// + ml = 7+29+24+17 = 77。旧口径 abs.x 残留 border（46）→ 106。
#[test]
fn r5378_static_arm_full_frame_no_border_double_count() {
    assert_static_x(
        "margin-left:7px; border-left:29px solid transparent; padding-left:24px; padding-right:14px; border-right:3px solid transparent; margin-right:18px;",
        "",
        77.0,
        "全 frame 静态位无 border 双计",
    );
}

/// 仅 border 中间层：静态位 = 0+29+17 = 46。旧口径 75（+29 双计臂）。
#[test]
fn r5378_static_arm_border_only() {
    assert_static_x("border-left:29px solid transparent;", "", 46.0, "仅 border 静态位");
}

/// 守卫：仅 padding 中间层——旧口径已正确剥 padding，行为不变（24+17 = 41）。
#[test]
fn r5378_static_arm_padding_only_unchanged() {
    assert_static_x("padding-left:24px;", "", 41.0, "仅 padding 静态位不变");
}

/// 守卫：无 frame 中间层直通（17）。
#[test]
fn r5378_static_arm_no_frame_plain() {
    assert_static_x("", "", 17.0, "无 frame 静态位直通");
}

/// 直挂 CB 带 border（R787 盲区类）：outer(border-left:20) 直接挂 all-auto abs——
/// 静态位 content 相对 = ml = 17（旧口径 37，+20 双计）。
#[test]
fn r5378_static_arm_cb_border_direct_hang() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px; border-left:20px solid transparent">
  <div id="mid"></div>
  <div id="abs" style="position:absolute; margin-left:17px; width:153px; height:1px;"></div>
</div>
</body></html>"#;
    let (_outer, _mid, abs) = layout(html);
    assert!(
        (abs.x - 17.0).abs() < 0.5,
        "直挂 CB 带 border 静态位应 17（content 相对），实际 {}",
        abs.x
    );
}

/// y 轴同病同修：仅 border-top 中间层 + mt:3 → abs.y = 3（content 相对），盒位 y =
/// mid.y + border_top + abs.y = 14。旧口径 abs.y 残留 border-top（14）→ 25。
#[test]
fn r5378_static_arm_border_top_y_axis() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="position:relative; width:650px; height:300px">
  <div id="mid" style="border-top:11px solid transparent">
    <div id="abs" style="position:absolute; margin-top:3px; width:153px; height:1px;"></div>
  </div>
</div>
</body></html>"#;
    let (_outer, mid, abs) = layout(html);
    let y = mid.y + mid.border_top + abs.y;
    assert!(
        (y - 14.0).abs() < 0.5,
        "y 轴静态位应 14，实际 {y:.1}（abs.y={:.1}）",
        abs.y
    );
}
