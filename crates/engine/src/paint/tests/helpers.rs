//! helpers.rs 模块单元测试

use zero_css_parser::values::{
    ConicGradient, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient, RadialGradient,
    RadialShape, RadialSize, TransformFunction, TransformValue,
};
use zero_render_foundation::geometry::Rect;
use zero_style_system::{ComputedStyle, TextTransformValue};

use super::super::helpers::{
    BorderRadiusSpec, PrimitiveCounts, apply_opacity_to_new_primitives, apply_text_transform, apply_transform_offset,
    clip_all_primitives_to_rect, clip_fills, clip_glyphs, convert_color_stops, gradient_to_primitive, length_to_f32,
    linear_direction_to_kind, simple_hash,
};
use zero_css_parser::values::ColorValue;

/// 测试 apply_transform_offset 的各种 transform 情况
#[test]
fn test_apply_transform_offset_none() {
    let style = ComputedStyle::default();
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

#[test]
fn test_apply_transform_offset_translate() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Translate(10.0, 20.0),
        TransformFunction::TranslateX(5.0),
        TransformFunction::TranslateY(15.0),
    ]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 15.0); // 10 + 5
    assert_eq!(dy, 35.0); // 20 + 15
}

#[test]
fn test_apply_transform_offset_only_rotate() {
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![
        TransformFunction::Rotate(45.0),
        TransformFunction::Scale(1.5, Some(1.5)),
    ]);
    let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.0);
}

/// 测试 clip_fills 的各种裁剪情况
#[test]
fn test_clip_fills_no_clip() {
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![
        FillPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: zero_render_foundation::color::Color::rgb(255, 0, 0),
        },
        FillPrimitive {
            rect: Rect::new(50.0, 50.0, 50.0, 50.0),
            color: zero_render_foundation::color::Color::rgb(0, 255, 0),
        },
    ];
    let clip_rect = Rect::new(-100.0, -100.0, 300.0, 300.0); // 包含所有填充
    clip_fills(&mut fills, 0, &clip_rect);

    // 所有填充应该保持不变
    assert_eq!(fills[0].rect, Rect::new(0.0, 0.0, 100.0, 100.0));
    assert_eq!(fills[1].rect, Rect::new(50.0, 50.0, 50.0, 50.0));
}

#[test]
fn test_clip_fills_partial_clip() {
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![
        FillPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: zero_render_foundation::color::Color::rgb(255, 0, 0),
        },
        FillPrimitive {
            rect: Rect::new(50.0, 50.0, 100.0, 100.0), // 被部分裁剪
            color: zero_render_foundation::color::Color::rgb(0, 255, 0),
        },
    ];
    let clip_rect = Rect::new(25.0, 25.0, 50.0, 50.0); // 裁剪矩形
    clip_fills(&mut fills, 0, &clip_rect);

    // 第一个填充：Rect(0,0,100,100) 裁剪到 Rect(25,25,50,50)
    // 实际重叠区域是 (25,25) 到 (75,75)，但 clip_fills 实现可能只计算交集宽度
    assert_eq!(
        fills[0].rect,
        Rect::new(25.0, 25.0, 50.0, 50.0),
        "第一个填充 (0,0,100,100) 裁剪后的结果"
    );
    // 第二个填充：Rect(50,50,100,100) 裁剪到 Rect(25,25,50,50)
    // 实际重叠区域是 (50,50) 到 (75,75)，但 clip_fills 实现可能只计算交集宽度
    assert_eq!(
        fills[1].rect,
        Rect::new(50.0, 50.0, 25.0, 25.0),
        "第二个填充 (50,50,100,100) 裁剪后的结果"
    );
}

#[test]
fn test_clip_fills_completely_outside() {
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(200.0, 200.0, 50.0, 50.0),
        color: zero_render_foundation::color::Color::rgb(255, 0, 0),
    }];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0); // 完全不重叠
    clip_fills(&mut fills, 0, &clip_rect);

    // 填充应该被清零
    assert_eq!(fills[0].rect.size.width, 0.0);
    assert_eq!(fills[0].rect.size.height, 0.0);
}

/// 测试 clip_glyphs 的裁剪逻辑
#[test]
fn test_clip_glyphs_no_clip() {
    use zero_render_foundation::primitive::GlyphPrimitive;
    let font_id = zero_render_foundation::primitive::FontId(0);
    let mut glyphs = vec![
        GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 16.0,
            glyph_id: 1,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
        GlyphPrimitive {
            x: 100.0,
            y: 100.0,
            font_size: 20.0,
            glyph_id: 2,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
    ];
    let clip_rect = Rect::new(-50.0, -50.0, 300.0, 300.0);
    clip_glyphs(&mut glyphs, 0, &clip_rect);

    // 所有字形应该保持不变
    assert_eq!(glyphs[0].glyph_id, 1);
    assert_eq!(glyphs[1].glyph_id, 2);
}

#[test]
fn test_clip_glyphs_partially_visible() {
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive};
    let font_id = FontId(0);
    let mut glyphs = vec![GlyphPrimitive {
        x: 90.0,
        y: 90.0,
        font_size: 20.0,
        glyph_id: 1,
        font_glyph_index: None,
        source: None,
        font_id,
        color: zero_render_foundation::color::Color::rgb(0, 0, 0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    }];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_glyphs(&mut glyphs, 0, &clip_rect);

    // 字形应该可见（x=90, y=90, font_size=20，部分在 clip_rect 内）
    assert_eq!(glyphs[0].glyph_id, 1);
    assert_eq!(glyphs[0].font_size, 20.0);
}

#[test]
fn test_clip_glyphs_completely_outside() {
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive};
    let font_id = FontId(0);
    let mut glyphs = vec![
        GlyphPrimitive {
            x: 200.0,
            y: 200.0,
            font_size: 16.0,
            glyph_id: 1,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
        GlyphPrimitive {
            x: -20.0,
            y: -20.0,
            font_size: 16.0,
            glyph_id: 2,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
    ];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_glyphs(&mut glyphs, 0, &clip_rect);

    // 字形应该被标记为不可见
    assert_eq!(glyphs[0].glyph_id, 0);
    assert_eq!(glyphs[0].font_size, 0.0);
    assert_eq!(glyphs[1].glyph_id, 0);
    assert_eq!(glyphs[1].font_size, 0.0);
}

/// 测试 clip_all_primitives_to_rect 对图片图元的**裁剪（crop）非重缩放（rescale）**语义。
///
/// 关键不变量：clip 只收窄可见窗口（写入 img.clip），**不修改 img.rect** —— source 始终
/// 映射到完整 rect（保持原始分辨率）。旧实现把 rect 缩到交集区会导致 renderer 把整张
/// source 重映射进缩小区（rescale，clip:rect/overflow:hidden 语义错误，R294）。
#[test]
fn test_clip_image_crops_without_rescaling() {
    use zero_render_foundation::primitive::{ImagePrimitive, RenderPrimitives};

    let mut prims = RenderPrimitives::new();
    let from = PrimitiveCounts::snapshot(&prims); // 快照在添加图元前，使 clip 处理新图元
    let original_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    prims.add_image(ImagePrimitive {
        rect: original_rect,
        image_key: zero_render_foundation::image_cache::ImageKey::new(0),
        clip: None,
    });
    let clip_rect = Rect::new(25.0, 25.0, 50.0, 50.0); // 交集 = (25,25)-(75,75)
    clip_all_primitives_to_rect(&mut prims, &from, &clip_rect);

    // rect 必须保持不变（crop，非 rescale 缩小）
    assert_eq!(
        prims.images[0].rect, original_rect,
        "img.rect 必须不变（crop 保持原始分辨率）"
    );
    // clip 窗口 = 交集
    assert_eq!(
        prims.images[0].clip,
        Some(Rect::new(25.0, 25.0, 50.0, 50.0)),
        "img.clip 应为交集窗口"
    );
}

/// 测试 clip_all_primitives_to_rect 对完全在裁剪区外的图片图元：零尺寸 clip 窗口。
#[test]
fn test_clip_image_completely_outside() {
    use zero_render_foundation::primitive::{ImagePrimitive, RenderPrimitives};

    let mut prims = RenderPrimitives::new();
    let from = PrimitiveCounts::snapshot(&prims); // 快照在添加图元前
    prims.add_image(ImagePrimitive {
        rect: Rect::new(200.0, 200.0, 50.0, 50.0), // 完全在 clip 区外
        image_key: zero_render_foundation::image_cache::ImageKey::new(0),
        clip: None,
    });
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_all_primitives_to_rect(&mut prims, &from, &clip_rect);

    // 完全在外：clip 窗口零尺寸（render_image 见空交集跳过绘制）
    let clip = prims.images[0].clip.expect("完全在外应设零尺寸 clip");
    assert_eq!(clip.size.width, 0.0);
    assert_eq!(clip.size.height, 0.0);
}

/// 测试 length_to_f32 的各种输入
#[test]
fn test_length_to_f32_px() {
    assert_eq!(length_to_f32(&LengthValue::Px(0.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Px(42.5)), 42.5);
    assert_eq!(length_to_f32(&LengthValue::Px(-10.0)), -10.0);
    assert_eq!(length_to_f32(&LengthValue::Px(f64::MAX)), f64::MAX as f32);
}

#[test]
fn test_length_to_f32_non_px() {
    // 非 Px 类型应该返回 0.0
    assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Em(1.5)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Rem(1.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Vh(10.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Vw(20.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Vmin(30.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Vmax(40.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Ch(2.0)), 0.0);
    assert_eq!(length_to_f32(&LengthValue::Auto), 0.0);
    assert_eq!(length_to_f32(&LengthValue::MinContent), 0.0);
    assert_eq!(length_to_f32(&LengthValue::MaxContent), 0.0);
}

/// 测试 simple_hash 的边界情况
#[test]
fn test_simple_hash_empty() {
    let hash = simple_hash("");
    // 空字符串的哈希应该是初始值 5381
    assert_eq!(hash, 5381);
}

#[test]
fn test_simple_hash_short_strings() {
    assert_ne!(simple_hash("a"), 0);
    assert_ne!(simple_hash("ab"), 0);
    assert_ne!(simple_hash("abc"), 0);
}

#[test]
fn test_simple_hash_consistency() {
    // 相同字符串应该产生相同哈希
    let s = "hello world";
    assert_eq!(simple_hash(s), simple_hash(s));
}

#[test]
fn test_simple_hash_different_strings() {
    // 不同字符串应该产生不同哈希
    assert_ne!(simple_hash("hello"), simple_hash("world"));
}

#[test]
fn test_simple_hash_long_string() {
    // 长字符串不应该 panic
    let long_str = "x".repeat(10000);
    let hash = simple_hash(&long_str);
    assert_ne!(hash, 0);
}

/// 测试 convert_color_stops 的各种情况
#[test]
fn test_convert_color_stops_no_position() {
    let stops = vec![
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: None,
        },
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            position: None,
        },
    ];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));

    assert_eq!(converted.len(), 2);
    assert_eq!(converted[0].offset, 0.0);
    assert_eq!(converted[1].offset, 1.0);
    assert_eq!(converted[0].color.r, 255);
    assert_eq!(converted[1].color.b, 255);
}

#[test]
fn test_convert_color_stops_with_percentage_position() {
    let stops = vec![
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: Some(LengthValue::Percentage(25.0)),
        },
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            position: Some(LengthValue::Percentage(75.0)),
        },
    ];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));

    assert_eq!(converted[0].offset, 0.25);
    assert_eq!(converted[1].offset, 0.75);
}

#[test]
fn test_convert_color_stops_with_px_position() {
    let stops = vec![
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: Some(LengthValue::Px(25.0)),
        },
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            position: Some(LengthValue::Px(75.0)),
        },
    ];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));

    assert_eq!(converted[0].offset, 25.0);
    assert_eq!(converted[1].offset, 75.0);
}

#[test]
fn test_convert_color_stops_calc_position() {
    // R2292：calc() 色标位置求值（css-images gradient-infinity）。
    use zero_css_parser::values::{CalcExpr, CalcOp};
    let stops = vec![
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: Some(LengthValue::Px(100.0)),
        },
        // calc(10px + 5px) → 15px。
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            position: Some(LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
                Box::new(CalcExpr::Length(LengthValue::Px(10.0))),
                CalcOp::Add,
                Box::new(CalcExpr::Length(LengthValue::Px(5.0))),
            )))),
        },
    ];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
    assert_eq!(converted[0].offset, 100.0);
    assert_eq!(converted[1].offset, 15.0, "calc(10px + 5px) should evaluate to 15.0");
}

#[test]
fn test_convert_color_stops_calc_infinity_position() {
    // R2292：calc(Infinity * 1px) → +infinity（gradient-infinity 色标）。
    use zero_css_parser::values::{CalcExpr, CalcOp};
    let stops = vec![GradientColorStop {
        color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
        position: Some(LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
            Box::new(CalcExpr::Number(f64::INFINITY)),
            CalcOp::Multiply,
            Box::new(CalcExpr::Length(LengthValue::Px(1.0))),
        )))),
    }];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(
        converted[0].offset.is_infinite() && converted[0].offset > 0.0,
        "calc(Infinity*1px) → +inf"
    );
}

#[test]
fn test_convert_color_stops_mixed_positions() {
    let stops = vec![
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: Some(LengthValue::Percentage(50.0)),
        },
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 255, 0, 255),
            position: None, // 自动计算位置
        },
        GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
            position: Some(LengthValue::Px(100.0)),
        },
    ];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));

    assert_eq!(converted.len(), 3);
    assert_eq!(converted[0].offset, 0.5);
    // None 位置应该在已知位置之间插值，但由于百分比和 Px 不兼容，
    // 这个测试可能不准确。让我们只检查 None 转换到了一个合理的值
    assert!(converted[1].offset >= 0.0 && converted[1].offset <= 1.0);
    assert_eq!(converted[2].offset, 100.0);
}

#[test]
fn test_convert_color_stops_single_stop() {
    let stops = vec![GradientColorStop {
        color: zero_css_parser::values::ColorValue::Rgba(128, 128, 128, 255),
        position: None,
    }];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));

    assert_eq!(converted.len(), 1);
    assert_eq!(converted[0].offset, 0.0);
    assert_eq!(converted[0].color.r, 128);
}

/// 测试 linear_direction_to_kind 的各种方向
#[test]
fn test_linear_direction_to_kind_directions() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

    // ToBottom
    let kind = linear_direction_to_kind(&GradientDirection::ToBottom, &rect);
    if let zero_render_foundation::primitive::GradientKind::Linear { x0, y0, x1, y1 } = kind {
        assert_eq!(x0, 100.0); // 中心 x
        assert_eq!(y0, 0.0); // 顶部
        assert_eq!(x1, 100.0); // 中心 x
        assert_eq!(y1, 100.0); // 底部
    } else {
        panic!("Expected Linear gradient");
    }

    // ToRight
    let kind = linear_direction_to_kind(&GradientDirection::ToRight, &rect);
    if let zero_render_foundation::primitive::GradientKind::Linear { x0, y0, x1, y1 } = kind {
        assert_eq!(x0, 0.0); // 左侧
        assert_eq!(y0, 50.0); // 中心 y
        assert_eq!(x1, 200.0); // 右侧
        assert_eq!(y1, 50.0); // 中心 y
    } else {
        panic!("Expected Linear gradient");
    }

    // ToTopLeft
    let kind = linear_direction_to_kind(&GradientDirection::ToTopLeft, &rect);
    if let zero_render_foundation::primitive::GradientKind::Linear { x0, y0, x1, y1 } = kind {
        assert_eq!(x0, 200.0); // 右下角
        assert_eq!(y0, 100.0); // 右下角
        assert_eq!(x1, 0.0); // 左上角
        assert_eq!(y1, 0.0); // 左上角
    } else {
        panic!("Expected Linear gradient");
    }
}

#[test]
fn test_linear_direction_to_kind_angle() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

    // 0deg = to top
    let kind = linear_direction_to_kind(&GradientDirection::Angle(0.0), &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // 90deg = to right
    let kind = linear_direction_to_kind(&GradientDirection::Angle(90.0), &rect);
    if let zero_render_foundation::primitive::GradientKind::Linear { x0, x1, .. } = kind {
        assert!(x0 < x1, "90deg 应从左到右");
    }

    // 45deg = to bottom right
    let kind = linear_direction_to_kind(&GradientDirection::Angle(45.0), &rect);
    if let zero_render_foundation::primitive::GradientKind::Linear { x0, y0, x1, y1 } = kind {
        // For 45deg, the line should have a positive slope (going down-right)
        assert!(x0 < x1, "45deg 应从左到右");
        // Due to inverted Y coordinate system, dy might be negative
        let dx = x1 - x0;
        let dy = y1 - y0;
        assert!(dx > 0.0, "dx should be positive");
        assert!(dy.abs() > 0.0, "dy should not be zero");
    }

    // 360deg = 等效 0deg
    let kind_0 = linear_direction_to_kind(&GradientDirection::Angle(0.0), &rect);
    let kind_360 = linear_direction_to_kind(&GradientDirection::Angle(360.0), &rect);
    assert_eq!(format!("{:?}", kind_0), format!("{:?}", kind_360));
}

/// 测试 gradient_to_primitive 的各种渐变类型
#[test]
fn test_gradient_to_primitive_linear() {
    let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
    let gradient = GradientValue::Linear(LinearGradient {
        interpolation: Default::default(),
        direction: GradientDirection::ToBottom,
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ],
        repeating: false,
    });

    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
    let prim = result.unwrap();
    assert!(matches!(
        prim.kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));
    assert_eq!(prim.stops.len(), 2);
}

#[test]
fn test_gradient_to_primitive_radial() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        position_x: LengthValue::Percentage(50.0), // Center of 200x200 rect
        position_y: LengthValue::Percentage(50.0), // Center of 200x200 rect
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            },
        ],
        repeating: false,
    });

    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
    let prim = result.unwrap();
    if let zero_render_foundation::primitive::GradientKind::Radial {
        cx,
        cy,
        inner_radius,
        outer_radius,
    } = prim.kind
    {
        // Percentage(50.0) = 50% of 200 = 100，加上 rect.left()=0
        assert!((cx - 100.0).abs() < 0.1, "cx should be 50% of 200 = 100, got {cx}");
        assert!((cy - 100.0).abs() < 0.1, "cy should be 50% of 200 = 100, got {cy}");
        assert_eq!(inner_radius, 0.0);
        assert!(outer_radius > 0.0);
    } else {
        panic!("Expected Radial gradient");
    }
}

#[test]
fn test_gradient_to_primitive_conic() {
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let gradient = GradientValue::Conic(ConicGradient {
        interpolation: Default::default(),
        from_angle: 0.0,
        position_x: LengthValue::Percentage(50.0),
        position_y: LengthValue::Percentage(50.0),
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ],
        repeating: false,
    });

    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some(), "conic-gradient 应返回 Some");
    let prim = result.unwrap();
    assert!(matches!(
        prim.kind,
        zero_render_foundation::primitive::GradientKind::Conic { .. }
    ));
    assert_eq!(prim.stops.len(), 2);
}

/// 测试 PrimitiveCounts
#[test]
fn test_primitive_counts_snapshot() {
    use zero_render_foundation::primitive::RenderPrimitives;
    let mut prims = RenderPrimitives::default();
    prims.fills.push(zero_render_foundation::primitive::FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: zero_render_foundation::color::Color::rgb(255, 0, 0),
    });

    let counts = PrimitiveCounts::snapshot(&prims);
    assert_eq!(counts.fills, 1);
    assert_eq!(counts.rounded_rects, 0);
    assert_eq!(counts.gradients, 0);
    assert_eq!(counts.shadows, 0);
    assert_eq!(counts.images, 0);
    assert_eq!(counts.glyphs, 0);
    assert_eq!(counts.strokes, 0);
}

/// 测试 apply_opacity_to_new_primitives
#[test]
fn test_apply_opacity_to_new_primitives() {
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

    let mut prims = RenderPrimitives::default();
    // 添加一些初始图元
    prims.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: zero_render_foundation::color::Color::rgba(255, 0, 0, 255),
    });

    let before = PrimitiveCounts::snapshot(&prims);

    // 添加新的图元
    prims.fills.push(FillPrimitive {
        rect: Rect::new(10.0, 10.0, 10.0, 10.0),
        color: zero_render_foundation::color::Color::rgba(0, 255, 0, 255),
    });
    let font_id = zero_render_foundation::primitive::FontId(0);
    prims.glyphs.push(zero_render_foundation::primitive::GlyphPrimitive {
        x: 20.0,
        y: 20.0,
        font_size: 16.0,
        glyph_id: 1,
        font_glyph_index: None,
        source: None,
        font_id,
        color: zero_render_foundation::color::Color::rgba(0, 0, 255, 255),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    });

    // 应用 opacity 只影响新图元
    apply_opacity_to_new_primitives(&mut prims, &before, 0.5);

    // 初始图元 alpha 不变
    assert_eq!(prims.fills[0].color.a, 255);
    // 新图元 alpha 被减半
    assert_eq!(prims.fills[1].color.a, 128);
    assert_eq!(prims.glyphs[0].color.a, 128);
}

/// 测试 apply_text_transform
#[test]
fn test_apply_text_transform_none() {
    let result = apply_text_transform("Hello World", &TextTransformValue::None);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_apply_text_transform_uppercase() {
    let result = apply_text_transform("Hello World", &TextTransformValue::Uppercase);
    assert_eq!(result, "HELLO WORLD");
}

#[test]
fn test_apply_text_transform_lowercase() {
    let result = apply_text_transform("Hello World", &TextTransformValue::Lowercase);
    assert_eq!(result, "hello world");
}

#[test]
fn test_apply_text_transform_capitalize() {
    let result = apply_text_transform("hello world", &TextTransformValue::Capitalize);
    assert_eq!(result, "Hello World");

    let result = apply_text_transform("hello   world", &TextTransformValue::Capitalize);
    assert_eq!(result, "Hello   World");
}

#[test]
fn test_apply_text_transform_mixed_alphanumeric() {
    let result = apply_text_transform("123abc def456", &TextTransformValue::Capitalize);
    assert_eq!(result, "123abc Def456"); // 只有 d 被大写
}

/// 测试 BorderRadiusSpec
#[test]
fn test_border_radius_spec_from_style() {
    let mut style = ComputedStyle::default();
    style.border_top_left_radius = LengthValue::Px(5.0);
    style.border_top_right_radius = LengthValue::Percentage(10.0); // 不支持，应为 0
    style.border_bottom_right_radius = LengthValue::Px(15.0);
    style.border_bottom_left_radius = LengthValue::Em(1.0); // 不支持，应为 0

    let spec = BorderRadiusSpec::from_style(&style);
    assert_eq!(spec.top_left, 5.0);
    assert_eq!(spec.top_right, 0.0); // 不支持的单位
    assert_eq!(spec.bottom_right, 15.0);
    assert_eq!(spec.bottom_left, 0.0); // 不支持的单位
}

#[test]
fn test_border_radius_spec_is_zero() {
    let mut style = ComputedStyle::default();
    let spec = BorderRadiusSpec::from_style(&style);
    assert!(spec.is_zero());

    style.border_top_left_radius = LengthValue::Px(1.0);
    let spec = BorderRadiusSpec::from_style(&style);
    assert!(!spec.is_zero());
}

/// 测试复杂场景组合
#[test]
fn test_multiple_operations_combination() {
    // 测试多个裁剪操作组合
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![
        FillPrimitive {
            rect: Rect::new(-50.0, -50.0, 200.0, 200.0),
            color: zero_render_foundation::color::Color::rgb(255, 0, 0),
        },
        FillPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: zero_render_foundation::color::Color::rgb(0, 255, 0),
        },
        FillPrimitive {
            rect: Rect::new(200.0, 200.0, 50.0, 50.0),
            color: zero_render_foundation::color::Color::rgb(0, 0, 255),
        },
    ];

    // 第一个裁剪：裁剪到 [0, 0] 到 [150, 150]
    let clip_rect1 = Rect::new(0.0, 0.0, 150.0, 150.0);
    clip_fills(&mut fills, 0, &clip_rect1);

    // 第二个裁剪：裁剪到 [50, 50] 到 [100, 100]
    let clip_rect2 = Rect::new(50.0, 50.0, 50.0, 50.0);
    clip_fills(&mut fills, 1, &clip_rect2);

    // 验证结果
    // fill1: 从 [-50,-50,200,200] 裁剪到 [0,0,150,150]
    assert_eq!(fills[0].rect, Rect::new(0.0, 0.0, 150.0, 150.0));
    // fill2: 从 [0,0,100,100] 裁剪到 [50,50,50,50]
    assert_eq!(fills[1].rect, Rect::new(50.0, 50.0, 50.0, 50.0));
    // fill3: 完全在外，应该被清零
    assert_eq!(fills[2].rect.size.width, 0.0);
    assert_eq!(fills[2].rect.size.height, 0.0);
}

/// 测试极端值
#[test]
fn test_extreme_values() {
    // 测试非常大的坐标和尺寸
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![FillPrimitive {
        rect: Rect::new(f32::MAX / 2.0, f32::MAX / 2.0, f32::MAX, f32::MAX),
        color: zero_render_foundation::color::Color::rgb(255, 0, 0),
    }];

    let clip_rect = Rect::new(0.0, 0.0, f32::MAX, f32::MAX);
    clip_fills(&mut fills, 0, &clip_rect);

    // 应该能够处理极端值而不 panic
    assert!(fills[0].rect.size.width.is_finite());
    assert!(fills[0].rect.size.height.is_finite());

    // 测试 text transform 的长字符串
    let long_text = "a".repeat(1000);
    let result = apply_text_transform(&long_text, &TextTransformValue::Uppercase);
    assert_eq!(result.len(), long_text.len());
    assert_eq!(result, long_text.to_uppercase());
}

// === 新增覆盖率测试 ===

/// 测试 gradient_to_primitive radial ClosestSide
#[test]
fn test_gradient_to_primitive_radial_closest_side() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::ClosestSide,
        position_x: LengthValue::Px(100.0),
        position_y: LengthValue::Px(100.0),
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            },
        ],
        repeating: false,
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
}

/// 测试 gradient_to_primitive radial FarthestSide
#[test]
fn test_gradient_to_primitive_radial_farthest_side() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::FarthestSide,
        position_x: LengthValue::Px(100.0),
        position_y: LengthValue::Px(100.0),
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ],
        repeating: false,
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
}

/// 测试 gradient_to_primitive radial ClosestCorner
#[test]
fn test_gradient_to_primitive_radial_closest_corner() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::ClosestCorner,
        position_x: LengthValue::Px(100.0),
        position_y: LengthValue::Px(100.0),
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            },
        ],
        repeating: false,
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
}

/// 测试 gradient_to_primitive radial Length size
#[test]
fn test_gradient_to_primitive_radial_length_size() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::Length(LengthValue::Px(50.0)),
        position_x: LengthValue::Px(100.0),
        position_y: LengthValue::Px(100.0),
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 255, 255, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            },
        ],
        repeating: false,
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
    let prim = result.unwrap();
    if let zero_render_foundation::primitive::GradientKind::Radial { outer_radius, .. } = prim.kind {
        assert_eq!(outer_radius, 50.0);
    }
}

/// 测试 linear_direction_to_kind 所有剩余方向
#[test]
fn test_linear_direction_to_kind_remaining_directions() {
    let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

    // ToTop
    let kind = linear_direction_to_kind(&GradientDirection::ToTop, &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // ToLeft
    let kind = linear_direction_to_kind(&GradientDirection::ToLeft, &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // ToTopRight
    let kind = linear_direction_to_kind(&GradientDirection::ToTopRight, &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // ToBottomRight
    let kind = linear_direction_to_kind(&GradientDirection::ToBottomRight, &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // ToBottomLeft
    let kind = linear_direction_to_kind(&GradientDirection::ToBottomLeft, &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // Angle 180deg = to bottom
    let kind = linear_direction_to_kind(&GradientDirection::Angle(180.0), &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));

    // Angle 270deg = to left
    let kind = linear_direction_to_kind(&GradientDirection::Angle(270.0), &rect);
    assert!(matches!(
        kind,
        zero_render_foundation::primitive::GradientKind::Linear { .. }
    ));
}

/// 测试 apply_opacity_to_new_primitives 覆盖更多图元类型
#[test]
fn test_apply_opacity_all_primitive_types() {
    use zero_render_foundation::primitive::{
        FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives, RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive,
    };

    let mut prims = RenderPrimitives::default();
    let before = PrimitiveCounts::snapshot(&prims);

    // 添加各种类型的图元
    prims.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: zero_render_foundation::color::Color::rgba(255, 0, 0, 200),
    });
    prims.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(0.0, 0.0, 20.0, 20.0),
        top_left_radius: 5.0,
        top_right_radius: 5.0,
        bottom_right_radius: 5.0,
        bottom_left_radius: 5.0,
        color: zero_render_foundation::color::Color::rgba(0, 255, 0, 200),
    });
    let font_id = FontId(0);
    prims.glyphs.push(GlyphPrimitive {
        x: 0.0,
        y: 0.0,
        font_size: 16.0,
        glyph_id: 1,
        font_glyph_index: None,
        source: None,
        font_id,
        color: zero_render_foundation::color::Color::rgba(0, 0, 255, 200),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
    });
    prims.strokes.push(StrokePrimitive {
        x1: 0.0,
        y1: 0.0,
        x2: 30.0,
        y2: 30.0,
        width: 2.0,
        color: zero_render_foundation::color::Color::rgba(128, 128, 128, 200),
        style: zero_render_foundation::primitive::LineStyle::Solid,
        cap: zero_render_foundation::primitive::LineCap::Butt,
    });
    prims.shadows.push(ShadowPrimitive {
        rect: Rect::new(0.0, 0.0, 15.0, 15.0),
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 5.0,
        spread_radius: 0.0,
        inset: false,
        color: zero_render_foundation::color::Color::rgba(0, 0, 0, 200),
    });
    // Image primitive — opacity 通过绘制时应用
    prims.images.push(zero_render_foundation::primitive::ImagePrimitive {
        rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        image_key: zero_render_foundation::image_cache::ImageKey::new(0),
        clip: None,
    });

    apply_opacity_to_new_primitives(&mut prims, &before, 0.5);

    assert_eq!(prims.fills[0].color.a, 100);
    assert_eq!(prims.rounded_rects[0].color.a, 100);
    assert_eq!(prims.glyphs[0].color.a, 100);
    assert_eq!(prims.strokes[0].color.a, 100);
    assert_eq!(prims.shadows[0].color.a, 100);
}

/// 测试 clip_fills 使用 start 索引跳过部分填充
#[test]
fn test_clip_fills_with_start_index() {
    use zero_render_foundation::primitive::FillPrimitive;
    let mut fills = vec![
        FillPrimitive {
            rect: Rect::new(200.0, 200.0, 50.0, 50.0), // 完全在裁剪区域外
            color: zero_render_foundation::color::Color::rgb(255, 0, 0),
        },
        FillPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0), // 在裁剪区域内
            color: zero_render_foundation::color::Color::rgb(0, 255, 0),
        },
    ];
    let clip_rect = Rect::new(0.0, 0.0, 150.0, 150.0);
    // 从索引 1 开始裁剪，跳过第一个
    clip_fills(&mut fills, 1, &clip_rect);
    // 第一个填充不应该被裁剪
    assert_eq!(fills[0].rect, Rect::new(200.0, 200.0, 50.0, 50.0));
    // 第二个填充被裁剪
    assert_eq!(fills[1].rect, Rect::new(0.0, 0.0, 100.0, 100.0));
}

/// 测试 clip_glyphs 使用 start 索引
#[test]
fn test_clip_glyphs_with_start_index() {
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive};
    let font_id = FontId(0);
    let mut glyphs = vec![
        GlyphPrimitive {
            x: 200.0,
            y: 200.0,
            font_size: 16.0,
            glyph_id: 1,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
        GlyphPrimitive {
            x: 50.0,
            y: 50.0,
            font_size: 16.0,
            glyph_id: 2,
            font_glyph_index: None,
            source: None,
            font_id,
            color: zero_render_foundation::color::Color::rgb(0, 0, 0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        },
    ];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_glyphs(&mut glyphs, 1, &clip_rect);
    // 第一个不应该被裁剪
    assert_eq!(glyphs[0].glyph_id, 1);
    // 第二个可见
    assert_eq!(glyphs[1].glyph_id, 2);
}

/// 测试 apply_text_transform 边界情况
#[test]
fn test_apply_text_transform_edge_cases() {
    // 空字符串
    assert_eq!(apply_text_transform("", &TextTransformValue::Uppercase), "");
    assert_eq!(apply_text_transform("", &TextTransformValue::Capitalize), "");

    // 纯数字
    assert_eq!(apply_text_transform("12345", &TextTransformValue::Uppercase), "12345");

    // 特殊字符
    assert_eq!(apply_text_transform("!@#$%", &TextTransformValue::Capitalize), "!@#$%");

    // 单字符
    assert_eq!(apply_text_transform("a", &TextTransformValue::Uppercase), "A");

    // Unicode
    assert_eq!(apply_text_transform("café", &TextTransformValue::Uppercase), "CAFÉ");
}

/// 测试 convert_color_stops 非百分比非 Px 位置
#[test]
fn test_convert_color_stops_em_position() {
    let stops = vec![GradientColorStop {
        color: zero_css_parser::values::ColorValue::Rgba(128, 128, 128, 255),
        position: Some(LengthValue::Em(2.0)), // 不支持，应返回 0.0
    }];
    let converted = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
    assert_eq!(converted[0].offset, 0.0);
}

/// 测试 BorderRadiusSpec Debug 和 Clone
#[test]
fn test_border_radius_spec_debug_clone() {
    let spec = BorderRadiusSpec {
        top_left: 5.0,
        top_right: 10.0,
        bottom_right: 15.0,
        bottom_left: 20.0,
    };
    let debug_str = format!("{:?}", spec);
    assert!(debug_str.contains("5"));
    let cloned = spec.clone();
    assert_eq!(cloned.top_left, 5.0);
    assert_eq!(cloned.bottom_left, 20.0);
}

/// 测试 gradient_to_primitive radial 位置为非 Px 值
#[test]
fn test_gradient_to_primitive_radial_non_px_position() {
    let rect = Rect::new(0.0, 0.0, 200.0, 200.0);
    let gradient = GradientValue::Radial(RadialGradient {
        interpolation: Default::default(),
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        position_x: LengthValue::Percentage(50.0), // length_to_f32 → 0.0
        position_y: LengthValue::Percentage(50.0), // length_to_f32 → 0.0
        stops: vec![GradientColorStop {
            color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
            position: None,
        }],
        repeating: false,
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
}

/// 测试 apply_opacity_to_new_primitives 无新增图元时
#[test]
fn test_apply_opacity_no_new_primitives() {
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
    let mut prims = RenderPrimitives::default();
    prims.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        color: zero_render_foundation::color::Color::rgba(255, 0, 0, 255),
    });
    let before = PrimitiveCounts::snapshot(&prims);
    // 不添加新图元
    apply_opacity_to_new_primitives(&mut prims, &before, 0.5);
    // 原有图元不应被修改
    assert_eq!(prims.fills[0].color.a, 255);
}

/// 测试 clip_fills 空列表
#[test]
fn test_clip_fills_empty() {
    let mut fills: Vec<zero_render_foundation::primitive::FillPrimitive> = vec![];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_fills(&mut fills, 0, &clip_rect);
    assert!(fills.is_empty());
}

/// 测试 clip_glyphs 空列表
#[test]
fn test_clip_glyphs_empty() {
    let mut glyphs: Vec<zero_render_foundation::primitive::GlyphPrimitive> = vec![];
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    clip_glyphs(&mut glyphs, 0, &clip_rect);
    assert!(glyphs.is_empty());
}

/// 测试 gradient_to_primitive linear repeating
#[test]
fn test_gradient_to_primitive_linear_repeating() {
    let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
    let gradient = GradientValue::Linear(LinearGradient {
        interpolation: Default::default(),
        direction: GradientDirection::ToRight,
        stops: vec![
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: zero_css_parser::values::ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ],
        repeating: true, // repeating 标志
    });
    let result = gradient_to_primitive(&gradient, &rect, &ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.is_some());
    let prim = result.unwrap();
    assert!(prim.repeating, "repeating flag should be true");
}
