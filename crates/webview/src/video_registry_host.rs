//! 开态宿主媒体面（`media-process` feature；C2 切片 2 registry 客户端切换）。
//!
//! [`HostMediaFacade`] 承载 [`crate::video_registry::VideoPlayerRegistry`] 在
//! 进程隔离开态下的源面：命令面转发 [`MediaHostClient`]（settle 全量喂入、
//! play/pause/seek/set 面与 `wait_for` 回执），真值面读 push 镜像（DESIGN
//! §10.4——registry getter 本体不变，backing 换镜像）。帧经 [`Self::tick_into`]
//! 排空注入 `ImageCache`（painter 同键）。MSE 源不经 `register_source`（
//! blob:MSE 句柄面独立登记），键不在镜像集即落内联直调——内核无 append 面。
//!
//! 降级纪律（IF-006，C3 重启面）：初始 spawn 失败由 client 置永久错误面后命令恒
//! `false`——不内联回落；运行期 host 失效经 [`MediaHostClient::poll_health`]
//! one-shot 崩溃通知 → 全活跃源广播 error 态（DC-3 页面可观察链真值口），client
//! 转退避重启（1s/2s/4s n≤3，泵 tick 推进）；重启后源全丢失不自动重注册（页面
//! 重新走资源装载面，epoch 重单调）。host `MediaError` 为播放级（镜像 error 态，
//! 不置失败）。旧 epoch 的状态/帧一律丢弃（DC-2「不返回旧代际数据」）。

use std::collections::HashMap;

use zero_protocol::media::{MediaErrorCode, MediaErrorParams, MediaPlaybackState, MediaStateUpdateParams};
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};

use crate::media_proxy::{MediaHostClient, MediaHostEvent};

/// register 后全量喂入的分片大小（64MiB 帧上限内；8MiB 幂对齐）。
const CHUNK_BYTES: usize = 8 * 1024 * 1024;

/// 单源 push 镜像（DC-2：镜像状态只由最新 epoch 消息建立）。
#[derive(Debug, Default, Clone)]
struct HostMirror {
    playing: bool,
    ended: bool,
    error: bool,
    media_time_ms: f64,
    duration_ms: Option<u64>,
    dims: Option<(u32, u32)>,
    /// 见过的最新代际——更低 epoch 的状态/帧一律丢弃（seek/replay/回卷 +1 面）。
    epoch: u64,
    loop_on: bool,
}

impl HostMirror {
    /// 状态跃迁应用（epoch 门控在调用方）。
    fn apply_state(&mut self, state: &MediaPlaybackState) {
        match *state {
            MediaPlaybackState::Playing { media_time_ms } => {
                self.playing = true;
                self.ended = false;
                self.media_time_ms = media_time_ms as f64;
            }
            MediaPlaybackState::Paused { media_time_ms } => {
                self.playing = false;
                self.media_time_ms = media_time_ms as f64;
            }
            MediaPlaybackState::Ended => {
                self.playing = false;
                self.ended = true;
            }
            MediaPlaybackState::Error { .. } => {
                self.playing = false;
                self.error = true;
            }
        }
    }
}

/// 开态宿主面（registry 持有；`MediaHostClient` 懒 spawn——未用媒体页零进程成本）。
pub(crate) struct HostMediaFacade {
    client: MediaHostClient,
    /// 登记过的源镜像（release/clear 同步移除；未知键的推送一律丢弃 = DC-2）。
    mirrors: HashMap<u64, HostMirror>,
    /// 纯视频 Ended + loop 的回卷待办（tick 排空时收集，flush 统一驱动——内核
    /// 不自动回卷纯视频，renderer 侧 play(0) 走 replay 链，§10.7-2 同构）。
    pending_replays: Vec<u64>,
    /// 命令后排空缓冲的帧（play/seek 回执后即时排空只入镜像元数据；帧留给
    /// 下次 tick_into 注入 cache——不丢帧，见 [`Self::drain_meta_after_ack`]）。
    deferred_frames: Vec<zero_protocol::media::MediaFrameParams>,
}

impl Default for HostMediaFacade {
    fn default() -> Self {
        Self::new()
    }
}

impl HostMediaFacade {
    pub(crate) fn new() -> Self {
        Self {
            client: MediaHostClient::new(),
            mirrors: HashMap::new(),
            pending_replays: Vec::new(),
            deferred_frames: Vec::new(),
        }
    }

    /// 键是否由宿主面接管（registry 命令分流判定——登记成功即接管，MSE 键
    /// 不经登记天然内联）。
    pub(crate) fn hosts(&self, key: u64) -> bool {
        self.mirrors.contains_key(&key)
    }

    /// 是否存在宿主面播放中的源（registry `is_any_playing` 并集面）。
    pub(crate) fn is_any_playing(&self) -> bool {
        self.mirrors.values().any(|m| m.playing)
    }

    /// 登记 + 全量喂入（settle 面；内核按内容嗅探路由，mime 记账字段不消费）。
    /// 成功即镜像建档（接管）；失败不建档——后续命令落内联空面（IF-006 不回落）。
    pub(crate) fn register_source(&mut self, key: u64, bytes: &[u8]) -> bool {
        if self.client.register_source(key, bytes.len() as u64, "").is_err() {
            return false;
        }
        for (seq, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
            if self.client.send_data_chunk(key, seq as u64, chunk.to_vec()).is_err() {
                return false;
            }
        }
        self.mirrors.insert(key, HostMirror::default());
        true
    }

    /// 起播/重播。回执载荷即时入镜像（无心跳盲区）；Ended 后 play = 内核
    /// replay 链（epoch+1 重头，registry `Ended→reset` 分支同构）。回执后即时
    /// 排空元数据（内核 SourceReady 先于 Playing 回执同批出队——JS `play()`
    /// 后立读 duration/尺寸即在真值，不等泵 tick）。
    pub(crate) fn play(&mut self, key: u64) -> bool {
        let position = self.mirrors.get(&key).map(|m| m.media_time_ms as u64).unwrap_or(0);
        match self.client.play(key, position) {
            Ok(state) => {
                if let Some(m) = self.mirrors.get_mut(&key) {
                    m.apply_state(&state.state);
                }
                self.drain_meta_after_ack();
                true
            }
            Err(_) => false,
        }
    }

    /// 命令回执后的即时排空：State/SourceReady/Error 入镜像；帧缓冲到
    /// [`Self::deferred_frames`] 由下次 tick_into 注入（try_recv_event 消费面，
    /// 直接丢弃会丢首帧窗）。
    fn drain_meta_after_ack(&mut self) {
        while let Ok(Some(event)) = self.client.try_recv_event() {
            match event {
                MediaHostEvent::Frame(frame) => self.deferred_frames.push(frame),
                other => {
                    self.apply_event(other, None);
                }
            }
        }
    }

    /// 暂停（回执载荷即时入镜像）。
    pub(crate) fn pause(&mut self, key: u64) {
        if let Ok(state) = self.client.pause(key) {
            if let Some(m) = self.mirrors.get_mut(&key) {
                m.apply_state(&state.state);
            }
        }
    }

    /// 精确 seek：`resolved_ms` 即镜像真值（内核回执）；播放态保持（内核面）。
    pub(crate) fn seek(&mut self, key: u64, target_ms: u64) -> bool {
        match self.client.seek(key, target_ms) {
            Ok(result) => {
                if let Some(m) = self.mirrors.get_mut(&key) {
                    m.media_time_ms = result.resolved_ms as f64;
                }
                true
            }
            Err(_) => false,
        }
    }

    /// 播放速率（fire-and-forget；内核只作用视频墙钟——registry 已知限制同面）。
    pub(crate) fn set_playback_rate(&mut self, key: u64, rate: f64) {
        let _ = self.client.set_rate(key, rate);
    }

    /// loop 属性（本地镜像供 Ended 回卷判定 + fire-and-forget 同步宿主）。
    pub(crate) fn set_loop(&mut self, key: u64, on: bool) {
        if let Some(m) = self.mirrors.get_mut(&key) {
            m.loop_on = on;
        }
        let _ = self.client.set_loop(key, on);
    }

    /// 增益/静音（fire-and-forget）。
    pub(crate) fn set_gain(&mut self, key: u64, volume: f64, muted: bool) {
        let _ = self.client.set_gain(key, volume, muted);
    }

    pub(crate) fn current_time(&self, key: u64) -> f64 {
        self.mirrors.get(&key).map(|m| m.media_time_ms / 1000.0).unwrap_or(0.0)
    }

    pub(crate) fn duration(&self, key: u64) -> Option<f64> {
        self.mirrors
            .get(&key)
            .and_then(|m| m.duration_ms)
            .map(|ms| ms as f64 / 1000.0)
    }

    /// 媒体元 (w, h, durMs)——`SourceReady` 推送后就位（play 建解码器后，
    /// §10.7-6）；未就绪 None（shim 保持占位态不升级）。
    pub(crate) fn source_media_meta(&self, key: u64) -> Option<(u32, u32, u64)> {
        let m = self.mirrors.get(&key)?;
        Some((m.dims?.0, m.dims?.1, m.duration_ms.unwrap_or(0)))
    }

    /// 固有尺寸——`SourceReady` 前不可得（内核喂入未齐 demux 不可行），落
    /// (0,0) 占位；升级时机偏差记 DESIGN §10.7（videoWidth 延后到 play 后）。
    pub(crate) fn probe_dimensions(&self, key: u64) -> (u32, u32) {
        self.mirrors.get(&key).and_then(|m| m.dims).unwrap_or((0, 0))
    }

    pub(crate) fn is_playing(&self, key: u64) -> bool {
        self.mirrors.get(&key).is_some_and(|m| m.playing)
    }

    /// 源错误态（崩溃广播/播放级错误镜像面——桥 `isErrored` 真值口，DC-3 页面
    /// 可观察链；§11.3）。
    pub(crate) fn is_errored(&self, key: u64) -> bool {
        self.mirrors.get(&key).is_some_and(|m| m.error)
    }

    /// 宿主进程 pid（诊断/契约测试 SIGKILL 注入面；未连接 `None`）。
    pub(crate) fn host_pid(&self) -> Option<u32> {
        self.client.host_pid()
    }

    pub(crate) fn is_ended(&self, key: u64) -> bool {
        self.mirrors.get(&key).is_some_and(|m| m.ended)
    }

    /// 释放源（导航/`release` 面）：宿主释放 + 镜像移除（未知键的后续推送丢弃）。
    pub(crate) fn release(&mut self, key: u64) {
        let _ = self.client.release_source(key);
        self.mirrors.remove(&key);
    }

    /// 导航清面：逐键释放 + 镜像清空 + client teardown（§11.4 reload 恢复路径
    /// ——状态机回 Idle，下次登记 spawn 全新 host；崩溃 Failed 态随导航清除）。
    pub(crate) fn clear(&mut self) {
        let keys: Vec<u64> = self.mirrors.keys().copied().collect();
        for key in keys {
            let _ = self.client.release_source(key);
        }
        self.mirrors.clear();
        self.pending_replays.clear();
        let _ = self.client.shutdown();
    }

    /// 泵推进（16ms 节拍被动消费面，DESIGN §10.4）：健康巡检（崩溃检测兜底路 +
    /// 重启退避推进，§11.2-b）→ 排空 host 推送 → 状态更新镜像 / 帧注入
    /// `image_cache`（painter 同键）/ Ended+loop 回卷待办。返回是否有帧更新。
    pub(crate) fn tick_into(&mut self, image_cache: &mut ImageCache) -> bool {
        let mut changed = false;
        // 崩溃检测兜底路（§11.2-b）：one-shot 通知 → 全活跃源广播 error 态
        // （§11.3——桥 `isErrored` 真值口，DC-3 页面可观察链）。
        if self.client.poll_health().is_err() {
            self.broadcast_host_error();
        }
        // 命令后排空缓冲的帧先行注入（play/seek 回执窗内到达的首帧不丢）。
        for frame in std::mem::take(&mut self.deferred_frames) {
            changed |= self.apply_event(MediaHostEvent::Frame(frame), Some(image_cache));
        }
        while let Ok(Some(event)) = self.client.try_recv_event() {
            changed |= self.apply_event(event, Some(image_cache));
        }
        changed
    }

    /// 崩溃广播（§11.3）：全部活跃镜像置 error 态（playing 清除；epoch 不变——
    /// 重启后新流 epoch 重单调，旧代际推送天然失效）。回卷待办随连接失效作废。
    fn broadcast_host_error(&mut self) {
        for m in self.mirrors.values_mut() {
            m.apply_state(&MediaPlaybackState::Error {
                code: MediaErrorCode::HostInternal,
            });
        }
        self.pending_replays.clear();
    }

    /// 驱动排空时收集的纯视频 loop 回卷（registry tick 面调用）。内核 replay
    /// 链重头（epoch+1，`rebuild_for_replay` 归零游标）后还会对 `position_ms`
    /// 做对齐 seek——镜像时钟须先归零，否则 play(dur) 会 seek 回流末瞬间再
    /// Ended（loop 失效）。
    pub(crate) fn flush_replays(&mut self) {
        for key in std::mem::take(&mut self.pending_replays) {
            if let Some(m) = self.mirrors.get_mut(&key) {
                m.media_time_ms = 0.0;
            }
            let _ = self.play(key);
        }
    }

    /// 单事件应用（镜像/帧/回卷待办；cache 传 `None` = 仅镜像元数据——命令
    /// 回执后排空面无 cache 可注入）。返回是否注入了帧。
    fn apply_event(&mut self, event: MediaHostEvent, mut image_cache: Option<&mut ImageCache>) -> bool {
        match event {
            MediaHostEvent::State(update) => {
                let MediaStateUpdateParams {
                    source_key,
                    epoch,
                    state,
                } = update;
                let Some(m) = self.mirrors.get_mut(&source_key) else {
                    return false;
                };
                if epoch < m.epoch {
                    return false; // 旧代际丢弃（DC-2）
                }
                m.epoch = epoch;
                m.apply_state(&state);
                // 纯视频 Ended + loop：内核不自动回卷——收集回卷待办，flush 统一
                // play(0)（内核 replay 链；纯音频/A-V 音频轨回卷在内核内完成）。
                if m.ended && m.loop_on {
                    self.pending_replays.push(source_key);
                }
                false
            }
            MediaHostEvent::Frame(frame) => {
                let Some(m) = self.mirrors.get_mut(&frame.source_key) else {
                    return false;
                };
                if frame.epoch < m.epoch {
                    return false; // 旧代际帧丢弃（DC-2）
                }
                // 帧即时钟（§10.3 推送时机 2——pts_ms 自带时钟信息）。
                m.media_time_ms = frame.pts_ms as f64;
                // FIXME(media-process): shm 传输帧（`shm_name` 非空、`rgba` 空）
                // 目前内核发射面未接 shm，全内联；帧路径接入时补 /dev/shm 读面。
                if frame.rgba.len() == frame.width as usize * frame.height as usize * 4 {
                    if let (Some(cache), Ok(data)) = (
                        image_cache.as_deref_mut(),
                        ImageData::from_rgba_variable(frame.rgba, frame.width, frame.height),
                    ) {
                        cache.insert_with_key(ImageKey::new(frame.source_key), data);
                        return true;
                    }
                }
                false
            }
            MediaHostEvent::SourceReady(ready) => {
                if let Some(m) = self.mirrors.get_mut(&ready.source_key) {
                    m.duration_ms = ready.duration_ms;
                    m.dims = ready.width.zip(ready.height);
                }
                false
            }
            MediaHostEvent::Error(err) => {
                let MediaErrorParams {
                    source_key,
                    epoch,
                    code,
                    ..
                } = err;
                if let Some(m) = self.mirrors.get_mut(&source_key) {
                    if epoch < m.epoch {
                        return false;
                    }
                    m.epoch = epoch;
                    m.apply_state(&MediaPlaybackState::Error { code });
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::media::{MediaErrorCode, MediaFrameParams, MediaStateUpdateParams};

    fn state_event(key: u64, epoch: u64, state: MediaPlaybackState) -> MediaHostEvent {
        MediaHostEvent::State(MediaStateUpdateParams {
            source_key: key,
            epoch,
            state,
        })
    }

    fn frame_event(key: u64, epoch: u64, pts_ms: u64) -> MediaHostEvent {
        MediaHostEvent::Frame(MediaFrameParams {
            source_key: key,
            epoch,
            pts_ms,
            key: false,
            width: 2,
            height: 2,
            rgba: vec![7; 16],
            shm_name: None,
        })
    }

    /// 建档镜像（绕开 client——镜像/事件逻辑与传输面分离可测）。
    fn facade_with(key: u64) -> HostMediaFacade {
        let mut facade = HostMediaFacade::new();
        facade.mirrors.insert(key, HostMirror::default());
        facade
    }

    #[test]
    fn mirror_state_transitions_track_push() {
        let mut facade = facade_with(7);
        assert!(!facade.is_playing(7));

        facade.apply_event(
            state_event(7, 0, MediaPlaybackState::Playing { media_time_ms: 120 }),
            None,
        );
        assert!(facade.is_playing(7));
        assert_eq!(facade.current_time(7), 0.12);

        facade.apply_event(
            state_event(7, 0, MediaPlaybackState::Paused { media_time_ms: 200 }),
            None,
        );
        assert!(!facade.is_playing(7));
        assert_eq!(facade.current_time(7), 0.2);

        facade.apply_event(state_event(7, 0, MediaPlaybackState::Ended), None);
        assert!(facade.is_ended(7));
        assert!(!facade.is_playing(7));
    }

    /// DC-2：旧代际的状态与帧一律丢弃——镜像不回退、旧帧不注入。
    #[test]
    fn stale_epoch_state_and_frames_are_dropped() {
        let mut facade = facade_with(7);
        facade.mirrors.get_mut(&7).expect("seeded").epoch = 3;

        let changed = facade.apply_event(
            state_event(7, 2, MediaPlaybackState::Playing { media_time_ms: 999 }),
            None,
        );
        assert!(!changed);
        assert!(!facade.is_playing(7), "旧代际状态不得置播放态");
        assert_eq!(facade.current_time(7), 0.0, "旧代际时钟不得回退镜像");

        assert!(!facade.apply_event(frame_event(7, 2, 50), None));
    }

    /// 帧注入：尺寸自洽入 cache（painter 同键）且帧即时钟推进镜像；无 cache
    /// 面（命令回执后排空窗）时钟照推、不注入（帧缓冲待 tick 注入）。
    #[test]
    fn frames_inject_into_cache_and_advance_mirror_clock() {
        let mut facade = facade_with(7);
        assert!(!facade.apply_event(frame_event(7, 0, 40), None));
        assert_eq!(facade.current_time(7), 0.04, "镜像时钟照推（元数据面）");
        let mut cache = ImageCache::default();
        assert!(facade.apply_event(frame_event(7, 0, 83), Some(&mut cache)));
        assert_eq!(facade.current_time(7), 0.083);
        assert_eq!(cache.len(), 1, "有 cache 才注入");
    }

    /// 纯视频 Ended + loop：回卷待办收集（flush 统一 play(0)——内核 replay 链）。
    #[test]
    fn ended_with_loop_defers_replay() {
        let mut facade = facade_with(7);
        facade.mirrors.get_mut(&7).expect("seeded").loop_on = true;

        facade.apply_event(state_event(7, 0, MediaPlaybackState::Ended), None);
        assert!(facade.is_ended(7), "回卷前镜像保持 Ended（flush 后由 replay 回执覆盖）");
        assert_eq!(facade.pending_replays, vec![7]);

        facade.apply_event(
            state_event(7, 1, MediaPlaybackState::Playing { media_time_ms: 0 }),
            None,
        );
        assert!(facade.is_playing(7));
        assert!(!facade.is_ended(7));
    }

    /// 错误推送置播放级错误面（不置永久失败——IF-006 边界；旧代际错误丢弃）。
    #[test]
    fn error_push_marks_mirror_not_permanent() {
        let mut facade = facade_with(7);
        facade.mirrors.get_mut(&7).expect("seeded").epoch = 5;
        facade.mirrors.get_mut(&7).expect("seeded").playing = true;

        facade.apply_event(
            MediaHostEvent::Error(MediaErrorParams {
                source_key: 7,
                epoch: 2,
                code: MediaErrorCode::DecodeFailure,
                message: "old epoch".into(),
            }),
            None,
        );
        assert!(facade.is_playing(7), "旧代际错误不得改镜像");

        facade.apply_event(
            MediaHostEvent::Error(MediaErrorParams {
                source_key: 7,
                epoch: 5,
                code: MediaErrorCode::DecodeFailure,
                message: "decode failed".into(),
            }),
            None,
        );
        assert!(!facade.is_playing(7));
        assert!(!facade.client.is_failed());
    }

    /// 崩溃广播（C3 §11.3）：全部活跃镜像置 error 态（playing 清除，DC-3 页面
    /// 可观察真值）+ 回卷待办随连接失效作废。
    #[test]
    fn host_crash_broadcasts_error_to_all_mirrors() {
        let mut facade = facade_with(7);
        facade.mirrors.insert(8, HostMirror::default());
        facade.mirrors.get_mut(&7).expect("seeded").playing = true;
        facade.mirrors.get_mut(&8).expect("seeded").playing = true;
        facade.pending_replays.push(7);

        facade.broadcast_host_error();
        assert!(!facade.is_playing(7) && !facade.is_playing(8));
        assert!(facade.is_errored(7) && facade.is_errored(8));
        assert!(facade.pending_replays.is_empty(), "回卷待办随连接失效作废");
    }

    /// SourceReady 升级镜像元数据（duration/尺寸——桥升级面真值）。
    #[test]
    fn source_ready_upgrades_meta() {
        let mut facade = facade_with(7);
        assert_eq!(facade.source_media_meta(7), None);
        assert_eq!(facade.probe_dimensions(7), (0, 0));

        facade.apply_event(
            MediaHostEvent::SourceReady(zero_protocol::media::MediaSourceReadyParams {
                source_key: 7,
                duration_ms: Some(2000),
                width: Some(320),
                height: Some(240),
            }),
            None,
        );
        assert_eq!(facade.source_media_meta(7), Some((320, 240, 2000)));
        assert_eq!(facade.probe_dimensions(7), (320, 240));
        assert_eq!(facade.duration(7), Some(2.0));
    }

    /// release 移除镜像：后续推送（迟到帧）丢弃 = DC-2 未知键面。
    #[test]
    fn released_key_pushes_are_dropped() {
        let mut facade = facade_with(7);
        facade.release(7);
        assert!(!facade.hosts(7));
        assert!(!facade.apply_event(frame_event(7, 0, 10), None));
    }
}
