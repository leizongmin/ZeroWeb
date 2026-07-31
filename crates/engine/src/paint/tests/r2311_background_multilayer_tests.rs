//! R2311：background 多层 longhand 逐层 cyclic 渲染集成测试。
//!
//! `background-position/size/repeat` 现为多层 Vec；`paint_bg_image_in_origin` 按图层
//! `longhands[layer_idx % len]` 取值。单值 longhand（len=1）→ 所有图层取 [0] = 旧「单值
//! 应用到所有层」行为（byte-identical）；多层 longhand → 各层独立定位/缩放/重复。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    BackgroundImageComputedValue, BackgroundPositionComputedValue, BackgroundRepeatComputedValue,
    BackgroundSizeComputedValue, ComputedStyle,
};

use super::super::painter::Painter;

fn make_box(node_id: Option<NodeId>, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x: 0.0,
        y: 0.0,
        width,
        height,
        content_width: width,
        content_height: height,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

/// 两层图像 + 两个不同 position → 两层各自按自己的 position 定位（cyclic）。
#[test]
fn test_r2311_multilayer_position_per_layer() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![
        BackgroundImageComputedValue::Url("a.png".to_string()),
        BackgroundImageComputedValue::Url("b.png".to_string()),
    ];
    // 两层 size 均 20px（确定 sized=20×20）
    style.background_size = vec![
        BackgroundSizeComputedValue::Length(20.0),
        BackgroundSizeComputedValue::Length(20.0),
    ];
    // 两层 position：第 0 层 0% 0%（→ offset 0），第 1 层 100% 100%（→ offset 80）
    style.background_position = vec![
        BackgroundPositionComputedValue::TwoValue(
            Box::new(BackgroundPositionComputedValue::Percent(0.0)),
            Box::new(BackgroundPositionComputedValue::Percent(0.0)),
        ),
        BackgroundPositionComputedValue::TwoValue(
            Box::new(BackgroundPositionComputedValue::Percent(100.0)),
            Box::new(BackgroundPositionComputedValue::Percent(100.0)),
        ),
    ];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let xs: Vec<f32> = painter.primitives().images.iter().map(|p| p.rect.origin.x).collect();
    // 两层各一个 tile（no-repeat），分别落在 x≈0 和 x≈80（100−20）
    assert_eq!(xs.len(), 2, "两层图像各应产生 1 个 tile");
    assert!(
        xs.iter().any(|&x| (x - 0.0).abs() < 0.5),
        "第 0 层应定位在 x≈0，got {xs:?}"
    );
    assert!(
        xs.iter().any(|&x| (x - 80.0).abs() < 0.5),
        "第 1 层应定位在 x≈80（100% position），got {xs:?}"
    );
}

/// R2313：background-position: calc()/min()/max() 单层 paint 求值（% 相对 container-image）。
#[test]
fn test_r2313_bg_position_calc_resolves() {
    let expr = zero_css_parser::values::parse_math_function("calc(50%)").expect("calc(50%) 解析");
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("a.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(20.0)];
    style.background_position = vec![BackgroundPositionComputedValue::Calc(expr)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let xs: Vec<f32> = painter.primitives().images.iter().map(|p| p.rect.origin.x).collect();
    // calc(50%) → (container-image)*0.5 = (100-20)*0.5 = 40
    assert_eq!(xs.len(), 1);
    assert!(
        (xs[0] - 40.0).abs() < 0.5,
        "calc(50%) 应定位在 x≈40（(container-image)*0.5），got {xs:?}"
    );
}

/// 两层图像 + 单值 position（cyclic mod 1）→ 两层均取 [0]，定位相同（byte-identical 回归守）。
#[test]
fn test_r2311_multilayer_single_position_byte_identical() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![
        BackgroundImageComputedValue::Url("a.png".to_string()),
        BackgroundImageComputedValue::Url("b.png".to_string()),
    ];
    style.background_size = vec![BackgroundSizeComputedValue::Length(20.0)];
    // 单值 position（len=1）→ cyclic 使两层均取 [0] = 0% 0%
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Percent(0.0)),
        Box::new(BackgroundPositionComputedValue::Percent(0.0)),
    )];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let xs: Vec<f32> = painter.primitives().images.iter().map(|p| p.rect.origin.x).collect();
    // 两层都取 position[0]=0% → 都定位在 x≈0（byte-identical 于旧「单值应用到所有层」）
    assert_eq!(xs.len(), 2, "两层图像各应产生 1 个 tile");
    assert!(
        xs.iter().all(|&x| (x - 0.0).abs() < 0.5),
        "单值 position 下两层都应定位在 x≈0（cyclic mod 1），got {xs:?}"
    );
}
