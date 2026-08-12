//! 渲染管线 — 编排 HTML→CSS→Layout→Paint 全流程。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use slotmap::Key;
use zero_css_parser::Stylesheet;
use zero_css_parser::media_query::PrefersColorSchemeValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::{LayoutEngine, LayoutResult};
use zero_render_foundation::color::Color;
use zero_render_foundation::display_list::DisplayList;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{RenderPrimitives, RenderStats};
use zero_style_system::ComputedStyle;
use zero_style_system::StyleSystem;

use crate::animation::AnimationClock;
use crate::dirty::DirtyTracker;
use crate::hit_test;
use crate::paint::Painter;
use crate::transition::TransitionClock;

mod extract;
pub use extract::*;

/// 动画事件类型（R3249/R3250/R3251）——区分 animationend（完成）/ animationiteration（迭代边界）/
/// animationstart（启动），CSS Animations 事件族。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEventKind {
    /// animationstart（CSS Animations §animationstart）——动画首次进入活跃间隔派发（elapsedTime=0）。
    Start,
    /// animationend（CSS Animations §animationend）——有限动画完成时派发。
    End,
    /// animationiteration（CSS Animations §animationiteration）——迭代边界派发（infinite 动画循环回调）。
    Iteration,
}

impl AnimationEventKind {
    /// 映射到 `AnimationEvent` 构造器的事件类型字符串（`new AnimationEvent(as_event_type(), {...})`）。
    /// 三类动画事件的 init dict 完全相同（{animationName, elapsedTime, bubbles}），仅事件名不同
    /// （CSS Animations §animationstart / §animationend / §animationiteration）。
    pub fn as_event_type(self) -> &'static str {
        match self {
            AnimationEventKind::Start => "animationstart",
            AnimationEventKind::End => "animationend",
            AnimationEventKind::Iteration => "animationiteration",
        }
    }
}

/// 过渡事件类型（R3248 end + R3252 run/start，CSS Transitions 事件族）——区分 transitionrun（创建）/
/// transitionstart（delay 过后活跃）/ transitionend（完成）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionEventKind {
    /// transitionrun（CSS Transitions §transitionrun）——过渡被创建即派发（可能在 delay 期）。
    Run,
    /// transitionstart（CSS Transitions §transitionstart）——delay 过后首次进入活跃间隔派发（elapsedTime=0）。
    Start,
    /// transitionend（CSS Transitions §transitionend）——过渡完成派发（elapsedTime=duration）。
    End,
}

impl TransitionEventKind {
    /// 映射到 `TransitionEvent` 构造器的事件类型字符串（`new TransitionEvent(as_event_type(), {...})`）。
    /// 三类过渡事件的 init dict 完全相同（{propertyName, elapsedTime, bubbles}），仅事件名不同
    /// （CSS Transitions §transitionrun / §transitionstart / §transitionend）。
    pub fn as_event_type(self) -> &'static str {
        match self {
            TransitionEventKind::Run => "transitionrun",
            TransitionEventKind::Start => "transitionstart",
            TransitionEventKind::End => "transitionend",
        }
    }
}

/// 「已派发待消费」的过渡事件（R3248 end + R3252 run/start）——宿主据此向 JS 派发 `TransitionEvent`。
#[derive(Debug, Clone)]
pub struct TransitionEvent {
    /// 事件类型（Run → transitionrun / Start → transitionstart / End → transitionend）。
    pub kind: TransitionEventKind,
    /// 元素选择器（unique_selector_for_node 产出）。
    pub selector: String,
    /// 属性名（propertyName）。
    pub property: String,
    /// 时长（elapsedTime，秒；run/start=0，end=duration）。
    pub elapsed: f64,
}

/// 「已派发待消费」的动画事件（R3249/R3250）——宿主据此向 JS 派发 `AnimationEvent`。
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    /// 事件类型（End → animationend / Iteration → animationiteration）。
    pub kind: AnimationEventKind,
    /// 元素选择器（unique_selector_for_node 产出）。
    pub selector: String,
    /// 动画名（animationName）。
    pub name: String,
    /// 时长（elapsedTime，秒）。
    pub elapsed: f64,
}

/// 渲染管线 — 编排 HTML→CSS→Layout→Paint 全流程。
///
/// 整合 DOM 解析、CSS 解析、样式计算、布局计算和绘制命令生成，
/// 提供完整的端到端渲染能力。
pub struct RenderPipeline {
    /// R3268 canvas 显示链路：CanvasRegistry（JS getContext 与 painter 共享）。
    pub(crate) canvas_registry: Option<std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>>>,
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
    /// 「已派发待消费」的过渡事件（R3248 transitionend + R3252 transitionrun/transitionstart）——元素选择器、
    /// 事件类型（Run/Start/End）、属性名（propertyName）、时长（elapsedTime）。过渡创建/启动/完成帧由
    /// `start_transitions` + `tick()` 收集 + `unique_selector_for_node` 映射元素后存此；宿主经
    /// `take_pending_transition_events()` 取出派发。
    pending_transition_events: Vec<TransitionEvent>,
    /// 「已派发待消费」的动画事件（R3249 animationend + R3250 animationiteration + R3251 animationstart）——
    /// 元素选择器、事件类型（Start/End/Iteration）、动画名（animationName）、时长（elapsedTime）。动画
    /// 启动/完成/迭代边界帧由 `tick()` 收集 + `unique_selector_for_node` 映射元素后存此；宿主经
    /// `take_pending_animation_events()` 取出派发。统一 enum 避免并行 channel 重复管线（transition 见
    /// `pending_transition_events`）。
    pending_animation_events: Vec<AnimationEvent>,
    /// 缓存的基础样式（用于过渡检测，存储覆盖前的原始计算样式）。
    cached_styles: HashMap<NodeId, ComputedStyle>,
    /// 文本表单控件的页面级当前值，独立于 HTML 内容属性。
    form_control_values: HashMap<NodeId, String>,
    /// 文本表单控件尚未提交的 IME preedit。
    pub(crate) form_control_compositions: HashMap<NodeId, (String, usize, usize)>,
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
    /// 文档 referrer（来源页 URL；`document.referrer` 读，导航层注入 = 导航前的页面 URL）。
    pub(crate) referrer: Option<String>,
    /// 缓存的布局结果。
    pub(crate) cached_layout: Option<LayoutResult>,
    /// 缓存的 DOM（用于命中测试）。`Rc<RefCell<Document>>` 共享（P1b L1a，R3106）——
    /// 原生 DOM 绑定（engine::dom_bindings）经 [`Self::cached_doc_shared`] 取同一句柄，
    /// 读/写同一 live Document（闭合 R3097 read-only 快照限制；详见
    /// `docs/specs/p1b-v8-native-bindings-rfc.md` §3.7）。`RefCell` 单线程顺序 borrow
    /// （脚本执行与渲染顺序，无并发 borrow_mut）。
    pub(crate) cached_doc: Option<Rc<RefCell<Document>>>,
    /// CSS 解析缓存（repaint_cached_viewport 路径）：外部 css 文本与对应 stylesheets。
    ///
    /// `render_incremental`（resize/color-scheme/media 帧）每帧调用
    /// `repaint_cached_viewport` 重新 `collect_stylesheets`——tokenize+parse 相同 CSS
    /// 文本是纯浪费（动画/交互帧每帧重复）。文本相同（DOM 未变——见失效点）直接复用
    /// 解析结果。`render_html` 族替换 cached_doc 时置 None 失效。
    pub(crate) cached_css_text: Option<String>,
    /// 与 `cached_css_text` 配对的解析结果。
    pub(crate) cached_stylesheets: Vec<Stylesheet>,
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
    /// 本次渲染执行 HTML 解析的次数。
    pub parse_count: u32,
    /// 本次渲染执行样式计算的次数。
    pub style_count: u32,
    /// 本次渲染执行布局计算的次数。
    pub layout_count: u32,
    /// 本次渲染执行绘制的次数。
    pub paint_count: u32,
}

/// 渲染结果 — 包含 display list、布局、计时和统计信息。
pub struct RenderResult {
    /// 本帧 display list（图元 + 脏区域）。
    pub display_list: DisplayList,
    /// 布局结果。
    pub layout: LayoutResult,
    /// 各阶段计时。
    pub timings: PipelineTimings,
    /// 渲染统计信息（draw call 估算、图元数量、剔除数量）。
    pub stats: RenderStats,
    /// R3268 canvas 显示链路：本帧 canvas 像素快照（ctx_id, w, h, rgba），
    /// 调用方在渲染前注入 ImageCache（图元 image_key = ctx_id）。
    pub canvas_images: Vec<(u64, u32, u32, Vec<u8>)>,
}

impl RenderResult {
    /// 本帧图元序列（`display_list.primitives` 的便捷访问）。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.display_list.primitives
    }
}

/// 从图元与脏区域组装 [`RenderResult`]（S1 DisplayList 契约）。
pub(crate) fn make_render_result(
    primitives: RenderPrimitives,
    dirty_rects: Vec<(f32, f32, f32, f32)>,
    layout: LayoutResult,
    timings: PipelineTimings,
    mut stats: RenderStats,
    canvas_images: Vec<(u64, u32, u32, Vec<u8>)>,
) -> RenderResult {
    let display_list = DisplayList::new(primitives, dirty_rects.clone());
    display_list.apply_stats_dirty_rects(&mut stats);
    RenderResult {
        display_list,
        layout,
        timings,
        stats,
        canvas_images,
    }
}

/// 从布局树收集指定 DOM 节点的脏矩形（视口 CSS 像素）。
fn layout_dirty_rects_for_nodes(
    root: &zero_layout_engine::LayoutBox,
    node_ids: &[NodeId],
    viewport_w: f32,
    viewport_h: f32,
) -> Vec<(f32, f32, f32, f32)> {
    use std::collections::HashSet;
    let targets: HashSet<NodeId> = node_ids.iter().copied().collect();
    let mut out = Vec::new();
    collect_node_dirty_rects(root, 0.0, 0.0, &targets, &mut out);
    if out.is_empty() {
        vec![(0.0, 0.0, viewport_w, viewport_h)]
    } else {
        out
    }
}

fn collect_node_dirty_rects(
    layout_box: &zero_layout_engine::LayoutBox,
    offset_x: f32,
    offset_y: f32,
    targets: &std::collections::HashSet<NodeId>,
    out: &mut Vec<(f32, f32, f32, f32)>,
) {
    let abs_x = offset_x + layout_box.x;
    let abs_y = offset_y + layout_box.y;
    if layout_box.node_id.is_some_and(|id| targets.contains(&id)) {
        out.push((abs_x, abs_y, layout_box.width, layout_box.height));
    }
    let child_ox = abs_x + layout_box.content_x;
    let child_oy = abs_y + layout_box.content_y;
    for child in &layout_box.children {
        collect_node_dirty_rects(child, child_ox, child_oy, targets, out);
    }
}

impl RenderPipeline {
    /// R3268：设置 CanvasRegistry（宿主创建，与 register_dom_callbacks 共享同一实例——
    /// canvas 显示链路：JS getContext 写入的像素经 painter 桥接为图元）。
    pub fn set_canvas_registry(
        &mut self,
        registry: Option<std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>>>,
    ) {
        self.canvas_registry = registry;
    }

    /// 创建新的渲染管线。
    ///
    /// # 参数
    ///
    /// - `viewport_width` — 视口宽度（像素）
    /// - `viewport_height` — 视口高度（像素）
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            canvas_registry: None,
            viewport_width,
            viewport_height,
            style_system: StyleSystem::new(),
            layout_engine: LayoutEngine::new(viewport_width, viewport_height),
            dirty_tracker: DirtyTracker::new(),
            animation_clock: AnimationClock::new(),
            transition_clock: TransitionClock::new(),
            pending_transition_events: Vec::new(),
            pending_animation_events: Vec::new(),
            cached_styles: HashMap::new(),
            form_control_values: HashMap::new(),
            form_control_compositions: HashMap::new(),
            cached_layout: None,
            cached_doc: None,
            cached_css_text: None,
            cached_stylesheets: Vec::new(),
            skip_indicators: false,
            image_sizes: HashMap::new(),
            image_ratios: HashMap::new(),
            image_no_ratio: HashMap::new(),
            font_resolver: HashMap::new(),
            document_url: None,
            referrer: None,
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

    /// 设置文档 referrer（导航时由 webview 传入 = 导航前的页面 URL，供 `document.referrer` 读）。
    pub fn set_referrer(&mut self, referrer: Option<&str>) {
        self.referrer = referrer.map(str::to_string);
    }

    /// 文档 referrer（来源页 URL）。
    pub fn referrer(&self) -> Option<&str> {
        self.referrer.as_deref()
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

    /// 一次 DOM 遍历同时构建三个 img 固有尺寸信号（旧实现 3 次全树遍历）。
    pub(crate) fn build_img_intrinsic_all(
        &self,
        doc: &Document,
    ) -> (
        HashMap<NodeId, (f32, f32)>,
        HashMap<NodeId, f32>,
        HashMap<NodeId, (Option<f32>, Option<f32>)>,
    ) {
        let mut sizes = HashMap::new();
        let mut ratios = HashMap::new();
        let mut no_ratio = HashMap::new();
        for img_id in doc.get_elements_by_tag_name("img") {
            if let Some(src) = doc.get_attribute(img_id, "src") {
                let key = crate::paint::image_resource_key(&src, self.document_url.as_deref());
                if let Some(&size) = self.image_sizes.get(&key) {
                    sizes.insert(img_id, size);
                }
                if let Some(&ratio) = self.image_ratios.get(&key) {
                    ratios.insert(img_id, ratio);
                }
                if let Some(&dims) = self.image_no_ratio.get(&key) {
                    no_ratio.insert(img_id, dims);
                }
            }
        }
        (sizes, ratios, no_ratio)
    }

    /// R2439：`content:url()` 普通元素 element-becomes-replaced 的 sizing pass。
    ///
    /// 对 `content:url(...)` 的普通元素（非 `<img>` 自身），按 image 固有尺寸（`image_sizes`
    /// 经 `extract_css_image_urls` property-agnostic 抓取）设置 width/height（仅 Auto 侧），
    /// 使元素盒自身 sizing 为图尺寸——build_subtree 已抑制其子节点，paint_img_element 渲染图片。
    /// 绕 R109 IFC（IFC 不测 inline replaced img，见 R2438 child-injection 证伪）。
    /// kill-switch `ZW_CONTENT_REPLACE=0`。
    pub(crate) fn apply_content_url_replaced_sizing(&self, styles: &mut HashMap<NodeId, ComputedStyle>) {
        if std::env::var("ZW_CONTENT_REPLACE").as_deref() == Ok("0") {
            return;
        }
        use zero_css_parser::values::LengthValue;
        use zero_style_system::property::types::ContentComputedValue;
        // 先收集（nid, w, h），避免迭代 styles 时 mutate 借用冲突。
        // content==Url 已 gate：正常 `<img src=x>`（content Normal）不受影响；仅 content:url
        // 元素（含 content:url 的 `<img>`，on-replaced-element）触发——content:url 覆盖 src。
        let mut targets: Vec<(NodeId, f32, f32)> = Vec::new();
        for (&nid, st) in styles.iter() {
            if let ContentComputedValue::Url(u) = &st.content {
                let key = crate::paint::image_resource_key(u, self.document_url.as_deref());
                if let Some(&(w, h)) = self.image_sizes.get(&key)
                    && w > 0.0
                    && h > 0.0
                {
                    targets.push((nid, w, h));
                }
            }
        }
        for (nid, w, h) in targets {
            if let Some(st) = styles.get_mut(&nid) {
                if matches!(st.width, LengthValue::Auto) {
                    st.width = LengthValue::Px(w.into());
                }
                if matches!(st.height, LengthValue::Auto) {
                    st.height = LengthValue::Px(h.into());
                }
            }
        }
    }

    /// 设置 CSS font-family 查找表。
    ///
    /// 由调用方从 `FontLoader::build_font_resolver()` 构建并传入。
    /// 用于将 CSS font-family 列表解析为具体的 FontId。
    pub fn set_font_resolver(&mut self, resolver: HashMap<String, u32>) {
        self.layout_engine.set_font_resolver(resolver.clone());
        if std::env::var("ZW_SHAPED_TEXT").as_deref() != Ok("0")
            && std::env::var("ZW_SHAPED_LAYOUT").as_deref() != Ok("0")
        {
            self.layout_engine
                .set_advance_source(std::rc::Rc::new(crate::text_metrics::ShapedAdvanceSource));
        }
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
        // R1999：同步传到 layout_engine，使 layout 层可判 Print 模式触发分页 post-process
        //（cascade 已用 style_system.media_type；layout 此前无 media 感知——R1998 缺口）。
        self.layout_engine.set_media_type(media_type);
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

    /// 取出「自上次 render 后新产生」的过渡事件（R3248 transitionend + R3252 transitionrun/transitionstart）。
    /// 返回 [`TransitionEvent`] 列表（`kind` 区分 Run/Start/End）；每次调用清空缓冲。宿主据此向 JS 派发
    /// `new TransitionEvent(kind.as_event_type(), {propertyName, elapsedTime, bubbles:true})`。
    pub fn take_pending_transition_events(&mut self) -> Vec<TransitionEvent> {
        std::mem::take(&mut self.pending_transition_events)
    }

    /// 取出「自上次 render 后新产生」的动画事件（R3249 animationend + R3250 animationiteration + R3251
    /// animationstart）。返回 [`AnimationEvent`] 列表（`kind` 区分 Start/End/Iteration）；每次调用清空缓冲。
    /// 宿主据此向 JS 派发 `new AnimationEvent(kind.as_event_type(), {animationName, elapsedTime, bubbles:true})`。
    pub fn take_pending_animation_events(&mut self) -> Vec<AnimationEvent> {
        std::mem::take(&mut self.pending_animation_events)
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
        // R2010/R2011/R2013：从 `@page { size; margin }` 解析 Print 页几何注入 layout_engine（Screen 零影响）。
        let (page_w, page_h, m_top, m_right, m_bottom, m_left) = extract_print_page_geometry(&stylesheets);
        self.layout_engine.set_print_page_height(page_h);
        self.layout_engine.set_print_page_width(page_w);
        self.layout_engine.set_print_page_margins(m_top, m_bottom);
        self.layout_engine.set_print_horizontal_margins(m_left, m_right);

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
        // R3249（animationend）/R3250（animationiteration）/R3251（animationstart）：apply_animation_overrides
        // 返「本轮新产生」动画事件 → 映射 NodeId→unique_selector，存 pending_animation_events 待宿主派发
        // （Start→animationstart / End→animationend / Iteration→animationiteration）。
        for (nid, kind, name, elapsed) in
            apply_animation_overrides(&mut self.animation_clock, &mut styles, current_time)
        {
            if let Some(sel) = crate::js_dom_bridge::unique_selector_for_node(&doc, nid) {
                self.pending_animation_events.push(AnimationEvent {
                    kind,
                    selector: sel,
                    name,
                    elapsed,
                });
            }
        }

        // 5b. 应用活跃的过渡插值
        let node_ids: Vec<NodeId> = styles.keys().copied().collect();
        for nid in &node_ids {
            let key = nid.data().as_ffi();
            let props = self.transition_clock.tick(key, current_time);
            if !props.is_empty()
                && let Some(s) = styles.get_mut(nid)
            {
                TransitionClock::apply_to_computed_style(&props, s);
            }
        }
        // R3248（§transitionend）/R3252（§transitionrun/§transitionstart）：收集「本轮新产生」过渡事件 →
        // 映射元素（element_key=u64 → NodeId，经 node_ids 建 key→nid 索引）→ unique_selector_for_node 转
        // selector，存 pending 待宿主派发。drain 顺序 Run → Start → End（spec 派发序：transitionrun 先于
        // transitionstart 先于 transitionend）。cleanup_finished 在此之后移除已完成 transition（不触
        // just_finished——drain 已先取）。just_run 由 §4b start_transitions 填充；just_started/just_finished
        // 由本节 tick 填充，故三者在同一收集块 drain。
        {
            let mut key_to_nid: HashMap<u64, NodeId> = HashMap::new();
            for nid in &node_ids {
                key_to_nid.insert(nid.data().as_ffi(), *nid);
            }
            let map_sel = |ek: &u64| -> Option<(NodeId, String)> {
                let nid = key_to_nid.get(ek)?;
                let sel = crate::js_dom_bridge::unique_selector_for_node(&doc, *nid)?;
                Some((*nid, sel))
            };
            // transitionrun（创建）——elapsedTime=0。
            for r in self.transition_clock.drain_just_run() {
                if let Some((_, sel)) = map_sel(&r.element_key) {
                    self.pending_transition_events.push(TransitionEvent {
                        kind: TransitionEventKind::Run,
                        selector: sel,
                        property: r.property,
                        elapsed: 0.0,
                    });
                }
            }
            // transitionstart（delay 过后活跃）——elapsedTime=0。
            for s in self.transition_clock.drain_just_started() {
                if let Some((_, sel)) = map_sel(&s.element_key) {
                    self.pending_transition_events.push(TransitionEvent {
                        kind: TransitionEventKind::Start,
                        selector: sel,
                        property: s.property,
                        elapsed: 0.0,
                    });
                }
            }
            // transitionend（完成）——elapsedTime=duration。
            for fin in self.transition_clock.drain_just_finished() {
                if let Some((_, sel)) = map_sel(&fin.element_key) {
                    self.pending_transition_events.push(TransitionEvent {
                        kind: TransitionEventKind::End,
                        selector: sel,
                        property: fin.property,
                        elapsed: fin.duration,
                    });
                }
            }
        }
        self.transition_clock.cleanup_finished();

        // 6. 计算布局
        let layout_start = Instant::now();
        let (img_sizes, img_ratios, img_no_ratio) = self.build_img_intrinsic_all(&doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(&doc, &styles, img_sizes, img_ratios, img_no_ratio);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 7. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.set_canvas_registry(self.canvas_registry.clone());
        painter.register_counter_styles(&stylesheets);
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let canvas_images = painter.canvas_images.clone();
        let mut primitives = painter.into_primitives();
        let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout_result.root);
        // S7b：cull_invisible 原位剔除（primitives 变量即结果，不再返回新对象）
        let stats = primitives.cull_invisible(viewport);
        // 性能门禁优化 S7（2026-08-08）：draw_order 路径下 batch_fills 是纯 clone
        // no-op（ops.rs:273-275），跳过免全量克隆（4400 元素页每帧 ~11k fills）
        let primitives = if primitives.draw_order.is_empty() {
            primitives.batch_fills()
        } else {
            primitives
        };
        let dirty_rects = vec![(0.0, 0.0, viewport.size.width, viewport.size.height)];
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;

        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.form_control_values.clear();
        self.form_control_compositions.clear();
        self.cached_doc = Some(Rc::new(RefCell::new(doc)));
        // DOM 已替换：CSS 解析缓存失效（新文档的 <style>/meta 内容可能不同）。
        self.cached_css_text = None;

        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
        };
        self.cached_layout = Some(layout_result);

        make_render_result(
            primitives,
            dirty_rects,
            layout,
            PipelineTimings {
                parse_ms,
                style_ms,
                layout_ms,
                paint_ms,
                total_ms,
                parse_count: 1,
                style_count: 1,
                layout_count: 1,
                paint_count: 1,
            },
            stats,
            canvas_images,
        )
    }

    /// 命中测试链接，返回点击位置处 `<a href>` 的目标 URL。
    pub fn hit_test_link(&self, x: f32, y: f32) -> Option<String> {
        let doc = self.cached_doc.as_ref()?.borrow();
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_link(&doc, &layout.root, x, y)
    }

    /// 命中测试图片，返回 `src`（文档原始值）。
    pub fn hit_test_image(&self, x: f32, y: f32) -> Option<String> {
        let doc = self.cached_doc.as_ref()?.borrow();
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_image(&doc, &layout.root, x, y)
    }

    /// 命中测试元素，返回点击位置处最深元素及其布局盒。
    pub fn hit_test_element(&self, x: f32, y: f32) -> Option<hit_test::ElementHit> {
        let doc = self.cached_doc.as_ref()?.borrow();
        let layout = self.cached_layout.as_ref()?;
        hit_test::hit_test_element(&doc, &layout.root, x, y)
    }

    /// 构建主线程只读命中测试快照（与当前缓存 DOM/布局一致）。
    pub fn build_hit_test_cache(&self) -> Option<hit_test::HitTestCache> {
        let doc = self.cached_doc.as_ref()?.borrow();
        let layout = self.cached_layout.as_ref()?;
        Some(hit_test::HitTestCache::from_document(&doc, &layout.root))
    }

    /// 取缓存 live Document 的共享句柄（`Rc<RefCell<Document>>` 克隆）。
    ///
    /// P1b L1a（R3106）：原生 DOM 绑定（engine::dom_bindings）经此取**同一** live Document，
    /// 读/写直接反映渲染状态（闭合 R3097 read-only 快照限制）。无缓存（未渲染）→ `None`，
    /// 调用方回落 re-parse 快照。详见 `docs/specs/p1b-v8-native-bindings-rfc.md` §3.7。
    pub fn cached_doc_shared(&self) -> Option<Rc<RefCell<Document>>> {
        self.cached_doc.clone()
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
        // R2010/R2011/R2013：从 `@page { size; margin }` 解析 Print 页几何注入 layout_engine（分页 + 页边界分隔线共用）。
        let (page_w, page_h, m_top, m_right, m_bottom, m_left) = extract_print_page_geometry(&stylesheets);
        self.layout_engine.set_print_page_height(page_h);
        self.layout_engine.set_print_page_width(page_w);
        self.layout_engine.set_print_page_margins(m_top, m_bottom);
        self.layout_engine.set_print_horizontal_margins(m_left, m_right);

        // 3. 计算样式
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let mut styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        // 3.5 把 ::before/::after 伪元素的 content 文本注入为合成文本子节点（doc 每帧
        // 重建，合成节点无累积、JS 不可见）。build_subtree 随后按普通文本子节点测量/绘制。
        inject_pseudo_text_nodes(&mut doc, &mut styles);

        // 3.6 R2439：`content:url()` 普通元素 element-becomes-replaced——元素盒自身按
        // image 固有尺寸 sizing（width/height Auto 时设为图尺寸），build_subtree 已抑制其
        // 子节点，paint_img_element 渲染图片。绕 R109 IFC（见 R2438 child-injection 证伪）。
        self.apply_content_url_replaced_sizing(&mut styles);

        // 4. 计算布局
        let layout_start = Instant::now();
        let (img_sizes, img_ratios, img_no_ratio) = self.build_img_intrinsic_all(&doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(&doc, &styles, img_sizes, img_ratios, img_no_ratio);
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        // 5. 生成绘制命令
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.set_canvas_registry(self.canvas_registry.clone());
        painter.register_counter_styles(&stylesheets);
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint(&layout_result.root, &styles, Some(&doc));
        let canvas_images = painter.canvas_images.clone();
        let mut primitives = painter.into_primitives();
        // 视口剔除 — 移除视口外的图元（高度取文档布局范围，供浏览器滚动消费）
        let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout_result.root);
        // S7b：cull_invisible 原位剔除（primitives 变量即结果，不再返回新对象）
        let stats = primitives.cull_invisible(viewport);
        let dirty_rects = vec![(0.0, 0.0, viewport.size.width, viewport.size.height)];
        // 对填充图元进行批处理优化（S7：draw_order 路径跳过——纯 clone no-op）
        let mut primitives = if primitives.draw_order.is_empty() {
            primitives.batch_fills()
        } else {
            primitives
        };
        // R2001 P1.5：Print 分页页边界分隔线（render-path 从 layout extent 重算）。
        inject_print_page_dividers(
            &mut primitives,
            self.style_system.media_type(),
            self.viewport_width,
            &layout_result.root,
            page_h,
        );
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;

        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.form_control_values.clear();
        self.form_control_compositions.clear();
        self.cached_doc = Some(Rc::new(RefCell::new(doc)));
        self.cached_styles = styles;
        // DOM 已替换：CSS 解析缓存失效（新文档的 <style>/meta 内容可能不同）。
        self.cached_css_text = None;

        // 缓存布局结果
        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
        };
        self.cached_layout = Some(layout_result);

        make_render_result(
            primitives,
            dirty_rects,
            layout,
            PipelineTimings {
                parse_ms,
                style_ms,
                layout_ms,
                paint_ms,
                total_ms,
                parse_count: 1,
                style_count: 1,
                layout_count: 1,
                paint_count: 1,
            },
            stats,
            canvas_images,
        )
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
        let (img_sizes, img_ratios, img_no_ratio) = self.build_img_intrinsic_all(doc);
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
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint(&layout_result.root, &styles, Some(doc));
        let primitives = painter.into_primitives();

        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
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
    ) -> Option<(RenderPrimitives, Vec<(u64, u32, u32, Vec<u8>)>)> {
        // 计算样式（全量——本路径无变更节点上下文；存 cached_styles 供 mutation
        // 增量路径作 base）
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let styles = self.style_system.compute_styles(doc, stylesheets);
        self.cached_styles = styles.clone();

        // 计算布局
        let (img_sizes, img_ratios, img_no_ratio) = self.build_img_intrinsic_all(doc);
        let layout_result =
            self.layout_engine
                .compute_with_img_intrinsic(doc, &styles, img_sizes, img_ratios, img_no_ratio);
        self.cached_layout = Some(LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
        });

        // 仅绘制脏区域内的节点
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint_in_rect(&layout_result.root, &styles, &dirty_rect, Some(doc));
        let canvas_images = painter.canvas_images.clone();
        Some((painter.into_primitives(), canvas_images))
    }

    /// 在已有 DOM 缓存上重绘整个视口（resize 等场景，走 `incremental_paint`）。
    pub fn repaint_cached_viewport(&mut self, css: &str) -> Option<RenderResult> {
        let total_start = Instant::now();
        let doc_rc = self.cached_doc.take()?;
        let dirty = zero_render_foundation::geometry::Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        // CSS 解析缓存：外部 css 文本相同（cached_doc 未变——render_html 族替换时已置
        // None 失效）直接复用解析结果，免每帧重新 tokenize+parse 全部 CSS。take 出 Rc 后放回
        //（incremental_paint 是 &mut self，无法与字段借用共存——take Rc 释放字段，borrow RefCell 不借字段）。
        let stylesheets = if self.cached_css_text.as_deref() == Some(css) {
            std::mem::take(&mut self.cached_stylesheets)
        } else {
            self.cached_css_text = Some(css.to_string());
            let doc = doc_rc.borrow();
            collect_stylesheets(&doc, css)
        };
        let (primitives, canvas_images) = {
            let doc = doc_rc.borrow();
            self.incremental_paint(&doc, &stylesheets, dirty)?
        };
        self.cached_stylesheets = stylesheets;
        self.cached_doc = Some(doc_rc);
        let layout_ref = self.cached_layout.as_ref()?;
        let layout = LayoutResult {
            root: layout_ref.root.clone(),
            viewport_width: layout_ref.viewport_width,
            viewport_height: layout_ref.viewport_height,
            paint_skip_node_ids: layout_ref.paint_skip_node_ids.clone(),
        };
        let dirty_rects = vec![(0.0, 0.0, self.viewport_width, self.viewport_height)];
        Some(make_render_result(
            primitives,
            dirty_rects,
            layout,
            PipelineTimings {
                total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
                style_count: 1,
                layout_count: 1,
                paint_count: 1,
                ..Default::default()
            },
            RenderStats::default(),
            canvas_images,
        ))
    }

    /// DOM 变更增量渲染（M3-S9 第一刀：消除 HTML 往返）。
    ///
    /// 把 JS 侧记录的 [`DomMutation`] 直接应用到缓存的活 DOM（`cached_doc`），
    /// 再走 `repaint_cached_viewport`（不重新 parse HTML——旧路径 `apply_mutations_to_html`
    /// 序列化回 HTML 后 `parse_html` 全量重建，大页面 parse 占整页渲染 ~30%）。
    ///
    /// # 返回
    ///
    /// `(RenderResult, 可选新 HTML 快照)`——纯表单当前值变更不修改内容属性，因而不生成
    /// 整页 HTML 快照；其余 DOM 变更返回快照供调用方同步 `cached_html`。
    pub fn render_with_dom_mutations(
        &mut self,
        mutations: &[crate::js_dom_bridge::DomMutation],
        css: &str,
    ) -> Result<(RenderResult, Option<String>, HashMap<String, String>), String> {
        let doc_rc = self.cached_doc.take().ok_or("no cached document")?;
        let all_form_value_only = !mutations.is_empty()
            && mutations
                .iter()
                .all(|mutation| Self::is_form_value_only_mutation(&doc_rc.borrow(), mutation));
        // 当前值先进入页面级 retained 状态；纯当前值编辑不改变 DOM 内容属性，也不序列化整页。
        {
            let doc = doc_rc.borrow();
            for mutation in mutations {
                match mutation {
                    crate::js_dom_bridge::DomMutation::SetFormValue { selector, value } => {
                        if let Some(node_id) = doc.query_selector(doc.root(), selector.trim()) {
                            self.form_control_values.insert(node_id, value.clone());
                            self.form_control_compositions.remove(&node_id);
                        } else {
                            // R3254-L12：selector 失配（页面 JS 重排 DOM 后）——输入无法落到
                            // 活 DOM，静默丢弃会表现为「打字无效果」。记 warn 而非静默。
                            tracing::warn!("SetFormValue: selector 失配，输入丢弃: {selector}");
                        }
                    }
                    crate::js_dom_bridge::DomMutation::SetFormComposition {
                        selector,
                        text,
                        selection_start,
                        selection_end,
                    } => {
                        if let Some(node_id) = doc.query_selector(doc.root(), selector.trim()) {
                            if text.is_empty() {
                                self.form_control_compositions.remove(&node_id);
                            } else {
                                self.form_control_compositions
                                    .insert(node_id, (text.clone(), *selection_start, *selection_end));
                            }
                        } else {
                            tracing::warn!("SetFormComposition: selector 失配: {selector}");
                        }
                    }
                    _ => {}
                }
            }
        }
        let (handle_selectors, html_snapshot) = {
            let mut doc = doc_rc.borrow_mut();
            let hs = crate::js_dom_bridge::apply_dom_mutations(&mut doc, mutations)?;
            let snapshot = (!all_form_value_only).then(|| doc.outer_html(doc.root()));
            (hs, snapshot)
        };
        if html_snapshot.is_some() {
            // DOM 已变（<style>/meta 内容可能变）：CSS 解析缓存失效。
            self.cached_css_text = None;
        }
        // 增量分层（mutation 全部同类时走轻量路径，否则全量兜底）：
        // 1. SetText-only → compute_incremental 增量布局（已验证与全量一致）
        // 2. SetStyle/RemoveStyle 布局无关属性（paint-only 白名单）→ 布局不变，
        //    复用 cached_layout 只重 style + paint（省 100% 布局）
        // 3. 其他（布局属性/结构变更）→ 全量布局（taffy style 单节点更新为后续专项）
        let all_text_only = mutations.iter().all(Self::is_text_only_mutation);
        let all_paint_only = !all_text_only && mutations.iter().all(Self::is_paint_only_mutation);
        // 增量分支：borrow RefCell（&mut self 方法与 Ref borrow 不冲突——后者借堆 RefCell 非字段），
        // 工作后 drop borrow 再把 doc_rc 放回；repaint 分支：先放回 doc_rc 再 repaint（它 take 自字段）。
        let result = if all_text_only {
            let r = {
                let doc = doc_rc.borrow();
                self.incremental_paint_after_text_mutations(&doc, mutations, css)
            };
            self.cached_doc = Some(doc_rc);
            r
        } else if all_form_value_only {
            let r = {
                let doc = doc_rc.borrow();
                self.paint_form_value_mutations(&doc, mutations)
            };
            self.cached_doc = Some(doc_rc);
            r
        } else if all_paint_only {
            let r = {
                let doc = doc_rc.borrow();
                self.paint_only_incremental(&doc, mutations, css)
            };
            self.cached_doc = Some(doc_rc);
            r
        } else {
            self.cached_doc = Some(doc_rc);
            self.repaint_cached_viewport(css)
        }
        .ok_or("repaint failed after mutations")?;
        Ok((result, html_snapshot, handle_selectors))
    }

    /// mutation 是否为纯文本变更（SetText 的 CSS-selector 变体——handle 变体无法
    /// 在 pipeline 侧定位节点，走全量）。
    fn is_text_only_mutation(m: &crate::js_dom_bridge::DomMutation) -> bool {
        matches!(m, crate::js_dom_bridge::DomMutation::SetText { .. })
    }

    /// 当前值不改变文本输入框的外部几何；没有依赖 `value` 的选择器时可只重绘。
    fn is_form_value_only_mutation(doc: &Document, mutation: &crate::js_dom_bridge::DomMutation) -> bool {
        let selector = match mutation {
            crate::js_dom_bridge::DomMutation::SetFormValue { selector, .. }
            | crate::js_dom_bridge::DomMutation::SetFormComposition { selector, .. } => selector,
            _ => return false,
        };
        let Some(node_id) = doc.query_selector(doc.root(), selector.trim()) else {
            return false;
        };
        let Some(node) = doc.get(node_id) else {
            return false;
        };
        let NodeKind::Element(element) = &node.kind else {
            return false;
        };
        if element.local_name().eq_ignore_ascii_case("textarea") {
            return true;
        }
        if !element.local_name().eq_ignore_ascii_case("input") {
            return false;
        }
        matches!(
            element
                .get_attribute("type")
                .as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str(),
            "" | "text" | "search" | "url" | "tel" | "email" | "password"
        )
    }

    fn paint_form_value_mutations(
        &mut self,
        doc: &Document,
        mutations: &[crate::js_dom_bridge::DomMutation],
    ) -> Option<RenderResult> {
        let total_start = Instant::now();
        if self.cached_styles.is_empty() {
            return None;
        }
        let changed: Vec<NodeId> = mutations
            .iter()
            .filter_map(|mutation| match mutation {
                crate::js_dom_bridge::DomMutation::SetFormValue { selector, .. }
                | crate::js_dom_bridge::DomMutation::SetFormComposition { selector, .. } => {
                    doc.query_selector(doc.root(), selector.trim())
                }
                _ => None,
            })
            .collect();
        if changed.is_empty() {
            return None;
        }
        let layout = self.cached_layout.as_ref()?;
        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout.paint_skip_node_ids.clone();
        painter.paint(&layout.root, &self.cached_styles, Some(doc));
        let canvas_images = painter.canvas_images.clone();
        let primitives = painter.into_primitives();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;
        let dirty_rects =
            layout_dirty_rects_for_nodes(&layout.root, &changed, self.viewport_width, self.viewport_height);
        Some(make_render_result(
            primitives,
            dirty_rects,
            LayoutResult {
                root: layout.root.clone(),
                viewport_width: layout.viewport_width,
                viewport_height: layout.viewport_height,
                paint_skip_node_ids: layout.paint_skip_node_ids.clone(),
            },
            PipelineTimings {
                paint_ms,
                total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
                paint_count: 1,
                ..Default::default()
            },
            RenderStats::default(),
            canvas_images,
        ))
    }

    /// mutation 是否只改布局无关（paint-only）样式——布局不变，可复用 cached_layout。
    fn is_paint_only_mutation(m: &crate::js_dom_bridge::DomMutation) -> bool {
        match m {
            crate::js_dom_bridge::DomMutation::SetStyle { property, .. }
            | crate::js_dom_bridge::DomMutation::RemoveStyle { property, .. } => Self::is_paint_only_property(property),
            _ => false,
        }
    }

    /// 布局无关（paint-only）属性白名单：这些属性只影响绘制，不影响 taffy 布局
    ///（尺寸/位置/流），故样式变更后可复用 cached_layout（省 100% 布局重算）。
    /// 白名单保守——未列属性（含未知）一律按布局相关走全量。
    fn is_paint_only_property(property: &str) -> bool {
        let p = property.trim();
        p == "color"
            || p == "opacity"
            || p == "visibility"
            || p == "z-index"
            || p == "box-shadow"
            || p == "text-shadow"
            || p == "filter"
            || p == "backdrop-filter"
            || p == "cursor"
            || p == "user-select"
            || p == "pointer-events"
            || p == "mix-blend-mode"
            || p == "isolation"
            || p == "clip-path"
            || p == "will-change"
            || p == "scroll-behavior"
            || p == "text-overflow"
            || p == "transform"
            || p == "transform-origin"
            || p.starts_with("background") // background / background-*
            || p.starts_with("border-radius") // border-radius / border-*-radius
            || p.starts_with("border-color") // border-color / border-*-color（仅颜色，无布局影响）
            || p.starts_with("text-decoration") // 绘制下划线等，无布局影响
            || p.starts_with("outline") // outline 不占布局
    }

    /// 布局无关样式变更增量渲染：全量 style（paint 消费新样式）+ 复用 cached_layout
    ///（布局不变——paint-only 白名单保证）+ 全量 paint。
    fn paint_only_incremental(
        &mut self,
        doc: &Document,
        mutations: &[crate::js_dom_bridge::DomMutation],
        css: &str,
    ) -> Option<RenderResult> {
        let stylesheets = collect_stylesheets(doc, css);
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        // 增量样式：只重算变更节点子树（base = cached_styles，全量路径已存）。
        // 变更节点 = SetStyle/RemoveStyle 的 selector 目标；selector 未定位到 → 无
        // 变更可重算，返回 None（调用方全量兜底）。
        let changed: Vec<NodeId> = mutations
            .iter()
            .filter_map(|m| match m {
                crate::js_dom_bridge::DomMutation::SetStyle { selector, .. }
                | crate::js_dom_bridge::DomMutation::RemoveStyle { selector, .. } => {
                    doc.query_selector(doc.root(), selector.trim())
                }
                _ => None,
            })
            .collect();
        if changed.is_empty() {
            return None;
        }
        if self.cached_styles.is_empty() {
            // 首帧（render_html 路径不维护 cached_styles）：全量计算并作为 base。
            let s = self.style_system.compute_styles(doc, &stylesheets);
            self.cached_styles = s;
        } else {
            self.style_system
                .compute_styles_incremental(doc, &stylesheets, &changed, &mut self.cached_styles);
        }
        let layout = self.cached_layout.as_ref()?;
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint(&layout.root, &self.cached_styles, Some(doc));
        let canvas_images = painter.canvas_images.clone();
        let primitives = painter.into_primitives();
        let dirty_rects =
            layout_dirty_rects_for_nodes(&layout.root, &changed, self.viewport_width, self.viewport_height);
        Some(make_render_result(
            primitives,
            dirty_rects,
            LayoutResult {
                root: layout.root.clone(),
                viewport_width: layout.viewport_width,
                viewport_height: layout.viewport_height,
                paint_skip_node_ids: layout.paint_skip_node_ids.clone(),
            },
            PipelineTimings {
                style_count: 1,
                paint_count: 1,
                ..Default::default()
            },
            RenderStats::default(),
            canvas_images,
        ))
    }

    /// 文本变更增量渲染：全量 style + 脏标记文本节点 → `compute_incremental` +
    /// 全量 paint（仅布局增量——样式/绘制增量是 M3-S9 后续切片）。
    fn incremental_paint_after_text_mutations(
        &mut self,
        doc: &Document,
        mutations: &[crate::js_dom_bridge::DomMutation],
        css: &str,
    ) -> Option<RenderResult> {
        let stylesheets = collect_stylesheets(doc, css);
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        // 增量样式：只重算 SetText 目标节点子树（base = cached_styles）。
        let changed: Vec<NodeId> = mutations
            .iter()
            .filter_map(|m| match m {
                crate::js_dom_bridge::DomMutation::SetText { selector, .. } => {
                    doc.query_selector(doc.root(), selector.trim())
                }
                _ => None,
            })
            .collect();
        if self.cached_styles.is_empty() {
            let s = self.style_system.compute_styles(doc, &stylesheets);
            self.cached_styles = s;
        } else {
            self.style_system
                .compute_styles_incremental(doc, &stylesheets, &changed, &mut self.cached_styles);
        }
        let (img_sizes, _img_ratios, _img_no_ratio) = self.build_img_intrinsic_all(doc);
        let mut tracker = zero_layout_engine::LayoutDirtyTracker::new();
        for m in mutations {
            if let crate::js_dom_bridge::DomMutation::SetText { selector, .. } = m
                && let Some(id) = doc.query_selector(doc.root(), selector.trim())
            {
                tracker.mark_dirty(id);
            }
        }
        let (layout, _stats) =
            self.layout_engine
                .compute_incremental(doc, &self.cached_styles, &mut tracker, &img_sizes);
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.set_form_control_values(self.form_control_values.clone());
        painter.set_form_control_compositions(self.form_control_compositions.clone());
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint(&layout.root, &self.cached_styles, Some(doc));
        let canvas_images = painter.canvas_images.clone();
        let primitives = painter.into_primitives();
        self.cached_layout = Some(LayoutResult {
            root: layout.root.clone(),
            viewport_width: layout.viewport_width,
            viewport_height: layout.viewport_height,
            paint_skip_node_ids: layout.paint_skip_node_ids.clone(),
        });
        let dirty_nodes: Vec<NodeId> = mutations
            .iter()
            .filter_map(|m| match m {
                crate::js_dom_bridge::DomMutation::SetText { selector, .. } => {
                    doc.query_selector(doc.root(), selector.trim())
                }
                _ => None,
            })
            .collect();
        let dirty_rects =
            layout_dirty_rects_for_nodes(&layout.root, &dirty_nodes, self.viewport_width, self.viewport_height);
        Some(make_render_result(
            primitives,
            dirty_rects,
            layout,
            PipelineTimings {
                style_count: 1,
                layout_count: 1,
                paint_count: 1,
                ..Default::default()
            },
            RenderStats::default(),
            canvas_images,
        ))
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

/// R2001 P1.5：Print 分页页边界分隔线——在每页边界（k × page_height）处绘制浅灰细线，
/// 使 Ctrl+P 打印预览的分页可视化（满页内容相邻处无 gap，分隔线标记边界）。
///
/// 仅 `media_type==Print` + 分页启用时绘制；Screen 零影响。页边界从 layout extent 重算
/// （`ceil(extent/page_height)`，与 `paginate_for_print` 的 push 目标 k×page_height 一致），
/// 避免 LayoutResult 新字段（types/tests 多处构造点 churn）。
fn inject_print_page_dividers(
    primitives: &mut RenderPrimitives,
    media_type: zero_css_parser::media_query::MediaType,
    viewport_w: f32,
    layout_root: &zero_layout_engine::LayoutBox,
    page_h: f32,
) {
    use zero_layout_engine::print_pagination;
    if !matches!(media_type, zero_css_parser::media_query::MediaType::Print)
        || !print_pagination::print_paginate_enabled()
    {
        return;
    }
    // R2010 P4：页高由 `@page { size }` 解析传入（default A4，与 paginate_for_print 同源）。
    // R2018 P5a：页边界经 `compute_print_page_sequence` 算（与 paginate/divider 单一真相）。
    // 分隔线用物理页边界（physical_top），边距传 0（分隔线标记物理页 break，与 margin 无关）。
    let extent = layout_extent_y(layout_root, 0.0);
    let pages = print_pagination::compute_print_page_sequence(extent, page_h, 0.0, 0.0);
    if pages.len() < 2 {
        return; // 单页：无内部页边界。
    }
    let color = Color::rgb(170, 170, 170);
    // 页 1..N-1 的物理顶 = 页间分隔线位置（页 0 顶 = 文档顶，不画）。
    for page in pages.iter().skip(1) {
        primitives.add_fill(Rect::new(0.0, page.physical_top, viewport_w, 2.0), color);
    }
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
    // HTML presentational hint（HTML §4.2.5）：首个（树序）`<meta name="color-scheme"
    // content="...">` 等价于在根元素声明 color-scheme。注入为最低优先级 stylesheet
    //（vector 首位），author CSS（`css` 参数 + `<style>`）可覆盖。content 原样作
    // color-scheme 值（"dark"/"light dark" 等，由 R2285/R2286 属性解析 + used-scheme 合成消费）。
    // meta/style 合并为单次 DOM 遍历（旧实现 2 次全树遍历）。
    let (meta_ids, style_ids): (Vec<_>, Vec<_>) = doc
        .get_elements_by_tag_names(&["meta", "style"])
        .into_iter()
        .partition(|id| {
            doc.get_attribute(*id, "name")
                .is_some_and(|n| n.eq_ignore_ascii_case("color-scheme"))
        });
    // partition 保持树序，首个即树序第一个 color-scheme meta（HTML spec 仅首个生效）
    if let Some(meta_id) = meta_ids.first().copied()
        && let Some(content) = doc.get_attribute(meta_id, "content")
    {
        let content = content.trim();
        if !content.is_empty() {
            let synthetic = format!("html {{ color-scheme: {content}; }}");
            stylesheets.push(zero_css_parser::Parser::parse_stylesheet(&synthetic));
        }
    }
    if !css.is_empty() {
        stylesheets.push(zero_css_parser::Parser::parse_stylesheet(css));
    }
    for style_id in style_ids {
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

/// 从样式表中提取 Print 分页页高（R2010 P4：`@page { size }` 解析）。
/// 从样式表中提取 Print 分页页几何（R2010 P4 `@page { size }` + R2011 `@page { margin }`）。
///
/// 返回 `(page_height, margin_top, margin_bottom)`：扫描首个有效 `@page` 的 `size`/`margin`
/// 描述符；无 `@page` 或描述符无效时回退默认（A4 高 + 0 边距）。仅 `media_type==Print` 时
/// `paginate_for_print` / `inject_print_page_dividers` 消费；Screen 不调用。
pub(crate) fn extract_print_page_geometry(stylesheets: &[Stylesheet]) -> (f32, f32, f32, f32, f32, f32) {
    use zero_css_parser::ast::Rule;
    let mut width = zero_layout_engine::print_pagination::PRINT_PAGE_WIDTH_A4;
    let mut height = zero_layout_engine::print_pagination::PRINT_PAGE_HEIGHT_A4;
    let mut margin_top = 0.0;
    let mut margin_right = 0.0;
    let mut margin_bottom = 0.0;
    let mut margin_left = 0.0;
    for ss in stylesheets {
        for rule in &ss.rules {
            if let Rule::Page(page) = rule {
                if let Some((w, h)) = page.size {
                    if w > 0.0 {
                        width = w;
                    }
                    if h > 0.0 {
                        height = h;
                    }
                }
                if let Some((mt, mr, mb, ml)) = page.margin {
                    if mt >= 0.0 {
                        margin_top = mt;
                    }
                    if mr >= 0.0 {
                        margin_right = mr;
                    }
                    if mb >= 0.0 {
                        margin_bottom = mb;
                    }
                    if ml >= 0.0 {
                        margin_left = ml;
                    }
                }
            }
        }
    }
    (width, height, margin_top, margin_right, margin_bottom, margin_left)
}

/// 为有 animation-name 的元素启动动画并将插值属性叠加到 ComputedStyle。
///
/// 遍历所有元素的样式，检查 animation-name 列表，
/// 通过 AnimationClock 启动/推进动画，然后应用插值结果。
///
/// 返回「本轮新产生」的动画事件（R3249 animationend + R3250 animationiteration + R3251 animationstart）——
/// `(NodeId, kind, animationName, elapsedTime)` 列表（animated_ids 持 elem_key↔NodeId 双键，故直接返 NodeId，
/// 调用方免往返）。调用方据此映射 selector 后存 pending 待宿主派发。
fn apply_animation_overrides(
    clock: &mut AnimationClock,
    styles: &mut HashMap<NodeId, ComputedStyle>,
    current_time: f64,
) -> Vec<(NodeId, AnimationEventKind, String, f64)> {
    // 收集有动画名称的元素 ID
    let animated_ids: Vec<(u64, NodeId)> = styles
        .iter()
        .filter(|(_, s)| !s.animation_name.is_empty() && s.animation_name.iter().any(|n| !n.is_empty() && n != "none"))
        .map(|(id, _)| {
            // 将 slotmap NodeId 转为 u64
            (id.data().as_ffi(), *id)
        })
        .collect();

    // elem_key(u64) → NodeId 索引（drain_just_finished 返 element_key:u64，须映射回 NodeId）。
    let key_to_nid: HashMap<u64, NodeId> = animated_ids.iter().cloned().collect();

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

    // R3251（CSS Animations §animationstart）：drain 本轮新启动动画 → 映射 element_key→NodeId，返
    // (NodeId, Start, name, elapsedTime=0)。置首（spec：animationstart 先于 animationend/animationiteration
    // 派发，即使同帧——瞬时动画先 start 再 end）。
    let mut events: Vec<(NodeId, AnimationEventKind, String, f64)> = clock
        .drain_just_started()
        .into_iter()
        .filter_map(|s| {
            key_to_nid
                .get(&s.element_key)
                .map(|nid| (*nid, AnimationEventKind::Start, s.name, 0.0))
        })
        .collect();
    // R3249：drain 本轮新完成动画 → 映射 element_key→NodeId，返 (NodeId, End, name, elapsedTime) 供调用方派发。
    events.extend(clock.drain_just_finished().into_iter().filter_map(|fin| {
        key_to_nid
            .get(&fin.element_key)
            .map(|nid| (*nid, AnimationEventKind::End, fin.name, fin.duration))
    }));
    // R3250（CSS Animations §animationiteration）：drain 本轮跨越的迭代边界 → 映射 element_key→NodeId，
    // 返 (NodeId, Iteration, name, elapsedTime)。infinite 动画永不 finish（无 animationend），靠此驱动循环回调。
    events.extend(clock.drain_just_iterated().into_iter().filter_map(|it| {
        key_to_nid
            .get(&it.element_key)
            .map(|nid| (*nid, AnimationEventKind::Iteration, it.name, it.elapsed))
    }));
    events
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
mod dirty_region_tests {
    use super::*;

    #[test]
    fn render_outputs_full_viewport_dirty_rect() {
        // S3 dirty region 契约：全量渲染 → 脏区域覆盖整个视口
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html("<html><body><p>hi</p></body></html>", "");
        assert!(!result.stats.dirty_rects.is_empty(), "应有脏区域输出");
        let (x, y, w, h) = result.stats.dirty_rects[0];
        assert_eq!((x, y), (0.0, 0.0));
        assert_eq!(w, 800.0);
        assert_eq!(h, 600.0);
    }
}

#[cfg(test)]
mod canvas_display_tests {
    use super::*;

    /// R3268 canvas 显示链路：canvas 元素（带 data-zw-canvas-ctx 属性）+ registry
    /// 预填 ctx → painter 产出 ImagePrimitive + canvas_images（像素快照）。
    #[test]
    fn canvas_element_bridges_to_image_primitive() {
        let registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
            std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        // 预填 ctx（模拟 JS getContext 后的状态）：2×2 画布，红像素
        {
            let mut reg = registry.lock().unwrap();
            reg.contexts.insert(1, zero_canvas::CanvasContext::new(2, 2));
            let ctx = reg.contexts.get_mut(&1).unwrap();
            // fillRect 便捷法不写 pixel_buffer——按 shim 路径：beginPath+rect+fill
            ctx.set_fill_color(zero_render_foundation::color::Color::rgba(255, 0, 0, 255));
            ctx.begin_path();
            ctx.move_to(0.0, 0.0);
            ctx.line_to(2.0, 0.0);
            ctx.line_to(2.0, 2.0);
            ctx.line_to(0.0, 2.0);
            ctx.close_path();
            ctx.fill(); // path-based 写 pixel_buffer
        }
        let mut pipeline = RenderPipeline::new(100.0, 100.0);
        pipeline.set_canvas_registry(Some(registry));
        let html = r#"<html><body><canvas data-zw-canvas-ctx="1" width="2" height="2"></canvas></body></html>"#;
        let result = pipeline.render_html(html, "");
        // ImagePrimitive：image_key = ctx_id = 1
        let has_canvas_image = result
            .display_list
            .primitives
            .images
            .iter()
            .any(|img| img.image_key.0 == 1);
        assert!(has_canvas_image, "应产出 canvas ImagePrimitive（key=1）");
        // canvas_images：像素快照
        assert_eq!(result.canvas_images.len(), 1, "应产出 canvas 像素快照");
        let (ctx_id, w, h, rgba) = &result.canvas_images[0];
        assert_eq!((*ctx_id, *w, *h), (1, 2, 2));
        assert_eq!(&rgba[..4], &[255, 0, 0, 255], "快照应含红像素");
    }

    /// 无内容画布（全透明）→ 不产出图元（snapshot_rgba None）。
    #[test]
    fn empty_canvas_produces_no_primitive() {
        let registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
            std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        registry
            .lock()
            .unwrap()
            .contexts
            .insert(1, zero_canvas::CanvasContext::new(2, 2));
        let mut pipeline = RenderPipeline::new(100.0, 100.0);
        pipeline.set_canvas_registry(Some(registry));
        let html = r#"<html><body><canvas data-zw-canvas-ctx="1" width="2" height="2"></canvas></body></html>"#;
        let result = pipeline.render_html(html, "");
        assert!(result.canvas_images.is_empty(), "空白画布不应产出图元（无内容快照）");
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

    /// R2411：@import 的 url() 是样式表引用，不当图片抓取（否则被当 background 重复 fetch+解码失败）。
    #[test]
    fn extract_css_image_urls_skips_import() {
        let css = r#"
            @import url(theme.css);
            @import "reset.css";
            .a { background-image: url(bg.png); }
        "#;
        let urls = extract_css_image_urls(css);
        // @import url() 与 bare string 均排除（bare string 非 url() token 本就不被收集），
        // 仅保留真实图片 bg.png。
        assert_eq!(urls, vec!["bg.png"]);
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

    /// R2133：转义函数名 `url()`（`U\r\4c (`）与大小写变体经 tokenizer 识别——原 raw
    /// `find("url(")` 漏此形式。driving：uri-015 `background: red U\r\4c ("...")`。
    /// 内容转义（`support/\'green\ block.png`）经 tokenizer 解码，与 painter key 对齐。
    #[test]
    fn extract_css_image_urls_handles_escaped_function_name() {
        let css = r#"
            .a { background: red U\r\4c ("support/swatch-green.png"); }
            .b { background: URL(img.png); }
            .c { background: url(support/\'green\ block.png); }
        "#;
        let urls = extract_css_image_urls(css);
        assert!(
            urls.iter().any(|u| u == "support/swatch-green.png"),
            "escaped function name U\\r\\4c ( should be detected: {urls:?}"
        );
        assert!(urls.iter().any(|u| u == "img.png"), "URL( case: {urls:?}");
        // 内容转义经解码 → "support/'green block.png"（与 painter 一致）。
        assert!(
            urls.iter().any(|u| u == "support/'green block.png"),
            "escaped content decoded: {urls:?}"
        );
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

    /// R2419：srcset 首 URL 提取（逗号分隔候选，每候选首 token = URL）。
    #[test]
    fn srcset_first_url_parses_first_candidate() {
        assert_eq!(srcset_first_url("a.jpg 1x, b.jpg 2x").as_deref(), Some("a.jpg"));
        assert_eq!(srcset_first_url("a.jpg").as_deref(), Some("a.jpg"));
        assert_eq!(srcset_first_url("  c.png 480w, d.png 800w  ").as_deref(), Some("c.png"));
        assert!(srcset_first_url("").is_none());
        assert!(srcset_first_url("  ").is_none());
    }

    /// R2419：srcset-only `<img>`（无 src）经 extract_img_resources 回退到 srcset 首 URL。
    #[test]
    fn extract_img_resources_falls_back_to_srcset() {
        let html = r#"<html><body>
            <img srcset="hi.jpg 1x, hi2.jpg 2x">
            <img src="real.jpg">
            <img srcset="narrow.jpg 480w, wide.jpg 800w" loading="lazy">
            </body></html>"#;
        let imgs = extract_img_resources(html);
        assert_eq!(imgs.len(), 3, "{imgs:?}");
        assert_eq!(imgs[0].src, "hi.jpg", "srcset-only 用首 URL");
        assert!(!imgs[0].lazy, "无 loading=lazy");
        assert_eq!(imgs[1].src, "real.jpg", "src 优先于 srcset");
        assert_eq!(imgs[2].src, "narrow.jpg", "srcset-only 用首 URL + lazy 保留");
        assert!(imgs[2].lazy, "loading=lazy 保留");
    }
}

#[cfg(test)]
mod font_face_extract_tests {
    use super::*;

    /// FR-001：多 @font-face 提取 (family, sources)，family 去引号、sources 按序、format() 忽略。
    #[test]
    fn extract_font_faces_family_and_sources() {
        let css = r#"
            @font-face {
                font-family: "JetBrains Mono";
                src: url(jb.woff2) format("woff2"), url(jb.ttf);
            }
            @font-face { font-family: 'Title'; src: url(t.woff); }
            p { color: red; }
        "#;
        let faces = extract_font_faces(css);
        assert_eq!(
            faces,
            vec![
                (
                    "JetBrains Mono".to_string(),
                    vec!["jb.woff2".to_string(), "jb.ttf".to_string()],
                    None,
                    false,
                    None,
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
                (
                    "Title".to_string(),
                    vec!["t.woff".to_string()],
                    None,
                    false,
                    None,
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
            ],
            "family dequoted; sources ordered; format() ignored; non-font-face rules skipped"
        );
    }

    /// extract_font_faces 返回 weight、style 与 stretch 描述符。
    #[test]
    fn extract_font_faces_returns_weight_and_style() {
        let css = r#"
            @font-face { font-family: "Bold"; src: url(b.woff); font-weight: bold; }
            @font-face { font-family: "Reg"; src: url(r.woff); font-stretch: condensed; }
            @font-face { font-family: "Italic"; src: url(i.woff); font-style: italic; }
            @font-face { font-family: "Oblique"; src: url(o.woff); font-style: oblique; }
        "#;
        let faces = extract_font_faces(css);
        assert_eq!(
            faces,
            vec![
                (
                    "Bold".to_string(),
                    vec!["b.woff".to_string()],
                    Some(700),
                    false,
                    None,
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
                (
                    "Reg".to_string(),
                    vec!["r.woff".to_string()],
                    None,
                    false,
                    Some(75.0),
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
                (
                    "Italic".to_string(),
                    vec!["i.woff".to_string()],
                    None,
                    true,
                    None,
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
                (
                    "Oblique".to_string(),
                    vec!["o.woff".to_string()],
                    None,
                    true,
                    None,
                    zero_css_parser::values::FontFeatureSettingsValue::Normal,
                ),
            ]
        );
    }

    /// FR-001：无 @font-face 返回空 Vec（解析失败/无规则同样为空）。
    #[test]
    fn extract_font_faces_empty() {
        assert!(extract_font_faces("p { color: red; }").is_empty());
        assert!(extract_font_faces("").is_empty());
    }

    /// FR-001：纯投影——extract_font_faces 输出与直接解析 FontFaceRule 一致（不额外过滤）。
    /// data:/local() 的过滤由抓取层负责，本函数只透传 css-parser 结果。
    #[test]
    fn extract_font_faces_matches_direct_parse() {
        use zero_css_parser::ast::Rule as CssRule;
        use zero_css_parser::values::types::FontStyleValue;
        let css = r#"@font-face { font-family: X; src: url(a.woff) format("woff"), url(b.ttf); }"#;
        let direct = zero_css_parser::Parser::parse_stylesheet(css)
            .rules
            .iter()
            .filter_map(|r| match r {
                CssRule::FontFace(ff) => {
                    let is_italic = matches!(
                        ff.style,
                        Some(FontStyleValue::Italic) | Some(FontStyleValue::Oblique(_))
                    );
                    Some((
                        ff.family.clone(),
                        ff.sources.clone(),
                        ff.weight,
                        is_italic,
                        ff.stretch,
                        ff.feature_settings.clone(),
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(extract_font_faces(css), direct);
    }

    /// @import URL 提取（`url()` 与 bare string 两种形式，按序，媒体查询忽略）。
    #[test]
    fn extract_import_urls_collects_urls() {
        let css = r#"
            @import url(theme.css);
            @import "reset.css" screen, print;
            p { color: red; }
        "#;
        assert_eq!(
            extract_import_urls(css),
            vec!["theme.css".to_string(), "reset.css".to_string()],
            "url() 与 bare string 均提取，媒体查询忽略"
        );
    }

    #[test]
    fn extract_import_urls_empty_when_none() {
        assert!(extract_import_urls("p { color: red; }").is_empty());
        assert!(extract_import_urls("").is_empty());
    }
}

#[cfg(test)]
mod print_page_divider_tests {
    use super::*;
    use zero_css_parser::media_query::MediaType;
    use zero_layout_engine::LayoutBox;

    /// 构造 layout 根，高度 = `height`（其余 default）。
    fn root_with_height(height: f32) -> LayoutBox {
        LayoutBox {
            height,
            ..Default::default()
        }
    }

    /// R2001 P1.5：Print 模式 + 分页启用 → extent 跨多页时每页边界加 1 条分隔线 fill。
    /// extent=2500px，A4 页高 1122.5 → ceil(2500/1122.5)=3 页 → 边界 1122.5 + 2245.0 = 2 条。
    #[test]
    fn r2001_inject_print_page_dividers_one_line_per_boundary() {
        let mut primitives = RenderPrimitives::new();
        let root = root_with_height(2500.0);
        // Print + 分页启用（env default-on；显式设 1 避并行噪声）。
        unsafe {
            std::env::set_var("ZW_PRINT_PAGINATE", "1");
        }
        inject_print_page_dividers(&mut primitives, MediaType::Print, 800.0, &root, 1122.5);
        unsafe {
            std::env::remove_var("ZW_PRINT_PAGINATE");
        }
        assert_eq!(primitives.fills.len(), 2, "3 页 → 2 条内部页边界分隔线");
        // 第 1 条分隔线 y ≈ A4 页高 1122.5，full-width 800，厚 2。
        let first = &primitives.fills[0];
        assert!(
            (first.rect.origin.y - 1122.5).abs() < 0.1,
            "首条 y≈1122.5, got {}",
            first.rect.origin.y
        );
        assert!((first.rect.size.width - 800.0).abs() < 0.1, "full-width 800");
        assert!((first.rect.size.height - 2.0).abs() < 0.1, "厚度 2px");
    }

    /// R2001 P1.5：Screen 模式 → 零分隔线（零回归核心不变量）。
    #[test]
    fn r2001_inject_print_page_dividers_screen_zero_dividers() {
        let mut primitives = RenderPrimitives::new();
        let root = root_with_height(2500.0);
        inject_print_page_dividers(&mut primitives, MediaType::Screen, 800.0, &root, 1122.5);
        assert_eq!(primitives.fills.len(), 0, "Screen 模式不得加分隔线");
    }

    /// R2001 P1.5：单页（extent ≤ 页高）→ 无内部页边界 → 零分隔线。
    #[test]
    fn r2001_inject_print_page_dividers_single_page_none() {
        let mut primitives = RenderPrimitives::new();
        let root = root_with_height(800.0); // < 1122.5 → 单页
        unsafe {
            std::env::set_var("ZW_PRINT_PAGINATE", "1");
        }
        inject_print_page_dividers(&mut primitives, MediaType::Print, 800.0, &root, 1122.5);
        unsafe {
            std::env::remove_var("ZW_PRINT_PAGINATE");
        }
        assert_eq!(primitives.fills.len(), 0, "单页无内部页边界");
    }
}

#[cfg(test)]
mod print_page_size_tests {
    use super::*;
    use zero_css_parser::Parser;

    /// R2010 P4：`@page { size }` 解析后的页高经 `extract_print_page_geometry` 提取，
    /// 覆盖默认 A4。letter (11in) = 1056px。
    #[test]
    fn r2010_extract_print_page_height_letter_overrides_a4_default() {
        let ws = vec![Parser::parse_stylesheet("@page { size: letter; }")];
        let (_w, h, _mt, _mr, _mb, _ml) = extract_print_page_geometry(&ws);
        assert!((h - 1056.0).abs() < 0.1, "letter height 1056, got {h}");
    }

    /// 无 @page 或 size 无效 → 回退默认 A4 高（PRINT_PAGE_HEIGHT_A4 = 1122.5）。
    #[test]
    fn r2010_extract_print_page_height_defaults_to_a4() {
        let ws = vec![Parser::parse_stylesheet("@page { margin: 2cm; } div { color: red; }")];
        let (_w, h, _mt, _mr, _mb, _ml) = extract_print_page_geometry(&ws);
        assert!((h - 1122.5).abs() < 0.1, "no size → A4 default 1122.5, got {h}");
        let empty: Vec<zero_css_parser::Stylesheet> = vec![];
        let (_w0, h0, _, _, _, _) = extract_print_page_geometry(&empty);
        assert!((h0 - 1122.5).abs() < 0.1, "empty → A4 default, got {h0}");
    }

    /// R2011：`@page { margin }` 解析为垂直边距注入分页内容区（top/bottom）。1 值简写 = 四边同。
    #[test]
    fn r2011_extract_print_page_margin_vertical() {
        let ws = vec![Parser::parse_stylesheet("@page { size: A4; margin: 2cm; }")];
        let (_w, h, mt, _mr, mb, _ml) = extract_print_page_geometry(&ws);
        assert!((h - 1122.5).abs() < 1.0, "A4 height");
        let two_cm = 2.0 * 96.0 / 2.54;
        assert!((mt - two_cm).abs() < 0.1, "margin-top 2cm, got {mt}");
        assert!((mb - two_cm).abs() < 0.1, "margin-bottom 2cm, got {mb}");
    }

    /// R2011：2 值 margin 简写 `(top bottom, right left)` → top/bottom = 第 1 值。
    #[test]
    fn r2011_extract_print_page_margin_two_value() {
        let ws = vec![Parser::parse_stylesheet("@page { margin: 100px 50px; }")];
        let (_w, _h, mt, _mr, mb, _ml) = extract_print_page_geometry(&ws);
        assert!((mt - 100.0).abs() < 0.1, "margin-top = 100px (1st value), got {mt}");
        assert!((mb - 100.0).abs() < 0.1, "margin-bottom = 100px (1st value), got {mb}");
    }

    /// R2011：无 margin → 边距默认 0（旧行为，零行为变更）。
    #[test]
    fn r2011_extract_print_page_margin_defaults_zero() {
        let ws = vec![Parser::parse_stylesheet("@page { size: A4; }")];
        let (_w, _h, mt, _mr, mb, _ml) = extract_print_page_geometry(&ws);
        assert_eq!((mt, mb), (0.0, 0.0), "no margin → (0, 0)");
    }

    /// R2013 layout-width-for-print：`@page { size }` 解析页宽注入。A4 默认宽 ≈793.7；
    /// `size: letter` 宽 = 8.5in × 96 = 816px。
    #[test]
    fn r2013_extract_print_page_width_letter() {
        let ws = vec![Parser::parse_stylesheet("@page { size: letter; }")];
        let (w, _h, _mt, _mr, _mb, _ml) = extract_print_page_geometry(&ws);
        assert!((w - 816.0).abs() < 0.1, "letter width 816, got {w}");
    }

    /// R2013：无 size → 页宽回退默认 A4（≈793.7）。A4 = 210mm @96dpi。
    #[test]
    fn r2013_extract_print_page_width_defaults_to_a4() {
        let a4_w = 210.0 / 25.4 * 96.0;
        let ws = vec![Parser::parse_stylesheet("@page { margin: 1in; }")];
        let (w, _h, _mt, _mr, _mb, _ml) = extract_print_page_geometry(&ws);
        assert!((w - a4_w).abs() < 0.1, "no size → A4 default width {a4_w}, got {w}");
    }

    /// R2013：`@page { margin }` 水平边距（left/right）解析注入。3 值简写 = top, right/left, bottom。
    #[test]
    fn r2013_extract_print_page_margin_horizontal() {
        let ws = vec![Parser::parse_stylesheet("@page { size: A4; margin: 1in 2in 3in; }")];
        let (_w, _h, mt, mr, mb, ml) = extract_print_page_geometry(&ws);
        let inch = 96.0;
        assert!((mt - 1.0 * inch).abs() < 0.1, "margin-top 1in");
        assert!((mr - 2.0 * inch).abs() < 0.1, "margin-right 2in, got {mr}");
        assert!((mb - 3.0 * inch).abs() < 0.1, "margin-bottom 3in");
        assert!(
            (ml - 2.0 * inch).abs() < 0.1,
            "margin-left 2in (3-value = r/l), got {ml}"
        );
    }

    /// R2013：4 值 margin 简写 `(top right bottom left)` → 完整四边。
    #[test]
    fn r2013_extract_print_page_margin_four_value() {
        let ws = vec![Parser::parse_stylesheet("@page { margin: 10px 20px 30px 40px; }")];
        let (_w, _h, mt, mr, mb, ml) = extract_print_page_geometry(&ws);
        assert_eq!((mt, mr, mb, ml), (10.0, 20.0, 30.0, 40.0), "4-value margin TRBL");
    }

    /// R2010 P4：inject_print_page_dividers 接受自定义页高——letter=1056，extent=2500
    /// → ceil(2500/1056)=3 页 → 边界 1056 + 2112 = 2 条（非 A4 的 1122.5 + 2245）。
    #[test]
    fn r2010_dividers_use_custom_page_height() {
        use zero_css_parser::media_query::MediaType;
        use zero_layout_engine::LayoutBox;
        let mut primitives = RenderPrimitives::new();
        let root = LayoutBox {
            height: 2500.0,
            ..Default::default()
        };
        unsafe {
            std::env::set_var("ZW_PRINT_PAGINATE", "1");
        }
        inject_print_page_dividers(&mut primitives, MediaType::Print, 800.0, &root, 1056.0);
        unsafe {
            std::env::remove_var("ZW_PRINT_PAGINATE");
        }
        let ys: Vec<f32> = primitives.fills.iter().map(|f| f.rect.origin.y).collect();
        assert_eq!(primitives.fills.len(), 2, "letter: 3 pages → 2 dividers");
        assert!(
            (ys[0] - 1056.0).abs() < 0.1,
            "first boundary at letter height 1056, got {}",
            ys[0]
        );
        assert!(
            (ys[1] - 2112.0).abs() < 0.1,
            "second boundary at 2×1056=2112, got {}",
            ys[1]
        );
    }
}

/// 提取首个样式规则的指定属性声明值（用于断言 collect_stylesheets 注入的合成规则）。
#[cfg(test)]
fn first_style_decl_value(rules: &[zero_css_parser::ast::Rule], property: &str) -> Option<String> {
    use zero_css_parser::ast::Rule;
    for rule in rules {
        if let Rule::Style(sr) = rule {
            for decl in &sr.declarations {
                if decl.property == property {
                    return Some(decl.value.clone());
                }
            }
        }
    }
    None
}

/// `<meta name="color-scheme">` HTML presentational hint 注入测试。
/// driving：真实世界暗模式 opt-in 最常见写法（`<meta name="color-scheme" content="dark">`），
/// 等价 `html { color-scheme: dark }`。
#[cfg(test)]
mod meta_color_scheme_hint_tests {
    use super::*;

    /// `<meta name="color-scheme" content="dark">` → 注入合成 `html { color-scheme: dark }`
    /// 规则（首位，最低优先级）。
    #[test]
    fn meta_color_scheme_dark_injects_root_rule() {
        let html = r#"<html><head><meta name="color-scheme" content="dark"></head><body></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert_eq!(sheets.len(), 1, "应注入 1 个合成 stylesheet");
        assert_eq!(
            first_style_decl_value(&sheets[0].rules, "color-scheme").as_deref(),
            Some("dark"),
            "合成规则应含 color-scheme: dark"
        );
    }

    /// 多值 content（`light dark`）原样传递，由 used-scheme 合成消费。
    #[test]
    fn meta_color_scheme_multi_value_passed_through() {
        let html = r#"<html><head><meta name="color-scheme" content="light dark"></head><body></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert_eq!(
            first_style_decl_value(&sheets[0].rules, "color-scheme").as_deref(),
            Some("light dark"),
            "多值 content 应原样作 color-scheme 值"
        );
    }

    /// 仅首个（树序）`<meta name="color-scheme">` 生效（HTML spec）。
    #[test]
    fn meta_color_scheme_only_first_in_tree_order_wins() {
        let html = r#"<html><head>
            <meta name="color-scheme" content="dark">
            <meta name="color-scheme" content="light">
        </head><body></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert_eq!(
            first_style_decl_value(&sheets[0].rules, "color-scheme").as_deref(),
            Some("dark"),
            "首个树序 meta（dark）应胜出"
        );
    }

    /// 合成规则注入为最低优先级（vector 首位），author `<style>` 在其后 → 可覆盖。
    #[test]
    fn meta_color_scheme_lowest_precedence_before_author_style() {
        let html = r#"<html><head><meta name="color-scheme" content="dark"></head>
            <body><style>html { color-scheme: light; }</style></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert!(sheets.len() >= 2, "meta 合成 + author style 至少 2 个");
        // 首位 = meta 合成（dark）；author style 在后（更高优先级，cascade 覆盖）
        assert_eq!(
            first_style_decl_value(&sheets[0].rules, "color-scheme").as_deref(),
            Some("dark"),
            "meta 合成应为首位（最低优先级）"
        );
    }

    /// 无关 meta（如 viewport）不应注入 color-scheme。
    #[test]
    fn unrelated_meta_does_not_inject_color_scheme() {
        let html = r#"<html><head><meta name="viewport" content="width=device-width"></head><body></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert!(sheets.is_empty(), "viewport meta 不应注入任何 stylesheet");
    }

    /// 空 content 不注入。
    #[test]
    fn meta_color_scheme_empty_content_not_injected() {
        let html = r#"<html><head><meta name="color-scheme" content=""></head><body></body></html>"#;
        let doc = zero_dom::parse_html(html);
        let sheets = collect_stylesheets(&doc, "");
        assert!(sheets.is_empty(), "空 content 不应注入");
    }
}
