use super::*;

fn assert_identifiers(kind: IpcMessageKind, expected: (u64, u64, u64)) {
    let actual = match kind {
        IpcMessageKind::CompositorFrame {
            surface_id,
            navigation_epoch,
            frame_id,
            ..
        }
        | IpcMessageKind::CompositorFrameResult {
            surface_id,
            navigation_epoch,
            frame_id,
        }
        | IpcMessageKind::GetCompositorFrame {
            surface_id,
            navigation_epoch,
            frame_id,
        }
        | IpcMessageKind::CompositorFrameData {
            surface_id,
            navigation_epoch,
            frame_id,
            ..
        } => (surface_id, navigation_epoch, frame_id),
        other => panic!("期望 compositor 消息，实际为 {other:?}"),
    };
    assert_eq!(actual, expected);
}

/// 验证 compositor 提交、完成和读取消息序列化后保留完整帧标识。
#[test]
fn compositor_messages_serialize_identifiers() {
    let identifiers = (17, 23, 42);
    let messages = [
        IpcMessageKind::CompositorFrame {
            surface_id: identifiers.0,
            navigation_epoch: identifiers.1,
            frame_id: identifiers.2,
            paint: Box::new(PaintSnapshotParams::default()),
        },
        IpcMessageKind::CompositorFrameResult {
            surface_id: identifiers.0,
            navigation_epoch: identifiers.1,
            frame_id: identifiers.2,
        },
        IpcMessageKind::GetCompositorFrame {
            surface_id: identifiers.0,
            navigation_epoch: identifiers.1,
            frame_id: identifiers.2,
        },
        IpcMessageKind::CompositorFrameData {
            surface_id: identifiers.0,
            navigation_epoch: identifiers.1,
            frame_id: identifiers.2,
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 4],
            shm_name: None,
        },
    ];

    for (id, kind) in messages.into_iter().enumerate() {
        let decoded = roundtrip(IpcMessage { id: id as u64, kind });
        assert_identifiers(decoded.kind, identifiers);
    }
}

/// 验证 compositor 帧标识支持 `u64` 的最小值和最大值。
#[test]
fn compositor_identifier_boundaries_round_trip() {
    for identifiers in [(0, 0, 0), (u64::MAX, u64::MAX, u64::MAX)] {
        let decoded = roundtrip(IpcMessage {
            id: u64::MAX,
            kind: IpcMessageKind::CompositorFrameResult {
                surface_id: identifiers.0,
                navigation_epoch: identifiers.1,
                frame_id: identifiers.2,
            },
        });
        assert_eq!(decoded.id, u64::MAX);
        assert_identifiers(decoded.kind, identifiers);
    }
}

/// 验证 compositor surface 释放消息序列化后保留 surface 标识。
#[test]
fn compositor_surface_release_round_trip() {
    for surface_id in [0, u64::MAX] {
        let decoded = roundtrip(IpcMessage {
            id: 7,
            kind: IpcMessageKind::ReleaseCompositorSurface { surface_id },
        });
        assert_eq!(decoded.id, 7);
        match decoded.kind {
            IpcMessageKind::ReleaseCompositorSurface {
                surface_id: decoded_surface_id,
            } => assert_eq!(decoded_surface_id, surface_id),
            other => panic!("期望 surface 释放消息，实际为 {other:?}"),
        }
    }
}

/// 验证 renderer 帧发布模式与立即重发控制消息可往返序列化。
#[test]
fn frame_publish_control_messages_round_trip() {
    for mode in [FramePublishMode::Legacy, FramePublishMode::Compositor] {
        let decoded = roundtrip(IpcMessage {
            id: 8,
            kind: IpcMessageKind::SetFramePublishMode(mode),
        });
        assert!(matches!(
            decoded.kind,
            IpcMessageKind::SetFramePublishMode(decoded_mode) if decoded_mode == mode
        ));
    }

    let decoded = roundtrip(IpcMessage {
        id: 9,
        kind: IpcMessageKind::RequestFrame,
    });
    assert!(matches!(decoded.kind, IpcMessageKind::RequestFrame));
}

/// 验证两个 surface 的提交与回执在双向通道中保持各自标识。
#[test]
fn compositor_two_surfaces_round_trip_independently() {
    let (mut client, mut compositor) = shared_channel_pair();
    let surfaces = [(1, (101, 7, 11)), (2, (202, 9, 13))];

    for (id, identifiers) in surfaces {
        client
            .send(IpcMessage {
                id,
                kind: IpcMessageKind::CompositorFrame {
                    surface_id: identifiers.0,
                    navigation_epoch: identifiers.1,
                    frame_id: identifiers.2,
                    paint: Box::new(PaintSnapshotParams::default()),
                },
            })
            .expect("提交 surface 帧");
    }

    for (id, expected) in surfaces {
        let request = compositor.recv().expect("接收 surface 帧");
        assert_eq!(request.id, id);
        assert_identifiers(request.kind, expected);
        compositor
            .send(IpcMessage {
                id,
                kind: IpcMessageKind::CompositorFrameResult {
                    surface_id: expected.0,
                    navigation_epoch: expected.1,
                    frame_id: expected.2,
                },
            })
            .expect("发送 surface 回执");
    }

    for (id, expected) in surfaces {
        let response = client.recv().expect("接收 surface 回执");
        assert_eq!(response.id, id);
        assert_identifiers(response.kind, expected);
    }
}
