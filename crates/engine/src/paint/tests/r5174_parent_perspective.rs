//! R5174 回归：父元素 `perspective` 属性的直子透视投影合成。
//!
//! css-transforms-2 §18-21：perspective ≠ none 的元素为直子建立 3D 透视上下文，
//! 恒定 z 平面（τ = translateZ/translate3d z 分量累计）子元素的投影 = 绕父
//! perspective-origin 等比缩放 s = d/(d−τ)。此前 translateZ 无解析臂（整个
//! transform 列表丢弃）+ 父 perspective 零消费——perspective-origin-001..006
//! 族：绿块（z=1, s=2）未放大露红块，1.56%×6。
//!
//! kill-switch `ZW_PARENT_PERSPECTIVE=0` 回退由 corpus OFF 轮承担（LazyLock/
//! 构造期快照，进程内 set_var 不可靠——R3858 教训）。

use std::collections::HashMap;

use zero_css_parser::values::{TransformFunction, TransformValue};
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::visual::make_box;

/// 子盒实色 fill 计数（测试装置在多层 DOM 下会附零宽杂散 fill，按矩形过滤）。
fn child_fill_count(prims: &zero_render_foundation::primitive::RenderPrimitives) -> usize {
    prims
        .fills
        .iter()
        .filter(|f| f.rect.left() == 0.0 && f.rect.top() == 25.0 && f.rect.right() == 50.0 && f.rect.bottom() == 75.0)
        .count()
}

/// 父 100×100 `perspective:2px; perspective-origin:0px 50%`，子 50×50 @(0,25)
/// `translateZ(1px)`（perspective-origin-001 ref 形态）——合成 TP：a=d=2、
/// ty = oy(1−s) = 50×(−1) = −50、union rect (0,0)-(100,100)（四角 (0,25)→(0,0)、
/// (50,75)→(100,100)）。
#[test]
fn test_r5174_translate_z_child_perspective_scale() {
    // 解析层：translateZ(t) 归一为 Translate3d(0, 0, t)（此前无解析臂整表丢弃）。
    assert!(matches!(
        zero_css_parser::values::parse_transform("translateZ(1px)"),
        Some(TransformValue::List(fs)) if matches!(fs.as_slice(), [TransformFunction::Translate3d(0.0, 0.0, 1.0)])
    ));

    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    // R5176 transformable display 门（Inline/Contents/None 不适用 perspective）——
    // default 是 Inline，测块级父须显式 Block（真实引擎 UA 层 div→block）。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);
    parent_style.perspective_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    parent_style.perspective_origin_y = zero_css_parser::values::LengthValue::Percentage(50.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(child_fill_count(prims), 1, "child solid fill kept");
    assert_eq!(prims.transforms.len(), 1, "synthesized perspective scale TP");
    assert_eq!(prims.path_fills.len(), 0, "no direct quad on this path");
    let tp = &prims.transforms[0];
    assert!(
        (tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6,
        "s = 2/(2-1) = 2"
    );
    assert!(tp.b.abs() < 1e-6 && tp.c.abs() < 1e-6);
    assert!(tp.tx.abs() < 1e-6, "tx = ox(1-s) = 0×(-1)");
    assert!((tp.ty + 50.0).abs() < 1e-6, "ty = oy(1-s) = 50×(-1)");
    assert!((tp.rect.left() - 0.0).abs() < 1e-4 && (tp.rect.top() - 0.0).abs() < 1e-4);
    assert!((tp.rect.right() - 100.0).abs() < 1e-4 && (tp.rect.bottom() - 100.0).abs() < 1e-4);
}

/// 自身仿射 + 透视缩放外层复合：`translate(10px, 0) translateZ(1px)` ——
/// 折叠后 tx = s·own_tx + ox(1−s) = 2×10 + 0 = 20（p' = s·M(p) + o·(1−s)）。
#[test]
fn test_r5174_own_transform_folds_perspective() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    // R5176 transformable display 门——default 是 Inline，测块级父须显式 Block。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);
    parent_style.perspective_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    parent_style.perspective_origin_y = zero_css_parser::values::LengthValue::Percentage(50.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![
        TransformFunction::Translate(10.0, 0.0),
        TransformFunction::Translate3d(0.0, 0.0, 1.0),
    ]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 1, "own matrix folded, not duplicated");
    let tp = &prims.transforms[0];
    assert!((tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6);
    assert!((tp.tx - 20.0).abs() < 1e-6, "tx = s·own_tx + ox(1-s) = 20");
    assert!((tp.ty + 50.0).abs() < 1e-6);
}

/// 隔代不承接（spec 默认 transform-style:flat，孙代先 flat 进父平面）——
/// 中间层无 perspective，孙元素 translateZ 不合成缩放。
#[test]
fn test_r5174_grandchild_not_composed() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let mid_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, mid_nid);
    let _ = doc.append_child(mid_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    let mut mid = make_box(Some(mid_nid), 0.0, 0.0, 100.0, 100.0);
    mid.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));
    parent.children.push(mid);

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(child_fill_count(prims), 1, "child fill kept");
    assert_eq!(prims.transforms.len(), 0, "grandchild gets no perspective scale");
}

/// 逐点 z 变函数不入：rotateX 子元素维持自身 2D 投影 TP（d = cos60° = 0.5），
/// 父 perspective 不折叠（精确投影须真 3D 管线，R3833 同谱局限）。
#[test]
fn test_r5174_rotate_x_child_not_folded() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::RotateX(60.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    // rotateX(60) 叶实心盒走 R5173 直绘 quad（axis_exact_3d 门），零 TP；父
    // perspective 不折叠（flat_plane_translate_z 对逐点 z 变函数 bail）——
    // quad 纵向范围 = 自身 2D 投影 [37.5, 62.5]（绕默认 origin (25,50) 收缩
    // cos60°=0.5），若误折叠 s=2 会外扩到 [25, 75]。
    assert_eq!(prims.transforms.len(), 0, "direct quad path, no TP");
    assert_eq!(prims.path_fills.len(), 1, "rotateX direct quad kept");
    let ys: Vec<f32> = prims.path_fills[0].vertices.chunks(2).map(|c| c[1]).collect();
    assert!(
        ys.iter().all(|y| (37.5_f32..=62.5).contains(y)),
        "quad y range unfolded (no perspective scale), got {ys:?}"
    );
}

/// translateZ(0)：s = d/(d−0) = 1，零合成（perspective-origin-001 test 侧红块形态）。
#[test]
fn test_r5174_translate_z_zero_no_scale() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 0.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(child_fill_count(prims), 1, "child fill kept");
    assert_eq!(prims.transforms.len(), 0, "s = 1, no transform emitted");
}

/// R5174 混合列表按语法失效（解析层）：translateZ 与逐点 z 变函数混用 → 整表
/// None（基线语义，perspective-transforms-equivalence 拆对根因——R3833 退化兜底
/// 对 rotY 归零、rotZ 幸存，代数等价表述投影互异）；纯 translateZ 平面列表与
/// translate3d 混合列表照常解析（后者基线本就可解析，transform3d-translate3d-001
/// 双侧近似同相）。individual rotate 合成不受列表有效性牵连。
#[test]
fn test_r5174_mixed_translatez_list_invalid() {
    assert!(zero_css_parser::values::parse_transform("rotateY(90deg) translateZ(250px)").is_none());
    assert!(zero_css_parser::values::parse_transform("translateZ(250px) rotateY(90deg)").is_none());
    assert!(zero_css_parser::values::parse_transform("rotateZ(90deg) rotateX(90deg) translateZ(250px)").is_none());
    assert!(zero_css_parser::values::parse_transform("translateZ(1px)").is_some());
    assert!(zero_css_parser::values::parse_transform("translateZ(-250px) rotateZ(45deg)").is_some());
    assert!(
        zero_css_parser::values::parse_transform(
            "translate3d(10px, 30px, -10px) rotatex(45deg) translate3d(-10px, -30px, 10px)"
        )
        .is_some()
    );

    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![TransformFunction::RotateY(90.0)]);
    style.individual_rotate = Some(TransformFunction::Rotate(45.0));
    let rect = zero_render_foundation::geometry::Rect::new(0.0, 0.0, 100.0, 100.0);
    assert!(
        super::super::transform::compute_transform_matrix_with_ref_box(&style, &rect, None).is_some(),
        "individual rotate synthesized regardless of list validity"
    );
}

/// preserve-3d 中间代链承接：parent(persp 2px) > mid(preserve-3d) > child(tZ(1)
/// 叶实心)——τ 经 mid 累加（0），child 折叠 s = 2（3d-scene-with-iframe-001 形态）。
#[test]
fn test_r5174_chain_preserve_3d_folds() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let mid_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, mid_nid);
    let _ = doc.append_child(mid_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    let mut mid = make_box(Some(mid_nid), 0.0, 0.0, 100.0, 100.0);
    mid.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));
    parent.children.push(mid);

    let mut parent_style = ComputedStyle::default();
    // R5176 transformable display 门——default 是 Inline，测块级父须显式 Block。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut mid_style = ComputedStyle::default();
    mid_style.transform_style = zero_style_system::TransformStyleValue::Preserve3d;

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(mid_nid, mid_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(child_fill_count(prims), 1, "child fill kept");
    assert_eq!(prims.transforms.len(), 1, "chain fold through preserve-3d mid");
    let tp = &prims.transforms[0];
    assert!(
        (tp.a - 2.0).abs() < 1e-6 && (tp.d - 2.0).abs() < 1e-6,
        "s = 2/(2-1) = 2"
    );
    assert!(
        (tp.tx + 50.0).abs() < 1e-6 && (tp.ty + 50.0).abs() < 1e-6,
        "origin (50,50)"
    );
}

/// 逐点 z 变中间代断链：mid preserve-3d + rotateX(60)——τ 不可加（精确投影须真
/// 3D 管线），child 不折叠；transforms 恰 1 条且为 mid 自身 rotateX 投影（d = 0.5），
/// 误折叠会多出合成 TP。
#[test]
fn test_r5174_chain_broken_by_pointwise_z_ancestor() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let mid_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, mid_nid);
    let _ = doc.append_child(mid_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    let mut mid = make_box(Some(mid_nid), 0.0, 0.0, 100.0, 100.0);
    mid.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));
    parent.children.push(mid);

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut mid_style = ComputedStyle::default();
    mid_style.transform_style = zero_style_system::TransformStyleValue::Preserve3d;
    mid_style.transform = TransformValue::List(vec![TransformFunction::RotateX(60.0)]);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(mid_nid, mid_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 1, "only mid's own rotateX TP");
    let tp = &prims.transforms[0];
    assert!((tp.d - 0.5).abs() < 1e-6, "mid rotateX cos60 projection, unfolded");
}

/// 非叶实心断链：child 自带子盒（差集 2 fill ≠ 单一边框盒 fill）——区域型 TP
/// 后处理无子树隔离，非叶盒合成重放会清白区域内先绘内容，不折叠。
#[test]
fn test_r5174_chain_non_leaf_solid_blocked() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let mid_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let grand_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, mid_nid);
    let _ = doc.append_child(mid_nid, child_nid);
    let _ = doc.append_child(child_nid, grand_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    let mut mid = make_box(Some(mid_nid), 0.0, 0.0, 100.0, 100.0);
    let mut child = make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0);
    child.children.push(make_box(Some(grand_nid), 5.0, 30.0, 20.0, 20.0));
    mid.children.push(child);
    parent.children.push(mid);

    let mut parent_style = ComputedStyle::default();
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut mid_style = ComputedStyle::default();
    mid_style.transform_style = zero_style_system::TransformStyleValue::Preserve3d;

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut grand_style = ComputedStyle::default();
    grand_style.background_color = zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(mid_nid, mid_style);
    styles.insert(child_nid, child_style);
    styles.insert(grand_nid, grand_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 0, "non-leaf subtree, no chain fold");
}

/// perspective-origin 百分比解析（默认 50% 50%）：父 200 宽 origin x = 100，
/// tx = 100×(1−2) = −100、tp.origin 记 (100, 50)。
#[test]
fn test_r5174_percent_origin_resolution() {
    let mut doc = zero_dom::Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 200.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    // R5176 transformable display 门——default 是 Inline，测块级父须显式 Block。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = zero_css_parser::values::LengthValue::Px(2.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 1.0)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 1);
    let tp = &prims.transforms[0];
    assert!((tp.tx + 100.0).abs() < 1e-6, "tx = ox(1-s) = 100×(-1)");
    assert!((tp.ty + 50.0).abs() < 1e-6, "ty = oy(1-s) = 50×(-1)");
    assert!((tp.origin_x - 100.0).abs() < 1e-6 && (tp.origin_y - 50.0).abs() < 1e-6);
}
