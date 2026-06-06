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
