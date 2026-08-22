#[test]
fn test_dedicated_worker_imported_self_property_is_bare_global() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let fetched = Arc::new(Mutex::new(Vec::new()));
    let fetched_for_callback = fetched.clone();
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let page = args.first().map(String::as_str).unwrap_or("");
            let src = args.get(1).map(String::as_str).unwrap_or("");
            fetched_for_callback.lock().unwrap().push(format!("{page}|{src}"));
            match (page, src) {
                ("https://example.com/tests/page.html", "workers/imported-helper-worker.js") => {
                    "importScripts('helper.js');\
                     onmessage = function (event) { postMessage(imported_helper(event.data)); };"
                        .to_string()
                }
                ("https://example.com/tests/workers/imported-helper-worker.js", "helper.js") => {
                    "(function () {\
                       function imported_helper(value) { return value + 1; }\
                       self.imported_helper = imported_helper;\
                     })();"
                        .to_string()
                }
                _ => String::new(),
            }
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/tests/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__workerImportedHelperResult = 'pending';\
             var worker = new Worker('workers/imported-helper-worker.js');\
             worker.onmessage = function (event) { globalThis.__workerImportedHelperResult = event.data; };\
             worker.postMessage(41);",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__workerImportedHelperPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__workerImportedHelperResult)").unwrap().value,
        "42",
        "worker global properties assigned by imported scripts should resolve as bare globals"
    );
    assert_eq!(
        fetched.lock().unwrap().as_slice(),
        &[
            "https://example.com/tests/page.html|workers/imported-helper-worker.js".to_string(),
            "https://example.com/tests/workers/imported-helper-worker.js|helper.js".to_string(),
        ],
        "worker importScripts fetch should use the worker script URL as base"
    );
}

#[test]
fn test_iframe_inline_script_xhr_uses_iframe_window_location() {
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
        "<html><body><iframe src=\"resources/fetch-event-respond-with-argument-iframe.html\"></iframe></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/fetch-event-respond-with-argument.https.html"
            .to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_sw_observe_window_client",
        Box::new(|_args| r#"{"ok":true}"#.to_string()),
    );
    sandbox.register_callback(
        "__zw_sw_controller",
        Box::new(|args| {
            if args.first().map(String::as_str)
                == Some("https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r1","scriptURL":"https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            fetches_for_callback.lock().unwrap().push(format!(
                "{}|{}|{}",
                url,
                args.get(5).cloned().unwrap_or_default(),
                args.get(6).cloned().unwrap_or_default()
            ));
            if url.ends_with("fetch-event-respond-with-argument-iframe.html") {
                return "__zwfr:200\x1fOK\x1f\x1f<script>\
                  function fetch_url(url) {\
                    return new Promise(function(resolve, reject) {\
                      var request = new XMLHttpRequest();\
                      request.addEventListener('load', function() { resolve(); });\
                      request.addEventListener('error', function() { reject(); });\
                      request.open('GET', url);\
                      request.send();\
                    });\
                  }\
                  function make_test(testcase) {\
                    var name = testcase.name;\
                    return fetch_url(window.location.href + '?' + name).then(\
                      function() {\
                        if (testcase.expect_load) return Promise.resolve();\
                        return Promise.reject(new Error(name + ': expected network error but loaded'));\
                      },\
                      function() {\
                        if (!testcase.expect_load) return Promise.resolve();\
                        return Promise.reject(new Error(name + ': expected to load but got network error'));\
                      });\
                  }\
                  function run_tests() {\
                    Promise.all([\
                      { name: 'response-object', expect_load: true },\
                      { name: 'response-promise-object', expect_load: true },\
                      { name: 'other-value', expect_load: false }\
                    ].map(make_test)).then(function() {\
                      window.parent.notify_test_done('PASS');\
                    }).catch(function(error) {\
                      window.parent.notify_test_done('FAIL: ' + error.message);\
                    });\
                  }\
                  if (!navigator.serviceWorker.controller) \
                    window.parent.notify_test_done('FAIL: no controller'); \
                  else \
                    run_tests();\
                </script>".to_string();
            }
            if url.ends_with("?other-value") {
                "__zw_fetch_error:network".to_string()
            } else {
                "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
            }
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeDone = '';\
             globalThis.notify_test_done = function(result) { __iframeDone = String(result); };\
             var frame = document.querySelector('iframe');\
             var win = frame.contentWindow;\
             win.__zwRunInlineScripts();",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox
            .execute(
                "JSON.stringify((function(){\
                   var frame = document.querySelector('iframe');\
                   var win = frame && frame.contentWindow;\
                   return {\
                     hasFrame: !!frame,\
                     hasWin: !!win,\
                     runner: win && typeof win.__zwRunInlineScripts,\
                     href: win && win.location && win.location.href,\
                     controller: !!(win && win.navigator && win.navigator.serviceWorker && win.navigator.serviceWorker.controller)\
                   };\
                 })())",
            )
            .unwrap()
            .value,
        r#"{"hasFrame":true,"hasWin":true,"runner":"function","href":"https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html","controller":true}"#
    );
    assert_eq!(
        fetches.lock().unwrap().as_slice(),
        &[
            "https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html||".to_string(),
            "https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html?response-object|iframe:iframe|https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html".to_string(),
            "https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html?response-promise-object|iframe:iframe|https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html".to_string(),
            "https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html?other-value|iframe:iframe|https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-argument-iframe.html".to_string(),
        ]
    );
    assert_eq!(sandbox.execute("__iframeDone").unwrap().value, "PASS");
}
