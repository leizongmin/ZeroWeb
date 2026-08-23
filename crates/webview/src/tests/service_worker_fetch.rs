use crate::WebViewConfig;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zero_engine::fetch_bridge::FetchResponse;
use zero_storage::ServiceWorkerState;

fn wait_for_state(webview: &mut crate::WebView, registration_id: u64, expected: ServiceWorkerState) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let _ = webview.poll_service_worker_runtime_events();
        if webview
            .service_worker_runtime_registration(registration_id)
            .is_some_and(|registration| registration.state == expected)
        {
            return;
        }
        assert!(Instant::now() < deadline, "Service Worker state timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn controlled_iframe_fetch_event_exposes_navigation_request_projection() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/request-end-to-end.https.html";
    let fallback_requests = Arc::new(Mutex::new(Vec::new()));
    let fallback_log = Arc::clone(&fallback_requests);
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/service-workers/service-worker/resources/request-end-to-end-worker.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: "addEventListener('fetch', event => {
                         event.respondWith((async () => {
                           let appendHeaderError = '';
                           try {
                             event.request.headers.append('X-Test', 'test');
                           } catch (error) {
                             appendHeaderError = error.name;
                           }
                           let requestConstructError = '';
                           try {
                             new Request(event.request);
                           } catch (error) {
                             requestConstructError = error.name;
                           }
                           return new Response(JSON.stringify({
                             url: event.request.url,
                             method: event.request.method,
                             referrer: event.request.referrer,
                             mode: event.request.mode,
                             credentials: event.request.credentials,
                             redirect: event.request.redirect,
                             append_header_error: appendHeaderError,
                             request_construct_error: requestConstructError,
                             has_user_agent: event.request.headers.has('user-agent')
                           }));
                         })());
                       });"
                .as_bytes()
                .to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(move |request| {
            fallback_log.lock().unwrap().push(request.url.clone());
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: "fallback".into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    let registration_id = webview
        .register_service_worker_runtime("resources/request-end-to-end-worker.js", Some("resources/"), PAGE_URL)
        .unwrap();
    wait_for_state(&mut webview, registration_id, ServiceWorkerState::Activated);

    webview.complete_load(
        "<iframe id=\"frame\" src=\"resources/blank.html\"></iframe>
         <script>
           const frame = document.body.firstElementChild;
           const data = JSON.parse(frame.contentDocument.body.textContent);
           globalThis.__swIframeFetch = [
             data.url === frame.src,
             data.url,
             data.method,
             data.referrer,
             data.mode,
             data.credentials,
             data.redirect,
             data.append_header_error,
             data.request_construct_error,
             String(data.has_user_agent)
           ].join('|');
         </script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();
    let result = webview.execute_script("globalThis.__swIframeFetch").unwrap();

    assert_eq!(
        result,
        "true|https://example.test/service-workers/service-worker/resources/blank.html|GET|\
         https://example.test/service-workers/service-worker/request-end-to-end.https.html|\
         navigate|include|manual|TypeError||false"
    );
    assert!(fallback_requests.lock().unwrap().is_empty());
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn controlled_iframe_fetch_delivers_async_respond_with_result_message() {
    const PAGE_URL: &str =
        "https://example.test/service-workers/service-worker/fetch-event-async-respond-with.https.html";
    let fallback_requests = Arc::new(Mutex::new(Vec::new()));
    let fallback_log = Arc::clone(&fallback_requests);
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script
                != "https://example.test/service-workers/service-worker/resources/fetch-event-async-respond-with-worker.js"
            {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: "let reportResult;
                       addEventListener('message', event => {
                         const resultPromise = new Promise(resolve => {
                           reportResult = resolve;
                           event.source.postMessage('messageHandlerInitialized');
                         });
                         event.waitUntil(resultPromise.then(result => {
                           reportResult = null;
                           event.source.postMessage(result);
                         }));
                       });
                       function tryRespondWith(event) {
                         try {
                           event.respondWith(new Response('ok'));
                           reportResult({didThrow: false});
                         } catch (error) {
                           reportResult({didThrow: true, error: error.name});
                         }
                       }
                       addEventListener('fetch', event => {
                         const test = new URL(event.request.url).pathname.split('/').pop();
                         if (test === 'respondWith-in-task') {
                           setTimeout(() => tryRespondWith(event), 0);
                         } else if (test === 'respondWith-in-microtask') {
                           Promise.resolve().then(() => tryRespondWith(event));
                         }
                       });"
                .as_bytes()
                .to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(move |request| {
            fallback_log.lock().unwrap().push(request.url.clone());
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/plain".into())],
                body: "fallback".into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<iframe id=\"frame\" src=\"resources/simple.html\"></iframe>
         <script>
           globalThis.__swSetup = 'pending';
           globalThis.__swMessages = [];
           navigator.serviceWorker.addEventListener('message', event => {
             globalThis.__swMessages.push(event.data);
           });
           navigator.serviceWorker.register(
             'resources/fetch-event-async-respond-with-worker.js',
             {scope: 'resources/simple.html'}
           ).then(reg => {
             globalThis.__swWorker = reg.installing || reg.active;
             function waitActive() {
               if (globalThis.__swWorker && globalThis.__swWorker.state === 'activated') {
                 globalThis.__swSetup = 'ready';
               } else {
                 setTimeout(waitActive, 0);
               }
             }
             waitActive();
           }, error => {
             globalThis.__swSetup = 'error:' + String(error);
           });
         </script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview.execute_script("globalThis.__swSetup").unwrap();
        if value != "pending" {
            assert_eq!(value, "ready");
            break;
        }
        assert!(Instant::now() < deadline, "Service Worker active handle timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    webview
        .execute_script("globalThis.__swWorker.postMessage('initializeMessageHandler');")
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview
            .execute_script("String(globalThis.__swMessages[0] || '')")
            .unwrap();
        if value == "messageHandlerInitialized" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Service Worker initialization message timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    webview
        .execute_script(
            "document.getElementById('frame').contentWindow
               .fetch('respondWith-in-task')
               .then(() => { globalThis.__swFetchDone = true; }, () => { globalThis.__swFetchDone = true; });
             'started';",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview
            .execute_script(
                "JSON.stringify({
                   done: !!globalThis.__swFetchDone,
                   messages: globalThis.__swMessages,
                   workerSequence: globalThis.__swWorker._messageSequence,
                   workerTarget: globalThis.__swWorker._messagePollTarget,
                   workerPending: globalThis.__swWorker._messagePollPending
                 })",
            )
            .unwrap();
        if value.contains("InvalidStateError") {
            assert!(value.contains("\"done\":true"), "{value}");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Service Worker fetch result message timed out: {value}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        fallback_requests.lock().unwrap().as_slice(),
        &[
            "https://example.test/service-workers/service-worker/resources/simple.html".to_string(),
            "https://example.test/service-workers/service-worker/resources/respondWith-in-task".to_string(),
        ]
    );
}
