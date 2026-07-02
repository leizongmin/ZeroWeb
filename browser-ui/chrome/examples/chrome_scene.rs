//! Chrome → Scene 快照（DC-7 验收产物，`cargo run -p zero-browser-chrome --example chrome_scene`）。
//!
//! 构造一个有数据的 BrowserChromeModel，经 DesktopBrowserShell 组装声明树，
//! 在 WidgetHost 注册 chrome 绘制工厂后 layout + paint，打印产出的统一 Scene 图元。
//! 证明浏览器领域组件输出进统一 SDK scene（不绕过 ui/render）。

use zero_browser_chrome::render::scene_texts;
use zero_browser_chrome::{
    BrowserChromeModel, BrowserChromeShell, DesktopBrowserShell, SecurityState, register_chrome_factories,
};
use zero_ui_core::geometry::{Constraints, Insets, Size};
use zero_ui_core::layout::WindowMetrics;
use zero_ui_render::render_node::RenderPrimitive;
use zero_ui_runtime::WidgetHost;

fn main() {
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

    let metrics = WindowMetrics {
        logical_size: Size::new(1280.0, 800.0),
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
    };

    let spec = DesktopBrowserShell.build(&model, &metrics);
    let mut host = WidgetHost::new();
    register_chrome_factories(&mut host);
    host.set_root(&spec);
    let root_size = host.layout(Constraints::loose(metrics.logical_size));
    let scene = host.paint();

    println!("DesktopBrowserShell scene snapshot (DC-7)");
    println!("root size: {:.0}x{:.0}", root_size.width, root_size.height);
    println!("scene entries: {}", scene.entries.len());
    for (i, e) in scene.entries.iter().enumerate() {
        let kind = match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => {
                format!("FillRect(rgba({:.2},{:.2},{:.2},_))", color.r, color.g, color.b)
            }
            RenderPrimitive::StrokeRect { .. } => "StrokeRect".to_string(),
            RenderPrimitive::Text { text, .. } => format!("Text({:?})", text),
            RenderPrimitive::TextBlob { .. } => "TextBlob".to_string(),
            RenderPrimitive::ExternalSurface { surface_id, .. } => format!("ExternalSurface(id={surface_id})"),
        };
        println!("  [{i:2}] source={:<22} {kind}", e.source.0);
    }
    println!("texts: {:?}", scene_texts(scene));
}
