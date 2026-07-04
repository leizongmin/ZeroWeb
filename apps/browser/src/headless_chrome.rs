//! Headless 模式下的 SDK chrome 渲染与控制。
//!
//! 把 SDK chrome 渲染管线接入无头协议，让 `browsingContext.captureScreenshot`
//! 能合成 chrome + 页面，并提供 chrome 控制命令（click / getLayout / getSemantics /
//! rectOf / emittedActions），用于浏览器 UI 自动化验收（product-acceptance）。
//!
//! 设计要点：
//! - 内联装配 `WidgetHost`（不能调 `render_chrome_via_sdk_with_webview_surface`，因为它
//!   内部消费 host），保留 host 引用用于后续 `dispatch_event` / `rect_of` / `semantics`
//! - 复用 chrome crate 的公开装配函数（register_chrome_factories / DesktopBrowserShell）
//! - feature `sdk-chrome` 关闭时整个模块不存在（main.rs 用 `#[cfg(feature = "sdk-chrome")]` gate）

use std::sync::Arc;

use serde_json::{Value, json};
use zero_browser_chrome::chrome_model::BrowserChromeModel;
use zero_browser_chrome::render::{self, ChromeTabColors};
use zero_browser_chrome::shell::{BrowserChromeShell, DesktopBrowserShell};
use zero_browser_shell::BrowserShell;
use zero_engine::PrefersColorSchemeValue;
use zero_render_foundation::color::Color as RfColor;
use zero_render_foundation::primitive::{FillPrimitive, GlyphPrimitive};
use zero_ui_adapter_render_foundation::RenderFoundationBackend;
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Insets, Point, Rect, Size};
use zero_ui_core::layout::{Orientation, WindowMetrics};
use zero_ui_core::theme::{Color as UiColor, ResolvedColorScheme, SemanticTokens};
use zero_ui_core::widget::WidgetId;
use zero_ui_render::paint_scene;
use zero_ui_runtime::WidgetHost;
use zero_ui_runtime::host::EmittedAction;
use zero_webview::WebView;

use crate::colors::ChromePalette;

/// viewport 节点 id（与 chrome crate 内部 ID_VIEWPORT 同字面量；该常量是 pub(crate) 不能直接 import）。
const ID_VIEWPORT: &str = "viewport";

/// 单次 chrome 渲染产物：合成后的 fills / glyphs + viewport_rect + host。
pub(crate) struct ChromeFrame {
    pub fills: Vec<FillPrimitive>,
    pub glyphs: Vec<GlyphPrimitive>,
    pub viewport_rect: Option<Rect>,
    /// 已 layout + paint 的 host；调用方可用于 dispatch_event / rect_of / semantics。
    pub host: WidgetHost,
}

/// 渲染 SDK chrome + 页面（page fills 偏移到 viewport_rect 后合并）。
///
/// 内联装配逻辑（与 chrome crate 的 render_chrome_via_sdk_with_webview_surface 等价，
/// 但不消费 host），让调用方能继续 dispatch_event。
pub(crate) fn render_chrome_frame(
    shell: &BrowserShell,
    webview: &mut WebView,
    viewport_width: u32,
    viewport_height: u32,
    scale_factor: f32,
) -> ChromeFrame {
    let palette = ChromePalette::for_scheme(PrefersColorSchemeValue::Light);
    let tokens = sdk_chrome_tokens(&palette);
    let tab_colors = sdk_chrome_tab_colors(&palette);

    let page_render = webview.render();
    let page_glyph_primitives: Vec<zero_render_foundation::primitive::GlyphPrimitive> =
        page_render.primitives.glyphs.clone();

    let backend = sdk_font_backend();
    let logical_size = Size::new(
        (viewport_width as f32 / scale_factor).max(1.0),
        (viewport_height as f32 / scale_factor).max(1.0),
    );
    let metrics = WindowMetrics {
        logical_size,
        scale_factor,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: 1.0,
        density: 1.0,
        orientation: Orientation::from_size(logical_size),
    };

    // 装配 host（与 chrome crate 内部装配等价，但保留 host 引用）。
    let model = BrowserChromeModel::from_shell_with_window_controls(
        shell,
        if cfg!(target_os = "windows") { 138.0 } else { 0.0 },
    );
    let spec = DesktopBrowserShell.build(&model, &metrics);
    let mut host = WidgetHost::new();
    render::register_chrome_factories_with_webview(&mut host, &tokens, ResolvedColorScheme::Light, tab_colors);
    host.set_tokens(tokens);
    inject_font_metrics(&mut host, &backend);
    host.set_root(&spec);
    host.layout(Constraints::loose(logical_size));

    let viewport_rect = host.rect_of(&WidgetId::new(ID_VIEWPORT));
    let scene = host.paint().clone();

    // 渲染 chrome scene 到 bridge。
    let mut bridge = RenderFoundationBackend::new_with_text_size(logical_size, backend);
    paint_scene(&scene, &mut bridge);
    let (mut chrome_primitives, _chrome_cache) = bridge.into_primitives_and_cache();

    // 把页面 fills 偏移到 viewport_rect.origin 后追加到 chrome fills 之后（z-order：chrome 在上，
    // 但 chrome 的 viewport 区域是透明的，page 透过来）。glyphs 直接合并。
    let mut combined_fills = std::mem::take(&mut chrome_primitives.fills);
    if let Some(vp) = viewport_rect {
        let dx = vp.origin.x;
        let dy = vp.origin.y;
        for mut fill in page_render.primitives.fills {
            fill.rect.origin.x += dx;
            fill.rect.origin.y += dy;
            combined_fills.push(fill);
        }
    }

    // 页面 glyphs 偏移到 viewport。
    let mut combined_glyphs = chrome_primitives.glyphs;
    if let Some(vp) = viewport_rect {
        let dx = vp.origin.x;
        let dy = vp.origin.y;
        for mut g in page_glyph_primitives {
            g.x += dx;
            g.y += dy;
            combined_glyphs.push(g);
        }
    } else {
        combined_glyphs.extend(page_glyph_primitives);
    }

    ChromeFrame {
        fills: combined_fills,
        glyphs: combined_glyphs,
        viewport_rect,
        host,
    }
}

fn inject_font_metrics(host: &mut WidgetHost, backend: &Arc<zero_text_foundation::FontdueBackend>) {
    use zero_text_foundation::FontId;
    const UI_FONT_SIZE: f32 = 13.0;
    if let Some((raw_ascent, raw_descent)) = backend.line_metrics(FontId(0), UI_FONT_SIZE) {
        host.set_font_metrics(raw_ascent / UI_FONT_SIZE, raw_descent / UI_FONT_SIZE);
    }
}

fn sdk_chrome_tokens(p: &ChromePalette) -> SemanticTokens {
    let f = rf_color_to_ui;
    SemanticTokens {
        background: f(p.address_bar_bg),
        on_background: f(p.address_bar_text),
        surface: f(p.toolbar_bg),
        on_surface: f(p.tab_text),
        primary: f(p.address_bar_border_focused),
        on_primary: UiColor::rgba(1.0, 1.0, 1.0, 1.0),
        error: f(p.tab_crashed),
        on_error: UiColor::rgba(1.0, 1.0, 1.0, 1.0),
        success: f(p.address_bar_secure),
        on_success: UiColor::rgba(1.0, 1.0, 1.0, 1.0),
        warning: f(p.address_bar_insecure),
        on_warning: UiColor::rgba(0.0, 0.0, 0.0, 1.0),
    }
}

fn sdk_chrome_tab_colors(p: &ChromePalette) -> ChromeTabColors {
    ChromeTabColors {
        strip_bg: rf_color_to_ui(p.toolbar_bg),
        active_bg: rf_color_to_ui(p.tab_active_bg),
        bar_bg: rf_color_to_ui(p.tab_bar_bg),
        separator: rf_color_to_ui(p.tab_separator),
        address_border: rf_color_to_ui(p.address_bar_border),
        window_icon: rf_color_to_ui(p.tab_close),
    }
}

fn rf_color_to_ui(c: RfColor) -> UiColor {
    UiColor::rgba(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

fn sdk_font_backend() -> Arc<zero_text_foundation::FontdueBackend> {
    use std::sync::OnceLock;
    static BACKEND: OnceLock<Arc<zero_text_foundation::FontdueBackend>> = OnceLock::new();
    BACKEND
        .get_or_init(|| Arc::new(zero_text_foundation::FontdueBackend::new()))
        .clone()
}

// ── chrome 控制命令实现 ──

pub(crate) fn make_click_events(x: f32, y: f32) -> [UiEvent; 2] {
    let pos = Point::new(x, y);
    let mk = |phase| UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position: pos,
        modifiers: Modifiers::default(),
        pointer_id: 0,
    };
    [mk(PointerPhase::Pressed), mk(PointerPhase::Released)]
}

pub(crate) fn emitted_actions_to_json(actions: &[EmittedAction]) -> Value {
    use zero_ui_core::action::ActionPayload;
    let arr: Vec<Value> = actions
        .iter()
        .map(|a| {
            let mut obj = json!({
                "action": a.action.0.as_str(),
            });
            if let Some(payload) = &a.payload {
                obj["payload"] = match payload {
                    ActionPayload::Unit => json!("unit"),
                    ActionPayload::Bool(b) => json!({"bool": b}),
                    ActionPayload::Int(i) => json!({"int": i}),
                    ActionPayload::Float(n) => json!({"float": n}),
                    ActionPayload::Text(t) => json!({"text": t}),
                    ActionPayload::Value(v) => {
                        // zero_ui_core::binding::Value → serde_json::Value（best-effort）。
                        match v {
                            zero_ui_core::binding::Value::Text(t) => json!({"text": t}),
                            zero_ui_core::binding::Value::Float(n) => json!({"float": n}),
                            zero_ui_core::binding::Value::Bool(b) => json!({"bool": b}),
                            _ => json!("structured"),
                        }
                    }
                };
            }
            obj
        })
        .collect();
    Value::Array(arr)
}

pub(crate) fn rect_to_json(rect: Rect) -> Value {
    json!({
        "x": rect.origin.x,
        "y": rect.origin.y,
        "width": rect.size.width,
        "height": rect.size.height,
    })
}

/// chrome.click 命令的最小 reducer：识别 chrome actions 并应用到 shell。
pub(crate) fn apply_chrome_action_to_shell(shell: &mut BrowserShell, action: &str) -> (usize, &'static str) {
    use zero_browser_chrome::actions;
    match action {
        actions::NAV_BACK => {
            let ok = shell.go_back();
            (1, if ok { "go_back applied" } else { "go_back no-op" })
        }
        actions::NAV_FORWARD => {
            let ok = shell.go_forward();
            (1, if ok { "go_forward applied" } else { "go_forward no-op" })
        }
        actions::NAV_HOME => {
            shell.navigate("about:home");
            (1, "go_home applied")
        }
        actions::TAB_NEW => {
            shell.new_tab(None);
            (1, "new_tab applied")
        }
        actions::MENU_TOGGLE => (1, "menu_toggle (client-side state)"),
        actions::FIND_CLOSE => {
            shell.find_close();
            (1, "find_close applied")
        }
        actions::FIND_NEXT => {
            shell.find_next();
            (1, "find_next applied")
        }
        actions::FIND_PREV => {
            shell.find_previous();
            (1, "find_previous applied")
        }
        _ => (0, "unknown action"),
    }
}
