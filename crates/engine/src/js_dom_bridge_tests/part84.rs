// part84 — html-semantics M2 切片 67（R5274，2026-10-24）：OBJECT/EMBED
// contentDocument/contentWindow/getSVGDocument null 反射。
//
// 根因：get-trap 只在 IFRAME 分支派发 contentDocument/contentWindow/getSVGDocument
//（part04 R115），OBJECT/EMBED fallthrough 返 undefined——WPT embedded-content
// document-getters-return-null-for-cross-origin 断言 null（spec：object 元素
// 不代表嵌套浏览上下文时 contentDocument/contentWindow 返 null「or null」尾步；
// GetSVGDocumentSupport 对非 SVG document 返 null）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#dom-object-contentdocument
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#dom-getsvgdocument

/// 钉①：(a)(b)(c) object.contentDocument/contentWindow/getSVGDocument() 恒 null；
/// (d) embed.getSVGDocument() === null 且 typeof function；(e) embed 无
/// contentDocument 成员（undefined——HTMLEmbedElement 无此 IDL，null 反射不扩杀）。
/// 断言翻转：(a)-(d) = OBJECT/EMBED 分支被撤（fallthrough 复活返 undefined）；
/// (e) = tag 门收成 OBJECT/EMBED 全员三成员（null 误反射到 EMBED）。
#[test]
fn object_embed_null_browsing_context_r5274() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><object id='o' data='data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\"><rect height=\"100\" width=\"100\"/></svg>'></object><embed id='e'></embed></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s67a".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var o = document.getElementById('o');\
             var e = document.getElementById('e');\
             globalThis.__r5274a = String(o.contentDocument === null);\
             globalThis.__r5274b = String(o.contentWindow === null);\
             globalThis.__r5274c = String(o.getSVGDocument() === null);\
             globalThis.__r5274d = String(e.getSVGDocument !== undefined && e.getSVGDocument() === null);\
             globalThis.__r5274e = String(e.contentDocument === undefined);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5274a").unwrap().value,
        "true",
        "(a) object.contentDocument === null（spec「does not represent a nested browsing context → null」）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5274b").unwrap().value,
        "true",
        "(b) object.contentWindow === null（同上尾步）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5274c").unwrap().value,
        "true",
        "(c) object.getSVGDocument() === null（GetSVGDocumentSupport 非 SVG document 臂）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5274d").unwrap().value,
        "true",
        "(d) embed.getSVGDocument() === null（mixin 双元素命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5274e").unwrap().value,
        "true",
        "(e) embed.contentDocument undefined（HTMLEmbedElement 无 contentDocument IDL）"
    );
}

/// 钉②：门不误伤 IFRAME 活浏览上下文面——无 src iframe contentDocument 为空
/// Document（R130 面，非 null）且 createElement 可用；iframe getSVGDocument()
/// 非 null（同源活跃文档臂）。
/// 断言翻转：tag 门误扩杀 IFRAME（contentDocument/getSVGDocument 变 null）。
#[test]
fn iframe_browsing_context_kept_r5274() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><iframe id='f'></iframe></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s67b".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var f = document.getElementById('f');\
             globalThis.__r5274g = String(f.contentDocument !== null && typeof f.contentDocument.createElement === 'function');\
             globalThis.__r5274h = String(f.getSVGDocument() !== null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5274g").unwrap().value,
        "true",
        "(g) 无 src iframe contentDocument 为空 Document 非 null（R130 既有面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5274h").unwrap().value,
        "true",
        "(h) iframe.getSVGDocument() 非 null（同源活跃文档臂，既有面）"
    );
}
