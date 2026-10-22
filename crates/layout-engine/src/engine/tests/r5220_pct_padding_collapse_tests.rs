//! R5220：% padding 解析基准——塌 0 真实 CB 传真值 0（CSS2 §8.4 × css-sizing-3
//! #fit-content-size）。
//!
//! `resolve_percentage_padding`（sizing.rs）按第一趟父 content_width 把 % padding
//! 预解析成 px。旧 else 臂把 content_width=0 一律透传父级基准（「匿名盒透传」
//! 语义）——真实块级容器塌 0（fit-content 真空塌 0 R5219 / width:0）被误当匿名，
//! 子 `padding-top:100%` 解析成祖先宽（探针实证 fit-content 塌 0 后子 pt=300=
//! 外层宽）。修后按 styles display 直判：非 Inline 的真实元素 content_width=0
//! 传真值 0；匿名盒与 inline 级盒（不构成独立 CB，子的 CB 是最近块级祖先）保持
//! 透传。float-clearance-with-margin-collapse-and-fit-content-and-padding-
//! percentage-001~004 四案 9.38%→0.00% 全翻绿（margin 塌穿/clearance 链此前被
//! 误解析的 padding 高整链挡住）。

use super::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 外层 300 定宽 + fit-content 真空塌 0 + 子 padding-top:100%：
/// 子 padding 应解析 0（CB 宽=0，塌 0 真值），修前透传解析 300。
#[test]
fn r5220_pct_padding_in_collapsed_fitcontent_resolves_zero() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let fit = doc.create_element("div");
    doc.append_child(outer, fit).unwrap();
    let pad = doc.create_element("div");
    doc.append_child(fit, pad).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut outer_st = ComputedStyle::default();
    outer_st.display = DisplayValue::Block;
    outer_st.width = LengthValue::Px(300.0);
    styles.insert(outer, outer_st);
    let mut fit_st = ComputedStyle::default();
    fit_st.display = DisplayValue::Block;
    fit_st.width = LengthValue::MaxContent; // bare fit-content，真空塌 0
    styles.insert(fit, fit_st);
    let mut pad_st = ComputedStyle::default();
    pad_st.display = DisplayValue::Block;
    pad_st.padding_top = LengthValue::Percentage(100.0);
    styles.insert(pad, pad_st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let pad_box = find_child_by_node_id(&result.root, pad).expect("pad found");
    assert!(
        (pad_box.padding_top - 0.0).abs() < 0.5,
        "塌 0 CB 内 % padding 应解析 0（修前透传解析 300），实际 {}",
        pad_box.padding_top
    );
    assert!(pad_box.height < 0.5, "padding 0 则盒高 0，实际 {}", pad_box.height);
}

/// 对照锚：正常 content_width 基准解析不变——300 容器直接子 padding-top:50%
/// 应解析 150（§8.4 CB 宽 300）。
#[test]
fn r5220_pct_padding_normal_cb_baseline_unchanged() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let pad = doc.create_element("div");
    doc.append_child(outer, pad).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut outer_st = ComputedStyle::default();
    outer_st.display = DisplayValue::Block;
    outer_st.width = LengthValue::Px(300.0);
    styles.insert(outer, outer_st);
    let mut pad_st = ComputedStyle::default();
    pad_st.display = DisplayValue::Block;
    pad_st.padding_top = LengthValue::Percentage(50.0);
    styles.insert(pad, pad_st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let pad_box = find_child_by_node_id(&result.root, pad).expect("pad found");
    assert!(
        (pad_box.padding_top - 150.0).abs() < 0.5,
        "正常 CB 宽 300 下 padding-top:50% 应解析 150，实际 {}",
        pad_box.padding_top
    );
}

/// width:0 显式定宽元素 content_width=0 是真值：子 % padding 解析 0。
#[test]
fn r5220_pct_padding_in_explicit_zero_width_resolves_zero() {
    let (mut doc, body) = make_doc_with_body();
    let zero = doc.create_element("div");
    doc.append_child(body, zero).unwrap();
    let pad = doc.create_element("div");
    doc.append_child(zero, pad).unwrap();

    let mut styles = HashMap::new();
    let mut body_st = ComputedStyle::default();
    body_st.display = DisplayValue::Block;
    body_st.width = LengthValue::Px(300.0);
    styles.insert(body, body_st);
    let mut zero_st = ComputedStyle::default();
    zero_st.display = DisplayValue::Block;
    zero_st.width = LengthValue::Px(0.0);
    styles.insert(zero, zero_st);
    let mut pad_st = ComputedStyle::default();
    pad_st.display = DisplayValue::Block;
    pad_st.padding_top = LengthValue::Percentage(100.0);
    styles.insert(pad, pad_st);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let pad_box = find_child_by_node_id(&result.root, pad).expect("pad found");
    assert!(
        (pad_box.padding_top - 0.0).abs() < 0.5,
        "width:0 真实 CB 内 % padding 应解析 0，实际 {}",
        pad_box.padding_top
    );
}
