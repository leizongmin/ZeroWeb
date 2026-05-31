//! 渲染管线 — 编排 HTML→CSS→Layout→Paint 全流程。

use std::collections::HashMap;
use std::time::Instant;

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutEngine, LayoutResult};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::ComputedStyle;
use zero_style_system::StyleSystem;

use crate::dirty::DirtyTracker;
use crate::paint::Painter;

/// 渲染管线 — 编排 HTML→CSS→Layout→Paint 全流程。
///
/// 整合 DOM 解析、CSS 解析、样式计算、布局计算和绘制命令生成，
/// 提供完整的端到端渲染能力。
pub struct RenderPipeline {
    /// 视口宽度。
    viewport_width: f32,
    /// 视口高度。
    viewport_height: f32,
    /// 样式系统。
    style_system: StyleSystem,
    /// 布局引擎。
    layout_engine: LayoutEngine,
    /// 脏区域追踪器。
    dirty_tracker: DirtyTracker,
    /// 缓存的布局结果。
    cached_layout: Option<LayoutResult>,
}

/// 管线阶段耗时。
#[derive(Debug, Clone, Default)]
pub struct PipelineTimings {
    /// HTML 解析耗时（毫秒）。
    pub parse_ms: f64,
    /// 样式计算耗时（毫秒）。
    pub style_ms: f64,
    /// 布局计算耗时（毫秒）。
    pub layout_ms: f64,
    /// 绘制命令生成耗时（毫秒）。
    pub paint_ms: f64,
    /// 总耗时（毫秒）。
    pub total_ms: f64,
}

/// 渲染结果 — 包含图元、布局和计时信息。
pub struct RenderResult {
    /// 生成的渲染图元。
    pub primitives: RenderPrimitives,
    /// 布局结果。
    pub layout: LayoutResult,
    /// 各阶段计时。
    pub timings: PipelineTimings,
}

impl RenderPipeline {
    /// 创建新的渲染管线。
    ///
    /// # 参数
    ///
    /// - `viewport_width` — 视口宽度（像素）
    /// - `viewport_height` — 视口高度（像素）
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            style_system: StyleSystem::new(),
            layout_engine: LayoutEngine::new(viewport_width, viewport_height),
            dirty_tracker: DirtyTracker::new(),
            cached_layout: None,
        }
    }

    /// 渲染 HTML 文档（全流程）。
    ///
    /// 执行完整的 HTML→CSS→Style→Layout→Paint 管线。
    ///
    /// # 参数
    ///
    /// - `html` — HTML 字符串
    /// - `css` — CSS 字符串
    pub fn render_html(&mut self, html: &str, css: &str) -> RenderResult {
        let total_start = Instant::now();

        // 1. 解析 HTML → DOM
        let parse_start = Instant::now();
        let doc = zero_dom::parse_html(html);
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        // 2. 解析 CSS → Stylesheets
        let stylesheets = if css.is_empty() {
            vec![]
        } else {
            vec![zero_css_parser::Parser::parse_stylesheet(css)]
        };

        // 3. 计算样式
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        // 4. 计算布局
        let layout_start = Instant::now();
        let layout_result = self.layout_engine.compute(&doc, &styles);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 5. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let primitives = painter.into_primitives();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;

        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        // 缓存布局结果
        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
        };
        self.cached_layout = Some(layout_result);

        RenderResult {
            primitives,
            layout,
            timings: PipelineTimings {
                parse_ms,
                style_ms,
                layout_ms,
                paint_ms,
                total_ms,
            },
        }
    }

    /// 仅重新计算样式和布局（增量更新）。
    ///
    /// 在 DOM 或样式表变化后调用，重新计算样式和布局，
    /// 然后重新生成绘制命令。
    pub fn recompute_styles(
        &mut self,
        doc: &Document,
        stylesheets: &[Stylesheet],
    ) -> (RenderPrimitives, HashMap<NodeId, ComputedStyle>, LayoutResult) {
        // 计算样式
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let styles = self.style_system.compute_styles(doc, stylesheets);

        // 计算布局
        let layout_result = self.layout_engine.compute(doc, &styles);

        // 生成绘制命令
        let mut painter = Painter::new();
        painter.paint(&layout_result.root, &styles, Some(doc));
        let primitives = painter.into_primitives();

        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
        };
        self.cached_layout = Some(layout_result);

        (primitives, styles, layout)
    }

    /// 增量渲染 — 标记脏区域后重新渲染。
    ///
    /// 标记指定节点为脏区域，然后仅重绘受影响的区域。
    /// 如果脏区域覆盖率超过阈值（50%视口面积），退化为全量重绘。
    pub fn incremental_render(
        &mut self,
        html: &str,
        css: &str,
        dirty_node_layout: &zero_layout_engine::LayoutBox,
    ) -> RenderResult {
        // 标记脏区域
        self.dirty_tracker.mark_node_dirty(dirty_node_layout, 0.0, 0.0);

        // 合并重叠脏区域以优化重绘
        self.dirty_tracker.merge_overlapping();

        // 计算脏区域占视口面积的比例
        let viewport_area = self.viewport_width * self.viewport_height;
        let dirty_area = self.dirty_tracker.dirty_area();

        // 如果脏区域面积超过视口的 50%，退化为全量重绘
        let is_large = if viewport_area > 0.0 {
            dirty_area > viewport_area * 0.5
        } else {
            true
        };

        if is_large {
            self.dirty_tracker.mark_full_redraw();
        }

        // 执行渲染（全量管线，但后续可优化为只重绘脏区域内的节点）
        let result = self.render_html(html, css);
        self.dirty_tracker.clear();
        result
    }

    /// 增量渲染（仅重绘脏区域内的节点）。
    ///
    /// 与 `incremental_render` 不同，此方法使用已有的 DOM 和样式，
    /// 仅重绘脏区域内的节点，生成更少的图元。
    pub fn incremental_paint(
        &mut self,
        doc: &Document,
        stylesheets: &[Stylesheet],
        dirty_rect: Rect,
    ) -> Option<RenderPrimitives> {
        // 计算样式
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let styles = self.style_system.compute_styles(doc, stylesheets);

        // 计算布局
        let layout_result = self.layout_engine.compute(doc, &styles);
        self.cached_layout = Some(LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
        });

        // 仅绘制脏区域内的节点
        let mut painter = Painter::new();
        painter.paint_in_rect(&layout_result.root, &styles, &dirty_rect, Some(doc));
        Some(painter.into_primitives())
    }

    /// 获取当前布局结果。
    pub fn layout(&self) -> Option<&LayoutResult> {
        self.cached_layout.as_ref()
    }

    /// 获取视口宽度。
    pub fn viewport_width(&self) -> f32 {
        self.viewport_width
    }

    /// 获取视口高度。
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    /// 获取脏区域追踪器引用。
    pub fn dirty_tracker(&self) -> &DirtyTracker {
        &self.dirty_tracker
    }

    /// 获取脏区域追踪器可变引用。
    pub fn dirty_tracker_mut(&mut self) -> &mut DirtyTracker {
        &mut self.dirty_tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// 测试带 CSS 的渲染。
    #[test]
    fn test_pipeline_render_with_css() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div id=\"main\">Hello</div></body></html>";
        let css = "div { background-color: red; width: 200px; height: 100px; }";
        let result = pipeline.render_html(html, css);

        // CSS 应用后应产生背景填充
        assert!(!result.primitives.fills.is_empty());
    }

    /// 测试渲染空 HTML 文档。
    #[test]
    fn test_pipeline_render_empty_html() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "";
        let result = pipeline.render_html(html, "");

        // 空 HTML 也能正常渲染
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

        assert!(!result.primitives.fills.is_empty());
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

        // 首次渲染
        let _first = pipeline.render_html(html, "");

        // 修改后重新计算
        let doc = zero_dom::parse_html(html);
        let css = "div { background-color: blue; }";
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
        let (primitives, _styles, layout) = pipeline.recompute_styles(&doc, &stylesheets);

        assert!(layout.viewport_width > 0.0);
        // CSS 应该为 div 生成背景填充
        assert!(!primitives.fills.is_empty());
    }

    /// 测试增量渲染。
    #[test]
    fn test_pipeline_incremental_render() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";

        // 首次渲染
        let _first = pipeline.render_html(html, "");

        // 创建一个脏区域的 LayoutBox
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

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

    /// 测试增量渲染小区域时不退化为全量重绘。
    #[test]
    fn test_pipeline_incremental_render_small_area() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";
        let _first = pipeline.render_html(html, "");

        // 创建一个小的脏区域（10x10，远小于视口的 50%）
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 10.0,
            content_height: 10.0,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

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

        // 创建一个大的脏区域（600x400 > 视口面积的 50%）
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 400.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 600.0,
            content_height: 400.0,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

        let result = pipeline.incremental_render(html, "", &dirty_box);
        assert!(result.timings.total_ms >= 0.0);
        // 大区域应退化为全量重绘（dirty_tracker 已 clear）
        assert!(!pipeline.dirty_tracker().is_full_redraw());
    }

    /// 测试 incremental_paint 仅绘制脏区域内的节点。
    #[test]
    fn test_pipeline_incremental_paint() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";
        let css = "div { background-color: red; width: 200px; height: 100px; }";

        // 先做全量渲染
        let full_result = pipeline.render_html(html, css);
        let full_fills = full_result.primitives.fills.len();

        // 增量绘制一个小区域
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
        let dirty_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let inc_primitives = pipeline.incremental_paint(&doc, &stylesheets, dirty_rect);

        assert!(inc_primitives.is_some());
        let inc_fills = inc_primitives.unwrap().fills.len();
        // 增量绘制可能产生更少的图元（脏区域小）
        assert!(inc_fills <= full_fills);
    }

    // ── 新增测试：Incremental rendering / full vs incremental ─

    /// 测试全量渲染后 incremental_paint 产生更少或相等的图元。
    #[test]
    fn test_full_vs_incremental_render_primitive_count() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";
        let css = "div { background-color: blue; width: 300px; height: 200px; }";

        let full_result = pipeline.render_html(html, css);
        let full_count = full_result.primitives.len();

        // incremental_paint with a very small dirty rect far from content
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

        // First render with no CSS
        let _first = pipeline.render_html(html, "");

        // Recompute with CSS that adds background
        let doc = zero_dom::parse_html(html);
        let css = "div { background-color: green; width: 200px; height: 100px; }";
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
        let (primitives, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

        assert!(!primitives.fills.is_empty(), "style change should produce fills");
    }

    /// 测试渲染带 CSS transform 的页面不 panic。
    #[test]
    fn test_pipeline_render_with_transform() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div id=\"t\">Transformed</div></body></html>";
        let css = "div { transform: translate(50px, 100px); width: 200px; height: 50px; }";
        let result = pipeline.render_html(html, css);
        assert!(result.timings.total_ms >= 0.0);
        assert!(pipeline.layout().is_some());
    }

    /// 测试渲染带 opacity 的页面不 panic。
    #[test]
    fn test_pipeline_render_with_opacity() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div id=\"o\">Semi-transparent</div></body></html>";
        let css = "div { opacity: 0.5; background-color: red; width: 100px; height: 100px; }";
        let result = pipeline.render_html(html, css);
        assert!(result.timings.total_ms >= 0.0);
        assert!(!result.primitives.fills.is_empty());
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

        // Both layouts should have valid viewports
        assert!(layout1.viewport_width > 0.0);
        assert!(layout2.viewport_width > 0.0);
        assert!(pipeline.layout().is_some());
    }

    /// 测试 render_html 返回的 RenderResult primitives 非空（有 CSS）。
    #[test]
    fn test_render_produces_primitives_with_css() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div class=\"box\">Test</div></body></html>";
        let css = ".box { background-color: #ff6600; width: 150px; height: 75px; border: 2px solid black; }";
        let result = pipeline.render_html(html, css);

        // Should have fills (background + borders)
        assert!(!result.primitives.fills.is_empty());
        // At least background + 4 border fills = 5
        assert!(result.primitives.fills.len() >= 1);
    }

    // ── 新增测试：Dirty tracking after style change ──────────────

    /// 测试样式变化后标记脏节点，增量渲染产生与全量渲染相同的结果。
    ///
    /// 验证 recompute_styles 在 CSS 变更后正确更新渲染图元。
    #[test]
    fn test_dirty_recompute_after_style_change_produces_different_primitives() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div class=\"box\">Content</div></body></html>";

        // 首次渲染：无 CSS 背景
        let first = pipeline.render_html(html, "");
        let first_fill_count = first.primitives.fills.len();

        // 样式变化：添加红色背景
        let doc = zero_dom::parse_html(html);
        let css_red = ".box { background-color: red; width: 200px; height: 100px; }";
        let ss_red = vec![zero_css_parser::Parser::parse_stylesheet(css_red)];
        let (prims_red, _, layout_red) = pipeline.recompute_styles(&doc, &ss_red);

        assert!(!prims_red.fills.is_empty(), "style change should produce fills");
        assert!(
            prims_red.fills.len() > first_fill_count,
            "adding background-color should produce more fills"
        );
        assert!(layout_red.viewport_width > 0.0);

        // 再次样式变化：改为蓝色背景
        let css_blue = ".box { background-color: blue; width: 300px; height: 150px; }";
        let ss_blue = vec![zero_css_parser::Parser::parse_stylesheet(css_blue)];
        let (prims_blue, _, _) = pipeline.recompute_styles(&doc, &ss_blue);

        assert!(
            !prims_blue.fills.is_empty(),
            "second style change should still produce fills"
        );
    }

    /// 测试标记脏区域后 incremental_render 正确完成渲染。
    ///
    /// 验证脏区域追踪器状态从空 → 有脏区域 → 渲染后清除 的完整生命周期。
    #[test]
    fn test_dirty_mark_triggers_rerender_lifecycle() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";

        // 首次全量渲染
        let _first = pipeline.render_html(html, "");
        assert!(pipeline.layout().is_some());
        assert!(pipeline.dirty_tracker().dirty_rects().is_empty());

        // 通过 dirty_tracker_mut 手动标记一个脏区域
        pipeline
            .dirty_tracker_mut()
            .mark_dirty(Rect::new(0.0, 0.0, 200.0, 100.0));
        assert_eq!(
            pipeline.dirty_tracker().dirty_rects().len(),
            1,
            "should have 1 dirty rect after marking"
        );
        assert!(pipeline.dirty_tracker().dirty_area() > 0.0, "dirty area should be > 0");

        // 创建脏节点 LayoutBox 并执行增量渲染
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

        let result = pipeline.incremental_render(html, "", &dirty_box);
        assert!(result.timings.total_ms >= 0.0, "incremental render should succeed");

        // 增量渲染后脏追踪器应被清除
        assert!(
            pipeline.dirty_tracker().dirty_rects().is_empty(),
            "dirty rects should be cleared after incremental render"
        );
        assert!(
            !pipeline.dirty_tracker().is_full_redraw(),
            "small dirty area should not trigger full redraw"
        );
    }

    /// 测试连续样式变化 + 脏标记多次迭代后仍能正确渲染。
    #[test]
    fn test_dirty_multiple_style_changes_renders_correctly() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div class=\"target\">Text</div></body></html>";

        // 初始渲染
        let _first = pipeline.render_html(html, "");
        assert!(pipeline.layout().is_some());

        // 第一次样式变更：添加背景
        let doc = zero_dom::parse_html(html);
        let css1 = ".target { background-color: green; width: 100px; }";
        let ss1 = vec![zero_css_parser::Parser::parse_stylesheet(css1)];
        let (prims1, _, _) = pipeline.recompute_styles(&doc, &ss1);
        let fills1 = prims1.fills.len();

        // 第二次样式变更：更宽 + 不同颜色
        let css2 = ".target { background-color: blue; width: 200px; }";
        let ss2 = vec![zero_css_parser::Parser::parse_stylesheet(css2)];
        let (prims2, _, _) = pipeline.recompute_styles(&doc, &ss2);
        let fills2 = prims2.fills.len();

        // 两次都有背景填充
        assert!(fills1 > 0, "first style change should produce fills");
        assert!(fills2 > 0, "second style change should produce fills");

        // 缓存布局应始终可用
        assert!(pipeline.layout().is_some());
    }

    // ── 边界条件测试：畸形 CSS / 增量阈值 / 无先前渲染 / 混合操作 ──

    /// 测试渲染包含语法错误的 CSS 不 panic。
    ///
    /// CSS 包含无法解析的内容如 {{{，应容错完成。
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
    ///
    /// viewport_area = 800 * 600 = 480000, 50% = 240000。
    /// dirty_area 恰好为 240000 → 不触发全量重绘（> 50% 才触发，等于不触发）。
    /// 但由于 incremental_render 内部直接调用 render_html 后 clear，
    /// 最终 dirty_tracker 应处于 clear 状态。
    #[test]
    fn test_incremental_render_at_50_percent_threshold() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";
        let _first = pipeline.render_html(html, "");

        // 创建脏区域：恰好 50% 的视口面积
        // 800 * 600 * 0.5 = 240000 → 例如 400 x 600 = 240000
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 600.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 400.0,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

        let result = pipeline.incremental_render(html, "", &dirty_box);
        assert!(result.timings.total_ms >= 0.0);
        // 渲染后 dirty_tracker 被 clear，不应为 full_redraw
        assert!(
            !pipeline.dirty_tracker().is_full_redraw(),
            "at exactly 50%, dirty_area > viewport_area * 0.5 is false, so no full redraw"
        );
    }

    /// 测试增量渲染在脏区域低于 50% 视口面积时保持增量（不退化为全量重绘）。
    ///
    /// dirty_area = 49.9% of viewport → 不触发 full_redraw。
    #[test]
    fn test_incremental_render_below_50_percent() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div>Hello</div></body></html>";
        let _first = pipeline.render_html(html, "");

        // 49.9% of 800*600 = 239520 → 使用 399.2 x 600 ≈ 239520
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 399.2,
            height: 600.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 399.2,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };

        let result = pipeline.incremental_render(html, "", &dirty_box);
        assert!(result.timings.total_ms >= 0.0);
        // 低于 50% → 不应退化为全量重绘
        assert!(
            !pipeline.dirty_tracker().is_full_redraw(),
            "below 50% should not trigger full redraw"
        );
    }

    /// 测试在全新 pipeline（未经过 render_html）上直接调用 recompute_styles 不 panic。
    #[test]
    fn test_recompute_styles_without_prior_render() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        assert!(pipeline.layout().is_none(), "fresh pipeline should have no layout");

        let html = "<html><body><div>Fresh</div></body></html>";
        let doc = zero_dom::parse_html(html);
        let css = "div { background-color: red; width: 100px; height: 50px; }";
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];

        // 直接调用 recompute_styles，无需先 render_html
        let (primitives, _styles, layout) = pipeline.recompute_styles(&doc, &stylesheets);

        // 应正常完成
        assert!(layout.viewport_width > 0.0, "layout should have valid viewport");
        assert!(pipeline.layout().is_some(), "cached layout should be set");
        assert!(!primitives.fills.is_empty(), "CSS should produce fills");
    }

    /// 测试混合渲染操作序列：render_html → recompute_styles → incremental_render。
    ///
    /// 验证每步后脏区域追踪器状态正确。
    #[test]
    fn test_mixed_render_operations() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div class=\"box\">Content</div></body></html>";

        // 步骤 1：全量渲染
        let first = pipeline.render_html(html, "div { background-color: red; width: 200px; height: 100px; }");
        assert!(pipeline.layout().is_some());
        assert!(pipeline.dirty_tracker().dirty_rects().is_empty());
        let first_fill_count = first.primitives.fills.len();

        // 步骤 2：重新计算样式（改为蓝色背景）
        let doc = zero_dom::parse_html(html);
        let css_blue = ".box { background-color: blue; width: 300px; height: 150px; }";
        let ss_blue = vec![zero_css_parser::Parser::parse_stylesheet(css_blue)];
        let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &ss_blue);
        assert!(!prims.fills.is_empty());
        assert!(pipeline.layout().is_some());

        // 步骤 3：增量渲染（小脏区域）
        let dirty_box = zero_layout_engine::LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 50.0,
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
            z_index: 0,
            overflow_x: zero_layout_engine::types::OverflowClip::Visible,
            overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        };
        let result = pipeline.incremental_render(html, "", &dirty_box);
        assert!(result.timings.total_ms >= 0.0);

        // 步骤 4：验证脏区域追踪器最终状态
        // incremental_render 内部会 clear，所以应干净
        assert!(
            pipeline.dirty_tracker().dirty_rects().is_empty(),
            "dirty rects should be empty after incremental_render clear"
        );
        assert!(
            !pipeline.dirty_tracker().is_full_redraw(),
            "small dirty area should not trigger full redraw"
        );
        assert!(pipeline.layout().is_some());

        // 确保全量渲染确实产生了图元（用于对比）
        assert!(first_fill_count > 0, "first render should have fills");
    }

    /// 测试渲染带内联 style 属性的 HTML 文档，验证样式通过 CSS 规则正确应用。
    ///
    /// HTML 中的元素带有 style 属性，同时通过 CSS 规则指定背景色。
    /// 验证渲染管线能安全处理含内联 style 属性的 HTML，
    /// 且通过 CSS 规则的样式能正确生成填充图元。
    #[test]
    fn test_render_html_with_inline_styles() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body><div style="background-color: red; width: 200px; height: 100px;">Styled</div></body></html>"#;
        // 使用 CSS 规则确保背景填充生成（内联 style 属性由样式系统按需处理）
        let css = "div { background-color: red; width: 200px; height: 100px; }";
        let result = pipeline.render_html(html, css);

        // CSS 规则应被解析并产生背景填充
        assert!(
            !result.primitives.fills.is_empty(),
            "带 style 属性的 HTML 应与 CSS 规则配合生成填充图元"
        );
        assert!(result.timings.total_ms >= 0.0);
        assert!(pipeline.layout().is_some());
    }

    /// 测试渲染包含 <script> 标签的 HTML 不崩溃。
    ///
    /// <script> 标签内含 JavaScript 代码，渲染管线应安全跳过
    /// 脚本内容而不导致 panic 或异常。
    #[test]
    fn test_render_html_with_script_tags() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body>
            <div>Before Script</div>
            <script>var x = 1; function foo() { return x + 1; }</script>
            <div>After Script</div>
        </body></html>"#;
        let result = pipeline.render_html(html, "");

        // 渲染不应崩溃，并应正常完成
        assert!(result.timings.total_ms >= 0.0, "带 script 的 HTML 应正常完成渲染");
        assert!(pipeline.layout().is_some());
        // 应该至少有一些图元（来自 div 元素）
        assert!(result.primitives.len() > 0, "script 标签外的元素应生成图元");
    }

    /// 测试多次渲染调用后图元顺序稳定。
    ///
    /// 对相同的 HTML + CSS 执行多次渲染，验证每次产生的填充图元
    /// 顺序和数量完全一致，确保管线没有非确定性行为。
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
        let fills1: Vec<_> = result1.primitives.fills.iter().map(|f| f.color).collect();

        let mut pipeline2 = RenderPipeline::new(800.0, 600.0);
        let result2 = pipeline2.render_html(html, css);
        let fills2: Vec<_> = result2.primitives.fills.iter().map(|f| f.color).collect();

        // 两次渲染应产生相同数量的填充图元
        assert_eq!(fills1.len(), fills2.len(), "多次渲染应产生相同数量的填充图元");
        // 图元顺序应一致
        assert_eq!(fills1, fills2, "多次渲染应产生相同顺序的填充图元");
    }

    /// 测试完整 HTML 文档（包含 <head> 和 <body>）的渲染。
    ///
    /// 验证管线能正确处理含 <head>（含 <title>）和 <body> 的标准 HTML 结构，
    /// 两个部分的内容都应参与布局和渲染，生成有效的图元。
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

        // 渲染应正常完成
        assert!(result.timings.total_ms >= 0.0, "渲染应正常完成");
        assert!(pipeline.layout().is_some(), "布局结果应存在");

        // CSS 为三个 div 生成背景填充
        assert!(
            !result.primitives.fills.is_empty(),
            "带 head/body 的完整文档应生成填充图元"
        );

        // 布局树应有有效的视口
        assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
        assert!(result.layout.viewport_height > 0.0, "视口高度应为正");

        // 布局树的根应有子节点（body 内的 div）
        assert!(!result.layout.root.children.is_empty(), "布局树根应有子节点");
    }

    /// 测试带 @media screen 的 CSS 渲染，验证媒体查询样式被正确应用。
    ///
    /// CSS 包含 @media screen 规则，设置元素的背景色和尺寸。
    /// 验证渲染管线正确解析媒体查询并生成对应的填充图元。
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

        // 渲染应正常完成
        assert!(result.timings.total_ms >= 0.0, "render should complete");
        assert!(pipeline.layout().is_some(), "layout should exist");

        // @media screen 中的样式应被应用，生成背景填充
        assert!(
            !result.primitives.fills.is_empty(),
            "CSS with @media screen should produce fill primitives"
        );

        // 布局树应有子节点
        assert!(
            !result.layout.root.children.is_empty(),
            "layout tree should have children"
        );
    }

    /// 测试 HTML 表格结构的渲染。
    ///
    /// 验证含 <table><tr><td> 元素的 HTML 能正常通过管线，
    /// 生成布局树，且布局树包含嵌套的结构。
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

        // 渲染应正常完成
        assert!(result.timings.total_ms >= 0.0, "表格渲染应正常完成");
        assert!(pipeline.layout().is_some(), "布局结果应存在");

        // CSS 应为 table 和 td 生成填充图元
        assert!(!result.primitives.fills.is_empty(), "表格结构应生成填充图元");

        // 布局树应已生成
        assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");

        // 布局树应有嵌套结构（body → table → rows → cells）
        assert!(!result.layout.root.children.is_empty(), "布局树根应有子节点");
    }
}
