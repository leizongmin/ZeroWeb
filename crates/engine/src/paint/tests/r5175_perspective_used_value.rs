//! R5175 回归：perspective used value clamp（属性 + 函数）与 perspective() 平面投影折叠。
//!
//! css-transforms-2 §perspective-property / §funcdef-perspective +
//! csswg-drafts#413/#6320：
//! - `perspective: 0`（<1px）used value clamp 到 1px，仍建立透视上下文（≠ none）——
//!   perspective-zero/zero-2/zero-3/transform3d-perspective-005：τ=0.5px → s=2 放大
//!   盖红块（store 层 clamp 断言在 style-system `test_perspective_apply`）。
//! - `perspective(d)` 函数：后缀（列表右侧，先应用于点）全为平面函数时，投影 = 绕
//!   transform-origin 均匀缩放 s = d/(d−τ)，整表近似 = 现有合成结果 × s——
//!   transform3d-perspective-006：perspective(100px) translatez(50px) → s=2。

use std::collections::HashMap;

use zero_css_parser::values::{TransformFunction, TransformValue};
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::super::transform::compute_transform_matrix;
use super::visual::make_box;

/// perspective-zero-3 形态：父 `perspective: 0px`（clamp 后 d=1px，origin top left），
/// 子 50×50 `translateZ(0.5px)` → s = 1/(1−0.5) = 2，绿块盖满 100×100 红块。
#[test]
fn test_r5175_property_zero_clamp_establishes_context() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 0.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    // 显式 0px 经 style-system store 层 clamp 后的存储值（Px(1.0)）；none 保持
    // Px(0.0) 哨兵不建立上下文（见 test_r5175_property_none_no_context）。
    // R5176 transformable display 门——default 是 Inline，测块级父须显式 Block。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(1.0);
    parent_style.perspective_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    parent_style.perspective_origin_y = zero_css_parser::values::LengthValue::Px(0.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 0.5)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 1, "synthesized perspective scale TP");
    let tp = &prims.transforms[0];
    assert!(
        (tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6,
        "s = 1/(1-0.5) = 2"
    );
    assert!(tp.tx.abs() < 1e-6 && tp.ty.abs() < 1e-6, "origin (0,0): o(1-s) = 0");
    assert!((tp.rect.right() - 100.0).abs() < 1e-4 && (tp.rect.bottom() - 100.0).abs() < 1e-4);
}

/// perspective-zero-2 反向门：τ ≥ d（子平面触达视点）投影非仿射——clamp 使 d=1px 后
/// translateZ(2px) 可达该域，须不折叠（s ≤ 0 守卫）。
#[test]
fn test_r5175_property_child_tau_ge_d_bails() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 0.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(1.0);
    parent_style.perspective_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    parent_style.perspective_origin_y = zero_css_parser::values::LengthValue::Px(0.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 2.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    assert!(painter.primitives().transforms.is_empty(), "w = 1-2/1 < 0: no fold");
}

/// none 哨兵（Px(0.0)）不建立透视上下文——clamp 不得把 none 误升为 1px 上下文。
#[test]
fn test_r5175_property_none_no_context() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 0.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(0.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 0.5)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    assert!(
        painter.primitives().transforms.is_empty(),
        "perspective: none → no context"
    );
}

/// perspective-zero painter 全链形态：单盒自身 [Perspective(0), Translate3d(0,0,0.5)]、
/// origin top left、绿底（无父 perspective）——fold 后 TP s=2。
#[test]
fn test_r5175_painter_function_perspective_zero_tp() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), nid);

    let node = make_box(Some(nid), 8.0, 50.0, 50.0, 50.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(0.0),
        TransformFunction::Translate3d(0.0, 0.0, 0.5),
    ]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(0.0);

    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&node, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(
        prims.transforms.len(),
        1,
        "folded TP emitted, got {}",
        prims.transforms.len()
    );
    let tp = &prims.transforms[0];
    assert!((tp.a - 2.0).abs() < 1e-6, "s=2, got a={}", tp.a);
}

/// 解析层 R5175 豁免：perspective() + translateZ 混用列表可解析（R5174 曾整表失效），
/// 并端到端折叠；translateZ + rotateX 仍失效（equivalence 拆对域保留）。
#[test]
fn test_r5175_parse_perspective_translatez_mix_allowed() {
    let parsed = zero_css_parser::values::parse_transform("perspective(100px) translatez(50px)");
    assert!(matches!(
        parsed,
        Some(TransformValue::List(ref fs))
            if matches!(fs.as_slice(),
                [TransformFunction::Perspective(100.0), TransformFunction::Translate3d(0.0, 0.0, 50.0)])
    ));
    assert!(
        zero_css_parser::values::parse_transform("translateZ(100px) rotateX(30deg)").is_none(),
        "translateZ + rotateX still invalid (R5174 equivalence guard)"
    );

    // 解析产物端到端进折叠（006 全链）：s=2 绕 origin。
    let TransformValue::List(funcs) = parsed.expect("parsed") else {
        panic!("list expected");
    };
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(funcs);
    let tp = compute_transform_matrix(&style, &Rect::new(25.0, 25.0, 50.0, 50.0)).expect("folded TP");
    assert!((tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6);
    assert!((tp.tx + 50.0).abs() < 1e-6 && (tp.ty + 50.0).abs() < 1e-6);
}

/// transform3d-perspective-006：`perspective(100px) translatez(50px)` 50×50 @(25,25)——
/// τ_right = 50，s = 100/(100−50) = 2 绕 transform-origin (50,50)：tx = −50，
/// quad (25,25)-(75,75) → (0,0)-(100,100)。
#[test]
fn test_r5175_function_perspective_folds_scale() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(100.0),
        TransformFunction::Translate3d(0.0, 0.0, 50.0),
    ]);
    let rect = Rect::new(25.0, 25.0, 50.0, 50.0);

    let tp = compute_transform_matrix(&style, &rect).expect("folded TP");
    assert!((tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6, "s = 2");
    assert!(
        (tp.tx + 50.0).abs() < 1e-6 && (tp.ty + 50.0).abs() < 1e-6,
        "o(1-s) = -50"
    );
    assert!((tp.rect.left() - 0.0).abs() < 1e-4 && (tp.rect.top() - 0.0).abs() < 1e-4);
    assert!((tp.rect.right() - 100.0).abs() < 1e-4 && (tp.rect.bottom() - 100.0).abs() < 1e-4);
}

/// perspective-zero：`perspective(0) translateZ(0.5px)`——d used value clamp 到 1px
/// 后 s = 2（#6320）。
#[test]
fn test_r5175_function_perspective_zero_clamped() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(0.0),
        TransformFunction::Translate3d(0.0, 0.0, 0.5),
    ]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(0.0);
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    let tp = compute_transform_matrix(&style, &rect).expect("folded TP");
    assert!(
        (tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6,
        "clamp d=1 → s=2"
    );
    assert!(tp.tx.abs() < 1e-6 && tp.ty.abs() < 1e-6, "origin (0,0)");
}

/// perspective 在前、逐点 z 变函数（rotateX）在后缀——投影非恒定 z 平面，不折叠。
#[test]
fn test_r5175_function_perspective_suffix_3d_bails() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(100.0),
        TransformFunction::RotateX(30.0),
    ]);
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    let tp = compute_transform_matrix(&style, &rect).expect("rotateX approx TP");
    assert!(
        (tp.a - 1.0).abs() < 1e-6,
        "no perspective fold (rotateX scaleY approx only)"
    );
}

/// quad 触达视点（τ ≥ d）：perspective(1px) translatez(50px) 投影非仿射，不折叠
///（恒等合成 → None）。
#[test]
fn test_r5175_function_perspective_behind_viewer_bails() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(1.0),
        TransformFunction::Translate3d(0.0, 0.0, 50.0),
    ]);
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    assert!(compute_transform_matrix(&style, &rect).is_none(), "w < 0: bail");
}

/// 多个 perspective() 非单源可折叠形，维持既有恒等近似。
#[test]
fn test_r5175_function_perspective_double_bails() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(100.0),
        TransformFunction::Translate3d(0.0, 0.0, 50.0),
        TransformFunction::Perspective(100.0),
    ]);
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    assert!(
        compute_transform_matrix(&style, &rect).is_none(),
        "two perspectives: bail"
    );
}

/// τ=0（后缀纯 2D）s=1 不折叠——`perspective(100px) translate(10px)` 保持纯平移。
#[test]
fn test_r5175_function_perspective_tau_zero_noop() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(100.0),
        TransformFunction::Translate(10.0, 0.0),
    ]);
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    let tp = compute_transform_matrix(&style, &rect).expect("translate-only TP");
    assert!((tp.a - 1.0).abs() < 1e-6, "s = 1: unscaled");
    assert!((tp.tx - 10.0).abs() < 1e-6, "pure translation kept");
}
