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
