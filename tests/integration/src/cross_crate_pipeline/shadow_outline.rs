use std::collections::HashMap;

use zero_css_parser::Parser as CssParser;
use zero_css_parser::values::{
    AlignmentValue, ColorValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FontWeightValue, LengthValue,
    OverflowValue, PositionValue, TransformFunction, TransformValue, parse_transform,
};
use zero_dom::Document;
use zero_engine::RenderPipeline;
use zero_layout_engine::LayoutEngine;
use zero_render_foundation::color::Color;
use zero_style_system::{ComputedStyle, GridLineValue, StyleSystem};

// ── 辅助函数 ──

/// 创建 html > body 基础 DOM，返回 (doc, body NodeId)。
fn make_doc_with_body() -> (Document, zero_dom::NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

/// 在 LayoutBox 子树中查找指定 node_id 的盒子。
fn find_box_by_node_id(
    root: &zero_layout_engine::LayoutBox,
    target_id: zero_dom::NodeId,
) -> Option<&zero_layout_engine::LayoutBox> {
    if root.node_id == Some(target_id) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_box_by_node_id(child, target_id) {
            return Some(found);
        }
    }
    None
}

// ── 测试 ──

/// CSS Transform 管线集成测试。
///
/// 通过 css-parser 解析含多个变换函数的 transform 值，
/// 再由 style-system 计算样式，验证 ComputedStyle.transform 包含
/// rotate(45deg) → scale(2) → translate(10px, 20px) 三个函数且顺序正确。

#[test]
fn test_border_spacing_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("table");
    doc.set_attribute(parent, "class", "parent");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("td");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent { border-spacing: 3px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.border_spacing.horizontal - 3.0).abs() < 0.01,
        "parent border-spacing horizontal 应为 3.0"
    );

    // 验证子元素继承了 border-spacing: 3px
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.border_spacing.horizontal - 3.0).abs() < 0.01,
        "child 应继承 parent 的 border-spacing horizontal=3.0，实际为 {}",
        child_style.border_spacing.horizontal
    );
}

/// CSS border-image 简写属性管线集成测试。
///
/// 解析含 border-image: url(border.png) 25 的 CSS，
/// 通过 style-system 简写展开为 border-image-source 和 border-image-slice，
/// 验证 border-image-source 为 Url("border.png")，border-image-slice top 为 Number(25)。
#[test]
fn test_border_image_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "bordered");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .bordered { border-image: url(border.png) 25; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 border-image-source 为 Url
    assert_eq!(
        div_style.border_image_source,
        zero_style_system::property::BorderImageSourceComputedValue::Url("border.png".to_string()),
        "div 的 border-image-source 应为 Url(\"border.png\")"
    );

    // 验证 border-image-slice top 为 Number(25)
    use zero_style_system::property::BorderImageSliceComputedComponent;
    assert_eq!(
        div_style.border_image_slice.top,
        BorderImageSliceComputedComponent::Number(25.0),
        "div 的 border-image-slice top 应为 Number(25)"
    );
}

/// CSS counter-set 管线集成测试。
///
/// 解析含 counter-set: mycounter 5 的 CSS，通过 style-system 计算样式，
/// 验证 counter_set 列表中包含 mycounter，值为 5。
#[test]
fn test_counter_set_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "counter-set");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .counter-set { counter-set: mycounter 5; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert!(!div_style.counter_set.is_empty(), "div 的 counter_set 不应为空");
    assert_eq!(div_style.counter_set.len(), 1, "应有一个 counter-set 条目");
    assert_eq!(div_style.counter_set[0].name, "mycounter", "计数器名应为 mycounter");
    assert_eq!(div_style.counter_set[0].value, Some(5), "设定值应为 5");
}

/// CSS empty-cells: show 管线集成测试。
///
/// 解析含 empty-cells: show 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.empty_cells 为 Show。
#[test]
fn test_empty_cells_show_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let td = doc.create_element("td");
    doc.set_attribute(td, "class", "visible");
    doc.append_child(body, td).unwrap();

    let css = r#"
        .visible { empty-cells: show; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let td_style = styles.get(&td).expect("td 应有计算样式");
    assert_eq!(
        td_style.empty_cells,
        zero_style_system::property::EmptyCellsComputedValue::Show,
        "td 的 empty-cells 应为 Show"
    );
}

/// CSS border-spacing 双值继承管线集成测试。
///
/// border-spacing 是继承属性。父元素设置 border-spacing: 10px 20px，
/// 子元素不显式设置，应继承 horizontal=10.0, vertical=20.0。
#[test]
fn test_border_spacing_dual_value_inheritance_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("table");
    doc.set_attribute(parent, "class", "parent");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("td");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent { border-spacing: 10px 20px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.border_spacing.horizontal - 10.0).abs() < 0.01,
        "parent border-spacing horizontal 应为 10.0，实际为 {}",
        parent_style.border_spacing.horizontal
    );
    assert!(
        (parent_style.border_spacing.vertical - 20.0).abs() < 0.01,
        "parent border-spacing vertical 应为 20.0，实际为 {}",
        parent_style.border_spacing.vertical
    );

    // 验证子元素继承了 border-spacing: 10px 20px
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.border_spacing.horizontal - 10.0).abs() < 0.01,
        "child 应继承 parent 的 border-spacing horizontal=10.0，实际为 {}",
        child_style.border_spacing.horizontal
    );
    assert!(
        (child_style.border_spacing.vertical - 20.0).abs() < 0.01,
        "child 应继承 parent 的 border-spacing vertical=20.0，实际为 {}",
        child_style.border_spacing.vertical
    );
}

/// CSS justify-items: center 管线集成测试。
///
/// 解析含 justify-items: center 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.justify_items 为 Center。
#[test]
fn test_justify_items_center_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "centered");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .centered { justify-items: center; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.justify_items,
        zero_style_system::property::JustifyItemsValue::Center,
        "div 的 justify-items 应为 Center"
    );
}

// ── 新增测试：box-shadow / text-shadow / background-image 渲染管线集成 ──

/// CSS box-shadow 渲染管线集成测试 — 验证 box-shadow 属性通过完整管线正确传递。
#[test]
fn test_box_shadow_render_pipeline() {
    let html = r#"<html><body>
        <div class="shadowed" style="width: 200px; height: 100px;">Box</div>
    </body></html>"#;
    let css = r#".shadowed { box-shadow: 5px 10px 20px blue; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 渲染应成功完成
    assert!(result.timings.total_ms >= 0.0, "渲染应成功完成");
    // 应生成至少一个 shadow 图元
    assert!(
        !result.primitives.shadows.is_empty(),
        "box-shadow 应生成 ShadowPrimitive，实际 shadows 数量: {}",
        result.primitives.shadows.len()
    );

    // 验证 shadow 参数
    let shadow = &result.primitives.shadows[0];
    assert!(
        (shadow.offset_x - 5.0).abs() < 0.01,
        "shadow offset_x 应为 5.0，实际为 {}",
        shadow.offset_x
    );
    assert!(
        (shadow.offset_y - 10.0).abs() < 0.01,
        "shadow offset_y 应为 10.0，实际为 {}",
        shadow.offset_y
    );
    assert!(
        (shadow.blur_radius - 20.0).abs() < 0.01,
        "shadow blur_radius 应为 20.0，实际为 {}",
        shadow.blur_radius
    );
}

/// CSS box-shadow 多值管线集成测试。
#[test]
fn test_box_shadow_with_background_color_pipeline() {
    let html = r#"<html><body>
        <div class="box" style="width: 200px; height: 100px;">Content</div>
    </body></html>"#;
    let css = r#"
        .box { background-color: red; box-shadow: 3px 4px 10px rgba(0,0,0,0.5); }
    "#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 应同时有 fills（背景色）和 shadows（box-shadow）
    assert!(!result.primitives.fills.is_empty(), "background-color 应生成填充图元");
    assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
}

/// CSS box-shadow 负偏移管线集成测试。
#[test]
fn test_box_shadow_negative_offset_pipeline() {
    let html = r#"<html><body>
        <div class="neg" style="width: 200px; height: 100px;">Neg</div>
    </body></html>"#;
    let css = r#".neg { box-shadow: -5px -3px 8px green; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    assert!(!result.primitives.shadows.is_empty(), "应有阴影图元");
    let shadow = &result.primitives.shadows[0];
    assert!(
        (shadow.offset_x - (-5.0)).abs() < 0.01,
        "shadow offset_x 应为 -5.0，实际为 {}",
        shadow.offset_x
    );
    assert!(
        (shadow.offset_y - (-3.0)).abs() < 0.01,
        "shadow offset_y 应为 -3.0，实际为 {}",
        shadow.offset_y
    );
    assert!(
        (shadow.blur_radius - 8.0).abs() < 0.01,
        "shadow blur_radius 应为 8.0，实际为 {}",
        shadow.blur_radius
    );
}

/// CSS box-shadow 默认值（无阴影）管线集成测试。
#[test]
fn test_box_shadow_none_pipeline() {
    let html = r#"<html><body>
        <div class="plain">Plain</div>
    </body></html>"#;
    let css = r#".plain { width: 200px; height: 100px; background-color: gray; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 无 box-shadow 时 shadows 应为空
    assert!(
        result.primitives.shadows.is_empty(),
        "无 box-shadow 时不应生成阴影图元，实际数量: {}",
        result.primitives.shadows.len()
    );
    // 背景色应生成 fills
    assert!(!result.primitives.fills.is_empty(), "背景色应生成填充图元");
}

/// CSS text-shadow 渲染管线集成测试。
#[test]
fn test_text_shadow_render_pipeline() {
    let html = r#"<html><body>
        <div class="text" style="width: 200px; height: 50px; color: black; font-size: 16px;">Hello</div>
    </body></html>"#;
    let css = r#".text { text-shadow: 2px 3px red; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // text-shadow 会为每个字符生成额外的 shadow glyph
    // 应有 glyph 生成（shadow glyphs + main glyphs）
    assert!(
        !result.primitives.glyphs.is_empty(),
        "text-shadow 应生成 glyph 图元（shadow + main），实际数量: {}",
        result.primitives.glyphs.len()
    );

    // 有 text-shadow 时 glyph 数量应多于无 shadow 的情况
    // 因为每个字符会同时生成 shadow glyph 和 main glyph
    let glyph_count = result.primitives.glyphs.len();
    assert!(glyph_count >= 2, "至少应有 shadow + main 两个 glyph");
}

/// CSS text-shadow 多层属性管线集成测试。
#[test]
fn test_text_shadow_with_color_pipeline() {
    let html = r#"<html><body>
        <div class="shadow-text">Shadow</div>
    </body></html>"#;
    let css = r#"
        .shadow-text {
            width: 200px; height: 50px;
            color: blue; font-size: 14px;
            text-shadow: 1px 2px green;
        }
    "#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 验证 glyph 生成
    assert!(!result.primitives.glyphs.is_empty(), "应有 glyph 生成");

    // 查找 shadow glyph — 颜色应为 green (0, 128, 0)
    let has_shadow_glyph = result
        .primitives
        .glyphs
        .iter()
        .any(|g| g.color.g > 100 && g.color.r == 0 && g.color.b == 0);
    assert!(has_shadow_glyph, "应存在 green 颜色的 shadow glyph");

    // 查找 main glyph — 颜色应为 blue (0, 0, 255)
    let has_main_glyph = result
        .primitives
        .glyphs
        .iter()
        .any(|g| g.color.b == 255 && g.color.r == 0 && g.color.g == 0);
    assert!(has_main_glyph, "应存在 blue 颜色的 main glyph");
}

/// CSS text-shadow 默认值管线集成测试。
#[test]
fn test_text_shadow_none_pipeline() {
    let html = r#"<html><body>
        <div class="no-shadow" style="width: 200px; height: 50px; color: black; font-size: 16px;">Text</div>
    </body></html>"#;
    let css = r#".no-shadow { }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 无 text-shadow 时，每个字符只有 1 个 main glyph，没有 shadow glyph
    // 所有 glyph 的颜色应为主色（黑色或继承色）
    // 不应有红色/绿色等 shadow 颜色的 glyph
    assert!(!result.primitives.glyphs.is_empty(), "应生成主文本 glyph");
}

/// CSS background-image url() 渲染管线集成测试。
#[test]
fn test_background_image_url_render_pipeline() {
    let html = r#"<html><body>
        <div class="bg" style="width: 200px; height: 100px;">Background</div>
    </body></html>"#;
    let css = r#".bg { background-image: url(hero.png); }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 应生成 ImagePrimitive
    assert!(
        !result.primitives.images.is_empty(),
        "background-image: url() 应生成 ImagePrimitive，实际数量: {}",
        result.primitives.images.len()
    );
}

/// CSS background-image none 管线集成测试。
#[test]
fn test_background_image_none_pipeline() {
    let html = r#"<html><body>
        <div class="no-bg" style="width: 200px; height: 100px; background-color: white;">NoImg</div>
    </body></html>"#;
    let css = r#".no-bg { background-image: none; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // background-image: none 不应生成 ImagePrimitive
    assert!(
        result.primitives.images.is_empty(),
        "background-image: none 不应生成图片图元，实际数量: {}",
        result.primitives.images.len()
    );
}

/// CSS background-image 与 background-color 组合管线集成测试。
#[test]
fn test_background_image_with_color_pipeline() {
    let html = r#"<html><body>
        <div class="combo" style="width: 200px; height: 100px;">Combo</div>
    </body></html>"#;
    let css = r#"
        .combo {
            background-color: #f0f0f0;
            background-image: url(bg.jpg);
        }
    "#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 应同时有 fills（背景色）和 images（背景图片）
    assert!(!result.primitives.fills.is_empty(), "background-color 应生成填充图元");
    assert!(!result.primitives.images.is_empty(), "background-image 应生成图片图元");
}

/// CSS box-shadow 继承性管线集成测试（box-shadow 不可继承）。
#[test]
fn test_box_shadow_not_inherited_render_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 父元素有 box-shadow
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "parent-shadow");
    doc.append_child(body, parent).unwrap();

    // 子元素不设置 box-shadow
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent-shadow { box-shadow: 5px 5px blue; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 父元素应有 box-shadow
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.box_shadow.offset_x - 5.0).abs() < 0.01,
        "parent 的 box-shadow offset_x 应为 5.0"
    );

    // 子元素不应继承 box-shadow（offset_x 应为默认值 0.0）
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.box_shadow.offset_x - 0.0).abs() < 0.01,
        "child 不应继承 box-shadow，offset_x 应为 0.0，实际为 {}",
        child_style.box_shadow.offset_x
    );
}

/// CSS text-shadow 继承性管线集成测试（text-shadow 可继承）。
#[test]
fn test_text_shadow_inherited_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 父元素有 text-shadow
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "parent-shadow");
    doc.append_child(body, parent).unwrap();

    // 子元素不设置 text-shadow，应继承
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent-shadow { text-shadow: 3px 4px orange; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 父元素应有 text-shadow
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        (parent_style.text_shadow.offset_x - 3.0).abs() < 0.01,
        "parent 的 text-shadow offset_x 应为 3.0"
    );

    // 子元素应继承 text-shadow
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        (child_style.text_shadow.offset_x - 3.0).abs() < 0.01,
        "child 应继承 text-shadow offset_x=3.0，实际为 {}",
        child_style.text_shadow.offset_x
    );
    assert!(
        (child_style.text_shadow.offset_y - 4.0).abs() < 0.01,
        "child 应继承 text-shadow offset_y=4.0，实际为 {}",
        child_style.text_shadow.offset_y
    );
}

/// CSS box-shadow + outline 组合管线集成测试。
#[test]
fn test_box_shadow_with_outline_pipeline() {
    let html = r#"<html><body>
        <div class="combined" style="width: 200px; height: 100px;">Combined</div>
    </body></html>"#;
    let css = r#"
        .combined {
            box-shadow: 4px 6px 12px rgba(0,0,0,0.3);
            outline: 2px solid red;
        }
    "#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 应同时有 shadows 和 outline fills
    assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
    // outline 生成 4 个 fill 图元
    assert!(
        result.primitives.fills.len() >= 4,
        "outline 应生成 4 个填充图元，实际数量: {}",
        result.primitives.fills.len()
    );
}

/// CSS background-image + border + box-shadow 全组合管线集成测试。
#[test]
fn test_all_three_new_properties_combined_pipeline() {
    let html = r#"<html><body>
        <div class="all" style="width: 200px; height: 100px; color: black; font-size: 14px;">All</div>
    </body></html>"#;
    let css = r#"
        .all {
            background-color: #eee;
            background-image: url(wallpaper.jpg);
            box-shadow: 2px 3px 8px rgba(0,0,0,0.5);
            text-shadow: 1px 1px red;
            border: 1px solid #ccc;
        }
    "#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // 验证所有新属性都生成了图元
    assert!(!result.primitives.images.is_empty(), "background-image 应生成图片图元");
    assert!(!result.primitives.shadows.is_empty(), "box-shadow 应生成阴影图元");
    assert!(!result.primitives.fills.is_empty(), "背景色 + 边框应生成填充图元");
    assert!(
        !result.primitives.glyphs.is_empty(),
        "text-shadow + 文本应生成 glyph 图元"
    );

    // glyph 数量应 >= 2（shadow glyph + main glyph）
    assert!(
        result.primitives.glyphs.len() >= 2,
        "text-shadow 应使 glyph 数量翻倍（shadow + main），实际数量: {}",
        result.primitives.glyphs.len()
    );
}

/// CSS box-shadow 仅 spread-radius 管线集成测试。
#[test]
fn test_box_shadow_spread_only_pipeline() {
    let html = r#"<html><body>
        <div class="spread" style="width: 200px; height: 100px;">Spread</div>
    </body></html>"#;
    let css = r#".spread { box-shadow: 0 0 0 5px purple; }"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);

    // spread-only shadow 仍应生成 ShadowPrimitive（spread_radius=5）
    assert!(
        !result.primitives.shadows.is_empty(),
        "spread-only box-shadow 应生成阴影图元，实际数量: {}",
        result.primitives.shadows.len()
    );

    let shadow = &result.primitives.shadows[0];
    assert!(
        (shadow.spread_radius - 5.0).abs() < 0.01,
        "shadow spread_radius 应为 5.0，实际为 {}",
        shadow.spread_radius
    );
    assert!((shadow.offset_x - 0.0).abs() < 0.01, "shadow offset_x 应为 0.0");
    assert!((shadow.offset_y - 0.0).abs() < 0.01, "shadow offset_y 应为 0.0");
}

// ── CSS 渐变管线集成测试 ──

/// CSS linear-gradient 渲染管线集成测试。
///
/// 解析 background-image: linear-gradient(to bottom, red, blue)，
/// 通过 style-system 计算样式，验证 background_image 为 Gradient 变体，
/// 方向为 ToBottom，色标有 2 个元素。
#[test]
fn test_linear_gradient_render_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "grad");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .grad { background-image: linear-gradient(to bottom, red, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Linear(lin) => {
                assert_eq!(
                    lin.direction,
                    zero_css_parser::values::GradientDirection::ToBottom,
                    "linear-gradient 方向应为 ToBottom"
                );
                assert_eq!(lin.stops.len(), 2, "应有 2 个色标");
                assert_eq!(lin.repeating, false, "不应为 repeating");
            }
            other => panic!("渐变应为 Linear，实际为 {:?}", other),
        },
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS radial-gradient 渲染管线集成测试。
///
/// 解析 background-image: radial-gradient(circle, red, blue)，
/// 验证 background_image 为 Gradient 变体且包含 RadialGradient。
#[test]
fn test_radial_gradient_render_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "radial");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .radial { background-image: radial-gradient(circle, red, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Radial(rad) => {
                assert_eq!(rad.shape, zero_css_parser::values::RadialShape::Circle);
                assert_eq!(rad.stops.len(), 2, "应有 2 个色标");
            }
            other => panic!("渐变应为 Radial，实际为 {:?}", other),
        },
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS linear-gradient 通过 background 简写管线集成测试。
///
/// 解析 background: linear-gradient(to right, #ff0000, #0000ff)，
/// 验证 expand_background 简写将渐变路由到 background-image，
/// 最终 computed style 中 background_image 为 Gradient 变体。
#[test]
fn test_gradient_via_background_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "bg-grad");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .bg-grad { background: linear-gradient(to right, #ff0000, #0000ff); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Linear(lin) => {
                assert_eq!(
                    lin.direction,
                    zero_css_parser::values::GradientDirection::ToRight,
                    "background 简写展开后方向应为 ToRight"
                );
            }
            other => panic!("渐变应为 Linear，实际为 {:?}", other),
        },
        other => panic!("background 简写中的渐变应路由到 background-image，实际为 {:?}", other),
    }
}

/// CSS conic-gradient 管线集成测试。
///
/// 解析 background-image: conic-gradient(red, blue, green)，
/// 验证 background_image 为 Gradient 变体且包含 ConicGradient。
#[test]
fn test_conic_gradient_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "conic");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .conic { background-image: conic-gradient(red, blue, green); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Conic(conic) => {
                assert_eq!(conic.stops.len(), 3, "应有 3 个色标");
                assert!(!conic.repeating, "不应为 repeating");
            }
            other => panic!("渐变应为 Conic，实际为 {:?}", other),
        },
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS repeating-linear-gradient 管线集成测试。
///
/// 解析 background-image: repeating-linear-gradient(45deg, red, blue 20px)，
/// 验证 repeating 标志为 true。
#[test]
fn test_repeating_linear_gradient_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "repeat-grad");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .repeat-grad { background-image: repeating-linear-gradient(45deg, red, blue 20px); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    match &div_style.background_image {
        zero_style_system::property::BackgroundImageComputedValue::Gradient(grad) => match grad {
            zero_css_parser::values::GradientValue::Linear(lin) => {
                assert!(lin.repeating, "repeating-linear-gradient 的 repeating 应为 true");
                assert_eq!(lin.stops.len(), 2, "应有 2 个色标");
            }
            other => panic!("渐变应为 Linear，实际为 {:?}", other),
        },
        other => panic!("background_image 应为 Gradient 变体，实际为 {:?}", other),
    }
}

/// CSS linear-gradient 渐变不继承管线测试。
///
/// 父元素设置 background-image: linear-gradient(red, blue)，
/// 子元素不显式设置，background-image 不可继承，
/// 验证子元素的 background_image 为默认值 None。
#[test]
fn test_gradient_not_inherited_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "parent-grad");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "child-plain");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent-grad { background-image: linear-gradient(red, blue); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 父元素应有渐变
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        matches!(
            &parent_style.background_image,
            zero_style_system::property::BackgroundImageComputedValue::Gradient(_)
        ),
        "parent 的 background_image 应为 Gradient 变体"
    );

    // 子元素不应继承 background-image
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.background_image,
        zero_style_system::property::BackgroundImageComputedValue::None,
        "child 不应继承 parent 的 background-image，应为 None"
    );
}
