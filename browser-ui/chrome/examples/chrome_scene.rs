//! Chrome → Scene 快照（DC-7 验收产物，`cargo run -p zero-browser-chrome --example chrome_scene`）。
//!
//! 构造一个有数据的 BrowserChromeModel，经 DesktopBrowserShell 组装声明树，
//! 在 WidgetHost 注册 chrome 绘制工厂后 layout + paint，打印产出的统一 Scene 图元。
//! 证明浏览器领域组件输出进统一 SDK scene（不绕过 ui/render）。
//!
//! DC-5：工厂接收 semantic token —— 同一声明树在 light/dark 主题下产出不同色值，
//! 证明 chrome 组件消费 semantic token（不硬编码色值）。

use zero_browser_chrome::render::scene_texts;
use zero_browser_chrome::{
    BrowserChromeModel, BrowserChromeShell, DesktopBrowserShell, SecurityState, register_chrome_factories,
};
use zero_ui_core::geometry::{Constraints, Insets, Size};
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::SemanticTokens;
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;
use zero_ui_runtime::WidgetHost;

fn build_model() -> BrowserChromeModel {
    let mut model = BrowserChromeModel::new();
    model.address_text = "https://example.com".into();
    model.security = SecurityState::Secure;
    model.navigation = zero_browser_chrome::NavigationButtons::new(true, false, false);
    model.tabs = vec![zero_browser_chrome::BrowserTab {
        id: zero_browser_shell::TabId(1),
        title: "Example".into(),
        loading: false,
    }];
    model.active_tab_index = Some(0);
    model
}

fn render(
    tokens: &SemanticTokens,
    label: &str,
    spec: &zero_ui_core::widget::WidgetSpec,
    metrics: &WindowMetrics,
) -> Scene {
    let mut host = WidgetHost::new();
    register_chrome_factories(&mut host, tokens);
    host.set_root(spec);
    let root_size = host.layout(Constraints::loose(metrics.logical_size));
    let scene = host.paint().clone();

    println!(
        "\n=== {label} (semantic tokens) === root {:.0}x{:.0}, {} entries",
        root_size.width,
        root_size.height,
        scene.entries.len()
    );
    for (i, e) in scene.entries.iter().enumerate() {
        let kind = match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => {
                format!("Fill(rgba({:.2},{:.2},{:.2},_))", color.r, color.g, color.b)
            }
            RenderPrimitive::StrokeRect { .. } => "Stroke".to_string(),
            RenderPrimitive::Text { text, .. } => format!("Text({:?})", text),
            RenderPrimitive::TextBlob { .. } => "TextBlob".to_string(),
            RenderPrimitive::ExternalSurface { surface_id, .. } => format!("ExternalSurface(id={surface_id})"),
            RenderPrimitive::Image { key, .. } => format!("Image(ref={})", key.0),
        };
        println!("  [{i:2}] source={:<16} {kind}", e.source.0);
    }
    println!("  texts: {:?}", scene_texts(&scene));
    scene
}

fn main() {
    let metrics = WindowMetrics {
        logical_size: Size::new(1280.0, 800.0),
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: 1.0,
        density: 1.0,
        orientation: zero_ui_core::layout::Orientation::Landscape,
    };
    let model = build_model();
    let spec = DesktopBrowserShell.build(&model, &metrics);

    let light = render(&SemanticTokens::light(), "Light theme", &spec, &metrics);
    let dark = render(&SemanticTokens::dark(), "Dark theme", &spec, &metrics);

    // DC-5：同一 chrome 节点在 light/dark 下色值不同（消费 semantic token，随主题切换）。
    let light_vp = light
        .entries
        .iter()
        .find(|e| e.source.0 == "viewport")
        .and_then(|e| match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => Some(*color),
            _ => None,
        });
    let dark_vp = dark
        .entries
        .iter()
        .find(|e| e.source.0 == "viewport")
        .and_then(|e| match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => Some(*color),
            _ => None,
        });
    if let (Some(l), Some(d)) = (light_vp, dark_vp) {
        println!(
            "\nDC-5 theme switch: viewport fill light=rgba({:.2},{:.2},{:.2},_) dark=rgba({:.2},{:.2},{:.2},_)",
            l.r, l.g, l.b, d.r, d.g, d.b
        );
    }
}
