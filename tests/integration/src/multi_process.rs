//! 多进程架构集成测试。
//!
//! 验证 IPC 传输层、进程管理器和模拟渲染进程之间的协作。
//! 使用 `shared_channel_pair` 模拟进程间通信，避免实际启动子进程
//! （实际子进程集成测试需要 GPU/Display 环境）。

use zero_protocol::message::{
    FetchParams, FetchResponseParams, IpcMessage, IpcMessageKind, KeyboardEventParams, KeyboardEventType,
    MouseEventParams, MouseEventType, NavigateParams, ScrollEventParams, StorageOpParams, StorageOperation,
    StorageType,
};
use zero_protocol::transport::{PipeTransport, SharedMemoryChannel, shared_channel_pair};
use zero_protocol::{IpcChannel, ProcessManager, ProtocolError};

// ── IPC 传输层集成测试 ──────────────────────────────────────────

/// 测试共享内存通道的完整双向通信。
#[test]
fn test_shared_channel_full_duplex() {
    let (mut browser, mut renderer) = shared_channel_pair();

    // 浏览器 → 渲染进程：导航命令
    browser
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        })
        .unwrap();

    // 渲染进程 → 浏览器：加载完成
    let nav_msg = renderer.recv().unwrap();
    assert!(matches!(nav_msg.kind, IpcMessageKind::Navigate(_)));

    renderer
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::LoadComplete,
        })
        .unwrap();

    let load_msg = browser.recv().unwrap();
    assert!(matches!(load_msg.kind, IpcMessageKind::LoadComplete));
}

/// 测试 PipeTransport 使用内存缓冲区的帧协议。
#[test]
fn test_pipe_transport_frame_protocol() {
    let msg = IpcMessage {
        id: 42,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://test.example/page?q=hello&lang=en".into(),
            referrer: Some("https://origin.example".into()),
            navigation_epoch: 0,
        }),
    };

    // 序列化
    let data = zero_protocol::serialize::serialize(&msg).unwrap();

    // 帧化
    let mut framed = Vec::new();
    use std::io::Write;
    let len = data.len() as u32;
    framed.write_all(&len.to_le_bytes()).unwrap();
    framed.write_all(&data).unwrap();

    // 反帧化和反序列化
    use std::io::Read;
    let mut reader = &framed[..];
    let mut transport = PipeTransport::new(reader, std::io::empty());
    let received = transport.recv().unwrap();

    assert_eq!(received.id, 42);
    if let IpcMessageKind::Navigate(params) = received.kind {
        assert_eq!(params.url, "https://test.example/page?q=hello&lang=en");
        assert_eq!(params.referrer.as_deref(), Some("https://origin.example"));
    } else {
        panic!("期望 Navigate 消息");
    }
}

/// 测试 IPC 序列化的确定性（同一消息两次序列化结果相同）。
#[test]
fn test_ipc_serialization_deterministic() {
    let msg = IpcMessage {
        id: 100,
        kind: IpcMessageKind::StorageOp(StorageOpParams {
            storage_type: StorageType::Local,
            operation: StorageOperation::Set,
            key: "test-key".into(),
            value: Some("test-value".into()),
            origin: "https://example.com".into(),
        }),
    };

    let bytes1 = zero_protocol::serialize::serialize(&msg).unwrap();
    let bytes2 = zero_protocol::serialize::serialize(&msg).unwrap();
    assert_eq!(bytes1, bytes2);
}

// ── 消息流程集成测试 ───────────────────────────────────────────

/// 测试完整的页面加载生命周期：导航 → URL变更 → 标题变更 → 加载完成。
#[test]
fn test_page_load_lifecycle() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // === 浏览器端：发送导航命令 ===
    browser_ch
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        })
        .unwrap();

    // === 渲染进程端：接收并处理 ===
    let nav_msg = renderer_ch.recv().unwrap();
    assert_eq!(nav_msg.id, 1);
    if let IpcMessageKind::Navigate(ref params) = nav_msg.kind {
        assert_eq!(params.url, "https://example.com");
    } else {
        panic!("期望 Navigate");
    }

    // === 渲染进程：报告 URL 变更 ===
    renderer_ch
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::UrlChanged("https://example.com".into()),
        })
        .unwrap();

    // === 浏览器端：确认 URL 变更 ===
    let url_msg = browser_ch.recv().unwrap();
    assert!(matches!(url_msg.kind, IpcMessageKind::UrlChanged(_)));

    // === 渲染进程：报告标题变更 ===
    renderer_ch
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::TitleChanged("Example Domain".into()),
        })
        .unwrap();

    let title_msg = browser_ch.recv().unwrap();
    assert!(matches!(title_msg.kind, IpcMessageKind::TitleChanged(_)));

    // === 渲染进程：报告加载完成 ===
    renderer_ch
        .send(IpcMessage {
            id: 4,
            kind: IpcMessageKind::LoadComplete,
        })
        .unwrap();

    let complete_msg = browser_ch.recv().unwrap();
    assert!(matches!(complete_msg.kind, IpcMessageKind::LoadComplete));
}

/// 测试渲染进程发起网络请求，浏览器进程转发并回复。
#[test]
fn test_fetch_proxy_flow() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // 渲染进程发起 CSS 资源请求
    renderer_ch
        .send(IpcMessage {
            id: 10,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id: 1001,
                url: "https://example.com/style.css".into(),
                method: "GET".into(),
                headers: vec![("Accept".into(), "text/css".into())],
                body: None,
            }),
        })
        .unwrap();

    // 浏览器进程接收请求
    let req_msg = browser_ch.recv().unwrap();
    if let IpcMessageKind::FetchRequest(ref params) = req_msg.kind {
        assert_eq!(params.request_id, 1001);
        assert_eq!(params.url, "https://example.com/style.css");
        assert_eq!(params.method, "GET");
    } else {
        panic!("期望 FetchRequest");
    }

    // 浏览器进程回复响应
    browser_ch
        .send(IpcMessage {
            id: 10,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1001,
                status_code: 200,
                headers: vec![("Content-Type".into(), "text/css".into())],
                body: b"body { margin: 0; }".to_vec(),
            }),
        })
        .unwrap();

    // 渲染进程接收响应
    let resp_msg = renderer_ch.recv().unwrap();
    if let IpcMessageKind::FetchResponse(ref params) = resp_msg.kind {
        assert_eq!(params.request_id, 1001);
        assert_eq!(params.status_code, 200);
        assert_eq!(params.body, b"body { margin: 0; }");
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试存储操作代理流程。
#[test]
fn test_storage_proxy_flow() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // 渲染进程发起 localStorage 写入
    renderer_ch
        .send(IpcMessage {
            id: 20,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Set,
                key: "user-preference".into(),
                value: Some("dark-mode".into()),
                origin: "https://example.com".into(),
            }),
        })
        .unwrap();

    // 浏览器进程处理并回复
    let msg = browser_ch.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::StorageOp(_)));

    browser_ch
        .send(IpcMessage {
            id: 20,
            kind: IpcMessageKind::Ok,
        })
        .unwrap();

    let resp = renderer_ch.recv().unwrap();
    assert!(matches!(resp.kind, IpcMessageKind::Ok));

    // 渲染进程发起 localStorage 读取
    renderer_ch
        .send(IpcMessage {
            id: 21,
            kind: IpcMessageKind::StorageOp(StorageOpParams {
                storage_type: StorageType::Local,
                operation: StorageOperation::Get,
                key: "user-preference".into(),
                value: None,
                origin: "https://example.com".into(),
            }),
        })
        .unwrap();

    let _ = browser_ch.recv().unwrap();
}

/// 测试输入事件转发（鼠标、键盘、滚动）。
#[test]
fn test_input_event_forwarding() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // 鼠标点击
    browser_ch
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::MouseEvent(MouseEventParams {
                x: 150.0,
                y: 300.0,
                button: 0,
                event_type: MouseEventType::Click,
            }),
        })
        .unwrap();

    let msg = renderer_ch.recv().unwrap();
    if let IpcMessageKind::MouseEvent(ref p) = msg.kind {
        assert_eq!(p.x, 150.0);
        assert_eq!(p.y, 300.0);
        assert_eq!(p.event_type, MouseEventType::Click);
    } else {
        panic!("期望 MouseEvent");
    }

    // 键盘输入
    browser_ch
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::KeyboardEvent(KeyboardEventParams {
                key: "a".into(),
                code: "KeyA".into(),
                ctrl: true,
                shift: false,
                alt: false,
                meta: false,
                event_type: KeyboardEventType::Down,
            }),
        })
        .unwrap();

    let msg = renderer_ch.recv().unwrap();
    if let IpcMessageKind::KeyboardEvent(ref p) = msg.kind {
        assert_eq!(p.key, "a");
        assert!(p.ctrl);
        assert_eq!(p.event_type, KeyboardEventType::Down);
    } else {
        panic!("期望 KeyboardEvent");
    }

    // 滚动
    browser_ch
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
                delta_x: 0.0,
                delta_y: 50.0,
                ..Default::default()
            }),
        })
        .unwrap();

    let msg = renderer_ch.recv().unwrap();
    if let IpcMessageKind::ScrollEvent(ref p) = msg.kind {
        assert_eq!(p.delta_y, 50.0);
    } else {
        panic!("期望 ScrollEvent");
    }
}

/// 测试心跳机制。
#[test]
fn test_heartbeat_mechanism() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // 渲染进程发送心跳
    renderer_ch
        .send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::Heartbeat,
        })
        .unwrap();

    // 浏览器进程收到心跳并回复
    let msg = browser_ch.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::Heartbeat));

    browser_ch
        .send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::Heartbeat,
        })
        .unwrap();

    // 渲染进程收到回复
    let reply = renderer_ch.recv().unwrap();
    assert!(matches!(reply.kind, IpcMessageKind::Heartbeat));
}

/// 测试页面加载失败场景。
#[test]
fn test_page_load_failure() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    browser_ch
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://unreachable.invalid".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        })
        .unwrap();

    let _ = renderer_ch.recv().unwrap();

    renderer_ch
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::LoadFailed("DNS 解析失败: NXDOMAIN".into()),
        })
        .unwrap();

    let msg = browser_ch.recv().unwrap();
    if let IpcMessageKind::LoadFailed(reason) = &msg.kind {
        assert!(reason.contains("DNS"));
    } else {
        panic!("期望 LoadFailed");
    }
}

/// 测试崩溃通知流程。
#[test]
fn test_crash_notification_flow() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    renderer_ch
        .send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::CrashNotification("OOM: exceeded 512MB".into()),
        })
        .unwrap();

    let msg = browser_ch.recv().unwrap();
    if let IpcMessageKind::CrashNotification(reason) = &msg.kind {
        assert!(reason.contains("OOM"));
    } else {
        panic!("期望 CrashNotification");
    }
}

// ── ProcessManager 集成测试 ─────────────────────────────────────

/// 测试 ProcessManager 创建和基本属性。
#[test]
fn test_process_manager_basic() {
    let mut pm = ProcessManager::new("/nonexistent/zero-renderer");
    assert_eq!(pm.active_count(), 0);
    assert!(pm.active_ids().is_empty());

    let id1 = pm.next_msg_id();
    let id2 = pm.next_msg_id();
    assert!(id2 > id1);

    // 关闭不存在的渲染进程不应报错
    assert!(pm.shutdown_renderer(999).is_ok());
    pm.shutdown_all();
    assert_eq!(pm.active_count(), 0);
}

/// 测试 ProcessManager 崩溃检测（空管理器）。
#[test]
fn test_process_manager_crash_detection_empty() {
    let mut pm = ProcessManager::new("/nonexistent/zero-renderer");
    let crashed = pm.check_crashes();
    assert!(crashed.is_empty());
}

/// 测试多消息交错传输。
#[test]
fn test_interleaved_messages() {
    let (mut a, mut b) = shared_channel_pair();

    // 连续发送多种类型的消息
    a.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Heartbeat,
    })
    .unwrap();
    a.send(IpcMessage {
        id: 2,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://a.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    })
    .unwrap();
    a.send(IpcMessage {
        id: 3,
        kind: IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 10.0,
            delta_y: 20.0,
            ..Default::default()
        }),
    })
    .unwrap();

    // 接收端按 FIFO 顺序收到（先发送的先接收）
    let msg = b.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::Heartbeat));

    let msg = b.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::Navigate(_)));

    let msg = b.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::ScrollEvent(_)));
}

/// 测试大载荷 IPC 传输。
#[test]
fn test_large_payload_transport() {
    let (mut a, mut b) = shared_channel_pair();

    // 构造包含大 body 的网络响应
    let large_body = vec![0xAB_u8; 100_000];
    a.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: 1,
            status_code: 200,
            headers: vec![],
            body: large_body.clone(),
        }),
    })
    .unwrap();

    let msg = b.recv().unwrap();
    if let IpcMessageKind::FetchResponse(ref params) = msg.kind {
        assert_eq!(params.body.len(), 100_000);
        assert_eq!(params.body[0], 0xAB);
        assert_eq!(params.body[99_999], 0xAB);
    } else {
        panic!("期望 FetchResponse");
    }
}

/// 测试双向并发通信（模拟多标签页）。
#[test]
fn test_concurrent_bidirectional() {
    let (mut ch_a, mut ch_b) = shared_channel_pair();

    // A → B
    ch_a.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://tab1.com".into(),
            referrer: None,
            navigation_epoch: 0,
        }),
    })
    .unwrap();

    // B → A
    ch_b.send(IpcMessage {
        id: 2,
        kind: IpcMessageKind::TitleChanged("Tab 1".into()),
    })
    .unwrap();

    // 各自接收
    let msg_b = ch_b.recv().unwrap();
    assert!(matches!(msg_b.kind, IpcMessageKind::Navigate(_)));

    let msg_a = ch_a.recv().unwrap();
    assert!(matches!(msg_a.kind, IpcMessageKind::TitleChanged(_)));
}

/// 测试错误响应传输。
#[test]
fn test_error_response_transport() {
    let (mut a, mut b) = shared_channel_pair();

    a.send(IpcMessage {
        id: 1,
        kind: IpcMessageKind::Error("渲染错误: 字体加载失败".into()),
    })
    .unwrap();

    let msg = b.recv().unwrap();
    if let IpcMessageKind::Error(reason) = &msg.kind {
        assert!(reason.contains("字体"));
    } else {
        panic!("期望 Error");
    }
}

/// 测试导航历史操作（前进/后退/重载）。
#[test]
fn test_navigation_history_commands() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    // 导航
    browser_ch
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: "https://page1.com".into(),
                referrer: None,
                navigation_epoch: 0,
            }),
        })
        .unwrap();
    let _ = renderer_ch.recv().unwrap();

    // 后退
    browser_ch
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::GoBack,
        })
        .unwrap();
    let msg = renderer_ch.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::GoBack));

    // 前进
    browser_ch
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::GoForward,
        })
        .unwrap();
    let msg = renderer_ch.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::GoForward));

    // 重载
    browser_ch
        .send(IpcMessage {
            id: 4,
            kind: IpcMessageKind::Reload,
        })
        .unwrap();
    let msg = renderer_ch.recv().unwrap();
    assert!(matches!(msg.kind, IpcMessageKind::Reload));
}

/// 测试多个存储操作组合。
#[test]
fn test_multiple_storage_operations() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

    let operations = vec![
        (StorageOperation::Set, "key1", Some("value1")),
        (StorageOperation::Set, "key2", Some("value2")),
        (StorageOperation::Get, "key1", None),
        (StorageOperation::Remove, "key2", None),
        (StorageOperation::Length, "", None),
        (StorageOperation::Clear, "", None),
    ];

    for (i, (op, key, value)) in operations.iter().enumerate() {
        renderer_ch
            .send(IpcMessage {
                id: i as u64,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: StorageType::Session,
                    operation: op.clone(),
                    key: (*key).into(),
                    value: value.map(|v| v.into()),
                    origin: "https://example.com".into(),
                }),
            })
            .unwrap();

        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::StorageOp(_)));
    }
}
