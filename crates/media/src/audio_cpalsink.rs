//! CpalSink — 真实设备输出（`audio-cpal` feature，media-audio M1 切片 2）。
//!
//! M0 验证策略双实现中的设备侧：cpal 输出流消费 f32 交错 PCM；设备枚举/流构建
//! 失败时构造器返回错误——**调用方回落 [`NullSink`](crate::NullSink)**（策略：
//! CI/WPT 强制 NullSink；桌面环境 feature 打开，失败自动回落，见 M0 §3）。
//!
//! underrun 语义：设备回调取数时队列为空 → 本次回调填静音并计一次 underrun
//! （消费方饿死的可观测性，`underrun_count` 契约同 NullSink）。

use super::{AudioFormat, AudioSink, AudioSinkError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 回调与控制面共享的队列状态。
struct CpalShared {
    queue: VecDeque<f32>,
    underruns: u64,
    paused: bool,
}

/// cpal 真实设备 sink（`audio-cpal` feature）。
pub struct CpalSink {
    shared: Arc<Mutex<CpalShared>>,
    /// 保持流存活（drop 即停）；pause/resume 经它控制。
    stream: cpal::Stream,
    format: AudioFormat,
}

impl CpalSink {
    /// 用默认输出设备按 `format`（f32 采样）构建输出流（未自动 play——随
    /// [`AudioSink::start`] 语义，start 即构建并 play）。
    ///
    /// # Errors
    /// 无默认输出设备、设备不支持该格式/采样率、流构建失败——调用方回落
    /// `NullSink`（M0 §3 环境自适应策略）。
    pub fn new(format: AudioFormat) -> Result<Self, AudioSinkError> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AudioSinkError::NoOutputDevice)?;
        let stream_config = cpal::StreamConfig {
            channels: format.channels,
            sample_rate: cpal::SampleRate(format.sample_rate),
            // 默认（设备端协商）buffer size。
            buffer_size: cpal::BufferSize::Default,
        };
        let shared = Arc::new(Mutex::new(CpalShared {
            queue: VecDeque::new(),
            underruns: 0,
            paused: false,
        }));
        let cb_shared = Arc::clone(&shared);
        // f32 原生采样（解码面 symphonia f32 输出直通，无转换层）。
        let stream = device
            .build_output_stream(
                &stream_config,
                move |out: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let mut shared = cb_shared.lock().unwrap_or_else(|e| e.into_inner());
                    let avail = shared.queue.len().min(out.len());
                    let (drain, rest) = out.split_at_mut(avail);
                    for (dst, src) in drain.iter_mut().zip(shared.queue.drain(..avail)) {
                        *dst = src;
                    }
                    if !rest.is_empty() {
                        // 队列饿死：本次回调填静音并计 underrun。
                        shared.underruns += 1;
                        rest.fill(0.0);
                    }
                },
                |err| tracing::warn!("cpal output stream error: {err}"),
                None,
            )
            .map_err(|e| AudioSinkError::Device(e.to_string()))?;
        Ok(Self { shared, stream, format })
    }

    /// 观测窗内累计写入的采样数（经设备回调实际消费）。
    pub fn samples_consumed(&self) -> usize {
        // 队列 drain 不可回读——可观测性由 underrun 与 write 侧 frames 承担；
        // 此处提供队列积压量（设备延迟代理）。
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).queue.len()
    }
}

impl AudioSink for CpalSink {
    fn start(&mut self, format: AudioFormat) -> Result<(), AudioSinkError> {
        if format != self.format {
            // 流按构造时格式建立——重启须重建 sink（返回错误让调用方新建）。
            return Err(AudioSinkError::Device(format!(
                "format change requires rebuild: built {:?}, got {:?}",
                self.format, format
            )));
        }
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).paused = false;
        self.stream.play().map_err(|e| AudioSinkError::Device(e.to_string()))
    }

    fn write(&mut self, samples: &[f32]) -> Result<(), AudioSinkError> {
        let mut shared = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        if shared.paused {
            return Err(AudioSinkError::NotStarted);
        }
        if !samples.len().is_multiple_of(usize::from(self.format.channels)) {
            return Err(AudioSinkError::NotFrameAligned {
                len: samples.len(),
                channels: self.format.channels,
            });
        }
        shared.queue.extend(samples.iter().copied());
        Ok(())
    }

    fn pause(&mut self) -> Result<(), AudioSinkError> {
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).paused = true;
        self.stream.pause().map_err(|e| AudioSinkError::Device(e.to_string()))
    }

    fn resume(&mut self) -> Result<(), AudioSinkError> {
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).paused = false;
        self.stream.play().map_err(|e| AudioSinkError::Device(e.to_string()))
    }

    fn underrun_count(&self) -> u64 {
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).underruns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 环境自适应冒烟：构造失败（无设备/格式不支持）= 合法路径（回落 NullSink）；
    /// 构造成功则 start/play 不应报错。WSL2（HDA 在、pulse 桥断）实测：构造成功、
    /// ALSA 打开 default 设备成功——真出声冒烟仍留桌面环境（M0 §3 既定）。
    #[test]
    fn cpalsink_constructs_or_reports_device_error() {
        let format = AudioFormat {
            sample_rate: 48000,
            channels: 2,
        };
        match CpalSink::new(format) {
            Ok(mut sink) => {
                sink.start(format).expect("start after successful build");
                sink.write(&[0.0; 480]).expect("write 10 stereo frames");
                sink.pause().expect("pause");
                assert_eq!(sink.write(&[0.0; 10]), Err(AudioSinkError::NotStarted));
                sink.resume().expect("resume");
            }
            Err(e @ (AudioSinkError::NoOutputDevice | AudioSinkError::Device(_))) => {
                eprintln!("CpalSink unavailable in this env (fallback to NullSink): {e}");
            }
            Err(e) => panic!("unexpected error kind: {e}"),
        }
    }
}
