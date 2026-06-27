//! 协议 crate 性能基准测试。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zero_protocol::{
    FetchParams, IpcMessage, IpcMessageKind, MouseEventParams, MouseEventType, NavigateParams, deserialize, serialize,
};

/// 基准：IPC 消息序列化
fn bench_serialize(c: &mut Criterion) {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com/page".to_string(),
            referrer: Some("https://google.com".to_string()),
            navigation_epoch: 0,
        }),
    };

    c.bench_function("ipc_serialize_10000", |b| {
        b.iter(|| {
            for _ in 0..10000 {
                let _ = black_box(serialize(&msg));
            }
        })
    });
}

/// 基准：IPC 消息反序列化
fn bench_deserialize(c: &mut Criterion) {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com/page".to_string(),
            referrer: Some("https://google.com".to_string()),
            navigation_epoch: 0,
        }),
    };
    let bytes = serialize(&msg).unwrap();

    c.bench_function("ipc_deserialize_10000", |b| {
        b.iter(|| {
            for _ in 0..10000 {
                let _ = black_box(deserialize(&bytes));
            }
        })
    });
}

/// 基准：序列化 + 反序列化往返
fn bench_roundtrip(c: &mut Criterion) {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::MouseEvent(MouseEventParams {
            x: 100.0,
            y: 200.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    };

    c.bench_function("ipc_roundtrip_10000", |b| {
        b.iter(|| {
            for _ in 0..10000 {
                let bytes = serialize(&msg).unwrap();
                let _ = black_box(deserialize(&bytes));
            }
        })
    });
}

/// 基准：大消息序列化（带 headers 和 body）
fn bench_large_message(c: &mut Criterion) {
    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchRequest(FetchParams {
            request_id: 100,
            url: "https://api.example.com/v1/data".to_string(),
            method: "POST".to_string(),
            headers: vec![
                ("Content-Type".into(), "application/json".into()),
                ("Authorization".into(), "Bearer token123".into()),
                ("Accept".into(), "application/json".into()),
                ("X-Custom".into(), "value".into()),
            ],
            body: Some(vec![0u8; 1024]),
        }),
    };

    c.bench_function("ipc_large_message_serialize_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _ = black_box(serialize(&msg));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_serialize,
    bench_deserialize,
    bench_roundtrip,
    bench_large_message,
);
criterion_main!(benches);
