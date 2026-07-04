//! adaptive-shell-demo — AdaptiveBrowserChrome + mock BrowserChromeModel 示例。
//!
//! `cargo run -p zero-browser-chrome --example adaptive_shell_demo`
//!
//! 展示 AdaptiveBrowserChrome 在不同视口大小下自动切换 shell（Desktop/Tablet/Phone）。

use zero_browser_chrome::{
    AdaptiveBrowserChrome, BrowserChromeModel, BrowserTab, NavigationButtons, SecurityState,
};
use zero_ui_core::layout::{InputClass, PlatformClass, WindowMetrics};

fn mock_model() -> BrowserChromeModel {
    let mut m = BrowserChromeModel::new();
    m.address_text = "https://example.com".into();
    m.security = SecurityState::Secure;
    m.navigation = NavigationButtons::new(true, false, false);
    m.tabs = vec![
        BrowserTab { id: zero_browser_shell::TabId(1), title: "Example".into(), loading: false },
    ];
    m.active_tab_index = Some(0);
    m
}

fn main() {
    let chrome = AdaptiveBrowserChrome::new();
    let model = mock_model();

    let scenarios: [(&str, WindowMetrics, PlatformClass, InputClass); 3] = [
        ("Desktop 1280×800", WindowMetrics::desktop(), PlatformClass::Desktop, InputClass::Pointer),
        ("Tablet  1024×768", WindowMetrics::tablet(), PlatformClass::Desktop, InputClass::Touch),
        ("Phone    390×844", WindowMetrics::phone(), PlatformClass::Mobile, InputClass::Touch),
    ];

    for (label, metrics, platform, input) in &scenarios {
        let result = chrome.build(&model, metrics, *platform, *input);
        println!(
            "[{label}] Shell={kind:?}  child_count={n}",
            kind = result.kind,
            n = result.spec.children.len(),
        );
    }
}
