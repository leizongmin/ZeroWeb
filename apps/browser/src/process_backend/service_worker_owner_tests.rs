use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;
use zero_protocol::message::{
    ServiceWorkerResult, ServiceWorkerScriptTypeWire, ServiceWorkerSnapshot, ServiceWorkerStateWire,
    ServiceWorkerUpdateViaCacheWire,
};
use zero_protocol::{AutomationOperation, AutomationResult, AutomationValue};

fn lock_multiprocess_tests() -> std::sync::MutexGuard<'static, ()> {
    crate::tests::MULTIPROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn execute_script(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    request_id: u64,
    script: &str,
) -> Result<AutomationValue, String> {
    backend.send_automation_request(
        tab_id,
        request_id,
        AutomationOperation::ExecuteScript {
            script: script.to_string(),
            arguments: Vec::new(),
        },
    )?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if let Some(response) = backend.take_automation_response(tab_id, request_id) {
            return match response.result {
                Ok(AutomationResult::Value(value)) => Ok(value),
                Ok(other) => Err(format!("unexpected automation result: {other:?}")),
                Err(error) => Err(format!("{:?}: {}", error.code, error.message)),
            };
        }
        if Instant::now() >= deadline {
            return Err(format!("automation request {request_id} timed out"));
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn load_committed_page(
    backend: &mut ProcessTabBackend,
    snapshots: &mut HashMap<TabId, TabSnapshot>,
    tab_id: TabId,
    url: &str,
) {
    snapshots.insert(
        tab_id,
        TabSnapshot {
            url: Some(url.to_string()),
            navigation_epoch: 1,
            ..Default::default()
        },
    );
    backend.ensure_renderer(tab_id, (800, 600));
    backend.load_html(tab_id, "<!doctype html><html><body></body></html>", None, Some(url), 1);
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        backend.poll(snapshots, &mut HashMap::new(), Some(tab_id), true);
        if backend
            .committed_document_urls
            .values()
            .any(|committed| committed == url)
        {
            return;
        }
        assert!(Instant::now() < deadline, "renderer did not commit test page");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn service_worker_authority_exists_only_for_committed_navigation() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(801);
    let renderer_id = 91;
    let url = "https://example.test/page";
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, url, 1);

    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.into(),
            navigation_epoch: 1,
        },
    );
    assert_eq!(
        backend.committed_document_urls.get(&renderer_id).map(String::as_str),
        Some(url)
    );

    backend.stage_indexed_db_navigation(renderer_id, "https://next.test/", 2);
    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
}

#[test]
fn mismatched_navigation_commit_does_not_grant_service_worker_authority() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(802);
    let renderer_id = 92;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, "https://expected.test/", 3);

    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: "https://attacker.test/".into(),
            navigation_epoch: 3,
        },
    );

    assert!(!backend.committed_document_urls.contains_key(&renderer_id));
}

#[test]
fn committed_navigation_observes_top_level_service_worker_client() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(809);
    let renderer_id = 99;
    let url = "https://example.test/app/page";
    backend.tab_to_renderer.insert(tab_id, renderer_id);

    backend.stage_indexed_db_navigation(renderer_id, url, 4);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.to_string(),
            navigation_epoch: 4,
        },
    );

    assert_eq!(
        backend.service_worker_owner.client_references_for_test(tab_id),
        [("99:4".to_string(), "top-level".to_string())]
    );
}

#[test]
fn navigation_replacement_removes_stale_service_worker_client() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(810);
    let renderer_id = 100;
    backend.tab_to_renderer.insert(tab_id, renderer_id);

    backend.stage_indexed_db_navigation(renderer_id, "https://example.test/app/old", 4);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: "https://example.test/app/old".into(),
            navigation_epoch: 4,
        },
    );
    backend.stage_indexed_db_navigation(renderer_id, "https://example.test/app/new", 5);
    assert!(
        backend
            .service_worker_owner
            .client_references_for_test(tab_id)
            .is_empty()
    );

    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: "https://example.test/app/new".into(),
            navigation_epoch: 5,
        },
    );

    assert_eq!(
        backend.service_worker_owner.client_references_for_test(tab_id),
        [("100:5".to_string(), "top-level".to_string())]
    );
}

#[test]
fn renderer_window_client_lifecycle_reaches_browser_owner() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(811);
    let renderer_id = 101;
    let url = "https://example.test/app/page";
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.stage_indexed_db_navigation(renderer_id, url, 6);
    backend.handle_navigation_committed(
        tab_id,
        renderer_id,
        NavigationCommittedParams {
            url: url.to_string(),
            navigation_epoch: 6,
        },
    );

    backend.handle_service_worker_request(
        tab_id,
        renderer_id,
        1,
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::ObserveWindowClient {
                client_id: "iframe:#child".into(),
                client_url: "child.html".into(),
                frame_type: "nested".into(),
            },
        },
    );

    assert_eq!(
        backend.service_worker_owner.client_references_for_test(tab_id),
        [
            ("101:6".to_string(), "top-level".to_string()),
            ("101:6:iframe:#child".to_string(), "nested".to_string()),
        ]
    );

    backend.handle_service_worker_request(
        tab_id,
        renderer_id,
        2,
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::RemoveWindowClient {
                client_id: "iframe:#child".into(),
            },
        },
    );

    assert_eq!(
        backend.service_worker_owner.client_references_for_test(tab_id),
        [("101:6".to_string(), "top-level".to_string())]
    );
}

#[test]
fn multiprocess_navigator_registration_uses_browser_owner() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let page_url = format!("http://{}/page", listener.local_addr().unwrap());
    let worker_source = "importScripts('/dependency.js');\
         addEventListener('install', event => event.waitUntil(Promise.resolve()));\
         addEventListener('activate', event => event.waitUntil(clients.claim()));\
         addEventListener('message', event => {\
           event.source.postMessage({echo:event.data.kind + ':' + globalThis.importedValue, sourceURL:event.source.url});\
         });";
    let server = std::thread::spawn(move || {
        for (path, source) in [
            ("/sw.js", worker_source),
            ("/dependency.js", "globalThis.importedValue = 'dependency';"),
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let size = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..size]).starts_with(&format!("GET {path} ")));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                source.len(),
                source
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    let mut backend = ProcessTabBackend::with_renderer_bin(renderer);
    let tab_id = TabId(803);
    let mut snapshots = HashMap::new();
    load_committed_page(&mut backend, &mut snapshots, tab_id, &page_url);
    execute_script(
        &mut backend,
        &mut snapshots,
        tab_id,
        1,
        "globalThis.__swResult = 'pending';\
         (async function () {\
           try {\
             globalThis.__swReply = 'pending';\
             navigator.serviceWorker.addEventListener('message', function(event) {\
               globalThis.__swReply = event.data.echo + '|' + event.data.sourceURL + '|' +\
                 String(event.source === globalThis.__swReady.active);\
             });\
             var reg = await navigator.serviceWorker.register('/sw.js', {updateViaCache:'all'});\
             var ready = await navigator.serviceWorker.ready;\
             ready.active.postMessage({kind:'page'});\
             globalThis.__swReg = reg;\
             globalThis.__swReady = ready;\
             globalThis.__swResult = 'ready';\
           } catch (error) {\
             globalThis.__swResult = 'error:' + String(error && error.message ? error.message : error);\
           }\
         })();",
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    let result = loop {
        let value = execute_script(
            &mut backend,
            &mut snapshots,
            tab_id,
            2,
            "if (globalThis.__swResult === 'ready' && navigator.serviceWorker.controller &&\
                 globalThis.__swReply !== 'pending') {\
               globalThis.__swResult = globalThis.__swReg.scope + '|' +\
                 globalThis.__swReg.updateViaCache + '|' +\
                 (globalThis.__swReady.active ? globalThis.__swReady.active.state : 'none') + '|' +\
                 String(navigator.serviceWorker.controller === globalThis.__swReady.active) + '|' +\
                 globalThis.__swReply;\
             }\
             return String(globalThis.__swResult);",
        )
        .unwrap();
        if let AutomationValue::String(value) = value
            && value != "pending"
            && value != "ready"
        {
            break value;
        }
        if Instant::now() >= deadline {
            let diagnostic = execute_script(
                &mut backend,
                &mut snapshots,
                tab_id,
                3,
                "return [globalThis.__swResult, __zw_sw_controller(),\
                   navigator.serviceWorker.controller && navigator.serviceWorker.controller.scriptURL,\
                   globalThis.__swReady && globalThis.__swReady.active &&\
                     globalThis.__swReady.active.state].join('|');",
            )
            .unwrap();
            panic!("Service Worker registration did not settle: {diagnostic:?}");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    server.join().unwrap();
    backend.remove_renderer(tab_id);

    let expected_scope = format!("http://{}/", page_url.split('/').nth(2).unwrap());
    assert_eq!(
        result,
        format!("{expected_scope}|all|activated|true|page:dependency|{page_url}|true")
    );
}

#[test]
fn multiprocess_registration_update_compares_script_bytes() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let page_url = format!("http://{}/page", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for source in [
            "globalThis.version = 1;",
            "globalThis.version = 1;",
            "globalThis.version = 2;",
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let size = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..size]).starts_with("GET /sw.js "));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                source.len(),
                source
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    let mut backend = ProcessTabBackend::with_renderer_bin(renderer);
    let tab_id = TabId(806);
    let mut snapshots = HashMap::new();
    load_committed_page(&mut backend, &mut snapshots, tab_id, &page_url);
    execute_script(
        &mut backend,
        &mut snapshots,
        tab_id,
        1,
        "globalThis.__updateResult = 'pending';\
         (async function () {\
           try {\
             var reg = await navigator.serviceWorker.register('/sw.js', {scope:'/'});\
             await navigator.serviceWorker.ready;\
             globalThis.__updateReg = reg;\
             globalThis.__updateActive = reg.active;\
             globalThis.__updateFound = 0;\
             reg.addEventListener('updatefound', function() { globalThis.__updateFound++; });\
             var same = await reg.update();\
             globalThis.__updateNoop =\
               String(same === reg) + '|' + String(reg.active === globalThis.__updateActive);\
             var changed = await reg.update();\
             globalThis.__updateChanged = String(changed === reg);\
             globalThis.__updateResult = 'waiting';\
           } catch (error) {\
             globalThis.__updateResult = 'error:' + String(error && error.message ? error.message : error);\
           }\
         })();",
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    let result = loop {
        let value = execute_script(
            &mut backend,
            &mut snapshots,
            tab_id,
            2,
            "if (globalThis.__updateResult === 'waiting' &&\
                 globalThis.__updateReg.active !== globalThis.__updateActive &&\
                 globalThis.__updateReg.active.state === 'activated' &&\
                 globalThis.__updateActive.state === 'redundant' &&\
                 globalThis.__updateFound === 1) {\
               globalThis.__updateResult = globalThis.__updateNoop + '|' +\
                 globalThis.__updateChanged + '|' +\
                 String(globalThis.__updateReg.active !== globalThis.__updateActive) + '|' +\
                 String(globalThis.__updateActive.state === 'redundant') + '|1';\
             }\
             return String(globalThis.__updateResult);",
        )
        .unwrap();
        if let AutomationValue::String(value) = value
            && value != "pending"
            && value != "waiting"
        {
            break value;
        }
        assert!(Instant::now() < deadline, "Service Worker update did not settle");
        std::thread::sleep(Duration::from_millis(5));
    };
    server.join().unwrap();
    backend.remove_renderer(tab_id);

    assert_eq!(result, "true|true|true|true|true|1");
}

#[test]
fn multiprocess_restart_restores_persisted_controller_without_refetch() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    let persistence = std::env::temp_dir()
        .join(format!("zeroweb-sw-process-restart-{}", std::process::id()))
        .join("registrations.json");
    let _ = std::fs::remove_dir_all(persistence.parent().unwrap());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let authority = listener.local_addr().unwrap().to_string();
    let register_page = format!("http://{authority}/register");
    let worker_source =
        "importScripts('/persisted-dependency.js'); if (!globalThis.persisted) throw new Error('missing import');";
    let server = std::thread::spawn(move || {
        for (path, source) in [
            ("/sw.js", worker_source),
            ("/persisted-dependency.js", "globalThis.persisted = true;"),
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let size = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..size]).starts_with(&format!("GET {path} ")));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                source.len(),
                source
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    {
        let mut backend =
            ProcessTabBackend::with_renderer_bin_and_service_worker_persistence(renderer.clone(), persistence.clone());
        let tab_id = TabId(807);
        let mut snapshots = HashMap::new();
        load_committed_page(&mut backend, &mut snapshots, tab_id, &register_page);
        execute_script(
            &mut backend,
            &mut snapshots,
            tab_id,
            1,
            "globalThis.__persistResult = 'pending';\
             navigator.serviceWorker.register('/sw.js', {scope:'/app/'}).then(function() {\
               return navigator.serviceWorker.ready;\
             }).then(function(reg) {\
               globalThis.__persistResult = reg.active && reg.active.state;\
             }, function(error) {\
               globalThis.__persistResult = 'error:' + String(error);\
             });",
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let value = execute_script(
                &mut backend,
                &mut snapshots,
                tab_id,
                2,
                "return String(globalThis.__persistResult);",
            )
            .unwrap();
            if value == AutomationValue::String("activated".into()) {
                break;
            }
            assert!(Instant::now() < deadline, "initial persistent registration timed out");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(persistence.is_file());
        backend.remove_renderer(tab_id);
    }
    server.join().unwrap();

    {
        let mut backend =
            ProcessTabBackend::with_renderer_bin_and_service_worker_persistence(renderer, persistence.clone());
        let tab_id = TabId(808);
        let mut snapshots = HashMap::new();
        let controlled_page = format!("http://{authority}/app/reloaded");
        load_committed_page(&mut backend, &mut snapshots, tab_id, &controlled_page);
        // 延迟恢复经 IPC 异步完成（首个 renderer 接入时 flush）：轮询至 controller 出现。
        let deadline = Instant::now() + Duration::from_secs(20);
        let value = loop {
            let value = execute_script(
                &mut backend,
                &mut snapshots,
                tab_id,
                3,
                "return navigator.serviceWorker.controller ?\
                   navigator.serviceWorker.controller.state + '|' +\
                   navigator.serviceWorker.controller.scriptURL : 'none';",
            )
            .unwrap();
            if value != AutomationValue::String("none".into()) {
                break value;
            }
            assert!(
                Instant::now() < deadline,
                "persistent restore did not activate a controller"
            );
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(
            value,
            AutomationValue::String(format!("activated|http://{authority}/sw.js"))
        );
        backend.remove_renderer(tab_id);
    }
    std::fs::remove_dir_all(persistence.parent().unwrap()).unwrap();
}

#[test]
fn new_renderer_discovers_browser_owned_registration() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let authority = listener.local_addr().unwrap().to_string();
    let first_page = format!("http://{authority}/first");
    let worker_source = "addEventListener('install', event => event.waitUntil(Promise.resolve()));";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        worker_source.len(),
        worker_source
    );
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 2048];
        let size = stream.read(&mut request).unwrap();
        assert!(String::from_utf8_lossy(&request[..size]).starts_with("GET /sw.js "));
        stream.write_all(response.as_bytes()).unwrap();
    });

    let mut backend = ProcessTabBackend::with_renderer_bin(renderer);
    let first_tab = TabId(804);
    let mut snapshots = HashMap::new();
    load_committed_page(&mut backend, &mut snapshots, first_tab, &first_page);
    execute_script(
        &mut backend,
        &mut snapshots,
        first_tab,
        1,
        "globalThis.__swResult = 'pending';\
         (async function () {\
           try {\
             var reg = await navigator.serviceWorker.register('/sw.js', { scope: '/app/' });\
             await navigator.serviceWorker.ready;\
             globalThis.__swResult = reg.active && reg.active.state;\
           } catch (error) {\
             globalThis.__swResult = 'error:' + String(error && error.message ? error.message : error);\
           }\
         })();",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = execute_script(
            &mut backend,
            &mut snapshots,
            first_tab,
            2,
            "return String(globalThis.__swResult);",
        )
        .unwrap();
        if value == AutomationValue::String("activated".into()) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "first renderer registration did not activate"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    server.join().unwrap();
    backend.remove_renderer(first_tab);
    snapshots.remove(&first_tab);

    let second_tab = TabId(805);
    let second_page = format!("http://{authority}/app/second");
    load_committed_page(&mut backend, &mut snapshots, second_tab, &second_page);
    execute_script(
        &mut backend,
        &mut snapshots,
        second_tab,
        3,
        "globalThis.__swDiscovery = 'pending';\
         (async function () {\
           try {\
             var controller = navigator.serviceWorker.controller;\
             var reg = await navigator.serviceWorker.getRegistration('/app/page');\
             var all = await navigator.serviceWorker.getRegistrations();\
             var state = reg && reg.active ? reg.active.state : 'none';\
             var same = !!reg && all.length === 1 && reg === all[0];\
             var controlled = !!controller && controller === reg.active &&\
               controller.scriptURL.endsWith('/sw.js');\
             var removed = reg ? await reg.unregister() : false;\
             globalThis.__swDiscovery = (reg ? reg.scope : 'none') + '|' + state + '|' +\
               String(all.length) + '|' + String(same) + '|' + String(controlled) + '|' +\
               String(removed);\
           } catch (error) {\
             globalThis.__swDiscovery = 'error:' + String(error && error.message ? error.message : error);\
           }\
         })();",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let result = loop {
        let value = execute_script(
            &mut backend,
            &mut snapshots,
            second_tab,
            4,
            "return String(globalThis.__swDiscovery);",
        )
        .unwrap();
        if let AutomationValue::String(value) = value
            && value != "pending"
        {
            break value;
        }
        assert!(Instant::now() < deadline, "new renderer discovery did not settle");
        std::thread::sleep(Duration::from_millis(5));
    };
    backend.remove_renderer(second_tab);

    assert_eq!(result, format!("http://{authority}/app/|activated|1|true|true|true"));
}

#[test]
fn committed_page_fetch_is_routed_through_active_service_worker() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let document_url = "https://example.test/app/page";
    let ServiceWorkerRequestDisposition::Fetch(plan) = owner.begin_request(
        TabId(812),
        false,
        1,
        Some(document_url),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Register {
                script_url: "/sw.js".into(),
                scope: Some("/app/".into()),
                document_url: document_url.into(),
                update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
                script_type: ServiceWorkerScriptTypeWire::Classic,
            },
        },
    ) else {
        panic!("registration must fetch script");
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    sender
        .send(Ok(zero_net::HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/javascript".into())],
            body: b"addEventListener('fetch', event => { event.respondWith(new Response('backend-sw', {status: 207})); });".to_vec(),
            url: plan.script_url().to_string(),
            redirect_count: 0,
        }))
        .unwrap();
    owner.attach_fetch(plan, receiver);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let responses = owner.poll();
        if responses
            .iter()
            .any(|response| matches!(response.params.result, Ok(ServiceWorkerResult::Registered { .. })))
        {
            break;
        }
        assert!(Instant::now() < deadline, "local registration did not complete");
        std::thread::sleep(Duration::from_millis(1));
    }
    loop {
        let _ = owner.poll();
        let ServiceWorkerRequestDisposition::Respond(controller) = owner.begin_request(
            TabId(812),
            false,
            2,
            Some(document_url),
            ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::Controller,
            },
        ) else {
            panic!("controller query must not fetch");
        };
        if matches!(
            controller.params.result,
            Ok(ServiceWorkerResult::OptionalSnapshot(Some(ServiceWorkerSnapshot {
                state: ServiceWorkerStateWire::Activated,
                ..
            })))
        ) {
            break;
        }
        assert!(Instant::now() < deadline, "local registration did not activate");
        std::thread::sleep(Duration::from_millis(1));
    }

    let mut backend =
        ProcessTabBackend::with_renderer_bin_and_service_worker_owner(PathBuf::from("unused-renderer"), owner);
    let tab_id = TabId(812);
    let renderer_id = 112;
    backend.tab_to_renderer.insert(tab_id, renderer_id);
    backend.committed_document_urls.insert(renderer_id, document_url.into());
    backend.committed_document_epochs.insert(renderer_id, 7);

    backend.handle_fetch_request(
        tab_id,
        FetchParams {
            request_id: 77,
            url: "https://example.test/app/data".into(),
            method: "GET".into(),
            headers: vec![("X-Zero-Resource-Type".into(), "script".into())],
            body: None,
        },
    );
    assert_eq!(backend.pending_fetch_count_for_test(), 0);

    let completed = loop {
        let _ = backend.service_worker_owner.poll();
        let completed = backend.service_worker_owner.take_completed_page_fetches();
        if !completed.is_empty() {
            break completed;
        }
        assert!(Instant::now() < deadline, "page fetch did not settle");
        std::thread::sleep(Duration::from_millis(1));
    };
    assert!(matches!(
        &completed[0],
        CompletedServiceWorkerPageFetch::Respond {
            tab_id: completed_tab,
            request_id: 77,
            status: 207,
            body,
            ..
        } if *completed_tab == tab_id && body == b"backend-sw"
    ));
}

#[test]
fn internal_dns_prefetch_is_not_routed_through_service_worker() {
    let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
    let tab_id = TabId(813);
    backend.tab_to_renderer.insert(tab_id, 113);
    backend
        .committed_document_urls
        .insert(113, "https://example.test/app/page".into());
    backend.committed_document_epochs.insert(113, 1);

    backend.handle_fetch_request(
        tab_id,
        FetchParams {
            request_id: 78,
            url: "https://example.test".into(),
            method: "DNS-PREFETCH".into(),
            headers: Vec::new(),
            body: None,
        },
    );

    assert_eq!(backend.pending_fetch_count_for_test(), 1);
}
