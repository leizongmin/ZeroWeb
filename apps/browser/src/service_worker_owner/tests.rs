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
