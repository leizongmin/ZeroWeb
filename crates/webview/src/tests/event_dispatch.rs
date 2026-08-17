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
fn indexed_db_factory_and_schema_route_to_host() {
    let mut wv = new_webview();
    wv.prepare_document_state("https://storage.example/page");
    wv.load_html(
        r#"<html><body><script>
        var openRequest = indexedDB.open("app", 1);
        openRequest.onupgradeneeded = function () {
          openRequest.result.createObjectStore("items", {keyPath:"id", autoIncrement:true});
        };
        openRequest.onsuccess = function () { globalThis.__idbOpened = true; };
        </script></body></html>"#,
        None,
    );

    wv.run_page_scripts_strict().unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__idbOpened)")
            .unwrap(),
        "true"
    );
    assert_eq!(
        wv.execute_script(
            r#"JSON.parse(__zw_idb(JSON.stringify({op:"inspect",name:"app"}))
               .slice("__zw_idb_ok:".length)).database.stores[0].name"#,
        )
        .unwrap(),
        "items"
    );

    wv.execute_script(
        r#"var aborted = indexedDB.open("app", 2);
           aborted.onupgradeneeded = function () { aborted.transaction.abort(); };"#,
    )
    .unwrap();
    assert_eq!(
        wv.execute_script(
            r#"String(JSON.parse(__zw_idb(JSON.stringify({op:"inspect",name:"app"}))
               .slice("__zw_idb_ok:".length)).database.version)"#,
        )
        .unwrap(),
        "1"
    );

    wv.execute_script(r#"indexedDB.deleteDatabase("app");"#).unwrap();
    assert_eq!(
        wv.execute_script(
            r#"String(JSON.parse(__zw_idb(JSON.stringify({op:"inspect",name:"app"}))
               .slice("__zw_idb_ok:".length)).database)"#,
        )
            .unwrap(),
        "null"
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
