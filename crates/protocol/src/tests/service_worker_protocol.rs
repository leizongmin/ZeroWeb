use super::*;

#[test]
fn service_worker_register_request_round_trips_without_script_source() {
    let message = IpcMessage {
        id: 41,
        kind: IpcMessageKind::ServiceWorkerRequest(ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::Register {
                script_url: "/sw.js".into(),
                scope: Some("/app/".into()),
                document_url: "https://example.test/page.html".into(),
            },
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerRequest(params) = decoded.kind else {
        panic!("expected ServiceWorkerRequest");
    };
    assert!(params.validate().is_ok());
    assert_eq!(
        params.operation,
        ServiceWorkerOperation::Register {
            script_url: "/sw.js".into(),
            scope: Some("/app/".into()),
            document_url: "https://example.test/page.html".into(),
        }
    );
}

#[test]
fn service_worker_snapshot_response_round_trips() {
    let message = IpcMessage {
        id: 42,
        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
            result: Ok(ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
                registration_id: 7,
                script_url: "https://example.test/sw.js".into(),
                scope: "https://example.test/app/".into(),
                state: ServiceWorkerStateWire::Activated,
            })),
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
        panic!("expected ServiceWorkerResponse");
    };
    assert_eq!(
        params.result,
        Ok(ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
            registration_id: 7,
            script_url: "https://example.test/sw.js".into(),
            scope: "https://example.test/app/".into(),
            state: ServiceWorkerStateWire::Activated,
        }))
    );
}

#[test]
fn service_worker_error_response_round_trips() {
    let message = IpcMessage {
        id: 43,
        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
            result: Err(ServiceWorkerError {
                code: ServiceWorkerErrorCode::InvalidArgument,
                message: "cross-origin script URL".into(),
            }),
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
        panic!("expected ServiceWorkerResponse");
    };
    assert_eq!(
        params.result,
        Err(ServiceWorkerError {
            code: ServiceWorkerErrorCode::InvalidArgument,
            message: "cross-origin script URL".into(),
        })
    );
}

#[test]
fn service_worker_request_rejects_oversized_urls() {
    let oversized = "x".repeat(64 * 1024 + 1);
    let params = ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::Register {
            script_url: oversized,
            scope: None,
            document_url: "https://example.test/".into(),
        },
    };
    assert!(params.validate().is_err());
}

#[test]
fn service_worker_id_operations_round_trip() {
    for (id, operation) in [
        (1, ServiceWorkerOperation::Snapshot { registration_id: 9 }),
        (2, ServiceWorkerOperation::Unregister { registration_id: 9 }),
        (3, ServiceWorkerOperation::ActivateWaiting { registration_id: 9 }),
    ] {
        let decoded = roundtrip(IpcMessage {
            id,
            kind: IpcMessageKind::ServiceWorkerRequest(ServiceWorkerRequestParams {
                operation: operation.clone(),
            }),
        });
        let IpcMessageKind::ServiceWorkerRequest(params) = decoded.kind else {
            panic!("expected ServiceWorkerRequest");
        };
        assert_eq!(params.operation, operation);
        assert!(params.validate().is_ok());
    }
}
