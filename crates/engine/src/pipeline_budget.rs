//! 分步预算渲染 — 将 parse/style/layout/paint 拆成可中断步骤，避免单帧长时间阻塞。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_layout_engine::LayoutResult;
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

use crate::animation::compositor_export::{ExportedAnimationSet, export_composable_animations};
use crate::paint::Painter;
use crate::pipeline::{
    PipelineTimings, RenderPipeline, RenderResult, apply_animation_overrides, collect_stylesheets,
    inject_pseudo_text_nodes, paint_cull_viewport,
};

/// 预算渲染会话的当前步骤。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetStep {
    /// 等待开始。
    Pending,
    /// HTML 解析。
    Parse,
    /// 样式计算。
    Style,
    /// 布局。
    Layout,
    /// 绘制。
    Paint,
    /// 已完成。
    Done,
}

/// 预算渲染推进结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetAdvance {
    /// 仍在进行中，调用方应稍后继续。
    InProgress,
    /// 本帧预算内已完成全部步骤。
    Complete,
}

/// 分步预算渲染会话 — 持有中间状态，跨多帧推进。
pub struct BudgetedRenderSession {
    html: String,
    css: String,
    step: BudgetStep,
    doc: Option<Document>,
    stylesheets: Vec<Stylesheet>,
    styles: HashMap<NodeId, ComputedStyle>,
    /// M2-S1b 缺口收口：Style 步导出的合成动画集（flag 关恒 None），Paint 步消费。
    export_set: Option<ExportedAnimationSet>,
    layout_result: Option<LayoutResult>,
    timings: PipelineTimings,
    result: Option<RenderResult>,
    /// S1 文档级指纹键：会话首个 budget 步计算一次（环境快照 = 会话起点状态），
    /// 跨多次 advance 保持（预算中断续推的会话 Complete 时仍可回填）；None = 禁用。
    fingerprint_key: Option<u128>,
}

impl BudgetedRenderSession {
    /// 创建尚未开始的预算渲染会话。
    pub fn new(html: impl Into<String>, css: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            css: css.into(),
            step: BudgetStep::Pending,
            doc: None,
            stylesheets: Vec::new(),
            styles: HashMap::new(),
            export_set: None,
            layout_result: None,
            timings: PipelineTimings::default(),
            result: None,
            fingerprint_key: None,
        }
    }

    /// 当前步骤。
    pub fn step(&self) -> BudgetStep {
        self.step
    }

    /// 是否已完成。
    pub fn is_complete(&self) -> bool {
        self.step == BudgetStep::Done
    }

    /// 各阶段耗时（完成后有效）。
    pub fn timings(&self) -> &PipelineTimings {
        &self.timings
    }

    /// 取出渲染结果（完成后调用一次）。
    pub fn take_result(&mut self) -> Option<RenderResult> {
        self.result.take()
    }
}

impl RenderPipeline {
    /// 在 `budget_ms` 毫秒内尽可能推进预算渲染会话。
    pub fn advance_budgeted_render(&mut self, session: &mut BudgetedRenderSession, budget_ms: f64) -> BudgetAdvance {
        if session.step == BudgetStep::Done {
            return BudgetAdvance::Complete;
        }

        let deadline = Instant::now() + std::time::Duration::from_secs_f64(budget_ms / 1000.0);

        // S1 文档级指纹短路（设计卡 §4-S1；键范式与 kill-switch 见
        // pipeline::frame_fingerprint 模块注记）：同输入热导航的新会话命中上一
        // styled 帧时，跳过 parse/style/layout/paint 四段整帧复用。OPTIMIZATION：
        // 同输入重算产出逐位相同帧（同 HTML/CSS/环境 → 同 DOM/级联/几何/图元），
        // 重算纯属重复计算；值跨 NodeId 代际可复用（ComputedStyle 零 NodeId 内嵌）。
        // 键在会话首个 budget 步计算一次（环境快照 = 会话起点状态），跨多次 advance
        // 保持——预算中断续推的会话 Complete 时仍可回填。
        if session.fingerprint_key.is_none() && session.step != BudgetStep::Done {
            session.fingerprint_key =
                crate::pipeline::frame_fingerprint::frame_fingerprint_key(self, &session.html, &session.css);
        }
        if session.step == BudgetStep::Pending
            && let Some(key) = session.fingerprint_key
        {
            // 一次性克隆命中帧（即命中开销：图元/样式 O(n) 拷贝 + 布局 O(1) Arc），
            // 结束借用后方可写回管线状态。
            let hit = self.frame_fingerprint_cache.lookup(key).map(|c| {
                (
                    c.doc.clone(),
                    c.styles.clone(),
                    c.stylesheets.clone(),
                    c.export_set.clone(),
                    c.result.clone(),
                )
            });
            if let Some((doc, styles, stylesheets, export_set, result)) = hit {
                let t_hit = Instant::now();
                // 命中臂逐一复刻全量 Complete 副作用（下方 Done 分支同序同语义）：
                // cached_doc/cached_styles 写回、CSS 解析缓存失效（DOM 已换代）。
                self.cached_doc = Some(doc);
                self.cached_styles = styles;
                self.cached_css_text = None;
                // slice37/slice38 换代清理（handle→NodeId 持久表 + form live value
                // 两表）——热导航会话同为文档换代，防旧代 NodeId ABA 污染。
                self.clear_persistent_handle_nodes();
                self.form_control_values.clear();
                self.form_control_compositions.clear();
                self.cached_layout = Some(result.layout.clone());
                // 动画时钟注册（全量在 Style 步内联；keyframes registry 同名
                // 覆盖幂等，重复注册无累积）。
                self.animation_clock
                    .register_from_stylesheets(&stylesheets, &self.media_context());
                // M2-S1b 导出缓存（flag 关恒 None，与全量同语义）。
                self.compositor_export_cache = if self.compositor_animations {
                    export_set
                        .as_ref()
                        .map(|set| (set.declarations.clone(), result.animation_scopes.clone()))
                } else {
                    None
                };
                let hit_ms = t_hit.elapsed().as_secs_f64() * 1000.0;
                session.timings = PipelineTimings {
                    parse_ms: 0.0,
                    style_ms: 0.0,
                    layout_ms: 0.0,
                    paint_ms: 0.0,
                    total_ms: hit_ms,
                    parse_count: 0,
                    style_count: 0,
                    layout_count: 0,
                    paint_count: 0,
                };
                session.export_set = export_set;
                session.result = Some(result);
                session.step = BudgetStep::Done;
                return BudgetAdvance::Complete;
            }
        }

        while Instant::now() < deadline {
            match session.step {
                BudgetStep::Pending | BudgetStep::Parse => {
                    let start = Instant::now();
                    let mut doc = zero_dom::parse_html(&session.html);
                    // R3169 注入页面 URL（导航层 → Document），供 native document.URL/documentURI 读。
                    doc.set_url(self.document_url.clone());
                    doc.set_referrer(self.referrer.clone());
                    session.doc = Some(doc);
                    session.timings.parse_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.step = BudgetStep::Style;
                }
                BudgetStep::Style => {
                    let doc = session.doc.as_ref().expect("parse must run before style");
                    let start = Instant::now();
                    session.stylesheets = collect_stylesheets(doc, &session.css);
                    self.style_system
                        .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
                    session.styles = self.style_system.compute_styles(doc, &session.stylesheets);
                    if let Some(doc_mut) = session.doc.as_mut() {
                        inject_pseudo_text_nodes(doc_mut, &mut session.styles, &session.stylesheets);
                    }
                    // M2-S1b 缺口收口（DC-2/M3）：预算路径与 render_html 同源导出。
                    // 生产导航帧全部经本路径（AsyncPageLoad 分步渲染），此前恒不导出
                    // （原「稳定后 render() 全量帧接手」的记档假设在生产无调用方——
                    // 声明永不达 compositor，keep-alive 自举死锁）。导出在 t=0 叠加
                    // 前取 underlying 快照，与 render_html 同序。
                    if session
                        .styles
                        .values()
                        .any(|s| s.animation_name.iter().any(|n| !n.is_empty() && n != "none"))
                    {
                        self.animation_clock
                            .register_from_stylesheets(&session.stylesheets, &self.media_context());
                        if self.compositor_animations {
                            let doc = session.doc.as_ref().expect("parse must run before style");
                            session.export_set = Some(export_composable_animations(
                                &self.animation_clock,
                                doc,
                                &session.styles,
                            ));
                        }
                        // t=0 求值是渲染近似非真实时间轴——事件丢弃，只保留插值叠加
                        //（已导出属性跳写，compositor 在干净基线上采样）。
                        let mut static_clock = self.animation_clock.clone();
                        for (nid, kind, _name, _elapsed) in apply_animation_overrides(
                            &mut static_clock,
                            &mut session.styles,
                            0.0,
                            session.export_set.as_ref(),
                        ) {
                            let _ = (nid, kind);
                        }
                    }
                    session.timings.style_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.step = BudgetStep::Layout;
                }
                BudgetStep::Layout => {
                    let doc = session.doc.as_ref().expect("parse must run before layout");
                    let start = Instant::now();
                    let (img_sizes, img_ratios, img_no_ratio) = self.build_img_intrinsic_all(doc);
                    session.layout_result = Some(self.layout_engine.compute_with_img_intrinsic(
                        doc,
                        &session.styles,
                        img_sizes,
                        img_ratios,
                        img_no_ratio,
                    ));
                    session.timings.layout_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.step = BudgetStep::Paint;
                }
                BudgetStep::Paint => {
                    let doc = session.doc.as_ref().expect("parse must run before paint");
                    let layout = session.layout_result.as_ref().expect("layout must run before paint");
                    let start = Instant::now();
                    let mut painter = Painter::new();
                    painter.skip_indicators = self.skip_indicators;
                    painter.image_sizes.clone_from(&self.image_sizes);
                    painter.image_solid_colors.clone_from(&self.image_solid_colors);
                    painter.image_solid_colors.clone_from(&self.image_solid_colors);
                    painter.image_no_ratio_keys = self.image_no_ratio.clone();
                    painter.image_ratio_keys = self.image_ratios.clone();
                    painter.set_font_resolver(self.font_resolver.clone());
                    painter.set_document_url(self.document_url.as_deref());
                    painter.set_canvas_registry(self.canvas_registry.clone());
                    painter.register_counter_styles(&crate::pipeline::flatten_conditional_rules(
                        &session.stylesheets,
                        &self.media_context(),
                    ));
                    painter.viewport_w = self.viewport_width;
                    painter.viewport_h = self.viewport_height;
                    painter.paint_skip_nodes = layout.paint_skip_node_ids.clone();
                    // M2-S1b 缺口收口：导出通道表注入 painter（空表 = flag 关/无导出，
                    // paint 行为不变），paint 期收集动画作用域。
                    if let Some(set) = &session.export_set {
                        painter.set_compositor_animation_channels(set.channels.clone());
                    }
                    painter.paint(&layout.root, &session.styles, Some(doc));
                    let canvas_images = painter.canvas_images.clone();
                    let mut animation_scopes = painter.take_animation_scopes();
                    let mut primitives = painter.into_primitives();
                    let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout.root);
                    // S7b：cull_invisible 原位剔除；导出域开启时同点修复动画作用域的
                    // draw 区间/transform 槽位（与 render_html 同款）。
                    let stats = primitives.cull_invisible_with_scopes(
                        viewport,
                        session.export_set.is_some().then_some(&mut animation_scopes),
                    );
                    // S7：draw_order 路径跳过 batch_fills（纯 clone no-op）
                    let primitives = if primitives.draw_order.is_empty() {
                        primitives.batch_fills()
                    } else {
                        primitives
                    };
                    session.timings.paint_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.timings.total_ms = session.timings.parse_ms
                        + session.timings.style_ms
                        + session.timings.layout_ms
                        + session.timings.paint_ms;
                    session.step = BudgetStep::Done;

                    self.cached_doc = session.doc.take().map(|d| Rc::new(RefCell::new(d)));
                    self.cached_styles = session.styles.clone();
                    // DOM 已替换：CSS 解析缓存失效（见 RenderPipeline.cached_css_text 注释）。
                    self.cached_css_text = None;
                    // slice37（mm-regression 根修，PR #93 审查 F1）：DOM 已换代即清
                    // handle→NodeId 持久表——预算路径的子资源重绘（图片到达/字体
                    // rerender/lazy 补绘）反复走本换代点，与 render_html 尾部同 R100
                    // 换代语义；不清则旧代 NodeId 经下一 apply 预植（js_dom_bridge
                    // apply_dom_mutations_full）ABA 命中同槽新节点，与跨文档缺陷同
                    // 机制同签名。导航边界清零（prepare_document_state）保留作双保险。
                    self.clear_persistent_handle_nodes();
                    // slice38（PR #93 同族收口，slice37 复核 §5 备案）：换代连 form
                    // live value 两表一起清——`form_control_values`/
                    // `form_control_compositions` 同按 NodeId 键，旧代键换代后失锚，
                    // 残留经三条路径 ABA 污染新代文档：表单提交快照/reset 消费方
                    //（form_control_value_overrides）、结构性 apply 的 retained 抢救
                    //（restore 锚错节点）、下一次 render_html 的 painter 灌值（ghost
                    // 文本画进无关 input）。与 render_html 尾部三表同清同语义。导航
                    // 边界刻意不清（prepare_document_state 只翻标志不换 doc，旧键仍
                    // 锚活文档；导航失败旧文档存活时用户输入必须保留）。
                    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value
                    self.form_control_values.clear();
                    self.form_control_compositions.clear();
                    let layout_out = LayoutResult {
                        root: layout.root.clone(),
                        viewport_width: layout.viewport_width,
                        viewport_height: layout.viewport_height,
                        paint_skip_node_ids: layout.paint_skip_node_ids.clone(),
                    };
                    self.cached_layout = Some(LayoutResult {
                        root: layout.root.clone(),
                        viewport_width: layout.viewport_width,
                        viewport_height: layout.viewport_height,
                        paint_skip_node_ids: layout.paint_skip_node_ids.clone(),
                    });

                    let dirty_rects = vec![(0.0, 0.0, self.viewport_width, self.viewport_height)];
                    // M2-S1b 缺口收口：缓存本次导出（声明 + 已修复作用域）供增量臂
                    // 携带；flag 关恒 None（与 render_html 尾部同语义）。
                    self.compositor_export_cache = if self.compositor_animations {
                        session
                            .export_set
                            .as_ref()
                            .map(|set| (set.declarations.clone(), animation_scopes.clone()))
                    } else {
                        None
                    };
                    let export_decls = session
                        .export_set
                        .as_ref()
                        .map(|set| set.declarations.clone())
                        .unwrap_or_default();
                    session.result = Some(crate::pipeline::make_render_result(
                        primitives,
                        dirty_rects,
                        layout_out,
                        session.timings.clone(),
                        stats,
                        canvas_images,
                        export_decls,
                        animation_scopes,
                    ));
                    // S1 回填：完整帧输出（doc/styles/stylesheets/export_set/result）
                    // 入跨代缓存。键=会话起点环境快照，值=该起点输入下的输出——
                    // 图片批/字体重绘轮各自以当轮起点为键回填，热导航重演同批序时
                    // 逐轮命中；不同批序只见异键 MISS 回退全量（健全）。
                    if let Some(key) = session.fingerprint_key
                        && let Some(result) = &session.result
                    {
                        self.frame_fingerprint_cache.insert(
                            key,
                            crate::pipeline::frame_fingerprint::CachedFrame {
                                doc: self.cached_doc.clone().expect("cached_doc set above"),
                                styles: session.styles.clone(),
                                stylesheets: session.stylesheets.clone(),
                                export_set: session.export_set.clone(),
                                result: result.clone(),
                            },
                        );
                    }
                    return BudgetAdvance::Complete;
                }
                BudgetStep::Done => return BudgetAdvance::Complete,
            }
        }

        BudgetAdvance::InProgress
    }

    /// 视口优先绘制 — 仅绘制与 `visible_rect` 相交的布局子树（用于滚动优化）。
    pub fn render_html_in_rect(&mut self, html: &str, css: &str, visible_rect: Rect) -> RenderResult {
        let total_start = Instant::now();
        let parse_start = Instant::now();
        let mut doc = zero_dom::parse_html(html);
        // R3169 注入页面 URL（导航层 → Document），供 native document.URL/documentURI 读。
        doc.set_url(self.document_url.clone());
        // R3176 注入 referrer（导航层 → Document = 导航前的页面 URL），供 native document.referrer 读。
        doc.set_referrer(self.referrer.clone());
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        let stylesheets = collect_stylesheets(&doc, css);
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let mut styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        inject_pseudo_text_nodes(&mut doc, &mut styles, &stylesheets);

        let layout_start = Instant::now();
        let (img_sizes, _img_ratios, img_no_ratio) = self.build_img_intrinsic_all(&doc);
        let layout_result = self.layout_engine.compute_with_img_intrinsic(
            &doc,
            &styles,
            img_sizes,
            std::collections::HashMap::new(),
            img_no_ratio,
        );
        let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;

        let paint_start = Instant::now();
        let mut painter = Painter::new();
        painter.skip_indicators = self.skip_indicators;
        painter.image_sizes.clone_from(&self.image_sizes);
        painter.image_solid_colors.clone_from(&self.image_solid_colors);
        painter.image_no_ratio_keys = self.image_no_ratio.clone();
        painter.image_ratio_keys = self.image_ratios.clone();
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.set_canvas_registry(self.canvas_registry.clone());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint_in_rect(&layout_result.root, &styles, &visible_rect, Some(&doc));
        let canvas_images = painter.canvas_images.clone();
        let primitives = painter.into_primitives();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;
        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.cached_doc = Some(Rc::new(RefCell::new(doc)));
        // DOM 已替换：CSS 解析缓存失效（见 RenderPipeline.cached_css_text 注释）。
        self.cached_css_text = None;
        // slice38：与 render_html 尾部同 R100 换代语义——doc 全量替换后旧代 NodeId
        // 键（handle 表 + form live value 两表）全部失锚，三表同清。本函数当前全仓
        // 零调用方（死路径），语义对齐防未来复活时携带 PR #93 同族缺口（旧代键经
        // slotmap 换代 ABA 命中新代同槽节点）。
        // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value
        self.persistent_handle_nodes.clear();
        self.form_control_values.clear();
        self.form_control_compositions.clear();
        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
        };
        self.cached_layout = Some(layout_result);

        let dirty_rects = vec![(
            visible_rect.origin.x,
            visible_rect.origin.y,
            visible_rect.size.width,
            visible_rect.size.height,
        )];
        crate::pipeline::make_render_result(
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
            zero_render_foundation::primitive::RenderStats::default(),
            canvas_images,
            // 预算路径不进合成导出域（同上，M2-S1b 记档缺口）。
            Vec::new(),
            Vec::new(),
        )
    }
}
