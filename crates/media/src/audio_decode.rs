//! 音频解码 — symphonia 0.6 封装：容器/编码探测 → f32 交错 PCM 逐包输出（M2c）。
//!
//! 路线 C 约束：纯 Rust（symphonia 无 C 依赖）。面宽：mp3 + ogg/vorbis（fixture
//! `sample-mp3.mp3` / `sample-ogg-vorbis.oga`——440Hz sine 生成，过零率可观测锚点
//! 880 = 2×频率，media-audio NullSink 断言契约）。**opus 不在面内**——symphonia 0.6
//! 无 opus 解码器（C 侧 libopus 才有，违反零 C 依赖），`sample-ogg-opus.oga` 留待
//! 后续选型（master.md M2c 注记）。
//!
//! 与 [`AudioSink`](crate::AudioSink) 对接：解码输出的 f32 交错采样直写 sink
//! （media-audio M1 输出面）；A/V 同步（audio clock 主时钟）归 media-audio M2。

use std::io::Cursor;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoderOptions};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

/// 音频解码错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AudioDecodeError {
    /// 容器/编码探测失败（不支持的格式或损坏位流）。
    #[error("audio probe failed: {0}")]
    Probe(String),
    /// 无音频轨。
    #[error("no audio track")]
    NoTrack,
    /// 解码期错误。
    #[error("audio decode error: {0}")]
    Decode(String),
}

/// 解码出的音频批次 — f32 交错 PCM + 流参数。
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    /// f32 交错采样（`channels` 路；值域 [-1,1]）。
    pub samples: Vec<f32>,
    /// 采样率（Hz）。
    pub sample_rate: u32,
    /// 声道数。
    pub channels: u16,
    /// 批次起始位置（毫秒，媒体时间轴——由采样计数推导）。
    pub pts_ms: u64,
}

/// 逐批读取的音频解码器（mp3 / ogg-vorbis 面）。
pub struct AudioDecoder {
    format: Box<dyn FormatReader + 'static>,
    decoder: Box<dyn symphonia::core::codecs::audio::AudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    /// 已输出的帧计数（pts 推导）。
    frames_out: u64,
    /// 流是否已到末尾。
    eos: bool,
}

impl AudioDecoder {
    /// 从字节打开音频解码器（symphonia probe 自动识别容器/编码）。
    pub fn open(data: &[u8]) -> Result<Self, AudioDecodeError> {
        // 'static source（owned Cursor）→ FormatReader 与 MSS 生命周期解耦（probe
        // 返回 Box<dyn FormatReader + 'static>，自持 MSS——无自引用问题）。
        let mss = MediaSourceStream::new(Box::new(Cursor::new(data.to_vec())), Default::default());
        let hint = Hint::new();
        let probed = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
            .map_err(|e| AudioDecodeError::Probe(e.to_string()))?;
        let format: Box<dyn FormatReader + 'static> = probed;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
            .ok_or(AudioDecodeError::NoTrack)?;
        let track_id = track.id;
        let Some(CodecParameters::Audio(params)) = &track.codec_params else {
            return Err(AudioDecodeError::NoTrack);
        };
        let sample_rate = params.sample_rate.ok_or(AudioDecodeError::NoTrack)?;
        let channels = params.channels.as_ref().map(|c| c.count()).unwrap_or(1) as u16;
        let audio_params = AudioCodecParameters {
            codec: params.codec,
            profile: params.profile,
            sample_rate: params.sample_rate,
            sample_format: params.sample_format,
            bits_per_sample: params.bits_per_sample,
            bits_per_coded_sample: params.bits_per_coded_sample,
            channels: params.channels.clone(),
            max_frames_per_packet: params.max_frames_per_packet,
            verification_check: params.verification_check,
            extra_data: params.extra_data.clone(),
            frames_per_block: params.frames_per_block,
        };
        // symphonia::default 全局注册表（按编译 feature 预注册——mp3/vorbis/pcm）。
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&audio_params, &AudioDecoderOptions::default())
            .map_err(|e| AudioDecodeError::Probe(e.to_string()))?;
        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            frames_out: 0,
            eos: false,
        })
    }

    /// 采样率（Hz）。
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 声道数。
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 解码下一批（一个 media packet 一批）；流末返回 `Ok(None)`。
    pub fn next_batch(&mut self) -> Result<Option<DecodedAudio>, AudioDecodeError> {
        if self.eos {
            return Ok(None);
        }
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(p)) => p,
                Ok(None) => {
                    self.eos = true;
                    return Ok(None);
                }
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.eos = true;
                    return Ok(None);
                }
                Err(e) => return Err(AudioDecodeError::Decode(e.to_string())),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    // f32 交错面直出（symphonia 内建跨格式转换；值域 [-1,1]——
                    // AudioSink 契约一致）。
                    let mut samples = Vec::with_capacity(decoded.samples_interleaved());
                    decoded.copy_to_vec_interleaved::<f32>(&mut samples);
                    let pts_ms = self.frames_out * 1000 / u64::from(self.sample_rate);
                    self.frames_out += (samples.len() / usize::from(self.channels)) as u64;
                    return Ok(Some(DecodedAudio {
                        samples,
                        sample_rate: self.sample_rate,
                        channels: self.channels,
                        pts_ms,
                    }));
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue, // 损坏包跳过
                Err(e) => return Err(AudioDecodeError::Decode(e.to_string())),
            }
        }
    }
}
