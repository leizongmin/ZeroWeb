//! s89（html-semantics-compat execution-timing）：外链样式表 × getComputedStyle 面。
//! WPT execution-timing/106-*「stylesheets blocking scripts」族可观察判据：
//! 脚本运行时级联已含外链样式表规则（runner 即时抓取序——prepare 先于
//! run_page_scripts，spec 样式表阻塞脚本语义在该序下的等价位）；107-*
//! document.write 写入的 link **不**反塑当拍脚本内的计算样式（写后值仍
//! initial）。探针走 runner 等价入口 complete_fetched_page +
//! run_page_scripts_strict；取值在页内脚本同步完成（镜像语料时序）。

use std::collections::HashMap;
use std::sync::Arc;

use crate::{WebView, WebViewConfig};

const PAGE_URL: &str = "https://wpt.test/execution-timing/probe.html";
/// WPT 上游 css/background.css（pin 31597693）——`#test{position:fixed}`。
const BACKGROUND_CSS: &str = "#test {position:fixed}";

fn webview_with_css(files: HashMap<String, String>) -> WebView {
    WebView::new(WebViewConfig {
        image_source_fetcher: Some(Arc::new(move |url| files.get(url).map(|css| css.clone().into_bytes()))),
        ..WebViewConfig::default()
    })
}

/// 加载页面（含外链 css 供给）→ 跑页面脚本 → 读脚本内取值。
fn load_and_probe(webview: &mut WebView, html: &str) -> String {
    webview.prepare_document_state(PAGE_URL);
    webview.complete_fetched_page(html, PAGE_URL);
    webview.run_page_scripts_strict().unwrap();
    webview.execute_script("globalThis.__zwProbe").unwrap()
}

/// 106-noimport 面：head 外链样式表先抓取并入级联 → 脚本读 `position` 得
/// `fixed`。红态：级联不含外链规则（getComputedStyle 纯快照路径恒空外部
/// CSS）→ `static`。
#[test]
fn external_stylesheet_reaches_computed_style_before_scripts() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/execution-timing/css/background.css".to_string(),
        BACKGROUND_CSS.to_string(),
    )]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head>
<link rel="stylesheet" href="css/background.css">
</head><body>
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "fixed"
    );
}

/// 106-import 面：`@import` 链文本展开（相对**导入表 URL** 解析、防环）——
/// import.css 引 background.css 规则同样入级联。红态：@import 不展开 →
/// `static`。
#[test]
fn imported_stylesheet_expands_into_computed_style() {
    let mut webview = webview_with_css(HashMap::from([
        (
            "https://wpt.test/execution-timing/css/import.css".to_string(),
            "@import url(\"background.css\")".to_string(),
        ),
        (
            "https://wpt.test/execution-timing/css/background.css".to_string(),
            BACKGROUND_CSS.to_string(),
        ),
    ]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head>
<link rel="stylesheet" href="css/import.css">
</head><body>
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "fixed"
    );
}

/// 107 面：脚本内 document.write 写入的 link 不反塑当拍计算样式——`position`
/// 保持初始值 `static`（写入的样式表不进入本轮已装填级联）。红态：`''`
/// （写后选择器解析失败）或 `fixed`（写入规则误入级联）。
#[test]
fn document_written_link_does_not_reshape_inflight_computed_style() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/execution-timing/css/background.css".to_string(),
        BACKGROUND_CSS.to_string(),
    )]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head>
</head><body>
<div id="test">Test</div>
<script>
document.write("<link rel='stylesheet' href='css/background.css'>");
globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;
</script>
</body></html>"#
        ),
        "static"
    );
}
