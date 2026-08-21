use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn register_request(document_url: &str) -> ServiceWorkerRequestParams {
    ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::Register {
            script_url: "/sw.js".into(),
            scope: Some("/app/".into()),
            document_url: document_url.into(),
            update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
            script_type: ServiceWorkerScriptTypeWire::Classic,
        },
    }
}

fn attach_script(owner: &mut BrowserServiceWorkerOwner, disposition: ServiceWorkerRequestDisposition, script: &str) {
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("expected fetch plan");
    };
    attach_script_plan(owner, plan, script);
}

fn attach_script_plan(owner: &mut BrowserServiceWorkerOwner, plan: ServiceWorkerFetchPlan, script: &str) {
    let (sender, receiver) = mpsc::channel();
    sender
        .send(Ok(HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/javascript".into())],
            body: script.as_bytes().to_vec(),
            url: plan.script_url.to_string(),
            redirect_count: 0,
        }))
        .unwrap();
    owner.attach_fetch(plan, receiver);
}

fn wait_for_response(owner: &mut BrowserServiceWorkerOwner) -> CompletedServiceWorkerResponse {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(response) = owner.poll().into_iter().next() {
            return response;
        }
        assert!(Instant::now() < deadline, "owner response timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn wait_for_import_plan(owner: &mut BrowserServiceWorkerOwner) -> ServiceWorkerImportFetchPlan {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let _ = owner.poll();
        if let Some(plan) = owner.take_import_fetch_plans().into_iter().next() {
            return plan;
        }
        assert!(Instant::now() < deadline, "owner import fetch plan timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn attach_import_scripts(
    owner: &mut BrowserServiceWorkerOwner,
    plan: ServiceWorkerImportFetchPlan,
    sources: &[(&str, &str)],
) {
    assert_eq!(plan.urls().len(), sources.len());
    let receivers = plan
        .urls()
        .iter()
        .zip(sources)
        .map(|(url, (source, mime))| {
            let (sender, receiver) = mpsc::channel();
            sender
                .send(Ok(HttpResponse {
                    status_code: 200,
                    headers: vec![("Content-Type".into(), (*mime).into())],
                    body: source.as_bytes().to_vec(),
                    url: url.clone(),
                    redirect_count: 0,
                }))
                .unwrap();
            receiver
        })
        .collect();
    owner.attach_import_fetches(plan, receivers);
}

fn wait_for_registration_state(
    owner: &mut BrowserServiceWorkerOwner,
    registration_id: u64,
    expected: ServiceWorkerState,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let _ = owner.poll();
        if owner
            .normal
            .registration(registration_id)
            .is_some_and(|registration| registration.state == expected)
        {
            return;
        }
        assert!(Instant::now() < deadline, "registration state timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn persistence_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .join(format!("zeroweb-sw-{label}-{}", std::process::id()))
        .join("registrations.json")
}

#[test]
fn update_via_cache_controls_main_script_fetch_policy_and_snapshot() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        49,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Register {
                script_url: "/sw.js".into(),
                scope: Some("/app/".into()),
                document_url: "https://example.test/page".into(),
                update_via_cache: ServiceWorkerUpdateViaCacheWire::All,
                script_type: ServiceWorkerScriptTypeWire::Classic,
            },
        },
    );
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("registration must fetch");
    };
    assert!(plan.bypass_cache(), "initial registration bypasses HTTP cache");
    attach_script_plan(&mut owner, plan, "globalThis.version = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };
    wait_for_registration_state(&mut owner, registration_id, ServiceWorkerState::Activated);
    assert_eq!(
        owner.normal.registration(registration_id).unwrap().update_via_cache,
        ServiceWorkerUpdateViaCache::All
    );

    let disposition = owner.begin_request(
        TabId(1),
        false,
        50,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id },
        },
    );
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("update must fetch");
    };
    assert!(!plan.bypass_cache(), "updateViaCache=all may reuse main-script cache");
}

#[test]
fn module_registration_type_reaches_browser_manager() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let mut request = register_request("https://example.test/page");
    if let ServiceWorkerOperation::Register { script_type, .. } = &mut request.operation {
        *script_type = ServiceWorkerScriptTypeWire::Module;
    }
    let disposition = owner.begin_request(TabId(1), false, 51, Some("https://example.test/page"), request);
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("module registration must fetch its main script");
    };
    assert!(matches!(
        plan.purpose,
        ServiceWorkerFetchPurpose::Register {
            script_type: ServiceWorkerScriptType::Module,
            ..
        }
    ));
    attach_script_plan(&mut owner, plan, "export const value = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("module registration failed");
    };
    assert_eq!(
        owner.normal.registration(registration_id).unwrap().script_type,
        ServiceWorkerScriptType::Module
    );
}

#[test]
fn repeated_register_returns_existing_version_until_script_type_changes() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let script = "globalThis.version = 1;";

    let first = owner.begin_request(
        TabId(1),
        false,
        52,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, first, script);
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered {
        registration_id: first_id,
    }) = response.params.result
    else {
        panic!("initial registration failed");
    };
    wait_for_registration_state(&mut owner, first_id, ServiceWorkerState::Activated);

    let unchanged = owner.begin_request(
        TabId(1),
        false,
        53,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    let ServiceWorkerRequestDisposition::Respond(response) = unchanged else {
        panic!("identical registration must not fetch");
    };
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::Registered { registration_id }) if registration_id == first_id
    ));

    let mut module_request = register_request("https://example.test/page");
    if let ServiceWorkerOperation::Register { script_type, .. } = &mut module_request.operation {
        *script_type = ServiceWorkerScriptTypeWire::Module;
    }
    let changed = owner.begin_request(TabId(1), false, 54, Some("https://example.test/page"), module_request);
    attach_script(&mut owner, changed, script);
    let response = wait_for_response(&mut owner);
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::Registered { registration_id }) if registration_id != first_id
    ));
}

#[test]
fn imported_classic_scripts_use_browser_fetch_policy_and_persist_graph() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let main_script = "importScripts('./first.js#ignored', '/shared/second.js');
         if (globalThis.importOrder !== 'first,second') throw new Error('wrong order');";
    let mut request = register_request("https://example.test/page");
    if let ServiceWorkerOperation::Register { update_via_cache, .. } = &mut request.operation {
        *update_via_cache = ServiceWorkerUpdateViaCacheWire::None;
    }
    let disposition = owner.begin_request(TabId(1), false, 48, Some("https://example.test/page"), request);
    attach_script(&mut owner, disposition, main_script);

    let plan = wait_for_import_plan(&mut owner);
    assert!(plan.bypass_cache());
    assert_eq!(
        plan.urls(),
        ["https://example.test/first.js", "https://example.test/shared/second.js",]
    );
    attach_import_scripts(
        &mut owner,
        plan,
        &[
            ("globalThis.importOrder = 'first';", "text/javascript"),
            ("globalThis.importOrder += ',second';", "application/javascript"),
        ],
    );

    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration with imports failed");
    };
    wait_for_registration_state(&mut owner, registration_id, ServiceWorkerState::Activated);
    let persisted = owner.normal.persistent_active_registrations();
    assert_eq!(persisted[0].imported_scripts.len(), 2);
    assert_eq!(persisted[0].imported_scripts[0].url, "https://example.test/first.js");
    assert_eq!(
        persisted[0].imported_scripts[1].url,
        "https://example.test/shared/second.js"
    );

    let disposition = owner.begin_request(
        TabId(1),
        false,
        49,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id },
        },
    );
    attach_script(&mut owner, disposition, main_script);
    let plan = wait_for_import_plan(&mut owner);
    assert!(plan.bypass_cache());
    attach_import_scripts(
        &mut owner,
        plan,
        &[
            ("globalThis.importOrder = 'first';", "text/javascript"),
            ("globalThis.importOrder += ',second';", "application/javascript"),
        ],
    );
    assert_eq!(
        wait_for_response(&mut owner).params.result,
        Ok(ServiceWorkerResult::Updated {
            registration_id,
            changed: false,
        })
    );

    let disposition = owner.begin_request(
        TabId(1),
        false,
        50,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id },
        },
    );
    attach_script(&mut owner, disposition, main_script);
    let plan = wait_for_import_plan(&mut owner);
    assert!(plan.bypass_cache());
    attach_import_scripts(
        &mut owner,
        plan,
        &[
            ("globalThis.importOrder = 'first';", "text/javascript"),
            (
                "globalThis.importOrder += ',second'; globalThis.dependencyVersion = 2;",
                "application/javascript",
            ),
        ],
    );
    let Ok(ServiceWorkerResult::Updated {
        registration_id: replacement,
        changed: true,
    }) = wait_for_response(&mut owner).params.result
    else {
        panic!("changed imported graph did not start a replacement");
    };
    assert_ne!(replacement, registration_id);
}

#[test]
fn imported_script_response_applies_classic_no_cors_and_module_cors() {
    let response = |headers: Vec<(String, String)>| {
        Ok(HttpResponse {
            status_code: 200,
            headers,
            body: b"globalThis.loaded = true;".to_vec(),
            url: "https://cdn.test/dependency.js".into(),
            redirect_count: 1,
        })
    };
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            false,
            response(vec![("Content-Type".into(), "text/plain".into())]),
        )
        .is_err()
    );
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            false,
            response(vec![("Content-Type".into(), "text/javascript".into())]),
        )
        .is_ok()
    );
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            false,
            response(vec![
                ("Content-Type".into(), "text/javascript; charset=utf-8".into()),
                ("Access-Control-Allow-Origin".into(), "https://example.test".into()),
            ]),
        )
        .is_ok()
    );
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            false,
            Ok(HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".into(), "text/javascript".into()),
                    ("Access-Control-Allow-Origin".into(), "*".into()),
                ],
                body: b"globalThis.loaded = true;".to_vec(),
                url: "http://cdn.test/dependency.js".into(),
                redirect_count: 1,
            }),
        )
        .is_err()
    );
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            true,
            response(vec![("Content-Type".into(), "text/javascript".into())]),
        )
        .is_err()
    );
    assert!(
        validate_imported_script_response(
            "https://cdn.test/dependency.js",
            "https://example.test",
            true,
            response(vec![
                ("Content-Type".into(), "text/javascript".into()),
                ("Access-Control-Allow-Origin".into(), "*".into()),
            ]),
        )
        .is_ok()
    );
}

#[test]
fn disconnect_cancels_queued_import_fetch_without_leaking_runtime() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        47,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "importScripts('/dependency.js');");
    let deadline = Instant::now() + Duration::from_secs(5);
    while owner.import_fetch_plans.is_empty() {
        let _ = owner.poll();
        assert!(Instant::now() < deadline, "import fetch plan was not queued");
        std::thread::sleep(Duration::from_millis(1));
    }

    owner.disconnect_tab(TabId(1));
    assert!(owner.import_fetch_plans.is_empty());
    while owner.normal.runtime_count() != 0 {
        let _ = owner.poll();
        assert!(Instant::now() < deadline, "disconnected import runtime did not close");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn ipc_event_time_import_uses_owned_renderer_after_evaluation_response() {
    let mut owner = BrowserServiceWorkerOwner::new();
    let disposition = owner.begin_request(
        TabId(7),
        false,
        48,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "addEventListener('install', () => {});");
    let _ = owner.poll();

    let evaluate = owner
        .take_host_commands()
        .into_iter()
        .find(|outgoing| matches!(outgoing.params.command, ServiceWorkerHostCommand::Evaluate { .. }))
        .expect("missing renderer evaluation command");
    let registration_id = evaluate.params.registration_id;
    owner.inject_host_event(
        TabId(7),
        false,
        ServiceWorkerHostEventParams {
            registration_id,
            event: ServiceWorkerHostEvent::Evaluated {
                script_url: "https://example.test/sw.js".into(),
            },
        },
    );
    let response = wait_for_response(&mut owner);
    assert_eq!(response.tab_id, TabId(7));
    assert!(owner.pending_evaluations.is_empty());
    assert!(owner.take_host_commands().into_iter().any(|outgoing| matches!(
        outgoing.params.command,
        ServiceWorkerHostCommand::DispatchLifecycle {
            phase: ServiceWorkerLifecycleWire::Install
        }
    )));

    owner.inject_host_event(
        TabId(7),
        false,
        ServiceWorkerHostEventParams {
            registration_id,
            event: ServiceWorkerHostEvent::ImportScriptsRequested {
                request_id: 9,
                specifiers: vec!["/event-import.js".into()],
            },
        },
    );
    let plan = wait_for_import_plan(&mut owner);
    assert_eq!(plan.tab_id(), TabId(7));
    assert_eq!(plan.urls(), ["https://example.test/event-import.js"]);
}

#[test]
fn ipc_module_request_preserves_referrer_and_fetch_policy() {
    let mut owner = BrowserServiceWorkerOwner::new();
    let mut request = register_request("https://example.test/page");
    if let ServiceWorkerOperation::Register {
        script_type,
        update_via_cache,
        ..
    } = &mut request.operation
    {
        *script_type = ServiceWorkerScriptTypeWire::Module;
        *update_via_cache = ServiceWorkerUpdateViaCacheWire::None;
    }
    let disposition = owner.begin_request(TabId(8), false, 49, Some("https://example.test/page"), request);
    attach_script(&mut owner, disposition, "import './lib/entry.js';");
    let _ = owner.poll();
    let evaluate = owner
        .take_host_commands()
        .into_iter()
        .find(|outgoing| matches!(outgoing.params.command, ServiceWorkerHostCommand::Evaluate { .. }))
        .expect("missing renderer evaluation command");
    let registration_id = evaluate.params.registration_id;

    owner.inject_host_event(
        TabId(8),
        false,
        ServiceWorkerHostEventParams {
            registration_id,
            event: ServiceWorkerHostEvent::ModuleScriptsRequested {
                request_id: 10,
                referrer_url: "https://example.test/sw.js".into(),
                specifiers: vec!["./lib/entry.js".into()],
            },
        },
    );
    let plan = wait_for_import_plan(&mut owner);
    assert_eq!(plan.tab_id(), TabId(8));
    assert_eq!(plan.urls(), ["https://example.test/lib/entry.js"]);
    assert!(plan.bypass_cache());
    assert!(plan.is_module());
}

#[test]
fn register_fetches_evaluates_and_returns_correlated_id() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        41,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.loaded = true;");

    let response = wait_for_response(&mut owner);
    assert_eq!(response.tab_id, TabId(1));
    assert_eq!(response.request_id, 41);
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::Registered { .. })
    ));
}

#[test]
fn client_update_during_initial_installation_reuses_candidate() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        50,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.version = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered {
        registration_id: installing,
    }) = response.params.result
    else {
        panic!("registration failed");
    };
    assert_eq!(
        owner
            .normal
            .registration(installing)
            .map(|registration| registration.state),
        Some(ServiceWorkerState::Installing)
    );
    let runtime_count = owner.normal.runtime_count();

    let ServiceWorkerRequestDisposition::Respond(response) = owner.begin_request(
        TabId(1),
        false,
        51,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update {
                registration_id: installing,
            },
        },
    ) else {
        panic!("client update during initial installation must not fetch");
    };
    assert_eq!(
        response.params.result,
        Ok(ServiceWorkerResult::Updated {
            registration_id: installing,
            changed: false,
        })
    );
    assert_eq!(owner.normal.runtime_count(), runtime_count);
}

#[test]
fn update_fetch_compares_bytes_before_starting_replacement() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        51,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.version = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id: first }) = response.params.result else {
        panic!("registration failed");
    };
    wait_for_registration_state(&mut owner, first, ServiceWorkerState::Activated);
    let runtime_count = owner.normal.runtime_count();

    let disposition = owner.begin_request(
        TabId(1),
        false,
        52,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id: first },
        },
    );
    attach_script(&mut owner, disposition, "globalThis.version = 1;");
    assert_eq!(
        wait_for_response(&mut owner).params.result,
        Ok(ServiceWorkerResult::Updated {
            registration_id: first,
            changed: false,
        })
    );
    let _ = owner.poll();
    assert_eq!(owner.normal.runtime_count(), runtime_count);

    let disposition = owner.begin_request(
        TabId(1),
        false,
        53,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id: first },
        },
    );
    attach_script(&mut owner, disposition, "globalThis.version = 2;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Updated {
        registration_id: replacement,
        changed: true,
    }) = response.params.result
    else {
        panic!("changed update failed");
    };
    assert_ne!(replacement, first);
    let slots = owner
        .normal
        .slots(
            &zero_page_runtime::ServiceWorkerRegistrationKey::new("https://example.test", "https://example.test/app/")
                .unwrap(),
        )
        .unwrap();
    assert_eq!(slots.active, Some(first));
    assert_eq!(slots.installing, Some(replacement));
}

#[test]
fn concurrent_updates_share_one_fetch_and_candidate() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        54,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.version = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered {
        registration_id: active,
    }) = response.params.result
    else {
        panic!("registration failed");
    };
    wait_for_registration_state(&mut owner, active, ServiceWorkerState::Activated);

    let disposition = owner.begin_request(
        TabId(1),
        false,
        55,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update {
                registration_id: active,
            },
        },
    );
    attach_script(&mut owner, disposition, "globalThis.version = 2;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Updated {
        registration_id: candidate,
        changed: true,
    }) = response.params.result
    else {
        panic!("changed update failed");
    };
    let runtime_count = owner.normal.runtime_count();

    for request_id in 56..65 {
        let ServiceWorkerRequestDisposition::Respond(response) = owner.begin_request(
            TabId(1),
            false,
            request_id,
            Some("https://example.test/page"),
            ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::Update {
                    registration_id: candidate,
                },
            },
        ) else {
            panic!("concurrent update must not fetch");
        };
        assert_eq!(
            response.params.result,
            Ok(ServiceWorkerResult::Updated {
                registration_id: candidate,
                changed: true,
            })
        );
    }
    assert_eq!(owner.normal.runtime_count(), runtime_count);
}

#[test]
fn active_worker_update_uses_browser_owned_fetch() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        65,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(
        &mut owner,
        disposition,
        "addEventListener('message', event => {
           registration.update().then(
             () => event.source.postMessage({success:true}),
             error => event.source.postMessage({success:false, exception:error.name})
           );
         });",
    );
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered {
        registration_id: active,
    }) = response.params.result
    else {
        panic!("registration failed");
    };
    wait_for_registration_state(&mut owner, active, ServiceWorkerState::Activated);
    owner.normal_channels.record_owned(active, TabId(1));

    let ServiceWorkerRequestDisposition::Respond(response) = owner.begin_request_for_client(
        TabId(1),
        false,
        66,
        Some("https://example.test/page"),
        "tab-1:1",
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::PostMessage {
                registration_id: active,
                data_json: "null".into(),
                transferred_port_ids: Vec::new(),
                data_port_index: None,
                target_port_id: None,
            },
        },
    ) else {
        panic!("postMessage must respond without network work");
    };
    assert_eq!(response.params.result, Ok(ServiceWorkerResult::Empty));

    let deadline = Instant::now() + Duration::from_secs(5);
    let plan = loop {
        let _ = owner.poll();
        if let Some(plan) = owner.take_update_fetch_plans().into_iter().next() {
            break plan;
        }
        assert!(Instant::now() < deadline, "worker update fetch plan timed out");
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(plan.script_url(), "https://example.test/sw.js");
    attach_script_plan(&mut owner, plan, "globalThis.version = 2;");

    loop {
        let _ = owner.poll();
        let (_, messages) = owner.normal.client_messages_since(active, "tab-1:1", 0);
        if !messages.is_empty() {
            assert_eq!(messages[0].data_json, r#"{"success":true}"#);
            break;
        }
        assert!(Instant::now() < deadline, "worker update response timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn update_rejects_non_javascript_main_script_without_replacing_active() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        54,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.version = 1;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };
    wait_for_registration_state(&mut owner, registration_id, ServiceWorkerState::Activated);

    let disposition = owner.begin_request(
        TabId(1),
        false,
        55,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Update { registration_id },
        },
    );
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("update must fetch");
    };
    let (sender, receiver) = mpsc::channel();
    sender
        .send(Ok(HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: b"globalThis.version = 2;".to_vec(),
            url: plan.script_url.to_string(),
            redirect_count: 0,
        }))
        .unwrap();
    owner.attach_fetch(plan, receiver);

    let response = wait_for_response(&mut owner);
    assert!(matches!(
        response.params.result,
        Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::Security,
            ..
        })
    ));
    let active = owner
        .normal
        .active_registration_for_url("https://example.test", "https://example.test/app/page")
        .unwrap();
    assert_eq!(active.id, registration_id);
    assert_eq!(active.state, ServiceWorkerState::Activated);
}

#[test]
fn persistent_owner_restores_active_runtime_and_unregisters_durably() {
    let path = persistence_path("restore");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    {
        let mut owner = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
        let mut request = register_request("https://example.test/app/page");
        if let ServiceWorkerOperation::Register { update_via_cache, .. } = &mut request.operation {
            *update_via_cache = ServiceWorkerUpdateViaCacheWire::None;
        }
        let disposition = owner.begin_request(TabId(1), false, 54, Some("https://example.test/app/page"), request);
        attach_script(
            &mut owner,
            disposition,
            "globalThis.restored = true; addEventListener('activate', event => event.waitUntil(Promise.resolve()));",
        );
        let response = wait_for_response(&mut owner);
        let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
            panic!("persistent registration failed");
        };
        wait_for_registration_state(&mut owner, registration_id, ServiceWorkerState::Activated);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !path.is_file() {
            let _ = owner.poll();
            assert!(Instant::now() < deadline, "persistence file was not written");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    let mut restored = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    let deadline = Instant::now() + Duration::from_secs(5);
    let restored_id = loop {
        let _ = restored.poll();
        if let Some(registration) = restored
            .normal
            .active_registration_for_url("https://example.test", "https://example.test/app/page")
        {
            break registration.id;
        }
        assert!(Instant::now() < deadline, "persistent runtime restore timed out");
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(
        restored.normal.registration(restored_id).unwrap().update_via_cache,
        ServiceWorkerUpdateViaCache::None
    );
    let ServiceWorkerRequestDisposition::Respond(controller) = restored.begin_request(
        TabId(2),
        false,
        55,
        Some("https://example.test/app/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Controller,
        },
    ) else {
        panic!("controller query must complete immediately");
    };
    assert!(matches!(
        controller.params.result,
        Ok(ServiceWorkerResult::OptionalSnapshot(Some(ServiceWorkerSnapshot {
            registration_id,
            state: ServiceWorkerStateWire::Activated,
            ..
        }))) if registration_id == restored_id
    ));

    let ServiceWorkerRequestDisposition::Respond(unregistered) = restored.begin_request(
        TabId(2),
        false,
        56,
        Some("https://example.test/app/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Unregister {
                registration_id: restored_id,
            },
        },
    ) else {
        panic!("unregister must complete immediately");
    };
    assert_eq!(unregistered.params.result, Ok(ServiceWorkerResult::Boolean(true)));
    drop(restored);

    let empty = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    assert!(empty.normal.registrations_for_origin("https://example.test").is_empty());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn private_and_invalid_persistence_never_restore_into_normal_profile() {
    let path = persistence_path("private");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    let disposition = owner.begin_request(
        TabId(7),
        true,
        57,
        Some("https://example.test/app/page"),
        register_request("https://example.test/app/page"),
    );
    attach_script(&mut owner, disposition, "globalThis.privateWorker = true;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("private registration failed");
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let _ = owner.poll();
        if owner
            .private
            .get(&TabId(7))
            .and_then(|manager| manager.registration(registration_id))
            .is_some_and(|registration| registration.state == ServiceWorkerState::Activated)
        {
            break;
        }
        assert!(Instant::now() < deadline, "private registration activation timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!path.exists(), "private profile must not write normal persistence");
    drop(owner);

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, r#"{"version":999,"registrations":[]}"#).unwrap();
    let invalid = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    assert!(
        invalid
            .normal
            .registrations_for_origin("https://example.test")
            .is_empty()
    );
    drop(invalid);
    std::fs::write(
        &path,
        r#"{"version":1,"registrations":[{"script_url":"https://example.test/sw.js","scope":"https://example.test/","origin":"https://example.test","script_source":""}]}"#,
    )
    .unwrap();
    let migrated = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    assert_eq!(
        migrated.normal.registrations_for_origin("https://example.test")[0].update_via_cache,
        ServiceWorkerUpdateViaCache::Imports
    );
    assert_eq!(
        migrated.normal.registrations_for_origin("https://example.test")[0].script_type,
        ServiceWorkerScriptType::Classic
    );
    drop(migrated);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn persistence_restore_keeps_valid_scope_when_sibling_script_fails() {
    let path = persistence_path("partial-restore");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    {
        let mut owner = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
        for (request_id, script_url, scope) in [(58, "/a.js", "/a/"), (59, "/b.js", "/b/")] {
            let disposition = owner.begin_request(
                TabId(1),
                false,
                request_id,
                Some("https://example.test/page"),
                ServiceWorkerRequestParams {
                    operation: ServiceWorkerOperation::Register {
                        script_url: script_url.into(),
                        scope: Some(scope.into()),
                        document_url: "https://example.test/page".into(),
                        update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
                        script_type: ServiceWorkerScriptTypeWire::Classic,
                    },
                },
            );
            attach_script(&mut owner, disposition, "globalThis.persistedScope = true;");
            let response = wait_for_response(&mut owner);
            let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
                panic!("scope registration failed");
            };
            wait_for_registration_state(&mut owner, registration_id, ServiceWorkerState::Activated);
        }
    }

    let mut state = serde_json::from_str::<PersistedServiceWorkers>(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(state.registrations.len(), 2);
    state
        .registrations
        .iter_mut()
        .find(|registration| registration.scope == "https://example.test/b/")
        .unwrap()
        .script_source = "function(".into();
    std::fs::write(&path, serde_json::to_string(&state).unwrap()).unwrap();

    let restored = BrowserServiceWorkerOwner::with_local_hosts_and_persistence(path.clone());
    let registrations = restored.normal.registrations_for_origin("https://example.test");
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].scope, "https://example.test/a/");
    let rewritten = serde_json::from_str::<PersistedServiceWorkers>(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(rewritten.registrations.len(), 1);
    assert_eq!(rewritten.registrations[0].scope, "https://example.test/a/");
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn lifecycle_state_changes_are_cursor_based_and_ordered() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        63,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "void 0;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let _ = owner.poll();
        let disposition = owner.begin_request(
            TabId(1),
            false,
            64,
            Some("https://example.test/page"),
            ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::StateChanges {
                    registration_id,
                    after_sequence: 0,
                },
            },
        );
        let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
            panic!("state changes must complete immediately");
        };
        if matches!(
            response.params.result,
            Ok(ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
                latest_sequence: 3,
                ref states,
                claim_clients: false,
            })) if states == &[
                ServiceWorkerStateWire::Installed,
                ServiceWorkerStateWire::Activating,
                ServiceWorkerStateWire::Activated,
            ]
        ) {
            break;
        }
        assert!(Instant::now() < deadline, "lifecycle state changes timed out");
        std::thread::sleep(Duration::from_millis(5));
    }

    let disposition = owner.begin_request(
        TabId(1),
        false,
        65,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::StateChanges {
                registration_id,
                after_sequence: 1,
            },
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("state changes must complete immediately");
    };
    assert_eq!(
        response.params.result,
        Ok(ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
            latest_sequence: 3,
            states: vec![ServiceWorkerStateWire::Activating, ServiceWorkerStateWire::Activated],
            claim_clients: false,
        }))
    );
}

#[test]
fn register_rejects_renderer_document_authority_mismatch_before_fetch() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        42,
        Some("https://example.test/page"),
        register_request("https://attacker.test/page"),
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("authority mismatch must not fetch");
    };
    assert!(matches!(
        response.params.result,
        Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::InvalidArgument,
            ..
        })
    ));
}

#[test]
fn registration_normalizes_fragments_and_classifies_url_errors() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        66,
        Some("https://example.test/service-worker/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Register {
                script_url: "resources/sw.js#script".into(),
                scope: Some("resources/app/#scope".into()),
                document_url: "https://example.test/service-worker/page".into(),
                update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
                script_type: ServiceWorkerScriptTypeWire::Classic,
            },
        },
    );
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("fragment-normalized registration must fetch");
    };
    assert_eq!(
        plan.script_url.as_str(),
        "https://example.test/service-worker/resources/sw.js"
    );
    assert_eq!(
        plan.scope.as_str(),
        "https://example.test/service-worker/resources/app/"
    );

    for (request_id, scope, code) in [
        (67, "null", ServiceWorkerErrorCode::Security),
        (68, "resources/app%2fchild", ServiceWorkerErrorCode::InvalidArgument),
    ] {
        let disposition = owner.begin_request(
            TabId(1),
            false,
            request_id,
            Some("https://example.test/service-worker/page"),
            ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::Register {
                    script_url: "resources/sw.js".into(),
                    scope: Some(scope.into()),
                    document_url: "https://example.test/service-worker/page".into(),
                    update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
                    script_type: ServiceWorkerScriptTypeWire::Classic,
                },
            },
        );
        let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
            panic!("invalid registration must not fetch");
        };
        assert!(matches!(
            response.params.result,
            Err(ServiceWorkerError {
                code: actual,
                ..
            }) if actual == code
        ));
    }
}

#[test]
fn navigation_disconnect_drops_stale_registration_response() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        43,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "void 0;");

    owner.disconnect_tab(TabId(1));

    assert!(owner.poll().is_empty());
}

#[test]
fn normal_registration_survives_renderer_disconnect() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        44,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "void 0;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };
    owner.disconnect_tab(TabId(1));

    let disposition = owner.begin_request(
        TabId(2),
        false,
        45,
        Some("https://example.test/next"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Snapshot { registration_id },
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("snapshot must complete immediately");
    };
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
            registration_id: id,
            ..
        })) if id == registration_id
    ));
}

#[test]
fn new_renderer_discovers_normal_registration_without_known_id() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        46,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "void 0;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };
    owner.disconnect_tab(TabId(1));

    let disposition = owner.begin_request(
        TabId(2),
        false,
        47,
        Some("https://example.test/next"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::GetRegistration {
                client_url: "/app/page".into(),
            },
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("discovery must complete immediately");
    };
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::OptionalSnapshot(Some(ServiceWorkerSnapshot {
            registration_id: id,
            ..
        }))) if id == registration_id
    ));

    let disposition = owner.begin_request(
        TabId(2),
        false,
        48,
        Some("https://example.test/next"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::GetRegistrations,
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("list discovery must complete immediately");
    };
    assert!(matches!(
        response.params.result,
        Ok(ServiceWorkerResult::Snapshots(snapshots))
            if snapshots.len() == 1 && snapshots[0].registration_id == registration_id
    ));
}

#[test]
fn discovery_rejects_cross_origin_client_url() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        49,
        Some("https://example.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::GetRegistration {
                client_url: "https://other.test/page".into(),
            },
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("invalid discovery must complete immediately");
    };
    assert!(matches!(
        response.params.result,
        Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::InvalidArgument,
            ..
        })
    ));
}

#[test]
fn private_tabs_have_isolated_registration_namespaces() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let mut registration_ids = Vec::new();
    for (tab_id, request_id) in [(TabId(1), 51), (TabId(2), 52)] {
        let disposition = owner.begin_request(
            tab_id,
            true,
            request_id,
            Some("https://example.test/page"),
            register_request("https://example.test/page"),
        );
        attach_script(&mut owner, disposition, "void 0;");
        let response = wait_for_response(&mut owner);
        let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
            panic!("private registration failed");
        };
        registration_ids.push(registration_id);
    }
    assert_eq!(registration_ids[0], registration_ids[1]);
}

#[test]
fn registration_ids_are_hidden_from_other_origins() {
    let mut owner = BrowserServiceWorkerOwner::with_local_hosts();
    let disposition = owner.begin_request(
        TabId(1),
        false,
        61,
        Some("https://example.test/page"),
        register_request("https://example.test/page"),
    );
    attach_script(&mut owner, disposition, "void 0;");
    let response = wait_for_response(&mut owner);
    let Ok(ServiceWorkerResult::Registered { registration_id }) = response.params.result else {
        panic!("registration failed");
    };

    let disposition = owner.begin_request(
        TabId(2),
        false,
        62,
        Some("https://other.test/page"),
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Snapshot { registration_id },
        },
    );
    let ServiceWorkerRequestDisposition::Respond(response) = disposition else {
        panic!("snapshot must complete immediately");
    };
    assert!(matches!(
        response.params.result,
        Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::NotFound,
            ..
        })
    ));
}
