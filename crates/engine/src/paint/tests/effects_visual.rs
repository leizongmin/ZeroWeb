//! 视觉效果渲染测试 — box-shadow、background-image、text-shadow、CSS 属性指示器。
//!
//! 从 effects.rs 拆分而来，包含视觉渲染效果的单元测试。

#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_render_foundation::color::Color;
use zero_style_system::{BackgroundImageComputedValue, BoxShadowComputedValue, ComputedStyle, TextShadowComputedValue};

use super::advanced::make_box;
use crate::paint::Painter;

// ── 新增测试：box-shadow 渲染 ──────────────────────────────

/// 测试 box-shadow 生成 ShadowPrimitive。
#[test]
fn test_paint_box_shadow_basic() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.shadows.len(), 1, "应生成 1 个阴影图元");
    let shadow = &prims.shadows[0];
    assert_eq!(shadow.offset_x, 4.0);
    assert_eq!(shadow.offset_y, 4.0);
    assert_eq!(shadow.blur_radius, 8.0);
    assert_eq!(shadow.spread_radius, 0.0);
    assert_eq!(shadow.color, Color::rgba(0, 0, 0, 128));
}

/// 测试 box-shadow 所有参数为零时不生成阴影。
#[test]
fn test_paint_box_shadow_zero_values_skip() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().shadows.is_empty(),
        "所有参数为零时不应生成阴影图元"
    );
}

/// 测试 box-shadow 仅 offset 非零时生成阴影。
#[test]
fn test_paint_box_shadow_offset_only() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 5.0,
        offset_y: 3.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().shadows.len(),
        1,
        "仅 offset 非零时应生成 1 个阴影图元"
    );
}

/// 测试 box-shadow 仅 blur 非零时生成阴影。
#[test]
fn test_paint_box_shadow_blur_only() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 10.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().shadows.len(),
        1,
        "仅 blur 非零时应生成 1 个阴影图元"
    );
}

/// 测试 box-shadow 仅 spread 非零时生成阴影。
#[test]
fn test_paint_box_shadow_spread_only() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread_radius: 5.0,
        color: ColorValue::Rgba(0, 0, 0, 255),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().shadows.len(),
        1,
        "仅 spread 非零时应生成 1 个阴影图元"
    );
}

/// 测试 box-shadow 颜色正确传递。
#[test]
fn test_paint_box_shadow_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(255, 0, 0, 255),
        inset: false,
    }];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let shadow = &painter.primitives().shadows[0];
    assert_eq!(shadow.color, Color::rgb(255, 0, 0), "阴影颜色应为红色");
}

/// 测试 box-shadow 与背景色同时生成。
#[test]
fn test_paint_box_shadow_with_background() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.box_shadow = vec![BoxShadowComputedValue {
        offset_x: 4.0,
        offset_y: 4.0,
        blur_radius: 8.0,
        spread_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
        inset: false,
    }];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "应生成 1 个背景填充");
    assert_eq!(prims.shadows.len(), 1, "应生成 1 个阴影图元");
}

/// R2304：多 box-shadow 列表按声明顺序生成多个 ShadowPrimitive（CSS Backgrounds §7.2）。
#[test]
fn test_paint_box_shadow_multiple_list() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![
        BoxShadowComputedValue {
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 128),
            inset: false,
        },
        BoxShadowComputedValue {
            offset_x: 2.0,
            offset_y: 2.0,
            blur_radius: 0.0,
            spread_radius: 1.0,
            color: ColorValue::Rgba(255, 0, 0, 255),
            inset: false,
        },
    ];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.shadows.len(), 2, "应生成 2 个阴影图元（多阴影）");
    // 按列表顺序绘制：首个阴影先入列
    assert_eq!(prims.shadows[0].offset_x, 4.0);
    assert_eq!(prims.shadows[1].offset_x, 2.0);
}

/// R2304：box-shadow 列表中含全零项时，该零项被跳过、其余正常生成。
#[test]
fn test_paint_box_shadow_list_skips_zero_entry() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.box_shadow = vec![
        BoxShadowComputedValue {
            // 全零：应被跳过
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 255),
            inset: false,
        },
        BoxShadowComputedValue {
            offset_x: 5.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 255),
            inset: false,
        },
    ];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(
        painter.primitives().shadows.len(),
        1,
        "全零阴影项被跳过，仅生成 1 个图元"
    );
    assert_eq!(painter.primitives().shadows[0].offset_x, 5.0);
}

// ── 新增测试：background-image 渲染 ──────────────────────────

/// 测试 background-image: url() 生成 ImagePrimitive。
#[test]
fn test_paint_background_image_url() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("test.png".to_string())];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.images.len(), 1, "应生成 1 个图片图元");
    assert_eq!(prims.images[0].rect.size.width, 100.0);
    assert_eq!(prims.images[0].rect.size.height, 50.0);
}

/// 测试 background-image: none 不生成 ImagePrimitive。
#[test]
fn test_paint_background_image_none() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::None];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().images.is_empty(),
        "background-image:none 不应生成图片图元"
    );
}

/// 测试 background-image 与背景色同时生成。
#[test]
fn test_paint_background_image_with_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.background_image = vec![BackgroundImageComputedValue::Url("bg.png".to_string())];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.fills.len(), 1, "应生成 1 个背景填充");
    assert_eq!(prims.images.len(), 1, "应生成 1 个图片图元");
}

/// 测试 background-image URL 哈希一致性。
#[test]
fn test_paint_background_image_url_hash_consistency() {
    let mut doc1 = zero_dom::Document::new();
    let elem1 = doc1.create_element("div");
    let layout1 = make_box(Some(elem1), 0.0, 0.0, 100.0, 50.0);

    let mut styles1 = HashMap::new();
    let mut style1 = ComputedStyle::default();
    style1.background_image = vec![BackgroundImageComputedValue::Url("same.png".to_string())];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style1.color = ColorValue::CurrentColor;
    styles1.insert(elem1, style1);

    let mut painter1 = Painter::new();
    painter1.paint(&layout1, &styles1, None);
    let key1 = painter1.primitives().images[0].image_key.clone();

    let mut doc2 = zero_dom::Document::new();
    let elem2 = doc2.create_element("div");
    let layout2 = make_box(Some(elem2), 10.0, 20.0, 80.0, 40.0);

    let mut styles2 = HashMap::new();
    let mut style2 = ComputedStyle::default();
    style2.background_image = vec![BackgroundImageComputedValue::Url("same.png".to_string())];
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style2.color = ColorValue::CurrentColor;
    styles2.insert(elem2, style2);

    let mut painter2 = Painter::new();
    painter2.paint(&layout2, &styles2, None);
    let key2 = painter2.primitives().images[0].image_key.clone();

    assert_eq!(key1, key2, "相同 URL 应产生相同的 ImageKey");
}

/// R1794：background-image 相对 url() + document_url → ImageKey 必须等于
/// `image_resource_key(url, document_url)`（与 webview 抓取路径 `image_resource_key(&abs, None)`
/// 一致，使 painter 查找与 image_cache 像素存储对齐）。改前 painter 用 `simple_hash(url)`
/// 哈希原始相对字符串，永不命中抓取 key。
#[test]
fn test_paint_background_image_url_resolves_against_document_url() {
    use crate::paint::helpers::image_resource_key;
    use zero_render_foundation::image_cache::ImageKey;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_image = vec![BackgroundImageComputedValue::Url("bg.png".to_string())];
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.set_document_url(Some("https://example.com/page"));
    painter.paint(&layout, &styles, None);

    let expected = image_resource_key("bg.png", Some("https://example.com/page"));
    assert_eq!(
        painter.primitives().images[0].image_key,
        ImageKey::new(expected),
        "相对 url() 应按 document_url 解析为绝对后哈希，与抓取 key 一致"
    );
}

// ── 新增测试：text-shadow 渲染 ──────────────────────────────

/// 测试 text-shadow 生成阴影 glyph。
#[test]
fn test_paint_text_shadow_basic() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 阴影 glyph + 主 glyph = 2
    assert_eq!(prims.glyphs.len(), 2, "应生成 2 个 glyph（阴影 + 主）");

    // 阴影 glyph 在前
    let shadow_glyph = &prims.glyphs[0];
    assert_eq!(
        shadow_glyph.color,
        Color::rgba(0, 0, 0, 128),
        "阴影 glyph 颜色应为半透明黑色"
    );

    // 主 glyph 在后
    let main_glyph = &prims.glyphs[1];
    assert_eq!(main_glyph.color, Color::rgb(0, 0, 0), "主 glyph 颜色应为黑色");

    // 阴影 glyph 位置偏移 (2, 2)
    assert_eq!(shadow_glyph.x, main_glyph.x + 2.0, "阴影 glyph x 偏移 2");
    assert_eq!(shadow_glyph.y, main_glyph.y + 2.0, "阴影 glyph y 偏移 2");
}

/// 测试 text-shadow 所有参数为零时不生成额外 glyph。
#[test]
fn test_paint_text_shadow_zero_skip() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // text-shadow 全为零 → 只有 1 个主 glyph，没有阴影 glyph
    assert_eq!(
        painter.primitives().glyphs.len(),
        1,
        "text-shadow 全为零时只应生成 1 个主 glyph"
    );
}

/// 测试 text-shadow 仅 offset_y 非零时生成阴影 glyph。
#[test]
fn test_paint_text_shadow_offset_y_only() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 0.0,
        offset_y: 3.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.glyphs.len(), 2, "应生成 2 个 glyph（阴影 + 主）");

    // 阴影 glyph y 偏移 3
    let shadow_glyph = &prims.glyphs[0];
    let main_glyph = &prims.glyphs[1];
    assert_eq!(shadow_glyph.x, main_glyph.x, "阴影 glyph x 不偏移");
    assert_eq!(shadow_glyph.y, main_glyph.y + 3.0, "阴影 glyph y 偏移 3");
}

/// 测试 text-shadow 颜色正确传递。
#[test]
fn test_paint_text_shadow_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(255, 0, 0, 255),
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let shadow_glyph = &painter.primitives().glyphs[0];
    assert_eq!(shadow_glyph.color, Color::rgb(255, 0, 0), "阴影 glyph 颜色应为红色");
}

/// 测试 text-shadow 与 transform 结合。
#[test]
fn test_paint_text_shadow_with_transform() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    };
    style.transform = TransformValue::List(vec![TransformFunction::Translate(10.0, 20.0)]);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    assert_eq!(prims.glyphs.len(), 2, "应生成 2 个 glyph（阴影 + 主）");

    // 主 glyph 位置应包含 transform 偏移
    let main_glyph = &prims.glyphs[1];
    // text_x = abs_x(0) + tx(10) = 10
    assert_eq!(main_glyph.x, 10.0, "主 glyph x 应包含 translate(10)");
    // text_y = abs_y(0) + ty(20) + font_size(16) = 36
    assert_eq!(main_glyph.y, 36.0, "主 glyph y 应包含 translate(20) + font_size");

    // 阴影 glyph 也应包含 transform 偏移 + shadow offset
    let shadow_glyph = &prims.glyphs[0];
    assert_eq!(
        shadow_glyph.x,
        main_glyph.x + 2.0,
        "阴影 glyph x 应包含 translate + shadow offset"
    );
    assert_eq!(
        shadow_glyph.y,
        main_glyph.y + 2.0,
        "阴影 glyph y 应包含 translate + shadow offset"
    );
}

// ── CSS direction 指示器测试 ──

/// direction:ltr（默认值）不应渲染指示器。
#[test]
fn test_paint_direction_ltr_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.direction = zero_style_system::DirectionValue::Ltr;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // ltr 不应产生任何 stroke
    assert!(painter.primitives().strokes.is_empty(), "ltr 不应渲染方向指示器");
}

/// direction:rtl 应在左上角渲染箭头指示器（3 条 stroke + 1 个 fill）。
#[test]
fn test_paint_direction_rtl_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.direction = zero_style_system::DirectionValue::Rtl;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 应产生 3 条 stroke（箭头主线 + 两个头部）+ 1 个 fill（标记方块）
    assert!(prims.strokes.len() >= 3, "rtl 应渲染方向箭头（≥3 stroke）");
    assert!(prims.fills.len() >= 1, "rtl 应渲染标记方块");
}

// ── CSS tab-size 指示器测试 ──

/// tab-size:8（默认值）不应渲染指示器。
#[test]
fn test_paint_tab_size_default_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.tab_size = zero_style_system::TabSizeValue::Number(8);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.is_empty(), "默认 tab-size 8 不应渲染指示器");
}

/// tab-size:4 应渲染指示器（4 个小方块）。
#[test]
fn test_paint_tab_size_four() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.tab_size = zero_style_system::TabSizeValue::Number(4);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 应产生 4 个 fill（每个 tab 一个小方块）
    assert!(painter.primitives().fills.len() >= 4, "tab-size:4 应渲染 4 个方块");
}

/// tab-size:0 不应渲染指示器。
#[test]
fn test_paint_tab_size_zero() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.tab_size = zero_style_system::TabSizeValue::Number(0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.is_empty(), "tab-size:0 不应渲染指示器");
}

// ── CSS border-collapse 指示器测试 ──

/// border-collapse:separate（默认值）不应渲染指示器。
#[test]
fn test_paint_border_collapse_separate_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_collapse = zero_style_system::BorderCollapseValue::Separate;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(
        painter.primitives().strokes.is_empty(),
        "separate 不应渲染边框合并指示器"
    );
}

/// border-collapse:collapse 应渲染双线指示器。
#[test]
fn test_paint_border_collapse_collapse_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_collapse = zero_style_system::BorderCollapseValue::Collapse;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 应产生 2 条 stroke（双线标记）
    assert!(painter.primitives().strokes.len() >= 2, "collapse 应渲染双线指示器");
}

// ── CSS table-layout 指示器测试 ──

/// table-layout:auto（默认值）不应渲染指示器。
#[test]
fn test_paint_table_layout_auto_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.table_layout = zero_style_system::TableLayoutValue::Auto;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.is_empty(), "auto 不应渲染表格布局指示器");
}

/// table-layout:fixed 应渲染网格图标指示器。
#[test]
fn test_paint_table_layout_fixed_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.table_layout = zero_style_system::TableLayoutValue::Fixed;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 应产生 ≥4 个 fill（网格外框 + 2 竖线 + 1 横线）
    assert!(painter.primitives().fills.len() >= 4, "fixed 应渲染网格图标");
}

// ── CSS font-variant-numeric 指示器测试 ──

/// font-variant-numeric:normal（默认值）不应渲染指示器。
#[test]
fn test_paint_font_variant_numeric_normal_no_indicator() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_variant_numeric = zero_style_system::FontVariantNumericValue::Normal;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.is_empty(), "normal 不应渲染数字变体指示器");
}

/// font-variant-numeric:tabular-nums 应渲染指示器。
#[test]
fn test_paint_font_variant_numeric_tabular_nums() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_variant_numeric = zero_style_system::FontVariantNumericValue::TabularNums;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 应产生 2 个 fill（背景 + 标记方块）
    assert!(
        painter.primitives().fills.len() >= 2,
        "tabular-nums 应渲染数字变体指示器"
    );
}

/// font-variant-numeric:slashed-zero 应渲染指示器。
#[test]
fn test_paint_font_variant_numeric_slashed_zero() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_variant_numeric = zero_style_system::FontVariantNumericValue::SlashedZero;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.len() >= 2, "slashed-zero 应渲染指示器");
}

/// font-variant-numeric:diagonal-fractions 应渲染指示器。
#[test]
fn test_paint_font_variant_numeric_diagonal_fractions() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_variant_numeric = zero_style_system::FontVariantNumericValue::DiagonalFractions;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().fills.len() >= 2, "diagonal-fractions 应渲染指示器");
}
