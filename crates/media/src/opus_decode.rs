//! Opus 解码 — Ogg Opus 容器（RFC 7845）+ opus-decoder 纯 Rust 位流解码（M2c opus 面）。
//!
//! symphonia 0.6 无 opus 解码器（libopus 为 C 依赖，违反路线 C 零 C 依赖）；本模块
//! 用 symphonia 的 ogg reader 做**容器 demux**（包级读取，编解码器无关），位流包喂
//! [`opus_decoder::OpusDecoder`]（纯 Rust RFC 6716/8251，零 unsafe 零 FFI）产出
//! f32 交错 PCM——与 [`AudioDecoder`](crate::AudioDecoder) 同契约。
//!
//! 选型依据（2026-09-01 实测）：opus-decoder 0.1.1（MIT OR Apache-2.0，MSRV 1.85，
//! 依赖仅 thiserror）——纯 Rust opus 生态中唯一 RFC 8251 conformant + conformance
//! 测试常驻的实现；opus/audiopus 等均为 libopus C 绑定。

use std::io::Cursor;

use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::audio_decode::AudioDecodeError;

/// Ogg Opus 首包 `OpusHead`（RFC 7845 §4.1）的最小长度：magic(8) + version(1) +
/// channels(1) + pre_skip(2) + input_sample_rate(4) + output_gain(2)。
const OPUS_HEAD_MIN_LEN: usize = 19;

/// 从 Ogg Opus 首包解析声道数与 pre-skip（RFC 7845 §4.1）。
///
/// https://datatracker.ietf.org/doc/html/rfc7845#section-4.1
fn parse_opus_head(packet: &[u8]) -> Result<(u16, u16), AudioDecodeError> {
    if packet.len() < OPUS_HEAD_MIN_LEN || &packet[0..8] != b"OpusHead" {
        return Err(AudioDecodeError::Probe("missing OpusHead".into()));
    }
    let channels = packet[9];
    let pre_skip = u16::from_le_bytes([packet[10], packet[11]]);
    Ok((u16::from(channels), pre_skip))
}

/// 持续解码的 Ogg Opus 音频轨（M2c opus 面）——symphonia ogg reader demux +
/// opus-decoder 位流解码，输出 f32 交错 PCM（48 kHz 固定——Opus 规范输出率）。
///
/// 生命周期：从 [`open_ogg_opus`] 构建，`next_batch` 前向解码（与
/// [`AudioDecoder`](crate::AudioDecoder) 同契约；流末 `Ok(None)`）。
pub struct OpusAudioTrack {
    format: Box<dyn symphonia::core::formats::FormatReader>,
    decoder: opus_decoder::OpusDecoder,
    track_id: u32,
    channels: u16,
    /// OpusHead pre-skip（首帧起始的丢弃采样数——编码器补偿用的起始静音）。
    pre_skip: u64,
    /// pre-skip 已丢弃计数。
    skipped: u64,
    frames_out: u64,
    eos: bool,
}

/// 从 Ogg Opus 字节构建解码轨（symphonia ogg probe + OpusHead 解析 + decoder 构建）。
pub fn open_ogg_opus(data: &[u8]) -> Result<OpusAudioTrack, AudioDecodeError> {
    let source: Box<dyn symphonia::core::io::MediaSource> = Box::new(Cursor::new(data.to_vec()));
    let mss = MediaSourceStream::new(source, Default::default());
    let mut hint = Hint::new();
    hint.with_extension("oga");
    let format = symphonia::default::get_probe()
        .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| AudioDecodeError::Probe(format!("ogg stream: {e}")))?;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
        .ok_or(AudioDecodeError::NoTrack)?;
    let track_id = track.id;
    // symphonia ogg reader 把 OpusHead/OpusTags 头页解析为 `extra_data`（首包前消费，
    // 数据包从首个音频帧起）——声道/pre-skip 从此读（RFC 7845 §4.1）。
    let Some(symphonia::core::codecs::CodecParameters::Audio(params)) = &track.codec_params else {
        return Err(AudioDecodeError::NoTrack);
    };
    let head = params
        .extra_data
        .as_deref()
        .ok_or(AudioDecodeError::Probe("missing OpusHead extra_data".into()))?;
    let (channels, pre_skip) = parse_opus_head(head)?;
    let decoder = opus_decoder::OpusDecoder::new(48_000, usize::from(channels))
        .map_err(|e| AudioDecodeError::Probe(format!("opus decoder: {e}")))?;
    Ok(OpusAudioTrack {
        format,
        decoder,
        track_id,
        channels,
        pre_skip: u64::from(pre_skip),
        skipped: 0,
        frames_out: 0,
        eos: false,
    })
}

/// 解码下一批（f32 交错 PCM @48kHz；流末 `Ok(None)`）——与
/// [`crate::AudioDecoder`] 同契约。Opus 固定输出 48 kHz（RFC 7845 §4）。
impl OpusAudioTrack {
    pub fn sample_rate(&self) -> u32 {
        48_000
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn next_batch(&mut self) -> Result<Option<crate::audio_decode::DecodedAudio>, AudioDecodeError> {
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
            let bytes: &[u8] = &packet.data;
            if bytes.starts_with(b"OpusTags") {
                continue;
            }
            // 每帧至多 120ms @48kHz = 5760 采样/声道（Opus 规范上限）。
            let mut pcm = vec![0.0f32; 5760 * usize::from(self.channels)];
            let samples_per_ch = self
                .decoder
                .decode_float(bytes, &mut pcm, false)
                .map_err(|e| AudioDecodeError::Decode(format!("opus: {e}")))?;
            let frame_len = samples_per_ch * usize::from(self.channels);
            // pre-skip 丢弃（RFC 7845 §4.4：解码输出起始丢弃 pre_skip 采样）。
            let mut samples: Vec<f32> = {
                let all = pcm[..frame_len].to_vec();
                let per_ch = samples_per_ch;
                let mut drop_per_ch = 0u64;
                if self.skipped < self.pre_skip {
                    drop_per_ch = (self.pre_skip - self.skipped).min(per_ch as u64);
                    self.skipped += drop_per_ch;
                }
                let drop_total = (drop_per_ch as usize) * usize::from(self.channels);
                all[drop_total..].to_vec()
            };
            if samples.is_empty() {
                continue;
            }
            let pts_ms = self.frames_out * 1000 / 48_000;
            self.frames_out += (samples.len() / usize::from(self.channels)) as u64;
            return Ok(Some(crate::audio_decode::DecodedAudio {
                samples: std::mem::take(&mut samples),
                sample_rate: 48_000,
                channels: self.channels,
                pts_ms,
            }));
        }
    }
}
