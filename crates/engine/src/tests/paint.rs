#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::ComputedStyle;

use crate::paint::Painter;
use crate::pipeline::RenderPipeline;
/// 测试空文档调用 paint 不 panic 且返回空图元。
#[test]
fn test_paint_empty_document() {
    let doc = zero_dom::Document::new();
    let layout = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
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
    };
    let styles = HashMap::new();

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    assert!(
        painter.primitives().is_empty(),
        "empty document should produce no render ops"
    );
}
/// 创建带 border-radius 和背景色的元素，验证 paint 生成填充图元，
/// 且填充矩形尺寸和颜色正确。圆角信息在当前架构下通过内部元数据标记。
#[test]
fn test_paint_border_radius_clip() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = std::collections::HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(100, 149, 237, 255); // cornflower blue
    style.border_top_left_radius = zero_css_parser::values::LengthValue::Px(15.0);
    style.border_top_right_radius = zero_css_parser::values::LengthValue::Px(15.0);
    style.border_bottom_right_radius = zero_css_parser::values::LengthValue::Px(15.0);
    style.border_bottom_left_radius = zero_css_parser::values::LengthValue::Px(15.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // border-radius 生成 RoundedRectPrimitive 而非 FillPrimitive
    assert_eq!(
        painter.primitives().rounded_rects.len(),
        1,
        "border-radius element should produce exactly 1 RoundedRectPrimitive"
    );
    let rr = &painter.primitives().rounded_rects[0];
    assert_eq!(rr.rect.size.width, 200.0, "width should match element width");
    assert_eq!(rr.rect.size.height, 100.0, "height should match element height");
    assert_eq!(rr.color.r, 100);
    assert_eq!(rr.color.g, 149);
    assert_eq!(rr.color.b, 237);
    assert_eq!(rr.color.a, 255);
}
/// outline-width 为 0 时 paint_outline 提前返回，不应产生填充图元。
#[test]
fn test_outline_render_no_width() {
    use zero_style_system::property::OutlineStyleValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_width = zero_css_parser::values::LengthValue::Px(0.0);
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    assert!(
        painter.primitives().fills.is_empty(),
        "outline-width:0 应不产生任何 outline 填充图元"
    );
}
/// outline-offset 使 outline 向外偏移，验证生成的填充矩形坐标反映偏移量。
#[test]
fn test_outline_render_with_offset() {
    use zero_style_system::property::OutlineStyleValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        content_x: 10.0,
        content_y: 20.0,
        content_width: 100.0,
        content_height: 50.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_width = zero_css_parser::values::LengthValue::Px(2.0);
    style.outline_offset = zero_css_parser::values::LengthValue::Px(5.0);
    style.outline_color = ColorValue::Rgba(0, 128, 255, 255);
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // outline 生成 4 个填充图元（上、下、左、右）
    assert_eq!(painter.primitives().fills.len(), 4, "outline 应生成 4 个填充图元");

    // 验证上 outline 偏移：total_offset = outline_width(2) + outline_offset(5) = 7
    // 上 outline y = abs_y(20) - total_offset(7) = 13
    let top = &painter.primitives().fills[0];
    assert_eq!(top.rect.origin.y, 13.0, "上 outline 应向外偏移 5px");
    // 上 outline x = abs_x(10) - total_offset(7) = 3
    assert_eq!(top.rect.origin.x, 3.0, "上 outline x 起始位置应反映偏移");
}
/// paint_node 检测到 visibility 为 Hidden 或 Collapse 时跳过所有绘制。
#[test]
fn test_visibility_hidden_render() {
    use zero_css_parser::values::VisibilityValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.visibility = VisibilityValue::Hidden;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // visibility:hidden 不绘制背景、边框、outline、文本
    assert!(
        painter.primitives().fills.is_empty(),
        "visibility:hidden 元素不应产生填充图元"
    );
    assert!(
        painter.primitives().glyphs.is_empty(),
        "visibility:hidden 元素不应产生文本图元"
    );
}
/// 与 visibility:hidden 不同，opacity:0 不阻止 paint 生成图元，
/// 图元仍然存在但 alpha 通道为 0。
#[test]
fn test_opacity_zero_render() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.opacity = 0.0;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // opacity:0 不阻止绘制，图元仍然生成
    assert!(
        !painter.primitives().fills.is_empty(),
        "opacity:0 元素仍应生成填充图元（由合成层处理透明度）"
    );
    // 背景填充的矩形尺寸正确
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.size.width, 200.0, "填充宽度应为元素宽度");
    assert_eq!(fill.rect.size.height, 100.0, "填充高度应为元素高度");
}
/// paint_borders 检查 border-style 是否为 None 或 Hidden，若是则跳过该边。
#[test]
fn test_border_style_none() {
    use zero_style_system::property::BorderStyleValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    // 布局有边框宽度，但 border-style 为 none
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 104.0,
        height: 104.0,
        content_x: 2.0,
        content_y: 2.0,
        content_width: 100.0,
        content_height: 100.0,
        border_top: 2.0,
        border_right: 2.0,
        border_bottom: 2.0,
        border_left: 2.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    // 所有边框 style 均为 none
    style.border_top_style = BorderStyleValue::None;
    style.border_right_style = BorderStyleValue::None;
    style.border_bottom_style = BorderStyleValue::None;
    style.border_left_style = BorderStyleValue::None;
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // border-style:none 即使有边框宽度也不绘制
    assert!(
        painter.primitives().fills.is_empty(),
        "border-style:none 不应产生任何边框填充图元"
    );
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试 visibility:hidden 的嵌套元素，父元素隐藏时子元素也不绘制。
///
/// visibility 是继承属性，子元素通过样式继承也会被隐藏。
#[test]
fn test_paint_with_visibility_hidden_nested() {
    use zero_css_parser::values::VisibilityValue;

    let mut doc = zero_dom::Document::new();
    let parent_elem = doc.create_element("div");
    let child_elem = doc.create_element("span");

    let child_box = LayoutBox {
        node_id: Some(child_elem),
        x: 10.0,
        y: 10.0,
        width: 50.0,
        height: 30.0,
        content_x: 10.0,
        content_y: 10.0,
        content_width: 50.0,
        content_height: 30.0,
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
    };
    let parent_box = LayoutBox {
        node_id: Some(parent_elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
        children: vec![child_box],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut parent_style = ComputedStyle::default();
    parent_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    parent_style.visibility = VisibilityValue::Hidden;
    styles.insert(parent_elem, parent_style);

    // visibility 是继承属性，子元素通过继承获得 hidden
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(0, 255, 0, 255);
    child_style.visibility = VisibilityValue::Hidden;
    styles.insert(child_elem, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, Some(&doc));

    assert!(
        painter.primitives().fills.is_empty(),
        "visibility:hidden 父元素及其子元素均不应产生填充图元"
    );
}
/// 测试 border-radius 裁剪：带圆角的元素尺寸信息正确。
#[test]
fn test_paint_border_radius_clipping_values() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 50.0,
        y: 50.0,
        width: 300.0,
        height: 200.0,
        content_x: 50.0,
        content_y: 50.0,
        content_width: 300.0,
        content_height: 200.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    style.border_top_left_radius = zero_css_parser::values::LengthValue::Px(20.0);
    style.border_top_right_radius = zero_css_parser::values::LengthValue::Px(20.0);
    style.border_bottom_right_radius = zero_css_parser::values::LengthValue::Px(20.0);
    style.border_bottom_left_radius = zero_css_parser::values::LengthValue::Px(20.0);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    assert_eq!(painter.primitives().rounded_rects.len(), 1);
    let rr = &painter.primitives().rounded_rects[0];
    // 验证圆角矩形位置和尺寸
    assert_eq!(rr.rect.origin.x, 50.0);
    assert_eq!(rr.rect.origin.y, 50.0);
    assert_eq!(rr.rect.size.width, 300.0);
    assert_eq!(rr.rect.size.height, 200.0);
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试 visibility:collapse 的元素在集成层面不产生渲染图元。
///
/// visibility:collapse 在非表格元素上与 hidden 行为一致，
/// 元素保留布局空间但不绘制。通过 Painter 完整管线验证：
/// 设置 collapse 后，背景填充和文本 glyph 均不应生成。
#[test]
fn test_visibility_collapse_no_primitives() {
    use zero_css_parser::values::VisibilityValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 100.0,
        content_x: 10.0,
        content_y: 20.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    style.visibility = VisibilityValue::Collapse;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // visibility:collapse 不绘制背景和文本
    assert!(
        painter.primitives().fills.is_empty(),
        "visibility:collapse 不应产生填充图元"
    );
    assert!(
        painter.primitives().glyphs.is_empty(),
        "visibility:collapse 不应产生文本图元"
    );
}
/// 正常 outline 在 border 外侧，负 outline-offset 使 outline 向内移动。
/// 验证 outline 图元的起始 y 坐标反映负偏移量。
#[test]
fn test_negative_outline_offset_inward() {
    use zero_style_system::property::OutlineStyleValue;
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_width = zero_css_parser::values::LengthValue::Px(3.0);
    // 负偏移：outline 向内移动 5px
    style.outline_offset = zero_css_parser::values::LengthValue::Px(-5.0);
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // outline 生成 4 个填充图元
    assert_eq!(painter.primitives().fills.len(), 4, "outline 应生成 4 个填充图元");

    // 验证上 outline 向内偏移：total_offset = outline_width(3) + offset(-5) = -2
    // 上 outline y = abs_y(0) - total_offset(-2) = 2
    let top = &painter.primitives().fills[0];
    assert_eq!(top.rect.origin.y, 2.0, "负 outline-offset 应使上 outline 向内偏移");
    // 上 outline 宽度 = w + 2 * total_offset = 200 + 2*(-2) = 196
    assert_eq!(top.rect.size.width, 196.0, "上 outline 宽度应反映负偏移");
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试 paint 处理 12 层深嵌套元素，交替设置 visibility: hidden/visible，
/// 验证填充图元数量仅匹配 visible 层级。
///
/// 构建一棵 12 层深的 LayoutBox 树，每个节点都有背景色，
/// 奇数层（第 0、2、4…层）为 visible，偶数层（第 1、3、5…层）为 hidden。
/// 最终只有 visible 层级的节点产生填充图元。
#[test]
fn test_paint_deeply_nested_alternating_visibility() {
    use zero_css_parser::values::VisibilityValue;

    let mut doc = zero_dom::Document::new();

    // 从最内层向外构建 12 层嵌套 LayoutBox
    let depth = 12;
    let mut elements = Vec::with_capacity(depth);
    for _ in 0..depth {
        elements.push(doc.create_element("div"));
    }

    // 最内层：叶子节点
    let innermost = LayoutBox {
        node_id: Some(elements[depth - 1]),
        x: (depth - 1) as f32 * 5.0,
        y: (depth - 1) as f32 * 5.0,
        width: 200.0 - (depth - 1) as f32 * 10.0,
        height: 100.0 - (depth - 1) as f32 * 5.0,
        content_x: (depth - 1) as f32 * 5.0,
        content_y: (depth - 1) as f32 * 5.0,
        content_width: 200.0 - (depth - 1) as f32 * 10.0,
        content_height: 100.0 - (depth - 1) as f32 * 5.0,
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
    };

    // 从内向外逐层包装
    let mut current_box = innermost;
    for i in (0..depth - 1).rev() {
        let level = i;
        current_box = LayoutBox {
            node_id: Some(elements[level]),
            x: level as f32 * 5.0,
            y: level as f32 * 5.0,
            width: 200.0 - level as f32 * 10.0,
            height: 100.0 - level as f32 * 5.0,
            content_x: level as f32 * 5.0,
            content_y: level as f32 * 5.0,
            content_width: 200.0 - level as f32 * 10.0,
            content_height: 100.0 - level as f32 * 5.0,
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
            children: vec![current_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            clear: zero_layout_engine::ClearValue::None,
            z_index: 0,
            float: zero_css_parser::values::FloatValue::None,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            ..Default::default()
        };
    }

    // 构建样式：奇数索引 visible，偶数索引 hidden
    let mut styles = HashMap::new();
    let mut visible_count = 0usize;
    for (i, &elem) in elements.iter().enumerate() {
        let mut style = ComputedStyle::default();
        style.background_color =
            ColorValue::Rgba((i as u8).wrapping_mul(20), 100, 200 - (i as u8).wrapping_mul(15), 255);
        if i % 2 == 0 {
            // 偶数层 visible
            style.visibility = VisibilityValue::Visible;
            visible_count += 1;
        } else {
            // 奇数层 hidden
            style.visibility = VisibilityValue::Hidden;
        }
        styles.insert(elem, style);
    }

    let mut painter = Painter::new();
    painter.paint(&current_box, &styles, Some(&doc));

    // 填充图元数量应等于 visible 层级数
    assert_eq!(
        painter.primitives().fills.len(),
        visible_count,
        "填充图元数应等于 visible 层级数 {}，实际 {}",
        visible_count,
        painter.primitives().fills.len()
    );
}
/// border-image-source 引用外部图片资源，在当前架构中 paint 应安全跳过
/// 或降级处理，不会因无法加载图片而崩溃。验证渲染输出存在且有效。
#[test]
fn test_paint_with_border_image_source_no_panic() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="bordered">Content with border-image</div></body></html>"#;
    let css = r#"
        .bordered {
            background-color: #4488aa;
            width: 200px;
            height: 100px;
            border: 10px solid #333333;
            border-image-source: url(test.png);
            border-image-slice: 30;
        }
    "#;

    // 渲染不 panic
    let result = pipeline.render_html(html, css);

    assert!(
        result.timings.total_ms >= 0.0,
        "含 border-image-source 的 CSS 应容错完成"
    );
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // 即使 border-image 无法加载，背景和边框仍应产生填充图元
    assert!(
        !result.primitives().fills.is_empty(),
        "含 border-image 的元素仍应产生填充图元"
    );
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试渲染管线处理含 box-shadow CSS 属性的元素不 panic 且产生填充图元。
///
/// box-shadow 属性在 CSS 中定义阴影效果（偏移、模糊、扩展、颜色）。
/// 当前架构下，paint 应安全处理 box-shadow 样式而不崩溃。
/// 验证渲染管线完成且背景填充仍然生成。
#[test]
fn test_paint_with_box_shadow_css_property() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="shadowed">Box Shadow</div></body></html>"#;
    let css = r#"
        .shadowed {
            background-color: #336699;
            width: 200px;
            height: 100px;
            box-shadow: 5px 10px 15px 3px rgba(0, 0, 0, 0.5);
        }
    "#;

    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "含 box-shadow 的 CSS 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // 背景填充应正常生成
    assert!(
        !result.primitives().fills.is_empty(),
        "含 box-shadow 的元素仍应产生背景填充图元"
    );
}
/// text-shadow 属性为文本添加阴影效果（偏移、模糊、颜色）。
/// 当前架构下，paint 应安全处理 text-shadow 样式而不崩溃。
/// 验证渲染管线完成且输出有效。
#[test]
fn test_paint_with_text_shadow_css_property() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="glow">Text Shadow</div></body></html>"#;
    let css = r#"
        .glow {
            background-color: #222222;
            width: 200px;
            height: 100px;
            color: white;
            text-shadow: 2px 2px 4px rgba(255, 255, 255, 0.6);
        }
    "#;

    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "含 text-shadow 的 CSS 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // 背景和文本图元应正常生成
    assert!(
        !result.primitives().fills.is_empty(),
        "含 text-shadow 的元素应产生背景填充图元"
    );
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试 paint 处理 border-image-source: url(test.png) 时不崩溃，背景填充正常生成。
///
/// border-image-source 引用外部图片，在当前架构中 paint 无法加载实际图片资源，
/// 但应安全降级：不 panic、背景填充仍然生成。通过 ComputedStyle 直接设置
/// border_image_source 为 Url 类型，验证 paint 输出有效。
#[test]
fn test_paint_border_image_source_url_degradation() {
    use zero_style_system::property::BorderImageSourceComputedValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 200.0,
        content_height: 100.0,
        border_top: 5.0,
        border_right: 5.0,
        border_bottom: 5.0,
        border_left: 5.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(60, 120, 180, 255);
    style.border_image_source = BorderImageSourceComputedValue::Url("test.png".to_string());
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // border-image 无法加载，但背景填充仍应正常生成
    assert!(
        !painter.primitives().fills.is_empty(),
        "border-image-source: url(test.png) 时背景填充仍应生成"
    );
    // 背景填充颜色正确
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.color.r, 60);
    assert_eq!(fill.color.g, 120);
    assert_eq!(fill.color.b, 180);
}
/// empty-cells: hide 指示浏览器隐藏空表格单元格的边框和背景。
/// 在当前架构中，paint 应安全处理此样式值而不崩溃。
/// 通过 ComputedStyle 直接设置 empty_cells 为 Hide，验证 paint 不 panic。
#[test]
fn test_paint_empty_cells_hide_no_panic() {
    use zero_style_system::property::EmptyCellsComputedValue;

    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("td");
    let layout = LayoutBox {
        node_id: Some(elem),
        x: 10.0,
        y: 20.0,
        width: 150.0,
        height: 80.0,
        content_x: 10.0,
        content_y: 20.0,
        content_width: 150.0,
        content_height: 80.0,
        border_top: 1.0,
        border_right: 1.0,
        border_bottom: 1.0,
        border_left: 1.0,
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
    };

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.empty_cells = EmptyCellsComputedValue::Hide;
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    // paint 不应 panic
    let mut painter = Painter::new();
    painter.paint(&layout, &styles, Some(&doc));

    // empty-cells:hide + no children → 不绘制背景和边框
    assert!(
        painter.primitives().fills.is_empty(),
        "empty-cells:hide empty cell should not generate fill primitives"
    );
}

// ── 新增边界条件测试：box-shadow 负偏移 / gradient 角度 / helper 函数 ──

/// 测试渲染管线处理含负偏移 box-shadow 的 CSS 不崩溃。
///
/// box-shadow 的 x/y 偏移为负值（阴影向左上方投射），
/// 验证渲染管线安全完成且背景填充正常生成。
#[test]
fn test_paint_box_shadow_negative_offset() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="shadow">Negative Shadow</div></body></html>"#;
    let css = r#"
        .shadow {
            background-color: #4488cc;
            width: 200px;
            height: 100px;
            box-shadow: -10px -5px 8px 2px rgba(0, 0, 0, 0.4);
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(
        result.timings.total_ms >= 0.0,
        "negative box-shadow offset should not crash"
    );
    assert!(pipeline.layout().is_some());
    assert!(
        !result.primitives().fills.is_empty(),
        "element with negative box-shadow should produce fills"
    );
}

/// 测试渲染管线处理零扩散 box-shadow 不崩溃。
#[test]
fn test_paint_box_shadow_zero_spread() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="shadow">Zero Spread</div></body></html>"#;
    let css = r#"
        .shadow {
            background-color: #336699;
            width: 200px;
            height: 100px;
            box-shadow: 5px 5px 10px 0px rgba(0, 0, 0, 0.3);
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(
        result.timings.total_ms >= 0.0,
        "zero-spread box-shadow should not crash"
    );
    assert!(!result.primitives().fills.is_empty());
}

/// 测试渲染管线处理 inset box-shadow 不崩溃。
#[test]
fn test_paint_box_shadow_inset() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="inset">Inset Shadow</div></body></html>"#;
    let css = r#"
        .inset {
            background-color: #dddddd;
            width: 200px;
            height: 100px;
            box-shadow: inset 3px 3px 5px rgba(0, 0, 0, 0.5);
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "inset box-shadow should not crash");
    assert!(!result.primitives().fills.is_empty());
}

/// 测试渲染管线处理含角度渐变的 CSS 不崩溃。
///
/// linear-gradient(45deg, red, blue) 使用角度渐变方向，
/// 验证渲染管线安全处理角度计算。
#[test]
fn test_paint_gradient_with_angle() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="gradient">Angled</div></body></html>"#;
    let css = r#"
        .gradient {
            background: linear-gradient(45deg, #ff0000, #0000ff);
            width: 200px;
            height: 200px;
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "angled gradient should not crash");
    assert!(pipeline.layout().is_some());
}

/// 测试渲染管线处理 0deg 渐变不崩溃。
#[test]
fn test_paint_gradient_0deg() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="g">Zero Deg</div></body></html>"#;
    let css = r#"
        .g {
            background: linear-gradient(0deg, #ff0000, #00ff00);
            width: 200px;
            height: 100px;
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "0deg gradient should not crash");
}

/// 测试渲染管线处理 180deg 渐变不崩溃。
#[test]
fn test_paint_gradient_180deg() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="g">180 Deg</div></body></html>"#;
    let css = r#"
        .g {
            background: linear-gradient(180deg, #000000, #ffffff);
            width: 200px;
            height: 100px;
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "180deg gradient should not crash");
}

/// 测试渲染管线处理 360deg 渐变（等同于 0deg）不崩溃。
#[test]
fn test_paint_gradient_360deg() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="g">360 Deg</div></body></html>"#;
    let css = r#"
        .g {
            background: linear-gradient(360deg, #ff0000, #0000ff);
            width: 200px;
            height: 100px;
        }
    "#;

    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "360deg gradient should not crash");
}

/// CSS §11.1.1：overflow 仅裁剪 CB 为本元素或其后代的 positioned 后代。
/// 非 positioned 的 overflow 元素（如 `<div style="overflow:hidden">`），
/// 其 abspos 后代的 CB 必为祖先，不应被该 overflow 裁剪。
/// 结构：relative(CB, 100x100) > overflow:hidden(h=0, 非 positioned) > abspos(100x100 green)
/// 旧实现把 abspos 当普通子元素绘制，被 h=0 的 overflow 裁剪到不可见。
/// 修复：非 positioned overflow 元素的 abspos/fixed 子元素移到裁剪之后绘制。
#[test]
fn test_overflow_nonpositioned_does_not_clip_abspos_with_ancestor_cb() {
    let mut doc = zero_dom::Document::new();
    let relative = doc.create_element("div");
    let overflow_el = doc.create_element("div");
    let abspos = doc.create_element("div");

    let abspos_box = LayoutBox {
        node_id: Some(abspos),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_width: 100.0,
        content_height: 100.0,
        is_absolute: true,
        ..Default::default()
    };
    let overflow_box = LayoutBox {
        node_id: Some(overflow_el),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 0.0,
        content_width: 100.0,
        content_height: 0.0,
        overflow_x: OverflowClip::Hidden,
        overflow_y: OverflowClip::Hidden,
        children: vec![abspos_box],
        ..Default::default()
    };
    let relative_box = LayoutBox {
        node_id: Some(relative),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        content_width: 100.0,
        content_height: 100.0,
        is_relative: true,
        children: vec![overflow_box],
        ..Default::default()
    };
    let root = LayoutBox {
        width: 800.0,
        height: 600.0,
        content_width: 800.0,
        content_height: 600.0,
        children: vec![relative_box],
        ..Default::default()
    };

    let mut styles = HashMap::new();
    let mut abspos_style = ComputedStyle::default();
    abspos_style.background_color = ColorValue::Rgba(0, 128, 0, 255); // green
    styles.insert(abspos, abspos_style);

    let mut painter = Painter::new();
    painter.paint(&root, &styles, Some(&doc));

    // abspos 绿色填充应存在且高度 ~100（未被 h=0 的非 positioned overflow 裁剪）
    let green_heights: Vec<f32> = painter
        .primitives()
        .fills
        .iter()
        .filter(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0)
        .map(|f| f.rect.size.height)
        .collect();
    assert!(
        !green_heights.is_empty(),
        "abspos 绿色填充应存在（未被非 positioned overflow 裁剪掉）"
    );
    let max_h = green_heights.iter().cloned().fold(0.0f32, f32::max);
    assert!(
        max_h > 50.0,
        "abspos 填充高度应 ~100（未被 overflow:h=0 裁剪），实际 max_h={}",
        max_h
    );
}
