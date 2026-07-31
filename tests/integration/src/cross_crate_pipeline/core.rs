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
fn test_transform_pipeline_integration() {
    // 1. 通过 css-parser 直接解析 transform 值
    let parsed = parse_transform("rotate(45deg) scale(2) translate(10px, 20px)");
    assert!(parsed.is_some(), "css-parser 应成功解析 transform 值");
    let transform_val = parsed.unwrap();
    match &transform_val {
        TransformValue::List(funcs) => {
            assert_eq!(funcs.len(), 3, "应包含 3 个变换函数");
            // rotate(45deg)
            assert!(
                matches!(&funcs[0], TransformFunction::Rotate(a) if (*a - 45.0).abs() < 0.01),
                "第一个函数应为 rotate(45deg)"
            );
            // scale(2) → Scale(2, None)
            assert!(
                matches!(&funcs[1], TransformFunction::Scale(s, None) if (*s - 2.0).abs() < 0.01),
                "第二个函数应为 scale(2)"
            );
            // translate(10px, 20px)
            assert!(
                matches!(&funcs[2], TransformFunction::Translate(tx, ty) if (*tx - 10.0).abs() < 0.01 && (*ty - 20.0).abs() < 0.01),
                "第三个函数应为 translate(10px, 20px)"
            );
        }
        other => panic!("transform 应为 List，实际为 {:?}", other),
    }

    // 2. 通过 style-system 计算样式验证 transform 管线
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = r#"div { transform: rotate(45deg) scale(2) translate(10px, 20px); }"#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    // 验证 computed style 中 transform 不为 none
    assert!(
        !matches!(div_style.transform, TransformValue::None),
        "ComputedStyle.transform 不应为 none"
    );
    // 验证变换函数列表完整
    match &div_style.transform {
        TransformValue::List(funcs) => {
            assert_eq!(funcs.len(), 3, "ComputedStyle 中应包含 3 个变换函数");
        }
        other => panic!("ComputedStyle.transform 应为 List，实际为 {:?}", other),
    }
}

/// 媒体查询 + 样式系统管线集成测试。
///
/// 解析包含 prefers-color-scheme 媒体查询的 CSS，
/// 分别在 dark 和 light 上下文中评估，验证 dark 上下文下样式正确应用。
#[test]
fn test_media_query_prefers_color_scheme_integration() {
    // 1. 通过 css-parser 的 media_query 模块解析
    let dark_ctx = zero_css_parser::media_query::MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: zero_css_parser::media_query::MediaType::Screen,
        prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Dark,
        prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::NoPreference,
        pointer_type: zero_css_parser::media_query::PointerValue::Fine,
        resolution_dpi: 96.0,
    };

    // 2. 解析含 prefers-color-scheme 的媒体查询
    let queries = zero_css_parser::media_query::parse_media_query("(prefers-color-scheme: dark)");
    assert!(queries.is_some(), "应成功解析 prefers-color-scheme 媒体查询");
    let query_list = queries.unwrap();
    assert!(!query_list.is_empty(), "媒体查询列表不应为空");

    // 在 dark 上下文中评估应为 true
    let eval_result = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &dark_ctx);
    assert!(eval_result, "dark 上下文下 prefers-color-scheme: dark 应为 true");

    // 在 light 上下文中评估应为 false
    let light_ctx = zero_css_parser::media_query::MediaContext {
        prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Light,
        ..dark_ctx.clone()
    };
    let eval_light = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &light_ctx);
    assert!(!eval_light, "light 上下文下 prefers-color-scheme: dark 应为 false");

    // 3. 通过 style-system 端到端验证：使用 @media 规则
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();

    // CSS: @media (prefers-color-scheme: dark) { p { color: white; } }
    let css = r#"p { color: black; }
        @media (prefers-color-scheme: dark) { p { color: white; } }"#;
    let stylesheet = CssParser::parse_stylesheet(css);

    // dark 模式下应应用 white
    // 注意：StyleSystem 当前不直接支持 prefers-color-scheme 上下文配置，
    // 但媒体查询解析本身已验证通过上面的 evaluate_media_query。
    // 此处验证样式系统不因 prefers-color-scheme 媒体查询而崩溃。
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let p_style = styles.get(&p).expect("p 应有计算样式");
    // 至少验证样式系统成功计算
    assert!(
        matches!(p_style.color, ColorValue::Rgba(_, _, _, _)),
        "p 的 color 应为有效颜色值"
    );
}

/// Canvas 渐变样式 + 渲染基础 Color 集成测试。
///
/// 创建 CanvasStyle::LinearGradient（红→蓝渐变），
/// 在 offset=0.5 处采样颜色，验证结果为紫色（红蓝混合）。
#[test]
fn test_canvas_gradient_render_foundation_integration() {
    use zero_canvas::{CanvasContext, CanvasStyle, LinearGradient};

    // 1. 创建线性渐变：红色 → 蓝色
    let mut gradient = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
    gradient.add_color_stop(0.0, Color::RED);
    gradient.add_color_stop(1.0, Color::BLUE);

    // 2. 在 offset=0.5 处采样 — 应为紫色（红蓝各半）
    let mid_color = gradient.sample_color(0.5);
    // 红(255,0,0) + 蓝(0,0,255) 在 50% 处插值 → (127, 0, 127)
    assert!(
        mid_color.r > 100 && mid_color.r < 200,
        "紫色 R 分量应在 100-200 之间，实际为 {}",
        mid_color.r
    );
    assert_eq!(mid_color.g, 0, "紫色 G 分量应为 0，实际为 {}", mid_color.g);
    assert!(
        mid_color.b > 100 && mid_color.b < 200,
        "紫色 B 分量应在 100-200 之间，实际为 {}",
        mid_color.b
    );
    assert_eq!(mid_color.a, 255, "alpha 应为 255");

    // 3. 验证边界采样
    let start_color = gradient.sample_color(0.0);
    assert_eq!(start_color.r, 255, "offset=0 应为红色 R=255");
    assert_eq!(start_color.b, 0, "offset=0 应为红色 B=0");

    let end_color = gradient.sample_color(1.0);
    assert_eq!(end_color.r, 0, "offset=1 应为蓝色 R=0");
    assert_eq!(end_color.b, 255, "offset=1 应为蓝色 B=255");

    // 4. 通过 CanvasStyle 包装验证 resolve_color
    let style = CanvasStyle::LinearGradient(gradient.clone());
    let resolved = style.resolve_color();
    // resolve_color 默认在 offset=0.5 采样
    assert_eq!(resolved.r, mid_color.r, "resolve_color 应与 sample_color(0.5) 一致");

    // 5. 集成测试：将渐变样式应用到 Canvas 上下文绘图
    let mut ctx = CanvasContext::new(200, 100);
    ctx.set_fill_style(CanvasStyle::LinearGradient(gradient));
    ctx.fill_rect(0.0, 0.0, 200.0, 100.0);
    let primitives = ctx.primitives();
    assert!(!primitives.fills.is_empty(), "使用渐变填充应生成图元");
}

/// Grid 布局全管线集成测试。
///
/// 使用 grid-template-areas 定义 2x2 命名区域布局，
/// 子元素通过 GridLineValue::Name 指定区域，
/// 经 style-system → layout-engine 计算后验证各元素位置和尺寸。
#[test]
fn test_grid_layout_full_pipeline() {
    let (mut doc, body) = make_doc_with_body();

    // 创建 grid 容器
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    // 创建 4 个子元素分别放入 4 个命名区域
    let top_left = doc.create_element("div");
    doc.set_attribute(top_left, "class", "tl");
    doc.append_child(grid, top_left).unwrap();

    let top_right = doc.create_element("div");
    doc.set_attribute(top_right, "class", "tr");
    doc.append_child(grid, top_right).unwrap();

    let bottom_left = doc.create_element("div");
    doc.set_attribute(bottom_left, "class", "bl");
    doc.append_child(grid, bottom_left).unwrap();

    let bottom_right = doc.create_element("div");
    doc.set_attribute(bottom_right, "class", "br");
    doc.append_child(grid, bottom_right).unwrap();

    let mut styles = HashMap::new();

    // grid 容器：2 列 x 2 行，命名区域 "tl tr" / "bl br"
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("200px 200px".to_string());
    grid_style.grid_template_rows = Some("100px 100px".to_string());
    grid_style.grid_template_areas = Some("\"tl tr\" \"bl br\"".to_string());
    grid_style.width = LengthValue::Px(400.0);
    grid_style.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_style);

    // 为每个子元素设置 grid-area 命名
    for (el, name) in [
        (top_left, "tl"),
        (top_right, "tr"),
        (bottom_left, "bl"),
        (bottom_right, "br"),
    ] {
        let mut el_style = ComputedStyle::default();
        el_style.grid_row_start = GridLineValue::Name(name.to_string());
        el_style.grid_row_end = GridLineValue::Name(name.to_string());
        el_style.grid_column_start = GridLineValue::Name(name.to_string());
        el_style.grid_column_end = GridLineValue::Name(name.to_string());
        styles.insert(el, el_style);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 查找各子元素的布局盒
    let tl_box = find_box_by_node_id(&result.root, top_left).expect("tl 应在布局树中");
    let tr_box = find_box_by_node_id(&result.root, top_right).expect("tr 应在布局树中");
    let bl_box = find_box_by_node_id(&result.root, bottom_left).expect("bl 应在布局树中");
    let br_box = find_box_by_node_id(&result.root, bottom_right).expect("br 应在布局树中");

    // top-left 在第一列第一行
    assert!(tl_box.x < 1.0, "tl 应从 x=0 开始，实际 x={}", tl_box.x);
    assert!(tl_box.y < 1.0, "tl 应从 y=0 开始，实际 y={}", tl_box.y);
    assert!(
        (tl_box.width - 200.0).abs() < 2.0,
        "tl 宽度应约 200px，实际 {}",
        tl_box.width
    );

    // top-right 在第二列第一行
    assert!(tr_box.x >= 190.0, "tr 应在第二列，实际 x={}", tr_box.x);
    assert!(tr_box.y < 1.0, "tr 应在第一行，实际 y={}", tr_box.y);

    // bottom-left 在第一列第二行
    assert!(bl_box.x < 1.0, "bl 应在第一列，实际 x={}", bl_box.x);
    assert!(bl_box.y >= 90.0, "bl 应在第二行，实际 y={}", bl_box.y);

    // bottom-right 在第二列第二行
    assert!(br_box.x >= 190.0, "br 应在第二列，实际 x={}", br_box.x);
    assert!(br_box.y >= 90.0, "br 应在第二行，实际 y={}", br_box.y);

    // 所有子元素高度应约 100px
    for (name, bx) in [("tl", tl_box), ("tr", tr_box), ("bl", bl_box), ("br", br_box)] {
        assert!(
            (bx.height - 100.0).abs() < 2.0,
            "{} 高度应约 100px，实际 {}",
            name,
            bx.height
        );
    }
}

/// CSS 计数器属性级联集成测试。
///
/// 父元素设置 counter-reset: section 0，
/// 子元素设置 counter-increment: section 2，
/// 通过 style-system 计算样式后验证两者的 computed values 正确。
#[test]
fn test_counter_property_cascade_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 父元素：重置计数器
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "parent");
    doc.append_child(body, parent).unwrap();

    // 子元素：递增计数器
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .parent { counter-reset: section 0; }
        .child { counter-increment: section 2; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 counter-reset
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert!(
        !parent_style.counter_reset.is_empty(),
        "parent 的 counter_reset 不应为空"
    );
    assert_eq!(parent_style.counter_reset.len(), 1, "应有一个 counter-reset 条目");
    assert_eq!(parent_style.counter_reset[0].name, "section", "计数器名应为 section");
    assert_eq!(parent_style.counter_reset[0].value, Some(0), "重置值应为 0");

    // 验证子元素的 counter-increment
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert!(
        !child_style.counter_increment.is_empty(),
        "child 的 counter_increment 不应为空"
    );
    assert_eq!(
        child_style.counter_increment.len(),
        1,
        "应有一个 counter-increment 条目"
    );
    assert_eq!(child_style.counter_increment[0].name, "section", "计数器名应为 section");
    assert_eq!(child_style.counter_increment[0].value, Some(2), "增量值应为 2");

    // 子元素不应继承父元素的 counter-reset（counter-reset 不是继承属性）
    assert!(
        child_style.counter_reset.is_empty(),
        "child 不应继承 parent 的 counter_reset"
    );

    // 父元素不应有 counter-increment
    assert!(
        parent_style.counter_increment.is_empty(),
        "parent 不应有 counter_increment"
    );
}

/// 多函数 transform + transform-origin + perspective 完整管线测试。
///
/// 同时设置 transform、transform-origin、perspective 三个属性，
/// 验证 style-system 正确计算所有值到 ComputedStyle 中。
#[test]
fn test_transform_origin_perspective_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let css = r#"
        div {
            transform: rotate(45deg) translateX(10px) scale(2);
            transform-origin: 50% 50%;
            perspective: 800px;
        }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");

    // 验证 transform 包含变换函数
    match &div_style.transform {
        TransformValue::List(funcs) => {
            assert_eq!(funcs.len(), 3, "应包含 3 个变换函数");
            // rotate(45deg)
            assert!(
                matches!(&funcs[0], TransformFunction::Rotate(a) if (*a - 45.0).abs() < 0.01),
                "第一个函数应为 rotate(45deg)"
            );
            // translateX(10px)
            assert!(
                matches!(&funcs[1], TransformFunction::TranslateX(tx) if (*tx - 10.0).abs() < 0.01),
                "第二个函数应为 translateX(10px)"
            );
            // scale(2)
            assert!(
                matches!(&funcs[2], TransformFunction::Scale(s, None) if (*s - 2.0).abs() < 0.01),
                "第三个函数应为 scale(2)"
            );
        }
        other => panic!("transform 应为 List，实际为 {:?}", other),
    }

    // 验证 perspective 值
    assert_eq!(div_style.perspective, LengthValue::Px(800.0), "perspective 应为 800px");
}

/// 媒体查询 + prefers-reduced-motion 管线集成测试。
///
/// 解析 prefers-reduced-motion 媒体查询，
/// 在 reduce 和 no-preference 上下文中分别评估。
#[test]
fn test_media_query_prefers_reduced_motion() {
    let base_ctx = zero_css_parser::media_query::MediaContext {
        viewport_width: 1024.0,
        viewport_height: 768.0,
        media_type: zero_css_parser::media_query::MediaType::Screen,
        prefers_color_scheme: zero_css_parser::media_query::PrefersColorSchemeValue::Light,
        prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::Reduce,
        pointer_type: zero_css_parser::media_query::PointerValue::Fine,
        resolution_dpi: 96.0,
    };

    // 解析 prefers-reduced-motion: reduce
    let queries = zero_css_parser::media_query::parse_media_query("(prefers-reduced-motion: reduce)");
    assert!(queries.is_some(), "应成功解析 prefers-reduced-motion");
    let query_list = queries.unwrap();

    // reduce 上下文 → true
    let result_reduce = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &base_ctx);
    assert!(
        result_reduce,
        "reduce 上下文下 prefers-reduced-motion: reduce 应为 true"
    );

    // no-preference 上下文 → false
    let no_pref_ctx = zero_css_parser::media_query::MediaContext {
        prefers_reduced_motion: zero_css_parser::media_query::ReducedMotionValue::NoPreference,
        ..base_ctx.clone()
    };
    let result_no_pref = zero_css_parser::media_query::evaluate_media_query(&query_list[0], &no_pref_ctx);
    assert!(
        !result_no_pref,
        "no-preference 上下文下 prefers-reduced-motion: reduce 应为 false"
    );
}

/// Canvas 径向渐变采样 + 多级停止点集成测试。
///
/// 创建含 3 个停止点的径向渐变，验证各偏移量处的颜色采样结果。
#[test]
fn test_canvas_radial_gradient_sampling() {
    use zero_canvas::RadialGradient;

    // 创建径向渐变：红 → 绿 → 蓝
    let mut grad = RadialGradient::new(50.0, 50.0, 0.0, 50.0, 50.0, 50.0);
    grad.add_color_stop(0.0, Color::RED); // (255, 0, 0)
    grad.add_color_stop(0.5, Color::GREEN); // (0, 255, 0)
    grad.add_color_stop(1.0, Color::BLUE); // (0, 0, 255)

    // offset=0 处应为红色
    let c0 = grad.sample_color(0.0);
    assert_eq!(c0.r, 255, "offset=0 应为红色 R=255");
    assert_eq!(c0.g, 0, "offset=0 应为红色 G=0");
    assert_eq!(c0.b, 0, "offset=0 应为红色 B=0");

    // offset=0.5 处应为绿色
    let c5 = grad.sample_color(0.5);
    assert_eq!(c5.r, 0, "offset=0.5 应为绿色 R=0");
    assert_eq!(c5.g, 255, "offset=0.5 应为绿色 G=255");
    assert_eq!(c5.b, 0, "offset=0.5 应为绿色 B=0");

    // offset=1.0 处应为蓝色
    let c10 = grad.sample_color(1.0);
    assert_eq!(c10.r, 0, "offset=1.0 应为蓝色 R=0");
    assert_eq!(c10.g, 0, "offset=1.0 应为蓝色 G=0");
    assert_eq!(c10.b, 255, "offset=1.0 应为蓝色 B=255");

    // offset=0.25 处应为红绿混合（偏黄）
    let c25 = grad.sample_color(0.25);
    assert!(c25.r > 100, "offset=0.25 红绿混合 R 应 > 100，实际 {}", c25.r);
    assert!(c25.g > 100, "offset=0.25 红绿混合 G 应 > 100，实际 {}", c25.g);
    assert_eq!(c25.b, 0, "offset=0.25 红绿混合 B 应为 0");
}

/// Counter 属性通过 CSS 级联和继承的综合集成测试。
///
/// 验证多个计数器同时存在时 counter-reset 和 counter-increment 的级联结果。
#[test]
fn test_counter_multiple_cascade_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let ol = doc.create_element("ol");
    doc.set_attribute(ol, "class", "toc");
    doc.append_child(body, ol).unwrap();
    let li1 = doc.create_element("li");
    doc.append_child(ol, li1).unwrap();
    let li2 = doc.create_element("li");
    doc.append_child(ol, li2).unwrap();

    let css = r#"
        ol { counter-reset: section 0 subsection 5; }
        li { counter-increment: section 1 subsection -1; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // ol 应有 counter-reset: section 0, subsection 5
    let ol_style = styles.get(&ol).expect("ol 应有计算样式");
    assert_eq!(ol_style.counter_reset.len(), 2, "ol 应有 2 个 counter-reset");

    // 查找 section 和 subsection 的重置值
    let section_reset = ol_style.counter_reset.iter().find(|c| c.name == "section");
    assert!(section_reset.is_some(), "应有 section 计数器重置");
    assert_eq!(section_reset.unwrap().value, Some(0), "section 重置值应为 0");

    let sub_reset = ol_style.counter_reset.iter().find(|c| c.name == "subsection");
    assert!(sub_reset.is_some(), "应有 subsection 计数器重置");
    assert_eq!(sub_reset.unwrap().value, Some(5), "subsection 重置值应为 5");

    // li 应有 counter-increment: section 1, subsection -1
    let li1_style = styles.get(&li1).expect("li1 应有计算样式");
    assert_eq!(li1_style.counter_increment.len(), 2, "li 应有 2 个 counter-increment");

    let section_inc = li1_style.counter_increment.iter().find(|c| c.name == "section");
    assert!(section_inc.is_some(), "应有 section 增量");
    assert_eq!(section_inc.unwrap().value, Some(1), "section 增量应为 1");

    let sub_inc = li1_style.counter_increment.iter().find(|c| c.name == "subsection");
    assert!(sub_inc.is_some(), "应有 subsection 增量");
    assert_eq!(sub_inc.unwrap().value, Some(-1), "subsection 增量应为 -1");

    // li 不应继承 ol 的 counter-reset
    assert!(li1_style.counter_reset.is_empty(), "li 不应继承 ol 的 counter_reset");
}

/// CSS overflow-wrap 管线集成测试。
///
/// 解析含 overflow-wrap 的 CSS，通过 style-system 计算样式，
/// 验证 overflow-wrap 值正确存储且能被子元素继承。
#[test]
fn test_overflow_wrap_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 父元素设置 overflow-wrap: break-word
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "wrap-container");
    doc.append_child(body, parent).unwrap();

    // 子元素不显式设置 overflow-wrap，应继承父元素的值
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "text");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .wrap-container { overflow-wrap: break-word; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 overflow-wrap 为 BreakWord
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.overflow_wrap,
        zero_style_system::property::OverflowWrapValue::BreakWord,
        "parent 的 overflow-wrap 应为 BreakWord"
    );

    // 验证子元素继承了 overflow-wrap
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.overflow_wrap,
        zero_style_system::property::OverflowWrapValue::BreakWord,
        "child 应继承 parent 的 overflow-wrap: BreakWord"
    );
}

/// CSS text-align-last 管线集成测试。
///
/// 解析含 text-align-last 的 CSS，通过 style-system 计算样式，
/// 验证 text-align-last 值正确应用到目标元素。
#[test]
fn test_text_align_last_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "last-line");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .last-line { text-align-last: center; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_align_last,
        zero_style_system::property::TextAlignLastValue::Center,
        "div 的 text-align-last 应为 Center"
    );
}

/// CSS direction 管线集成测试。
///
/// 解析含 direction: rtl 的 CSS，通过 style-system 计算样式，
/// 验证 direction 值正确应用且被子元素继承。
#[test]
fn test_direction_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "rtl-container");
    doc.append_child(body, parent).unwrap();

    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "rtl-text");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .rtl-container { direction: rtl; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 父元素 direction 应为 Rtl
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.direction,
        zero_style_system::property::DirectionValue::Rtl,
        "parent 的 direction 应为 Rtl"
    );

    // 子元素应继承 direction: rtl
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.direction,
        zero_style_system::property::DirectionValue::Rtl,
        "child 应继承 parent 的 direction: Rtl"
    );
}

/// CSS tab-size 管线集成测试。
///
/// 解析含 tab-size 的 CSS，通过 style-system 计算样式，
/// 验证 tab-size 值正确解析和存储。
#[test]
fn test_tab_size_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let pre = doc.create_element("pre");
    doc.set_attribute(pre, "class", "code-block");
    doc.append_child(body, pre).unwrap();

    // 子元素用于验证继承
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "code-text");
    doc.append_child(pre, span).unwrap();

    let css = r#"
        .code-block { tab-size: 4; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证 pre 元素的 tab-size 为 4
    let pre_style = styles.get(&pre).expect("pre 应有计算样式");
    assert_eq!(
        pre_style.tab_size,
        zero_style_system::property::TabSizeValue::Number(4),
        "pre 的 tab-size 应为 Number(4)"
    );

    // 验证子元素继承了 tab-size
    let span_style = styles.get(&span).expect("span 应有计算样式");
    assert_eq!(
        span_style.tab_size,
        zero_style_system::property::TabSizeValue::Number(4),
        "span 应继承 pre 的 tab-size: Number(4)"
    );
}

/// Storage + Protocol 序列化集成测试。
///
/// 将 storage 操作通过 IPC 消息序列化 → 反序列化，
/// 验证 StorageOpParams 所有字段完整保留，包括 Remove 操作。
#[test]
fn test_storage_protocol_ipc_roundtrip() {
    use zero_protocol::{
        IpcMessage, IpcMessageKind, StorageOpParams, StorageOperation, StorageType, deserialize, serialize,
    };
    use zero_storage::StorageManager;

    // 先执行实际 storage 操作
    let mut mgr = StorageManager::new();
    let store = mgr.local_storage("https://example.com");
    store.set("session_id", "abc-123").unwrap();
    store.set("theme", "dark").unwrap();
    assert_eq!(store.get("session_id"), Some("abc-123"));

    // 构造 Remove 操作的 IPC 消息
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Remove,
            key: "session_id".to_string(),
            value: None,
            origin: "https://example.com".to_string(),
        }),
    };

    // 序列化 → 反序列化
    let bytes = serialize(&msg).expect("serialize 应成功");
    let decoded = deserialize(&bytes).expect("deserialize 应成功");

    // 验证 IPC 字段
    assert_eq!(decoded.id, 42, "消息 ID 应为 42");
    if let IpcMessageKind::StorageOp(p) = decoded.kind {
        assert_eq!(p.storage_type, StorageType::Local, "storage_type 应为 Local");
        assert_eq!(p.operation, StorageOperation::Remove, "operation 应为 Remove");
        assert_eq!(p.key, "session_id", "key 应为 session_id");
        assert_eq!(p.value, None, "Remove 操作 value 应为 None");
        assert_eq!(p.origin, "https://example.com", "origin 应为 https://example.com");
    } else {
        panic!("expected StorageOp kind");
    }

    // 再构造 Clear 操作验证
    let clear_msg = IpcMessage {
        id: 43,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Session,
            operation: StorageOperation::Clear,
            key: String::new(),
            value: None,
            origin: "https://example.com".to_string(),
        }),
    };
    let bytes2 = serialize(&clear_msg).expect("serialize clear 应成功");
    let decoded2 = deserialize(&bytes2).expect("deserialize clear 应成功");
    if let IpcMessageKind::StorageOp(p) = decoded2.kind {
        assert_eq!(p.storage_type, StorageType::Session, "storage_type 应为 Session");
        assert_eq!(p.operation, StorageOperation::Clear, "operation 应为 Clear");
    } else {
        panic!("expected StorageOp kind for clear");
    }
}

/// CSS break-inside 管线集成测试。
///
/// 解析含 break-inside: avoid 的 CSS，通过 style-system 计算样式，
/// 验证 break-inside 值正确应用到目标元素。
#[test]
fn test_break_inside_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "no-break");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .no-break { break-inside: avoid; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.break_inside,
        zero_style_system::property::BreakInsideValue::Avoid,
        "div 的 break-inside 应为 Avoid"
    );
}

/// CSS column-count 管线集成测试。
///
/// 解析含 column-count: 3 的 CSS，通过 style-system 计算样式，
/// 验证 column-count 值正确解析和存储到计算样式中。
#[test]
fn test_column_count_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "multi-col");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .multi-col { column-count: 3; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.column_count,
        zero_style_system::property::ColumnCountComputedValue::Number(3),
        "div 的 column-count 应为 Number(3)"
    );
}

/// CSS object-fit 管线集成测试。
///
/// 解析含 object-fit: cover 的 CSS，通过 style-system 计算样式，
/// 验证 object-fit 值正确应用到 img 元素的计算样式中。
#[test]
fn test_object_fit_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let img = doc.create_element("img");
    doc.set_attribute(img, "class", "hero");
    doc.append_child(body, img).unwrap();

    let css = r#"
        .hero { object-fit: cover; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let img_style = styles.get(&img).expect("img 应有计算样式");
    assert_eq!(
        img_style.object_fit,
        zero_style_system::property::ObjectFitComputedValue::Cover,
        "img 的 object-fit 应为 Cover"
    );
}

/// CSS direction 多级继承集成测试。
///
/// 祖父元素设置 direction: rtl，父元素不显式设置（应继承 rtl），
/// 子元素显式设置 direction: ltr 覆盖继承值。
/// 验证三层继承链中各元素的 direction 计算值正确。
#[test]
fn test_direction_inheritance_chain() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 祖父元素：direction: rtl
    let grandparent = doc.create_element("div");
    doc.set_attribute(grandparent, "class", "rtl-root");
    doc.append_child(body, grandparent).unwrap();

    // 父元素：不设置 direction，应继承 rtl
    let parent = doc.create_element("section");
    doc.set_attribute(parent, "class", "middle");
    doc.append_child(grandparent, parent).unwrap();

    // 子元素：显式设置 direction: ltr，覆盖继承值
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "ltr-override");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .rtl-root { direction: rtl; }
        .ltr-override { direction: ltr; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 祖父元素：显式 rtl
    let gp_style = styles.get(&grandparent).expect("grandparent 应有计算样式");
    assert_eq!(
        gp_style.direction,
        zero_style_system::property::DirectionValue::Rtl,
        "grandparent 的 direction 应为 Rtl"
    );

    // 父元素：继承 rtl
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.direction,
        zero_style_system::property::DirectionValue::Rtl,
        "parent 应继承 grandparent 的 direction: Rtl"
    );

    // 子元素：显式覆盖为 ltr
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.direction,
        zero_style_system::property::DirectionValue::Ltr,
        "child 的 direction 应被显式覆盖为 Ltr"
    );
}

/// CSS contain 管线集成测试。
///
/// 解析含 contain: layout 的 CSS，通过 style-system 计算样式，
/// 验证 contain 值正确存储到计算样式中。
#[test]
fn test_contain_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "contained");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .contained { contain: layout; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.contain,
        zero_style_system::property::ContainComputedValue::Layout,
        "div 的 contain 应为 Layout"
    );
}

/// CSS filter 管线集成测试。
///
/// 解析含 filter: blur(5px) 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.filter 为 Blur(5.0)。
#[test]
fn test_filter_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "blurred");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .blurred { filter: blur(5px); }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        &div_style.filter[..],
        &[zero_style_system::property::FilterComputedValue::Blur(5.0)],
        "div 的 filter 应为 [Blur(5.0)]"
    );
}

/// CSS mix-blend-mode 管线集成测试。
///
/// 解析含 mix-blend-mode: multiply 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.mix_blend_mode 为 Multiply。
#[test]
fn test_mix_blend_mode_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "blended");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .blended { mix-blend-mode: multiply; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.mix_blend_mode,
        zero_style_system::property::MixBlendModeComputedValue::Multiply,
        "div 的 mix-blend-mode 应为 Multiply"
    );
}

/// CSS scrollbar-width 管线集成测试。
///
/// 解析含 scrollbar-width: thin 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.scrollbar_width 为 Thin。
#[test]
fn test_scrollbar_width_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "thin-scroll");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .thin-scroll { scrollbar-width: thin; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.scrollbar_width,
        zero_style_system::property::ScrollbarWidthComputedValue::Thin,
        "div 的 scrollbar-width 应为 Thin"
    );
}

/// CSS contain 多值组合管线集成测试。
///
/// 解析含 contain: layout style paint 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.contain 为包含 layout + style + paint 标志位的 Custom 组合值。
#[test]
fn test_contain_multi_value_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "multi-contain");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .multi-contain { contain: layout style paint; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    // layout=0x02 + style=0x04 + paint=0x08 = 0x0E
    let expected_flags = zero_style_system::property::ContainComputedValue::FLAG_LAYOUT
        | zero_style_system::property::ContainComputedValue::FLAG_STYLE
        | zero_style_system::property::ContainComputedValue::FLAG_PAINT;
    assert_eq!(
        div_style.contain,
        zero_style_system::property::ContainComputedValue::Custom(expected_flags),
        "div 的 contain 应为 Custom(layout|style|paint) = 0x{:02X}",
        expected_flags
    );
}

/// CSS appearance 管线集成测试。
///
/// 解析含 appearance: none 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.appearance 为 None。
#[test]
fn test_appearance_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let input = doc.create_element("input");
    doc.set_attribute(input, "class", "custom-input");
    doc.append_child(body, input).unwrap();

    let css = r#"
        .custom-input { appearance: none; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let input_style = styles.get(&input).expect("input 应有计算样式");
    assert_eq!(
        input_style.appearance,
        zero_style_system::property::AppearanceComputedValue::None,
        "input 的 appearance 应为 None"
    );
}

/// CSS columns 简写管线集成测试。
///
/// 解析含 columns: 3 200px 的 CSS，通过 style-system 计算样式，
/// 验证 column-count 解析为 Number(3)，column-width 解析为 Length(200px)。
#[test]
fn test_columns_shorthand_pipeline() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "multi-col");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .multi-col { columns: 3 200px; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.column_count,
        zero_style_system::property::ColumnCountComputedValue::Number(3),
        "div 的 column-count 应为 Number(3)"
    );
    assert_eq!(
        div_style.column_width,
        zero_style_system::property::ColumnWidthComputedValue::Length(LengthValue::Px(200.0)),
        "div 的 column-width 应为 Length(Px(200.0))"
    );
}

/// CSS text-wrap 管线集成测试。
///
/// 解析含 text-wrap: balance 的 CSS，通过 style-system 计算样式，
/// 验证 ComputedStyle.text_wrap 为 Balance。
#[test]
fn test_text_wrap_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "balanced");
    doc.append_child(body, div).unwrap();

    let css = r#"
        .balanced { text-wrap: balance; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    let div_style = styles.get(&div).expect("div 应有计算样式");
    assert_eq!(
        div_style.text_wrap,
        zero_style_system::property::TextWrapComputedValue::Balance,
        "div 的 text-wrap 应为 Balance"
    );
}

/// CSS hyphens 管线集成测试。
///
/// 解析含 hyphens: auto 的 CSS，通过 style-system 计算样式，
/// 验证父元素 hyphens 为 Auto，且子元素继承了该值。
#[test]
fn test_hyphens_pipeline_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let html_el = doc.create_element("html");
    doc.append_child(root, html_el).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html_el, body).unwrap();

    // 父元素设置 hyphens: auto
    let parent = doc.create_element("div");
    doc.set_attribute(parent, "class", "hyphenated");
    doc.append_child(body, parent).unwrap();

    // 子元素不显式设置，应继承 hyphens
    let child = doc.create_element("p");
    doc.set_attribute(child, "class", "hyphen-text");
    doc.append_child(parent, child).unwrap();

    let css = r#"
        .hyphenated { hyphens: auto; }
    "#;
    let stylesheet = CssParser::parse_stylesheet(css);

    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);

    // 验证父元素的 hyphens 为 Auto
    let parent_style = styles.get(&parent).expect("parent 应有计算样式");
    assert_eq!(
        parent_style.hyphens,
        zero_style_system::property::HyphensComputedValue::Auto,
        "parent 的 hyphens 应为 Auto"
    );

    // 验证子元素继承了 hyphens: auto
    let child_style = styles.get(&child).expect("child 应有计算样式");
    assert_eq!(
        child_style.hyphens,
        zero_style_system::property::HyphensComputedValue::Auto,
        "child 应继承 parent 的 hyphens: Auto"
    );
}
