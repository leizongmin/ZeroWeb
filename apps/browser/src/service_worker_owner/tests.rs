use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn register_request(document_url: &str) -> ServiceWorkerRequestParams {
    ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::Register {
            script_url: "/sw.js".into(),
            scope: Some("/app/".into()),
            document_url: document_url.into(),
        },
    }
}

fn attach_script(owner: &mut BrowserServiceWorkerOwner, disposition: ServiceWorkerRequestDisposition, script: &str) {
    let ServiceWorkerRequestDisposition::Fetch(plan) = disposition else {
        panic!("expected fetch plan");
    };
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

#[test]
fn register_fetches_evaluates_and_returns_correlated_id() {
    let mut owner = BrowserServiceWorkerOwner::new();
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
fn lifecycle_state_changes_are_cursor_based_and_ordered() {
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
    let mut owner = BrowserServiceWorkerOwner::new();
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
