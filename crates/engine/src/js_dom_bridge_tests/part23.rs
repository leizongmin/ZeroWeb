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
fn test_iframe_inline_script_function_is_exposed_on_content_window() {
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
        "<html><body><iframe src=\"resources/unregister-controller-page.html\"></iframe></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/unregister-controller.https.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/unregister-controller-page.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script>\
                  function fetch_url(url) {\
                    return Promise.resolve('loaded:' + url);\
                  }\
                </script>"
                    .to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeInlineFunction = 'pending';\
             var win = document.querySelector('iframe').contentWindow;\
             win.__zwRunInlineScripts();\
             win.fetch_url('simple.txt').then(function(value) {\
               globalThis.__iframeInlineFunction = value;\
             }, function(error) {\
               globalThis.__iframeInlineFunction = 'error:' + String(error && error.message || error);\
             });",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("String(globalThis.__iframeInlineFunction)").unwrap().value,
        "loaded:simple.txt",
        "iframe inline function declarations should be exposed as contentWindow properties"
    );
}

#[test]
fn test_iframe_service_worker_controller_identity_is_stable() {
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
        "<html><body><iframe src=\"resources/unregister-controller-page.html?load-before-unregister\"></iframe></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/unregister-controller.https.html".to_string(),
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
                == Some("https://wpt.test/service-workers/service-worker/resources/unregister-controller-page.html?load-before-unregister")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r1","scriptURL":"https://wpt.test/service-workers/service-worker/resources/simple-intercept-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.contains("resources/unregister-controller-page.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );

    assert_eq!(
        sandbox
            .execute(
                "var win = document.querySelector('iframe').contentWindow;\
                 var first = win.navigator.serviceWorker.controller;\
                 var second = win.navigator.serviceWorker.controller;\
                 String(first === second && first instanceof win.ServiceWorker &&\
                   first.scriptURL.endsWith('/resources/simple-intercept-worker.js') &&\
                   second.state === 'activated');",
            )
            .unwrap()
            .value,
        "true",
        "iframe ServiceWorkerContainer.controller should preserve object identity for the same worker"
    );
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
fn test_iframe_service_worker_message_poll_refreshes_controller() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><iframe src=\"resources/blank.html\"></iframe></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/claim-fetch.https.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.register_callback(
        "__zw_sw_observe_window_client",
        Box::new(|_args| r#"{"ok":true}"#.to_string()),
    );
    let claimed = Arc::new(AtomicBool::new(false));
    let claimed_for_controller = claimed.clone();
    sandbox.register_callback(
        "__zw_sw_controller",
        Box::new(move |args| {
            if claimed_for_controller.load(Ordering::Relaxed)
                && args.first().map(String::as_str)
                    == Some("https://wpt.test/service-workers/service-worker/resources/blank.html")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r1","scriptURL":"https://wpt.test/service-workers/service-worker/resources/claim-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/blank.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_sw_post_message",
        Box::new(|_args| r#"{"ok":true}"#.to_string()),
    );
    let claimed_for_messages = claimed.clone();
    sandbox.register_callback(
        "__zw_sw_client_messages",
        Box::new(move |args| {
            let after_sequence = args.get(1).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
            if after_sequence == 0 {
                claimed_for_messages.store(true, Ordering::Relaxed);
                return r#"{"ok":true,"latestSequence":1,"messages":[{"data":"PASS","portId":2,"transferredPortIds":[],"dataPortIndex":null,"targetClientId":null}]}"#.to_string();
            }
            format!(r#"{{"ok":true,"latestSequence":{after_sequence},"messages":[]}}"#)
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeClaimControllerChange = 'pending';\
             var frame = document.querySelector('iframe');\
             var win = frame.contentWindow;\
             win.navigator.serviceWorker.oncontrollerchange = function() {\
               var controller = win.navigator.serviceWorker.controller;\
               globalThis.__iframeClaimControllerChange = String(!!controller &&\
                 controller.scriptURL.endsWith('/resources/claim-worker.js') &&\
                 controller.state === 'activated');\
             };\
             var worker = Object.create(ServiceWorker.prototype);\
             worker._id = 'r1';\
             worker.scriptURL = 'https://wpt.test/service-workers/service-worker/resources/claim-worker.js';\
             worker.state = 'activated';\
             __zwInitServiceWorkerMessageBridge(worker, {\
               id: 'iframe:iframe',\
               url: 'https://wpt.test/service-workers/service-worker/resources/blank.html',\
               container: win.navigator.serviceWorker\
             });\
             worker.postMessage('claim');",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox
            .execute("String(globalThis.__iframeClaimControllerChange)")
            .unwrap()
            .value,
        "true",
        "iframe ServiceWorker message polling should refresh controller state"
    );
}

#[test]
fn test_iframe_service_worker_container_constructor_brand() {
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
        Arc::new(Mutex::new("<html><body><iframe src=\"resources/blank.html\"></iframe></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/service-worker/skip-waiting-using-registration.https.html"
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
                == Some("https://wpt.test/service-workers/service-worker/resources/blank.html")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r2","scriptURL":"https://wpt.test/service-workers/service-worker/resources/skip-waiting-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(|args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/blank.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                "__zw_fetch_error:not found".to_string()
            }
        }),
    );

    assert_eq!(
        sandbox
            .execute(
                "var win = document.querySelector('iframe').contentWindow;\
                 var eventSeen = null;\
                 win.navigator.serviceWorker.addEventListener('controllerchange', function(event) {\
                   eventSeen = event;\
                 });\
                 win.navigator.serviceWorker.__zwRefreshServiceWorkerController();\
                 String(typeof win.ServiceWorkerContainer === 'function') + '|' +\
                   String(win.navigator.serviceWorker instanceof win.ServiceWorkerContainer) + '|' +\
                   String(win.navigator.serviceWorker instanceof win.EventTarget) + '|' +\
                   String(eventSeen && eventSeen.target instanceof win.ServiceWorkerContainer);",
            )
            .unwrap()
            .value,
        "true|true|true|true",
        "iframe ServiceWorkerContainer constructor should brand container and controllerchange event target"
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
fn test_iframe_service_worker_messages_dispatch_to_iframe_container() {
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
        Arc::new(Mutex::new("<html><body><iframe src=\"../resources/credentials-iframe.html\"></iframe></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/cache-storage/serviceworker/credentials.https.html".to_string(),
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
                == Some("https://wpt.test/service-workers/cache-storage/resources/credentials-iframe.html")
                && args.get(1).map(String::as_str) == Some("iframe:iframe")
            {
                r#"{"ok":true,"controller":{"id":"r1","scriptURL":"https://wpt.test/service-workers/cache-storage/resources/credentials-worker.js","state":"activated"}}"#.to_string()
            } else {
                r#"{"ok":true,"controller":null}"#.to_string()
            }
        }),
    );
    let posted = Arc::new(Mutex::new(Vec::new()));
    let posted_for_callback = posted.clone();
    sandbox.register_callback(
        "__zw_sw_post_message",
        Box::new(move |args| {
            posted_for_callback.lock().unwrap().push(format!(
                "{}|{}|{}|{}",
                args.get(5).cloned().unwrap_or_default(),
                args.get(6).cloned().unwrap_or_default(),
                args.first().cloned().unwrap_or_default(),
                args.get(1).cloned().unwrap_or_default()
            ));
            r#"{"ok":true}"#.to_string()
        }),
    );
    let xhr_fetches = Arc::new(Mutex::new(Vec::new()));
    let xhr_fetches_for_callback = xhr_fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("resources/credentials-iframe.html") {
                "__zwfr:200\x1fOK\x1f\x1f<script></script>".to_string()
            } else {
                xhr_fetches_for_callback.lock().unwrap().push(format!(
                    "{}|{}|{}",
                    args.get(2).cloned().unwrap_or_default(),
                    args.get(5).cloned().unwrap_or_default(),
                    args.get(6).cloned().unwrap_or_default()
                ));
                "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
            }
        }),
    );
    sandbox.register_callback(
        "__zw_sw_client_messages",
        Box::new(|args| {
            if args.get(2).map(String::as_str) == Some("iframe:iframe") {
                r#"{"ok":true,"latestSequence":1,"messages":[{"data":["https://wpt.test/file.txt"],"portId":null,"transferredPortIds":[],"dataPortIndex":null,"targetClientId":"iframe:iframe"}]}"#.to_string()
            } else {
                let after_sequence = args.get(1).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
                format!(r#"{{"ok":true,"latestSequence":{after_sequence},"messages":[]}}"#)
            }
        }),
    );

    sandbox
        .execute(
            "globalThis.__iframeSwMessageResult = 'pending';\
             globalThis.__iframeXhrDone = 'pending';\
             var frame = document.querySelector('iframe');\
             var win = frame.contentWindow;\
             win.navigator.serviceWorker.onmessage = function(event) {\
               globalThis.__iframeSwMessageResult = event.data[0];\
             };\
             var xhr = new win.XMLHttpRequest();\
             xhr.open('GET', 'file.txt', true, 'aa', 'bb');\
             xhr.onreadystatechange = function() {\
               if (xhr.readyState === win.XMLHttpRequest.DONE) globalThis.__iframeXhrDone = xhr.responseText;\
             };\
             xhr.send();\
             win.navigator.serviceWorker.controller.postMessage('keys');",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox
            .execute("String(globalThis.__iframeSwMessageResult)")
            .unwrap()
            .value,
        "https://wpt.test/file.txt",
        "iframe ServiceWorkerContainer should poll and dispatch messages for the iframe client"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__iframeXhrDone)").unwrap().value,
        "body",
        "iframe XMLHttpRequest should expose DONE and complete after a credentialed open()"
    );
    assert!(
        xhr_fetches.lock().unwrap().contains(
            &"https://aa:bb@wpt.test/service-workers/cache-storage/resources/file.txt|iframe:iframe|https://wpt.test/service-workers/cache-storage/resources/credentials-iframe.html".to_string()
        ),
        "iframe XMLHttpRequest should preserve username/password in the request URL"
    );
    assert_eq!(
        posted.lock().unwrap().as_slice(),
        &[r#"iframe:iframe|https://wpt.test/service-workers/cache-storage/resources/credentials-iframe.html|r1|"keys""#.to_string()]
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


// R225（js-dom M4）：Range.insertNode 三修复回归——① doc 级 insertBefore 先从原父
// 摘除（spec pre-insert adopt 步骤；PI/comment 移位不重复入列）；② 空片段不占位
// （_zwMEl/工厂元素 insertBefore/appendChild 展平 + syncEnd 的 handle-aware
// indexOf + fragment newOffset 语义）；③ collapsed 插入后 endOffset 同步到 sim 的
// newOffset。WPT Range-insertNode 25/26/29/31,16/18 + 0/4/8/10/15,20 共 +21P。
#[test]
fn r225_insert_node_doc_order_and_fragment_flatten() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
// 复刻 common.js setupRangeTests 的 xmlDoc 段 + WPT 31,16 形态
var xmlDoctype = document.implementation.createDocumentType('qorflesnorf', 'abcde', 'x');
var xmlDoc = document.implementation.createDocument(null, null, xmlDoctype);
var processingInstruction = xmlDoc.createProcessingInstruction('somePI', 'data');
var xmlComment = xmlDoc.createComment('comment');
var xmlElement = xmlDoc.createElement('igiveuponcreativenames');
xmlElement.appendChild(xmlDoc.createTextNode('do re mi'));
xmlDoc.appendChild(xmlElement);
xmlDoc.appendChild(processingInstruction);
xmlDoc.appendChild(xmlComment);
function dumpKids(d) {
  var out = [];
  for (var i = 0; i < d.childNodes.length; i++) {
    out.push(d.childNodes[i].nodeType + ':' + String(d.childNodes[i].nodeName).slice(0, 6));
  }
  return out.join(',');
}
var r = xmlDoc.createRange();
r.setStart(xmlDoc, 1);
r.setEnd(xmlComment, 0);
r.insertNode(processingInstruction);
var xmlAfter = dumpKids(xmlDoc);
// foreignDoc 域 docfrag：折叠 text 插入空 df → endOffset 1（sim newOffset 语义）
var foreignDoc = document.implementation.createHTMLDocument('');
var fb = foreignDoc.body;
var t0 = foreignDoc.createTextNode('x');
fb.appendChild(t0);
var rd = foreignDoc.createRange();
rd.setStart(t0, 0);
rd.setEnd(t0, 0);
rd.insertNode(foreignDoc.createDocumentFragment());
globalThis.__r225out = [
  'xml:' + xmlAfter,
  'df-endoff:' + rd.endOffset,
  'pi-once:' + (xmlDoc.childNodes.filter(function (c) { return c === processingInstruction; }).length),
].join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r225out").unwrap().value;
    assert_eq!(
        out,
        "xml:10:qorfle,7:somePI,1:igiveu,8:#comme|df-endoff:1|pi-once:1",
        "R225 doc 级 insertBefore 摘除移位（PI 单次）+ 空 docfrag endOffset 同步"
    );
}

// R226（js-dom M4）：`_zwMEl` insertBefore 的 **ref 位先取后摘**——spec
// `concept-node-pre-insert` 的 referenceNode index 在 adopt 摘除之前固定；旧版先
// remove 再 indexOf(ref)，c===ref 自引用形态（WPT Range-insertNode 30,4 的
// foreignDoc.body[0] === node）detach 后 ref miss 落尾部。28,0 形态（工厂 div
// off 0 自引用）同时回归。整文件 1840P / 0F（100%）。
#[test]
fn r226_self_ref_insert_before_keeps_position() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
// 30,4 形态：foreignDoc.body 折叠 off 0，node=body[0]=foreignPara1
var foreignDoc = document.implementation.createHTMLDocument('');
var fp1 = foreignDoc.createElement('p');
fp1.appendChild(foreignDoc.createTextNode('Efghijkl'));
var fp2 = foreignDoc.createElement('p');
fp2.appendChild(foreignDoc.createTextNode('Mnopqrst'));
foreignDoc.body.appendChild(fp1);
foreignDoc.body.appendChild(fp2);
var ftn = foreignDoc.createTextNode('I admit that I harbor doubts');
foreignDoc.body.appendChild(ftn);
var r30 = foreignDoc.createRange();
r30.setStart(foreignDoc.body, 0);
r30.setEnd(ftn, 0);
r30.insertNode(fp1);
var bodyKids = [];
for (var i = 0; i < foreignDoc.body.childNodes.length; i++) {
  var c = foreignDoc.body.childNodes[i];
  bodyKids.push(c.nodeType + ':' + String(c.textContent || '').slice(0, 4));
}
// 28,0 形态：工厂 div 容器 off 0，node=div[0]=p0
var td = document.createElement('div');
var ps = [];
for (var k = 0; k < 3; k++) {
  ps.push(document.createElement('p'));
  ps[k].appendChild(document.createTextNode('P' + k));
  td.appendChild(ps[k]);
}
var cm = document.createComment('c');
td.appendChild(cm);
var r28 = document.createRange();
r28.setStart(td, 0);
r28.setEnd(cm, 0);
r28.insertNode(ps[0]);
var tdKids = [];
for (var j = 0; j < td.childNodes.length; j++) {
  var d = td.childNodes[j];
  tdKids.push(d.nodeType + ':' + String(d.textContent || d.data || '').slice(0, 3));
}
globalThis.__r226out = [
  'body:' + bodyKids.join(','),
  'td:' + tdKids.join(','),
].join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r226out").unwrap().value;
    assert_eq!(
        out,
        "body:1:Efgh,1:Mnop,3:I ad|td:1:P0,1:P1,1:P2,8:c",
        "R226 自引用 insertBefore 的 ref 位先取后摘（fp1 保持首位 + p0 不动）"
    );
}

// R228（js-dom M4）：detached 同节点 CharData（comment/PI/text）区间 surround——
// extractContents 的 R211 分支放宽 parentNode 守卫（同节点中段切片 + deleteData +
// collapse 到 (容器, startOffset)），surroundContents 的 _r212 门同款放宽 +
// insertNode 的 HRE 不再吞（sim 序：extract 变更树后抛）。
// WPT Range-surroundContents 35–38,x「must be thrown + Stuwxyz」族 +50P。
#[test]
fn r228_detached_chardata_interval_surround() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var dc = document.createComment('Stuvwxyz');
var r = document.createRange();
r.setStart(dc, 3);
r.setEnd(dc, 4);
var p = document.createElement('p');
var threwName = 'none';
try { r.surroundContents(p); } catch (e) { threwName = (e && e.name) || String(e); }
// R229：同节点 collapse 到 (容器, startOffset)（sim 的 isAncestorContainer self
// 分支）——PI 同节点区间 [0,4] extract 后 startOffset 保持 0 非 (父, idx+1)。
var xd = document.implementation.createDocument(null, null, null);
var pi2 = xd.createProcessingInstruction('t', 'abcdefgh');
var rp = xd.createRange();
rp.setStart(pi2, 0);
rp.setEnd(pi2, 4);
var pe = document.createElement('p');
var pThrew = 'none';
try { rp.surroundContents(pe); } catch (e) { pThrew = (e && e.name) || String(e); }
// R229：leaf newParent（Text）对 comment 容器先 extract（data 切片）再抛 HRE。
var dc2 = document.createComment('Stuvwxyz');
var rt = document.createRange();
rt.setStart(dc2, 3);
rt.setEnd(dc2, 4);
var tThrew = 'none';
try { rt.surroundContents(document.createTextNode('z')); } catch (e) { tThrew = (e && e.name) || String(e); }
// R230：Text 同节点容器的 leaf-newParent（Text 型）也先 extract（源 text 削为
// 前缀 "Op"）再 insertNode 再抛 HRE（sim 序步骤 3-5；旧版源保留 "Opqrstuv"）。
var dpara = document.createElement('p');
var dtext = document.createTextNode('Opqrstuv');
dpara.appendChild(dtext);
var r9 = document.createRange();
r9.setStart(dtext, 2);
r9.setEnd(dtext, 8);
var t9 = 'none';
try { r9.surroundContents(document.createTextNode('z')); } catch (e) { t9 = (e && e.name) || String(e); }
globalThis.__r228out = [
  'data:' + dc.data,
  'threw:' + threwName,
  'pi-so:' + rp.startOffset + ',pi-threw:' + pThrew,
  'leaf-data:' + dc2.data + ',leaf-threw:' + tThrew,
  'r230:' + dtext.data + ',' + t9,
  'r231:' + r9.startOffset + ',' + r9.endOffset,
].join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r228out").unwrap().value;
    assert_eq!(
        out,
        "data:Stuwxyz|threw:HierarchyRequestError|pi-so:0,pi-threw:HierarchyRequestError|leaf-data:Stuwxyz,leaf-threw:HierarchyRequestError|r230:Op,HierarchyRequestError|r231:2,2",
        "R228/R229 detached comment 区间 surround：extract 切片 + HRE 上抛 + 同节点 collapse (容器, startOffset) + leaf-newParent 先 extract 再抛（R260 更新：deleteData 现按 spec concept-node-replace-data 调整 live-range 边界，(2→8) 折到 (2→2)——旧断言 2,8 是无调整机制的观测，WPT Range-extractContents 以折叠为准）"
    );
}

#[test]
fn r234_dynamic_document_element_and_extract_semantics() {
    // R234（js-dom M4）：三件断言——① iframe doc 的 documentElement 动态 getter
    // （restoreIframe 摘除工厂 docEl + appendChild 克隆后，documentElement 读到
    // 克隆子树而非脱离的空壳工厂 docEl——surround 12–14,x 的 cDP 108F 簇根因）；
    // ② 跨容器提取塌缩（[docEl,1,body,0] 类 extract 后 start/end 同容器——
    // harness「must always be the same」断言）；    // ③ plain 子提取摘除后的
    // 无父登记（摘除原件 parentNode 记到 fragment，不成为第二棵无根树）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// ① 动态 documentElement：iframe doc（no-src 退化路径）append 克隆 docEl 后
//    documentElement 读到该克隆（旧固定闭包返脱离空壳）。
var ifr = document.createElement('iframe');
document.body.appendChild(ifr);
var idoc = ifr.contentDocument;
var refDoc = document.implementation.createHTMLDocument('');
var cloneRoot = refDoc.documentElement.cloneNode(true);
// restoreIframe 等价：先清 doc 现有非 doctype 首末子，再 append 克隆。
while (idoc.firstChild && idoc.firstChild.nodeType !== 10) { idoc.removeChild(idoc.firstChild); }
while (idoc.lastChild && idoc.lastChild.nodeType !== 10) { idoc.removeChild(idoc.lastChild); }
idoc.appendChild(cloneRoot);
out.push('dyn:' + (idoc.documentElement === cloneRoot));
out.push('cdp:' + (typeof cloneRoot.compareDocumentPosition === 'function'));
// ② 跨容器提取塌缩（detached doc 克隆树同款形态）：docEl(1) → body(0) 提取后同容器。
var r49 = refDoc.createRange();
r49.setStart(refDoc.documentElement, 1);
r49.setEnd(refDoc.body, 0);
var frag49 = r49.extractContents();
out.push('same:' + (r49.startContainer === r49.endContainer));
// ③ plain 子提取的无父登记：docEl(0..1)（head）提取后摘除原件非无根。
var r12 = refDoc.createRange();
r12.setStart(refDoc.documentElement, 0);
r12.setEnd(refDoc.documentElement, 1);
var frag12 = r12.extractContents();
var headNode = frag12.childNodes.length ? frag12.childNodes[0] : null;
out.push('rooted:' + (headNode == null || headNode.parentNode != null));
globalThis.__r234out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r234out").unwrap().value;
    assert_eq!(
        out, "dyn:true|cdp:true|same:true|rooted:true",
        "R234 动态 documentElement getter + 跨容器提取塌缩 + plain 子提取无父登记"
    );
}

#[test]
fn r235_leaf_newparent_extract_first_variants() {
    // R235（js-dom M4）：leaf-newParent（Text/Comment）的「先 extract 再 insert 后抛
    // HRE」序扩展两形态——① 异节点同父 CharData 区间（WPT 6,x
    // `[paras[5].firstChild,2,paras[5].lastChild,4]`：extract 先削首尾切片）；
    // ② 元素容器含覆盖子（WPT 18,x `[paras[0],0,paras[0],1]`：extract 先移出
    // covered 子）。旧版两形态直接抛 HRE 使树保留区间原文。
    // https://dom.spec.whatwg.org/#dom-range-surroundcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// ① 异节点同父 CharData 区间：p 内 text "abcdefgh" 与 text "ijklmnop"，
//    range [t1,2, t2,3] + Text newParent → extract 先行（t1 削尾 "ab"、t2 削头
//    "lmnop"）再 HRE。
var p6 = document.createElement('p');
var t1 = document.createTextNode('abcdefgh');
var t2 = document.createTextNode('ijklmnop');
p6.appendChild(t1); p6.appendChild(t2);
document.body.appendChild(p6);
var r6 = document.createRange();
r6.setStart(t1, 2); r6.setEnd(t2, 3);
var threw6 = 'none';
try { r6.surroundContents(document.createTextNode('z')); } catch (e) { threw6 = (e && e.name) || String(e); }
out.push('xnode:' + t1.data + ',' + t2.data + ',' + threw6);
// ② 元素容器含覆盖子：p 内 text "Opqrstuv"，range [p,0,p,1] 整子区间 +
//    Text newParent → extract 先移出子（p 空）再 HRE。
var p18 = document.createElement('p');
var t18 = document.createTextNode('Opqrstuv');
p18.appendChild(t18);
document.body.appendChild(p18);
var r18 = document.createRange();
r18.setStart(p18, 0); r18.setEnd(p18, 1);
var threw18 = 'none';
try { r18.surroundContents(document.createTextNode('z')); } catch (e) { threw18 = (e && e.name) || String(e); }
out.push('elem:' + t18.parentNode + ',' + p18.childNodes.length + ',' + (p18.firstChild ? p18.firstChild.data : '?') + ',' + threw18);
globalThis.__r235out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r235out").unwrap().value;
    assert_eq!(
        out,
        "xnode:ab,lmnop,HierarchyRequestError|elem:null,1,z,HierarchyRequestError",
        "R235 leaf-newParent 两形态先 extract 后抛：异节点同父区间削首尾 + 元素容器移出 covered 子"
    );
}

#[test]
fn r236_ancestor_element_range_surround_extract() {
    // R236（js-dom M4）：sc 是 ec 的元素祖先容器且 ec 为直接 CharData 子的
    // surround/extract——extractContents 削 ec 头部（clone [0,eo) 入 frag +
    // deleteData，remainder 留树，range 塌缩到 (sc, so)）；surroundContents
    // leaf-newParent 先 extract 再 insert 后抛 HRE，元素 newParent 清子后
    // insert + appendChild(frag) + selectNode。
    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    // https://dom.spec.whatwg.org/#dom-range-surroundcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// extract：[p,0,t,7] → frag 头切片 "Abcdef"（前 7 code unit）+ t 削头 + 塌缩 (p,0)。
var p23 = document.createElement('p');
var t23 = document.createTextNode('Abcdefgh');
p23.appendChild(t23);
document.body.appendChild(p23);
var r23 = document.createRange();
r23.setStart(p23, 0); r23.setEnd(t23, 7);
var frag23 = r23.extractContents();
out.push('ex:' + frag23.childNodes[0].data + ',' + t23.data + ',' + r23.startContainer.nodeName + ',' + r23.startOffset + ',' + r23.endOffset);
// leaf newParent：先 extract 再 insert 后 HRE。
var p23b = document.createElement('p');
var t23b = document.createTextNode('Abcdefgh');
p23b.appendChild(t23b);
document.body.appendChild(p23b);
var r23b = document.createRange();
r23b.setStart(p23b, 0); r23b.setEnd(t23b, 7);
var threw = 'none';
try { r23b.surroundContents(document.createTextNode('z')); } catch (e) { threw = (e && e.name) || String(e); }
out.push('leaf:' + t23b.data + ',' + p23b.childNodes.length + ',' + threw);
// 元素 newParent：清子 + insert + appendChild(frag) + selectNode。
var wrap = document.createElement('span');
wrap.appendChild(document.createTextNode('old'));
var p23c = document.createElement('p');
var t23c = document.createTextNode('Abcdefgh');
p23c.appendChild(t23c);
document.body.appendChild(p23c);
var r23c = document.createRange();
r23c.setStart(p23c, 0); r23c.setEnd(t23c, 7);
r23c.surroundContents(wrap);
out.push('el:' + wrap.childNodes.length + ',' + (wrap.firstChild ? wrap.firstChild.data : '?') + ','
  + r23c.startContainer.nodeName + ',' + r23c.startOffset + ',' + r23c.endOffset);
globalThis.__r236out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r236out").unwrap().value;
    assert_eq!(
        out,
        "ex:Abcdefg,h,P,0,0|leaf:h,2,HierarchyRequestError|el:1,Abcdefg,P,0,1",
        "R236 祖先元素区间：extract 削头 + leaf 先 extract 后抛 + 元素 wrap 全序（清子/insert/select）"
    );
}

#[test]
fn r237_surround_path4_full_order() {
    // R237（js-dom M4）：surroundContents 路径 4（元素容器 covered 子 + 元素
    // newParent）收尾对齐 sim 全序——① 清 newParent 既有子（步骤 2）；② 插到
    // (容器, startOffset) 位而非 appendChild 末尾（步骤 4，探针实证 host docEl=
    // [BODY,P] vs sim [P{head},BODY]）；③ selectNode(newParent) 边界（步骤 6）。
    // https://dom.spec.whatwg.org/#dom-range-surroundcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// 容器 p 内 [text, span]，range 覆盖 span（offset 1..2）+ 元素 newParent（含旧子）。
var host237 = document.createElement('div');
var kid1 = document.createTextNode('keep');
var kid2 = document.createElement('span');
kid2.appendChild(document.createTextNode('inner'));
host237.appendChild(kid1); host237.appendChild(kid2);
document.body.appendChild(host237);
var np237 = document.createElement('em');
np237.appendChild(document.createTextNode('old'));
var r237 = document.createRange();
r237.setStart(host237, 1); r237.setEnd(host237, 2);
r237.surroundContents(np237);
out.push('order:' + host237.childNodes[0].nodeName + ',' + host237.childNodes[1].nodeName);
out.push('cleared:' + np237.childNodes.length + ',' + (np237.firstChild ? np237.firstChild.nodeName : '?'));
out.push('sel:' + r237.startOffset + ',' + r237.endOffset + ',' + (r237.startContainer === host237));
globalThis.__r237out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r237out").unwrap().value;
    assert_eq!(
        out, "order:#text,EM|cleared:1,SPAN|sel:1,2,true",
        "R237 路径 4：清 newParent 子 + 按位插入（非 appendChild）+ selectNode 边界"
    );
}

#[test]
fn r238_node_prototype_remove_generic() {
    // R238（js-dom M4）：`Node.prototype.remove` 泛型（spec `dom-child-remove`）——
    // Range extract/surround 的 covered 子摘除经 `typeof kids[j].remove === 'function'`
    // 守卫；iframe 子文档工厂文本无 remove 方法使守卫静默跳过、区间原文残留
    // （WPT 19,x 探针实证 detachedPara1=[Ä,Op] 双文本）。断言：工厂 text 的
    // remove() 经父 removeChild 生效 + 元素自有 remove 优先不受影响。
    // https://dom.spec.whatwg.org/#dom-child-remove
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// 工厂 text 的 remove()：经父 removeChild 生效。
var p238 = document.createElement('p');
var t238 = document.createTextNode('Opqrstuv');
p238.appendChild(t238);
document.body.appendChild(p238);
out.push('has-remove:' + (typeof t238.remove === 'function'));
t238.remove();
out.push('removed:' + p238.childNodes.length);
globalThis.__r238out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r238out").unwrap().value;
    assert_eq!(
        out, "has-remove:true|removed:0",
        "R238 Node.prototype.remove 泛型：工厂 text remove 生效 + 元素语义不变"
    );
}

#[test]
fn r239_partial_check_order_and_traversal() {
    // R239（js-dom M4）：surroundContents 的部分包含检查两件——① **先于 newParent
    // 类型检查**（common.js mySurroundContents 序：partial → INVALID_STATE 在
    // nodeType → INVALID_NODE_TYPE 前；WPT 20–22,x/29/31,x 的 Document/Doctype
    // 作 newParent 期望 INVALID_STATE）；② **nextNode 序遍历**（sim 的遍历原语
    // ——hasChildNodes→firstChild / 爬 nextSibling，盲区与 sim 对齐；旧 DFS 更
    // 完备使 24,x 反向翻转 12F）。
    // https://dom.spec.whatwg.org/#dom-range-surroundcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// ① 跨 text 边界部分包含 + Document newParent → INVALID_STATE（先于 nodeType）。
var wrap239 = document.createElement('div');
var p0 = document.createElement('p');
p0.appendChild(document.createTextNode('Ab'));
var p1 = document.createElement('p');
p1.appendChild(document.createTextNode('Cd'));
wrap239.appendChild(p0); wrap239.appendChild(p1);
document.body.appendChild(wrap239);
var r239 = document.createRange();
r239.setStart(p0.firstChild, 0); r239.setEnd(p1.firstChild, 0);
var threwA = 'none';
try { r239.surroundContents(document.implementation.createHTMLDocument('')); }
catch (e) { threwA = (e && e.name) || String(e); }
out.push('doc-np:' + threwA);
// ② 同形态 + 元素 newParent → 同样 INVALID_STATE（partial 命中 p0）。
var r239b = document.createRange();
r239b.setStart(p0.firstChild, 0); r239b.setEnd(p1.firstChild, 0);
var threwB = 'none';
try { r239b.surroundContents(document.createElement('em')); }
catch (e) { threwB = (e && e.name) || String(e); }
out.push('el-np:' + threwB);
globalThis.__r239out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r239out").unwrap().value;
    assert_eq!(
        out, "doc-np:InvalidStateError|el-np:InvalidStateError",
        "R239 部分包含检查：先于 newParent 类型检查 + nextNode 序遍历"
    );
}


#[test]
fn r240_ancestor_extract_moves_contained_children() {
    // R240（js-dom M4）：extractContents 祖先分支的 **contained 中段子移动**——
    // `[sc,so,ec,eo]`（ec 为 sc 直接 CharData 子）除 ec 削头外，sc 的
    // [so, ecIdx) 子本体移入 frag（spec containedChildren）。**快照后移动**：
    // appendChild 使 sc.childNodes 同步收缩，按下标迭代滑位会把 ec 本体也
    // 移入 frag（探针实证 ex-frag=[P,"oup?","bet s"] 错序含 remainder）。
    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
var d240 = document.createElement('div');
var p0 = document.createElement('p'); p0.appendChild(document.createTextNode('Ab'));
var p1 = document.createElement('p'); p1.appendChild(document.createTextNode('Ij'));
var cm = document.createComment('bet soup?');
d240.appendChild(p0); d240.appendChild(p1); d240.appendChild(cm);
document.body.appendChild(d240);
var r240 = document.createRange();
r240.setStart(d240, 0); r240.setEnd(cm, 5);
var frag = r240.extractContents();
function kids(n) {
  var s = [];
  for (var i = 0; i < n.childNodes.length; i++) {
    var k = n.childNodes[i];
    s.push((k.nodeName || k.nodeType) + (k.data != null ? '"' + String(k.data) + '"' : ''));
  }
  return s.join(',');
}
out.push('div:[' + kids(d240) + ']');
out.push('frag:[' + kids(frag) + ']');
globalThis.__r240out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r240out").unwrap().value;
    assert_eq!(
        out,
        "div:[#comment\"oup?\"]|frag:[P,P,#comment\"bet s\"]",
        "R240 祖先 extract：中段子本体移入 frag（快照防滑位）+ ec 削头 remainder 留树"
    );
}

#[test]
fn r241_extract_move_semantics_on_clone_append() {
    // R241（js-dom M4）：祖先 extract 的 contained 子移动 **move 语义兜底**——
    // WPT iframe 的 wrapper 域子对 fragment 的 appendChild 是 clone 语义
    // （R241-probe 实证树双份：newParent[拷贝…] + 原件残留），append 后原件
    // 仍在 sc.childNodes 时 removeChild 强制离场（spec containedChildren 是
    // move）。断言：extract 后 sc 只剩 ec remainder、frag 含中段子、无残留。
    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
// 主文档形态（append 是 move）——不因兜底误伤：无双份。
var d241 = document.createElement('div');
var p0 = document.createElement('p'); p0.appendChild(document.createTextNode('Ab'));
var p1 = document.createElement('p'); p1.appendChild(document.createTextNode('Ij'));
var cm = document.createComment('bet soup?');
d241.appendChild(p0); d241.appendChild(p1); d241.appendChild(cm);
document.body.appendChild(d241);
var r241 = document.createRange();
r241.setStart(d241, 0); r241.setEnd(cm, 5);
var frag = r241.extractContents();
function cnt(n, name) {
  var c = 0;
  for (var i = 0; i < n.childNodes.length; i++) {
    if ((n.childNodes[i].nodeName || '') === name) c++;
  }
  return c;
}
out.push('divP:' + cnt(d241, 'P') + ',divCm:' + cnt(d241, '#comment'));
out.push('fragP:' + cnt(frag, 'P') + ',fragCm:' + cnt(frag, '#comment'));
out.push('cm-data:' + cm.data);
globalThis.__r241out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r241out").unwrap().value;
    assert_eq!(
        out, "divP:0,divCm:1|fragP:2,fragCm:1|cm-data:oup?",
        "R241 move 兜底：frag 得中段子 + 头切片克隆，sc 只剩 remainder，无双份"
    );
}

#[test]
fn r242_element_ec_ancestor_extract() {
    // R242（js-dom M4）：**sc 元素祖先 + ec 元素直接子 + 双侧 clean 边界**（
    // `[testDiv,2,paras[4],1]` 形态）的 extractContents——contained 中段子本体
    // 移入 frag（R241 move 兜底）+ ec shallow clone 承接 [0,eo) 子 + 塌缩
    // (sc,so)。surround 全序断言由 WPT 24,x 承载（harness 克隆树的 partial
    // 检查遍历盲区使 surround 成功路径可观测；engine 沙箱树形态完好时 spec
    // 正确行为是 partial 检查先抛 InvalidStateError）。
    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
var d242 = document.createElement('div');
for (var q = 0; q < 4; q++) {
  var p = document.createElement('p');
  p.appendChild(document.createTextNode('T' + q + 'x'));
  p.appendChild(document.createTextNode('T' + q + 'y'));
  d242.appendChild(p);
}
document.body.appendChild(d242);
var r242 = document.createRange();
r242.setStart(d242, 1); r242.setEnd(d242.childNodes[3], 1);
var frag242 = r242.extractContents();
function sig242(n) {
  var s = [];
  for (var i = 0; i < n.childNodes.length; i++) {
    var k = n.childNodes[i];
    s.push((k.nodeName || k.nodeType) + (k.childNodes && k.childNodes.length ? '{' + k.childNodes.length + '}' : (k.data != null ? '"' + k.data + '"' : '')));
  }
  return s.join(',');
}
out.push('div:' + sig242(d242));
out.push('frag:' + sig242(frag242));
out.push('bound:' + r242.startOffset + ',' + r242.endOffset);
globalThis.__r242out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r242out").unwrap().value;
    assert_eq!(
        out,
        "div:P{2},P{1}|frag:P{2},P{2},P{1}|bound:1,1",
        "R242 元素 ec 祖先 extract：中段本体移动 + shallow clone 承接首子 + 塌缩"
    );
}

#[test]
fn r243_detached_doc_docelement_cdp_surface() {
    // R243（js-dom M4）：_makeDetachedDocument 内部 docEl/headEl/body 补
    // contains/cDP own-property（R235 首测净 -28 回退；R236–R242 全序分支落地后
    // 重评转正 +26——sim（common.js isAncestorContainer/getPosition）深入
    // foreignDoc/iframe doc 的合成树不再 TypeError）。
    // https://dom.spec.whatwg.org/#dom-node-contains
    // https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
var fd = document.implementation.createHTMLDocument('');
var de = fd.documentElement;
out.push('cdp:' + (typeof de.compareDocumentPosition === 'function'));
out.push('contains:' + (typeof de.contains === 'function'));
out.push('head:' + (typeof fd.head.compareDocumentPosition === 'function'));
out.push('body:' + (typeof fd.body.compareDocumentPosition === 'function'));
// 语义：body 在 docEl 内 → CONTAINS 位。
var Node243 = globalThis.Node;
var pos = de.compareDocumentPosition(fd.body);
out.push('pos-num:' + (typeof pos === 'number' && pos >= 0));
out.push('self:' + de.contains(de));
globalThis.__r243out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r243out").unwrap().value;
    assert_eq!(
        out, "cdp:true|contains:true|head:true|body:true|pos-num:true|self:true",
        "R243 detached-doc docEl/headEl/body 的 contains/cDP 方法面 + CONTAINS 位语义"
    );
}

#[test]
fn r244_doc_container_doctype_contained_surround_hre() {
    // R244（js-dom M4）：**contained children 含 DocumentType → HRE（树不变）**
    // ——spec `dom-range-extract-contents` 步骤 9（surroundContents 步骤 3 调
    // extractContents，HRE 原样上抛；common.js myExtractContents 的
    // containedChildren 循环同款）。WPT Range-surroundContents 25/26,x 元素
    // newParent 12F 簇：range 覆盖 doc 的 doctype 子（`[document,0,document,1/2]`）
    // 时 sim 步骤 3 先抛 HRE 而 host 对元素 newParent 无拦截（探针 24 行实证
    // j 非 6 元素族全部 NO_THROW）。跨容器 sideIdx 形态（sc/ec 深容器）+ 非含
    // doctype 形态（不误伤正常 wrap）双断言。
    // https://dom.spec.whatwg.org/#dom-range-extractcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var out = [];
globalThis.__r244err = '';
try {
// 形态 A（WPT 25,x 同构）：iframe doc = [doctype, html]，range [doc,0,doc,1]
// 覆盖 doctype 子 → 元素 newParent 抛 HRE 且 doc 子数不变（树不变）。
var ifr = document.createElement('iframe');
document.body.appendChild(ifr);
var idoc = ifr.contentDocument;
var refDoc = document.implementation.createHTMLDocument('');
var cloneRoot = refDoc.documentElement.cloneNode(true);
while (idoc.firstChild && idoc.firstChild.nodeType !== 10) { idoc.removeChild(idoc.firstChild); }
while (idoc.lastChild && idoc.lastChild.nodeType !== 10) { idoc.removeChild(idoc.lastChild); }
// engine sandbox 的 iframe doc 无内建 doctype——WPT harness 的 doc 形态
// [doctype, html] 需显式构造（implementation.createDocumentType）。
if (!idoc.firstChild || idoc.firstChild.nodeType !== 10) {
  try { idoc.insertBefore(idoc.implementation.createDocumentType('html', '', ''), idoc.firstChild); } catch (_e244dt) {}
  if ((!idoc.firstChild || idoc.firstChild.nodeType !== 10) && idoc.childNodes.length === 0) {
    idoc.appendChild(cloneRoot);
    // prepend doctype via insertBefore if it threw earlier
  }
}
if (!idoc.firstChild || idoc.firstChild.nodeType !== 10) {
  out.push('SHAPE:kids=' + idoc.childNodes.length);
} else {
  var hasHtml = false;
  for (var _q244 = 0; _q244 < idoc.childNodes.length; _q244++) {
    if (idoc.childNodes[_q244].nodeType === 1) hasHtml = true;
  }
  if (!hasHtml) idoc.appendChild(cloneRoot);
}
var rA = idoc.createRange();
rA.setStart(idoc, 0); rA.setEnd(idoc, 1);
var newP = idoc.createElement('p');
var threwA = 'none';
try { rA.surroundContents(newP); } catch (e) { threwA = (e && e.name) || String(e); }
out.push('A:' + threwA + ',kids:' + idoc.childNodes.length + ',dt-first:' + (idoc.firstChild && idoc.firstChild.nodeType));

// 形态 B（WPT 26,x 同构）：range [doc,0,doc,2] 覆盖 doctype+html → 同抛 HRE。
var rB = idoc.createRange();
rB.setStart(idoc, 0); rB.setEnd(idoc, 2);
var newP2 = idoc.createElement('span');
var threwB = 'none';
try { rB.surroundContents(newP2); } catch (e) { threwB = (e && e.name) || String(e); }
out.push('B:' + threwB + ',kids:' + idoc.childNodes.length);

// 形态 C（负例——不误伤正常 wrap）：doc 下 [doctype, html] 但 range 在 body
// 内部（不覆盖 doctype）→ wrap 正常完成不抛。
var body244 = idoc.body || (cloneRoot.getElementsByTagName('body')[0]);
var t1 = idoc.createTextNode('wx');
var t2 = idoc.createTextNode('yz');
body244.appendChild(t1); body244.appendChild(t2);
var rC = idoc.createRange();
var _baseC = body244.childNodes.length;
rC.setStart(body244, _baseC - 2);
rC.setEnd(body244, _baseC);
var wrapC = idoc.createElement('b');
var threwC = 'none';
try { rC.surroundContents(wrapC); } catch (e) { threwC = (e && e.name) || String(e); }
out.push('C:' + threwC + ',wrap-tag:' + (wrapC.parentNode ? wrapC.parentNode.nodeName : 'null') + ',wrap-n:' + wrapC.childNodes.length);

// 形态 D（跨容器 sideIdx）：sc 是 doc 直接子（html）内部深容器、ec 也是——
// doctype 不在任何边界子树 → 不抛（contained 判定不误伤跨容器）。
var rD = idoc.createRange();
rD.setStart(body244, 0); rD.setEnd(body244, 0);
var wrapD = idoc.createElement('i');
var threwD = 'none';
try { rD.surroundContents(wrapD); } catch (e) { threwD = (e && e.name) || String(e); }
out.push('D:' + threwD);
} catch (_e244) { globalThis.__r244err = (_e244 && _e244.name) + ': ' + (_e244 && _e244.message); }
globalThis.__r244out = out.join('|') + (globalThis.__r244err ? ' ||ERR[' + globalThis.__r244err + ']' : '');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r244out").unwrap().value;
    assert_eq!(
        out,
        "A:HierarchyRequestError,kids:2,dt-first:10|B:HierarchyRequestError,kids:2|C:none,wrap-tag:BODY,wrap-n:2|D:none",
        "R244 doc 容器 doctype contained surround：HRE 树不变 + 正常 wrap 零误伤 + 跨容器零误伤"
    );
}

#[test]
fn r245_factory_parent_move_semantics() {
    // R245（js-dom M4）：factory doc（implementation.createHTMLDocument）内部
    // headEl/body 的 parentNode 是 getter-only accessor——fragment appendChild 的
    // `c.parentNode = this` 赋值被吞（parentNode 恒指 factory docEl），后续把
    // HEAD 移入元素时 `_zwMEl.appendChild` 的「从旧父摘除」调 factory
    // docEl.removeChild(HEAD)（HEAD 已摘出）抛 NotFoundError 且未包裹直接传播
    // （micro-probe 实证：p1.appendChild(frag-with-HEAD) THREW NotFoundError）。
    // 修两件：① factory fragment appendChild 的父链经 defineProperty 强写
    // （getter-only 遮蔽）；② _zwMEl appendChild 的摘除 try/catch + 入树父链
    // defineProperty 强写（spec `concept-node-pre-insert` 的 adopt 摘除幂等语义）。
    // https://dom.spec.whatwg.org/#concept-node-pre-insert
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            r#"
var out = [];
var fd = document.implementation.createHTMLDocument('');
var bodyF = fd.body;
var fp1 = fd.createElement('p');
fp1.appendChild(fd.createTextNode('Efghijkl'));
bodyF.appendChild(fp1);
var deF = fd.documentElement;
var frag = fd.createDocumentFragment();
// HEAD 移入 frag（extract 的 contained-children 步同款）
frag.appendChild(deF.childNodes[0]);
out.push('headP:' + (frag.childNodes[0].parentNode && frag.childNodes[0].parentNode.nodeName));
// 再移入元素（surround 步骤 5 同款）——旧版在此抛 NotFoundError
try { fp1.appendChild(frag); out.push('move:ok:' + fp1.childNodes.length); }
catch (e) { out.push('move:THREW:' + ((e && e.name) || String(e))); }
// 17,4 形态端到端：docEl 区间 surround 同 doc 元素 newParent
var fd2 = document.implementation.createHTMLDocument('');
var b2 = fd2.body;
var q2 = fd2.createElement('p');
q2.appendChild(fd2.createTextNode('Efghijkl'));
b2.appendChild(q2);
var d2 = fd2.documentElement;
var r2 = fd2.createRange();
r2.setStart(d2, 0); r2.setEnd(d2, 1);
var oc2;
try { r2.surroundContents(q2); oc2 = 'NO_THROW'; }
catch (e) { oc2 = 'THREW:' + ((e && e.name) || String(e)); }
function sig(n) {
  var s = [];
  var ks = n.childNodes || [];
  for (var i = 0; i < ks.length; i++) s.push((ks[i].nodeName || ks[i].nodeType) + '(' + (ks[i].childNodes ? ks[i].childNodes.length : 0) + ')');
  return s.join(',');
}
out.push('surround:' + oc2 + ',docEl:' + sig(d2) + ',q2p:' + (q2.parentNode && q2.parentNode.nodeName));
globalThis.__r245out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r245out").unwrap().value;
    assert_eq!(
        out,
        "headP:#document-fragment|move:ok:2|surround:NO_THROW,docEl:P(1),BODY(0),q2p:HTML",
        "R245 factory parentNode getter-only 移动语义：fragment 父链强写 + 摘除守卫"
    );
}

#[test]
fn r249_iframe_element_own_remove_child_splices() {
    // R249（js-dom M4）：iframe 工厂元素（_zwIframeCreateElement 产物）补 own
    // removeChild——旧版经 Node.prototype.removeChild 的数组分支，对 childNodes 为
    // getter 视图的容器 splice 到副本（源未动）而父链置空持久——单向断链
    //（WPT Range-surroundContents 13/14,0「幽灵 P」：R249 栈捕获实证
    // remove → proto removeChild → 父链 null 但 testDiv.childNodes 未失）。
    // 本实现单次读列表 + identity indexOf + 就地 splice。
    // https://dom.spec.whatwg.org/#dom-node-removechild
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            r#"
var out = [];
var ifr = document.createElement('iframe');
document.body.appendChild(ifr);
var idoc = ifr.contentDocument;
var div = idoc.createElement('div');
var kids = [];
for (var q = 0; q < 3; q++) {
  var p = idoc.createElement('p');
  p.appendChild(idoc.createTextNode('T' + q));
  div.appendChild(p);
  kids.push(p);
}
out.push('pre:' + div.childNodes.length);
// remove 中间子——own removeChild 应就地 splice
var rm = div.removeChild(kids[1]);
out.push('rm:' + (rm === kids[1]) + ',len:' + div.childNodes.length);
out.push('order:' + (div.childNodes[0] === kids[0] && div.childNodes[1] === kids[2]));
out.push('p1p:' + (kids[1].parentNode === null));
// 不在子列表 → NotFoundError
var nf = 'none';
try { div.removeChild(kids[1]); } catch (e) { nf = (e && e.name) || String(e); }
out.push('nf:' + nf);
// 非 Node → TypeError
var te = 'none';
try { div.removeChild(null); } catch (e) { te = (e && e.name) || String(e); }
out.push('te:' + te);
globalThis.__r249out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r249out").unwrap().value;
    assert_eq!(
        out,
        "pre:3|rm:true,len:2|order:true|p1p:true|nf:NotFoundError|te:TypeError",
        "R249 iframe 工厂元素 own removeChild：就地 splice + 位置保序 + 父链置空 + NotFound/TypeError 校验"
    );
}

#[test]
fn r254_surround_clone_detaches_newparent_before_deepclone() {
    // R254（js-dom M4）：surroundContents 主路径（covered-children 形态）的
    // clone 循环**前**先摘除 newParent——旧版克隆循环先于 removal 循环
    //（R2930 正序 clone → 逆序 remove），covered 子树（docEl[0,2] 含
    // BODY>div#test>paras[0]=newParent 自身）深克隆时把 newParent 的
    // **克隆中间态**（先克隆进 HEAD-clone、BODY 未克隆时的半完成形态）烘进
    // BODY-clone 内的 div#test（WPT Range-surroundContents 13/14,x「幽灵
    // P#a{HEAD-only}」——probe R254-v5/v6 实证 div#test 首子 = isNP=false 的
    // 第三对象）。spec 序（surround 步骤 3 extract 先移出原件）等效于「克隆
    // 前 newParent 不在覆盖子树内」。摘除幂等（已 detached 时 no-op）。
    // https://dom.spec.whatwg.org/#dom-range-surroundcontents
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            r#"
var out = [];
// iframe 工厂域（与 WPT 13,0 同形态）：idoc 的 docEl 容器覆盖 [h1, wrap]，
// wrap 内含 newParent(np)——克隆循环深克隆 wrap 时旧版把 np 的克隆中间态
// 烘进 wrap-clone（幽灵）；R254 摘除后 wrap-clone 不含 np。
var ifr = document.createElement('iframe');
document.body.appendChild(ifr);
var idoc = ifr.contentDocument;
var container = idoc.createElement('div');
var h1 = idoc.createElement('h1'); h1.appendChild(idoc.createTextNode('H'));
var wrap = idoc.createElement('div'); wrap.id = 'w';
var np = idoc.createElement('p'); np.id = 'np';
np.appendChild(idoc.createTextNode('NP'));
wrap.appendChild(np);
container.appendChild(h1);
container.appendChild(wrap);
var r = idoc.createRange();
r.setStart(container, 0);
r.setEnd(container, 2);
out.push('kids:' + (r._coveredChildren ? r._coveredChildren().length : 'n/a'));
// R254 前置：np 摘除即时生效（工厂域 own remove → removeChild 就地 splice）
out.push('preRm wrapKids:' + wrap.childNodes.length + ',npPn:' + (np.parentNode === wrap));
np.remove();
out.push('postRm wrapKids:' + wrap.childNodes.length + ',npPn:' + (np.parentNode === null));
// 覆盖子重挂回（模拟摘除发生在 surround 内部前的状态）——直接构造克隆形态验证：
// 摘除后 np 的深克隆不在 container 覆盖子树里
wrap.appendChild(np);
r.setStart(container, 0);
r.setEnd(container, 2);
r.surroundContents(np);
// 断言 1：container 首子是 np（上移）
out.push('first:' + (container.childNodes[0] === np ? 'np' : String(container.childNodes[0] && container.childNodes[0].nodeName)));
// 断言 2（R254 核心不变式）：np 子树内无「np 自身的克隆」（幽灵）
var ghost = 'none';
(function walk(n, depth) {
  if (!n || !n.childNodes || depth > 8) return;
  for (var q = 0; q < n.childNodes.length; q++) {
    var c = n.childNodes[q];
    if (c !== np && c.nodeName === 'P' && c.id === 'np') { ghost = 'found'; return; }
    walk(c, depth + 1);
    if (ghost !== 'none') return;
  }
})(np, 0);
out.push('ghost:' + ghost);
// 断言 3：np 内的 wrap 克隆是空壳（克隆前 np 已摘除）
var wrapInNp = null;
for (var wq = 0; wq < np.childNodes.length; wq++) {
  if (np.childNodes[wq].id === 'w') wrapInNp = np.childNodes[wq];
}
out.push('wrapGhost:' + (wrapInNp ? wrapInNp.childNodes.length : 'missing'));
globalThis.__r254out = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r254out").unwrap().value;
    assert_eq!(
        out,
        "kids:2|preRm wrapKids:1,npPn:true|postRm wrapKids:0,npPn:true|first:np|ghost:none|wrapGhost:0",
        "R254 surround 克隆前摘除 newParent：无幽灵克隆中间态烘进覆盖子树克隆"
    );
}

#[test]
fn r255_repro_reference_doc_clone_chain() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
var ifr = document.createElement('iframe');
document.body.appendChild(ifr);
var idoc = ifr.contentDocument;
try { idoc.body.innerHTML = '<div id="test">x</div>\n<script src="../common.js"><\/script>\n<script>"use strict";<\/script>'; } catch (e) { out.push('setERR'); }
out.push('bodyN=' + idoc.body.childNodes.length);
var referenceDoc = document.implementation.createHTMLDocument('');
referenceDoc.removeChild(referenceDoc.documentElement);
var cl1 = idoc.documentElement.cloneNode(true);
var cl1Body = null;
for (var q = 0; q < cl1.childNodes.length; q++) if (String(cl1.childNodes[q].nodeName).toUpperCase()==='BODY') cl1Body = cl1.childNodes[q];
out.push('cl1BodyN=' + (cl1Body ? cl1Body.childNodes.length : 'null'));
referenceDoc.appendChild(cl1);
var rdEl = referenceDoc.documentElement;
var rdKids = rdEl.childNodes || [];
var rdBody = null;
for (var q2 = 0; q2 < rdKids.length; q2++) if (String(rdKids[q2].nodeName).toUpperCase()==='BODY') rdBody = rdKids[q2];
out.push('rdBodyN=' + (rdBody ? rdBody.childNodes.length : 'null'));
globalThis.__r255c = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r255c").unwrap().value;
    assert_eq!(
        out, "bodyN=5|cl1BodyN=5|rdBodyN=5",
        "R255 iframe docEl own cloneNode 保真：克隆链 appendChild 后 body 子数不变（head/body 视图经 _zwDeepCloneEl 深克隆 + R221 rebind 前置条件）"
    );
}

#[test]
fn r256_factory_docelement_mutation_rewires_siblings() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R256 regression: factory docEl mutation re-wires sibling getters.
// setup: foreignDoc = createHTMLDocument('') (factory docEl with [headEl, body]);
// insert a wrapper-domain P before BODY; nextNode walk must traverse BODY subtree.
var foreignDoc = document.implementation.createHTMLDocument('T');
var paras0 = document.createElement('p');
paras0.id = 'a';
var de = foreignDoc.documentElement;
de.insertBefore(paras0, de.childNodes[de.childNodes.length - 1]);
// walk via nextNode (getter chain): HTML -> P -> (no kids) -> next sibling must be BODY
out.push('pNext=' + (paras0.nextSibling ? paras0.nextSibling.nodeName : 'null'));
out.push('hNext=' + (de.childNodes[0].nextSibling ? de.childNodes[0].nextSibling.nodeName : 'null'));
// title ns parity: original factory title vs its deep clone
var t1 = foreignDoc.querySelector ? null : null;
var headEl = foreignDoc.head;
var titleEl = headEl ? headEl.firstChild : null;
out.push('titleNS=' + (titleEl && titleEl.namespaceURI === 'http://www.w3.org/1999/xhtml'));
// removeChild also re-wires: remove P -> BODY.previousSibling === headEl? After removal kids=[HEAD,BODY]
de.removeChild(paras0);
out.push('bodyPrev=' + (de.lastChild.previousSibling ? de.lastChild.previousSibling.nodeName : 'null'));
globalThis.__r256r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r256r").unwrap().value;
    assert_eq!(
        out, "pNext=BODY|hNext=P|titleNS=true|bodyPrev=HEAD",
        "R256 factory docEl mutation 兄弟 getter 重接线 + title ns 显式标注：insertBefore/removeChild 后 nextNode 遍历连续（P.nextSibling=BOD Y），title 与克隆形态 ns 等价"
    );
}

#[test]
fn r257_self_surround_ancestor_hre_and_div_wrap() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// 18,0 form: [paras[0],0,paras[0],1] + paras[0] (self-surround) -> HRE, tree = paras[0] emptied
var paras0 = document.createElement('p');
paras0.id = 'a';
paras0.appendChild(document.createTextNode('Axyz'));
document.body.appendChild(paras0);
var r1 = document.createRange();
r1.setStart(paras0, 0);
r1.setEnd(paras0, 1);
try { r1.surroundContents(paras0); out.push('noThrow'); }
catch (e1) { out.push('t:' + e1.name); }
out.push('kids=' + paras0.childNodes.length);
out.push('inBody=' + (paras0.parentNode === document.body));
// 19,6 form: [detachedPara1,0,detachedPara1,1] + detachedPara1 (self) -> HRE
var dp1 = document.createElement('p');
dp1.appendChild(document.createTextNode('Opqrstuv'));
var r2 = document.createRange();
r2.setStart(dp1, 0);
r2.setEnd(dp1, 1);
try { r2.surroundContents(dp1); out.push('noThrow2'); }
catch (e2) { out.push('t2:' + e2.name); }
out.push('kids2=' + dp1.childNodes.length);
// 19,9 form: [detachedPara1,0,detachedPara1,1] + detachedDiv (parent) -> HRE
// (R262 语义翻转：清子循环 dd.removeChild(dp1b) 按 spec concept-node-pre-remove
// 末段把边界 (dp1b,0) 迁到 (dd,0)，R257 ancestor 检查从 sc=dd 命中 newParent=dd
// 自身 → HRE。旧引擎不迁移边界才「成功 wrap」——真浏览器（spec）同抛 HRE，
// WPT 19,9 两侧（sim+actual）同步翻转为 assert_throws 通过，1840P/0F 保持。)
var dp1b = document.createElement('p');
dp1b.appendChild(document.createTextNode('Opqrstuv'));
var dd = document.createElement('div');
dd.appendChild(dp1b);
var dp2 = document.createElement('p');
dp2.appendChild(document.createTextNode('Wxyz'));
dd.appendChild(dp2);
var r3 = document.createRange();
r3.setStart(dp1b, 0);
r3.setEnd(dp1b, 1);
try { r3.surroundContents(dd); out.push('ok3'); }
catch (e3) { out.push('t3:' + e3.name); }
var ddK = ''; for (var q = 0; q < dd.childNodes.length; q++) { var c257 = dd.childNodes[q]; ddK += (c257.nodeType === undefined ? 'UNT' : c257.nodeType) + (c257.data !== undefined ? ':' + c257.data : ':' + c257.nodeName) + ';'; }
out.push('wrap=' + (dp1b.firstChild === dd ? 'Y' : 'N') + ' ddKids=' + ddK);
globalThis.__r257r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r257r").unwrap().value;
    assert_eq!(
        out, "t:HierarchyRequestError|kids=0|inBody=true|t2:HierarchyRequestError|kids2=0|t3:HierarchyRequestError|wrap=N ddKids=",
        "R257 self-surround HRE（先清子后判 inclusive ancestor）+ R262 后 19,9 detachedDiv 父 newParent 同抛 HRE（清子 removeChild 按 spec 迁移边界到 (dd,0)，ancestor 自检查命中）"
    );
}

#[test]
fn r258_selectnode_sc_fallback_endoffset() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R258 regression: 30,x shape — newParent itself inside covered children of
// [foreignDoc.body,0,foreignTextNode,N]. Extract moves newParent into frag2
// (parentNode -> fragment); the selectNode tail must fall back to sc's
// childNodes view so endOffset lands at idx+1 (WPT "expected 1 got 0").
var foreignDoc = document.implementation.createHTMLDocument('');
var foreignPara1 = foreignDoc.createElement('p');
foreignPara1.appendChild(foreignDoc.createTextNode('Efghijkl'));
foreignDoc.body.appendChild(foreignPara1);
var foreignTextNode = foreignDoc.createTextNode('I admit that I harbor doubts about whether we really need so many things to test, but it is too late to stop now.');
foreignDoc.body.appendChild(foreignTextNode);
var fb = foreignDoc.body;
var range = foreignDoc.createRange();
range.setStart(fb, 0);
range.setEnd(foreignTextNode, 36);
try {
  range.surroundContents(foreignPara1);
  out.push('noThrow');
} catch (e) { out.push('throw:' + e.name); }
out.push('so=' + range.startOffset + ' eo=' + range.endOffset);
out.push('scIsFb=' + (range.startContainer === fb));
out.push('fbKids=' + fb.childNodes.length + ':k0=' + fb.childNodes[0].nodeName);
globalThis.__r258r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r258r").unwrap().value;
    assert_eq!(
        out, "noThrow|so=0 eo=1|scIsFb=true|fbKids=2:k0=P",
        "R258 selectNode 落位 sc 回退：newParent 自身被 extract 移入 frag 后，尾部落位经 sc 的 childNodes 视图回退（endOffset=idx+1）"
    );
}

#[test]
fn r259_leaf_hre_extract_first_boundaries() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R259 regression: 16,x shape — element container, empty covered children,
// leaf (Text) newParent -> HRE with range collapsed to (sc, newOffset).
var testDiv = document.createElement('div');
document.body.appendChild(testDiv);
var textNewParent = document.createTextNode('leaf');
var range = document.createRange();
range.setStart(document.body, 4);
range.setEnd(document.body, 5);
try { range.surroundContents(textNewParent); out.push('noThrow'); }
catch (e) { out.push('t:' + e.name); }
out.push('so=' + range.startOffset + ' eo=' + range.endOffset);
out.push('scIsBody=' + (range.startContainer === document.body));
out.push('bodyLast=' + (document.body.lastChild === textNewParent ? 'leaf' : document.body.lastChild.nodeName));
globalThis.__r259r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r259r").unwrap().value;
    assert_eq!(
        out, "t:HierarchyRequestError|so=3 eo=3|scIsBody=true|bodyLast=leaf",
        "R259 leaf-HRE 先 extract 折叠 + insertNode R219 setEnd 经 crossing 重设：终态 (sc, newOffset) 对（16,x 形态）"
    );
}


#[test]
fn r260_data_mutation_adjusts_live_ranges() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R260 regression: CharacterData mutations adjust live-range boundaries
// (spec concept-node-replace-data). deleteData [2,8) on a range (2->8):
// both boundaries land at 2 (eo was inside the removed span).
var t = document.createTextNode('AbcdefghIjklmno');
var range = document.createRange();
range.setStart(t, 2);
range.setEnd(t, 8);
t.deleteData(2, 6);
out.push('del=' + range.startOffset + '/' + range.endOffset + ':' + t.data);
// insertData at 0 shifts both +len
var t2 = document.createTextNode('Ab');
var range2 = document.createRange();
range2.setStart(t2, 1);
range2.setEnd(t2, 2);
t2.insertData(0, 'xx');
out.push('ins=' + range2.startOffset + '/' + range2.endOffset + ':' + t2.data);
// replaceData adjusts by delta
var t3 = document.createTextNode('Abcdefgh');
var range3 = document.createRange();
range3.setStart(t3, 3);
range3.setEnd(t3, 7);
t3.replaceData(2, 4, 'Z');
out.push('rep=' + range3.startOffset + '/' + range3.endOffset + ':' + t3.data);
// boundary before offset untouched
var t4 = document.createTextNode('Abcdefgh');
var range4 = document.createRange();
range4.setStart(t4, 1);
range4.setEnd(t4, 2);
t4.deleteData(4, 2);
out.push('before=' + range4.startOffset + '/' + range4.endOffset);
globalThis.__r260r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r260r").unwrap().value;
    assert_eq!(
        out, "del=2/2:AbIjklmno|ins=3/4:xxAb|rep=2/4:AbZgh|before=1/2",
        "R260 CharacterData 变更的 live-range 边界调整（spec replace-data 末段：删区间内折叠/区间后偏移/区间前不动）"
    );
}



#[test]
fn r261_splittext_range_retarget() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// parented split: boundary retargets to (new tail node, off - o)
var p = document.createElement('p');
var t = document.createTextNode('Abcdefgh');
p.appendChild(t);
document.body.appendChild(p);
var r1 = document.createRange();
r1.setStart(t, 1);
r1.setEnd(t, 3);
var tail = t.splitText(1);
out.push('parented=' + (r1.startContainer === tail ? 'tail' : (r1.startContainer === t ? 'orig' : 'other'))
  + ':' + r1.startOffset + '/' + (r1.endContainer === tail ? 'tail' : 'orig') + ':' + r1.endOffset);
// detached split: boundary stays on original, shrink-to-offset per replace-data
var t2 = document.createTextNode('Abcdefgh');
var r2 = document.createRange();
r2.setStart(t2, 1);
r2.setEnd(t2, 3);
var tail2 = t2.splitText(1);
out.push('detached=' + (r2.startContainer === t2 ? 'orig' : 'other') + ':' + r2.startOffset
  + '/' + (r2.endContainer === t2 ? 'orig' : 'other') + ':' + r2.endOffset);
globalThis.__r261r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r261r").unwrap().value;
    assert_eq!(
        out, "parented=orig:1/tail:2|detached=orig:1/orig:1",
        "R261 splitText live-range retarget：so=1 不>o=1 保持 (orig,1)；eo=3>1 → (tail, 3-1=2)（split 段判 original offset）；detached 无 split 段仅 replace-data 收缩（eo 3→1）"
    );
}

#[test]
fn r262_removechild_range_boundary_migration() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// Case 1 (WPT 2,x/3,x): boundary ON the removed node -> migrates to (parent, index)
var testDiv = document.createElement('div');
var p0 = document.createElement('p'); p0.appendChild(document.createTextNode('A'));
var p1 = document.createElement('p'); p1.appendChild(document.createTextNode('B'));
var p2 = document.createElement('p'); p2.appendChild(document.createTextNode('C'));
testDiv.appendChild(p0); testDiv.appendChild(p1); testDiv.appendChild(p2);
document.body.appendChild(testDiv);
var r1 = document.createRange();
r1.setStart(p1, 0);
r1.setEnd(p1, 1);
testDiv.removeChild(p1);
out.push('onNode=' + (r1.startContainer === testDiv ? 'div' : 'other') + ':' + r1.startOffset
  + '/' + (r1.endContainer === testDiv ? 'div' : 'other') + ':' + r1.endOffset);
// Case 2 (WPT 4-9,x): boundary in parent, offset > index -> offset - 1
var testDiv2 = document.createElement('div');
var q0 = document.createElement('p'); q0.appendChild(document.createTextNode('A'));
var q1 = document.createElement('p'); q1.appendChild(document.createTextNode('B'));
var q2 = document.createElement('p'); q2.appendChild(document.createTextNode('C'));
testDiv2.appendChild(q0); testDiv2.appendChild(q1); testDiv2.appendChild(q2);
document.body.appendChild(testDiv2);
var r2 = document.createRange();
r2.setStart(testDiv2, 0);
r2.setEnd(testDiv2, 2);
testDiv2.removeChild(q0);
out.push('inParent=' + (r2.startContainer === testDiv2 ? 'div' : 'other') + ':' + r2.startOffset
  + '/' + (r2.endContainer === testDiv2 ? 'div' : 'other') + ':' + r2.endOffset);
// Case 3: offset <= index -> unchanged
var testDiv3 = document.createElement('div');
var w0 = document.createElement('p'); w0.appendChild(document.createTextNode('A'));
var w1 = document.createElement('p'); w1.appendChild(document.createTextNode('B'));
testDiv3.appendChild(w0); testDiv3.appendChild(w1);
document.body.appendChild(testDiv3);
var r3 = document.createRange();
r3.setStart(testDiv3, 0);
r3.setEnd(testDiv3, 1);
testDiv3.removeChild(w1);
out.push('leIdx=' + (r3.startContainer === testDiv3 ? 'div' : 'other') + ':' + r3.startOffset
  + '/' + (r3.endContainer === testDiv3 ? 'div' : 'other') + ':' + r3.endOffset);
globalThis.__r262r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r262r").unwrap().value;
    assert_eq!(
        out, "onNode=div:1/div:1|inParent=div:0/div:1|leIdx=div:0/div:1",
        "R262 removeChild live-range 边界迁移：边界在被移除节点上 → (父, 旧索引)；边界在父且 offset>索引 → -1；offset<=索引不动"
    );
}

#[test]
fn r263_insert_range_boundary_adjust() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// appendChild move: remove(old pos) + insert(new pos) — WPT testAppendChild 序
var testDiv = document.createElement('div');
var p0 = document.createElement('p'); p0.appendChild(document.createTextNode('A'));
var p1 = document.createElement('p'); p1.appendChild(document.createTextNode('B'));
var p2 = document.createElement('p'); p2.appendChild(document.createTextNode('C'));
testDiv.appendChild(p0); testDiv.appendChild(p1); testDiv.appendChild(p2);
document.body.appendChild(testDiv);
// boundary ON the moved node (WPT "collapsed at (testDiv.lastChild, 0)")
var r1 = document.createRange();
r1.setStart(p2, 0);
r1.setEnd(p2, 1);
testDiv.appendChild(p2); // move last to last: remove(idx2)+insert(idx2)
out.push('onMoved=' + (r1.startContainer === testDiv ? 'div' : 'other') + ':' + r1.startOffset
  + '/' + (r1.endContainer === testDiv ? 'div' : 'other') + ':' + r1.endOffset);
// boundary in parent, offset > old index and > new index (net 0 via -1+1)
var testDiv2 = document.createElement('div');
var q0 = document.createElement('p'); q0.appendChild(document.createTextNode('A'));
var q1 = document.createElement('p'); q1.appendChild(document.createTextNode('B'));
var q2 = document.createElement('p'); q2.appendChild(document.createTextNode('C'));
testDiv2.appendChild(q0); testDiv2.appendChild(q1); testDiv2.appendChild(q2);
document.body.appendChild(testDiv2);
var r2 = document.createRange();
r2.setStart(testDiv2, 3);
r2.setEnd(testDiv2, 3);
testDiv2.appendChild(q0); // move first to last: remove(idx0)+insert(idx2)
out.push('inParent=' + (r2.startContainer === testDiv2 ? 'div' : 'other') + ':' + r2.startOffset
  + '/' + (r2.endContainer === testDiv2 ? 'div' : 'other') + ':' + r2.endOffset);
// replaceChild: remove(old)+remove(new)+insert(new) — WPT testReplaceChild 序
var testDiv3 = document.createElement('div');
var w0 = document.createElement('p'); w0.appendChild(document.createTextNode('A'));
var w1 = document.createElement('p'); w1.appendChild(document.createTextNode('B'));
testDiv3.appendChild(w0); testDiv3.appendChild(w1);
document.body.appendChild(testDiv3);
var wNew = document.createElement('p'); wNew.appendChild(document.createTextNode('N'));
var r3 = document.createRange();
r3.setStart(testDiv3, 1);
r3.setEnd(testDiv3, 2);
testDiv3.replaceChild(wNew, w1);
out.push('replaced=' + (r3.startContainer === testDiv3 ? 'div' : 'other') + ':' + r3.startOffset
  + '/' + (r3.endContainer === testDiv3 ? 'div' : 'other') + ':' + r3.endOffset);
globalThis.__r263r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r263r").unwrap().value;
    assert_eq!(
        out, "onMoved=div:2/div:2|inParent=div:2/div:2|replaced=div:1/div:1",
        "R263 insert 侧边界调整：append 移动 = remove(旧位)+insert(新位) 两段（边界在 moved 节点 → (父, 旧 idx)；父内 offset 净 0 经 -1+1）；replaceChild 三段序"
    );
}

#[test]
fn r265_insertbefore_text_ref_registry_insert() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// R265 form: insertBefore(el, textEl-ref) — paras[0].insertBefore(paras[1], paras[0].firstChild)
var p0 = document.createElement('p');
p0.textContent = 'Axyz';
document.body.appendChild(p0);
var p1 = document.createElement('p');
p1.textContent = 'B';
document.body.appendChild(p1);
var fk = p0.firstChild;
p0.insertBefore(p1, fk);
// ① childNodes 视图含 p1（死循环根因 = indexOf 在视图 miss 上无终止自旋）
var p1In = false, order = '';
for (var i = 0; i < p0.childNodes.length; i++) {
  var c = p0.childNodes[i];
  if (c === p1) p1In = true;
  order += (c === p1 ? 'P1' : (c === fk ? 'TXT' : (c && c.nodeType))) + ';';
}
out.push('kids=' + p0.childNodes.length + ' p1In=' + p1In + ' order=' + order);
// ② 无终止 indexOf 的等价安全读（splice 后视图命中即验证不挂）
var idx = -1;
for (var j = 0; j < p0.childNodes.length; j++) if (p0.childNodes[j] === p1) { idx = j; break; }
out.push('idx=' + idx);
// ③ text identity 保持 + data 仍可编辑（materialize 不注销 node 闭包）
out.push('txtSame=' + (p0.childNodes[1] === fk) + ' len=' + (p0.lastChild && p0.lastChild.length));
// ④ 再插一个（物化后路径——无 textEl 注册表，走 handle-handle splice）
var p2 = document.createElement('p');
p2.textContent = 'C';
p0.insertBefore(p2, fk);
var order2 = '';
for (var k = 0; k < p0.childNodes.length; k++) {
  var c2 = p0.childNodes[k];
  order2 += (c2 === p1 ? 'P1' : (c2 === p2 ? 'P2' : (c2 === fk ? 'TXT' : (c2 && c2.nodeType)))) + ';';
}
out.push('order2=' + order2);
globalThis.__r265r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r265r").unwrap().value;
    assert_eq!(
        out, "kids=2 p1In=true order=P1;TXT;|idx=0|txtSame=true len=4|order2=P1;P2;TXT;",
        "R265 textEl ref 的 insertBefore registry 插入：视图含 newNode（indexOf 不再自旋）+ 顺序 [new, text] + text identity/data 保持 + 物化后二次插入位次正确"
    );
}

#[test]
fn r266_deletecontents_detached_chardata() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
var out = [];
// full-range delete on detached text
var dn = document.createTextNode('Uvwxyzab');
var r1 = dn.ownerDocument.createRange();
r1.setStart(dn, 0);
r1.setEnd(dn, 8);
r1.deleteContents();
out.push('full=' + dn.data + '/collapsed=' + r1.collapsed);
// mid-range delete on detached text
var dn2 = document.createTextNode('Uvwxyzab');
var r2 = dn2.ownerDocument.createRange();
r2.setStart(dn2, 2);
r2.setEnd(dn2, 5);
r2.deleteContents();
out.push('mid=' + dn2.data);
// detached comment
var dc = document.createComment('cabcdef');
var r3 = dc.ownerDocument.createRange();
r3.setStart(dc, 1);
r3.setEnd(dc, 4);
r3.deleteContents();
out.push('cmt=' + dc.data);
globalThis.__r266r = out.join('|');
"#,
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r266r").unwrap().value;
    assert_eq!(
        out, "full=/collapsed=true|mid=Uvzab|cmt=cdef",
        "R266 deleteContents 同节点 CharData（detached）放宽：deleteData 削区间 + collapse（R228 extract 同款——sc===ec 无需父容器）"
    );
}

#[test]
fn r267_deletecontents_ancestor_chardata() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        // 23,x form: sc = p (element), ec = its direct text child; so=0, eo=6 of "Aabcdef" -> remainder "ef"
        "var p = document.createElement('p');",
        "p.textContent = 'Aabcdef';",
        "document.body.appendChild(p);",
        "var t = p.firstChild;",
        "var r = document.createRange();",
        "r.setStart(p, 0); r.setEnd(t, 4);",
        "r.deleteContents();",
        "out.push('data=' + t.data + '/kids=' + p.childNodes.length",
        "  + '/sc=' + (r.startContainer === p ? 'p' : 'other') + ':' + r.startOffset);",
        // contained middle children: [p2, 0, t2, 2] with leading span removed
        "var p2 = document.createElement('p');",
        "var sp = document.createElement('span');",
        "p2.appendChild(sp);",
        "var t2 = document.createTextNode('tail');",
        "p2.appendChild(t2);",
        "document.body.appendChild(p2);",
        "var r2 = document.createRange();",
        "r2.setStart(p2, 0); r2.setEnd(t2, 2);",
        "r2.deleteContents();",
        "out.push('kids2=' + p2.childNodes.length + '/data2=' + t2.data + '/so2=' + r2.startOffset);",
        "globalThis.__r267r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    let expected = "data=def/kids=1/sc=p:0|kids2=1/data2=il/so2=0";
    assert_eq!(
        out, expected,
        "R267 deleteContents ancestor branch: ec head deleteData + contained middle removal + collapse to (sc, so)"
    );
}

#[test]
fn r268_deletecontents_cross_chardata() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        // 20/21,x form: sc=text in p0, ec=text in p1 (disjoint subtrees under testDiv)
        "var testDiv = document.createElement('div');",
        "var p0 = document.createElement('p'); p0.textContent = 'Aabcdef';",
        "var p1 = document.createElement('p'); p1.textContent = 'Ijklmnop';",
        "var p2 = document.createElement('p'); p2.textContent = 'Qrstuvwx';",
        "testDiv.appendChild(p0); testDiv.appendChild(p1); testDiv.appendChild(p2);",
        "document.body.appendChild(testDiv);",
        "var r = document.createRange();",
        "r.setStart(p0.firstChild, 3); r.setEnd(p1.firstChild, 4);",
        "r.deleteContents();",
        "out.push('p0t=' + p0.firstChild.data + '/p1t=' + p1.firstChild.data",
        "  + '/kids=' + testDiv.childNodes.length",
        "  + '/sc=' + (r.startContainer === testDiv ? 'div' : 'other') + ':' + r.startOffset);",
        "globalThis.__r268r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "p0t=Aab/p1t=mnop/kids=3/sc=div:1",
        "R268 deleteContents cross-container CharData: sc tail trim + middle p-removal + ec head trim + collapse to (cac, refIdx+1)"
    );
}

#[test]
fn r269_deletecontents_document_container() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        // (doc, 0)-(doc, 1): contained doctype removed (child count 2 -> 1)
        "var r1 = document.createRange();",
        "r1.setStart(document, 0); r1.setEnd(document, 1);",
        "r1.deleteContents();",
        "out.push('kids=' + document.childNodes.length",
        "  + '/so=' + r1.startOffset + '/scIsDoc=' + (r1.startContainer === document));",
        "globalThis.__r269r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "kids=1/so=0/scIsDoc=true",
        "R269 deleteContents document-container: contained doctype removed + collapse to (doc, 0)"
    );
}

#[test]
fn r270_title_cdp_methods() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var fd = document.implementation.createHTMLDocument('T');",
        // walk the whole subtree: every node must have cDP + contains (getPosition contract)
        "var missing = [];",
        "(function walk(n) {",
        "  if (!n) return;",
        "  if (typeof n.compareDocumentPosition !== 'function' || typeof n.contains !== 'function') {",
        "    missing.push(n.nodeName + '(' + n.nodeType + ')');",
        "  }",
        "  var kids = n.childNodes || [];",
        "  for (var i = 0; i < kids.length; i++) walk(kids[i]);",
        "})(fd);",
        // and a live call must return a numeric bitmask
        "var de = fd.documentElement;",
        "var pos = 0;",
        "try { pos = de.compareDocumentPosition(fd.doctype); } catch (e) { missing.push('THREW:' + e.message); }",
        "globalThis.__r270r = 'missing=' + missing.join(',') + ';pos=' + pos;",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "missing=;pos=2",
        "R270 title cDP/contains: full foreignDoc subtree has the methods (getPosition contract); cDP(docEl, doctype) = PRECEDING (2)"
    );
}

#[test]
fn r271_removechild_plain_child_in_registry() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        "// CDATA via the XML-document realm (same as WPT common.js paras[5])",
        "var xmlDocument = new Document();",
        "var c1 = xmlDocument.createCDATASection('1234');",
        "var c2 = xmlDocument.createCDATASection('5678');",
        "var t = document.createTextNode('9012');",
        "var p = document.createElement('p');",
        "document.body.appendChild(p);",
        "p.appendChild(c1); p.appendChild(c2); p.appendChild(t);",
        "p.removeChild(c2);",
        "var kids = '';",
        "for (var i = 0; i < p.childNodes.length; i++) {",
        "  var k = p.childNodes[i];",
        "  kids += k.nodeName + ':' + String(k.data != null ? k.data : '') + ';';",
        "}",
        "out.push('kids=' + kids + ' c2parent=' + (c2.parentNode === null ? 'null' : 'live'));",
        "// full range-6 shape deleteContents",
        "var p2 = document.createElement('p');",
        "document.body.appendChild(p2);",
        "var f2 = xmlDocument.createCDATASection('1234');",
        "var m2 = xmlDocument.createCDATASection('5678');",
        "var l2 = document.createTextNode('9012');",
        "p2.appendChild(f2); p2.appendChild(m2); p2.appendChild(l2);",
        "var r = document.createRange();",
        "r.setStart(f2, 2); r.setEnd(l2, 4);",
        "r.deleteContents();",
        "var kids2 = '';",
        "for (var j = 0; j < p2.childNodes.length; j++) {",
        "  var k2 = p2.childNodes[j];",
        "  kids2 += k2.nodeName + ':' + String(k2.data != null ? k2.data : '') + ';';",
        "}",
        "out.push('range6=' + kids2 + ' so=' + r.startOffset);",
        "globalThis.__r271r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "kids=#cdata-section:1234;#text:9012; c2parent=live|range6=#cdata-section:12;#text:; so=1",
        "R271 removeChild plain-child registry splice: CDATA mid removed from normal element parent (childNodes contract); parentNode slot of the XML-realm wrapper is a closure (documented limitation); range-6 shape deleteContents trims both ends + removes middle + collapses to (p,1)"
    );
}

#[test]
fn r273_cdata_sibling_navigation() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        "var xmlDocument = new Document();",
        "var c1 = xmlDocument.createCDATASection('1234');",
        "var c2 = xmlDocument.createCDATASection('5678');",
        "var t = document.createTextNode('9012');",
        "var p = document.createElement('p');",
        "document.body.appendChild(p);",
        "p.appendChild(c1); p.appendChild(c2); p.appendChild(t);",
        // detached: null
        "var d0 = xmlDocument.createCDATASection('dd');",
        "out.push('detached=' + (d0.nextSibling === null ? 'null' : 'BAD') + '/' + (d0.previousSibling === null ? 'null' : 'BAD'));",
        // parented: sibling navigation via parent's childNodes
        "var ns = c1.nextSibling === c2 ? 'c2' : String(c1.nextSibling);",
        "var ps = t.previousSibling === c2 ? 'c2' : String(t.previousSibling);",
        "var last = c2.nextSibling === t ? 't' : String(c2.nextSibling);",
        "var end = t.nextSibling === null ? 'null' : 'BAD';",
        "out.push('parented=' + ns + '/' + ps + '/' + last + '/' + end);",
        "globalThis.__r273r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "detached=null/null|parented=c2/c2/t/null",
        "R273 CDATA sibling getters: self-computed via parent childNodes indexOf (oracle nextNode climb contract); null when detached or at edges"
    );
}

#[test]
fn r278_clone_realm_paras_sibling_chain() {
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
        "<html><body><div id=\"test\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R278 repro: mimic restoreIframe + setupRangeTests — a fresh doc gets a
    // cloned docElement appended, then handle-element paras are appended into
    // the (clone-realm) testDiv/body. The oracle nextNode climb contract:
    // P#a.nextSibling === P#b, and DIV's nextSibling via body chain.
    let js = [
        "var out = [];",
        // referenceDoc clone chain (restoreIframe shape)
        "var referenceDoc = document.implementation.createHTMLDocument('');",
        "referenceDoc.removeChild(referenceDoc.documentElement);",
        "var ifr = document.createElement('iframe');",
        "document.body.appendChild(ifr);",
        "var idoc = ifr.contentDocument;",
        "idoc.appendChild(referenceDoc.documentElement.cloneNode(true));",
        // setupRangeTests shape inside the iframe doc realm
        "var testDiv = idoc.createElement('div');",
        "testDiv.id = 'test';",
        "idoc.body.insertBefore(testDiv, idoc.body.firstChild);",
        "var paras = [];",
        "for (var i = 0; i < 3; i++) {",
        "  var p = idoc.createElement('p');",
        "  p.id = ['a','b','c'][i];",
        "  p.textContent = 'p' + i;",
        "  testDiv.appendChild(p);",
        "  paras.push(p);",
        "}",
        "var cm = idoc.createComment('tail');",
        "testDiv.appendChild(cm);",
        // oracle climb contract probes
        "var nsA = paras[0].nextSibling === paras[1] ? 'b' : String(paras[0].nextSibling && paras[0].nextSibling.nodeName);",
        "var nsB = paras[1].nextSibling === paras[2] ? 'c' : String(paras[1].nextSibling && paras[1].nextSibling.nodeName);",
        "var nsC = paras[2].nextSibling === cm ? 'cm' : String(paras[2].nextSibling && paras[2].nextSibling.nodeName);",
        "var nsDiv = testDiv.nextSibling === null ? 'null' : String(testDiv.nextSibling && testDiv.nextSibling.nodeName);",
        "var fnA = paras[0].firstChild != null && String(paras[0].firstChild.data);",
        "out.push('chain=' + nsA + '/' + nsB + '/' + nsC + '/' + nsDiv + ' text=' + fnA);",
        "globalThis.__r278r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out, "chain=b/c/cm/null text=p0",
        "R278 clone-realm paras sibling chain: oracle nextNode climb needs paras[i].nextSibling resolvable in the restoreIframe clone realm (append into iframe-doc body/testDiv); WPT Range-deleteContents 22,x oracle walk n=0 root cause"
    );
}

#[test]
fn r279_sc_element_cross_container_delete() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R279 repro: sc=element cross-container deleteContents in the iframe-realm
    // shape (restoreIframe + setupRangeTests paras). Covers three forms:
    // 24,x sc-el/ec-el (DIV[2..4) removed + P#e emptied + collapse (DIV,2)),
    // 48,x sc-el/ec-CharData (DIV[1..2) removed + ec head-trimmed + collapse (DIV,1)),
    // 49,x same-tree-position (empty delete + collapse (sc,so)).
    let js = [
        "var out = [];",
        "var referenceDoc = document.implementation.createHTMLDocument('');",
        "referenceDoc.removeChild(referenceDoc.documentElement);",
        "var ifr = document.createElement('iframe');",
        "document.body.appendChild(ifr);",
        "var idoc = ifr.contentDocument;",
        "idoc.appendChild(referenceDoc.documentElement.cloneNode(true));",
        "var testDiv = idoc.createElement('div');",
        "idoc.body.insertBefore(testDiv, idoc.body.firstChild);",
        "var mk = function (id, text) {",
        "  var p = idoc.createElement('p');",
        "  p.id = id;",
        "  p.textContent = text;",
        "  testDiv.appendChild(p);",
        "  return p;",
        "};",
        "var pa = mk('a', 'A0123');",
        "var pb = mk('b', 'B0123');",
        "var pc = mk('c', 'C0123');",
        "var pd = mk('d', 'D0123');",
        "var pe = mk('e', 'E0123');",
        "var cm = idoc.createComment('tailcm');",
        "testDiv.appendChild(cm);",
        "function dumpDiv() {",
        "  var ks = testDiv.childNodes, s = [];",
        "  for (var i = 0; i < ks.length; i++) {",
        "    var k = ks[i];",
        "    s.push(k.nodeName + (k.id ? '#' + k.id : '') + '('",
        "      + String(k.firstChild && k.firstChild.data != null ? k.firstChild.data : (k.data != null ? k.data : '')) + ')');",
        "  }",
        "  return ks.length + '[' + s.join(',') + ']';",
        "}",
        // form 24,x: [testDiv, 2, paras[4] (P#e), 1] — kids=[pa,pb,pc,pd,pe,cm],
        // delete [2,4)=pc,pd, empty P#e, collapse (DIV,2) -> [pa,pb,pe(),cm]
        "var r1 = idoc.createRange();",
        "r1.setStart(testDiv, 2); r1.setEnd(pe, 1);",
        "r1.deleteContents();",
        "out.push('f24=' + dumpDiv() + ' col=' + (r1.startContainer === testDiv) + '/' + r1.startOffset);",
        // form 48,x: [testDiv, 1, paras[2] (P#c) firstChild, 5] — fresh paras so
        // the f24 empties don't bleed in; kids=[pa,pb,pc2,pd2,pe2,cm], delete
        // [1,2)=pb, head-trim pc2 text by 5 ("C012345"->"45"), collapse (DIV,1)
        "var pc2 = mk('c', 'C012345');",
        "var pd2 = mk('d', 'D0123');",
        "var pe2 = mk('e', 'E0123');",
        "testDiv.appendChild(cm);",
        "var r2 = idoc.createRange();",
        "r2.setStart(testDiv, 1); r2.setEnd(pc2.firstChild, 5);",
        "r2.deleteContents();",
        "out.push('f48=' + dumpDiv() + ' col=' + (r2.startContainer === testDiv) + '/' + r2.startOffset + ' pcText=' + String(pc2.firstChild.data));",
        // form 49,x: same-tree-position — sc=testDiv so=1 (pc2 index), ec=pc2 eo=0:
        // empty delete, collapse (sc,so)=(testDiv,1). (WPT 49,x uses docEl/body —
        // here testDiv/pc2 keeps the shape minimal while sk0[so]===ec holds.)
        "var r3 = idoc.createRange();",
        "r3.setStart(testDiv, 1); r3.setEnd(pc2, 0);",
        "var before3 = dumpDiv();",
        "r3.deleteContents();",
        "out.push('f49=same=' + (before3 === dumpDiv()) + ' col=' + (r3.startContainer === testDiv) + '/' + r3.startOffset);",
        "globalThis.__r279r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out,
        "f24=4[P#a(A0123),P#b(B0123),P#e(),#comment(tailcm)] col=true/2|f48=5[P#a(A0123),P#c(45),P#d(D0123),P#e(E0123),#comment(tailcm)] col=true/1 pcText=45|f49=same=true col=true/1",
        "R279 sc-element cross-container deleteContents: 24,x form removes DIV[2,4) + empties P#e + collapses (DIV,2); 48,x form removes DIV[1,2) + head-trims ec text; 49,x same-tree-position deletes nothing + collapses (sc,so)"
    );
}

#[test]
fn r280_cross_container_extract_probe() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        "var referenceDoc = document.implementation.createHTMLDocument('');",
        "referenceDoc.removeChild(referenceDoc.documentElement);",
        "var ifr = document.createElement('iframe');",
        "document.body.appendChild(ifr);",
        "var idoc = ifr.contentDocument;",
        "idoc.appendChild(referenceDoc.documentElement.cloneNode(true));",
        "var testDiv = idoc.createElement('div');",
        "idoc.body.insertBefore(testDiv, idoc.body.firstChild);",
        "var mk = function (id, text) {",
        "  var p = idoc.createElement('p');",
        "  p.id = id;",
        "  p.textContent = text;",
        "  testDiv.appendChild(p);",
        "  return p;",
        "};",
        "var pa = mk('a', 'A0123');",
        "var pb = mk('b', 'B0123');",
        "var pc = mk('c', 'C0123');",
        "var pd = mk('d', 'D0123');",
        "var cm = idoc.createComment('tailcm');",
        "testDiv.appendChild(cm);",
        "function dump(n) {",
        "  var ks = n.childNodes, s = [];",
        "  for (var i = 0; i < ks.length; i++) {",
        "    var k = ks[i];",
        "    s.push(k.nodeName + (k.id ? '#' + k.id : '') + '('",
        "      + String(k.firstChild && k.firstChild.data != null ? k.firstChild.data : (k.data != null ? k.data : '')) + ')');",
        "  }",
        "  return ks.length + '[' + s.join(',') + ']';",
        "}",
        // 52,x shape: [pc.firstChild, 4, comment, 2]
        "var r = idoc.createRange();",
        "r.setStart(pc.firstChild, 4); r.setEnd(cm, 2);",
        "var frag = r.extractContents();",
        "out.push('tree=' + dump(testDiv) + ' frag=' + dump(frag));",
        "out.push('col=' + (r.startContainer === testDiv) + '/' + r.startOffset);",
        "globalThis.__r280p = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    eprintln!("R280PROBE: {}", out);
    assert_eq!(
        out,
        "tree=4[P#a(A0123),P#b(B0123),P#c(C012),#comment(ilcm)] frag=3[P#c(3),P#d(D0123),#comment(ta)]|col=true/3",
        "R280 cross-container extract 52,x shape: firstPartial P#c clone wraps the sc tail text, middle P#d moved, ec comment head-clone last; source tree pruned; collapse (DIV,3)"
    );
}

#[test]
fn r281_cross_container_clone_contents() {
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = [
        "var out = [];",
        "var referenceDoc = document.implementation.createHTMLDocument('');",
        "referenceDoc.removeChild(referenceDoc.documentElement);",
        "var ifr = document.createElement('iframe');",
        "document.body.appendChild(ifr);",
        "var idoc = ifr.contentDocument;",
        "idoc.appendChild(referenceDoc.documentElement.cloneNode(true));",
        "var testDiv = idoc.createElement('div');",
        "idoc.body.insertBefore(testDiv, idoc.body.firstChild);",
        "var mk = function (id, text) {",
        "  var p = idoc.createElement('p');",
        "  p.id = id;",
        "  p.textContent = text;",
        "  testDiv.appendChild(p);",
        "  return p;",
        "};",
        "var pa = mk('a', 'A0123');",
        "var pb = mk('b', 'B0123');",
        "var pc = mk('c', 'C0123');",
        "var cm = idoc.createComment('tailcm');",
        "testDiv.appendChild(cm);",
        "function dump(n) {",
        "  var ks = n.childNodes, s = [];",
        "  for (var i = 0; i < ks.length; i++) {",
        "    var k = ks[i];",
        "    s.push(k.nodeName + (k.id ? '#' + k.id : '') + '('",
        "      + String(k.firstChild && k.firstChild.data != null ? k.firstChild.data : (k.data != null ? k.data : '')) + ')');",
        "  }",
        "  return ks.length + '[' + s.join(',') + ']';",
        "}",
        // cross CD→CD: [pa.firstChild, 2, pb.firstChild, 3]
        "var r1 = idoc.createRange();",
        "r1.setStart(pa.firstChild, 2); r1.setEnd(pb.firstChild, 3);",
        "var f1 = r1.cloneContents();",
        "out.push('cdcd=' + dump(f1) + ' tree=' + dump(testDiv));",
        // cross CD→comment: [pc.firstChild, 1, cm, 2]
        "var r2 = idoc.createRange();",
        "r2.setStart(pc.firstChild, 1); r2.setEnd(cm, 2);",
        "var f2 = r2.cloneContents();",
        "out.push('cdcm=' + dump(f2));",
        // same-node CD slice: [cm, 2, cm, 5]
        "var r3 = idoc.createRange();",
        "r3.setStart(cm, 2); r3.setEnd(cm, 5);",
        "var f3 = r3.cloneContents();",
        "out.push('samecd=' + dump(f3));",
        // same-node CD empty slice: collapsed → empty frag
        "var r4 = idoc.createRange();",
        "r4.setStart(cm, 2); r4.setEnd(cm, 2);",
        "var f4 = r4.cloneContents();",
        "out.push('empty=' + f4.childNodes.length);",
        "globalThis.__r281r = out.join('|');",
    ].join("\n");
    let out = sandbox.execute(&js).unwrap().value;
    assert_eq!(
        out,
        "cdcd=2[P#a(123),P#b(B01)] tree=4[P#a(A0123),P#b(B0123),P#c(C0123),#comment(tailcm)]|cdcm=2[P#c(0123),#comment(ta)]|samecd=1[#comment(ilc)]|empty=0",
        "R281 cross-container cloneContents: CD→CD path-clone [P#a(tail), P#b(head)], CD→comment [P#c(tail), middles none, comment head-clone], same-node comment slice, collapsed empty frag; source tree untouched"
    );
}
