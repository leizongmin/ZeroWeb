// part110 — html-semantics M2 切片 411（2026-11-12）：maxlength 仅适用于
// text/search/url/tel/email/password 六态 + textarea（s411 勘根钉）。
//
// maxlength-number.html（file-level Timeout）红根：`<input type=number
// maxlength=1>` send_keys '1234' 断言 value 全保留——旧版 plan/live 两侧均
// 无 type 适用性门，第二键起 `MaxLengthReached` noop → dispatch_action 错误
// 串 reject send_keys promise 链 → 无 catch 的 .then 案 unhandled rejection
// 挂死 → 文件 Timeout。修复两侧同口径：plan 侧（webview `TextActionState`
// 构造）+ shim live 守卫（`__zw_text_input`）均按 `_zwNormalizeInputType`
// 六态白名单门控 maxlength 读面。spec：maxlength 只适用于 Text/Search/URL/
// Tel/Email/Password 状态，type 缺省/非法值按 missing/invalid value default
// 落 Text 态 → 限制仍生效（对照腿）。
// https://html.spec.whatwg.org/multipage/input.html#attr-input-maxlength

/// 钉：shim live 守卫三腿——number 不限长（驱动案镜像；旧版第二键起截断）
/// / text 六态限制生效（既有绿面不回退）/ 非法 type 值落 Text 态限制生效
///（invalid value default；旧版无门时同效，此处钉 normalize 规则不被后续
/// 重构抹掉）。
#[test]
fn input_maxlength_state_gate_faces_s411() {
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
        "<html><body>\
         <input id=\"n\" type=\"number\" maxlength=\"1\">\
         <input id=\"t\" type=\"text\" maxlength=\"1\">\
         <input id=\"f\" type=\"foo\" maxlength=\"1\">\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/dir/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 腿 A（driving 案镜像）：number + maxlength=1 键入 '12' → value 全保留
    //（旧版第二键起 maxlength 截断 → '1'；runner 侧还叠加 MaxLengthReached
    // noop reject 链，桥测只钉 live 写面）。
    sandbox
        .execute(
            "__zw_text_input('#n', '1'); __zw_text_input('#n', '2');\
             globalThis.__s411n = document.querySelector('#n').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s411n)").unwrap().value,
        "12",
        "number 态 maxlength 不适用 → 键入全保留实得 {}",
        sandbox.execute("String(globalThis.__s411n)").unwrap().value
    );

    // 腿 B（既有绿面不回退）：text + maxlength=1 → 第二键截断 → '1'。
    sandbox
        .execute(
            "__zw_text_input('#t', '1'); __zw_text_input('#t', '2');\
             globalThis.__s411t = document.querySelector('#t').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s411t)").unwrap().value,
        "1",
        "text 态 maxlength 仍生效实得 {}",
        sandbox.execute("String(globalThis.__s411t)").unwrap().value
    );

    // 腿 C（invalid value default）：非法 type 值落 Text 态 → 限制生效 → '1'。
    sandbox
        .execute(
            "__zw_text_input('#f', '1'); __zw_text_input('#f', '2');\
             globalThis.__s411f = document.querySelector('#f').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s411f)").unwrap().value,
        "1",
        "非法 type 值 invalid default = Text 态限制生效实得 {}",
        sandbox.execute("String(globalThis.__s411f)").unwrap().value
    );
}
