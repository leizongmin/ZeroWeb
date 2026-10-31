//! border-image-repeat 模式测试 — stretch/repeat/round/space 混合模式。

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;

use zero_style_system::ComputedStyle;

use super::super::painter::Painter;
use super::visual::make_box;

/// 测试 border-image-repeat: stretch（默认）— 边框边只有 1 个 tile 覆盖整条边。
#[test]
fn test_border_image_repeat_stretch() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Stretch,
        vertical: BorderImageRepeatComputedMode::Stretch,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    // 4 corners + 4 edges + 0 center (fill=false) = 8
    assert_eq!(
        images.len(),
        8,
        "stretch mode should produce 8 image primitives (4 corners + 4 edges)"
    );

    // 验证上边只有 1 个 tile 且覆盖整条边
    // 上边从 x=10 开始，宽度 180，y=0，高度 10
    let top_edge: Vec<_> = images
        .iter()
        .filter(|img| {
            img.rect.origin.y == 0.0
                && img.rect.size.height == 10.0
                && img.rect.origin.x > 0.0
                && img.rect.origin.x < 190.0 // 排除右上角
                && img.rect.size.width > 20.0 // 排除角落（角落宽度=边框宽度=10）
        })
        .collect();
    assert_eq!(top_edge.len(), 1, "top edge should be a single stretched tile");
    assert!(
        (top_edge[0].rect.size.width - 180.0).abs() < 1.0,
        "top edge width should be ~180 (200 - 10 - 10), got {}",
        top_edge[0].rect.size.width
    );
}

/// 测试 border-image-repeat: repeat — 边框边以自然大小重复多个 tile。
#[test]
fn test_border_image_repeat_repeat() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Repeat,
        vertical: BorderImageRepeatComputedMode::Repeat,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    // 4 corners + 边的 repeat tiles（应该比 stretch 多）
    assert!(
        images.len() > 8,
        "repeat mode should produce more images than stretch, got {}",
        images.len()
    );

    // 验证上边有多个 tile
    let top_tiles: Vec<_> = images
        .iter()
        .filter(|img| img.rect.origin.y == 0.0 && img.rect.size.height == 10.0)
        .collect();
    assert!(
        top_tiles.len() > 1,
        "top edge should have multiple repeated tiles, got {}",
        top_tiles.len()
    );
}

/// 测试 border-image-repeat: round — tile 拉伸后整数个刚好覆盖边。
#[test]
fn test_border_image_repeat_round() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Round,
        vertical: BorderImageRepeatComputedMode::Round,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;

    // 验证上边的 round tiles 总宽度精确覆盖边
    let top_tiles: Vec<_> = images
        .iter()
        .filter(|img| {
            img.rect.origin.y == 0.0
                && img.rect.size.height == 10.0
                && img.rect.origin.x >= 10.0
                && img.rect.origin.x < 190.0 // 排除右上角 (190, 0)
        })
        .collect();
    assert!(!top_tiles.is_empty(), "top edge should have round tiles");

    let total_w: f32 = top_tiles.iter().map(|t| t.rect.size.width).sum();
    assert!(
        (total_w - 180.0).abs() < 1.0,
        "round tiles total width should cover 180.0, got {}",
        total_w
    );

    // 每个 tile 宽度应相同（round 保证均匀）
    let widths: Vec<f32> = top_tiles.iter().map(|t| t.rect.size.width).collect();
    let first_w = widths[0];
    for (i, &w) in widths.iter().enumerate() {
        assert!(
            (w - first_w).abs() < 0.1,
            "round tile {} width {} should equal first tile width {}",
            i,
            w,
            first_w
        );
    }
}

/// 测试 border-image-repeat: space — tile 均匀分布。
#[test]
fn test_border_image_repeat_space() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Space,
        vertical: BorderImageRepeatComputedMode::Space,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;

    // 验证上边的 space tiles
    let top_tiles: Vec<_> = images
        .iter()
        .filter(|img| img.rect.origin.y == 0.0 && img.rect.size.height == 10.0 && img.rect.origin.x >= 10.0)
        .collect();
    assert!(!top_tiles.is_empty(), "top edge should have space tiles");

    // space tiles 应保持自然宽度（10px）
    for (i, tile) in top_tiles.iter().enumerate() {
        assert!(
            (tile.rect.size.width - 10.0).abs() < 0.1,
            "space tile {} width should be ~10.0 (natural), got {}",
            i,
            tile.rect.size.width
        );
    }
}

/// 测试 border-image-repeat 混合模式 — 水平 round + 垂直 space。
#[test]
fn test_border_image_repeat_mixed() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 100.0);
    layout.border_top = 10.0;
    layout.border_right = 10.0;
    layout.border_bottom = 10.0;
    layout.border_left = 10.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Round,
        vertical: BorderImageRepeatComputedMode::Space,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert!(
        images.len() >= 8,
        "mixed repeat mode should produce at least 8 images, got {}",
        images.len()
    );

    // 验证上边是 round（tile 宽度相同）
    let top_tiles: Vec<_> = images
        .iter()
        .filter(|img| img.rect.origin.y == 0.0 && img.rect.size.height == 10.0 && img.rect.origin.x >= 10.0)
        .collect();
    if top_tiles.len() > 1 {
        let w0 = top_tiles[0].rect.size.width;
        for t in &top_tiles[1..] {
            assert!(
                (t.rect.size.width - w0).abs() < 0.1,
                "horizontal round tiles should have equal width"
            );
        }
    }

    // 验证左边是 space（tile 高度保持自然大小 10px）
    let left_tiles: Vec<_> = images
        .iter()
        .filter(|img| img.rect.origin.x == 0.0 && img.rect.size.width == 10.0 && img.rect.origin.y >= 10.0)
        .collect();
    for (i, tile) in left_tiles.iter().enumerate() {
        assert!(
            (tile.rect.size.height - 10.0).abs() < 0.1,
            "vertical space tile {} height should be ~10.0, got {}",
            i,
            tile.rect.size.height
        );
    }
}

/// R3906：border-width:0（border-style:none）**不**禁用 border-image——显式
/// border-image-width（CSS Backgrounds 3 §6.1/§7.3）创建的绘制区照常出图。
///
/// 旧实现在「至少有一条边框 > 0」处整体早退，零边框 + `border-image-width:50px` 页面
/// 全白（driving: border-image-width-005..007，chromium 绘出延伸进 padding/margin 的
/// 绿方块）。修复后：默认 width=Number(1.0) × 0 = 零厚度 → 逐块守卫仍不绘（旧行为）；
/// 显式 Length 厚度 → 四角出图（厚度 50、区域 100×100 = 四角铺满）。
#[test]
fn test_border_image_zero_border_width_with_explicit_image_width() {
    use zero_style_system::{
        BorderImageOutsetComputedComponent, BorderImageOutsetComputedValue, BorderImageSourceComputedValue,
        BorderImageWidthComputedComponent, BorderImageWidthComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // 50×50 盒、零边框（border-style:none → used border 0）
    let mut layout = make_box(Some(nid), 0.0, 0.0, 50.0, 50.0);
    layout.border_top = 0.0;
    layout.border_right = 0.0;
    layout.border_bottom = 0.0;
    layout.border_left = 0.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    style.border_image_width = BorderImageWidthComputedValue {
        top: BorderImageWidthComputedComponent::Length(50.0),
        right: BorderImageWidthComputedComponent::Length(50.0),
        bottom: BorderImageWidthComputedComponent::Length(50.0),
        left: BorderImageWidthComputedComponent::Length(50.0),
    };
    style.border_image_outset = BorderImageOutsetComputedValue {
        top: BorderImageOutsetComputedComponent::Length(25.0),
        right: BorderImageOutsetComputedComponent::Length(25.0),
        bottom: BorderImageOutsetComputedComponent::Length(25.0),
        left: BorderImageOutsetComputedComponent::Length(25.0),
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let images = &painter.primitives().images;
    assert!(
        !images.is_empty(),
        "explicit border-image-width must draw despite zero border-width"
    );
    // 边框区域含 outset 外扩 = (0-25, 0-25, 50+50, 50+50) = (-25,-25,100,100)；
    // 厚度 50 → 四角各 50×50，恰好铺满 100×100。
    let total: f32 = images
        .iter()
        .map(|img| img.rect.size.width * img.rect.size.height)
        .sum();
    assert!(
        (total - 100.0 * 100.0).abs() < 1.0,
        "four 50x50 corners should tile the full 100x100 outset area, got {total}"
    );
    // 绘制区左上角 = border box 原点 − outset
    let min_x = images.iter().map(|img| img.rect.origin.x).fold(f32::INFINITY, f32::min);
    let min_y = images.iter().map(|img| img.rect.origin.y).fold(f32::INFINITY, f32::min);
    assert!((min_x - (-25.0)).abs() < 0.1 && (min_y - (-25.0)).abs() < 0.1);
}

/// R3906 守卫对称面：零边框 + 默认 border-image-width（Number(1.0)）→ 厚度 0，不出图
///（与旧「至少一条边框」早退行为逐字节一致，防修复引入越界绘制）。
#[test]
fn test_border_image_zero_border_width_default_still_skips() {
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 50.0, 50.0);
    layout.border_top = 0.0;
    layout.border_right = 0.0;
    layout.border_bottom = 0.0;
    layout.border_left = 0.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("border.png".to_string());
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().images.is_empty(),
        "default border-image-width (1x border-width=0) must not draw"
    );
}

/// R3909：gradient border-image-source 按 9-slice 绘制（css-backgrounds-3 §6.1）。
///
/// 此前 Gradient 源在 paint 层直接 return（等同 none），且 border-image 简写的 source
/// 槽只识别 url()/none——渐变 token 落入 slice 组解析失败整条简写被丢。修复后：
/// gradient 源按同款 border-image-width/outset/repeat 计算绘制区，每片以 clip 窗口
/// 发射渐变（GradientPrimitive.clip = crop 语义）。
#[test]
fn test_border_image_gradient_source_nine_slice() {
    use zero_css_parser::values::{
        ColorValue, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
    };
    use zero_render_foundation::geometry::Rect;
    use zero_style_system::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // 100×100 盒、边框 20px → 默认 width=Number(1.0)=边框厚度
    let mut layout = make_box(Some(nid), 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 20.0;
    layout.border_right = 20.0;
    layout.border_bottom = 20.0;
    layout.border_left = 20.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Gradient(GradientValue::Linear(LinearGradient {
        interpolation: Default::default(),
        direction: GradientDirection::ToBottom,
        stops: vec![
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: Some(LengthValue::Px(0.0)),
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: Some(LengthValue::Px(100.0)),
            },
        ],
        repeating: false,
    }));
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // URL 源路径产出 image 图元；gradient 源路径产出 gradient 图元（clip = 片 rect）。
    assert!(
        painter.primitives().images.is_empty(),
        "gradient source must not emit image primitives"
    );
    let grads = &painter.primitives().gradients;
    assert!(!grads.is_empty(), "gradient source must emit gradient primitives");
    // 每片 clip = 自身 rect（crop 语义），且全部落在绘制区（= border box）内。
    for g in grads {
        let clip = g.clip.expect("each piece must carry a clip window");
        assert!((clip.origin.x - g.rect.origin.x).abs() < 0.01 && (clip.origin.y - g.rect.origin.y).abs() < 0.01);
        assert!((clip.size.width - g.rect.size.width).abs() < 0.01);
        // 四角片 = 20×20（默认厚度 = 边框宽度）
        // 绘制区 = border box (0,0,100,100)；无片越过边界。
        assert!(clip.origin.x >= -0.01 && clip.origin.y >= -0.01);
        assert!(clip.origin.x + clip.size.width <= 100.01);
        assert!(clip.origin.y + clip.size.height <= 100.01);
    }
    // 四角存在：至少各一片 20×20 角片（clip 精确等于角 rect）。
    let corners = [
        Rect::new(0.0, 0.0, 20.0, 20.0),
        Rect::new(80.0, 0.0, 20.0, 20.0),
        Rect::new(80.0, 80.0, 20.0, 20.0),
        Rect::new(0.0, 80.0, 20.0, 20.0),
    ];
    for c in corners {
        assert!(
            grads.iter().any(|g| g.clip.is_some_and(|clip| {
                (clip.origin.x - c.origin.x).abs() < 0.01
                    && (clip.origin.y - c.origin.y).abs() < 0.01
                    && (clip.size.width - c.size.width).abs() < 0.01
                    && (clip.size.height - c.size.height).abs() < 0.01
            })),
            "missing corner piece {c:?} in {:?}",
            grads.iter().map(|g| g.clip).collect::<Vec<_>>()
        );
    }
    // fill 关键字缺省 → 无中心片。
    let center = Rect::new(20.0, 20.0, 60.0, 60.0);
    assert!(
        !grads.iter().any(|g| g.clip.is_some_and(|clip| {
            (clip.origin.x - center.origin.x).abs() < 0.01 && (clip.origin.y - center.origin.y).abs() < 0.01
        })),
        "no center piece without fill keyword"
    );
}

/// R3909 对称面：border-image-source 非 none 时常规 border-style 边框不再绘制
///（css-backgrounds-3 §6.1「applied instead of the border-style」）。此前 border
/// 从图像条带下方露出（driving: outset-003 黑框 / border-image-006 红框）。
#[test]
fn test_border_image_replaces_border_style_painting() {
    use zero_css_parser::values::{
        ColorValue, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
    };
    use zero_style_system::{BorderImageSourceComputedValue, BorderStyleValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 100.0, 100.0);
    layout.border_top = 20.0;
    layout.border_right = 20.0;
    layout.border_bottom = 20.0;
    layout.border_left = 20.0;

    let mut style = ComputedStyle::default();
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_image_source = BorderImageSourceComputedValue::Gradient(GradientValue::Linear(LinearGradient {
        interpolation: Default::default(),
        direction: GradientDirection::ToBottom,
        stops: vec![
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: Some(LengthValue::Px(0.0)),
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: Some(LengthValue::Px(100.0)),
            },
        ],
        repeating: false,
    }));
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 常规 border-style 面不画（无 fill 图元带红色 border 色）。
    let red_fills: Vec<_> = painter
        .primitives()
        .fills
        .iter()
        .filter(|f| f.color.r == 255 && f.color.g == 0 && f.color.b == 0 && f.color.a == 255)
        .collect();
    assert!(
        red_fills.is_empty(),
        "border-style faces must be suppressed when border-image is present"
    );
    // border-image 渐变片照画。
    assert!(!painter.primitives().gradients.is_empty());
}

/// R3909：border-image 简写 source 槽识别 gradient 函数（此前只认 url()/none，
/// 渐变 token 落入 slice 组 → parse_border_image_slice 失败 → 整条简写被丢，
/// driving: border-image-outset-003 / border-image-image-type-003）。
#[test]
fn test_border_image_shorthand_gradient_source_expands() {
    let decls = zero_style_system::shorthand::expand_shorthands(&[(
        "border-image".to_string(),
        "linear-gradient(green, green) 1 fill / 10px".to_string(),
        false,
        (0, 0, 0),
    )]);
    let source = decls.iter().find(|(p, _, _, _)| p == "border-image-source");
    assert!(
        source.is_some_and(|(_, v, _, _)| v.contains("linear-gradient")),
        "gradient token must land in border-image-source, got {decls:?}"
    );
    let slice = decls.iter().find(|(p, _, _, _)| p == "border-image-slice");
    assert!(
        slice.is_some_and(|(_, v, _, _)| v == "1 fill"),
        "slice must not swallow the gradient"
    );
    let width = decls.iter().find(|(p, _, _, _)| p == "border-image-width");
    assert!(width.is_some_and(|(_, v, _, _)| v == "10px"));
}

// R5024：filter() 图像函数 border-image-source——每片图像图元后随滤镜图元
//（filter-effects-1 #FilterCSSImageValue：filter() 产出 <image> 后整体 9-slice，
// 片源子矩形携带给 CPU raster DropShadow 形状采样）。
#[test]
fn test_border_image_filter_function_emits_piece_filters() {
    use zero_render_foundation::color::Color;
    use zero_render_foundation::primitive::FilterKind;
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSourceComputedValue,
        FilterComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 50.0, 100.0);
    layout.border_top = 25.0;
    layout.border_right = 25.0;
    layout.border_bottom = 25.0;
    layout.border_left = 25.0;

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Filtered {
        image: Box::new(BorderImageSourceComputedValue::Url("green.png".to_string())),
        filters: vec![FilterComputedValue::DropShadow(
            50.0,
            0.0,
            0.0,
            zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255),
        )],
    };
    style.border_image_slice = zero_style_system::BorderImageSliceComputedValue {
        top: zero_style_system::BorderImageSliceComputedComponent::Number(75.0),
        right: zero_style_system::BorderImageSliceComputedComponent::Number(75.0),
        bottom: zero_style_system::BorderImageSliceComputedComponent::Number(75.0),
        left: zero_style_system::BorderImageSliceComputedComponent::Number(75.0),
        fill: false,
    };
    style.border_image_width = zero_style_system::BorderImageWidthComputedValue {
        top: zero_style_system::BorderImageWidthComputedComponent::Length(75.0),
        right: zero_style_system::BorderImageWidthComputedComponent::Length(75.0),
        bottom: zero_style_system::BorderImageWidthComputedComponent::Length(75.0),
        left: zero_style_system::BorderImageWidthComputedComponent::Length(75.0),
    };
    style.border_image_outset = zero_style_system::BorderImageOutsetComputedValue {
        top: zero_style_system::BorderImageOutsetComputedComponent::Length(50.0),
        right: zero_style_system::BorderImageOutsetComputedComponent::Length(50.0),
        bottom: zero_style_system::BorderImageOutsetComputedComponent::Length(50.0),
        left: zero_style_system::BorderImageOutsetComputedComponent::Length(50.0),
    };
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Stretch,
        vertical: BorderImageRepeatComputedMode::Stretch,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    // 解码尺寸在案（真实管线 image_sizes + image_natural_sizes 双表在案——R5355 起
    // 仅 natural 表 = intrinsic 缺失 SVG，Number 切片改按绘制区栅格解析）→ 固有
    // 尺寸口径，9-slice 源子矩形生效，Number 切片 = 源图像素。
    painter.image_sizes.insert(
        super::super::helpers::image_resource_key("green.png", None),
        (200.0, 200.0),
    );
    painter.image_natural_sizes.insert(
        super::super::helpers::image_resource_key("green.png", None),
        (200.0, 200.0),
    );
    painter.paint(&layout, &styles, None);

    // 4 角 + 左右边条 = 6 片（无 fill，上下边条宽 0）。
    let images = &painter.primitives().images;
    assert_eq!(images.len(), 6, "expected 6 border-image pieces, got {images:?}");
    // 绘制区 = border box(0,0,50,100) 外扩 outset 50 → (-50,-50,150,200)；width 75px
    // 四边 → 角 75×75 + 左右边条 75×50（上下边条宽 0）。
    let piece_of = |x: f32, y: f32| {
        images
            .iter()
            .find(|img| (img.rect.origin.x - x).abs() < 0.5 && (img.rect.origin.y - y).abs() < 0.5)
            .unwrap_or_else(|| panic!("piece at ({x},{y}) not found, got {images:?}"))
    };
    let tl = piece_of(-50.0, -50.0);
    let tr = piece_of(25.0, -50.0);
    // TL 片源子矩形 = [0,0,0.375,0.375]（slice 75 / 200px）。
    assert!(
        (tl.source.as_ref().unwrap().size.width - 0.375).abs() < 1e-4,
        "TL source subrect must be the slice 75/200 region, got {:?}",
        tl.source
    );
    // TR 片源起点 x = (200-75)/200 = 0.625。
    assert!(
        (tr.source.as_ref().unwrap().origin.x - 0.625).abs() < 1e-4,
        "TR source subrect origin must be 0.625, got {:?}",
        tr.source
    );

    // 每片紧随一个滤镜图元，rect = 片 rect，源子矩形 = 片源子矩形。
    let filters = &painter.primitives().filters;
    assert_eq!(filters.len(), 6, "each piece must be followed by a filter primitive");
    for f in filters {
        assert!(
            f.raster_drop_shadow_source.is_some(),
            "filter primitive must carry raster drop-shadow source"
        );
        let (_, rect, src) = f.raster_drop_shadow_source.as_ref().unwrap();
        assert!(
            src.size.width < 1.0 && src.size.height < 1.0,
            "piece source subrect must not be the whole image, got {src:?}"
        );
        assert!(
            images.iter().any(|img| img.rect == *rect
                && img
                    .source
                    .as_ref()
                    .is_some_and(|s| (s.origin.x - src.origin.x).abs() < 1e-4)),
            "filter rect/src must mirror its piece, got rect {rect:?} src {src:?}"
        );
        assert!(
            f.filters.contains(&FilterKind::DropShadow(
                50.0,
                0.0,
                0.0,
                Color {
                    r: 0,
                    g: 128,
                    b: 0,
                    a: 255
                }
            )),
            "drop-shadow kind must be preserved, got {:?}",
            f.filters
        );
    }
}

/// R5084 组共用 env 锁——kill-switch 用例改写进程全局 `ZW_BORDER_IMAGE_DRAW_PROCESS`，
/// 同组用例须串行防互踩（R5083 `R5083_ENV_LOCK` 同款形态；workspace 并行相位实测
/// 无锁时 kill-switch 的 env=0 窗口会让 §5.2 用例读到旧路径翻红）。
static R5084_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// R5084：css-backgrounds-3 §5.2 — slice 值之和 ≥ 源图维时，对应边片与 middle 源
/// 为空（零面积源矩形 → 不发射），角部切片保持原值整幅参与。驱动案
/// border-image-006（默认 slice 100% + 1×1 源）与 border-image-slice-006。
#[test]
fn test_border_image_slice_overflow_edges_empty() {
    let _guard = R5084_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSliceComputedComponent,
        BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 50.0, 50.0);
    for b in [
        &mut layout.border_top,
        &mut layout.border_right,
        &mut layout.border_bottom,
        &mut layout.border_left,
    ] {
        *b = 25.0;
    }

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("one.png".to_string());
    style.border_image_slice = BorderImageSliceComputedValue {
        top: BorderImageSliceComputedComponent::Percent(100.0),
        right: BorderImageSliceComputedComponent::Percent(100.0),
        bottom: BorderImageSliceComputedComponent::Percent(100.0),
        left: BorderImageSliceComputedComponent::Percent(100.0),
        fill: false,
    };
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Stretch,
        vertical: BorderImageRepeatComputedMode::Stretch,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter
        .image_natural_sizes
        .insert(super::super::helpers::image_resource_key("one.png", None), (1.0, 1.0));
    painter.paint(&layout, &styles, None);

    // 仅 4 角发射（整幅源图），边片与 middle 源为空不发射。
    let images = painter.primitives().images.clone();
    assert_eq!(images.len(), 4, "only 4 corners should be painted, got {images:?}");
    for img in &images {
        let src = img.source.expect("corner must carry slice source subrect");
        assert!(
            (src.size.width - 1.0).abs() < 1e-4 && (src.size.height - 1.0).abs() < 1e-4,
            "corner source must be the whole image, got {src:?}"
        );
        assert!(
            (img.rect.size.width - 25.0).abs() < 1e-3 && (img.rect.size.height - 25.0).abs() < 1e-3,
            "corner dest must be the 25x25 region, got {:?}",
            img.rect
        );
    }
}

/// R5084：css-backgrounds-3 §5.6 middle 片绘制过程 — 缩放因子（top/left 片，零切片
/// 回退 bottom/right → 仍无则不缩放）+ repeat 轴自然 tile 居中平铺 + 裁剪窗口同步
/// src 子矩形。驱动案 border-image-slice-fill-002（slice 0 fill + repeat：middle =
/// 整图自然尺寸，居中后窗口恰为源图中央 green 区）。
#[test]
fn test_border_image_middle_natural_tile_repeat() {
    let _guard = R5084_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSliceComputedComponent,
        BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // border box 200×200（内容 100×100 + 50px 边）。
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 200.0);
    for b in [
        &mut layout.border_top,
        &mut layout.border_right,
        &mut layout.border_bottom,
        &mut layout.border_left,
    ] {
        *b = 50.0;
    }

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("big.png".to_string());
    style.border_image_slice = BorderImageSliceComputedValue {
        top: BorderImageSliceComputedComponent::Number(0.0),
        right: BorderImageSliceComputedComponent::Number(0.0),
        bottom: BorderImageSliceComputedComponent::Number(0.0),
        left: BorderImageSliceComputedComponent::Number(0.0),
        fill: true,
    };
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Repeat,
        vertical: BorderImageRepeatComputedMode::Repeat,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    // 位图源 = 固有尺寸口径（R5355：双表在案同真实管线，Number 切片 = 源图像素）。
    painter.image_sizes.insert(
        super::super::helpers::image_resource_key("big.png", None),
        (500.0, 500.0),
    );
    painter.image_natural_sizes.insert(
        super::super::helpers::image_resource_key("big.png", None),
        (500.0, 500.0),
    );
    painter.paint(&layout, &styles, None);

    // 八片源为零面积不发射；middle 单 tile 500×500 居中于 (50..150)² → 可见窗口
    // (50..150)² ↔ tile 局部 (200..300)² = src (0.4..0.6)²。
    let images = painter.primitives().images.clone();
    assert_eq!(
        images.len(),
        1,
        "only the middle piece should be painted, got {images:?}"
    );
    let img = &images[0];
    assert!(
        (img.rect.origin.x - 50.0).abs() < 1e-3
            && (img.rect.origin.y - 50.0).abs() < 1e-3
            && (img.rect.size.width - 100.0).abs() < 1e-3
            && (img.rect.size.height - 100.0).abs() < 1e-3,
        "middle visible rect must be the region, got {:?}",
        img.rect
    );
    let src = img.source.expect("middle must carry clipped source subrect");
    assert!(
        (src.origin.x - 0.4).abs() < 1e-4
            && (src.origin.y - 0.4).abs() < 1e-4
            && (src.size.width - 0.2).abs() < 1e-4
            && (src.size.height - 0.2).abs() < 1e-4,
        "middle source window must be the centered 0.4..0.6 region, got {src:?}"
    );
}

/// R5084：middle 缩放因子有限路径 — slice 50 fill（500px 源、50px 边）→ middle
/// 因子 = 50/50 = 1 → tile = 400 自然尺寸，repeat 居中后窗口 = 源图中央 green 区。
/// 驱动案 border-image-slice-fill-001。
#[test]
fn test_border_image_middle_factor_from_top_slice() {
    let _guard = R5084_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSliceComputedComponent,
        BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // border box 200×200（内容 100×100 + 50px 边）。
    let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 200.0);
    for b in [
        &mut layout.border_top,
        &mut layout.border_right,
        &mut layout.border_bottom,
        &mut layout.border_left,
    ] {
        *b = 50.0;
    }

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("mid.png".to_string());
    style.border_image_slice = BorderImageSliceComputedValue {
        top: BorderImageSliceComputedComponent::Number(50.0),
        right: BorderImageSliceComputedComponent::Number(50.0),
        bottom: BorderImageSliceComputedComponent::Number(50.0),
        left: BorderImageSliceComputedComponent::Number(50.0),
        fill: true,
    };
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Repeat,
        vertical: BorderImageRepeatComputedMode::Repeat,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    // 位图源 = 固有尺寸口径（R5355：双表在案同真实管线，Number 切片 = 源图像素）。
    painter.image_sizes.insert(
        super::super::helpers::image_resource_key("mid.png", None),
        (500.0, 500.0),
    );
    painter.image_natural_sizes.insert(
        super::super::helpers::image_resource_key("mid.png", None),
        (500.0, 500.0),
    );
    painter.paint(&layout, &styles, None);

    // middle 源 = 400×400（slice 50），因子 1 → tile 400；repeat 居中于 100 区域 →
    // 起点 -150 → 可见窗口 ↔ tile 局部 150..250 = src (50+150)/500..(50+250)/500。
    let images = painter.primitives().images.clone();
    let mid: Vec<_> = images
        .iter()
        .filter(|img| {
            (img.rect.origin.x - 50.0).abs() < 1e-3
                && (img.rect.origin.y - 50.0).abs() < 1e-3
                && (img.rect.size.width - 100.0).abs() < 1e-3
        })
        .collect();
    assert_eq!(mid.len(), 1, "single visible middle tile expected, got {images:?}");
    let src = mid[0].source.expect("middle tile must carry clipped source");
    assert!(
        (src.origin.x - 0.4).abs() < 1e-4
            && (src.origin.y - 0.4).abs() < 1e-4
            && (src.size.width - 0.2).abs() < 1e-4
            && (src.size.height - 0.2).abs() < 1e-4,
        "middle source window must be the green center 0.4..0.6, got {src:?}"
    );
}

/// R5084 kill-switch：`ZW_BORDER_IMAGE_DRAW_PROCESS=0` 恢复旧近似（重叠切片等比
/// 缩减使边片非空 + 零面积源片整图映射回退 + middle 恒 stretch）。env var 用例，
/// 抢锁防并行互踩（R5083 同款形态）。
#[test]
fn test_r5084_border_image_draw_process_kill_switch() {
    let _guard = R5084_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_BORDER_IMAGE_DRAW_PROCESS", "0") };

    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSliceComputedComponent,
        BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    };

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 0.0, 0.0, 50.0, 50.0);
    for b in [
        &mut layout.border_top,
        &mut layout.border_right,
        &mut layout.border_bottom,
        &mut layout.border_left,
    ] {
        *b = 10.0;
    }

    let mut style = ComputedStyle::default();
    style.border_image_source = BorderImageSourceComputedValue::Url("one.png".to_string());
    style.border_image_slice = BorderImageSliceComputedValue {
        top: BorderImageSliceComputedComponent::Percent(100.0),
        right: BorderImageSliceComputedComponent::Percent(100.0),
        bottom: BorderImageSliceComputedComponent::Percent(100.0),
        left: BorderImageSliceComputedComponent::Percent(100.0),
        fill: false,
    };
    style.border_image_repeat = BorderImageRepeatComputedValue {
        horizontal: BorderImageRepeatComputedMode::Stretch,
        vertical: BorderImageRepeatComputedMode::Stretch,
    };
    let mut styles = HashMap::new();
    styles.insert(nid, style);
    let mut painter = Painter::new();
    painter
        .image_natural_sizes
        .insert(super::super::helpers::image_resource_key("one.png", None), (1.0, 1.0));
    painter.paint(&layout, &styles, None);

    // 旧路径：切片等比缩减（各 0.5px）→ 边片源矩形零宽仍发射（整图映射回退）→
    // 4 角 + 4 边 = 8 片（边区 30×10 非零）。
    let images = painter.primitives().images.clone();
    unsafe { std::env::remove_var("ZW_BORDER_IMAGE_DRAW_PROCESS") };
    assert_eq!(
        images.len(),
        8,
        "kill-switch must restore the legacy 8-piece emission, got {images:?}"
    );
}

/// R5355：css-backgrounds-3 §5.2 — intrinsic 缺失源（viewBox-only / %-dim SVG，仅
/// `image_natural_sizes` 在案、`image_sizes` 无条目——R717 ratio-only 分类）的
/// Number 切片按 **border-image 绘制区栅格 px** 解析（chromium oracle 探针定谳：
/// 200×200 绘制区上 40/30/20/10 切在 20/15/10/5 viewBox 单位处；驱动案
/// border-image-image-type-001 ✗1.43% → ✓0.00%）。固有尺寸源维持源图 px 口径
///（对照腿：同几何下 image_sizes 在案 → 数字 = 源图 px，切分分数 ×2）。
#[test]
fn test_r5355_border_image_intrinsic_less_source_number_slice_raster_space() {
    use zero_style_system::{
        BorderImageRepeatComputedMode, BorderImageRepeatComputedValue, BorderImageSliceComputedComponent,
        BorderImageSliceComputedValue, BorderImageSourceComputedValue,
    };

    // 驱动案几何：CSS width/height 100 + 四边 50px border → border box（= 绘制区）
    // 200×200（make_box 宽高为 border-box 口径，无 outset）。
    let run = |intrinsic: Option<(f32, f32)>| {
        let mut doc = zero_dom::Document::new();
        let nid = doc.create_element("div");
        let mut layout = make_box(Some(nid), 0.0, 0.0, 200.0, 200.0);
        for b in [
            &mut layout.border_top,
            &mut layout.border_right,
            &mut layout.border_bottom,
            &mut layout.border_left,
        ] {
            *b = 50.0;
        }

        let mut style = ComputedStyle::default();
        style.border_image_source = BorderImageSourceComputedValue::Url("nine.svg".to_string());
        style.border_image_slice = BorderImageSliceComputedValue {
            top: BorderImageSliceComputedComponent::Number(40.0),
            right: BorderImageSliceComputedComponent::Number(30.0),
            bottom: BorderImageSliceComputedComponent::Number(20.0),
            left: BorderImageSliceComputedComponent::Number(10.0),
            fill: false,
        };
        style.border_image_repeat = BorderImageRepeatComputedValue {
            horizontal: BorderImageRepeatComputedMode::Stretch,
            vertical: BorderImageRepeatComputedMode::Stretch,
        };
        let mut styles = HashMap::new();
        styles.insert(nid, style);
        let mut painter = Painter::new();
        painter.image_natural_sizes.insert(
            super::super::helpers::image_resource_key("nine.svg", None),
            (100.0, 100.0),
        );
        if let Some((w, h)) = intrinsic {
            painter
                .image_sizes
                .insert(super::super::helpers::image_resource_key("nine.svg", None), (w, h));
        }
        painter.paint(&layout, &styles, None);

        let images = &painter.primitives().images;
        assert_eq!(images.len(), 8, "4 corners + 4 edges, got {images:?}");
        let src_at = |x: f32, y: f32| {
            images
                .iter()
                .find(|img| (img.rect.origin.x - x).abs() < 0.5 && (img.rect.origin.y - y).abs() < 0.5)
                .unwrap_or_else(|| panic!("piece at ({x},{y}) not found, got {images:?}"))
                .source
                .expect("piece must carry a normalized source subrect")
        };
        (src_at(0.0, 0.0), src_at(50.0, 0.0))
    };

    // intrinsic 缺失腿（viewBox-only SVG）：源坐标系 = 绘制区 200×200 →
    // 左上角 src 尺寸 (sl/w, st/h) = (10/200, 40/200)；上边（dest 起点 x = bl = 50）
    // src 起点 x = 0.05、宽 (200-10-30)/200 = 0.8。
    let (tl, top) = run(None);
    assert!(
        (tl.size.width - 0.05).abs() < 1e-4 && (tl.size.height - 0.2).abs() < 1e-4,
        "intrinsic-less slice must resolve against the 200×200 drawing area, got tl {tl:?}"
    );
    assert!(
        (top.origin.x - 0.05).abs() < 1e-4 && (top.size.width - 0.8).abs() < 1e-4,
        "top edge src must span (200-10-30)/200 of the raster, got {top:?}"
    );

    // 对照腿（固有尺寸源）：数字回源图 px 口径 → 切分分数 = 10/100、40/100。
    let (tl, _) = run(Some((100.0, 100.0)));
    assert!(
        (tl.size.width - 0.1).abs() < 1e-4 && (tl.size.height - 0.4).abs() < 1e-4,
        "intrinsic source keeps source-px slice caliber, got tl {tl:?}"
    );
}
// ── R5085：变换叶实心盒直绘（transform-origin-01 驱动）─────────────────

/// R5085 组共用 env 锁——kill-switch 用例改写进程全局 `ZW_TRANSFORM_SOLID_FILL_DIRECT`，
/// 同组用例须串行防互踩（R5083/R5084 同款形态；并行相位无锁时直绘用例会读到
/// env=0 走旧路径翻红）。
static R5085_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// R5085：叶实心盒 + transform（rotate）→ 背景以四角映射 path_fill 直绘，
/// 不发射 TransformPrimitive——CPU raster apply_transform_post 的「union rect
/// 清白 + 反向采样」会连同 union 内先绘兄弟内容一并重排（transform-origin-01：
/// 先绘 #blue 在 #red 变换 union 内被 shuffle 成 50×50 碎块）。
#[test]
fn test_r5085_transform_solid_leaf_fill_direct_path() {
    let _guard = R5085_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    use zero_css_parser::values::{TransformFunction, TransformValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    // 盒 (100,50) 100×100，origin left center = (100,100)，rotate(90deg)
    // → 变换后四边形 (100..200,50..150) → 旋转 bbox (50..200,...) 具体四角断言如下。
    let mut layout = make_box(Some(nid), 100.0, 50.0, 100.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
    style.transform_origin_x = zero_css_parser::values::LengthValue::Px(0.0);
    style.transform_origin_y = zero_css_parser::values::LengthValue::Px(50.0);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    // doc 入参（含该 div、无文本子节点）——doc=None 时 paint_text 会为空 run 发射
    // notdef 占位字形（glyph_id 0，测试语境伪影），干扰「仅实心 fill」门。
    painter.paint(&layout, &styles, Some(&doc));

    let prims = painter.primitives();
    // fill 已被替换为 path_fill：fills 空增、恰 1 个新 path_fill、零 TransformPrimitive。
    assert_eq!(prims.fills.len(), 0, "solid leaf fill must be re-emitted as path_fill");
    assert_eq!(prims.transforms.len(), 0, "no TransformPrimitive for direct path");
    assert_eq!(prims.path_fills.len(), 1, "exactly one transformed quad fill");
    let pf = &prims.path_fills[0];
    // 四角 = (100,50),(200,50),(200,150),(100,150) 绕 (100,100) 旋转 90°（屏幕 y 向下
    // 屏坐标系 b=sin>0 顺时针）：→ (100+0,100-50+?)… 精确断言：顶点集应等于
    // (150,100),(150,200),(50,200),(50,100)。
    let verts = &pf.vertices;
    assert_eq!(verts.len(), 8, "quad = 4 points");
    let mut got: Vec<(u32, u32)> = verts
        .chunks(2)
        .map(|c| [(c[0] * 10.0).round() as u32, (c[1] * 10.0).round() as u32])
        .map(|a| (a[0], a[1]))
        .collect();
    got.sort();
    let mut want: Vec<(u32, u32)> = vec![(1500, 1000), (1500, 2000), (500, 2000), (500, 1000)];
    want.sort();
    assert_eq!(got, want, "rotated quad corners, got {got:?}");
}

/// R5085 kill-switch：`ZW_TRANSFORM_SOLID_FILL_DIRECT=0` 恢复 TransformPrimitive
/// 后处理路径（fill + transforms 各 1）。env 用例抢锁防并行互踩（R5084 同款）。
#[test]
fn test_r5085_transform_solid_fill_direct_kill_switch() {
    let _guard = R5085_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    unsafe { std::env::set_var("ZW_TRANSFORM_SOLID_FILL_DIRECT", "0") };

    use zero_css_parser::values::{TransformFunction, TransformValue};

    let mut doc = zero_dom::Document::new();
    let nid = doc.create_element("div");
    let mut layout = make_box(Some(nid), 100.0, 50.0, 100.0, 100.0);

    let mut style = ComputedStyle::default();
    style.background_color = zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255);
    style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
    let mut styles = HashMap::new();
    styles.insert(nid, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));
    let prims = painter.primitives();
    unsafe { std::env::remove_var("ZW_TRANSFORM_SOLID_FILL_DIRECT") };

    assert_eq!(prims.fills.len(), 1, "legacy path keeps the plain fill");
    assert_eq!(prims.transforms.len(), 1, "legacy path emits TransformPrimitive");
    assert_eq!(prims.path_fills.len(), 0, "legacy path emits no path_fill");
}
