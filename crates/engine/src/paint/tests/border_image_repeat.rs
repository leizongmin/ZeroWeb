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
