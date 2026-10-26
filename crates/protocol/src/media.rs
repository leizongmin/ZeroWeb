//! 媒体 host 协议消息族（media-runtime-isolation 方案 C，C1 批次）。
//!
//! 定义 renderer ↔ `zero-media-host` 独立进程的 wire format（RFC
//! [media-runtime-isolation-spec-rfc](../../../docs/specs/media-runtime-isolation-spec-rfc.md)
//! §9 IF-001/002/003/004/005/006 契约的实现定义区）。全部消息走既有
//! [`crate::IpcMessage`] 信封 + bincode over `PipeTransport`（4B LE 长度前缀，
//! 单帧受 64MiB 帧上限约束——`MediaDataChunk` 上限内分片，`MediaFrame` 超限
//! → `MediaError{FrameTooLarge}`）；请求/响应以信封 `id` 配对（`ImageDecodeParams`
//! 先例），fire-and-forget 仅限无回执面（`MediaReleaseSource`/`MediaShutdown`）。
//!
//! 纪律：`IpcMessageKind` 变体只能 append 在枚举末尾（bincode 1 按声明序编码
//! 判别值，跨版本 peer 静默错位风险——见 message.rs 纪律注释）。

use serde::{Deserialize, Serialize};

/// 媒体 host 协议版本（IF-001 握手）。
///
/// host 连接后首发 `MediaReady{protocol_version, capabilities}`；renderer 校验
/// 版本一致，不符按 spawn 失败处理（永久错误面，不内联回落）。不兼容 wire
/// 变更递增本值。
pub const MEDIA_PROTOCOL_VERSION: u32 = 1;

/// 媒体帧 shm 段名前缀（IF-004；镜像 compositor `zeroweb-cmp-` 先例）。
///
/// Linux 上帧像素经 `/dev/shm/zeroweb-media-*` 传递，IPC 消息只带段名；
/// `ZW_MEDIA_SHM=0` 关断或非 Linux 回落内联 `rgba`。段生命周期随 C2 帧路径接入。
pub const MEDIA_SHM_PREFIX: &str = "zeroweb-media-";

/// media host 单源字节上限（IF-002 准入；对齐 registry 侧 256MB 既有量级）。
///
/// `MediaRegisterSource.total_len` 超限 → `MediaError{QuotaExceeded}`；喂入累计
/// 字节超限同样拒绝（total_len 是 renderer 自报值，host 按实收字节封顶）。
pub const MEDIA_SOURCE_MAX_BYTES: u64 = 256 * 1024 * 1024;

/// 是否启用媒体帧 shm 传输（Linux 默认开；`ZW_MEDIA_SHM=0` 禁用）。
pub fn media_shm_enabled() -> bool {
    crate::frame_shm::env_linux_default_on("ZW_MEDIA_SHM")
}

/// 媒体播放状态（IF-003；host 权威时钟的镜像状态值）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MediaPlaybackState {
    /// 播放中。
    Playing {
        /// host 媒体时钟真值（毫秒；audio cursor 主时钟）。
        media_time_ms: u64,
    },
    /// 暂停（时钟冻结）。
    Paused {
        /// 冻结时刻的媒体时钟真值（毫秒）。
        media_time_ms: u64,
    },
    /// 流末（ended）。
    Ended,
    /// 播放错误。
    Error {
        /// 具体错误码。
        code: MediaErrorCode,
    },
}

/// 媒体错误码（IF-006）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaErrorCode {
    /// 解码失败（codec 层）。
    DecodeFailure,
    /// 源字节损坏（容器/码流非法）。
    SourceCorrupt,
    /// 超出字节配额（[`MEDIA_SOURCE_MAX_BYTES`]；对齐 MSE 封顶语义）。
    QuotaExceeded,
    /// 单帧超过 64MiB 帧上限（4K RGBA ≈33MB 在界内）。
    FrameTooLarge,
    /// host 内部错误（spawn 失败/握手失败/解码内核缺失/协议违例）。
    HostInternal,
}

// ── 控制面（renderer → host，IF-002）────────────────────────────

/// 登记媒体源（renderer → host）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRegisterSourceParams {
    /// 源键（registry 键，去 query/fragment；IF-001 寻址面）。
    pub source_key: u64,
    /// 源总字节数（renderer 自报；超 [`MEDIA_SOURCE_MAX_BYTES`] 拒绝）。
    pub total_len: u64,
    /// 源 MIME（如 video/webm、audio/ogg）。
    pub mime: String,
}

/// 源字节流分片（renderer → host；64MiB 帧上限内分片）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDataChunkParams {
    /// 源键。
    pub source_key: u64,
    /// 分片序号（源内从 0 单调递增；回执按 seq 累积确认）。
    pub seq: u64,
    /// 分片字节。
    pub data: Vec<u8>,
}

/// 起播/重播（renderer → host；Ended 后重播 = 重头，registry 现语义）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPlayParams {
    /// 源键。
    pub source_key: u64,
    /// 起播位置（毫秒）。
    pub position_ms: u64,
}

/// 暂停（renderer → host；host 时钟冻结）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaPauseParams {
    /// 源键。
    pub source_key: u64,
}

/// 精确 seek（renderer → host；cue 路径语义跨进程保持，M1 §3.4 契约）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaSeekParams {
    /// 源键。
    pub source_key: u64,
    /// 目标位置（毫秒）。
    pub target_ms: u64,
    /// 请求 ID（`MediaSeekResult` 回带匹配；`request_id` 先例同构）。
    pub request_id: u64,
}

/// 释放源与播放器（renderer → host；导航/`release` 面）。
///
/// fire-and-forget：无回执（IF-002 既定）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaReleaseSourceParams {
    /// 源键。
    pub source_key: u64,
}

/// 设置播放速率（renderer → host；宿主音频消费与时钟缩放的物理面）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaSetRateParams {
    /// 源键。
    pub source_key: u64,
    /// 播放速率（1.0 = 常速）。
    pub rate: f64,
}

/// 设置循环（renderer → host；音频流末回卷随内核入宿主）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaSetLoopParams {
    /// 源键。
    pub source_key: u64,
    /// 是否循环。
    pub on: bool,
}

/// 设置增益/静音（renderer → host；宿主持音频输出面时生效）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaSetGainParams {
    /// 源键。
    pub source_key: u64,
    /// 音量（0.0–1.0+）。
    pub volume: f64,
    /// 是否静音。
    pub muted: bool,
}

// ── host → renderer（握手/状态/帧/错误面，IF-001/003/004/006）──

/// 握手（host → renderer；连接后首发）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaReadyParams {
    /// 协议版本（[`MEDIA_PROTOCOL_VERSION`]；不符按 spawn 失败处理）。
    pub protocol_version: u32,
    /// codec 能力真值（与页面 canPlayType 同源，不虚报；随 zero-media feature 门控）。
    pub capabilities: Vec<String>,
}

/// 源就绪（host → renderer；register 的响应）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSourceReadyParams {
    /// 源键。
    pub source_key: u64,
    /// 媒体时长（毫秒；demux settle 后已知，C1 骨架无解码内核为 `None`）。
    pub duration_ms: Option<u64>,
    /// 视频宽（像素；纯音频为 `None`）。
    pub width: Option<u32>,
    /// 视频高（像素；纯音频为 `None`）。
    pub height: Option<u32>,
}

/// 数据回执（host → renderer；按 seq 累积确认）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaDataAckParams {
    /// 源键。
    pub source_key: u64,
    /// 已确认的最后分片序号。
    pub last_seq: u64,
}

/// 状态/时钟推送（host → renderer；DC-2 契约核心）。
///
/// 世代规则：每源持单调 [`MediaStateUpdateParams::epoch`]，seek/replay/换源递增；
/// renderer 侧旧 epoch 的状态或帧一律丢弃（不返回旧代际数据）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaStateUpdateParams {
    /// 源键。
    pub source_key: u64,
    /// 源级代际（单调递增）。
    pub epoch: u64,
    /// 最新状态。
    pub state: MediaPlaybackState,
}

/// seek 结果（host → renderer；与 [`MediaSeekParams::request_id`] 配对）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaSeekResultParams {
    /// 与请求一致的 ID。
    pub request_id: u64,
    /// 实际落点（毫秒；cue 锚定/精确 seek 后的可读位）。
    pub resolved_ms: u64,
}

/// 解码帧（host → renderer；IF-004）。
///
/// 默认内联 `rgba`；Linux shm 传输（默认开，`ZW_MEDIA_SHM=0` 关断）时
/// `shm_name` 非空且 `rgba` 为空，像素在 `/dev/shm/zeroweb-media-{shm_name}`。
/// 单帧超 64MiB 帧上限 → `MediaError{FrameTooLarge}`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFrameParams {
    /// 源键。
    pub source_key: u64,
    /// 源级代际（旧代际帧 renderer 侧丢弃）。
    pub epoch: u64,
    /// 帧演示时间戳（毫秒；自带时钟信息，`media_time_ms` 随帧推）。
    pub pts_ms: u64,
    /// 是否关键帧。
    pub key: bool,
    /// 帧宽（像素）。
    pub width: u32,
    /// 帧高（像素）。
    pub height: u32,
    /// RGBA 像素（width × height × 4；shm 传输时为空）。
    pub rgba: Vec<u8>,
    /// POSIX shm 段名（不含 [`MEDIA_SHM_PREFIX`] 前缀）；`None` = 内联 `rgba`。
    pub shm_name: Option<String>,
}

/// 媒体错误（host → renderer；IF-006）。
///
/// 既是请求的响应面（quota/解码失败），也是故障自发推送面（C3 崩溃检测 →
/// 全部活跃源 `MediaError{HostInternal}`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaErrorParams {
    /// 源键（会话级故障无源关联时为 0）。
    pub source_key: u64,
    /// 源级代际（未建立播放代际时为 0）。
    pub epoch: u64,
    /// 错误码。
    pub code: MediaErrorCode,
    /// 人类可读诊断（含 host stderr tail 摘要，C3 面）。
    pub message: String,
}
