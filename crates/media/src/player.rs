//! 播放驱动 — 帧率时钟 + play/pause/ended + currentTime 真值化（media-playback M2a）。
//!
//! RFC §3.1 player 模块：对上实现 [`VideoClock`]（media-elements 语义层的
//! headless 近似驱动源替换点——语义层只换驱动源，不返工），对下驱动
//! [`VideoDecoder`](crate::VideoDecoder) 逐帧推进。
//!
//! 时序模型：调用方注入单调时钟 `now_ms`（可测试性；生产侧挂 rAF event loop
//! P1a 底座）。`tick(now)` 推进播放位置（`Δt × playbackRate`），把 `pts ≤
//! currentTime` 的帧依序产出（落后多帧时逐帧快进到最新可展示帧——低帧率时钟
//! 下不积压）。静音播放（首期无音频面）；seek/playbackRate 变速归 M2b。

use crate::clock::VideoClock;
use crate::decode::{DecodedVideoFrame, VideoDecoder};

/// 播放器状态机（spec readyState 推进的真值源；事件派发归语义层）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    /// 有解码器、未播放（对应 HAVE_METADATA——首帧未呈现前）。
    Ready,
    /// 播放中（时钟推进）。
    Playing,
    /// 已到流末且末帧已呈现（ended；spec 「ended playback」态）。
    Ended,
}

/// 帧率驱动播放器 — 包装解码器 + 播放时钟。
pub struct VideoPlayer {
    decoder: VideoDecoder,
    state: PlayerState,
    /// 当前播放位置（媒体时间轴毫秒；play 起点为 0 或暂停时的保持值）。
    position_ms: f64,
    /// 上次 tick 的墙钟（推进增量基准；None = 未起播）。
    last_tick_ms: Option<u64>,
    /// 播放速率（0 非法；trait 契约由实现方 clamp——此处拒绝置 0 以下）。
    playback_rate: f64,
    /// 已呈现的最后一帧 pts（ended 判定 + 换帧去重）。
    presented_pts: Option<u64>,
}

impl VideoPlayer {
    /// 从解码器构建播放器（初始 Ready，位置 0）。
    pub fn new(decoder: VideoDecoder) -> Self {
        Self {
            decoder,
            state: PlayerState::Ready,
            position_ms: 0.0,
            last_tick_ms: None,
            playback_rate: 1.0,
            presented_pts: None,
        }
    }

    /// 开始播放（`now_ms` 为本次起播的墙钟锚点；Ended 后再 play = 重头播放——
    /// spec 「if ended and playback direction forwards, seek to earliest position」）。
    pub fn play(&mut self, now_ms: u64) {
        if self.state == PlayerState::Ended {
            self.position_ms = 0.0;
            self.presented_pts = None;
            // 重头播放需新解码器：由调用方经 reset() 提供（解码器单向流）。
        }
        self.state = PlayerState::Playing;
        self.last_tick_ms = Some(now_ms);
    }

    /// 暂停（保持当前位置；tick 不再推进）。
    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.state = PlayerState::Ready;
            self.last_tick_ms = None;
        }
    }

    /// Ended 后重头播放的解码器复位（替换底层解码器，位置清零）。
    pub fn reset(&mut self, decoder: VideoDecoder) {
        self.decoder = decoder;
        self.state = PlayerState::Ready;
        self.position_ms = 0.0;
        self.last_tick_ms = None;
        self.presented_pts = None;
    }

    /// seek 到目标位置（毫秒）——M2b 精确 seek（spec「seek」算法：
    /// 位置设为 target，解码器定位（关键帧粒度 + 前向精确解码），下一 tick
    /// 呈现 ≥ target 的首帧；播放中 seek 保持播放（时钟锚点重置防 Δt 跳变），
    /// 暂停中 seek 保持暂停（spec「seeking 不改 paused」）。
    /// https://html.spec.whatwg.org/multipage/media.html#seek
    pub fn seek_to_ms(&mut self, target_ms: u64) -> Result<(), crate::DecodeError> {
        // clamp 到 [0, duration]（spec seek 步 5：位置 clamp 到可 seek 范围）。
        let clamped = match self.duration() {
            Some(d) => (target_ms as f64).min(d * 1000.0).max(0.0) as u64,
            None => target_ms,
        };
        self.decoder.seek_to_ms(clamped)?;
        self.position_ms = clamped as f64;
        // 时钟锚点重置：下次 tick 从现在起算 Δt（播放中 seek 无跳变）。
        self.last_tick_ms = None;
        self.presented_pts = None;
        Ok(())
    }

    /// 设置播放速率（clamp 到 (0, 16]——spec dom-media-playbackrate「归零即静默
    /// 忽略」的保守面；变速精细语义归 M2b）。
    pub fn set_playback_rate(&mut self, rate: f64) {
        if rate.is_finite() && rate > 0.0 {
            self.playback_rate = rate.min(16.0);
        }
    }

    /// 推进时钟并产出应展示的帧（`now_ms` 单调；pause 期间调用为 no-op 返回 None）。
    ///
    /// 帧调度：把播放位置推到 `now`，将解码器中 `pts ≤ position` 的帧依次弹出，
    /// 返回**最后一帧**（最新可展示帧；中间帧跳过——时钟落后时不积压）。
    /// 解码器耗尽 → 转 Ended。
    pub fn tick(&mut self, now_ms: u64) -> Result<Option<DecodedVideoFrame>, crate::DecodeError> {
        if self.state != PlayerState::Playing {
            return Ok(None);
        }
        let last = self.last_tick_ms.unwrap_or(now_ms);
        self.last_tick_ms = Some(now_ms);
        let delta = now_ms.saturating_sub(last) as f64;
        self.position_ms += delta * self.playback_rate;
        self.present_pending()
    }

    /// 主时钟对齐呈现（A/V 同步——audio clock 主时钟，media-audio M2 契约）：
    /// 把播放位置对齐到外部主时钟游标后按同一帧调度呈现。
    ///
    /// 位置只前进不回退（播放期主时钟游标单调）；`media_ms` 落后时保持现位置
    /// （主时钟停 → 视频帧调度停——master clock 语义）。drift 由构造校正：位置
    /// 每 tick 派生自主时钟游标，不积累墙钟差。
    /// https://html.spec.whatwg.org/multipage/media.html#synchronising-multiple-media-elements
    pub fn sync_to_media_time(&mut self, media_ms: f64) -> Result<Option<DecodedVideoFrame>, crate::DecodeError> {
        if self.state != PlayerState::Playing {
            return Ok(None);
        }
        if media_ms > self.position_ms {
            self.position_ms = media_ms;
        }
        self.present_pending()
    }

    /// 帧调度共用核：弹出 `pts ≤ position` 的帧并返回最新可展示帧（tick 与
    /// sync_to_media_time 的公共尾部）。
    fn present_pending(&mut self) -> Result<Option<DecodedVideoFrame>, crate::DecodeError> {
        let mut newest: Option<DecodedVideoFrame> = None;
        loop {
            match self.decoder.next_frame()? {
                Some(frame) => {
                    if frame.pts_ms as f64 <= self.position_ms {
                        newest = Some(frame);
                    } else {
                        // 首帧超出播放位置：未来帧——**退回解码器队首**（R3936）。
                        // 旧形态把它返回给调用方（渲染后丢弃），该时间槽永久
                        // 丢失；粗 tick 背压下逐 tick 丢未来帧使解码器在
                        // position < duration 处提前耗尽（Ended 早于流末——
                        // track-cues-enter-exit 的 cue@4-5s 永不触发的根因）。
                        // spec ended：「currentTime 到达媒体资源末尾」——帧调度
                        // 不得超越时钟消费时间线。
                        self.decoder.un_read(frame);
                        break;
                    }
                }
                None => {
                    self.state = PlayerState::Ended;
                    break;
                }
            }
            if self.state == PlayerState::Ended {
                break;
            }
        }
        if let Some(frame) = &newest {
            self.presented_pts = Some(frame.pts_ms);
        }
        Ok(newest)
    }

    /// 当前状态。
    pub fn state(&self) -> PlayerState {
        self.state
    }

    /// 最后一帧 pts（ms；未呈现任何帧时 None）。
    pub fn presented_pts_ms(&self) -> Option<u64> {
        self.presented_pts
    }
}

impl VideoClock for VideoPlayer {
    fn current_time(&self) -> f64 {
        self.position_ms / 1000.0
    }

    fn duration(&self) -> Option<f64> {
        self.decoder.duration_ms().map(|ms| ms as f64 / 1000.0)
    }

    fn is_playing(&self) -> bool {
        self.state == PlayerState::Playing
    }

    fn playback_rate(&self) -> f64 {
        self.playback_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::fixture_path;
    use std::fs;

    fn fixture_player() -> VideoPlayer {
        let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
        VideoPlayer::new(VideoDecoder::open_webm_vp9(&data).unwrap())
    }

    #[test]
    fn player_initial_state_ready_with_duration() {
        let p = fixture_player();
        assert_eq!(p.state(), PlayerState::Ready);
        assert!(!p.is_playing());
        assert_eq!(p.current_time(), 0.0);
        // fixture duration 2000ms（M1a 实测）。
        assert_eq!(p.duration(), Some(2.0));
    }

    #[test]
    fn player_tick_advances_clock_and_presents_frames_in_order() {
        let mut p = fixture_player();
        p.play(1000);
        assert!(p.is_playing());
        // 首 tick：+16ms → 帧率 24fps（≈41.7ms/帧）下首帧 pts=0 应呈现。
        let f = p.tick(1016).unwrap().expect("frame at t=16ms");
        assert_eq!(f.pts_ms, 0);
        assert_eq!(p.presented_pts_ms(), Some(0));
        // +42ms → 位置 ≈58ms → pts 0 与 41.67ms 两帧弹出，呈现后者。
        let f2 = p.tick(1058).unwrap().expect("frame at t=58ms");
        assert!(f2.pts_ms > 0, "第二帧应为后续帧，got pts={}", f2.pts_ms);
        assert_eq!(p.presented_pts_ms(), Some(f2.pts_ms));
        assert!(p.current_time() > 0.0);
    }

    #[test]
    fn player_pause_freezes_clock() {
        let mut p = fixture_player();
        p.play(0);
        p.tick(50).unwrap();
        let pos = p.current_time();
        p.pause();
        assert!(!p.is_playing());
        assert_eq!(p.current_time(), pos, "暂停保持位置");
        assert!(p.tick(1000).unwrap().is_none(), "暂停期 tick 不产出帧");
        assert_eq!(p.current_time(), pos, "暂停期位置不动");
    }

    #[test]
    fn player_reaches_ended_after_stream_exhausted() {
        let mut p = fixture_player();
        p.play(0);
        // 大步进 tick 快进到流末（48 帧 / 2s；单 tick 10s 必耗尽）。
        let mut frames = 0;
        let mut now = 0u64;
        while p.state() != PlayerState::Ended {
            now += 500;
            if p.tick(now).unwrap().is_some() {
                frames += 1;
            }
            assert!(now < 60_000, "runaway loop");
        }
        assert!(frames > 0, "ended 前应呈现过帧");
        assert!(!p.is_playing());
        // ended 后 tick 无帧、位置不越界（fixture 2000ms + 容差）。
        let _ = p.tick(60_000).unwrap();
        assert!(p.current_time() <= 10.5, "ended 后位置不应无限增长");
    }

    #[test]
    fn player_playback_rate_scales_and_clamps() {
        let mut p = fixture_player();
        p.play(0);
        p.set_playback_rate(2.0);
        assert_eq!(p.playback_rate(), 2.0);
        p.tick(100).unwrap();
        assert!(
            (p.current_time() - 0.2).abs() < 1e-9,
            "2x 速率：100ms 墙钟 = 200ms 媒体时间"
        );
        // 非法值静默忽略（spec dom-media-playbackrate 保守面）。
        p.set_playback_rate(0.0);
        p.set_playback_rate(-1.0);
        p.set_playback_rate(f64::NAN);
        assert_eq!(p.playback_rate(), 2.0);
        p.set_playback_rate(100.0);
        assert_eq!(p.playback_rate(), 16.0, "上界 clamp 16");
    }

    #[test]
    fn player_seek_mid_stream_and_resume() {
        // M2b：seek 后位置 = target、下一 tick 呈现 ≥ target 首帧、时钟续走不跳变。
        let mut p = fixture_player();
        p.play(0);
        p.tick(16).unwrap(); // 首帧
        p.seek_to_ms(1000).unwrap();
        assert!((p.current_time() - 1.0).abs() < 1e-9, "seek 后位置 = target");
        assert_eq!(p.state(), PlayerState::Playing, "播放中 seek 保持播放");
        // tick：呈现 ≥ 1000ms 首帧（precise-seek），后续 PTS 单调。
        let f = p.tick(2000).unwrap().expect("seek 后 tick 应有帧");
        assert!(f.pts_ms >= 1000, "seek 后呈现帧应 ≥ target，got {}", f.pts_ms);
        // 暂停中 seek：保持暂停（spec）。
        p.pause();
        p.seek_to_ms(500).unwrap();
        assert_eq!(p.state(), PlayerState::Ready, "暂停中 seek 不改 paused");
        assert!((p.current_time() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn player_seek_clamps_to_duration() {
        let mut p = fixture_player();
        p.seek_to_ms(99_999).unwrap();
        assert!((p.current_time() - 2.0).abs() < 1e-6, "越界 seek clamp 到 duration");
    }

    #[test]
    fn player_sync_to_media_time_follows_master_clock() {
        // A/V 同步（audio clock 主时钟）：视频呈现对齐外部游标——每 tick 位置
        // 派生自主时钟（构造 drift 校正），主时钟停则视频帧调度停。
        let mut p = fixture_player();
        p.play(1000);
        // 主时钟游标 20ms → 呈现首帧（pts 0 ≤ 20；下一帧 41.7 未到）。
        let f = p.sync_to_media_time(20.0).unwrap().expect("pts 0 ≤ 20ms");
        assert_eq!(f.pts_ms, 0);
        // 主时钟推进到 120ms → 呈现 ≥120ms 前最新帧（pts ≈41.7/83.3）。
        let f2 = p.sync_to_media_time(120.0).unwrap().expect("后续帧");
        assert!(f2.pts_ms > 0, "游标推进后应呈现后续帧");
        assert!((p.current_time() - 0.120).abs() < 1e-9, "位置 = 主时钟游标");
        // 主时钟停滞（游标不前进）→ 不再弹出新帧（位置不回退也不自走）。
        let held = p.sync_to_media_time(120.0).unwrap();
        assert!(held.is_none() || p.current_time() <= 0.120, "主时钟停 → 位置停");
        assert!((p.current_time() - 0.120).abs() < 1e-9, "位置不回退");
        // 主时钟倒退（异常序）：保持现位置（只前进不回退）。
        let _ = p.sync_to_media_time(10.0).unwrap();
        assert!((p.current_time() - 0.120).abs() < 1e-9, "游标倒退不回退位置");
        // Ready（未播放）态 no-op。
        let mut q = fixture_player();
        assert!(q.sync_to_media_time(100.0).unwrap().is_none());
        assert_eq!(q.current_time(), 0.0, "未播放时 sync 不动位置");
    }

    #[test]
    fn player_reset_replays_from_zero() {
        let mut p = fixture_player();
        p.play(0);
        let mut now = 0u64;
        while p.state() != PlayerState::Ended {
            now += 500;
            p.tick(now).unwrap();
            assert!(now < 60_000);
        }
        // reset 换新解码器 → Ready、位置 0、可再播。
        let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
        p.reset(VideoDecoder::open_webm_vp9(&data).unwrap());
        assert_eq!(p.state(), PlayerState::Ready);
        assert_eq!(p.current_time(), 0.0);
        p.play(now);
        let f = p.tick(now + 16).unwrap().expect("reset 后重播首帧");
        assert_eq!(f.pts_ms, 0);
    }
}
