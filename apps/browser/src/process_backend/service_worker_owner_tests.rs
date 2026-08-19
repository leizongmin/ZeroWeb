use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;
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
fn multiprocess_navigator_registration_uses_browser_owner() {
    let _multiprocess_guard = lock_multiprocess_tests();
    let renderer = resolve_renderer_binary().expect("fresh zero-renderer binary is required");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let page_url = format!("http://{}/page", listener.local_addr().unwrap());
    let worker_source = "addEventListener('install', event => event.waitUntil(Promise.resolve()));\
         addEventListener('activate', event => event.waitUntil(clients.claim()));\
         addEventListener('message', event => { globalThis.lastMessage = event.data.kind; });";
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
             var reg = await navigator.serviceWorker.register('/sw.js');\
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
            "if (globalThis.__swResult === 'ready' && navigator.serviceWorker.controller) {\
               globalThis.__swResult = globalThis.__swReg.scope + '|' +\
                 (globalThis.__swReady.active ? globalThis.__swReady.active.state : 'none') + '|' +\
                 String(navigator.serviceWorker.controller === globalThis.__swReady.active);\
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
    assert_eq!(result, format!("{expected_scope}|activated|true"));
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
