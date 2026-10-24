//! 音频播放管线 — 解码条目 + 样本级批内对齐队列（A1 时钟语义单一真值）。
//!
//! 自 webview `video_registry.rs` 迁入（media-runtime-isolation C2，DESIGN §10.3
//! 「同构代码随内核入宿主」）：独立媒体进程（media-host）与 registry 进程内直调
//! （feature 关）共享同一 A1 修复语义——两形态各自持有条目实例，时钟代码单一
//! 真值，C4 三态配对不因副本漂移失真。类型仅依赖本 crate 面（解码器/sink），
//! 不涉 webview/renderer。

use std::collections::VecDeque;

use crate::{
    AudioDecodeError, AudioDecoder, AudioFormat, AudioSink, DecodedAudio, NullSink, OpusAudioTrack, WebmAudioTrack,
    WebmOpusAudioTrack, open_ogg_opus, open_webm_audio_track, open_webm_opus_audio_track,
};

/// 音频解码流源 — symphonia 面（mp3/vorbis）与 opus 面（opus-decoder）的统一契约
/// （M2c opus 接线：`sample_rate`/`channels`/`next_batch` 透传；单测面在 zero-media）。
pub enum AudioStreamDecoder {
    Symphonia(AudioDecoder),
    Opus(Box<OpusAudioTrack>),
}

impl AudioStreamDecoder {
    pub fn sample_rate(&self) -> u32 {
        match self {
            Self::Symphonia(d) => d.sample_rate(),
            Self::Opus(t) => t.sample_rate(),
        }
    }

    pub fn channels(&self) -> u16 {
        match self {
            Self::Symphonia(d) => d.channels(),
            Self::Opus(t) => t.channels(),
        }
    }

    pub fn next_batch(&mut self) -> Result<Option<DecodedAudio>, AudioDecodeError> {
        match self {
            Self::Symphonia(d) => d.next_batch(),
            Self::Opus(t) => t.next_batch(),
        }
    }
}

/// 批间样本队列 — `advance_to` 的样本级精确消费面（A1 时钟批内对齐，M3/RFC 批次）。
///
/// 缺陷机制（M1 FINDING，[DESIGN §2](../../../docs/goal/media-runtime-isolation/evidence/2026-10-24-m3-prep-design/DESIGN.md)）：
/// vorbis/symphonia 按包交付（~1024 样本/ch ≈ 23.3ms @44.1kHz），批长 > 16ms 泵节拍
/// → 每 tick 整批消费 → 游标以批长/tick 推进 → media clock rate ≈ 1.45（实测
/// 1.45402）。修复 = 解码批先入队（解码超前有界 ≈1 批），按 tick 目标帧数（毫秒
/// →帧定点换算，余数跨 tick 累积）精确写出——构造上 Σ写出时长 ≡ Σelapsed，
/// rate ≡ 1.0；解码推进与游标推进保持耦合（解码超前不随时钟发散）。
///
/// 语义面：EOF/loop/错误分支与 seek 追赶区静默线（批粒度判定）均保持原行为；
/// 轨重建（seek/回卷/复活）随解码器重建 [`Self::clear`]。格式以帧为换算粒度
/// （帧 = 一组全声道交错采样），保证 sink 写入恒帧对齐（[`AudioSink::write`]
/// 契约）。
pub struct BatchSampleQueue {
    /// 已解码未写 sink 的交错样本（f32；批粒度入队，样本粒度写出）。
    pub pending: VecDeque<f32>,
    /// 队列当前格式（首批登记；批间格式变更时清队列防混排）。`None` = 未拉到
    /// 可入队批次（含追赶区全丢弃面）。
    pub format: Option<(u32, u16)>,
    /// 毫秒→帧定点换算余数（1/1000 帧单位，跨 tick 累积）。
    pub frac_rem: u32,
}

impl BatchSampleQueue {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            format: None,
            frac_rem: 0,
        }
    }

    /// 轨重建/游标重置面（seek/回卷/复活）清队——旧位样本不得入新轨游标。
    pub fn clear(&mut self) {
        self.pending.clear();
        self.format = None;
        self.frac_rem = 0;
    }

    /// 批次入队（交错样本追加；与已登记格式不符时先清队——不同格式样本混排
    /// 会破坏帧换算与 sink 对齐）。
    pub fn push_batch(&mut self, batch: &DecodedAudio) {
        if self
            .format
            .is_some_and(|(rate, ch)| rate != batch.sample_rate || ch != batch.channels)
        {
            self.pending.clear();
            self.frac_rem = 0;
        }
        self.format = Some((batch.sample_rate, batch.channels));
        self.pending.extend(batch.samples.iter().copied());
    }

    /// 本 tick 应写样本数：`elapsed` 毫秒 → 帧数定点换算（余数累积）× 声道数。
    /// 格式未登记（追赶区未产出可入队批次）时 `None`，调用方继续拉批。
    pub fn need_for_tick(&mut self, elapsed_ms: u64) -> Option<usize> {
        let (rate, channels) = self.format?;
        let milli_frames = elapsed_ms * u64::from(rate) + u64::from(self.frac_rem);
        self.frac_rem = (milli_frames % 1000) as u32;
        Some((milli_frames / 1000) as usize * usize::from(channels))
    }

    /// 写出队首至多 `need` 个样本（增益变换原样；`usize::MAX` = 全量，EOF 尾段
    /// 回卷面用）。返回实际写出样本数（供游标按实际时长推进）。
    pub fn write_to(&mut self, sink: &mut NullSink, need: usize, gain: f32) -> usize {
        let n = need.min(self.pending.len());
        if n == 0 {
            return 0;
        }
        if gain == 1.0 {
            // 零拷贝面：as_slices 双段覆盖跨环包裹（常规态单段）。
            let (head, tail) = self.pending.as_slices();
            let (a, b) = if head.len() >= n {
                (&head[..n], &[][..])
            } else {
                (head, &tail[..n - head.len()])
            };
            let _ = sink.write(a);
            if !b.is_empty() {
                let _ = sink.write(b);
            }
        } else {
            let gained: Vec<f32> = self.pending.range(..n).map(|s| s * gain).collect();
            let _ = sink.write(&gained);
        }
        self.pending.drain(..n);
        n
    }

    /// 已写出样本数 → 毫秒（当前格式；格式未登记返回 0，游标不动）。
    pub fn duration_ms_of(&self, samples: usize) -> u64 {
        match self.format {
            Some((rate, channels)) if channels > 0 => (samples as u64 * 1000) / (u64::from(rate) * u64::from(channels)),
            _ => 0,
        }
    }
}

/// 音频播放条目 — 解码器 + NullSink（可观测）+ 增益（M2c 后续：播放管线接 sink）。
///
/// 时序模型与 VideoPlayer 同源：调用方注入单调时钟，`advance_to` 按已流逝媒体时间
/// **实时节奏**解码（每 tick 解码至 `cursor_ms` ≈ 墙钟位置——流式等价物，不做整段
/// 预解）。增益对齐 media-elements IDL 面：`muted` → 0 增益、`volume` [0,1] 乘法
/// （muted/volume setter 桥推与 play 起播同步，切片 5b 同款）。
pub struct AudioEntry {
    pub decoder: AudioStreamDecoder,
    pub sink: NullSink,
    /// 播放中（时钟推进门）。
    pub playing: bool,
    /// 媒体时间游标（毫秒）。
    pub cursor_ms: u64,
    /// seek 后前向解码的静默丢弃线（毫秒）——解码器单向流，seek 重建后须解码至此
    /// 才到目标位置；追赶区采样不入 sink（spec precise-seek：seek 中不输出目标前内容）。
    pub skip_until_ms: u64,
    /// 上次 advance 的墙钟锚点（None = 未起播/暂停）。
    pub last_tick_ms: Option<u64>,
    pub volume: f32,
    pub muted: bool,
    /// loop 属性真面（M3 扩批 XXIV）：流末回卷重播（spec「ended playback」步 6.4
    /// loop 分支——「seek to earliest position」；解码器单向流经 restart 重建）。
    /// 语义层 seeked 派发由 shim march 面（ms.loop 时 ended→seeked）承接。
    pub loop_on: bool,
    /// 流末到达标志（loop=false 停止时置位，play/seek/restart 清除）——桥 isEnded
    /// 的音频面（march ended/loop 分叉的驱动源；此前仅 video player Ended 态可见）。
    pub reached_end: bool,
    /// loop 回卷待观测标志（media-elements M3 扩批 XXXIX）：loop=true 的流末回卷
    /// 此前为静默 restart——语义层 march 的 ended/loop 分叉以 isEnded 为驱动源，
    /// 恒 false 使 seeking/seeked 派发不可达（audio_loop_base Timeout 根因）。
    /// 回卷时置位，语义层 loop 分叉消费（seek(0)+play(0)）后清除。
    pub wrap_pending: bool,
    /// 样本级消费队列（A1 批内对齐——[`BatchSampleQueue`] 模块文档）。
    pub queue: BatchSampleQueue,
}

impl AudioEntry {
    pub fn new(decoder: AudioStreamDecoder) -> Self {
        let (rate, channels) = (decoder.sample_rate(), decoder.channels());
        let mut sink = NullSink::new();
        let _ = sink.start(AudioFormat {
            sample_rate: rate,
            channels,
        });
        Self {
            decoder,
            sink,
            playing: false,
            cursor_ms: 0,
            skip_until_ms: 0,
            last_tick_ms: None,
            volume: 1.0,
            muted: false,
            loop_on: false,
            reached_end: false,
            wrap_pending: false,
            queue: BatchSampleQueue::new(),
        }
    }

    /// 流末回卷（loop=true 的 advance_to 流末分支）：重建解码器、游标归零、
    /// 追赶线清零、播放态保持（spec loop「seek to earliest position」）。
    /// 解码器单向流 → 重建（fixture 级小源可接受，audio_seek 同款）。
    /// 重建需源字节——registry 侧 `sources` 留存（register_audio_source 同键共享）。
    pub fn restart(&mut self, bytes: &[u8]) {
        let decoder = match AudioDecoder::open(bytes) {
            Ok(d) => AudioStreamDecoder::Symphonia(d),
            Err(_) => match open_ogg_opus(bytes) {
                Ok(t) => AudioStreamDecoder::Opus(Box::new(t)),
                Err(_) => {
                    // 重建失败：回落停止面（语义层照常派 ended）。
                    self.playing = false;
                    return;
                }
            },
        };
        let (rate, channels) = (decoder.sample_rate(), decoder.channels());
        self.decoder = decoder;
        let _ = self.sink.start(AudioFormat {
            sample_rate: rate,
            channels,
        });
        self.cursor_ms = 0;
        self.skip_until_ms = 0;
        self.reached_end = false;
        // 解码器重建 = 流位重置，旧位已解码样本作废（清队面）。
        self.queue.clear();
    }

    /// 推进到墙钟 `now_ms`（实时节奏解码；带增益写入 sink）。返回是否写入采样。
    /// `restart_bytes`：loop=true 时流末回卷所需的源字节（解码器重建）——None 时
    /// 流末照旧停止。
    pub fn advance_to(&mut self, now_ms: u64, restart_bytes: Option<&[u8]>) -> bool {
        if !self.playing {
            return false;
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let elapsed = now_ms.saturating_sub(last);
        let target = self.cursor_ms + elapsed;
        let gain = if self.muted { 0.0 } else { self.volume };
        let mut wrote = false;
        // A1 批内对齐：解码批入队（解码超前有界），按本 tick 目标样本数精确
        // 写出（毫秒→帧定点换算，余数跨 tick 累积）——批粒度整批推进曾使
        // media clock rate≈1.45（M1 FINDING，[`BatchSampleQueue`] 模块文档）。
        let mut need = self.queue.need_for_tick(elapsed);
        // 本 tick 已解码到的流位（含追赶区丢弃批——终止条件与原批粒度循环同）。
        let mut decoded_end = self.cursor_ms;
        while decoded_end < target {
            let Ok(Some(batch)) = self.decoder.next_batch() else {
                // 流末：队内已解码尾段全量冲入 sink 再停/回卷（解码超前有界后
                // 停拍时刻队内尚有未消费尾段——原行为末批整批入 sink 不丢样）。
                let drained = self.queue.write_to(&mut self.sink, usize::MAX, gain);
                wrote |= drained > 0;
                self.cursor_ms += self.queue.duration_ms_of(drained);
                // loop=true → 回卷重播（spec「ended playback」步 6.4——
                // 位置 seek 到最早位置继续播，语义层派 seeked 非 ended）；
                // loop=false → 停在末尾（ended 面归语义层）。
                if self.loop_on {
                    match restart_bytes {
                        Some(bytes) => {
                            self.restart(bytes);
                            self.reached_end = false;
                            // M3 扩批 XXXIX：回卷对语义层可观测——march ended/loop
                            // 分叉（isEnded 驱动）据此派 seeking/seeked 并
                            // seek(0)+play(0) 复位本标志（spec「ended playback」
                            // 步 6.4 loop 分支的事件面）。
                            self.wrap_pending = true;
                        }
                        None => {
                            self.playing = false;
                            self.reached_end = true;
                        }
                    }
                } else {
                    self.playing = false;
                    self.reached_end = true;
                }
                break;
            };
            decoded_end = batch.pts_ms
                + (batch.samples.len() as u64 * 1000)
                    / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1));
            // seek 追赶区（批粒度判定不变）：批末 ≤ 丢弃线整批静默丢弃，不入队。
            if decoded_end > self.skip_until_ms {
                self.queue.push_batch(&batch);
            }
            if need.is_none() {
                need = self.queue.need_for_tick(elapsed);
            }
        }
        if let Some(n) = need {
            let written = self.queue.write_to(&mut self.sink, n, gain);
            wrote |= written > 0;
            if written == n {
                // 全额写出：游标精确到 target（余数已累积，rate ≡ 1.0）。
                self.cursor_ms = target;
            } else if written > 0 {
                // 部分写出（EOF 尾段/等待面）：游标按实际写出时长推进；
                // 零写出（回卷面队列已清/等待首批）游标不动。
                self.cursor_ms += self.queue.duration_ms_of(written);
            }
        }
        wrote
    }
}

/// A/V pair 音频条目 — webm 双轨源的视频伴生音频轨（M2 切片 D）。
///
/// 与 [`AudioEntry`]（纯音频源）分立：解码数据来自 `WebmAudioTrack`（OGG 页重封装
/// + symphonia），生命周期跟随 video player（play 时懒建，pause/clear 释放）。
pub struct WebmAudioEntry {
    /// 伴生音频轨（codec 泛化——A_VORBIS 走 OGG 重封装 + symphonia；A_OPUS 直解
    /// opus-decoder。同 `sample_rate`/`channels`/`next_batch` 输出契约）。
    pub track: WebmAudioTrackKind,
    pub sink: NullSink,
    pub playing: bool,
    /// 已解码累计游标（毫秒）——audio clock 主时钟的媒体时间真值面。
    pub cursor_ms: u64,
    /// seek 追赶区静默线（毫秒；目标前采样不入 sink——AudioEntry 同面）。
    pub skip_until_ms: u64,
    pub last_tick_ms: Option<u64>,
    pub volume: f32,
    pub muted: bool,
    /// loop 真面（M3 扩批 XXIV，AudioEntry 同语义）——伴生轨流末回卷。
    pub loop_on: bool,
    /// 样本级消费队列（A1 批内对齐——[`BatchSampleQueue`] 模块文档）。
    pub queue: BatchSampleQueue,
}

/// webm 伴生音频轨的 codec 形态（M3 扩批 2026-09-02：A_OPUS 加入——WPT 上游
/// media/*.webm 实测全为 VP9+Opus；此前仅 A_VORBIS）。
pub enum WebmAudioTrackKind {
    Vorbis(Box<WebmAudioTrack>),
    Opus(Box<WebmOpusAudioTrack>),
    /// H.264 切片 2（D-RFC-3c AAC 随期）：mp4 容器 AAC 轨（symphonia
    /// aac+isomp4——AudioDecoder probe 自动路由，同 next_batch 输出契约）。
    Mp4Aac(Box<AudioDecoder>),
}

impl WebmAudioTrackKind {
    pub fn sample_rate(&self) -> u32 {
        match self {
            Self::Vorbis(t) => t.sample_rate(),
            Self::Opus(t) => t.sample_rate(),
            Self::Mp4Aac(t) => t.sample_rate(),
        }
    }
    pub fn channels(&self) -> u16 {
        match self {
            Self::Vorbis(t) => t.channels(),
            Self::Opus(t) => t.channels(),
            Self::Mp4Aac(t) => t.channels(),
        }
    }
    pub fn next_batch(&mut self) -> Result<Option<DecodedAudio>, AudioDecodeError> {
        match self {
            Self::Vorbis(t) => t.next_batch(),
            Self::Opus(t) => t.next_batch(),
            Self::Mp4Aac(t) => t.next_batch(),
        }
    }
}

impl WebmAudioEntry {
    pub fn new(track: WebmAudioTrackKind) -> Self {
        Self {
            track,
            sink: NullSink::new(),
            playing: false,
            cursor_ms: 0,
            skip_until_ms: 0,
            last_tick_ms: None,
            volume: 1.0,
            muted: false,
            loop_on: false,
            queue: BatchSampleQueue::new(),
        }
    }

    /// 推进到墙钟 `now_ms`（实时节奏解码写 sink；增益同 AudioEntry 面）。
    /// `skip_until_ms` ≥ 0 时为 seek 追赶区静默线（目标前采样不入 sink）。
    /// `restart_bytes`：loop=true 时流末回卷所需的 webm 双轨源字节（轨重建）。
    pub fn advance_to(&mut self, now_ms: u64, restart_bytes: Option<&[u8]>) -> bool {
        if !self.playing {
            return false;
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let elapsed = now_ms.saturating_sub(last);
        let target = self.cursor_ms + elapsed;
        let gain = if self.muted { 0.0 } else { self.volume };
        let mut wrote = false;
        // A1 批内对齐（AudioEntry 同构，[`BatchSampleQueue`] 模块文档）。
        let mut need = self.queue.need_for_tick(elapsed);
        // 本 tick 已解码到的流位（含追赶区丢弃批——终止条件与原批粒度循环同）。
        let mut decoded_end = self.cursor_ms;
        while decoded_end < target {
            let Ok(Some(batch)) = self.track.next_batch() else {
                // 流末：队内已解码尾段全量冲入 sink 再停/回卷（AudioEntry EOF 面
                // 同构——停拍时刻队内尚有未消费尾段，不丢样）。
                let drained = self.queue.write_to(&mut self.sink, usize::MAX, gain);
                wrote |= drained > 0;
                self.cursor_ms += self.queue.duration_ms_of(drained);
                // loop=true → 回卷（重建伴生轨 + 游标归零）；否则停止。
                if self.loop_on {
                    // codec 泛化重建序与 play 懒建同：A_VORBIS 优先、A_OPUS 次之。
                    let rebuilt = restart_bytes.and_then(|bytes| {
                        if let Ok(track) = open_webm_audio_track(bytes) {
                            Some(WebmAudioTrackKind::Vorbis(Box::new(track)))
                        } else {
                            open_webm_opus_audio_track(bytes)
                                .ok()
                                .map(|t| WebmAudioTrackKind::Opus(Box::new(t)))
                        }
                    });
                    match rebuilt {
                        Some(track) => {
                            let (rate, ch) = (track.sample_rate(), track.channels());
                            self.track = track;
                            let _ = self.sink.start(AudioFormat {
                                sample_rate: rate,
                                channels: ch,
                            });
                            self.cursor_ms = 0;
                            self.skip_until_ms = 0;
                            // 轨重建 = 流位重置，旧位已解码样本作废（清队面）。
                            self.queue.clear();
                        }
                        None => self.playing = false,
                    }
                } else {
                    self.playing = false;
                }
                break;
            };
            decoded_end = batch.pts_ms
                + (batch.samples.len() as u64 * 1000)
                    / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1));
            // seek 追赶区（批粒度判定不变）：批末 ≤ 丢弃线整批静默丢弃，不入队
            // （spec precise-seek——与 AudioEntry 同面）。
            if decoded_end > self.skip_until_ms {
                self.queue.push_batch(&batch);
            }
            if need.is_none() {
                need = self.queue.need_for_tick(elapsed);
            }
        }
        if let Some(n) = need {
            let written = self.queue.write_to(&mut self.sink, n, gain);
            wrote |= written > 0;
            if written == n {
                // 全额写出：游标精确到 target（余数已累积，rate ≡ 1.0）。
                self.cursor_ms = target;
            } else if written > 0 {
                // 部分写出（EOF 尾段/等待面）：游标按实际写出时长推进；
                // 零写出（回卷面队列已清）游标不动。
                self.cursor_ms += self.queue.duration_ms_of(written);
            }
        }
        wrote
    }
}

/// MSE 伴生音频条目（t8s-3）——audio SourceBuffer 独立字节流的播放面。
///
/// 与 [`WebmAudioEntry`] 同 sink/cursor/增益契约，差异：源是 [`crate::MseAudioFeed`]
/// 增长流——`next_batch` 的 None 是「等待更多 append」而非流末，停止判定走
/// `is_exhausted()`（ended 旗标 + demux 耗尽）；loop 回卷不适用（流式源，FIXME(mse-full-semantics)）。
#[cfg(feature = "decode-h264")]
pub struct MseAudioEntry {
    pub feed: crate::MseAudioFeed,
    pub sink: NullSink,
    pub sink_started: bool,
    pub playing: bool,
    /// 已解码累计游标（毫秒）——audio clock 主时钟的媒体时间真值面。
    pub cursor_ms: u64,
    /// seek 追赶区静默线（毫秒）。
    pub skip_until_ms: u64,
    pub last_tick_ms: Option<u64>,
    pub volume: f32,
    pub muted: bool,
    /// 样本级消费队列（A1 批内对齐——[`BatchSampleQueue`] 模块文档）。
    pub queue: BatchSampleQueue,
}

#[cfg(feature = "decode-h264")]
impl MseAudioEntry {
    pub fn new(feed: crate::MseAudioFeed) -> Self {
        Self {
            feed,
            sink: NullSink::new(),
            sink_started: false,
            playing: false,
            cursor_ms: 0,
            skip_until_ms: 0,
            last_tick_ms: None,
            volume: 1.0,
            muted: false,
            queue: BatchSampleQueue::new(),
        }
    }

    /// 推进到墙钟 `now_ms`（WebmAudioEntry 同面——batch pts + 采样计数游标）。
    pub fn advance_to(&mut self, now_ms: u64) -> bool {
        if !self.playing {
            return false;
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let elapsed = now_ms.saturating_sub(last);
        let target = self.cursor_ms + elapsed;
        let gain = if self.muted { 0.0 } else { self.volume };
        let mut wrote = false;
        // A1 批内对齐（AudioEntry/WebmAudioEntry 同构，[`BatchSampleQueue`]
        // 模块文档）。
        let mut need = self.queue.need_for_tick(elapsed);
        let mut decoded_end = self.cursor_ms;
        while decoded_end < target {
            match self.feed.next_batch() {
                Ok(Some(batch)) => {
                    if !self.sink_started {
                        let _ = self.sink.start(AudioFormat {
                            sample_rate: batch.sample_rate,
                            channels: batch.channels,
                        });
                        self.sink_started = true;
                    }
                    // saturating：pts 饱和值（u64::MAX，负 pts/edit-list 形态）
                    // 不进裸加法（debug overflow panic / release 回绕）。
                    decoded_end = batch.pts_ms.saturating_add(
                        (batch.samples.len() as u64 * 1000)
                            / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1)),
                    );
                    // seek 追赶区（批粒度判定不变）：批末 ≤ 丢弃线整批静默丢弃，
                    // 不入队（spec precise-seek——与 AudioEntry 同面）。
                    if decoded_end > self.skip_until_ms {
                        self.queue.push_batch(&batch);
                    }
                    if need.is_none() {
                        need = self.queue.need_for_tick(elapsed);
                    }
                }
                Ok(None) => {
                    // MSE 流边缘：ended + 耗尽才停；增长等待面保持播放态
                    //（下一 tick 续解码——appendBuffer 与解码泵异步交错）。
                    if self.feed.is_exhausted() {
                        // 停止前队内已解码尾段全量冲入 sink（AudioEntry EOF 面
                        // 同构——不丢样），游标按实际写出推进到流末。
                        let drained = self.queue.write_to(&mut self.sink, usize::MAX, gain);
                        wrote |= drained > 0;
                        self.cursor_ms += self.queue.duration_ms_of(drained);
                        self.playing = false;
                    }
                    break;
                }
                Err(_) => {
                    // 解码错误停轨：同 EOF 冲队面（已解码样本不丢），错误批本身
                    // 已失——事件面归后续切片（FIXME(mse-full-semantics)）。
                    let drained = self.queue.write_to(&mut self.sink, usize::MAX, gain);
                    wrote |= drained > 0;
                    self.cursor_ms += self.queue.duration_ms_of(drained);
                    self.playing = false;
                    break;
                }
            }
        }
        if let Some(n) = need {
            let written = self.queue.write_to(&mut self.sink, n, gain);
            wrote |= written > 0;
            if written == n {
                // 全额写出：游标精确到 target（余数已累积，rate ≡ 1.0）。
                self.cursor_ms = target;
            } else if written > 0 {
                // 部分写出（EOF 尾段/等待面）：游标按实际写出时长推进；
                // 零写出（停止面队列已冲/等待首批）游标不动。
                self.cursor_ms += self.queue.duration_ms_of(written);
            }
        }
        wrote
    }
}
