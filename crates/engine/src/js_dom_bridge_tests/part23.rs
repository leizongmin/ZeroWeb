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
        "data:Stuwxyz|threw:HierarchyRequestError|pi-so:0,pi-threw:HierarchyRequestError|leaf-data:Stuwxyz,leaf-threw:HierarchyRequestError|r230:Op,HierarchyRequestError|r231:2,8",
        "R228/R229 detached comment 区间 surround：extract 切片 + HRE 上抛 + 同节点 collapse (容器, startOffset) + leaf-newParent 先 extract 再抛"
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
