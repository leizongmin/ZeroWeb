//! 分步预算渲染 — 将 parse/style/layout/paint 拆成可中断步骤，避免单帧长时间阻塞。

use std::collections::HashMap;
use std::time::Instant;

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_layout_engine::LayoutResult;
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

use crate::paint::Painter;
use crate::pipeline::{
    PipelineTimings, RenderPipeline, RenderResult, collect_stylesheets, inject_pseudo_text_nodes, paint_cull_viewport,
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
    layout_result: Option<LayoutResult>,
    timings: PipelineTimings,
    result: Option<RenderResult>,
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
            layout_result: None,
            timings: PipelineTimings::default(),
            result: None,
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

        while Instant::now() < deadline {
            match session.step {
                BudgetStep::Pending | BudgetStep::Parse => {
                    let start = Instant::now();
                    session.doc = Some(zero_dom::parse_html(&session.html));
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
                        inject_pseudo_text_nodes(doc_mut, &mut session.styles);
                    }
                    session.timings.style_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.step = BudgetStep::Layout;
                }
                BudgetStep::Layout => {
                    let doc = session.doc.as_ref().expect("parse must run before layout");
                    let start = Instant::now();
                    let img_sizes = self.build_img_intrinsic_sizes(doc);
                    let img_ratios = self.build_img_intrinsic_ratios(doc);
                    let img_no_ratio = self.build_img_intrinsic_no_ratio(doc);
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
                    painter.set_font_resolver(self.font_resolver.clone());
                    painter.set_document_url(self.document_url.as_deref());
                    painter.register_counter_styles(&session.stylesheets);
                    painter.viewport_w = self.viewport_width;
                    painter.viewport_h = self.viewport_height;
                    painter.paint_skip_nodes = layout.paint_skip_node_ids.clone();
                    painter.paint(&layout.root, &session.styles, Some(doc));
                    let primitives = painter.into_primitives();
                    let viewport = paint_cull_viewport(self.viewport_width, self.viewport_height, &layout.root);
                    let (primitives, stats) = primitives.cull_invisible(viewport);
                    let primitives = primitives.batch_fills();
                    session.timings.paint_ms = start.elapsed().as_secs_f64() * 1000.0;
                    session.timings.total_ms = session.timings.parse_ms
                        + session.timings.style_ms
                        + session.timings.layout_ms
                        + session.timings.paint_ms;
                    session.step = BudgetStep::Done;

                    self.cached_doc = session.doc.take();
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

                    session.result = Some(RenderResult {
                        primitives,
                        layout: layout_out,
                        timings: session.timings.clone(),
                        stats,
                    });
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
        let parse_ms = parse_start.elapsed().as_secs_f64() * 1000.0;

        let stylesheets = collect_stylesheets(&doc, css);
        let style_start = Instant::now();
        self.style_system
            .set_viewport(self.viewport_width as f64, self.viewport_height as f64);
        let mut styles = self.style_system.compute_styles(&doc, &stylesheets);
        let style_ms = style_start.elapsed().as_secs_f64() * 1000.0;

        inject_pseudo_text_nodes(&mut doc, &mut styles);

        let layout_start = Instant::now();
        let img_sizes = self.build_img_intrinsic_sizes(&doc);
        let img_no_ratio = self.build_img_intrinsic_no_ratio(&doc);
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
        painter.set_font_resolver(self.font_resolver.clone());
        painter.set_document_url(self.document_url.as_deref());
        painter.viewport_w = self.viewport_width;
        painter.viewport_h = self.viewport_height;
        painter.paint_skip_nodes = layout_result.paint_skip_node_ids.clone();
        painter.paint_in_rect(&layout_result.root, &styles, &visible_rect, Some(&doc));
        let primitives = painter.into_primitives();
        let paint_ms = paint_start.elapsed().as_secs_f64() * 1000.0;
        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        self.cached_doc = Some(doc);
        let layout = LayoutResult {
            root: layout_result.root.clone(),
            viewport_width: layout_result.viewport_width,
            viewport_height: layout_result.viewport_height,
            paint_skip_node_ids: layout_result.paint_skip_node_ids.clone(),
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
            stats: zero_render_foundation::primitive::RenderStats::default(),
        }
    }
}
