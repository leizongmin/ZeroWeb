//! mp4（ISO-BMFF）容器 + H.264 解码（media-playback M3 / D-RFC-3 获批实施）。
//!
//! demux 面用 `symphonia` `isomp4` feature（纯 Rust）枚举 H.264 视频轨并逐包
//! 提取；位流面用 `openh264`（Cisco OpenH264 安全 Rust 绑定，feature
//! `decode-h264`——构建期源码编译）解码。mp4 的 H.264 轨为 **avcC 配置 +
//! 长度前缀 NALU**（非 Annex-B），喂解码器前做两步转换：
//!
//! 1. avcC（AVCDecoderConfigurationRecord）→ SPS/PPS NALU（Annex-B 前缀，
//!    仅首包前注入——openh264 无 in-band 参数集 API 时即此形态）；
//! 2. 长度前缀 NALU 序列 → Annex-B 起始码分隔。
//!
//! 与 [`crate::decode`] 的 VP9/AV1 路径共用 [`DecodedVideoFrame`] 输出面与
//! [`crate::decode::planes_to_rgba`] 色彩转换（H.264 YUV 为 8bit 4:2:0 BT.601
//! 主面——fixture 与 web 存量一致）。
//!
//! https://www.iso.org/standard/68982.html (ISO 14496-15: avcC / AVC NALU stream in MP4)
//! https://www.itu.int/rec/T-REC-H.264

use std::io::Cursor;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::video::well_known::CODEC_ID_H264;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::TimeBase;

use crate::decode::{ColorSpace, DecodeError, DecodedVideoFrame};

/// mp4 容器的 H.264 逐帧解码器（push/pull 与 [`crate::decode::VideoDecoder`]
/// 同形——codec 路由的消费面）。
pub struct Mp4H264Decoder {
    reader: Box<dyn FormatReader>,
    /// H.264 视频轨的 symphonia track id。
    video_track_id: u32,
    /// mp4 timescale（TimeBase：ticks/秒 的倒数——`numer=1, denom=timescale`）。
    time_base: TimeBase,
    /// avcC 的 SPS/PPS Annex-B 前缀（首包前注入一次）。
    parameter_set_prefix: Vec<u8>,
    /// openh264 解码器（B 帧重排由其输出面承担——display 顺序）。
    decoder: openh264::decoder::Decoder,
    /// pending 帧（解码器一次喂入可能产出 0/1 帧——B 帧前瞻；无立即输出时
    /// 缓存 pts，输出时补挂）。
    pending: Option<DecodedVideoFrame>,
    eof: bool,
    /// 已注入参数集（只注入一次）。
    ps_injected: bool,
    /// 容器声明的视频轨时长（毫秒）——TrackEntry duration × timebase。
    duration_ms: Option<u64>,
    /// 源字节留存（seek 重建 reader/decoder 所需——webm 面源字节由 registry 留存，
    /// mp4 面 reader 不可重置，自持一份）。
    source_bytes: std::sync::Arc<Vec<u8>>,
}

impl Mp4H264Decoder {
    /// 从 mp4 字节打开 H.264 视频解码器。
    ///
    /// 校验容器含 H.264（AVC）视频轨，否则 [`DecodeError::NoVideoTrack`]。
    pub fn open(data: &[u8]) -> Result<Self, DecodeError> {
        let src = Cursor::new(data.to_vec());
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("mp4");
        let probed = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
            .map_err(mp4_err)?;
        let mut video_track_id = 0u32;
        let mut time_base = None;
        let mut avcc: Option<Vec<u8>> = None;
        let mut duration_ms = None;
        for track in probed.tracks() {
            match &track.codec_params {
                Some(CodecParameters::Video(v)) if v.codec == CODEC_ID_H264 => {
                    video_track_id = track.id;
                    time_base = track.time_base;
                    // avcC = AVCDecoderConfigurationRecord（symphonia 已从 stsd/avcC
                    // 提取——extra_data 首项）。
                    if let Some(ed) = v.extra_data.first() {
                        avcc = Some(ed.data.to_vec());
                    }
                    // 容器时长：tkhd/mdhd duration（timebase tick）→ 毫秒。
                    if let (Some(tb), Some(dur)) = (track.time_base, track.duration) {
                        duration_ms = Some(
                            (dur.get() as u128 * 1_000 * u128::from(tb.numer.get()) / u128::from(tb.denom.get()))
                                as u64,
                        );
                    }
                }
                _ => {}
            }
        }
        if video_track_id == 0 {
            return Err(DecodeError::NoVideoTrack);
        }
        let time_base = time_base.ok_or_else(|| DecodeError::Container("mp4 video track missing time base".into()))?;
        let parameter_set_prefix = avcc.as_deref().map(avcc_record_to_annex_b).unwrap_or_default();
        Ok(Self {
            reader: probed,
            video_track_id,
            time_base,
            parameter_set_prefix,
            decoder: openh264::decoder::Decoder::new().map_err(h264_err)?,
            pending: None,
            eof: false,
            ps_injected: false,
            duration_ms,
            source_bytes: std::sync::Arc::new(data.to_vec()),
        })
    }

    /// 解码并返回下一帧（display 顺序）；流末返回 `Ok(None)`。
    pub fn next_frame(&mut self) -> Result<Option<DecodedVideoFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
        }
        if self.eof {
            return Ok(None);
        }
        loop {
            match self.reader.next_packet() {
                Ok(Some(packet)) => {
                    if packet.track_id != self.video_track_id {
                        continue;
                    }
                    // mp4 时间轴：packet.pts 为 timescale tick——1 tick =
                    // numer/denom 秒（symphonia TimeBase 语义），毫秒 =
                    // ticks × 1000 × numer / denom（numer=1, denom=timescale
                    // 时即 ticks × 1000 / timescale）。
                    let pts_ms = (packet.pts.get() as u128 * 1_000 * u128::from(self.time_base.numer.get())
                        / u128::from(self.time_base.denom.get())) as u64;
                    let mut annex_b = if !self.ps_injected && !self.parameter_set_prefix.is_empty() {
                        self.ps_injected = true;
                        self.parameter_set_prefix.clone()
                    } else {
                        Vec::new()
                    };
                    annex_b.extend(length_prefixed_to_annex_b(&packet.data));
                    // openh264 按整段 Annex-B 解码（SPS/PPS + 本包 NALU 序列）。
                    // B 帧前瞻：无输出（None）继续 demux 喂下一包。
                    match self.decoder.decode(&annex_b) {
                        Ok(Some(yuv)) => {
                            use openh264::formats::YUVSource;
                            let (w, h) = yuv.dimensions();
                            let frame = crate::decode::planes_to_rgba(
                                &[yuv.y().to_vec(), yuv.u().to_vec(), yuv.v().to_vec()],
                                &[yuv.strides().0, yuv.strides().1, yuv.strides().2],
                                w,
                                h,
                                8,
                                1,
                                1,
                                &ColorSpace::default(),
                                pts_ms,
                            );
                            return Ok(Some(frame));
                        }
                        Ok(None) => continue,
                        Err(e) => return Err(DecodeError::H264(e.to_string())),
                    }
                }
                Ok(None) => {
                    self.eof = true;
                    return Ok(None);
                }
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.eof = true;
                    return Ok(None);
                }
                Err(e) => return Err(DecodeError::Container(e.to_string())),
            }
        }
    }

    /// seek 到目标位置（毫秒，媒体时间轴）——spec「seek」precise-seek 前向回退
    /// 形态（与 [`crate::decode::VideoDecoder::seek_to_ms`] 的 ② 回退路径同构）：
    /// mp4 无 cue 索引面（stss/sync sample 索引随切片 3 评估）——从流首重建解码器
    /// 前向解码至 ≥ target 的首帧，写入 pending（下一次 next_frame 先弹出）。
    /// 语义契约：完成后的「下一次 [`Self::next_frame`]」返回 pts ≥ target 的首帧。
    /// https://html.spec.whatwg.org/multipage/media.html#seek
    pub fn seek_to_ms(&mut self, target_ms: u64) -> Result<(), DecodeError> {
        // 重建：reader 重 probe + 解码器重造（参考链作废）——open() 的轨枚举面
        // 不可复用（reader 已消费），但 mp4 box 结构支持随机访问（re-probe廉价）。
        let fresh = Self::open(&self.source_bytes)?;
        *self = fresh;
        self.pending = None;
        loop {
            match self.next_frame() {
                Ok(Some(frame)) => {
                    if frame.pts_ms >= target_ms {
                        self.pending = Some(frame);
                        return Ok(());
                    }
                }
                Ok(None) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// 容器声明的视频轨时长（毫秒）；未声明 `None`。
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    /// 帧退回（R3936 播放器背压契约——与 [`crate::decode::VideoDecoder::un_read`]
    /// 同语义）。
    pub fn un_read(&mut self, frame: DecodedVideoFrame) {
        debug_assert!(self.pending.is_none(), "un_read on occupied pending slot");
        self.pending = Some(frame);
    }

    /// 流末冲刷（openh264 内部前瞻缓冲的残余帧）。
    pub fn flush(&mut self) {
        let _ = self.decoder.flush_remaining();
    }
}

/// mp4 长度前缀 NALU 序列（4 字节大端长度 + NALU）→ Annex-B（0001 起始码）。
fn length_prefixed_to_annex_b(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 64);
    let mut i = 0usize;
    while i + 4 <= data.len() {
        let len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        i += 4;
        if len == 0 || i + len > data.len() {
            break;
        }
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&data[i..i + len]);
        i += len;
    }
    out
}

/// avcC（AVCDecoderConfigurationRecord）→ Annex-B（SPS/PPS NALU 序列）。
///
/// Record 布局（ISO 14496-15 §5.3.3.1）：
/// `[0]` configurationVersion、`[1]` profile、`[2]` compat、`[3]` level、
/// `[4]` 0xFF（6bit reserved + 2bit lengthSizeMinusOne）、`[5]` 0xE1（3bit
/// reserved + 5bit numOfSequenceParameterSets），随后逐个 2 字节长度前缀的
/// SPS；末字节为 numOfPictureParameterSets，随后 PPS。
fn avcc_record_to_annex_b(record: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if record.len() < 7 {
        return out;
    }
    let num_sps = (record[5] & 0x1f) as usize;
    let mut i = 6usize;
    for _ in 0..num_sps {
        if i + 2 > record.len() {
            return out;
        }
        let len = u16::from_be_bytes([record[i], record[i + 1]]) as usize;
        i += 2;
        if i + len > record.len() {
            return out;
        }
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&record[i..i + len]);
        i += len;
    }
    if i >= record.len() {
        return out;
    }
    let num_pps = (record[i] & 0x1f) as usize;
    i += 1;
    for _ in 0..num_pps {
        if i + 2 > record.len() {
            return out;
        }
        let len = u16::from_be_bytes([record[i], record[i + 1]]) as usize;
        i += 2;
        if i + len > record.len() {
            return out;
        }
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&record[i..i + len]);
        i += len;
    }
    out
}

fn h264_err(e: openh264::Error) -> DecodeError {
    DecodeError::H264(e.to_string())
}

fn mp4_err(e: symphonia::core::errors::Error) -> DecodeError {
    DecodeError::Container(e.to_string())
}
