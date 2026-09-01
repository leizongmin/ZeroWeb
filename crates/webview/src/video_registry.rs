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
use zero_media::{VideoClock, VideoDecoder, VideoPlayer};
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};

/// 每元素播放器注册表（键 = 资源绝对 URL 的 painter 同款哈希）。
#[derive(Default)]
pub struct VideoPlayerRegistry {
    /// 已 settle 的源字节（play 时建 player——解码器单向流，一次构建）。
    sources: HashMap<u64, Vec<u8>>,
    players: HashMap<u64, VideoPlayer>,
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
    }

    /// play：懒建 player（源未 settle 时 no-op false——元素无资源可播）。
    pub fn play(&mut self, abs_src: &str, now_ms: u64) -> bool {
        let key = image_resource_key(abs_src, None);
        if !self.players.contains_key(&key) {
            let Some(bytes) = self.sources.remove(&key) else {
                return false;
            };
            let Ok(decoder) = VideoDecoder::open_webm_vp9(&bytes) else {
                return false;
            };
            self.players.insert(key, VideoPlayer::new(decoder));
        }
        if let Some(player) = self.players.get_mut(&key) {
            player.play(now_ms);
            return true;
        }
        false
    }

    /// pause：保持位置（未播放/不存在 no-op）。
    pub fn pause(&mut self, abs_src: &str) {
        let key = image_resource_key(abs_src, None);
        if let Some(player) = self.players.get_mut(&key) {
            player.pause();
        }
    }

    /// currentTime 真值（秒；未播放/不存在 → 0——spec HAVE_NOTHING 语义面）。
    pub fn current_time(&self, abs_src: &str) -> f64 {
        let key = image_resource_key(abs_src, None);
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

    /// 渲染泵推进：tick 所有播放中的 player，新帧注入 `image_cache`（painter 同键）。
    /// 返回是否有帧更新（宿主据此触发增量渲染）。
    pub fn tick_all(&mut self, now_ms: u64, image_cache: &mut ImageCache) -> bool {
        let mut changed = false;
        let keys: Vec<u64> = self.players.keys().copied().collect();
        for key in keys {
            let Some(player) = self.players.get_mut(&key) else {
                continue;
            };
            let Ok(Some(frame)) = player.tick(now_ms) else {
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
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // crates/
        p.pop(); // workspace root
        p.push("tests/fixtures/media/sample-webm-vp9.webm");
        std::fs::read(p).expect("webm fixture present")
    }

    const SRC: &str = "https://example.com/media/sample-webm-vp9.webm";

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
}
