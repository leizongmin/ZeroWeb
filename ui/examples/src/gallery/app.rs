use std::collections::HashSet;

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetId, WidgetSpec,
};
use zero_ui_runtime::{UiApp, WidgetHost};

use super::highlight::{highlight_rust, highlight_yaml, token_color};
use super::model::{DemoPage, GroupId, Locale};
use super::pages::ALL_PAGES;

/// 画廊应用状态
pub struct GalleryApp {
    pub current_page: String,
    pub locale: Locale,
    pub collapsed_groups: HashSet<GroupId>,
    pub search_query: String,
}

impl GalleryApp {
    pub fn new() -> GalleryApp {
        GalleryApp {
            current_page: String::from("button"),
            locale: Locale::En,
            collapsed_groups: HashSet::new(),
            search_query: String::new(),
        }
    }

    fn current_page_info(&self) -> Option<&'static DemoPage> {
        ALL_PAGES.iter().find(|p| p.id == self.current_page.as_str())
    }

    fn filtered_pages(&self) -> Vec<&'static DemoPage> {
        if self.search_query.is_empty() {
            ALL_PAGES.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            ALL_PAGES.iter().filter(|p| {
                p.id.contains(&q) || p.title.to_lowercase().contains(&q) || p.title_zh.contains(&q)
            }).collect()
        }
    }

    pub fn root_spec(&self) -> WidgetSpec {
        let mut root = WidgetSpec::new("Column");
        root.id = Some(WidgetId::new("gallery_root"));

        // Header bar
        root.children.push(self.build_header());

        // Body: sidebar + demo area
        let mut body = WidgetSpec::new("Row");
        body.id = Some(WidgetId::new("gallery_body"));
        body.props.insert("gap", Value::Float(0.0));

        body.children.push(self.build_sidebar());
        body.children.push(self.build_demo_area());

        root.children.push(body);
        root
    }

    fn build_header(&self) -> WidgetSpec {
        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("gallery_header"));

        let mut title = WidgetSpec::new("HeaderTitle");
        title.id = Some(WidgetId::new("header_title"));
        title.props.insert("locale", Value::Text(match self.locale {
            Locale::En => "Component Gallery".into(),
            Locale::Zh => "组件画廊".into(),
        }));
        row.children.push(title);

        let mut spacer = WidgetSpec::new("Spacer");
        spacer.id = Some(WidgetId::new("header_spacer"));
        row.children.push(spacer);

        let mut locale_btn = WidgetSpec::new("HeaderButton");
        locale_btn.id = Some(WidgetId::new("locale_btn"));
        locale_btn.props.insert("label", Value::Text(self.locale.label().into()));
        locale_btn.props.insert("action", Value::Text("gallery.locale.toggle".into()));
        row.children.push(locale_btn);

        row
    }

    fn build_sidebar(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("sidebar"));

        // Search box area
        let mut search_box = WidgetSpec::new("NavSearch");
        search_box.id = Some(WidgetId::new("nav_search"));
        search_box.props.insert("query", Value::Text(self.search_query.clone()));
        col.children.push(search_box);

        // Group navigation
        let filtered = self.filtered_pages();
        let mut groups_seen = HashSet::new();
        for page in &filtered {
            if !groups_seen.contains(&page.group) {
                groups_seen.insert(page.group);
                let is_collapsed = self.collapsed_groups.contains(&page.group);
                let mut group_header = WidgetSpec::new("GroupHeader");
                group_header.id = Some(WidgetId::new(&format!("group_{}", page.id)));
                group_header.props.insert("label", Value::Text(
                    page.group.name_en().into()
                ));
                group_header.props.insert("collapsed", Value::Bool(is_collapsed));
                group_header.props.insert("group", Value::Text(format!("{:?}", page.group)));
                col.children.push(group_header);

                if !is_collapsed {
                    for p in filtered.iter().filter(|p| p.group == page.group) {
                        let mut nav = WidgetSpec::new("NavItem");
                        nav.id = Some(WidgetId::new(&format!("nav_{}", p.id)));
                        nav.props.insert("label", Value::Text(p.title_for(self.locale).into()));
                        nav.props.insert("page_id", Value::Text(p.id.into()));
                        nav.props.insert("selected", Value::Bool(p.id == self.current_page.as_str()));
                        col.children.push(nav);
                    }
                }
            }
        }

        col
    }

    fn build_demo_area(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("demo_area"));

        if let Some(page) = self.current_page_info() {
            let mut title = WidgetSpec::new("DemoTitle");
            title.id = Some(WidgetId::new("demo_title"));
            title.props.insert("text", Value::Text(page.title_for(self.locale).into()));
            title.props.insert("desc", Value::Text(page.description_for(self.locale).into()));
            col.children.push(title);

            let mut preview = WidgetSpec::new("DemoPreview");
            preview.id = Some(WidgetId::new("demo_preview"));
            preview.props.insert("page_id", Value::Text(page.id.into()));
            col.children.push(preview);

            let mut dsl_label = WidgetSpec::new("SourceLabel");
            dsl_label.id = Some(WidgetId::new("dsl_label"));
            dsl_label.props.insert("text", Value::Text("DSL YAML".into()));
            col.children.push(dsl_label);

            let mut dsl_src = WidgetSpec::new("SourceCode");
            dsl_src.id = Some(WidgetId::new("dsl_source"));
            dsl_src.props.insert("source", Value::Text(page.source_dsl.into()));
            dsl_src.props.insert("lang", Value::Text("yaml".into()));
            col.children.push(dsl_src);

            let mut rust_label = WidgetSpec::new("SourceLabel");
            rust_label.id = Some(WidgetId::new("rust_label"));
            rust_label.props.insert("text", Value::Text("Rust API".into()));
            col.children.push(rust_label);

            let mut rust_src = WidgetSpec::new("SourceCode");
            rust_src.id = Some(WidgetId::new("rust_source"));
            rust_src.props.insert("source", Value::Text(page.source_rust.into()));
            rust_src.props.insert("lang", Value::Text("rust".into()));
            col.children.push(rust_src);
        }

        col
    }
}

impl Default for GalleryApp {
    fn default() -> GalleryApp {
        GalleryApp::new()
    }
}

impl UiApp for GalleryApp {
    fn root_spec(&self) -> WidgetSpec {
        self.root_spec()
    }

    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult {
        match action.0.as_str() {
            "gallery.nav.select" => {
                if let Some(ActionPayload::Text(page_id)) = &payload
                    && ALL_PAGES.iter().any(|p| p.id == page_id.as_str())
                {
                    self.current_page = page_id.clone();
                }
                ActionResult::Handled
            }
            "gallery.locale.toggle" => {
                self.locale = self.locale.toggle();
                ActionResult::Handled
            }
            "gallery.group.toggle" => {
                if let Some(ActionPayload::Text(group)) = &payload {
                    // Parse group name from Debug format
                    if let Some(g) = ALL_PAGES.iter().find_map(|p| {
                        if format!("{:?}", p.group) == *group { Some(p.group) } else { None }
                    }) {
                        if !self.collapsed_groups.remove(&g) {
                            self.collapsed_groups.insert(g);
                        }
                        return ActionResult::Handled;
                    }
                }
                ActionResult::Handled
            }
            "gallery.search" => {
                if let Some(ActionPayload::Text(q)) = &payload {
                    self.search_query = q.clone();
                    return ActionResult::Handled;
                }
                ActionResult::Handled
            }
            _ => ActionResult::UnknownAction(action.clone()),
        }
    }
}

// ========== Custom Widgets ==========

/// Header 标题
pub struct HeaderTitle {
    text: String,
}

impl Widget for HeaderTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(t)) = props.get("text") && t != &self.text {
            self.text = t.clone();
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let w = (self.text.chars().count() as f32 * 9.0).clamp(c.min_width, c.max_width);
        Size::new(w, 40.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder.draw_text(&self.text, Point::new(12.0, 26.0), 18.0, Color::rgb(0.05, 0.05, 0.05));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 弹性占位（填满剩余空间）
pub struct Spacer;

impl Widget for Spacer {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, c.max_height)
    }
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Header 按钮（语言/主题切换等）
pub struct HeaderButton {
    label: String,
    action: ActionId,
    pressed: bool,
}

impl Widget for HeaderButton {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(l)) = props.get("label") { self.label = l.clone(); }
        if let Some(Value::Text(a)) = props.get("action") { self.action = ActionId::new(a); }
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer { phase: PointerPhase::Pressed, .. } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer { phase: PointerPhase::Released, .. } => {
                self.pressed = false;
                EventResult::Emit(self.action.clone())
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(60.0_f32.clamp(c.min_width, c.max_width), 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(60.0, 32.0));
        if self.pressed {
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::ZERO, size),
                Color::rgb(0.7, 0.7, 0.8),
            );
        }
        ctx.recorder.draw_text(&self.label, Point::new(8.0, 22.0), 14.0, Color::rgb(0.1, 0.1, 0.1));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 导航项
pub struct NavItem {
    label: String,
    page_id: String,
    selected: bool,
    pressed: bool,
}

impl Widget for NavItem {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(l)) = props.get("label") { self.label = l.clone(); }
        if let Some(Value::Text(p)) = props.get("page_id") { self.page_id = p.clone(); }
        if let Some(Value::Bool(s)) = props.get("selected") { self.selected = *s; }
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer { phase: PointerPhase::Pressed, .. } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer { phase: PointerPhase::Released, .. } => {
                self.pressed = false;
                EventResult::EmitWithPayload(
                    ActionId::new("gallery.nav.select"),
                    ActionPayload::Text(self.page_id.clone()),
                )
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        if self.selected {
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::ZERO, size),
                Color::rgb(0.8, 0.85, 0.95),
            );
        }
        if self.pressed {
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::ZERO, size),
                Color::rgb(0.7, 0.75, 0.9),
            );
        }
        ctx.recorder.draw_text(&self.label, Point::new(16.0, 22.0), 14.0, Color::rgb(0.15, 0.15, 0.15));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool { true }
}

/// 分组标题
pub struct GroupHeader {
    label: String,
    collapsed: bool,
}

impl Widget for GroupHeader {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(l)) = props.get("label") { self.label = l.clone(); }
        if let Some(Value::Bool(c)) = props.get("collapsed") { self.collapsed = *c; }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 28.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let prefix = if self.collapsed { "▸ " } else { "▾ " };
        let display = format!("{}{}", prefix, self.label);
        ctx.recorder.draw_text(&display, Point::new(8.0, 18.0), 12.0, Color::rgb(0.3, 0.3, 0.3));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 导航搜索框
pub struct NavSearch {
    query: String,
}

impl Widget for NavSearch {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(q)) = props.get("query") { self.query = q.clone(); }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        ctx.recorder.stroke_rect(
            Rect::from_origin_size(Point::ZERO, size),
            Color::rgb(0.6, 0.6, 0.6),
            1.0,
        );
        let display = if self.query.is_empty() {
            "Search...".into()
        } else {
            format!("🔍 {}", self.query)
        };
        ctx.recorder.draw_text(&display, Point::new(8.0, 22.0), 13.0, Color::rgb(0.4, 0.4, 0.4));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Demo 标题区域
pub struct DemoTitle {
    text: String,
    desc: String,
}

impl Widget for DemoTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(t)) = props.get("text") { self.text = t.clone(); }
        if let Some(Value::Text(d)) = props.get("desc") { self.desc = d.clone(); }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let h = 60.0_f32.clamp(c.min_height, c.max_height);
        Size::new(c.max_width, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder.draw_text(&self.text, Point::new(16.0, 24.0), 20.0, Color::rgb(0.05, 0.05, 0.05));
        ctx.recorder.draw_text(&self.desc, Point::new(16.0, 46.0), 13.0, Color::rgb(0.4, 0.4, 0.4));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Demo 预览区
pub struct DemoPreview {
    page_id: String,
}

impl Widget for DemoPreview {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(p)) = props.get("page_id") { self.page_id = p.clone(); }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 120.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, 120.0));
        // 预览背景框
        ctx.recorder.stroke_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(size.width - 16.0, size.height - 8.0)),
            Color::rgb(0.75, 0.75, 0.75),
            1.0,
        );
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(size.width - 16.0, size.height - 8.0)),
            Color::rgb(0.97, 0.97, 0.97),
        );
        let label = format!("{} preview", self.page_id.replace('_', " "));
        ctx.recorder.draw_text(&label, Point::new(20.0, 30.0), 14.0, Color::rgb(0.3, 0.3, 0.3));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 源码标签
pub struct SourceLabel {
    text: String,
}

impl Widget for SourceLabel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(t)) = props.get("text") && t != &self.text {
            self.text = t.clone();
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 24.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder.draw_text(&self.text, Point::new(12.0, 16.0), 12.0, Color::rgb(0.3, 0.3, 0.3));
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 语法高亮源码展示
pub struct SourceCode {
    source: String,
    lang: String,
}

impl Widget for SourceCode {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(s)) = props.get("source") { self.source = s.clone(); }
        if let Some(Value::Text(l)) = props.get("lang") { self.lang = l.clone(); }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult { EventResult::Ignored }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let line_h = 16.0;
        let lines = self.source.lines().count() as f32;
        let h = (lines * line_h).clamp(c.min_height, c.max_height).max(40.0);
        Size::new(c.max_width, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, 100.0));
        // 源码背景
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 0.0), Size::new(size.width - 16.0, size.height)),
            Color::rgb(0.95, 0.95, 0.97),
        );
        // 语法高亮 token 渲染
        let tokens = match self.lang.as_str() {
            "yaml" => highlight_yaml(&self.source),
            "rust" => highlight_rust(&self.source),
            _ => vec![(&self.source as &str, "default")],
        };
        let mut x = 16.0_f32;
        let mut y = 14.0_f32;
        for (text, kind) in &tokens {
            let (r, g, b) = token_color(kind);
            for ch in text.chars() {
                if ch == '\n' {
                    x = 16.0;
                    y += 16.0;
                } else {
                    ctx.recorder.draw_text(
                        &ch.to_string(),
                        Point::new(x, y),
                        12.0,
                        Color::rgb(r, g, b),
                    );
                    x += 7.2;
                }
            }
        }
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ========== Factory Registration ==========

fn str_prop(spec: &WidgetSpec, key: &str) -> Option<String> {
    match spec.props.get(key) {
        Some(Value::Text(s)) => Some(s.clone()),
        _ => None,
    }
}

fn bool_prop(spec: &WidgetSpec, key: &str) -> bool {
    match spec.props.get(key) {
        Some(Value::Bool(b)) => *b,
        _ => false,
    }
}

/// 注册画廊所有自定义控件工厂
pub fn register_gallery_factories(host: &mut WidgetHost) {
    host.register("HeaderTitle", |_spec| Box::new(HeaderTitle { text: String::new() }));
    host.register("Spacer", |_spec| Box::new(Spacer));
    host.register("HeaderButton", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let action = str_prop(spec, "action").map(|a| ActionId::new(&a)).unwrap_or_else(|| ActionId::new("noop"));
        Box::new(HeaderButton { label, action, pressed: false })
    });
    host.register("NavItem", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let page_id = str_prop(spec, "page_id").unwrap_or_default();
        let selected = bool_prop(spec, "selected");
        Box::new(NavItem { label, page_id, selected, pressed: false })
    });
    host.register("GroupHeader", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let collapsed = bool_prop(spec, "collapsed");
        Box::new(GroupHeader { label, collapsed })
    });
    host.register("NavSearch", |spec| {
        let query = str_prop(spec, "query").unwrap_or_default();
        Box::new(NavSearch { query })
    });
    host.register("DemoTitle", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        let desc = str_prop(spec, "desc").unwrap_or_default();
        Box::new(DemoTitle { text, desc })
    });
    host.register("DemoPreview", |spec| {
        let page_id = str_prop(spec, "page_id").unwrap_or_default();
        Box::new(DemoPreview { page_id })
    });
    host.register("SourceLabel", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        Box::new(SourceLabel { text })
    });
    host.register("SourceCode", |spec| {
        let source = str_prop(spec, "source").unwrap_or_default();
        let lang = str_prop(spec, "lang").unwrap_or_else(|| "yaml".into());
        Box::new(SourceCode { source, lang })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::layout::WindowMetrics;

    fn setup_gallery() -> GalleryApp {
        let mut app = GalleryApp::new();
        app.current_page = "button".into();
        app
    }

    #[test]
    fn begin_produces_non_empty_scene() {
        let mut app = setup_gallery();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::tablet());
        driver.host_mut().register("HeaderTitle", |_| Box::new(HeaderTitle { text: String::new() }));
        driver.host_mut().register("Spacer", |_| Box::new(Spacer));
        driver.host_mut().register("HeaderButton", |spec| {
            let label = str_prop(spec, "label").unwrap_or_default();
            let action = str_prop(spec, "action").map(|a| ActionId::new(&a)).unwrap_or_else(|| ActionId::new("noop"));
            Box::new(HeaderButton { label, action, pressed: false })
        });
        driver.host_mut().register("NavItem", |spec| {
            let label = str_prop(spec, "label").unwrap_or_default();
            let page_id = str_prop(spec, "page_id").unwrap_or_default();
            let selected = bool_prop(spec, "selected");
            Box::new(NavItem { label, page_id, selected, pressed: false })
        });
        driver.host_mut().register("GroupHeader", |spec| {
            let label = str_prop(spec, "label").unwrap_or_default();
            let collapsed = bool_prop(spec, "collapsed");
            Box::new(GroupHeader { label, collapsed })
        });
        driver.host_mut().register("NavSearch", |spec| {
            let query = str_prop(spec, "query").unwrap_or_default();
            Box::new(NavSearch { query })
        });
        driver.host_mut().register("DemoTitle", |spec| {
            let text = str_prop(spec, "text").unwrap_or_default();
            let desc = str_prop(spec, "desc").unwrap_or_default();
            Box::new(DemoTitle { text, desc })
        });
        driver.host_mut().register("DemoPreview", |spec| {
            let page_id = str_prop(spec, "page_id").unwrap_or_default();
            Box::new(DemoPreview { page_id })
        });
        driver.host_mut().register("SourceLabel", |spec| {
            let text = str_prop(spec, "text").unwrap_or_default();
            Box::new(SourceLabel { text })
        });
        driver.host_mut().register("SourceCode", |spec| {
            let source = str_prop(spec, "source").unwrap_or_default();
            let lang = str_prop(spec, "lang").unwrap_or_else(|| "yaml".into());
            Box::new(SourceCode { source, lang })
        });
        driver.begin();
        assert!(!driver.host().scene().entries.is_empty(), "Gallery begin 产出非空 scene");
        assert_eq!(driver.pump_frame(), zero_ui_adapter_winit::FrameOutcome::Idle);
    }

    #[test]
    fn nav_click_switches_page() {
        let mut app = setup_gallery();
        assert_eq!(app.current_page, "button");
        // Direct dispatch test
        let action = ActionId::new("gallery.nav.select");
        let payload = Some(ActionPayload::Text("toggle".into()));
        let result = app.dispatch(&action, payload);
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(app.current_page, "toggle");
    }

    #[test]
    fn nav_click_unknown_page_uses_same() {
        let mut app = setup_gallery();
        assert_eq!(app.current_page, "button");
        let action = ActionId::new("gallery.nav.select");
        let payload = Some(ActionPayload::Text("nonexistent".into()));
        let result = app.dispatch(&action, payload);
        assert_eq!(result, ActionResult::Handled);
        // Unknown page should keep current page
        assert_eq!(app.current_page, "button");
    }

    #[test]
    fn locale_toggle_switches_language() {
        let mut app = GalleryApp::new();
        assert_eq!(app.locale, Locale::En);
        let action = ActionId::new("gallery.locale.toggle");
        app.dispatch(&action, None);
        assert_eq!(app.locale, Locale::Zh);
        app.dispatch(&action, None);
        assert_eq!(app.locale, Locale::En);
    }

    #[test]
    fn root_spec_produces_valid_tree() {
        let app = setup_gallery();
        let spec = app.root_spec();
        assert_eq!(spec.component.0.as_str(), "Column");
        assert_eq!(spec.children.len(), 2, "gallery root 应有 header + body");
    }

    #[test]
    fn nav_item_emits_correct_action() {
        let mut nav = NavItem {
            label: "Toggle".into(),
            page_id: "toggle".into(),
            selected: false,
            pressed: false,
        };
        let ev = UiEvent::Pointer {
            phase: PointerPhase::Released,
            button: Some(zero_ui_core::event::PointerButton::Primary),
            position: Point::new(10.0, 10.0),
            modifiers: zero_ui_core::event::Modifiers::NONE,
            pointer_id: 0,
        };
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ctx = EventCtx { invalidation: &mut flags };
        let result = nav.event(&mut ctx, &ev);
        match result {
            EventResult::EmitWithPayload(a, ActionPayload::Text(p)) => {
                assert_eq!(a.0.as_str(), "gallery.nav.select");
                assert_eq!(p.as_str(), "toggle");
            }
            _ => panic!("Expected EmitWithPayload"),
        }
    }

    #[test]
    fn all_pages_have_unique_ids() {
        let mut ids = std::collections::HashSet::new();
        for page in super::super::pages::ALL_PAGES {
            assert!(ids.insert(page.id), "重复 page_id: {}", page.id);
        }
    }
}
