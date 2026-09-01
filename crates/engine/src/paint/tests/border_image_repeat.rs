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
