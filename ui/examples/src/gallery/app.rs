use std::collections::HashSet;

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{KeyAction, PointerPhase, UiEvent};
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
        // 垂直滚动容器（DC-16）：内容超出视口时 host 按 scroll_offset 偏移子节点 y，
        // 并通过 clip 链裁掉视口外的部分。Wheel 事件命中 sidebar 时累加 scroll_offset。
        col.props.insert("scroll", Value::Text("vertical".into()));

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
        // 按内容估宽：每字符 ~10 + 左右内边距各 12；最小 64。
        let content_w = (self.label.chars().count() as f32 * 10.0 + 24.0).max(64.0);
        Size::new(
            content_w.clamp(c.min_width, c.max_width),
            32.0_f32.clamp(c.min_height, c.max_height),
        )
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(64.0, 32.0));
        let bg = if self.pressed { tokens.primary } else { tokens.surface };
        ctx.recorder.fill_rect(Rect::from_origin_size(Point::ZERO, size), bg);
        // 边框：让按钮在 header 上有可见边界（避免与背景同色看不出来）。
        let border = Color::rgb(
            tokens.on_background.r * 0.3 + tokens.surface.r * 0.7,
            tokens.on_background.g * 0.3 + tokens.surface.g * 0.7,
            tokens.on_background.b * 0.3 + tokens.surface.b * 0.7,
        );
        ctx.recorder
            .stroke_rect(Rect::from_origin_size(Point::ZERO, size), border, 1.0);
        let on_bg = if self.pressed {
            tokens.on_primary
        } else {
            tokens.on_surface
        };
        ctx.recorder.draw_text(&self.label, Point::new(12.0, 22.0), 14.0, on_bg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
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
    /// 序列化进 action payload 的 group 标识（与 dispatch 端的 `{:?}` 解析对应）。
    group: String,
    collapsed: bool,
    pressed: bool,
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
        if let Some(Value::Text(g)) = props.get("group") {
            self.group = g.clone();
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
                    ActionId::new("gallery.group.toggle"),
                    ActionPayload::Text(self.group.clone()),
                )
            }
            _ => EventResult::Ignored,
        }
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
        let fg = if self.pressed {
            tokens.primary
        } else {
            Color::rgb(
                tokens.on_background.r * 0.6 + tokens.surface.r * 0.4,
                tokens.on_background.g * 0.6 + tokens.surface.g * 0.4,
                tokens.on_background.b * 0.6 + tokens.surface.b * 0.4,
            )
        };
        ctx.recorder.draw_text(&display, Point::new(8.0, 18.0), 12.0, fg);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
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
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        // 仅处理 Key Pressed（host 已截获 Tab 用于焦点遍历）。
        let UiEvent::Key {
            code,
            action: KeyAction::Pressed,
            text,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
        match code.0.as_str() {
            "Backspace" => {
                let mut q = self.query.clone();
                q.pop();
                EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(q))
            }
            "Enter" | "Escape" => {
                // Enter/Escape 清空搜索（轻量 UX：用户敲回车表示「就是这些」时清场）。
                if self.query.is_empty() {
                    EventResult::Ignored
                } else {
                    EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(String::new()))
                }
            }
            _ => match text {
                Some(ch) => {
                    // 忽略控制字符（空格除外，允许搜索含空格的查询）。
                    if ch.chars().any(|c| c.is_control()) {
                        return EventResult::Ignored;
                    }
                    let q = format!("{}{}", self.query, ch);
                    EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(q))
                }
                None => EventResult::Ignored,
            },
        }
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
    fn focusable(&self) -> bool {
        true
    }
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
    /// 内部交互状态：随 page 不同语义不同（如 toggle 的 on/off、button 的 pressed index）。
    /// 用 u64 位掩码：低 8 位用于 toggle on/off 标志位 0..7。
    state: u64,
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
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        // 仅响应在 toggle 预览区内的点击：翻转第 i 位。
        let UiEvent::Pointer {
            phase: PointerPhase::Released,
            position,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
        if self.page_id == "toggle" {
            // 3 个 toggle，y 区间分别为 [20,60] / [60,100] / [100,140]。
            let x = position.x;
            let y = position.y;
            if (40.0..=200.0).contains(&x) {
                let y_ranges: [(usize, f32, f32); 3] = [(0, 20.0, 60.0), (1, 60.0, 100.0), (2, 100.0, 140.0)];
                for (i, y_lo, y_hi) in y_ranges {
                    if y >= y_lo && y < y_hi {
                        // 第 2 个 (i=2) 视为 disabled 不可点。
                        if i == 2 {
                            return EventResult::Consumed;
                        }
                        self.state ^= 1 << i;
                        return EventResult::Consumed;
                    }
                }
            }
            EventResult::Ignored
        } else {
            let _ = position;
            EventResult::Ignored
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 不同 demo 高度不同：交互/多行示例需要更多垂直空间。
        let h: f32 = match self.page_id.as_str() {
            "toggle" | "button" | "theme_demo" | "list_view" | "menu" | "data_list" | "form_demo" | "gesture_demo"
            | "animation_demo" | "collection_demo" | "dialog_scaffold" | "nav_demo" | "command_palette" | "tab_bar"
            | "popover" | "popup" | "toolbar" => 200.0,
            "badge" | "progress" | "text_input" | "tabs" | "tooltip" | "icon_button" | "search_field"
            | "status_bubble" | "i18n_demo" | "dsl_demo" => 140.0,
            _ => 120.0,
        };
        Size::new(c.max_width, h.clamp(c.min_height, c.max_height))
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

        match self.page_id.as_str() {
            "button" => self.paint_button_preview(ctx, &tokens),
            "toggle" => self.paint_toggle_preview(ctx, &tokens),
            "theme_demo" => self.paint_theme_preview(ctx, &tokens, size),
            "icon_button" => self.paint_icon_button_preview(ctx, &tokens),
            "badge" => self.paint_badge_preview(ctx, &tokens),
            "progress" => self.paint_progress_preview(ctx, &tokens),
            "text_input" => self.paint_text_input_preview(ctx, &tokens),
            "tabs" => self.paint_tabs_preview(ctx, &tokens),
            "tooltip" => self.paint_tooltip_preview(ctx, &tokens),
            "list_view" => self.paint_list_view_preview(ctx, &tokens),
            "menu" => self.paint_menu_preview(ctx, &tokens),
            "search_field" => self.paint_search_field_preview(ctx, &tokens),
            "status_bubble" => self.paint_status_bubble_preview(ctx, &tokens),
            "collection_demo" => self.paint_collection_preview(ctx, &tokens),
            "i18n_demo" => self.paint_i18n_preview(ctx, &tokens),
            "dsl_demo" => self.paint_dsl_preview(ctx, &tokens),
            "data_list" => self.paint_data_list_preview(ctx, &tokens),
            "command_palette" => self.paint_command_palette_preview(ctx, &tokens),
            "tab_bar" => self.paint_tab_bar_preview(ctx, &tokens),
            "dialog_scaffold" => self.paint_dialog_preview(ctx, &tokens),
            "toolbar" => self.paint_toolbar_preview(ctx, &tokens),
            "popover" => self.paint_popover_preview(ctx, &tokens),
            "popup" => self.paint_popup_preview(ctx, &tokens),
            "form_demo" => self.paint_form_preview(ctx, &tokens),
            "gesture_demo" => self.paint_gesture_preview(ctx, &tokens),
            "animation_demo" => self.paint_animation_preview(ctx, &tokens),
            "nav_demo" => self.paint_nav_preview(ctx, &tokens),
            other => {
                // 占位：未实现真实交互预览的页面继续显示 "{page} preview" 文案。
                let label = format!("{} preview", other.replace('_', " "));
                ctx.recorder
                    .draw_text(&label, Point::new(20.0, 30.0), 14.0, tokens.on_surface);
            }
        }
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

impl DemoPreview {
    fn paint_button_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 三个示例按钮：default / pressed / disabled，水平排列。
        let labels = ["Default", "Pressed", "Disabled"];
        let colors = [
            tokens.surface,
            tokens.primary,
            Color::rgb(
                tokens.surface.r * 0.7 + tokens.background.r * 0.3,
                tokens.surface.g * 0.7 + tokens.background.g * 0.3,
                tokens.surface.b * 0.7 + tokens.background.b * 0.3,
            ),
        ];
        let fg_colors = [tokens.on_surface, tokens.on_primary, tokens.on_surface];
        for (i, label) in labels.iter().enumerate() {
            let x = 24.0 + i as f32 * 130.0;
            let rect = Rect::from_origin_size(Point::new(x, 40.0), Size::new(110.0, 36.0));
            ctx.recorder.fill_rect(rect, colors[i]);
            ctx.recorder.stroke_rect(rect, border_of(tokens, colors[i]), 1.0);
            ctx.recorder
                .draw_text(label, Point::new(x + 12.0, 64.0), 14.0, fg_colors[i]);
        }
        ctx.recorder.draw_text(
            "Click → emit Action (state held by parent app)",
            Point::new(24.0, 110.0),
            12.0,
            tokens.on_surface,
        );
    }

    fn paint_toggle_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 三个 toggle：state bit 0/1 = on/off；bit 2 固定 disabled off。
        let labels = ["On/Off (interactive)", "On/Off (interactive)", "Disabled"];
        for (i, label) in labels.iter().enumerate() {
            let y = 28.0 + i as f32 * 40.0;
            // 标签
            ctx.recorder
                .draw_text(label, Point::new(24.0, y + 18.0), 13.0, tokens.on_surface);
            // Toggle 轨道
            let track_x = 200.0;
            let is_on = i < 2 && (self.state & (1 << i)) != 0;
            let track_color = if i == 2 {
                Color::rgb(
                    tokens.surface.r * 0.6 + tokens.background.r * 0.4,
                    tokens.surface.g * 0.6 + tokens.background.g * 0.4,
                    tokens.surface.b * 0.6 + tokens.background.b * 0.4,
                )
            } else if is_on {
                tokens.primary
            } else {
                Color::rgb(
                    tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
                    tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
                    tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
                )
            };
            let track_rect = Rect::from_origin_size(Point::new(track_x, y), Size::new(48.0, 24.0));
            ctx.recorder.fill_rect(track_rect, track_color);
            // Thumb：on 时靠右，off 时靠左
            let thumb_x = if is_on { track_x + 26.0 } else { track_x + 2.0 };
            let thumb_rect = Rect::from_origin_size(Point::new(thumb_x, y + 2.0), Size::new(20.0, 20.0));
            ctx.recorder.fill_rect(thumb_rect, tokens.background);
            ctx.recorder.stroke_rect(thumb_rect, tokens.on_background, 1.0);
        }
    }

    fn paint_theme_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens, size: Size) {
        // 列出几个关键 semantic token 的色板，每行一个：色块 + 名字。
        let entries: &[(&str, zero_ui_core::theme::Color)] = &[
            ("background", tokens.background),
            ("surface", tokens.surface),
            ("primary", tokens.primary),
            ("on_primary", tokens.on_primary),
            ("on_background", tokens.on_background),
            ("error", tokens.error),
        ];
        for (i, (name, color)) in entries.iter().enumerate() {
            let y = 20.0 + i as f32 * 22.0;
            let swatch = Rect::from_origin_size(Point::new(24.0, y), Size::new(32.0, 16.0));
            ctx.recorder.fill_rect(swatch, *color);
            ctx.recorder.stroke_rect(
                swatch,
                Color::rgb(
                    tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
                    tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
                    tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
                ),
                1.0,
            );
            ctx.recorder
                .draw_text(name, Point::new(70.0, y + 14.0), 12.0, tokens.on_background);
        }
        let _ = size;
    }

    fn paint_icon_button_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 4 个示例图标按钮：用文字符号代替真实图标资源。
        let icons = ["◀", "▶", "⟳", "✕"];
        for (i, icon) in icons.iter().enumerate() {
            let x = 24.0 + i as f32 * 70.0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(56.0, 40.0));
            ctx.recorder.fill_rect(rect, tokens.surface);
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder
                .draw_text(icon, Point::new(x + 18.0, 56.0), 18.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Icon-only buttons; emit action on click",
            Point::new(24.0, 100.0),
            12.0,
            tokens.on_surface,
        );
    }

    fn paint_badge_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 两个带角标的图标：count=3 / count=99+。
        let samples = ["3", "99+"];
        for (i, count) in samples.iter().enumerate() {
            let x = 32.0 + i as f32 * 130.0;
            // 底层"图标"方块
            let icon_rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(56.0, 56.0));
            ctx.recorder.fill_rect(icon_rect, tokens.surface);
            ctx.recorder
                .stroke_rect(icon_rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder
                .draw_text("▣", Point::new(x + 18.0, 66.0), 22.0, tokens.on_surface);
            // 角标圆（用 rect 近似）
            let badge_rect = Rect::from_origin_size(Point::new(x + 40.0, 22.0), Size::new(28.0, 20.0));
            ctx.recorder.fill_rect(badge_rect, tokens.error);
            ctx.recorder
                .draw_text(count, Point::new(x + 45.0, 36.0), 12.0, tokens.on_primary);
        }
        ctx.recorder.draw_text(
            "Count badge clamped to max (default 99)",
            Point::new(24.0, 110.0),
            12.0,
            tokens.on_surface,
        );
    }

    fn paint_progress_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 3 个进度条：determinate 0.3 / 0.7 / indeterminate。
        let fracs = [0.3_f32, 0.7_f32];
        for (i, frac) in fracs.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let track = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 12.0));
            ctx.recorder.fill_rect(track, tokens.background);
            let fill_w = track.size.width * frac;
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::new(24.0, y), Size::new(fill_w, 12.0)),
                tokens.primary,
            );
            ctx.recorder
                .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
        }
        // indeterminate：动画条（静态位置占位）。
        let y = 30.0 + 2.0 * 32.0;
        let track = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 12.0));
        ctx.recorder.fill_rect(track, tokens.background);
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(60.0, y), Size::new(120.0, 12.0)),
            tokens.primary,
        );
        ctx.recorder
            .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
        let labels = ["Determinate 30%", "Determinate 70%", "Indeterminate"];
        for (i, label) in labels.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            ctx.recorder
                .draw_text(label, Point::new(24.0, y + 28.0), 11.0, tokens.on_background);
        }
    }

    fn paint_text_input_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 两个输入框：placeholder / filled。
        let placeholder_rect = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(placeholder_rect, tokens.background);
        ctx.recorder
            .stroke_rect(placeholder_rect, border_of(tokens, tokens.background), 1.0);
        let ph_color = Color::rgb(
            tokens.on_background.r * 0.4 + tokens.background.r * 0.6,
            tokens.on_background.g * 0.4 + tokens.background.g * 0.6,
            tokens.on_background.b * 0.4 + tokens.background.b * 0.6,
        );
        ctx.recorder
            .draw_text("Placeholder...", Point::new(36.0, 54.0), 14.0, ph_color);

        let filled_rect = Rect::from_origin_size(Point::new(24.0, 76.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(filled_rect, tokens.background);
        ctx.recorder.stroke_rect(filled_rect, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("Hello", Point::new(36.0, 100.0), 14.0, tokens.on_background);
        // caret 近似：在文字末尾画细线。
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(70.0, 80.0), Size::new(1.5, 28.0)),
            tokens.primary,
        );
    }

    fn paint_tabs_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 3 个 tab：第一个 selected。
        let tabs = ["General", "Privacy", "Security"];
        for (i, label) in tabs.iter().enumerate() {
            let x = 24.0 + i as f32 * 110.0;
            let is_selected = i == 0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(100.0, 36.0));
            ctx.recorder
                .fill_rect(rect, if is_selected { tokens.surface } else { tokens.background });
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            if is_selected {
                // 底部高亮线
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(x, 64.0), Size::new(100.0, 2.0)),
                    tokens.primary,
                );
            }
            ctx.recorder.draw_text(
                label,
                Point::new(x + 12.0, 54.0),
                13.0,
                if is_selected {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
        }
        // 内容区
        let content = Rect::from_origin_size(Point::new(24.0, 70.0), Size::new(336.0, 60.0));
        ctx.recorder.fill_rect(content, tokens.surface);
        ctx.recorder
            .stroke_rect(content, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Selected tab content", Point::new(36.0, 100.0), 13.0, tokens.on_surface);
    }

    fn paint_tooltip_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 一个按钮 + 悬浮 tooltip 气泡。
        let btn = Rect::from_origin_size(Point::new(60.0, 50.0), Size::new(80.0, 36.0));
        ctx.recorder.fill_rect(btn, tokens.surface);
        ctx.recorder.stroke_rect(btn, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Hover me", Point::new(72.0, 74.0), 13.0, tokens.on_surface);
        // tooltip
        let tip = Rect::from_origin_size(Point::new(70.0, 8.0), Size::new(110.0, 28.0));
        ctx.recorder.fill_rect(tip, tokens.on_background);
        ctx.recorder
            .draw_text("Helpful hint", Point::new(80.0, 26.0), 12.0, tokens.background);
        ctx.recorder.draw_text(
            "Tooltip anchored above target on hover (delay 300ms)",
            Point::new(24.0, 110.0),
            11.0,
            tokens.on_background,
        );
    }

    fn paint_list_view_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 5 行列表，第 2 行 selected。
        for i in 0..5 {
            let y = 20.0 + i as f32 * 32.0;
            let row = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 30.0));
            let is_selected = i == 1;
            if is_selected {
                let washed = Color::rgb(
                    tokens.primary.r * 0.3 + tokens.surface.r * 0.7,
                    tokens.primary.g * 0.3 + tokens.surface.g * 0.7,
                    tokens.primary.b * 0.3 + tokens.surface.b * 0.7,
                );
                ctx.recorder.fill_rect(row, washed);
            }
            ctx.recorder.stroke_rect(row, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                &format!("Item {}", i + 1),
                Point::new(36.0, y + 20.0),
                13.0,
                if is_selected {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
        }
    }

    fn paint_menu_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 下拉菜单：5 项，第 1 项 hover。
        let items = [
            "Open...  ⌘O",
            "Save      ⌘S",
            "Save As...",
            "──────────",
            "Exit      ⌘Q",
        ];
        let menu_rect = Rect::from_origin_size(Point::new(40.0, 20.0), Size::new(220.0, 160.0));
        ctx.recorder.fill_rect(menu_rect, tokens.surface);
        ctx.recorder
            .stroke_rect(menu_rect, border_of(tokens, tokens.surface), 1.0);
        for (i, item) in items.iter().enumerate() {
            let y = 20.0 + 8.0 + i as f32 * 30.0;
            if i == 0 {
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(40.0, y - 4.0), Size::new(220.0, 28.0)),
                    tokens.primary,
                );
                ctx.recorder
                    .draw_text(item, Point::new(56.0, y + 16.0), 13.0, tokens.on_primary);
            } else {
                ctx.recorder
                    .draw_text(item, Point::new(56.0, y + 16.0), 13.0, tokens.on_background);
            }
        }
    }

    fn paint_search_field_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        let field = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(field, tokens.background);
        ctx.recorder
            .stroke_rect(field, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("🔍", Point::new(36.0, 54.0), 14.0, tokens.on_background);
        ctx.recorder
            .draw_text("compo", Point::new(64.0, 54.0), 14.0, tokens.on_background);
        // suggestion 下拉
        let sugg = Rect::from_origin_size(Point::new(24.0, 70.0), Size::new(360.0, 60.0));
        ctx.recorder.fill_rect(sugg, tokens.surface);
        ctx.recorder.stroke_rect(sugg, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("component gallery", Point::new(36.0, 90.0), 12.0, tokens.on_surface);
        ctx.recorder
            .draw_text("composer pattern", Point::new(36.0, 110.0), 12.0, tokens.on_surface);
    }

    fn paint_status_bubble_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 3 个 bubble：success / warning / error。
        let samples = [
            ("✓ Saved", tokens.primary, tokens.on_primary),
            ("! Pending", Color::rgb(0.9, 0.7, 0.2), Color::rgb(0.0, 0.0, 0.0)),
            ("✗ Failed", tokens.error, tokens.on_primary),
        ];
        for (i, (text, bg, fg)) in samples.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let bubble = Rect::from_origin_size(Point::new(24.0, y), Size::new(200.0, 24.0));
            ctx.recorder.fill_rect(bubble, *bg);
            ctx.recorder.draw_text(text, Point::new(36.0, y + 16.0), 12.0, *fg);
        }
    }

    fn paint_collection_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 4×3 网格，每个 cell 一个色块。
        for row in 0..3 {
            for col in 0..4 {
                let x = 24.0 + col as f32 * 92.0;
                let y = 20.0 + row as f32 * 56.0;
                let cell = Rect::from_origin_size(Point::new(x, y), Size::new(84.0, 48.0));
                let is_selected = row == 1 && col == 2;
                ctx.recorder
                    .fill_rect(cell, if is_selected { tokens.primary } else { tokens.surface });
                ctx.recorder.stroke_rect(cell, border_of(tokens, tokens.surface), 1.0);
                ctx.recorder.draw_text(
                    &format!("{row}-{col}"),
                    Point::new(x + 28.0, y + 28.0),
                    11.0,
                    if is_selected {
                        tokens.on_primary
                    } else {
                        tokens.on_background
                    },
                );
            }
        }
    }

    fn paint_i18n_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 多语言样例：左列语言名，右列样例。
        let rows = [
            ("English", "Hello, world!"),
            ("中文", "你好，世界！"),
            ("RTL", "مرحبا بالعالم"),
        ];
        for (i, (lang, sample)) in rows.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            ctx.recorder
                .draw_text(lang, Point::new(24.0, y + 20.0), 13.0, tokens.on_background);
            ctx.recorder
                .draw_text(sample, Point::new(160.0, y + 20.0), 13.0, tokens.on_surface);
        }
    }

    fn paint_dsl_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 左侧 YAML 片段，右侧 Rust 片段。
        let left = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(170.0, 110.0));
        let right = Rect::from_origin_size(Point::new(210.0, 20.0), Size::new(170.0, 110.0));
        ctx.recorder.fill_rect(left, tokens.background);
        ctx.recorder.fill_rect(right, tokens.background);
        ctx.recorder
            .stroke_rect(left, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .stroke_rect(right, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("Row:", Point::new(32.0, 38.0), 11.0, tokens.primary);
        ctx.recorder
            .draw_text("  - Text: Hi", Point::new(32.0, 56.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("  - Spacer", Point::new(32.0, 74.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("Row::new()", Point::new(218.0, 38.0), 11.0, tokens.primary);
        ctx.recorder
            .draw_text("  .child(Text)", Point::new(218.0, 56.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("  .child(Spacer)", Point::new(218.0, 74.0), 11.0, tokens.on_background);
    }

    fn paint_data_list_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 5 行 list，其中第 1 行 loading、第 4 行 error。
        for i in 0..5 {
            let y = 20.0 + i as f32 * 32.0;
            let row = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 30.0));
            ctx.recorder.fill_rect(row, tokens.background);
            ctx.recorder.stroke_rect(row, border_of(tokens, tokens.background), 1.0);
            match i {
                0 => {
                    // loading：横线占位
                    ctx.recorder.fill_rect(
                        Rect::from_origin_size(Point::new(36.0, y + 13.0), Size::new(60.0, 6.0)),
                        tokens.surface,
                    );
                }
                3 => {
                    ctx.recorder
                        .draw_text("⚠ Failed to load", Point::new(36.0, y + 20.0), 12.0, tokens.error);
                }
                _ => {
                    ctx.recorder.draw_text(
                        &format!("Row {}", i),
                        Point::new(36.0, y + 20.0),
                        12.0,
                        tokens.on_background,
                    );
                }
            }
        }
    }

    fn paint_command_palette_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 顶部输入框 + 命令列表。
        let input = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(360.0, 32.0));
        ctx.recorder.fill_rect(input, tokens.background);
        ctx.recorder.stroke_rect(input, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("> opa", Point::new(36.0, 42.0), 13.0, tokens.on_background);
        let list = Rect::from_origin_size(Point::new(24.0, 56.0), Size::new(360.0, 120.0));
        ctx.recorder.fill_rect(list, tokens.surface);
        ctx.recorder.stroke_rect(list, border_of(tokens, tokens.surface), 1.0);
        let cmds = ["file.open  Open File", "file.save  Save", "go.back  Go Back"];
        for (i, c) in cmds.iter().enumerate() {
            let y = 56.0 + 8.0 + i as f32 * 30.0;
            if i == 0 {
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(24.0, y - 4.0), Size::new(360.0, 28.0)),
                    tokens.primary,
                );
                ctx.recorder
                    .draw_text(c, Point::new(36.0, y + 16.0), 12.0, tokens.on_primary);
            } else {
                ctx.recorder
                    .draw_text(c, Point::new(36.0, y + 16.0), 12.0, tokens.on_background);
            }
        }
    }

    fn paint_tab_bar_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 3 个标签页，第一个 selected + 关闭按钮。
        let tabs = ["Home", "Docs", "About"];
        for (i, t) in tabs.iter().enumerate() {
            let x = 24.0 + i as f32 * 120.0;
            let is_sel = i == 0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(110.0, 32.0));
            ctx.recorder
                .fill_rect(rect, if is_sel { tokens.surface } else { tokens.background });
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                t,
                Point::new(x + 12.0, y_text_center(30.0, 32.0)),
                13.0,
                if is_sel {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
            // 关闭 X
            ctx.recorder
                .draw_text("×", Point::new(x + 88.0, 52.0), 14.0, tokens.on_background);
        }
        ctx.recorder.draw_text(
            "TabBar: selected tab + per-tab close button + reorderable",
            Point::new(24.0, 90.0),
            11.0,
            tokens.on_background,
        );
    }

    fn paint_dialog_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 模态对话框：title + body + 两按钮。
        let dlg = Rect::from_origin_size(Point::new(60.0, 20.0), Size::new(280.0, 150.0));
        // 半透明遮罩
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(384.0, 192.0)),
            Color::rgb(0.0, 0.0, 0.0),
        );
        ctx.recorder.fill_rect(dlg, tokens.surface);
        ctx.recorder.stroke_rect(dlg, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Confirm", Point::new(76.0, 44.0), 15.0, tokens.on_surface);
        ctx.recorder
            .draw_text("Are you sure?", Point::new(76.0, 74.0), 13.0, tokens.on_background);
        // 两按钮
        let ok = Rect::from_origin_size(Point::new(76.0, 120.0), Size::new(110.0, 32.0));
        ctx.recorder.fill_rect(ok, tokens.primary);
        ctx.recorder
            .draw_text("OK", Point::new(116.0, 142.0), 13.0, tokens.on_primary);
        let cancel = Rect::from_origin_size(Point::new(200.0, 120.0), Size::new(110.0, 32.0));
        ctx.recorder.fill_rect(cancel, tokens.background);
        ctx.recorder
            .stroke_rect(cancel, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("Cancel", Point::new(228.0, 142.0), 13.0, tokens.on_background);
    }

    fn paint_toolbar_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 水平工具栏：5 个图标按钮。
        let bar = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 44.0));
        ctx.recorder.fill_rect(bar, tokens.surface);
        ctx.recorder.stroke_rect(bar, border_of(tokens, tokens.surface), 1.0);
        let icons = ["◀", "▶", "⟳", "⌂", "⋮"];
        for (i, icon) in icons.iter().enumerate() {
            let x = 40.0 + i as f32 * 64.0;
            ctx.recorder
                .draw_text(icon, Point::new(x, 58.0), 18.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Toolbar: row of IconButtons with optional overflow menu",
            Point::new(24.0, 100.0),
            11.0,
            tokens.on_background,
        );
    }

    fn paint_popover_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 锚点按钮 + 浮起 popover 含子内容。
        let anchor = Rect::from_origin_size(Point::new(60.0, 70.0), Size::new(120.0, 32.0));
        ctx.recorder.fill_rect(anchor, tokens.surface);
        ctx.recorder.stroke_rect(anchor, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Share ▾", Point::new(82.0, 92.0), 13.0, tokens.on_surface);
        let pop = Rect::from_origin_size(Point::new(60.0, 20.0), Size::new(200.0, 44.0));
        ctx.recorder.fill_rect(pop, tokens.surface);
        ctx.recorder.stroke_rect(pop, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("Copy link", Point::new(76.0, 42.0), 13.0, tokens.on_surface);
    }

    fn paint_popup_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 全屏遮罩 + 弹出面板（同 dialog，但更通用）。
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(384.0, 192.0)),
            Color::rgb(0.0, 0.0, 0.0),
        );
        let popup = Rect::from_origin_size(Point::new(40.0, 30.0), Size::new(320.0, 140.0));
        ctx.recorder.fill_rect(popup, tokens.surface);
        ctx.recorder.stroke_rect(popup, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Popup (modal)", Point::new(56.0, 54.0), 15.0, tokens.on_surface);
        ctx.recorder.draw_text(
            "Blocks underlying UI until dismissed",
            Point::new(56.0, 80.0),
            12.0,
            tokens.on_background,
        );
    }

    fn paint_form_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 两字段表单：email + password + submit。
        let labels_y = [30.0, 76.0];
        let labels = ["Email", "Password"];
        for (i, label) in labels.iter().enumerate() {
            ctx.recorder
                .draw_text(label, Point::new(24.0, labels_y[i]), 12.0, tokens.on_background);
            let field = Rect::from_origin_size(Point::new(24.0, labels_y[i] + 8.0), Size::new(360.0, 32.0));
            ctx.recorder.fill_rect(field, tokens.background);
            ctx.recorder
                .stroke_rect(field, border_of(tokens, tokens.background), 1.0);
        }
        // 字段内文字
        ctx.recorder
            .draw_text("user@example.com", Point::new(36.0, 56.0), 13.0, tokens.on_background);
        ctx.recorder
            .draw_text("••••••••", Point::new(36.0, 102.0), 13.0, tokens.on_background);
        // 提交按钮
        let submit = Rect::from_origin_size(Point::new(24.0, 130.0), Size::new(120.0, 36.0));
        ctx.recorder.fill_rect(submit, tokens.primary);
        ctx.recorder
            .draw_text("Sign in", Point::new(56.0, 154.0), 14.0, tokens.on_primary);
    }

    fn paint_gesture_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 大手势区 + 4 种手势标识。
        let pad = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(360.0, 110.0));
        ctx.recorder.fill_rect(pad, tokens.background);
        ctx.recorder.stroke_rect(pad, border_of(tokens, tokens.background), 1.0);
        let gestures = [("Tap", 60.0), ("Pan", 150.0), ("Pinch", 240.0), ("Long press", 320.0)];
        for (label, x) in gestures.iter() {
            ctx.recorder
                .draw_text(label, Point::new(*x, 70.0), 13.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Gesture arena: tap / pan / pinch / long-press recognition",
            Point::new(24.0, 160.0),
            11.0,
            tokens.on_background,
        );
    }

    fn paint_animation_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // 3 个进度条对应 3 种曲线：linear / ease / spring。
        let curves = ["Linear", "EaseOut", "Spring"];
        for (i, name) in curves.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let track = Rect::from_origin_size(Point::new(120.0, y), Size::new(240.0, 12.0));
            ctx.recorder.fill_rect(track, tokens.background);
            ctx.recorder
                .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
            let fill_w = 240.0
                * match i {
                    0 => 0.5,
                    1 => 0.7,
                    _ => 0.85,
                };
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::new(120.0, y), Size::new(fill_w, 12.0)),
                tokens.primary,
            );
            ctx.recorder
                .draw_text(name, Point::new(24.0, y + 10.0), 12.0, tokens.on_background);
        }
    }

    fn paint_nav_preview(&self, ctx: &mut PaintCtx, tokens: &SemanticTokens) {
        // stack 转场示意：3 层堆叠卡片。
        for i in 0..3 {
            let offset = i as f32 * 16.0;
            let card = Rect::from_origin_size(
                Point::new(60.0 + offset, 30.0 + offset),
                Size::new(240.0 - offset, 120.0 - offset),
            );
            ctx.recorder.fill_rect(card, tokens.surface);
            ctx.recorder.stroke_rect(card, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                &format!("Screen {}", 3 - i),
                Point::new(76.0 + offset, 56.0 + offset),
                14.0,
                tokens.on_surface,
            );
        }
        ctx.recorder.draw_text(
            "Navigation stack: push / pop / modal present",
            Point::new(24.0, 170.0),
            11.0,
            tokens.on_background,
        );
    }
}

/// 文本垂直居中近似：在 [row_y, row_y + row_h] 区间内放 13px 字体的基线 y。
fn y_text_center(row_y: f32, row_h: f32) -> f32 {
    row_y + (row_h + 13.0) * 0.5
}

fn border_of(tokens: &SemanticTokens, fill: zero_ui_core::theme::Color) -> zero_ui_core::theme::Color {
    Color::rgb(
        tokens.on_background.r * 0.25 + fill.r * 0.75,
        tokens.on_background.g * 0.25 + fill.g * 0.75,
        tokens.on_background.b * 0.25 + fill.b * 0.75,
    )
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
        // 按字符遍历，遇换行重置 x；同色段累计成字符串，整段一次 draw_text 调用——
        // 让 fontdue 内部按真实 advance 绘制字符（避免每字符硬编码 7.2px 导致
        // 窄字符间距过大、宽字符/中文重叠的问题）。
        //
        // 段间 x 推进按「字符数 × 字体平均宽度」估算：对 Noto Sans 12px 约 6.6px/字符，
        // 中文 12px/字符。误差 < 1px，肉眼不可察。
        let mut x = 16.0_f32;
        let mut y = 14.0_f32;
        let line_h = 16.0_f32;
        for (text, kind) in &code_tokens {
            let color = mix(token_color(kind));
            // 把 token 内按行切分：每行单独画（同色段不跨行）。
            let mut first_segment = true;
            for segment in text.split('\n') {
                if !first_segment {
                    // 遇到 '\n'：换行。
                    x = 16.0;
                    y += line_h;
                }
                first_segment = false;
                if segment.is_empty() {
                    continue;
                }
                ctx.recorder.draw_text(segment, Point::new(x, y), 12.0, color);
                // 推进 x：ASCII 窄字符 ~6.6，CJK ~12。按是否含 CJK 估算。
                let ascii_count = segment.chars().filter(|c| c.is_ascii()).count() as f32;
                let cjk_count = segment.chars().count() as f32 - ascii_count;
                x += ascii_count * 6.6 + cjk_count * 12.0;
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
        let group = str_prop(spec, "group").unwrap_or_default();
        let collapsed = bool_prop(spec, "collapsed");
        Box::new(GroupHeader {
            label,
            group,
            collapsed,
            pressed: false,
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
            state: 0,
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

    /// RFC §8「分组折叠」横向能力：dispatch 收到 group.toggle 翻转 collapsed_groups。
    #[test]
    fn group_toggle_collapses_and_expands() {
        let mut app = setup_gallery();
        assert!(
            !app.collapsed_groups.contains(&crate::gallery::model::GroupId::Widgets),
            "默认不折叠"
        );
        let action = ActionId::new("gallery.group.toggle");
        let payload = Some(ActionPayload::Text("Widgets".into()));
        app.dispatch(&action, payload);
        assert!(
            app.collapsed_groups.contains(&crate::gallery::model::GroupId::Widgets),
            "toggle 一次后应折叠"
        );
        app.dispatch(&action, Some(ActionPayload::Text("Widgets".into())));
        assert!(
            !app.collapsed_groups.contains(&crate::gallery::model::GroupId::Widgets),
            "toggle 二次后应展开"
        );
    }

    /// RFC §8「搜索过滤」横向能力：dispatch 收到 gallery.search 更新 query，
    /// 且 filtered_pages 按查询缩小返回集。
    #[test]
    fn search_query_filters_pages() {
        let mut app = setup_gallery();
        let total = app.filtered_pages().len();
        assert!(total > 1, "默认应有多页");

        app.dispatch(
            &ActionId::new("gallery.search"),
            Some(ActionPayload::Text("toggle".into())),
        );
        let filtered = app.filtered_pages();
        let ids: Vec<_> = filtered.iter().map(|p| p.id).collect();
        assert_eq!(filtered.len(), 1, "应只剩 toggle 相关页: {ids:?}");
        assert_eq!(filtered[0].id, "toggle");

        // 清空搜索 → 恢复全集
        app.dispatch(
            &ActionId::new("gallery.search"),
            Some(ActionPayload::Text(String::new())),
        );
        assert_eq!(app.filtered_pages().len(), total, "清空 query 恢复全量");
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
