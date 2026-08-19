use crate::WebViewBuilder;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zero_storage::ServiceWorkerState;

fn wait_for_state(webview: &mut crate::WebView, registration_id: u64, expected: ServiceWorkerState) {
    let deadline = Instant::now() + Duration::from_secs(5);
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
fn fetched_script_runs_real_install_and_activate_events() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&requests);
    let mut webview = WebViewBuilder::new()
        .script_source_fetcher(Arc::new(move |page, script| {
            request_log.lock().unwrap().push((page.to_string(), script.to_string()));
            Ok("addEventListener('install', event => {
                    event.waitUntil(Promise.resolve());
                });
                addEventListener('activate', event => {
                    event.waitUntil(Promise.resolve());
                });"
            .to_string())
        }))
        .build();

    let id = webview
        .register_service_worker_runtime("./sw.js", Some("./app/"), "https://example.test/page/index.html")
        .unwrap();
    wait_for_state(&mut webview, id, ServiceWorkerState::Activated);

    let registration = webview.service_worker_runtime_registration(id).unwrap();
    assert_eq!(registration.script_url, "https://example.test/page/sw.js");
    assert_eq!(registration.scope, "https://example.test/page/app/");
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        &[(
            "https://example.test/page/index.html".to_string(),
            "https://example.test/page/sw.js".to_string(),
        )]
    );
}

#[test]
fn default_scope_uses_script_directory() {
    let mut webview = WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_, _| Ok(String::new())))
        .build();
    let id = webview
        .register_service_worker_runtime("/workers/sw.js", None, "https://example.test/app/page.html")
        .unwrap();
    wait_for_state(&mut webview, id, ServiceWorkerState::Activated);
    assert_eq!(
        webview.service_worker_runtime_registration(id).unwrap().scope,
        "https://example.test/workers/"
    );
}

#[test]
fn rejected_install_marks_registration_redundant() {
    let mut webview = WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('install', event => {
                    event.waitUntil(Promise.reject(new Error('no install')));
                });"
            .to_string())
        }))
        .build();
    let id = webview
        .register_service_worker_runtime("/sw.js", Some("/"), "https://example.test/page.html")
        .unwrap();
    wait_for_state(&mut webview, id, ServiceWorkerState::Redundant);
}

#[test]
fn insecure_and_cross_origin_registration_fail_before_fetch() {
    let fetch_count = Arc::new(Mutex::new(0usize));
    let count = Arc::clone(&fetch_count);
    let mut webview = WebViewBuilder::new()
        .script_source_fetcher(Arc::new(move |_, _| {
            *count.lock().unwrap() += 1;
            Ok(String::new())
        }))
        .build();

    assert!(
        webview
            .register_service_worker_runtime("/sw.js", None, "http://example.test/page.html",)
            .is_err()
    );
    assert!(
        webview
            .register_service_worker_runtime("https://other.test/sw.js", None, "https://example.test/page.html",)
            .is_err()
    );
    assert!(
        webview
            .register_service_worker_runtime("/sw.js", Some("/app/#fragment"), "https://example.test/page.html",)
            .is_err()
    );
    assert_eq!(*fetch_count.lock().unwrap(), 0);
}
