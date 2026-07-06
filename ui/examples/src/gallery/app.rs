use std::collections::HashSet;

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget, WidgetId, WidgetSpec};
use zero_ui_runtime::{UiApp, WidgetHost};

use super::chrome::{
    DemoTitle, GroupHeader, HeaderButton, HeaderTitle, NavItem, NavSearch, Spacer, mark_paint_if_changed, sync_text,
};
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
    /// Demo 内部状态（按 page 隔离，P2-13 namespace 化）。
    /// key = page_id，value = 该 page 的 demo state。切 page 时互不干扰。
    pub demo_states: std::collections::HashMap<String, DemoState>,
}

/// 单个 page 的 demo state。
/// 不同 page 复用同一结构，按需使用各字段（语义由 page 决定）。
#[derive(Debug, Clone, Default)]
pub struct DemoState {
    /// button-like 索引（最近一次点击的按钮编号；0=未点）。
    pub pressed: u32,
    /// toggle 位掩码（最多 8 位）。
    pub toggles: u8,
    /// 文本输入当前内容。
    pub text: String,
}

impl DemoState {
    pub fn for_page<'a>(app: &'a mut GalleryApp, page: &str) -> &'a mut DemoState {
        app.demo_states.entry(page.to_string()).or_default()
    }
}

impl GalleryApp {
    pub fn new() -> GalleryApp {
        GalleryApp {
            current_page: String::from("button"),
            locale: Locale::En,
            theme: ThemeKind::Light,
            collapsed_groups: HashSet::new(),
            search_query: String::new(),
            demo_states: std::collections::HashMap::new(),
        }
    }

    /// 获取当前 page 的 demo state（按 page 隔离，避免跨 page 误染）。
    pub fn current_demo(&mut self) -> &mut DemoState {
        let page = self.current_page.clone();
        DemoState::for_page(self, &page)
    }

    /// 只读访问当前 page 的 demo state。
    pub fn current_demo_read(&self) -> DemoState {
        self.demo_states
            .get(self.current_page.as_str())
            .cloned()
            .unwrap_or_default()
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
        let mut col = WidgetSpec::new("ScrollVertical");
        col.id = Some(WidgetId::new("sidebar"));
        col.props.insert("theme", Value::Text(self.theme.as_str().into()));
        // 垂直滚动容器（DC-16）：内容超出视口时 host 按 scroll_offset 偏移子节点 y，
        // 并通过 clip 链裁掉视口外的部分。Wheel 事件命中 sidebar 时累加 scroll_offset。

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

            let mut preview = self.build_demo_preview(page);
            preview.id = Some(WidgetId::new("demo_preview"));
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

    // build_demo_preview / build_button_demo / build_toggle_demo / build_text_input_demo
    // 及所有其它 page demo builder 实现已迁移到 demo_builders.rs（P2-11/P2-12 真控件化扩展）。
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

    /// P1-6 主题单源：把 self.theme 映射为 semantic tokens，由 driver 注入 host.set_tokens。
    /// 控件 paint 直接读 PaintCtx.tokens，无需各自存 theme 字段。
    fn theme_tokens(&self) -> Option<zero_ui_core::theme::SemanticTokens> {
        Some(match self.theme {
            ThemeKind::Light => zero_ui_core::theme::SemanticTokens::light(),
            ThemeKind::Dark => zero_ui_core::theme::SemanticTokens::dark(),
        })
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
            // ── Demo 内部 actions（真控件 emit）──────────────────────────────────
            // P2-13 namespace 化：所有 demo action 写入"当前 page"的 state，
            // 切到其它 page 时不会污染。button_click.N / toggle.N 用同一个 pressed/toggles 字段。
            s if s.starts_with("gallery.demo.button_click.") => {
                if let Some(n) = s
                    .strip_prefix("gallery.demo.button_click.")
                    .and_then(|t| t.parse::<u32>().ok())
                    && n <= 4
                {
                    self.current_demo().pressed = n;
                }
                ActionResult::Handled
            }
            s if s.starts_with("gallery.demo.toggle.") => {
                if let Some(i) = s
                    .strip_prefix("gallery.demo.toggle.")
                    .and_then(|t| t.parse::<u32>().ok())
                    && i < 8
                {
                    self.current_demo().toggles ^= 1 << i;
                }
                ActionResult::Handled
            }
            "gallery.demo.text_changed" | "text_input.changed" => {
                if let Some(ActionPayload::Text(t)) = &payload {
                    self.current_demo().text = t.clone();
                }
                ActionResult::Handled
            }
            // P2-14：Button hover_action 触发，payload = "enter" / "leave"。
            "gallery.demo.hover" => {
                match payload {
                    Some(ActionPayload::Text(s)) if s == "enter" => {
                        self.current_demo().pressed = 1;
                    }
                    Some(ActionPayload::Text(s)) if s == "leave" => {
                        self.current_demo().pressed = 0;
                    }
                    _ => {}
                }
                ActionResult::Handled
            }
            _ => ActionResult::UnknownAction(action.clone()),
        }
    }
}

/// 源码标签
pub struct SourceLabel {
    text: String,
}

impl Widget for SourceLabel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let changed = sync_text(props, zero_ui_core::prop_keys::TEXT, &mut self.text);
        mark_paint_if_changed(ctx, changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(c.max_width, 24.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let fg = Color::rgb(
            tokens.on_background.r * 0.6 + tokens.background.r * 0.4,
            tokens.on_background.g * 0.6 + tokens.background.g * 0.4,
            tokens.on_background.b * 0.6 + tokens.background.b * 0.4,
        );
        ctx.recorder.draw_text(&self.text, Point::new(12.0, 16.0), 12.0, fg);
    }
}

/// 语法高亮源码展示
pub struct SourceCode {
    source: String,
    lang: String,
}

impl Widget for SourceCode {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        // source 行数决定高度 → layout；lang 仅 paint。
        let source_changed = sync_text(props, zero_ui_core::prop_keys::SOURCE, &mut self.source);
        let lang_changed = sync_text(props, zero_ui_core::prop_keys::LANG, &mut self.lang);
        super::chrome::mark_layout_if_changed(ctx, source_changed);
        mark_paint_if_changed(ctx, lang_changed);
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
        let tokens = ctx.tokens;
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
        // 段间 x 推进通过 `PaintRecorder::measure_text` 查询（DC-17）。
        // SceneRecorder 的默认实现按字符 Unicode 属性精确估算（ASCII/CJK/标点各异），
        // 比 `ascii_count * 6.6 + cjk * 12.0` 估算误差更小。
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
                x += ctx.recorder.measure_text(segment, 12.0);
            }
        }
    }
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
        })
    });
    host.register("NavSearch", |spec| {
        let query = str_prop(spec, "query").unwrap_or_default();
        Box::new(NavSearch {
            query,
            locale: Locale::En,
        })
    });
    host.register("DemoTitle", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        let desc = str_prop(spec, "desc").unwrap_or_default();
        Box::new(DemoTitle { text, desc })
    });
    host.register("SourceLabel", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        Box::new(SourceLabel { text })
    });
    host.register("ColoredBox", |_spec| Box::new(zero_ui_widgets::ColoredBox::new()));
    host.register("SourceCode", |spec| {
        let source = str_prop(spec, "source").unwrap_or_default();
        let lang = str_prop(spec, "lang").unwrap_or_else(|| "yaml".into());
        Box::new(SourceCode { source, lang })
    });

    // ── 真控件（来自 widgets crate，P2-11 真控件化）──────────────────────────
    host.register("Button", |spec| {
        let label = str_prop(spec, "label").unwrap_or_else(|| "Button".into());
        let action = str_prop(spec, "action")
            .map(|a| ActionId::new(&a))
            .unwrap_or_else(|| ActionId::new("noop"));
        let enabled = !matches!(spec.props.get("enabled"), Some(Value::Bool(false)));
        let hover_action = str_prop(spec, "hover_action").map(|a| ActionId::new(&a));
        let variant = match str_prop(spec, "variant").as_deref() {
            Some("neutral") => zero_ui_widgets::ButtonVariant::Neutral,
            Some("selected") => zero_ui_widgets::ButtonVariant::Selected,
            _ => zero_ui_widgets::ButtonVariant::Primary,
        };
        Box::new(zero_ui_widgets::Button::new(zero_ui_widgets::ButtonSpec {
            label,
            action,
            enabled,
            hover_action,
            variant,
        }))
    });
    host.register("ToggleWidget", |spec| {
        let checked = matches!(spec.props.get("checked"), Some(Value::Bool(true)));
        let enabled = !matches!(spec.props.get("enabled"), Some(Value::Bool(false)));
        let action = str_prop(spec, "action")
            .map(|a| ActionId::new(&a))
            .unwrap_or_else(|| ActionId::new("noop"));
        let mut s = zero_ui_widgets::ToggleSpec::new(checked, action.0.as_str());
        if let Some(l) = str_prop(spec, "label") {
            s = s.with_label(&l);
        }
        if !enabled {
            s = s.with_enabled(false);
        }
        Box::new(zero_ui_widgets::ToggleWidget::new(s))
    });
    host.register("TextInputWidget", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        let placeholder = str_prop(spec, "placeholder").unwrap_or_default();
        let mut w = zero_ui_widgets::TextInputWidget::new().with_placeholder(&placeholder);
        // 受控模式：把 props.text 写入内部 state（mount 后第一帧 update 也会同步）。
        w.set_text_from_props(&text);
        Box::new(w)
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
        };
        let ev = UiEvent::Pointer {
            phase: zero_ui_core::event::PointerPhase::Released,
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
