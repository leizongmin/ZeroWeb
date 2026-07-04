use std::collections::HashSet;

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetId, WidgetSpec,
};
use zero_ui_runtime::{UiApp, WidgetHost};

use super::highlight::{highlight_rust, highlight_yaml, token_color};
use super::model::{DemoPage, GroupId, Locale, ThemeKind};
use super::pages::ALL_PAGES;

/// 画廊应用状态
pub struct GalleryApp {
    pub current_page: String,
    pub locale: Locale,
    pub theme: ThemeKind,
    pub collapsed_groups: HashSet<GroupId>,
    pub search_query: String,
}

impl GalleryApp {
    pub fn new() -> GalleryApp {
        GalleryApp {
            current_page: String::from("button"),
            locale: Locale::En,
            theme: ThemeKind::Light,
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
            ALL_PAGES
                .iter()
                .filter(|p| p.id.contains(&q) || p.title.to_lowercase().contains(&q) || p.title_zh.contains(&q))
                .collect()
        }
    }

    pub fn root_spec(&self) -> WidgetSpec {
        let mut root = WidgetSpec::new("Column");
        root.id = Some(WidgetId::new("gallery_root"));
        root.props.insert("theme", Value::Text(self.theme.as_str().into()));

        // Header bar
        root.children.push(self.build_header());

        // Body: sidebar + demo area
        let mut body = WidgetSpec::new("Row");
        body.id = Some(WidgetId::new("gallery_body"));
        body.props.insert("theme", Value::Text(self.theme.as_str().into()));
        body.props.insert("gap", Value::Float(0.0));

        body.children.push(self.build_sidebar());
        body.children.push(self.build_demo_area());

        root.children.push(body);
        root
    }

    fn build_header(&self) -> WidgetSpec {
        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("gallery_header"));
        row.props.insert("theme", Value::Text(self.theme.as_str().into()));

        let mut title = WidgetSpec::new("HeaderTitle");
        title.id = Some(WidgetId::new("header_title"));
        title.props.insert("theme", Value::Text(self.theme.as_str().into()));
        title.props.insert(
            "text",
            Value::Text(match self.locale {
                Locale::En => "Component Gallery".into(),
                Locale::Zh => "组件画廊".into(),
            }),
        );
        row.children.push(title);

        let mut spacer = WidgetSpec::new("Spacer");
        spacer.id = Some(WidgetId::new("header_spacer"));
        spacer.props.insert("theme", Value::Text(self.theme.as_str().into()));
        spacer.props.insert("flex", Value::Float(1.0));
        row.children.push(spacer);

        let mut theme_btn = WidgetSpec::new("HeaderButton");
        theme_btn.id = Some(WidgetId::new("theme_btn"));
        theme_btn.props.insert("theme", Value::Text(self.theme.as_str().into()));
        theme_btn
            .props
            .insert("label", Value::Text(self.theme.button_label().into()));
        theme_btn
            .props
            .insert("action", Value::Text("gallery.theme.toggle".into()));
        row.children.push(theme_btn);

        let mut locale_btn = WidgetSpec::new("HeaderButton");
        locale_btn.id = Some(WidgetId::new("locale_btn"));
        locale_btn
            .props
            .insert("theme", Value::Text(self.theme.as_str().into()));
        locale_btn
            .props
            .insert("label", Value::Text(self.locale.label().into()));
        locale_btn
            .props
            .insert("action", Value::Text("gallery.locale.toggle".into()));
        row.children.push(locale_btn);

        row
    }

    fn build_sidebar(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("sidebar"));
        col.props.insert("theme", Value::Text(self.theme.as_str().into()));

        // Search box area
        let mut search_box = WidgetSpec::new("NavSearch");
        search_box.id = Some(WidgetId::new("nav_search"));
        search_box
            .props
            .insert("theme", Value::Text(self.theme.as_str().into()));
        search_box
            .props
            .insert("locale", Value::Text(self.locale.as_str().into()));
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
                group_header
                    .props
                    .insert("theme", Value::Text(self.theme.as_str().into()));
                group_header
                    .props
                    .insert("label", Value::Text(page.group.name_for(self.locale).into()));
                group_header.props.insert("collapsed", Value::Bool(is_collapsed));
                group_header
                    .props
                    .insert("group", Value::Text(format!("{:?}", page.group)));
                col.children.push(group_header);

                if !is_collapsed {
                    for p in filtered.iter().filter(|p| p.group == page.group) {
                        let mut nav = WidgetSpec::new("NavItem");
                        nav.id = Some(WidgetId::new(&format!("nav_{}", p.id)));
                        nav.props.insert("theme", Value::Text(self.theme.as_str().into()));
                        nav.props.insert("label", Value::Text(p.title_for(self.locale).into()));
                        nav.props.insert("page_id", Value::Text(p.id.into()));
                        nav.props
                            .insert("selected", Value::Bool(p.id == self.current_page.as_str()));
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
        col.props.insert("theme", Value::Text(self.theme.as_str().into()));

        if let Some(page) = self.current_page_info() {
            let mut title = WidgetSpec::new("DemoTitle");
            title.id = Some(WidgetId::new("demo_title"));
            title.props.insert("theme", Value::Text(self.theme.as_str().into()));
            title
                .props
                .insert("text", Value::Text(page.title_for(self.locale).into()));
            title
                .props
                .insert("desc", Value::Text(page.description_for(self.locale).into()));
            col.children.push(title);

            let mut preview = WidgetSpec::new("DemoPreview");
            preview.id = Some(WidgetId::new("demo_preview"));
            preview.props.insert("theme", Value::Text(self.theme.as_str().into()));
            preview.props.insert("page_id", Value::Text(page.id.into()));
            col.children.push(preview);

            let mut dsl_label = WidgetSpec::new("SourceLabel");
            dsl_label.id = Some(WidgetId::new("dsl_label"));
            dsl_label.props.insert("theme", Value::Text(self.theme.as_str().into()));
            dsl_label
                .props
                .insert("text", Value::Text(self.locale.dsl_label().into()));
            col.children.push(dsl_label);

            let mut dsl_src = WidgetSpec::new("SourceCode");
            dsl_src.id = Some(WidgetId::new("dsl_source"));
            dsl_src.props.insert("theme", Value::Text(self.theme.as_str().into()));
            dsl_src.props.insert("source", Value::Text(page.source_dsl.into()));
            dsl_src.props.insert("lang", Value::Text("yaml".into()));
            col.children.push(dsl_src);

            let mut rust_label = WidgetSpec::new("SourceLabel");
            rust_label.id = Some(WidgetId::new("rust_label"));
            rust_label
                .props
                .insert("theme", Value::Text(self.theme.as_str().into()));
            rust_label
                .props
                .insert("text", Value::Text(self.locale.rust_label().into()));
            col.children.push(rust_label);

            let mut rust_src = WidgetSpec::new("SourceCode");
            rust_src.id = Some(WidgetId::new("rust_source"));
            rust_src.props.insert("theme", Value::Text(self.theme.as_str().into()));
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
            "gallery.theme.toggle" => {
                self.theme = self.theme.toggle();
                ActionResult::Handled
            }
            "gallery.group.toggle" => {
                if let Some(ActionPayload::Text(group)) = &payload {
                    // Parse group name from Debug format
                    if let Some(g) = ALL_PAGES.iter().find_map(|p| {
                        if format!("{:?}", p.group) == *group {
                            Some(p.group)
                        } else {
                            None
                        }
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

/// 从 props 解析 theme（缺失或非法时回落 Light，与默认主题 token 一致）。
fn theme_from_props(props: &Props) -> ThemeKind {
    match props.get("theme") {
        Some(Value::Text(s)) => ThemeKind::parse_str(s).unwrap_or(ThemeKind::Light),
        _ => ThemeKind::Light,
    }
}

/// 从 `locale` prop 解析当前语言（非法/缺省回落 En）。
fn locale_from_props(props: &Props) -> Locale {
    match props.get("locale") {
        Some(Value::Text(s)) => Locale::parse_str(s).unwrap_or(Locale::En),
        _ => Locale::En,
    }
}

/// 由 ThemeKind 取 semantic token 色板（与 host 持有的 tokens 同口径）。
fn tokens_for(theme: ThemeKind) -> SemanticTokens {
    match theme {
        ThemeKind::Light => SemanticTokens::light(),
        ThemeKind::Dark => SemanticTokens::dark(),
    }
}

/// Header 标题
pub struct HeaderTitle {
    text: String,
    theme: ThemeKind,
}

impl Widget for HeaderTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = false;
        if new_theme != self.theme {
            self.theme = new_theme;
            changed = true;
        }
        if let Some(Value::Text(t)) = props.get("text")
            && t != &self.text
        {
            self.text = t.clone();
            changed = true;
        }
        if changed {
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
        let tokens = tokens_for(self.theme);
        ctx.recorder
            .draw_text(&self.text, Point::new(12.0, 26.0), 18.0, tokens.on_background);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 弹性占位（与 `flex: 1` prop 配合吃剩余主轴空间）。
///
/// 由于 layout 引擎不会把 clamp 后的尺寸写回子节点 `cached_size`（仅用于父级排列），
/// Spacer 必须自己控制返回的尺寸：仅吃主轴、cross 维度返回 0，避免把父容器
/// 的 cross（常常是窗口另一维）拉满导致兄弟容器被挤出视口。
/// `axis` prop 标识所在容器（"horizontal"=Row / "vertical"=Column）。
pub struct Spacer {
    axis: String,
}

impl Widget for Spacer {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(a)) = props.get("axis")
            && a != &self.axis
        {
            self.axis = a.clone();
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 主轴吃满父级给的 max；cross 维度返回 0，由父容器 cross-axis alignment 决定位置。
        match self.axis.as_str() {
            "vertical" => Size::new(0.0, c.max_height.max(c.min_height)),
            _ => Size::new(c.max_width.max(c.min_width), 0.0),
        }
    }
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Header 按钮（语言/主题切换等）
pub struct HeaderButton {
    label: String,
    action: ActionId,
    pressed: bool,
    theme: ThemeKind,
}

impl Widget for HeaderButton {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(l)) = props.get("label")
            && l != &self.label
        {
            self.label = l.clone();
            changed = true;
        }
        if let Some(Value::Text(a)) = props.get("action") {
            self.action = ActionId::new(a);
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            } => {
                self.pressed = false;
                EventResult::Emit(self.action.clone())
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(
            60.0_f32.clamp(c.min_width, c.max_width),
            32.0_f32.clamp(c.min_height, c.max_height),
        )
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(60.0, 32.0));
        let bg = if self.pressed { tokens.primary } else { tokens.surface };
        ctx.recorder.fill_rect(Rect::from_origin_size(Point::ZERO, size), bg);
        let on_bg = if self.pressed {
            tokens.on_primary
        } else {
            tokens.on_surface
        };
        ctx.recorder.draw_text(&self.label, Point::new(8.0, 22.0), 14.0, on_bg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 导航项
pub struct NavItem {
    label: String,
    page_id: String,
    selected: bool,
    pressed: bool,
    theme: ThemeKind,
}

impl Widget for NavItem {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(l)) = props.get("label")
            && l != &self.label
        {
            self.label = l.clone();
            changed = true;
        }
        if let Some(Value::Text(p)) = props.get("page_id") {
            self.page_id = p.clone();
        }
        if let Some(Value::Bool(s)) = props.get("selected")
            && *s != self.selected
        {
            self.selected = *s;
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            } => {
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
        // 内容宽度 = 缩进(16) + 文本宽度估算（每字符 ~8）+ 右内边距(16)。
        let content_w = 16.0 + self.label.chars().count() as f32 * 8.0 + 16.0;
        let w = content_w.clamp(c.min_width, c.max_width);
        Size::new(w, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        if self.selected {
            // 选中态用 primary 的低饱和 washed 变体（与 primary 30% 混合 on surface）。
            let washed = Color::rgb(
                tokens.primary.r * 0.35 + tokens.surface.r * 0.65,
                tokens.primary.g * 0.35 + tokens.surface.g * 0.65,
                tokens.primary.b * 0.35 + tokens.surface.b * 0.65,
            );
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), washed);
        }
        if self.pressed {
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.primary);
            ctx.recorder
                .draw_text(&self.label, Point::new(16.0, 22.0), 14.0, tokens.on_primary);
            return;
        }
        ctx.recorder
            .draw_text(&self.label, Point::new(16.0, 22.0), 14.0, tokens.on_surface);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

/// 分组标题
pub struct GroupHeader {
    label: String,
    collapsed: bool,
    theme: ThemeKind,
}

impl Widget for GroupHeader {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(l)) = props.get("label")
            && l != &self.label
        {
            self.label = l.clone();
            changed = true;
        }
        if let Some(Value::Bool(c)) = props.get("collapsed")
            && *c != self.collapsed
        {
            self.collapsed = *c;
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let content_w = 8.0 + self.label.chars().count() as f32 * 8.0 + 8.0;
        let w = content_w.clamp(c.min_width, c.max_width);
        Size::new(w, 28.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let prefix = if self.collapsed { "▸ " } else { "▾ " };
        let display = format!("{}{}", prefix, self.label);
        // 二级文本用 on_background 与 surface 之间的中间灰。
        let fg = Color::rgb(
            tokens.on_background.r * 0.6 + tokens.surface.r * 0.4,
            tokens.on_background.g * 0.6 + tokens.surface.g * 0.4,
            tokens.on_background.b * 0.6 + tokens.surface.b * 0.4,
        );
        ctx.recorder.draw_text(&display, Point::new(8.0, 18.0), 12.0, fg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 导航搜索框
pub struct NavSearch {
    query: String,
    theme: ThemeKind,
    locale: Locale,
}

impl Widget for NavSearch {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let new_locale = locale_from_props(props);
        let mut changed = new_theme != self.theme || new_locale != self.locale;
        self.theme = new_theme;
        self.locale = new_locale;
        if let Some(Value::Text(q)) = props.get("query")
            && q != &self.query
        {
            self.query = q.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 侧栏建议 200 宽；如果父级给的 max_width 更小，遵循父级。
        let w = 200.0_f32.clamp(c.min_width, c.max_width);
        Size::new(w, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        // 边框用 surface 的弱化变体。
        let border = Color::rgb(
            tokens.on_background.r * 0.4 + tokens.background.r * 0.6,
            tokens.on_background.g * 0.4 + tokens.background.g * 0.6,
            tokens.on_background.b * 0.4 + tokens.background.b * 0.6,
        );
        ctx.recorder
            .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.background);
        ctx.recorder
            .stroke_rect(Rect::from_origin_size(Point::ZERO, size), border, 1.0);
        let display = if self.query.is_empty() {
            self.locale.search_placeholder().to_string()
        } else {
            format!("🔍 {}", self.query)
        };
        // 占位/搜索结果都走次级色。
        let fg = Color::rgb(
            tokens.on_background.r * 0.5 + tokens.background.r * 0.5,
            tokens.on_background.g * 0.5 + tokens.background.g * 0.5,
            tokens.on_background.b * 0.5 + tokens.background.b * 0.5,
        );
        ctx.recorder.draw_text(&display, Point::new(8.0, 22.0), 13.0, fg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Demo 标题区域
pub struct DemoTitle {
    text: String,
    desc: String,
    theme: ThemeKind,
}

impl Widget for DemoTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(t)) = props.get("text")
            && t != &self.text
        {
            self.text = t.clone();
            changed = true;
        }
        if let Some(Value::Text(d)) = props.get("desc")
            && d != &self.desc
        {
            self.desc = d.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let h = 60.0_f32.clamp(c.min_height, c.max_height);
        Size::new(c.max_width, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        ctx.recorder
            .draw_text(&self.text, Point::new(16.0, 24.0), 20.0, tokens.on_background);
        let desc_fg = Color::rgb(
            tokens.on_background.r * 0.55 + tokens.background.r * 0.45,
            tokens.on_background.g * 0.55 + tokens.background.g * 0.45,
            tokens.on_background.b * 0.55 + tokens.background.b * 0.45,
        );
        ctx.recorder
            .draw_text(&self.desc, Point::new(16.0, 46.0), 13.0, desc_fg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// Demo 预览区
pub struct DemoPreview {
    page_id: String,
    theme: ThemeKind,
}

impl Widget for DemoPreview {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(p)) = props.get("page_id")
            && p != &self.page_id
        {
            self.page_id = p.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 120.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, 120.0));
        let frame = Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(size.width - 16.0, size.height - 8.0));
        ctx.recorder.fill_rect(frame, tokens.surface);
        let border = Color::rgb(
            tokens.on_background.r * 0.3 + tokens.surface.r * 0.7,
            tokens.on_background.g * 0.3 + tokens.surface.g * 0.7,
            tokens.on_background.b * 0.3 + tokens.surface.b * 0.7,
        );
        ctx.recorder.stroke_rect(frame, border, 1.0);
        let label = format!("{} preview", self.page_id.replace('_', " "));
        ctx.recorder
            .draw_text(&label, Point::new(20.0, 30.0), 14.0, tokens.on_surface);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 源码标签
pub struct SourceLabel {
    text: String,
    theme: ThemeKind,
}

impl Widget for SourceLabel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(t)) = props.get("text")
            && t != &self.text
        {
            self.text = t.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 24.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let fg = Color::rgb(
            tokens.on_background.r * 0.6 + tokens.background.r * 0.4,
            tokens.on_background.g * 0.6 + tokens.background.g * 0.4,
            tokens.on_background.b * 0.6 + tokens.background.b * 0.4,
        );
        ctx.recorder.draw_text(&self.text, Point::new(12.0, 16.0), 12.0, fg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 语法高亮源码展示
pub struct SourceCode {
    source: String,
    lang: String,
    theme: ThemeKind,
}

impl Widget for SourceCode {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_theme = theme_from_props(props);
        let mut changed = new_theme != self.theme;
        self.theme = new_theme;
        if let Some(Value::Text(s)) = props.get("source")
            && s != &self.source
        {
            self.source = s.clone();
            changed = true;
        }
        if let Some(Value::Text(l)) = props.get("lang")
            && l != &self.lang
        {
            self.lang = l.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let line_h = 16.0;
        let lines = self.source.lines().count() as f32;
        let h = (lines * line_h).clamp(c.min_height, c.max_height).max(40.0);
        Size::new(c.max_width, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, 100.0));
        // 代码块用比 surface 略亮/略深的"卡片"色：浅色 → 比 surface 略亮，深色 → 比 surface 略深。
        let card = Color::rgb(
            tokens.surface.r * 0.97 + tokens.background.r * 0.03,
            tokens.surface.g * 0.97 + tokens.background.g * 0.03,
            tokens.surface.b * 0.97 + tokens.background.b * 0.03,
        );
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 0.0), Size::new(size.width - 16.0, size.height)),
            card,
        );
        // 语法高亮 token 渲染：把 token_color 的 (r,g,b) 与 on_surface 文本色做混合，
        // 保证 dark 主题下不至于太亮、light 主题下不至于太暗。
        let base = tokens.on_background;
        let mix = |c: (f32, f32, f32)| {
            Color::rgb(
                c.0 * 0.85 + base.r * 0.15,
                c.1 * 0.85 + base.g * 0.15,
                c.2 * 0.85 + base.b * 0.15,
            )
        };
        let code_tokens = match self.lang.as_str() {
            "yaml" => highlight_yaml(&self.source),
            "rust" => highlight_rust(&self.source),
            _ => vec![(&self.source as &str, "default")],
        };
        let mut x = 16.0_f32;
        let mut y = 14.0_f32;
        for (text, kind) in &code_tokens {
            let color = mix(token_color(kind));
            for ch in text.chars() {
                if ch == '\n' {
                    x = 16.0;
                    y += 16.0;
                } else {
                    ctx.recorder.draw_text(&ch.to_string(), Point::new(x, y), 12.0, color);
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
    host.register("HeaderTitle", |_spec| {
        Box::new(HeaderTitle {
            text: String::new(),
            theme: ThemeKind::Light,
        })
    });
    host.register("Spacer", |spec| {
        let axis = str_prop(spec, "axis").unwrap_or_else(|| "horizontal".into());
        Box::new(Spacer { axis })
    });
    host.register("HeaderButton", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let action = str_prop(spec, "action")
            .map(|a| ActionId::new(&a))
            .unwrap_or_else(|| ActionId::new("noop"));
        Box::new(HeaderButton {
            label,
            action,
            pressed: false,
            theme: ThemeKind::Light,
        })
    });
    host.register("NavItem", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let page_id = str_prop(spec, "page_id").unwrap_or_default();
        let selected = bool_prop(spec, "selected");
        Box::new(NavItem {
            label,
            page_id,
            selected,
            pressed: false,
            theme: ThemeKind::Light,
        })
    });
    host.register("GroupHeader", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let collapsed = bool_prop(spec, "collapsed");
        Box::new(GroupHeader {
            label,
            collapsed,
            theme: ThemeKind::Light,
        })
    });
    host.register("NavSearch", |spec| {
        let query = str_prop(spec, "query").unwrap_or_default();
        Box::new(NavSearch {
            query,
            theme: ThemeKind::Light,
            locale: Locale::En,
        })
    });
    host.register("DemoTitle", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        let desc = str_prop(spec, "desc").unwrap_or_default();
        Box::new(DemoTitle {
            text,
            desc,
            theme: ThemeKind::Light,
        })
    });
    host.register("DemoPreview", |spec| {
        let page_id = str_prop(spec, "page_id").unwrap_or_default();
        Box::new(DemoPreview {
            page_id,
            theme: ThemeKind::Light,
        })
    });
    host.register("SourceLabel", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        Box::new(SourceLabel {
            text,
            theme: ThemeKind::Light,
        })
    });
    host.register("SourceCode", |spec| {
        let source = str_prop(spec, "source").unwrap_or_default();
        let lang = str_prop(spec, "lang").unwrap_or_else(|| "yaml".into());
        Box::new(SourceCode {
            source,
            lang,
            theme: ThemeKind::Light,
        })
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
        register_gallery_factories(driver.host_mut());
        driver.begin();
        assert!(
            !driver.host().scene().entries.is_empty(),
            "Gallery begin 产出非空 scene"
        );
        assert_eq!(driver.pump_frame(), zero_ui_adapter_winit::FrameOutcome::Idle);
    }

    #[test]
    fn layout_places_header_and_body_in_viewport() {
        // 回归测试：Spacer/NavItem 在 cross 维度不抢空间，Header 不吃满全屏高度。
        // 之前 Spacer.layout 返回 c.max_width × c.max_height 把 Header 拉到 1024 高，
        // Body 被挤到 y=1024（视口外）→ demo_area / sidebar 全部画在屏幕外 → 窗口空白。
        let mut app = setup_gallery();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::tablet());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let root = driver
            .host()
            .rect_of(&WidgetId::new("gallery_root"))
            .expect("root rect");
        assert_eq!(root.size.width, 768.0);
        assert_eq!(root.size.height, 1024.0);

        let header = driver
            .host()
            .rect_of(&WidgetId::new("gallery_header"))
            .expect("header rect");
        assert!(
            header.size.height <= 48.0,
            "Header 高度应 ≤ 48（含 Spacer cross=0），实际 = {}",
            header.size.height
        );
        assert!(header.origin.y == 0.0, "Header 应贴顶，origin.y = {}", header.origin.y);

        let body = driver
            .host()
            .rect_of(&WidgetId::new("gallery_body"))
            .expect("body rect");
        assert!(
            body.origin.y + body.size.height <= root.size.height + 0.5,
            "Body 底部 ({}) 应不超 root 底部 ({})",
            body.origin.y + body.size.height,
            root.size.height
        );
        assert!(body.size.height > 0.0, "Body 应有非零高度");

        // Sidebar 在 body 内（左侧），demo_area 在 body 内（右侧），二者不重叠。
        let sidebar = driver.host().rect_of(&WidgetId::new("sidebar")).expect("sidebar rect");
        let demo = driver
            .host()
            .rect_of(&WidgetId::new("demo_area"))
            .expect("demo_area rect");
        assert!(
            sidebar.origin.x + sidebar.size.width <= demo.origin.x + 0.5,
            "Sidebar 右边 ({}) 应 ≤ Demo 区左边 ({})",
            sidebar.origin.x + sidebar.size.width,
            demo.origin.x
        );
        assert!(sidebar.size.width > 0.0, "Sidebar 应有非零宽度");
        assert!(demo.size.width > 0.0, "Demo 区应有非零宽度");
    }

    #[test]
    fn scene_contains_visible_fill_primitives() {
        // 回归测试：scene 必须包含至少 1 个非零尺寸的 FillRect，否则 GPU 端因 fill ∩ clip = ∅
        // 全部跳过 → 帧空白。Header 标题文本也要存在（之前 text prop 被错写成 "locale" → 空串）。
        use zero_ui_core::geometry::Rect;
        use zero_ui_render::render_node::RenderPrimitive;
        let mut app = setup_gallery();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::tablet());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let scene = driver.host().scene();
        let visible_fills = scene
            .entries
            .iter()
            .filter(|e| {
                matches!(&e.primitive, RenderPrimitive::FillRect { rect, .. }
                if rect.size.width > 0.0 && rect.size.height > 0.0)
                    && !matches!(e.clip, Some(Rect::ZERO))
            })
            .count();
        assert!(
            visible_fills > 0,
            "至少要有 1 个非零尺寸的 FillRect，实际 {}",
            visible_fills
        );
        let has_title = scene
            .entries
            .iter()
            .any(|e| matches!(&e.primitive, RenderPrimitive::Text { text, .. } if text == "Component Gallery"));
        assert!(has_title, "scene 应包含 HeaderTitle 文本");
    }

    #[test]
    fn nav_click_switches_page() {
        let mut app = setup_gallery();
        assert_eq!(app.current_page, "button");
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
    fn locale_toggle_propagates_to_header_title() {
        // 回归：HeaderTitle 的 text prop 必须按 locale 切换。
        // 历史 bug：text prop 被错写成 "locale" 导致文本始终为空。
        let mut app = GalleryApp::new();
        app.dispatch(&ActionId::new("gallery.locale.toggle"), None);
        let spec = app.root_spec();
        let header = spec
            .children
            .iter()
            .find(|c| c.id.as_ref().map(|i| i.0.as_str()) == Some("gallery_header"))
            .expect("header 存在");
        let title = header
            .children
            .iter()
            .find(|c| c.component.0 == "HeaderTitle")
            .expect("HeaderTitle 存在");
        match title.props.get("text") {
            Some(Value::Text(t)) => assert_eq!(t, "组件画廊", "locale=Zh 时 HeaderTitle 应为中文"),
            other => panic!("HeaderTitle text prop 应为 Value::Text，实际 {other:?}"),
        }
    }

    #[test]
    fn locale_toggle_propagates_to_group_header_label() {
        // 回归：GroupHeader 的 label 应通过 GroupId::name_for(locale) 选文案，
        // 而非硬编码 name_en()。
        let mut app = GalleryApp::new();
        app.dispatch(&ActionId::new("gallery.locale.toggle"), None);
        let spec = app.root_spec();
        let body = spec
            .children
            .iter()
            .find(|c| c.id.as_ref().map(|i| i.0.as_str()) == Some("gallery_body"))
            .expect("body 存在");
        let sidebar = body
            .children
            .iter()
            .find(|c| c.id.as_ref().map(|i| i.0.as_str()) == Some("sidebar"))
            .expect("sidebar 存在");
        let group_labels: Vec<String> = sidebar
            .children
            .iter()
            .filter(|c| c.component.0 == "GroupHeader")
            .filter_map(|c| match c.props.get("label") {
                Some(Value::Text(t)) => Some(t.clone()),
                _ => None,
            })
            .collect();
        assert!(
            group_labels.iter().any(|l| l == "控件"),
            "locale=Zh 时分组标签应含中文「控件」，实际 {:?}",
            group_labels
        );
    }

    #[test]
    fn nav_search_renders_localized_placeholder() {
        // 验证 NavSearch widget 收到 locale prop 后按语言选占位符。
        let mut app = GalleryApp::new();
        app.dispatch(&ActionId::new("gallery.locale.toggle"), None);
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::tablet());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        // 在 scene 中找一个文本是 "搜索..." 的 entry
        use zero_ui_render::render_node::RenderPrimitive;
        let scene = driver.host().scene();
        let has_zh_placeholder = scene
            .entries
            .iter()
            .any(|e| matches!(&e.primitive, RenderPrimitive::Text { text, .. } if text == "搜索..."));
        assert!(has_zh_placeholder, "locale=Zh 时 NavSearch 应渲染中文占位符「搜索...」");
    }

    #[test]
    fn theme_toggle_switches_kind() {
        let mut app = GalleryApp::new();
        assert_eq!(app.theme, ThemeKind::Light, "默认浅色");
        let action = ActionId::new("gallery.theme.toggle");
        assert_eq!(app.dispatch(&action, None), ActionResult::Handled);
        assert_eq!(app.theme, ThemeKind::Dark);
        app.dispatch(&action, None);
        assert_eq!(app.theme, ThemeKind::Light);
    }

    #[test]
    fn root_spec_propagates_theme_to_children() {
        let mut app = setup_gallery();
        app.theme = ThemeKind::Dark;
        let spec = app.root_spec();
        assert_eq!(
            spec.props.get("theme"),
            Some(&Value::Text("dark".into())),
            "root spec 应携带 theme prop",
        );
        // Header 中的主题按钮也应有 theme prop。
        let header = &spec.children[0];
        let theme_btn = header
            .children
            .iter()
            .find(|c| c.id.as_ref().map(|i| i.0.as_str()) == Some("theme_btn"))
            .expect("theme_btn 存在");
        assert_eq!(theme_btn.props.get("theme"), Some(&Value::Text("dark".into())));
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
            theme: ThemeKind::Light,
        };
        let ev = UiEvent::Pointer {
            phase: PointerPhase::Released,
            button: Some(zero_ui_core::event::PointerButton::Primary),
            position: Point::new(10.0, 10.0),
            modifiers: zero_ui_core::event::Modifiers::NONE,
            pointer_id: 0,
        };
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ctx = EventCtx {
            invalidation: &mut flags,
        };
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

    /// RFC §4：每个 Demo 页必须有非空的 title/description/source_dsl/source_rust。
    #[test]
    fn all_pages_have_non_empty_metadata() {
        for page in super::super::pages::ALL_PAGES {
            assert!(!page.title.is_empty(), "page {} title 为空", page.id);
            assert!(!page.title_zh.is_empty(), "page {} title_zh 为空", page.id);
            assert!(!page.description.is_empty(), "page {} description 为空", page.id);
            assert!(!page.description_zh.is_empty(), "page {} description_zh 为空", page.id);
            assert!(!page.source_dsl.trim().is_empty(), "page {} source_dsl 为空", page.id);
            assert!(!page.source_rust.trim().is_empty(), "page {} source_rust 为空", page.id);
        }
    }
}
