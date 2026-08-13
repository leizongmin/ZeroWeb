//! M4 automation IPC contract tests.

use super::*;

fn element() -> AutomationElementRef {
    AutomationElementRef {
        navigation_epoch: 7,
        document_generation: 3,
        node_handle: 42,
    }
}

#[test]
fn automation_operations_roundtrip() {
    let operations = vec![
        AutomationOperation::FindElement {
            using: AutomationLocatorStrategy::CssSelector,
            value: "#name".into(),
        },
        AutomationOperation::ElementClick { element: element() },
        AutomationOperation::SendKeys {
            element: element(),
            keys: vec![
                AutomationKey::Text("ZeroWeb".into()),
                AutomationKey::Tab,
                AutomationKey::ShiftTab,
                AutomationKey::Backspace,
                AutomationKey::Enter,
            ],
        },
        AutomationOperation::GetActiveElement,
        AutomationOperation::ExecuteScript {
            script: "return arguments[0]".into(),
            arguments: vec![AutomationValue::Object(vec![(
                "items".into(),
                AutomationValue::Array(vec![AutomationValue::Bool(true), AutomationValue::Number(2.0)]),
            )])],
        },
        AutomationOperation::Unsupported {
            name: "test_driver.set_permission".into(),
        },
    ];

    for (index, operation) in operations.into_iter().enumerate() {
        let message = IpcMessage {
            id: index as u64 + 1,
            kind: IpcMessageKind::AutomationRequest(AutomationRequest {
                operation: operation.clone(),
            }),
        };
        let decoded = roundtrip(message);
        let IpcMessageKind::AutomationRequest(request) = decoded.kind else {
            panic!("expected AutomationRequest");
        };
        assert_eq!(request.operation, operation);
    }
}

#[test]
fn automation_unicode_and_long_text_roundtrip() {
    let text = format!("浏览器🙂e\u{301}{}", "x".repeat(32 * 1024));
    let message = IpcMessage {
        id: 9,
        kind: IpcMessageKind::AutomationRequest(AutomationRequest {
            operation: AutomationOperation::SendKeys {
                element: element(),
                keys: vec![AutomationKey::Text(text.clone())],
            },
        }),
    };

    let decoded = roundtrip(message);
    let IpcMessageKind::AutomationRequest(AutomationRequest {
        operation: AutomationOperation::SendKeys { keys, .. },
    }) = decoded.kind
    else {
        panic!("expected SendKeys");
    };
    assert_eq!(keys, vec![AutomationKey::Text(text)]);
}

#[test]
fn automation_response_roundtrips_missing_element_and_values() {
    let responses = vec![
        AutomationResponse {
            navigation_epoch: 7,
            document_generation: 3,
            result: Ok(AutomationResult::Element(None)),
        },
        AutomationResponse {
            navigation_epoch: 7,
            document_generation: 3,
            result: Ok(AutomationResult::Value(AutomationValue::String("ok".into()))),
        },
        AutomationResponse {
            navigation_epoch: 7,
            document_generation: 3,
            result: Err(AutomationError {
                code: AutomationErrorCode::StaleElementReference,
                message: "element belongs to an old document".into(),
            }),
        },
        AutomationResponse {
            navigation_epoch: 7,
            document_generation: 3,
            result: Err(AutomationError {
                code: AutomationErrorCode::UnsupportedOperation,
                message: "unsupported automation operation".into(),
            }),
        },
    ];

    for (index, response) in responses.into_iter().enumerate() {
        let decoded = roundtrip(IpcMessage {
            id: index as u64 + 20,
            kind: IpcMessageKind::AutomationResponse(response.clone()),
        });
        let IpcMessageKind::AutomationResponse(actual) = decoded.kind else {
            panic!("expected AutomationResponse");
        };
        assert_eq!(actual, response);
    }
}
