#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_style_system::{
    BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundOriginComputedValue,
    BackgroundPositionComputedValue, BackgroundRepeatComputedValue, BackgroundSizeComputedValue, BorderStyleValue,
    ComputedStyle, OutlineStyleValue,
};

use super::super::color::{hsla_to_rgba, named_color_to_render};
use super::super::helpers::{BorderRadiusSpec, apply_transform_offset};
use super::super::painter::Painter;
use super::visual::make_box;

// ── background-position / background-size / background-clip 测试 ──────

/// 辅助：创建带 node_id 和样式的背景测试环境。
fn setup_bg_test() -> (Painter, zero_dom::NodeId, LayoutBox, HashMap<NodeId, ComputedStyle>) {
    let mut doc = zero_dom::Document::new();
    let node_id = doc.create_element("div");
    let layout = make_box(Some(node_id), 0.0, 0.0, 200.0, 100.0);
    let styles = HashMap::new();
    (Painter::new(), node_id, layout, styles)
}

/// 辅助：创建带 border/padding 的 box。
fn make_box_with_padding(
    node_id: Option<NodeId>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    bt: f32,
    bl: f32,
    pt: f32,
    pl: f32,
    cw: f32,
    ch: f32,
) -> LayoutBox {
    let mut b = make_box(node_id, x, y, w, h);
    b.border_top = bt;
    b.border_left = bl;
    b.padding_top = pt;
    b.padding_left = pl;
    b.content_width = cw;
    b.content_height = ch;
    b
}

/// 测试 background-position: center 居中偏移。
#[test]
fn test_background_position_center() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_position = vec![BackgroundPositionComputedValue::Center];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 0.0);
    assert_eq!(img[0].rect.origin.y, 0.0);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-position: right bottom 右下角偏移。
#[test]
fn test_background_position_right_bottom() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Right),
        Box::new(BackgroundPositionComputedValue::Bottom),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 0.0);
    assert_eq!(img[0].rect.origin.y, 0.0);
}

/// 测试 background-position 长度值偏移。
#[test]
fn test_background_position_length() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Length(20.0)),
        Box::new(BackgroundPositionComputedValue::Length(10.0)),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 20.0);
    assert_eq!(img[0].rect.origin.y, 10.0);
}

/// 测试 background-position 百分比偏移。
#[test]
fn test_background_position_percent() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::Percent(50.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 50.0);
    // percent 50: offset_x = (200 - 50) * 50 / 100 = 75.0
    assert_eq!(img[0].rect.origin.x, 75.0);
}

/// 测试 background-size: cover 覆盖容器。
#[test]
fn test_background_size_cover() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Cover];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-size: contain 包含在容器内。
#[test]
fn test_background_size_contain() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Contain];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    assert_eq!(img[0].rect.size.height, 100.0);
}

/// 测试 background-size: Length(100px) 固定宽度。
#[test]
fn test_background_size_length() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 300.0, 200.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(100.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 100.0);
    let expected_h = 100.0 * 200.0 / 300.0;
    assert!((img[0].rect.size.height - expected_h).abs() < 0.01);
}

/// 测试 background-size: Percent(50%) 百分比尺寸。
#[test]
fn test_background_size_percent() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 400.0, 200.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Percent(50.0)];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 200.0);
    let expected_h = 200.0 * 200.0 / 400.0;
    assert!((img[0].rect.size.height - expected_h).abs() < 0.01);
}

/// 测试 background-clip: content-box 限制背景绘制区域。
#[test]
fn test_background_clip_content_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.background_clip = BackgroundClipComputedValue::ContentBox;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 15.0);
    assert_eq!(fills[0].rect.origin.y, 15.0);
    assert_eq!(fills[0].rect.size.width, 180.0);
    assert_eq!(fills[0].rect.size.height, 80.0);
}

/// 测试 background-clip: padding-box 限制背景绘制区域。
#[test]
fn test_background_clip_padding_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 0.0, 0.0, 190.0, 90.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.background_clip = BackgroundClipComputedValue::PaddingBox;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 10.0);
    assert_eq!(fills[0].rect.origin.y, 10.0);
    assert_eq!(fills[0].rect.size.width, 190.0);
    assert_eq!(fills[0].rect.size.height, 90.0);
}

/// 测试 background-clip: border-box（默认值）与无 border 时等价于整盒。
#[test]
fn test_background_clip_border_box_default() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 0.0);
    assert_eq!(fills[0].rect.origin.y, 0.0);
    assert_eq!(fills[0].rect.size.width, 200.0);
    assert_eq!(fills[0].rect.size.height, 100.0);
}

/// 测试 background-origin: content-box 影响图片定位。
#[test]
fn test_background_origin_content_box() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_origin = BackgroundOriginComputedValue::ContentBox;
    // R2312：no-repeat 隔离 origin 定位断言（repeat 现平铺 painting area=clip box，tile 数会变）。
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.origin.x, 15.0);
    assert_eq!(img[0].rect.origin.y, 15.0);
    assert_eq!(img[0].rect.size.width, 180.0);
    assert_eq!(img[0].rect.size.height, 80.0);
}

/// R2312：background-clip 应用于背景图像（painting area = background-clip box，非 origin box）。
/// 旧 impl 误把 origin box 当 clip；本测试守 `background-clip: content-box` 把图像裁到 content-box。
#[test]
fn test_r2312_background_clip_applied_to_image() {
    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // border 10 + padding 5 → content-box at (15,15) 180x80；padding-box (10,10) 190x90。
    let layout = make_box_with_padding(Some(nid), 0.0, 0.0, 200.0, 100.0, 10.0, 10.0, 5.0, 5.0, 180.0, 80.0);

    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_clip = BackgroundClipComputedValue::ContentBox; // origin 仍默认 padding-box
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    // 图像从 padding-box origin (10,10) 起绘（Auto size=190×90），被裁到 content-box (15,15) 180×80。
    assert_eq!(img[0].rect.origin.x, 15.0);
    assert_eq!(img[0].rect.origin.y, 15.0);
    assert_eq!(img[0].rect.size.width, 180.0);
    assert_eq!(img[0].rect.size.height, 80.0);
}

/// 测试 background-position + background-size 组合。
#[test]
fn test_background_position_right_bottom_with_small_size() {
    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("img.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_size = vec![BackgroundSizeComputedValue::Length(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Right),
        Box::new(BackgroundPositionComputedValue::Bottom),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let img = &painter.primitives().images;
    assert_eq!(img.len(), 1);
    assert_eq!(img[0].rect.size.width, 50.0);
    assert_eq!(img[0].rect.origin.x, 150.0);
    assert_eq!(img[0].rect.origin.y, 75.0);
}

/// 测试渐变也受 background-position/size 影响。
#[test]
fn test_gradient_with_position_and_size() {
    use zero_css_parser::values::{GradientColorStop, GradientDirection, GradientValue, LinearGradient};

    let (mut painter, nid, layout, _) = setup_bg_test();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToRight,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
            repeating: false,
        },
    ))];
    style.background_size = vec![BackgroundSizeComputedValue::Percent(50.0)];
    style.background_position = vec![BackgroundPositionComputedValue::TwoValue(
        Box::new(BackgroundPositionComputedValue::Left),
        Box::new(BackgroundPositionComputedValue::Top),
    )];
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    painter.paint(&layout, &styles, None);

    let gradients = &painter.primitives().gradients;
    assert_eq!(gradients.len(), 1);
    assert_eq!(gradients[0].rect.size.width, 100.0);
    assert_eq!(gradients[0].rect.origin.x, 0.0);
    assert_eq!(gradients[0].rect.origin.y, 0.0);
}

// ── border-image 渲染测试 ──────────────────────────────────────────

/// 测试 border-image: url() 生成 9 宫格图片图元。
#[test]
fn test_border_image_url_9region() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 4 corners + 4 edges = 8 image primitives (fill=false, no center)
    let images = &painter.primitives().images;
    assert!(
        images.len() >= 8,
        "border-image should generate at least 8 image primitives, got {}",
        images.len()
    );
}

/// 测试 border-image-source: none 不生成图片图元。
#[test]
fn test_border_image_none() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 255, 255, 255);
    // default is None
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert_eq!(
        images.len(),
        0,
        "border-image:none should not generate image primitives"
    );
}

/// 测试 border-image 带不同边框宽度（不对称）。
#[test]
fn test_border_image_asymmetric_borders() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 300.0, 150.0);
    layout.border_top = 5.0;
    layout.border_right = 15.0;
    layout.border_bottom = 10.0;
    layout.border_left = 20.0;

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.border_image_source = BorderImageSourceComputedValue::Url("frame.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert!(
        images.len() >= 8,
        "asymmetric border-image should generate at least 8 images, got {}",
        images.len()
    );

    // 验证左上角位置和尺寸
    let top_left = &images[0];
    assert_eq!(top_left.rect.origin.x, 0.0);
    assert_eq!(top_left.rect.origin.y, 0.0);
    assert_eq!(top_left.rect.size.width, 20.0); // border-left
    assert_eq!(top_left.rect.size.height, 5.0); // border-top
}

/// 测试 border-image 带无 border 时跳过绘制。
#[test]
fn test_border_image_no_border() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    // no borders set

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert_eq!(images.len(), 0, "no border width should skip border-image");
}

/// 测试 column-rule: solid 在 3 列之间绘制 2 条分隔线。
#[test]
fn test_column_rules_solid() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_gap = LengthValue::Px(20.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Thin;
    style.column_rule_color = ColorValue::Rgba(128, 128, 128, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    // 3 列 → 2 条 rule → 2 个 fill 图元（background 可能也产生 fill，但 rule 的 fill 至少 2 个）
    // 检查至少有 rule 的 fill（每条 rule 是一个细矩形）
    let rule_fills: Vec<_> = fills
        .iter()
        .filter(|f| f.color.a > 0 && f.rect.size.width < 5.0 && f.rect.size.height > 100.0)
        .collect();
    assert!(
        rule_fills.len() >= 2,
        "3 columns should produce at least 2 column-rule fills, got {} rule fills",
        rule_fills.len()
    );
}

/// 测试 column-rule-width 关键字与 border-width 同值（CSS Multi-column：thin=1/medium=3/
/// thick=5px）。修复前 ZW 用 Medium=2/Thick=3 偏离 border-width 与 Chromium。solid rule 是
/// width=rule_w 的 fill 图元，故直接断言 fill 宽度。
#[test]
fn test_column_rule_width_keywords_match_border_width() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let assert_rule_width = |kw: ColumnRuleWidthComputedValue, expected: f32| {
        let mut doc = zero_dom::Document::new();
        let nid = doc.create_element("div");
        let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

        let mut style = ComputedStyle::default();
        style.column_count = ColumnCountComputedValue::Number(2);
        style.column_gap = LengthValue::Px(20.0);
        style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
        style.column_rule_width = kw;
        style.column_rule_color = ColorValue::Rgba(128, 128, 128, 255);
        let mut styles = HashMap::new();
        styles.insert(nid, style);
        let mut painter = Painter::new();
        painter.paint(&layout, &styles, None);

        let has_rule = painter
            .primitives()
            .fills
            .iter()
            .any(|f| f.color.a > 0 && (f.rect.size.width - expected).abs() < 0.1 && f.rect.size.height > 100.0);
        assert!(has_rule, "column-rule solid 应产出一个宽度≈{expected}px 的 fill 图元");
    };

    assert_rule_width(ColumnRuleWidthComputedValue::Thin, 1.0);
    assert_rule_width(ColumnRuleWidthComputedValue::Medium, 3.0);
    assert_rule_width(ColumnRuleWidthComputedValue::Thick, 5.0);
}

#[test]
fn test_column_rule_width_relative_length_resolves_in_paint() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_gap = LengthValue::Px(20.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Length(LengthValue::Em(0.5));
    style.column_rule_color = ColorValue::Rgba(128, 128, 128, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let has_rule = painter
        .primitives()
        .fills
        .iter()
        .any(|f| f.color.a > 0 && (f.rect.size.width - 10.0).abs() < 0.1 && f.rect.size.height > 100.0);
    assert!(has_rule, "column-rule-width:0.5em 应按 font-size:20px 绘制为 10px");
}

/// CSS Multi-column §4.3：column-rule-color 初始 = currentColor，paint 须解析为元素自身 color。
/// 元素 color:red + column-rule solid（无显式 column-rule-color）→ 分隔线应为红色（非黑）。
#[test]
fn test_column_rule_color_currentcolor_resolves_to_element_color() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.color = ColorValue::Rgba(255, 0, 0, 255); // red
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_gap = LengthValue::Px(20.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium;
    // column_rule_color 留默认 currentColor（不显式设）
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 分隔线 fill 应为红色（currentColor 解析为元素 color=red）。修复前初始=黑 + paint 无元素
    // 色上下文 → 黑色 fill，本断言 red；修复后初始=currentColor + resolve_color_current → 红。
    let has_red_rule =
        painter.primitives().fills.iter().any(|f| {
            f.color.a > 0 && f.color.r == 255 && f.color.g == 0 && f.color.b == 0 && f.rect.size.height > 100.0
        });
    assert!(
        has_red_rule,
        "column-rule solid + color:red 应产出红色（currentColor）fill 图元，非黑色"
    );
}

/// CSS Multi-column §4.1：column-gap 初始 normal = 1em（layout multicol.rs 解析）。
/// paint（text_multicol）默认 gap 须与 layout 一致，否则 3+ 列默认 gap 的 column-rule
/// X 坐标偏离实际列位置。3 列 content_w=600 + 默认 gap(1em=16) + Medium(3px) rule：
/// col_w=(600-32)/3≈189.33，rule@i=1 x≈195.83、@i=2 x≈401.17（修复前 gap=0 → 198.5/398.5）。
#[test]
fn test_column_rule_default_gap_matches_layout_3cols() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    // column_gap 留默认 Auto（normal→1em=16px，须与 layout 一致）
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium; // 3px
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 默认 gap=16：col_w=(600-32)/3≈189.333，rule_w=3。
    // rule@i=1 x = 0 + 1*189.333 + 0.5*16 - 1.5 ≈ 195.833
    // rule@i=2 x = 0 + 2*189.333 + 1.5*16 - 1.5 ≈ 401.167
    // 修复前（Auto→gap=0）：col_w=200，rule@198.5/398.5 → 这两个 x 无 fill（red）。
    let has_rule_at = |expected_x: f32| {
        painter.primitives().fills.iter().any(|f| {
            f.color.a > 0 && (f.rect.origin.x - expected_x).abs() < 0.5 && (f.rect.size.width - 3.0).abs() < 0.1
        })
    };
    assert!(
        has_rule_at(195.833),
        "默认 gap(1em) 3 列 rule@i=1 应在 x≈195.83（修复前 gap=0 → 198.5）"
    );
    assert!(
        has_rule_at(401.167),
        "默认 gap(1em) 3 列 rule@i=2 应在 x≈401.17（修复前 gap=0 → 398.5）"
    );
}

/// CSS Multi-column：column-gap: 2em 须按 font-size 解析（与 layout 一致），否则 em/%
/// column-gap 的 rule X 偏离列位置。3 列 content_w=600 + gap=2em(font16→32) + Medium(3px)：
/// col_w=(600-64)/3≈178.67，rule@i=1 x≈193.17、@i=2 x≈403.83（修复前 em→0 → 198.5/398.5）。
#[test]
fn test_column_rule_em_gap_matches_layout_3cols() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_gap = LengthValue::Em(2.0); // 2em → 32px（font-size 16）
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium; // 3px
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // gap=32：col_w=(600-64)/3≈178.667，rule_w=3。
    // rule@i=1 x = 0 + 1*178.667 + 0.5*32 - 1.5 ≈ 193.167
    // rule@i=2 x = 0 + 2*178.667 + 1.5*32 - 1.5 ≈ 403.833
    // 修复前（em→gap=0）：col_w=200，rule@198.5/398.5 → 这两个 x 无 fill（red）。
    let has_rule_at = |expected_x: f32| {
        painter.primitives().fills.iter().any(|f| {
            f.color.a > 0 && (f.rect.origin.x - expected_x).abs() < 0.5 && (f.rect.size.width - 3.0).abs() < 0.1
        })
    };
    assert!(
        has_rule_at(193.167),
        "column-gap:2em 3 列 rule@i=1 应在 x≈193.17（修复前 em→0 → 198.5）"
    );
    assert!(
        has_rule_at(403.833),
        "column-gap:2em 3 列 rule@i=2 应在 x≈403.83（修复前 em→0 → 398.5）"
    );
}

/// CSS Multi-column：column-gap: 4ch 须按 layout 的 multicol length resolver 解析为 32px
///（ch≈8px），否则 paint column-rule X 坐标与 layout 列位置分叉。
#[test]
fn test_column_rule_ch_gap_matches_layout_3cols() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_gap = LengthValue::Ch(4.0); // 4ch → 32px（layout multicol resolver）
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let has_rule_at = |expected_x: f32| {
        painter.primitives().fills.iter().any(|f| {
            f.color.a > 0 && (f.rect.origin.x - expected_x).abs() < 0.5 && (f.rect.size.width - 3.0).abs() < 0.1
        })
    };
    assert!(
        has_rule_at(193.167),
        "column-gap:4ch 3 列 rule@i=1 应在 x≈193.17（修复前 ch→0 → 198.5）"
    );
    assert!(
        has_rule_at(403.833),
        "column-gap:4ch 3 列 rule@i=2 应在 x≈403.83（修复前 ch→0 → 398.5）"
    );
}

/// CSS Multi-column：column-count:auto 时 column-width 须按 em/% 解析推列数（与 layout 一致）。
/// column-width:10em(font16→160) + 默认 gap(16) + content_w=600 → count=3，col_w≈189.33，
/// rule@i=1 x≈195.83、@i=2 x≈401.17。修复前 paint 只认 Px column-width → em 触发 `_=>return`
/// 不画任何 rule（red）。
#[test]
fn test_column_rule_em_column_width_draws_rules() {
    use zero_style_system::{
        ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue, ColumnWidthComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Auto;
    style.column_width = ColumnWidthComputedValue::Length(LengthValue::Em(10.0)); // 10em → 160px
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium; // 3px
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // count = floor((600+16)/(160+16)) = 3，col_w=(600-32)/3≈189.333，rule_w=3。
    // rule@i=1 x ≈ 195.833，rule@i=2 x ≈ 401.167。
    // 修复前 em column-width → `_=>return` 不画 rule（0 fill）→ 两 x 无 fill（red）。
    let has_rule_at = |expected_x: f32| {
        painter.primitives().fills.iter().any(|f| {
            f.color.a > 0 && (f.rect.origin.x - expected_x).abs() < 0.5 && (f.rect.size.width - 3.0).abs() < 0.1
        })
    };
    assert!(
        has_rule_at(195.833),
        "column-width:10em → 3 列 rule@i=1 应在 x≈195.83（修复前 em 不画 rule）"
    );
    assert!(
        has_rule_at(401.167),
        "column-width:10em → 3 列 rule@i=2 应在 x≈401.17（修复前 em 不画 rule）"
    );
}

/// 测试 column-rule-style: none 不绘制分隔线。
#[test]
fn test_column_rules_none() {
    use zero_style_system::ColumnCountComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 600.0, 200.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    // column_rule_style 默认为 None
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // column-rule:none 不应产生额外的 stroke 图元
    let strokes = &painter.primitives().strokes;
    assert_eq!(strokes.len(), 0, "column-rule:none should not generate strokes");
}

/// 测试 column-rule-style: dashed 生成 stroke 图元。
#[test]
fn test_column_rules_dashed() {
    use zero_render_foundation::primitive::LineStyle;
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 400.0, 100.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_gap = LengthValue::Px(10.0);
    style.column_rule_style = ColumnRuleStyleComputedValue::Dashed;
    style.column_rule_width = ColumnRuleWidthComputedValue::Medium;
    style.column_rule_color = ColorValue::Rgba(0, 0, 0, 255);
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let strokes = &painter.primitives().strokes;
    let dashed: Vec<_> = strokes.iter().filter(|s| s.style == LineStyle::Dashed).collect();
    assert!(
        dashed.len() >= 1,
        "2 columns with dashed rule should produce at least 1 dashed stroke, got {}",
        dashed.len()
    );
}

/// 测试 column-count:1（只有 1 列）不绘制 rule。
#[test]
fn test_column_rules_single_column() {
    use zero_style_system::{ColumnCountComputedValue, ColumnRuleStyleComputedValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(1);
    style.column_rule_style = ColumnRuleStyleComputedValue::Solid;
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let strokes = &painter.primitives().strokes;
    assert_eq!(strokes.len(), 0, "1 column should not produce column-rule strokes");
}

/// 测试 list-style-image:url() 生成 ImagePrimitive 标记。
#[test]
fn test_list_style_image_url() {
    use zero_style_system::ListStyleImageComputedValue;

    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_image = ListStyleImageComputedValue::Url("bullet.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let images = &painter.primitives().images;
    assert!(
        images
            .iter()
            .any(|img| img.rect.size.width > 0.0 && img.rect.size.height > 0.0),
        "list-style-image should generate at least one image primitive"
    );
}

/// 测试 list-style-image:none 不生成图片图元。
#[test]
fn test_list_style_image_none() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    // list-style-image defaults to None
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Disc;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let images = &painter.primitives().images;
    assert_eq!(
        images.len(),
        0,
        "list-style-image:none should not generate image primitives"
    );
}

/// R1882：list-style-type:disc 生成实心圆 marker（圆角矩形 radius=size/2），非方块。
///
/// CSS §12.5 / chromium：disc 为实心圆。旧实现用 add_fill(Rect) 绘方块。修复后用
/// RoundedRectPrimitive（radius = marker_size/2 = 正方形四角全圆 → 圆）近似实心圆。
#[test]
fn test_list_style_disc_renders_circle() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Disc;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // disc 应产出 rounded_rect（实心圆），非 fill（方块）。
    assert!(
        !prims.rounded_rects.is_empty(),
        "list-style-type:disc 应生成 rounded_rect（实心圆 marker），实际 rounded_rects 为空"
    );
    let r = &prims.rounded_rects[0];
    // 圆 = 正方形四角 radius = size/2。
    let size = r.rect.size.width;
    assert!(
        (r.top_left_radius - size / 2.0).abs() < 0.01,
        "disc marker 应四角 radius=size/2（实心圆），实际 top_left_radius={} size={}",
        r.top_left_radius,
        size
    );
    assert!(
        prims
            .fills
            .iter()
            .all(|f| (f.rect.size.width - f.rect.size.height).abs() > 0.5 || f.rect.size.width < 2.0),
        "disc marker 不应残留方块 fill（与圆 marker 同尺寸的 fill 应消失）"
    );
}

/// R1883：list-style-type:circle 生成空心圆 outline（PathStroke 多边形），非 2:1 胶囊。
///
/// CSS §12.5 / chromium：circle 为空心圆。旧实现用 add_stroke（length=width + Round cap）
/// 实为 2:1 胶囊（椭圆）。修复后用 add_path_stroke 多边形（24 点圆周）描真圆。
#[test]
fn test_list_style_circle_renders_true_circle() {
    let mut doc = zero_dom::Document::new();
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let _ = doc.append_child(ul, li);

    let layout = make_box(Some(li), 0.0, 0.0, 200.0, 30.0);

    let mut style = ComputedStyle::default();
    style.list_style_type = zero_css_parser::values::ListStyleTypeValue::Circle;
    let mut styles = HashMap::new();
    styles.insert(li, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // circle 应产出 path_stroke（多边形真圆），非 stroke（线段胶囊）。
    assert!(
        !prims.path_strokes.is_empty(),
        "list-style-type:circle 应生成 path_stroke（多边形真圆 outline），实际 path_strokes 为空"
    );
    // 24 点圆周 = 48 个 f32 顶点。
    assert_eq!(
        prims.path_strokes[0].vertices.len(),
        48,
        "circle marker 应为 24 点圆周多边形（48 f32），实际 {}",
        prims.path_strokes[0].vertices.len()
    );
}

/// 测试 empty-cells:hide 跳过空单元格的背景绘制。
#[test]
fn test_empty_cells_hide() {
    use zero_style_system::EmptyCellsComputedValue;

    let mut doc = zero_dom::Document::new();
    let td = doc.create_element("td");
    let layout = make_box(Some(td), 0.0, 0.0, 100.0, 50.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.empty_cells = EmptyCellsComputedValue::Hide;
    // No children → empty cell
    let mut styles = HashMap::new();
    styles.insert(td, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // empty-cells:hide should skip background for empty cell
    let fills = &painter.primitives().fills;
    assert!(
        fills.iter().all(|f| f.color.r != 255 || f.color.a == 0),
        "empty-cells:hide should not render background for empty cell"
    );
}

/// 测试 empty-cells:show 绘制空单元格的背景。
#[test]
fn test_empty_cells_show() {
    use zero_style_system::EmptyCellsComputedValue;

    let mut doc = zero_dom::Document::new();
    let td = doc.create_element("td");
    let layout = make_box(Some(td), 0.0, 0.0, 100.0, 50.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.empty_cells = EmptyCellsComputedValue::Show;
    let mut styles = HashMap::new();
    styles.insert(td, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(
        fills.iter().any(|f| f.color.r == 255 && f.color.a > 0),
        "empty-cells:show should render background for empty cell"
    );
}

// ═══════════════════════════════════════════════════════════════
//  CSS mix-blend-mode 渲染集成测试
// ═══════════════════════════════════════════════════════════════

/// 测试 mix-blend-mode:multiply 生成 BlendModePrimitive。
#[test]
fn test_mix_blend_mode_multiply_generates_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Multiply;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:multiply should generate BlendModePrimitive"
    );
    assert_eq!(
        painter.primitives().blend_modes[0].mode,
        zero_render_foundation::primitive::BlendMode::Multiply
    );
}

/// 测试 mix-blend-mode:normal 不生成 BlendModePrimitive。
#[test]
fn test_mix_blend_mode_normal_no_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Normal;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:normal should not generate BlendModePrimitive"
    );
}

/// 测试 mix-blend-mode:screen 生成正确模式。
#[test]
fn test_mix_blend_mode_screen_generates_blend_primitive() {
    use zero_style_system::MixBlendModeComputedValue;

    let mut doc = zero_dom::Document::new();
    let div = doc.create_element("div");
    let layout = make_box(Some(div), 0.0, 0.0, 200.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    style.mix_blend_mode = MixBlendModeComputedValue::Screen;

    let mut styles = HashMap::new();
    styles.insert(div, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        !painter.primitives().blend_modes.is_empty(),
        "mix-blend-mode:screen should generate BlendModePrimitive"
    );
    assert_eq!(
        painter.primitives().blend_modes[0].mode,
        zero_render_foundation::primitive::BlendMode::Screen
    );
}
