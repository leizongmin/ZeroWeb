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
                update_via_cache: ServiceWorkerUpdateViaCacheWire::None,
                script_type: ServiceWorkerScriptTypeWire::Module,
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
            update_via_cache: ServiceWorkerUpdateViaCacheWire::None,
            script_type: ServiceWorkerScriptTypeWire::Module,
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
                update_via_cache: ServiceWorkerUpdateViaCacheWire::All,
                script_type: ServiceWorkerScriptTypeWire::Module,
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
            update_via_cache: ServiceWorkerUpdateViaCacheWire::All,
            script_type: ServiceWorkerScriptTypeWire::Module,
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
            update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
            script_type: ServiceWorkerScriptTypeWire::Classic,
        },
    };
    assert!(params.validate().is_err());

    let params = ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::PostMessage {
            registration_id: 1,
            data_json: "x".repeat(1024 * 1024 + 1),
            transferred_port_ids: Vec::new(),
            data_port_index: None,
            target_port_id: None,
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
        (6, ServiceWorkerOperation::Controller),
        (
            7,
            ServiceWorkerOperation::PostMessage {
                registration_id: 9,
                data_json: r#"{"value":"hello"}"#.into(),
                transferred_port_ids: Vec::new(),
                data_port_index: None,
                target_port_id: None,
            },
        ),
        (
            8,
            ServiceWorkerOperation::ClientMessages {
                registration_id: 9,
                after_sequence: 2,
            },
        ),
        (9, ServiceWorkerOperation::Update { registration_id: 9 }),
        (
            10,
            ServiceWorkerOperation::ObserveWindowClient {
                client_id: "iframe:#frame".into(),
                client_url: "/frame.html".into(),
                frame_type: "nested".into(),
            },
        ),
        (
            11,
            ServiceWorkerOperation::RemoveWindowClient {
                client_id: "iframe:#frame".into(),
            },
        ),
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
        claim_clients: true,
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
fn service_worker_client_messages_round_trip() {
    let messages = ServiceWorkerClientMessages {
        latest_sequence: 2,
        messages: vec![r#"{"echo":"hello"}"#.into()],
    };
    let decoded = roundtrip(IpcMessage {
        id: 46,
        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
            result: Ok(ServiceWorkerResult::ClientMessages(messages.clone())),
        }),
    });
    let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
        panic!("expected ServiceWorkerResponse");
    };
    assert_eq!(params.result, Ok(ServiceWorkerResult::ClientMessages(messages)));
}

#[test]
fn service_worker_update_result_round_trips() {
    let decoded = roundtrip(IpcMessage {
        id: 47,
        kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
            result: Ok(ServiceWorkerResult::Updated {
                registration_id: 13,
                changed: true,
            }),
        }),
    });
    let IpcMessageKind::ServiceWorkerResponse(params) = decoded.kind else {
        panic!("expected ServiceWorkerResponse");
    };
    assert_eq!(
        params.result,
        Ok(ServiceWorkerResult::Updated {
            registration_id: 13,
            changed: true,
        })
    );
}

#[test]
fn service_worker_snapshot_list_response_round_trips() {
    let snapshot = ServiceWorkerSnapshot {
        registration_id: 11,
        script_url: "https://example.test/sw.js".into(),
        scope: "https://example.test/app/".into(),
        update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
        script_type: ServiceWorkerScriptTypeWire::Classic,
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
    assert!(
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::ObserveWindowClient {
                client_id: String::new(),
                client_url: "https://example.test/frame.html".into(),
                frame_type: "nested".into(),
            },
        }
        .validate()
        .is_err()
    );
    assert!(
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::ObserveWindowClient {
                client_id: "iframe:#frame".into(),
                client_url: "https://example.test/frame.html".into(),
                frame_type: "none".into(),
            },
        }
        .validate()
        .is_err()
    );
    assert!(
        ServiceWorkerRequestParams {
            operation: ServiceWorkerOperation::RemoveWindowClient {
                client_id: String::new(),
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
            update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
            script_type: ServiceWorkerScriptTypeWire::Classic,
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
    assert_eq!(discriminant(&ServiceWorkerOperation::Controller), 7);
    assert_eq!(
        discriminant(&ServiceWorkerOperation::PostMessage {
            registration_id: 1,
            data_json: "null".into(),
            transferred_port_ids: Vec::new(),
            data_port_index: None,
            target_port_id: None,
        }),
        8
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::ClientMessages {
            registration_id: 1,
            after_sequence: 0,
        }),
        9
    );
    assert_eq!(discriminant(&ServiceWorkerOperation::Update { registration_id: 1 }), 10);
    assert_eq!(
        discriminant(&ServiceWorkerOperation::ObserveWindowClient {
            client_id: "iframe:#frame".into(),
            client_url: "https://example.test/frame.html".into(),
            frame_type: "nested".into(),
        }),
        11
    );
    assert_eq!(
        discriminant(&ServiceWorkerOperation::RemoveWindowClient {
            client_id: "iframe:#frame".into(),
        }),
        12
    );

    assert_eq!(discriminant(&ServiceWorkerResult::Registered { registration_id: 1 }), 0);
    assert_eq!(
        discriminant(&ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
            registration_id: 1,
            script_url: "https://example.test/sw.js".into(),
            scope: "https://example.test/".into(),
            update_via_cache: ServiceWorkerUpdateViaCacheWire::Imports,
            script_type: ServiceWorkerScriptTypeWire::Classic,
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
            claim_clients: false,
        })),
        6
    );
    assert_eq!(
        discriminant(&ServiceWorkerResult::ClientMessages(ServiceWorkerClientMessages {
            latest_sequence: 0,
            messages: Vec::new(),
        })),
        7
    );
    assert_eq!(
        discriminant(&ServiceWorkerResult::Updated {
            registration_id: 1,
            changed: true,
        }),
        8
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

#[test]
fn service_worker_host_command_round_trips_and_validates() {
    let message = IpcMessage {
        id: 51,
        kind: IpcMessageKind::ServiceWorkerHostCommand(ServiceWorkerHostCommandParams {
            registration_id: 7,
            command: ServiceWorkerHostCommand::Evaluate {
                script_url: "https://example.test/sw.js".into(),
                script: "globalThis.ready = true;".into(),
                script_type: ServiceWorkerScriptTypeWire::Module,
            },
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerHostCommand(params) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert!(params.validate().is_ok());
    assert_eq!(
        params.command,
        ServiceWorkerHostCommand::Evaluate {
            script_url: "https://example.test/sw.js".into(),
            script: "globalThis.ready = true;".into(),
            script_type: ServiceWorkerScriptTypeWire::Module,
        }
    );

    let invalid = ServiceWorkerHostCommandParams {
        registration_id: 7,
        command: ServiceWorkerHostCommand::Evaluate {
            script_url: String::new(),
            script: "void 0;".into(),
            script_type: ServiceWorkerScriptTypeWire::Classic,
        },
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn service_worker_host_event_round_trips() {
    let message = IpcMessage {
        id: 52,
        kind: IpcMessageKind::ServiceWorkerHostEvent(ServiceWorkerHostEventParams {
            registration_id: 7,
            event: ServiceWorkerHostEvent::ScriptError {
                script_url: "https://example.test/sw.js".into(),
                kind: ServiceWorkerScriptErrorKindWire::Compile,
                message: "SyntaxError".into(),
            },
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerHostEvent(params) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(
        params.event,
        ServiceWorkerHostEvent::ScriptError {
            script_url: "https://example.test/sw.js".into(),
            kind: ServiceWorkerScriptErrorKindWire::Compile,
            message: "SyntaxError".into(),
        }
    );
}

#[test]
fn service_worker_host_message_command_round_trips_and_validates() {
    let message = IpcMessage {
        id: 53,
        kind: IpcMessageKind::ServiceWorkerHostCommand(ServiceWorkerHostCommandParams {
            registration_id: 4,
            command: ServiceWorkerHostCommand::DispatchMessage {
                event_id: 11,
                data_json: "{\"type\":\"ping\"}".into(),
                client_id: "tab-7".into(),
                client_url: "https://example.test/page".into(),
                transferred_port_ids: Vec::new(),
                data_port_index: None,
                target_port_id: None,
            },
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerHostCommand(params) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert!(params.validate().is_ok());
    assert!(matches!(
        params.command,
        ServiceWorkerHostCommand::DispatchMessage { event_id: 11, ref client_id, .. } if client_id == "tab-7"
    ));

    let invalid = ServiceWorkerHostCommandParams {
        registration_id: 4,
        command: ServiceWorkerHostCommand::DispatchMessage {
            event_id: 11,
            data_json: "null".into(),
            client_id: String::new(),
            client_url: "https://example.test/page".into(),
            transferred_port_ids: Vec::new(),
            data_port_index: None,
            target_port_id: None,
        },
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn service_worker_host_message_event_round_trips() {
    let message = IpcMessage {
        id: 54,
        kind: IpcMessageKind::ServiceWorkerHostEvent(ServiceWorkerHostEventParams {
            registration_id: 4,
            event: ServiceWorkerHostEvent::MessageDispatched {
                event_id: 11,
                client_id: "tab-7".into(),
                outbound: vec!["{\"type\":\"pong\"}".into()],
            },
        }),
    };
    let decoded = roundtrip(message);
    let IpcMessageKind::ServiceWorkerHostEvent(params) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(
        params.event,
        ServiceWorkerHostEvent::MessageDispatched {
            event_id: 11,
            client_id: "tab-7".into(),
            outbound: vec!["{\"type\":\"pong\"}".into()],
        }
    );

    let settled = IpcMessage {
        id: 55,
        kind: IpcMessageKind::ServiceWorkerHostEvent(ServiceWorkerHostEventParams {
            registration_id: 4,
            event: ServiceWorkerHostEvent::LifecycleSettled {
                phase: ServiceWorkerLifecycleWire::Install,
                succeeded: true,
                skip_waiting: true,
                claim_clients: false,
                message: String::new(),
            },
        }),
    };
    let decoded = roundtrip(settled);
    let IpcMessageKind::ServiceWorkerHostEvent(params) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(
        params.event,
        ServiceWorkerHostEvent::LifecycleSettled {
            phase: ServiceWorkerLifecycleWire::Install,
            succeeded: true,
            skip_waiting: true,
            claim_clients: false,
            message: String::new(),
        }
    );
}

#[test]
fn service_worker_message_port_and_update_wires_round_trip() {
    let request = ServiceWorkerRequestParams {
        operation: ServiceWorkerOperation::PostMessage {
            registration_id: 7,
            data_json: "null".into(),
            transferred_port_ids: vec![2],
            data_port_index: Some(0),
            target_port_id: None,
        },
    };
    assert!(request.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 70,
        kind: IpcMessageKind::ServiceWorkerRequest(request.clone()),
    });
    let IpcMessageKind::ServiceWorkerRequest(decoded_request) = decoded.kind else {
        panic!("expected ServiceWorkerRequest");
    };
    assert_eq!(decoded_request, request);

    let event = ServiceWorkerHostEventParams {
        registration_id: 7,
        event: ServiceWorkerHostEvent::UpdateRequested { request_id: 11 },
    };
    let decoded = roundtrip(IpcMessage {
        id: 71,
        kind: IpcMessageKind::ServiceWorkerHostEvent(event.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, event);

    let command = ServiceWorkerHostCommandParams {
        registration_id: 7,
        command: ServiceWorkerHostCommand::CompleteUpdate {
            request_id: 11,
            result: Err(ServiceWorkerUpdateError {
                exception_name: "InvalidStateError".into(),
                message: "installing worker cannot update".into(),
            }),
        },
    };
    assert!(command.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 72,
        kind: IpcMessageKind::ServiceWorkerHostCommand(command.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostCommand(decoded_command) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert_eq!(decoded_command, command);

    let event = ServiceWorkerHostEventParams {
        registration_id: 7,
        event: ServiceWorkerHostEvent::ClientsMatchAllRequested {
            request_id: 12,
            include_uncontrolled: true,
            client_type: "window".into(),
        },
    };
    let decoded = roundtrip(IpcMessage {
        id: 73,
        kind: IpcMessageKind::ServiceWorkerHostEvent(event.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, event);

    let command = ServiceWorkerHostCommandParams {
        registration_id: 7,
        command: ServiceWorkerHostCommand::CompleteClientsMatchAll {
            request_id: 12,
            result: Ok(vec![ServiceWorkerClientInfoWire {
                id: "client-1".into(),
                url: "https://example.test/page".into(),
                client_type: "window".into(),
                frame_type: "auxiliary".into(),
                visibility_state: "visible".into(),
                focused: false,
            }]),
        },
    };
    assert!(command.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 74,
        kind: IpcMessageKind::ServiceWorkerHostCommand(command.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostCommand(decoded_command) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert_eq!(decoded_command, command);

    let event = ServiceWorkerHostEventParams {
        registration_id: 7,
        event: ServiceWorkerHostEvent::ClientsGetRequested {
            request_id: 13,
            client_id: "client-1".into(),
        },
    };
    assert!(event.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 76,
        kind: IpcMessageKind::ServiceWorkerHostEvent(event.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, event);

    let command = ServiceWorkerHostCommandParams {
        registration_id: 7,
        command: ServiceWorkerHostCommand::CompleteClientsGet {
            request_id: 13,
            result: Ok(Some(ServiceWorkerClientInfoWire {
                id: "client-1".into(),
                url: "https://example.test/page".into(),
                client_type: "window".into(),
                frame_type: "auxiliary".into(),
                visibility_state: "visible".into(),
                focused: false,
            })),
        },
    };
    assert!(command.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 77,
        kind: IpcMessageKind::ServiceWorkerHostCommand(command.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostCommand(decoded_command) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert_eq!(decoded_command, command);

    let emitted = ServiceWorkerHostEventParams {
        registration_id: 0,
        event: ServiceWorkerHostEvent::ClientMessagesEmitted {
            outbound: vec![ServiceWorkerMessage {
                data_json: "\"matched\"".into(),
                port_id: None,
                transferred_port_ids: Vec::new(),
                data_port_index: None,
                target_client_id: Some("client-1".into()),
            }],
        },
    };
    assert!(emitted.validate().is_ok(), "registration zero is a valid registry id");
    let decoded = roundtrip(IpcMessage {
        id: 75,
        kind: IpcMessageKind::ServiceWorkerHostEvent(emitted.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, emitted);
    assert!(
        ServiceWorkerHostEventParams {
            registration_id: 7,
            event: ServiceWorkerHostEvent::ClientsMatchAllRequested {
                request_id: 12,
                include_uncontrolled: true,
                client_type: "invalid".into(),
            },
        }
        .validate()
        .is_err()
    );
    assert!(
        ServiceWorkerHostEventParams {
            registration_id: 7,
            event: ServiceWorkerHostEvent::ClientsGetRequested {
                request_id: 13,
                client_id: String::new(),
            },
        }
        .validate()
        .is_err()
    );
    assert!(
        ServiceWorkerHostCommandParams {
            registration_id: 7,
            command: ServiceWorkerHostCommand::CompleteClientsGet {
                request_id: 13,
                result: Ok(Some(ServiceWorkerClientInfoWire {
                    id: "client-1".into(),
                    url: "https://example.test/page".into(),
                    client_type: "window".into(),
                    frame_type: "detached".into(),
                    visibility_state: "visible".into(),
                    focused: false,
                })),
            },
        }
        .validate()
        .is_err()
    );
}

#[test]
fn service_worker_host_import_scripts_round_trips_and_validates() {
    let command = ServiceWorkerHostCommandParams {
        registration_id: 9,
        command: ServiceWorkerHostCommand::CompleteImportScripts {
            request_id: 3,
            result: Ok(vec!["globalThis.imported = true;".into()]),
        },
    };
    assert!(command.validate().is_ok());
    let decoded = roundtrip(IpcMessage {
        id: 56,
        kind: IpcMessageKind::ServiceWorkerHostCommand(command.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostCommand(decoded_command) = decoded.kind else {
        panic!("expected ServiceWorkerHostCommand");
    };
    assert_eq!(decoded_command, command);

    let event = ServiceWorkerHostEventParams {
        registration_id: 9,
        event: ServiceWorkerHostEvent::ImportScriptsRequested {
            request_id: 3,
            specifiers: vec!["./dependency.js".into()],
        },
    };
    let decoded = roundtrip(IpcMessage {
        id: 57,
        kind: IpcMessageKind::ServiceWorkerHostEvent(event.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, event);

    let module_event = ServiceWorkerHostEventParams {
        registration_id: 9,
        event: ServiceWorkerHostEvent::ModuleScriptsRequested {
            request_id: 4,
            referrer_url: "https://example.test/workers/sw.js".into(),
            specifiers: vec!["./dependency.js".into()],
        },
    };
    let decoded = roundtrip(IpcMessage {
        id: 58,
        kind: IpcMessageKind::ServiceWorkerHostEvent(module_event.clone()),
    });
    let IpcMessageKind::ServiceWorkerHostEvent(decoded_event) = decoded.kind else {
        panic!("expected ServiceWorkerHostEvent");
    };
    assert_eq!(decoded_event, module_event);

    assert!(
        ServiceWorkerHostCommandParams {
            registration_id: 9,
            command: ServiceWorkerHostCommand::CompleteImportScripts {
                request_id: 0,
                result: Ok(Vec::new()),
            },
        }
        .validate()
        .is_err()
    );
}
