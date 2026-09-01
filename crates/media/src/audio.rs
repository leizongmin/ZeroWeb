//! 音频输出面 — `AudioSink` trait + `NullSink` 可观测实现（media-audio M1）。
//!
//! M0 验证策略（[evidence/2026-09-01-m0-environment-probe.md]）成立的双实现形态：
//! - [`NullSink`]：headless/CI 默认。吞掉 PCM 但**可观测**——写入帧数、过零率
//!   （频域代理，O(n) 无需 FFT）可断言，「play() 后 sink 收到 ≥N 帧」即播放驱动
//!   的可观测等价物。CI 与 WPT 环境强制走此实现。
//! - `CpalSink`（真实设备）：feature-gated `audio-cpal`，M1 后续切片接入
//!   （编译面已由 D2 探针验证：libasound2-dev + cpal 0.16 ALSA host）。
//!
//! [evidence/2026-09-01-m0-environment-probe.md]: ../../docs/goal/media-audio/evidence/2026-09-01-m0-environment-probe.md

/// PCM 流格式（sink 端点契约；与解码面 symphonia 输出的采样率/声道对齐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    /// 采样率（Hz，如 44100 / 48000）。
    pub sample_rate: u32,
    /// 声道数（交错存储；1 = mono，2 = stereo）。
    pub channels: u16,
}

/// 音频输出错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AudioSinkError {
    /// sink 未 `start`（或已 `pause` 且不支持静默写入）即写入。
    #[error("sink not started")]
    NotStarted,
    /// 采样块声道数与 start 时的格式不一致。
    #[error("channel mismatch: expected {expected}, got {got}")]
    ChannelMismatch {
        /// start 时声明的声道数。
        expected: u16,
        /// 本次写入的实际声道数。
        got: u16,
    },
    /// 采样长度非声道数整倍（不成完整帧）。
    #[error("samples not frame-aligned: len {len} not a multiple of {channels} channels")]
    NotFrameAligned {
        /// 实际采样长度。
        len: usize,
        /// start 时声明的声道数。
        channels: u16,
    },
    /// 无可用输出设备（CpalSink 构造面——调用方回落 NullSink）。
    #[error("no output device available")]
    NoOutputDevice,
    /// 设备/流层错误（cpal 构建、play/pause 失败等）。
    #[error("device error: {0}")]
    Device(String),
}

/// 音频输出端点 — PCM f32 交错帧消费者（media-audio M1 输出面契约）。
///
/// 实现方：[`NullSink`]（可观测）与 `CpalSink`（真实设备，`audio-cpal` feature）。
/// 播放驱动（media-playback M2c）与本 trait 对接；A/V 同步（audio clock 主时钟）
/// 归 media-audio M2——本 trait 不承载时钟（读侧归 [`super::VideoClock`] 体系）。
pub trait AudioSink {
    /// 以给定格式启动输出流。重复调用以新格式重启（帧计数不清零——统计累计）。
    fn start(&mut self, format: AudioFormat) -> Result<(), AudioSinkError>;

    /// 写入交错 PCM f32 采样（长度须为声道数整数倍；一「帧」= 一组全声道采样）。
    fn write(&mut self, samples: &[f32]) -> Result<(), AudioSinkError>;

    /// 暂停输出（后续 write 拒收——播放驱动的 pause 语义对接点）。
    fn pause(&mut self) -> Result<(), AudioSinkError>;

    /// 恢复输出。
    fn resume(&mut self) -> Result<(), AudioSinkError>;

    /// 累计 underrun 次数（消费方饿死——可观测性，真实设备驱动层喂给）。
    fn underrun_count(&self) -> u64;
}

/// headless/CI 默认 sink — 吞掉 PCM 但全量可观测。
///
/// 断言形态（M0 §3）：
/// - `frames_written`：「play() 后 NullSink 收到 ≥N 帧」；
/// - [`Self::zero_crossings_per_second`]：440Hz 正弦 ≈ 880（2×频率）→ 解码+混音
///   频域代理。
#[derive(Debug, Default)]
pub struct NullSink {
    format: Option<AudioFormat>,
    started: bool,
    paused: bool,
    frames_written: u64,
    zero_crossings: u64,
    /// 上一采样（过零判定；跨 write 边界连续）。
    last_sample: Option<f32>,
    underruns: u64,
}

impl NullSink {
    /// 新建空 sink（未启动）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 累计写入的帧数（一帧 = 一组全声道交错采样）。
    pub fn frames_written(&self) -> u64 {
        self.frames_written
    }

    /// 观测窗内的过零次数（跨 write 边界连续计数，从 start 起累计）。
    pub fn zero_crossings(&self) -> u64 {
        self.zero_crossings
    }

    /// 累计过零率（Hz 代理）：zero_crossings / 观测时长。格式未启动时 `None`。
    pub fn zero_crossings_per_second(&self) -> Option<f64> {
        let format = self.format?;
        let seconds = self.frames_written as f64 / f64::from(format.sample_rate);
        (seconds > 0.0).then(|| self.zero_crossings as f64 / seconds)
    }
}

impl AudioSink for NullSink {
    fn start(&mut self, format: AudioFormat) -> Result<(), AudioSinkError> {
        self.format = Some(format);
        self.started = true;
        self.paused = false;
        Ok(())
    }

    fn write(&mut self, samples: &[f32]) -> Result<(), AudioSinkError> {
        let format = self.format.ok_or(AudioSinkError::NotStarted)?;
        if self.paused {
            self.underruns += 1;
            return Err(AudioSinkError::NotStarted);
        }
        if !samples.len().is_multiple_of(usize::from(format.channels)) {
            return Err(AudioSinkError::NotFrameAligned {
                len: samples.len(),
                channels: format.channels,
            });
        }
        let frames = samples.len() / usize::from(format.channels);
        if format.channels == 1 {
            // mono：采样即帧，直接逐样本过零判定。
            for &s in samples {
                if let Some(last) = self.last_sample
                    && (last < 0.0) != (s < 0.0)
                {
                    self.zero_crossings += 1;
                }
                self.last_sample = Some(s);
            }
        } else {
            // 多声道：按首声道过零（频域代理只需一个稳定通道；声道内交错不影响）。
            for frame in samples.chunks_exact(usize::from(format.channels)) {
                let s = frame[0];
                if let Some(last) = self.last_sample
                    && (last < 0.0) != (s < 0.0)
                {
                    self.zero_crossings += 1;
                }
                self.last_sample = Some(s);
            }
        }
        self.frames_written += frames as u64;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), AudioSinkError> {
        self.paused = true;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), AudioSinkError> {
        self.paused = false;
        Ok(())
    }

    fn underrun_count(&self) -> u64 {
        self.underruns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mono(rate: u32) -> AudioFormat {
        AudioFormat {
            sample_rate: rate,
            channels: 1,
        }
    }

    #[test]
    fn nullsink_counts_frames_and_rejects_before_start() {
        let mut sink = NullSink::new();
        assert_eq!(sink.write(&[0.0, 0.0]), Err(AudioSinkError::NotStarted));
        sink.start(mono(44100)).unwrap();
        sink.write(&[0.5; 100]).unwrap();
        assert_eq!(sink.frames_written(), 100);
    }

    #[test]
    fn nullsink_pause_gates_write_and_counts_underrun() {
        let mut sink = NullSink::new();
        sink.start(mono(48000)).unwrap();
        sink.write(&[0.1; 10]).unwrap();
        sink.pause().unwrap();
        assert_eq!(sink.write(&[0.1; 10]), Err(AudioSinkError::NotStarted));
        assert_eq!(sink.underrun_count(), 1);
        sink.resume().unwrap();
        sink.write(&[0.1; 10]).unwrap();
        assert_eq!(sink.frames_written(), 20);
        assert_eq!(sink.underrun_count(), 1);
    }

    #[test]
    fn nullsink_channel_mismatch_on_non_multiple_length() {
        let mut sink = NullSink::new();
        sink.start(AudioFormat {
            sample_rate: 48000,
            channels: 2,
        })
        .unwrap();
        // 3 采样 / 2 声道：非整帧写入拒收。
        assert!(matches!(
            sink.write(&[0.0, 0.0, 0.0]),
            Err(AudioSinkError::NotFrameAligned { len: 3, channels: 2 })
        ));
        assert_eq!(sink.frames_written(), 0);
    }

    #[test]
    fn nullsink_zero_crossing_rate_tracks_sine_frequency() {
        // 440Hz 正弦 0.5s @ 48kHz：220 个整周期，每周期上/下各过零一次 →
        // 过零率 ≈ 2 × 440 = 880（过零率是频率的两倍，断言按此锚定）。
        let mut sink = NullSink::new();
        sink.start(mono(48000)).unwrap();
        let mut buf = Vec::with_capacity(24000);
        for n in 0..24000u32 {
            let t = f64::from(n) / 48000.0;
            buf.push((2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32);
        }
        sink.write(&buf).unwrap();
        let zcr = sink.zero_crossings_per_second().unwrap();
        assert!(
            (zcr - 880.0).abs() < 16.0,
            "440Hz sine zero-crossing rate should be ≈880 (2×freq), got {zcr}"
        );
    }

    #[test]
    fn nullsink_restart_updates_format_keeps_stats() {
        let mut sink = NullSink::new();
        sink.start(mono(48000)).unwrap();
        sink.write(&[0.5; 50]).unwrap();
        sink.start(mono(44100)).unwrap();
        sink.write(&[0.5; 50]).unwrap();
        assert_eq!(sink.frames_written(), 100, "重启不清零统计（累计语义）");
    }
}
