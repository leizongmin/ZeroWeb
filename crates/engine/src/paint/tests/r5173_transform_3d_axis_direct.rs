//! R5173 回归：rotateX/X 单位轴 rotate3d 叶实心盒走 R5085 直绘 quad。
//!
//! 收缩投影（|cos| < 1）下 TP 重采样路径的「rect 清白」会抹除元素布局矩形内
//! 先绘下层内容且露出区域无下层像素可恢复（css-transform-3d-rotateX-positive
//! 族：绿块投影几何正确、红块 border 上下横带被清白，1.11%×4）。矩阵构建器
//! 对 rotateX 产出精确轴对齐 scale，四角映射 = 精确几何，与 2D 直绘集合同构。
//! rotateY 不入直绘（backdrop-filter-3d-transform-perspective 配对实证 45°
//! 非整数 texel 相位单侧改路拆对），维持 TP。
//!
//! kill-switch `ZW_TRANSFORM_3D_AXIS_DIRECT=0` 回退由 corpus OFF 轮承担
//! （LazyLock 进程级快照，进程内 set_var 不可靠——R3858 教训）。

use std::collections::HashMap;

use zero_css_parser::values::{TransformFunction, TransformValue};
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::visual::make_box;

/// rotateX(60deg) 叶实心盒（css-transform-3d-rotateX-positive 形态：120×240 盒
/// 投影为 120×120）——fill 替换为四角映射 path_fill、零 TransformPrimitive，
/// 四角 = 布局矩形绕中心 (140,140) 纵向收缩 cos(60°)=0.5。
#[test]
fn test_r5173_rotate_x_solid_leaf_direct_quad() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 80.0, 20.0, 120.0, 240.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::RotateX(60.0)]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(60.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(120.0);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 0, "solid leaf fill must be re-emitted as path_fill");
    assert_eq!(prims.transforms.len(), 0, "no TransformPrimitive for direct path");
    assert_eq!(prims.path_fills.len(), 1, "exactly one transformed quad fill");
    let mut got: Vec<(u32, u32)> = prims.path_fills[0]
        .vertices
        .chunks(2)
        .map(|c| [(c[0] * 10.0).round() as u32, (c[1] * 10.0).round() as u32])
        .map(|a| (a[0], a[1]))
        .collect();
    got.sort();
    // 四角 = (80,80),(200,80),(200,200),(80,200)：y' = 140 + (y−140)×0.5。
    let mut want: Vec<(u32, u32)> = vec![(800, 800), (2000, 800), (2000, 2000), (800, 2000)];
    want.sort();
    assert_eq!(got, want, "projected quad corners, got {got:?}");
}

/// rotate3d(1,0,0,60deg) X 单位轴——经 rotate3d_unit_axis 归一到 RotateX，同一直绘
/// 路径与同一投影四角（css-transform-3d-rotate3d-X-positive 形态）。
#[test]
fn test_r5173_rotate3d_x_axis_direct_quad() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 80.0, 20.0, 120.0, 240.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Rotate3d(1.0, 0.0, 0.0, 60.0)]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(60.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(120.0);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 0, "X-axis rotate3d takes direct path");
    assert_eq!(prims.path_fills.len(), 1, "exactly one transformed quad fill");
}

/// rotateY(60deg) 维持 TP 路径——backdrop-filter-3d-transform-perspective 配对
/// 实证 45° 非整数 texel 相位下单侧改路拆对（test 侧 backdrop-filter 门锁死 TP、
/// ref 侧同几何实心盒直绘 AA 分化），rotateY 整轴不入直绘。
#[test]
fn test_r5173_rotate_y_keeps_tp_path() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 20.0, 80.0, 240.0, 120.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::RotateY(60.0)]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(120.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(60.0);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "rotateY keeps the plain fill");
    assert_eq!(prims.transforms.len(), 1, "rotateY keeps TransformPrimitive path");
    assert_eq!(prims.path_fills.len(), 0, "rotateY emits no path_fill");
}

/// 有子盒的变换元素不入直绘——直绘限叶实心盒形态，与 R5085 2D 门的风险面隔离；
/// 子盒零可见图元（fill 恰 +1）也照样排除。
#[test]
fn test_r5173_with_child_box_keeps_tp_path() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let child_nid = doc.create_element("span");
    let mut layout = make_box(Some(nid), 80.0, 20.0, 120.0, 240.0);
    layout
        .children
        .push(make_box(Some(child_nid), 80.0, 20.0, 120.0, 240.0));

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::RotateX(60.0)]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(60.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(120.0);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "child box emits no primitive, plain fill kept");
    assert_eq!(prims.transforms.len(), 1, "child box forces TransformPrimitive path");
    assert_eq!(prims.path_fills.len(), 0, "no direct quad with child box");
}

/// 混合轴 rotate3d（非恰一分量非零）维持 TP 路径——恒等近似非精确几何，不入直绘。
#[test]
fn test_r5173_mixed_axis_rotate3d_keeps_tp_path() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 80.0, 20.0, 120.0, 240.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Rotate3d(1.0, 1.0, 0.0, 60.0)]);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // 混合轴矩阵 = 恒等近似，compute_transform_matrix 的 is_identity 早退不发射
    // TP（既有语义：无视觉效果即无图元）——直绘门同样不放行（恒等非精确几何）。
    assert_eq!(prims.fills.len(), 1, "mixed-axis keeps the plain fill");
    assert_eq!(prims.transforms.len(), 0, "mixed-axis identity TP is skipped");
    assert_eq!(prims.path_fills.len(), 0, "mixed-axis emits no path_fill");
}
