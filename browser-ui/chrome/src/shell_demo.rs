//! browser-shell-demo 参考组合（DC-14 / DC-7 集成验收）。
//!
//! 把 browser-ui/chrome 领域组件 + [`WebViewWidget`] + [`ui_widgets::scrollbar`] 滚动条
//! 经 [`WidgetHost`] 跑 retained 闭环，验证浏览器 chrome 组合力（`examples/browser_shell_demo`
//! 二进制与本模块 `tests` 共用这里的组合函数）：
//!
//! - **统一 Scene**：chrome 填充/文案 + WebView `ExternalSurface` marker + 滚动条 track/thumb
//!   同时进入一个 Scene（不绕过 `ui/render`，DC-7 + DC-3 + DC-4）。
//! - **a11y 树**：[`WidgetHost::semantics`] 覆盖 WebView 节点（"web content"，绝对 rect 来自 layout）
//!   + 可聚焦控件（FOCUSABLE），DC-8。
//! - **焦点**：Tab 在可聚焦控件间遍历；`enter_focus_scope` 把 Tab 困在菜单子树（modal/popup trap，DC-8 phase-3）。
//!
//! 不接真实浏览器进程；WebViewWidget 仅作为占位自定义组件记录 `ExternalSurface`。

use crate::chrome_model::BrowserChromeModel;
use crate::render::{ChromeTabColors, register_chrome_factories, security_color_name};
use zero_ui_adapter_webview::WebViewWidget;
use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Insets, Point, Rect, Size};
use zero_ui_core::layout::{DEFAULT_DENSITY, DEFAULT_TEXT_SCALE, Orientation, WindowMetrics};
use zero_ui_core::scroll::ScrollMetrics;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{
    ComponentType, EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetId,
    WidgetSpec,
};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;
use zero_ui_runtime::{UiApp, WidgetHost};
use zero_ui_widgets::scrollbar::{
    ScrollBarGeometry, ScrollBarStyle, ScrollOrientation, layout_scrollbar, paint_scrollbar,
};

// ── 稳定 WidgetId（demo 断言用）────────────────────────────────────────────────
pub const ID_ROOT: &str = "demo_root";
pub const ID_TOOLBAR: &str = "toolbar";
pub const ID_ADDRESS_BAR: &str = "address_bar";
pub const ID_SECURITY: &str = "security_badge";
pub const ID_NEW_TAB: &str = "new_tab";
pub const ID_TAB_STRIP: &str = "tab_strip";
pub const ID_CONTENT: &str = "content";
pub const ID_WEBVIEW: &str = "webview";
pub const ID_SCROLLBAR: &str = "scrollbar";
pub const ID_MENU: &str = "menu";
pub const ID_MENU_OPEN: &str = "menu_open";
pub const ID_MENU_CLOSE: &str = "menu_close";

fn node_layout(component: &str, id: &str, layout: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new(component);
    s.id = Some(WidgetId::new(id));
    s.props.insert("layout", Value::Text(layout.into()));
    s
}

fn leaf(component: &str, id: &str, bg: &str, text: Option<String>) -> WidgetSpec {
    let mut s = WidgetSpec::new(component);
    s.id = Some(WidgetId::new(id));
    s.props.insert("bg", Value::Text(bg.into()));
    if let Some(t) = text {
        s.props.insert("text", Value::Text(t));
    }
    s
}

/// `message_id` 为 `crate::i18n::ids::*` 常量；label 由 i18n catalog 解析（消除硬编码）。
fn focus_button(id: &str, message_id: &str) -> WidgetSpec {
    let label = crate::i18n::localized_label(message_id);
    let mut s = WidgetSpec::new("demo.FocusButton");
    s.id = Some(WidgetId::new(id));
    s.props.insert("label", Value::Text(label));
    s.props.insert("action", Value::Text(message_id.into()));
    s
}

/// WebView 占位节点（paint 记录 `ExternalSurface`；真实纹理合成在 M2 后期 product-smoke 步）。
fn webview_node(id: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new("demo.WebView");
    s.id = Some(WidgetId::new(id));
    s
}

/// 滚动条节点：`content_h`/`viewport_h`/`offset` 由 props 传入（ScrollBarWidget 据此画 thumb）。
fn scrollbar_node(id: &str, content_h: f32, viewport_h: f32, offset: f32) -> WidgetSpec {
    let mut s = WidgetSpec::new("demo.ScrollBar");
    s.id = Some(WidgetId::new(id));
    s.props.insert("content_h", Value::Float(content_h as f64));
    s.props.insert("viewport_h", Value::Float(viewport_h as f64));
    s.props.insert("offset", Value::Float(offset as f64));
    s
}

/// 构造 browser-shell-demo 声明树（桌面式）。
///
/// 结构（column）：security_badge 栏 / toolbar(row)[NewTab·MenuOpen·AddressBar(flex)] / TabStrip /
/// content(row)[ScrollBar·WebView(flex)] / menu(column)[MenuClose]。
///
/// **布局**：host 的 Row/Column 支持 `flex` 弹性权重（`ui/runtime::host`）。填宽子节点声明
/// `flex=1`，按权重瓜分非弹性子节点占用后的剩余主轴空间（`Expanded` 语义），与子节点声明顺序无关。
/// toolbar 中 AddressBar(flex=1) 填满固定按钮之后的剩余宽；content 中 WebView(flex=1) 填满 12px
/// 滚动条之后的剩余宽。security_badge 仍单独成栏（语义独立，非争宽规避）。
pub fn build_demo_spec(model: &BrowserChromeModel) -> WidgetSpec {
    let mut root = node_layout("browser.DesktopBrowserShell", ID_ROOT, "column");

    // security_badge 单独一栏（全宽，语义独立）。
    root.children.push(leaf(
        "browser.SecurityBadge",
        ID_SECURITY,
        security_color_name(model.security),
        Some(crate::i18n::security_status_label(security_color_name(model.security))),
    ));

    // toolbar：固定宽按钮在前，AddressBar(flex=1) 填剩余宽。
    let mut toolbar = node_layout("browser.ToolbarRow", ID_TOOLBAR, "row");
    toolbar
        .children
        .push(focus_button(ID_NEW_TAB, crate::i18n::ids::NEW_TAB));
    toolbar
        .children
        .push(focus_button(ID_MENU_OPEN, crate::i18n::ids::OPEN_MENU));
    let mut address = leaf(
        "browser.AddressBar",
        ID_ADDRESS_BAR,
        "chrome",
        Some(model.address_text.clone()),
    );
    address.props.insert("flex", Value::Int(1));
    toolbar.children.push(address);
    root.children.push(toolbar);

    root.children.push(leaf(
        "browser.BrowserTabStrip",
        ID_TAB_STRIP,
        "chrome",
        Some(
            model
                .tabs
                .iter()
                .map(|t| t.title.clone())
                .collect::<Vec<_>>()
                .join(" | "),
        ),
    ));

    // content：滚动条在前（固定 12px），WebView(flex=1) 填剩余宽。
    let mut content = node_layout("browser.ContentRow", ID_CONTENT, "row");
    content
        .children
        .push(scrollbar_node(ID_SCROLLBAR, 2000.0, 600.0, 300.0));
    let mut webview = webview_node(ID_WEBVIEW);
    webview.props.insert("flex", Value::Int(1));
    content.children.push(webview);
    root.children.push(content);

    // menu 子树（FocusScope trap 演示）：含一个可聚焦 Close 项。
    let mut menu = node_layout("browser.MenuPanel", ID_MENU, "column");
    menu.children
        .push(focus_button(ID_MENU_CLOSE, crate::i18n::ids::CLOSE_MENU));
    root.children.push(menu);

    root
}

/// 可聚焦按钮（demo 控件）：背景 + 文案；点击/Enter emit action；a11y 角色 BUTTON。
pub struct FocusBarButton {
    label: String,
    action: ActionId,
    bg: Color,
    text_color: Color,
}

impl FocusBarButton {
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> FocusBarButton {
        let label = match spec.props.get("label") {
            Some(Value::Text(s)) => s.clone(),
            _ => "Button".to_string(),
        };
        let action = match spec.props.get("action") {
            Some(Value::Text(s)) => ActionId::new(s),
            _ => ActionId::new("tap"),
        };
        FocusBarButton {
            label,
            action,
            bg: tokens.primary,
            text_color: tokens.on_primary,
        }
    }
}

impl Widget for FocusBarButton {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: zero_ui_core::event::PointerPhase::Released,
                button: Some(zero_ui_core::event::PointerButton::Primary),
                ..
            } => EventResult::Emit(self.action.clone()),
            UiEvent::Key {
                code,
                action: key_action,
                ..
            } if code.0.as_str() == "Enter" && matches!(key_action, zero_ui_core::event::KeyAction::Pressed) => {
                EventResult::Emit(self.action.clone())
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 估计宽度 = label 字符数 * 8 + 16 padding；固定高 32。
        let w = (self.label.chars().count() as f32 * 8.0 + 24.0).clamp(constraints.min_width, constraints.max_width);
        let h = 32.0_f32.clamp(constraints.min_height, constraints.max_height);
        Size::new(w, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(80.0, 32.0));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        ctx.recorder
            .draw_text(&self.label, Point::new(8.0, 20.0), 13.0, self.text_color);
    }
    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: WidgetId::new(""),
            rect: Rect::ZERO,
            flags: SemanticsFlags::BUTTON,
            label: Some(SemanticsLabel::Literal(self.label.clone().into())),
            value: None,
            children: Vec::new(),
        });
    }
    fn focusable(&self) -> bool {
        true
    }
}

/// 滚动条控件（demo）：用 `ui/widgets::scrollbar` 的几何/绘制 helper 画 track+thumb 进统一 Scene。
/// props：`content_h`/`viewport_h`/`offset`（垂直滚动）；固定 12px 宽，填高。
pub struct ScrollBarWidget {
    content_h: f32,
    viewport_h: f32,
    offset: f32,
    style: ScrollBarStyle,
}

impl ScrollBarWidget {
    pub fn from_spec(spec: &WidgetSpec) -> ScrollBarWidget {
        let num = |key: &str| -> f32 {
            match spec.props.get(key) {
                Some(Value::Float(f)) => *f as f32,
                Some(Value::Int(i)) => *i as f32,
                _ => 0.0,
            }
        };
        ScrollBarWidget {
            content_h: num("content_h"),
            viewport_h: num("viewport_h"),
            offset: num("offset"),
            style: ScrollBarStyle::default(),
        }
    }
}

impl Widget for ScrollBarWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 固定 12px 宽（不填宽，让 Row 把剩余宽留给 WebView）；填高。
        Size::new(
            12.0_f32.clamp(constraints.min_width, constraints.max_width),
            constraints.max_height,
        )
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(12.0, 100.0));
        let track = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
        let metrics = ScrollMetrics {
            content_width: size.width,
            content_height: self.content_h,
            viewport_width: size.width,
            viewport_height: self.viewport_h,
            scroll_x: 0.0,
            scroll_y: self.offset,
        };
        if let Some(geom) = layout_scrollbar(track, metrics, ScrollOrientation::Vertical) {
            paint_scrollbar(ctx.recorder, &geom, &self.style);
        }
    }
    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: WidgetId::new(""),
            rect: Rect::ZERO,
            flags: SemanticsFlags::NONE,
            label: Some(SemanticsLabel::Literal("scrollbar".into())),
            value: None,
            children: Vec::new(),
        });
    }
}

/// 注册 demo 全部工厂：chrome 叶子（`register_chrome_factories`）+ FocusButton + WebView + ScrollBar。
pub fn register_demo_factories(host: &mut WidgetHost, tokens: &SemanticTokens, theme: &zero_ui_core::theme::Theme) {
    register_chrome_factories(host, tokens, ChromeTabColors::from_tokens(tokens));
    let t = *tokens;
    host.register("demo.FocusButton", move |s| Box::new(FocusBarButton::from_spec(s, &t)));
    let theme = theme.clone();
    host.register("demo.WebView", move |_s| {
        // 占位 WebView：viewport rect 由 host layout 决定（widget 填充分配区域）。
        Box::new(WebViewWidget::new(Rect::ZERO, 1.0, theme.clone()).with_surface_id(1))
    });
    host.register("demo.ScrollBar", move |s| Box::new(ScrollBarWidget::from_spec(s)));
}

/// browser-shell-demo 的 UiApp 适配（spec IF-006）：把 demo 声明树接入 [`WinitDriver`]。
///
/// demo 是**只读渲染 + 焦点/FocusScope 演示** fixture（无应用 reducer）：`root_spec` 返回
/// [`build_demo_spec`] 产出的声明树；`dispatch` 不处理 action（返回 `UnknownAction`，driver
/// 不重建）。真实浏览器宿主在此接入 BrowserAction reducer（参考 counter/form 示例）。
pub struct ShellDemoApp {
    spec: WidgetSpec,
}

impl UiApp for ShellDemoApp {
    fn root_spec(&self) -> WidgetSpec {
        self.spec.clone()
    }

    fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
        ActionResult::UnknownAction(action.clone())
    }
}

/// 构造并运行 demo 到一次 paint：经 [`WinitDriver`]（DC-2 run-loop 核心）驱动 retained 闭环
/// （register → begin = set_root+layout+tight 约束+paint），返回 [`WidgetHost`] 供调用方检视
/// scene / semantics / 焦点 / FocusScope。所有 demo 测试因此均经 driver 路径证明。
pub fn build_demo_host(
    model: &BrowserChromeModel,
    tokens: &SemanticTokens,
    theme: &zero_ui_core::theme::Theme,
    viewport: Size,
) -> WidgetHost {
    let mut app = ShellDemoApp {
        spec: build_demo_spec(model),
    };
    let metrics = WindowMetrics {
        logical_size: viewport,
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: DEFAULT_TEXT_SCALE,
        density: DEFAULT_DENSITY,
        orientation: Orientation::from_size(viewport),
    };
    let mut driver = WinitDriver::new(&mut app, metrics);
    register_demo_factories(driver.host_mut(), tokens, theme);
    driver.begin();
    driver.into_host()
}

/// 按图元种类统计 Scene（demo 打印用）。
pub fn count_primitives(scene: &Scene) -> (usize, usize, usize, usize) {
    let mut fill = 0;
    let mut text = 0;
    let mut surface = 0;
    let mut other = 0;
    for e in &scene.entries {
        match &e.primitive {
            RenderPrimitive::FillRect { .. } => fill += 1,
            RenderPrimitive::Text { .. } | RenderPrimitive::TextBlob { .. } => text += 1,
            RenderPrimitive::ExternalSurface { .. } => surface += 1,
            _ => other += 1,
        }
    }
    (fill, text, surface, other)
}

/// 在 a11y 树里按标签文案查节点（demo 断言用）。
pub fn find_sem_by_label<'a>(node: &'a SemanticsNode, label_part: &str) -> Option<&'a SemanticsNode> {
    let matched = node
        .label
        .as_ref()
        .map(|l| match l {
            SemanticsLabel::Literal(s) | SemanticsLabel::Message(s) => s.as_str(),
        })
        .is_some_and(|s| s.contains(label_part));
    if matched {
        return Some(node);
    }
    for c in &node.children {
        if let Some(n) = find_sem_by_label(c, label_part) {
            return Some(n);
        }
    }
    None
}

// 抑制未使用告警：ComponentType/ScrollBarGeometry 在 demo API/未来扩展用到。
#[allow(dead_code)]
fn _type_anchors(_a: ComponentType, _b: ScrollBarGeometry) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SecurityState;
    use zero_ui_core::theme::{ColorPalette, ResolvedColorScheme, ThemeId, ThemeResolver};

    fn tokens() -> SemanticTokens {
        SemanticTokens::light()
    }
    fn theme() -> zero_ui_core::theme::Theme {
        ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        )
    }
    fn model() -> BrowserChromeModel {
        let mut m = BrowserChromeModel::new();
        m.address_text = "https://example.com".into();
        m.security = SecurityState::Secure;
        m.tabs = vec![crate::BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "Example".into(),
            loading: false,
        }];
        m.active_tab_index = Some(0);
        m
    }

    /// 空地址栏模型（触发 placeholder "Search or enter URL..." 渲染）。
    fn empty_model() -> BrowserChromeModel {
        let mut m = BrowserChromeModel::new();
        m.address_text = "".into(); // 空 → AddressBarWidget 渲染 placeholder
        m.security = SecurityState::Secure;
        m.tabs = vec![crate::BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "New Tab".into(),
            loading: false,
        }];
        m.active_tab_index = Some(0);
        m
    }

    #[test]
    fn unified_scene_contains_chrome_webview_and_scrollbar() {
        // DC-7 + DC-3 + DC-4 组合：chrome 填充/文案 + WebView ExternalSurface + 滚动条 track/thumb 同进一 Scene。
        let host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let scene = host.scene();
        let (fill, text, surface, _other) = count_primitives(scene);
        assert!(fill >= 4, "chrome bars + scrollbar track/thumb fills, got {fill}");
        assert!(text >= 1, "address URL text, got {text}");
        assert!(surface >= 1, "webview external surface, got {surface}");
        // URL 文案进 Scene。
        let texts = crate::render::scene_texts(scene);
        assert!(
            texts.iter().any(|t| t.contains("example.com")),
            "address URL rendered, got {texts:?}"
        );
        // WebView ExternalSurface 带 surface_id=1。
        assert!(
            scene
                .entries
                .iter()
                .any(|e| matches!(&e.primitive, RenderPrimitive::ExternalSurface { surface_id: 1, .. }))
        );
    }

    #[test]
    fn placeholder_text_renders_in_empty_address_bar() {
        // DC-11 R57：地址栏文本为空时渲染 placeholder "Search or enter URL..."。
        let host = build_demo_host(&empty_model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let scene = host.scene();
        let texts = crate::render::scene_texts(scene);
        assert!(
            texts.iter().any(|t| t.contains("Search or enter URL")),
            "placeholder text should appear in empty address bar, got {texts:?}"
        );
    }

    #[test]
    fn url_text_does_not_contain_placeholder() {
        // 反向守卫：有 URL 文本时不应出现 placeholder 文案。
        let host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let scene = host.scene();
        let texts = crate::render::scene_texts(scene);
        assert!(
            texts.iter().any(|t| t.contains("https://example.com")),
            "URL text should appear in address bar, got {texts:?}"
        );
        assert!(
            !texts.iter().any(|t| t.contains("Search or enter URL")),
            "placeholder must NOT appear when URL is present, got {texts:?}"
        );
    }

    #[test]
    fn semantics_tree_covers_webview_and_focusables() {
        // DC-8：a11y 树覆盖 WebView 节点（绝对 rect）+ 可聚焦按钮。
        let host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let sem = host.semantics().expect("root laid out");
        // WebView "web content" 节点存在且 rect 来自 layout（非 ZERO）。
        let wv = find_sem_by_label(&sem, "web content").expect("webview semantics node");
        assert!(
            wv.rect.size.width > 0.0 && wv.rect.size.height > 0.0,
            "webview rect from layout, got {:?}",
            wv.rect
        );
        // 可聚焦按钮（New Tab）带 FOCUSABLE。
        let btn = find_sem_by_label(&sem, "New Tab").expect("new tab button semantics");
        assert!(btn.flags.contains(SemanticsFlags::FOCUSABLE));
        assert!(btn.flags.contains(SemanticsFlags::BUTTON));
    }

    #[test]
    fn focus_tab_cycles_across_focusable_buttons() {
        // DC-8：Tab 在可聚焦控件间遍历。可聚焦集合 = {new_tab, menu_open, menu_close}。
        let mut host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
        let first = host.focused_id().unwrap().0.to_string();
        host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
        let second = host.focused_id().unwrap().0.to_string();
        assert_ne!(first, second, "Tab must move focus between focusables");
        // 声明顺序首个可聚焦 = new_tab（toolbar 中）。
        assert_eq!(first, ID_NEW_TAB);
    }

    #[test]
    fn focus_scope_traps_tab_in_menu_subtree() {
        // DC-8 phase-3：进入 menu 子树作用域（trap）→ Tab 在 menu 内折返，不逃逸到 toolbar 按钮。
        let mut host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        host.enter_focus_scope(WidgetId::new(ID_MENU), true);
        // menu 内仅 menu_close 一个可聚焦项 → Tab 折返回 menu_close，绝不跳到 new_tab。
        host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
        assert_eq!(host.focused_id().unwrap().0.as_str(), ID_MENU_CLOSE);
        host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
        assert_eq!(
            host.focused_id().unwrap().0.as_str(),
            ID_MENU_CLOSE,
            "trap wraps within menu, never escapes to toolbar"
        );
        // 退出作用域后恢复全局遍历。
        host.exit_focus_scope();
        host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
        assert_ne!(
            host.focused_id().unwrap().0.as_str(),
            ID_MENU_CLOSE,
            "after exit, global traversal reaches toolbar focusables"
        );
    }

    #[test]
    fn demo_host_is_browser_chrome_composition() {
        // 结构断言：viewport 与 scrollbar 都被布局（rect 非零），且 scrollbar 在 webview 左侧（x 更小）。
        let host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let sb = host.rect_of(&WidgetId::new(ID_SCROLLBAR)).expect("scrollbar laid out");
        let wv = host.rect_of(&WidgetId::new(ID_WEBVIEW)).expect("webview laid out");
        assert!(sb.size.width > 0.0 && wv.size.width > 0.0);
        assert!(
            sb.origin.x < wv.origin.x,
            "scrollbar placed before webview in row (sb.x={}, wv.x={})",
            sb.origin.x,
            wv.origin.x
        );
        // WebView 填充剩余宽（远大于 scrollbar 12px）。
        assert!(
            wv.size.width > 1000.0,
            "webview fills remaining width, got {}",
            wv.size.width
        );
    }

    #[test]
    fn flex_children_fill_toolbar_and_content_remaining_width() {
        // flex 布局增强：AddressBar(flex=1) 填满 toolbar 固定按钮之后的剩余宽；
        // WebView(flex=1) 填满 content 中 12px 滚动条之后的剩余宽。两者均与固定子节点共存。
        let host = build_demo_host(&model(), &tokens(), &theme(), Size::new(1280.0, 800.0));
        let ab = host
            .rect_of(&WidgetId::new(ID_ADDRESS_BAR))
            .expect("address bar laid out");
        let new_tab = host
            .rect_of(&WidgetId::new(ID_NEW_TAB))
            .expect("new tab button laid out");
        // AddressBar 在 NewTab 之后（同 toolbar row，声明顺序）。
        assert!(
            ab.origin.x >= new_tab.right(),
            "address bar placed after fixed buttons (ab.x={}, new_tab.right={})",
            ab.origin.x,
            new_tab.right()
        );
        // AddressBar 填满 toolbar 剩余宽（1280 - 两个固定按钮宽 ≈ 远大于按钮宽）。
        assert!(
            ab.size.width > 1000.0,
            "address bar (flex=1) fills toolbar remaining width, got {}",
            ab.size.width
        );
        // AddressBar 右沿接近视口右边（填满至 1280）。
        assert!(
            ab.right() > 1270.0,
            "address bar reaches right edge, got right={}",
            ab.right()
        );
    }
}
