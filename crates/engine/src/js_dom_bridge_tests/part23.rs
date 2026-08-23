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

#[test]
fn test_fetch_passes_request_credentials_to_host() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            calls_for_callback.lock().unwrap().push(format!(
                "{}|{}",
                args.get(2).cloned().unwrap_or_default(),
                args.get(9).cloned().unwrap_or_default()
            ));
            "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
        }),
    );

    sandbox
        .execute(
            "Promise.all([\
               fetch('https://example.com/default'),\
               fetch(new Request('https://example.com/request', { credentials: 'omit' })),\
               fetch('https://example.com/init', { credentials: 'include' })\
             ]).then(function() { globalThis.__credentialsDone = 'done'; });",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(sandbox.execute("String(globalThis.__credentialsDone)").unwrap().value, "done");
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[
            "https://example.com/default|same-origin".to_string(),
            "https://example.com/request|omit".to_string(),
            "https://example.com/init|include".to_string(),
        ]
    );
}

#[test]
fn test_iframe_service_worker_controller_post_message_transfers_object_port() {
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
        Arc::new(Mutex::new("<html><body><iframe src=\"resources/simple.html\"></iframe></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/fetch-event-respond-with-stops-propagation.https.html"
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
                == Some("https://wpt.test/service-workers/service-worker/resources/simple.html")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r1","scriptURL":"https://wpt.test/service-workers/service-worker/resources/fetch-event-respond-with-stops-propagation-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/simple.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );
    let posted = Arc::new(Mutex::new(Vec::new()));
    let posted_for_callback = posted.clone();
    let transferred_port_id = Arc::new(Mutex::new(None));
    let transferred_port_id_for_post = transferred_port_id.clone();
    sandbox.register_callback(
        "__zw_sw_post_message",
        Box::new(move |args| {
            let port_id = args
                .get(2)
                .map(|value| value.trim().trim_start_matches('[').trim_end_matches(']'))
                .and_then(|value| value.parse::<u64>().ok());
            if let Some(port_id) = port_id {
                *transferred_port_id_for_post.lock().unwrap() = Some(port_id);
            }
            posted_for_callback.lock().unwrap().push(format!(
                "{}|{}|{}|{}|{}",
                args.first().cloned().unwrap_or_default(),
                args.get(1).cloned().unwrap_or_default(),
                args.get(2).cloned().unwrap_or_default(),
                args.get(3).cloned().unwrap_or_default(),
                args.get(4).cloned().unwrap_or_default()
            ));
            r#"{"ok":true}"#.to_string()
        }),
    );
    let transferred_port_id_for_poll = transferred_port_id.clone();
    sandbox.register_callback(
        "__zw_sw_client_messages",
        Box::new(move |args| {
            let after_sequence = args.get(1).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
            let port_id = *transferred_port_id_for_poll.lock().unwrap();
            if let (0, Some(port_id)) = (after_sequence, port_id) {
                return format!(
                    r#"{{"ok":true,"latestSequence":1,"messages":[{{"data":{{"result":"PASS"}},"portId":{port_id},"transferredPortIds":[],"dataPortIndex":null,"targetClientId":null}}]}}"#
                );
            }
            format!(r#"{{"ok":true,"latestSequence":{after_sequence},"messages":[]}}"#)
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeControllerPortResult = 'pending';\
             var frame = document.querySelector('iframe');\
             var win = frame.contentWindow;\
             var worker = win.navigator.serviceWorker.controller;\
             var channel = new MessageChannel();\
             channel.port1.onmessage = function(event) {\
               globalThis.__iframeControllerPortResult = event.data.result;\
             };\
             worker.postMessage({port: channel.port2}, [channel.port2]);",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox
            .execute("String(globalThis.__iframeControllerPortResult)")
            .unwrap()
            .value,
        "PASS",
        "iframe ServiceWorker controller postMessage should poll messages for transferred ports"
    );
    assert_eq!(
        posted.lock().unwrap().as_slice(),
        &[r#"r1|{"port":{"__zwServiceWorkerTransferredPortIndex":0}}|[2]||"#.to_string()]
    );
}

#[test]
fn test_iframe_content_window_post_message_transfers_ports() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/iso-latin1-header.https.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/iso-latin1-header-iframe.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeMessageResult = 'pending';\
             var frame = document.createElement('iframe');\
             frame.src = 'resources/iso-latin1-header-iframe.html';\
             document.body.appendChild(frame);\
             var win = frame.contentWindow;\
             win.addEventListener('message', function(evt) {\
               evt.ports[0].postMessage({ result: evt.data.kind + ':' + evt.origin + ':' + (evt.source === window) });\
             });\
             var channel = new MessageChannel();\
             channel.port1.onmessage = function(evt) { globalThis.__iframeMessageResult = evt.data.result; };\
             win.postMessage({ kind: 'ping' }, 'https://wpt.test', [channel.port2]);",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__iframeMessageResult)").unwrap().value,
        "ping:https://wpt.test:true",
        "iframe contentWindow.postMessage should dispatch a message event with transferred ports"
    );
}

#[test]
fn test_iframe_content_window_cache_add_uses_iframe_fetch_context() {
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
        "<html><body><iframe src=\"resources/simple.html\"></iframe></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/fetch-event-within-sw.https.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_sw_observe_window_client",
        Box::new(|_args| r#"{"ok":true}"#.to_string()),
    );

    let fetches = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            fetches_for_callback.lock().unwrap().push(format!(
                "{}|{}|{}",
                args.get(2).cloned().unwrap_or_default(),
                args.get(5).cloned().unwrap_or_default(),
                args.get(6).cloned().unwrap_or_default()
            ));
            "__zwfr:200\x1fOK\x1f\x1fintercepted".to_string()
        }),
    );
    let cache_requests = Arc::new(Mutex::new(Vec::new()));
    let cache_requests_for_callback = cache_requests.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            cache_requests_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"test","cache_id":17}"#.to_string();
            }
            if request.contains(r#""op":"put""#)
                && request.contains(r#""url":"https://wpt.test/service-workers/service-worker/resources/sample.txt""#)
                && request.contains(r#""body":"intercepted""#)
            {
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            if request.contains(r#""op":"match""#) {
                return "__zw_cache_ok:{\"response\":\"__zwcr2:200\\u001fOK\\u001fbasic\\u001fhttps://wpt.test/service-workers/service-worker/resources/sample.txt\\u001f\\u001fintercepted\"}".to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeCacheAdd = 'pending';\
             var frame = document.querySelector('iframe');\
             var cache;\
             frame.contentWindow.caches.match().then(function () {\
               globalThis.__iframeCacheStorageMatchMissing = 'resolved';\
             }, function (error) {\
               globalThis.__iframeCacheStorageMatchMissing = error && error.name;\
             });\
             frame.contentWindow.caches.open('test').then(function (opened) {\
               cache = opened;\
               return cache.add('sample.txt');\
             }).then(function () {\
               return cache.match('sample.txt');\
             }).then(function (response) {\
               return response.text();\
             }).then(function (body) {\
               globalThis.__iframeCacheAdd = body;\
             }, function (error) {\
               globalThis.__iframeCacheAdd = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..12 {
        sandbox.execute(&format!("globalThis.__iframeCacheAddPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__iframeCacheAdd)").unwrap().value,
        "intercepted",
        "iframe contentWindow.caches should expose CacheStorage and cache.add should round-trip"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__iframeCacheStorageMatchMissing)")
            .unwrap()
            .value,
        "TypeError",
        "iframe CacheStorage.match should preserve the main CacheStorage missing-request validation"
    );
    assert_eq!(
        fetches.lock().unwrap().as_slice(),
        &[
            "https://wpt.test/service-workers/service-worker/resources/simple.html||".to_string(),
            "https://wpt.test/service-workers/service-worker/resources/sample.txt|iframe:iframe|https://wpt.test/service-workers/service-worker/resources/simple.html".to_string(),
        ],
        "cache.add should fetch relative to the iframe document URL with the iframe SW client metadata"
    );
    assert!(
        cache_requests
            .lock()
            .unwrap()
            .iter()
            .any(|request| request.contains(r#""op":"put""#)
                && request.contains(r#""url":"https://wpt.test/service-workers/service-worker/resources/sample.txt""#)),
        "Cache.put should store the iframe-resolved request URL"
    );
}

#[test]
fn test_iframe_sandbox_without_same_origin_denies_cache_storage() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/cache-storage/window/sandboxed-iframes.https.html"
            .to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).cloned().unwrap_or_default();
            if url.ends_with("/resources/iframe.html") {
                return "__zwfr:200\x1fOK\x1f\x1f<html><body></body></html>".to_string();
            }
            "__zwfr:200\x1fOK\x1f\x1f".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(|args| {
            let request = args.first().cloned().unwrap_or_default();
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"test","cache_id":23}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeSandboxCache = 'pending';\
             var allowed = document.createElement('iframe');\
             allowed.sandbox = 'allow-scripts allow-same-origin';\
             allowed.src = '../resources/iframe.html';\
             document.documentElement.appendChild(allowed);\
             var denied = document.createElement('iframe');\
             denied.sandbox = 'allow-scripts';\
             denied.src = '../resources/iframe.html';\
             document.documentElement.appendChild(denied);\
             Promise.all([\
               allowed.contentWindow.caches.open('allowed').then(function () { return 'allowed'; }, function (e) { return 'allowed-error:' + e.name; }),\
               denied.contentWindow.caches.open('denied').then(function () { return 'denied-opened'; }, function (e) { return 'denied-error:' + e.name; })\
             ]).then(function (results) { globalThis.__iframeSandboxCache = results.join('|'); });",
        )
        .unwrap();
    for i in 0..12 {
        sandbox.execute(&format!("globalThis.__iframeSandboxCachePump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__iframeSandboxCache)").unwrap().value,
        "allowed|denied-error:SecurityError",
        "sandboxed iframe without allow-same-origin should reject CacheStorage access"
    );
}

#[test]
fn test_cache_api_storage_buckets_namespace_and_delete() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) && request.contains("attachments") {
                let name = if request.contains("__zw_storage_bucket__0069006e0062006f0078:attachments") {
                    "__zw_storage_bucket__0069006e0062006f0078:attachments"
                } else if request.contains("__zw_storage_bucket__006400720061006600740073:attachments") {
                    "__zw_storage_bucket__006400720061006600740073:attachments"
                } else {
                    return "__zw_cache_error:unexpected open".to_string();
                };
                return format!(r#"__zw_cache_ok:{{"name":"{name}"}}"#);
            }
            if request.contains(r#""op":"put""#) {
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            if request.contains(r#""op":"keys""#) {
                return r#"__zw_cache_ok:{"keys":["__zw_storage_bucket__0069006e0062006f0078:attachments","__zw_storage_bucket__006400720061006600740073:attachments","global"]}"#.to_string();
            }
            if request.contains(r#""op":"delete""#) {
                return r#"__zw_cache_ok:{"deleted":true}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__bucketCacheResult = 'pending';\
             Promise.all([\
               navigator.storageBuckets.open('inbox'),\
               navigator.storageBuckets.open('drafts')\
             ]).then(function (buckets) {\
               return Promise.all([\
                 buckets[0].caches.open('attachments'),\
                 buckets[1].caches.open('attachments')\
               ]).then(function (caches) {\
                 return caches[0].put('receipt.txt', new Response('bread')).then(function () {\
                   return caches[1].put('receipt.txt', new Response('eggs'));\
                 }).then(function () {\
                   return Promise.all([buckets[0].caches.keys(), buckets[1].caches.keys()]);\
                 }).then(function (keys) {\
                   return navigator.storageBuckets.delete('inbox').then(function (deleted) {\
                     return buckets[0].caches.open('attachments').then(function () {\
                       return 'old-open-resolved';\
                     }, function (error) {\
                       return navigator.storageBuckets.open('inbox').then(function (newInbox) {\
                         return newInbox.caches.open('attachments').then(function () {\
                           return buckets[0].caches.has('attachments').then(function () {\
                             return 'old-has-resolved';\
                           }, function (oldError) {\
                             return [\
                               keys[0].join(','),\
                               keys[1].join(','),\
                               String(deleted),\
                               error.name,\
                               newInbox.name,\
                               oldError.name\
                             ].join('|');\
                           });\
                         });\
                       });\
                     });\
                   });\
                 });\
               });\
             }).then(function (result) {\
               globalThis.__bucketCacheResult = result;\
             }, function (error) {\
               globalThis.__bucketCacheResult = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..12 {
        sandbox.execute(&format!("globalThis.__bucketCachePump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__bucketCacheResult)").unwrap().value,
        "attachments|attachments|true|UnknownError|inbox|UnknownError",
        "bucket cache result mismatch; host calls: {:?}",
        calls.lock().unwrap()
    );
    let calls = calls.lock().unwrap();
    assert!(calls.iter().any(|request| {
        request.contains(r#""op":"put""#)
            && request.contains("__zw_storage_bucket__0069006e0062006f0078:attachments")
            && request.contains(r#""body":"bread""#)
    }));
    assert!(calls.iter().any(|request| {
        request.contains(r#""op":"put""#)
            && request.contains("__zw_storage_bucket__006400720061006600740073:attachments")
            && request.contains(r#""body":"eggs""#)
    }));
    assert!(calls.iter().any(|request| {
        request.contains(r#""op":"delete""#)
            && request.contains("__zw_storage_bucket__0069006e0062006f0078:attachments")
    }));
}

// R175（js-dom M4 DBG）：fragment 子串直发后的合成 body 过滤——host 侧
// `filter_synthetic` 对无 `<body` 开标签的源串必须剔合成 body 命中
//（WPT Fragment "Type selector, matching body element" expect 0）。
#[test]
fn zz_r175_frag_body_filter() {
    let src = r#"<div id="root"><div id="universal"><p id="universal-p1">x</p></div></div>"#;
    let out = super::parse_html_element_json_full(src, "body", true, None, true);
    assert_eq!(out, "[]", "synthetic body must be filtered: {out}");
    let out2 = super::parse_html_element_json_full(src, "div", true, None, true);
    assert!(out2.contains("root"), "div should match: {out2}");
}
