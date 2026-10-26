//! 媒体宿主进程角色入口（方案 C；C2 解码内核 + 自走时钟）。
//!
//! wire format 契约 = `zero_protocol::media` 模块（RFC
//! media-runtime-isolation-spec-rfc §9 IF-001~006）。解码内核在
//! [`kernel::MediaKernel`]（纯函数面，虚拟时钟单测直接驱动）；本 crate 另提供
//! 事件循环 [`run_role_feed`] 与 IO 面 [`RoleIo`]：
//!
//! - 生产（`main.rs`）：stdio 拆两半——stdin 读线程阻塞 recv → mpsc，主线程
//!   `recv_timeout` 等消息（超时即 Idle → 时钟推进）；stdout 写半承载回写。
//! - 测试/内存通道：[`IpcChannel`] 适配（阻塞 recv；消息耗尽 = 断开）——
//!   时钟推进路径不经此适配（kernel 单测覆盖），循环编排由 [`ScriptedIo`]
//!   驱动验证。
//!
//! 事件循环（DESIGN §10 自走时钟）：无播放中源时无限期阻塞等消息；有播放中
//! 源时按内核唤醒截止（音频样本队列排空时刻/纯视频泵量子）等待——超时 Idle
//! 推进时钟，不做固定节拍轮询。

use std::time::Duration;

use zero_protocol::media::{MEDIA_PROTOCOL_VERSION, MediaReadyParams};
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::{IpcChannel, is_disconnected_channel_message};

pub mod kernel;

pub use kernel::MediaKernel;

/// 事件循环单次等待的产出。
pub enum FeedTick {
    /// 收到一条入站消息（Box 抹平变体尺寸差——`IpcMessage` 368B 经值传递）。
    Message(Box<IpcMessage>),
    /// 等待超时（事件循环推进内核时钟）。
    Idle,
    /// 对端断开/通道故障（角色循环结束）。
    Disconnected,
}

/// 事件循环的 IO 面。超时等待是 host 自走时钟的前提（[`IpcChannel`] 的阻塞
/// `recv` 无超时面）：生产 stdio 经读线程 + `mpsc::recv_timeout` 实现
/// （main.rs [`StdioRoleIo`]），内存通道经 [`IpcChannel`] 适配。
pub trait RoleIo {
    /// 等待至多 `timeout` 取一条入站消息；超时返回 [`FeedTick::Idle`]；
    /// 对端断开返回 [`FeedTick::Disconnected`]。
    fn next_message(&mut self, timeout: Duration) -> FeedTick;

    /// 回写一条消息；`false` = 对端断开（角色循环结束）。
    fn send_reply(&mut self, message: IpcMessage) -> bool;
}

/// `IpcChannel` 适配（内存通道/既有通道面）：阻塞 recv 无超时——通道消息序
/// 确定性驱动（推消息即唤醒）；通道耗尽（recv Err）= 断开。空闲推进由
/// kernel 单测直接注入虚拟时钟覆盖，不经本适配（`timeout` 恒被忽略）。
impl<C: IpcChannel> RoleIo for C {
    fn next_message(&mut self, _timeout: Duration) -> FeedTick {
        match self.recv() {
            Ok(message) => FeedTick::Message(Box::new(message)),
            Err(error) => {
                if !is_disconnected_channel_message(&error.to_string()) {
                    tracing::warn!("media-host: receive failed: {error}");
                }
                FeedTick::Disconnected
            }
        }
    }

    fn send_reply(&mut self, message: IpcMessage) -> bool {
        match self.send(message) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!("media-host: response write failed: {error}");
                false
            }
        }
    }
}

/// 角色事件循环：握手首发 → 循环 { 按内核下次唤醒截止等消息 → Message 分发 /
/// Idle 时钟推进 / Disconnected 退出 → 出站 flush }。
///
/// `clock` 注入单调毫秒（生产 Instant；测试脚本钟）。`MediaShutdown` 在循环
/// 截获（内核无终态语义）。
pub fn run_role_feed(io: &mut impl RoleIo, kernel: &mut MediaKernel, clock: &mut dyn FnMut() -> u64) {
    tracing::info!("media-host: ready, sending handshake");
    // IF-001 握手：连接后首发 MediaReady。capabilities 报内核实际路由真值
    //（DC-4 不虚报；feature 门控见 `codec_capabilities`）。
    let ready = IpcMessage {
        id: 0,
        kind: IpcMessageKind::MediaReady(MediaReadyParams {
            protocol_version: MEDIA_PROTOCOL_VERSION,
            capabilities: kernel::codec_capabilities(),
        }),
    };
    if !io.send_reply(ready) {
        tracing::warn!("media-host: handshake write failed");
        return;
    }

    loop {
        let now_ms = clock();
        let timeout = kernel
            .next_wake_ms(now_ms)
            .map(|deadline| Duration::from_millis(deadline.saturating_sub(now_ms)));
        let tick = io.next_message(timeout.unwrap_or(Duration::MAX));

        let mut out: Vec<IpcMessage> = Vec::new();
        match tick {
            FeedTick::Message(message) => {
                if matches!(message.kind, IpcMessageKind::MediaShutdown) {
                    tracing::info!("media-host: shutdown requested");
                    return;
                }
                let now_ms = clock();
                kernel.handle(*message, now_ms, &mut out);
            }
            FeedTick::Idle => {
                let now_ms = clock();
                kernel.advance(now_ms, &mut out);
            }
            FeedTick::Disconnected => {
                tracing::info!("media-host: IPC peer disconnected");
                return;
            }
        }
        // 出站 flush；写失败 = 对端断开（角色循环结束，IF-006 永久错误面）。
        for message in out.drain(..) {
            if !io.send_reply(message) {
                tracing::warn!("media-host: outbound write failed; ending role loop");
                return;
            }
        }
    }
}

/// 运行媒体宿主角色直到 IPC 对端断开或收到 `MediaShutdown`（内存通道/既有
/// 调用方面；时钟取进程启动起的单调毫秒）。生产 stdio 入口见 `main.rs`。
pub fn run_role<C: IpcChannel>(transport: &mut C) {
    let start = std::time::Instant::now();
    run_role_feed(transport, &mut MediaKernel::new(), &mut || {
        start.elapsed().as_millis() as u64
    });
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::kernel::codec_capabilities;
    use zero_protocol::ProtocolError;
    use zero_protocol::media::{
        MEDIA_SOURCE_MAX_BYTES, MediaDataAckParams as Ack, MediaErrorCode as Code, MediaFrameParams,
        MediaStateUpdateParams,
    };

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

    /// 脚本化 IO：入站序交替消息与 Idle 节拍（None = Idle），供事件循环编排
    /// 验证（时钟由脚本钟注入）。
    struct ScriptedIo {
        inbound: VecDeque<Option<IpcMessage>>,
        outbound: Vec<IpcMessage>,
    }

    impl RoleIo for ScriptedIo {
        fn next_message(&mut self, _timeout: Duration) -> FeedTick {
            match self.inbound.pop_front() {
                Some(Some(message)) => FeedTick::Message(Box::new(message)),
                Some(None) => FeedTick::Idle,
                None => FeedTick::Disconnected,
            }
        }

        fn send_reply(&mut self, message: IpcMessage) -> bool {
            self.outbound.push(message);
            true
        }
    }

    fn msg(id: u64, kind: IpcMessageKind) -> IpcMessage {
        IpcMessage { id, kind }
    }

    /// 首条出站必为 MediaReady 握手（IF-001；版本锁定 + 能力真值）。
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
        // C2 内核能力真值（DC-4 不虚报；与 feature 门控同源）。
        assert_eq!(ready.capabilities, codec_capabilities());
        assert!(ready.capabilities.iter().any(|c| c.contains("vp9")));
        // D4 欠申报订正后真值（C4 ANALYSIS §四）——路由面可解的两条目必须在列。
        assert!(
            ready.capabilities.iter().any(|c| c == "audio/ogg; codecs=vorbis")
                && ready.capabilities.iter().any(|c| c == "audio/wav"),
            "ogg-vorbis/wav 欠申报回归"
        );
    }

    /// 空源生命周期：register → SourceReady(None 元数据) → chunk → Ack →
    /// play（字节未齐）→ HostInternal 错误面 → release/shutdown 静默。
    #[test]
    fn empty_source_lifecycle_contract() {
        let mut channel = TestChannel {
            inbound: VecDeque::from([
                msg(
                    1,
                    IpcMessageKind::MediaRegisterSource(zero_protocol::media::MediaRegisterSourceParams {
                        source_key: 11,
                        total_len: 100,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    2,
                    IpcMessageKind::MediaDataChunk(zero_protocol::media::MediaDataChunkParams {
                        source_key: 11,
                        seq: 3,
                        data: vec![0u8; 10],
                    }),
                ),
                // C2：字节未齐（10/100）play → 如实报错误面（不虚起半流）。
                msg(
                    3,
                    IpcMessageKind::MediaPlay(zero_protocol::media::MediaPlayParams {
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

        // 出站序：MediaReady / SourceReady / DataAck / MediaError；release 与
        // shutdown 无回执。
        assert_eq!(channel.outbound.len(), 4, "outbound: {:?}", channel.outbound);
        let kinds: Vec<&IpcMessageKind> = channel.outbound.iter().map(|m| &m.kind).collect();
        assert!(matches!(kinds[0], IpcMessageKind::MediaReady(_)));
        match kinds[1] {
            IpcMessageKind::MediaSourceReady(ready) => {
                assert_eq!(ready.source_key, 11);
                assert_eq!(ready.duration_ms, None, "字节未齐 demux 不可行，元数据如实 None");
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
                assert_eq!(err.code, Code::HostInternal, "字节未齐 play 如实报错误面");
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
                    IpcMessageKind::MediaRegisterSource(zero_protocol::media::MediaRegisterSourceParams {
                        source_key: 22,
                        total_len: MEDIA_SOURCE_MAX_BYTES + 1,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    2,
                    IpcMessageKind::MediaRegisterSource(zero_protocol::media::MediaRegisterSourceParams {
                        source_key: 23,
                        total_len: MEDIA_SOURCE_MAX_BYTES,
                        mime: "video/webm".into(),
                    }),
                ),
                msg(
                    3,
                    IpcMessageKind::MediaDataChunk(zero_protocol::media::MediaDataChunkParams {
                        source_key: 23,
                        seq: 0,
                        data: vec![0u8; (MEDIA_SOURCE_MAX_BYTES + 1) as usize],
                    }),
                ),
                msg(
                    4,
                    IpcMessageKind::MediaDataChunk(zero_protocol::media::MediaDataChunkParams {
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

    /// 事件循环编排：Idle 节拍驱动内核时钟推进（起播后 Idle 产出帧推送；
    /// 无播放源时 Idle 不产出状态面）。
    #[test]
    fn event_loop_idle_ticks_advance_kernel_clock() {
        let bytes = fixture_bytes("sample-webm-vp9.webm");
        let total = bytes.len() as u64;
        let scripted = ScriptedIo {
            inbound: VecDeque::from([
                Some(msg(
                    1,
                    IpcMessageKind::MediaRegisterSource(zero_protocol::media::MediaRegisterSourceParams {
                        source_key: 7,
                        total_len: total,
                        mime: "video/webm".into(),
                    }),
                )),
                Some(msg(
                    2,
                    IpcMessageKind::MediaDataChunk(zero_protocol::media::MediaDataChunkParams {
                        source_key: 7,
                        seq: 0,
                        data: bytes,
                    }),
                )),
                Some(msg(
                    3,
                    IpcMessageKind::MediaPlay(zero_protocol::media::MediaPlayParams {
                        source_key: 7,
                        position_ms: 0,
                    }),
                )),
                // 脚本钟每调用 +50ms：Idle#1 → now 250（首帧 + 状态帧），Idle#2
                // → now 300（心跳未到，无新增）。
                None,
                None,
            ]),
            outbound: Vec::new(),
        };
        let mut tick: u64 = 0;
        let mut io = scripted;
        // 脚本钟：每次调用 +50ms（超时计算与分发/推进各计一次）。
        let mut clock = || {
            tick += 1;
            tick * 50
        };
        run_role_feed(&mut io, &mut MediaKernel::new(), &mut clock);

        let frames: Vec<&MediaFrameParams> = io
            .outbound
            .iter()
            .filter_map(|m| match &m.kind {
                IpcMessageKind::MediaFrame(f) => Some(f),
                _ => None,
            })
            .collect();
        assert!(!frames.is_empty(), "Idle 推进应产出帧，outbound: {:?}", io.outbound);
        assert_eq!(frames[0].source_key, 7);
        assert_eq!(frames[0].epoch, 0);
        // 状态推送：play 即时 Playing + Idle 心跳。
        let states: Vec<&MediaStateUpdateParams> = io
            .outbound
            .iter()
            .filter_map(|m| match &m.kind {
                IpcMessageKind::MediaStateUpdate(s) => Some(s),
                _ => None,
            })
            .collect();
        assert!(
            states
                .iter()
                .any(|s| matches!(s.state, zero_protocol::media::MediaPlaybackState::Playing { .. }))
        );
    }

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // apps/
        p.pop(); // workspace root
        p.push("tests/fixtures/media");
        p.push(name);
        std::fs::read(p).expect("media fixture present")
    }
}
