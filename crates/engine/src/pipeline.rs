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
    pub(crate) viewport_width: f32,
    /// 视口高度。
    pub(crate) viewport_height: f32,
    /// 样式系统。
    pub(crate) style_system: StyleSystem,
    /// 布局引擎。
    pub(crate) layout_engine: LayoutEngine,
    /// 脏区域追踪器。
    dirty_tracker: DirtyTracker,
    /// CSS 动画时钟。
    animation_clock: AnimationClock,
    /// CSS 过渡时钟。
    transition_clock: TransitionClock,
    /// 缓存的基础样式（用于过渡检测，存储覆盖前的原始计算样式）。
    cached_styles: HashMap<NodeId, ComputedStyle>,
    /// 是否跳过属性指示器（用于 reftest 精确像素对比）。
    pub(crate) skip_indicators: bool,
    /// 图像固有尺寸缓存（image_key hash → (width, height)）。
    pub(crate) image_sizes: HashMap<u64, (f32, f32)>,
    /// 仅含宽高比、无确定固有尺寸的图像信号（image_key hash → ratio）。
    ///
    /// 仅 %-dim / viewBox-only SVG 出现（CSS §10.3.2）：这些 SVG 无确定固有尺寸，
    /// 仅有 viewBox 宽高比。布局须以 ratio-only 处理（不设确定 size，仅设 aspect_ratio），
    /// 让 taffy/flex 按上下文 ratio-derive。由调用方从解码后的 ImageCache 填充。
    pub(crate) image_ratios: HashMap<u64, f32>,
    /// no-ratio 图像信号（image_key hash → (真实固有宽, 真实固有高)，各 Option）。
    ///
    /// 仅 no-ratio SVG 出现（CSS §10.3.2）：width/height 非双绝对且无 viewBox，既无确定
    /// 固有尺寸也无固有宽高比。usvg 对缺失维的默认值非真实固有尺寸。值为真实固有维
    ///（仅 abs 属性存在的维，缺失维 None）；布局须**不设 aspect_ratio**，缺失维按
    /// default object size（宽 300 / 高 150）回退。由调用方从解码后的 ImageCache 填充。
    pub(crate) image_no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>,
    /// CSS font-family 查找表（字体族名 → FontId）。
    pub(crate) font_resolver: HashMap<String, u32>,
    /// 当前文档 URL（用于解析相对 `<img src>` 与 image_sizes 键）。
    pub(crate) document_url: Option<String>,
    /// 缓存的布局结果。
    pub(crate) cached_layout: Option<LayoutResult>,
    /// 缓存的 DOM（用于命中测试）。
    pub(crate) cached_doc: Option<Document>,
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
            skip_indicators: false,
            image_sizes: HashMap::new(),
            image_ratios: HashMap::new(),
            image_no_ratio: HashMap::new(),
            font_resolver: HashMap::new(),
            document_url: None,
        }
    }

    /// 设置当前文档 URL（导航时由 webview 传入，供相对路径子资源解析）。
    pub fn set_document_url(&mut self, url: Option<&str>) {
        self.document_url = url.map(str::to_string);
    }

    /// 当前文档 URL。
    pub fn document_url(&self) -> Option<&str> {
        self.document_url.as_deref()
    }

    /// 设置是否跳过属性指示器。
    ///
    /// 用于 reftest 精确像素对比——指示器是绘制在元素边角的调试标记，
    /// 会干扰像素级对比。
    pub fn set_skip_indicators(&mut self, skip: bool) {
        self.skip_indicators = skip;
    }

    /// 设置图像固有尺寸缓存。
    ///
    /// 用于 background-image 的 background-size: auto 计算。
    /// 键为图像 URL 的 hash 值，值为 (width, height) 像素尺寸。
    pub fn set_image_sizes(&mut self, sizes: HashMap<u64, (f32, f32)>) {
        self.image_sizes = sizes;
    }

    /// 设置 ratio-only 图像信号缓存（CSS §10.3.2，仅 SVG 出现）。
    ///
    /// 键为图像 URL 的 hash 值，值为 viewBox 宽高比（width/height）。这些图像无确定
    /// 固有尺寸，布局须仅设 aspect_ratio、不设确定 size。
    pub fn set_image_ratios(&mut self, ratios: HashMap<u64, f32>) {
        self.image_ratios = ratios;
    }

    /// 设置 no-ratio 图像信号缓存（CSS §10.3.2，仅 no-ratio SVG 出现）。
    ///
    /// 键为图像 URL 的 hash 值，值为 (真实固有宽, 真实固有高)（各 Option，缺失维 None）。
    /// 这些 SVG 既无确定固有尺寸也无固有宽高比，布局须不设 aspect_ratio、缺失维按
    /// default object size（宽 300 / 高 150）回退。
    pub fn set_image_no_ratio(&mut self, no_ratio: HashMap<u64, (Option<f32>, Option<f32>)>) {
        self.image_no_ratio = no_ratio;
    }

    /// 从 `self.image_sizes`（按 URL hash 索引）解析出 `<img>` 元素的解码固有尺寸，
    /// 按 DOM NodeId 索引返回，供布局引擎对无 width/height 属性的 `<img>` 注入固有尺寸。
    ///
    /// hash 解析在 engine 层完成（simple_hash 定义于本 crate），避免把 hash 函数
    /// 泄漏到 layout-engine（layout-engine 依赖 render-foundation 但不依赖 engine）。
    pub(crate) fn build_img_intrinsic_sizes(&self, doc: &Document) -> HashMap<NodeId, (f32, f32)> {
        let mut map = HashMap::new();
        for img_id in doc.get_elements_by_tag_name("img") {
            if let Some(src) = doc.get_attribute(img_id, "src") {
                let key = crate::paint::image_resource_key(&src, self.document_url.as_deref());
                if let Some(&size) = self.image_sizes.get(&key) {
                    map.insert(img_id, size);
                }
            }
        }
        map
    }

    /// 从 `self.image_ratios`（按 URL hash 索引）解析出 `<img>` 元素的 ratio-only 信号，
    /// 按 DOM NodeId 索引返回，供布局引擎对无 width/height 属性且无确定固有尺寸的
    /// `<img>`（%-dim / viewBox-only SVG）仅设 aspect_ratio（CSS §10.3.2）。
    pub(crate) fn build_img_intrinsic_ratios(&self, doc: &Document) -> HashMap<NodeId, f32> {
        let mut map = HashMap::new();
        for img_id in doc.get_elements_by_tag_name("img") {
            if let Some(src) = doc.get_attribute(img_id, "src") {
                let key = crate::paint::image_resource_key(&src, self.document_url.as_deref());
                if let Some(&ratio) = self.image_ratios.get(&key) {
                    map.insert(img_id, ratio);
                }
            }
        }
        map
    }

    /// 从 `self.image_no_ratio`（按 URL hash 索引）解析出 `<img>` 元素的 no-ratio 信号，
    /// 按 DOM NodeId 索引返回，供布局引擎对无 width/height 属性、无确定固有尺寸且无
    /// 固有宽高比的 `<img>`（no-ratio SVG）按 CSS §10.3.2 default object size sizing。
    pub(crate) fn build_img_intrinsic_no_ratio(&self, doc: &Document) -> HashMap<NodeId, (Option<f32>, Option<f32>)> {
        let mut map = HashMap::new();
        for img_id in doc.get_elements_by_tag_name("img") {
            if let Some(src) = doc.get_attribute(img_id, "src") {
                let key = crate::paint::image_resource_key(&src, self.document_url.as_deref());
                if let Some(&dims) = self.image_no_ratio.get(&key) {
                    map.insert(img_id, dims);
                }
            }
        }
        map
    }

    /// 设置 CSS font-family 查找表。
    ///
    /// 由调用方从 `FontLoader::build_font_resolver()` 构建并传入。
    /// 用于将 CSS font-family 列表解析为具体的 FontId。
    pub fn set_font_resolver(&mut self, resolver: HashMap<String, u32>) {
        self.font_resolver = resolver;
    }

    /// U1b-wiring 激活（per-font line-height）：注入 per-family 行度量映射。
    ///
    /// 调用方从 `FontLoader::build_line_metric_map()` 构建并传入（拥有所有权，避
    /// FontLoader Rc-share 冲突）。包装为 `FontMetricMap` provider 注入 LayoutEngine，
    /// 经 compute_final_inline_layouts + measure_text_content 双路径触达 IFC，使
    /// line-height:normal 走 per-family hhea（+ populate TextRun.font_id，C3 前置）。
    /// **dormant**：不调用 = provider None = 常数度量 = 零回归。
    pub fn set_font_metric_map(&mut self, map: HashMap<String, (u32, f32, f32, f32)>) {
        let provider: std::rc::Rc<dyn zero_layout_engine::FontMetricProvider> =
            std::rc::Rc::new(zero_layout_engine::FontMetricMap(map));
        self.layout_engine.set_font_metric_provider(provider);
    }

    /// 设置用户颜色方案偏好。
    pub fn set_prefers_color_scheme(&mut self, scheme: PrefersColorSchemeValue) {
        self.style_system.set_prefers_color_scheme(scheme);
    }

    /// 设置渲染媒体类型（`@media print/screen/all` 级联过滤据此生效，DC-12）。
    ///
    /// 默认 `Screen`（`StyleSystem` 初始值）= 零行为变更。`Print` 使 `@media print`
    /// 规则在级联中生效、`@media screen` 失效——用于打印预览与 reftest `--media=print`
    /// 量真实 WPT yield（R1991）。镜像 `set_prefers_color_scheme` 的全栈接线入口。
    pub fn set_media_type(&mut self, media_type: zero_css_parser::media_query::MediaType) {
        self.style_system.set_media_type(media_type);
    }

    /// 更新视口尺寸（保留 DOM 缓存，后续需 `repaint_cached_viewport` 重布局）。
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        if (self.viewport_width - width).abs() < f32::EPSILON && (self.viewport_height - height).abs() < f32::EPSILON {
            return;
        }
        self.viewport_width = width;
        self.viewport_height = height;
        self.layout_engine.set_viewport(width, height);
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
        let img_sizes = self.build_img_intrinsic_sizes(&doc);
        let img_ratios = self.build_img_intrinsic_ratios(&doc);
        let img_no_ratio = self.build_img_intrinsic_no_ratio(&doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(&doc, &styles, img_sizes, img_ratios, img_no_ratio);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 7. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let primitives = painter.into_primitives();
        let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout_result.root);
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

    /// 命中测试图片，返回 `src`（文档原始值）。
    pub fn hit_test_image(&self, x: f32, y: f32) -> Option<String> {
        let doc = self.cached_doc.as_ref()?;
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_image(doc, &layout.root, x, y)
    }

    /// 命中测试元素，返回点击位置处最深元素及其布局盒。
    pub fn hit_test_element(&self, x: f32, y: f32) -> Option<hit_test::ElementHit> {
        let doc = self.cached_doc.as_ref()?;
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_element(doc, &layout.root, x, y)
    }

    /// 构建主线程只读命中测试快照（与当前缓存 DOM/布局一致）。
    pub fn build_hit_test_cache(&self) -> Option<hit_test::HitTestCache> {
        let doc = self.cached_doc.as_ref()?;
        let layout = self.cached_layout.as_ref()?;
        Some(hit_test::HitTestCache::from_document(doc, &layout.root))
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
        let mut doc = zero_dom::parse_html(html);
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        // 2. 解析 CSS → Stylesheets（外部 CSS + HTML 内 `<style>`）
        let stylesheets = collect_stylesheets(&doc, css);

        // 3. 计算样式
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let mut styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        // 3.5 把 ::before/::after 伪元素的 content 文本注入为合成文本子节点（doc 每帧
        // 重建，合成节点无累积、JS 不可见）。build_subtree 随后按普通文本子节点测量/绘制。
        inject_pseudo_text_nodes(&mut doc, &mut styles);

        // 4. 计算布局
        let layout_start = Instant::now();
        let img_sizes = self.build_img_intrinsic_sizes(&doc);
        let img_ratios = self.build_img_intrinsic_ratios(&doc);
        let img_no_ratio = self.build_img_intrinsic_no_ratio(&doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(&doc, &styles, img_sizes, img_ratios, img_no_ratio);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 5. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let primitives = painter.into_primitives();
        // 视口剔除 — 移除视口外的图元（高度取文档布局范围，供浏览器滚动消费）
        let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout_result.root);
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
        let img_sizes = self.build_img_intrinsic_sizes(doc);
        let img_ratios = self.build_img_intrinsic_ratios(doc);
        let img_no_ratio = self.build_img_intrinsic_no_ratio(doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(doc, &styles, img_sizes, img_ratios, img_no_ratio);

        // 生成绘制命令
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
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
        let img_sizes = self.build_img_intrinsic_sizes(doc);
        let img_ratios = self.build_img_intrinsic_ratios(doc);
        let img_no_ratio = self.build_img_intrinsic_no_ratio(doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(doc, &styles, img_sizes, img_ratios, img_no_ratio);
        self.cached_layout = Some(LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
        });

        // 仅绘制脏区域内的节点
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_in_rect(&layout_result.root, &styles, &dirty_rect, Some(doc));
        Some(painter.into_primitives())
    }

    /// 在已有 DOM 缓存上重绘整个视口（resize 等场景，走 `incremental_paint`）。
    pub fn repaint_cached_viewport(&mut self, css: &str) -> Option<RenderResult> {
        let doc = self.cached_doc.take()?;
        let dirty = zero_render_foundation::geometry::Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        let stylesheets = collect_stylesheets(&doc, css);
        let primitives = self.incremental_paint(&doc, &stylesheets, dirty)?;
        self.cached_doc = Some(doc);
        let layout_ref = self.cached_layout.as_ref()?;
        let layout = LayoutResult {
            root: layout_ref.root.clone(),
            viewport_width: layout_ref.viewport_width,
            viewport_height: layout_ref.viewport_height,
        };
        Some(RenderResult {
            primitives,
            layout,
            timings: PipelineTimings::default(),
            stats: RenderStats::default(),
        })
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

/// 绘制阶段剔除矩形：宽度仍限视口，高度扩展到完整文档，避免浏览器滚动时丢失页内图元。
pub(crate) fn paint_cull_viewport(
    viewport_w: f32,
    viewport_h: f32,
    layout_root: &zero_layout_engine::LayoutBox,
) -> Rect {
    let doc_h = layout_extent_y(layout_root, 0.0);
    Rect::new(0.0, 0.0, viewport_w, doc_h.max(viewport_h))
}

/// 把 `::before`/`::after` 伪元素的 `content` 文本注入为元素的合成文本子节点。
///
/// 在 `compute_styles` 之后、布局之前调用。`Document` 每次渲染由 `parse_html` 重建，
/// 故合成节点天然无累积、对 JS 不可见。`before` 经 `insert_before` 插为首个子节点
/// （保证渲染在元素内容之前），`after` 经 `append_child` 追加为末子节点。伪元素的
/// `ComputedStyle` 写入 `styles`，使测量/绘制按该样式渲染（颜色、字号等）。
///
/// 复用全部既有机制（文本测量、匿名盒包裹、绘制）——伪元素合成节点即普通文本子节点。
pub(crate) fn inject_pseudo_text_nodes(doc: &mut Document, styles: &mut HashMap<NodeId, ComputedStyle>) {
    use zero_css_parser::values::{DisplayValue, FloatValue, PositionValue};
    use zero_style_system::property::types::ContentComputedValue;

    // 先收集待注入项，避免在遍历 styles 时变更它。
    // (parent, is_before, text, pseudo_style)
    let mut pending: Vec<(NodeId, bool, String, ComputedStyle)> = Vec::new();
    // R1988：content:url() 图片 content（parent, is_before, url, pseudo_style）——注入 `<img>`
    // 元素，图片已由 extract_css_image_urls 抓取+缓存，build_layout_tree 按替换元素处理。
    let mut pending_img: Vec<(NodeId, bool, String, ComputedStyle)> = Vec::new();
    for (&nid, st) in styles.iter() {
        // 解析伪元素 content 的文本：String 直接用；attr(name) 读宿主元素（nid）的
        // 属性值（CSS generated content 的 attr() 函数，如 `content: attr(bgcolor)`
        // 显示属性值）。Counter 由 paint_content 渲染，此处跳过。
        let resolve_text = |content: &ContentComputedValue| -> Option<String> {
            match content {
                ContentComputedValue::String(s) => Some(s.clone()),
                ContentComputedValue::Attr(name) => doc.get(nid).and_then(|n| match &n.kind {
                    zero_dom::NodeKind::Element(elem) => elem.get_attribute(name).as_deref().map(str::to_string),
                    _ => None,
                }),
                _ => None,
            }
        };
        // R1988：content:url() → 图片 url。
        let resolve_url = |content: &ContentComputedValue| -> Option<String> {
            match content {
                ContentComputedValue::Url(u) => Some(u.clone()),
                _ => None,
            }
        };
        if let Some(b) = st.before_pseudo.as_ref() {
            if let Some(t) = resolve_text(&b.content) {
                pending.push((nid, true, t, (**b).clone()));
            } else if let Some(u) = resolve_url(&b.content) {
                pending_img.push((nid, true, u, (**b).clone()));
            }
        }
        if let Some(a) = st.after_pseudo.as_ref() {
            if let Some(t) = resolve_text(&a.content) {
                pending.push((nid, false, t, (**a).clone()));
            } else if let Some(u) = resolve_url(&a.content) {
                pending_img.push((nid, false, u, (**a).clone()));
            }
        }
    }

    for (parent, is_before, text, mut pseudo_style) in pending {
        // content 字段对文本节点测量无意义（测量读 doc 文本）；清为 Normal 防止下游
        // 把合成文本节点误当伪元素再处理。
        pseudo_style.content = ContentComputedValue::Normal;
        // R1307：需要独立盒的伪元素（position != static / float != none / display != inline）
        // 且 content 为空（content:""）→ 创建 ELEMENT 节点（zw-pseudo），让 build_layout_tree
        // 产出正确的 positioned/floated/block 盒。旧 text-node 注入只把 content 作 inline 文本
        // 渲染，忽略伪元素上的 position/float/display/width/height（before-after-positioned-
        // 002/003/004：content:"" + position:fixed/absolute/relative + width/height → 应是
        // 50×100 盒而非空 inline 文本）。非空 content 或 inline-static 伪元素仍走 text-node
        // 路径（保留 102 通过的 generated-content 案 + 不触 content-list 多 token，避 R554
        // net-negative）。kill-switch ZW_PSEUDO_BOX=0。
        let needs_box = std::env::var("ZW_PSEUDO_BOX").as_deref() != Ok("0")
            && text.is_empty()
            && (pseudo_style.position != PositionValue::Static
                || pseudo_style.float != FloatValue::None
                || pseudo_style.display != DisplayValue::Inline);
        let new_id = if needs_box {
            doc.create_element("zw-pseudo")
        } else {
            doc.create_text_node(&text)
        };
        styles.insert(new_id, pseudo_style);
        let inserted = if is_before {
            // before：插为首个子节点（content 渲染在元素内容之前）。
            match doc.get(parent).and_then(|n| n.children.first().copied()) {
                Some(fc) => doc.insert_before(parent, new_id, fc).is_ok(),
                None => doc.append_child(parent, new_id).is_ok(),
            }
        } else {
            doc.append_child(parent, new_id).is_ok()
        };
        if !inserted {
            // 插入失败（如 parent 不存在）则回滚 styles 条目，避免悬空引用。
            styles.remove(&new_id);
        }
    }

    // R1988：content:url() → 注入 `<img src=url>` 元素。图片已由 extract_css_image_urls
    //（property-agnostic 扫所有 url()）在 fetch_image_subresources 抓取+解码+缓存（image_sizes
    // 含其固有尺寸，image_cache 含像素）。build_layout_tree 把 `<img>` 当替换元素按 image_sizes
    // 定尺寸，painter 渲染缓存图。伪元素 content:url()（如 `::before { content: url(icon.png) }`）
    // 渲染为 inline 替换图片。
    for (parent, is_before, url, mut pseudo_style) in pending_img {
        pseudo_style.content = ContentComputedValue::Normal;
        let img_id = doc.create_element("img");
        doc.set_attribute(img_id, "src", &url);
        styles.insert(img_id, pseudo_style);
        let inserted = if is_before {
            match doc.get(parent).and_then(|n| n.children.first().copied()) {
                Some(fc) => doc.insert_before(parent, img_id, fc).is_ok(),
                None => doc.append_child(parent, img_id).is_ok(),
            }
        } else {
            doc.append_child(parent, img_id).is_ok()
        };
        if !inserted {
            styles.remove(&img_id);
        }
    }
}

/// 收集样式表：外部 CSS 字符串 + 文档内 `<style>` 元素文本。
pub(crate) fn collect_stylesheets(doc: &Document, css: &str) -> Vec<Stylesheet> {
    let mut stylesheets = Vec::new();
    if !css.is_empty() {
        stylesheets.push(zero_css_parser::Parser::parse_stylesheet(css));
    }
    for style_id in doc.get_elements_by_tag_name("style") {
        let Some(css_text) = doc.text_content(style_id) else {
            continue;
        };
        let css_text = strip_cdata(css_text.trim());
        if !css_text.is_empty() {
            stylesheets.push(zero_css_parser::Parser::parse_stylesheet(&css_text));
        }
    }
    stylesheets
}

/// 提取 HTML 中所有 `<link rel="stylesheet" href="...">` 的 href 原始值。
///
/// 用于 URL 导航路径下外链样式表的加载（goal doc P1 缺口「外部样式表加载缺失」）：
/// `collect_stylesheets` 仅收集调用方传入 CSS 与文档内 `<style>`，不抓取 `<link>`。
/// 本函数复用 `zero_dom` 解析（DOM 精确，区别于脆弱的正则扫描），返回原始 href
/// 字符串（可能是相对路径）；URL 解析与网络抓取由调用方（webview 层，持有 base URL
/// 与 http client）负责，保持 engine 不直接耦合网络。
///
/// - `rel` 以空白拆分后任一 token 等于 `stylesheet`（大小写不敏感）即匹配，
///   覆盖 `rel="stylesheet"` 与 `rel="stylesheet preload"` 等写法。
/// - 空 href 与 `rel` 不含 stylesheet 的 link（如 icon / preload 非 stylesheet）被忽略。
pub fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut hrefs = Vec::new();
    for link_id in doc.get_elements_by_tag_name("link") {
        let rel = doc.get_attribute(link_id, "rel").unwrap_or_default();
        let is_stylesheet = rel.split_whitespace().any(|t| t.eq_ignore_ascii_case("stylesheet"));
        if !is_stylesheet {
            continue;
        }
        if let Some(href) = doc.get_attribute(link_id, "href") {
            let href = href.trim();
            if !href.is_empty() {
                hrefs.push(href.to_string());
            }
        }
    }
    hrefs
}

/// 页面脚本来源：内联文本或 `<script src>` 原始值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageScript {
    /// 内联经典脚本。
    Inline(String),
    /// 外链经典脚本 `src`（可能为相对 URL）。
    External(String),
    /// 内联 ES module。
    InlineModule(String),
    /// 外链 ES module `src`。
    ExternalModule(String),
}

fn script_type_is_javascript(type_attr: Option<&str>) -> bool {
    match type_attr.map(str::trim).filter(|t| !t.is_empty()) {
        None => true,
        Some(t) if t.eq_ignore_ascii_case("text/javascript") => true,
        Some(t) if t.eq_ignore_ascii_case("application/javascript") => true,
        Some(t) if t.eq_ignore_ascii_case("module") => true,
        Some(t) if t.ends_with("javascript") => true,
        _ => false,
    }
}

fn script_is_module(type_attr: Option<&str>) -> bool {
    type_attr
        .map(|t| t.trim().eq_ignore_ascii_case("module"))
        .unwrap_or(false)
}

/// 按文档顺序提取 `<script>` 内联文本与 `src`。
pub fn extract_page_scripts(html: &str) -> Vec<PageScript> {
    let doc = zero_dom::parse_html(html);
    let mut scripts = Vec::new();
    for script_id in doc.get_elements_by_tag_name("script") {
        let type_attr = doc.get_attribute(script_id, "type");
        if !script_type_is_javascript(type_attr.as_deref()) {
            continue;
        }
        let is_module = script_is_module(type_attr.as_deref());
        if let Some(src) = doc.get_attribute(script_id, "src") {
            let src = src.trim();
            if !src.is_empty() {
                if is_module {
                    scripts.push(PageScript::ExternalModule(src.to_string()));
                } else {
                    scripts.push(PageScript::External(src.to_string()));
                }
                continue;
            }
        }
        if let Some(raw) = doc.text_content(script_id) {
            // XHTML 脚本常以 `<![CDATA[ ... ]]>` 包裹；html5ever 按 HTML 模式解析会把 CDATA
            // 标记作为文本保留。若不剥离，传给 JS 引擎会触发 `SyntaxError: Unexpected token '<'`
            // 致整个脚本失效（函数未定义 → onload 回调再抛 ReferenceError）。CSS21 测试套件
            // 大量 .xht 用 CDATA 包裹脚本（insert-* 动态簇等）。兼容两种写法：裸 `<![CDATA[`
            //（占绝大多数）与 `//<![CDATA[`（JS 注释隐藏，HTML/XHTML 双兼容）。
            let code = strip_script_cdata(raw.trim()).trim();
            if !code.is_empty() {
                if is_module {
                    scripts.push(PageScript::InlineModule(code.to_string()));
                } else {
                    scripts.push(PageScript::Inline(code.to_string()));
                }
            }
        }
    }
    scripts
}

/// 提取 HTML 中所有 `<img src="...">` 的 src 原始值。
///
/// 用于 URL 导航路径下图片子资源的加载（goal doc P1 缺口「图片子资源 / ImageCache
/// 未贯通」）。与 `extract_stylesheet_hrefs` 同模式：复用 `zero_dom` 解析（DOM 精确），
/// 返回原始 src 字符串（可能相对）；URL 解析、抓取与解码由调用方（webview 层）负责。
/// 空 src 过滤；`data:` URI 原样返回（由调用方识别处理）。
pub fn extract_img_srcs(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut srcs = Vec::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        if let Some(src) = doc.get_attribute(img_id, "src") {
            let src = src.trim();
            if !src.is_empty() {
                srcs.push(src.to_string());
            }
        }
    }
    srcs
}

/// `<img>` 子资源（含 lazy 标记）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImgResource {
    /// `src` 原始值。
    pub src: String,
    /// `loading=lazy`。
    pub lazy: bool,
}

/// 提取 HTML 中所有 `<img>` 的 src 与 lazy 属性。
pub fn extract_img_resources(html: &str) -> Vec<ImgResource> {
    let doc = zero_dom::parse_html(html);
    let mut out = Vec::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        if let Some(src) = doc.get_attribute(img_id, "src") {
            let src = src.trim();
            if src.is_empty() {
                continue;
            }
            let lazy = doc
                .get_attribute(img_id, "loading")
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("lazy"));
            out.push(ImgResource {
                src: src.to_string(),
                lazy,
            });
        }
    }
    out
}

/// 从 CSS 文本提取 `@font-face` 中的 `url(...)`（简单扫描）。
pub fn extract_font_face_urls(css: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let lower = css.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(ff) = lower[search_from..].find("@font-face") {
        let start = search_from + ff;
        let block_end = lower[start..].find('}').map(|i| start + i).unwrap_or(css.len());
        let block = &css[start..block_end];
        let mut u = 0;
        while let Some(ui) = block[u..].find("url(") {
            let rest = &block[u + ui + 4..];
            let end = rest.find(')').unwrap_or(rest.len());
            let raw = rest[..end].trim().trim_matches('"').trim_matches('\'');
            if !raw.is_empty() && !raw.starts_with("data:") {
                urls.push(raw.to_string());
            }
            u += ui + 4;
        }
        search_from = block_end;
    }
    urls
}

/// R1794：从 CSS 文本提取所有**图片类** `url(...)` 引用。
///
/// 与 `extract_font_face_urls` 互补：本函数扫描**全部** `url(...)`，但**排除**
/// `@font-face` 块内的 url（字体由 `extract_font_face_urls` 单独处理，避免重复抓取）
/// 与 `data:` URI（调用方识别，此处亦过滤以保持集合干净）。结果去重并保留首次出现顺序。
///
/// 覆盖 `background-image` / `list-style-image` / `border-image-source` 等所有
/// CSS 图片引用——它们都经 `decode_image_bytes` 解码后入 `image_cache`，painter
/// 按 `image_resource_key(url, document_url)` 查找像素。
pub fn extract_css_image_urls(css: &str) -> Vec<String> {
    // 先定位所有 @font-face 块的 [start, end) 区间，扫描时跳过。
    let lower = css.to_ascii_lowercase();
    let mut font_blocks: Vec<(usize, usize)> = Vec::new();
    let mut search_from = 0;
    while let Some(ff) = lower[search_from..].find("@font-face") {
        let start = search_from + ff;
        let end = lower[start..].find('}').map(|i| start + i + 1).unwrap_or(css.len());
        font_blocks.push((start, end));
        search_from = end;
    }
    let in_font_block = |pos: usize| font_blocks.iter().any(|(s, e)| *s <= pos && pos < *e);

    let mut urls: Vec<String> = Vec::new();
    let mut search_from = 0;
    while let Some(ui) = lower[search_from..].find("url(") {
        let lparen = search_from + ui;
        let rest_start = lparen + 4;
        let rest = &css[rest_start..];
        let end = rest.find(')').unwrap_or(rest.len());
        let raw = rest[..end].trim().trim_matches('"').trim_matches('\'');
        // 排除 data: URI、空串、以及位于 @font-face 块内的 url。
        if !raw.is_empty()
            && !raw.starts_with("data:")
            && !in_font_block(lparen)
            && !urls.iter().any(|u: &String| u == raw)
        {
            urls.push(raw.to_string());
        }
        search_from = rest_start + end + 1;
    }
    urls
}

/// R1794：提取 HTML 中所有文档级 CSS 文本并拼接——`<style>` 块 + 元素 inline `style=` 属性。
///
/// 供 `extract_css_image_urls` 扫描图片 `url()` 引用（R1796 起覆盖 inline
/// `style="background-image: url(...)"` 等属性内引用）。与 `extract_img_srcs` 同模式：
/// 复用 `zero_dom` 解析（DOM 精确，比正则稳健）。
pub fn extract_html_style_text(html: &str) -> String {
    let doc = zero_dom::parse_html(html);
    let mut out = String::new();
    for style_id in doc.get_elements_by_tag_name("style") {
        if let Some(text) = doc.text_content(style_id) {
            out.push_str(&text);
            out.push('\n');
        }
    }
    // R1796：inline `style=` 属性值（如 `style="background-image: url(x)"`）亦是 CSS 文本，
    // 收集后交 extract_css_image_urls 扫描。通配 `"*"` 匹配所有元素。
    for elem_id in doc.get_elements_by_tag_name_ns(None, "*") {
        if let Some(style_attr) = doc.get_attribute(elem_id, "style") {
            out.push_str(&style_attr);
            out.push('\n');
        }
    }
    out
}

/// 去除 XHTML CDATA 包装（`<![CDATA[...]]>`）。
///
/// html5ever 仅支持 HTML 模式解析，会将 `<style>` 中的 CDATA 标记
/// 作为文本内容保留。CSS 解析器遇到 `<![CDATA[` 时，错误恢复路径
/// 会贪婪吞噬后续所有 token（`[` 触发 `skip_to_rbracket()`），
/// 导致整个样式表提取 0 条规则。因此必须在传递给 CSS 解析器前去除。
fn strip_cdata(css: &str) -> std::borrow::Cow<'_, str> {
    if let Some(stripped) = css.strip_prefix("<![CDATA[").and_then(|s| s.strip_suffix("]]>")) {
        std::borrow::Cow::Owned(stripped.to_string())
    } else {
        std::borrow::Cow::Borrowed(css)
    }
}

/// 去除 `<script>` 内的 XHTML CDATA 包装，兼容两种写法：
/// - 裸 `<![CDATA[ ... ]]>`（CSS21 .xht 套件绝大多数）
/// - `//<![CDATA[ ... //]]>`（JS 行注释隐藏 CDATA，HTML/XHTML 双兼容写法）
///
/// 与 [`strip_cdata`]（专用于 `<style>` CSS）的区别：脚本侧另需处理 `//` 注释前缀。
/// `//` 不会出现在 CSS CDATA 中（CSS 注释是 `/* */`），故二者独立。
fn strip_script_cdata(code: &str) -> &str {
    let mut s = code;
    if let Some(rest) = s.strip_prefix("//<![CDATA[").or_else(|| s.strip_prefix("<![CDATA[")) {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix("//]]>").or_else(|| s.strip_suffix("]]>")) {
        s = rest;
    }
    s
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

#[cfg(test)]
mod pseudo_tests {
    use super::*;
    use zero_style_system::property::types::ContentComputedValue;

    /// 递归查找首个标签名为 `tag` 的元素 NodeId。
    fn find_element(doc: &Document, id: NodeId, tag: &str) -> Option<NodeId> {
        if let Some(n) = doc.get(id) {
            if let zero_dom::NodeKind::Element(e) = &n.kind
                && e.local_name().eq_ignore_ascii_case(tag)
            {
                return Some(id);
            }
            for child in doc.child_nodes(id) {
                if let Some(found) = find_element(doc, child, tag) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// `inject_pseudo_text_nodes`：::before content 注入为元素的首个文本子节点，
    /// 且其 ComputedStyle 进入 styles 供测量/绘制按伪元素样式渲染。
    #[test]
    fn inject_before_pseudo_as_first_text_child() {
        let html = r#"<html><body><div>X</div></body></html>"#;
        let mut doc = zero_dom::parse_html(html);
        // 找到 div
        let div = find_element(&doc, doc.root(), "div").expect("div 存在");
        let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.before_pseudo = Some(Box::new(ComputedStyle {
            content: ContentComputedValue::String("Y".to_string()),
            ..ComputedStyle::default()
        }));
        styles.insert(div, div_style);

        inject_pseudo_text_nodes(&mut doc, &mut styles);

        // div 的首个子节点应为注入的文本节点 "Y"
        let first_child = doc
            .get(div)
            .and_then(|n| n.children.first().copied())
            .expect("有子节点");
        match &doc.get(first_child).unwrap().kind {
            zero_dom::NodeKind::Text(t) => assert_eq!(t.content, "Y"),
            other => panic!("首个子节点应为文本节点，实际 {other:?}"),
        }
        // 注入节点的样式已进入 styles（content 被清为 Normal）
        let inj_style = styles.get(&first_child).expect("注入节点有样式");
        assert!(matches!(inj_style.content, ContentComputedValue::Normal));
        // 原 "X" 文本节点仍是 div 的第二个子节点
        let second = doc.get(div).unwrap().children.get(1).copied();
        assert!(matches!(
            doc.get(second.unwrap()).map(|n| &n.kind),
            Some(zero_dom::NodeKind::Text(_))
        ));
    }

    /// R1988：::before `content: url(icon.png)` → 注入 `<img src="icon.png">` 元素
    ///（非文本节点）。图片已由 extract_css_image_urls 抓取+缓存（fetch_image_subresources），
    /// build_layout_tree 按替换元素处理，painter 渲染缓存图。
    #[test]
    fn inject_before_pseudo_url_as_img_element() {
        let html = r#"<html><body><div>X</div></body></html>"#;
        let mut doc = zero_dom::parse_html(html);
        let div = find_element(&doc, doc.root(), "div").expect("div 存在");
        let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.before_pseudo = Some(Box::new(ComputedStyle {
            content: ContentComputedValue::Url("icon.png".to_string()),
            ..ComputedStyle::default()
        }));
        styles.insert(div, div_style);

        inject_pseudo_text_nodes(&mut doc, &mut styles);

        let first_child = doc
            .get(div)
            .and_then(|n| n.children.first().copied())
            .expect("有子节点");
        match &doc.get(first_child).unwrap().kind {
            zero_dom::NodeKind::Element(e) => {
                assert_eq!(e.local_name(), "img", "应为 img 元素");
                assert_eq!(e.get_attribute("src").as_deref(), Some("icon.png"));
            }
            other => panic!("首个子节点应为 img 元素，实际 {other:?}"),
        }
        let inj_style = styles.get(&first_child).expect("注入节点有样式");
        assert!(matches!(inj_style.content, ContentComputedValue::Normal));
    }

    /// R1307：需要独立盒的伪元素（position != static / float / display != inline）且
    /// content 为空（content:""）→ 注入 ELEMENT 节点（zw-pseudo），让 build_layout_tree
    /// 产出 positioned/floated/block 盒。旧路径注入 text node，忽略 position/float/display/
    /// width/height（before-after-positioned-002/003/004）。default-on PASS / kill=0 回退
    /// text node（首个子为 Text）。load-bearing：守护 element-node 路径。
    #[test]
    fn inject_positioned_empty_pseudo_as_element_node() {
        use zero_css_parser::values::PositionValue;
        let html = r#"<html><body><div>X</div></body></html>"#;
        let mut doc = zero_dom::parse_html(html);
        let div = find_element(&doc, doc.root(), "div").expect("div 存在");
        let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.before_pseudo = Some(Box::new(ComputedStyle {
            content: ContentComputedValue::String(String::new()),
            position: PositionValue::Absolute,
            ..ComputedStyle::default()
        }));
        styles.insert(div, div_style);

        inject_pseudo_text_nodes(&mut doc, &mut styles);

        let first_child = doc
            .get(div)
            .and_then(|n| n.children.first().copied())
            .expect("有子节点");
        // R1307 default-on：positioned empty-content 伪元素 = ELEMENT 节点（非 text）。
        match &doc.get(first_child).unwrap().kind {
            zero_dom::NodeKind::Element(_) => {}
            other => panic!("R1307: positioned empty-content 伪元素应为 ELEMENT 节点，实际 {other:?}"),
        }
        let inj_style = styles.get(&first_child).expect("注入元素有样式");
        assert_eq!(
            inj_style.position,
            PositionValue::Absolute,
            "zw-pseudo 元素须携带 position:absolute"
        );
    }

    /// `inject_pseudo_text_nodes`：::after 追加为末子节点；content:none 不注入。
    #[test]
    fn inject_after_pseudo_and_skip_none() {
        let html = r#"<html><body><div>X</div></body></html>"#;
        let mut doc = zero_dom::parse_html(html);
        let div = find_element(&doc, doc.root(), "div").unwrap();
        let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
        let mut div_style = ComputedStyle::default();
        // after = String("Z"); before = None（content:none 不应触发——这里直接测 after）
        div_style.after_pseudo = Some(Box::new(ComputedStyle {
            content: ContentComputedValue::String("Z".to_string()),
            ..ComputedStyle::default()
        }));
        styles.insert(div, div_style);

        inject_pseudo_text_nodes(&mut doc, &mut styles);

        let children = &doc.get(div).unwrap().children;
        // 末子节点应为 "Z"
        let last = *children.last().unwrap();
        match &doc.get(last).unwrap().kind {
            zero_dom::NodeKind::Text(t) => assert_eq!(t.content, "Z"),
            other => panic!("末子节点应为文本节点，实际 {other:?}"),
        }
        // 首子节点仍是原 "X"
        let first = *children.first().unwrap();
        match &doc.get(first).unwrap().kind {
            zero_dom::NodeKind::Text(t) => assert_eq!(t.content, "X"),
            _ => panic!("首子节点应仍是原文本 X"),
        }
    }

    #[test]
    fn extract_page_scripts_collects_inline_and_external() {
        let html = r#"<html><body>
            <script>var a = 1;</script>
            <script src="app.js"></script>
            <script type="text/plain">skip</script>
        </body></html>"#;
        let scripts = extract_page_scripts(html);
        assert_eq!(scripts.len(), 2);
        assert!(matches!(&scripts[0], PageScript::Inline(s) if s.contains("var a")));
        assert!(matches!(&scripts[1], PageScript::External(s) if s == "app.js"));
    }

    #[test]
    fn extract_page_scripts_collects_es_modules() {
        let html = r#"<html><body>
            <script type="module">export const x = 1;</script>
            <script type="module" src="main.mjs"></script>
        </body></html>"#;
        let scripts = extract_page_scripts(html);
        assert_eq!(scripts.len(), 2);
        assert!(matches!(&scripts[0], PageScript::InlineModule(s) if s.contains("export")));
        assert!(matches!(&scripts[1], PageScript::ExternalModule(s) if s == "main.mjs"));
    }

    /// XHTML 脚本常以 `<![CDATA[ ... ]]>` 包裹（CSS21 测试套件 .xht 大量使用）。
    /// html5ever 按 HTML 模式解析会把 CDATA 标记作为文本保留；若不剥离，传给 JS 引擎
    /// 会触发 `SyntaxError: Unexpected token '<'` 致整个脚本失效。回归守护剥离行为。
    #[test]
    fn extract_page_scripts_strips_xhtml_cdata_wrapper() {
        let html = r#"<html><body>
            <script type="text/javascript">//<![CDATA[
                function f() { return 1; }
            //]]></script>
        </body></html>"#;
        let scripts = extract_page_scripts(html);
        assert_eq!(scripts.len(), 1);
        match &scripts[0] {
            PageScript::Inline(s) => {
                assert!(!s.contains("<![CDATA["), "CDATA 起始标记应被剥离，得到: {s}");
                assert!(!s.contains("]]>"), "CDATA 结束标记应被剥离，得到: {s}");
                assert!(s.contains("function f()"), "脚本体应保留: {s}");
            }
            other => panic!("应为 Inline，得到 {other:?}"),
        }
    }
}

#[cfg(test)]
mod css_image_url_tests {
    use super::*;

    #[test]
    fn extract_css_image_urls_collects_background_and_list() {
        let css = r#"
            .hero { background-image: url("/img/bg.png"); }
            ul { list-style-image: url('bullet.png'); }
        "#;
        let urls = extract_css_image_urls(css);
        assert_eq!(urls, vec!["/img/bg.png", "bullet.png"]);
    }

    #[test]
    fn extract_css_image_urls_skips_font_face_and_data() {
        let css = r#"
            @font-face { font-family: x; src: url(font.woff2) format('woff2'); }
            .a { background-image: url(real.png); }
            .b { background-image: url(data:image/png;base64,iVBOR=); }
        "#;
        let urls = extract_css_image_urls(css);
        // 字体 url 与 data: URI 均排除，仅保留 real.png。
        assert_eq!(urls, vec!["real.png"]);
    }

    #[test]
    fn extract_css_image_urls_dedupes() {
        let css = "a{background-image:url(a.png)}b{background-image:url(a.png)}";
        let urls = extract_css_image_urls(css);
        assert_eq!(urls, vec!["a.png"]);
    }

    #[test]
    fn extract_css_image_urls_empty_when_no_url() {
        assert!(extract_css_image_urls(".a { color: red }").is_empty());
        assert!(extract_css_image_urls("").is_empty());
    }

    #[test]
    fn extract_html_style_text_collects_inline_blocks() {
        let html = r#"<html><head>
            <style>.a { background-image: url(a.png) }</style>
            </head><body>
            <style>.b { background-image: url(b.png) }</style>
            <p>not css</p>
            </body></html>"#;
        let text = extract_html_style_text(html);
        assert!(text.contains("a.png"), "应含首个 style 块: {text}");
        assert!(text.contains("b.png"), "应含第二个 style 块: {text}");
    }

    /// R1796：inline `style=` 属性内的 CSS（含 `url()`）亦被收集。
    #[test]
    fn extract_html_style_text_collects_inline_style_attrs() {
        let html = r#"<html><body>
            <div style="background-image: url('hero.png')"></div>
            <span style="color: red">no image</span>
            </body></html>"#;
        let text = extract_html_style_text(html);
        assert!(text.contains("hero.png"), "应含 inline style= 属性内 url(): {text}");
    }

    #[test]
    fn extract_html_style_text_empty_when_no_style() {
        let html = "<html><body><p>no styles</p></body></html>";
        assert!(extract_html_style_text(html).is_empty());
    }

    /// R1796：端到端——inline `style=` 中的 `url()` 经 extract_css_image_urls 提取。
    #[test]
    fn extract_css_image_urls_from_inline_style_attr() {
        let html = r#"<html><body>
            <div style="background-image: url(bg.png)"></div>
            </body></html>"#;
        let css = extract_html_style_text(html);
        let urls = extract_css_image_urls(&css);
        assert_eq!(urls, vec!["bg.png"]);
    }
}
