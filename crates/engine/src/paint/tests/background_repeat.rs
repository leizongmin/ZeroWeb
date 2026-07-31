//! background-repeat 渲染集成测试。
//!
//! 覆盖 repeat/repeat-x/repeat-y/no-repeat/space/round 六种模式。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::{
    BackgroundImageComputedValue, BackgroundRepeatComputedValue, BackgroundSizeComputedValue, ComputedStyle,
};

use super::super::painter::Painter;

/// 辅助函数：创建简单 LayoutBox。
fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: 0.0,
        content_y: 0.0,
        content_width: width,
        content_height: height,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    }
}

/// 默认 repeat 模式下，背景图片铺满容器。
#[test]
fn test_background_repeat_default() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(50.0)];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 默认 repeat 模式：50px 宽 tile 在 100px 容器中应生成 2 列
    // 50px 高 tile 在 50px 容器中应生成 1 行
    assert!(
        prims.images.len() >= 2,
        "repeat 默认应生成多个 tile，实际 {}",
        prims.images.len()
    );
}

/// repeat-x 模式：仅水平平铺。
#[test]
fn test_background_repeat_x() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(30.0)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::RepeatX];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 30px tile 在 100px 容器中：水平 4 个，垂直 1 个
    assert!(
        prims.images.len() >= 3,
        "repeat-x 应水平平铺，实际 {}",
        prims.images.len()
    );

    // 所有 tile 的 y 应一致（单行）
    let first_y = prims.images[0].rect.origin.y;
    for img in &prims.images {
        assert!(
            (img.rect.origin.y - first_y).abs() < 1.0,
            "repeat-x 所有 tile 应在同一行"
        );
    }
}

/// repeat-y 模式：仅垂直平铺。
#[test]
fn test_background_repeat_y() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 使用正方形容器，避免背景尺寸按容器宽高比缩放
    let layout = make_box(Some(elem), 0.0, 0.0, 50.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    // 使用百分比尺寸确保正方形 tile
    style.background_size = vec![BackgroundSizeComputedValue::Percent(30.0)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::RepeatY];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 30% of 50 = 15px tile，在 50px 容器中约 3-4 行，但水平只有 1 列
    assert!(
        prims.images.len() >= 3,
        "repeat-y 应垂直平铺，实际 {}",
        prims.images.len()
    );

    // 所有 tile 的 x 应一致（单列）
    let first_x = prims.images[0].rect.origin.x;
    for img in &prims.images {
        assert!(
            (img.rect.origin.x - first_x).abs() < 1.0,
            "repeat-y 所有 tile 应在同一列"
        );
    }
}

/// no-repeat 模式：仅生成单个 tile。
#[test]
fn test_background_no_repeat() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(30.0)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.images.len(), 1, "no-repeat 应只生成 1 个 tile");
}

/// no-repeat + 默认尺寸：图片占满容器。
#[test]
fn test_background_no_repeat_auto_size() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("bg.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.images.len(), 1);
    assert_eq!(prims.images[0].rect.size.width, 100.0);
    assert_eq!(prims.images[0].rect.size.height, 50.0);
}

/// round 模式：缩放 tile 使整数个刚好覆盖容器。
#[test]
fn test_background_repeat_round() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(30.0)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::Round];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // round 模式：100/30 ≈ 3.33 → round = 3 个 tile，每个宽 100/3 ≈ 33.33
    // 垂直：50/30 ≈ 1.67 → round = 2 个 tile，每个高 50/2 = 25
    let _expected_count = 3 * 2; // 6 个 tile
    assert!(
        prims.images.len() >= 4,
        "round 模式应平铺覆盖容器，实际 {}",
        prims.images.len()
    );

    // 验证 tile 宽度一致
    let first_w = prims.images[0].rect.size.width;
    for img in &prims.images {
        assert!(
            (img.rect.size.width - first_w).abs() < 1.0,
            "round 模式所有 tile 宽度应一致"
        );
    }
}

/// space 模式：均匀分布 tile。
#[test]
fn test_background_repeat_space() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(30.0)];
    style.background_repeat = vec![BackgroundRepeatComputedValue::Space];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // space 模式：100/30 = 3 个 tile（floor），间距 = (100 - 90) / 2 = 5
    assert!(
        prims.images.len() >= 2,
        "space 模式应均匀分布多个 tile，实际 {}",
        prims.images.len()
    );
}

/// repeat 模式：小 tile 应生成大量平铺。
#[test]
fn test_background_repeat_many_tiles() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tiny.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(10.0)];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 10px tile 在 100x100 容器中：10x10 = 100 个 tile
    assert!(
        prims.images.len() >= 90,
        "10px tile 在 100x100 容器中应生成约 100 个，实际 {}",
        prims.images.len()
    );
}

/// no-repeat 渐变不受影响。
#[test]
fn test_background_repeat_gradient_unchanged() {
    use zero_css_parser::values::{GradientColorStop, GradientDirection, GradientValue, LinearGradient};
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
        LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::Angle(90.0),
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: Some(LengthValue::Px(0.0)),
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: Some(LengthValue::Px(100.0)),
                },
            ],
            repeating: false,
        },
    ))];
    style.background_repeat = vec![BackgroundRepeatComputedValue::Repeat];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 渐变不受 repeat 影响，仍生成单个 gradient primitive
    assert_eq!(prims.images.len(), 0, "渐变不应生成 image primitives");
    assert!(prims.gradients.len() >= 1, "渐变应生成 gradient primitive");
}

/// repeat 模式下 tile 不超出 origin 区域。
#[test]
fn test_background_repeat_clips_to_origin() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 50.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.background_size = vec![BackgroundSizeComputedValue::Length(30.0)];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 验证所有 tile 都在容器范围内
    for img in &prims.images {
        assert!(
            img.rect.origin.x >= -0.1,
            "tile 不应超出左边界: x={}",
            img.rect.origin.x
        );
        assert!(
            img.rect.right() <= 50.1,
            "tile 不应超出右边界: right={}",
            img.rect.right()
        );
        assert!(
            img.rect.origin.y >= -0.1,
            "tile 不应超出上边界: y={}",
            img.rect.origin.y
        );
        assert!(
            img.rect.origin.y + img.rect.size.height <= 50.1,
            "tile 不应超出下边界: bottom={}",
            img.rect.origin.y + img.rect.size.height
        );
    }
}

/// 零尺寸容器不生成 tile。
#[test]
fn test_background_repeat_zero_container() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 0.0, 0.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.images.len(), 0, "零尺寸容器不应生成 tile");
}

/// R1428：canvas 传播背景图 anchor（根元素盒偏移）应平移 positioned 位置。
///
/// CSS §14.2.3：根背景传播到画布时，背景图 positioning area = 根元素盒（含 margin 偏移），
/// painting area = 画布。修复前 paint_bg_image_in_origin 把 origin 同时当锚和绘制区，canvas
/// 调用传 (0,0) 致锚定画布左上（background-root-002 html margin:1in 时绿条 y=0 应 y=96）。
/// 修复：加 anchor_x/y 参数，positioned = origin + offset + anchor；canvas 传根盒 layout.x/y。
/// 本测试直接调 paint_bg_image_in_origin 验证 anchor 平移 gradient primitive 位置。
#[test]
fn r1428_canvas_bg_image_anchor_shifts_gradient_position() {
    use zero_css_parser::values::{GradientColorStop, GradientDirection, GradientValue, LinearGradient};
    let mk_style = || {
        let mut style = ComputedStyle::default();
        style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
            LinearGradient {
                interpolation: Default::default(),
                direction: GradientDirection::Angle(90.0),
                stops: vec![
                    GradientColorStop {
                        color: ColorValue::Rgba(255, 0, 0, 255),
                        position: Some(LengthValue::Px(0.0)),
                    },
                    GradientColorStop {
                        color: ColorValue::Rgba(0, 0, 255, 255),
                        position: Some(LengthValue::Px(100.0)),
                    },
                ],
                repeating: false,
            },
        ))];
        style.color = ColorValue::CurrentColor;
        style
    };

    // anchor=(50,50)（canvas 根盒偏移）：positioned = origin(0) + offset(0,bg-pos 默认) + anchor(50) = 50。
    let mut p1 = Painter::new();
    p1.paint_bg_image_in_origin(0.0, 0.0, 100.0, 100.0, 0.0, 0.0, 100.0, 100.0, &mk_style(), 50.0, 50.0);
    let g1 = &p1.primitives().gradients;
    assert!(g1.len() >= 1, "R1428: anchor 测试应生成 gradient primitive");
    assert!(
        (g1[0].rect.left() - 50.0).abs() < 0.5 && (g1[0].rect.top() - 50.0).abs() < 0.5,
        "R1428: anchor=(50,50) 应把 gradient 平移到 (50,50)，got ({}, {})",
        g1[0].rect.left(),
        g1[0].rect.top()
    );

    // anchor=(0,0)（正常元素）：positioned = 0。
    let mut p0 = Painter::new();
    p0.paint_bg_image_in_origin(0.0, 0.0, 100.0, 100.0, 0.0, 0.0, 100.0, 100.0, &mk_style(), 0.0, 0.0);
    let g0 = &p0.primitives().gradients;
    assert!(g0.len() >= 1);
    assert!(
        (g0[0].rect.left() - 0.0).abs() < 0.5 && (g0[0].rect.top() - 0.0).abs() < 0.5,
        "R1428: anchor=(0,0) gradient 应在 (0,0)，got ({}, {})",
        g0[0].rect.left(),
        g0[0].rect.top()
    );
}

/// R2063：background-attachment:fixed 的 positioning area = 视口（初始包含块），
/// 非 background-origin 盒。即 fixed 背景「锚定视口、裁剪到元素」。
///
/// 驱动 background-attachment-applies-to-*（10 案）：img fixed + repeat-x，元素仅显示
/// 与视口锚定 tile 重叠的条带。修复前 fixed 当 scroll（锚定元素盒）→ 整块图像。
/// 本测试用 gradient（其 primitive 直接取 positioned_x/y）验证 fixed 时 positioned 锚定视口。
/// 直接调 paint_bg_image_in_origin（pub(crate)）传 fixed-bg 参数（origin=视口、clip=元素盒）。
#[test]
fn r2063_bg_attachment_fixed_positions_against_viewport() {
    use zero_css_parser::values::{GradientColorStop, GradientDirection, GradientValue, LinearGradient};

    let mk_style = || {
        let mut style = ComputedStyle::default();
        style.background_image = vec![BackgroundImageComputedValue::Gradient(GradientValue::Linear(
            LinearGradient {
                interpolation: Default::default(),
                direction: GradientDirection::Angle(90.0),
                stops: vec![
                    GradientColorStop {
                        color: ColorValue::Rgba(255, 0, 0, 255),
                        position: Some(LengthValue::Px(0.0)),
                    },
                    GradientColorStop {
                        color: ColorValue::Rgba(0, 0, 255, 255),
                        position: Some(LengthValue::Px(100.0)),
                    },
                ],
                repeating: false,
            },
        ))];
        style.color = ColorValue::CurrentColor;
        style
    };

    // R2063 fixed：positioning area（origin）= 视口 (0,0,800,600)，painting area（clip）= 元素盒 (72,72,96,192)。
    let mut painter = Painter::new();
    painter.paint_bg_image_in_origin(0.0, 0.0, 800.0, 600.0, 72.0, 72.0, 96.0, 192.0, &mk_style(), 0.0, 0.0);
    let g = &painter.primitives().gradients;
    assert!(g.len() >= 1, "R2063: fixed bg 应生成 gradient primitive");
    // fixed：positioning area = 视口 → positioned 锚定 (0,0)，非元素盒 (72,72)。
    assert!(
        (g[0].rect.left() - 0.0).abs() < 0.5 && (g[0].rect.top() - 0.0).abs() < 0.5,
        "R2063: fixed bg gradient 应锚定视口 (0,0)，got ({}, {})",
        g[0].rect.left(),
        g[0].rect.top()
    );

    // 对照 scroll：origin ≡ clip = 元素盒 (72,72,96,192) → positioned 锚定 (72,72)。
    let mut painter2 = Painter::new();
    painter2.paint_bg_image_in_origin(72.0, 72.0, 96.0, 192.0, 72.0, 72.0, 96.0, 192.0, &mk_style(), 0.0, 0.0);
    let g2 = &painter2.primitives().gradients;
    assert!(g2.len() >= 1);
    assert!(
        (g2[0].rect.left() - 72.0).abs() < 0.5 && (g2[0].rect.top() - 72.0).abs() < 0.5,
        "scroll bg gradient 应锚定元素盒 (72,72)，got ({}, {})",
        g2[0].rect.left(),
        g2[0].rect.top()
    );
}
