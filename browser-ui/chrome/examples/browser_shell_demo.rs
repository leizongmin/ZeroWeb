//! browser-shell-demo（DC-14）—— 浏览器 chrome + WebView + ScrollBar 组合示例。
//!
//! `cargo run -p zero-browser-chrome --example browser_shell_demo`
//!
//! 构造一个有数据的 [`BrowserChromeModel`]，经 [`zero_browser_chrome::shell_demo`] 组合
//! （chrome 领域组件 + `WebViewWidget` + 滚动条）跑 retained 闭环，打印统一 Scene、a11y 树、
//! 焦点与 FocusScope trap 演示。证明浏览器 chrome 在 UI SDK 下的组合力（不接真实浏览器进程）。

use zero_browser_chrome::shell_demo::{ID_MENU, ID_MENU_CLOSE, build_demo_host, count_primitives, find_sem_by_label};
use zero_browser_chrome::{BrowserChromeModel, BrowserTab, NavigationButtons, SecurityState};
use zero_ui_core::focus::FocusDirection;
use zero_ui_core::geometry::Size;
use zero_ui_core::theme::{ColorPalette, ResolvedColorScheme, SemanticTokens, ThemeId, ThemeResolver};
use zero_ui_render::render_node::RenderPrimitive;

fn main() {
    let mut model = BrowserChromeModel::new();
    model.address_text = "https://example.com".into();
    model.security = SecurityState::Secure;
    model.navigation = NavigationButtons::new(true, false, false);
    model.tabs = vec![BrowserTab {
        id: zero_browser_shell::TabId(1),
        title: "Example".into(),
        loading: false,
    }];
    model.active_tab_index = Some(0);

    let tokens = SemanticTokens::light();
    let theme = ThemeResolver::build_theme(
        ThemeId::new("zero"),
        "Zero",
        ResolvedColorScheme::Light,
        ColorPalette::default(),
    );
    let viewport = Size::new(1280.0, 800.0);

    let host = build_demo_host(&model, &tokens, &theme, viewport);
    let scene = host.scene();
    let (fill, text, surface, other) = count_primitives(scene);

    println!(
        "\n=== browser-shell-demo (DC-14) — viewport {:.0}x{:.0} ===",
        viewport.width, viewport.height
    );
    println!(
        "Scene: {} entries (fill={}, text={}, external_surface={}, other={})",
        scene.entries.len(),
        fill,
        text,
        surface,
        other
    );
    for (i, e) in scene.entries.iter().enumerate() {
        let kind = match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => {
                format!("Fill(rgba({:.2},{:.2},{:.2},_))", color.r, color.g, color.b)
            }
            RenderPrimitive::Text { text, .. } => format!("Text({text:?})"),
            RenderPrimitive::TextBlob { .. } => "TextBlob".to_string(),
            RenderPrimitive::ExternalSurface { surface_id, .. } => {
                format!("ExternalSurface(id={surface_id})")
            }
            RenderPrimitive::StrokeRect { .. } => "Stroke".to_string(),
            RenderPrimitive::Image { key, .. } => format!("Image(ref={})", key.0),
        };
        println!("  [{i:2}] source={:<22} {kind}", e.source.0);
    }

    // 关键区域 rect。
    println!("\nLayout regions:");
    for id in ["address_bar", "security_badge", "tab_strip", "scrollbar", "webview"] {
        if let Some(r) = host.rect_of(&zero_ui_core::widget::WidgetId::new(id)) {
            println!(
                "  {id:<14} origin=({:.0},{:.0}) size=({:.0},{:.0})",
                r.origin.x, r.origin.y, r.size.width, r.size.height
            );
        }
    }

    // a11y 树：WebView + 可聚焦按钮。
    let sem = host.semantics().expect("root semantics");
    println!("\na11y tree root children: {}", sem.children.len());
    if let Some(wv) = find_sem_by_label(&sem, "web content") {
        println!(
            "  webview node: rect=({:.0},{:.0},{:.0},{:.0})",
            wv.rect.origin.x, wv.rect.origin.y, wv.rect.size.width, wv.rect.size.height
        );
    }
    if let Some(btn) = find_sem_by_label(&sem, "New Tab") {
        println!(
            "  focusable button: {:?} (FOCUSABLE={})",
            btn.label,
            btn.flags.contains(zero_ui_core::semantics::SemanticsFlags::FOCUSABLE)
        );
    }

    // 焦点遍历 + FocusScope trap 演示（独立 host，避免借用 host 的 scene）。
    let mut fhost = build_demo_host(&model, &tokens, &theme, viewport);
    fhost.focus_next(FocusDirection::Forward);
    println!("\nFocus: first Tab → {}", fhost.focused_id().unwrap().0);
    fhost.enter_focus_scope(zero_ui_core::widget::WidgetId::new(ID_MENU), true);
    fhost.focus_next(FocusDirection::Forward);
    println!(
        "FocusScope trap: enter '{ID_MENU}' → Tab trapped on '{ID_MENU_CLOSE}' = {}",
        fhost.focused_id().unwrap().0
    );
    fhost.exit_focus_scope();
    println!("(exit scope; global traversal resumes)");
}
