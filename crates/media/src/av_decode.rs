//! A/V 同源解码 — webm 含音频轨（A_VORBIS）的音频面（M2 切片 D，A/V 同步前置）。
//!
//! webm 内嵌 vorbis 与独立 ogg 的差异仅在封装：位流（三段 Xiph 头 + 数据包）同源。
//! 本模块把 Matroska demux 出的音频包重新封装为 OGG 页流（RFC 3533），交
//! [`AudioDecoder`](crate::AudioDecoder)（symphonia ogg reader + vorbis codec）
//! 解码——复用 M2c 的 f32 PCM 输出面，audio clock 主时钟（media-audio M2 契约）
//! 由此获得与视频轨同源 demux 的解码数据。
//!
//! 路线 C 约束保持：页合成是纯字节操作，无新增 C 依赖。

use std::io::Cursor;

use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSource;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::audio_decode::AudioDecodeError;

/// OGG CRC-32：多项式 0x04c11db7，无反射、初值 0、无异或（RFC 3533 §Appendix A）。
fn ogg_crc(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, entry) in table.iter_mut().enumerate() {
        let mut r = (i as u32) << 24;
        for _ in 0..8 {
            r = if r & 0x8000_0000 != 0 {
                (r << 1) ^ 0x04c1_1db7
            } else {
                r << 1
            };
        }
        *entry = r;
    }
    let mut crc = 0u32;
    for &b in data {
        crc = (crc << 8) ^ table[(((crc >> 24) as u8) ^ b) as usize];
    }
    crc
}

/// 单个 OGG 页（RFC 3533 §4）——27 字节头 + 段表 + 载荷。
struct OggPage {
    header_type: u8,
    /// granule position（本页最后采样位置；-1 表示未定义——BOS/EOS/头页）。
    granule: u64,
    sequence: u32,
    payload: Vec<u8>,
}

impl OggPage {
    const HEADER_TYPE_BOS: u8 = 0x02;

    fn serialize(&self, serial: u32) -> Vec<u8> {
        let full_segments = self.payload.len() / 255;
        let last = self.payload.len() % 255;
        // 末段恰为 255 的整数倍时需要一个 0 长度收尾段。
        let segment_count = if self.payload.is_empty() {
            1
        } else if last == 0 {
            full_segments
        } else {
            full_segments + 1
        };
        debug_assert!(segment_count <= 255, "page payload > 65025 bytes 不支持");

        let mut out = Vec::with_capacity(27 + segment_count + self.payload.len());
        out.extend_from_slice(b"OggS");
        out.push(0); // version
        out.push(self.header_type);
        out.extend_from_slice(&self.granule.to_le_bytes());
        out.extend_from_slice(&serial.to_le_bytes());
        out.extend_from_slice(&self.sequence.to_le_bytes());
        out.extend_from_slice(&[0; 4]); // CRC 占位
        out.push(segment_count as u8);
        out.extend(std::iter::repeat_n(255u8, full_segments));
        if last != 0 {
            out.push(last as u8);
        } else if self.payload.is_empty() {
            out.push(0);
        }
        out.extend_from_slice(&self.payload);
        // CRC 覆盖全页，checksum 字段置 0 计算（RFC 3533 §6）。
        let crc = ogg_crc(&out);
        out[22..26].copy_from_slice(&crc.to_le_bytes());
        out
    }
}

/// 把 Matroska `CodecPrivate`（A_VORBIS 三段 Xiph 头：count-1 + 先全部长度、后连续
/// 数据）拆出头数据。
///
/// https://www.matroska.org/technical/codec_specs.html#vorbis
fn parse_vorbis_codec_private(private: &[u8]) -> Result<Vec<Vec<u8>>, AudioDecodeError> {
    if private.is_empty() {
        return Err(AudioDecodeError::Probe("empty CodecPrivate".into()));
    }
    let header_count = private[0] as usize + 1;
    let mut pos = 1usize;
    // ① 先读前 n-1 段长度（Xiph lacing：255 续段 + 余数收尾）；末段长度 =
    //    数据区剩余 − 前 n-1 段长度之和（长度表与数据是两个连续区）。
    let mut lengths = Vec::with_capacity(header_count);
    for _ in 0..header_count - 1 {
        let mut len = 0usize;
        loop {
            let Some(&size_byte) = private.get(pos) else {
                return Err(AudioDecodeError::Probe("truncated header length".into()));
            };
            pos += 1;
            len += size_byte as usize;
            if size_byte != 255 {
                break;
            }
        }
        lengths.push(len);
    }
    let data_area = private.len().saturating_sub(pos);
    if data_area < lengths.iter().sum::<usize>() {
        return Err(AudioDecodeError::Probe("headers exceed CodecPrivate".into()));
    }
    lengths.push(data_area - lengths.iter().sum::<usize>());
    // ② 长度后紧跟连续的头数据。
    let mut headers = Vec::with_capacity(header_count);
    for len in lengths {
        let end = pos + len;
        if end > private.len() {
            return Err(AudioDecodeError::Probe("header exceeds CodecPrivate".into()));
        }
        headers.push(private[pos..end].to_vec());
        pos = end;
    }
    Ok(headers)
}

/// 持续解码的 webm 音频轨（A_VORBIS 面）——数据包经 OGG 页重封装进 symphonia。
///
/// 生命周期：从 [`AudioDecoder::open_webm_audio`](crate::AudioDecoder::open_webm_audio)
/// 构建（内含 BOS + 三段头页序列化），`next_batch` 前向解码（与独立音频面同契约）。
pub struct WebmAudioTrack {
    decoder: Box<dyn symphonia::core::codecs::audio::AudioDecoder>,
    format: Box<dyn symphonia::core::formats::FormatReader>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    frames_out: u64,
    eos: bool,
}

/// 从 webm 字节构建音频轨解码器（demux 音频轨 + CodecPrivate 头 + 音频包 → OGG 流）。
pub fn open_webm_audio_track(data: &[u8]) -> Result<WebmAudioTrack, AudioDecodeError> {
    let cursor = Cursor::new(data.to_vec());
    let demuxer =
        matroska_demuxer::MatroskaFile::open(cursor).map_err(|e| AudioDecodeError::Probe(format!("container: {e}")))?;
    let track = demuxer
        .tracks()
        .iter()
        .find(|t| t.track_type() == matroska_demuxer::TrackType::Audio && t.codec_id() == "A_VORBIS")
        .ok_or(AudioDecodeError::NoTrack)?;
    let audio_track = track.track_number().get();
    let Some(private) = track.codec_private() else {
        return Err(AudioDecodeError::Probe("no CodecPrivate for A_VORBIS".into()));
    };
    let headers = parse_vorbis_codec_private(private)?;

    // 音频包逐块收集（fixture 级小源——整段缓冲可接受；真实流面后续做流式封装）。
    let mut demuxer = demuxer;
    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut block = matroska_demuxer::Frame::default();
    while matches!(demuxer.next_frame(&mut block), Ok(true)) {
        if block.track == audio_track {
            packets.push(block.data.clone());
        }
    }

    // 页序列：BOS(ident) → comment → setup → 数据页（一包一页）。
    //
    // granule 语义：symphonia 的 ogg reader 用页 granule（page_end_ts）对包做端部
    // 裁剪——granule 小于包累计时长会把采样整段裁空（实测 0 样本缺陷）。本轨不做
    // seek，granule 只需**单调递增且 ≥ 包累计时长**：每页取 (i+1)×16384 的保守
    // 高估（16384 ≥ vorbis 最大块 8192 的包时长），使 next_pkt_pts ≤ page_end 恒
    // 成立、端部裁剪恒零。数据页不设 EOS 位（symphonia 以 EOF 判流末，EOS 页的
    // 末页 padding 裁剪反而会裁掉有效采样）。
    let serial = 0x5a_5e_b0_1du32;
    let mut stream: Vec<u8> = Vec::new();
    let mut sequence = 0u32;
    let mut push_page = |header_type: u8, granule: u64, payload: &[u8], stream: &mut Vec<u8>| {
        let page = OggPage {
            header_type,
            granule,
            sequence,
            payload: payload.to_vec(),
        };
        stream.extend_from_slice(&page.serialize(serial));
        sequence += 1;
    };
    // 头页：三段头各自一页（vorbis 规范：comment/setup 头可与 ident 同页不同页，
    // symphonia PageReader 按段解析——一页一头最稳）；granule = -1（全 1）。
    push_page(OggPage::HEADER_TYPE_BOS, u64::MAX, &headers[0], &mut stream);
    for header in &headers[1..] {
        push_page(0, u64::MAX, header, &mut stream);
    }
    for (i, packet) in packets.iter().enumerate() {
        push_page(0, (i as u64 + 1) * 16384, packet, &mut stream);
    }

    // debug：合成流落盘供外部工具验证（ZW_AV_SYNTH_DUMP=1）。
    if std::env::var("ZW_AV_SYNTH_DUMP").is_ok() {
        let _ = std::fs::write("/tmp/av-synth.ogg", &stream);
    }

    // symphonia probe（ogg reader + vorbis codec 按 feature 注册）。
    let source: Box<dyn MediaSource> = Box::new(Cursor::new(stream));
    let mss = MediaSourceStream::new(source, Default::default());
    let hint = Hint::new();
    let probed = symphonia::default::get_probe()
        .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| AudioDecodeError::Probe(format!("ogg stream: {e}")))?;
    let format = probed;
    let decoded_track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
        .ok_or(AudioDecodeError::NoTrack)?;
    let track_id = decoded_track.id;
    let Some(symphonia::core::codecs::CodecParameters::Audio(params)) = &decoded_track.codec_params else {
        return Err(AudioDecodeError::NoTrack);
    };
    let sample_rate = params.sample_rate.ok_or(AudioDecodeError::NoTrack)?;
    let channels = params.channels.as_ref().map(|c| c.count()).unwrap_or(1) as u16;
    let audio_params = symphonia::core::codecs::audio::AudioCodecParameters {
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
    let decoder = symphonia::default::get_codecs()
        .make_audio_decoder(
            &audio_params,
            &symphonia::core::codecs::audio::AudioDecoderOptions::default(),
        )
        .map_err(|e| AudioDecodeError::Probe(e.to_string()))?;
    Ok(WebmAudioTrack {
        decoder,
        format,
        track_id,
        sample_rate,
        channels,
        frames_out: 0,
        eos: false,
    })
}

impl WebmAudioTrack {
    /// 采样率（Hz）。
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 声道数。
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 解码下一批（f32 交错 PCM；流末 `Ok(None)`）——与 [`crate::AudioDecoder`] 同契约。
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
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut samples = Vec::with_capacity(decoded.samples_interleaved());
                    decoded.copy_to_vec_interleaved::<f32>(&mut samples);
                    let pts_ms = self.frames_out * 1000 / u64::from(self.sample_rate);
                    self.frames_out += (samples.len() / usize::from(self.channels)) as u64;
                    return Ok(Some(crate::audio_decode::DecodedAudio {
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

// ── A_OPUS 面（M3 扩批 2026-09-02，fixture-mounted 播放切片前置）────────────────
//
// webm 内嵌 Opus 与独立 Ogg Opus 的差异同样仅在封装：Matroska A_OPUS 的 CodecPrivate
// **就是** OpusHead（Matroska spec codec 规定），数据包即 Opus 数据包——无需 OGG 重
// 封装，逐包直接喂 `opus_decoder::OpusDecoder`（M2c opus 面，纯 Rust RFC 6716/8251）。
// WPT 上游媒体文件（media/*.webm）实测全为 VP9+Opus 双轨——本面落地后 WPT 播放推进族
// （track-cues-* / time-marches-on）的媒体源在 zero-media 解码面全部就绪。

/// Matroska A_OPUS 的 CodecPrivate（= OpusHead，RFC 7845 §4.1）解析——声道数 +
/// pre-skip + 输入采样率。
/// https://www.matroska.org/technical/codec_specs.html#opus
fn parse_opus_codec_private(private: &[u8]) -> Result<(u16, u16, u32), AudioDecodeError> {
    if private.len() < 19 || &private[0..8] != b"OpusHead" {
        return Err(AudioDecodeError::Probe("missing OpusHead in CodecPrivate".into()));
    }
    let channels = private[9];
    let pre_skip = u16::from_le_bytes([private[10], private[11]]);
    let input_sample_rate = u32::from_le_bytes([private[12], private[13], private[14], private[15]]);
    Ok((u16::from(channels), pre_skip, input_sample_rate))
}

/// 持续解码的 webm A_OPUS 音频轨——Matroska demux + opus-decoder 直解（无 OGG 重封装）。
///
/// 输出面与 [`WebmAudioTrack`] 同契约（f32 交错 PCM、`sample_rate`/`channels`/
/// `next_batch`）；Opus 规范输出率固定 48 kHz。pre-skip（编码器起始补偿静音）在首批
/// 丢弃。包间隙不填补（webm timestamp 语义由调用方按 pts 推进；WPT 源为连续编码）。
///
/// 生命周期：从 [`open_webm_opus_audio_track`] 构建，`next_batch` 前向解码（流末
/// `Ok(None)`）。
pub struct WebmOpusAudioTrack {
    demuxer: matroska_demuxer::MatroskaFile<Cursor<Vec<u8>>>,
    audio_track: u64,
    decoder: opus_decoder::OpusDecoder,
    channels: u16,
    /// OpusHead pre-skip（首批起始丢弃的采样数，per channel）。
    pre_skip: u64,
    /// pre-skip 已丢弃计数（per channel）。
    skipped: u64,
    /// 已输出帧数（per channel）——pts 推导。
    frames_out: u64,
    eos: bool,
}

/// 从 webm 字节构建 A_OPUS 音频轨解码器（demux 音频轨 + OpusHead + opus-decoder）。
pub fn open_webm_opus_audio_track(data: &[u8]) -> Result<WebmOpusAudioTrack, AudioDecodeError> {
    let cursor = Cursor::new(data.to_vec());
    let demuxer =
        matroska_demuxer::MatroskaFile::open(cursor).map_err(|e| AudioDecodeError::Probe(format!("container: {e}")))?;
    let track = demuxer
        .tracks()
        .iter()
        .find(|t| t.track_type() == matroska_demuxer::TrackType::Audio && t.codec_id() == "A_OPUS")
        .ok_or(AudioDecodeError::NoTrack)?;
    let audio_track = track.track_number().get();
    let Some(private) = track.codec_private() else {
        return Err(AudioDecodeError::Probe("no CodecPrivate for A_OPUS".into()));
    };
    let (channels, pre_skip, _input_rate) = parse_opus_codec_private(private)?;
    let decoder = opus_decoder::OpusDecoder::new(48_000, usize::from(channels))
        .map_err(|e| AudioDecodeError::Probe(format!("opus decoder: {e}")))?;
    Ok(WebmOpusAudioTrack {
        demuxer,
        audio_track,
        decoder,
        channels,
        pre_skip: u64::from(pre_skip),
        skipped: 0,
        frames_out: 0,
        eos: false,
    })
}

impl WebmOpusAudioTrack {
    /// 采样率（Hz）——Opus 规范输出率固定 48 kHz。
    pub fn sample_rate(&self) -> u32 {
        48_000
    }

    /// 声道数（OpusHead）。
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 解码下一批（f32 交错 PCM；流末 `Ok(None)`）——与 [`WebmAudioTrack`] 同契约。
    pub fn next_batch(&mut self) -> Result<Option<crate::audio_decode::DecodedAudio>, AudioDecodeError> {
        if self.eos {
            return Ok(None);
        }
        // Opus 帧最长 120ms @48kHz = 5760 样本/声道（RFC 6716 §2）；缓冲按上限预分配。
        const MAX_FRAME: usize = 5760;
        let mut pcm = vec![0.0f32; MAX_FRAME * usize::from(self.channels)];
        loop {
            let mut block = matroska_demuxer::Frame::default();
            match self.demuxer.next_frame(&mut block) {
                Ok(true) => {
                    if block.track != self.audio_track {
                        continue;
                    }
                    let per_ch = match self.decoder.decode_float(&block.data, &mut pcm, false) {
                        Ok(n) => n,
                        Err(_) => continue, // 损坏包跳过（AudioDecoder 同面）
                    };
                    let total = per_ch * usize::from(self.channels);
                    // pre-skip 丢弃（首解码面按采样序剥除）。
                    let mut start = 0usize;
                    if self.skipped < self.pre_skip {
                        let remain = self.pre_skip - self.skipped;
                        let drop = (remain as usize).min(per_ch);
                        self.skipped += drop as u64;
                        start = drop * usize::from(self.channels);
                    }
                    if start >= total {
                        continue; // 整批都在 pre-skip 内
                    }
                    let samples = pcm[start..total].to_vec();
                    let pts_ms = self.frames_out * 1000 / 48_000;
                    self.frames_out += (samples.len() / usize::from(self.channels)) as u64;
                    return Ok(Some(crate::audio_decode::DecodedAudio {
                        samples,
                        sample_rate: 48_000,
                        channels: self.channels,
                        pts_ms,
                    }));
                }
                Ok(false) => {
                    self.eos = true;
                    return Ok(None);
                }
                Err(e) => return Err(AudioDecodeError::Decode(e.to_string())),
            }
        }
    }
}
