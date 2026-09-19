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
    // R3760：repeat tile 溢出 painting area 时图元语义改为「rect = 完整 tile 尺寸 +
    // clip = 与 painting area 交集」（crop 不重缩放，修复 cover/contain 溢出重缩放）。
    // 约束改为：rect 起点不越 painting area 左/上边界；溢出 tile 必须携带 clip 且
    // clip 完全在 painting area 内（渲染可见部分不越界）。
    for img in &prims.images {
        assert!(
            img.rect.origin.x >= -0.1,
            "tile 不应超出左边界: x={}",
            img.rect.origin.x
        );
        assert!(
            img.rect.origin.y >= -0.1,
            "tile 不应超出上边界: y={}",
            img.rect.origin.y
        );
        if img.rect.right() > 50.1 || img.rect.origin.y + img.rect.size.height > 50.1 {
            let clip = img.clip.expect("溢出 painting area 的 tile 必须携带 clip");
            assert!(
                clip.right() <= 50.1 && clip.origin.y + clip.size.height <= 50.1,
                "clip 应完全在 painting area 内: clip_right={}",
                clip.right()
            );
        }
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
    p1.paint_bg_image_in_origin(
        0.0,
        0.0,
        100.0,
        100.0,
        0.0,
        0.0,
        100.0,
        100.0,
        &mk_style(),
        50.0,
        50.0,
        None,
        false,
        None,
        None,
        None,
        None,
    );
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
    p0.paint_bg_image_in_origin(
        0.0,
        0.0,
        100.0,
        100.0,
        0.0,
        0.0,
        100.0,
        100.0,
        &mk_style(),
        0.0,
        0.0,
        None,
        false,
        None,
        None,
        None,
        None,
    );
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
    painter.paint_bg_image_in_origin(
        0.0,
        0.0,
        800.0,
        600.0,
        72.0,
        72.0,
        96.0,
        192.0,
        &mk_style(),
        0.0,
        0.0,
        None,
        false,
        None,
        None,
        None,
        None,
    );
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
    painter2.paint_bg_image_in_origin(
        72.0,
        72.0,
        96.0,
        192.0,
        72.0,
        72.0,
        96.0,
        192.0,
        &mk_style(),
        0.0,
        0.0,
        None,
        false,
        None,
        None,
        None,
        None,
    );
    let g2 = &painter2.primitives().gradients;
    assert!(g2.len() >= 1);
    assert!(
        (g2[0].rect.left() - 72.0).abs() < 0.5 && (g2[0].rect.top() - 72.0).abs() < 0.5,
        "scroll bg gradient 应锚定元素盒 (72,72)，got ({}, {})",
        g2[0].rect.left(),
        g2[0].rect.top()
    );
}

/// R3908：background-clip: border-area（css-backgrounds-4 §2.1）——背景图仅绘制在边框
/// 环带（border-box 减 padding-box，4 条带）。tile 覆盖整盒时，逐带 clip 发射 4 枚
/// ImagePrimitive（条带互不重叠无双绘），clip 外（padding 区）无图元覆盖。
#[test]
fn test_background_clip_border_area_ring() {
    use zero_style_system::{BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundRepeatComputedValue};

    let mk_style = || {
        let mut style = ComputedStyle::default();
        style.background_image = vec![BackgroundImageComputedValue::Url("test.png".to_string())];
        style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
        style.background_clip = vec![BackgroundClipComputedValue::BorderArea];
        style.color = zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255);
        style
    };

    // 100×100 盒、border 20 → 环带 = 外 100×100 减内 60×60。
    let ring = Some(vec![
        zero_render_foundation::geometry::Rect::new(0.0, 0.0, 100.0, 20.0),
        zero_render_foundation::geometry::Rect::new(0.0, 80.0, 100.0, 20.0),
        zero_render_foundation::geometry::Rect::new(0.0, 20.0, 20.0, 60.0),
        zero_render_foundation::geometry::Rect::new(80.0, 20.0, 20.0, 60.0),
    ]);

    let mut painter = Painter::new();
    painter.paint_bg_image_in_origin(
        0.0,
        0.0,
        100.0,
        100.0,
        0.0,
        0.0,
        100.0,
        100.0,
        &mk_style(),
        0.0,
        0.0,
        ring,
        false,
        None,
        None,
        None,
        None,
    );

    let images = &painter.primitives().images;
    assert!(!images.is_empty(), "border-area 环带应发射图元");
    // 每枚图元 clip 必须完全落在环带内：clip 矩形与内盒 (20,20,60,60) 无重叠。
    let inner = zero_render_foundation::geometry::Rect::new(20.0, 20.0, 60.0, 60.0);
    for img in images {
        if let Some(clip) = &img.clip {
            let overlap = !(clip.right() <= inner.left()
                || clip.left() >= inner.right()
                || clip.bottom() <= inner.top()
                || clip.top() >= inner.bottom());
            assert!(
                !overlap,
                "border-area 图元 clip 不得伸入 padding 区：clip=({}, {}, {}, {})",
                clip.left(),
                clip.top(),
                clip.size.width,
                clip.size.height
            );
        } else {
            panic!("border-area 图元必须携带环带 clip，不得整 tile 无裁剪发射");
        }
    }
    // 环带覆盖率 = 4 条带面积和（tile 覆盖整盒时）：10000 - 3600 = 6400。
    let covered: f32 = images
        .iter()
        .filter_map(|img| img.clip.as_ref())
        .map(|c| c.size.width * c.size.height)
        .sum();
    assert!(
        (covered - 6400.0).abs() < 1.0,
        "环带 clip 面积和应 = border 面积 6400，got {covered}"
    );
}

/// R3908 守卫：border-area 但无 border（环带为空）→ 不发射任何图元。
#[test]
fn test_background_clip_border_area_no_border_emits_nothing() {
    use zero_style_system::{BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundRepeatComputedValue};

    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("test.png".to_string())];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    style.background_clip = vec![BackgroundClipComputedValue::BorderArea];
    style.color = zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255);

    let mut painter = Painter::new();
    // 无边框 → 调用方传空条带集（Some(empty)）：所有 tile 不绘。
    painter.paint_bg_image_in_origin(
        0.0,
        0.0,
        100.0,
        100.0,
        0.0,
        0.0,
        100.0,
        100.0,
        &style,
        0.0,
        0.0,
        Some(Vec::new()),
        false,
        None,
        None,
        None,
        None,
    );
    assert!(
        painter.primitives().images.is_empty(),
        "环带为空（空条带集）时 tile 全部不绘"
    );
}

/// R4529：border-area 环带 border-style 感知——double 边框的墨迹 = 外带 + 内带
///（镜像 paint_border_edge：gap = max(t/3,1)、line_w = max((t−gap)/2,1)），中缝无墨迹
/// 不绘背景图（clip-border-area-double：ref 双带间白隙）。300×150 盒 border 50 双线 →
/// 8 条带，矩形即墨迹区间本身。
#[test]
fn test_r4529_double_border_area_ring_two_bands() {
    use zero_style_system::BorderStyleValue;

    let mut style = ComputedStyle::default();
    for s in [
        &mut style.border_top_style,
        &mut style.border_right_style,
        &mut style.border_bottom_style,
        &mut style.border_left_style,
    ] {
        *s = BorderStyleValue::Double;
    }

    let mut layout = make_box(None, 0.0, 0.0, 300.0, 150.0);
    layout.border_top = 50.0;
    layout.border_right = 50.0;
    layout.border_bottom = 50.0;
    layout.border_left = 50.0;

    let strips = crate::paint::painter::border_area_ring_strips(&style, &layout, 0.0, 0.0, 300.0, 150.0)
        .expect("border-area 有边框 → Some(条带集)");
    assert_eq!(strips.len(), 8, "double 墨迹 = 8 条带（4 侧 × 外/内带）");
    let lw = 50.0 / 3.0;
    // 条带序 = 侧序 × 侧内区间序（top-out/in、bottom-out/in、left-out/in、right-out/in）。
    let expected = [
        (0.0, 0.0, 300.0, lw),              // top 外带
        (0.0, 2.0 * lw, 300.0, lw),         // top 内带
        (0.0, 150.0 - lw, 300.0, lw),       // bottom 带一（外缘侧）
        (0.0, 150.0 - 3.0 * lw, 300.0, lw), // bottom 带二（内缘侧）
        (0.0, 50.0, lw, 50.0),              // left 外带（padding 盒竖向区间）
        (2.0 * lw, 50.0, lw, 50.0),         // left 内带
        (300.0 - lw, 50.0, lw, 50.0),       // right 外带
        (300.0 - 3.0 * lw, 50.0, lw, 50.0), // right 内带
    ];
    for (strip, (ex, ey, ew, eh)) in strips.iter().zip(expected.iter()) {
        assert!(
            (strip.left() - ex).abs() < 0.01
                && (strip.top() - ey).abs() < 0.01
                && (strip.size.width - ew).abs() < 0.01
                && (strip.size.height - eh).abs() < 0.01,
            "条带几何不匹配：got ({}, {}, {}, {}) want ({}, {}, {}, {})",
            strip.left(),
            strip.top(),
            strip.size.width,
            strip.size.height,
            ex,
            ey,
            ew,
            eh
        );
    }
}

/// R4529：solid 边框的墨迹 = 整带——环带几何与 R3908 4 条带逐位一致（byte-identical
/// 守卫：既有 border-area 绿基线不受墨迹化改动影响）。
#[test]
fn test_r4529_solid_border_area_ring_unchanged() {
    use zero_style_system::BackgroundClipComputedValue;

    let style = ComputedStyle::default();
    let mut layout = make_box(None, 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 20.0;
    layout.border_right = 20.0;
    layout.border_bottom = 20.0;
    layout.border_left = 20.0;

    let strips = crate::paint::painter::border_area_ring_strips(&style, &layout, 0.0, 0.0, 100.0, 100.0).unwrap();
    let got: Vec<(f32, f32, f32, f32)> = strips
        .iter()
        .map(|r| (r.left(), r.top(), r.size.width, r.size.height))
        .collect();
    let want = [
        (0.0, 0.0, 100.0, 20.0),
        (0.0, 80.0, 100.0, 20.0),
        (0.0, 20.0, 20.0, 60.0),
        (80.0, 20.0, 20.0, 60.0),
    ];
    assert_eq!(got, want, "solid 整带环带 = R3908 4 条带原几何");
}

/// R4530：dashed 边框墨迹 = 逐 dash 沿边区间（dash=2t、gap=t，相位锚定线起点）——
/// 条带按像素中心闭区间整数化（与 cpu render_image floor/ceil clip 窗口逐像素等价）。
/// 300×150 盒 border 50：top/bottom 各 2 dash（[0,100]、[150,250]），left/right 各 1
///（线长 50 < pattern）。
#[test]
fn test_r4530_dashed_border_area_ring_dash_intervals() {
    use zero_style_system::BorderStyleValue;

    let mut style = ComputedStyle::default();
    for s in [
        &mut style.border_top_style,
        &mut style.border_right_style,
        &mut style.border_bottom_style,
        &mut style.border_left_style,
    ] {
        *s = BorderStyleValue::Dashed;
    }
    let mut layout = make_box(None, 0.0, 0.0, 300.0, 150.0);
    layout.border_top = 50.0;
    layout.border_right = 50.0;
    layout.border_bottom = 50.0;
    layout.border_left = 50.0;

    let strips = crate::paint::painter::border_area_ring_strips(&style, &layout, 0.0, 0.0, 300.0, 150.0).unwrap();
    let got: Vec<(f32, f32, f32, f32)> = strips
        .iter()
        .map(|r| (r.left(), r.top(), r.size.width, r.size.height))
        .collect();
    let want = [
        (0.0, 0.0, 100.0, 50.0),     // top dash1
        (150.0, 0.0, 100.0, 50.0),   // top dash2
        (0.0, 100.0, 100.0, 50.0),   // bottom dash1
        (150.0, 100.0, 100.0, 50.0), // bottom dash2
        (0.0, 50.0, 50.0, 50.0),     // left（线长 50 ≤ dash 整段）
        (250.0, 50.0, 50.0, 50.0),   // right
    ];
    assert_eq!(got, want, "dashed 墨迹 = 逐 dash 矩形（整数化像素区间）");
}

/// R4530：dotted 边框墨迹 = 逐 dot 逐行圆盘弦段（圆心距 2t、r=t/2，镜像
/// render_dotted_line 的 dx²+dy²<=r² 像素中心判定）。100×100 盒 border 20：
/// 上下各 3 dot × 20 行、左右各 2 dot × 20 行 = 200 条带；全部不得伸入 padding 盒，
/// 且含 (30,9,20,1) 特征弦段（cx=40 dot、dy=−0.5 行）。
#[test]
fn test_r4530_dotted_border_area_ring_dot_rows() {
    use zero_style_system::BorderStyleValue;

    let mut style = ComputedStyle::default();
    for s in [
        &mut style.border_top_style,
        &mut style.border_right_style,
        &mut style.border_bottom_style,
        &mut style.border_left_style,
    ] {
        *s = BorderStyleValue::Dotted;
    }
    let mut layout = make_box(None, 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 20.0;
    layout.border_right = 20.0;
    layout.border_bottom = 20.0;
    layout.border_left = 20.0;

    let strips = crate::paint::painter::border_area_ring_strips(&style, &layout, 0.0, 0.0, 100.0, 100.0).unwrap();
    assert_eq!(strips.len(), 200, "上下 3 dot + 左右 2 dot × 每点 20 行");
    let inner = zero_render_foundation::geometry::Rect::new(20.0, 20.0, 60.0, 60.0);
    let mut spot = false;
    for s in &strips {
        let overlap = !(s.right() <= inner.left()
            || s.left() >= inner.right()
            || s.bottom() <= inner.top()
            || s.top() >= inner.bottom());
        assert!(
            !overlap,
            "dot 弦段不得伸入 padding 盒：({}, {}, {}, {})",
            s.left(),
            s.top(),
            s.size.width,
            s.size.height
        );
        if (s.left() - 30.0).abs() < 0.01
            && (s.top() - 9.0).abs() < 0.01
            && (s.size.width - 20.0).abs() < 0.01
            && (s.size.height - 1.0).abs() < 0.01
        {
            spot = true;
        }
    }
    assert!(spot, "应含 cx=40 dot 的 dy=−0.5 行弦段 (30,9,20,1)");
}

/// R4529（slice-2）：逐层 painting area——多值 clip（`border-area, content-box`）时各层
/// 按己值裁剪（css-backgrounds-3 §3.7 cyclic）：layer0 图元 clip 落在环带，layer1 图元
/// clip 落在 content 盒；单值 clip 页全层同值 = 旧行为。
#[test]
fn test_r4529_per_layer_clip_rects() {
    use zero_style_system::{
        BackgroundClipComputedValue, BackgroundImageComputedValue, BackgroundPositionComputedValue,
        BackgroundRepeatComputedValue,
    };

    let mut style = ComputedStyle::default();
    style.background_image = vec![
        BackgroundImageComputedValue::Url("a.png".to_string()),
        BackgroundImageComputedValue::Url("b.png".to_string()),
    ];
    style.background_repeat = vec![BackgroundRepeatComputedValue::NoRepeat];
    // tile 100×100 定位 (0,0)（origin=padding 盒 (20,20) + offset (−20,−20)）——覆盖整盒，
    // 使 border-area 环带与 content 盒两类 clip 都被 tile 命中。
    style.background_size = vec![
        BackgroundSizeComputedValue::Length(100.0),
        BackgroundSizeComputedValue::Length(100.0),
    ];
    style.background_position = vec![
        BackgroundPositionComputedValue::TwoValue(
            Box::new(BackgroundPositionComputedValue::Length(-20.0)),
            Box::new(BackgroundPositionComputedValue::Length(-20.0)),
        ),
        BackgroundPositionComputedValue::TwoValue(
            Box::new(BackgroundPositionComputedValue::Length(-20.0)),
            Box::new(BackgroundPositionComputedValue::Length(-20.0)),
        ),
    ];
    style.background_clip = vec![
        BackgroundClipComputedValue::BorderArea,
        BackgroundClipComputedValue::ContentBox,
    ];
    style.color = zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255);

    // 100×100 盒、border 20、padding 10 → content 盒 = (30, 30, 40, 40)。
    let mut layout = make_box(None, 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 20.0;
    layout.border_right = 20.0;
    layout.border_bottom = 20.0;
    layout.border_left = 20.0;
    layout.padding_left = 10.0;
    layout.padding_right = 10.0;
    layout.padding_top = 10.0;
    layout.padding_bottom = 10.0;
    layout.content_width = 40.0;
    layout.content_height = 40.0;

    let mut painter = Painter::new();
    // 走完整 paint 入口（paint_background_image 对 tests 私有——端到端覆盖 caller
    // 预计算 + 逐层消费两段）。
    let mut styles = HashMap::new();
    let doc_node = zero_dom::Document::new().create_element("div");
    styles.insert(doc_node, style);
    layout.node_id = Some(doc_node);
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    // layer0（border-area）tile 覆盖整盒 → 4 条带各 1 枚；layer1（content-box）1 枚。
    assert_eq!(images.len(), 5, "环带 4 枚 + content 1 枚");
    let content = zero_render_foundation::geometry::Rect::new(30.0, 30.0, 40.0, 40.0);
    let inner = zero_render_foundation::geometry::Rect::new(20.0, 20.0, 60.0, 60.0);
    let mut ring_clips = 0;
    let mut content_clips = 0;
    for img in images {
        let clip = img.clip.as_ref().expect("逐层图元必须携带 clip");
        let within_content = clip.left() >= content.left() - 0.01
            && clip.top() >= content.top() - 0.01
            && clip.right() <= content.right() + 0.01
            && clip.bottom() <= content.bottom() + 0.01;
        let touches_ring = clip.left() < 20.0 - 0.01
            || clip.top() < 20.0 - 0.01
            || clip.right() > 80.0 + 0.01
            || clip.bottom() > 80.0 + 0.01;
        if within_content {
            content_clips += 1;
        } else if touches_ring {
            // 环带层 clip 不得伸入 padding 区。
            let overlap_inner = !(clip.right() <= inner.left()
                || clip.left() >= inner.right()
                || clip.bottom() <= inner.top()
                || clip.top() >= inner.bottom());
            assert!(!overlap_inner, "border-area 层 clip 不得伸入 padding 区");
            ring_clips += 1;
        }
    }
    assert_eq!(ring_clips, 4, "layer0（border-area）应恰有 4 枚环带条带图元");
    assert_eq!(content_clips, 1, "layer1（content-box）应恰有 1 枚 content 图元");
}

/// R3923：repeat tile 网格相位锚定 background-position（CSS Backgrounds §3.4）。
/// origin=padding-box（8..92）且 clip=border-box（0..100）时 positioned=(8,8) > clip 原点：
/// 网格 = positioned + k*sized（…,-40, 8, 56,…），非旧「对齐 clip 边」（-8, 40, 88）。
/// driving：css-backgrounds background-origin/origin-padding-box_with_size（chromium 实证
/// tile 内容偏移 16px 缺陷）。
#[test]
fn test_r3923_repeat_grid_phase_anchored_at_position() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let mut layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 8.0;
    layout.border_left = 8.0;

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("tile.png".to_string())];
    // 48px tile：positioned=(8,8)；网格相位 ≡ 8 (mod 48)。
    style.background_size = vec![BackgroundSizeComputedValue::Length(48.0)];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    let xs: Vec<f32> = prims.images.iter().map(|i| i.rect.origin.x).collect();
    // 相位正确性：所有 tile x ≡ 8 (mod 48)（covering clip 0..100：-40, 8, 56）。
    let phase_ok = xs
        .iter()
        .all(|x| ((x - 8.0) % 48.0).abs() < 0.001 || ((x - 8.0) % 48.0 + 48.0).abs() < 0.001);
    assert!(
        phase_ok,
        "tile x 应满足 x ≡ 8 (mod 48)（相位锚定 positioned），实际 {xs:?}"
    );
    assert!(
        xs.iter().any(|&x| (x + 40.0).abs() < 0.001),
        "应存在后向 tile x=-40（覆盖 clip 左条带），实际 {xs:?}"
    );
    assert!(
        xs.iter().any(|&x| (x - 8.0).abs() < 0.001),
        "首 tile 应锚定 positioned x=8，实际 {xs:?}"
    );
    assert!(
        !xs.iter().any(|&x| (x + 8.0).abs() < 0.001),
        "不应出现旧 clip 对齐相位 x=-8，实际 {xs:?}"
    );
}
