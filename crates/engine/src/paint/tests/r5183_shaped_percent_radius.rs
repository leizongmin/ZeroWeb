//! R5183 回归：corner-shape 形角路径的百分比半径解析（css-borders-4 §3.7）。
//!
//! 旧行为：形角三消费端（形角边框带外/内圈、overflow clip 形角）半径取裸
//! `length_to_f32`——Percentage 返 0，`border-radius: 50%` + corner-shape 整盒
//! 退直角（同页 plain 弧带路径走 `BorderRadiusSpec::from_style_with_box` 出圆弧
//! 的自相矛盾渲染；corner-shape-iframe-border 案 5.33% 域核顺带发现）。修复：
//! 形角路径半径统一走 spec 规范口径（百分比相对 border-box、min(w,h)/2 钳制、
//! §5.5 相邻重叠收缩）。

use super::super::helpers::shaped_corner_polygon;
use zero_css_parser::values::{CornerShapeKind, CornerShapeValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 顶点扁平数组中是否存在 ≈(px,py)（容差 0.6px）。
fn has_vert(verts: &[(f32, f32)], px: f32, py: f32) -> bool {
    verts
        .iter()
        .any(|(x, y)| (*x - px).abs() < 0.6 && (*y - py).abs() < 0.6)
}

/// 200×200 border-radius:50% + corner-shape: bevel scoop notch square——四角半径
/// 应解析为 100（50% 相对 border-box，相邻和 200 = 边长，§5.5 收缩因子 f=1 不缩），
/// TL bevel 切弦 (100,0)→(0,100) 在场、直角点 (0,0) 不在场；BL square 经盒角点
/// (0,200) 在场。
#[test]
fn test_r5183_shaped_percent_radius_bevel_cut() {
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Percentage(50.0);
    }
    style.corner_shape = CornerShapeValue::PerCorner(Box::new([
        CornerShapeKind::Bevel,
        CornerShapeKind::Scoop,
        CornerShapeKind::Notch,
        CornerShapeKind::Square,
    ]));
    let pts = shaped_corner_polygon(&style, 200.0, 200.0, 0.0, 0.0).expect("非全 round 形角应返回多边形");
    assert!(
        has_vert(&pts, 100.0, 0.0) && has_vert(&pts, 0.0, 100.0),
        "TL bevel r=100 切弦端点应在场，got {pts:?}"
    );
    assert!(!has_vert(&pts, 0.0, 0.0), "TL 直角点不应在场（bevel 已切削）");
    assert!(has_vert(&pts, 0.0, 200.0), "BL square 应经盒角点");
}

/// px 半径行为不变（解析器换轨不改数值）：30px bevel 切弦 (30,0)→(0,30)。
#[test]
fn test_r5183_shaped_px_radius_unchanged() {
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Px(30.0);
    }
    style.corner_shape = CornerShapeValue::All(CornerShapeKind::Bevel);
    let pts = shaped_corner_polygon(&style, 200.0, 200.0, 0.0, 0.0).expect("多边形");
    assert!(
        has_vert(&pts, 30.0, 0.0) && has_vert(&pts, 0.0, 30.0),
        "px bevel 切弦端点应在场，got {pts:?}"
    );
    assert!(!has_vert(&pts, 0.0, 0.0), "直角点不应在场");
}

/// backdrop-filter 元素与普通元素同口径：滤区随形角轮廓裁剪（R5183 shaped
/// backdrop clip，css-filters-2 #BackdropFilter），% 半径规范解析不变。
#[test]
fn test_r5183_backdrop_element_percent_same_spec_resolution() {
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Percentage(50.0);
    }
    style.corner_shape = CornerShapeValue::All(CornerShapeKind::Bevel);
    style.backdrop_filter = vec![zero_style_system::property::types::FilterComputedValue::Brightness(0.0)];
    let pts = shaped_corner_polygon(&style, 200.0, 200.0, 0.0, 0.0).expect("多边形");
    assert!(
        has_vert(&pts, 100.0, 0.0) && has_vert(&pts, 0.0, 100.0),
        "backdrop 元素 % 半径同规范解析（r=100 bevel 切弦在场），got {pts:?}"
    );
    assert!(!has_vert(&pts, 0.0, 0.0), "backdrop 元素直角点不应在场（bevel 已切削）");
}

/// R5339（css-filters-2 #BackdropFilter）：plain border-radius（corner-shape 初始态
/// 全 round）+ backdrop-filter——shaped 入口返 None 后回退 plain 圆角多边形，滤区
/// 随圆角轮廓（backdrop-filter-clip-rect-2 实证：backdrop 行方角 vs 背景行圆角）。
#[test]
fn test_r5339_backdrop_plain_radius_round_clip_fallback() {
    let mut style = ComputedStyle::default();
    for (r, expect) in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ]
    .into_iter()
    .zip([10.0f32, 20.0, 30.0, 40.0])
    {
        *r = LengthValue::Px(expect.into());
    }
    style.backdrop_filter = vec![zero_style_system::property::types::FilterComputedValue::Brightness(0.0)];
    // corner_shape 保持默认（全 round）——旧行为在此返回 None（矩形滤区方角）。
    let pts = crate::paint::helpers::backdrop_filter_clip_polygon(&style, 0.0, 0.0, 100.0, 100.0)
        .expect("plain 半径应回退圆角多边形");
    assert!(
        has_vert(&pts, 80.0, 0.0) && has_vert(&pts, 100.0, 20.0) && has_vert(&pts, 100.0, 70.0),
        "TR r=20 弧段 (80,0)→(100,20) 与 BR r=30 弧段起点 (100,70) 应在场，got {pts:?}"
    );
    assert!(!has_vert(&pts, 0.0, 0.0), "直角点不应在场（r=10 圆角切削）");
}

/// R5339 守卫：无半径（radii 全 0）不裁——矩形滤区既有行为不变（全量 corpus
/// backdrop 族矩形路径零扰动哨）。
#[test]
fn test_r5339_backdrop_no_radius_keeps_rect_region() {
    let mut style = ComputedStyle::default();
    style.backdrop_filter = vec![zero_style_system::property::types::FilterComputedValue::Brightness(0.0)];
    assert!(
        crate::paint::helpers::backdrop_filter_clip_polygon(&style, 0.0, 0.0, 100.0, 100.0).is_none(),
        "无半径应返回 None（矩形滤区）"
    );
}
