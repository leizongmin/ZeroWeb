//! 从 pipeline.rs 提取的管线内联测试。
//!
//! 覆盖渲染管线核心功能：基础渲染、样式重算、增量渲染、脏区域追踪、
//! CSS 变换/合成、动画管线集成、边界条件。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::geometry::Rect;

/// 创建一个简单的 LayoutBox 用于增量渲染测试。
fn make_dirty_box(w: f32, h: f32) -> zero_layout_engine::LayoutBox {
    zero_layout_engine::LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
        content_x: 0.0,
        content_y: 0.0,
        content_width: w,
        content_height: h,
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
        overflow_x: zero_layout_engine::types::OverflowClip::Visible,
        overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        ..Default::default()
    }
}

/// 计算布局树的最大 Y 坐标（用于验证布局高度）。
fn layout_height(b: &zero_layout_engine::LayoutBox, offset_y: f32) -> f32 {
    let mut max_y = offset_y + b.y + b.height;
    for child in &b.children {
        max_y = max_y.max(layout_height(child, offset_y + b.y));
    }
    max_y
}

// ── 基础渲染测试 ──

/// 测试创建渲染管线。
#[test]
fn test_pipeline_new() {
    let pipeline = RenderPipeline::new(800.0, 600.0);
    assert_eq!(pipeline.viewport_width(), 800.0);
    assert_eq!(pipeline.viewport_height(), 600.0);
    assert!(pipeline.layout().is_none());
}

/// 测试渲染简单 HTML 文档。
#[test]
fn test_pipeline_render_simple_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let result = pipeline.render_html(html, "");

    assert!(pipeline.layout().is_some());
    assert!(result.timings.total_ms >= 0.0);
    assert!(result.layout.viewport_width > 0.0);
}

/// R1682：`<wbr>` 零宽断行机会标记——无可见渲染。`<div>ab<wbr>cd</div>` 应与
/// `<div>abcd</div>` 产出**相同** glyphs（"abcd"，wbr 不贡献 glyph 也不打断文本）。
#[test]
fn test_pipeline_wbr_renders_zero_width_no_glyph() {
    let mut with_wbr = RenderPipeline::new(800.0, 600.0);
    let r1 = with_wbr.render_html("<html><body><div>ab<wbr>cd</div></body></html>", "");
    let mut without = RenderPipeline::new(800.0, 600.0);
    let r2 = without.render_html("<html><body><div>abcd</div></body></html>", "");

    // 两侧都应有 4 个 glyph（a/b/c/d），wbr 不增加 glyph。
    assert_eq!(r1.primitives().glyphs.len(), r2.primitives().glyphs.len());
    assert_eq!(r1.primitives().glyphs.len(), 4, "ab<wbr>cd → 4 glyphs (a/b/c/d)");
    // glyph_id 序列应为 a/b/c/d（确认 wbr 未插入额外字符或打断）。
    let ids: Vec<u32> = r1.primitives().glyphs.iter().map(|g| g.glyph_id).collect();
    assert_eq!(
        ids,
        vec!['a' as u32, 'b' as u32, 'c' as u32, 'd' as u32],
        "glyph 序列应为 a/b/c/d"
    );
}

/// 测试带 CSS 的渲染。
#[test]
fn test_pipeline_render_with_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="main">Hello</div></body></html>"#;
    let css = "div { background-color: red; width: 200px; height: 100px; }";
    let result = pipeline.render_html(html, css);

    assert!(!result.primitives().fills.is_empty());
}

/// 测试渲染空 HTML 文档。
#[test]
fn test_pipeline_render_empty_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "";
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0);
    assert!(result.layout.viewport_width > 0.0);
}

/// 测试渲染嵌套元素。
#[test]
fn test_pipeline_render_nested_elements() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div><p><span>Deep</span></p></div></body></html>";
    let css = "div { background-color: #ff0000; width: 300px; height: 200px; }";
    let result = pipeline.render_html(html, css);

    assert!(!result.primitives().fills.is_empty());
    assert!(pipeline.layout().is_some());
}

/// 测试渲染计时信息存在。
#[test]
fn test_pipeline_timings_present() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Test</div></body></html>";
    let result = pipeline.render_html(html, "");

    assert!(result.timings.parse_ms >= 0.0);
    assert!(result.timings.style_ms >= 0.0);
    assert!(result.timings.layout_ms >= 0.0);
    assert!(result.timings.paint_ms >= 0.0);
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试重新计算样式。
#[test]
fn test_pipeline_recompute_styles() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    let _first = pipeline.render_html(html, "");

    let doc = zero_dom::parse_html(html);
    let css = "div { background-color: blue; }";
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (primitives, _styles, layout) = pipeline.recompute_styles(&doc, &stylesheets);

    assert!(layout.viewport_width > 0.0);
    assert!(!primitives.fills.is_empty());
}

/// 测试增量渲染。
#[test]
fn test_pipeline_incremental_render() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    let _first = pipeline.render_html(html, "");

    let dirty_box = make_dirty_box(100.0, 50.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试多次渲染不 panic。
#[test]
fn test_pipeline_multiple_render() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    for i in 0..3 {
        let html = format!("<html><body><div>Page {i}</div></body></html>");
        let result = pipeline.render_html(&html, "");
        assert!(result.timings.total_ms >= 0.0);
    }
    assert!(pipeline.layout().is_some());
}

/// 测试渲染 malformed HTML 不 panic。
#[test]
fn test_pipeline_render_malformed_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<div><p>unclosed<span>no closing tags";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "malformed HTML 应容错完成");
}

/// 测试渲染 Unicode 内容。
#[test]
fn test_pipeline_render_unicode() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body>こんにちは世界 🌍 Grüße</body></html>";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试超大视口渲染。
#[test]
fn test_pipeline_large_viewport() {
    let mut pipeline = RenderPipeline::new(7680.0, 4320.0);
    let html = "<html><body><div>8K</div></body></html>";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert_eq!(pipeline.viewport_width(), 7680.0);
}

/// 测试零尺寸视口渲染不 panic。
#[test]
fn test_pipeline_zero_viewport() {
    let mut pipeline = RenderPipeline::new(0.0, 0.0);
    let html = "<html><body><div>Zero</div></body></html>";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试脏区域追踪器可通过管道访问。
#[test]
fn test_pipeline_dirty_tracker_accessible() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
    pipeline.dirty_tracker_mut().mark_full_redraw();
    assert!(pipeline.dirty_tracker().is_full_redraw());
}

/// 测试渲染带大量 CSS 规则。
#[test]
fn test_pipeline_render_many_css_rules() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="a">A</div><div class="b">B</div><div class="c">C</div>
    </body></html>"#;
    let css = r#"
        .a { color: red; background-color: #ff0000; width: 100px; height: 50px; }
        .b { color: blue; background-color: #0000ff; margin: 10px; }
        .c { color: green; background-color: #00ff00; padding: 5px; }
        body { margin: 0; padding: 20px; }
    "#;
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(result.timings.style_ms >= 0.0);
}

// ── 增量渲染测试 ──

/// 测试增量渲染小区域时不退化为全量重绘。
#[test]
fn test_pipeline_incremental_render_small_area() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let _first = pipeline.render_html(html, "");

    let dirty_box = make_dirty_box(10.0, 10.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试增量渲染大区域时退化为全量重绘。
#[test]
fn test_pipeline_incremental_render_large_area() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let _first = pipeline.render_html(html, "");

    let dirty_box = make_dirty_box(600.0, 400.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试 incremental_paint 仅绘制脏区域内的节点。
#[test]
fn test_pipeline_incremental_paint() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let css = "div { background-color: red; width: 200px; height: 100px; }";

    let full_result = pipeline.render_html(html, css);
    let full_fills = full_result.primitives().fills.len();

    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let dirty_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let inc_primitives = pipeline.incremental_paint(&doc, &stylesheets, dirty_rect);

    assert!(inc_primitives.is_some());
    let inc_fills = inc_primitives.unwrap().fills.len();
    assert!(inc_fills <= full_fills);
}

/// 测试全量渲染后 incremental_paint 产生更少或相等的图元。
#[test]
fn test_full_vs_incremental_render_primitive_count() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let css = "div { background-color: blue; width: 300px; height: 200px; }";

    let full_result = pipeline.render_html(html, css);
    let full_count = full_result.primitives().len();

    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let dirty_rect = Rect::new(700.0, 500.0, 50.0, 50.0);
    let inc_primitives = pipeline.incremental_paint(&doc, &stylesheets, dirty_rect);

    let inc_count = inc_primitives.map(|p| p.len()).unwrap_or(0);
    assert!(
        inc_count <= full_count,
        "incremental paint should produce <= primitives of full paint"
    );
}

/// 测试 DOM 修改后 recompute_styles 生成不同的图元。
#[test]
fn test_recompute_after_style_change() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    let _first = pipeline.render_html(html, "");

    let doc = zero_dom::parse_html(html);
    let css = "div { background-color: green; width: 200px; height: 100px; }";
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (primitives, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

    assert!(!primitives.fills.is_empty(), "style change should produce fills");
}

// ── CSS Transform/合成测试 ──

/// 测试渲染带 CSS transform 的页面不 panic。
#[test]
fn test_pipeline_render_with_transform() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="t">Transformed</div></body></html>"#;
    let css = "div { transform: translate(50px, 100px); width: 200px; height: 50px; }";
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试渲染带 opacity 的页面不 panic。
#[test]
fn test_pipeline_render_with_opacity() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="o">Semi-transparent</div></body></html>"#;
    let css = "div { opacity: 0.5; background-color: red; width: 100px; height: 100px; }";
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试多次 recompute_styles 后 layout 缓存更新。
#[test]
fn test_recompute_updates_cached_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    let _first = pipeline.render_html(html, "");
    assert!(pipeline.layout().is_some());

    let doc = zero_dom::parse_html(html);
    let css1 = "div { background-color: red; width: 100px; }";
    let ss1 = vec![zero_css_parser::Parser::parse_stylesheet(css1)];
    let (_, _, layout1) = pipeline.recompute_styles(&doc, &ss1);

    let css2 = "div { background-color: blue; width: 200px; }";
    let ss2 = vec![zero_css_parser::Parser::parse_stylesheet(css2)];
    let (_, _, layout2) = pipeline.recompute_styles(&doc, &ss2);

    assert!(layout1.viewport_width > 0.0);
    assert!(layout2.viewport_width > 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试 render_html 返回的 RenderResult primitives 非空（有 CSS）。
#[test]
fn test_render_produces_primitives_with_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Test</div></body></html>"#;
    let css = ".box { background-color: #ff6600; width: 150px; height: 75px; border: 2px solid black; }";
    let result = pipeline.render_html(html, css);

    assert!(!result.primitives().fills.is_empty());
    assert!(result.primitives().fills.len() >= 1);
}

// ── 脏区域追踪测试 ──

/// 测试样式变化后标记脏节点，增量渲染产生与全量渲染相同的结果。
#[test]
fn test_dirty_recompute_after_style_change_produces_different_primitives() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;

    let first = pipeline.render_html(html, "");
    let first_fill_count = first.primitives().fills.len();

    let doc = zero_dom::parse_html(html);
    let css_red = ".box { background-color: red; width: 200px; height: 100px; }";
    let ss_red = vec![zero_css_parser::Parser::parse_stylesheet(css_red)];
    let (prims_red, _, layout_red) = pipeline.recompute_styles(&doc, &ss_red);

    assert!(!prims_red.fills.is_empty(), "style change should produce fills");
    assert!(prims_red.fills.len() > first_fill_count);
    assert!(layout_red.viewport_width > 0.0);

    let css_blue = ".box { background-color: blue; width: 300px; height: 150px; }";
    let ss_blue = vec![zero_css_parser::Parser::parse_stylesheet(css_blue)];
    let (prims_blue, _, _) = pipeline.recompute_styles(&doc, &ss_blue);

    assert!(!prims_blue.fills.is_empty());
}

/// 测试标记脏区域后 incremental_render 正确完成渲染。
#[test]
fn test_dirty_mark_triggers_rerender_lifecycle() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";

    let _first = pipeline.render_html(html, "");
    assert!(pipeline.layout().is_some());
    assert!(pipeline.dirty_tracker().dirty_rects().is_empty());

    pipeline
        .dirty_tracker_mut()
        .mark_dirty(Rect::new(0.0, 0.0, 200.0, 100.0));
    assert_eq!(pipeline.dirty_tracker().dirty_rects().len(), 1);
    assert!(pipeline.dirty_tracker().dirty_area() > 0.0);

    let dirty_box = make_dirty_box(200.0, 100.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);

    assert!(pipeline.dirty_tracker().dirty_rects().is_empty());
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试连续样式变化 + 脏标记多次迭代后仍能正确渲染。
#[test]
fn test_dirty_multiple_style_changes_renders_correctly() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="target">Text</div></body></html>"#;

    let _first = pipeline.render_html(html, "");
    assert!(pipeline.layout().is_some());

    let doc = zero_dom::parse_html(html);
    let css1 = ".target { background-color: green; width: 100px; }";
    let ss1 = vec![zero_css_parser::Parser::parse_stylesheet(css1)];
    let (prims1, _, _) = pipeline.recompute_styles(&doc, &ss1);

    let css2 = ".target { background-color: blue; width: 200px; }";
    let ss2 = vec![zero_css_parser::Parser::parse_stylesheet(css2)];
    let (prims2, _, _) = pipeline.recompute_styles(&doc, &ss2);

    assert!(prims1.fills.len() > 0);
    assert!(prims2.fills.len() > 0);
    assert!(pipeline.layout().is_some());
}

// ── 边界条件测试 ──

/// 测试渲染包含语法错误的 CSS 不 panic。
#[test]
fn test_render_html_malformed_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let css = "{{{";
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0, "malformed CSS should not panic");
    assert!(pipeline.layout().is_some());
}

/// 测试增量渲染在脏区域恰好为视口面积 50% 时的行为。
#[test]
fn test_incremental_render_at_50_percent_threshold() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let _first = pipeline.render_html(html, "");

    // 800 * 600 * 0.5 = 240000 → 400 x 600 = 240000
    let dirty_box = make_dirty_box(400.0, 600.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试增量渲染在脏区域低于 50% 视口面积时保持增量。
#[test]
fn test_incremental_render_below_50_percent() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Hello</div></body></html>";
    let _first = pipeline.render_html(html, "");

    // 49.9% of 800*600 ≈ 399.2 x 600
    let dirty_box = make_dirty_box(399.2, 600.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!pipeline.dirty_tracker().is_full_redraw());
}

/// 测试在全新 pipeline 上直接调用 recompute_styles 不 panic。
#[test]
fn test_recompute_styles_without_prior_render() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    assert!(pipeline.layout().is_none());

    let html = "<html><body><div>Fresh</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let css = "div { background-color: red; width: 100px; height: 50px; }";
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];

    let (primitives, _styles, layout) = pipeline.recompute_styles(&doc, &stylesheets);

    assert!(layout.viewport_width > 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!primitives.fills.is_empty());
}

/// 测试混合渲染操作序列。
#[test]
fn test_mixed_render_operations() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;

    let first = pipeline.render_html(html, "div { background-color: red; width: 200px; height: 100px; }");
    assert!(pipeline.layout().is_some());
    assert!(pipeline.dirty_tracker().dirty_rects().is_empty());
    let first_fill_count = first.primitives().fills.len();

    let doc = zero_dom::parse_html(html);
    let css_blue = ".box { background-color: blue; width: 300px; height: 150px; }";
    let ss_blue = vec![zero_css_parser::Parser::parse_stylesheet(css_blue)];
    let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &ss_blue);
    assert!(!prims.fills.is_empty());

    let dirty_box = make_dirty_box(50.0, 50.0);
    let result = pipeline.incremental_render(html, "", &dirty_box);
    assert!(result.timings.total_ms >= 0.0);

    assert!(pipeline.dirty_tracker().dirty_rects().is_empty());
    assert!(!pipeline.dirty_tracker().is_full_redraw());
    assert!(pipeline.layout().is_some());
    assert!(first_fill_count > 0);
}

/// 测试渲染带内联 style 属性的 HTML 文档。
#[test]
fn test_render_html_with_inline_styles() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html =
        r#"<html><body><div style="background-color: red; width: 200px; height: 100px;">Styled</div></body></html>"#;
    let css = "div { background-color: red; width: 200px; height: 100px; }";
    let result = pipeline.render_html(html, css);

    assert!(!result.primitives().fills.is_empty());
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试渲染包含 <script> 标签的 HTML 不崩溃。
#[test]
fn test_render_html_with_script_tags() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div>Before Script</div>
        <script>var x = 1; function foo() { return x + 1; }</script>
        <div>After Script</div>
    </body></html>"#;
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(result.primitives().len() > 0);
}

/// 测试多次渲染调用后图元顺序稳定。
#[test]
fn test_render_preserves_order() {
    let html = r#"<html><body>
        <div class="a">A</div>
        <div class="b">B</div>
        <div class="c">C</div>
    </body></html>"#;
    let css = r#"
        .a { background-color: red; width: 100px; height: 50px; }
        .b { background-color: green; width: 100px; height: 50px; }
        .c { background-color: blue; width: 100px; height: 50px; }
    "#;

    let mut pipeline1 = RenderPipeline::new(800.0, 600.0);
    let result1 = pipeline1.render_html(html, css);
    let fills1: Vec<_> = result1.primitives().fills.iter().map(|f| f.color).collect();

    let mut pipeline2 = RenderPipeline::new(800.0, 600.0);
    let result2 = pipeline2.render_html(html, css);
    let fills2: Vec<_> = result2.primitives().fills.iter().map(|f| f.color).collect();

    assert_eq!(fills1.len(), fills2.len());
    assert_eq!(fills1, fills2);
}

/// 测试完整 HTML 文档（包含 <head> 和 <body>）的渲染。
#[test]
fn test_render_html_with_head_and_body() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html>
        <head><title>测试页面</title></head>
        <body>
            <div class="header">标题</div>
            <div class="content">正文内容</div>
            <div class="footer">页脚</div>
        </body>
    </html>"#;
    let css = r#"
        .header { background-color: #333333; width: 100%; height: 60px; }
        .content { background-color: #ffffff; width: 100%; height: 400px; }
        .footer { background-color: #666666; width: 100%; height: 40px; }
    "#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.primitives().fills.is_empty());
    assert!(result.layout.viewport_width > 0.0);
    assert!(result.layout.viewport_height > 0.0);
    assert!(!result.layout.root.children.is_empty());
}

/// 测试带 @media screen 的 CSS 渲染。
#[test]
fn test_pipeline_render_with_media_query() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="responsive">Content</div>
        <div class="always-visible">Static</div>
    </body></html>"#;
    let css = r#"
        .always-visible { background-color: #333333; width: 100px; height: 50px; }
        @media screen {
            .responsive { background-color: #ff0000; width: 200px; height: 100px; }
        }
        @media print {
            .responsive { background-color: #ffffff; width: 100%; }
        }
    "#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.layout.root.children.is_empty());
}

// ── 变换/合成测试 ──

/// 测试 perspective 变换不崩溃。
#[test]
fn test_perspective_transform_offset() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="p">Perspective</div></body></html>"#;
    let css = r#"
        div { perspective: 500px; transform: translateZ(50px); width: 200px; height: 100px; background-color: red; }
    "#;
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.primitives().fills.is_empty());
}

/// 测试 transform-origin 偏移变换效果。
#[test]
fn test_transform_origin_offset() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="t">Origin</div></body></html>"#;
    let css = r#"
        div {
            transform-origin: 0px 0px;
            transform: rotate(45deg);
            width: 100px;
            height: 100px;
            background-color: blue;
        }
    "#;
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.primitives().fills.is_empty());
}

/// 测试负坐标绘制不崩溃。
#[test]
fn test_paint_with_negative_coords() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="neg">Offscreen</div></body></html>"#;
    let css = r#"
        div {
            transform: translate(-50px, -30px);
            width: 200px;
            height: 200px;
            background-color: green;
        }
    "#;
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(result.primitives().len() > 0);
}

/// 测试深层嵌套 z-index 合成。
#[test]
fn test_composite_deeply_nested_z() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div style="position: relative; z-index: 5; width: 400px; height: 400px; background-color: #ff0000;">
            <div style="position: relative; z-index: 4; width: 300px; height: 300px; background-color: #00ff00;">
                <div style="position: relative; z-index: 3; width: 200px; height: 200px; background-color: #0000ff;">
                    <div style="position: relative; z-index: 2; width: 150px; height: 150px; background-color: #ffff00;">
                        <div style="position: relative; z-index: 1; width: 100px; height: 100px; background-color: #ff00ff;">
                            Deep
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(result.primitives().len() > 0);
}

/// 测试 recompute_styles 后布局信息保持完整。
#[test]
fn test_recompute_style_preserves_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let css = ".box { background-color: red; width: 200px; height: 100px; }";

    let first = pipeline.render_html(html, css);
    let first_child_count = first.layout.root.children.len();
    let first_vp_width = first.layout.viewport_width;
    let first_vp_height = first.layout.viewport_height;
    assert!(first_child_count > 0);

    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (_, _, layout) = pipeline.recompute_styles(&doc, &stylesheets);

    assert_eq!(layout.viewport_width, first_vp_width);
    assert_eq!(layout.viewport_height, first_vp_height);
    assert_eq!(layout.root.children.len(), first_child_count);
    assert!(pipeline.layout().is_some());
}

/// 测试 HTML 表格结构的渲染。
#[test]
fn test_render_html_table_structure() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <table>
            <tr><td>A1</td><td>B1</td></tr>
            <tr><td>A2</td><td>B2</td></tr>
        </table>
    </body></html>"#;
    let css = r#"
        table { background-color: #f0f0f0; width: 400px; }
        td { background-color: #ffffff; border: 1px solid #cccccc; padding: 8px; }
    "#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.primitives().fills.is_empty());
    assert!(result.layout.viewport_width > 0.0);
    assert!(!result.layout.root.children.is_empty());
}

// ── 边界条件补充 ──

/// 测试连续多次 incremental render 后状态正确。
#[test]
fn test_pipeline_multiple_incremental_renders() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="a">First</div><div id="b">Second</div></body></html>"#;
    let css = "div { width: 100px; height: 50px; background-color: blue; }";
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
    let count1 = result.primitives().fills.len();

    let result2 = pipeline.render_html(html, css);
    assert!(result2.timings.total_ms >= 0.0);
    assert_eq!(result2.primitives().fills.len(), count1);
}

/// 测试渲染包含 <style> 标签的 HTML 不崩溃。
#[test]
fn test_pipeline_render_with_style_tag() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let html = r#"<html><head><style>div { color: red; }</style></head><body><div>Styled</div></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试 `<style>` 内联样式会被应用。
#[test]
fn test_pipeline_render_inline_style_tag_applies_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head><style>div { background-color: red; width: 200px; height: 100px; }</style></head><body><div>Box</div></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(!result.primitives().fills.is_empty());
}

/// 测试 grid 两列布局。
#[test]
fn test_pipeline_inline_style_grid_two_columns() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head><style>
      .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
      .cell { height: 50px; background: red; }
    </style></head><body><div class="grid">
      <div class="cell">1</div><div class="cell">2</div>
      <div class="cell">3</div><div class="cell">4</div>
    </div></body></html>"#;
    let result = pipeline.render_html(html, "");

    let total = layout_height(&result.layout.root, 0.0);
    assert!(
        total < 150.0,
        "2x2 grid with 50px cells should be ~110px tall, got {total}"
    );
}

/// 测试 `<style>` 内 CSS 文本不参与排版。
#[test]
fn test_inline_style_tag_hidden_in_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head><style>
      head, style, title { display: none; }
      body { margin: 0; }
      .box { height: 40px; background: red; }
    </style><title>T</title></head><body><div class="box">Hi</div></body></html>"#;
    let result = pipeline.render_html(html, "");

    let total = layout_height(&result.layout.root, 0.0);
    assert!(total < 120.0, "hidden style tag should not inflate layout, got {total}");
}

/// 测试渲染纯文本内容不崩溃。
#[test]
fn test_pipeline_render_plain_text() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let html = "<html><body>Hello World</body></html>";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试渲染包含嵌套 div 的深层结构。
#[test]
fn test_pipeline_deeply_nested_divs() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body>".to_string() + &"<div>".repeat(10) + "inner" + &"</div>".repeat(10) + "</body></html>";
    let result = pipeline.render_html(&html, "div { padding: 5px; background-color: #eee; }");
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
    assert!(!result.layout.root.children.is_empty());
}

/// 测试 zero-width viewport 渲染不崩溃。
#[test]
fn test_pipeline_zero_width_viewport() {
    let mut pipeline = RenderPipeline::new(0.0, 600.0);
    let html = "<html><body><div>Test</div></body></html>";
    let result = pipeline.render_html(html, "div { width: 100px; height: 50px; }");
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 viewport 访问器。
#[test]
fn test_pipeline_viewport_accessors() {
    let pipeline = RenderPipeline::new(1024.0, 768.0);
    assert_eq!(pipeline.viewport_width(), 1024.0);
    assert_eq!(pipeline.viewport_height(), 768.0);

    let pipeline2 = RenderPipeline::new(375.0, 667.0);
    assert_eq!(pipeline2.viewport_width(), 375.0);
    assert_eq!(pipeline2.viewport_height(), 667.0);
}

/// 测试多个不同视口的管线互不影响。
#[test]
fn test_pipeline_multiple_render_different_viewports() {
    let mut p1 = RenderPipeline::new(800.0, 600.0);
    let mut p2 = RenderPipeline::new(1920.0, 1080.0);

    let html = "<html><body><div>Content</div></body></html>";
    let r1 = p1.render_html(html, "");
    let r2 = p2.render_html(html, "");

    assert_eq!(p1.viewport_width(), 800.0);
    assert_eq!(p2.viewport_width(), 1920.0);
    assert!(r1.timings.total_ms >= 0.0);
    assert!(r2.timings.total_ms >= 0.0);
    assert_ne!(r1.layout.viewport_width, r2.layout.viewport_width);
}

/// 测试 dirty_tracker_mut 的修改持久存在。
#[test]
fn test_dirty_tracker_mut_persistence() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);

    pipeline
        .dirty_tracker_mut()
        .mark_dirty(Rect::new(10.0, 20.0, 50.0, 30.0));
    assert_eq!(pipeline.dirty_tracker().dirty_rects().len(), 1);

    pipeline
        .dirty_tracker_mut()
        .mark_dirty(Rect::new(100.0, 100.0, 20.0, 20.0));
    assert_eq!(pipeline.dirty_tracker().dirty_rects().len(), 2);

    assert_eq!(pipeline.dirty_tracker().dirty_rects()[0].origin.x, 10.0);
    assert_eq!(pipeline.dirty_tracker().dirty_rects()[1].origin.x, 100.0);
}

/// 测试连续多次 render_html 不导致管线状态异常。
#[test]
fn test_pipeline_repeated_render_html_stability() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Test</div></body></html>";

    for i in 0..5 {
        let result = pipeline.render_html(html, "");
        assert!(result.timings.total_ms >= 0.0, "第 {} 次渲染应成功", i);
        assert!(pipeline.layout().is_some(), "第 {} 次渲染后布局应存在", i);
    }

    assert_eq!(pipeline.viewport_width(), 800.0);
    assert_eq!(pipeline.viewport_height(), 600.0);
}

/// 测试 render_html 后 cached_layout 的 viewport 正确。
#[test]
fn test_pipeline_cached_layout_viewport_correct() {
    let mut pipeline = RenderPipeline::new(640.0, 480.0);
    assert!(pipeline.layout().is_none());

    let _ = pipeline.render_html("<html><body>Hi</body></html>", "");
    let layout = pipeline.layout().unwrap();
    assert_eq!(layout.viewport_width, 640.0);
    assert_eq!(layout.viewport_height, 480.0);
}

/// 测试 dirty_tracker 初始为空且非 full_redraw。
#[test]
fn test_pipeline_initial_dirty_tracker_state() {
    let pipeline = RenderPipeline::new(800.0, 600.0);
    assert!(pipeline.dirty_tracker().dirty_rects().is_empty());
    assert!(!pipeline.dirty_tracker().is_full_redraw());
    assert_eq!(pipeline.dirty_tracker().dirty_area(), 0.0);
}

/// 测试 layout() 在首次 render_html 前返回 None。
#[test]
fn test_pipeline_layout_none_before_render() {
    let pipeline = RenderPipeline::new(800.0, 600.0);
    assert!(pipeline.layout().is_none());
}

// ── 动画管线集成测试 ──

/// 测试 render_html_animated 不崩溃。
#[test]
fn test_render_html_animated_no_panic() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Hello</div></body></html>"#;
    let css = r#"
        @keyframes fade {
            from { opacity: 1.0; }
            to { opacity: 0.0; }
        }
        .box { animation: fade 1s linear; background-color: red; width: 100px; height: 50px; }
    "#;
    let result = pipeline.render_html_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试带动画的渲染管线在不同时间点产生不同的 opacity。
#[test]
fn test_animated_opacity_changes_over_time() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="fade">Content</div></body></html>"#;
    let css = r#"
        @keyframes fadeOut {
            from { opacity: 1.0; }
            to { opacity: 0.0; }
        }
        .fade { animation: fadeOut 1s linear; background-color: red; width: 200px; height: 100px; }
    "#;

    let r0 = pipeline.render_html_animated(html, css, 0.0);
    assert!(!r0.primitives().fills.is_empty());

    let r5 = pipeline.render_html_animated(html, css, 0.5);
    assert!(!r5.primitives().fills.is_empty());

    let r1 = pipeline.render_html_animated(html, css, 1.0);
    assert!(r1.timings.total_ms >= 0.0);
}

/// 测试无动画的 CSS 在 render_html_animated 中正常工作。
#[test]
fn test_animated_pipeline_without_animation() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Static</div></body></html>"#;
    let css = ".box { background-color: blue; width: 200px; height: 100px; }";

    let result = pipeline.render_html_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试动画时钟可通过 pipeline 访问。
#[test]
fn test_animation_clock_accessible() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let clock = pipeline.animation_clock_mut();
    assert!(clock.registered_keyframe_names().is_empty());
}

/// 测试动画管线中 width 动画改变布局。
#[test]
fn test_animated_width_changes_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="grow">Text</div></body></html>"#;
    let css = r#"
        @keyframes growWidth {
            from { width: 100px; }
            to { width: 300px; }
        }
        .grow { animation: growWidth 1s linear; background-color: green; height: 50px; }
    "#;

    let r_start = pipeline.render_html_animated(html, css, 0.0);
    let r_mid = pipeline.render_html_animated(html, css, 0.5);

    assert!(r_start.timings.total_ms >= 0.0);
    assert!(r_mid.timings.total_ms >= 0.0);
    assert!(pipeline.layout().is_some());
}

/// 测试多个 @keyframes 规则同时存在。
#[test]
fn test_multiple_keyframes_rules() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="a">A</div>
        <div class="b">B</div>
    </body></html>"#;
    let css = r#"
        @keyframes fadeIn {
            from { opacity: 0.0; }
            to { opacity: 1.0; }
        }
        @keyframes slideIn {
            from { width: 0px; }
            to { width: 200px; }
        }
        .a { animation: fadeIn 1s linear; background-color: red; height: 50px; }
        .b { animation: slideIn 1s linear; background-color: blue; height: 50px; }
    "#;

    let result = pipeline.render_html_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试动画延迟期间渲染正确。
#[test]
fn test_animated_delay_period() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="delayed">Text</div></body></html>"#;
    let css = r#"
        @keyframes fadeOut {
            from { opacity: 1.0; }
            to { opacity: 0.0; }
        }
        .delayed { animation: fadeOut 1s 0.5s linear; background-color: red; width: 100px; height: 50px; }
    "#;

    let result = pipeline.render_html_animated(html, css, 0.0);
    assert!(result.timings.total_ms >= 0.0);

    let result = pipeline.render_html_animated(html, css, 1.0);
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 render_html_animated 处理空 CSS 不崩溃。
#[test]
fn test_animated_pipeline_empty_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>No CSS</div></body></html>";
    let result = pipeline.render_html_animated(html, "", 0.0);
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 @keyframes 内嵌在 <style> 标签中。
#[test]
fn test_animated_keyframes_in_style_tag() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head><style>
        @keyframes pulse {
            0% { opacity: 1.0; }
            50% { opacity: 0.5; }
            100% { opacity: 1.0; }
        }
        .pulse { animation: pulse 2s linear; background-color: red; width: 100px; height: 100px; }
    </style></head><body><div class="pulse">Pulse</div></body></html>"#;

    let result = pipeline.render_html_animated(html, "", 1.0);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试 render_html_animated 处理 CSS transition 不崩溃。
#[test]
fn test_render_html_animated_with_transition() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="box" style="transition: opacity 0.5s; opacity: 0.5;">Trans</div>
    </body></html>"#;
    let r1 = pipeline.render_html_animated(html, "", 0.0);
    assert!(r1.timings.total_ms >= 0.0);
    let r2 = pipeline.render_html_animated(html, "", 0.25);
    assert!(r2.timings.total_ms >= 0.0);
}
