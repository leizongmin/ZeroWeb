//! R5219：width:fit-content / max-content 真空子树塌 0（css-sizing-3 #max-content）。
//!
//! `apply_intrinsic_content_sizing` 的 R1018 回退臂（intrinsic ≤1 → 置 Auto fill）
//! 曾把「真空子树的真实 max-content = 0」与「测量盲区退化 0」混淆：空盒 fit-content
//! 被 fill 成 CB 满（float-clearance-with-margin-collapse-and-fit-content-and-
//! padding-percentage-001~004 实证 w=784/800）。修后回退臂以
//! `subtree_may_contribute_intrinsic_width`（DOM 直读递归）区分两义：真空子树保持
//! converter 的 length(0) 塌缩（规范解 fit-content ≡ fit-content(max-content) =
//! min(W_max=0, …) = 0）；盲区信号（in-flow 元素 / 非空白文本 / replaced /
//! aspect-ratio）照旧回退 fill（intrinsic-size-005 multicol + aspect-ratio 子）。

use super::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_style_system::ComputedStyle;

/// width:fit-content 真空空盒（无子无文本）：宽塌 0（修前回退 fill = 容器满宽）。
#[test]
fn r5219_vacuum_fitcontent_empty_box_collapses_to_zero() {
    let (mut doc, body) = make_doc_with_body();
    let target = doc.create_element("div");
    doc.append_child(body, target).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut st = ComputedStyle::default();
    st.display = DisplayValue::Block;
    st.width = LengthValue::MaxContent; // bare fit-content 经 parser 映射
    styles.insert(target, st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find_child_by_node_id(&result.root, target).expect("target found");
    assert!(
        b.width < 1.0,
        "真空 fit-content 空盒应塌 0（修前 fill 300），实际 {}",
        b.width
    );
}

/// 盲区信号防误塌：fit-content 盒带 aspect-ratio 子（有 transferred 贡献潜力）不得被
/// 真空守卫塌 0。此形态 R4151 AR-aware 测量可信（transferred 宽 100），不进退化臂；
/// 真正的测量盲区回退保持（multicol columns×content 塌 0 → R1018 fill）由 upstream
/// intrinsic-size-005 reftest 常驻对照。
#[test]
fn r5219_aspect_ratio_child_not_vacuum_collapsed() {
    let (mut doc, body) = make_doc_with_body();
    let target = doc.create_element("div");
    doc.append_child(body, target).unwrap();
    let child = doc.create_element("div");
    doc.append_child(target, child).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut st = ComputedStyle::default();
    st.display = DisplayValue::Block;
    st.width = LengthValue::MaxContent;
    styles.insert(target, st);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Block;
    cs.aspect_ratio = Some(1.0);
    cs.height = LengthValue::Px(100.0);
    styles.insert(child, cs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find_child_by_node_id(&result.root, target).expect("target found");
    assert!(
        b.width > 1.0,
        "aspect-ratio 子有贡献潜力，不得被真空守卫塌 0（R5219 盲区信号），实际 {}",
        b.width
    );
}

/// 嵌套真空：fit-content 盒内空 div 链（无 in-flow 元素信号源）仍塌 0——
/// padding-percentage-001 形态（fit-content > 空 div）。
#[test]
fn r5219_nested_empty_div_chain_collapses_to_zero() {
    let (mut doc, body) = make_doc_with_body();
    let target = doc.create_element("div");
    doc.append_child(body, target).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(target, inner).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut st = ComputedStyle::default();
    st.display = DisplayValue::Block;
    st.width = LengthValue::MaxContent;
    styles.insert(target, st);
    let mut inner_st = ComputedStyle::default();
    inner_st.display = DisplayValue::Block;
    styles.insert(inner, inner_st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find_child_by_node_id(&result.root, target).expect("target found");
    assert!(
        b.width < 1.0,
        "嵌套空 div 链仍属真空，fit-content 应塌 0（修前 fill 300），实际 {}",
        b.width
    );
}

/// 文本信号不误伤：fit-content 盒含非空白文本 → 正常 max-content 收缩（非 0 非 fill），
/// 守卫只在 intrinsic ≤1 的退化臂生效。
#[test]
fn r5219_text_content_box_not_affected() {
    let (mut doc, body) = make_doc_with_body();
    let target = doc.create_element("div");
    doc.append_child(body, target).unwrap();
    let text_id = doc.create_text_node("hello world");
    doc.append_child(target, text_id).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut st = ComputedStyle::default();
    st.display = DisplayValue::Block;
    st.width = LengthValue::MaxContent;
    styles.insert(target, st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find_child_by_node_id(&result.root, target).expect("target found");
    assert!(
        b.width > 10.0 && b.width < 299.0,
        "文本 fit-content 应 max-content 收缩（介于内容宽与容器宽之间），实际 {}",
        b.width
    );
}
