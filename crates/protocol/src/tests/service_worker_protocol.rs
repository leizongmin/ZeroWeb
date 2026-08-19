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

#[test]
fn service_worker_discovery_operations_round_trip() {
    for (id, operation) in [
        (
            4,
            ServiceWorkerOperation::GetRegistration {
                client_url: "https://example.test/app/page".into(),
            },
        ),
        (5, ServiceWorkerOperation::GetRegistrations),
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

#[test]
fn service_worker_state_changes_round_trip() {
    let operation = ServiceWorkerOperation::StateChanges {
        registration_id: 12,
        after_sequence: 1,
    };
    let decoded = roundtrip(IpcMessage {
        id: 45,
        kind: IpcMessageKind::ServiceWorkerRequest(ServiceWorkerRequestParams {
            operation: operation.clone(),
        }),
    });
    let IpcMessageKind::ServiceWorkerRequest(params) = decoded.kind else {
        panic!("expected ServiceWorkerRequest");
    };
    assert_eq!(params.operation, operation);
    assert!(params.validate().is_ok());

    let changes = ServiceWorkerStateChanges {
        latest_sequence: 3,
        states: vec![ServiceWorkerStateWire::Activating, ServiceWorkerStateWire::Activated],
    };
    let decoded = roundtrip(IpcMessage {
        id: 45,
        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
            result: Ok(ServiceWorkerResult::StateChanges(changes.clone())),
        }),
    });
    let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
        panic!("expected ServiceWorkerResponse");
    };
    assert_eq!(params.result, Ok(ServiceWorkerResult::StateChanges(changes)));
}

#[test]
fn service_worker_snapshot_list_response_round_trips() {
    let snapshot = ServiceWorkerSnapshot {
        registration_id: 11,
        script_url: "https://example.test/sw.js".into(),
        scope: "https://example.test/app/".into(),
        state: ServiceWorkerStateWire::Activated,
    };
    for result in [
        ServiceWorkerResult::OptionalSnapshot(Some(snapshot.clone())),
        ServiceWorkerResult::OptionalSnapshot(None),
        ServiceWorkerResult::Snapshots(vec![snapshot.clone()]),
    ] {
        let decoded = roundtrip(IpcMessage {
            id: 44,
            kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
                result: Ok(result.clone()),
            }),
        });
        let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
            panic!("expected ServiceWorkerResponse");
        };
        assert_eq!(params.result, Ok(result));
    }
}

#[test]
fn service_worker_discovery_rejects_invalid_client_url() {
    assert!(
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::GetRegistration {
                client_url: String::new(),
            },
        }
        .validate()
        .is_err()
    );
    assert!(
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::GetRegistration {
                client_url: "x".repeat(64 * 1024 + 1),
            },
        }
        .validate()
        .is_err()
    );
}

#[test]
fn service_worker_nested_enum_discriminants_remain_append_only() {
    fn discriminant(value: &impl serde::Serialize) -> u32 {
        let bytes = bincode::serialize(value).unwrap();
        u32::from_le_bytes(bytes[..4].try_into().unwrap())
    }

    assert_eq!(
        discriminant(&ServiceWorkerOperation::Register {
            script_url: "/sw.js".into(),
            scope: None,
            document_url: "https://example.test/".into(),
        }),
        0
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::Snapshot { registration_id: 1 }),
        1
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::Unregister { registration_id: 1 }),
        2
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::ActivateWaiting { registration_id: 1 }),
        3
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::GetRegistration {
            client_url: "https://example.test/".into(),
        }),
        4
    );
    assert_eq!(discriminant(&ServiceWorkerOperation::GetRegistrations), 5);
    assert_eq!(
        discriminant(&ServiceWorkerOperation::StateChanges {
            registration_id: 1,
            after_sequence: 0,
        }),
        6
    );

    assert_eq!(discriminant(&ServiceWorkerResult::Registered { registration_id: 1 }), 0);
    assert_eq!(
        discriminant(&ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
            registration_id: 1,
            script_url: "https://example.test/sw.js".into(),
            scope: "https://example.test/".into(),
            state: ServiceWorkerStateWire::Activated,
        })),
        1
    );
    assert_eq!(discriminant(&ServiceWorkerResult::Boolean(true)), 2);
    assert_eq!(discriminant(&ServiceWorkerResult::Empty), 3);
    assert_eq!(discriminant(&ServiceWorkerResult::OptionalSnapshot(None)), 4);
    assert_eq!(discriminant(&ServiceWorkerResult::Snapshots(Vec::new())), 5);
    assert_eq!(
        discriminant(&ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
            latest_sequence: 0,
            states: Vec::new(),
        })),
        6
    );

    for (index, code) in [
        ServiceWorkerErrorCode::InvalidArgument,
        ServiceWorkerErrorCode::NotFound,
        ServiceWorkerErrorCode::InvalidState,
        ServiceWorkerErrorCode::Network,
        ServiceWorkerErrorCode::Script,
        ServiceWorkerErrorCode::Capacity,
        ServiceWorkerErrorCode::Internal,
        ServiceWorkerErrorCode::Security,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(discriminant(&code), index as u32);
    }
}
