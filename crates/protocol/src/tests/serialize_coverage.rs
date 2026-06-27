//! 序列化/反序列化错误路径覆盖测试。

use super::*;

#[test]
fn test_deserialize_empty_data() {
    let result = deserialize(&[]);
    assert!(result.is_err());
    if let Err(ProtocolError::Deserialization(msg)) = result {
        assert!(!msg.is_empty());
    } else {
        panic!("expected Deserialization error");
    }
}

#[test]
fn test_deserialize_garbage_data() {
    let garbage: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03];
    let result = deserialize(garbage);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_partial_valid_data() {
    // 序列化一个有效消息，然后截断
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    };
    let full_bytes = serialize(&msg).unwrap();
    // 截断后一半
    let truncated = &full_bytes[..full_bytes.len() / 2];
    let result = deserialize(truncated);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_corrupted_data() {
    // 序列化后大幅破坏数据
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::GoBack,
    };
    let bytes = serialize(&msg).unwrap();
    // 替换所有字节为垃圾数据
    let mut corrupted = vec![0xFFu8; bytes.len()];
    // 保留 bincode 长度前缀使其尝试解析
    if corrupted.len() >= 8 {
        corrupted[0..8].copy_from_slice(&bytes[0..8]);
    }
    let result = deserialize(&corrupted);
    // 可能成功但数据不对，也可能失败 — 关键是不 panic
    if let Ok(out) = result {
        // 如果意外成功，id 不应匹配
        assert_ne!(out.id, 1);
    }
}

#[test]
fn test_serialize_error_display() {
    let err = ProtocolError::Serialization("test error".to_string());
    let display = format!("{err}");
    assert!(display.contains("test error"));
    assert!(display.contains("Serialization"));
}

#[test]
fn test_deserialize_error_display() {
    let err = ProtocolError::Deserialization("bad data".to_string());
    let display = format!("{err}");
    assert!(display.contains("bad data"));
    assert!(display.contains("Deserialization"));
}

#[test]
fn test_channel_error_display() {
    let err = ProtocolError::Channel("broken".to_string());
    let display = format!("{err}");
    assert!(display.contains("broken"));
}

#[test]
fn test_process_error_display() {
    let err = ProtocolError::Process("crashed".to_string());
    let display = format!("{err}");
    assert!(display.contains("crashed"));
}
