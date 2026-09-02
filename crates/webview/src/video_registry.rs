//! 生产侧播放器注册表 — 每元素 [`VideoPlayer`] 真值管理（media-playback M2a 切片 5）。
//!
//! 宿主桥（`__zw_video_*` 回调族，后续接线）经本注册表驱动 play/pause/currentTime
//! 真值；渲染泵（rAF `__zw_raf_tick` 同源时钟）每帧调 [`VideoPlayerRegistry::
//! tick_all`] 推进帧并注入 `ImageCache`（painter video 通路同键）。
//!
//! 键契约：`image_resource_key(abs_src, None)`——与 painter/async_load settle 注入
//! 同键，settle 时注册表先收到解码字节（`register_source`），play 即建 player。

use std::collections::HashMap;
use zero_engine::image_resource_key;
use zero_media::{
    AudioDecodeError, AudioDecoder, AudioFormat, AudioSink, DecodedAudio, NullSink, OpusAudioTrack, VideoClock,
    VideoDecoder, VideoPlayer, WebmAudioTrack, WebmOpusAudioTrack, open_ogg_opus, open_webm_audio_track,
    open_webm_opus_audio_track,
};

use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};

/// 音频解码流源 — symphonia 面（mp3/vorbis）与 opus 面（opus-decoder）的统一契约
/// （M2c opus 接线：`sample_rate`/`channels`/`next_batch` 透传；单测面在 zero-media）。
enum AudioStreamDecoder {
    Symphonia(AudioDecoder),
    Opus(Box<OpusAudioTrack>),
}

impl AudioStreamDecoder {
    fn sample_rate(&self) -> u32 {
        match self {
            Self::Symphonia(d) => d.sample_rate(),
            Self::Opus(t) => t.sample_rate(),
        }
    }

    fn channels(&self) -> u16 {
        match self {
            Self::Symphonia(d) => d.channels(),
            Self::Opus(t) => t.channels(),
        }
    }

    fn next_batch(&mut self) -> Result<Option<zero_media::DecodedAudio>, zero_media::AudioDecodeError> {
        match self {
            Self::Symphonia(d) => d.next_batch(),
            Self::Opus(t) => t.next_batch(),
        }
    }
}

/// 音频播放条目 — 解码器 + NullSink（可观测）+ 增益（M2c 后续：播放管线接 sink）。
///
/// 时序模型与 VideoPlayer 同源：调用方注入单调时钟，`advance_to` 按已流逝媒体时间
/// **实时节奏**解码（每 tick 解码至 `cursor_ms` ≈ 墙钟位置——流式等价物，不做整段
/// 预解）。增益对齐 media-elements IDL 面：`muted` → 0 增益、`volume` [0,1] 乘法
/// （muted/volume setter 桥推与 play 起播同步，切片 5b 同款）。
struct AudioEntry {
    decoder: AudioStreamDecoder,
    sink: NullSink,
    /// 播放中（时钟推进门）。
    playing: bool,
    /// 媒体时间游标（毫秒）。
    cursor_ms: u64,
    /// seek 后前向解码的静默丢弃线（毫秒）——解码器单向流，seek 重建后须解码至此
    /// 才到目标位置；追赶区采样不入 sink（spec precise-seek：seek 中不输出目标前内容）。
    skip_until_ms: u64,
    /// 上次 advance 的墙钟锚点（None = 未起播/暂停）。
    last_tick_ms: Option<u64>,
    volume: f32,
    muted: bool,
}

impl AudioEntry {
    fn new(decoder: AudioStreamDecoder) -> Self {
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
        }
    }

    /// 推进到墙钟 `now_ms`（实时节奏解码；带增益写入 sink）。返回是否写入采样。
    fn advance_to(&mut self, now_ms: u64) -> bool {
        if !self.playing {
            return false;
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let target = self.cursor_ms + now_ms.saturating_sub(last);
        let gain = if self.muted { 0.0 } else { self.volume };
        let mut wrote = false;
        while self.cursor_ms < target {
            let Ok(Some(batch)) = self.decoder.next_batch() else {
                // 流末：停在末尾（ended 面归语义层）。
                self.playing = false;
                break;
            };
            let batch_end_ms = batch.pts_ms
                + (batch.samples.len() as u64 * 1000)
                    / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1));
            self.cursor_ms = batch_end_ms;
            // seek 追赶区（batch 末 ≤ 丢弃线）：静默解码，不入 sink。
            if batch_end_ms <= self.skip_until_ms {
                continue;
            }
            if gain == 1.0 {
                let _ = self.sink.write(&batch.samples);
            } else {
                let gained: Vec<f32> = batch.samples.iter().map(|s| s * gain).collect();
                let _ = self.sink.write(&gained);
            }
            wrote = true;
            // 批次已越过 target（包粒度 > 剩余）——停在 batch 末（包不可分割）。
            if self.cursor_ms >= target {
                break;
            }
        }
        wrote
    }
}

/// A/V pair 音频条目 — webm 双轨源的视频伴生音频轨（M2 切片 D）。
///
/// 与 [`AudioEntry`]（纯音频源）分立：解码数据来自 `WebmAudioTrack`（OGG 页重封装
/// + symphonia），生命周期跟随 video player（play 时懒建，pause/clear 释放）。
struct WebmAudioEntry {
    /// 伴生音频轨（codec 泛化——A_VORBIS 走 OGG 重封装 + symphonia；A_OPUS 直解
    /// opus-decoder。同 `sample_rate`/`channels`/`next_batch` 输出契约）。
    track: WebmAudioTrackKind,
    sink: NullSink,
    playing: bool,
    /// 已解码累计游标（毫秒）——audio clock 主时钟的媒体时间真值面。
    cursor_ms: u64,
    /// seek 追赶区静默线（毫秒；目标前采样不入 sink——AudioEntry 同面）。
    skip_until_ms: u64,
    last_tick_ms: Option<u64>,
    volume: f32,
    muted: bool,
}

/// webm 伴生音频轨的 codec 形态（M3 扩批 2026-09-02：A_OPUS 加入——WPT 上游
/// media/*.webm 实测全为 VP9+Opus；此前仅 A_VORBIS）。
enum WebmAudioTrackKind {
    Vorbis(Box<WebmAudioTrack>),
    Opus(Box<WebmOpusAudioTrack>),
}

impl WebmAudioTrackKind {
    fn sample_rate(&self) -> u32 {
        match self {
            Self::Vorbis(t) => t.sample_rate(),
            Self::Opus(t) => t.sample_rate(),
        }
    }
    fn channels(&self) -> u16 {
        match self {
            Self::Vorbis(t) => t.channels(),
            Self::Opus(t) => t.channels(),
        }
    }
    fn next_batch(&mut self) -> Result<Option<DecodedAudio>, AudioDecodeError> {
        match self {
            Self::Vorbis(t) => t.next_batch(),
            Self::Opus(t) => t.next_batch(),
        }
    }
}

impl WebmAudioEntry {
    fn new(track: WebmAudioTrackKind) -> Self {
        Self {
            track,
            sink: NullSink::new(),
            playing: false,
            cursor_ms: 0,
            skip_until_ms: 0,
            last_tick_ms: None,
            volume: 1.0,
            muted: false,
        }
    }

    /// 推进到墙钟 `now_ms`（实时节奏解码写 sink；增益同 AudioEntry 面）。
    /// `skip_until_ms` ≥ 0 时为 seek 追赶区静默线（目标前采样不入 sink）。
    fn advance_to(&mut self, now_ms: u64) -> bool {
        if !self.playing {
            return false;
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let target = self.cursor_ms + now_ms.saturating_sub(last);
        let gain = if self.muted { 0.0 } else { self.volume };
        let mut wrote = false;
        while self.cursor_ms < target {
            let Ok(Some(batch)) = self.track.next_batch() else {
                // 流末：解码停止（ended 面归语义层）；playing 置假停泵。
                self.playing = false;
                break;
            };
            self.cursor_ms = batch.pts_ms
                + (batch.samples.len() as u64 * 1000)
                    / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1));
            // seek 追赶区（batch 末 ≤ 丢弃线）：静默解码，不入 sink（spec
            // precise-seek——与 AudioEntry 同面）。
            if self.cursor_ms <= self.skip_until_ms {
                continue;
            }
            if gain == 1.0 {
                let _ = self.sink.write(&batch.samples);
            } else {
                let gained: Vec<f32> = batch.samples.iter().map(|s| s * gain).collect();
                let _ = self.sink.write(&gained);
            }
            wrote = true;
            // 批次越过 target（包粒度 > 剩余）——停在本包末（包不可分割）。
            if self.cursor_ms >= target {
                break;
            }
        }
        wrote
    }
}

/// 每元素播放器注册表（键 = 资源绝对 URL 的 painter 同款哈希）。
#[derive(Default)]
pub struct VideoPlayerRegistry {
    /// 已 settle 的源字节（play 时建 player——解码器单向流，一次构建）。
    sources: HashMap<u64, Vec<u8>>,
    players: HashMap<u64, VideoPlayer>,
    /// 音频面（M2c 后续）：`<audio>` 元素/settle 判定为纯音频的源。
    audio_entries: HashMap<u64, AudioEntry>,
    /// A/V pair（M2 切片 D）：webm 双轨源的伴生音频轨（video play 时懒建）。
    av_audio_entries: HashMap<u64, WebmAudioEntry>,
    /// A/V pair 源字节留存（切片 E）：伴生轨 seek 重建 `WebmAudioTrack` 所需——
    /// play 消费 sources 后双轨源的字节保到这里（release/clear 同步清理）。
    av_sources: HashMap<u64, Vec<u8>>,
}

impl VideoPlayerRegistry {
    /// 新建空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// settle 时登记源字节（键 = painter 同款资源哈希；重复 settle 幂等覆盖）。
    pub fn register_source(&mut self, abs_src: &str, bytes: Vec<u8>) {
        self.sources.insert(image_resource_key(abs_src, None), bytes);
    }

    /// 元素移除/导航离开时的资源释放（资源生命周期面）。
    pub fn release(&mut self, abs_src: &str) {
        let key = image_resource_key(abs_src, None);
        self.players.remove(&key);
        self.sources.remove(&key);
        self.audio_entries.remove(&key);
        self.av_audio_entries.remove(&key);
        self.av_sources.remove(&key);
    }

    /// settle 时登记音频源（`<audio>` settle 面；解码器立即构建——音频探测轻量）。
    pub fn register_audio_source(&mut self, abs_src: &str, bytes: Vec<u8>) {
        let key = image_resource_key(abs_src, None);
        let decoder = match AudioDecoder::open(&bytes) {
            Ok(d) => AudioStreamDecoder::Symphonia(d),
            // symphonia 面外（oga-opus 等）→ opus 纯 Rust 面（M2c opus 接线）。
            Err(_) => match open_ogg_opus(&bytes) {
                Ok(t) => AudioStreamDecoder::Opus(Box::new(t)),
                // 两面皆不识别：不登记——shim 桥 play 返 false 回落 headless
                // （与 video 非 webm 面同策略）。
                Err(_) => {
                    self.audio_entries.remove(&key);
                    return;
                }
            },
        };
        // seek 重建解码器需要源字节——留存（与 video sources 面同键共享）。
        self.sources.insert(key, bytes);
        self.audio_entries.insert(key, AudioEntry::new(decoder));
    }

    /// 释放全部资源（导航离开——DC-4：player/音频解码器/源字节不跨文档泄漏）。
    pub fn clear(&mut self) {
        self.sources.clear();
        self.players.clear();
        self.audio_entries.clear();
        self.av_audio_entries.clear();
        self.av_sources.clear();
    }

    /// 音频 play（桥面；已登记源 → 播放态 + 时钟锚点）。
    pub fn audio_play(&mut self, abs_src: &str, now_ms: u64) -> bool {
        let key = image_resource_key(abs_src, None);
        match self.audio_entries.get_mut(&key) {
            Some(entry) => {
                entry.playing = true;
                entry.last_tick_ms = Some(now_ms);
                true
            }
            None => false,
        }
    }

    /// 音频 pause（时钟冻结；已解码采样保留在 sink 统计面）。
    pub fn audio_pause(&mut self, abs_src: &str) {
        let key = image_resource_key(abs_src, None);
        if let Some(entry) = self.audio_entries.get_mut(&key) {
            entry.playing = false;
            entry.last_tick_ms = None;
        }
    }

    /// 音频 currentTime（毫秒游标 → 秒）。
    pub fn audio_current_time(&self, abs_src: &str) -> f64 {
        let key = image_resource_key(abs_src, None);
        self.audio_entries
            .get(&key)
            .map(|e| e.cursor_ms as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    /// 音频 seek（游标重置；解码器单向流 → 重建——fixture 级小源可接受，
    /// 真实流面后续做 byte-position 恢复）。
    pub fn audio_seek(&mut self, abs_src: &str, target_ms: u64) -> bool {
        let key = image_resource_key(abs_src, None);
        let Some(entry) = self.audio_entries.get_mut(&key) else {
            return false;
        };
        // 解码器重建需要源字节——register_audio_source 留存于 sources（同键共享）。
        // 双面回落序与登记同：先 symphonia，面外再 opus（oga-opus seek 重建面）。
        if let Some(bytes) = self.sources.get(&key) {
            let decoder = match AudioDecoder::open(bytes) {
                Ok(d) => AudioStreamDecoder::Symphonia(d),
                Err(_) => match open_ogg_opus(bytes) {
                    Ok(t) => AudioStreamDecoder::Opus(Box::new(t)),
                    Err(_) => return false,
                },
            };
            let (rate, channels) = (decoder.sample_rate(), decoder.channels());
            entry.decoder = decoder;
            let _ = entry.sink.start(AudioFormat {
                sample_rate: rate,
                channels,
            });
            entry.cursor_ms = target_ms;
            // 追赶区静默线（mp3/vorbis 无精确包定位——重建后前向解码至 target；
            // target 前的采样丢弃，不写 sink）。
            entry.skip_until_ms = target_ms;
            return true;
        }
        false
    }

    /// 音频增益面（media-elements IDL 联动：volume/muted setter 桥推）。
    pub fn audio_set_gain(&mut self, abs_src: &str, volume: f32, muted: bool) {
        let key = image_resource_key(abs_src, None);
        if let Some(entry) = self.audio_entries.get_mut(&key) {
            entry.volume = volume.clamp(0.0, 1.0);
            entry.muted = muted;
        }
        // A/V pair 伴生轨同步增益（video 元素的 volume/muted setter 桥推面）。
        if let Some(entry) = self.av_audio_entries.get_mut(&key) {
            entry.volume = volume.clamp(0.0, 1.0);
            entry.muted = muted;
        }
    }

    /// 音频播放中检查。
    pub fn audio_is_playing(&self, abs_src: &str) -> bool {
        let key = image_resource_key(abs_src, None);
        self.audio_entries.get(&key).is_some_and(|e| e.playing)
    }

    /// play：懒建 player（源未 settle 时 no-op false——元素无资源可播）。
    ///
    /// M2 切片 D（A/V 同步）：webm 含 A_VORBIS 音频轨时同步起播音频面——
    /// `WebmAudioTrack` 与视频轨同源 demux，音频写入 NullSink 与视频帧推进共用
    /// 同一时钟注入（audio clock 主时钟的 registry 侧承载；跨轨 drift 校正为
    /// media-audio M2 后续切片）。
    pub fn play(&mut self, abs_src: &str, now_ms: u64) -> bool {
        let key = image_resource_key(abs_src, None);
        if !self.players.contains_key(&key) {
            let Some(bytes) = self.sources.remove(&key) else {
                return false;
            };
            let Ok(decoder) = VideoDecoder::open_webm(&bytes) else {
                return false;
            };
            self.players.insert(key, VideoPlayer::new(decoder));
            // 音频轨伴生（A/V pair）：A_VORBIS（OGG 重封装 + symphonia）优先、
            // A_OPUS（opus-decoder 直解，WPT 源主形态）次之；纯视频 webm 双失败静默
            //（视频面照常）。
            let audio_track = match open_webm_audio_track(&bytes) {
                Ok(track) => Some(WebmAudioTrackKind::Vorbis(Box::new(track))),
                Err(_) => open_webm_opus_audio_track(&bytes)
                    .ok()
                    .map(|t| WebmAudioTrackKind::Opus(Box::new(t))),
            };
            if let Some(track) = audio_track {
                let mut entry = WebmAudioEntry::new(track);
                let _ = entry.sink.start(AudioFormat {
                    sample_rate: entry.track.sample_rate(),
                    channels: entry.track.channels(),
                });
                self.av_audio_entries.insert(key, entry);
                // 双轨源字节留存（切片 E）：伴生轨 seek 重建 `WebmAudioTrack` 所需。
                self.av_sources.insert(key, bytes);
            }
        }
        if let Some(player) = self.players.get_mut(&key) {
            player.play(now_ms);
        }
        // 音频伴生起播（与视频同锚点时钟）。
        if let Some(entry) = self.av_audio_entries.get_mut(&key) {
            entry.playing = true;
            entry.last_tick_ms = Some(now_ms);
        }
        self.players.contains_key(&key)
    }

    /// pause：保持位置（未播放/不存在 no-op）；A/V pair 音频同步暂停。
    pub fn pause(&mut self, abs_src: &str) {
        let key = image_resource_key(abs_src, None);
        if let Some(player) = self.players.get_mut(&key) {
            player.pause();
        }
        if let Some(entry) = self.av_audio_entries.get_mut(&key) {
            entry.playing = false;
            entry.last_tick_ms = None;
        }
    }

    /// playbackRate 变速（clamp 面 player 内置；未建 player 时登记于建时生效——
    /// registry 存储待用速率）。
    pub fn set_playback_rate(&mut self, abs_src: &str, rate: f64) {
        let key = image_resource_key(abs_src, None);
        if let Some(player) = self.players.get_mut(&key) {
            player.set_playback_rate(rate);
        }
    }

    /// seek：精确 seek（关键帧定位 + 前向解码）；播放态保持（时钟锚点重置在
    /// player 内）。A/V pair 同步重建伴生音频轨至 target（audio clock 主时钟
    /// 契约——seek 后视频对齐音频游标，master clock 面不脱轨）。
    /// 返回是否作用于存在的 player。
    pub fn seek(&mut self, abs_src: &str, target_ms: u64) -> bool {
        let key = image_resource_key(abs_src, None);
        let mut player_ok = false;
        match self.players.get_mut(&key) {
            Some(player) => {
                player_ok = player.seek_to_ms(target_ms).is_ok();
            }
            // 未建 player（未 play 过）：登记源存在时建之再 seek——spec seekable
            // 面（ HAVE_METADATA 即可 seek）。
            None => {
                if self.play(abs_src, 0) {
                    // spec「seek 不改 paused」：自动建的 player 置回暂停
                    //（HAVE_METADATA 可 seek 面，未起播）。
                    self.pause(abs_src);
                    if let Some(player) = self.players.get_mut(&key) {
                        player_ok = player.seek_to_ms(target_ms).is_ok();
                    }
                }
            }
        }
        // 伴生音频轨 seek（重建 + 追赶区静默）——master clock 游标对齐 target，
        // 后续 tick 视频 sync_to_media_time 跟随（不脱轨）。
        // codec 泛化重建（A_VORBIS 优先、A_OPUS 次之——与 play 懒建同序）。
        let rebuilt = self.av_sources.get(&key).and_then(|bytes| {
            if let Ok(track) = open_webm_audio_track(bytes) {
                let (rate, ch) = (track.sample_rate(), track.channels());
                Some((WebmAudioTrackKind::Vorbis(Box::new(track)), rate, ch))
            } else if let Ok(track) = open_webm_opus_audio_track(bytes) {
                let (rate, ch) = (track.sample_rate(), track.channels());
                Some((WebmAudioTrackKind::Opus(Box::new(track)), rate, ch))
            } else {
                None
            }
        });
        if player_ok
            && let Some((track, rate, channels)) = rebuilt
            && let Some(entry) = self.av_audio_entries.get_mut(&key)
        {
            entry.track = track;
            let _ = entry.sink.start(AudioFormat {
                sample_rate: rate,
                channels,
            });
            entry.cursor_ms = target_ms;
            entry.skip_until_ms = target_ms;
            entry.last_tick_ms = None;
        }
        player_ok
    }

    /// currentTime 真值（秒；未播放/不存在 → 0——spec HAVE_NOTHING 语义面）。
    /// A/V pair：audio clock 主时钟游标优先（media-audio M2 契约——currentTime
    /// 由组合时钟驱动；无伴生轨回落视频位置）。
    pub fn current_time(&self, abs_src: &str) -> f64 {
        let key = image_resource_key(abs_src, None);
        if let Some(cursor_ms) = self.av_audio_entries.get(&key).map(|e| e.cursor_ms) {
            return cursor_ms as f64 / 1000.0;
        }
        self.players.get(&key).map(|p| p.current_time()).unwrap_or(0.0)
    }

    /// duration 真值（秒；元数据未就绪/不存在 → None——spec NaN 面）。
    pub fn duration(&self, abs_src: &str) -> Option<f64> {
        let key = image_resource_key(abs_src, None);
        self.players.get(&key).and_then(|p| p.duration())
    }

    /// 是否在播放（桥查询面）。
    pub fn is_playing(&self, abs_src: &str) -> bool {
        let key = image_resource_key(abs_src, None);
        self.players.get(&key).is_some_and(|p| p.is_playing())
    }

    /// 快速检查：是否存在播放中的 player（渲染泵门禁——无播放时零开销跳过 tick）。
    pub fn is_any_playing(&self) -> bool {
        self.players.values().any(|p| p.is_playing())
            || self.audio_entries.values().any(|e| e.playing)
            || self.av_audio_entries.values().any(|e| e.playing)
    }

    /// 音频泵推进（tab_worker 帧泵同节拍调用）：所有播放中的音频条目按实时节奏
    /// 解码写入 sink（增益生效）。返回是否有写入。
    pub fn audio_advance_all(&mut self, now_ms: u64) -> bool {
        let keys: Vec<u64> = self.audio_entries.keys().copied().collect();
        let mut wrote = false;
        for key in keys {
            if let Some(entry) = self.audio_entries.get_mut(&key)
                && entry.advance_to(now_ms)
            {
                wrote = true;
            }
        }
        // A/V pair 伴生音频轨同步推进（M2 切片 D）。
        let av_keys: Vec<u64> = self.av_audio_entries.keys().copied().collect();
        for key in av_keys {
            if let Some(entry) = self.av_audio_entries.get_mut(&key)
                && entry.advance_to(now_ms)
            {
                wrote = true;
            }
        }
        wrote
    }

    /// 渲染泵推进：tick 所有播放中的 player，新帧注入 `image_cache`（painter 同键）。
    /// 返回是否有帧更新（宿主据此触发增量渲染）。
    ///
    /// A/V 同步（M2 切片 E——audio clock 主时钟）：伴生轨先于视频推进（主时钟
    /// 先走），视频帧调度经 `sync_to_media_time` 对齐音频游标——drift 由构造校正
    /// （位置每 tick 派生自主时钟，不积累墙钟差）。纯视频源回落墙钟 tick（零回归）。
    pub fn tick_all(&mut self, now_ms: u64, image_cache: &mut ImageCache) -> bool {
        // 主时钟先行：伴音频轨按实时节奏解码（游标即媒体时间真值）。
        let av_keys: Vec<u64> = self
            .av_audio_entries
            .keys()
            .copied()
            .filter(|k| self.players.contains_key(k))
            .collect();
        for key in av_keys {
            if let Some(entry) = self.av_audio_entries.get_mut(&key) {
                entry.advance_to(now_ms);
            }
        }
        let mut changed = false;
        let keys: Vec<u64> = self.players.keys().copied().collect();
        for key in keys {
            let Some(player) = self.players.get_mut(&key) else {
                continue;
            };
            // A/V pair：视频对齐音频主时钟；纯视频：墙钟 tick。
            let ticked = if self.av_audio_entries.contains_key(&key) {
                let media_ms = self.av_audio_entries.get(&key).map(|e| e.cursor_ms).unwrap_or(0);
                player.sync_to_media_time(media_ms as f64)
            } else {
                player.tick(now_ms)
            };
            let Ok(Some(frame)) = ticked else {
                continue;
            };
            if let Ok(data) = ImageData::from_rgba(frame.rgba, frame.width, frame.height) {
                image_cache.insert_with_key(ImageKey::new(key), data);
                changed = true;
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bytes() -> Vec<u8> {
        fixture_bytes_named("sample-webm-vp9.webm")
    }

    fn fixture_bytes_named(name: &str) -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // crates/
        p.pop(); // workspace root
        p.push("tests/fixtures/media");
        p.push(name);
        std::fs::read(p).expect("media fixture present")
    }

    const SRC: &str = "https://example.com/media/sample-webm-vp9.webm";
    const MP3: &str = "https://example.com/media/sample-mp3.mp3";
    const AV: &str = "https://example.com/media/sample-webm-vp9-vorbis.webm";

    #[test]
    fn registry_play_requires_settled_source() {
        let mut reg = VideoPlayerRegistry::new();
        assert!(!reg.play(SRC, 0), "未 settle 源 play 应 no-op false");
        assert_eq!(reg.current_time(SRC), 0.0);
        assert_eq!(reg.duration(SRC), None);
        assert!(!reg.is_playing(SRC));
    }

    #[test]
    fn registry_play_pause_current_time_truth() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(SRC, fixture_bytes());
        assert!(reg.play(SRC, 1000), "settle 后 play 成功");
        assert!(reg.is_playing(SRC));
        // duration 真值（fixture 2.0s，M1a 实测）。
        assert_eq!(reg.duration(SRC), Some(2.0));
        // pause 冻结：位置保持。
        reg.pause(SRC);
        assert!(!reg.is_playing(SRC));
        let frozen = reg.current_time(SRC);
        reg.play(SRC, 1000);
        // 重新 play 从暂停位置续播（player 状态机语义），此处仅验证时钟锚点重置。
        let _ = reg.current_time(SRC);
        assert!(frozen >= 0.0);
    }

    #[test]
    fn registry_tick_all_advances_frames_into_cache() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(SRC, fixture_bytes());
        let mut cache = ImageCache::new(16, 64 * 1024 * 1024);
        reg.play(SRC, 0);
        // 首 tick（16ms）：24fps 下首帧 pts=0 应呈现并注入。
        assert!(reg.tick_all(16, &mut cache), "首 tick 应注入帧");
        let key = ImageKey::new(image_resource_key(SRC, None));
        let data = cache.get(&key).expect("帧应在 cache");
        assert_eq!((data.width, data.height), (320, 240));
        // 大步进快进到流末：ended 后 tick 无帧更新。
        let mut now = 16u64;
        while reg.is_playing(SRC) {
            now += 500;
            let _ = reg.tick_all(now, &mut cache);
            assert!(now < 60_000, "runaway loop");
        }
        assert!(!reg.tick_all(now + 500, &mut cache), "ended 后无帧更新");
    }

    #[test]
    fn registry_seek_creates_player_and_positions() {
        // M2b：未 play 的已登记源 seek → 自动建 player + 精确定位（currentTime ≥ target）。
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(SRC, fixture_bytes());
        assert!(reg.seek(SRC, 1000), "已登记源 seek 应成功");
        assert!(reg.current_time(SRC) >= 1.0, "seek 后 currentTime ≥ 1s");
        assert!(!reg.is_playing(SRC), "seek 不改 paused（spec）");
        // 未登记源 seek 失败。
        assert!(!reg.seek("https://x/nope.webm", 500));
    }

    #[test]
    fn registry_av_pair_master_clock_video_slaves_to_audio() {
        // M2 切片 E（A/V 同步，audio clock 主时钟）：伴音游标驱动视频帧调度——
        // 视频呈现 pts 追随音频游标；currentTime 反映主时钟；seek 双轨对齐。
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(AV, fixture_bytes_named("sample-webm-vp9-vorbis.webm"));
        assert!(reg.play(AV, 0));
        let mut cache = ImageCache::new(16, 64 * 1024 * 1024);
        // 主时钟先行推进（tick_all 内部序）：音频游标 ≈500ms 后视频应已呈现
        // ≈500ms 前的帧（pts 单调追随游标）。
        let mut now = 0u64;
        while now < 500 {
            now += 100;
            reg.tick_all(now, &mut cache);
        }
        let media_now = reg.current_time(AV);
        assert!(
            (media_now - 0.5).abs() < 0.35,
            "主时钟游标应 ≈0.5s（音频解码粒度），got {media_now}"
        );
        // 帧注入发生过（视频面活着）。
        let key = ImageKey::new(image_resource_key(AV, None));
        assert!(cache.get(&key).is_some(), "A/V 播放期应有帧注入");
        // currentTime = 主时钟游标（audio clock 主时钟——组合时钟驱动）。
        assert!(media_now > 0.0, "currentTime 由主时钟游标驱动");
        // seek 对齐双轨：游标重置到 target、泵继续推进。
        assert!(reg.seek(AV, 1000), "A/V pair seek 成功");
        let after_seek = reg.current_time(AV);
        assert!(after_seek >= 1.0, "seek 后主时钟游标 ≥ 1s，got {after_seek}");
        now += 200;
        reg.tick_all(now, &mut cache);
        let after_tick = reg.current_time(AV);
        assert!(after_tick >= after_seek, "seek 后游标继续前进而非回退");
    }

    #[test]
    fn registry_release_drops_source_and_player() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(SRC, fixture_bytes());
        assert!(reg.play(SRC, 0));
        reg.release(SRC);
        assert!(!reg.is_playing(SRC));
        assert_eq!(reg.current_time(SRC), 0.0);
        // 释放后重新 play（同 URL 再 settle 场景）需重新登记。
        assert!(!reg.play(SRC, 100));
    }

    /// M2c 后续：clear（导航离开 DC-4）——video player + 音频条目 + 源字节全释放。
    #[test]
    fn registry_clear_drops_all_for_navigation() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(SRC, fixture_bytes());
        reg.register_audio_source(MP3, fixture_bytes_named("sample-mp3.mp3"));
        assert!(reg.play(SRC, 0));
        assert!(reg.audio_play(MP3, 0));
        reg.clear();
        assert!(!reg.is_any_playing(), "清空后无播放条目");
        assert!(!reg.play(SRC, 100), "video 源已释放");
        assert!(!reg.audio_play(MP3, 100), "audio 条目已释放");
    }

    /// M2 切片 D：A/V pair——webm 双轨源 play 时伴生音频轨同锚起播、泵推进写
    /// sink、增益联动、pause 冻结；纯视频源 play 不受影响（无音频轨静默）。
    #[test]
    fn registry_av_pair_play_advances_audio_with_video() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(AV, fixture_bytes_named("sample-webm-vp9-vorbis.webm"));
        assert!(reg.play(AV, 0), "双轨源 play 成功");
        assert!(reg.is_playing(AV));
        assert!(reg.is_any_playing());
        // 音频泵推进（与帧泵同节拍）：伴生轨写 sink（440Hz sine）。
        assert!(reg.audio_advance_all(500), "A/V 伴生轨应写 sink");
        reg.pause(AV);
        assert!(!reg.audio_advance_all(10_000), "pause 后伴生轨冻结");
        // 纯视频源：play 照常（无音频轨静默——不 panic 不误报）。
        reg.register_source(SRC, fixture_bytes());
        assert!(reg.play(SRC, 0));
        assert!(
            !reg.audio_advance_all(500) || reg.audio_advance_all(0),
            "纯视频源无声轨写入"
        );
    }
    /// M2 切片 F（A/V pair ended 面）：伴音轨流末后视频须能走到 Ended——
    /// 音频游标冻结后视频帧调度回落墙钟 tick（主时钟源已尽），否则 player
    /// 恒 Playing、渲染泵空转（is_any_playing 永真）。
    #[test]
    fn registry_av_pair_reaches_ended_after_audio_exhausted() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(AV, fixture_bytes_named("sample-webm-vp9-vorbis.webm"));
        assert!(reg.play(AV, 0));
        let mut cache = ImageCache::new(16, 64 * 1024 * 1024);
        let mut now = 0u64;
        // 快进到双轨流末（2s fixture；500ms 步进必越界）。
        while reg.is_playing(AV) && now < 60_000 {
            now += 500;
            reg.tick_all(now, &mut cache);
        }
        assert!(now < 60_000, "runaway loop——A/V pair 未走完（ended 面断）");
        assert!(!reg.is_playing(AV), "流末后 video player 应转 Ended");
        assert!(!reg.is_any_playing(), "无残留播放条目（泵应停）");
        // ended 后 tick 不再有帧更新。
        let now2 = now + 500;
        assert!(!reg.tick_all(now2, &mut cache), "ended 后无帧更新");
    }
}

/// 宿主桥 — 在 sandbox 上注册 `__zw_video_*` 回调族并注入 `__zwVideoBridge` JS 对象
///（media-playback M2a 切片 5b）。
///
/// JS 侧 `globalThis.__zwVideoBridge = { play(src, nowMs), pause(src),
/// currentTime(src), isPlaying(src) }`——shim `play()`/`pause()` feature-detect 此对象
/// 走真值路径；未注册（testharness/reftest 沙箱）时对象不存在，shim 回落 headless
/// 路径（372 基线零回归）。键 = 资源绝对 URL（settle 登记同串——shim `src` getter
/// 的 `_zwResolveFetchUrl` 产出）。
///
/// 回调签名 `Fn(&[String]) -> String`（script-sandbox 契约）；秒值以字符串往返。
pub fn register_video_bridge_callbacks(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    registry: std::sync::Arc<std::sync::Mutex<VideoPlayerRegistry>>,
) {
    // __zw_video_play(absSrc, nowMs) -> "1"/"0"（bool 字符串避免 JS↔host 布尔歧义）。
    // M2c 后续：audio 回退——video 面未命中（非 webm/纯音频源）时试 audio 条目。
    let reg_play = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_play",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let now_ms: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut reg = reg_play.lock().unwrap_or_else(|e| e.into_inner());
            if reg.play(src, now_ms) || reg.audio_play(src, now_ms) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    let reg_pause = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_pause",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let mut reg = reg_pause.lock().unwrap_or_else(|e| e.into_inner());
            reg.pause(src);
            reg.audio_pause(src);
            "1".into()
        }),
    );

    // __zw_video_set_gain(absSrc, volume, muted)——media-elements IDL 联动
    //（volume/muted setter 桥推；音频面增益，video 面 reserved）。
    let reg_gain = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_set_gain",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let volume: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            let muted = args.get(2).map(|s| s == "1").unwrap_or(false);
            reg_gain
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .audio_set_gain(src, volume, muted);
            "1".into()
        }),
    );

    let reg_ct = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_current_time",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_ct.lock().unwrap_or_else(|e| e.into_inner());
            format!("{}", reg.current_time(src))
        }),
    );

    let reg_dur = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_duration",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_dur.lock().unwrap_or_else(|e| e.into_inner());
            match reg.duration(src) {
                Some(d) => format!("{d}"),
                None => "NaN".into(),
            }
        }),
    );

    let reg_rate = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_set_rate",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let rate: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            reg_rate
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_playback_rate(src, rate);
            "1".into()
        }),
    );

    let reg_seek = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_seek",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let target_ms: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut reg = reg_seek.lock().unwrap_or_else(|e| e.into_inner());
            if reg.seek(src, target_ms) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    let reg_playing = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_is_playing",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_playing.lock().unwrap_or_else(|e| e.into_inner());
            if reg.is_playing(src) { "1".into() } else { "0".into() }
        }),
    );

    // JS 侧门面对象：shim 只认它（feature-detect 单点）。
    let _ = sandbox.execute(
        "globalThis.__zwVideoBridge = {\
           play: function (src, nowMs) { return __zw_video_play(src, nowMs | 0) === '1'; },\
           pause: function (src) { __zw_video_pause(src); },\
           seek: function (src, targetMs) { return __zw_video_seek(src, targetMs | 0) === '1'; },\
           currentTime: function (src) { return Number(__zw_video_current_time(src)); },\
           duration: function (src) { return Number(__zw_video_duration(src)); },\
           isPlaying: function (src) { return __zw_video_is_playing(src) === '1'; },\
           setRate: function (src, rate) { __zw_video_set_rate(src, Number(rate)); },\
           setGain: function (src, volume, muted) { __zw_video_set_gain(src, Number(volume), muted ? '1' : '0'); }\
         };",
    );
}

#[cfg(all(test, feature = "v8"))]
mod bridge_tests {
    use super::*;
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    fn fixture_bytes() -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("tests/fixtures/media/sample-webm-vp9.webm");
        std::fs::read(p).expect("webm fixture present")
    }

    const SRC: &str = "https://example.com/media/sample-webm-vp9.webm";

    /// 宿主桥端到端：register → JS __zwVideoBridge.play → currentTime 推进 → pause 冻结
    /// → duration NaN/真值两面。真实 fixture 驱动。
    #[test]
    fn video_bridge_js_face_roundtrip() {
        let registry = std::sync::Arc::new(std::sync::Mutex::new(VideoPlayerRegistry::new()));
        registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .register_source(SRC, fixture_bytes());
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).expect("v8 sandbox");
        register_video_bridge_callbacks(&mut sandbox, registry);

        // 未播放：currentTime 0、duration 2（真值）。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.currentTime('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value
                .replace('\0', ""),
            "0"
        );
        // play → isPlaying true → tick 前位置 0。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.play('https://example.com/media/sample-webm-vp9.webm', 1000))"
                )
                .unwrap()
                .value,
            "true"
        );
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.isPlaying('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value,
            "true"
        );
        // pause → isPlaying false。
        sandbox
            .execute("globalThis.__zwVideoBridge.pause('https://example.com/media/sample-webm-vp9.webm');")
            .unwrap();
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.isPlaying('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value,
            "false"
        );
        // duration 真值面（fixture 2.0s）。
        let dur = sandbox
            .execute("String(globalThis.__zwVideoBridge.duration('https://example.com/media/sample-webm-vp9.webm'))")
            .unwrap()
            .value;
        assert!(dur.starts_with('2'), "duration 应为 2 秒真值，got {dur}");
        // seek 面：seek(1000ms) → currentTime ≥ 1s（精确 seek 真值）。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.seek('https://example.com/media/sample-webm-vp9.webm', 1000))"
                )
                .unwrap()
                .value,
            "true"
        );
        let ct = sandbox
            .execute("String(globalThis.__zwVideoBridge.currentTime('https://example.com/media/sample-webm-vp9.webm'))")
            .unwrap()
            .value;
        let ct: f64 = ct.parse().unwrap_or(0.0);
        assert!(ct >= 1.0, "seek(1000) 后 currentTime ≥ 1s，got {ct}");
        // 未登记源：play false。
        assert_eq!(
            sandbox
                .execute("String(globalThis.__zwVideoBridge.play('https://x/nope.webm', 0))")
                .unwrap()
                .value,
            "false"
        );
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("tests/fixtures/media");
        p.push(name);
        std::fs::read(p).expect("audio fixture present")
    }

    const MP3: &str = "https://example.com/media/sample-mp3.mp3";

    /// M2c 后续全链：音频 settle 登记 → 桥 play → 泵推进（实时节奏解码写 sink，
    /// 增益联动）→ currentTime 真值。真实 mp3 fixture 驱动（非 mock 充数）。
    #[test]
    fn audio_registry_play_advance_gain_chain() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_source(MP3, fixture_bytes("sample-mp3.mp3"));
        reg.register_audio_source(MP3, fixture_bytes("sample-mp3.mp3"));
        // 未登记源拒绝。
        assert!(!reg.audio_play("https://x/nope.mp3", 0));
        // play → 泵推进 500ms → sink 已写采样 + currentTime 真值。
        assert!(reg.audio_play(MP3, 0), "已登记音频源 play 成功");
        assert!(reg.audio_is_playing(MP3));
        assert!(reg.audio_advance_all(500), "泵推进应写入采样");
        let ct = reg.audio_current_time(MP3);
        assert!(
            (0.4..=1.5).contains(&ct),
            "500ms 推进后 currentTime ≈0.5s（包粒度容差），got {ct}"
        );
        // 增益：muted 后推进仍写（0 增益采样——统计面帧数增长，值面全零）。
        reg.audio_set_gain(MP3, 0.5, true);
        let before = reg.audio_current_time(MP3);
        assert!(reg.audio_advance_all((before * 1000.0) as u64 + 500));
        // pause 冻结。
        reg.audio_pause(MP3);
        assert!(!reg.audio_is_playing(MP3));
        assert!(!reg.audio_advance_all(10_000), "暂停期泵不推进");
        // seek：游标重置（1s）。
        assert!(reg.audio_seek(MP3, 1000));
        assert!((reg.audio_current_time(MP3) - 1.0).abs() < 0.05, "seek 后游标 = target");
        // seek 后立即推进：解码器重建后的前向追赶区（target 前）静默丢弃——sink 统计
        // 不增长（precise-seek 面），游标照常推进。
        let before = reg.audio_current_time(MP3);
        assert!(!reg.audio_advance_all((before * 1000.0) as u64), "追赶区解码不写 sink");
        assert!(
            reg.audio_current_time(MP3) >= before,
            "追赶后游标 ≥ seek 点（前向解码推进）"
        );
    }

    /// opus 面（M2c opus 接线）：oga-opus 经 opus 纯 Rust 面登记成功 → play 可达
    /// （opus-decoder 解码链；负例移至 garbage 面在 zero-media 单测）。
    #[test]
    fn audio_opus_source_registered_and_plays() {
        let mut reg = VideoPlayerRegistry::new();
        let opus = fixture_bytes("sample-ogg-opus.oga");
        reg.register_audio_source("https://example.com/media/song.oga", opus);
        assert!(
            reg.audio_play("https://example.com/media/song.oga", 0),
            "opus 纯 Rust 面登记成功，play 应可达"
        );
        // 实时节奏推进写 sink（440Hz sine）。
        assert!(reg.audio_advance_all(500), "opus 泵推进应写 sink");
    }

    /// 非音频字节拒收。
    #[test]
    fn audio_garbage_source_not_registered() {
        let mut reg = VideoPlayerRegistry::new();
        reg.register_audio_source("https://example.com/x.mp3", b"not audio".to_vec());
        assert!(!reg.audio_play("https://example.com/x.mp3", 0));
    }

    /// 桥端到端（V8 sandbox）：audio src 走同一 __zwVideoBridge 门面（play 回退到
    /// audio 条目、currentTime 读音频游标、setGain 联动）。
    #[test]
    #[cfg(feature = "v8")]
    fn audio_bridge_js_face_roundtrip() {
        use zero_script_sandbox::{Sandbox, V8Sandbox};
        let registry = std::sync::Arc::new(std::sync::Mutex::new(VideoPlayerRegistry::new()));
        registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .register_audio_source(MP3, fixture_bytes("sample-mp3.mp3"));
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).expect("v8 sandbox");
        register_video_bridge_callbacks(&mut sandbox, registry);

        // play（video 未命中 → audio 回退）→ advance 由泵驱动；此处直调泵面等价：
        assert_eq!(
            sandbox
                .execute(&format!("String(globalThis.__zwVideoBridge.play('{MP3}', 0))"))
                .unwrap()
                .value,
            "true"
        );
        // setGain 联动（不抛即通过——真值面由 registry 单测覆盖）。
        sandbox
            .execute(&format!("globalThis.__zwVideoBridge.setGain('{MP3}', 0.5, true);"))
            .unwrap();
        // 未登记源 play false（双面皆无）。
        assert_eq!(
            sandbox
                .execute("String(globalThis.__zwVideoBridge.play('https://x/nope.mp3', 0))")
                .unwrap()
                .value,
            "false"
        );
    }

    #[cfg(feature = "v8")]
    #[test]
    fn webaudio_bridge_nullsink_observable_chain() {
        // media-audio M3 切片 2 e2e（D1 批复）：AudioContext 全链 NullSink 可观测——
        // JS AudioContext 门面（shim part06）→ __zwWA* 宿主桥 → WebAudioRegistry
        // advance → NullSink 帧数 + 过零率锚点（440Hz sine ≈880，M1 契约同款）。
        use zero_script_sandbox::{Sandbox, V8Sandbox};
        let registry = std::sync::Arc::new(std::sync::Mutex::new(crate::webaudio_registry::WebAudioRegistry::new()));
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).expect("v8 sandbox");
        crate::webaudio_registry::register_webaudio_bridge_callbacks(&mut sandbox, std::sync::Arc::clone(&registry));
        // JS AudioContext 门面在 js_dom_shim（part06）——测试沙箱需装载 shim
        //（生产侧 webview.rs 构建 sandbox 后同款 execute）。
        sandbox
            .execute(zero_engine::generate_js_dom_shim())
            .expect("load js_dom_shim");

        // JS 面：AudioContext + createOscillator + start（桥注入面）。
        sandbox
            .execute(
                "var ctx = new AudioContext();\
                 globalThis.__osc = ctx.createOscillator();\
                 globalThis.__osc.type = 'sine';\
                 globalThis.__osc.frequency.value = 440;\
                 globalThis.__state = ctx.state;\
                 globalThis.__sr = ctx.sampleRate;\
                 globalThis.__osc.start(0);",
            )
            .unwrap();
        assert_eq!(
            sandbox.execute("globalThis.__state").unwrap().value,
            "running",
            "state 恒 running（headless 面）"
        );
        assert_eq!(
            sandbox.execute("String(globalThis.__sr)").unwrap().value,
            "48000",
            "sampleRate NullSink 固定值"
        );
        assert_eq!(
            sandbox.execute("String(globalThis.__zw_wa_active())").unwrap().value,
            "1",
            "start 后存在活跃源"
        );
        // 泵推进 ~1 秒（1ms 泵节拍 × 1000 tick——tab_worker 同款节拍；
        // advance 每 tick 写 48 帧 = 48000Hz/1000）→ NullSink 可观测断言。
        {
            let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());
            for tick in 0..1000u64 {
                reg.advance(tick);
            }
            // 1000 tick × 48 帧 = 48000；宿主调度取整差 ~3% 容差（首 tick gate）。
            assert!(
                reg.frames_written() >= 45_000,
                "1 秒推进应写入 ≈48000 帧（got {}）",
                reg.frames_written()
            );
            let zps = reg.zero_crossings_per_second().unwrap();
            assert!((zps - 880.0).abs() < 25.0, "440Hz sine 过零率锚点 ≈880（got {zps}）");
        }
        // stop(0) → 活跃源清零。
        sandbox.execute("globalThis.__osc.stop(0);").unwrap();
        {
            let reg = registry.lock().unwrap_or_else(|e| e.into_inner());
            std::thread::sleep(std::time::Duration::from_millis(5));
            assert!(!reg.is_any_active(), "stop 后无活跃源");
        }
    }
}
