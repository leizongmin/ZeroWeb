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
            let msg = IpcMessage {
                id: i as u64,
                kind,
            };
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
}
