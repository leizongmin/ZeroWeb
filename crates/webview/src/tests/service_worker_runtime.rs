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

#[test]
fn navigator_register_projects_real_manager_state() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('install', event => {
                    event.waitUntil(Promise.resolve());
                });
                addEventListener('activate', event => {
                    event.waitUntil(Promise.resolve());
                });"
            .to_string())
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__swResult = 'pending';
             globalThis.__swReady = 'pending';
             navigator.serviceWorker.ready.then(function() {
               globalThis.__swReady = 'ready';
             });
             navigator.serviceWorker.register('/sw.js', {scope:'/app/'}).then(
               function(reg) {
                 globalThis.__swReg = reg;
                 globalThis.__swResult = 'resolved';
               },
               function(error) {
                 globalThis.__swResult = 'rejected:' + error;
               }
             );
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview
            .execute_script(
                "[
                   globalThis.__swResult,
                   globalThis.__swReady,
                   globalThis.__swReg && globalThis.__swReg.scope,
                   globalThis.__swReg && globalThis.__swReg.active && globalThis.__swReg.active.state,
                   navigator.serviceWorker.controller === null
                 ].join('|')",
            )
            .unwrap();
        if value == "resolved|ready|https://example.test/app/|activated|true" {
            break;
        }
        assert!(Instant::now() < deadline, "page registration did not activate: {value}");
        std::thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(
        webview
            .execute_script(
                "globalThis.__swReg.unregister().then(function(value) {
                   globalThis.__swUnregistered = String(value);
                 });
                 'unregistering';",
            )
            .unwrap(),
        "unregistering"
    );
    assert_eq!(webview.execute_script("globalThis.__swUnregistered").unwrap(), "true");
}

#[test]
fn navigator_lifecycle_events_preserve_state_and_slot_task_order() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| Ok(String::new())))
        .build();

    webview
        .execute_script(
            "globalThis.__swEvents = [];
             globalThis.__swEventsDone = false;
             navigator.serviceWorker.register('/sw.js', {scope:'/events/'}).then(function(reg) {
               var worker = reg.installing;
               globalThis.__swBrands =
                 String(worker instanceof ServiceWorker) + '|' +
                 String(reg instanceof ServiceWorkerRegistration) + '|' +
                 String(worker instanceof EventTarget) + '|' +
                 String(reg instanceof EventTarget);
               function slots() {
                 return (reg.installing === worker ? 'I' : '-') +
                   (reg.waiting === worker ? 'W' : '-') +
                   (reg.active === worker ? 'A' : '-');
               }
               reg.addEventListener('updatefound', function(event) {
                 globalThis.__swEvents.push(
                   'updatefound:' + worker.state + ':' + slots() + ':' +
                   String(event.target === reg && event.currentTarget === reg &&
                     event instanceof Event));
               });
               worker.addEventListener('statechange', function(event) {
                 globalThis.__swEvents.push(
                   worker.state + ':' + slots() + ':' +
                   String(event.target === worker && event.currentTarget === worker &&
                     event instanceof Event));
                 if (worker.state === 'activated') reg.unregister();
                 if (worker.state === 'redundant') globalThis.__swEventsDone = true;
               });
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if webview.execute_script("String(globalThis.__swEventsDone)").unwrap() == "true" {
            break;
        }
        assert!(Instant::now() < deadline, "Service Worker lifecycle events timed out");
        std::thread::sleep(Duration::from_millis(5));
    }

    assert_eq!(
        webview.execute_script("globalThis.__swBrands").unwrap(),
        "true|true|true|true"
    );
    assert_eq!(
        webview.execute_script("globalThis.__swEvents.join('|')").unwrap(),
        "updatefound:installing:I--:true|installed:-W-:true|activating:--A:true|\
         activated:--A:true|redundant:---:true"
    );
}

#[test]
fn navigator_register_rejects_script_compile_failure() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| Ok("function(".to_string())))
        .build();

    webview
        .execute_script(
            "globalThis.__swFailure = 'pending';
             navigator.serviceWorker.register('/bad-sw.js').then(
               function() { globalThis.__swFailure = 'unexpected success'; },
               function(error) { globalThis.__swFailure = 'rejected:' + error.name; }
             );
             'started';",
        )
        .unwrap();
    assert_eq!(
        webview.execute_script("globalThis.__swFailure").unwrap(),
        "rejected:TypeError"
    );
}

#[test]
fn navigator_replacement_reuses_registration_identity_for_scope() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| Ok(String::new())))
        .build();

    webview
        .execute_script(
            "globalThis.__identity = 'pending';
             navigator.serviceWorker.register('/sw-v1.js', {scope:'/app/'}).then(function(first) {
               globalThis.__firstReg = first;
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__firstWorker = globalThis.__firstReg.active;
               return navigator.serviceWorker.register('/sw-v2.js', {scope:'/app/'});
             }).then(function(second) {
               var versionIdentity =
                 String(second.installing !== globalThis.__firstWorker) + '|' +
                 String(second.installing.scriptURL.endsWith('/sw-v2.js')) + '|' +
                 String(second.active === globalThis.__firstWorker);
               return navigator.serviceWorker.getRegistrations().then(function(all) {
                 globalThis.__identity = String(second === globalThis.__firstReg) + '|' +
                   String(all.length) + '|' + String(all[0] === globalThis.__firstReg) + '|' +
                   versionIdentity;
               });
             }, function(error) {
               globalThis.__identity = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview.execute_script("globalThis.__identity").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|1|true|true|true|true");
            break;
        }
        assert!(Instant::now() < deadline, "replacement registration did not settle");
        std::thread::sleep(Duration::from_millis(10));
    }
}
