//! # zero-protocol
//!
//! 多进程 IPC 与协议定义。

#![warn(missing_docs)]

pub mod channel;
pub mod message;
pub mod serialize;

pub use channel::*;
pub use message::*;
pub use serialize::*;

use thiserror::Error;

/// 协议错误类型。
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// 序列化错误。
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// 反序列化错误。
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    /// 通道错误。
    #[error("Channel error: {0}")]
    Channel(String),
    /// 进程错误。
    #[error("Process error: {0}")]
    Process(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(msg: IpcMessage) -> IpcMessage {
        let bytes = serialize(&msg).expect("serialize");
        deserialize(&bytes).expect("deserialize")
    }

    #[test]
    fn test_serialize_deserialize_navigate() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: Some("https://referrer.com".into()),
            }),
        };
        let out = roundtrip(msg.clone());
        assert_eq!(msg.id, out.id);
        assert!(matches!(out.kind, IpcMessageKind::Navigate(_)));
    }

    #[test]
    fn test_serialize_deserialize_go_back() {
        let msg = IpcMessage {
            id: 2,
            kind: IpcMessageKind::GoBack,
        };
        let out = roundtrip(msg);
        assert_eq!(2, out.id);
        assert!(matches!(out.kind, IpcMessageKind::GoBack));
    }

    #[test]
    fn test_serialize_deserialize_title_changed() {
        let msg = IpcMessage {
            id: 3,
            kind: IpcMessageKind::TitleChanged("Hello".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::TitleChanged(t) = out.kind {
            assert_eq!("Hello", t);
        } else {
            panic!("expected TitleChanged");
        }
    }

    #[test]
    fn test_serialize_deserialize_fetch_request() {
        let msg = IpcMessage {
            id: 4,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 100,
                url: "https://api.example.com/data".into(),
                method: "GET".into(),
                headers: vec![("Accept".into(), "application/json".into())],
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(100, p.request_id);
            assert_eq!("GET", p.method);
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_serialize_deserialize_fetch_response() {
        let msg = IpcMessage {
            id: 5,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 100,
                status_code: 200,
                headers: vec![("Content-Type".into(), "text/html".into())],
                body: b"<html>".to_vec(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(200, p.status_code);
            assert_eq!(b"<html>", p.body.as_slice());
        } else {
            panic!("expected FetchResponse");
        }
    }

    #[test]
    fn test_serialize_deserialize_storage_op() {
        let msg = IpcMessage {
            id: 6,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "theme".into(),
                value: Some("dark".into()),
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageType::Local, p.storage_type);
            assert_eq!(StorageOperation::Set, p.operation);
            assert_eq!("theme", p.key);
            assert_eq!(Some("dark".into()), p.value);
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_serialize_deserialize_mouse_event() {
        let msg = IpcMessage {
            id: 7,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 100.0,
                y: 200.0,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(100.0, p.x);
            assert_eq!(200.0, p.y);
            assert_eq!(MouseEventType::Click, p.event_type);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_serialize_deserialize_keyboard_event() {
        let msg = IpcMessage {
            id: 8,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".into(),
                code: "KeyA".into(),
                ctrl: true,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert!(p.ctrl);
            assert!(!p.shift);
            assert_eq!(KeyboardEventType::Down, p.event_type);
        } else {
            panic!("expected KeyboardEvent");
        }
    }

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let msg = IpcMessage {
            id: 9,
            kind: IpcMessageKind::Heartbeat,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::Heartbeat));
    }

    #[test]
    fn test_serialize_deserialize_crash() {
        let msg = IpcMessage {
            id: 10,
            kind: IpcMessageKind::CrashNotification("OOM".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::CrashNotification(reason) = out.kind {
            assert_eq!("OOM", reason);
        } else {
            panic!("expected CrashNotification");
        }
    }

    #[test]
    fn test_serialize_deserialize_ok() {
        let msg = IpcMessage {
            id: 11,
            kind: IpcMessageKind::Ok,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::Ok));
    }

    #[test]
    fn test_serialize_deserialize_error() {
        let msg = IpcMessage {
            id: 12,
            kind: IpcMessageKind::Error("something went wrong".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Error(e) = out.kind {
            assert_eq!("something went wrong", e);
        } else {
            panic!("expected Error");
        }
    }

    #[test]
    fn test_roundtrip_all_message_types() {
        let msgs: Vec<IpcMessageKind> = vec![
            IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
            }),
            IpcMessageKind::GoBack,
            IpcMessageKind::GoForward,
            IpcMessageKind::StopLoading,
            IpcMessageKind::Reload,
            IpcMessageKind::TitleChanged("T".into()),
            IpcMessageKind::UrlChanged("https://example.com".into()),
            IpcMessageKind::LoadComplete,
            IpcMessageKind::LoadFailed("timeout".into()),
            IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://example.com".into(),
                method: "POST".into(),
                headers: vec![],
                body: Some(vec![1, 2, 3]),
            }),
            IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 200,
                headers: vec![],
                body: vec![],
            }),
            IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Get,
                key: "k".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
            IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: 1,
                event_type: MouseEventType::Down,
            }),
            IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "Enter".into(),
                code: "Enter".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Press,
            }),
            IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 10.0,
                delta_y: -20.0,
            }),
            IpcMessageKind::Heartbeat,
            IpcMessageKind::CrashNotification("segfault".into()),
            IpcMessageKind::Ok,
            IpcMessageKind::Error("err".into()),
        ];

        for (i, kind) in msgs.into_iter().enumerate() {
            let msg = IpcMessage { id: i as u64, kind };
            let bytes = serialize(&msg).expect("serialize should succeed");
            let out: IpcMessage = deserialize(&bytes).expect("deserialize should succeed");
            assert_eq!(i as u64, out.id);
        }
    }

    #[test]
    fn test_message_id_preserved() {
        let msg = IpcMessage {
            id: u64::MAX,
            kind: IpcMessageKind::Ok,
        };
        let out = roundtrip(msg);
        assert_eq!(u64::MAX, out.id);
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let result = deserialize(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_empty_data() {
        let result = deserialize(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_navigate_params_fields() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com/page".into(),
                referrer: Some("https://google.com".into()),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Navigate(p) = out.kind {
            assert_eq!("https://example.com/page", p.url);
            assert_eq!(Some("https://google.com".into()), p.referrer);
        } else {
            panic!("expected Navigate");
        }
    }

    #[test]
    fn test_fetch_params_with_body() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 42,
                url: "https://api.example.com".into(),
                method: "POST".into(),
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: Some(b"{\"key\":\"value\"}".to_vec()),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(Some(b"{\"key\":\"value\"}".to_vec()), p.body);
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_fetch_params_without_body() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 42,
                url: "https://api.example.com".into(),
                method: "GET".into(),
                headers: vec![],
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert!(p.body.is_none());
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_storage_op_get() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Get,
                key: "token".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Get, p.operation);
            assert!(p.value.is_none());
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_storage_op_set() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Set,
                key: "session_id".into(),
                value: Some("abc123".into()),
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Set, p.operation);
            assert_eq!(Some("abc123".into()), p.value);
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_mouse_event_click() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 150.5,
                y: 300.25,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(150.5, p.x);
            assert_eq!(300.25, p.y);
            assert_eq!(0, p.button);
            assert_eq!(MouseEventType::Click, p.event_type);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_keyboard_event_with_modifiers() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "c".into(),
                code: "KeyC".into(),
                ctrl: true,
                shift: false,
                alt: false,
                meta: true,
                event_type: KeyboardEventType::Press,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert!(p.ctrl);
            assert!(!p.shift);
            assert!(!p.alt);
            assert!(p.meta);
            assert_eq!(KeyboardEventType::Press, p.event_type);
        } else {
            panic!("expected KeyboardEvent");
        }
    }

    #[test]
    fn test_scroll_event() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: -5.0,
                delta_y: 15.5,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::ScrollEvent(p) = out.kind {
            assert_eq!(-5.0, p.delta_x);
            assert_eq!(15.5, p.delta_y);
        } else {
            panic!("expected ScrollEvent");
        }
    }

    // ── 缺失的 IpcMessageKind 变体字段验证 ──

    #[test]
    fn test_serialize_deserialize_go_forward() {
        let msg = IpcMessage {
            id: 20,
            kind: IpcMessageKind::GoForward,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::GoForward));
    }

    #[test]
    fn test_serialize_deserialize_stop_loading() {
        let msg = IpcMessage {
            id: 21,
            kind: IpcMessageKind::StopLoading,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::StopLoading));
    }

    #[test]
    fn test_serialize_deserialize_reload() {
        let msg = IpcMessage {
            id: 22,
            kind: IpcMessageKind::Reload,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::Reload));
    }

    #[test]
    fn test_serialize_deserialize_url_changed() {
        let msg = IpcMessage {
            id: 23,
            kind: IpcMessageKind::UrlChanged("https://example.com/new".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::UrlChanged(url) = out.kind {
            assert_eq!("https://example.com/new", url);
        } else {
            panic!("expected UrlChanged");
        }
    }

    #[test]
    fn test_serialize_deserialize_load_complete() {
        let msg = IpcMessage {
            id: 24,
            kind: IpcMessageKind::LoadComplete,
        };
        let out = roundtrip(msg);
        assert!(matches!(out.kind, IpcMessageKind::LoadComplete));
    }

    #[test]
    fn test_serialize_deserialize_load_failed() {
        let msg = IpcMessage {
            id: 25,
            kind: IpcMessageKind::LoadFailed("connection timeout".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::LoadFailed(reason) = out.kind {
            assert_eq!("connection timeout", reason);
        } else {
            panic!("expected LoadFailed");
        }
    }

    // ── 缺失的 StorageOperation 变体 ──

    #[test]
    fn test_storage_op_remove() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Remove,
                key: "cache".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Remove, p.operation);
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_storage_op_clear() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Clear,
                key: String::new(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Clear, p.operation);
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_storage_op_length() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Length,
                key: String::new(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Length, p.operation);
        } else {
            panic!("expected StorageOp");
        }
    }

    #[test]
    fn test_storage_op_key() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Key,
                key: "0".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert_eq!(StorageOperation::Key, p.operation);
        } else {
            panic!("expected StorageOp");
        }
    }

    // ── 缺失的 MouseEventType 变体 ──

    #[test]
    fn test_mouse_event_down() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: 1,
                event_type: MouseEventType::Down,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(MouseEventType::Down, p.event_type);
            assert_eq!(1, p.button);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_mouse_event_up() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 50.0,
                y: 75.0,
                button: 0,
                event_type: MouseEventType::Up,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(MouseEventType::Up, p.event_type);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_mouse_event_move() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 123.45,
                y: 678.9,
                button: 0,
                event_type: MouseEventType::Move,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(MouseEventType::Move, p.event_type);
            assert_eq!(123.45, p.x);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_mouse_event_dblclick() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 200.0,
                y: 300.0,
                button: 0,
                event_type: MouseEventType::DblClick,
            }),
        };
        let out = roundtrip(msg);
        assert!(matches!(
            out.kind,
            IpcMessageKind::MouseEvent(MouseEventParams {
                event_type: MouseEventType::DblClick,
                ..
            })
        ));
    }

    // ── 缺失的 KeyboardEventType ──

    #[test]
    fn test_keyboard_event_up() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".into(),
                code: "KeyA".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Up,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert_eq!(KeyboardEventType::Up, p.event_type);
        } else {
            panic!("expected KeyboardEvent");
        }
    }

    // ── ProcessRole ──

    #[test]
    fn test_process_role_equality() {
        assert_eq!(ProcessRole::Browser, ProcessRole::Browser);
        assert_eq!(ProcessRole::Renderer, ProcessRole::Renderer);
        assert_ne!(ProcessRole::Browser, ProcessRole::Renderer);
    }

    #[test]
    fn test_process_role_copy_clone() {
        let role = ProcessRole::Browser;
        let cloned = role;
        assert_eq!(role, cloned);
        // Copy semantics — role is still usable
        let copied = role;
        assert_eq!(copied, cloned);
    }

    // ── ProtocolError 显示格式 ──

    #[test]
    fn test_protocol_error_display_serialization() {
        let err = ProtocolError::Serialization("overflow".into());
        let msg = err.to_string();
        assert!(msg.contains("Serialization error"), "message: {msg}");
        assert!(msg.contains("overflow"));
    }

    #[test]
    fn test_protocol_error_display_deserialization() {
        let err = ProtocolError::Deserialization("truncated".into());
        let msg = err.to_string();
        assert!(msg.contains("Deserialization error"), "message: {msg}");
    }

    #[test]
    fn test_protocol_error_display_channel() {
        let err = ProtocolError::Channel("broken pipe".into());
        let msg = err.to_string();
        assert!(msg.contains("Channel error"), "message: {msg}");
        assert!(msg.contains("broken pipe"));
    }

    #[test]
    fn test_protocol_error_display_process() {
        let err = ProtocolError::Process("segfault".into());
        let msg = err.to_string();
        assert!(msg.contains("Process error"), "message: {msg}");
    }

    // ── 边界条件和错误恢复 ──

    #[test]
    fn test_message_id_zero() {
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::Ok,
        };
        let out = roundtrip(msg);
        assert_eq!(0, out.id);
    }

    #[test]
    fn test_fetch_response_status_code_boundary() {
        for code in [0u16, 100, 404, 500, 599, u16::MAX] {
            let msg = IpcMessage {
                id: 1,
                kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                    request_id: 1,
                    status_code: code,
                    headers: vec![],
                    body: vec![],
                }),
            };
            let out = roundtrip(msg);
            if let IpcMessageKind::FetchResponse(p) = out.kind {
                assert_eq!(code, p.status_code);
            } else {
                panic!("expected FetchResponse for code {code}");
            }
        }
    }

    #[test]
    fn test_fetch_request_empty_method() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://example.com".into(),
                method: String::new(),
                headers: vec![],
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert!(p.method.is_empty());
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_navigate_empty_url() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: String::new(),
                referrer: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Navigate(p) = out.kind {
            assert!(p.url.is_empty());
            assert!(p.referrer.is_none());
        } else {
            panic!("expected Navigate");
        }
    }

    #[test]
    fn test_mouse_event_button_max() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: u8::MAX,
                event_type: MouseEventType::Down,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::MouseEvent(p) = out.kind {
            assert_eq!(u8::MAX, p.button);
        } else {
            panic!("expected MouseEvent");
        }
    }

    #[test]
    fn test_scroll_event_zero_deltas() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 0.0,
                delta_y: 0.0,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::ScrollEvent(p) = out.kind {
            assert_eq!(0.0, p.delta_x);
            assert_eq!(0.0, p.delta_y);
        } else {
            panic!("expected ScrollEvent");
        }
    }

    #[test]
    fn test_keyboard_event_all_modifiers_on() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "x".into(),
                code: "KeyX".into(),
                ctrl: true,
                shift: true,
                alt: true,
                meta: true,
                event_type: KeyboardEventType::Press,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert!(p.ctrl && p.shift && p.alt && p.meta);
        } else {
            panic!("expected KeyboardEvent");
        }
    }

    #[test]
    fn test_keyboard_event_all_modifiers_off() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "z".into(),
                code: "KeyZ".into(),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::KeyboardEvent(p) = out.kind {
            assert!(!p.ctrl && !p.shift && !p.alt && !p.meta);
        } else {
            panic!("expected KeyboardEvent");
        }
    }

    #[test]
    fn test_fetch_multiple_headers() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://example.com".into(),
                method: "GET".into(),
                headers: vec![
                    ("Accept".into(), "text/html".into()),
                    ("Accept-Language".into(), "en".into()),
                    ("Accept-Encoding".into(), "gzip".into()),
                    ("Cookie".into(), "a=1".into()),
                    ("Cookie".into(), "b=2".into()),
                ],
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(5, p.headers.len());
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_deserialize_truncated_valid_header() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        };
        let bytes = serialize(&msg).expect("serialize");
        // 截取前几个字节，反序列化应失败
        let truncated = &bytes[..bytes.len() / 2];
        assert!(deserialize(truncated).is_err());
    }

    #[test]
    fn test_serialize_idempotent() {
        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::Heartbeat,
        };
        let bytes1 = serialize(&msg).expect("serialize 1");
        let bytes2 = serialize(&msg).expect("serialize 2");
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_fetch_response_empty_body() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 204,
                headers: vec![],
                body: vec![],
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert!(p.body.is_empty());
        } else {
            panic!("expected FetchResponse");
        }
    }

    // ══════════════════════════════════════════════════════════
    //  新增测试：提升覆盖率至 80+
    // ══════════════════════════════════════════════════════════

    // ── 1. 消息序列化往返 ──

    #[test]
    fn test_roundtrip_navigate_with_referrer_some() {
        let msg = IpcMessage {
            id: 100,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: Some("https://referrer.com".into()),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Navigate(p) = out.kind {
            assert_eq!("https://example.com", p.url);
            assert_eq!(Some("https://referrer.com".into()), p.referrer);
        } else {
            panic!("expected Navigate");
        }
    }

    #[test]
    fn test_roundtrip_navigate_with_referrer_none() {
        let msg = IpcMessage {
            id: 101,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Navigate(p) = out.kind {
            assert_eq!("https://example.com", p.url);
            assert!(p.referrer.is_none());
        } else {
            panic!("expected Navigate");
        }
    }

    #[test]
    fn test_roundtrip_crash_notification() {
        let msg = IpcMessage {
            id: 102,
            kind: IpcMessageKind::CrashNotification("segfault at 0xdeadbeef".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::CrashNotification(reason) = out.kind {
            assert_eq!("segfault at 0xdeadbeef", reason);
        } else {
            panic!("expected CrashNotification");
        }
    }

    #[test]
    fn test_roundtrip_error_response() {
        let msg = IpcMessage {
            id: 103,
            kind: IpcMessageKind::Error("network unreachable".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::Error(e) = out.kind {
            assert_eq!("network unreachable", e);
        } else {
            panic!("expected Error");
        }
    }

    #[test]
    fn test_roundtrip_nested_fetch_response_with_headers_and_body() {
        let msg = IpcMessage {
            id: 104,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 999,
                status_code: 301,
                headers: vec![
                    ("Location".into(), "https://example.com/new".into()),
                    ("Content-Length".into(), "0".into()),
                ],
                body: b"redirecting...".to_vec(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(999, p.request_id);
            assert_eq!(301, p.status_code);
            assert_eq!(2, p.headers.len());
            assert_eq!(b"redirecting...", p.body.as_slice());
        } else {
            panic!("expected FetchResponse");
        }
    }

    // ── 2. 二进制序列化边界情况 ──

    #[test]
    fn test_fetch_response_large_binary_body() {
        let body: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let msg = IpcMessage {
            id: 205,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 200,
                headers: vec![("Content-Length".into(), "4096".into())],
                body: body.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(4096, p.body.len());
            assert_eq!(body, p.body);
        } else {
            panic!("expected FetchResponse");
        }
    }

    #[test]
    fn test_large_payload_10kb() {
        let large_body: Vec<u8> = (0..10_240).map(|i| (i % 256) as u8).collect();
        let msg = IpcMessage {
            id: 200,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 200,
                headers: vec![],
                body: large_body.clone(),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(10_240, p.body.len());
            assert_eq!(large_body, p.body);
        } else {
            panic!("expected FetchResponse");
        }
    }

    #[test]
    fn test_binary_data_with_zeros() {
        let zero_body = vec![0u8; 256];
        let msg = IpcMessage {
            id: 201,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://example.com".into(),
                method: "POST".into(),
                headers: vec![],
                body: Some(zero_body.clone()),
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(Some(zero_body), p.body);
        } else {
            panic!("expected FetchRequest");
        }
    }

    #[test]
    fn test_unicode_strings_in_messages() {
        let msg = IpcMessage {
            id: 202,
            kind: IpcMessageKind::TitleChanged("こんにちは世界 🌍 Ñoño café".into()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::TitleChanged(t) = out.kind {
            assert_eq!("こんにちは世界 🌍 Ñoño café", t);
        } else {
            panic!("expected TitleChanged");
        }
    }

    #[test]
    fn test_empty_string_in_url_changed() {
        let msg = IpcMessage {
            id: 203,
            kind: IpcMessageKind::UrlChanged(String::new()),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::UrlChanged(url) = out.kind {
            assert!(url.is_empty());
        } else {
            panic!("expected UrlChanged");
        }
    }

    #[test]
    fn test_nested_vec_header_structures() {
        let many_headers: Vec<(String, String)> = (0..50)
            .map(|i| (format!("X-Custom-{i}"), format!("value-{i}")))
            .collect();
        let msg = IpcMessage {
            id: 204,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 42,
                url: "https://example.com".into(),
                method: "GET".into(),
                headers: many_headers.clone(),
                body: None,
            }),
        };
        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(50, p.headers.len());
            assert_eq!(many_headers, p.headers);
        } else {
            panic!("expected FetchRequest");
        }
    }

    // ── 3. 消息类型覆盖 ──

    #[test]
    fn test_navigation_messages_sequence() {
        let msgs = vec![
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://a.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::GoBack,
            },
            IpcMessage {
                id: 3,
                kind: IpcMessageKind::GoForward,
            },
            IpcMessage {
                id: 4,
                kind: IpcMessageKind::Reload,
            },
            IpcMessage {
                id: 5,
                kind: IpcMessageKind::StopLoading,
            },
        ];
        for msg in msgs {
            let out = roundtrip(msg.clone());
            assert_eq!(msg.id, out.id);
        }
    }

    #[test]
    fn test_dom_related_messages() {
        let title_msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::TitleChanged("My Page".into()),
        };
        let url_msg = IpcMessage {
            id: 2,
            kind: IpcMessageKind::UrlChanged("https://example.com/page2".into()),
        };
        let load_msg = IpcMessage {
            id: 3,
            kind: IpcMessageKind::LoadComplete,
        };
        let failed_msg = IpcMessage {
            id: 4,
            kind: IpcMessageKind::LoadFailed("DNS error".into()),
        };

        let out = roundtrip(title_msg);
        if let IpcMessageKind::TitleChanged(t) = out.kind {
            assert_eq!("My Page", t);
        } else {
            panic!("expected TitleChanged");
        }

        let out = roundtrip(url_msg);
        if let IpcMessageKind::UrlChanged(u) = out.kind {
            assert_eq!("https://example.com/page2", u);
        } else {
            panic!("expected UrlChanged");
        }

        assert!(matches!(roundtrip(load_msg).kind, IpcMessageKind::LoadComplete));
        if let IpcMessageKind::LoadFailed(r) = roundtrip(failed_msg).kind {
            assert_eq!("DNS error", r);
        } else {
            panic!("expected LoadFailed");
        }
    }

    #[test]
    fn test_resource_loading_messages() {
        let fetch_req = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 10,
                url: "https://cdn.example.com/app.js".into(),
                method: "GET".into(),
                headers: vec![("Accept".into(), "*/*".into())],
                body: None,
            }),
        };
        let fetch_resp = IpcMessage {
            id: 2,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 10,
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: b"console.log('hi')".to_vec(),
            }),
        };

        let out_req = roundtrip(fetch_req);
        if let IpcMessageKind::FetchRequest(p) = out_req.kind {
            assert_eq!(10, p.request_id);
            assert_eq!("https://cdn.example.com/app.js", p.url);
        } else {
            panic!("expected FetchRequest");
        }

        let out_resp = roundtrip(fetch_resp);
        if let IpcMessageKind::FetchResponse(p) = out_resp.kind {
            assert_eq!(10, p.request_id);
            assert_eq!(b"console.log('hi')", p.body.as_slice());
        } else {
            panic!("expected FetchResponse");
        }
    }

    #[test]
    fn test_script_execution_via_keyboard_events() {
        let key_events = vec![
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key: "F5".into(),
                    code: "F5".into(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    meta: false,
                    event_type: KeyboardEventType::Down,
                }),
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key: "F5".into(),
                    code: "F5".into(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    meta: false,
                    event_type: KeyboardEventType::Up,
                }),
            },
        ];
        for msg in key_events {
            let out = roundtrip(msg);
            if let IpcMessageKind::KeyboardEvent(p) = out.kind {
                assert_eq!("F5", p.key);
            } else {
                panic!("expected KeyboardEvent");
            }
        }
    }

    #[test]
    fn test_layout_render_scroll_and_mouse() {
        let scroll = IpcMessage {
            id: 1,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 100.0,
                delta_y: -50.0,
            }),
        };
        let mouse = IpcMessage {
            id: 2,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 400.0,
                y: 300.0,
                button: 0,
                event_type: MouseEventType::Move,
            }),
        };

        let out_scroll = roundtrip(scroll);
        if let IpcMessageKind::ScrollEvent(p) = out_scroll.kind {
            assert_eq!(100.0, p.delta_x);
            assert_eq!(-50.0, p.delta_y);
        } else {
            panic!("expected ScrollEvent");
        }

        let out_mouse = roundtrip(mouse);
        if let IpcMessageKind::MouseEvent(p) = out_mouse.kind {
            assert_eq!(400.0, p.x);
            assert_eq!(300.0, p.y);
            assert_eq!(MouseEventType::Move, p.event_type);
        } else {
            panic!("expected MouseEvent");
        }
    }

    // ── 4. 错误处理 ──

    #[test]
    fn test_deserialize_invalid_bytes_returns_error() {
        let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA];
        let result = deserialize(&garbage);
        assert!(result.is_err());
        if let Err(ProtocolError::Deserialization(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("expected Deserialization error");
        }
    }

    #[test]
    fn test_deserialize_empty_bytes_returns_error() {
        let result: Result<IpcMessage, ProtocolError> = deserialize(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_truncated_mid_field() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1,
                url: "https://example.com/very/long/path".into(),
                method: "POST".into(),
                headers: vec![("Content-Type".into(), "text/plain".into())],
                body: Some(b"some body data here".to_vec()),
            }),
        };
        let bytes = serialize(&msg).expect("serialize");
        // Truncate to 1/3 of original length
        let truncated = &bytes[..bytes.len() / 3];
        assert!(deserialize(truncated).is_err());
    }

    #[test]
    fn test_deserialize_single_byte_returns_error() {
        let result = deserialize(&[0x00]);
        assert!(result.is_err());
    }

    // ── 5. 并发消息处理 ──

    #[test]
    fn test_multiple_messages_serialized_in_sequence() {
        let messages: Vec<IpcMessage> = (0..20)
            .map(|i| IpcMessage {
                id: i,
                kind: IpcMessageKind::Heartbeat,
            })
            .collect();

        let serialized: Vec<Vec<u8>> = messages.iter().map(|m| serialize(m).expect("s")).collect();
        let deserialized: Vec<IpcMessage> = serialized.iter().map(|b| deserialize(b).expect("d")).collect();

        for (i, msg) in deserialized.iter().enumerate() {
            assert_eq!(i as u64, msg.id);
            assert!(matches!(msg.kind, IpcMessageKind::Heartbeat));
        }
    }

    #[test]
    fn test_message_ordering_preserved() {
        let msgs = vec![
            IpcMessage {
                id: 0,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://a.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::LoadComplete,
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::TitleChanged("A".into()),
            },
            IpcMessage {
                id: 3,
                kind: IpcMessageKind::MouseEvent(MouseEventParams {
                    x: 1.0,
                    y: 2.0,
                    button: 0,
                    event_type: MouseEventType::Click,
                }),
            },
            IpcMessage {
                id: 4,
                kind: IpcMessageKind::GoBack,
            },
        ];

        let pairs: Vec<(Vec<u8>, IpcMessage)> = msgs
            .into_iter()
            .map(|m| {
                let bytes = serialize(&m).expect("s");
                (bytes, m)
            })
            .collect();

        for (bytes, original) in pairs {
            let out = deserialize(&bytes).expect("d");
            assert_eq!(original.id, out.id);
            // Verify correct kind by re-serializing and comparing bytes
            let re_bytes = serialize(&out).expect("re-s");
            assert_eq!(bytes, re_bytes);
        }
    }

    // ══════════════════════════════════════════════════════════
    //  新增测试：IpcChannel 契约、序列化确定性、对抗性反序列化
    // ══════════════════════════════════════════════════════════

    /// 基于 Vec 的内存 mock IpcChannel，用于验证 trait 契约。
    struct MockChannel {
        queue: Vec<IpcMessage>,
        closed: bool,
    }

    impl MockChannel {
        fn new() -> Self {
            Self {
                queue: Vec::new(),
                closed: false,
            }
        }
    }

    impl crate::IpcChannel for MockChannel {
        fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError> {
            if self.closed {
                return Err(ProtocolError::Channel("channel closed".into()));
            }
            self.queue.push(msg);
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            if self.closed {
                return Err(ProtocolError::Channel("channel closed".into()));
            }
            if self.queue.is_empty() {
                return Err(ProtocolError::Channel("empty".into()));
            }
            Ok(self.queue.remove(0))
        }

        fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
            if self.closed {
                return Err(ProtocolError::Channel("channel closed".into()));
            }
            if self.queue.is_empty() {
                return Ok(None);
            }
            Ok(Some(self.queue.remove(0)))
        }

        fn close(&mut self) {
            self.closed = true;
        }
    }

    #[test]
    fn test_ipc_channel_mock_send_recv() {
        let mut ch = MockChannel::new();

        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: Some("https://ref.com".into()),
            }),
        };

        // send → recv：消息保持不变
        ch.send(msg.clone()).expect("send");
        let out = ch.recv().expect("recv");
        assert_eq!(msg.id, out.id);
        if let IpcMessageKind::Navigate(p) = out.kind {
            assert_eq!("https://example.com", p.url);
            assert_eq!(Some("https://ref.com".into()), p.referrer);
        } else {
            panic!("expected Navigate");
        }

        // 空通道 recv 应返回错误
        assert!(ch.recv().is_err());
    }

    #[test]
    fn test_ipc_channel_mock_try_recv() {
        let mut ch = MockChannel::new();

        // 空通道 try_recv 返回 Ok(None)
        assert!(ch.try_recv().expect("try_recv empty").is_none());

        ch.send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Heartbeat,
        })
        .expect("send");

        // 有消息时 try_recv 返回 Ok(Some(..))
        let opt = ch.try_recv().expect("try_recv");
        assert!(opt.is_some());
        assert_eq!(1, opt.unwrap().id);

        // 再取一次应为空
        assert!(ch.try_recv().expect("try_recv again").is_none());
    }

    #[test]
    fn test_ipc_channel_mock_close() {
        let mut ch = MockChannel::new();

        ch.send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        })
        .expect("send before close");

        ch.close();

        // 关闭后 send 应失败
        let res = ch.send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::Ok,
        });
        assert!(res.is_err());

        // 关闭后 recv 也应失败（即使队列有消息）
        assert!(ch.recv().is_err());
        assert!(ch.try_recv().is_err());
    }

    #[test]
    fn test_ipc_channel_mock_fifo_order() {
        let mut ch = MockChannel::new();

        for i in 0u64..5 {
            ch.send(IpcMessage {
                id: i,
                kind: IpcMessageKind::Heartbeat,
            })
            .expect("send");
        }

        for i in 0u64..5 {
            let msg = ch.recv().expect("recv");
            assert_eq!(i, msg.id, "FIFO order violated");
        }
    }

    #[test]
    fn test_serialization_deterministic() {
        let msgs = vec![
            IpcMessage {
                id: 100,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 200,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: 1,
                    url: "https://api.com".into(),
                    method: "POST".into(),
                    headers: vec![("X-Foo".into(), "bar".into())],
                    body: Some(vec![1, 2, 3]),
                }),
            },
            IpcMessage {
                id: 300,
                kind: IpcMessageKind::Heartbeat,
            },
        ];

        for msg in &msgs {
            let b1 = serialize(msg).expect("serialize 1");
            let b2 = serialize(msg).expect("serialize 2");
            assert_eq!(b1, b2, "deterministic encoding violated for message id={}", msg.id);
        }
    }

    #[test]
    fn test_deserialization_trailing_bytes() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        };
        let mut bytes = serialize(&msg).expect("serialize");

        // 在有效载荷末尾追加额外字节
        bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        // bincode 1.x 默认配置不拒绝尾部多余数据，会成功解析并忽略尾部。
        // 记录此行为：反序列化成功，消息内容不变。
        let result = deserialize(&bytes);
        assert!(
            result.is_ok(),
            "bincode 1.x default config accepts trailing bytes — document this behavior"
        );
        let out = result.expect("deserialize");
        assert_eq!(1, out.id);
        assert!(matches!(out.kind, IpcMessageKind::Ok));
    }

    #[test]
    fn test_deserialization_random_bytes() {
        // 各种长度的随机/对抗性字节，反序列化均应返回错误
        let cases: Vec<Vec<u8>> = vec![
            vec![],
            vec![0x00],
            vec![0xFF],
            vec![0x01, 0x02, 0x03],
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            vec![0xFF; 8],
            vec![0xFF; 16],
            vec![0xFF; 32],
            vec![0xFF; 64],
            vec![0xFF; 256],
            // 看起来像合法 bincode 头但内容不完整
            vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ];

        for (i, bytes) in cases.iter().enumerate() {
            let result = deserialize(bytes);
            assert!(
                result.is_err(),
                "random bytes case {i} (len={}) should fail to deserialize",
                bytes.len()
            );
        }
    }

    #[test]
    fn test_different_message_types_interleaved() {
        let msgs: Vec<IpcMessage> = vec![
            IpcMessage {
                id: 0,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::Heartbeat,
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: 1,
                    url: "https://api.com".into(),
                    method: "GET".into(),
                    headers: vec![],
                    body: None,
                }),
            },
            IpcMessage {
                id: 3,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: StorageType::Local,
                    operation: StorageOperation::Set,
                    key: "k".into(),
                    value: Some("v".into()),
                    origin: "https://example.com".into(),
                }),
            },
            IpcMessage {
                id: 4,
                kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                    key: "a".into(),
                    code: "KeyA".into(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    meta: false,
                    event_type: KeyboardEventType::Down,
                }),
            },
            IpcMessage {
                id: 5,
                kind: IpcMessageKind::Ok,
            },
        ];

        // Serialize all, then deserialize all — interleaved types must not corrupt each other
        let encoded: Vec<Vec<u8>> = msgs.iter().map(|m| serialize(m).expect("s")).collect();
        for (i, bytes) in encoded.iter().enumerate() {
            let out = deserialize(bytes).expect("d");
            assert_eq!(i as u64, out.id);
        }
    }

    // ══════════════════════════════════════════════════════════
    //  IPC 消息边界条件测试
    // ══════════════════════════════════════════════════════════

    /// 测试大载荷（100KB 字符串）消息的序列化/反序列化往返正确性。
    #[test]
    fn test_ipc_message_large_payload() {
        let large_string: String = "A".repeat(100 * 1024); // 100KB
        let msg = IpcMessage {
            id: 999,
            kind: IpcMessageKind::TitleChanged(large_string.clone()),
        };
        let out = roundtrip(msg);
        assert_eq!(999, out.id);
        if let IpcMessageKind::TitleChanged(t) = out.kind {
            assert_eq!(100 * 1024, t.len());
            assert_eq!(large_string, t);
        } else {
            panic!("expected TitleChanged");
        }
    }

    /// 测试空字符串和零值字段的消息序列化/反序列化。
    #[test]
    fn test_ipc_message_empty_fields() {
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Clear,
                key: String::new(),
                value: Some(String::new()),
                origin: String::new(),
            }),
        };
        let out = roundtrip(msg);
        assert_eq!(0, out.id);
        if let IpcMessageKind::StorageOp(p) = out.kind {
            assert!(p.key.is_empty());
            assert_eq!(Some(String::new()), p.value);
            assert!(p.origin.is_empty());
            assert_eq!(StorageOperation::Clear, p.operation);
        } else {
            panic!("expected StorageOp");
        }
    }

    /// 测试连续序列化多条不同消息后逐个反序列化，验证每条消息还原正确。
    #[test]
    fn test_ipc_message_concurrent_serialization() {
        let messages: Vec<IpcMessage> = vec![
            IpcMessage {
                id: 10,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 20,
                kind: IpcMessageKind::TitleChanged("测试标题".into()),
            },
            IpcMessage {
                id: 30,
                kind: IpcMessageKind::MouseEvent(MouseEventParams {
                    x: 42.0,
                    y: 99.5,
                    button: 2,
                    event_type: MouseEventType::DblClick,
                }),
            },
            IpcMessage {
                id: 40,
                kind: IpcMessageKind::Error("未知错误".into()),
            },
            IpcMessage {
                id: 50,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: 7,
                    url: "https://api.example.com/data".into(),
                    method: "POST".into(),
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: Some(b"{\"key\":\"value\"}".to_vec()),
                }),
            },
        ];

        // 连续序列化所有消息
        let serialized: Vec<Vec<u8>> = messages.iter().map(|m| serialize(m).expect("serialize")).collect();

        // 逐个反序列化并验证
        for (i, bytes) in serialized.iter().enumerate() {
            let out = deserialize(bytes).expect("deserialize");
            assert_eq!(messages[i].id, out.id, "消息 id 不匹配：索引 {i}");

            // 对每条消息按类型验证字段
            match (&messages[i].kind, &out.kind) {
                (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                    assert_eq!(a.url, b.url);
                    assert_eq!(a.referrer, b.referrer);
                }
                (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::MouseEvent(a), IpcMessageKind::MouseEvent(b)) => {
                    assert_eq!(a.x, b.x);
                    assert_eq!(a.y, b.y);
                    assert_eq!(a.button, b.button);
                    assert_eq!(a.event_type, b.event_type);
                }
                (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::FetchRequest(a), IpcMessageKind::FetchRequest(b)) => {
                    assert_eq!(a.request_id, b.request_id);
                    assert_eq!(a.url, b.url);
                    assert_eq!(a.method, b.method);
                    assert_eq!(a.headers, b.headers);
                    assert_eq!(a.body, b.body);
                }
                _ => panic!("消息类型不匹配：索引 {i}"),
            }
        }
    }

    /// 测试通过 mock 通道发送多条消息后按 FIFO 顺序接收。
    #[test]
    fn test_ipc_channel_ordering() {
        let mut ch = MockChannel::new();

        let messages: Vec<IpcMessage> = vec![
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://a.com".into(),
                    referrer: None,
                }),
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::LoadComplete,
            },
            IpcMessage {
                id: 3,
                kind: IpcMessageKind::TitleChanged("页面标题".into()),
            },
            IpcMessage {
                id: 4,
                kind: IpcMessageKind::Heartbeat,
            },
            IpcMessage {
                id: 5,
                kind: IpcMessageKind::Error("超时".into()),
            },
        ];

        // 按顺序发送所有消息
        for msg in &messages {
            ch.send(msg.clone()).expect("send");
        }

        // 按 FIFO 顺序接收并验证
        for (i, expected) in messages.iter().enumerate() {
            let received = ch.recv().expect("recv");
            assert_eq!(
                expected.id, received.id,
                "FIFO 顺序违反：期望 id={}，实际 id={}（索引 {}）",
                expected.id, received.id, i
            );

            // 验证消息类型匹配
            match (&expected.kind, &received.kind) {
                (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                    assert_eq!(a.url, b.url);
                }
                (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::LoadComplete, IpcMessageKind::LoadComplete)
                | (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat) => {}
                _ => panic!("消息类型不匹配：索引 {i}"),
            }
        }

        // 所有消息已消费，再次 recv 应返回错误
        assert!(ch.recv().is_err());
    }

    // ══════════════════════════════════════════════════════════
    //  IPC 压力测试与剩余边界条件测试
    // ══════════════════════════════════════════════════════════

    /// 测试超大消息（1MB+ 载荷）的序列化/反序列化往返正确性。
    /// 验证 bincode 在大载荷下不会截断或损坏数据。
    #[test]
    fn test_large_message_serialization() {
        // 构造一个 1MB+ 的二进制 body（恰好 1MB + 1 字节以确保超过 1MB）
        let large_body: Vec<u8> = (0..1_048_577).map(|i| (i % 256) as u8).collect();
        assert!(large_body.len() > 1_048_576, "载荷必须超过 1MB");

        let msg = IpcMessage {
            id: u64::MAX,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: u64::MAX,
                status_code: 200,
                headers: vec![("Content-Length".into(), format!("{}", large_body.len()))],
                body: large_body.clone(),
            }),
        };

        let bytes = serialize(&msg).expect("序列化 1MB+ 消息应成功");
        // 序列化后的字节长度应明显大于原始 body
        assert!(
            bytes.len() > large_body.len(),
            "序列化结果应包含完整载荷，实际 {} 字节",
            bytes.len()
        );

        let out = deserialize(&bytes).expect("反序列化 1MB+ 消息应成功");
        assert_eq!(u64::MAX, out.id);
        if let IpcMessageKind::FetchResponse(p) = out.kind {
            assert_eq!(u64::MAX, p.request_id);
            assert_eq!(200, p.status_code);
            assert_eq!(large_body.len(), p.body.len());
            assert_eq!(large_body, p.body, "大载荷往返后内容应完全一致");
        } else {
            panic!("期望 FetchResponse");
        }
    }

    /// 测试多条不同类型消息的序列化/反序列化顺序保持不变。
    /// 模拟高并发场景：连续序列化 100 条消息，反序列化后 ID 和类型顺序必须一致。
    #[test]
    fn test_concurrent_message_ordering() {
        let messages: Vec<IpcMessage> = (0..100)
            .map(|i| {
                let kind = match i % 5 {
                    0 => IpcMessageKind::Navigate(NavigateParams {
                        url: format!("https://example.com/page/{i}"),
                        referrer: if i % 2 == 0 {
                            Some("https://referrer.com".into())
                        } else {
                            None
                        },
                    }),
                    1 => IpcMessageKind::FetchRequest(FetchParams {
                        request_id: i as u64,
                        url: format!("https://api.example.com/{i}"),
                        method: "GET".into(),
                        headers: vec![],
                        body: None,
                    }),
                    2 => IpcMessageKind::MouseEvent(MouseEventParams {
                        x: i as f32 * 1.5,
                        y: i as f32 * 2.5,
                        button: (i % 3) as u8,
                        event_type: MouseEventType::Click,
                    }),
                    3 => IpcMessageKind::TitleChanged(format!("页面标题 #{i}")),
                    _ => IpcMessageKind::Heartbeat,
                };
                IpcMessage { id: i as u64, kind }
            })
            .collect();

        // 连续序列化所有消息
        let serialized: Vec<Vec<u8>> = messages.iter().map(|m| serialize(m).expect("序列化应成功")).collect();

        assert_eq!(messages.len(), serialized.len());

        // 逐个反序列化，验证顺序和内容
        for (i, bytes) in serialized.iter().enumerate() {
            let out = deserialize(bytes).expect("反序列化应成功");
            assert_eq!(i as u64, out.id, "消息顺序不一致：期望 id={}，实际 id={}", i, out.id);

            // 按消息类型验证关键字段
            match (&messages[i].kind, &out.kind) {
                (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                    assert_eq!(a.url, b.url, "Navigate url 不匹配，索引 {i}");
                    assert_eq!(a.referrer, b.referrer, "Navigate referrer 不匹配，索引 {i}");
                }
                (IpcMessageKind::FetchRequest(a), IpcMessageKind::FetchRequest(b)) => {
                    assert_eq!(a.request_id, b.request_id, "FetchRequest request_id 不匹配，索引 {i}");
                    assert_eq!(a.url, b.url, "FetchRequest url 不匹配，索引 {i}");
                }
                (IpcMessageKind::MouseEvent(a), IpcMessageKind::MouseEvent(b)) => {
                    assert_eq!(a.x, b.x, "MouseEvent x 不匹配，索引 {i}");
                    assert_eq!(a.y, b.y, "MouseEvent y 不匹配，索引 {i}");
                    assert_eq!(a.button, b.button, "MouseEvent button 不匹配，索引 {i}");
                }
                (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                    assert_eq!(a, b, "TitleChanged 不匹配，索引 {i}");
                }
                (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat) => {}
                _ => panic!("消息类型不匹配：索引 {i}"),
            }
        }
    }

    /// 测试所有可选字段均为空值时的消息序列化/反序列化。
    /// 覆盖 None、空字符串、空 Vec 等边界情况。
    #[test]
    fn test_message_with_empty_fields() {
        // Navigate：url 为空字符串，referrer 为 None
        let msg1 = IpcMessage {
            id: 0,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: String::new(),
                referrer: None,
            }),
        };
        let out1 = roundtrip(msg1);
        if let IpcMessageKind::Navigate(p) = out1.kind {
            assert!(p.url.is_empty(), "url 应为空字符串");
            assert!(p.referrer.is_none(), "referrer 应为 None");
        } else {
            panic!("期望 Navigate");
        }

        // FetchRequest：空 headers，body 为 None
        let msg2 = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 0,
                url: String::new(),
                method: String::new(),
                headers: vec![],
                body: None,
            }),
        };
        let out2 = roundtrip(msg2);
        if let IpcMessageKind::FetchRequest(p) = out2.kind {
            assert_eq!(0, p.request_id);
            assert!(p.url.is_empty());
            assert!(p.method.is_empty());
            assert!(p.headers.is_empty());
            assert!(p.body.is_none());
        } else {
            panic!("期望 FetchRequest");
        }

        // FetchResponse：空 headers，空 body
        let msg3 = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 0,
                status_code: 0,
                headers: vec![],
                body: vec![],
            }),
        };
        let out3 = roundtrip(msg3);
        if let IpcMessageKind::FetchResponse(p) = out3.kind {
            assert_eq!(0, p.status_code);
            assert!(p.headers.is_empty());
            assert!(p.body.is_empty());
        } else {
            panic!("期望 FetchResponse");
        }

        // StorageOp：value 为 None，key 和 origin 为空字符串
        let msg4 = IpcMessage {
            id: 0,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Get,
                key: String::new(),
                value: None,
                origin: String::new(),
            }),
        };
        let out4 = roundtrip(msg4);
        if let IpcMessageKind::StorageOp(p) = out4.kind {
            assert!(p.key.is_empty());
            assert!(p.value.is_none());
            assert!(p.origin.is_empty());
        } else {
            panic!("期望 StorageOp");
        }

        // MouseEvent 和 KeyboardEvent：零值坐标/按键
        let msg5 = IpcMessage {
            id: 0,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 0.0,
                y: 0.0,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        };
        let out5 = roundtrip(msg5);
        if let IpcMessageKind::MouseEvent(p) = out5.kind {
            assert_eq!(0.0, p.x);
            assert_eq!(0.0, p.y);
            assert_eq!(0, p.button);
        } else {
            panic!("期望 MouseEvent");
        }
    }

    /// 测试包含 Unicode 字符串的消息序列化/反序列化往返正确性。
    /// 覆盖中日韩文字、emoji、组合字符、特殊 Unicode 等。
    #[test]
    fn test_message_with_unicode_payload() {
        let long_ascii = "x".repeat(1000);
        let unicode_cases: Vec<&str> = vec![
            // 中文
            "这是一个中文标题，包含标点符号：【】《》、。，！？",
            // 日文
            "こんにちは世界 🌍 日本語テスト",
            // 韩文
            "안녕하세요 세계",
            // emoji 丰富文本
            "🎉🚀💻🔒 ✓ — Unicode 测试 © 2024",
            // 混合 RTL/LTR
            "Hello مرحبا العالم שלום",
            // 组合字符（é 可以是 e + ́ ）
            "caf\u{0065}\u{0301} = café",
            // 零宽字符
            "abc\u{200B}\u{200C}\u{200D}def",
            // 四字节 emoji
            "👨‍👩‍👧‍👦 family emoji",
            // 空字符串
            "",
            // 纯 ASCII 但很长的字符串
            &long_ascii,
        ];

        for (i, text) in unicode_cases.iter().enumerate() {
            // TitleChanged
            let msg = IpcMessage {
                id: i as u64,
                kind: IpcMessageKind::TitleChanged(text.to_string()),
            };
            let out = roundtrip(msg);
            if let IpcMessageKind::TitleChanged(t) = out.kind {
                assert_eq!(text, &t, "TitleChanged Unicode 往返失败：索引 {i}");
            } else {
                panic!("期望 TitleChanged，索引 {i}");
            }

            // UrlChanged
            let msg2 = IpcMessage {
                id: i as u64 + 100,
                kind: IpcMessageKind::UrlChanged(text.to_string()),
            };
            let out2 = roundtrip(msg2);
            if let IpcMessageKind::UrlChanged(u) = out2.kind {
                assert_eq!(text, &u, "UrlChanged Unicode 往返失败：索引 {i}");
            } else {
                panic!("期望 UrlChanged，索引 {i}");
            }

            // CrashNotification
            let msg3 = IpcMessage {
                id: i as u64 + 200,
                kind: IpcMessageKind::CrashNotification(text.to_string()),
            };
            let out3 = roundtrip(msg3);
            if let IpcMessageKind::CrashNotification(r) = out3.kind {
                assert_eq!(text, &r, "CrashNotification Unicode 往返失败：索引 {i}");
            } else {
                panic!("期望 CrashNotification，索引 {i}");
            }

            // Error
            let msg4 = IpcMessage {
                id: i as u64 + 300,
                kind: IpcMessageKind::Error(text.to_string()),
            };
            let out4 = roundtrip(msg4);
            if let IpcMessageKind::Error(e) = out4.kind {
                assert_eq!(text, &e, "Error Unicode 往返失败：索引 {i}");
            } else {
                panic!("期望 Error，索引 {i}");
            }

            // Navigate url 字段
            let msg5 = IpcMessage {
                id: i as u64 + 400,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: text.to_string(),
                    referrer: Some(text.to_string()),
                }),
            };
            let out5 = roundtrip(msg5);
            if let IpcMessageKind::Navigate(p) = out5.kind {
                assert_eq!(text, &p.url, "Navigate url Unicode 往返失败：索引 {i}");
                assert_eq!(
                    &Some(text.to_string()),
                    &p.referrer,
                    "Navigate referrer Unicode 往返失败：索引 {i}"
                );
            } else {
                panic!("期望 Navigate，索引 {i}");
            }

            // FetchParams headers 中的 Unicode 键值对
            let msg6 = IpcMessage {
                id: i as u64 + 500,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: i as u64,
                    url: text.to_string(),
                    method: text.to_string(),
                    headers: vec![
                        (text.to_string(), text.to_string()),
                        ("ASCII-Key".into(), text.to_string()),
                    ],
                    body: Some(text.as_bytes().to_vec()),
                }),
            };
            let out6 = roundtrip(msg6);
            if let IpcMessageKind::FetchRequest(p) = out6.kind {
                assert_eq!(text, &p.url, "FetchParams url Unicode 往返失败：索引 {i}");
                assert_eq!(text, &p.method, "FetchParams method Unicode 往返失败：索引 {i}");
                assert_eq!(2, p.headers.len(), "FetchParams headers 长度不匹配：索引 {i}");
                assert_eq!(
                    text, &p.headers[0].0,
                    "FetchParams header key Unicode 往返失败：索引 {i}"
                );
                assert_eq!(
                    text, &p.headers[0].1,
                    "FetchParams header value Unicode 往返失败：索引 {i}"
                );
                assert_eq!(
                    Some(text.as_bytes().to_vec()),
                    p.body,
                    "FetchParams body Unicode 往返失败：索引 {i}"
                );
            } else {
                panic!("期望 FetchRequest，索引 {i}");
            }

            // StorageOp key/value/origin 全部 Unicode
            let msg7 = IpcMessage {
                id: i as u64 + 600,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: StorageType::Session,
                    operation: StorageOperation::Set,
                    key: text.to_string(),
                    value: Some(text.to_string()),
                    origin: text.to_string(),
                }),
            };
            let out7 = roundtrip(msg7);
            if let IpcMessageKind::StorageOp(p) = out7.kind {
                assert_eq!(text, &p.key, "StorageOp key Unicode 往返失败：索引 {i}");
                assert_eq!(
                    &Some(text.to_string()),
                    &p.value,
                    "StorageOp value Unicode 往返失败：索引 {i}"
                );
                assert_eq!(text, &p.origin, "StorageOp origin Unicode 往返失败：索引 {i}");
            } else {
                panic!("期望 StorageOp，索引 {i}");
            }
        }
    }

    /// 测试包含多层嵌套结构的消息序列化/反序列化往返。
    /// IpcMessage 本身包含嵌套的 params 结构体（含 Vec<(String, String)> 等），
    /// 验证这些嵌套层级在 bincode 编码/解码后完整还原。
    #[test]
    fn test_nested_message_round_trip() {
        // 构造具有多层嵌套的消息：
        // IpcMessage -> FetchRequest -> headers: Vec<(String, String)> (多层嵌套)
        //                                  body: Option<Vec<u8>>
        let nested_headers: Vec<(String, String)> = (0..20)
            .map(|i| {
                let nested_value = format!(
                    "level0={i}; level1={{nested_{i}: {{deep: true}}}}; arr=[{},{},{}]",
                    i,
                    i + 1,
                    i + 2
                );
                (format!("X-Nested-Key-{i}"), nested_value)
            })
            .collect();

        // 嵌套的二进制 body，内部包含一个模拟的 JSON 结构
        let nested_body: Vec<u8> = format!(
            "{{\"outer\":{{\"inner\":[{}]}}}}",
            (0..10)
                .map(|i| format!("{{\"id\":{i},\"value\":\"nested_{i}\"}}"))
                .collect::<Vec<_>>()
                .join(",")
        )
        .into_bytes();

        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 12345,
                url: "https://example.com/api/v1/deeply/nested/endpoint".into(),
                method: "POST".into(),
                headers: nested_headers.clone(),
                body: Some(nested_body.clone()),
            }),
        };

        let out = roundtrip(msg);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(12345, p.request_id);
            assert_eq!("POST", p.method);
            assert_eq!(20, p.headers.len(), "嵌套 headers 长度应保持一致");
            assert_eq!(nested_headers, p.headers, "嵌套 headers 内容应完全一致");
            assert_eq!(Some(nested_body), p.body, "嵌套 body 内容应完全一致");
        } else {
            panic!("期望 FetchRequest");
        }

        // 测试更深层的嵌套：StorageOp 中 value 包含嵌套 JSON 字符串
        // 手动构造嵌套 JSON 字符串（不依赖 serde_json，直接拼字符串）
        let deeply_nested_value = format!(
            "{{\"users\":[{}]}}",
            (0..5)
                .map(|i| format!(
                    "{{\"name\":\"user_{i}\",\"settings\":{{\"theme\":\"dark\",\"lang\":\"zh\",\"tags\":[\"a\",\"b\",\"c\"]}}}}"
                ))
                .collect::<Vec<_>>()
                .join(",")
        );

        let msg2 = IpcMessage {
            id: 43,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "deeply_nested_key".into(),
                value: Some(deeply_nested_value.clone()),
                origin: "https://example.com".into(),
            }),
        };

        let out2 = roundtrip(msg2);
        if let IpcMessageKind::StorageOp(p) = out2.kind {
            assert_eq!(Some(deeply_nested_value), p.value, "深层嵌套 value 往返应一致");
        } else {
            panic!("期望 StorageOp");
        }

        // 测试 FetchResponse 中 headers 嵌套元组的往返
        let response_headers: Vec<(String, String)> = vec![
            ("Content-Type".into(), "text/html; charset=utf-8".into()),
            ("Set-Cookie".into(), "session=abc123; Path=/; HttpOnly; Secure".into()),
            ("X-Forwarded-For".into(), "10.0.0.1, 172.16.0.1, 192.168.1.1".into()),
            ("Link".into(), "</api/page/2>; rel=\"next\"; title=\"下一页\"".into()),
        ];

        let msg3 = IpcMessage {
            id: 44,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 99999,
                status_code: 206,
                headers: response_headers.clone(),
                body: vec![0u8; 1024], // 1KB 零填充 body
            }),
        };

        let out3 = roundtrip(msg3);
        if let IpcMessageKind::FetchResponse(p) = out3.kind {
            assert_eq!(99999, p.request_id);
            assert_eq!(206, p.status_code);
            assert_eq!(response_headers, p.headers, "嵌套响应 headers 往返应一致");
            assert_eq!(1024, p.body.len());
            assert!(p.body.iter().all(|&b| b == 0), "body 应全为零");
        } else {
            panic!("期望 FetchResponse");
        }
    }

    /// 测试使用每种支持的字段类型的 IPC 消息的序列化/反序列化。
    ///
    /// 构造一条消息，使用 u64、String、Option、Vec、enum variants、
    /// bool、u8、f32 以及嵌套结构体——验证完整的往返行程。
    #[test]
    fn test_ipc_message_with_all_field_types() {
        // 消息，该消息使用了每一种支持的字段类型
        let msg = IpcMessage {
            id: u64::MAX, // u64
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 42,                                           // u64
                url: "https://example.com/api?v=1&lang=中文#frag".into(), // 带有 unicode 的 String
                method: "POST".into(),                                    // String
                headers: vec![
                    // Vec<(String, String)>
                    ("Content-Type".into(), "application/json".into()),
                    ("X-Empty".into(), String::new()), // 空字符串
                    ("X-Unicode".into(), "Ñoño café ☕".into()),
                ],
                body: Some(vec![0u8, 127, 255, 0xDE, 0xAD]), // Option<Vec<u8>>
            }),
        };
        let bytes = serialize(&msg).expect("serialize");
        let out = deserialize(&bytes).expect("deserialize");

        assert_eq!(u64::MAX, out.id);
        if let IpcMessageKind::FetchRequest(p) = out.kind {
            assert_eq!(42, p.request_id);
            assert_eq!("https://example.com/api?v=1&lang=中文#frag", p.url);
            assert_eq!("POST", p.method);
            assert_eq!(3, p.headers.len());
            assert_eq!("application/json", p.headers[0].1);
            assert!(p.headers[1].1.is_empty());
            assert_eq!("Ñoño café ☕", p.headers[2].1);
            assert_eq!(Some(vec![0u8, 127, 255, 0xDE, 0xAD]), p.body);
        } else {
            panic!("expected FetchRequest");
        }

        // 测试鼠标事件字段：f32 x/y, u8 按钮, 枚举 event_type
        let mouse_msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: -100.5,
                y: 0.0,
                button: u8::MAX,
                event_type: MouseEventType::Move,
            }),
        };
        let out2 = roundtrip(mouse_msg);
        if let IpcMessageKind::MouseEvent(p) = out2.kind {
            assert_eq!(-100.5, p.x);
            assert_eq!(0.0, p.y);
            assert_eq!(u8::MAX, p.button);
            assert_eq!(MouseEventType::Move, p.event_type);
        } else {
            panic!("expected MouseEvent");
        }

        // 测试键盘事件字段：bool 修饰键
        let kb_msg = IpcMessage {
            id: 2,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: String::new(),
                code: String::new(),
                ctrl: true,
                shift: true,
                alt: true,
                meta: true,
                event_type: KeyboardEventType::Up,
            }),
        };
        let out3 = roundtrip(kb_msg);
        if let IpcMessageKind::KeyboardEvent(p) = out3.kind {
            assert!(p.ctrl && p.shift && p.alt && p.meta);
            assert!(p.key.is_empty() && p.code.is_empty());
        } else {
            panic!("expected KeyboardEvent");
        }

        // 测试存储操作字段：枚举存储类型 + 操作
        let store_msg = IpcMessage {
            id: 3,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Session,
                operation: StorageOperation::Remove,
                key: "k".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        };
        let out4 = roundtrip(store_msg);
        if let IpcMessageKind::StorageOp(p) = out4.kind {
            assert_eq!(StorageType::Session, p.storage_type);
            assert_eq!(StorageOperation::Remove, p.operation);
            assert!(p.value.is_none());
        } else {
            panic!("expected StorageOp");
        }

        // 测试滚动事件字段：f32 增量
        let scroll_msg = IpcMessage {
            id: 4,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: f32::MAX,
                delta_y: f32::MIN,
            }),
        };
        let out5 = roundtrip(scroll_msg);
        if let IpcMessageKind::ScrollEvent(p) = out5.kind {
            assert_eq!(f32::MAX, p.delta_x);
            assert_eq!(f32::MIN, p.delta_y);
        } else {
            panic!("expected ScrollEvent");
        }

        // 测试 FetchResponse 字段：u16 状态码, Vec<u8> 主体
        let resp_msg = IpcMessage {
            id: 5,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 0,
                status_code: 418,
                headers: vec![],
                body: vec![],
            }),
        };
        let out6 = roundtrip(resp_msg);
        if let IpcMessageKind::FetchResponse(p) = out6.kind {
            assert_eq!(418, p.status_code);
            assert!(p.body.is_empty() && p.headers.is_empty());
        } else {
            panic!("expected FetchResponse");
        }
    }

    /// 测试 IPC 向后兼容性：添加新变体不会破坏旧的反序列化。
    ///
    /// 在 `IpcMessageKind` 中添加新变体不会改变现有变体的序列化形式。
    /// 序列化一条旧样式的消息，验证它仍然能正确反序列化。
    #[test]
    fn test_ipc_backward_compatibility() {
        // 序列化一条包含所有当前变体的消息
        let old_kinds: Vec<IpcMessageKind> = vec![
            IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
            }),
            IpcMessageKind::GoBack,
            IpcMessageKind::GoForward,
            IpcMessageKind::StopLoading,
            IpcMessageKind::Reload,
            IpcMessageKind::TitleChanged("Test".into()),
            IpcMessageKind::UrlChanged("https://example.com".into()),
            IpcMessageKind::LoadComplete,
            IpcMessageKind::LoadFailed("timeout".into()),
            IpcMessageKind::Heartbeat,
            IpcMessageKind::CrashNotification("segfault".into()),
            IpcMessageKind::Ok,
            IpcMessageKind::Error("failure".into()),
        ];

        // 序列化所有旧的消息并存储字节
        let serialized: Vec<(u64, Vec<u8>, IpcMessageKind)> = old_kinds
            .into_iter()
            .enumerate()
            .map(|(i, kind)| {
                let msg = IpcMessage {
                    id: i as u64,
                    kind: kind.clone(),
                };
                let bytes = serialize(&msg).expect("serialize");
                (i as u64, bytes, kind)
            })
            .collect();

        // 反序列化每个消息并验证所有字段完好无损
        for (id, bytes, original_kind) in &serialized {
            let out: IpcMessage = deserialize(bytes).expect("deserialize");
            assert_eq!(*id, out.id, "id mismatch for message {id}");

            // 按类型验证原始内容与往返行程完全匹配
            match (original_kind, &out.kind) {
                (IpcMessageKind::Navigate(a), IpcMessageKind::Navigate(b)) => {
                    assert_eq!(a.url, b.url);
                    assert_eq!(a.referrer, b.referrer);
                }
                (IpcMessageKind::TitleChanged(a), IpcMessageKind::TitleChanged(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::UrlChanged(a), IpcMessageKind::UrlChanged(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::LoadFailed(a), IpcMessageKind::LoadFailed(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::CrashNotification(a), IpcMessageKind::CrashNotification(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::Error(a), IpcMessageKind::Error(b)) => {
                    assert_eq!(a, b);
                }
                (IpcMessageKind::GoBack, IpcMessageKind::GoBack)
                | (IpcMessageKind::GoForward, IpcMessageKind::GoForward)
                | (IpcMessageKind::StopLoading, IpcMessageKind::StopLoading)
                | (IpcMessageKind::Reload, IpcMessageKind::Reload)
                | (IpcMessageKind::LoadComplete, IpcMessageKind::LoadComplete)
                | (IpcMessageKind::Heartbeat, IpcMessageKind::Heartbeat)
                | (IpcMessageKind::Ok, IpcMessageKind::Ok) => {}
                _ => panic!("kind mismatch for message {id}"),
            }

            // 重新序列化并验证字节是相同的（往返行程的确定性）
            let re_bytes = serialize(&out).expect("re-serialize");
            assert_eq!(bytes, &re_bytes, "byte mismatch for message {id}");
        }
    }

    /// 测试同一消息多次序列化产生完全相同的二进制输出（bincode 确定性）。
    /// 对于 IPC 协议的可靠性和幂等性至关重要。
    #[test]
    fn test_binary_round_trip_determinism() {
        let test_messages: Vec<IpcMessage> = vec![
            // 简单变体
            IpcMessage {
                id: 1,
                kind: IpcMessageKind::Ok,
            },
            IpcMessage {
                id: 2,
                kind: IpcMessageKind::Heartbeat,
            },
            // 带字符串的变体
            IpcMessage {
                id: 3,
                kind: IpcMessageKind::TitleChanged("确定性测试标题".into()),
            },
            IpcMessage {
                id: 4,
                kind: IpcMessageKind::Error("错误信息 123!@#".into()),
            },
            // 带可选字段的变体
            IpcMessage {
                id: 5,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com/path?q=hello&lang=zh".into(),
                    referrer: Some("https://google.com".into()),
                }),
            },
            IpcMessage {
                id: 6,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                }),
            },
            // 带复杂数据结构的变体
            IpcMessage {
                id: 7,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: 42,
                    url: "https://api.example.com/v1/data".into(),
                    method: "POST".into(),
                    headers: vec![
                        ("Content-Type".into(), "application/json".into()),
                        ("Authorization".into(), "Bearer token123".into()),
                        ("X-Request-Id".into(), "abc-def-ghi".into()),
                    ],
                    body: Some(b"{\"key\":\"value\",\"nested\":{\"a\":1}}".to_vec()),
                }),
            },
            // 带枚举字段的变体
            IpcMessage {
                id: 8,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: StorageType::Session,
                    operation: StorageOperation::Set,
                    key: "test_key".into(),
                    value: Some("test_value".into()),
                    origin: "https://example.com".into(),
                }),
            },
            // 带浮点数的变体
            IpcMessage {
                id: 9,
                kind: IpcMessageKind::MouseEvent(MouseEventParams {
                    x: 123.456,
                    y: 789.012,
                    button: 2,
                    event_type: MouseEventType::DblClick,
                }),
            },
            IpcMessage {
                id: 10,
                kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                    delta_x: -3.14,
                    delta_y: 2.718,
                }),
            },
        ];

        for msg in &test_messages {
            // 序列化 3 次，验证每次输出完全一致
            let bytes1 = serialize(msg).expect("第一次序列化应成功");
            let bytes2 = serialize(msg).expect("第二次序列化应成功");
            let bytes3 = serialize(msg).expect("第三次序列化应成功");

            assert_eq!(
                bytes1, bytes2,
                "确定性违反：同一消息 (id={}) 的两次序列化结果不同",
                msg.id
            );
            assert_eq!(
                bytes2, bytes3,
                "确定性违反：同一消息 (id={}) 的第三次序列化与前两次不同",
                msg.id
            );

            // 反序列化后重新序列化，验证也产生相同字节（完整的往返确定性）
            let roundtrip_bytes = {
                let deserialized = deserialize(&bytes1).expect("反序列化应成功");
                serialize(&deserialized).expect("重新序列化应成功")
            };
            assert_eq!(
                bytes1, roundtrip_bytes,
                "往返确定性违反：消息 (id={}) 反序列化后重新序列化的结果与原始不同",
                msg.id
            );
        }
    }
}
