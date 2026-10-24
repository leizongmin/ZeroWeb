//! 媒体 host 消息族契约测试（方案 C C1）：全变体 bincode 往返 + 伴生枚举覆盖 +
//! 常量不变式（`--type=media` 映射断言在 process.rs 内联测试）。

use super::*;

/// 构造 17 个媒体变体的全覆盖消息集（wire 契约面逐变体锁定）。
fn media_messages() -> Vec<IpcMessage> {
    vec![
        // 控制面（renderer → host）
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                source_key: 0xA11CE,
                total_len: 1_400_000,
                mime: "video/webm".into(),
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                source_key: 0xA11CE,
                seq: 7,
                data: vec![0xEB, 0x4D, 0x42],
            }),
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::MediaPlay(MediaPlayParams {
                source_key: 0xA11CE,
                position_ms: 15_000,
            }),
        },
        IpcMessage {
            id: 4,
            kind: IpcMessageKind::MediaPause(MediaPauseParams { source_key: 0xA11CE }),
        },
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::MediaSeek(MediaSeekParams {
                source_key: 0xA11CE,
                target_ms: 45_000,
                request_id: 99,
            }),
        },
        IpcMessage {
            id: 6,
            kind: IpcMessageKind::MediaReleaseSource(MediaReleaseSourceParams { source_key: 0xA11CE }),
        },
        IpcMessage {
            id: 7,
            kind: IpcMessageKind::MediaShutdown,
        },
        IpcMessage {
            id: 8,
            kind: IpcMessageKind::MediaSetRate(MediaSetRateParams {
                source_key: 0xA11CE,
                rate: 1.5,
            }),
        },
        IpcMessage {
            id: 9,
            kind: IpcMessageKind::MediaSetLoop(MediaSetLoopParams {
                source_key: 0xA11CE,
                on: true,
            }),
        },
        IpcMessage {
            id: 10,
            kind: IpcMessageKind::MediaSetGain(MediaSetGainParams {
                source_key: 0xA11CE,
                volume: 0.75,
                muted: false,
            }),
        },
        // host → renderer（握手/状态/帧/错误面）
        IpcMessage {
            id: 0,
            kind: IpcMessageKind::MediaReady(MediaReadyParams {
                protocol_version: MEDIA_PROTOCOL_VERSION,
                capabilities: vec!["codec/vp9".into(), "codec/vorbis".into()],
            }),
        },
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                source_key: 0xA11CE,
                duration_ms: Some(60_000),
                width: Some(320),
                height: Some(240),
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::MediaDataAck(MediaDataAckParams {
                source_key: 0xA11CE,
                last_seq: 7,
            }),
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
                source_key: 0xA11CE,
                epoch: 4,
                state: MediaPlaybackState::Playing { media_time_ms: 15_041 },
            }),
        },
        IpcMessage {
            id: 5,
            kind: IpcMessageKind::MediaSeekResult(MediaSeekResultParams {
                request_id: 99,
                resolved_ms: 45_000,
            }),
        },
        IpcMessage {
            id: 11,
            kind: IpcMessageKind::MediaFrame(MediaFrameParams {
                source_key: 0xA11CE,
                epoch: 4,
                pts_ms: 15_041,
                key: false,
                width: 320,
                height: 240,
                rgba: vec![1, 2, 3, 4],
                shm_name: Some("0xA11CE-1207-9".into()),
            }),
        },
        IpcMessage {
            id: 12,
            kind: IpcMessageKind::MediaError(MediaErrorParams {
                source_key: 0xA11CE,
                epoch: 4,
                code: MediaErrorCode::QuotaExceeded,
                message: "source exceeds per-source cap".into(),
            }),
        },
    ]
}

/// 17 变体逐一 bincode 往返且载荷逐字段保真。
#[test]
fn media_message_family_round_trips() {
    for msg in media_messages() {
        let out = roundtrip(msg.clone());
        assert_eq!(out.id, msg.id);
        // 逐字段相等：IpcMessageKind 未 derive PartialEq，比对调试表示。
        assert_eq!(
            format!("{:?}", out.kind),
            format!("{:?}", msg.kind),
            "variant mismatch on id {}",
            msg.id
        );
    }
}

/// 伴生枚举 MediaPlaybackState 四变体覆盖（IF-003 状态面）。
#[test]
fn media_playback_state_variants_round_trip() {
    let states = [
        MediaPlaybackState::Playing { media_time_ms: 1 },
        MediaPlaybackState::Paused { media_time_ms: 2 },
        MediaPlaybackState::Ended,
        MediaPlaybackState::Error {
            code: MediaErrorCode::DecodeFailure,
        },
    ];
    for state in states {
        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
                source_key: 1,
                epoch: 3,
                state,
            }),
        };
        let out = roundtrip(msg);
        let IpcMessageKind::MediaStateUpdate(params) = out.kind else {
            panic!("expected MediaStateUpdate");
        };
        assert_eq!(params.state, state);
    }
}

/// 伴生枚举 MediaErrorCode 五变体覆盖（IF-006 错误面）。
#[test]
fn media_error_code_variants_round_trip() {
    let codes = [
        MediaErrorCode::DecodeFailure,
        MediaErrorCode::SourceCorrupt,
        MediaErrorCode::QuotaExceeded,
        MediaErrorCode::FrameTooLarge,
        MediaErrorCode::HostInternal,
    ];
    for code in codes {
        let msg = IpcMessage {
            id: 7,
            kind: IpcMessageKind::MediaError(MediaErrorParams {
                source_key: 1,
                epoch: 0,
                code,
                message: format!("{code:?}"),
            }),
        };
        let out = roundtrip(msg);
        let IpcMessageKind::MediaError(params) = out.kind else {
            panic!("expected MediaError");
        };
        assert_eq!(params.code, code);
    }
}

/// MediaFrame shm/内联双路径载荷保真（IF-004；关断态 shm_name=None + rgba 非空）。
#[test]
fn media_frame_dual_path_round_trips() {
    let inline = IpcMessage {
        id: 21,
        kind: IpcMessageKind::MediaFrame(MediaFrameParams {
            source_key: 2,
            epoch: 1,
            pts_ms: 33,
            key: true,
            width: 2,
            height: 2,
            rgba: vec![0; 16],
            shm_name: None,
        }),
    };
    let out = roundtrip(inline);
    let IpcMessageKind::MediaFrame(frame) = out.kind else {
        panic!("expected MediaFrame");
    };
    assert_eq!(frame.rgba.len(), 16);
    assert_eq!(frame.shm_name, None);

    let shm = IpcMessage {
        id: 22,
        kind: IpcMessageKind::MediaFrame(MediaFrameParams {
            shm_name: Some("s-1-n".into()),
            rgba: Vec::new(),
            ..out_kind_frame()
        }),
    };
    let out = roundtrip(shm);
    let IpcMessageKind::MediaFrame(frame) = out.kind else {
        panic!("expected MediaFrame");
    };
    assert!(frame.rgba.is_empty());
    assert_eq!(frame.shm_name.as_deref(), Some("s-1-n"));
}

/// MediaFrame 双路径测试的公共字段基准。
fn out_kind_frame() -> MediaFrameParams {
    MediaFrameParams {
        source_key: 2,
        epoch: 1,
        pts_ms: 33,
        key: true,
        width: 2,
        height: 2,
        rgba: Vec::new(),
        shm_name: None,
    }
}

/// 常量不变式：单源配额上限低于 u64 域且为 256MB（对齐 registry MSE_FACE 量级）。
#[allow(clippy::assertions_on_constants)] // 跨常量钉版：任一侧漂移即失败
#[test]
fn media_source_cap_is_256mb() {
    assert_eq!(MEDIA_SOURCE_MAX_BYTES, 256 * 1024 * 1024);
}

/// 常量不变式：媒体 shm 前缀与关断开关命名锁定（`zeroweb-media-` / `ZW_MEDIA_SHM`）。
#[test]
fn media_shm_prefix_and_kill_switch_names() {
    assert_eq!(MEDIA_SHM_PREFIX, "zeroweb-media-");
    // 非 Linux 平台恒 false（IF-004 关断回落内联），不读环境变量。
    #[cfg(not(target_os = "linux"))]
    assert!(!media_shm_enabled());
}

/// 握手协议版本锁定（IF-001；不兼容 wire 变更须递增并同步两端校验）。
#[test]
fn media_protocol_version_is_1() {
    assert_eq!(MEDIA_PROTOCOL_VERSION, 1);
}
