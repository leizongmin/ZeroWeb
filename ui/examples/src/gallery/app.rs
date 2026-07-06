use std::collections::HashSet;

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult};
use zero_ui_core::binding::Value;
use zero_ui_core::widget::{WidgetId, WidgetSpec};
use zero_ui_runtime::{UiApp, WidgetHost};

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
        // U-5 修复：根容器铺 background 底色，暗色主题下不透明、配色一致。
        root.props.insert("bg", Value::Text("background".into()));

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
        // U-5：侧边栏用 surface 底色（与主区背景区分）。
        col.props.insert("bg", Value::Text("surface".into()));
        // 垂直滚动容器（DC-16）：内容超出视口时 host 按 scroll_offset 偏移子节点 y，
        // 并通过 clip 链裁掉视口外的部分。Wheel 事件命中 sidebar 时累加 scroll_offset。

        // Search box area
        let mut search_box = WidgetSpec::new("NavSearch");
        search_box.id = Some(WidgetId::new("nav_search"));
        search_box
            .props
            .insert("theme", Value::Text(self.theme.as_str().into()));
        search_box.props.insert("action", Value::Text("gallery.search".into()));
        search_box
            .props
            .insert("placeholder", Value::Text(self.locale.search_placeholder().into()));
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
                    .insert("action", Value::Text("gallery.group.toggle".into()));
                group_header
                    .props
                    .insert("label", Value::Text(page.group.name_for(self.locale).into()));
                group_header.props.insert("collapsed", Value::Bool(is_collapsed));
                // P1-9 修复：用 name_en() 稳定字符串标识（替代 Debug 格式），中文环境也工作。
                group_header
                    .props
                    .insert("group", Value::Text(page.group.name_en().into()));
                col.children.push(group_header);

                if !is_collapsed {
                    for p in filtered.iter().filter(|p| p.group == page.group) {
                        let mut nav = WidgetSpec::new("NavItem");
                        nav.id = Some(WidgetId::new(&format!("nav_{}", p.id)));
                        nav.props.insert("theme", Value::Text(self.theme.as_str().into()));
                        nav.props.insert("action", Value::Text("gallery.nav.select".into()));
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
        // U-5：主预览区铺 background 底色。
        col.props.insert("bg", Value::Text("background".into()));

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

            let mut dsl_label = WidgetSpec::new("Text");
            dsl_label.id = Some(WidgetId::new("dsl_label"));
            dsl_label.props.insert("theme", Value::Text(self.theme.as_str().into()));
            dsl_label
                .props
                .insert("text", Value::Text(self.locale.dsl_label().into()));
            col.children.push(dsl_label);

            let mut dsl_src = WidgetSpec::new("CodeBlock");
            dsl_src.id = Some(WidgetId::new("dsl_source"));
            dsl_src.props.insert("theme", Value::Text(self.theme.as_str().into()));
            dsl_src.props.insert("source", Value::Text(page.source_dsl.into()));
            dsl_src.props.insert("lang", Value::Text("yaml".into()));
            col.children.push(dsl_src);

            let mut rust_label = WidgetSpec::new("Text");
            rust_label.id = Some(WidgetId::new("rust_label"));
            rust_label
                .props
                .insert("theme", Value::Text(self.theme.as_str().into()));
            rust_label
                .props
                .insert("text", Value::Text(self.locale.rust_label().into()));
            col.children.push(rust_label);

            let mut rust_src = WidgetSpec::new("CodeBlock");
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

    // ── P3-4-3 浮层视觉子树（popover/popup/dialog_scaffold）─────────────────
    // 这些独立于 demo_builders.rs，因为它们由 host.overlay_root paint（在主树之上），
    // 而非主 demo_preview 子树。每个返回带背景色块（muted）+ 内容的 Column。

    fn build_popover_overlay(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("demo_overlay_root"));
        col.props.insert("gap", Value::Float(8.0));
        // 背景色块（muted）作为卡片视觉。
        let mut bg = WidgetSpec::new("ColoredBox");
        bg.id = Some(WidgetId::new("demo_overlay_bg"));
        bg.props.insert("color", Value::Text("muted".into()));
        bg.props.insert("width", Value::Float(280.0));
        bg.props.insert("height", Value::Float(60.0));
        bg.props.insert("radius", Value::Float(8.0));
        bg.props.insert("label", Value::Text("Popover overlay".into()));
        col.children.push(bg);

        let mut content = WidgetSpec::new("Text");
        content.id = Some(WidgetId::new("demo_overlay_text"));
        content.props.insert(
            "text",
            Value::Text("Popover: floats above other content (real overlay).".into()),
        );
        col.children.push(content);

        let mut dismiss_hint = WidgetSpec::new("Button");
        dismiss_hint.id = Some(WidgetId::new("demo_overlay_dismiss"));
        dismiss_hint.props.insert("label", Value::Text("Close".into()));
        dismiss_hint.props.insert("variant", Value::Text("neutral".into()));
        dismiss_hint
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(dismiss_hint);
        col
    }

    fn build_popup_overlay(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("demo_overlay_root"));
        col.props.insert("gap", Value::Float(8.0));

        let mut title = WidgetSpec::new("Text");
        title.id = Some(WidgetId::new("demo_overlay_title"));
        title.props.insert("text", Value::Text("Popup (modal)".into()));
        col.children.push(title);

        let mut body = WidgetSpec::new("Text");
        body.id = Some(WidgetId::new("demo_overlay_body"));
        body.props.insert(
            "text",
            Value::Text("Click outside or press Escape to dismiss (modal barrier active).".into()),
        );
        col.children.push(body);

        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("demo_overlay_actions"));
        row.props.insert("gap", Value::Float(8.0));

        let mut ok = WidgetSpec::new("Button");
        ok.id = Some(WidgetId::new("demo_overlay_ok"));
        ok.props.insert("label", Value::Text("OK".into()));
        ok.props
            .insert("action", Value::Text("gallery.demo.button_click.2".into()));
        row.children.push(ok);

        let mut cancel = WidgetSpec::new("Button");
        cancel.id = Some(WidgetId::new("demo_overlay_cancel"));
        cancel.props.insert("label", Value::Text("Cancel".into()));
        cancel.props.insert("variant", Value::Text("neutral".into()));
        cancel
            .props
            .insert("action", Value::Text("gallery.demo.button_click.3".into()));
        row.children.push(cancel);
        col.children.push(row);
        col
    }

    fn build_dialog_overlay(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("demo_overlay_root"));
        col.props.insert("gap", Value::Float(8.0));

        let mut bg = WidgetSpec::new("ColoredBox");
        bg.id = Some(WidgetId::new("demo_overlay_bg"));
        bg.props.insert("color", Value::Text("muted".into()));
        bg.props.insert("width", Value::Float(320.0));
        bg.props.insert("height", Value::Float(120.0));
        bg.props.insert("radius", Value::Float(8.0));
        bg.props.insert("label", Value::Text("Dialog".into()));
        col.children.push(bg);

        let mut body = WidgetSpec::new("Text");
        body.id = Some(WidgetId::new("demo_overlay_body"));
        body.props
            .insert("text", Value::Text("Are you sure? (modal dialog)".into()));
        col.children.push(body);

        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("demo_overlay_actions"));
        row.props.insert("gap", Value::Float(8.0));

        let mut confirm = WidgetSpec::new("Button");
        confirm.id = Some(WidgetId::new("demo_overlay_confirm"));
        confirm.props.insert("label", Value::Text("Confirm".into()));
        confirm
            .props
            .insert("action", Value::Text("gallery.demo.button_click.2".into()));
        row.children.push(confirm);

        let mut cancel = WidgetSpec::new("Button");
        cancel.id = Some(WidgetId::new("demo_overlay_cancel"));
        cancel.props.insert("label", Value::Text("Cancel".into()));
        cancel.props.insert("variant", Value::Text("neutral".into()));
        cancel
            .props
            .insert("action", Value::Text("gallery.demo.button_click.3".into()));
        row.children.push(cancel);
        col.children.push(row);
        col
    }

    /// P3-5-1：tooltip 浮层视觉子树。
    ///
    /// 设计：深色背景胶囊（radius=12）+ 信息 Icon + 一行文字。
    /// 视觉上像真实 tooltip（小卡片浮在触发元素上方）。
    fn build_tooltip_overlay(&self) -> WidgetSpec {
        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("demo_overlay_root"));
        row.props.insert("gap", Value::Float(8.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        // 深色胶囊背景（radius=12 让边缘圆润）。
        let mut bg = WidgetSpec::new("ColoredBox");
        bg.id = Some(WidgetId::new("demo_overlay_bg"));
        bg.props.insert("color", Value::Text("muted".into()));
        bg.props.insert("width", Value::Float(320.0));
        bg.props.insert("height", Value::Float(36.0));
        bg.props.insert("radius", Value::Float(12.0));
        bg.props.insert("label", Value::Text("Tooltip".into()));
        row.children.push(bg);

        // 信息 Icon（左前缀，让 tooltip 视觉更专业）。
        let mut icon = WidgetSpec::new("Icon");
        icon.id = Some(WidgetId::new("demo_overlay_icon"));
        icon.props.insert("name", Value::Text("info".into()));
        icon.props.insert("size", Value::Float(16.0));
        icon.props.insert("color", Value::Text("primary".into()));
        icon.props.insert("label", Value::Text("Info".into()));
        row.children.push(icon);

        let mut text = WidgetSpec::new("Text");
        text.id = Some(WidgetId::new("demo_overlay_text"));
        text.props.insert(
            "text",
            Value::Text("Helpful hint: tooltip floats above (real overlay).".into()),
        );
        row.children.push(text);
        row
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

    /// P1-6 主题单源：把 self.theme 映射为 semantic tokens，由 driver 注入 host.set_tokens。
    /// 控件 paint 直接读 PaintCtx.tokens，无需各自存 theme 字段。
    fn theme_tokens(&self) -> Option<zero_ui_core::theme::SemanticTokens> {
        Some(match self.theme {
            ThemeKind::Light => zero_ui_core::theme::SemanticTokens::light(),
            ThemeKind::Dark => zero_ui_core::theme::SemanticTokens::dark(),
        })
    }

    /// P3-4-3：popover/popup/dialog_scaffold/tooltip 四个 demo 在打开时返回真浮动层。
    ///
    /// - popover：OutsideClick dismiss（点外部关）
    /// - popup：modal barrier（屏蔽主树事件）+ Escape dismiss
    /// - dialog_scaffold：modal barrier + Escape dismiss
    /// - tooltip：锚定 OutsideClick dismiss（hover 触发，离开/hover 别处自动关）
    ///
    /// 浮层视觉 spec 由 build_*_overlay 按 current_page 构造；host 把它 paint 在主树之上。
    fn overlay(&self) -> Option<(zero_ui_overlay::OverlayEntry, Option<WidgetSpec>)> {
        let st = self.current_demo_read();
        let open = st.pressed == 1;
        if !open {
            return None;
        }
        let (entry, spec) = match self.current_page.as_str() {
            "popover" => (
                zero_ui_overlay::OverlayEntry::popover("demo_overlay", zero_ui_core::geometry::Rect::ZERO)
                    .with_anchor_widget("demo_popover_trigger"),
                self.build_popover_overlay(),
            ),
            "popup" => (
                zero_ui_overlay::OverlayEntry::modal("demo_overlay"),
                self.build_popup_overlay(),
            ),
            "dialog_scaffold" => (
                zero_ui_overlay::OverlayEntry::modal("demo_overlay"),
                self.build_dialog_overlay(),
            ),
            "tooltip" => (
                zero_ui_overlay::OverlayEntry::tooltip("demo_overlay", zero_ui_core::geometry::Rect::ZERO)
                    .with_anchor_widget("demo_tooltip_btn"),
                self.build_tooltip_overlay(),
            ),
            _ => return None,
        };
        Some((entry, Some(spec)))
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
                    // P1-9 修复：用 name_en() 稳定字符串匹配（替代 Debug 格式），中文环境也工作。
                    if let Some(g) = ALL_PAGES.iter().find_map(|p| {
                        if p.group.name_en() == group.as_str() {
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
                // P0-2 修复：去掉 n <= 4 硬上限，支持 list_view/command_palette 多项。
                if let Some(n) = s
                    .strip_prefix("gallery.demo.button_click.")
                    .and_then(|t| t.parse::<u32>().ok())
                    && (1..=16).contains(&n)
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

// ========== Factory Registration ==========

fn str_prop(spec: &WidgetSpec, key: &str) -> Option<String> {
    match spec.props.get(key) {
        Some(Value::Text(s)) => Some(s.clone()),
        _ => None,
    }
}

/// 注册画廊所有控件工厂（P3-6-5：全部改用 ui-sdk widgets crate，不再有 gallery 内部组件）。
pub fn register_gallery_factories(host: &mut WidgetHost) {
    // ── chrome（来自 widgets crate，P3-6-3/4 提升）──────────────────────────
    host.register("HeaderTitle", |_spec| Box::new(zero_ui_widgets::HeaderTitle::new()));
    host.register("Spacer", |_spec| Box::new(zero_ui_widgets::Spacer::new()));
    host.register("HeaderButton", |_spec| Box::new(zero_ui_widgets::HeaderButton::new()));
    host.register("NavItem", |_spec| Box::new(zero_ui_widgets::NavItem::new()));
    host.register("GroupHeader", |_spec| Box::new(zero_ui_widgets::GroupHeader::new()));
    host.register("NavSearch", |_spec| Box::new(zero_ui_widgets::NavSearch::new()));
    host.register("DemoTitle", |_spec| Box::new(zero_ui_widgets::DemoTitle::new()));

    // ── 文本/代码（来自 widgets crate，P3-6-1/2 提升）─────────────────────────
    host.register("Text", |_spec| Box::new(zero_ui_widgets::Text::new()));
    host.register("CodeBlock", |_spec| Box::new(zero_ui_widgets::CodeBlock::new()));

    // ── 视觉辅助（来自 widgets crate，P3-4-2）───────────────────────────────
    host.register("ColoredBox", |_spec| Box::new(zero_ui_widgets::ColoredBox::new()));
    host.register("Icon", |_spec| Box::new(zero_ui_widgets::Icon::new()));

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
    use zero_ui_core::event::UiEvent;
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::widget::Widget;

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
        // P3-6-5：NavItem 现在来自 widgets crate，通过 action prop 接收业务 action 名。
        let mut nav = zero_ui_widgets::NavItem::new();
        let mut props = zero_ui_core::widget::Props::new();
        props.insert("label", Value::Text("Toggle".into()));
        props.insert("page_id", Value::Text("toggle".into()));
        props.insert("action", Value::Text("gallery.nav.select".into()));
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        nav.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut flags,
            },
            &props,
        );
        let ev = UiEvent::Pointer {
            phase: zero_ui_core::event::PointerPhase::Released,
            button: Some(zero_ui_core::event::PointerButton::Primary),
            position: zero_ui_core::geometry::Point::new(10.0, 10.0),
            modifiers: zero_ui_core::event::Modifiers::NONE,
            pointer_id: 0,
        };
        let mut ctx = zero_ui_core::widget::EventCtx {
            invalidation: &mut flags,
        };
        let result = nav.event(&mut ctx, &ev);
        match result {
            zero_ui_core::action::EventResult::EmitWithPayload(a, ActionPayload::Text(p)) => {
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
