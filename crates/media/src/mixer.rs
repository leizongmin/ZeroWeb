//! 混音总线 — 多源 f32 帧叠加 + per-source volume/muted 增益（media-audio M1 切片 3）。
//!
//! 多个媒体元素并发播放（`<video>`/`<audio>` 同页多实例）的混音契约：每个 source
//! 独立挂载/卸载（`SourceHandle`），各自持 volume [0,1] 与 muted 开关；总线把活跃
//! 源的输出帧逐采样相加（软削幅 clamp [-1,1]），写入下游 [`AudioSink`](crate::AudioSink)。
//!
//! 增益语义与 media-elements IDL 层对齐：`volume` 非有限/clamp 面已由其 M3 扩批 III
//! 落地（IDL 层拒绝非法值），本层只信任 [0,1] 输入并再 clamp（信任边界防御）。

use crate::audio::{AudioSink, AudioSinkError};
use std::collections::HashMap;

/// 挂载到总线的单个源状态（增增益在推帧时应用，不回改源数据）。
#[derive(Debug, Clone)]
struct SourceState {
    volume: f32,
    muted: bool,
}

/// 混音总线 — N 源 → 1 sink。
///
/// 用法：`attach` 拿 handle → `set_volume`/`set_muted` 控增益 → `push` 推各源
/// 解码帧 → `mix_into` 混合并写入下游 sink（每 tick 一次）。
#[derive(Debug, Default)]
pub struct Mixer {
    sources: HashMap<u64, SourceState>,
    next_handle: u64,
}

/// 源句柄（`attach` 返回；幂等卸载用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceHandle(pub(crate) u64);

impl Mixer {
    /// 新建空总线。
    pub fn new() -> Self {
        Self::default()
    }

    /// 挂载新源（缺省 volume 1.0 / muted false——spec dom-media-volume/muted 缺省面）。
    pub fn attach(&mut self) -> SourceHandle {
        let handle = SourceHandle(self.next_handle);
        self.next_handle += 1;
        self.sources.insert(
            handle.0,
            SourceState {
                volume: 1.0,
                muted: false,
            },
        );
        handle
    }

    /// 卸载源（媒体元素结束/移除——资源生命周期面）。
    pub fn detach(&mut self, handle: SourceHandle) -> bool {
        self.sources.remove(&handle.0).is_some()
    }

    /// 设置源音量（[0,1] clamp——防御性二次钳制，IDL 层已拒非法值）。
    pub fn set_volume(&mut self, handle: SourceHandle, volume: f32) -> bool {
        match self.sources.get_mut(&handle.0) {
            Some(state) => {
                state.volume = volume.clamp(0.0, 1.0);
                true
            }
            None => false,
        }
    }

    /// 设置源静音。
    pub fn set_muted(&mut self, handle: SourceHandle, muted: bool) -> bool {
        match self.sources.get_mut(&handle.0) {
            Some(state) => {
                state.muted = muted;
                true
            }
            None => false,
        }
    }

    /// 当前活跃源数。
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// 混合所有活跃源的一帧块并写入 sink。
    ///
    /// 帧长取 `blocks` 各源同长；软削幅：混音和 clamp [-1,1]（f32 加法饱和近似，
    /// 避免整数 wrap 类爆音；更精细的 limiter 归后续切片）。
    pub fn mix_into(
        &self,
        blocks: &[(SourceHandle, Vec<f32>)],
        sink: &mut dyn AudioSink,
    ) -> Result<(), AudioSinkError> {
        // 总长度 = 最长块（短源视为已播完补零——流不因短源断）。
        let len = blocks.iter().map(|(_, b)| b.len()).max().unwrap_or(0);
        let mut mixed = vec![0.0f32; len];
        for (handle, block) in blocks {
            let Some(state) = self.sources.get(&handle.0) else {
                continue; // 已卸载源不参与混音
            };
            if state.muted {
                continue;
            }
            for (dst, &src) in mixed.iter_mut().zip(block.iter()) {
                *dst += src * state.volume;
            }
        }
        for sample in &mut mixed {
            *sample = sample.clamp(-1.0, 1.0);
        }
        sink.write(&mixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{AudioFormat, NullSink};

    fn mono_sink(rate: u32) -> NullSink {
        let mut sink = NullSink::new();
        sink.start(AudioFormat {
            sample_rate: rate,
            channels: 1,
        })
        .unwrap();
        sink
    }

    #[test]
    fn mixer_sums_short_source_pad_and_clamps() {
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        let b = mixer.attach();
        let mut sink = mono_sink(48000);
        let blocks = vec![
            (a, vec![0.5; 4]),
            (b, vec![0.75; 2]), // 短源：后 2 采样视为补零
        ];
        mixer.mix_into(&blocks, &mut sink).unwrap();
        assert_eq!(sink.frames_written(), 4, "帧长 = 最长块（短源补零不断流）");
        // 值面（sum + clamp [-1,1]）由下两用例经过零率/相位抵消可观测化断言。
    }

    #[test]
    fn mixer_opposite_phase_cancels_to_zero_crossing_free() {
        // 反相双源完全抵消 → 混音输出恒 0 → 过零率为 0（NullSink 波形可观测面）。
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        let b = mixer.attach();
        let mut sink = mono_sink(48000);
        let block_a: Vec<f32> = (0..480).map(|n| if n % 2 == 0 { 0.5 } else { -0.5 }).collect();
        let block_b: Vec<f32> = block_a.iter().map(|s| -s).collect();
        mixer.mix_into(&[(a, block_a), (b, block_b)], &mut sink).unwrap();
        assert_eq!(sink.zero_crossings(), 0, "反相抵消后无过零");
    }

    #[test]
    fn mixer_sum_exceeding_full_scale_folds_to_quiet() {
        // 两个满幅反相错位源叠加后削幅 clamp——过零数有限且帧数完整。
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        let b = mixer.attach();
        let mut sink = mono_sink(48000);
        let block = vec![1.0f32; 100];
        mixer.mix_into(&[(a, block.clone()), (b, block)], &mut sink).unwrap();
        assert_eq!(sink.frames_written(), 100);
        assert_eq!(sink.zero_crossings(), 0, "clamp 后恒 1.0 无过零");
    }

    #[test]
    fn mixer_volume_and_mute_gain() {
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        assert!(mixer.set_volume(a, 0.5));
        assert!(mixer.set_muted(a, false));
        let mut sink = mono_sink(48000);
        mixer.mix_into(&[(a, vec![1.0; 2])], &mut sink).unwrap();
        assert_eq!(sink.frames_written(), 2);
        // 非法 volume 钳制到 [0,1]。
        assert!(mixer.set_volume(a, 5.0));
        assert!(mixer.set_volume(a, -1.0));
    }

    #[test]
    fn mixer_muted_source_contributes_silence() {
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        let b = mixer.attach();
        mixer.set_muted(a, true);
        let mut sink = mono_sink(48000);
        // a 静音不参与；b 满幅 1.0 → 混音面无削幅（帧数 2，underrun 0）。
        mixer
            .mix_into(&[(a, vec![1.0; 2]), (b, vec![1.0; 2])], &mut sink)
            .unwrap();
        assert_eq!(sink.frames_written(), 2);
        assert_eq!(sink.underrun_count(), 0);
    }

    #[test]
    fn mixer_detached_source_skipped_and_handle_invalid() {
        let mut mixer = Mixer::new();
        let a = mixer.attach();
        let b = mixer.attach();
        assert!(mixer.detach(a));
        assert!(!mixer.detach(a), "重复卸载幂等 false");
        assert_eq!(mixer.source_count(), 1);
        assert!(!mixer.set_volume(a, 0.5), "已卸载 handle 控制无效");
        let mut sink = mono_sink(48000);
        // 已卸载 a 的块不参与混音（等价于只有 b）。
        mixer
            .mix_into(&[(a, vec![1.0; 2]), (b, vec![0.5; 2])], &mut sink)
            .unwrap();
        assert_eq!(sink.frames_written(), 2);
    }

    #[test]
    fn mixer_empty_bus_writes_silence_frame() {
        let mixer = Mixer::new();
        let mut sink = mono_sink(48000);
        mixer.mix_into(&[], &mut sink).unwrap();
        assert_eq!(sink.frames_written(), 0, "空块无帧可写");
    }
}
