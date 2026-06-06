//! 渲染管线 — 编排 HTML→CSS→Layout→Paint 全流程。

use std::collections::HashMap;
use std::time::Instant;

use slotmap::Key;
use zero_css_parser::Stylesheet;
use zero_css_parser::media_query::PrefersColorSchemeValue;
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutEngine, LayoutResult};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{RenderPrimitives, RenderStats};
use zero_style_system::ComputedStyle;
use zero_style_system::StyleSystem;

use crate::animation::AnimationClock;
use crate::dirty::DirtyTracker;
use crate::hit_test;
use crate::paint::Painter;
use crate::transition::TransitionClock;

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
    /// CSS 动画时钟。
    animation_clock: AnimationClock,
    /// CSS 过渡时钟。
    transition_clock: TransitionClock,
    /// 缓存的基础样式（用于过渡检测，存储覆盖前的原始计算样式）。
    cached_styles: HashMap<NodeId, ComputedStyle>,
    /// 缓存的布局结果。
    cached_layout: Option<LayoutResult>,
    /// 缓存的 DOM（用于命中测试）。
    cached_doc: Option<Document>,
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

/// 渲染结果 — 包含图元、布局、计时和统计信息。
pub struct RenderResult {
    /// 生成的渲染图元。
    pub primitives: RenderPrimitives,
    /// 布局结果。
    pub layout: LayoutResult,
    /// 各阶段计时。
    pub timings: PipelineTimings,
    /// 渲染统计信息（draw call 估算、图元数量、剔除数量）。
    pub stats: RenderStats,
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
            animation_clock: AnimationClock::new(),
            transition_clock: TransitionClock::new(),
            cached_styles: HashMap::new(),
            cached_layout: None,
            cached_doc: None,
        }
    }

    /// 设置用户颜色方案偏好。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.style_system.set_prefers_color_scheme(scheme);
    }

    /// 获取动画时钟的可变引用。
    pub fn animation_clock_mut(&mut self) -> &mut AnimationClock {
        &mut self.animation_clock
    }

    /// 获取过渡时钟的可变引用。
    pub fn transition_clock_mut(&mut self) -> &mut TransitionClock {
        &mut self.transition_clock
    }

    /// 渲染 HTML 文档（带动画）。
    ///
    /// 与 `render_html` 相同管线，但在样式计算后注册 @keyframes、
    /// 为有 `animation-name` 的元素启动动画、推进时钟并将
    /// 插值属性叠加到 ComputedStyle 上，再进行布局和绘制。
    ///
    /// # 参数
    ///
    /// - `html` — HTML 字符串
    /// - `css` — CSS 字符串
    /// - `current_time` — 当前时间（秒），用于动画时钟推进
    pub fn render_html_animated(&mut self, html: &str, css: &str, current_time: f64) -> RenderResult {
        let total_start = Instant::now();

        // 1. 解析 HTML → DOM
        let parse_start = Instant::now();
        let doc = zero_dom::parse_html(html);
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        // 2. 解析 CSS → Stylesheets
        let stylesheets = collect_stylesheets(&doc, css);

        // 3. 注册 @keyframes 到动画时钟
        self.animation_clock.register_from_stylesheets(&stylesheets);

        // 4. 计算样式
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let mut styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        // 4b. 过渡检测：比较新旧基础样式，启动必要的过渡
        {
            let old = std::mem::replace(&mut self.cached_styles, styles.clone());
            for (nid, ns) in &styles {
                if let Some(os) = old.get(nid) {
                    self.transition_clock
                        .start_transitions(nid.data().as_ffi(), os, ns, current_time);
                }
            }
        }

        // 5. 启动动画并应用插值覆盖
        apply_animation_overrides(&mut self.animation_clock, &mut styles, current_time);

        // 5b. 应用活跃的过渡插值
        let node_ids: Vec<NodeId> = styles.keys().copied().collect();
        for nid in node_ids {
            let key = nid.data().as_ffi();
            let props = self.transition_clock.tick(key, current_time);
            if !props.is_empty()
                && let Some(s) = styles.get_mut(&nid)
            {
                TransitionClock::apply_to_computed_style(&props, s);
            }
        }
        self.transition_clock.cleanup_finished();

        // 6. 计算布局
        let layout_start = Instant::now();
        let layout_result = self.layout_engine.compute(&doc, &styles);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 7. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let primitives = painter.into_primitives();
        let viewport = Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        let (primitives, stats) = primitives.cull_invisible(viewport);
        let primitives = primitives.batch_fills();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;

        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.cached_doc = Some(doc);

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
            stats,
        }
    }

    /// 命中测试链接，返回点击位置处 `<a href>` 的目标 URL。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let doc = self.cached_doc.as_ref()?;
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_link(doc, &layout.root, x, y)
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

        // 2. 解析 CSS → Stylesheets（外部 CSS + HTML 内 `<style>`）
        let stylesheets = collect_stylesheets(&doc, css);

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
        // 视口剔除 — 移除视口外的图元
        let viewport = Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        let (primitives, stats) = primitives.cull_invisible(viewport);
        // 对填充图元进行批处理优化
        let primitives = primitives.batch_fills();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;

        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.cached_doc = Some(doc);

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
            stats,
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

    /// 文档布局高度（CSS 逻辑像素，含溢出内容）。
    pub fn document_height(&self) -> Option<f32> {
        self.cached_layout
            .as_ref()
            .map(|layout| layout_extent_y(&layout.root, 0.0))
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

fn layout_extent_y(b: &zero_layout_engine::LayoutBox, offset_y: f32) -> f32 {
    let mut max_y = offset_y + b.y + b.height;
    for child in &b.children {
        max_y = max_y.max(layout_extent_y(child, offset_y + b.y));
    }
    max_y
}

/// 收集样式表：外部 CSS 字符串 + 文档内 `<style>` 元素文本。
fn collect_stylesheets(doc: &Document, css: &str) -> Vec<Stylesheet> {
    let mut stylesheets = Vec::new();
    if !css.is_empty() {
        stylesheets.push(zero_css_parser::Parser::parse_stylesheet(css));
    }
    for style_id in doc.get_elements_by_tag_name("style") {
        let Some(css_text) = doc.text_content(style_id) else {
            continue;
        };
        let css_text = css_text.trim();
        if !css_text.is_empty() {
            stylesheets.push(zero_css_parser::Parser::parse_stylesheet(css_text));
        }
    }
    stylesheets
}

/// 为有 animation-name 的元素启动动画并将插值属性叠加到 ComputedStyle。
///
/// 遍历所有元素的样式，检查 animation-name 列表，
/// 通过 AnimationClock 启动/推进动画，然后应用插值结果。
fn apply_animation_overrides(
    clock: &mut AnimationClock,
    styles: &mut HashMap<NodeId, ComputedStyle>,
    current_time: f64,
) {
    // 收集有动画名称的元素 ID
    let animated_ids: Vec<(u64, NodeId)> = styles
        .iter()
        .filter(|(_, s)| !s.animation_name.is_empty() && s.animation_name.iter().any(|n| !n.is_empty() && n != "none"))
        .map(|(id, _)| {
            // 将 slotmap NodeId 转为 u64
            (id.data().as_ffi(), *id)
        })
        .collect();

    for (elem_key, node_id) in animated_ids {
        let Some(style) = styles.get(&node_id) else {
            continue;
        };

        // 为元素启动动画（如果尚未启动）
        clock.start_from_computed_style(elem_key, style, current_time);

        // 推进时钟并获取插值属性
        let props = clock.tick(elem_key, current_time);

        if !props.is_empty() {
            // 将插值属性叠加到 ComputedStyle
            if let Some(style) = styles.get_mut(&node_id) {
                AnimationClock::apply_to_computed_style(&props, style);
            }
        }
    }
}
