//! M2：页面脚本执行 + DOM 事件派发（run_page_scripts / dispatch_event）。

use crate::{WebView, WebViewConfig};

fn new_webview() -> WebView {
    WebView::new(WebViewConfig::default())
}

const PAGE_WITH_LISTENER: &str = r#"<html><body>
<button id="btn">Click</button>
<script>
document.getElementById('btn').addEventListener('click', function() {
  document.getElementById('btn').textContent = 'clicked';
});
</script>
</body></html>"#;

#[test]
fn run_page_scripts_registers_listeners_and_applies_mutations() {
    let mut wv = new_webview();
    let _ = wv.load_html(PAGE_WITH_LISTENER, None);

    // 页面脚本执行：注册监听器（无 mutation 时应返回原 HTML）
    let html = wv.run_page_scripts().expect("run page scripts");
    assert!(html.contains("addEventListener"), "脚本已内联在 HTML 中");

    // 脚本执行后按钮文本未变（监听器未触发）
    assert!(!wv.html_content().contains(">clicked<"));
}

#[test]
fn run_page_scripts_registers_indexed_db_host() {
    let mut wv = new_webview();
    wv.prepare_document_state("https://storage.example/page");
    wv.load_html(
        r#"<html><body><script>
        globalThis.__idbWire = __zw_idb(JSON.stringify({op:"open",name:"app",version:1}));
        </script></body></html>"#,
        None,
    );

    wv.run_page_scripts_strict().unwrap();
    assert_eq!(
        wv.execute_script("globalThis.__idbWire.startsWith('__zw_idb_ok:')")
            .unwrap(),
        "true"
    );
    assert_eq!(
        wv.execute_script("globalThis.__idbWire.includes('\"name\":\"app\"')")
            .unwrap(),
        "true"
    );
}

#[test]
fn dispatch_event_triggers_listener_and_applies_mutations() {
    let mut wv = new_webview();
    let _ = wv.load_html(PAGE_WITH_LISTENER, None);

    // 派发 click → 监听器修改按钮文本（mutation）→ 应用到 HTML 重新渲染
    wv.dispatch_event("#btn", "click").expect("dispatch click");

    let html = wv.html_content();
    assert!(html.contains("clicked"), "点击后按钮文本应变为 clicked: {html}");
    assert!(!html.contains(">Click<"), "原文本应被替换");
}

// 注：missing selector 的 dispatch 行为依赖 shim 内部 selector fallback
//（非本层契约）——WebDriver 侧 Click 前先 Find Element 保证存在性（404 语义），
// 此处不覆盖。
