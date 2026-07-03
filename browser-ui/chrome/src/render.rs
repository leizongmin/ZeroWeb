//! Chrome → Scene 渲染桥（DC-7 完整验收）。
//!
//! 把 adaptive shell 产出的 `browser.*` [`WidgetSpec`] 声明树，经 [`WidgetHost`] 注册的
//! chrome 绘制工厂 paint 进统一 [`Scene`]（走 `ui/render` 的 `PaintRecorder`，**不绕过** ui/render）。
//!
//! - **容器节点**（shell 根 / ToolbarRow / Phone 顶部）由 shell 在 `props.layout` 声明为
//!   `column`/`row`（见 `ui/runtime::host` 的 `node_container_kind`），无 widget —— host 据此布局子节点。
//! - **叶子节点**（AddressBar/SecurityBadge/NavigationButtons/BrowserMenu/BrowserTabStrip/
//!   BookmarksBar/PageViewportFrame/FindBar）注册绘制工厂 [`ChromePanel`]，按 props 语义色 + 文案 paint。
//!
//! 本模块是「chrome 组件 → 可渲染 Scene」的桥，证明浏览器领域组件输出进统一 SDK scene（DC-7）。
//! 真实主题色解析在 DC-5 主题系统接入后由 semantic token 解析；此处用 [`chrome_color`] 语义色名映射。

use zero_ui_core::action::EventResult;
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetSpec,
};
use zero_ui_render::Scene;
use zero_ui_runtime::WidgetHost;

/// chrome 色名 → semantic token 名（组件消费 semantic token，不硬编码浏览器色值；DC-5）。
///
/// chrome 领域别名（`chrome`/`accent`/`secure`/`insecure`/`mixed`/`dangerous`/`viewport`）
/// 映射到通用 semantic token；非别名（含标准 token 名 `surface`/`primary`/... 与未知名）
/// 返回 `None`，由 [`chrome_color_themed`] 直接当 token 名解析或回落 `surface`。
fn chrome_alias_token(name: &str) -> Option<&'static str> {
    match name {
        "chrome" | "toolbar_bg" => Some("surface"),
        "tab_strip_bg" => Some("surface"),
        "address_bar_bg" => Some("background"),
        "accent" => Some("primary"),
        "secure" => Some("success"),
        "insecure" | "mixed" => Some("warning"),
        "dangerous" => Some("error"),
        "viewport" => Some("background"),
        _ => None,
    }
}

/// chrome 色名 + 主题 token → [`Color`]（DC-5：组件经 semantic token 解析，随主题切换）。
///
/// 先按 chrome 别名映射，否则直接当 token 名（`surface`/`primary`/`on_surface`/...）解析；
/// 未知名回落 `surface`。
/// `tab_strip_bg` 在 light 主题下用 on_surface 混入 surface 产生肉眼可见的中灰层次。
pub fn chrome_color_themed(name: &str, tokens: &SemanticTokens) -> Color {
    let token = chrome_alias_token(name).unwrap_or(name);
    let base = tokens.color_for(token).unwrap_or(tokens.surface);
    match name {
        "tab_strip_bg" => base.mix(tokens.on_surface, 0.12),
        _ => base,
    }
}

/// chrome 色名 → [`Color`]（浅色基线；等价 `chrome_color_themed(name, &SemanticTokens::light())`）。
pub fn chrome_color(name: &str) -> Color {
    chrome_color_themed(name, &SemanticTokens::light())
}

/// chrome 安全状态 → 语义色名。
pub fn security_color_name(state: crate::security_badge::SecurityState) -> &'static str {
    use crate::security_badge::SecurityState;
    match state {
        SecurityState::Secure => "secure",
        SecurityState::Insecure => "insecure",
        SecurityState::Mixed => "mixed",
        SecurityState::Dangerous => "dangerous",
    }
}

/// chrome 叶子绘制控件：填充背景（语义色）+ 可选文案。
///
/// `fill_viewport = true` 时占满可用宽高（用于 PageViewportFrame）；否则占满宽、固定高（bars）。
/// `fill_background = false` 时跳过 `fill_rect`（DC-14 替换式迁移：PageViewportFrame 不填底色，
/// 页面内容由 WebView 绘制，避免 SDK chrome 的 viewport 底色覆盖页面内容）。
/// paint 读 `ctx.clip` 得节点可视尺寸，以局部坐标 (0,0) 填充 —— host 按节点 origin 平移后覆盖节点 rect。
pub struct ChromePanel {
    bg: Color,
    text: Option<String>,
    text_color: Color,
    height: f32,
    fill_viewport: bool,
    fill_background: bool,
    /// 占满可用宽度（cross-axis 拉伸）。用于全宽 bar（TabStrip / BookmarksBar）——
    /// 这些 bar 在 column 容器里须铺满窗口宽，而非按文案宽度收缩。
    fill_width: bool,
    /// 背景圆角半径（逻辑像素，四角同）。> 0 时用 `fill_rounded_rect`（地址栏 pill 等）。
    corner_radius: f32,
}

impl ChromePanel {
    /// 由声明节点构造：`bg`/`text`/`text_color`/`height` 从 props 读，颜色经 semantic token
    /// 解析（`tokens`，DC-5）；缺省 bg 用 `default_bg` 别名。
    pub fn from_spec(
        spec: &WidgetSpec,
        default_bg: &str,
        default_height: f32,
        fill_viewport: bool,
        tokens: &SemanticTokens,
    ) -> ChromePanel {
        Self::from_spec_with_fill(spec, default_bg, default_height, fill_viewport, true, tokens)
    }

    /// 同 [`from_spec`]，但 `fill_background` 控制是否绘制底色（DC-14 替换式迁移：
    /// PageViewportFrame 设 `false` 不填底色，页面内容由 WebView 绘制在上层）。
    pub fn from_spec_with_fill(
        spec: &WidgetSpec,
        default_bg: &str,
        default_height: f32,
        fill_viewport: bool,
        fill_background: bool,
        tokens: &SemanticTokens,
    ) -> ChromePanel {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => default_bg,
        };
        let bg = chrome_color_themed(bg_name, tokens);
        let text = match spec.props.get("text") {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        };
        let text_color = match spec.props.get("text_color") {
            Some(Value::Text(s)) => chrome_color_themed(s, tokens),
            _ => tokens.on_surface,
        };
        let height = match spec.props.get("height") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => default_height,
        };
        let fill_width = match spec.props.get("fill_width") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        let corner_radius = match spec.props.get("corner_radius") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => 0.0,
        };
        ChromePanel {
            bg,
            text,
            text_color,
            height,
            fill_viewport,
            fill_background,
            fill_width,
            corner_radius,
        }
    }
}

impl Widget for ChromePanel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        if self.fill_viewport {
            Size::new(constraints.max_width, constraints.max_height)
        } else {
            let height = self.height.clamp(constraints.min_height, constraints.max_height);
            let width = if self.fill_width {
                // 全宽 bar（TabStrip / BookmarksBar）：占满 column 容器可用宽。
                constraints.max_width
            } else if let Some(text) = &self.text {
                let char_count = text.chars().count().max(1) as f32;
                let approx = (char_count * 14.0 * 0.55 + 16.0).ceil();
                approx.clamp(constraints.min_width, constraints.max_width)
            } else {
                let icon = (height * 0.75).ceil().max(24.0);
                icon.clamp(constraints.min_width, constraints.max_width)
            };
            Size::new(width, height)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // ctx.clip = 祖先 clip ∩ 节点 rect（绝对坐标）；其 size 即节点可视尺寸。
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, self.height));
        if self.fill_background {
            let rect = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
            if self.corner_radius > 0.0 {
                ctx.recorder.fill_rounded_rect(rect, self.corner_radius, self.bg);
            } else {
                ctx.recorder.fill_rect(rect, self.bg);
            }
        }
        if let Some(text) = &self.text {
            // 文本基线：bars 在 height-8 处；viewport 顶部 18px。
            let baseline = if self.fill_viewport {
                18.0
            } else {
                (self.height - 8.0).max(10.0)
            };
            ctx.recorder
                .draw_text(text, Point::new(6.0, baseline), 14.0, self.text_color);
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 把 chrome `browser.*` 叶子组件工厂注册到 host。
///
/// 容器节点（shell 根 / ToolbarRow）不注册 widget —— 它们经 `props.layout` 由 host 布局。
/// 调用方仍需先 `host.set_root(&shell.build(...))` 注入声明树。
pub fn register_chrome_factories(host: &mut WidgetHost, tokens: &SemanticTokens) {
    // bars：固定高度的语义色条 + 可选文案；颜色经 semantic token 解析（DC-5）。
    // 高度对齐 apps/browser/src/layout.rs 手绘 chrome（DC-14 像素级等价）：
    //   TAB_STRIP_HEIGHT = TAB_BAR_TOP_INSET(6) + TAB_BAR_HEIGHT(34) = 40
    //   ADDRESS_BAR_HEIGHT = 44（地址行：AddressBar / NavigationButtons / BrowserMenu / SecurityBadge）
    //   BOOKMARKS_BAR_HEIGHT = 28
    // SemanticTokens 是 Copy：每个 move 闭包各持一份副本。
    let t = *tokens;
    host.register("browser.AddressBar", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 44.0, false, &t))
    });
    host.register("browser.NavigationButtons", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 44.0, false, &t))
    });
    host.register("browser.BrowserMenu", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 44.0, false, &t))
    });
    host.register("browser.BrowserTabStrip", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 40.0, false, &t))
    });
    host.register("browser.BookmarksBar", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 28.0, false, &t))
    });
    host.register("browser.SecurityBadge", move |s| {
        Box::new(ChromePanel::from_spec(s, "secure", 44.0, false, &t))
    });
    host.register("browser.FindBar", move |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 36.0, false, &t))
    });
    // 视口：占满剩余区域，**不填底色**（DC-14 替换式迁移：页面内容由 WebView 绘制在上层，
    // SDK chrome viewport 只负责布局空间，不覆盖页面像素）。
    host.register("browser.PageViewportFrame", move |s| {
        Box::new(ChromePanel::from_spec_with_fill(s, "viewport", 0.0, true, false, &t))
    });
}

/// 同 [`register_chrome_factories`]，但 viewport 使用 [`WebViewWidget`] 替代 [`ChromePanel`]
/// 占位符（DC-3 phase-2）。WebViewWidget paint 产出 `ExternalSurface` marker，供后端按
/// `surface_id` 合成 WebView 纹理。
///
/// props 约定：`surface_id`（u64，默认 0）由宿主分配；`scale_factor`（f32，默认 1.0）。
/// viewport rect 由 layout 填充可用空间确定。
pub fn register_chrome_factories_with_webview(
    host: &mut WidgetHost,
    tokens: &SemanticTokens,
    scheme: zero_ui_core::theme::ResolvedColorScheme,
) {
    register_chrome_factories(host, tokens);
    // 替换 PageViewportFrame：使用 WebViewWidget 产出 ExternalSurface marker。
    let theme = zero_ui_core::theme::ThemeResolver::build_theme(
        zero_ui_core::theme::ThemeId::new("zero"),
        "Zero",
        scheme,
        zero_ui_core::theme::ColorPalette::default(),
    );
    host.register("browser.PageViewportFrame", move |s| {
        let surface_id = match s.props.get("surface_id") {
            Some(zero_ui_core::binding::Value::Int(v)) => *v as u64,
            _ => 0,
        };
        let scale_factor = match s.props.get("scale_factor") {
            Some(zero_ui_core::binding::Value::Float(v)) => *v as f32,
            _ => 1.0,
        };
        Box::new(
            zero_ui_adapter_webview::WebViewWidget::new(
                zero_ui_core::geometry::Rect::from_ltrb(0.0, 0.0, 0.0, 0.0),
                scale_factor,
                theme.clone(),
            )
            .with_surface_id(surface_id),
        )
    });
}

/// 取 Scene 中所有文本图元的文案（断言用）。
pub fn scene_texts(scene: &Scene) -> Vec<String> {
    use zero_ui_render::render_node::RenderPrimitive;
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrowserChromeShell;
    use crate::chrome_model::BrowserChromeModel;
    use crate::shell::DesktopBrowserShell;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::render_node::RenderPrimitive;

    fn metrics(w: f32, h: f32) -> WindowMetrics {
        let logical_size = Size::new(w, h);
        WindowMetrics {
            logical_size,
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
        }
    }

    #[test]
    fn desktop_shell_paints_into_unified_scene() {
        // DC-7：chrome 组件输出进统一 Scene（不绕过 ui/render）。
        let mut model = BrowserChromeModel::new();
        model.address_text = "https://example.com".into();
        model.security = crate::security_badge::SecurityState::Secure;
        model.tabs = vec![crate::BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "Example".into(),
            loading: false,
        }];
        model.active_tab_index = Some(0);

        let m = metrics(1280.0, 800.0);
        let spec = DesktopBrowserShell.build(&model, &m);

        let mut host = WidgetHost::new();
        register_chrome_factories(&mut host, &zero_ui_core::theme::SemanticTokens::light());
        host.set_root(&spec);
        let root_size = host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(
            m.logical_size.width,
            m.logical_size.height,
        )));
        // shell 根为 column 容器，铺满视口。
        assert!((root_size.width - 1280.0).abs() < 0.01);
        let scene = host.paint().clone();

        // 1. 地址栏 URL 文案进 Scene。
        let texts = scene_texts(&scene);
        assert!(
            texts.iter().any(|t| t.contains("example.com")),
            "address URL must render into scene, got {texts:?}"
        );
        // 2. 至少有 toolbar/tabstrip/bookmarks/address 的背景 fill + 视口 fill。
        let fills = scene
            .entries
            .iter()
            .filter(|e| matches!(e.primitive, RenderPrimitive::FillRect { .. }))
            .count();
        assert!(fills >= 4, "expected several chrome bar fills, got {fills}");
        // 3. 视口节点（fill_viewport）铺满剩余高度：rect 高接近 800-顶部bars。
        let vp = host.rect_of(&WidgetId::new("viewport")).expect("viewport laid out");
        assert!(
            vp.size.height > 600.0,
            "viewport should fill remaining height, got {}",
            vp.size.height
        );
    }

    #[test]
    fn security_state_drives_badge_color() {
        // 安全状态 → chrome 色名 → semantic token → Color（DC-5：经 token 解析）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let dark = zero_ui_core::theme::SemanticTokens::dark();
        assert_eq!(
            security_color_name(crate::security_badge::SecurityState::Secure),
            "secure"
        );
        // dangerous → error token（浅色/深色不同色值，证明经主题解析而非硬编码）。
        assert_eq!(chrome_color_themed("dangerous", &light), light.error);
        assert_eq!(chrome_color_themed("dangerous", &dark), dark.error);
        // secure → success token。
        assert_eq!(chrome_color_themed("secure", &light), light.success);
        // chrome → surface；viewport → background；未知 → surface 回落。
        assert_eq!(chrome_color_themed("chrome", &light), light.surface);
        assert_eq!(chrome_color_themed("viewport", &light), light.background);
        assert_eq!(chrome_color_themed("unknown", &light), light.surface);
        // accent → primary。
        assert_eq!(chrome_color_themed("accent", &light), light.primary);
    }

    #[test]
    fn chrome_panel_default_text_color_uses_token() {
        // ChromePanel 默认文案色 = on_surface token（不硬编码），随主题切换。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.AddressBar");
        spec.props.insert("bg", Value::Text("chrome".into()));
        let panel = ChromePanel::from_spec(&spec, "chrome", 36.0, false, &light);
        assert_eq!(panel.text_color, light.on_surface);
        assert_eq!(panel.bg, light.surface);
    }

    #[test]
    fn viewport_frame_skips_background_fill() {
        // DC-14 替换式迁移：PageViewportFrame 不填底色（页面内容由 WebView 绘制）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let spec = WidgetSpec::new("browser.PageViewportFrame");
        let panel = ChromePanel::from_spec_with_fill(&spec, "viewport", 0.0, true, false, &light);
        assert!(!panel.fill_background, "PageViewportFrame fill_background = false");
        assert!(panel.fill_viewport, "fill_viewport 仍为 true（占满剩余空间）");
    }

    #[test]
    fn bars_keep_background_fill_default() {
        // 非视口 chrome 控件（AddressBar/TabStrip 等）默认 fill_background = true（向后兼容）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.AddressBar");
        spec.props.insert("bg", Value::Text("chrome".into()));
        let panel = ChromePanel::from_spec(&spec, "chrome", 36.0, false, &light);
        assert!(panel.fill_background, "bars 默认 fill_background = true");
    }

    #[test]
    fn webview_viewport_paints_external_surface() {
        // DC-3 phase-2：register_chrome_factories_with_webview 注册 WebViewWidget 为
        // PageViewportFrame，paint 产出 ExternalSurface marker（而非 ChromePanel 的 fill_rect）。

        use zero_ui_core::geometry::Size;
        use zero_ui_render::render_node::RenderPrimitive;
        use zero_ui_runtime::WidgetHost;

        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let mut host = WidgetHost::new();
        register_chrome_factories_with_webview(&mut host, &tokens, zero_ui_core::theme::ResolvedColorScheme::Light);

        // 构造含 surface_id 的 PageViewportFrame spec。
        let mut spec = WidgetSpec::new("browser.PageViewportFrame");
        spec.id = Some(zero_ui_core::widget::WidgetId::new("viewport"));
        spec.props.insert("surface_id", Value::Int(42));
        spec.props.insert("scale_factor", Value::Float(2.0));
        host.set_root(&spec);
        host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(1280.0, 800.0)));

        let scene = host.paint().clone();
        let external = scene
            .entries
            .iter()
            .find(|e| matches!(e.primitive, RenderPrimitive::ExternalSurface { .. }));
        assert!(
            external.is_some(),
            "WebViewWidget factory should produce ExternalSurface marker"
        );
    }
}
