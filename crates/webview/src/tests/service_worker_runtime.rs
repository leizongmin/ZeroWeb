use crate::{WebViewBuilder, WebViewConfig};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zero_engine::fetch_bridge::FetchResponse;
use zero_storage::ServiceWorkerState;

fn wait_for_state(webview: &mut crate::WebView, registration_id: u64, expected: ServiceWorkerState) {
    let deadline = Instant::now() + Duration::from_secs(60);
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
#[serial_test::serial(service_worker_runtime)]
fn embedded_main_script_request_carries_service_worker_metadata() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0_u8; 2048];
        let size = stream.read(&mut bytes).unwrap();
        let request = String::from_utf8_lossy(&bytes[..size]).to_ascii_lowercase();
        assert!(request.starts_with("get /sw.js "));
        assert!(request.contains("\r\nservice-worker: script\r\n"));
        assert!(request.contains("\r\nsec-fetch-mode: same-origin\r\n"));
        assert!(request.contains("\r\ncache-control: no-cache\r\n"));
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\n\
                  Content-Length: 0\r\n\r\n",
            )
            .unwrap();
    });

    let mut webview = WebViewBuilder::new().build();
    let document_url = format!("{origin}/page.html");
    let script_url = format!("{origin}/sw.js");
    webview
        .register_service_worker_runtime(&script_url, Some("/"), &document_url)
        .unwrap();
    server.join().unwrap();
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn embedded_network_main_script_rejects_non_javascript_mime() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0_u8; 2048];
        let size = stream.read(&mut bytes).unwrap();
        assert!(String::from_utf8_lossy(&bytes[..size]).starts_with("GET /sw.js "));
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\
                  Content-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
    });

    let mut webview = WebViewBuilder::new().build();
    let document_url = format!("{origin}/page.html");
    let script_url = format!("{origin}/sw.js");
    let error = webview
        .register_service_worker_runtime(&script_url, Some("/"), &document_url)
        .unwrap_err();
    server.join().unwrap();
    assert!(error.to_string().contains("unsupported MIME type text/html"));
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn classic_page_script_async_function_is_visible_to_later_script() {
    let mut webview = WebViewBuilder::new().build();
    webview.load_html(
        "<script>async function sharedHelper() { return 1; }</script>\
         <script>globalThis.__sharedHelperType = typeof sharedHelper;</script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();
    assert_eq!(
        webview.execute_script("globalThis.__sharedHelperType").unwrap(),
        "function"
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn lifecycle_imports_use_persistent_worker_fetch_context() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&requests);
    let mut webview = WebViewBuilder::new()
        .script_source_fetcher(Arc::new(move |context, script| {
            request_log
                .lock()
                .unwrap()
                .push((context.to_string(), script.to_string()));
            match script {
                "https://example.test/sw.js" => Ok("importScripts('/shared-import.js');
                     addEventListener('install', () => {
                       importScripts('/install-import.js');
                       if (globalThis.installImported !== true) throw new Error('install import missing');
                     });
                     addEventListener('activate', () => {
                       globalThis.sharedImported = null;
                       globalThis.installImported = null;
                       importScripts('/shared-import.js', '/install-import.js');
                       if (globalThis.sharedImported !== true || globalThis.installImported !== true) {
                         throw new Error('activate import replay missing');
                       }
                     });"
                .to_string()),
                "https://example.test/shared-import.js" => Ok("globalThis.sharedImported = true;".to_string()),
                "https://example.test/install-import.js" => Ok("globalThis.installImported = true;".to_string()),
                _ => Err(format!("unexpected script: {script}")),
            }
        }))
        .build();

    let id = webview
        .register_service_worker_runtime("/sw.js", Some("/"), "https://example.test/page.html")
        .unwrap();
    wait_for_state(&mut webview, id, ServiceWorkerState::Activated);

    assert_eq!(
        requests.lock().unwrap().as_slice(),
        &[
            (
                "https://example.test/page.html".to_string(),
                "https://example.test/sw.js".to_string(),
            ),
            (
                "https://example.test/sw.js".to_string(),
                "https://example.test/shared-import.js".to_string(),
            ),
            (
                "https://example.test/sw.js".to_string(),
                "https://example.test/install-import.js".to_string(),
            ),
        ]
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
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
#[serial_test::serial(service_worker_runtime)]
fn navigator_register_update_via_cache_noop_keeps_active_projection() {
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
            "globalThis.__swUpdateViaCacheNoop = 'pending';
             (async function () {
               var first = await navigator.serviceWorker.register('/sw.js', {scope: '/app/'});
               await navigator.serviceWorker.ready;
               var second = await navigator.serviceWorker.register('/sw.js', {
                 scope: '/app/',
                 updateViaCache: 'all'
               });
               globalThis.__swUpdateViaCacheNoop = [
                 String(first === second),
                 second.updateViaCache,
                 second.installing === null ? 'null' : second.installing.state,
                 second.waiting === null ? 'null' : second.waiting.state,
                 second.active && second.active.state
               ].join('|');
             })().catch(function(error) {
               globalThis.__swUpdateViaCacheNoop = 'error:' + error;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview.execute_script("globalThis.__swUpdateViaCacheNoop").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|all|null|null|activated");
            break;
        }
        assert!(Instant::now() < deadline, "updateViaCache no-op registration timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn unregistered_page_registration_stops_state_polling() {
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
            "globalThis.__zw_timers = [];
             globalThis.__zw_setTimeout = function(id, delay) {
               globalThis.__zw_timers.push({id: id, at: Date.now() + (delay | 0)});
             };
             globalThis.__zw_fire_due_timers = function() {
               var timers = globalThis.__zw_timers || [];
               if (!timers.length) return;
               var due = timers.shift();
               globalThis.__zw_timers = timers;
               var fn = globalThis.__zw_pending[due.id];
               if (fn) {
                 delete globalThis.__zw_pending[due.id];
                 try { fn(); } catch (_e) {}
               }
             };
             globalThis.__swUnregisterPoll = 'pending';
             (async function () {
               var registration = await navigator.serviceWorker.register('/sw.js', {scope: '/app/'});
               await navigator.serviceWorker.ready;
               await registration.unregister();
               globalThis.__swUnregisterPoll = [
                 registration.updateViaCache,
                 registration.active && registration.active.state
               ].join('|');
             })().catch(function(error) {
               globalThis.__swUnregisterPoll = 'error:' + error.name + ':' + error.message;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview
            .execute_script(
                "if (typeof globalThis.__zw_fire_due_timers === 'function') {
                   globalThis.__zw_fire_due_timers();
                 }
                 String(globalThis.__swUnregisterPoll)",
            )
            .unwrap();
        if value != "pending" {
            assert_eq!(value, "imports|activated");
            break;
        }
        assert!(Instant::now() < deadline, "registration unregister timed out");
        std::thread::sleep(Duration::from_millis(10));
    }

    let pending = webview
        .execute_script(
            "for (var i = 0; i < 4; i++) globalThis.__zw_fire_due_timers();
             Object.keys(globalThis.__zw_pending || {}).filter(function(key) {
               return key.indexOf('__zwtid:') === 0;
             }).length",
        )
        .unwrap();
    assert_eq!(pending, "0");
}

#[test]
#[serial_test::serial(service_worker_runtime)]
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
#[serial_test::serial(service_worker_runtime)]
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
            .register_service_worker_runtime("/sw.js", Some("/app%2fchild"), "https://example.test/page.html",)
            .is_err()
    );
    assert_eq!(*fetch_count.lock().unwrap(), 0);
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_module_registration_fetches_static_graph_and_activates() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&requests);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |context, script| {
            request_log
                .lock()
                .unwrap()
                .push((context.to_string(), script.to_string()));
            match script {
                "https://example.test/module-sw.js" => Ok("import { value } from './lib/value.js';
                     if (value !== 7) throw new Error('wrong module value');"
                    .to_string()),
                "https://example.test/lib/value.js" => Ok("export const value = 7;".to_string()),
                _ => Err(format!("unexpected script: {script}")),
            }
        }))
        .build();
    webview
        .execute_script(
            "globalThis.__moduleRegistrationResult = 'pending';
             navigator.serviceWorker.register('/module-sw.js', {type:'module'}).then(
               registration => {
                 globalThis.__moduleRegistrationResult = 'ok:' + registration._id + ':' + registration.scope;
               },
               error => { globalThis.__moduleRegistrationResult = error.name + ':' + error.message; });",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let _ = webview.poll_service_worker_runtime_events();
        if webview.execute_script("globalThis.__moduleRegistrationResult").unwrap() != "pending" {
            break;
        }
        assert!(Instant::now() < deadline, "module registration promise timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    let result = webview.execute_script("globalThis.__moduleRegistrationResult").unwrap();
    let parts = result.splitn(3, ':').collect::<Vec<_>>();
    assert_eq!(parts[0], "ok", "{result}");
    assert_eq!(parts[2], "https://example.test/", "{result}");
    let registration_id = parts[1].parse::<u64>().expect("registration id");
    wait_for_state(&mut webview, registration_id, ServiceWorkerState::Activated);
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        &[
            (
                "https://example.test/page.html".to_string(),
                "https://example.test/module-sw.js".to_string(),
            ),
            (
                "https://example.test/module-sw.js".to_string(),
                "https://example.test/lib/value.js".to_string(),
            ),
        ]
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
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
             navigator.serviceWorker.register('/sw.js', {
               scope:'/app/',
               updateViaCache:'none'
             }).then(
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
                   globalThis.__swReg && globalThis.__swReg.updateViaCache,
                   globalThis.__swReg && globalThis.__swReg.active && globalThis.__swReg.active.state,
                   globalThis.__swReg && Object.prototype.toString.call(globalThis.__swReg),
                   globalThis.__swReg && globalThis.__swReg.active &&
                     Object.prototype.toString.call(globalThis.__swReg.active),
                   navigator.serviceWorker.controller === null
                 ].join('|')",
            )
            .unwrap();
        if value
            == "resolved|ready|https://example.test/app/|none|activated|\
                     [object ServiceWorkerRegistration]|[object ServiceWorker]|true"
        {
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
#[serial_test::serial(service_worker_runtime)]
fn navigator_register_executes_imported_classic_scripts_in_order() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&requests);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, script| {
            request_log.lock().unwrap().push(script.to_string());
            match script {
                "https://example.test/workers/sw.js" => Ok("importScripts('./first.js#ignored', '/shared/second.js');
                     addEventListener('install', () => {
                       if (globalThis.importOrder !== 'first,second') throw new Error('wrong import order');
                     });"
                .into()),
                "https://example.test/workers/first.js" => {
                    Ok("globalThis.importOrder = 'first'; var importedBinding = 7;".into())
                }
                "https://example.test/shared/second.js" => Ok(
                    "if (globalThis.importedBinding !== 7) throw new Error('missing first import');
                     globalThis.importOrder += ',second';"
                        .into(),
                ),
                _ => Err(format!("unexpected script URL: {script}")),
            }
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__importStage = 'pending';
             navigator.serviceWorker.register('/workers/sw.js').then(function(reg) {
               return navigator.serviceWorker.ready.then(function() {
                 globalThis.__importStage = reg.active.state;
               });
             }, function(error) {
               globalThis.__importStage = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();
    // 组合态高负载下 SW 激活 promise 链推进可慢于单次轮询——同 e55038a2a 的
    // deadline 轮询形态（本测试此前无轮询，属该修复的漏改补片）。
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let stage = webview.execute_script("globalThis.__importStage").unwrap();
        if stage == "activated" || stage.starts_with("error:") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "import order registration timed out: {stage}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(webview.execute_script("globalThis.__importStage").unwrap(), "activated");
    assert_eq!(
        requests.lock().unwrap().as_slice(),
        [
            "https://example.test/workers/sw.js",
            "https://example.test/workers/first.js",
            "https://example.test/shared/second.js",
        ]
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn structured_import_response_allows_cross_origin_without_cors_headers() {
    let mut webview = WebViewBuilder::new()
        .service_worker_script_fetcher(Arc::new(|_, script| {
            let body = match script {
                "https://example.test/sw.js" => {
                    "importScripts('https://cdn.test/dependency.js'); globalThis.loaded = imported;"
                }
                "https://cdn.test/dependency.js" => "globalThis.imported = true;",
                _ => return Err(format!("unexpected script URL: {script}")),
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "text/javascript".into())],
                body: body.as_bytes().to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        }))
        .build();

    let id = webview
        .register_service_worker_runtime("/sw.js", None, "https://example.test/page.html")
        .unwrap();
    wait_for_state(&mut webview, id, ServiceWorkerState::Activated);
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn module_import_response_rejects_cross_origin_without_cors_headers() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .service_worker_script_fetcher(Arc::new(|_, script| {
            let body = match script {
                "https://example.test/sw.js" => "import 'https://cdn.test/dependency.js';",
                "https://cdn.test/dependency.js" => "export const value = 1;",
                _ => return Err(format!("unexpected script URL: {script}")),
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "text/javascript".into())],
                body: body.as_bytes().to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__moduleCorsResult = 'pending';
             navigator.serviceWorker.register('/sw.js', {type:'module'}).then(
               () => { globalThis.__moduleCorsResult = 'unexpected-success'; },
               error => { globalThis.__moduleCorsResult = String(error.message); });",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let _ = webview.poll_service_worker_runtime_events();
        if webview.execute_script("globalThis.__moduleCorsResult").unwrap() != "pending" {
            break;
        }
        assert!(Instant::now() < deadline, "module CORS rejection timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        webview
            .execute_script("globalThis.__moduleCorsResult")
            .unwrap()
            .contains("failed CORS validation")
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn structured_import_response_rejects_non_javascript_mime() {
    let mut webview = WebViewBuilder::new()
        .service_worker_script_fetcher(Arc::new(|_, script| {
            let (body, mime) = if script == "https://example.test/sw.js" {
                ("importScripts('/dependency.js');", "text/javascript")
            } else {
                ("globalThis.loaded = true;", "text/plain")
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), mime.into())],
                body: body.as_bytes().to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        }))
        .build();

    let error = webview
        .register_service_worker_runtime("/sw.js", None, "https://example.test/page.html")
        .unwrap_err();
    assert!(error.to_string().contains("unsupported MIME type text/plain"));
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_update_rejects_non_javascript_main_script_as_security_error() {
    let visits = Arc::new(Mutex::new(0usize));
    let fetch_visits = Arc::clone(&visits);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .service_worker_script_fetcher(Arc::new(move |_, script| {
            let mut visits = fetch_visits.lock().unwrap();
            *visits += 1;
            let mime = if *visits == 1 {
                "application/javascript"
            } else {
                "text/html"
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), mime.into())],
                body: format!("globalThis.version = {};", *visits).into_bytes(),
                url: script.to_string(),
                redirect_count: 0,
            })
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__mimeUpdate = 'pending';
             navigator.serviceWorker.register('/sw.js', {scope:'/out-of-scope/'}).then(function(reg) {
               return navigator.serviceWorker.ready.then(function() {
                 globalThis.__mimeReg = reg;
                 globalThis.__mimeActive = reg.active;
                 return reg.update();
               });
             }).then(function() {
               globalThis.__mimeUpdate = 'unexpected-success';
             }, function(error) {
               globalThis.__mimeUpdate =
                 error.name + '|' +
                 String(globalThis.__mimeReg.active === globalThis.__mimeActive) + '|' +
                 String(globalThis.__mimeReg.installing === null);
             });",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__mimeUpdate").unwrap();
        if value != "pending" {
            assert_eq!(value, "SecurityError|true|true");
            break;
        }
        assert!(Instant::now() < deadline, "MIME update rejection timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(*visits.lock().unwrap(), 2);
}

#[test]
#[serial_test::serial(service_worker_runtime)]
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

    let deadline = Instant::now() + Duration::from_secs(60);
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
#[serial_test::serial(service_worker_runtime)]
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
#[serial_test::serial(service_worker_runtime)]
fn navigator_registration_normalizes_urls_and_preserves_error_types() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| Ok(String::new())))
        .build();

    webview
        .execute_script(
            "globalThis.__registrationContract = 'pending';
             navigator.serviceWorker.register('/workers/sw.js#script', {
               scope: '/workers/app/#scope'
             }).then(function(registration) {
               var worker = registration.installing || registration.waiting || registration.active;
               var normalized = worker.scriptURL + '|' + registration.scope;
               return registration.unregister().then(function() { return normalized; });
             }).then(function(normalized) {
               return navigator.serviceWorker.register('/workers/sw.js', {scope: null}).then(
                 function() { return 'null-resolved'; },
                 function(error) { return normalized + '|' + error.name; }
               );
             }).then(function(result) {
               return navigator.serviceWorker.register('/workers/sw.js', {
                 scope: '/workers/app%2fchild'
               }).then(
                 function() { return 'encoded-resolved'; },
                 function(error) { return result + '|' + error.name; }
               );
             }).then(function(result) {
               return navigator.serviceWorker.register('/workers/sw.js', {
                 updateViaCache: 'invalid'
               }).then(
                 function() { return 'cache-mode-resolved'; },
                 function(error) { return result + '|' + error.name; }
               );
             }).then(function(result) {
               return navigator.serviceWorker.register('https://other.test/sw.js').then(
                 function() { return 'cross-origin-resolved'; },
                 function(error) {
                   globalThis.__registrationContract = result + '|' + error.name + '|' +
                     String(error instanceof DOMException) + '|' + String(error instanceof Error);
                 }
               );
             }, function(error) {
               globalThis.__registrationContract = 'unexpected:' + error;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview.execute_script("globalThis.__registrationContract").unwrap();
        if value != "pending" {
            assert_eq!(
                value,
                "https://example.test/workers/sw.js|https://example.test/workers/app/|\
                 SecurityError|TypeError|TypeError|SecurityError|true|true"
            );
            break;
        }
        assert!(Instant::now() < deadline, "registration contract timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
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
               globalThis.__identityStage = 'first-registered';
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__identityStage = 'first-ready';
               globalThis.__firstWorker = globalThis.__firstReg.active;
               return navigator.serviceWorker.register('/sw-v2.js', {scope:'/app/'});
             }).then(function(second) {
               globalThis.__identityStage = 'second-registered:' +
                 String(second.installing && second.installing.state);
               var installing = second.installing;
               var versionIdentity =
                 String(!!installing) + '|' +
                 String(installing !== globalThis.__firstWorker) + '|' +
                 String(!!installing && installing.scriptURL.endsWith('/sw-v2.js')) + '|' +
                 String(second.active === globalThis.__firstWorker);
               return navigator.serviceWorker.getRegistrations().then(function(all) {
                 globalThis.__identity = String(second === globalThis.__firstReg) + '|' +
                   String(all.length) + '|' + String(all[0] === globalThis.__firstReg) + '|' +
                   versionIdentity;
               });
             }, function(error) {
               globalThis.__identity = 'error:' + String(error);
             }).catch(function(error) {
               globalThis.__identity = 'throw:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__identity").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|1|true|true|true|true|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "replacement registration did not settle: {}",
            webview.execute_script("String(globalThis.__identityStage)").unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_update_compares_script_bytes_and_dispatches_updatefound_only_when_changed() {
    let source = Arc::new(Mutex::new("globalThis.version = 1;".to_string()));
    let fetch_source = Arc::clone(&source);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, _| Ok(fetch_source.lock().unwrap().clone())))
        .build();

    webview
        .execute_script(
            "globalThis.__updateStage = 'pending';
             navigator.serviceWorker.register('/sw.js', {scope:'/update/'}).then(function(reg) {
               globalThis.__updateReg = reg;
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__updateStage = 'ready';
             }, function(error) {
               globalThis.__updateStage = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while webview.execute_script("globalThis.__updateStage").unwrap() == "pending" {
        assert!(Instant::now() < deadline, "initial update registration timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(webview.execute_script("globalThis.__updateStage").unwrap(), "ready");

    webview
        .execute_script(
            "globalThis.__updateFound = 0;
             globalThis.__updateReg.addEventListener('updatefound', function() {
               globalThis.__updateFound++;
             });
             globalThis.__updateNoop = 'pending';
             globalThis.__updateReg.update().then(function(reg) {
               globalThis.__updateNoop =
                 String(reg === globalThis.__updateReg) + '|' +
                 String(reg.active === globalThis.__updateReg.active);
             }, function(error) {
               globalThis.__updateNoop = 'error:' + String(error);
             });
             'updating';",
        )
        .unwrap();
    assert_eq!(webview.execute_script("globalThis.__updateNoop").unwrap(), "true|true");
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(webview.execute_script("String(globalThis.__updateFound)").unwrap(), "0");

    *source.lock().unwrap() = "globalThis.version = 2;".to_string();
    webview
        .execute_script(
            "globalThis.__updateChanged = 'pending';
             globalThis.__updateActive = globalThis.__updateReg.active;
             globalThis.__updateReg.update().then(function(reg) {
               globalThis.__updateChanged =
                 String(reg === globalThis.__updateReg) + '|' +
                 String(!!reg.installing) + '|' +
                 String(reg.active === globalThis.__updateActive);
             }, function(error) {
               globalThis.__updateChanged = 'error:' + String(error);
             });
             'updating';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview
            .execute_script(
                "[
                   globalThis.__updateChanged,
                   globalThis.__updateFound,
                   !!globalThis.__updateReg.waiting,
                   globalThis.__updateReg.active === globalThis.__updateActive
                 ].join('|')",
            )
            .unwrap();
        if value == "true|true|true|1|false|false" {
            break;
        }
        assert!(Instant::now() < deadline, "changed update timed out: {value}");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        webview.execute_script("globalThis.__updateActive.state").unwrap(),
        "redundant"
    );

    *source.lock().unwrap() = "function(".to_string();
    webview
        .execute_script(
            "globalThis.__updateFailure = 'pending';
             globalThis.__updateActive = globalThis.__updateReg.active;
             globalThis.__updateWaiting = globalThis.__updateReg.waiting;
             globalThis.__updateReg.update().then(function() {
               globalThis.__updateFailure = 'unexpected success';
             }, function(error) {
               globalThis.__updateFailure =
                 error.name + '|' +
                 String(globalThis.__updateReg.active === globalThis.__updateActive) + '|' +
                 String(globalThis.__updateReg.waiting === globalThis.__updateWaiting) + '|' +
                 String(globalThis.__updateFound);
             });
             'updating';",
        )
        .unwrap();
    assert_eq!(
        webview.execute_script("globalThis.__updateFailure").unwrap(),
        "TypeError|true|true|1"
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_update_succeeds_while_initial_worker_is_installing() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('install', event => event.waitUntil(Promise.resolve()));".into())
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__installingUpdate = 'pending';
             navigator.serviceWorker.register('/sw.js', {scope:'/installing/'}).then(function(reg) {
               var worker = reg.installing;
               return reg.update().then(function(updated) {
                 globalThis.__installingUpdate =
                   String(updated === reg) + '|' +
                   String(updated.installing === worker) + '|' +
                   String(worker.state);
               });
             }, function(error) {
               globalThis.__installingUpdate = 'error:' + error.name;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__installingUpdate").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|true|installing");
            break;
        }
        assert!(Instant::now() < deadline, "installing update timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_update_activates_replacement_without_a_controlled_client() {
    let source = Arc::new(Mutex::new("globalThis.version = 1;".to_string()));
    let fetch_source = Arc::clone(&source);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, _| Ok(fetch_source.lock().unwrap().clone())))
        .build();

    webview
        .execute_script(
            "globalThis.__uncontrolledUpdate = 'pending';
             navigator.serviceWorker.register('/sw.js', {scope:'/out-of-scope/'}).then(function(reg) {
               globalThis.__uncontrolledReg = reg;
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__uncontrolledActive = globalThis.__uncontrolledReg.active;
               globalThis.__uncontrolledUpdate = 'ready';
             });",
        )
        .unwrap();
    assert_eq!(
        webview.execute_script("globalThis.__uncontrolledUpdate").unwrap(),
        "ready"
    );

    *source.lock().unwrap() = "globalThis.version = 2;".to_string();
    webview
        .execute_script(
            "globalThis.__uncontrolledReg.update().then(function() {
               globalThis.__uncontrolledUpdate = 'updated';
             });",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview
            .execute_script(
                "[
                   globalThis.__uncontrolledUpdate,
                   globalThis.__uncontrolledReg.active !== globalThis.__uncontrolledActive,
                   globalThis.__uncontrolledReg.active &&
                     globalThis.__uncontrolledReg.active.state,
                   globalThis.__uncontrolledActive.state
                 ].join('|')",
            )
            .unwrap();
        if value == "updated|true|activated|redundant" {
            break;
        }
        assert!(Instant::now() < deadline, "uncontrolled update timed out: {value}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_update_detects_imported_script_byte_changes() {
    let dependency_version = Arc::new(Mutex::new(1u32));
    let fetch_version = Arc::clone(&dependency_version);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, script| {
            if script.ends_with("/sw.js") {
                Ok("importScripts('./dependency.js');".into())
            } else if script.ends_with("/dependency.js") {
                Ok(format!(
                    "globalThis.dependencyVersion = {};",
                    *fetch_version.lock().unwrap()
                ))
            } else {
                Err(format!("unexpected script URL: {script}"))
            }
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__graphStage = 'pending';
             navigator.serviceWorker.register('/sw.js').then(function(reg) {
               globalThis.__graphReg = reg;
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__graphStage = 'ready';
             });
             'started';",
        )
        .unwrap();
    assert_eq!(webview.execute_script("globalThis.__graphStage").unwrap(), "ready");
    webview
        .execute_script(
            "globalThis.__graphUpdates = 0;
             globalThis.__graphReg.addEventListener('updatefound', function() {
               globalThis.__graphUpdates++;
             });",
        )
        .unwrap();

    webview
        .execute_script(
            "globalThis.__graphNoop = 'pending';
             globalThis.__graphReg.update().then(function(reg) {
               globalThis.__graphNoop = String(reg === globalThis.__graphReg);
             });
             'updating';",
        )
        .unwrap();
    assert_eq!(webview.execute_script("globalThis.__graphNoop").unwrap(), "true");
    assert_eq!(
        webview.execute_script("String(globalThis.__graphUpdates)").unwrap(),
        "0"
    );

    *dependency_version.lock().unwrap() = 2;
    webview
        .execute_script(
            "globalThis.__graphChanged = 'pending';
             globalThis.__graphReg.update().then(function(reg) {
               globalThis.__graphChanged =
                 String(reg === globalThis.__graphReg) + '|' + String(!!reg.installing);
             });
             'updating';",
        )
        .unwrap();
    assert_eq!(
        webview.execute_script("globalThis.__graphChanged").unwrap(),
        "true|true"
    );
    assert_eq!(
        webview.execute_script("String(globalThis.__graphUpdates)").unwrap(),
        "1"
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_skip_waiting_activates_replacement_version() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, script| {
            if script.ends_with("/sw-v2.js") {
                Ok("addEventListener('install', event => {
                    event.waitUntil(skipWaiting());
                });"
                .to_string())
            } else {
                Ok(String::new())
            }
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__skipWaitingResult = 'pending';
             navigator.serviceWorker.register('/sw-v1.js', {scope:'/'}).then(function(reg) {
               return navigator.serviceWorker.ready.then(function() {
                 globalThis.__skipWaitingReg = reg;
                 globalThis.__skipWaitingFirst = reg.active;
                 return navigator.serviceWorker.register('/sw-v2.js', {scope:'/'});
               });
             }).then(function(reg) {
               globalThis.__skipWaitingSameReg =
                 String(reg === globalThis.__skipWaitingReg);
             }, function(error) {
               globalThis.__skipWaitingResult = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview
            .execute_script(
                "var reg = globalThis.__skipWaitingReg;
                 if (globalThis.__skipWaitingResult !== 'pending') {
                   globalThis.__skipWaitingResult;
                 } else if (reg && reg.active &&
                            reg.active.scriptURL.endsWith('/sw-v2.js') &&
                            globalThis.__skipWaitingFirst.state === 'redundant') {
                   globalThis.__skipWaitingResult =
                     globalThis.__skipWaitingSameReg + '|activated|redundant';
                 } else {
                   'pending';
                 }",
            )
            .unwrap();
        if value != "pending" {
            assert_eq!(value, "true|activated|redundant");
            break;
        }
        assert!(Instant::now() < deadline, "skipWaiting replacement timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_controller_tracks_document_and_skip_waiting_replacement() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, script| {
            if script.ends_with("/sw-v2.js") {
                Ok("addEventListener('install', event => {
                    event.waitUntil(skipWaiting());
                });"
                .to_string())
            } else {
                Ok(String::new())
            }
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__controllerSetup = 'pending';
             navigator.serviceWorker.register('/app/sw-v1.js', {scope:'/app/'}).then(function() {
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__controllerSetup =
                 String(navigator.serviceWorker.controller === null);
             }, function(error) {
               globalThis.__controllerSetup = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__controllerSetup").unwrap();
        if value != "pending" {
            assert_eq!(value, "true");
            break;
        }
        assert!(Instant::now() < deadline, "initial controller setup timed out");
        std::thread::sleep(Duration::from_millis(10));
    }

    webview.load_url("https://example.test/app/page.html");
    webview.complete_load("<html><body></body></html>", None);
    assert_eq!(
        webview
            .execute_script(
                "[
                   navigator.serviceWorker.controller &&
                     navigator.serviceWorker.controller.scriptURL.endsWith('/app/sw-v1.js'),
                   navigator.serviceWorker.controller &&
                     navigator.serviceWorker.controller.state,
                   navigator.serviceWorker instanceof EventTarget
                 ].join('|')",
            )
            .unwrap(),
        "true|activated|true"
    );

    webview
        .execute_script(
            "globalThis.__controllerChange = 'pending';
             globalThis.__firstController = navigator.serviceWorker.controller;
             navigator.serviceWorker.addEventListener('controllerchange', function(event) {
               // R381 flaky 修复：事件事实（target/currentTarget/instanceof/scriptURL/
               // controller.state）在事件点即时定格；旧 controller 的 redundant 转换
               // 走宿主 setTimeout 线程（每 timer 一个 OS 线程，无 FIFO 保证），与
               // controllerchange 事件派发是**线程级竞争**（flaky 观测到 redundant 与
               // activated 双态）。契约在 __firstController.state 变 'redundant' 后
               // 才 finalize（setInterval 轮询，bounded）——spec 语义冗余态必然到达，
               // 只是时点不与 controllerchange 构成 happens-before；20s 宿主 deadline
               // 兜底（Rust 侧同款 deadline）。
               var eventFacts = [
                 event.target === navigator.serviceWorker,
                 event.currentTarget === navigator.serviceWorker,
                 event.target instanceof ServiceWorkerContainer,
                 navigator.serviceWorker.controller.scriptURL.endsWith('/app/sw-v2.js'),
                 navigator.serviceWorker.controller.state
               ].join('|');
               var polls = 0;
               var iv = setInterval(function () {
                 polls++;
                 if (globalThis.__firstController.state === 'redundant' || polls > 2000) {
                   clearInterval(iv);
                   globalThis.__controllerChange =
                     eventFacts + '|' + globalThis.__firstController.state;
                 }
               }, 5);
             });
             navigator.serviceWorker.register('/app/sw-v2.js', {scope:'/app/'}).catch(function(error) {
               globalThis.__controllerChange = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__controllerChange").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|true|true|true|activating|redundant");
            break;
        }
        assert!(Instant::now() < deadline, "controllerchange timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn iframe_controller_tracks_skip_waiting_replacement() {
    const PAGE_URL: &str =
        "https://example.test/service-workers/service-worker/skip-waiting-using-registration.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            let body = if script.ends_with("/skip-waiting-worker.js") {
                "addEventListener('install', event => {
                   event.waitUntil(skipWaiting());
                 });
                 addEventListener('message', event => {
                   event.source.postMessage({type:'reply', script:self.location.href});
                 });"
            } else {
                ""
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: body.as_bytes().to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| {
            let body = if request
                .url
                .ends_with("/resources/blank.html?skip-waiting-using-registration")
            {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load("<html><body></body></html>", None);
    webview.run_page_scripts_strict().unwrap();
    webview
        .execute_script(
            "globalThis.__iframeSetup = 'pending';
             navigator.serviceWorker.register('resources/empty.js', {
               scope:'resources/blank.html?skip-waiting-using-registration'
             }).then(function(reg) {
               return navigator.serviceWorker.ready;
             }).then(function() {
               globalThis.__iframeSetup = 'ready';
             }).catch(function(error) {
               globalThis.__iframeSetup = 'error:' + error.name + ':' + error.message;
             });
             'started';",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("String(globalThis.__iframeSetup)").unwrap();
        if value != "pending" {
            assert_eq!(value, "ready");
            break;
        }
        assert!(Instant::now() < deadline, "iframe setup timed out");
        std::thread::sleep(Duration::from_millis(10));
    }

    webview.complete_load(
        "<script>
           globalThis.__iframeSkipWaiting = 'pending';
           Promise.resolve().then(function() {
               return new Promise(function(resolve) {
                 var frame = document.createElement('iframe');
                 frame.id = 'frame';
                 frame.src = 'resources/blank.html?skip-waiting-using-registration';
                 frame.onload = function() { resolve(frame); };
                 document.body.appendChild(frame);
               });
             }).then(function(frame) {
               var frameSw = frame.contentWindow.navigator.serviceWorker;
               globalThis.__iframeFirstController = frameSw.controller;
               frameSw.oncontrollerchange = function(event) {
                 var raw = JSON.parse(__zw_sw_controller(
                   frame.contentWindow.document._zwURL,
                   frame.contentWindow.document._zwSwClientId
                 )).controller;
                 globalThis.__iframeSkipWaiting = [
                   event.target === frameSw,
                   event.target instanceof frame.contentWindow.ServiceWorkerContainer,
                   frameSw.controller !== globalThis.__iframeFirstController,
                   frameSw.controller._id !== globalThis.__iframeFirstController._id,
                   frameSw.controller.scriptURL.endsWith('/skip-waiting-worker.js'),
                   frameSw.controller.state,
                   raw && raw.id === frameSw.controller._id,
                   raw && raw.scriptURL.endsWith('/skip-waiting-worker.js')
                 ].join('|');
               };
               navigator.serviceWorker.onmessage = function(event) {
                 globalThis.__iframeWorkerMessage =
                   event.data.type + '|' + event.data.script.endsWith('/skip-waiting-worker.js');
               };
               navigator.serviceWorker.register('resources/skip-waiting-worker.js', {
                 scope:'resources/blank.html?skip-waiting-using-registration'
               }).then(function(registration) {
                 globalThis.__iframeReplacement = registration;
               });
             }).catch(function(error) {
               globalThis.__iframeSkipWaiting = 'error:' + error.name + ':' + error.message;
             });
         </script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview
            .execute_script("String(globalThis.__iframeSkipWaiting)")
            .unwrap();
        if value != "pending" {
            assert_eq!(value, "true|true|true|true|true|activating|true|true");
            break;
        }
        if Instant::now() >= deadline {
            let state = webview
                .execute_script(
                    "var frame = document.getElementById('frame');
                     var frameSw = frame && frame.contentWindow.navigator.serviceWorker;
                     var raw = frame && JSON.parse(__zw_sw_controller(
                       frame.contentWindow.document._zwURL,
                       frame.contentWindow.document._zwSwClientId
                     )).controller;
                     JSON.stringify({
                       value: globalThis.__iframeSkipWaiting,
                       first: globalThis.__iframeFirstController && {
                         id: globalThis.__iframeFirstController._id,
                         scriptURL: globalThis.__iframeFirstController.scriptURL,
                         state: globalThis.__iframeFirstController.state
                       },
                       controller: frameSw && frameSw.controller && {
                         id: frameSw.controller._id,
                         scriptURL: frameSw.controller.scriptURL,
                         state: frameSw.controller.state
                       },
                       raw: raw
                     })",
                )
                .unwrap();
            panic!("iframe skipWaiting controllerchange timed out: {state}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    webview
        .execute_script("globalThis.__iframeReplacement.active.postMessage({type:'ping'});")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview
            .execute_script("String(globalThis.__iframeWorkerMessage || 'pending')")
            .unwrap();
        if value != "pending" {
            assert_eq!(value, "reply|true");
            break;
        }
        assert!(Instant::now() < deadline, "iframe replacement worker message timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn navigator_clients_claim_controls_current_matching_document() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/app/page.html")
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('activate', event => {
                    event.waitUntil(clients.claim());
                });"
            .to_string())
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__claimResult = 'pending';
             navigator.serviceWorker.addEventListener('controllerchange', function(event) {
               globalThis.__claimResult = [
                 event.target === navigator.serviceWorker,
                 event.currentTarget === navigator.serviceWorker,
                 navigator.serviceWorker.controller === globalThis.__claimReg.active,
                 navigator.serviceWorker.controller.scriptURL.endsWith('/app/sw.js'),
                 navigator.serviceWorker.controller.state
               ].join('|');
             });
             navigator.serviceWorker.register('/app/sw.js', {scope:'/app/'}).then(function(reg) {
               globalThis.__claimReg = reg;
             }, function(error) {
               globalThis.__claimResult = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__claimResult").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|true|true|true|activated");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "clients.claim controllerchange timed out: {}",
            webview
                .execute_script(
                    "[
                       navigator.serviceWorker.controller &&
                         navigator.serviceWorker.controller.scriptURL,
                       navigator.serviceWorker.controller &&
                         navigator.serviceWorker.controller.state,
                       globalThis.__claimReg && globalThis.__claimReg.active &&
                         globalThis.__claimReg.active.state,
                       globalThis.__claimResult
                     ].join('|')",
                )
                .unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn message_time_clients_claim_controls_existing_iframe() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/claim-fetch.https.html";
    let fallback_requests = Arc::new(Mutex::new(Vec::new()));
    let fallback_log = Arc::clone(&fallback_requests);
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/service-workers/service-worker/resources/claim-worker.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: "self.addEventListener('message', function(event) {
                         self.clients.claim().then(function(result) {
                           event.data.port.postMessage(result === undefined ? 'PASS' : 'FAIL');
                         });
                       });
                       self.addEventListener('fetch', function(event) {
                         event.respondWith(new Response('Intercepted!'));
                       });"
                .as_bytes()
                .to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(move |request| {
            fallback_log.lock().unwrap().push(request.url.clone());
            let body = if request
                .url
                .ends_with("/resources/blank.html?using-different-registration")
            {
                "<!doctype html><title>blank</title>"
            } else if request.url.ends_with("/resources/simple.txt") {
                "a simple text file\n"
            } else {
                "fallback"
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/plain".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<iframe id=\"frame\" src=\"resources/blank.html?using-different-registration\"></iframe>
         <script>
           globalThis.__claimFetchStage = 'starting';
           (async function() {
             const frame = document.getElementById('frame');
             const before = await frame.contentWindow.fetch('simple.txt').then(r => r.text());
             globalThis.__claimFetchStage = 'before:' + before;
             const registration = await navigator.serviceWorker.register(
               'resources/claim-worker.js', {scope:'resources/blank.html?using-different-registration'});
             const worker = registration.installing;
             await new Promise(resolve => {
               if (worker.state === 'activated') {
                 resolve();
                 return;
               }
               worker.addEventListener('statechange', function listener() {
                 if (worker.state === 'activated') {
                   worker.removeEventListener('statechange', listener);
                   resolve();
                 }
               });
             });
             globalThis.__claimFetchStage = 'activated';
             const controllerChanged = new Promise(resolve => {
               frame.contentWindow.navigator.serviceWorker.oncontrollerchange = function() {
                 resolve(frame.contentWindow.navigator.serviceWorker.controller);
               };
             });
             const sawMessage = new Promise(resolve => {
               const channel = new MessageChannel();
               channel.port1.onmessage = event => resolve(event.data);
               worker.postMessage({port: channel.port2}, [channel.port2]);
             });
             const data = await sawMessage;
             globalThis.__claimFetchStage = 'message:' + data;
             const controller = await controllerChanged;
             if (!controller || !controller.scriptURL.endsWith('/resources/claim-worker.js')) {
               globalThis.__claimFetchStage = 'controller:' + (controller && controller.scriptURL);
               return;
             }
             const after = await frame.contentWindow.fetch('simple.txt').then(r => r.text());
             globalThis.__claimFetchStage = 'after:' + after;
           })().catch(error => {
             globalThis.__claimFetchStage = 'error:' + error.name + ':' + error.message;
           });
         </script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("String(globalThis.__claimFetchStage)").unwrap();
        if value.starts_with("after:") || value.starts_with("error:") {
            assert_eq!(value, "after:Intercepted!");
            break;
        }
        if Instant::now() >= deadline {
            let state = webview
                .execute_script(
                    "const frame = document.getElementById('frame');
                     const sw = frame && frame.contentWindow.navigator.serviceWorker;
                     const raw = frame && JSON.parse(__zw_sw_controller(
                       frame.contentWindow.document._zwURL,
                       frame.contentWindow.document._zwSwClientId
                     )).controller;
                     JSON.stringify({
                       value: globalThis.__claimFetchStage,
                       controller: sw && sw.controller && sw.controller.scriptURL,
                       raw: raw && raw.scriptURL,
                       client: frame && frame.contentWindow.document._zwSwClientId
                     })",
                )
                .unwrap();
            panic!("message-time clients.claim iframe fetch timed out: {state}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        fallback_requests.lock().unwrap().as_slice(),
        &[
            "https://example.test/service-workers/service-worker/resources/blank.html?using-different-registration"
                .to_string(),
            "https://example.test/service-workers/service-worker/resources/simple.txt".to_string(),
        ]
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn waiting_worker_clients_claim_rejects_over_message_port() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/claim-using-registration.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            let body = if script.ends_with("/resources/claim-worker.js") {
                "self.addEventListener('message', function(event) {
                   self.clients.claim().then(function(result) {
                     event.data.port.postMessage(result === undefined ? 'PASS' : 'FAIL');
                   }).catch(function(error) {
                     event.data.port.postMessage('FAIL: exception: ' + error.name);
                   });
                 });"
            } else {
                ""
            };
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: body.as_bytes().to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| {
            let body = if request.url.ends_with("/resources/blank.html?using-same-registration") {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<script>
           globalThis.__waitingClaimResult = 'starting';
           (async function() {
             const scope = 'resources/blank.html?using-same-registration';
             const existing = await navigator.serviceWorker.getRegistration(scope);
             if (existing) {
               await existing.unregister();
             }
             const first = await navigator.serviceWorker.register('resources/empty.js', {
               scope: scope
             });
             globalThis.__waitingClaimResult = 'registered-first';
             const firstWorker = first.installing;
             await new Promise(resolve => {
               if (firstWorker.state === 'activated') {
                 resolve();
                 return;
               }
               firstWorker.addEventListener('statechange', function listener() {
                 if (firstWorker.state === 'activated') {
                   firstWorker.removeEventListener('statechange', listener);
                   resolve();
                 }
               });
             });
             globalThis.__waitingClaimResult = 'activated-first';
             await new Promise(resolve => {
               const frame = document.createElement('iframe');
               frame.id = 'frame';
               frame.src = 'resources/blank.html?using-same-registration';
               frame.onload = () => resolve(frame);
               document.body.appendChild(frame);
             });
             globalThis.__waitingClaimResult = 'loaded-frame';
             const registration = await navigator.serviceWorker.register(
               'resources/claim-worker.js',
               {scope: scope});
             const worker = registration.installing;
             await new Promise(resolve => {
               if (worker.state === 'installed') {
                 resolve();
                 return;
               }
               worker.addEventListener('statechange', function listener() {
                 if (worker.state === 'installed') {
                   worker.removeEventListener('statechange', listener);
                   resolve();
                 }
               });
             });
             globalThis.__waitingClaimResult = 'installed:' + worker.state;
             const channel = new MessageChannel();
             channel.port1.onmessage = event => {
               globalThis.__waitingClaimResult = 'message:' + event.data;
             };
             worker.postMessage({port: channel.port2}, [channel.port2]);
           })().catch(error => {
             globalThis.__waitingClaimResult =
               'error:' + globalThis.__waitingClaimResult + ':' + error.name + ':' + error.message;
           });
         </script>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let _ = webview.poll_service_worker_runtime_events();
        let value = webview
            .execute_script(
                "if (typeof globalThis.__zwPollServiceWorkerRegistrations === 'function') {
                   globalThis.__zwPollServiceWorkerRegistrations();
                 }
                 if (typeof globalThis.__zwPollServiceWorkerMessages === 'function') {
                   globalThis.__zwPollServiceWorkerMessages();
                 }
                 String(globalThis.__waitingClaimResult)",
            )
            .unwrap();
        if value.starts_with("message:") || value.starts_with("error:") {
            assert_eq!(value, "message:FAIL: exception: InvalidStateError");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "waiting worker clients.claim message timed out: {value}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn service_worker_post_message_dispatches_structured_page_payload() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('message', event => {
                    globalThis.received =
                      event.data.kind + ':' + event.data.items[1] + ':' +
                      String(event instanceof MessageEvent);
                    if (event.data.kind !== 'silent') {
                      event.source.postMessage({
                        echo: event.data.kind,
                        sourceURL: event.source.url
                      });
                    }
                });"
            .to_string())
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__messageResult = 'pending';
             globalThis.__messageReplies = [];
             navigator.serviceWorker.addEventListener('message', function(event) {
               globalThis.__messageReplies.push(event.data.echo);
               globalThis.__messageResult = [
                 event.data.echo,
                 event.data.sourceURL,
                 event.source === globalThis.__messageWorker,
                 event instanceof MessageEvent,
                 event.target === navigator.serviceWorker
               ].join('|');
             });
             navigator.serviceWorker.register('/sw.js').then(function() {
               return navigator.serviceWorker.ready;
             }).then(function(reg) {
               globalThis.__messageWorker = reg.active;
               reg.active.postMessage({kind:'page', items:[1, 2]});
             }, function(error) {
               globalThis.__messageResult = 'error:' + String(error);
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__messageResult").unwrap();
        if value != "pending" {
            assert_eq!(value, "page|https://example.test/page.html|true|true|true");
            break;
        }
        assert!(Instant::now() < deadline, "ServiceWorker.postMessage timed out");
        std::thread::sleep(Duration::from_millis(10));
    }

    webview
        .execute_script(
            "globalThis.__messageWorker.postMessage({kind:'one', items:[1, 2]});
             globalThis.__messageWorker.postMessage({kind:'two', items:[1, 2]});",
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let replies = webview.execute_script("globalThis.__messageReplies.join('|')").unwrap();
        if replies == "page|one|two" {
            break;
        }
        if Instant::now() >= deadline {
            let state = webview
                .execute_script(
                    "JSON.stringify({
                       replies: globalThis.__messageReplies,
                       sequence: globalThis.__messageWorker._messageSequence,
                       target: globalThis.__messageWorker._messagePollTarget,
                       pending: globalThis.__messageWorker._messagePollPending
                     })",
                )
                .unwrap();
            panic!("consecutive worker replies timed out: {state}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    webview
        .execute_script("globalThis.__messageWorker.postMessage({kind:'silent'});")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let pending = webview
            .execute_script("String(globalThis.__messageWorker._messagePollPending)")
            .unwrap();
        if pending == "false" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "empty worker reply batch did not complete polling"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn service_worker_message_port_transfers_bidirectionally() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(|_, _| {
            Ok("addEventListener('message', event => {
                  const port = event.data;
                  port.onmessage = command => port.postMessage({echo: command.data});
                });"
            .into())
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__portResult = 'pending';
             navigator.serviceWorker.register('/sw.js', {scope:'/'}).then(function() {
               return navigator.serviceWorker.ready;
             }).then(function(reg) {
               var channel = new MessageChannel();
               channel.port2.onmessage = function(event) {
                 globalThis.__portResult = event.data.echo;
               };
               reg.active.postMessage(channel.port1, [channel.port1]);
               channel.port2.postMessage('ping');
             }, function(error) {
               globalThis.__portResult = 'error:' + error.name;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = webview.execute_script("globalThis.__portResult").unwrap();
        if value != "pending" {
            assert_eq!(value, "ping");
            break;
        }
        assert!(Instant::now() < deadline, "Service Worker MessagePort timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn clients_match_all_during_evaluation_reaches_registering_page() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/clients-matchall-on-evaluation.https.html")
        .script_source_fetcher(Arc::new(|_, script| match script {
            "https://example.test/sw.js" => Ok("importScripts('test-helpers.sub.js');
                     var page_url = normalizeURL('clients-matchall-on-evaluation.https.html');
                     clients.matchAll({includeUncontrolled: true}).then(clientList => {
                       for (const client of clientList) {
                         if (client.url === page_url) client.postMessage({matched: client.url});
                       }
                     });"
            .into()),
            "https://example.test/test-helpers.sub.js" => Ok(
                "function normalizeURL(url) { return new URL(url, self.location).toString().replace(/#.*$/, ''); }"
                    .into(),
            ),
            _ => Err(format!("unexpected script: {script}")),
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__matchAllResult = 'pending';
             navigator.serviceWorker.onmessage = function(event) {
               globalThis.__matchAllResult = event.data.matched;
             };
             navigator.serviceWorker.register('/sw.js', {scope:'/scope/'}).then(function(registration) {
               globalThis.__matchAllRegistration = registration && registration.scope;
             }, function(error) {
               globalThis.__matchAllResult = 'register-error:' + error.name + ':' + error.message;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = webview
            .execute_script(
                "JSON.stringify({
               message: globalThis.__matchAllResult,
               registration: globalThis.__matchAllRegistration || null
             })",
            )
            .unwrap();
        if result.contains("https://example.test/clients-matchall-on-evaluation.https.html") {
            assert!(result.contains(r#""registration":"https://example.test/scope/""#));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "clients.matchAll page message timed out: {result}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn clients_get_during_evaluation_reaches_registering_page() {
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/clients-get-on-evaluation.https.html")
        .script_source_fetcher(Arc::new(|_, script| match script {
            "https://example.test/sw.js" => Ok("clients.matchAll({includeUncontrolled: true}).then(clientList => {
                     const id = clientList[0].id;
                     return Promise.all([clients.get(id), clients.get('missing')]);
                   }).then(results => {
                     results[0].postMessage({
                       matched: results[0].url,
                       missing: results[1] === undefined
                     });
                   });"
            .into()),
            _ => Err(format!("unexpected script: {script}")),
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__clientsGetResult = 'pending';
             navigator.serviceWorker.onmessage = function(event) {
               globalThis.__clientsGetResult = event.data.matched + ':' + event.data.missing;
             };
             navigator.serviceWorker.register('/sw.js', {scope:'/scope/'}).then(function(registration) {
               globalThis.__clientsGetRegistration = registration && registration.scope;
             }, function(error) {
               globalThis.__clientsGetResult = 'register-error:' + error.name + ':' + error.message;
             });",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = webview
            .execute_script(
                "JSON.stringify({
               message: globalThis.__clientsGetResult,
               registration: globalThis.__clientsGetRegistration || null
             })",
            )
            .unwrap();
        if result.contains("https://example.test/clients-get-on-evaluation.https.html:true") {
            assert!(result.contains(r#""registration":"https://example.test/scope/""#));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "clients.get page message timed out: {result}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn update_permissions_follow_calling_worker_state_during_installation() {
    let version = Arc::new(Mutex::new(0usize));
    let fetch_version = Arc::clone(&version);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, _| {
            let mut version = fetch_version.lock().unwrap();
            *version += 1;
            Ok(format!(
                "// version {}\n\
                 const installEventFired = new Promise(resolve => {{ self.fireInstallEvent = resolve; }});\n\
                 const installFinished = new Promise(resolve => {{ self.finishInstall = resolve; }});\n\
                 addEventListener('install', event => {{ fireInstallEvent(); event.waitUntil(installFinished); }});\n\
                 addEventListener('message', event => {{\n\
                   const port = event.data;\n\
                   port.onmessage = command => {{\n\
                     if (command.data === 'awaitInstallEvent') {{\n\
                       installEventFired.then(() => port.postMessage('installEventFired'));\n\
                     }} else if (command.data === 'finishInstall') {{\n\
                       finishInstall();\n\
                       installFinished.then(() => port.postMessage('installFinished'));\n\
                     }} else if (command.data === 'callUpdate') {{\n\
                       const response = new MessageChannel();\n\
                       registration.update().then(\n\
                         () => response.port2.postMessage({{success:true}}),\n\
                         error => response.port2.postMessage({{success:false, exception:error.name}})\n\
                       );\n\
                       port.postMessage(response.port1, [response.port1]);\n\
                     }}\n\
                   }};\n\
                 }});",
                *version
            ))
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__updatePermissions = 'pending';
             globalThis.__updatePermissionStage = 'starting';
             function send(worker, message) {
               return new Promise(function(resolve) {
                 var channel = new MessageChannel();
                 channel.port2.onmessage = function(event) { resolve(event.data); };
                 worker.postMessage(channel.port1, [channel.port1]);
                 channel.port2.postMessage(message);
               });
             }
             function nextMessage(port) {
               return new Promise(function(resolve) {
                 port.onmessage = function(event) { resolve(event.data); };
               });
             }
             function waitState(worker, state) {
               if (worker.state === state) return Promise.resolve();
               return new Promise(function(resolve) {
                 worker.addEventListener('statechange', function listener() {
                   if (worker.state === state) {
                     worker.removeEventListener('statechange', listener);
                     resolve();
                   }
                 });
               });
             }
             (async function() {
               var reg = await navigator.serviceWorker.register('/sw.js', {scope:'/'});
               globalThis.__updatePermissionReg = reg;
               var first = reg.installing;
               globalThis.__updatePermissionFirst = first;
               globalThis.__updatePermissionStage = 'await-first-install';
               await send(first, 'awaitInstallEvent');
               globalThis.__updatePermissionStage = 'client-update';
               var clientUpdate = reg.update();
               globalThis.__updatePermissionStage = 'installing-update';
               var installingPort = await send(first, 'callUpdate');
               var installingResult = await nextMessage(installingPort);
               globalThis.__updatePermissionStage = 'finish-first';
               await send(first, 'finishInstall');
               await waitState(first, 'activated');
               await clientUpdate;

               globalThis.__updatePermissionStage = 'replacement-update';
               var updateFound = new Promise(function(resolve) {
                 reg.addEventListener('updatefound', resolve, {once:true});
               });
               reg.update();
               await updateFound;
               var second = reg.installing;
               globalThis.__updatePermissionSecond = second;
               globalThis.__updatePermissionStage = 'await-second-install';
               await send(second, 'awaitInstallEvent');
               globalThis.__updatePermissionStage = 'active-update';
               var activePort = await send(first, 'callUpdate');
               var activeResult = await nextMessage(activePort);
               globalThis.__updatePermissionStage = 'finish-second';
               await send(second, 'finishInstall');

               globalThis.__updatePermissions = [
                 installingResult.success,
                 installingResult.exception,
                 activeResult.success
               ].join('|');
             })().catch(function(error) {
               globalThis.__updatePermissions = 'error:' + error.name + ':' + error.message;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__updatePermissions").unwrap();
        if value != "pending" {
            assert_eq!(value, "false|InvalidStateError|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "update permission matrix timed out: {}",
            webview
                .execute_script(
                    "[
                       globalThis.__updatePermissionStage,
                       globalThis.__updatePermissionFirst && globalThis.__updatePermissionFirst.state,
                       globalThis.__updatePermissionSecond && globalThis.__updatePermissionSecond.state,
                       globalThis.__updatePermissionReg && !!globalThis.__updatePermissionReg.installing,
                       globalThis.__updatePermissionReg && !!globalThis.__updatePermissionReg.active,
                       globalThis.__updatePermissionFirst && globalThis.__updatePermissionFirst._messageSequence,
                       globalThis.__updatePermissionFirst && globalThis.__updatePermissionFirst._messagePollTarget,
                       globalThis.__updatePermissionFirst && globalThis.__updatePermissionFirst._messagePollPending
                     ].join('|')",
                )
                .unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn repeated_registration_changes_script_type_and_worker_message() {
    let visits = Arc::new(Mutex::new(0usize));
    let fetch_visits = Arc::clone(&visits);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |_, script| {
            if script.contains("update-registration-with-type.py") {
                let mut visits = fetch_visits.lock().unwrap();
                *visits += 1;
                return if *visits == 1 {
                    Ok("importScripts('./imported-classic-script.js');
                        self.onmessage = event => event.source.postMessage(imported);"
                        .into())
                } else {
                    Ok("import * as module from './imported-module-script.js';
                        self.onmessage = event => event.source.postMessage(module.imported);"
                        .into())
                };
            }
            if script.ends_with("/imported-classic-script.js") {
                return Ok("const imported = 'A classic script.';".into());
            }
            if script.ends_with("/imported-module-script.js") {
                return Ok("export const imported = 'A module script.';".into());
            }
            Err(format!("unexpected Service Worker script: {script}"))
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__typeUpdate = 'registering-classic';
             navigator.serviceWorker.onmessage = function(event) {
               if (event.data === 'A classic script.') {
                 globalThis.__typeUpdate = 'registering-module';
                 navigator.serviceWorker.register(
                   '/resources/update-registration-with-type.py?key=test',
                   {scope:'/resources/type-update', type:'module'}
                 ).then(function(registration) {
                   globalThis.__secondRegistration = registration;
                   registration.installing.postMessage(' ');
                 }, function(error) {
                   globalThis.__typeUpdate = 'module-error:' + error;
                 });
               } else {
                 globalThis.__typeUpdate =
                   event.data + '|' +
                   String(globalThis.__firstRegistration === globalThis.__secondRegistration) + '|' +
                   String(globalThis.__firstWorker !== globalThis.__secondRegistration.installing);
               }
             };
             navigator.serviceWorker.register(
               '/resources/update-registration-with-type.py?key=test',
               {scope:'/resources/type-update', type:'classic'}
             ).then(function(registration) {
               globalThis.__firstRegistration = registration;
               globalThis.__firstWorker = registration.installing;
               function sendWhenActive() {
                 if (globalThis.__firstWorker.state === 'activated') {
                   globalThis.__firstWorker.postMessage(' ');
                 } else {
                   setTimeout(sendWhenActive, 0);
                 }
               }
               sendWhenActive();
             }, function(error) {
               globalThis.__typeUpdate = 'classic-error:' + error;
             });
             'started';",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__typeUpdate").unwrap();
        if value.contains('|') || value.contains("error:") {
            assert_eq!(value, "A module script.|true|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "script type update message timed out: {value}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn message_import_replays_persistent_worker_resource_map() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let request_log = Arc::clone(&requests);
    let mut webview = WebViewBuilder::new()
        .url("https://example.test/page.html")
        .script_source_fetcher(Arc::new(move |context, script| {
            request_log
                .lock()
                .unwrap()
                .push((context.to_string(), script.to_string()));
            match script {
                "https://example.test/sw.js" => Ok("importScripts('/message-import.js');
                     addEventListener('message', event => {
                       globalThis.messageImported = null;
                       importScripts('/message-import.js');
                       event.source.postMessage({value: globalThis.messageImported});
                     });"
                .to_string()),
                "https://example.test/message-import.js" => {
                    Ok("globalThis.messageImported = 'event-import-ok';".to_string())
                }
                _ => Err(format!("unexpected script: {script}")),
            }
        }))
        .build();

    webview
        .execute_script(
            "globalThis.__eventImportResult = 'pending';
             navigator.serviceWorker.addEventListener('message', event => {
               globalThis.__eventImportResult = event.data.value;
             });
             navigator.serviceWorker.register('/sw.js').then(() => navigator.serviceWorker.ready)
               .then(reg => reg.active.postMessage({type:'load'}));",
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let value = webview.execute_script("globalThis.__eventImportResult").unwrap();
        if value != "pending" {
            assert_eq!(value, "event-import-ok");
            break;
        }
        assert!(Instant::now() < deadline, "event-time message import timed out");
        std::thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(
        requests.lock().unwrap().as_slice(),
        &[
            (
                "https://example.test/page.html".to_string(),
                "https://example.test/sw.js".to_string(),
            ),
            (
                "https://example.test/sw.js".to_string(),
                "https://example.test/message-import.js".to_string(),
            ),
        ]
    );
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn worker_global_fetch_powers_cache_add_in_service_worker_runtime() {
    let config = WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/sw.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: "addEventListener('install', event => {
                         event.waitUntil((async () => {
                           const cache = await caches.open('runtime');
                           await cache.add('/app/a.txt');
                           await cache.addAll(['/app/b.txt']);
                           const one = await cache.match('/app/a.txt');
                           const two = await cache.match('/app/b.txt');
                           if ((await one.text()) !== 'alpha') throw new Error('wrong a');
                           if ((await two.text()) !== 'beta') throw new Error('wrong b');
                         })());
                       });"
                .as_bytes()
                .to_vec(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| match request.url.as_str() {
            "https://example.test/app/a.txt" => Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/plain".into())],
                body: "alpha".into(),
                body_bytes: None,
            }),
            "https://example.test/app/b.txt" => Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/plain".into())],
                body: "beta".into(),
                body_bytes: None,
            }),
            other => Err(format!("unexpected fetch URL: {other}")),
        })),
        ..Default::default()
    };
    let mut webview = crate::WebView::new(config);
    let registration_id = webview
        .register_service_worker_runtime("/sw.js", Some("/app/"), "https://example.test/page.html")
        .unwrap();
    wait_for_state(&mut webview, registration_id, ServiceWorkerState::Activated);

    let registration = webview.service_worker_runtime_registration(registration_id).unwrap();
    let cache = registration.cache_storage.get("runtime").unwrap();
    assert_eq!(
        cache
            .match_request(&zero_storage::CacheRequest::new("https://example.test/app/a.txt"))
            .unwrap()
            .body,
        b"alpha"
    );
    assert_eq!(
        cache
            .match_request(&zero_storage::CacheRequest::new("https://example.test/app/b.txt"))
            .unwrap()
            .body,
        b"beta"
    );
}
