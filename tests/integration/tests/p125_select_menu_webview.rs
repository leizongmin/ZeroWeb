//! t-p125 常驻回归：github.com/trending select-menu 锚点页在 WebView 层（含
//! run_page_scripts js-worker 路径）的全路径复现。引擎 RenderPipeline 直驱
//!（crates/engine/tests/p125_select_menu_anchors.rs）与 DOM 查询层
//!（tests/integration/tests/p125_select_menu_query.rs）分层回归之外的 WebView
//! 全路径护栏：p125 修复前该页在 js-worker DOM 查询上楔死（gdb/WorkerCensus 取证）。
//!
//! 真站形状与探针链见 engine 侧同题文件头部注释。

use zero_webview::{WebView, WebViewConfig};

const FIXTURE_846: &str = include_str!("../../../crates/engine/tests/fixtures/p125-select-menu-846.html");

/// WebView 全路径：load_html → run_page_scripts（无脚本早退路径）→ DOM 查询。
#[test]
#[serial_test::serial]
fn select_menu_846_webview_full_path() {
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    wv.load_html(FIXTURE_846, None);
    let scripts = wv.run_page_scripts();
    assert!(scripts.is_ok(), "run_page_scripts 须成功: {:?}", scripts.err());
    let anchors = wv.execute_script_with_dom("document.getElementsByTagName('a').length");
    assert!(
        matches!(&anchors, Ok(v) if v.contains("846")),
        "846 锚点须可查询: {:?}",
        anchors
    );
}
