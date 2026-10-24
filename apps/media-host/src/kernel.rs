//! 媒体解码内核 — host 权威时钟 + 播放控制 + 推送面（media-runtime-isolation C2）。
//!
//! 播放语义与 registry（`zero-webview/video_registry.rs`）同构：A/V pair 走
//! audio clock 主时钟（伴生轨游标驱动视频 `sync_to_media_time`），纯视频走墙钟
//! tick，纯音频走 `AudioEntry` 游标；seek/replay/loop 的轨重建链同序。同构代码
//! 已随 C2 第 1 步迁入 `zero-media::pipeline`——两侧引用同一 A1 时钟实现，
//! C4 三态配对不因副本漂移失真。
//!
//! 内核为纯函数面：`handle`/`advance` 注入单调时钟、把出站消息追加到调用方
//! 向量，不做任何 IO——事件循环（lib.rs）与 stdio 传输（main.rs）分立，单测
//! 以虚拟时钟直接驱动。推送策略（DESIGN §10）：状态迁移即时推
//! `MediaStateUpdate`；播放中 media_time 以帧 pts 携带，音频面/帧间隙以 250ms
//! 心跳补充（shim timeupdate 节流同款，renderer march 节奏不变）；帧内联
//! RGBA 推 `MediaFrame`（shm 传输面归后续切片）。

use std::collections::HashMap;

use zero_media::{
    AudioDecoder, AudioEntry, AudioFormat, AudioSink, AudioStreamDecoder, BatchSampleQueue, DecodedVideoFrame,
    PlayerState, VideoClock, VideoPlayer, VideoTrackDecoder, WebmAudioEntry, WebmAudioTrackKind, open_ogg_opus,
    open_webm_audio_track, open_webm_opus_audio_track,
};
use zero_protocol::media::{
    MEDIA_SOURCE_MAX_BYTES, MediaDataAckParams, MediaDataChunkParams, MediaErrorCode, MediaErrorParams,
    MediaFrameParams, MediaPlayParams, MediaPlaybackState, MediaRegisterSourceParams, MediaSeekParams,
    MediaSeekResultParams, MediaSetGainParams, MediaSetLoopParams, MediaSetRateParams, MediaSourceReadyParams,
    MediaStateUpdateParams,
};
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::MAX_FRAME_SIZE;

/// 播放中 media_time 心跳节流（毫秒）——shim timeupdate 250ms 节流同款。
pub(crate) const HEARTBEAT_MS: u64 = 250;

/// 音频队列为空时的解码补给唤醒量子（毫秒）——解码批 ≈23ms@44.1kHz，8ms
/// 补给保持队列不空腹（追赶区清队后的重建窗口同量子）。
pub(crate) const AUDIO_FEED_QUANTUM_MS: u64 = 8;

/// 纯视频墙钟 tick 唤醒量子（毫秒）——rAF 泵节拍等价。播放器无「下一未呈现
/// 帧 pts」查询面（`presented_pts_ms` 仅回看），精确帧截止唤醒需 player API
/// 扩展；本量子只覆盖纯视频源（A/V 与纯音频源走音频样本截止，见
/// [`MediaKernel::next_wake_ms`]）。// FIXME(C3 后): player 暴露 next_pts_ms 后改精确截止。
pub(crate) const VIDEO_PUMP_QUANTUM_MS: u64 = 16;

/// play 授权位置与游标的容差（毫秒）——renderer 以秒换算毫秒存在舍入，±1ms
/// 内不触发 seek 链（避免代际churn）。
pub(crate) const PLAY_POSITION_TOLERANCE_MS: u64 = 1;

/// host 侧单源运行态（解码器 + 控制面 + 时钟锚点；registry 条目同构）。
struct HostSource {
    /// 源字节（解码器/伴生轨重建所需——registry `sources`/`av_sources` 留存
    /// 先例）。host 恒留存：seek/replay/loop 的轨重建随时要读（registry 单轨
    /// video play 后移除字节的内存优化不复刻——host 无 settle 再喂路径）。
    bytes: Vec<u8>,
    /// 源级代际（DC-2：seek/replay/回卷递增；renderer 侧旧代际状态与帧丢弃）。
    epoch: u64,
    /// 视频播放器（纯视频 + A/V pair；play 时懒建）。
    video: Option<VideoPlayer>,
    /// 纯音频条目（webm 双轨探针与 mp4 AAC 探针均失败的源——mp3/ogg 等）。
    audio: Option<AudioEntry>,
    /// A/V pair 伴生音频条目（主时钟）。
    av_audio: Option<WebmAudioEntry>,
    /// renderer 自报总长（登记时校验过上限）。
    total_len: u64,
    /// 实收累计字节（信任边界：不信任 total_len 自报值，按实收封顶）。
    received: u64,
    /// 已确认的最后分片序号（累积回执面）。
    last_seq: u64,
    rate: f64,
    loop_playback: bool,
    volume: f64,
    muted: bool,
    /// 源 MIME（登记自报；解码路由按容器嗅探，mime 仅为诊断/能力比对面）。
    // FIXME(C4): 与握手 capabilities 做 canPlayType 比对。
    #[allow(dead_code)]
    mime: String,
    /// 容器声明时长（建解码器时探针缓存；`MediaSourceReady` 推送真值）。
    duration_ms: Option<u64>,
    /// 首帧尺寸（建解码器时探针缓存；纯音频为 None）。
    dims: Option<(u32, u32)>,
    /// 最近一次向 renderer 推送 Playing 状态的墙钟（心跳节流锚点）。
    last_push_ms: Option<u64>,
}

impl HostSource {
    /// 当前媒体时钟读数（毫秒；audio clock 主时钟优先——registry current_time
    /// 同构）。
    fn media_time_ms(&self) -> u64 {
        if let Some(av) = self.av_audio.as_ref() {
            return av.cursor_ms;
        }
        if let Some(audio) = self.audio.as_ref() {
            return audio.cursor_ms;
        }
        self.video
            .as_ref()
            .map(|p| (p.current_time() * 1000.0) as u64)
            .unwrap_or(0)
    }

    fn is_playing(&self) -> bool {
        self.video.as_ref().is_some_and(VideoPlayer::is_playing)
            || self.audio.as_ref().is_some_and(|a| a.playing)
            || self.av_audio.as_ref().is_some_and(|a| a.playing)
    }

    /// 懒建解码器（registry play 同构）：`open_media` 视频 + 独立探针读元数据 +
    /// 伴生轨链（A_VORBIS → A_OPUS → mp4/AAC）；视频失败回落纯音频链
    /// （symphonia → ogg opus）。返回是否成功。
    fn build_decoders(&mut self) -> bool {
        let bytes = self.bytes.clone();
        if let Ok(decoder) = VideoTrackDecoder::open_media(&bytes) {
            // 探针：独立解码器读容器声明时长 + 首帧尺寸（registry play 同构；
            // 不消费播放流首帧）。
            if let Ok(mut probe) = VideoTrackDecoder::open_media(&bytes) {
                self.duration_ms = probe.duration_ms();
                if let Ok(Some(frame)) = probe.next_frame() {
                    self.dims = Some((frame.width, frame.height));
                }
            }
            // 伴生轨链与 registry play 懒建同序：A_VORBIS 优先、A_OPUS 次之、
            // mp4/AAC 兜底（symphonia probe——非 mp4 或无音轨静默跳过）。
            let audio_track = match open_webm_audio_track(&bytes) {
                Ok(track) => Some(WebmAudioTrackKind::Vorbis(Box::new(track))),
                Err(_) => open_webm_opus_audio_track(&bytes)
                    .ok()
                    .map(|t| WebmAudioTrackKind::Opus(Box::new(t)))
                    .or_else(|| {
                        AudioDecoder::open(&bytes)
                            .ok()
                            .map(|t| WebmAudioTrackKind::Mp4Aac(Box::new(t)))
                    }),
            };
            if let Some(track) = audio_track {
                let mut av = WebmAudioEntry::new(track);
                let _ = av.sink.start(AudioFormat {
                    sample_rate: av.track.sample_rate(),
                    channels: av.track.channels(),
                });
                self.av_audio = Some(av);
            }
            self.video = Some(VideoPlayer::new(decoder));
            return true;
        }
        // 纯音频回落链（registry register_audio_source 双面序同构：先 symphonia，
        // 面外再 ogg opus）。
        let decoder = match AudioDecoder::open(&bytes) {
            Ok(d) => AudioStreamDecoder::Symphonia(d),
            Err(_) => match open_ogg_opus(&bytes) {
                Ok(t) => AudioStreamDecoder::Opus(Box::new(t)),
                Err(_) => return false,
            },
        };
        self.audio = Some(AudioEntry::new(decoder));
        // 建前控制面随构建生效（registry 条目登记即存控制态，host 懒建——
        // set_loop/set_gain 先于 play 到达时不丢面）。
        self.apply_stored_controls();
        true
    }

    /// 把源控制面真值（loop/volume/muted）套到已建解码条目（懒建对齐面；
    /// registry 条目存活期 set_* 即时生效 + 登记即存，两条路径在此合流）。
    fn apply_stored_controls(&mut self) {
        let volume = (self.volume as f32).clamp(0.0, 1.0);
        if let Some(audio) = self.audio.as_mut() {
            audio.loop_on = self.loop_playback;
            audio.volume = volume;
            audio.muted = self.muted;
        }
        if let Some(av) = self.av_audio.as_mut() {
            av.loop_on = self.loop_playback;
            av.volume = volume;
            av.muted = self.muted;
        }
    }

    /// seek 链（registry seek 同构）：视频解码器内精确定位；伴生轨/纯音频轨重建
    /// + 追赶区静默线 + 清队。返回实际落点（毫秒）；失败返回 None（错误面归调用方）。
    fn seek_to(&mut self, target_ms: u64) -> Option<u64> {
        if let Some(player) = self.video.as_mut() {
            player.seek_to_ms(target_ms).ok()?;
            // player 内已 clamp 到 [0, duration]（registry seek 同面）。
            let resolved = (player.current_time() * 1000.0) as u64;
            if let Some(av) = self.av_audio.as_mut()
                && let Some(track) = rebuild_webm_audio_track_full(&self.bytes)
            {
                let (rate, channels) = (track.sample_rate(), track.channels());
                av.track = track;
                let _ = av.sink.start(AudioFormat {
                    sample_rate: rate,
                    channels,
                });
                // 游标对齐视频落点（audio clock 主时钟——registry seek 同构）；
                // 追赶区静默线 + 清队（轨重建 = 流位重置）。
                av.cursor_ms = resolved;
                av.skip_until_ms = resolved;
                av.last_tick_ms = None;
                av.queue.clear();
            }
            return Some(resolved);
        }
        if let Some(audio) = self.audio.as_mut() {
            // 纯音频 seek：解码器单向流 → 重建（registry audio_seek 同构；
            // 双面回落序与登记同）。
            let bytes = self.bytes.clone();
            let decoder = match AudioDecoder::open(&bytes) {
                Ok(d) => AudioStreamDecoder::Symphonia(d),
                Err(_) => match open_ogg_opus(&bytes) {
                    Ok(t) => AudioStreamDecoder::Opus(Box::new(t)),
                    Err(_) => return None,
                },
            };
            let (rate, channels) = (decoder.sample_rate(), decoder.channels());
            audio.decoder = decoder;
            let _ = audio.sink.start(AudioFormat {
                sample_rate: rate,
                channels,
            });
            audio.cursor_ms = target_ms;
            audio.skip_until_ms = target_ms;
            audio.queue.clear();
            return Some(target_ms);
        }
        None
    }

    /// Ended 后重头播放的重建（registry play Ended 分支同构）：视频 reset 新
    /// 解码器；伴生轨重建 + 游标归零；纯音频 `restart`（audio clock 主时钟
    /// 读数随重建归零，复活重播不静默）。
    fn rebuild_for_replay(&mut self) {
        let bytes = self.bytes.clone();
        if let Some(player) = self.video.as_mut()
            && let Ok(decoder) = VideoTrackDecoder::open_media(&bytes)
        {
            player.reset(decoder);
        }
        if let Some(av) = self.av_audio.as_mut()
            && let Some(track) = rebuild_webm_audio_track(&bytes)
        {
            let (rate, channels) = (track.sample_rate(), track.channels());
            av.track = track;
            let _ = av.sink.start(AudioFormat {
                sample_rate: rate,
                channels,
            });
            av.cursor_ms = 0;
            av.skip_until_ms = 0;
            av.last_tick_ms = None;
            av.queue.clear();
        }
        if let Some(audio) = self.audio.as_mut() {
            // AudioEntry::restart 内含双面回落与游标/清队面；重建失败回落停止
            //（错误面照常派 ended）。
            audio.restart(&bytes);
            audio.last_tick_ms = None;
        }
    }

    /// 单源时钟推进（事件循环 Idle 唤醒；registry tick_all 主时钟序同构——
    /// 伴生轨先行、视频对齐游标、纯视频墙钟 tick）。推送帧/终态/心跳面。
    fn advance(&mut self, now_ms: u64, source_key: u64, out: &mut Vec<IpcMessage>) {
        let mut wrapped = false;
        let mut ended_now = false;
        let mut frame_out: Option<DecodedVideoFrame> = None;
        let mut decode_error = false;
        if self.video.as_ref().is_some_and(VideoPlayer::is_playing) {
            match self.av_audio.as_mut() {
                // A/V pair：伴生轨主时钟先行，视频对齐游标（registry tick_all 同构）。
                Some(av) => {
                    if av.playing {
                        let restart = self.bytes.as_slice();
                        // loop 回卷在 advance_to 内静默重建（registry WebmAudioEntry
                        // 同面——无 wrap 观测旗标；A/V 的 ended→loop 分叉由视频面
                        // Ended 推送驱动 renderer march）。
                        av.advance_to(now_ms, Some(restart));
                    }
                    let media_ms = av.cursor_ms as f64;
                    match self
                        .video
                        .as_mut()
                        .expect("A/V 不变量：av_audio 在则 video 在")
                        .sync_to_media_time(media_ms)
                    {
                        Ok(Some(frame)) => frame_out = Some(frame),
                        Ok(None) => {}
                        Err(_) => decode_error = true,
                    }
                    // 视频到流末 = 资源终态：伴生轨同步停走（host 权威状态须
                    // 一致——registry 侧视频 Ended 后伴生轨残走的 shim 可观测
                    // 怪癖不复刻，偏差记录 DESIGN §10.7）。
                    ended_now = self.video.as_ref().expect("A/V 不变量").state() == PlayerState::Ended;
                    if ended_now {
                        // 伴生轨同步停走（本 tick 转换面；此后 is_playing 门不再进）。
                        av.playing = false;
                        av.last_tick_ms = None;
                    }
                }
                None => {
                    // 纯视频：墙钟 tick（registry 纯视频回落同构）。
                    match self.video.as_mut().expect("is_playing 已门").tick(now_ms) {
                        Ok(Some(frame)) => frame_out = Some(frame),
                        Ok(None) => {}
                        Err(_) => decode_error = true,
                    }
                    ended_now = self.video.as_ref().expect("is_playing 已门").state() == PlayerState::Ended;
                }
            }
        } else if let Some(audio) = self.audio.as_mut()
            && audio.playing
        {
            let restart = self.bytes.as_slice();
            audio.advance_to(now_ms, Some(restart));
            if audio.wrap_pending {
                audio.wrap_pending = false;
                wrapped = true;
            }
            ended_now = audio.reached_end;
        }

        // 解码失败终态面（IF-006）：推 Error 状态，播放停止，不内联回落。
        if decode_error {
            tracing::warn!(source_key, "media-host: decode error");
            if let Some(player) = self.video.as_mut() {
                player.pause();
            }
            if let Some(av) = self.av_audio.as_mut() {
                av.playing = false;
                av.last_tick_ms = None;
            }
            out.push(state_message(
                source_key,
                self.epoch,
                MediaPlaybackState::Error {
                    code: MediaErrorCode::DecodeFailure,
                },
            ));
            self.last_push_ms = None;
            return;
        }
        // 帧推送（借用面已收口）。
        if let Some(frame) = frame_out {
            self.push_frame(source_key, frame, out);
        }
        // 代际/状态推送面：回卷 = 新代际（renderer 丢弃旧代际残留帧）。
        if wrapped {
            self.epoch += 1;
            self.last_push_ms = None;
        }
        if ended_now {
            out.push(state_message(source_key, self.epoch, MediaPlaybackState::Ended));
            self.last_push_ms = None;
        } else if self.is_playing()
            && self
                .last_push_ms
                .is_none_or(|t| now_ms.saturating_sub(t) >= HEARTBEAT_MS)
        {
            out.push(state_message(
                source_key,
                self.epoch,
                MediaPlaybackState::Playing {
                    media_time_ms: self.media_time_ms(),
                },
            ));
            self.last_push_ms = Some(now_ms);
        }
    }

    /// 内联帧推送（IF-004：shm 传输面归后续切片，`shm_name` 恒 None）。rgba 超
    /// 传输 64MiB 帧上限 → `MediaError{FrameTooLarge}`（不静默丢帧）。
    fn push_frame(&self, source_key: u64, frame: DecodedVideoFrame, out: &mut Vec<IpcMessage>) {
        if frame.rgba.len() > MAX_FRAME_SIZE {
            // 自发推送面（id 0）：帧超限不是任何请求的响应。
            out.push(error_message(
                0,
                source_key,
                self.epoch,
                MediaErrorCode::FrameTooLarge,
                format!(
                    "frame {}x{} exceeds transport cap {MAX_FRAME_SIZE}",
                    frame.width, frame.height
                ),
            ));
            return;
        }
        out.push(IpcMessage {
            id: 0,
            kind: IpcMessageKind::MediaFrame(MediaFrameParams {
                source_key,
                epoch: self.epoch,
                pts_ms: frame.pts_ms,
                // 解码器未暴露关键帧旗标——恒 false（renderer 镜像面不消费；
                // 旗标随解码器 API 扩展补真值）。
                // FIXME(C4 前): DecodedVideoFrame 暴露 keyframe 旗标后接线。
                key: false,
                width: frame.width,
                height: frame.height,
                rgba: frame.rgba,
                shm_name: None,
            }),
        });
    }
}

/// 音频样本截止：队列排空时刻（当前格式换算，留 2ms 补给余量）；空队/追赶区
/// 清队窗口走补给量子。
fn audio_queue_deadline(queue: &BatchSampleQueue, now_ms: u64) -> u64 {
    match queue.format {
        Some((rate, channels)) if channels > 0 => {
            let frames_left = queue.pending.len() as u64 / u64::from(channels);
            let drain_ms = frames_left * 1000 / u64::from(rate);
            now_ms
                + drain_ms
                    .saturating_sub(2)
                    .max(if drain_ms > 0 { 1 } else { AUDIO_FEED_QUANTUM_MS })
        }
        _ => now_ms + AUDIO_FEED_QUANTUM_MS,
    }
}

/// webm 伴生轨重建（A_VORBIS 优先、A_OPUS 次之——registry replay 重建序同构；
/// seek 面多一档 mp4/AAC 兜底，见 [`rebuild_webm_audio_track_full`]）。
fn rebuild_webm_audio_track(bytes: &[u8]) -> Option<WebmAudioTrackKind> {
    if let Ok(track) = open_webm_audio_track(bytes) {
        return Some(WebmAudioTrackKind::Vorbis(Box::new(track)));
    }
    open_webm_opus_audio_track(bytes)
        .ok()
        .map(|t| WebmAudioTrackKind::Opus(Box::new(t)))
}

/// seek 面的伴生轨重建（registry seek 同构）：A_VORBIS → A_OPUS → mp4/AAC。
fn rebuild_webm_audio_track_full(bytes: &[u8]) -> Option<WebmAudioTrackKind> {
    if let Ok(track) = open_webm_audio_track(bytes) {
        return Some(WebmAudioTrackKind::Vorbis(Box::new(track)));
    }
    if let Ok(track) = open_webm_opus_audio_track(bytes) {
        return Some(WebmAudioTrackKind::Opus(Box::new(track)));
    }
    AudioDecoder::open(bytes)
        .ok()
        .map(|t| WebmAudioTrackKind::Mp4Aac(Box::new(t)))
}

/// 构造 `MediaStateUpdate` 推送信封（id 0 = 非请求响应）。
fn state_message(source_key: u64, epoch: u64, state: MediaPlaybackState) -> IpcMessage {
    IpcMessage {
        id: 0,
        kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
            source_key,
            epoch,
            state,
        }),
    }
}

/// 构造 `MediaError` 信封（IF-002：请求响应面 id 配对——`request_id` 传请求
/// 信封 id；自发推送面传 0）。
fn error_message(request_id: u64, source_key: u64, epoch: u64, code: MediaErrorCode, message: String) -> IpcMessage {
    tracing::warn!(source_key, epoch, ?code, "media-host: error: {message}");
    IpcMessage {
        id: request_id,
        kind: IpcMessageKind::MediaError(MediaErrorParams {
            source_key,
            epoch,
            code,
            message,
        }),
    }
}

/// C2 codec 能力真值（DC-4 不虚报）：与内核实际路由面/feature 门控同源。
pub(crate) fn codec_capabilities() -> Vec<String> {
    let mut caps = vec![
        "video/webm; codecs=vp9".to_string(),
        "audio/webm; codecs=vorbis".to_string(),
        "audio/webm; codecs=opus".to_string(),
        "audio/mpeg".to_string(),
        "audio/ogg; codecs=opus".to_string(),
        "audio/mp4; codecs=aac".to_string(),
    ];
    if cfg!(feature = "decode-av1") {
        caps.push("video/webm; codecs=av1".to_string());
    }
    if cfg!(feature = "decode-h264") {
        caps.push("video/mp4; codecs=h264".to_string());
    }
    caps
}

/// 媒体解码内核 — 源表 + 播放控制 + 时钟推进（纯函数面，见模块文档）。
#[derive(Default)]
pub struct MediaKernel {
    sources: HashMap<u64, HostSource>,
}

impl MediaKernel {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理一条入站控制消息（`now_ms` 单调毫秒；出站消息追加到 `out`）。
    pub fn handle(&mut self, message: IpcMessage, now_ms: u64, out: &mut Vec<IpcMessage>) {
        match message.kind {
            IpcMessageKind::MediaRegisterSource(params) => {
                let MediaRegisterSourceParams {
                    source_key,
                    total_len,
                    mime,
                } = params;
                if total_len > MEDIA_SOURCE_MAX_BYTES {
                    out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::QuotaExceeded,
                        format!("total_len {total_len} exceeds per-source cap {MEDIA_SOURCE_MAX_BYTES}"),
                    ));
                    return;
                }
                self.sources.insert(
                    source_key,
                    HostSource {
                        bytes: Vec::new(),
                        epoch: 0,
                        video: None,
                        audio: None,
                        av_audio: None,
                        total_len,
                        received: 0,
                        last_seq: 0,
                        rate: 1.0,
                        loop_playback: false,
                        volume: 1.0,
                        muted: false,
                        duration_ms: None,
                        dims: None,
                        last_push_ms: None,
                        // 登记即锁 mime：解码路由按容器嗅探（open_media），mime
                        // 仅为诊断/能力比对面（FIXME(C4): canPlayType 比对）。
                        mime,
                    },
                );
                // 字节未齐前 demux 不可行：元数据如实 None（齐后 play 建解码器
                // 时以 SourceReady 推送补真值）。
                out.push(IpcMessage {
                    id: message.id,
                    kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                        source_key,
                        duration_ms: None,
                        width: None,
                        height: None,
                    }),
                });
            }
            IpcMessageKind::MediaDataChunk(params) => {
                let MediaDataChunkParams { source_key, seq, data } = params;
                let failure = match self.sources.get_mut(&source_key) {
                    None => Some((
                        MediaErrorCode::HostInternal,
                        "data chunk for unknown source".to_string(),
                    )),
                    Some(entry) => {
                        let new_total = entry.received.saturating_add(data.len() as u64);
                        if new_total > MEDIA_SOURCE_MAX_BYTES {
                            // 实收超限：拒绝并摘除源（后续分片走 unknown source
                            // 错误面，不静默截断——信任边界按实收字节封顶）。
                            self.sources.remove(&source_key);
                            Some((
                                MediaErrorCode::QuotaExceeded,
                                format!("received {new_total} exceeds per-source cap {MEDIA_SOURCE_MAX_BYTES}"),
                            ))
                        } else {
                            entry.received = new_total;
                            entry.last_seq = seq;
                            entry.bytes.extend_from_slice(&data);
                            None
                        }
                    }
                };
                match failure {
                    Some((code, detail)) => out.push(error_message(message.id, source_key, 0, code, detail)),
                    None => out.push(IpcMessage {
                        id: message.id,
                        kind: IpcMessageKind::MediaDataAck(MediaDataAckParams {
                            source_key,
                            last_seq: seq,
                        }),
                    }),
                }
            }
            IpcMessageKind::MediaPlay(params) => {
                let MediaPlayParams {
                    source_key,
                    position_ms,
                } = params;
                self.play(source_key, position_ms, now_ms, message.id, out);
            }
            IpcMessageKind::MediaPause(params) => {
                let source_key = params.source_key;
                match self.sources.get_mut(&source_key) {
                    None => out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::HostInternal,
                        "pause for unknown source".to_string(),
                    )),
                    Some(entry) => {
                        if let Some(player) = entry.video.as_mut() {
                            player.pause();
                        }
                        if let Some(audio) = entry.audio.as_mut() {
                            audio.playing = false;
                            audio.last_tick_ms = None;
                        }
                        if let Some(av) = entry.av_audio.as_mut() {
                            av.playing = false;
                            av.last_tick_ms = None;
                        }
                        // Paused 回执兼即时状态推送（IF-002：请求响应面 id 配对）。
                        out.push(IpcMessage {
                            id: message.id,
                            kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
                                source_key,
                                epoch: entry.epoch,
                                state: MediaPlaybackState::Paused {
                                    media_time_ms: entry.media_time_ms(),
                                },
                            }),
                        });
                        entry.last_push_ms = None;
                    }
                }
            }
            IpcMessageKind::MediaSeek(params) => {
                let MediaSeekParams {
                    source_key,
                    target_ms,
                    request_id,
                } = params;
                match self.sources.get_mut(&source_key) {
                    None => out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::HostInternal,
                        format!("seek#{request_id} for unknown source {source_key}"),
                    )),
                    Some(entry) if entry.received < entry.total_len => {
                        out.push(error_message(
                            message.id,
                            source_key,
                            entry.epoch,
                            MediaErrorCode::HostInternal,
                            format!("source data incomplete ({}/{})", entry.received, entry.total_len),
                        ));
                    }
                    Some(entry) => {
                        // HAVE_METADATA 可 seek 面：未建解码器先建（保持暂停——
                        // registry seek 自动建 player 同构）。
                        if entry.video.is_none() && entry.audio.is_none() && !entry.build_decoders() {
                            out.push(error_message(
                                message.id,
                                source_key,
                                entry.epoch,
                                MediaErrorCode::SourceCorrupt,
                                "decoder build failed (unsupported or corrupt source)".to_string(),
                            ));
                            return;
                        }
                        match entry.seek_to(target_ms) {
                            Some(resolved_ms) => {
                                entry.epoch += 1;
                                out.push(IpcMessage {
                                    id: message.id,
                                    kind: IpcMessageKind::MediaSeekResult(MediaSeekResultParams {
                                        request_id,
                                        resolved_ms,
                                    }),
                                });
                                // seek 即时状态推送（新代际首推；播放态保持——
                                // registry seek 不改 paused/playing 同面）。
                                let state = if entry.is_playing() {
                                    MediaPlaybackState::Playing {
                                        media_time_ms: resolved_ms,
                                    }
                                } else {
                                    MediaPlaybackState::Paused {
                                        media_time_ms: resolved_ms,
                                    }
                                };
                                out.push(state_message(source_key, entry.epoch, state));
                                entry.last_push_ms = Some(now_ms);
                            }
                            None => out.push(error_message(
                                message.id,
                                source_key,
                                entry.epoch,
                                MediaErrorCode::HostInternal,
                                format!("seek#{request_id}@{target_ms} failed (decoder seek error)"),
                            )),
                        }
                    }
                }
            }
            IpcMessageKind::MediaSetRate(params) => {
                let MediaSetRateParams { source_key, rate } = params;
                match self.sources.get_mut(&source_key) {
                    Some(entry) => {
                        entry.rate = rate;
                        // 速率只作用于视频墙钟（registry set_playback_rate 同面；
                        // 音频游标按墙钟走——registry 已知限制，不在此扩面）。
                        if let Some(player) = entry.video.as_mut() {
                            player.set_playback_rate(rate);
                        }
                    }
                    None => out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::HostInternal,
                        "set_rate for unknown source".to_string(),
                    )),
                }
            }
            IpcMessageKind::MediaSetLoop(params) => {
                let MediaSetLoopParams { source_key, on } = params;
                match self.sources.get_mut(&source_key) {
                    Some(entry) => {
                        entry.loop_playback = on;
                        if let Some(audio) = entry.audio.as_mut() {
                            audio.loop_on = on;
                        }
                        if let Some(av) = entry.av_audio.as_mut() {
                            av.loop_on = on;
                        }
                    }
                    None => out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::HostInternal,
                        "set_loop for unknown source".to_string(),
                    )),
                }
            }
            IpcMessageKind::MediaSetGain(params) => {
                let MediaSetGainParams {
                    source_key,
                    volume,
                    muted,
                } = params;
                match self.sources.get_mut(&source_key) {
                    Some(entry) => {
                        entry.volume = volume;
                        entry.muted = muted;
                        let clamped = volume as f32;
                        if let Some(audio) = entry.audio.as_mut() {
                            audio.volume = clamped.clamp(0.0, 1.0);
                            audio.muted = muted;
                        }
                        if let Some(av) = entry.av_audio.as_mut() {
                            av.volume = clamped.clamp(0.0, 1.0);
                            av.muted = muted;
                        }
                    }
                    None => out.push(error_message(
                        message.id,
                        source_key,
                        0,
                        MediaErrorCode::HostInternal,
                        "set_gain for unknown source".to_string(),
                    )),
                }
            }
            IpcMessageKind::MediaReleaseSource(params) => {
                // fire-and-forget：无回执（IF-002）。
                let removed = self.sources.remove(&params.source_key);
                tracing::debug!(
                    source_key = params.source_key,
                    found = removed.is_some(),
                    "media-host: source released"
                );
            }
            // MediaShutdown 在事件循环截获（lib.rs run_role_feed）。
            _ => tracing::warn!("media-host: ignored unsupported IPC message"),
        }
    }

    /// 起播/重播（registry play 同构 + wire position 授权位）：字节齐 → 懒建
    /// 解码器 → replay 重建 → 位置对齐 → 起播 → 即时 Playing 推送。
    fn play(&mut self, source_key: u64, position_ms: u64, now_ms: u64, request_id: u64, out: &mut Vec<IpcMessage>) {
        let Some(entry) = self.sources.get_mut(&source_key) else {
            out.push(error_message(
                request_id,
                source_key,
                0,
                MediaErrorCode::HostInternal,
                format!("play for unknown source {source_key}"),
            ));
            return;
        };
        // 数据完整性面：play 要求字节喂完（settle 先于 play 的代理侧契约）；
        // 半流起播 demux 失败如实报错，不虚起。
        if entry.received < entry.total_len {
            out.push(error_message(
                request_id,
                source_key,
                entry.epoch,
                MediaErrorCode::HostInternal,
                format!("source data incomplete ({}/{})", entry.received, entry.total_len),
            ));
            return;
        }
        let freshly_built = entry.video.is_none() && entry.audio.is_none();
        if freshly_built && !entry.build_decoders() {
            out.push(error_message(
                request_id,
                source_key,
                entry.epoch,
                MediaErrorCode::SourceCorrupt,
                "decoder build failed (unsupported or corrupt source)".to_string(),
            ));
            return;
        }
        if freshly_built {
            // 元数据真值推送（建解码器后首推；renderer duration/尺寸升级面）。
            out.push(IpcMessage {
                id: 0,
                kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                    source_key,
                    duration_ms: entry.duration_ms,
                    width: entry.dims.map(|d| d.0),
                    height: entry.dims.map(|d| d.1),
                }),
            });
        }
        // replay 代际：Ended 后 play = 重头（registry Ended→reset 分支同构）。
        let replay = match (&entry.video, entry.audio.as_ref()) {
            (Some(player), _) => player.state() == PlayerState::Ended,
            (None, Some(audio)) => audio.reached_end,
            (None, None) => false,
        };
        if replay {
            entry.epoch += 1;
            entry.rebuild_for_replay();
        }
        // wire 授权位置 ≠ 当前游标（±1ms 舍入容差）→ seek 链（epoch +1）。
        let cursor = entry.media_time_ms();
        if position_ms.abs_diff(cursor) > PLAY_POSITION_TOLERANCE_MS && entry.seek_to(position_ms).is_none() {
            out.push(error_message(
                request_id,
                source_key,
                entry.epoch,
                MediaErrorCode::HostInternal,
                format!("play@{position_ms} seek chain failed"),
            ));
            return;
        }
        // 起播（registry play 同锚点时钟：视频 play + 伴生轨/纯音频同 now 锚）。
        if let Some(player) = entry.video.as_mut() {
            player.play(now_ms);
        }
        if let Some(audio) = entry.audio.as_mut() {
            audio.playing = true;
            audio.reached_end = false;
            audio.last_tick_ms = Some(now_ms);
        }
        if let Some(av) = entry.av_audio.as_mut() {
            av.playing = true;
            av.last_tick_ms = Some(now_ms);
        }
        let media_time = entry.media_time_ms();
        // 即时 Playing 推送兼作 play 回执（IF-002：请求响应面 id 配对；后续
        // 心跳/终态为自发推送面 id 0）。
        out.push(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
                source_key,
                epoch: entry.epoch,
                state: MediaPlaybackState::Playing {
                    media_time_ms: media_time,
                },
            }),
        });
        entry.last_push_ms = Some(now_ms);
    }

    /// 时钟推进：所有播放中源按实时节奏解码/产出帧 + 心跳与终态推送（事件循环
    /// Idle 唤醒时调用；`now_ms` 由事件循环注入）。
    pub fn advance(&mut self, now_ms: u64, out: &mut Vec<IpcMessage>) {
        let keys: Vec<u64> = self.sources.keys().copied().collect();
        for key in keys {
            if let Some(entry) = self.sources.get_mut(&key) {
                entry.advance(now_ms, key, out);
            }
        }
    }

    /// 事件循环下次唤醒时刻（毫秒墙钟；None = 无播放中源，可无限期阻塞等消息）。
    pub fn next_wake_ms(&self, now_ms: u64) -> Option<u64> {
        let mut wake: Option<u64> = None;
        for entry in self.sources.values() {
            if !entry.is_playing() {
                continue;
            }
            let deadline = match entry.av_audio.as_ref() {
                Some(av) => audio_queue_deadline(&av.queue, now_ms),
                None => match entry.audio.as_ref() {
                    Some(audio) => audio_queue_deadline(&audio.queue, now_ms),
                    None => now_ms + VIDEO_PUMP_QUANTUM_MS,
                },
            };
            if wake.is_none_or(|w| deadline < w) {
                wake = Some(deadline);
            }
        }
        wake
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::media::MediaPauseParams;

    /// 事件循环外核直驱的测试面：register → 整源喂入 → play（虚拟时钟）。
    struct Driven {
        kernel: MediaKernel,
        out: Vec<IpcMessage>,
    }

    impl Driven {
        fn new(name: &str, key: u64) -> Self {
            let bytes = fixture_bytes(name);
            let mut d = Self {
                kernel: MediaKernel::new(),
                out: Vec::new(),
            };
            d.handle(
                msg(
                    1,
                    IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                        source_key: key,
                        total_len: bytes.len() as u64,
                        mime: "video/webm".into(),
                    }),
                ),
                0,
            );
            d.handle(
                msg(
                    2,
                    IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                        source_key: key,
                        seq: 0,
                        data: bytes,
                    }),
                ),
                0,
            );
            d
        }

        fn handle(&mut self, message: IpcMessage, now_ms: u64) {
            self.kernel.handle(message, now_ms, &mut self.out);
        }

        fn advance(&mut self, now_ms: u64) {
            self.kernel.advance(now_ms, &mut self.out);
        }

        fn play(&mut self, key: u64, position_ms: u64, now_ms: u64) {
            self.handle(
                msg(
                    3,
                    IpcMessageKind::MediaPlay(MediaPlayParams {
                        source_key: key,
                        position_ms,
                    }),
                ),
                now_ms,
            );
        }

        fn states(&self) -> Vec<(u64, &MediaPlaybackState)> {
            self.out
                .iter()
                .filter_map(|m| match &m.kind {
                    IpcMessageKind::MediaStateUpdate(s) => Some((s.epoch, &s.state)),
                    _ => None,
                })
                .collect()
        }

        fn frames(&self) -> Vec<&MediaFrameParams> {
            self.out
                .iter()
                .filter_map(|m| match &m.kind {
                    IpcMessageKind::MediaFrame(f) => Some(f),
                    _ => None,
                })
                .collect()
        }
    }

    fn msg(id: u64, kind: IpcMessageKind) -> IpcMessage {
        IpcMessage { id, kind }
    }

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // apps/
        p.pop(); // workspace root
        p.push("tests/fixtures/media");
        p.push(name);
        std::fs::read(p).expect("media fixture present")
    }

    fn ended_count(d: &Driven) -> usize {
        d.states()
            .iter()
            .filter(|(_, s)| matches!(s, MediaPlaybackState::Ended))
            .count()
    }

    /// play：字节齐 → 元数据真值推送（SourceReady）→ 即时 Playing 状态（epoch 0）。
    #[test]
    fn play_pushes_metadata_then_playing_state() {
        let mut d = Driven::new("sample-webm-vp9.webm", 7);
        d.play(7, 0, 1000);
        let readies: Vec<&MediaSourceReadyParams> = d
            .out
            .iter()
            .filter_map(|m| match &m.kind {
                IpcMessageKind::MediaSourceReady(r) => Some(r),
                _ => None,
            })
            .collect();
        // 登记 SourceReady（None 元数据）+ play 后元数据真值推送。
        assert_eq!(readies.len(), 2, "outbound: {:?}", d.out);
        assert_eq!((readies[0].duration_ms, readies[0].width), (None, None));
        assert_eq!(readies[1].duration_ms, Some(2000), "容器声明时长真值");
        assert_eq!(readies[1].width, Some(320));
        assert_eq!(readies[1].height, Some(240));
        let states = d.states();
        let (_, state) = states.last().expect("play 即时状态推送");
        assert!(
            matches!(state, MediaPlaybackState::Playing { media_time_ms: 0 }),
            "play 即时 Playing@0，got {state:?}"
        );
    }

    /// 时钟推进：帧推送 pts 单调、epoch 0；250ms 心跳携带 media_time。
    #[test]
    fn advance_pushes_frames_then_heartbeat() {
        let mut d = Driven::new("sample-webm-vp9.webm", 7);
        d.play(7, 0, 0);
        d.out.clear();
        d.advance(16);
        let frames = d.frames();
        assert!(!frames.is_empty(), "首 tick 应呈现首帧");
        assert_eq!(frames[0].pts_ms, 0);
        assert_eq!(frames[0].epoch, 0);
        assert_eq!((frames[0].width, frames[0].height), (320, 240));
        assert!(d.states().is_empty(), "心跳未到（16ms < 250ms）不推状态");
        d.advance(300);
        // pts 单调（跳帧面：tick 返回最新可展示帧）。
        let all = d.frames();
        assert!(
            all.windows(2).all(|w| w[0].pts_ms < w[1].pts_ms),
            "pts 单调: {:?}",
            all.iter().map(|f| f.pts_ms).collect::<Vec<_>>()
        );
        // 心跳：距 play 推送 300ms ≥ 250ms。
        let states = d.states();
        assert!(
            states.iter().any(|(epoch, s)| matches!(s, MediaPlaybackState::Playing { media_time_ms } if *media_time_ms >= 250 && *epoch == 0)),
            "心跳 Playing 携带推进后的 media_time，got {states:?}"
        );
    }

    /// pause：即时 Paused 状态；暂停期推进零产出（无帧/无心跳）。
    #[test]
    fn pause_freezes_and_pushes_paused() {
        let mut d = Driven::new("sample-webm-vp9.webm", 7);
        d.play(7, 0, 0);
        d.advance(100);
        d.handle(
            msg(4, IpcMessageKind::MediaPause(MediaPauseParams { source_key: 7 })),
            100,
        );
        let states = d.states();
        let (_, state) = states.last().expect("pause 即时推送");
        let frozen = match state {
            MediaPlaybackState::Paused { media_time_ms } => *media_time_ms,
            other => panic!("expected Paused, got {other:?}"),
        };
        assert!(frozen > 0, "暂停位 > 0，got {frozen}");
        d.out.clear();
        d.advance(2000);
        assert!(d.out.is_empty(), "暂停期推进零产出，got {:?}", d.out);
    }

    /// seek：SeekResult 配对 request_id、实际落点；代际 +1；后续帧携新代际且
    /// pts ≥ 目标（precise-seek 跨进程契约）。
    #[test]
    fn seek_replies_result_bumps_epoch_frames_follow() {
        let mut d = Driven::new("sample-webm-vp9.webm", 7);
        d.play(7, 0, 0);
        d.advance(100);
        d.out.clear();
        d.handle(
            msg(
                9,
                IpcMessageKind::MediaSeek(MediaSeekParams {
                    source_key: 7,
                    target_ms: 1000,
                    request_id: 42,
                }),
            ),
            100,
        );
        let seek_reply = d
            .out
            .iter()
            .find_map(|m| match &m.kind {
                IpcMessageKind::MediaSeekResult(r) => Some((m.id, r)),
                _ => None,
            })
            .expect("seek 回执");
        assert_eq!(seek_reply.0, 9, "信封 id 配对请求");
        assert_eq!(seek_reply.1.request_id, 42);
        assert!(
            seek_reply.1.resolved_ms >= 1000,
            "落点 ≥ target，got {}",
            seek_reply.1.resolved_ms
        );
        // 新代际状态即时推送。
        let states = d.states();
        let (epoch, state) = states.last().expect("seek 后状态推送");
        assert_eq!(*epoch, 1, "seek 代际 +1");
        assert!(
            matches!(state, MediaPlaybackState::Playing { .. }),
            "播放中 seek 保持播放，got {state:?}"
        );
        // 推进：帧携新代际，pts ≥ 目标。
        d.advance(1200);
        let frames = d.frames();
        assert!(!frames.is_empty());
        assert!(frames.iter().all(|f| f.epoch == 1), "seek 后帧携新代际");
        assert!(frames.iter().all(|f| f.pts_ms >= 1000), "precise-seek：pts ≥ target");
    }

    /// 流末：Ended 恰推一次；ended 后推进零产出。
    #[test]
    fn ended_pushes_once_then_silent() {
        let mut d = Driven::new("sample-webm-vp9.webm", 7);
        d.play(7, 0, 0);
        let mut now = 0u64;
        while ended_count(&d) == 0 {
            now += 500;
            d.advance(now);
            assert!(now < 60_000, "runaway loop");
        }
        assert_eq!(ended_count(&d), 1, "Ended 恰一次");
        d.out.clear();
        d.advance(now + 5000);
        assert!(d.out.is_empty(), "ended 后推进零产出");
    }

    /// 纯音频源：零帧推送；心跳携带音频游标（audio clock 主时钟）。
    #[test]
    fn audio_only_source_heartbeats_without_frames() {
        let mut d = Driven::new("sample-ogg-vorbis.oga", 8);
        d.play(8, 0, 0);
        d.out.clear();
        d.advance(100);
        d.advance(400);
        assert!(d.frames().is_empty(), "纯音频零帧");
        let states = d.states();
        assert!(
            states
                .iter()
                .any(|(_, s)| matches!(s, MediaPlaybackState::Playing { media_time_ms } if *media_time_ms >= 250)),
            "心跳携带音频游标，got {states:?}"
        );
    }

    /// A/V pair：帧 pts 追随音频游标（主时钟）；心跳 media_time = 游标。
    #[test]
    fn av_pair_frames_follow_audio_cursor() {
        let mut d = Driven::new("sample-webm-vp9-vorbis.webm", 9);
        d.play(9, 0, 0);
        d.out.clear();
        d.advance(120);
        d.advance(400);
        let frames = d.frames();
        assert!(!frames.is_empty(), "A/V pair 应产出帧");
        assert!(frames.windows(2).all(|w| w[0].pts_ms < w[1].pts_ms), "pts 单调");
        // 心跳 media_time 与最新呈现帧同代际推进（游标授权帧调度）。
        let states = d.states();
        assert!(
            states
                .iter()
                .any(|(_, s)| matches!(s, MediaPlaybackState::Playing { media_time_ms } if *media_time_ms >= 250)),
            "心跳 = 主时钟游标，got {states:?}"
        );
    }

    /// loop 回卷：音频流末回卷 → 代际 +1 + 新代际 Playing 推送（小 media_time）；
    /// 不推 Ended。
    #[test]
    fn loop_wrap_bumps_epoch_without_ended() {
        let mut d = Driven::new("sample-ogg-vorbis.oga", 10);
        d.handle(
            msg(
                5,
                IpcMessageKind::MediaSetLoop(MediaSetLoopParams {
                    source_key: 10,
                    on: true,
                }),
            ),
            0,
        );
        d.play(10, 0, 0);
        let mut now = 0u64;
        let mut wrapped_epoch = None;
        while wrapped_epoch.is_none() {
            now += 400;
            d.advance(now);
            wrapped_epoch = d
                .states()
                .iter()
                .filter(|(epoch, _)| *epoch >= 1)
                .map(|(epoch, _)| *epoch)
                .next_back();
            assert!(now < 60_000, "runaway loop");
        }
        assert!(
            d.states().iter().all(|(_, s)| !matches!(s, MediaPlaybackState::Ended)),
            "loop 面不推 Ended，got {:?}",
            d.states()
        );
        // 回卷后游标回小值：新代际 Playing 小 media_time。
        let fresh: Vec<&MediaPlaybackState> = d
            .out
            .iter()
            .filter_map(|m| match &m.kind {
                IpcMessageKind::MediaStateUpdate(s) if s.epoch >= 1 => Some(&s.state),
                _ => None,
            })
            .collect();
        assert!(
            fresh
                .iter()
                .any(|s| matches!(s, MediaPlaybackState::Playing { media_time_ms } if *media_time_ms < 400)),
            "回卷后新代际小 media_time，got {fresh:?}"
        );
    }

    /// Ended 后 replay：代际 +1、重头播放（Playing@0）、帧重新从流首产出。
    #[test]
    fn replay_after_ended_bumps_epoch_and_restarts() {
        let mut d = Driven::new("sample-webm-vp9.webm", 11);
        d.play(11, 0, 0);
        let mut now = 0u64;
        while ended_count(&d) == 0 {
            now += 500;
            d.advance(now);
            assert!(now < 60_000, "runaway loop");
        }
        d.out.clear();
        d.play(11, 0, now + 100);
        let states = d.states();
        let (epoch, state) = states.last().expect("replay 即时状态");
        assert_eq!(*epoch, 1, "replay 代际 +1");
        assert!(
            matches!(state, MediaPlaybackState::Playing { media_time_ms: 0 }),
            "重头 Playing@0，got {state:?}"
        );
        d.advance(now + 108);
        let frames = d.frames();
        assert!(!frames.is_empty(), "replay 后帧重新产出");
        assert!(frames.iter().all(|f| f.epoch == 1), "replay 帧携新代际");
        assert_eq!(frames[0].pts_ms, 0, "重头播放首帧 pts=0");
    }

    /// 事件循环截止：空闲 None（可无限期阻塞）；播放中 ≤ 心跳窗口。
    #[test]
    fn next_wake_none_idle_bounded_playing() {
        let kernel = MediaKernel::new();
        assert_eq!(kernel.next_wake_ms(1000), None, "空闲无截止");
        let mut d = Driven::new("sample-webm-vp9.webm", 12);
        d.play(12, 0, 0);
        let wake = d.kernel.next_wake_ms(0).expect("播放中有截止");
        assert!(wake <= HEARTBEAT_MS, "截止 ≤ 心跳窗口，got {wake}");
    }
}
