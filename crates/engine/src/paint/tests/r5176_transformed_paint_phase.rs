//! R5176 回归：变换元素 App E step 8 绘制相位（transformed 与 positioned 同层
//! tree order）。
//!
//! css-transforms-1 §4 + CSS 2.1 Appendix E step 8：transform ≠ none 建立堆叠上下文，
//! 与 positioned 后代同属 step 8 绘制组按 tree order 混排——此前变换元素留在
//! steps 3/4/5 in-flow 相位，后随 positioned 兄弟（#cover-me position:relative）
//! 晚绘盖住已正确缩放的变换元素（perspective-zero/zero-point-five：10,000px
//! 精确红块全露，变换数学本身 R5175 已证正确）。
//!
//! 实现三处：collect_positioned_descendants 收集变换子（z_index==0 桶 tree order
//! 混排、不下钻——transform 自成 SC scope）；主循环排除；perspective ≠ none 补入
//! layout 侧 `style_creates_stacking_context`（transformable display 门）——perspective
//! 父入 scope 是变换子收集落父 ctx 窗口的前提（否则 R5174 折叠失去 ctx 栈配对）。
//! multicol 片段领地不入（cso 子树无收集器可达，主循环排除即漏盒）。
//!
//! kill-switch `ZW_TRANSFORM_PAINT_PHASE=0` / `ZW_PERSPECTIVE_STACKING_CONTEXT=0`
//! 回退由 corpus OFF 轮承担（构造期快照，进程内 set_var 不可靠——R3858 教训）。

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_dom::Document;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::visual::make_box;

/// 绘制序断言：首个红 fill 与首个绿 fill 的图元序（fills 按绘制顺序追加）。
fn red_green_fill_order(prims: &RenderPrimitives) -> (Option<usize>, Option<usize>) {
    let red = prims
        .fills
        .iter()
        .position(|f| f.color.r == 255 && f.color.g == 0 && f.color.b == 0);
    let green = prims
        .fills
        .iter()
        .position(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0);
    (red, green)
}

/// perspective-zero 案形：root 下 [#cover-me（红 100×100，position:relative，DOM 前）、
/// #test（绿 50×50，`perspective(0) translateZ(0.5px)` origin top left，DOM 后）]。
/// 二者同属 step 8 组按 tree order：红先绿后，绿 ×2 缩放 (0,0)-(100,100) 盖红。
#[test]
fn test_r5176_transformed_after_positioned_sibling() {
    let mut doc = Document::new();
    let cover_nid = doc.create_element("div");
    let test_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), cover_nid);
    let _ = doc.append_child(doc.root(), test_nid);

    let mut cover = make_box(Some(cover_nid), 0.0, 0.0, 100.0, 100.0);
    cover.is_relative = true;
    let mut test = make_box(Some(test_nid), 0.0, 0.0, 50.0, 50.0);
    test.children.clear();

    let mut cover_style = ComputedStyle::default();
    cover_style.background_color = ColorValue::Rgba(255, 0, 0, 255);

    let mut test_style = ComputedStyle::default();
    test_style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    test_style.transform = TransformValue::List(vec![
        TransformFunction::Perspective(0.0),
        TransformFunction::Translate3d(0.0, 0.0, 0.5),
    ]);
    test_style.transform_origin_x = LengthValue::Px(0.0);
    test_style.transform_origin_y = LengthValue::Px(0.0);

    let mut styles = HashMap::new();
    styles.insert(cover_nid, cover_style);
    styles.insert(test_nid, test_style);

    let mut root = make_box(None, 0.0, 0.0, 100.0, 100.0);
    root.children.push(cover);
    root.children.push(test);

    let mut painter = Painter::new();
    painter.paint(&root, &styles, Some(&doc));

    let prims = painter.primitives();
    // 相位重排：红（positioned，tree order 先）先绘、绿（transformed）后绘盖红。
    let (red, green) = red_green_fill_order(prims);
    let (Some(red), Some(green)) = (red, green) else {
        panic!("red/green fills expected, got red={red:?} green={green:?}");
    };
    assert!(
        green > red,
        "transformed sibling must paint after positioned sibling (App E step 8 tree order), got green={green} red={red}"
    );
    // R5175 折叠不变：TP s=2（clamp d=1px、τ=0.5）。
    assert_eq!(prims.transforms.len(), 1, "folded TP");
    assert!((prims.transforms[0].a - 2.0).abs() < 1e-6);
}

/// R5174 形态在新相位下的 ctx 配对：perspective 父（layout 侧 SC → scope）+ 变换子
/// ——子收集落父 scope flush（父 ctx 窗口内），折叠 s=2 不丢失。
#[test]
fn test_r5176_perspective_parent_scope_keeps_ctx_pairing() {
    let mut doc = Document::new();
    let parent_nid = doc.create_element("div");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));
    // layout 侧 perspective-SC 产物（engine sizing.rs style_creates_stacking_context）。
    parent.creates_stacking_context = true;

    let mut parent_style = ComputedStyle::default();
    // transformable display 门（同 r5174 修正）：default 是 Inline，块级父显式 Block。
    parent_style.display = zero_css_parser::values::DisplayValue::Block;
    parent_style.perspective = LengthValue::Px(1.0);
    parent_style.perspective_origin_x = LengthValue::Px(0.0);
    parent_style.perspective_origin_y = LengthValue::Px(0.0);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 0.5)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert_eq!(prims.transforms.len(), 1, "deferred child still folds in ctx window");
    assert!((prims.transforms[0].a - 2.0).abs() < 1e-6, "s = 1/(1-0.5) = 2");
}

/// inline span 上的 perspective 不适用（非 transformable，css-transforms-2
/// §perspective-property）——不建 scope、不推 ctx：变换子不折叠父透视
///（perspective-untransformable-no-stacking-context 案形，layout 侧同门见
/// layout-engine r2302 测试）。
#[test]
fn test_r5176_inline_perspective_no_ctx() {
    let mut doc = Document::new();
    let parent_nid = doc.create_element("span");
    let child_nid = doc.create_element("div");
    let _ = doc.append_child(doc.root(), parent_nid);
    let _ = doc.append_child(parent_nid, child_nid);

    let mut parent = make_box(Some(parent_nid), 0.0, 0.0, 100.0, 100.0);
    parent.children.push(make_box(Some(child_nid), 0.0, 25.0, 50.0, 50.0));

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_css_parser::values::DisplayValue::Inline;
    parent_style.perspective = LengthValue::Px(1.0);

    let mut child_style = ComputedStyle::default();
    child_style.transform = TransformValue::List(vec![TransformFunction::Translate3d(0.0, 0.0, 0.5)]);

    let mut styles = HashMap::new();
    styles.insert(parent_nid, parent_style);
    styles.insert(child_nid, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent, &styles, Some(&doc));

    let prims = painter.primitives();
    assert!(
        prims.transforms.iter().all(|t| (t.a - 2.0).abs() > 1e-6),
        "no parent perspective fold for non-transformable (inline) perspective"
    );
}
