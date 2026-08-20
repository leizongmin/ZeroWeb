//! 视觉效果渲染测试 — box-shadow、background-image、text-shadow、CSS 属性指示器。
//!
//! 从 effects.rs 拆分而来，包含视觉渲染效果的单元测试。

#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, LengthValue, TransformFunction, TransformValue};
use zero_render_foundation::color::Color;
use zero_style_system::{
    BackgroundImageComputedValue, BoxShadowComputedValue, ClipPathComputedValue, ClipPathRadius, ComputedStyle,
    TextShadowComputedValue,
};

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
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    }];
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
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    }];
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
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 0.0,
        offset_y: 3.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    }];
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
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(255, 0, 0, 255),
    }];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let shadow_glyph = &painter.primitives().glyphs[0];
    assert_eq!(shadow_glyph.color, Color::rgb(255, 0, 0), "阴影 glyph 颜色应为红色");
}

/// 测试 text-shadow 省略颜色（currentColor）按元素 `color` 解析（CSS Text Decoration §3）。
/// driving: R2364 — `color: red` + `text-shadow: 2px 2px`（省略颜色）应渲染红阴影，
/// 此前 paint 用 color_value_to_render 把 currentColor 回落黑色。
#[test]
fn test_paint_text_shadow_currentcolor_resolves_to_element_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(255, 0, 0, 255); // 元素色 = 红
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::CurrentColor, // 省略颜色 → currentColor
    }];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let shadow_glyph = &painter.primitives().glyphs[0];
    assert_eq!(
        shadow_glyph.color,
        Color::rgb(255, 0, 0),
        "currentColor 阴影应解析为元素 color（红），非黑色"
    );
}

/// 测试 clip-path: inset(<percentage>) 按 border-box 尺寸解析百分比裁剪（CSS Masking §inset）。
/// driving: R2365 — `inset(10% 20% 10% 20%)` 此前 paint 用 length_to_f32 把百分比丢为 0
/// → 不裁剪；应 top/bottom = % of height、left/right = % of width。
#[test]
fn test_paint_clip_path_inset_percentage_clips() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 100×50 框
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.color = ColorValue::CurrentColor; // 避免生成 glyph
    style.clip_path = ClipPathComputedValue::Inset {
        top: LengthValue::Percentage(10.0),    // 10% of h=50 → 5
        right: LengthValue::Percentage(20.0),  // 20% of w=100 → 20
        bottom: LengthValue::Percentage(10.0), // 5
        left: LengthValue::Percentage(20.0),   // 20
        round: None,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(!fills.is_empty(), "应有背景 fill");
    // inset 后 clip 宽 = 100 - left(20) - right(20) = 60；fill 被裁剪到 60 宽。
    // 修复前：百分比被丢为 0 → 不裁剪 → fill 宽仍 100。
    assert_eq!(
        fills[0].rect.size.width, 60.0,
        "inset(%) 应按 border-box 尺寸解析百分比裁剪（top/bottom→height, left/right→width）"
    );
    assert_eq!(
        fills[0].rect.size.height, 40.0,
        "clip 高 = 50 - top(5) - bottom(5) = 40"
    );
}

/// 测试 clip-path: inset(<em>) 按元素 font-size 解析（CSS Masking §inset + CSS Values §length）。
/// driving: R2365 — inset em 未在 computed 预解析，paint 须按 font-size 解析（否则丢为 0 不裁剪）。
#[test]
fn test_paint_clip_path_inset_em_clips() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(10.0); // 1em = 10px
    style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    style.color = ColorValue::CurrentColor;
    // inset(2em 1em 2em 1em) → top/bottom=2em=20, left/right=1em=10
    style.clip_path = ClipPathComputedValue::Inset {
        top: LengthValue::Em(2.0),
        right: LengthValue::Em(1.0),
        bottom: LengthValue::Em(2.0),
        left: LengthValue::Em(1.0),
        round: None,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(!fills.is_empty(), "应有背景 fill");
    // clip 宽 = 100 - left(10) - right(10) = 80（修复前 em 丢为 0 → 100 不裁剪）。
    assert_eq!(fills[0].rect.size.width, 80.0, "inset(em) 应按 font-size 解析");
    assert_eq!(
        fills[0].rect.size.height, 10.0,
        "clip 高 = 50 - top(20) - bottom(20) = 10"
    );
}

#[test]
fn test_paint_clip_path_inset_residual_font_size_length() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Em(2.0);
    style.background_color = ColorValue::Rgba(0, 0, 255, 255);
    style.color = ColorValue::CurrentColor;
    style.clip_path = ClipPathComputedValue::Inset {
        top: LengthValue::Em(1.0),
        right: LengthValue::Em(1.0),
        bottom: LengthValue::Em(1.0),
        left: LengthValue::Em(1.0),
        round: None,
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fills = &painter.primitives().fills;
    assert!(!fills.is_empty(), "应有背景 fill");
    assert_eq!(
        fills[0].rect.size.width, 36.0,
        "font-size:2em 下 inset(1em) 应按 32px 裁剪左右"
    );
    assert_eq!(
        fills[0].rect.size.height, 36.0,
        "font-size:2em 下 inset(1em) 应按 32px 裁剪上下"
    );
}

/// 测试 clip-path: circle(<percentage>) 半径按 sqrt(w²+h²)/√2 解析（CSS basic-shape circle）。
/// driving: R2366 — `circle(50%)` 此前 paint 用 length_to_f32 把百分比丢为 0 → 退化半径 0
/// → 裁剪区域为零（元素被完全裁掉）；应 radius = 50%×sqrt(w²+h²)/√2。
#[test]
fn test_paint_clip_path_circle_percentage_clips() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 100.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    style.color = ColorValue::CurrentColor;
    style.clip_path = ClipPathComputedValue::Circle {
        radius: ClipPathRadius::Length(LengthValue::Percentage(50.0)),
        position: None, // 默认居中 (50,50)
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // circle(50%) → radius = 50%（100×100 上 sqrt(100²+100²)/√2=100，×0.5=50）→ 内切圆裁剪。
    // 修复前：radius=0 → 退化多边形 → fill 全部裁零（总面积 0）。
    let total_area: f32 = painter
        .primitives()
        .fills
        .iter()
        .map(|f| f.rect.size.width * f.rect.size.height)
        .sum();
    assert!(
        total_area > 1000.0,
        "circle(50%) 应产生非零裁剪区域（内切圆≈π·50²≈7854），修复前为 0"
    );
    // 圆形裁剪区域应小于完整 100×100=10000（确有裁剪）。
    assert!(total_area < 9500.0, "circle(50%) 应裁掉四角（面积 < 完整框）");
}

/// 测试 clip-path: polygon(<percentage>) 顶点按 width/height 解析（CSS basic-shape polygon）。
/// driving: R2366 — `polygon(...)` 顶点百分比此前丢为 0 → 退化（全部顶点在原点）。
#[test]
fn test_paint_clip_path_polygon_percentage_clips() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 0, 200, 255);
    style.color = ColorValue::CurrentColor;
    // 左半矩形：0% 0%, 50% 0%, 50% 100%, 0% 100% → x∈[0,50], y∈[0,50]
    style.clip_path = ClipPathComputedValue::Polygon {
        fill_rule: zero_css_parser::values::PolygonFillRule::NonZero,
        points: vec![
            (LengthValue::Percentage(0.0), LengthValue::Percentage(0.0)),
            (LengthValue::Percentage(50.0), LengthValue::Percentage(0.0)),
            (LengthValue::Percentage(50.0), LengthValue::Percentage(100.0)),
            (LengthValue::Percentage(0.0), LengthValue::Percentage(100.0)),
        ],
    };
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 左半裁剪 → 总面积 ≈ 50×50 = 2500。修复前：百分比丢为 0 → 退化 → 0。
    let total_area: f32 = painter
        .primitives()
        .fills
        .iter()
        .map(|f| f.rect.size.width * f.rect.size.height)
        .sum();
    assert!(
        (2000.0..=3000.0).contains(&total_area),
        "polygon 左半(50%) 应裁剪到约 2500px²（50×50），修复前为 0，得到 {total_area}"
    );
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
    style.text_shadow = vec![TextShadowComputedValue {
        offset_x: 2.0,
        offset_y: 2.0,
        blur_radius: 0.0,
        color: ColorValue::Rgba(0, 0, 0, 128),
    }];
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

/// R2305：多 text-shadow 列表每个字符生成多组阴影 glyph（CSS Text Decoration §3）。
#[test]
fn test_paint_text_shadow_multiple_list() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = vec![
        TextShadowComputedValue {
            offset_x: 2.0,
            offset_y: 2.0,
            blur_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 128),
        },
        TextShadowComputedValue {
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 0.0,
            color: ColorValue::Rgba(255, 0, 0, 128),
        },
    ];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 2 个阴影 glyph + 1 个主 glyph = 3
    assert_eq!(prims.glyphs.len(), 3, "应生成 3 个 glyph（2 阴影 + 1 主）");
    // 按列表顺序绘制：首个阴影先入列
    assert_eq!(prims.glyphs[0].color, Color::rgba(0, 0, 0, 128), "首个阴影 glyph 颜色");
    assert_eq!(
        prims.glyphs[1].color,
        Color::rgba(255, 0, 0, 128),
        "第二个阴影 glyph 颜色"
    );
}

/// R2305：text-shadow 列表中含全零项时，该零项被跳过（不生成阴影 glyph）。
#[test]
fn test_paint_text_shadow_list_skips_zero_entry() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.color = ColorValue::Rgba(0, 0, 0, 255);
    style.text_shadow = vec![
        TextShadowComputedValue {
            // 全零：应被跳过
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 128),
        },
        TextShadowComputedValue {
            offset_x: 0.0,
            offset_y: 3.0,
            blur_radius: 0.0,
            color: ColorValue::Rgba(0, 0, 0, 128),
        },
    ];
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let prims = painter.primitives();
    // 全零阴影项被跳过，仅 1 个阴影 glyph + 1 个主 glyph = 2
    assert_eq!(
        prims.glyphs.len(),
        2,
        "全零阴影项被跳过，应生成 2 个 glyph（1 阴影 + 1 主）"
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
