#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::{Document, NodeId};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::color::Color;
use zero_style_system::{BorderStyleValue, ComputedStyle, OutlineStyleValue};

use super::super::color::{color_value_to_render, hsla_to_rgba, named_color_to_render};
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

/// 辅助函数：创建带边框的 LayoutBox。
fn make_box_with_border(
    node_id: Option<NodeId>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    border_left: f32,
) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        content_x: border_left,
        content_y: border_top,
        content_width: width - border_left - border_right,
        content_height: height - border_top - border_bottom,
        border_top,
        border_right,
        border_bottom,
        border_left,
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
#[test]
fn test_painter_empty_layout() {
    let layout = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 0.0,
        content_height: 0.0,
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
    let mut painter = Painter::new();
    let styles = HashMap::new();
    painter.paint(&layout, &styles, None);
    assert!(painter.primitives().is_empty());
}
/// 测试背景色生成填充图元。
#[test]
fn test_painter_background_color() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let primitives = painter.primitives();
    assert_eq!(primitives.fills.len(), 1);
    assert_eq!(primitives.fills[0].color, Color::rgb(255, 0, 0));
    assert_eq!(primitives.fills[0].rect.origin.x, 0.0);
    assert_eq!(primitives.fills[0].rect.origin.y, 0.0);
    assert_eq!(primitives.fills[0].rect.size.width, 100.0);
    assert_eq!(primitives.fills[0].rect.size.height, 50.0);
}

/// R979：CSS §14.2 画布背景传播——html 透明时 body 背景传播到画布，body 自身盒不再绘 bg color。
/// 旧实现仅跳过 body 的 bg image（effects.rs:69 canvas_propagated_node 检查），仍绘 bg color
/// → body 盒 bg color 覆盖画布 image（background-root-007：body red 覆盖画布 tiled image）。
/// 验证 body 传播到画布时**恰好 1 个** red fill（画布），非 2 个（画布 + body 双绘）。
#[test]
fn test_canvas_propagation_body_skips_own_bg_color() {
    use zero_layout_engine::LayoutEngine;
    use zero_style_system::StyleSystem;
    let html = r#"<html><body style="background:red"><p>X</p></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(100.0, 100.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(100.0, 100.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.viewport_w = 100.0;
    painter.viewport_h = 100.0;
    painter.paint(&result.root, &styles, Some(&doc));
    let red_fills = painter
        .primitives()
        .fills
        .iter()
        .filter(|f| f.color == Color::rgb(255, 0, 0))
        .count();
    assert_eq!(
        red_fills, 1,
        "propagated body should paint exactly 1 red fill (canvas), not 2 (canvas + body double-paint)"
    );
}

/// 测试透明背景不生成填充图元。
#[test]
fn test_painter_transparent_background() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Transparent;
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().is_empty());
}

/// 测试上边框生成填充图元。
#[test]
fn test_painter_border_top() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 5.0, 0.0, 0.0, 0.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(painter.primitives().fills.len(), 1);
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 0.0);
    assert_eq!(fill.rect.origin.y, 0.0);
    assert_eq!(fill.rect.size.width, 100.0);
    assert_eq!(fill.rect.size.height, 5.0);
}

/// 测试四条边框都生成填充图元。
#[test]
fn test_painter_border_all_sides() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 3.0, 4.0, 5.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 应该有 4 个边框填充
    assert_eq!(painter.primitives().fills.len(), 4);
}

/// 测试嵌套盒子的绘制。
#[test]
fn test_painter_nested_boxes() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 10.0, 10.0, 30.0, 20.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 80.0,
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
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    assert_eq!(painter.primitives().fills.len(), 2);

    // 第一个填充是父元素背景
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
    // 第二个填充是子元素背景（位置偏移 10,10）
    assert_eq!(painter.primitives().fills[1].rect.origin.x, 10.0);
    assert_eq!(painter.primitives().fills[1].rect.origin.y, 10.0);
}

/// 测试 ColorValue::Rgba 转换。
#[test]
fn test_painter_color_value_rgba() {
    let color = color_value_to_render(&ColorValue::Rgba(128, 64, 32, 255));
    assert_eq!(color.r, 128);
    assert_eq!(color.g, 64);
    assert_eq!(color.b, 32);
    assert_eq!(color.a, 255);
}

/// 测试 ColorValue::Transparent 转换。
#[test]
fn test_painter_color_value_transparent() {
    let color = color_value_to_render(&ColorValue::Transparent);
    assert_eq!(color.a, 0);
}

/// R1080：multicol 列子元素（overflow:hidden）内的 position:relative 后代必须渲染
///（修 collect_positioned_descendants 跳过列子元素致其后代 positioned drop 的 bug——
/// multicol-overflow-clip-positioned 蓝块完全不渲染）。本地 flush + clip 到列子元素 overflow box。
#[test]
fn r1080_multicol_column_positioned_descendant_not_dropped() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(400.0, 400.0);
    let html = "<html><body style=\"margin:0\"><div style=\"columns:2\">\
                <div style=\"height:200px; overflow:hidden\">\
                <div style=\"height:800px; background:blue; position:relative\"></div>\
                </div></div></body></html>";
    let result = pipeline.render_html(html, "");
    let has_blue = result
        .primitives
        .fills
        .iter()
        .any(|f| f.color.b > 150 && f.color.r < 100 && f.color.g < 100);
    assert!(
        has_blue,
        "R1080: position:relative blue in overflow:hidden multicol column should render (was dropped), got {} fills",
        result.primitives.fills.len()
    );
}

/// R1446：multicol 容器内 **inline 元素（`<span>`）** 的 Ahem 文本须按列宽换行填满列。
///
/// 驱动案 css-multicol/multicol-basic-001（`column-count:3` + 3 个 `<span>` 各含 7 个 Ahem
/// 单词）。R1423 修复了**直接文本** multicol 的 is_ahem 传递，但 span 文本走 IFC 的
/// `collect_inline_items` **flatten 路径**（`doc.text_content` 扁平化，node_id=元素）。该路径下
/// col_ctx（layout 期，真实 styles）记录 `text_node_is_ahem[span]=true`（元素键），但 paint 期
/// `parent_is_ahem` 构造用 `is_text(tn)` 过滤把元素键丢弃 → 覆盖映射空；且 paint IFC flatten 路径
/// `is_ahem_font` 仅读 `style`（空 → false），无 override 回退。后果：paint IFC 把 Ahem 字符估宽
/// 当 11px（应 20px）→ 少换行 → 列欠填（11 行应 21）。
///
/// R1446 两处修复：① flatten 路径 `is_ahem_font` 加 override 回退；② `parent_is_ahem` 构造
/// 对元素键映射到自身（不过滤）。本测试 load-bearing：无修复时 paint IFC is_ahem=false →
/// 字符估宽偏小 → 少换行 → 列欠填（y 行数少）。
#[test]
fn r1446_multicol_span_ahem_text_fills_columns() {
    use crate::pipeline::RenderPipeline;
    let mut pipeline = RenderPipeline::new(400.0, 400.0);
    // column-count:2 → col_w=120（240/2, gap0）。span 含 12 个 Ahem 单词（20px/1）。
    // 修复后（is_ahem 传到 paint IFC）："XXXX"=80px → 1 词/行 → 12 行 → 6 行/列
    //   （y=0,20,40,60,80,100 → 两列 rebase 后去重 6 个 y 行）。
    // 修复前（is_ahem=false，估宽≈11px）："XXXX XXXX"≈99px<120 → 2 词/行 → 6 行 → 3 行/列
    //   （y=0,20,40 → 去重 3 个 y 行）。
    let html = "<html><body style=\"margin:0\">\
                <div style=\"column-count:2; column-gap:0; width:240px; font:20px/1 Ahem\">\
                <span>XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX</span>\
                </div></body></html>";
    let result = pipeline.render_html(html, "");
    // 仅统计 Ahem 尺寸（font_size≈20）字形，按行盒顶 y 分桶（两列 rebase 后 y 重叠）。
    let y_rows: std::collections::BTreeSet<i32> = result
        .primitives
        .glyphs
        .iter()
        .filter(|g| (g.font_size - 20.0).abs() < 1.0)
        .map(|g| (g.y + 0.5) as i32)
        .collect();
    assert!(
        y_rows.len() >= 5,
        "R1446: multicol 内 span(Ahem) 文本应按列宽换行填满列（is_ahem 传到 paint IFC），\
         期望 ≥5 个 y 行（6 行/列去重），got {} 行 {:?}（修复前 bug：paint IFC is_ahem=false \
         → 字符估宽偏小 → 少换行 → 列欠填）",
        y_rows.len(),
        y_rows
    );
}

/// 测试命名颜色转换（red, blue, black, white）。
#[test]
fn test_painter_color_value_named() {
    assert_eq!(named_color_to_render("red"), Color::rgb(255, 0, 0));
    assert_eq!(named_color_to_render("blue"), Color::rgb(0, 0, 255));
    assert_eq!(named_color_to_render("black"), Color::rgb(0, 0, 0));
    assert_eq!(named_color_to_render("white"), Color::rgb(255, 255, 255));
    // 大小写不敏感
    assert_eq!(named_color_to_render("Red"), Color::rgb(255, 0, 0));
    assert_eq!(named_color_to_render("BLUE"), Color::rgb(0, 0, 255));
    // 未知颜色回退为黑色
    assert_eq!(named_color_to_render("unknown"), Color::rgb(0, 0, 0));
}

/// 测试零尺寸盒子不产生有效图元（宽度为 0 时 Rect 退化为零面积）。
#[test]
fn test_painter_zero_size_box() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 0.0, 0.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 会生成一个填充，但尺寸为 0
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].rect.size.width, 0.0);
    assert_eq!(painter.primitives().fills[0].rect.size.height, 0.0);
}

/// 测试绝对偏移计算正确。
#[test]
fn test_painter_absolute_offset() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 50.0, 30.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 50.0);
    assert_eq!(fill.rect.origin.y, 30.0);
}

#[test]
fn test_negative_z_index_child_paints_before_in_flow_sibling() {
    let mut doc = Document::new();
    let normal_id = doc.create_element("div");
    let negative_id = doc.create_element("div");

    let normal_child = make_box(Some(normal_id), 0.0, 0.0, 40.0, 40.0);
    let mut negative_child = make_box(Some(negative_id), 0.0, 0.0, 40.0, 40.0);
    negative_child.is_absolute = true;
    negative_child.z_index = -1;

    let parent_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 40.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 40.0,
        content_height: 40.0,
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
        children: vec![normal_child, negative_child],
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
    let mut normal_style = ComputedStyle::default();
    normal_style.background_color = ColorValue::Named("green".to_string());
    styles.insert(normal_id, normal_style);
    let mut negative_style = ComputedStyle::default();
    negative_style.background_color = ColorValue::Named("red".to_string());
    styles.insert(negative_id, negative_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 2);
    assert_eq!(fills[0].color, Color::rgb(255, 0, 0));
    assert_eq!(fills[1].color, Color::rgb(0, 128, 0));
}

#[test]
fn test_positioned_zero_z_index_child_paints_after_in_flow_sibling() {
    let mut doc = Document::new();
    let positioned_id = doc.create_element("div");
    let normal_id = doc.create_element("div");

    let mut positioned_child = make_box(Some(positioned_id), 0.0, 0.0, 40.0, 40.0);
    positioned_child.is_absolute = true;
    positioned_child.creates_stacking_context = true; // z-index: 0 (explicit) creates stacking context
    let normal_child = make_box(Some(normal_id), 0.0, 0.0, 40.0, 40.0);

    let parent_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 40.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 40.0,
        content_height: 40.0,
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
        children: vec![positioned_child, normal_child],
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
    let mut positioned_style = ComputedStyle::default();
    positioned_style.background_color = ColorValue::Named("blue".to_string());
    styles.insert(positioned_id, positioned_style);
    let mut normal_style = ComputedStyle::default();
    normal_style.background_color = ColorValue::Named("green".to_string());
    styles.insert(normal_id, normal_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    let fills = &painter.primitives().fills;
    assert_eq!(fills.len(), 2);
    assert_eq!(fills[0].color, Color::rgb(0, 128, 0));
    assert_eq!(fills[1].color, Color::rgb(0, 0, 255));
}

/// 测试多个子节点都能生成填充图元。
#[test]
fn test_painter_multiple_children() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("span");

    let child_box1 = make_box(Some(child1), 0.0, 0.0, 50.0, 20.0);
    let child_box2 = make_box(Some(child2), 0.0, 20.0, 50.0, 20.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 80.0,
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
        children: vec![child_box1, child_box2],
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
    for id in [child1, child2] {
        let mut s = ComputedStyle::default();
        s.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(id, s);
    }

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 只有子节点有背景色，父节点没有
    assert_eq!(painter.primitives().fills.len(), 2);
}

/// 测试 into_primitives 消费 painter。
#[test]
fn test_painter_into_primitives() {
    let mut painter = Painter::new();
    let layout = make_box(None, 0.0, 0.0, 0.0, 0.0);
    let styles = HashMap::new();
    painter.paint(&layout, &styles, None);
    let primitives = painter.into_primitives();
    assert!(primitives.is_empty());
}

/// 测试 Default 实现。
#[test]
fn test_painter_default() {
    let painter = Painter::default();
    assert!(painter.primitives().is_empty());
}

/// 测试 background + border 同时存在时填充数量（1 background + 4 border = 5）。
#[test]
fn test_painter_background_plus_border_fill_count() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 1 background fill + 4 border fills = 5
    assert_eq!(painter.primitives().fills.len(), 5);
    // First fill is background
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
}

/// 测试无样式节点（no node_id）不产生任何填充。
#[test]
fn test_painter_no_style_no_fills() {
    let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
    let mut painter = Painter::new();
    let styles = HashMap::new();
    painter.paint(&layout, &styles, None);
    assert!(painter.primitives().is_empty());
}

/// 测试 only background（no border）产生恰好 1 个填充。
#[test]
fn test_painter_only_background_fill_count() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 80.0, 40.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(0, 128, 255, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert_eq!(painter.primitives().fills.len(), 1);
}

/// 测试 only border（transparent background）产生恰好 4 个填充。
#[test]
fn test_painter_only_border_fill_count() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 80.0, 40.0, 1.0, 1.0, 1.0, 1.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    // background is transparent by default
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 4 border fills, no background fill
    assert_eq!(painter.primitives().fills.len(), 4);
}

/// 测试带 padding 的子节点偏移。
#[test]
fn test_painter_child_offset_with_padding() {
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 10.0,
        content_y: 10.0,
        content_width: 80.0,
        content_height: 60.0,
        border_top: 5.0,
        border_right: 5.0,
        border_bottom: 5.0,
        border_left: 5.0,
        padding_top: 5.0,
        padding_right: 5.0,
        padding_bottom: 5.0,
        padding_left: 5.0,
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
    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // 子节点偏移 = padding_left(5) + border_left(5) = 10
    let fill = &painter.primitives().fills[0];
    assert_eq!(fill.rect.origin.x, 10.0);
    assert_eq!(fill.rect.origin.y, 10.0);
}

/// 测试 visibility: hidden 的元素不生成填充图元。
#[test]
fn test_painter_visibility_hidden() {
    use zero_css_parser::values::VisibilityValue;
    let mut doc = zero_dom::Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
    let parent_box = LayoutBox {
        node_id: Some(parent),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 80.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 100.0,
        content_height: 80.0,
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
    parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    parent_style.visibility = VisibilityValue::Hidden;
    styles.insert(parent, parent_style);

    let mut child_style = ComputedStyle::default();
    child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
    styles.insert(child, child_style);

    let mut painter = Painter::new();
    painter.paint(&parent_box, &styles, None);

    // parent 的 visibility:hidden 阻止了父节点绘制，但子节点不受影响
    assert_eq!(painter.primitives().fills.len(), 1);
    assert_eq!(painter.primitives().fills[0].color, Color::rgb(100, 100, 255));
}

/// 测试 border-style: none 的边框不生成填充图元。
#[test]
fn test_painter_border_style_none() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    // 所有边框 style 都是 none（默认值）
    style.border_top_style = BorderStyleValue::None;
    style.border_right_style = BorderStyleValue::None;
    style.border_bottom_style = BorderStyleValue::None;
    style.border_left_style = BorderStyleValue::None;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // border-style: none 不绘制边框
    assert_eq!(painter.primitives().fills.len(), 0);
}

/// 测试 border-style: solid 的边框正常绘制。
#[test]
fn test_painter_border_style_solid() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
    style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // border-style: solid 正常绘制 4 条边框
    assert_eq!(painter.primitives().fills.len(), 4);
}

/// 测试 outline 绘制。
#[test]
fn test_painter_outline() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 10.0, 20.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_width = zero_css_parser::values::LengthValue::Px(3.0);
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // outline 生成 4 个填充图元
    assert_eq!(painter.primitives().fills.len(), 4);
    // 上 outline：从 (7, 17) 开始，宽 106，高 3
    let top = &painter.primitives().fills[0];
    assert_eq!(top.rect.origin.x, 7.0);
    assert_eq!(top.rect.origin.y, 17.0);
    assert_eq!(top.rect.size.width, 106.0);
    assert_eq!(top.rect.size.height, 3.0);
    assert_eq!(top.color, Color::rgb(255, 0, 0));
}

/// 测试 outline-style: none 不绘制。
#[test]
fn test_painter_outline_style_none() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.outline_width = zero_css_parser::values::LengthValue::Px(3.0);
    style.outline_style = OutlineStyleValue::None;
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    // 设置 color 为 CurrentColor 以避免生成 glyph
    style.color = ColorValue::CurrentColor;
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    assert!(painter.primitives().is_empty());
}

/// 测试 outline + background + border 同时绘制。
#[test]
fn test_painter_background_border_outline() {
    let mut doc = zero_dom::Document::new();
    let elem = doc.create_element("div");
    let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.background_color = ColorValue::Rgba(200, 200, 200, 255);
    style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    style.outline_width = zero_css_parser::values::LengthValue::Px(2.0);
    style.outline_style = OutlineStyleValue::Solid;
    style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
    styles.insert(elem, style);

    let mut painter = Painter::new();
    painter.paint(&layout, &styles, None);

    // 1 background + 4 border + 4 outline = 9
    assert_eq!(painter.primitives().fills.len(), 9);
}

// ── 新增测试：HSL/HSLA 颜色转换 ──────────────────────────

/// 测试 HSL 红色（0°, 100%, 50%）转换为 RGB(255, 0, 0)。
#[test]
fn test_hsla_red() {
    let color = hsla_to_rgba(0.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 绿色（120°, 100%, 50%）转换为 RGB(0, 255, 0)。
#[test]
fn test_hsla_green() {
    let color = hsla_to_rgba(120.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 蓝色（240°, 100%, 50%）转换为 RGB(0, 0, 255)。
#[test]
fn test_hsla_blue() {
    let color = hsla_to_rgba(240.0, 100.0, 50.0, 1.0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 255);
    assert_eq!(color.a, 255);
}

/// 测试 HSL 半透明值。
#[test]
fn test_hsla_with_alpha() {
    let color = hsla_to_rgba(240.0, 100.0, 50.0, 0.5);
    assert_eq!(color.a, 128); // 0.5 * 255 ≈ 128
}

/// 测试 HSL 灰色（0°, 0%, 50%）。
#[test]
fn test_hsla_gray() {
    let color = hsla_to_rgba(0.0, 0.0, 50.0, 1.0);
    assert_eq!(color.r, 128);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 128);
}

/// 测试 ColorValue::Hsla 通过 color_value_to_render 正确转换。
#[test]
fn test_color_value_hsla_conversion() {
    let hsla = ColorValue::Hsla(0.0, 100.0, 50.0, 1.0);
    let color = color_value_to_render(&hsla);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}
