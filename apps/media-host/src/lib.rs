//! 媒体宿主进程角色入口（方案 C C1 骨架）。
//!
//! wire format 契约 = `zero_protocol::media` 模块（RFC
//! media-runtime-isolation-spec-rfc §9 IF-001~006）。角色循环镜像 `apps/image-decoder`
//! `run_role` 先例：读信封 → 分发 → 回写；对端断开（stdin EOF / broken pipe）退出。
//!
//! C1 边界：只实现握手 + 空源生命周期（register/data-ack/release/shutdown）+ 错误面。
//! 解码内核（zero-media player/audio entry 迁移，携 A1 时钟语义）由 C2 接入——播放
//! 控制（play/pause/seek）在骨架上如实报 `MediaError{HostInternal}`，不虚报状态。

use std::collections::HashMap;

use zero_protocol::media::{
    MEDIA_PROTOCOL_VERSION, MEDIA_SOURCE_MAX_BYTES, MediaDataAckParams, MediaDataChunkParams, MediaErrorCode,
    MediaErrorParams, MediaPlayParams, MediaReadyParams, MediaRegisterSourceParams, MediaSeekParams,
    MediaSetGainParams, MediaSetLoopParams, MediaSetRateParams, MediaSourceReadyParams,
};
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::{IpcChannel, is_disconnected_channel_message};

/// 已登记源的骨架态（C1 无解码内核：字节记账 + 控制面预留）。
struct SourceEntry {
    /// renderer 自报总长（登记时校验过上限）。
    // FIXME(C2): 解码内核接入后由 demux 读取（进度面/元数据比对）。
    #[allow(dead_code)]
    total_len: u64,
    // FIXME(C2): 解码内核按 mime 路由 demuxer。
    #[allow(dead_code)]
    mime: String,
    /// 已确认的最后分片序号（累积回执面）。
    last_seq: u64,
    /// 实收累计字节（信任边界：不信任 total_len 自报值，按实收封顶）。
    received: u64,
    rate: f64,
    loop_playback: bool,
    volume: f64,
    muted: bool,
}

/// 运行媒体宿主角色直到 IPC 对端断开或收到 `MediaShutdown`。
///
/// 传输由平台 bootstrap 提供：桌面走 stdio（`main.rs`），测试注入内存通道。
pub fn run_role<C: IpcChannel>(transport: &mut C) {
    tracing::info!("media-host: ready, sending handshake");
    // IF-001 握手：连接后首发 MediaReady。capabilities 如实报空集——C1 无解码内核，
    // 不虚报 codec 能力（DC-4「能力查询不虚报」）；C2 随内核接 zero-media feature 门控真值。
    let ready = IpcMessage {
        id: 0,
        kind: IpcMessageKind::MediaReady(MediaReadyParams {
            protocol_version: MEDIA_PROTOCOL_VERSION,
            capabilities: Vec::new(),
        }),
    };
    if let Err(error) = transport.send(ready) {
        tracing::warn!("media-host: handshake write failed: {error}");
        return;
    }

    let mut sources: HashMap<u64, SourceEntry> = HashMap::new();
    loop {
        let message: IpcMessage = match transport.recv() {
            Ok(message) => message,
            Err(error) => {
                if is_disconnected_channel_message(&error.to_string()) {
                    tracing::info!("media-host: IPC peer disconnected");
                } else {
                    tracing::warn!("media-host: receive failed: {error}");
                }
                return;
            }
        };

        match message.kind {
            IpcMessageKind::MediaRegisterSource(params) => {
                let MediaRegisterSourceParams {
                    source_key,
                    total_len,
                    mime,
                } = params;
                if total_len > MEDIA_SOURCE_MAX_BYTES {
                    reply_error(
                        transport,
                        message.id,
                        source_key,
                        MediaErrorCode::QuotaExceeded,
                        format!("total_len {total_len} exceeds per-source cap {MEDIA_SOURCE_MAX_BYTES}"),
                    );
                    continue;
                }
                sources.insert(
                    source_key,
                    SourceEntry {
                        total_len,
                        mime,
                        last_seq: 0,
                        received: 0,
                        rate: 1.0,
                        loop_playback: false,
                        volume: 1.0,
                        muted: false,
                    },
                );
                let response = IpcMessage {
                    id: message.id,
                    kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                        source_key,
                        // C1 骨架无 demux：媒体元数据如实未知，C2 随解码内核带回。
                        duration_ms: None,
                        width: None,
                        height: None,
                    }),
                };
                if reply(transport, response) {
                    return;
                }
            }
            IpcMessageKind::MediaDataChunk(params) => {
                let MediaDataChunkParams { source_key, seq, data } = params;
                let failure = match sources.get_mut(&source_key) {
                    None => Some((
                        MediaErrorCode::HostInternal,
                        "data chunk for unknown source".to_string(),
                    )),
                    Some(entry) => {
                        let new_total = entry.received.saturating_add(data.len() as u64);
                        if new_total > MEDIA_SOURCE_MAX_BYTES {
                            // 实收超限：拒绝并摘除源（后续分片走 unknown source 错误面，
                            // 不静默截断——信任边界按实收字节封顶）。
                            sources.remove(&source_key);
                            Some((
                                MediaErrorCode::QuotaExceeded,
                                format!("received {new_total} exceeds per-source cap {MEDIA_SOURCE_MAX_BYTES}"),
                            ))
                        } else {
                            entry.received = new_total;
                            entry.last_seq = seq;
                            None
                        }
                    }
                };
                match failure {
                    Some((code, detail)) => {
                        reply_error(transport, message.id, source_key, code, detail);
                    }
                    None => {
                        let response = IpcMessage {
                            id: message.id,
                            kind: IpcMessageKind::MediaDataAck(MediaDataAckParams {
                                source_key,
                                last_seq: seq,
                            }),
                        };
                        if reply(transport, response) {
                            return;
                        }
                    }
                }
            }
            IpcMessageKind::MediaPlay(params) => {
                // C1 骨架无解码内核：播放控制如实报 HostInternal，不伪造播放状态。
                let MediaPlayParams {
                    source_key,
                    position_ms,
                } = params;
                let detail = if sources.contains_key(&source_key) {
                    format!("decode kernel not wired (C2); play@{position_ms} rejected")
                } else {
                    format!("decode kernel not wired (C2); unknown source {source_key}")
                };
                reply_error(transport, message.id, source_key, MediaErrorCode::HostInternal, detail);
            }
            IpcMessageKind::MediaPause(params) => {
                let source_key = params.source_key;
                reply_error(
                    transport,
                    message.id,
                    source_key,
                    MediaErrorCode::HostInternal,
                    "decode kernel not wired (C2); pause rejected".to_string(),
                );
            }
            IpcMessageKind::MediaSeek(params) => {
                let MediaSeekParams {
                    source_key,
                    target_ms,
                    request_id,
                } = params;
                reply_error(
                    transport,
                    message.id,
                    source_key,
                    MediaErrorCode::HostInternal,
                    format!("decode kernel not wired (C2); seek#{request_id}@{target_ms} rejected"),
                );
            }
            IpcMessageKind::MediaReleaseSource(params) => {
                // fire-and-forget：无回执（IF-002）。
                let removed = sources.remove(&params.source_key);
                tracing::debug!(
                    source_key = params.source_key,
                    found = removed.is_some(),
                    "media-host: source released"
                );
            }
            IpcMessageKind::MediaShutdown => {
                tracing::info!("media-host: shutdown requested");
                return;
            }
            IpcMessageKind::MediaSetRate(params) => {
                let MediaSetRateParams { source_key, rate } = params;
                match sources.get_mut(&source_key) {
                    Some(entry) => entry.rate = rate,
                    None => {
                        reply_error(
                            transport,
                            message.id,
                            source_key,
                            MediaErrorCode::HostInternal,
                            "set_rate for unknown source".to_string(),
                        );
                    }
                }
            }
            IpcMessageKind::MediaSetLoop(params) => {
                let MediaSetLoopParams { source_key, on } = params;
                match sources.get_mut(&source_key) {
                    Some(entry) => entry.loop_playback = on,
                    None => {
                        reply_error(
                            transport,
                            message.id,
                            source_key,
                            MediaErrorCode::HostInternal,
                            "set_loop for unknown source".to_string(),
                        );
                    }
                }
            }
            IpcMessageKind::MediaSetGain(params) => {
                let MediaSetGainParams {
                    source_key,
                    volume,
                    muted,
                } = params;
                match sources.get_mut(&source_key) {
                    Some(entry) => {
                        entry.volume = volume;
                        entry.muted = muted;
                    }
                    None => {
                        reply_error(
                            transport,
                            message.id,
                            source_key,
                            MediaErrorCode::HostInternal,
                            "set_gain for unknown source".to_string(),
                        );
                    }
                }
            }
            _ => tracing::warn!("media-host: ignored unsupported IPC message"),
        }
    }
}

/// 回写一条消息；对端断开（写失败）时结束角色循环。
fn reply<C: IpcChannel>(transport: &mut C, response: IpcMessage) -> bool {
    match transport.send(response) {
        Ok(()) => false,
        Err(error) => {
            tracing::warn!("media-host: response write failed: {error}");
            true
        }
    }
}

/// 构造并回写 `MediaError`（请求响应面，信封 id 配对）。
fn reply_error<C: IpcChannel>(
    transport: &mut C,
    request_id: u64,
    source_key: u64,
    code: MediaErrorCode,
    message: String,
) -> bool {
    tracing::warn!(source_key, ?code, "media-host: error reply: {message}");
    let response = IpcMessage {
        id: request_id,
        kind: IpcMessageKind::MediaError(MediaErrorParams {
            source_key,
            epoch: 0,
            code,
            message,
        }),
    };
    reply(transport, response)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use zero_protocol::ProtocolError;
    use zero_protocol::media::{MediaDataAckParams as Ack, MediaErrorCode as Code};

    struct TestChannel {
        inbound: VecDeque<IpcMessage>,
        outbound: Vec<IpcMessage>,
    }

    impl IpcChannel for TestChannel {
        fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError> {
            self.outbound.push(message);
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            self.inbound
                .pop_front()
                .ok_or_else(|| ProtocolError::Channel("IPC 通道已关闭".to_string()))
        }

        fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
            Ok(self.inbound.pop_front())
        }

        fn close(&mut self) {}
    }

    fn msg(id: u64, kind: IpcMessageKind) -> IpcMessage {
        IpcMessage { id, kind }
    }

    /// 首条出站必为 MediaReady 握手（IF-001；版本锁定）。
    #[test]
    fn handshake_media_ready_sent_first_with_protocol_version() {
        let mut channel = TestChannel {
            inbound: VecDeque::new(),
            outbound: Vec::new(),
        };
        run_role(&mut channel);
        assert_eq!(channel.outbound.len(), 1);
        let IpcMessageKind::MediaReady(ready) = &channel.outbound[0].kind else {
            panic!("expected MediaReady, got {:?}", channel.outbound[0].kind);
        };
        assert_eq!(ready.protocol_version, MEDIA_PROTOCOL_VERSION);
        // C1 骨架不虚报 codec 能力（DC-4）。
        assert!(ready.capabilities.is_empty());
    }

    /// 空源生命周期：register → SourceReady(None 元数据) → chunk → Ack → play →
    /// HostInternal 错误面 → release/shutdown 静默。
    #[test]
    fn empty_source_lifecycle_contract() {
        let mut channel = TestChannel {
            inbound: VecDeque::from([
                msg(
                    1,
                    IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                        source_key: 11,
                        total_len: 100,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    2,
                    IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                        source_key: 11,
                        seq: 3,
                        data: vec![0u8; 10],
                    }),
                ),
                msg(
                    3,
                    IpcMessageKind::MediaPlay(MediaPlayParams {
                        source_key: 11,
                        position_ms: 0,
                    }),
                ),
                msg(
                    4,
                    IpcMessageKind::MediaReleaseSource(zero_protocol::media::MediaReleaseSourceParams {
                        source_key: 11,
                    }),
                ),
                msg(5, IpcMessageKind::MediaShutdown),
            ]),
            outbound: Vec::new(),
        };
        run_role(&mut channel);

        // 出站序：MediaReady / SourceReady / DataAck / MediaError；release 与 shutdown 无回执。
        assert_eq!(channel.outbound.len(), 4, "outbound: {:?}", channel.outbound);
        let kinds: Vec<&IpcMessageKind> = channel.outbound.iter().map(|m| &m.kind).collect();
        assert!(matches!(kinds[0], IpcMessageKind::MediaReady(_)));
        match kinds[1] {
            IpcMessageKind::MediaSourceReady(ready) => {
                assert_eq!(ready.source_key, 11);
                assert_eq!(ready.duration_ms, None, "C1 骨架无 demux，元数据如实 None");
                assert_eq!(ready.width, None);
                assert_eq!(ready.height, None);
            }
            other => panic!("expected MediaSourceReady, got {other:?}"),
        }
        match kinds[2] {
            IpcMessageKind::MediaDataAck(Ack { source_key, last_seq }) => {
                assert_eq!((*source_key, *last_seq), (11, 3), "累积回执按 seq");
            }
            other => panic!("expected MediaDataAck, got {other:?}"),
        }
        match kinds[3] {
            IpcMessageKind::MediaError(err) => {
                assert_eq!(err.code, Code::HostInternal, "C1 骨架 play 如实报错误面");
                assert_eq!(err.source_key, 11);
            }
            other => panic!("expected MediaError, got {other:?}"),
        }
        // 信封 id 配对（请求/响应以 id 匹配，IF-002）。
        assert_eq!(channel.outbound[1].id, 1);
        assert_eq!(channel.outbound[2].id, 2);
        assert_eq!(channel.outbound[3].id, 3);
    }

    /// 配额面：登记超限 → QuotaExceeded；实收超限 → QuotaExceeded 且源摘除。
    #[test]
    fn quota_exceeded_on_register_and_on_actual_bytes() {
        let mut channel = TestChannel {
            inbound: VecDeque::from([
                msg(
                    1,
                    IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                        source_key: 22,
                        total_len: MEDIA_SOURCE_MAX_BYTES + 1,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    2,
                    IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                        source_key: 23,
                        total_len: MEDIA_SOURCE_MAX_BYTES,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    3,
                    IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                        source_key: 23,
                        seq: 0,
                        data: vec![0u8; (MEDIA_SOURCE_MAX_BYTES + 1) as usize],
                    }),
                ),
                msg(
                    4,
                    IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                        source_key: 23,
                        seq: 1,
                        data: vec![0u8; 8],
                    }),
                ),
                msg(5, IpcMessageKind::MediaShutdown),
            ]),
            outbound: Vec::new(),
        };
        run_role(&mut channel);

        let errors: Vec<Code> = channel
            .outbound
            .iter()
            .filter_map(|m| match &m.kind {
                IpcMessageKind::MediaError(err) => Some(err.code),
                _ => None,
            })
            .collect();
        assert_eq!(
            errors,
            vec![Code::QuotaExceeded, Code::QuotaExceeded, Code::HostInternal],
            "登记超限/实收超限/摘除后 unknown source，实际出站: {:?}",
            channel.outbound
        );
    }

    /// 对端断开（空通道 = recv 失败）时角色循环退出，不 panic。
    #[test]
    fn role_exits_on_peer_disconnect() {
        let mut channel = TestChannel {
            inbound: VecDeque::new(),
            outbound: Vec::new(),
        };
        run_role(&mut channel);
        assert_eq!(channel.outbound.len(), 1, "仅握手");
    }
}
